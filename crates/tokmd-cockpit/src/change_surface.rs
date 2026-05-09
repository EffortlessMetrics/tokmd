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
