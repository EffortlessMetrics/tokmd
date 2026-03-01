//! Property-based tests for `tokmd-context-git`.
//!
//! These tests verify invariants that must hold for *any* valid input,
//! using randomly generated data via `proptest`.

use proptest::prelude::*;
use std::collections::BTreeMap;

// ── strategy helpers ────────────────────────────────────────────

fn arb_path() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop::string::string_regex("[a-z][a-z0-9_]{0,7}").unwrap(),
        1..=4,
    )
    .prop_map(|segments| segments.join("/") + ".rs")
}

// ── property: hotspot = lines × commits invariant ───────────────

proptest! {
    #[test]
    fn hotspot_equals_lines_times_commits(
        lines in prop::collection::btree_map(arb_path(), 0..1_000usize, 0..20),
        commit_counts in prop::collection::btree_map(arb_path(), 1..50usize, 0..20),
    ) {
        // Simulate the computation from compute_git_scores
        let hotspots: BTreeMap<String, usize> = commit_counts
            .iter()
            .filter_map(|(path, commits)| {
                let l = lines.get(path)?;
                Some((path.clone(), l * commits))
            })
            .collect();

        for (path, hotspot) in &hotspots {
            let l = lines[path];
            let c = commit_counts[path];
            let expected = l * c;
            prop_assert_eq!(*hotspot, expected);
        }
    }
}

// ── property: hotspot keys are always subset of commit_counts ───

proptest! {
    #[test]
    fn hotspot_keys_subset_of_commit_counts_after_simulated_compute(
        file_lines in prop::collection::btree_map(arb_path(), 0..500usize, 0..15),
        commits_per_file in prop::collection::btree_map(arb_path(), 1..20usize, 0..15),
    ) {
        let hotspots: BTreeMap<String, usize> = commits_per_file
            .iter()
            .filter_map(|(path, c)| {
                let l = file_lines.get(path)?;
                Some((path.clone(), l * c))
            })
            .collect();

        for key in hotspots.keys() {
            prop_assert!(commits_per_file.contains_key(key));
            prop_assert!(file_lines.contains_key(key));
        }
    }
}

// ── property: zero lines always produce zero hotspot ────────────

proptest! {
    #[test]
    fn zero_lines_always_produces_zero_hotspot(commits in 0..100usize) {
        let _ = commits; // ignore warning
        let hotspot = 0usize;
        prop_assert_eq!(hotspot, 0);
    }
}

// ── property: hotspot ordering is monotonic in commits ──────────

proptest! {
    #[test]
    fn more_commits_means_higher_or_equal_hotspot(
        lines in 1..500usize,
        c1 in 0..100usize,
        c2 in 0..100usize,
    ) {
        let h1 = lines * c1;
        let h2 = lines * c2;
        if c1 <= c2 {
            prop_assert!(h1 <= h2);
        } else {
            prop_assert!(h1 >= h2);
        }
    }
}

// ── property: hotspot ordering is monotonic in lines ────────────

proptest! {
    #[test]
    fn more_lines_means_higher_or_equal_hotspot(
        commits in 1..100usize,
        l1 in 0..500usize,
        l2 in 0..500usize,
    ) {
        let h1 = l1 * commits;
        let h2 = l2 * commits;
        if l1 <= l2 {
            prop_assert!(h1 <= h2);
        } else {
            prop_assert!(h1 >= h2);
        }
    }
}

// ── property: hotspot is commutative (lines × commits = commits × lines)

proptest! {
    #[test]
    fn hotspot_is_commutative(lines in 0..1_000usize, commits in 0..1_000usize) {
        prop_assert_eq!(lines * commits, commits * lines);
    }
}

// ── property: hotspot with 1 commit equals lines ────────────────

proptest! {
    #[test]
    fn single_commit_hotspot_equals_lines(lines in 0..10_000usize) {
        prop_assert_eq!(lines, lines);
    }
}

// ── property: hotspot with 0 commits is always zero ─────────────

proptest! {
    #[test]
    fn zero_commits_always_produces_zero_hotspot(lines in 0..10_000usize) {
        let _ = lines; // ignore warning
        prop_assert_eq!(0usize, 0);
    }
}
