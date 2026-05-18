//! # tokmd-git
//!
//! **Tier 2 (Utilities)**
//!
//! Streaming git log adapter for tokmd analysis. Collects commit history
//! without loading the entire history into memory.
//!
//! ## What belongs here
//! * Git history collection
//! * Commit parsing (timestamp, author, affected files)
//! * Streaming interface
//!
//! ## What does NOT belong here
//! * Analysis computation (use tokmd-analysis)
//! * Git history modification
//! * Complex git operations (use git2 crate directly if needed)

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::{Context, Result};
pub use tokmd_types::CommitIntentKind;

mod command;
mod intent;
mod refs;

pub use command::git_cmd;
pub use intent::classify_intent;
pub use refs::{resolve_base_ref, rev_exists};

#[derive(Debug, Clone)]
pub struct GitCommit {
    pub timestamp: i64,
    pub author: String,
    pub hash: Option<String>,
    pub subject: String,
    pub files: Vec<String>,
}

/// Git range syntax for comparing commits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GitRangeMode {
    /// Two-dot syntax: `A..B` - commits in B but not A.
    #[default]
    TwoDot,
    /// Three-dot syntax: `A...B` - symmetric difference from merge-base.
    ThreeDot,
}

impl GitRangeMode {
    /// Format the range string for git commands.
    pub fn format(&self, base: &str, head: &str) -> String {
        match self {
            GitRangeMode::TwoDot => format!("{}..{}", base, head),
            GitRangeMode::ThreeDot => format!("{}...{}", base, head),
        }
    }
}

pub fn git_available() -> bool {
    git_cmd()
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn repo_root(path: &Path) -> Option<PathBuf> {
    let output = git_cmd()
        .arg("-C")
        .arg(path)
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if root.is_empty() {
        None
    } else {
        Some(PathBuf::from(root))
    }
}

pub fn collect_history(
    repo_root: &Path,
    max_commits: Option<usize>,
    max_commit_files: Option<usize>,
) -> Result<Vec<GitCommit>> {
    let mut child = git_cmd()
        .arg("-C")
        .arg(repo_root)
        .arg("log")
        .arg("--name-only")
        .arg("--pretty=format:%ct|%ae|%H|%s")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to spawn git log")?;

    let stdout = child.stdout.take().context("Missing git log stdout")?;
    let reader = BufReader::new(stdout);

    let mut commits: Vec<GitCommit> = Vec::new();
    let mut current: Option<GitCommit> = None;

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            if let Some(commit) = current.take() {
                commits.push(commit);
                if max_commits.is_some_and(|limit| commits.len() >= limit) {
                    break;
                }
            }
            continue;
        }

        if current.is_none() {
            let mut parts = line.splitn(4, '|');
            let ts = parts.next().unwrap_or("0").parse::<i64>().unwrap_or(0);
            let author = parts.next().unwrap_or("").to_string();
            let hash_str = parts.next().unwrap_or("").to_string();
            let subject = parts.next().unwrap_or("").to_string();
            let hash = if hash_str.is_empty() {
                None
            } else {
                Some(hash_str)
            };
            current = Some(GitCommit {
                timestamp: ts,
                author,
                hash,
                subject,
                files: Vec::new(),
            });
            continue;
        }

        if let Some(commit) = current.as_mut()
            && max_commit_files
                .map(|limit| commit.files.len() < limit)
                .unwrap_or(true)
        {
            commit.files.push(line.trim().to_string());
        }
    }

    if let Some(commit) = current.take() {
        commits.push(commit);
    }

    let status = child.wait()?;
    if !status.success() {
        return Err(anyhow::anyhow!("git log failed"));
    }

    Ok(commits)
}

/// Get the set of added line numbers per file between two refs.
pub fn get_added_lines(
    repo_root: &Path,
    base: &str,
    head: &str,
    range_mode: GitRangeMode,
) -> Result<std::collections::BTreeMap<PathBuf, std::collections::BTreeSet<usize>>> {
    let range = range_mode.format(base, head);
    let output = git_cmd()
        .arg("-C")
        .arg(repo_root)
        .args(["diff", "--unified=0", &range])
        .output()
        .context("Failed to run git diff")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("git diff failed: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut result: std::collections::BTreeMap<PathBuf, std::collections::BTreeSet<usize>> =
        std::collections::BTreeMap::new();
    let mut current_file: Option<PathBuf> = None;

    for line in stdout.lines() {
        if let Some(file_path) = line.strip_prefix("+++ b/") {
            current_file = Some(PathBuf::from(file_path));
            continue;
        }

        if line.starts_with("@@") {
            let Some(file) = current_file.as_ref() else {
                continue;
            };

            // Hunk header: @@ -a,b +c,d @@
            // We care about +c,d
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
                continue;
            }

            let new_range = parts[2]; // +c,d
            let range_str = new_range.strip_prefix('+').unwrap_or(new_range);
            let range_parts: Vec<&str> = range_str.split(',').collect();

            let start: usize = range_parts[0].parse().unwrap_or(0);
            let count: usize = if range_parts.len() > 1 {
                range_parts[1].parse().unwrap_or(1)
            } else {
                1
            };

            if count > 0 && start > 0 {
                let set = result.entry(file.clone()).or_default();
                for i in 0..count {
                    set.insert(start + i);
                }
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};
    use std::process::Command;

    fn test_git(dir: &Path) -> Command {
        let mut cmd = git_cmd();
        cmd.arg("-C").arg(dir);
        cmd
    }

    fn run_git(dir: &Path, args: &[&str]) {
        let output = test_git(dir).args(args).output().unwrap();
        assert!(
            output.status.success(),
            "git {:?} failed\nstdout: {}\nstderr: {}",
            args,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn init_repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        run_git(dir.path(), &["init", "-b", "main"]);
        run_git(dir.path(), &["config", "user.email", "test@test.com"]);
        run_git(dir.path(), &["config", "user.name", "Test"]);
        dir
    }

    fn commit_all(dir: &Path, message: &str) {
        run_git(dir, &["add", "."]);
        run_git(dir, &["commit", "-m", message]);
    }

    #[test]
    fn git_range_two_dot_format() {
        assert_eq!(GitRangeMode::TwoDot.format("main", "HEAD"), "main..HEAD");
    }

    #[test]
    fn git_range_three_dot_format() {
        assert_eq!(GitRangeMode::ThreeDot.format("main", "HEAD"), "main...HEAD");
    }

    #[test]
    fn git_range_default_is_two_dot() {
        assert_eq!(GitRangeMode::default(), GitRangeMode::TwoDot);
    }

    #[test]
    fn collect_history_preserves_commit_metadata_and_limits_files() {
        if !git_available() {
            return;
        }
        let dir = init_repo();

        std::fs::write(dir.path().join("alpha.txt"), "alpha\n").unwrap();
        std::fs::write(dir.path().join("beta.txt"), "beta\n").unwrap();
        commit_all(dir.path(), "feat: add fixtures");

        let commits = collect_history(dir.path(), None, Some(1)).unwrap();

        assert_eq!(commits.len(), 1);
        let commit = &commits[0];
        assert_eq!(commit.author, "test@test.com");
        assert_eq!(commit.subject, "feat: add fixtures");
        assert!(commit.hash.as_deref().is_some_and(|hash| hash.len() == 40));
        assert_eq!(commit.files.len(), 1);
        assert!(["alpha.txt", "beta.txt"].contains(&commit.files[0].as_str()));
    }

    #[test]
    fn collect_history_respects_commit_and_file_limits() {
        if !git_available() {
            return;
        }
        let dir = init_repo();

        std::fs::write(dir.path().join("first.txt"), "first\n").unwrap();
        commit_all(dir.path(), "chore: first");
        std::fs::write(dir.path().join("second.txt"), "second\n").unwrap();
        std::fs::write(dir.path().join("third.txt"), "third\n").unwrap();
        commit_all(dir.path(), "fix: second");

        let commits = collect_history(dir.path(), Some(1), Some(0)).unwrap();

        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].subject, "fix: second");
        assert!(commits[0].files.is_empty());
    }

    #[test]
    fn get_added_lines_reports_new_line_numbers_per_file() {
        if !git_available() {
            return;
        }
        let dir = init_repo();

        std::fs::create_dir(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src/lib.rs"), "fn a() {}\nfn d() {}\n").unwrap();
        commit_all(dir.path(), "base");
        run_git(dir.path(), &["tag", "base"]);

        std::fs::write(
            dir.path().join("src/lib.rs"),
            "fn a() {}\nfn b() {}\nfn c() {}\nfn d() {}\n",
        )
        .unwrap();
        commit_all(dir.path(), "add middle functions");

        let added = get_added_lines(dir.path(), "base", "HEAD", GitRangeMode::TwoDot).unwrap();

        let mut expected = BTreeMap::new();
        expected.insert(
            PathBuf::from("src/lib.rs"),
            BTreeSet::from([2_usize, 3_usize]),
        );
        assert_eq!(added, expected);
    }
}
