//! Deep tests for tokmd-context-git (w67).
//!
//! Covers: GitScores construction, BTreeMap invariants, scoring logic,
//! feature-gated git integration, edge cases (empty repos, missing files).

use std::collections::BTreeMap;
use tokmd_context_git::GitScores;

// ===========================================================================
// 1. GitScores struct construction
// ===========================================================================

#[test]
fn empty_git_scores_both_maps_empty() {
    let scores = GitScores {
        hotspots: BTreeMap::new(),
        commit_counts: BTreeMap::new(),
    };
    assert!(scores.hotspots.is_empty());
    assert!(scores.commit_counts.is_empty());
}

#[test]
fn git_scores_with_single_entry() {
    let mut hotspots = BTreeMap::new();
    hotspots.insert("src/main.rs".to_string(), 42);
    let mut commit_counts = BTreeMap::new();
    commit_counts.insert("src/main.rs".to_string(), 7);

    let scores = GitScores {
        hotspots,
        commit_counts,
    };
    assert_eq!(scores.hotspots.len(), 1);
    assert_eq!(scores.commit_counts.len(), 1);
    assert_eq!(scores.hotspots["src/main.rs"], 42);
    assert_eq!(scores.commit_counts["src/main.rs"], 7);
}

#[test]
fn git_scores_multiple_entries() {
    let mut hotspots = BTreeMap::new();
    hotspots.insert("a.rs".to_string(), 10);
    hotspots.insert("b.rs".to_string(), 20);
    hotspots.insert("c.rs".to_string(), 30);

    let scores = GitScores {
        hotspots,
        commit_counts: BTreeMap::new(),
    };
    assert_eq!(scores.hotspots.len(), 3);
}

// ===========================================================================
// 2. BTreeMap ordering invariants
// ===========================================================================

#[test]
fn hotspots_keys_sorted_alphabetically() {
    let mut hotspots = BTreeMap::new();
    hotspots.insert("z/file.rs".to_string(), 1);
    hotspots.insert("a/file.rs".to_string(), 2);
    hotspots.insert("m/file.rs".to_string(), 3);

    let scores = GitScores {
        hotspots,
        commit_counts: BTreeMap::new(),
    };
    let keys: Vec<&String> = scores.hotspots.keys().collect();
    assert_eq!(keys, vec!["a/file.rs", "m/file.rs", "z/file.rs"]);
}

#[test]
fn commit_counts_keys_sorted() {
    let mut commit_counts = BTreeMap::new();
    commit_counts.insert("util.rs".to_string(), 1);
    commit_counts.insert("api.rs".to_string(), 5);
    commit_counts.insert("main.rs".to_string(), 3);

    let scores = GitScores {
        hotspots: BTreeMap::new(),
        commit_counts,
    };
    let keys: Vec<&String> = scores.commit_counts.keys().collect();
    assert_eq!(keys, vec!["api.rs", "main.rs", "util.rs"]);
}

// ===========================================================================
// 3. Hotspot scoring logic (simulated)
// ===========================================================================

/// Simulate hotspot = lines × commits.
fn simulate_hotspot(lines: usize, commits: usize) -> usize {
    lines * commits
}

#[test]
fn hotspot_formula_lines_times_commits() {
    assert_eq!(simulate_hotspot(100, 5), 500);
    assert_eq!(simulate_hotspot(0, 10), 0);
    assert_eq!(simulate_hotspot(50, 0), 0);
}

#[test]
fn hotspot_ordering_highest_first() {
    let scores = vec![
        ("lib.rs", simulate_hotspot(200, 3)),   // 600
        ("main.rs", simulate_hotspot(100, 10)),  // 1000
        ("util.rs", simulate_hotspot(50, 2)),    // 100
    ];
    let mut sorted = scores.clone();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    assert_eq!(sorted[0].0, "main.rs");
    assert_eq!(sorted[1].0, "lib.rs");
    assert_eq!(sorted[2].0, "util.rs");
}

#[test]
fn hotspot_keys_subset_of_commit_counts() {
    let mut commit_counts = BTreeMap::new();
    commit_counts.insert("a.rs".to_string(), 3);
    commit_counts.insert("b.rs".to_string(), 2);
    commit_counts.insert("c.rs".to_string(), 1);

    // Only a.rs and b.rs have known line counts
    let file_lines: BTreeMap<String, usize> =
        [("a.rs".to_string(), 100), ("b.rs".to_string(), 50)]
            .into_iter()
            .collect();

    let hotspots: BTreeMap<String, usize> = commit_counts
        .iter()
        .filter_map(|(path, commits)| {
            let lines = file_lines.get(path)?;
            Some((path.clone(), lines * commits))
        })
        .collect();

    // c.rs has commits but no lines → not in hotspots
    assert!(hotspots.contains_key("a.rs"));
    assert!(hotspots.contains_key("b.rs"));
    assert!(!hotspots.contains_key("c.rs"));
    for key in hotspots.keys() {
        assert!(commit_counts.contains_key(key));
    }
}

// ===========================================================================
// 4. Edge cases
// ===========================================================================

#[test]
fn empty_file_lines_means_empty_hotspots() {
    let commit_counts: BTreeMap<String, usize> =
        [("x.rs".to_string(), 5)].into_iter().collect();
    let file_lines: BTreeMap<String, usize> = BTreeMap::new();

    let hotspots: BTreeMap<String, usize> = commit_counts
        .iter()
        .filter_map(|(path, commits)| {
            let lines = file_lines.get(path)?;
            Some((path.clone(), lines * commits))
        })
        .collect();

    assert!(hotspots.is_empty());
}

#[test]
fn zero_commits_produces_zero_hotspot() {
    assert_eq!(simulate_hotspot(1000, 0), 0);
}

#[test]
fn zero_lines_produces_zero_hotspot() {
    assert_eq!(simulate_hotspot(0, 100), 0);
}

#[test]
fn large_values_no_overflow_panic() {
    // Ensure reasonable large values don't panic
    let _ = simulate_hotspot(100_000, 10_000);
}

// ===========================================================================
// 5. Feature-gated git integration tests
// ===========================================================================

#[cfg(feature = "git")]
mod git_integration {
    use std::process::Command;
    use tokmd_context_git::compute_git_scores;
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

        Command::new("git").args(["init"]).current_dir(root).output().ok()?;
        Command::new("git")
            .args(["config", "user.email", "test@w67.com"])
            .current_dir(root)
            .output()
            .ok()?;
        Command::new("git")
            .args(["config", "user.name", "W67Test"])
            .current_dir(root)
            .output()
            .ok()?;

        std::fs::write(root.join("main.rs"), "fn main() {}\n").ok()?;
        Command::new("git").args(["add", "."]).current_dir(root).output().ok()?;
        Command::new("git")
            .args(["commit", "-m", "init"])
            .current_dir(root)
            .output()
            .ok()?;

        std::fs::write(root.join("main.rs"), "fn main() {\n    println!(\"hi\");\n}\n").ok()?;
        Command::new("git").args(["add", "."]).current_dir(root).output().ok()?;
        Command::new("git")
            .args(["commit", "-m", "update"])
            .current_dir(root)
            .output()
            .ok()?;

        std::fs::write(root.join("lib.rs"), "pub fn hello() {}\n").ok()?;
        Command::new("git").args(["add", "."]).current_dir(root).output().ok()?;
        Command::new("git")
            .args(["commit", "-m", "add lib"])
            .current_dir(root)
            .output()
            .ok()?;

        Some(dir)
    }

    #[test]
    fn non_repo_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let rows = vec![make_row("main.rs", 10)];
        assert!(compute_git_scores(dir.path(), &rows, 100, 100).is_none());
    }

    #[test]
    fn valid_repo_returns_some() {
        let repo = match create_test_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![make_row("main.rs", 3)];
        assert!(compute_git_scores(repo.path(), &rows, 100, 100).is_some());
    }

    #[test]
    fn commit_counts_match_expected() {
        let repo = match create_test_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![make_row("main.rs", 3), make_row("lib.rs", 1)];
        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };
        assert_eq!(scores.commit_counts.get("main.rs"), Some(&2));
        assert_eq!(scores.commit_counts.get("lib.rs"), Some(&1));
    }

    #[test]
    fn hotspot_equals_lines_times_commits() {
        let repo = match create_test_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![make_row("main.rs", 3), make_row("lib.rs", 1)];
        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };
        // main.rs: 3 lines × 2 commits = 6
        assert_eq!(scores.hotspots.get("main.rs"), Some(&6));
        // lib.rs: 1 line × 1 commit = 1
        assert_eq!(scores.hotspots.get("lib.rs"), Some(&1));
    }

    #[test]
    fn child_rows_are_filtered_out() {
        let repo = match create_test_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![FileRow {
            path: "main.rs".to_string(),
            module: "(root)".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Child,
            code: 3,
            comments: 0,
            blanks: 0,
            lines: 3,
            bytes: 30,
            tokens: 15,
        }];
        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };
        assert!(scores.commit_counts.is_empty());
        assert!(scores.hotspots.is_empty());
    }

    #[test]
    fn empty_rows_returns_empty_scores() {
        let repo = match create_test_repo() {
            Some(r) => r,
            None => return,
        };
        let Some(scores) = compute_git_scores(repo.path(), &[], 100, 100) else {
            return;
        };
        assert!(scores.commit_counts.is_empty());
        assert!(scores.hotspots.is_empty());
    }

    #[test]
    fn untracked_file_not_in_scores() {
        let repo = match create_test_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![make_row("nonexistent.rs", 10)];
        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };
        assert!(!scores.commit_counts.contains_key("nonexistent.rs"));
    }
}
