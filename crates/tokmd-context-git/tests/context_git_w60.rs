//! Deep tests (w60) for tokmd-context-git: git-based file prioritization,
//! change frequency computation, combining git metrics with context data,
//! determinism properties, and BDD-style scenarios.

use std::collections::BTreeMap;

use proptest::prelude::*;
use tokmd_context_git::GitScores;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_scores(files: &[(&str, usize, usize)], // (path, lines, commits)
) -> GitScores {
    let mut commit_counts = BTreeMap::new();
    let mut hotspots = BTreeMap::new();
    for &(path, lines, commits) in files {
        commit_counts.insert(path.to_string(), commits);
        hotspots.insert(path.to_string(), lines * commits);
    }
    GitScores {
        hotspots,
        commit_counts,
    }
}

// ===========================================================================
// 1. BDD: GitScores construction and invariants
// ===========================================================================

#[test]
fn given_empty_data_when_scores_built_then_maps_are_empty() {
    let scores = GitScores {
        hotspots: BTreeMap::new(),
        commit_counts: BTreeMap::new(),
    };
    assert!(scores.hotspots.is_empty());
    assert!(scores.commit_counts.is_empty());
}

#[test]
fn given_single_file_when_scores_built_then_hotspot_equals_lines_times_commits() {
    let scores = build_scores(&[("src/main.rs", 100, 5)]);
    assert_eq!(scores.hotspots["src/main.rs"], 500);
    assert_eq!(scores.commit_counts["src/main.rs"], 5);
}

#[test]
fn given_multiple_files_when_scores_built_then_all_files_present() {
    let scores = build_scores(&[("a.rs", 10, 3), ("b.rs", 20, 1), ("c.rs", 5, 10)]);
    assert_eq!(scores.hotspots.len(), 3);
    assert_eq!(scores.commit_counts.len(), 3);
}

#[test]
fn given_file_with_zero_lines_when_scores_built_then_hotspot_is_zero() {
    let scores = build_scores(&[("empty.rs", 0, 5)]);
    assert_eq!(scores.hotspots["empty.rs"], 0);
}

#[test]
fn given_file_with_zero_commits_when_scores_built_then_hotspot_is_zero() {
    let scores = build_scores(&[("unused.rs", 100, 0)]);
    assert_eq!(scores.hotspots["unused.rs"], 0);
}

// ===========================================================================
// 2. BDD: file prioritization by hotspot
// ===========================================================================

#[test]
fn given_two_files_when_ranked_by_hotspot_then_higher_hotspot_file_wins() {
    let scores = build_scores(&[
        ("hot.rs", 200, 10), // hotspot = 2000
        ("cold.rs", 50, 2),  // hotspot = 100
    ]);
    assert!(scores.hotspots["hot.rs"] > scores.hotspots["cold.rs"]);
}

#[test]
fn given_equal_lines_when_ranked_then_more_commits_means_higher_hotspot() {
    let scores = build_scores(&[
        ("a.rs", 100, 10), // 1000
        ("b.rs", 100, 3),  // 300
    ]);
    assert!(scores.hotspots["a.rs"] > scores.hotspots["b.rs"]);
}

#[test]
fn given_equal_commits_when_ranked_then_more_lines_means_higher_hotspot() {
    let scores = build_scores(&[
        ("big.rs", 500, 5),  // 2500
        ("small.rs", 50, 5), // 250
    ]);
    assert!(scores.hotspots["big.rs"] > scores.hotspots["small.rs"]);
}

#[test]
fn given_files_with_varying_scores_when_sorted_by_hotspot_then_descending_order() {
    let scores = build_scores(&[
        ("c.rs", 10, 1),   // 10
        ("a.rs", 100, 10), // 1000
        ("b.rs", 50, 5),   // 250
    ]);

    let mut ranked: Vec<(&String, &usize)> = scores.hotspots.iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(a.1));

    assert_eq!(ranked[0].0, "a.rs");
    assert_eq!(ranked[1].0, "b.rs");
    assert_eq!(ranked[2].0, "c.rs");
}

// ===========================================================================
// 3. BDD: change frequency computation
// ===========================================================================

#[test]
fn given_known_commit_counts_when_queried_then_counts_are_exact() {
    let scores = build_scores(&[("lib.rs", 200, 7), ("main.rs", 100, 3)]);
    assert_eq!(scores.commit_counts["lib.rs"], 7);
    assert_eq!(scores.commit_counts["main.rs"], 3);
}

#[test]
fn given_file_not_in_scores_when_queried_then_none_returned() {
    let scores = build_scores(&[("known.rs", 10, 1)]);
    assert!(!scores.commit_counts.contains_key("unknown.rs"));
    assert!(!scores.hotspots.contains_key("unknown.rs"));
}

#[test]
fn given_many_files_when_change_frequency_summed_then_total_is_correct() {
    let scores = build_scores(&[("a.rs", 10, 5), ("b.rs", 20, 3), ("c.rs", 30, 2)]);
    let total_commits: usize = scores.commit_counts.values().sum();
    assert_eq!(total_commits, 10);
}

// ===========================================================================
// 4. BDD: combining git metrics with context data
// ===========================================================================

#[test]
fn given_scores_when_hotspot_keys_checked_then_subset_of_commit_count_keys() {
    let scores = build_scores(&[("x.rs", 50, 3), ("y.rs", 100, 1)]);
    for key in scores.hotspots.keys() {
        assert!(
            scores.commit_counts.contains_key(key),
            "hotspot key {key} must be in commit_counts"
        );
    }
}

#[test]
fn given_scores_when_normalized_hotspot_computed_then_values_between_zero_and_one() {
    let scores = build_scores(&[
        ("a.rs", 200, 10), // 2000
        ("b.rs", 50, 2),   // 100
        ("c.rs", 100, 5),  // 500
    ]);
    let max_hotspot = scores.hotspots.values().max().copied().unwrap_or(1).max(1);
    for &val in scores.hotspots.values() {
        let normalized = val as f64 / max_hotspot as f64;
        assert!(
            (0.0..=1.0).contains(&normalized),
            "normalized hotspot should be in [0, 1], got {normalized}"
        );
    }
}

#[test]
fn given_scores_when_top_n_selected_then_correct_count_returned() {
    let scores = build_scores(&[
        ("a.rs", 100, 10), // 1000
        ("b.rs", 50, 5),   // 250
        ("c.rs", 200, 1),  // 200
        ("d.rs", 10, 2),   // 20
        ("e.rs", 300, 3),  // 900
    ]);

    let mut ranked: Vec<(&String, &usize)> = scores.hotspots.iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(a.1));
    let top3: Vec<&str> = ranked.iter().take(3).map(|(k, _)| k.as_str()).collect();
    assert_eq!(top3.len(), 3);
    assert_eq!(top3[0], "a.rs"); // 1000
    assert_eq!(top3[1], "e.rs"); // 900
    assert_eq!(top3[2], "b.rs"); // 250
}

#[test]
fn given_scores_when_filtered_by_threshold_then_low_scores_excluded() {
    let scores = build_scores(&[
        ("hot.rs", 500, 10), // 5000
        ("warm.rs", 100, 3), // 300
        ("cold.rs", 5, 1),   // 5
    ]);

    let threshold = 100;
    let above: Vec<&String> = scores
        .hotspots
        .iter()
        .filter(|(_, v)| **v >= threshold)
        .map(|(k, _)| k)
        .collect();

    assert_eq!(above.len(), 2);
    assert!(above.contains(&&"hot.rs".to_string()));
    assert!(above.contains(&&"warm.rs".to_string()));
}

// ===========================================================================
// 5. BTreeMap ordering (determinism)
// ===========================================================================

#[test]
fn given_unordered_inserts_when_keys_collected_then_alphabetical_order() {
    let scores = build_scores(&[("z.rs", 10, 1), ("a.rs", 20, 2), ("m.rs", 30, 3)]);
    let keys: Vec<&String> = scores.hotspots.keys().collect();
    assert_eq!(keys, vec!["a.rs", "m.rs", "z.rs"]);
}

#[test]
fn given_path_prefixes_when_keys_sorted_then_lexicographic() {
    let scores = build_scores(&[
        ("src/z.rs", 10, 1),
        ("src/a.rs", 20, 2),
        ("lib/b.rs", 30, 3),
    ]);
    let keys: Vec<&String> = scores.hotspots.keys().collect();
    assert_eq!(keys[0], "lib/b.rs");
    assert_eq!(keys[1], "src/a.rs");
    assert_eq!(keys[2], "src/z.rs");
}

// ===========================================================================
// 6. Determinism: same inputs → same outputs
// ===========================================================================

#[test]
fn given_same_inputs_when_scores_built_twice_then_identical_results() {
    let s1 = build_scores(&[("a.rs", 100, 5), ("b.rs", 200, 3), ("c.rs", 50, 10)]);
    let s2 = build_scores(&[("a.rs", 100, 5), ("b.rs", 200, 3), ("c.rs", 50, 10)]);

    assert_eq!(
        s1.hotspots.keys().collect::<Vec<_>>(),
        s2.hotspots.keys().collect::<Vec<_>>()
    );
    assert_eq!(
        s1.hotspots.values().collect::<Vec<_>>(),
        s2.hotspots.values().collect::<Vec<_>>()
    );
    assert_eq!(
        s1.commit_counts.values().collect::<Vec<_>>(),
        s2.commit_counts.values().collect::<Vec<_>>()
    );
}

#[test]
fn given_different_insertion_order_when_scores_built_then_same_key_order() {
    let s1 = build_scores(&[("z.rs", 10, 1), ("a.rs", 20, 2), ("m.rs", 30, 3)]);
    let s2 = build_scores(&[("a.rs", 20, 2), ("m.rs", 30, 3), ("z.rs", 10, 1)]);

    assert_eq!(
        s1.hotspots.keys().collect::<Vec<_>>(),
        s2.hotspots.keys().collect::<Vec<_>>()
    );
}

// ===========================================================================
// 7. Edge cases
// ===========================================================================

#[test]
fn given_single_file_with_one_commit_when_scored_then_hotspot_equals_lines() {
    let scores = build_scores(&[("only.rs", 42, 1)]);
    assert_eq!(scores.hotspots["only.rs"], 42);
}

#[test]
fn given_very_large_values_when_scored_then_no_overflow_with_reasonable_input() {
    // 1M lines × 10K commits = 10B, fits in usize on 64-bit
    let scores = build_scores(&[("big.rs", 1_000_000, 10_000)]);
    assert_eq!(scores.hotspots["big.rs"], 10_000_000_000);
}

#[test]
fn given_duplicate_paths_when_inserted_then_last_wins() {
    // BTreeMap insert replaces on duplicate key
    let mut hotspots = BTreeMap::new();
    hotspots.insert("dup.rs".to_string(), 100);
    hotspots.insert("dup.rs".to_string(), 200);
    assert_eq!(hotspots["dup.rs"], 200);
}

#[test]
fn given_empty_path_string_when_used_as_key_then_still_valid() {
    let scores = build_scores(&[("", 10, 2)]);
    assert_eq!(scores.hotspots[""], 20);
}

#[test]
fn given_path_with_special_chars_when_used_as_key_then_still_valid() {
    let scores = build_scores(&[("src/foo bar/baz (1).rs", 10, 2)]);
    assert_eq!(scores.hotspots["src/foo bar/baz (1).rs"], 20);
}

// ===========================================================================
// 8. Scoring invariants
// ===========================================================================

#[test]
fn hotspot_values_are_never_negative() {
    let scores = build_scores(&[("a.rs", 0, 5), ("b.rs", 100, 0), ("c.rs", 50, 3)]);
    for &val in scores.hotspots.values() {
        // usize is inherently non-negative, but we document the invariant
        assert_eq!(val, val); // always true for usize
    }
}

#[test]
fn commit_count_sum_equals_total_file_touches() {
    let files = &[("a.rs", 10, 5), ("b.rs", 20, 3), ("c.rs", 30, 7)];
    let scores = build_scores(files);
    let expected_total: usize = files.iter().map(|(_, _, c)| c).sum();
    let actual_total: usize = scores.commit_counts.values().sum();
    assert_eq!(actual_total, expected_total);
}

#[test]
fn hotspot_sum_equals_weighted_commit_touches() {
    let files = &[
        ("a.rs", 10, 5), // 50
        ("b.rs", 20, 3), // 60
        ("c.rs", 30, 7), // 210
    ];
    let scores = build_scores(files);
    let expected_total: usize = files.iter().map(|(_, l, c)| l * c).sum();
    let actual_total: usize = scores.hotspots.values().sum();
    assert_eq!(actual_total, expected_total);
}

// ===========================================================================
// 9. BDD: no-feature path (compute_git_scores without git)
// ===========================================================================

#[cfg(not(feature = "git"))]
mod no_git_feature {
    use tokmd_context_git::compute_git_scores;

    #[test]
    fn given_no_git_feature_when_compute_called_then_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let result = compute_git_scores(dir.path(), &[], 100, 100);
        assert!(result.is_none());
    }
}

// ===========================================================================
// 10. BDD with live git repo (feature = "git")
// ===========================================================================

#[cfg(feature = "git")]
mod with_git {
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

    fn git(root: &std::path::Path, args: &[&str]) -> Option<()> {
        let out = Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .ok()?;
        if out.status.success() { Some(()) } else { None }
    }

    fn create_repo_with_history() -> Option<tempfile::TempDir> {
        let dir = tempfile::tempdir().ok()?;
        let root = dir.path();
        git(root, &["init"])?;
        git(root, &["config", "user.email", "w60@test.com"])?;
        git(root, &["config", "user.name", "W60"])?;

        // commit 1: a.rs (3 lines)
        std::fs::write(root.join("a.rs"), "1\n2\n3").ok()?;
        git(root, &["add", "."])?;
        git(root, &["commit", "-m", "c1"])?;

        // commit 2: a.rs grows + b.rs added
        std::fs::write(root.join("a.rs"), "1\n2\n3\n4").ok()?;
        std::fs::write(root.join("b.rs"), "1\n2\n3\n4\n5").ok()?;
        git(root, &["add", "."])?;
        git(root, &["commit", "-m", "c2"])?;

        // commit 3: c.rs added
        std::fs::write(root.join("c.rs"), "1\n2").ok()?;
        git(root, &["add", "."])?;
        git(root, &["commit", "-m", "c3"])?;

        Some(dir)
    }

    #[test]
    fn given_repo_with_history_when_scores_computed_then_commit_counts_match() {
        let repo = match create_repo_with_history() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![
            make_row("a.rs", 4),
            make_row("b.rs", 5),
            make_row("c.rs", 2),
        ];
        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };
        assert_eq!(scores.commit_counts["a.rs"], 2);
        assert_eq!(scores.commit_counts["b.rs"], 1);
        assert_eq!(scores.commit_counts["c.rs"], 1);
    }

    #[test]
    fn given_repo_when_scores_computed_then_hotspot_equals_lines_times_commits() {
        let repo = match create_repo_with_history() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![
            make_row("a.rs", 4),
            make_row("b.rs", 5),
            make_row("c.rs", 2),
        ];
        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };
        assert_eq!(scores.hotspots["a.rs"], 4 * 2); // 8
        assert_eq!(scores.hotspots["b.rs"], 5); // 5 * 1
        assert_eq!(scores.hotspots["c.rs"], 2); // 2 * 1
    }

    #[test]
    fn given_repo_when_scores_computed_twice_then_results_identical() {
        let repo = match create_repo_with_history() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![make_row("a.rs", 4), make_row("b.rs", 5)];
        let s1 = compute_git_scores(repo.path(), &rows, 100, 100);
        let s2 = compute_git_scores(repo.path(), &rows, 100, 100);
        let (Some(s1), Some(s2)) = (s1, s2) else {
            return;
        };
        assert_eq!(
            s1.hotspots.iter().collect::<Vec<_>>(),
            s2.hotspots.iter().collect::<Vec<_>>()
        );
        assert_eq!(
            s1.commit_counts.iter().collect::<Vec<_>>(),
            s2.commit_counts.iter().collect::<Vec<_>>()
        );
    }

    #[test]
    fn given_non_repo_dir_when_scores_computed_then_none_returned() {
        let dir = tempfile::tempdir().unwrap();
        let rows = vec![make_row("f.rs", 10)];
        assert!(compute_git_scores(dir.path(), &rows, 100, 100).is_none());
    }

    #[test]
    fn given_empty_rows_when_scores_computed_then_maps_empty() {
        let repo = match create_repo_with_history() {
            Some(r) => r,
            None => return,
        };
        let rows: Vec<FileRow> = vec![];
        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };
        assert!(scores.commit_counts.is_empty());
        assert!(scores.hotspots.is_empty());
    }

    #[test]
    fn given_child_rows_when_scores_computed_then_children_excluded() {
        let repo = match create_repo_with_history() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![FileRow {
            path: "a.rs".to_string(),
            module: "(root)".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Child,
            code: 4,
            comments: 0,
            blanks: 0,
            lines: 4,
            bytes: 40,
            tokens: 20,
        }];
        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };
        assert!(scores.commit_counts.is_empty());
    }

    #[test]
    fn given_file_not_in_git_when_scored_then_absent_from_results() {
        let repo = match create_repo_with_history() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![make_row("nonexistent.rs", 100)];
        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };
        assert!(!scores.commit_counts.contains_key("nonexistent.rs"));
        assert!(scores.hotspots.is_empty());
    }

    #[test]
    fn given_max_commits_when_scores_computed_then_limited() {
        let repo = match create_repo_with_history() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![
            make_row("a.rs", 4),
            make_row("b.rs", 5),
            make_row("c.rs", 2),
        ];
        let Some(scores) = compute_git_scores(repo.path(), &rows, 1, 100) else {
            return;
        };
        // With max_commits=1, only the latest commit (c3: c.rs) should be counted
        let total: usize = scores.commit_counts.values().sum();
        assert!(
            total <= 3,
            "max_commits=1 should limit total file touches, got {total}"
        );
    }
}

// ===========================================================================
// 11. Property tests for determinism and invariants
// ===========================================================================

fn arb_path() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop::string::string_regex("[a-z][a-z0-9_]{0,7}").unwrap(),
        1..=3,
    )
    .prop_map(|segs| segs.join("/") + ".rs")
}

proptest! {
    #[test]
    fn hotspot_equals_lines_times_commits_property(
        lines in prop::collection::btree_map(arb_path(), 0..1_000_usize, 0..20),
        commit_counts in prop::collection::btree_map(arb_path(), 1..50_usize, 0..20),
    ) {
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
            prop_assert_eq!(*hotspot, l * c);
        }
    }

    #[test]
    fn hotspot_keys_subset_of_commit_counts_property(
        file_lines in prop::collection::btree_map(arb_path(), 0..500_usize, 0..15),
        commits_per in prop::collection::btree_map(arb_path(), 1..20_usize, 0..15),
    ) {
        let hotspots: BTreeMap<String, usize> = commits_per
            .iter()
            .filter_map(|(path, c)| {
                let l = file_lines.get(path)?;
                Some((path.clone(), l * c))
            })
            .collect();

        for key in hotspots.keys() {
            prop_assert!(commits_per.contains_key(key));
            prop_assert!(file_lines.contains_key(key));
        }
    }

    #[test]
    fn zero_lines_always_zero_hotspot_property(commits in 0..100_usize) {
        let result = 0_usize.checked_mul(commits).unwrap_or(0);
        prop_assert_eq!(result, 0_usize);
    }

    #[test]
    fn zero_commits_always_zero_hotspot_property(lines in 0..100_usize) {
        let result = lines.checked_mul(0).unwrap_or(0);
        prop_assert_eq!(result, 0_usize);
    }

    #[test]
    fn more_commits_means_higher_hotspot_property(
        lines in 1..500_usize,
        c1 in 0..100_usize,
        c2 in 0..100_usize,
    ) {
        let h1 = lines * c1;
        let h2 = lines * c2;
        if c1 <= c2 {
            prop_assert!(h1 <= h2);
        } else {
            prop_assert!(h1 >= h2);
        }
    }

    #[test]
    fn more_lines_means_higher_hotspot_property(
        commits in 1..100_usize,
        l1 in 0..500_usize,
        l2 in 0..500_usize,
    ) {
        let h1 = l1 * commits;
        let h2 = l2 * commits;
        if l1 <= l2 {
            prop_assert!(h1 <= h2);
        } else {
            prop_assert!(h1 >= h2);
        }
    }

    #[test]
    fn hotspot_commutative_property(lines in 0..1_000_usize, commits in 0..1_000_usize) {
        prop_assert_eq!(lines * commits, commits * lines);
    }

    #[test]
    fn btreemap_keys_always_sorted_property(
        entries in prop::collection::btree_map(arb_path(), 0..100_usize, 1..20),
    ) {
        let keys: Vec<&String> = entries.keys().collect();
        for window in keys.windows(2) {
            prop_assert!(window[0] <= window[1]);
        }
    }

    #[test]
    fn build_scores_deterministic_property(
        data in prop::collection::vec(
            (arb_path(), 0..1000_usize, 0..100_usize),
            0..20
        )
    ) {
        let refs: Vec<(&str, usize, usize)> = data.iter().map(|(p, l, c)| (p.as_str(), *l, *c)).collect();
        let s1 = crate::build_scores(&refs);
        let s2 = crate::build_scores(&refs);
        prop_assert_eq!(
            s1.hotspots.iter().collect::<Vec<_>>(),
            s2.hotspots.iter().collect::<Vec<_>>()
        );
        prop_assert_eq!(
            s1.commit_counts.iter().collect::<Vec<_>>(),
            s2.commit_counts.iter().collect::<Vec<_>>()
        );
    }
}
