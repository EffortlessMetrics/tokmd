//! Git diff file-stat collection and change-surface metrics.

use std::path::Path;

use anyhow::{Context, Result, bail};
use tokmd_types::cockpit::ChangeSurface;

use crate::FileStat;

/// Get file stats for changed files.
pub fn get_file_stats(
    repo_root: &Path,
    base: &str,
    head: &str,
    range_mode: tokmd_git::GitRangeMode,
) -> Result<Vec<FileStat>> {
    let range = range_mode.format(base, head);
    let output = tokmd_git::git_cmd()
        .arg("-C")
        .arg(repo_root)
        .args(["diff", "--numstat", &range])
        .output()
        .context("Failed to run git diff --numstat")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git diff --numstat failed: {}", stderr.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut stats = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() == 3 {
            let insertions = parts[0].parse().unwrap_or(0);
            let deletions = parts[1].parse().unwrap_or(0);
            let path = parts[2].to_string();
            stats.push(FileStat {
                path,
                insertions,
                deletions,
            });
        }
    }

    Ok(stats)
}

/// Compute change surface metrics.
pub(crate) fn compute_change_surface(
    repo_root: &Path,
    base: &str,
    head: &str,
    file_stats: &[FileStat],
    range_mode: tokmd_git::GitRangeMode,
) -> Result<ChangeSurface> {
    let range = range_mode.format(base, head);
    let output = tokmd_git::git_cmd()
        .arg("-C")
        .arg(repo_root)
        .args(["rev-list", "--count", &range])
        .output()
        .context("Failed to run git rev-list --count")?;

    let commits = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .unwrap_or(0);

    let files_changed = file_stats.len();
    let insertions = file_stats.iter().map(|s| s.insertions).sum();
    let deletions = file_stats.iter().map(|s| s.deletions).sum();
    let net_lines = (insertions as i64) - (deletions as i64);

    let churn_velocity = if commits > 0 {
        (insertions + deletions) as f64 / commits as f64
    } else {
        0.0
    };

    // Simple change concentration: what % of changes are in top 20% of files.
    let mut changes: Vec<usize> = file_stats
        .iter()
        .map(|s| s.insertions + s.deletions)
        .collect();
    changes.sort_unstable_by(|a, b| b.cmp(a));

    let top_count = (files_changed as f64 * 0.2).ceil() as usize;
    let total_changes: usize = changes.iter().sum();
    let top_changes: usize = changes.iter().take(top_count).sum();

    let change_concentration = if total_changes > 0 {
        top_changes as f64 / total_changes as f64
    } else {
        0.0
    };

    Ok(ChangeSurface {
        commits,
        files_changed,
        insertions,
        deletions,
        net_lines,
        churn_velocity,
        change_concentration,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_stat(path: &str, insertions: usize, deletions: usize) -> FileStat {
        FileStat {
            path: path.to_string(),
            insertions,
            deletions,
        }
    }

    /// Initialise a temp git repository with two commits so the change surface
    /// helpers have a real range to walk. Returns the temp dir handle (which
    /// owns the lifetime of the repository).
    fn init_repo_with_two_commits(file_changes: &[(&str, &str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let run_git = |args: &[&str]| {
            let status = tokmd_git::git_cmd()
                .args(args)
                .current_dir(dir.path())
                .status()
                .unwrap();
            assert!(status.success(), "git {:?} failed", args);
        };

        run_git(&["init", "-b", "main"]);
        run_git(&["config", "user.email", "tokmd@example.com"]);
        run_git(&["config", "user.name", "tokmd"]);
        run_git(&["config", "commit.gpgsign", "false"]);

        // Seed each file with its "base" content and commit.
        for (path, base_content, _) in file_changes {
            let full = dir.path().join(path);
            if let Some(parent) = full.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(&full, base_content).unwrap();
        }
        run_git(&["add", "."]);
        run_git(&["commit", "-m", "base"]);

        // Overwrite with "head" content and commit again. The commit must
        // succeed even if no files actually changed (some tests pass identical
        // base/head content to exercise the change-surface bookkeeping
        // independently of the git diff), so use --allow-empty.
        for (path, _, head_content) in file_changes {
            std::fs::write(dir.path().join(path), head_content).unwrap();
        }
        run_git(&["add", "."]);
        run_git(&["commit", "--allow-empty", "-m", "head"]);

        dir
    }

    #[test]
    fn compute_change_surface_empty_stats() {
        let dir = init_repo_with_two_commits(&[("src/lib.rs", "fn a() {}\n", "fn a() {}\n")]);
        let surface = compute_change_surface(
            dir.path(),
            "HEAD~1",
            "HEAD",
            &[],
            tokmd_git::GitRangeMode::TwoDot,
        )
        .unwrap();

        assert_eq!(surface.files_changed, 0);
        assert_eq!(surface.insertions, 0);
        assert_eq!(surface.deletions, 0);
        assert_eq!(surface.net_lines, 0);
        // 0 changes / N commits is still 0.0.
        assert_eq!(surface.churn_velocity, 0.0);
        // No files -> no concentration to report.
        assert_eq!(surface.change_concentration, 0.0);
    }

    #[test]
    fn compute_change_surface_single_file_sums_and_averages() {
        let dir = init_repo_with_two_commits(&[("src/lib.rs", "fn a() {}\n", "fn b() {}\n")]);
        let stats = vec![make_stat("src/lib.rs", 10, 4)];

        let surface = compute_change_surface(
            dir.path(),
            "HEAD~1",
            "HEAD",
            &stats,
            tokmd_git::GitRangeMode::TwoDot,
        )
        .unwrap();

        assert_eq!(surface.commits, 1, "HEAD~1..HEAD spans one commit");
        assert_eq!(surface.files_changed, 1);
        assert_eq!(surface.insertions, 10);
        assert_eq!(surface.deletions, 4);
        assert_eq!(surface.net_lines, 6);
        // 1 file: churn = (10 + 4) / 1 commit = 14.0; all changes are
        // concentrated in the single file → 100%.
        assert_eq!(surface.churn_velocity, 14.0);
        assert_eq!(surface.change_concentration, 1.0);
    }

    #[test]
    fn compute_change_surface_negative_net_lines() {
        let dir = init_repo_with_two_commits(&[("src/lib.rs", "fn a() {}\n", "fn b() {}\n")]);
        let stats = vec![make_stat("src/lib.rs", 3, 20)];

        let surface = compute_change_surface(
            dir.path(),
            "HEAD~1",
            "HEAD",
            &stats,
            tokmd_git::GitRangeMode::TwoDot,
        )
        .unwrap();

        assert_eq!(surface.insertions, 3);
        assert_eq!(surface.deletions, 20);
        assert_eq!(surface.net_lines, -17);
    }

    #[test]
    fn compute_change_surface_concentration_in_top_files() {
        // 5 files: top 20% = ceil(5 * 0.2) = 1 file. The largest file
        // contributes 50/(50+10+5+3+2) = 50/70 of the changes.
        let dir = init_repo_with_two_commits(&[("src/a.rs", "x\n", "x\n")]);
        let stats = vec![
            make_stat("src/a.rs", 30, 20), // 50
            make_stat("src/b.rs", 5, 5),   // 10
            make_stat("src/c.rs", 4, 1),   // 5
            make_stat("src/d.rs", 2, 1),   // 3
            make_stat("src/e.rs", 1, 1),   // 2
        ];

        let surface = compute_change_surface(
            dir.path(),
            "HEAD~1",
            "HEAD",
            &stats,
            tokmd_git::GitRangeMode::TwoDot,
        )
        .unwrap();

        assert_eq!(surface.files_changed, 5);
        let expected = 50.0_f64 / 70.0;
        assert!(
            (surface.change_concentration - expected).abs() < 1e-9,
            "concentration {} should be ~{}",
            surface.change_concentration,
            expected
        );
    }

    #[test]
    fn compute_change_surface_handles_no_change_files() {
        // file_stats reports two zero-line files: total_changes = 0, so
        // change_concentration short-circuits to 0.0 without dividing.
        let dir = init_repo_with_two_commits(&[("src/lib.rs", "x\n", "x\n")]);
        let stats = vec![make_stat("a.rs", 0, 0), make_stat("b.rs", 0, 0)];

        let surface = compute_change_surface(
            dir.path(),
            "HEAD~1",
            "HEAD",
            &stats,
            tokmd_git::GitRangeMode::TwoDot,
        )
        .unwrap();

        assert_eq!(surface.files_changed, 2);
        assert_eq!(surface.insertions, 0);
        assert_eq!(surface.deletions, 0);
        assert_eq!(surface.change_concentration, 0.0);
        assert_eq!(surface.churn_velocity, 0.0);
    }

    #[test]
    fn compute_change_surface_zero_commits_short_circuits_velocity() {
        // HEAD..HEAD has zero commits, so the `commits > 0` guard must keep
        // churn_velocity at 0.0 instead of triggering a division by zero.
        let dir = init_repo_with_two_commits(&[("src/lib.rs", "x\n", "y\n")]);
        let stats = vec![make_stat("src/lib.rs", 10, 5)];

        let surface = compute_change_surface(
            dir.path(),
            "HEAD",
            "HEAD",
            &stats,
            tokmd_git::GitRangeMode::TwoDot,
        )
        .unwrap();

        assert_eq!(surface.commits, 0);
        assert_eq!(surface.insertions, 10);
        assert_eq!(surface.deletions, 5);
        assert_eq!(surface.churn_velocity, 0.0);
    }

    #[test]
    fn get_file_stats_reports_per_file_numstat() {
        let dir = init_repo_with_two_commits(&[
            ("a.txt", "one\n", "one\ntwo\nthree\n"),
            ("b.txt", "alpha\nbeta\ngamma\n", "alpha\n"),
        ]);

        let stats = get_file_stats(
            dir.path(),
            "HEAD~1",
            "HEAD",
            tokmd_git::GitRangeMode::TwoDot,
        )
        .unwrap();

        let a = stats
            .iter()
            .find(|s| s.path == "a.txt")
            .expect("a.txt should appear in numstat");
        let b = stats
            .iter()
            .find(|s| s.path == "b.txt")
            .expect("b.txt should appear in numstat");

        // a.txt: added two lines, deleted zero.
        assert_eq!(a.insertions, 2);
        assert_eq!(a.deletions, 0);
        // b.txt: kept one of three lines.
        assert_eq!(b.insertions, 0);
        assert_eq!(b.deletions, 2);
    }

    #[test]
    fn get_file_stats_invalid_range_errors() {
        let dir = tempfile::tempdir().unwrap();
        // No git repo here: git diff must fail and bubble up an Err.
        let err = get_file_stats(
            dir.path(),
            "main",
            "feature",
            tokmd_git::GitRangeMode::TwoDot,
        )
        .unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.to_lowercase().contains("git"),
            "expected error to mention git, got: {msg}"
        );
    }

    #[test]
    fn get_file_stats_empty_diff_returns_empty_vec() {
        // Same commit on both sides → no numstat output → empty Vec.
        let dir = init_repo_with_two_commits(&[("src/lib.rs", "x\n", "y\n")]);
        let stats =
            get_file_stats(dir.path(), "HEAD", "HEAD", tokmd_git::GitRangeMode::TwoDot).unwrap();
        assert!(stats.is_empty());
    }

    #[test]
    fn change_surface_top_count_ceiling_picks_at_least_one() {
        // 3 files: ceil(3 * 0.2) = 1, the largest contributes 100/(100+1+1).
        let dir = init_repo_with_two_commits(&[("src/lib.rs", "x\n", "y\n")]);
        let stats = vec![
            make_stat("big.rs", 60, 40),
            make_stat("tiny_a.rs", 1, 0),
            make_stat("tiny_b.rs", 0, 1),
        ];

        let surface = compute_change_surface(
            dir.path(),
            "HEAD~1",
            "HEAD",
            &stats,
            tokmd_git::GitRangeMode::TwoDot,
        )
        .unwrap();

        let expected = 100.0_f64 / 102.0;
        assert!(
            (surface.change_concentration - expected).abs() < 1e-9,
            "concentration {} should be ~{}",
            surface.change_concentration,
            expected
        );
    }
}
