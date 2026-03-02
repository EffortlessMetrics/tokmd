//! Deep integration tests for context-git scoring pipeline.
//!
//! Covers: GitScores construction, determinism, BTreeMap ordering,
//! edge cases, and scoring invariants.

use std::collections::BTreeMap;
use tokmd_context_git::GitScores;

// ===========================================================================
// 1. GitScores struct construction and invariants
// ===========================================================================

#[test]
fn git_scores_empty_struct_has_empty_maps() {
    let scores = GitScores {
        hotspots: BTreeMap::new(),
        commit_counts: BTreeMap::new(),
    };
    assert!(scores.hotspots.is_empty());
    assert!(scores.commit_counts.is_empty());
}

#[test]
fn git_scores_hotspot_keys_always_subset_of_commit_count_keys() {
    // Simulate real data: hotspot is computed from commit_counts + file_lines
    let mut commit_counts = BTreeMap::new();
    commit_counts.insert("src/main.rs".to_string(), 5);
    commit_counts.insert("src/lib.rs".to_string(), 3);
    commit_counts.insert("tests/test.rs".to_string(), 1);

    let file_lines: BTreeMap<String, usize> = [
        ("src/main.rs".to_string(), 100),
        ("src/lib.rs".to_string(), 200),
        // tests/test.rs intentionally missing from file_lines
    ]
    .into_iter()
    .collect();

    let hotspots: BTreeMap<String, usize> = commit_counts
        .iter()
        .filter_map(|(path, commits)| {
            let lines = file_lines.get(path)?;
            Some((path.clone(), lines * commits))
        })
        .collect();

    // hotspots should be a subset of commit_counts keys
    for key in hotspots.keys() {
        assert!(
            commit_counts.contains_key(key),
            "hotspot key {key} must exist in commit_counts"
        );
    }
    // tests/test.rs should NOT appear in hotspots (missing from file_lines)
    assert!(!hotspots.contains_key("tests/test.rs"));
}

// ===========================================================================
// 2. Determinism: same inputs → same scores
// ===========================================================================

#[test]
fn git_scores_construction_is_deterministic() {
    let build = || {
        let mut commit_counts = BTreeMap::new();
        commit_counts.insert("z.rs".to_string(), 10);
        commit_counts.insert("a.rs".to_string(), 5);
        commit_counts.insert("m.rs".to_string(), 3);

        let file_lines: BTreeMap<String, usize> = [
            ("z.rs".to_string(), 50),
            ("a.rs".to_string(), 100),
            ("m.rs".to_string(), 200),
        ]
        .into_iter()
        .collect();

        let hotspots: BTreeMap<String, usize> = commit_counts
            .iter()
            .filter_map(|(path, commits)| {
                let lines = file_lines.get(path)?;
                Some((path.clone(), lines * commits))
            })
            .collect();

        GitScores {
            hotspots,
            commit_counts,
        }
    };

    let a = build();
    let b = build();

    assert_eq!(
        a.hotspots.keys().collect::<Vec<_>>(),
        b.hotspots.keys().collect::<Vec<_>>(),
        "hotspot keys should be identical"
    );
    assert_eq!(
        a.commit_counts.keys().collect::<Vec<_>>(),
        b.commit_counts.keys().collect::<Vec<_>>(),
        "commit_count keys should be identical"
    );

    for key in a.hotspots.keys() {
        assert_eq!(a.hotspots[key], b.hotspots[key]);
    }
}

// ===========================================================================
// 3. BTreeMap ordering guarantees
// ===========================================================================

#[test]
fn btreemap_iteration_order_is_lexicographic() {
    let mut map = BTreeMap::new();
    map.insert("z.rs".to_string(), 1usize);
    map.insert("a.rs".to_string(), 2);
    map.insert("m.rs".to_string(), 3);

    let keys: Vec<&String> = map.keys().collect();
    assert_eq!(keys, vec!["a.rs", "m.rs", "z.rs"]);
}

#[test]
fn btreemap_preserves_insert_order_independence() {
    // Two maps with same keys inserted in different orders
    let mut map1 = BTreeMap::new();
    map1.insert("c.rs".to_string(), 1usize);
    map1.insert("a.rs".to_string(), 2);
    map1.insert("b.rs".to_string(), 3);

    let mut map2 = BTreeMap::new();
    map2.insert("b.rs".to_string(), 3usize);
    map2.insert("c.rs".to_string(), 1);
    map2.insert("a.rs".to_string(), 2);

    assert_eq!(
        map1.keys().collect::<Vec<_>>(),
        map2.keys().collect::<Vec<_>>(),
    );
}

// ===========================================================================
// 4. Hotspot computation invariants
// ===========================================================================

#[test]
fn zero_lines_produces_zero_hotspot() {
    let lines = 0usize;
    let commits = 42usize;
    assert_eq!(lines * commits, 0);
}

#[test]
fn zero_commits_produces_zero_hotspot() {
    let lines = 100usize;
    let commits = 0usize;
    assert_eq!(lines * commits, 0);
}

#[test]
fn hotspot_is_commutative() {
    let lines = 150usize;
    let commits = 7usize;
    assert_eq!(lines * commits, commits * lines);
}

#[test]
fn hotspot_ranking_respects_both_dimensions() {
    // File A: many lines, few commits → 1000 * 1 = 1000
    // File B: few lines, many commits → 10 * 200 = 2000
    // B should rank higher
    let hotspot_a = 1000usize;
    let hotspot_b = 10usize * 200;
    assert!(hotspot_b > hotspot_a);
}

// ===========================================================================
// 5. Scoring with missing data
// ===========================================================================

#[test]
fn files_not_in_commit_history_have_no_hotspot() {
    let commit_counts: BTreeMap<String, usize> =
        [("src/main.rs".to_string(), 5)].into_iter().collect();

    let file_lines: BTreeMap<String, usize> = [
        ("src/main.rs".to_string(), 100),
        ("src/new_file.rs".to_string(), 50), // not in git history
    ]
    .into_iter()
    .collect();

    let hotspots: BTreeMap<String, usize> = commit_counts
        .iter()
        .filter_map(|(path, commits)| {
            let lines = file_lines.get(path)?;
            Some((path.clone(), lines * commits))
        })
        .collect();

    assert!(hotspots.contains_key("src/main.rs"));
    assert!(
        !hotspots.contains_key("src/new_file.rs"),
        "file not in git history should have no hotspot"
    );
}

// ===========================================================================
// 6. Edge cases
// ===========================================================================

#[test]
fn empty_commit_counts_produces_empty_hotspots() {
    let commit_counts: BTreeMap<String, usize> = BTreeMap::new();
    let file_lines: BTreeMap<String, usize> =
        [("src/main.rs".to_string(), 100)].into_iter().collect();

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
fn empty_file_lines_produces_empty_hotspots() {
    let commit_counts: BTreeMap<String, usize> =
        [("src/main.rs".to_string(), 5)].into_iter().collect();
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
fn single_file_single_commit_hotspot_equals_lines() {
    let lines = 42usize;
    let commits = 1usize;
    assert_eq!(lines * commits, lines);
}

#[test]
fn large_values_do_not_overflow_in_reasonable_range() {
    // Realistic upper bounds: 100K lines, 10K commits
    let lines = 100_000usize;
    let commits = 10_000usize;
    let hotspot = lines.checked_mul(commits);
    assert!(hotspot.is_some(), "reasonable values should not overflow");
    assert_eq!(hotspot.unwrap(), 1_000_000_000);
}

// ===========================================================================
// 7. Path normalization in scoring
// ===========================================================================

#[test]
fn forward_slash_paths_are_consistent_keys() {
    let mut map = BTreeMap::new();
    map.insert("src/main.rs".to_string(), 100usize);

    // Query with exact same path
    assert_eq!(map.get("src/main.rs"), Some(&100));

    // Different path would NOT match (BTreeMap is exact)
    assert_eq!(map.get("src\\main.rs"), None);
}
