//! Deep tests for tokmd-context-git: GitScores construction and invariants.

use std::collections::BTreeMap;

use tokmd_context_git::GitScores;

// ─── GitScores construction ────────────────────────────────────

#[test]
fn empty_scores() {
    let scores = GitScores {
        hotspots: BTreeMap::new(),
        commit_counts: BTreeMap::new(),
    };
    assert!(scores.hotspots.is_empty());
    assert!(scores.commit_counts.is_empty());
}

#[test]
fn scores_with_single_file() {
    let mut hotspots = BTreeMap::new();
    hotspots.insert("src/lib.rs".to_string(), 42);
    let mut commit_counts = BTreeMap::new();
    commit_counts.insert("src/lib.rs".to_string(), 7);

    let scores = GitScores {
        hotspots,
        commit_counts,
    };
    assert_eq!(scores.hotspots.get("src/lib.rs"), Some(&42));
    assert_eq!(scores.commit_counts.get("src/lib.rs"), Some(&7));
}

#[test]
fn scores_with_many_files() {
    let mut hotspots = BTreeMap::new();
    let mut commit_counts = BTreeMap::new();
    for i in 0..100 {
        let path = format!("src/file_{}.rs", i);
        hotspots.insert(path.clone(), i * 10);
        commit_counts.insert(path, i);
    }

    let scores = GitScores {
        hotspots,
        commit_counts,
    };
    assert_eq!(scores.hotspots.len(), 100);
    assert_eq!(scores.commit_counts.len(), 100);
    assert_eq!(scores.hotspots.get("src/file_50.rs"), Some(&500));
}

// ─── BTreeMap ordering ─────────────────────────────────────────

#[test]
fn hotspots_sorted_lexicographically() {
    let mut hotspots = BTreeMap::new();
    hotspots.insert("z/last.rs".to_string(), 1);
    hotspots.insert("a/first.rs".to_string(), 2);
    hotspots.insert("m/middle.rs".to_string(), 3);

    let scores = GitScores {
        hotspots,
        commit_counts: BTreeMap::new(),
    };
    let keys: Vec<&String> = scores.hotspots.keys().collect();
    assert_eq!(keys, vec!["a/first.rs", "m/middle.rs", "z/last.rs"]);
}

#[test]
fn commit_counts_sorted_lexicographically() {
    let mut commit_counts = BTreeMap::new();
    commit_counts.insert("tests/z.rs".to_string(), 1);
    commit_counts.insert("src/a.rs".to_string(), 5);
    commit_counts.insert("lib/m.rs".to_string(), 3);

    let scores = GitScores {
        hotspots: BTreeMap::new(),
        commit_counts,
    };
    let keys: Vec<&String> = scores.commit_counts.keys().collect();
    assert_eq!(keys, vec!["lib/m.rs", "src/a.rs", "tests/z.rs"]);
}

// ─── score values ──────────────────────────────────────────────

#[test]
fn hotspot_zero_score() {
    let mut hotspots = BTreeMap::new();
    hotspots.insert("empty.rs".to_string(), 0);
    let scores = GitScores {
        hotspots,
        commit_counts: BTreeMap::new(),
    };
    assert_eq!(scores.hotspots.get("empty.rs"), Some(&0));
}

#[test]
fn hotspot_large_score() {
    let mut hotspots = BTreeMap::new();
    hotspots.insert("huge.rs".to_string(), usize::MAX);
    let scores = GitScores {
        hotspots,
        commit_counts: BTreeMap::new(),
    };
    assert_eq!(scores.hotspots.get("huge.rs"), Some(&usize::MAX));
}

#[test]
fn commit_count_zero() {
    let mut commit_counts = BTreeMap::new();
    commit_counts.insert("untracked.rs".to_string(), 0);
    let scores = GitScores {
        hotspots: BTreeMap::new(),
        commit_counts,
    };
    assert_eq!(scores.commit_counts.get("untracked.rs"), Some(&0));
}

// ─── path handling ─────────────────────────────────────────────

#[test]
fn forward_slash_paths() {
    let mut hotspots = BTreeMap::new();
    hotspots.insert("src/deep/nested/file.rs".to_string(), 100);
    let scores = GitScores {
        hotspots,
        commit_counts: BTreeMap::new(),
    };
    assert!(scores.hotspots.contains_key("src/deep/nested/file.rs"));
}

#[test]
fn missing_path_returns_none() {
    let scores = GitScores {
        hotspots: BTreeMap::new(),
        commit_counts: BTreeMap::new(),
    };
    assert_eq!(scores.hotspots.get("nonexistent.rs"), None);
    assert_eq!(scores.commit_counts.get("nonexistent.rs"), None);
}

// ─── independent hotspots and commit counts ────────────────────

#[test]
fn hotspots_and_counts_can_differ() {
    let mut hotspots = BTreeMap::new();
    hotspots.insert("a.rs".to_string(), 100);
    // commit_counts has different set of files
    let mut commit_counts = BTreeMap::new();
    commit_counts.insert("b.rs".to_string(), 5);

    let scores = GitScores {
        hotspots,
        commit_counts,
    };
    assert!(scores.hotspots.contains_key("a.rs"));
    assert!(!scores.hotspots.contains_key("b.rs"));
    assert!(scores.commit_counts.contains_key("b.rs"));
    assert!(!scores.commit_counts.contains_key("a.rs"));
}

#[test]
fn overwrite_existing_key() {
    let mut hotspots = BTreeMap::new();
    hotspots.insert("file.rs".to_string(), 10);
    hotspots.insert("file.rs".to_string(), 20);
    let scores = GitScores {
        hotspots,
        commit_counts: BTreeMap::new(),
    };
    // BTreeMap overwrites, last value wins
    assert_eq!(scores.hotspots.get("file.rs"), Some(&20));
}
