//! Lightweight git scoring for context ranking.

use std::collections::BTreeMap;
#[cfg(feature = "git")]
use std::path::Path;

#[cfg(feature = "git")]
use tokmd_types::{FileKind, FileRow};

/// Git-derived scores for file ranking.
pub struct GitScores {
    /// Per-file hotspot scores: path → (lines × commits)
    pub hotspots: BTreeMap<String, usize>,
    /// Per-file commit counts: path → commits
    pub commit_counts: BTreeMap<String, usize>,
}

#[cfg(feature = "git")]
pub fn compute_git_scores(
    root: &Path,
    rows: &[FileRow],
    max_commits: usize,
    max_commit_files: usize,
) -> Option<GitScores> {
    let repo_root = tokmd_git::repo_root(root)?;
    let commits =
        tokmd_git::collect_history(&repo_root, Some(max_commits), Some(max_commit_files)).ok()?;

    // Build file → lines map (only parent files)
    let file_lines: BTreeMap<String, usize> = rows
        .iter()
        .filter(|r| r.kind == FileKind::Parent)
        .map(|r| (normalize_path(&r.path), r.lines))
        .collect();

    // Count commits per file
    let mut commit_counts: BTreeMap<String, usize> = BTreeMap::new();
    for commit in &commits {
        for file in &commit.files {
            let key = normalize_path(file);
            if file_lines.contains_key(&key) {
                *commit_counts.entry(key).or_insert(0) += 1;
            }
        }
    }

    // Compute hotspot scores: lines × commits
    let hotspots: BTreeMap<String, usize> = commit_counts
        .iter()
        .filter_map(|(path, commits)| {
            let lines = file_lines.get(path)?;
            Some((path.clone(), lines * commits))
        })
        .collect();

    Some(GitScores {
        hotspots,
        commit_counts,
    })
}

#[cfg(not(feature = "git"))]
pub fn compute_git_scores(
    _root: &std::path::Path,
    _rows: &[tokmd_types::FileRow],
    _max_commits: usize,
    _max_commit_files: usize,
) -> Option<GitScores> {
    None
}

/// Normalize path for consistent matching between FileRow and git paths.
/// Converts backslashes to forward slashes and strips leading `./` prefix.
#[cfg(feature = "git")]
fn normalize_path(path: &str) -> String {
    let out = path.replace('\\', "/");
    out.strip_prefix("./").unwrap_or(&out).to_string()
}
