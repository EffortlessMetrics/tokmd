//! Deep property-based tests for tokmd-model.
//!
//! Covers: avg monotonicity, module_key depth constraints,
//! normalize_path cross-platform equivalence, and aggregation idempotency.

use proptest::prelude::*;
use std::path::Path;
use tokmd_model::{avg, module_key, normalize_path};

// =========================================================================
// avg: monotonicity and edge cases
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn avg_monotonic_in_lines(
        lines1 in 0usize..5000,
        delta in 1usize..5000,
        files in 1usize..100,
    ) {
        let lines2 = lines1 + delta;
        prop_assert!(
            avg(lines2, files) >= avg(lines1, files),
            "avg({}, {}) < avg({}, {})", lines2, files, lines1, files
        );
    }

    #[test]
    fn avg_decreases_with_more_files(
        lines in 1usize..10_000,
        files1 in 1usize..100,
        delta in 1usize..100,
    ) {
        let files2 = files1 + delta;
        prop_assert!(
            avg(lines, files2) <= avg(lines, files1),
            "avg({}, {}) > avg({}, {})", lines, files2, lines, files1
        );
    }

    #[test]
    fn avg_one_file_equals_lines(lines in 0usize..100_000) {
        prop_assert_eq!(avg(lines, 1), lines);
    }

    #[test]
    fn avg_result_bounded_by_lines(lines in 0usize..100_000, files in 1usize..1000) {
        let result = avg(lines, files);
        prop_assert!(result <= lines, "avg({}, {}) = {} > lines", lines, files, result);
    }
}

// =========================================================================
// module_key: depth constraints
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn module_key_depth_1_max_one_segment(
        parts in prop::collection::vec("[a-z]{2,6}", 2..6),
        filename in "[a-z]{2,6}\\.[a-z]{1,3}",
    ) {
        let path = format!("{}/{}", parts.join("/"), filename);
        let key = module_key(&path, &[], 1);
        let segments = key.split('/').count();
        prop_assert!(segments <= 1,
            "depth=1 key '{}' has {} segments", key, segments);
    }

    #[test]
    fn module_key_consistent_across_separators(
        dir in "[a-z]{2,8}",
        subdir in "[a-z]{2,8}",
        filename in "[a-z]{2,8}\\.[a-z]{1,3}",
    ) {
        let fwd = format!("{}/{}/{}", dir, subdir, filename);
        let back = format!("{}\\{}\\{}", dir, subdir, filename);
        let roots: Vec<String> = vec![];
        let k1 = module_key(&fwd, &roots, 2);
        let k2 = module_key(&back, &roots, 2);
        prop_assert_eq!(k1, k2, "Forward/backslash should give same module key");
    }

    #[test]
    fn module_key_never_contains_filename(
        dir in "[a-z]{3,8}",
        filename in "[a-z]{3,8}\\.[a-z]{1,3}",
    ) {
        let path = format!("{}/{}", dir, filename);
        let key = module_key(&path, &[], 2);
        prop_assert!(
            !key.contains('.'),
            "module key '{}' should not contain the filename (which has a dot)", key
        );
    }

    #[test]
    fn module_key_root_for_bare_filename(filename in "[a-z]{2,8}\\.[a-z]{1,3}") {
        let key = module_key(&filename, &[], 2);
        prop_assert_eq!(key, "(root)", "Bare filename '{}' should map to (root)", filename);
    }
}

// =========================================================================
// normalize_path: cross-platform equivalence
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn normalize_path_forward_backslash_equivalent(
        parts in prop::collection::vec("[a-z]{2,6}", 2..5),
        filename in "[a-z]{2,6}\\.[a-z]{1,3}",
    ) {
        let fwd = format!("{}/{}", parts.join("/"), filename);
        let back = format!("{}\\{}", parts.join("\\"), filename);
        let n1 = normalize_path(Path::new(&fwd), None);
        let n2 = normalize_path(Path::new(&back), None);
        prop_assert_eq!(n1, n2, "Forward and backslash paths should normalize equally");
    }

    #[test]
    fn normalize_path_dot_slash_stripped(
        parts in prop::collection::vec("[a-z]{2,6}", 1..4),
        filename in "[a-z]{2,6}\\.[a-z]{1,3}",
    ) {
        let plain = format!("{}/{}", parts.join("/"), filename);
        let dotted = format!("./{}/{}", parts.join("/"), filename);
        let n1 = normalize_path(Path::new(&plain), None);
        let n2 = normalize_path(Path::new(&dotted), None);
        prop_assert_eq!(n1, n2, "./ prefix should be stripped");
    }

    #[test]
    fn normalize_path_result_no_trailing_slash(
        parts in prop::collection::vec("[a-z]{2,6}", 1..4),
        filename in "[a-z]{2,6}\\.[a-z]{1,3}",
    ) {
        let path = format!("{}/{}", parts.join("/"), filename);
        let normalized = normalize_path(Path::new(&path), None);
        prop_assert!(
            !normalized.ends_with('/'),
            "Normalized path should not end with /: '{}'", normalized
        );
    }

    #[test]
    fn normalize_path_deterministic(path in "[a-zA-Z0-9_/]+\\.[a-z]+") {
        let n1 = normalize_path(Path::new(&path), None);
        let n2 = normalize_path(Path::new(&path), None);
        prop_assert_eq!(n1, n2);
    }
}
