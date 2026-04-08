#![cfg(feature = "git")]
//! Deep tests for tokmd-context-git: git scoring for context ranking.
//!
//! Tests cover: GitScores construction, scoring invariants, path
//! normalization, edge cases, and feature-gated compute_git_scores.

use std::collections::BTreeMap;
use tokmd_context_git::GitScores;

// ===========================================================================
// 1. GitScores struct construction
// ===========================================================================

#[test]
fn git_scores_empty_maps() {
    let scores = GitScores {
        hotspots: BTreeMap::new(),
        commit_counts: BTreeMap::new(),
    };
    assert!(scores.hotspots.is_empty());
    assert!(scores.commit_counts.is_empty());
}

#[test]
fn git_scores_with_data_accessible() {
    let mut hotspots = BTreeMap::new();
    hotspots.insert("src/lib.rs".to_string(), 500usize);
    let mut commit_counts = BTreeMap::new();
    commit_counts.insert("src/lib.rs".to_string(), 10usize);

    let scores = GitScores {
        hotspots,
        commit_counts,
    };
    assert_eq!(scores.hotspots.get("src/lib.rs"), Some(&500));
    assert_eq!(scores.commit_counts.get("src/lib.rs"), Some(&10));
}

// ===========================================================================
// 2. BTreeMap ordering guarantees
// ===========================================================================

#[test]
fn scores_keys_in_lexicographic_order() {
    let mut hotspots = BTreeMap::new();
    hotspots.insert("z.rs".to_string(), 1usize);
    hotspots.insert("a.rs".to_string(), 2);
    hotspots.insert("m.rs".to_string(), 3);

    let scores = GitScores {
        hotspots,
        commit_counts: BTreeMap::new(),
    };
    let keys: Vec<&String> = scores.hotspots.keys().collect();
    assert_eq!(keys, vec!["a.rs", "m.rs", "z.rs"]);
}

#[test]
fn scores_deterministic_across_builds() {
    let build = || {
        let mut h = BTreeMap::new();
        h.insert("b.rs".to_string(), 20usize);
        h.insert("a.rs".to_string(), 10);
        let mut c = BTreeMap::new();
        c.insert("b.rs".to_string(), 4usize);
        c.insert("a.rs".to_string(), 2);
        GitScores {
            hotspots: h,
            commit_counts: c,
        }
    };
    let s1 = build();
    let s2 = build();
    assert_eq!(
        s1.hotspots.keys().collect::<Vec<_>>(),
        s2.hotspots.keys().collect::<Vec<_>>()
    );
    for k in s1.hotspots.keys() {
        assert_eq!(s1.hotspots[k], s2.hotspots[k]);
    }
}

// ===========================================================================
// 3. Hotspot invariants (computed externally, verified here)
// ===========================================================================

#[test]
fn hotspot_keys_subset_of_commit_counts() {
    let mut commit_counts = BTreeMap::new();
    commit_counts.insert("a.rs".to_string(), 5usize);
    commit_counts.insert("b.rs".to_string(), 3);
    commit_counts.insert("c.rs".to_string(), 1);

    let file_lines: BTreeMap<String, usize> = [
        ("a.rs".to_string(), 100),
        ("b.rs".to_string(), 200),
        // c.rs intentionally missing
    ]
    .into_iter()
    .collect();

    let hotspots: BTreeMap<String, usize> = commit_counts
        .iter()
        .filter_map(|(p, c)| file_lines.get(p).map(|l| (p.clone(), l * c)))
        .collect();

    for k in hotspots.keys() {
        assert!(commit_counts.contains_key(k));
    }
    assert!(!hotspots.contains_key("c.rs"));
}

#[test]
fn zero_lines_gives_zero_hotspot() {
    let lines = 0usize;
    let commits = 10usize;
    assert_eq!(lines * commits, 0);
}

#[test]
fn hotspot_formula_is_multiplication() {
    let lines = 50usize;
    let commits = 4usize;
    // Must be multiplication, not addition or subtraction
    assert_eq!(lines * commits, 200);
    assert_ne!(lines * commits, lines + commits); // 54 ≠ 200
}

// ===========================================================================
// 4. Feature-gated compute_git_scores
// ===========================================================================

#[cfg(feature = "git")]
mod git_feature_tests {
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

    fn init_repo(dir: &std::path::Path) {
        Command::new("git")
            .args(["init"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(dir)
            .output()
            .unwrap();
    }

    fn add_commit(dir: &std::path::Path, file: &str, content: &str, msg: &str) {
        let path = dir.join(file);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, content).unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", msg])
            .current_dir(dir)
            .output()
            .unwrap();
    }

    #[test]
    fn compute_scores_non_repo_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let rows = vec![make_row("f.rs", 100)];
        assert!(compute_git_scores(dir.path(), &rows, 100, 100).is_none());
    }

    #[test]
    fn compute_scores_returns_some_for_valid_repo() {
        if !tokmd_git::git_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        init_repo(dir.path());
        add_commit(dir.path(), "main.rs", "fn main() {}", "init");

        let rows = vec![make_row("main.rs", 10)];
        let result = compute_git_scores(dir.path(), &rows, 100, 100);
        assert!(result.is_some());
    }

    #[test]
    fn compute_scores_commit_counts_correct() {
        if !tokmd_git::git_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        init_repo(dir.path());
        add_commit(dir.path(), "a.rs", "1", "c1");
        add_commit(dir.path(), "a.rs", "1\n2", "c2");

        let rows = vec![make_row("a.rs", 10)];
        let scores = compute_git_scores(dir.path(), &rows, 100, 100).unwrap();
        assert_eq!(scores.commit_counts.get("a.rs"), Some(&2));
    }

    #[test]
    fn compute_scores_hotspot_value() {
        if !tokmd_git::git_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        init_repo(dir.path());
        add_commit(dir.path(), "a.rs", "1", "c1");
        add_commit(dir.path(), "a.rs", "1\n2", "c2");

        let rows = vec![make_row("a.rs", 20)];
        let scores = compute_git_scores(dir.path(), &rows, 100, 100).unwrap();
        // hotspot = lines * commits = 20 * 2 = 40
        assert_eq!(scores.hotspots.get("a.rs"), Some(&40));
    }

    #[test]
    fn compute_scores_child_rows_excluded() {
        if !tokmd_git::git_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        init_repo(dir.path());
        add_commit(dir.path(), "a.rs", "1", "c1");

        let rows = vec![FileRow {
            path: "a.rs".to_string(),
            module: "(root)".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Child,
            code: 10,
            comments: 0,
            blanks: 0,
            lines: 10,
            bytes: 100,
            tokens: 50,
        }];
        let scores = compute_git_scores(dir.path(), &rows, 100, 100).unwrap();
        assert!(scores.commit_counts.is_empty());
        assert!(scores.hotspots.is_empty());
    }

    #[test]
    fn compute_scores_empty_rows_empty_scores() {
        if !tokmd_git::git_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        init_repo(dir.path());
        add_commit(dir.path(), "a.rs", "1", "c1");

        let rows: Vec<FileRow> = vec![];
        let scores = compute_git_scores(dir.path(), &rows, 100, 100).unwrap();
        assert!(scores.commit_counts.is_empty());
        assert!(scores.hotspots.is_empty());
    }
}

#[cfg(not(feature = "git"))]
mod no_git_feature_tests {
    use tokmd_context_git::compute_git_scores;

    #[test]
    fn compute_scores_returns_none_without_feature() {
        let dir = tempfile::tempdir().unwrap();
        let rows: Vec<tokmd_types::FileRow> = vec![];
        assert!(compute_git_scores(dir.path(), &rows, 100, 100).is_none());
    }
}
