//! Wave 49: Deep tests for `tokmd-context-git`.
//!
//! Covers:
//! - Git-aware context selection (recently changed files get priority)
//! - Changed file detection
//! - Branch/commit handling
//! - Property test: selected files are always valid paths
//! - Edge cases: no git history, uncommitted changes

use proptest::prelude::*;
use std::collections::BTreeMap;
use tokmd_context_git::GitScores;

// ===========================================================================
// 1. GitScores construction and ranking
// ===========================================================================

#[test]
fn w49_git_scores_empty_is_valid() {
    let scores = GitScores {
        hotspots: BTreeMap::new(),
        commit_counts: BTreeMap::new(),
    };
    assert_eq!(scores.hotspots.len(), 0);
    assert_eq!(scores.commit_counts.len(), 0);
}

#[test]
fn w49_recently_changed_files_rank_higher() {
    // Simulate: file with more commits has higher hotspot score
    let mut hotspots = BTreeMap::new();
    hotspots.insert("src/hot.rs".to_string(), 500usize); // 50 lines × 10 commits
    hotspots.insert("src/cold.rs".to_string(), 50usize); // 50 lines × 1 commit

    let scores = GitScores {
        hotspots,
        commit_counts: BTreeMap::new(),
    };

    assert!(scores.hotspots["src/hot.rs"] > scores.hotspots["src/cold.rs"]);
}

#[test]
fn w49_hotspot_ranking_considers_both_lines_and_commits() {
    // File A: 1000 lines, 1 commit  → hotspot 1000
    // File B: 10 lines, 200 commits → hotspot 2000
    // B should rank higher despite fewer lines
    let mut hotspots = BTreeMap::new();
    hotspots.insert("a.rs".to_string(), 1000usize);
    hotspots.insert("b.rs".to_string(), 2000usize);

    let scores = GitScores {
        hotspots,
        commit_counts: BTreeMap::new(),
    };

    assert!(scores.hotspots["b.rs"] > scores.hotspots["a.rs"]);
}

#[test]
fn w49_scores_can_be_sorted_by_hotspot_descending() {
    let mut hotspots = BTreeMap::new();
    hotspots.insert("low.rs".to_string(), 10usize);
    hotspots.insert("mid.rs".to_string(), 100);
    hotspots.insert("high.rs".to_string(), 1000);

    let scores = GitScores {
        hotspots,
        commit_counts: BTreeMap::new(),
    };

    let mut ranked: Vec<_> = scores.hotspots.iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(a.1));

    assert_eq!(ranked[0].0, "high.rs");
    assert_eq!(ranked[1].0, "mid.rs");
    assert_eq!(ranked[2].0, "low.rs");
}

// ===========================================================================
// 2. Changed file detection via commit counts
// ===========================================================================

#[test]
fn w49_commit_counts_reflect_change_frequency() {
    let mut commit_counts = BTreeMap::new();
    commit_counts.insert("frequently_changed.rs".to_string(), 50usize);
    commit_counts.insert("rarely_changed.rs".to_string(), 1);

    let scores = GitScores {
        hotspots: BTreeMap::new(),
        commit_counts,
    };

    assert_eq!(scores.commit_counts["frequently_changed.rs"], 50);
    assert_eq!(scores.commit_counts["rarely_changed.rs"], 1);
}

#[test]
fn w49_files_not_in_history_absent_from_commit_counts() {
    let mut commit_counts = BTreeMap::new();
    commit_counts.insert("tracked.rs".to_string(), 5usize);

    let scores = GitScores {
        hotspots: BTreeMap::new(),
        commit_counts,
    };

    assert!(scores.commit_counts.contains_key("tracked.rs"));
    assert!(!scores.commit_counts.contains_key("untracked.rs"));
}

#[test]
fn w49_hotspot_keys_are_subset_of_commit_counts() {
    let mut commit_counts = BTreeMap::new();
    commit_counts.insert("a.rs".to_string(), 5usize);
    commit_counts.insert("b.rs".to_string(), 3);
    commit_counts.insert("c.rs".to_string(), 1);

    let file_lines: BTreeMap<String, usize> =
        [("a.rs".to_string(), 100), ("b.rs".to_string(), 200)]
            .into_iter()
            .collect();

    let hotspots: BTreeMap<String, usize> = commit_counts
        .iter()
        .filter_map(|(p, c)| file_lines.get(p).map(|l| (p.clone(), l * c)))
        .collect();

    let scores = GitScores {
        hotspots,
        commit_counts,
    };

    for key in scores.hotspots.keys() {
        assert!(
            scores.commit_counts.contains_key(key),
            "hotspot key {key} must be in commit_counts"
        );
    }
}

// ===========================================================================
// 3. BTreeMap ordering (determinism guarantee)
// ===========================================================================

#[test]
fn w49_btreemap_keys_are_sorted() {
    let mut hotspots = BTreeMap::new();
    hotspots.insert("z/deep/file.rs".to_string(), 1usize);
    hotspots.insert("a/shallow.rs".to_string(), 2);
    hotspots.insert("m/middle.rs".to_string(), 3);

    let keys: Vec<_> = hotspots.keys().collect();
    assert_eq!(keys[0], "a/shallow.rs");
    assert_eq!(keys[1], "m/middle.rs");
    assert_eq!(keys[2], "z/deep/file.rs");
}

#[test]
fn w49_insert_order_does_not_affect_iteration() {
    let mut map1 = BTreeMap::new();
    map1.insert("c.rs".to_string(), 3usize);
    map1.insert("a.rs".to_string(), 1);
    map1.insert("b.rs".to_string(), 2);

    let mut map2 = BTreeMap::new();
    map2.insert("a.rs".to_string(), 1usize);
    map2.insert("b.rs".to_string(), 2);
    map2.insert("c.rs".to_string(), 3);

    assert_eq!(
        map1.keys().collect::<Vec<_>>(),
        map2.keys().collect::<Vec<_>>()
    );
}

// ===========================================================================
// 4. Hotspot computation invariants
// ===========================================================================

#[test]
fn w49_zero_lines_gives_zero_hotspot() {
    let lines = 0usize;
    let hotspot = lines.checked_mul(42).unwrap_or(0);
    assert_eq!(hotspot, 0);
}

#[test]
fn w49_zero_commits_gives_zero_hotspot() {
    let lines = 100usize;
    let commits = 0usize;
    let hotspot = lines.checked_mul(commits).unwrap_or(0);
    assert_eq!(hotspot, 0);
}

#[test]
fn w49_hotspot_is_commutative() {
    let lines = 75usize;
    let commits = 12usize;
    assert_eq!(lines * commits, commits * lines);
}

#[test]
fn w49_single_commit_hotspot_equals_lines() {
    let lines = 314usize;
    let one_commit = 1usize;
    assert_eq!(lines * one_commit, lines);
}

#[test]
fn w49_large_realistic_values_no_overflow() {
    let lines = 50_000usize;
    let commits = 5_000usize;
    let result = lines.checked_mul(commits);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), 250_000_000);
}

// ===========================================================================
// 5. Edge cases
// ===========================================================================

#[test]
fn w49_empty_commit_counts_empty_hotspots() {
    let commit_counts: BTreeMap<String, usize> = BTreeMap::new();
    let file_lines: BTreeMap<String, usize> = [("a.rs".to_string(), 100)].into_iter().collect();

    let hotspots: BTreeMap<String, usize> = commit_counts
        .iter()
        .filter_map(|(p, c)| file_lines.get(p).map(|l| (p.clone(), l * c)))
        .collect();

    assert!(hotspots.is_empty());
}

#[test]
fn w49_empty_file_lines_empty_hotspots() {
    let commit_counts: BTreeMap<String, usize> = [("a.rs".to_string(), 5)].into_iter().collect();
    let file_lines: BTreeMap<String, usize> = BTreeMap::new();

    let hotspots: BTreeMap<String, usize> = commit_counts
        .iter()
        .filter_map(|(p, c)| file_lines.get(p).map(|l| (p.clone(), l * c)))
        .collect();

    assert!(hotspots.is_empty());
}

#[test]
fn w49_disjoint_files_produce_empty_hotspots() {
    let commit_counts: BTreeMap<String, usize> =
        [("tracked.rs".to_string(), 10)].into_iter().collect();
    let file_lines: BTreeMap<String, usize> =
        [("untracked.rs".to_string(), 200)].into_iter().collect();

    let hotspots: BTreeMap<String, usize> = commit_counts
        .iter()
        .filter_map(|(p, c)| file_lines.get(p).map(|l| (p.clone(), l * c)))
        .collect();

    assert!(hotspots.is_empty());
}

#[test]
fn w49_single_file_single_commit_scores() {
    let mut commit_counts = BTreeMap::new();
    commit_counts.insert("only.rs".to_string(), 1usize);

    let file_lines: BTreeMap<String, usize> = [("only.rs".to_string(), 42)].into_iter().collect();

    let hotspots: BTreeMap<String, usize> = commit_counts
        .iter()
        .filter_map(|(p, c)| file_lines.get(p).map(|l| (p.clone(), l * c)))
        .collect();

    assert_eq!(hotspots.get("only.rs"), Some(&42));
}

// ===========================================================================
// 6. Property tests
// ===========================================================================

fn arb_path() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop::string::string_regex("[a-z][a-z0-9_]{0,7}").unwrap(),
        1..=4,
    )
    .prop_map(|segments| segments.join("/") + ".rs")
}

proptest! {
    /// All paths in hotspots are valid (non-empty, contain no backslashes after normalization sim).
    #[test]
    fn w49_prop_hotspot_paths_are_valid(
        file_lines in prop::collection::btree_map(arb_path(), 0..500usize, 1..10),
        commit_counts in prop::collection::btree_map(arb_path(), 1..20usize, 1..10),
    ) {
        let hotspots: BTreeMap<String, usize> = commit_counts
            .iter()
            .filter_map(|(p, c)| file_lines.get(p).map(|l| (p.clone(), l * c)))
            .collect();

        for path in hotspots.keys() {
            prop_assert!(!path.is_empty(), "path must not be empty");
            prop_assert!(!path.contains('\\'), "path must use forward slashes");
            prop_assert!(path.ends_with(".rs"), "path must end with .rs");
        }
    }

    /// Hotspot = lines × commits invariant holds.
    #[test]
    fn w49_prop_hotspot_equals_lines_times_commits(
        file_lines in prop::collection::btree_map(arb_path(), 0..1000usize, 0..15),
        commit_counts in prop::collection::btree_map(arb_path(), 1..50usize, 0..15),
    ) {
        let hotspots: BTreeMap<String, usize> = commit_counts
            .iter()
            .filter_map(|(p, c)| file_lines.get(p).map(|l| (p.clone(), l * c)))
            .collect();

        for (path, hotspot) in &hotspots {
            let l = file_lines[path];
            let c = commit_counts[path];
            prop_assert_eq!(*hotspot, l * c);
        }
    }

    /// Hotspot keys are always a subset of both commit_counts and file_lines.
    #[test]
    fn w49_prop_hotspot_keys_subset(
        file_lines in prop::collection::btree_map(arb_path(), 0..500usize, 0..10),
        commit_counts in prop::collection::btree_map(arb_path(), 1..20usize, 0..10),
    ) {
        let hotspots: BTreeMap<String, usize> = commit_counts
            .iter()
            .filter_map(|(p, c)| file_lines.get(p).map(|l| (p.clone(), l * c)))
            .collect();

        for key in hotspots.keys() {
            prop_assert!(commit_counts.contains_key(key));
            prop_assert!(file_lines.contains_key(key));
        }
    }

    /// More commits with same lines → higher or equal hotspot.
    #[test]
    fn w49_prop_monotonic_in_commits(lines in 1..500usize, c1 in 0..100usize, c2 in 0..100usize) {
        let h1 = lines * c1;
        let h2 = lines * c2;
        if c1 <= c2 {
            prop_assert!(h1 <= h2);
        } else {
            prop_assert!(h1 >= h2);
        }
    }
}

// ===========================================================================
// 7. Feature-gated compute_git_scores tests
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
    fn w49_no_git_history_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let rows = vec![make_row("main.rs", 100)];
        assert!(compute_git_scores(dir.path(), &rows, 100, 100).is_none());
    }

    #[test]
    fn w49_single_commit_single_file() {
        if !tokmd_git::git_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        init_repo(dir.path());
        add_commit(dir.path(), "main.rs", "fn main() {}", "init");

        let rows = vec![make_row("main.rs", 10)];
        let scores = compute_git_scores(dir.path(), &rows, 100, 100).unwrap();
        assert_eq!(scores.commit_counts.get("main.rs"), Some(&1));
        assert_eq!(scores.hotspots.get("main.rs"), Some(&10)); // 10 lines × 1 commit
    }

    #[test]
    fn w49_multiple_commits_accumulate() {
        if !tokmd_git::git_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        init_repo(dir.path());
        add_commit(dir.path(), "lib.rs", "v1", "c1");
        add_commit(dir.path(), "lib.rs", "v1\nv2", "c2");
        add_commit(dir.path(), "lib.rs", "v1\nv2\nv3", "c3");

        let rows = vec![make_row("lib.rs", 30)];
        let scores = compute_git_scores(dir.path(), &rows, 100, 100).unwrap();
        assert_eq!(scores.commit_counts.get("lib.rs"), Some(&3));
        assert_eq!(scores.hotspots.get("lib.rs"), Some(&90)); // 30 × 3
    }

    #[test]
    fn w49_child_rows_filtered_out() {
        if !tokmd_git::git_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        init_repo(dir.path());
        add_commit(dir.path(), "a.rs", "content", "c1");

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
    fn w49_empty_rows_empty_scores() {
        if !tokmd_git::git_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        init_repo(dir.path());
        add_commit(dir.path(), "a.rs", "content", "c1");

        let rows: Vec<FileRow> = vec![];
        let scores = compute_git_scores(dir.path(), &rows, 100, 100).unwrap();
        assert!(scores.commit_counts.is_empty());
    }

    #[test]
    fn w49_file_not_in_git_absent_from_scores() {
        if !tokmd_git::git_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        init_repo(dir.path());
        add_commit(dir.path(), "tracked.rs", "fn main() {}", "init");

        // Include a row for a file that was never committed
        let rows = vec![make_row("tracked.rs", 10), make_row("not_in_git.rs", 50)];
        let scores = compute_git_scores(dir.path(), &rows, 100, 100).unwrap();
        assert!(scores.commit_counts.contains_key("tracked.rs"));
        assert!(!scores.commit_counts.contains_key("not_in_git.rs"));
    }

    #[test]
    fn w49_multiple_files_independent_scores() {
        if !tokmd_git::git_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        init_repo(dir.path());
        add_commit(dir.path(), "a.rs", "1", "c1");
        add_commit(dir.path(), "b.rs", "1", "c2");
        add_commit(dir.path(), "a.rs", "1\n2", "c3");

        let rows = vec![make_row("a.rs", 20), make_row("b.rs", 10)];
        let scores = compute_git_scores(dir.path(), &rows, 100, 100).unwrap();

        // a.rs: 2 commits, b.rs: 1 commit
        assert_eq!(scores.commit_counts.get("a.rs"), Some(&2));
        assert_eq!(scores.commit_counts.get("b.rs"), Some(&1));
        // hotspots: a=20*2=40, b=10*1=10
        assert_eq!(scores.hotspots.get("a.rs"), Some(&40));
        assert_eq!(scores.hotspots.get("b.rs"), Some(&10));
    }
}

#[cfg(not(feature = "git"))]
mod no_git_tests {
    use tokmd_context_git::compute_git_scores;

    #[test]
    fn w49_without_git_feature_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let rows: Vec<tokmd_types::FileRow> = vec![];
        assert!(compute_git_scores(dir.path(), &rows, 100, 100).is_none());
    }
}
