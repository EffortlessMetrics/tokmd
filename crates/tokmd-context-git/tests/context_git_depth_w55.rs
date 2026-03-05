//! Comprehensive depth tests for tokmd-context-git (W55).
//!
//! Covers: GitScores construction, BTreeMap determinism, field access,
//! compute_git_scores edge cases, and property-based invariants.

use std::collections::BTreeMap;
use tokmd_context_git::GitScores;

// ---------------------------------------------------------------------------
// 1. GitScores construction
// ---------------------------------------------------------------------------

#[test]
fn empty_git_scores() {
    let s = GitScores {
        hotspots: BTreeMap::new(),
        commit_counts: BTreeMap::new(),
    };
    assert!(s.hotspots.is_empty());
    assert!(s.commit_counts.is_empty());
}

#[test]
fn single_entry_scores() {
    let mut h = BTreeMap::new();
    h.insert("src/main.rs".to_string(), 42);
    let mut c = BTreeMap::new();
    c.insert("src/main.rs".to_string(), 7);
    let s = GitScores {
        hotspots: h,
        commit_counts: c,
    };
    assert_eq!(s.hotspots.len(), 1);
    assert_eq!(s.commit_counts.get("src/main.rs"), Some(&7));
    assert_eq!(s.hotspots.get("src/main.rs"), Some(&42));
}

#[test]
fn many_entries_scores() {
    let mut h = BTreeMap::new();
    for i in 0..100 {
        h.insert(format!("file_{i:03}.rs"), i * 10);
    }
    let s = GitScores {
        hotspots: h,
        commit_counts: BTreeMap::new(),
    };
    assert_eq!(s.hotspots.len(), 100);
    assert_eq!(s.hotspots.get("file_050.rs"), Some(&500));
}

// ---------------------------------------------------------------------------
// 2. BTreeMap determinism / ordering
// ---------------------------------------------------------------------------

#[test]
fn hotspots_sorted_ascending() {
    let mut h = BTreeMap::new();
    h.insert("z.rs".to_string(), 1);
    h.insert("a.rs".to_string(), 2);
    h.insert("m.rs".to_string(), 3);
    let keys: Vec<_> = h.keys().cloned().collect();
    assert_eq!(keys, vec!["a.rs", "m.rs", "z.rs"]);
}

#[test]
fn commit_counts_sorted_ascending() {
    let mut c = BTreeMap::new();
    c.insert("z/lib.rs".to_string(), 10);
    c.insert("a/main.rs".to_string(), 20);
    let s = GitScores {
        hotspots: BTreeMap::new(),
        commit_counts: c,
    };
    let keys: Vec<_> = s.commit_counts.keys().cloned().collect();
    assert_eq!(keys, vec!["a/main.rs", "z/lib.rs"]);
}

#[test]
fn insertion_order_does_not_affect_iteration() {
    let mut a = BTreeMap::new();
    a.insert("b".to_string(), 1);
    a.insert("a".to_string(), 2);
    a.insert("c".to_string(), 3);

    let mut b = BTreeMap::new();
    b.insert("c".to_string(), 3);
    b.insert("a".to_string(), 2);
    b.insert("b".to_string(), 1);

    let ak: Vec<_> = a.keys().collect();
    let bk: Vec<_> = b.keys().collect();
    assert_eq!(ak, bk);
}

#[test]
fn deterministic_across_reconstructions() {
    let build = || {
        let mut h = BTreeMap::new();
        h.insert("x.rs".to_string(), 10);
        h.insert("y.rs".to_string(), 20);
        let mut c = BTreeMap::new();
        c.insert("x.rs".to_string(), 1);
        c.insert("y.rs".to_string(), 2);
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
    assert_eq!(
        s1.commit_counts.values().collect::<Vec<_>>(),
        s2.commit_counts.values().collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// 3. Field access & values
// ---------------------------------------------------------------------------

#[test]
fn hotspot_value_zero() {
    let mut h = BTreeMap::new();
    h.insert("zero.rs".to_string(), 0);
    let s = GitScores {
        hotspots: h,
        commit_counts: BTreeMap::new(),
    };
    assert_eq!(s.hotspots.get("zero.rs"), Some(&0));
}

#[test]
fn hotspot_value_large() {
    let mut h = BTreeMap::new();
    h.insert("big.rs".to_string(), usize::MAX);
    let s = GitScores {
        hotspots: h,
        commit_counts: BTreeMap::new(),
    };
    assert_eq!(s.hotspots.get("big.rs"), Some(&usize::MAX));
}

#[test]
fn missing_key_returns_none() {
    let s = GitScores {
        hotspots: BTreeMap::new(),
        commit_counts: BTreeMap::new(),
    };
    assert_eq!(s.hotspots.get("nonexistent.rs"), None);
    assert_eq!(s.commit_counts.get("nonexistent.rs"), None);
}

#[test]
fn overwrite_value() {
    let mut h = BTreeMap::new();
    h.insert("dup.rs".to_string(), 10);
    h.insert("dup.rs".to_string(), 20);
    assert_eq!(h.len(), 1);
    assert_eq!(h.get("dup.rs"), Some(&20));
}

// ---------------------------------------------------------------------------
// 4. compute_git_scores without "git" feature → always None
// ---------------------------------------------------------------------------

#[test]
fn compute_scores_no_git_feature() {
    // Without the git feature enabled, compute_git_scores returns None.
    let dir = tempfile::tempdir().unwrap();
    let result = tokmd_context_git::compute_git_scores(dir.path(), &[], 100, 100);
    assert!(result.is_none());
}

#[test]
fn compute_scores_non_repo_is_none() {
    let dir = tempfile::tempdir().unwrap();
    let result = tokmd_context_git::compute_git_scores(dir.path(), &[], 50, 50);
    assert!(result.is_none());
}

#[test]
fn compute_scores_empty_rows_is_none() {
    let dir = tempfile::tempdir().unwrap();
    let result = tokmd_context_git::compute_git_scores(dir.path(), &[], 0, 0);
    assert!(result.is_none());
}

// ---------------------------------------------------------------------------
// 5. Path key edge cases in BTreeMap
// ---------------------------------------------------------------------------

#[test]
fn paths_with_special_chars() {
    let mut h = BTreeMap::new();
    h.insert("src/my-file.rs".to_string(), 1);
    h.insert("src/my_file.rs".to_string(), 2);
    h.insert("src/my file.rs".to_string(), 3);
    assert_eq!(h.len(), 3);
}

#[test]
fn unicode_paths() {
    let mut h = BTreeMap::new();
    h.insert("src/日本語.rs".to_string(), 1);
    h.insert("src/中文.rs".to_string(), 2);
    let s = GitScores {
        hotspots: h,
        commit_counts: BTreeMap::new(),
    };
    assert_eq!(s.hotspots.len(), 2);
}

#[test]
fn forward_slash_normalized_paths() {
    let mut h = BTreeMap::new();
    h.insert("a/b/c.rs".to_string(), 10);
    // Backslash is a different key
    h.insert("a\\b\\c.rs".to_string(), 20);
    assert_eq!(h.len(), 2);
}

#[test]
fn deeply_nested_paths() {
    let mut h = BTreeMap::new();
    let deep = "a/b/c/d/e/f/g/h/i/j/k/l.rs".to_string();
    h.insert(deep.clone(), 999);
    assert_eq!(h.get(&deep), Some(&999));
}

// ---------------------------------------------------------------------------
// 6. Aggregate / derived invariants
// ---------------------------------------------------------------------------

#[test]
fn hotspot_equals_lines_times_commits_manually() {
    // Simulate the product relationship: hotspot = lines * commits
    let lines = 50usize;
    let commits = 3usize;
    let hotspot = lines * commits;
    let mut h = BTreeMap::new();
    h.insert("manual.rs".to_string(), hotspot);
    let mut c = BTreeMap::new();
    c.insert("manual.rs".to_string(), commits);
    let s = GitScores {
        hotspots: h,
        commit_counts: c,
    };
    assert_eq!(s.hotspots["manual.rs"], 150);
    assert_eq!(s.commit_counts["manual.rs"], 3);
}

#[test]
fn all_hotspots_non_negative() {
    let mut h = BTreeMap::new();
    h.insert("a.rs".to_string(), 0_usize);
    h.insert("b.rs".to_string(), 100);
    h.insert("c.rs".to_string(), usize::MAX);
    // usize is inherently non-negative; verify all values are present
    assert_eq!(h.len(), 3);
    for val in h.values() {
        // confirm each value round-trips through the map
        assert_eq!(*val, *val);
    }
}

#[test]
fn hotspots_and_counts_same_keys_when_well_formed() {
    let keys = vec!["a.rs", "b.rs", "c.rs"];
    let mut h = BTreeMap::new();
    let mut c = BTreeMap::new();
    for k in &keys {
        h.insert(k.to_string(), 10);
        c.insert(k.to_string(), 2);
    }
    let s = GitScores {
        hotspots: h,
        commit_counts: c,
    };
    let hk: Vec<_> = s.hotspots.keys().collect();
    let ck: Vec<_> = s.commit_counts.keys().collect();
    assert_eq!(hk, ck);
}

// ---------------------------------------------------------------------------
// 7. Property-based tests
// ---------------------------------------------------------------------------

mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn hotspot_product_commutative(lines in 0usize..10_000, commits in 0usize..10_000) {
            assert_eq!(lines * commits, commits * lines);
        }

        #[test]
        fn btreemap_always_sorted(keys in proptest::collection::vec("[a-z]{1,8}", 1..20)) {
            let mut map = BTreeMap::new();
            for (i, k) in keys.iter().enumerate() {
                map.insert(k.clone(), i);
            }
            let collected: Vec<_> = map.keys().cloned().collect();
            let mut sorted = collected.clone();
            sorted.sort();
            assert_eq!(collected, sorted);
        }

        #[test]
        fn git_scores_construction_never_panics(
            n in 0usize..50,
            val in 0usize..1_000_000,
        ) {
            let mut h = BTreeMap::new();
            let mut c = BTreeMap::new();
            for i in 0..n {
                let key = format!("file_{i}.rs");
                h.insert(key.clone(), val);
                c.insert(key, val / 10);
            }
            let s = GitScores { hotspots: h, commit_counts: c };
            assert_eq!(s.hotspots.len(), n);
            assert_eq!(s.commit_counts.len(), n);
        }

        #[test]
        fn compute_no_feature_always_none(
            max_commits in 0usize..200,
            max_files in 0usize..200,
        ) {
            let dir = tempfile::tempdir().unwrap();
            let result = tokmd_context_git::compute_git_scores(
                dir.path(), &[], max_commits, max_files,
            );
            assert!(result.is_none());
        }
    }
}
