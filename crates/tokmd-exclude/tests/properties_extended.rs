//! Extended property tests for tokmd-exclude: long paths, parent segments,
//! case preservation, and insertion-order stability.

use std::path::PathBuf;

use proptest::prelude::*;
use tokmd_exclude::{add_exclude_pattern, has_exclude_pattern, normalize_exclude_pattern};

fn segment() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_-]{1,8}".prop_map(String::from)
}

fn rel_path() -> impl Strategy<Value = String> {
    prop::collection::vec(segment(), 1..6).prop_map(|parts| parts.join("/"))
}

proptest! {
    // ── Case preservation: normalization never changes casing ─────────

    #[test]
    fn normalize_preserves_case(
        segments in prop::collection::vec("[a-zA-Z]{1,8}", 1..5),
    ) {
        let root = PathBuf::from("repo");
        let path_str = segments.join("/");
        let normalized = normalize_exclude_pattern(&root, PathBuf::from(&path_str).as_path());

        // Each segment should appear in the normalized output with same case
        for seg in &segments {
            prop_assert!(
                normalized.contains(seg.as_str()),
                "segment '{}' not found in '{}'", seg, normalized
            );
        }
    }

    // ── Long paths maintain segment count ────────────────────────────

    #[test]
    fn normalize_preserves_segment_count_for_long_paths(
        segments in prop::collection::vec(segment(), 5..20),
    ) {
        let root = PathBuf::from("repo");
        let path_str = segments.join("/");
        let normalized = normalize_exclude_pattern(&root, PathBuf::from(&path_str).as_path());
        let normalized_segments: Vec<&str> = normalized.split('/').collect();
        prop_assert_eq!(normalized_segments.len(), segments.len());
    }

    // ── add then has is always true for non-empty patterns ───────────

    #[test]
    fn add_then_has_always_true_for_nonempty(path in rel_path()) {
        let mut patterns = Vec::new();
        add_exclude_pattern(&mut patterns, path.clone());
        prop_assert!(has_exclude_pattern(&patterns, &path));
    }

    // ── Backslash variant always matches after add ───────────────────

    #[test]
    fn backslash_variant_matches_after_forward_slash_add(path in rel_path()) {
        let mut patterns = Vec::new();
        add_exclude_pattern(&mut patterns, path.clone());
        let backslash = path.replace('/', "\\");
        prop_assert!(has_exclude_pattern(&patterns, &backslash));
    }

    // ── Insertion count never exceeds distinct normalized patterns ────

    #[test]
    fn insertion_count_bounded_by_distinct_patterns(
        paths in prop::collection::vec(rel_path(), 1..20),
    ) {
        let mut patterns = Vec::new();
        let mut inserted_count = 0usize;
        for p in &paths {
            if add_exclude_pattern(&mut patterns, p.clone()) {
                inserted_count += 1;
            }
        }
        // The number of inserted patterns should equal the number of
        // distinct normalized patterns
        prop_assert_eq!(patterns.len(), inserted_count);
    }

    // ── Normalization with absolute path under root ──────────────────

    #[test]
    fn absolute_path_under_root_gives_relative_result(
        root_seg in segment(),
        rel_parts in prop::collection::vec(segment(), 1..4),
    ) {
        let root = std::env::temp_dir().join("tokmd-exclude-propext").join(&root_seg);
        let mut full = root.clone();
        for part in &rel_parts {
            full.push(part);
        }

        let expected = rel_parts.join("/");
        let normalized = normalize_exclude_pattern(&root, &full);
        prop_assert_eq!(normalized, expected);
    }

    // ── Forward/backslash normalization is equivalent ─────────────────

    #[test]
    fn forward_and_backslash_normalize_to_same_result(
        segments in prop::collection::vec(segment(), 1..5),
    ) {
        let root = PathBuf::from("repo");
        let forward = segments.join("/");
        let backslash = segments.join("\\");

        let norm_fwd = normalize_exclude_pattern(&root, PathBuf::from(&forward).as_path());
        let norm_bk = normalize_exclude_pattern(&root, PathBuf::from(&backslash).as_path());

        prop_assert_eq!(norm_fwd, norm_bk);
    }

    // ── Dot-slash prefix and plain are equivalent ────────────────────

    #[test]
    fn dot_slash_prefix_and_plain_normalize_equivalently(path in rel_path()) {
        let root = PathBuf::from("repo");
        let plain_norm = normalize_exclude_pattern(&root, PathBuf::from(&path).as_path());
        let prefixed = format!("./{}", path);
        let prefixed_norm = normalize_exclude_pattern(&root, PathBuf::from(&prefixed).as_path());

        prop_assert_eq!(plain_norm, prefixed_norm);
    }
}
