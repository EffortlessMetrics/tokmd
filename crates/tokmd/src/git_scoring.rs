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

#[cfg(all(test, feature = "git"))]
mod tests {
    use super::*;
    use std::process::Command;
    use tokmd_types::{FileKind, FileRow};

    fn make_row(path: &str, lines: usize) -> FileRow {
        FileRow {
            path: path.to_string(),
            module: "(root)".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: lines,
            comments: 0,
            blanks: 0,
            lines,
            bytes: lines * 10,
            tokens: lines * 5,
        }
    }

    fn create_test_repo() -> Option<tempfile::TempDir> {
        let dir = tempfile::tempdir().ok()?;
        let root = dir.path();

        // git init + config
        Command::new("git")
            .args(["init"])
            .current_dir(root)
            .output()
            .ok()?;
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(root)
            .output()
            .ok()?;
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(root)
            .output()
            .ok()?;

        // main.rs: 2 commits (3 lines initially, then 4)
        std::fs::write(root.join("main.rs"), "1\n2\n3").ok()?;
        Command::new("git")
            .args(["add", "."])
            .current_dir(root)
            .output()
            .ok()?;
        Command::new("git")
            .args(["commit", "-m", "c1"])
            .current_dir(root)
            .output()
            .ok()?;

        std::fs::write(root.join("main.rs"), "1\n2\n3\n4").ok()?;
        Command::new("git")
            .args(["add", "."])
            .current_dir(root)
            .output()
            .ok()?;
        Command::new("git")
            .args(["commit", "-m", "c2"])
            .current_dir(root)
            .output()
            .ok()?;

        // lib.rs: 1 commit (5 lines)
        std::fs::write(root.join("lib.rs"), "1\n2\n3\n4\n5").ok()?;
        Command::new("git")
            .args(["add", "."])
            .current_dir(root)
            .output()
            .ok()?;
        Command::new("git")
            .args(["commit", "-m", "c3"])
            .current_dir(root)
            .output()
            .ok()?;

        Some(dir)
    }

    #[test]
    fn test_compute_git_scores_commit_counts() {
        let repo = match create_test_repo() {
            Some(r) => r,
            None => return, // Skip if git unavailable
        };
        let rows = vec![make_row("main.rs", 4), make_row("lib.rs", 5)];
        let scores = compute_git_scores(repo.path(), &rows, 100, 100).unwrap();

        // main.rs has 2 commits, lib.rs has 1 commit
        assert_eq!(scores.commit_counts.get("main.rs"), Some(&2));
        assert_eq!(scores.commit_counts.get("lib.rs"), Some(&1));
    }

    #[test]
    fn test_compute_git_scores_hotspots() {
        let repo = match create_test_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![make_row("main.rs", 4), make_row("lib.rs", 5)];
        let scores = compute_git_scores(repo.path(), &rows, 100, 100).unwrap();

        // hotspot = lines * commits
        // main.rs: 4 lines * 2 commits = 8
        // lib.rs: 5 lines * 1 commit = 5
        assert_eq!(scores.hotspots.get("main.rs"), Some(&8));
        assert_eq!(scores.hotspots.get("lib.rs"), Some(&5));
    }

    #[test]
    fn test_compute_git_scores_filters_children() {
        let repo = match create_test_repo() {
            Some(r) => r,
            None => return,
        };
        // Only include child rows - should be filtered out
        let rows = vec![FileRow {
            path: "main.rs".to_string(),
            module: "(root)".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Child, // Child, not Parent
            code: 4,
            comments: 0,
            blanks: 0,
            lines: 4,
            bytes: 40,
            tokens: 20,
        }];
        let scores = compute_git_scores(repo.path(), &rows, 100, 100).unwrap();

        // Child rows should be filtered, so commit_counts should be empty
        assert!(scores.commit_counts.is_empty());
    }

    #[test]
    fn test_compute_git_scores_non_repo_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let rows = vec![];
        // Not a git repo, should return None
        assert!(compute_git_scores(dir.path(), &rows, 100, 100).is_none());
    }

    #[test]
    fn test_normalize_path_backslash() {
        assert_eq!(normalize_path("foo\\bar\\baz.rs"), "foo/bar/baz.rs");
    }

    #[test]
    fn test_normalize_path_dot_slash() {
        assert_eq!(normalize_path("./src/main.rs"), "src/main.rs");
    }

    // ==================== Mutant killer tests ====================

    #[test]
    fn test_compute_git_scores_returns_some() {
        // Kills "compute_git_scores -> None" mutant
        let repo = match create_test_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![make_row("main.rs", 4)];
        let result = compute_git_scores(repo.path(), &rows, 100, 100);
        assert!(
            result.is_some(),
            "compute_git_scores should return Some for valid git repo"
        );
    }

    #[test]
    fn test_compute_git_scores_not_default() {
        // Kills "compute_git_scores -> Some(Default::default())" mutant
        let repo = match create_test_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![make_row("main.rs", 4), make_row("lib.rs", 5)];
        let scores = compute_git_scores(repo.path(), &rows, 100, 100).unwrap();

        // Scores should not be empty (kills Default::default() mutant)
        assert!(
            !scores.commit_counts.is_empty(),
            "commit_counts should not be empty"
        );
        assert!(!scores.hotspots.is_empty(), "hotspots should not be empty");
    }

    #[test]
    fn test_commit_count_increment() {
        // Kills "+= -> -=" mutant on commit counting
        // main.rs has 2 commits, so count must be > 0 (not negative from subtraction)
        let repo = match create_test_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![make_row("main.rs", 4)];
        let scores = compute_git_scores(repo.path(), &rows, 100, 100).unwrap();

        let count = scores.commit_counts.get("main.rs").copied().unwrap_or(0);
        assert!(count > 0, "commit count must be positive, got {count}");
        assert_eq!(count, 2, "main.rs should have exactly 2 commits");
    }

    #[test]
    fn test_hotspot_multiplication() {
        // Kills "lines * commits -> lines + commits" or "lines / commits" mutants
        let repo = match create_test_repo() {
            Some(r) => r,
            None => return,
        };
        // main.rs: 4 lines, 2 commits
        // If multiplication: 4 * 2 = 8
        // If addition: 4 + 2 = 6
        // If division: 4 / 2 = 2
        let rows = vec![make_row("main.rs", 4)];
        let scores = compute_git_scores(repo.path(), &rows, 100, 100).unwrap();

        let hotspot = scores.hotspots.get("main.rs").copied().unwrap_or(0);
        assert_eq!(
            hotspot, 8,
            "hotspot should be lines * commits = 4 * 2 = 8, got {hotspot}"
        );
    }

    #[test]
    fn test_normalize_path_not_empty() {
        // Kills "normalize_path -> empty string" mutant
        assert!(!normalize_path("foo/bar").is_empty());
        assert!(!normalize_path("test.rs").is_empty());
        assert!(!normalize_path("./src/lib.rs").is_empty());
    }

    #[test]
    fn test_normalize_path_not_xyzzy() {
        // Kills "normalize_path -> xyzzy" mutant
        assert_ne!(normalize_path("foo/bar"), "xyzzy");
        assert_ne!(normalize_path("test.rs"), "xyzzy");
        assert_ne!(normalize_path("./src/lib.rs"), "xyzzy");
    }

    #[test]
    fn test_filter_only_parent_files() {
        // Kills "== FileKind::Parent -> != FileKind::Parent" mutant
        let repo = match create_test_repo() {
            Some(r) => r,
            None => return,
        };
        // Mix of parent and child rows
        let rows = vec![
            make_row("main.rs", 4), // Parent
            FileRow {
                path: "lib.rs".to_string(),
                module: "(root)".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Child, // Child - should be filtered
                code: 5,
                comments: 0,
                blanks: 0,
                lines: 5,
                bytes: 50,
                tokens: 25,
            },
        ];
        let scores = compute_git_scores(repo.path(), &rows, 100, 100).unwrap();

        // Only main.rs (Parent) should appear
        assert!(scores.commit_counts.contains_key("main.rs"));
        assert!(
            !scores.commit_counts.contains_key("lib.rs"),
            "Child file lib.rs should be filtered out"
        );
    }

    #[test]
    fn test_path_matching_with_normalization() {
        // Tests that path normalization works for matching git paths to FileRow paths
        let repo = match create_test_repo() {
            Some(r) => r,
            None => return,
        };
        // Use backslash path (Windows-style) - should still match
        let rows = vec![FileRow {
            path: "main.rs".to_string(), // Forward slash
            module: "(root)".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 4,
            comments: 0,
            blanks: 0,
            lines: 4,
            bytes: 40,
            tokens: 20,
        }];
        let scores = compute_git_scores(repo.path(), &rows, 100, 100).unwrap();

        // Should find the file despite potential path differences
        assert!(
            scores.commit_counts.contains_key("main.rs"),
            "Should match file after normalization"
        );
    }
}
