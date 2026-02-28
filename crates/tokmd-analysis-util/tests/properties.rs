//! Property-based tests for `tokmd-analysis-util`.

use std::path::PathBuf;

use proptest::prelude::*;

use tokmd_analysis_util::{
    gini_coefficient, is_infra_lang, is_test_path, normalize_path, path_depth, percentile,
    round_f64, safe_ratio,
};

// ── normalize_path properties ───────────────────────────────────────────────

proptest! {
    #[test]
    fn normalize_path_never_contains_backslash(
        path in r"[a-zA-Z0-9_./\\]{0,80}",
        root in r"[a-zA-Z0-9_]{1,20}"
    ) {
        let result = normalize_path(&path, &PathBuf::from(&root));
        prop_assert!(!result.contains('\\'), "Normalized path should have no backslashes: {}", result);
    }

    #[test]
    fn normalize_path_never_starts_with_dot_slash(
        path in r"[a-zA-Z0-9_./\\]{0,80}",
        root in r"[a-zA-Z0-9_]{1,20}"
    ) {
        let result = normalize_path(&path, &PathBuf::from(&root));
        prop_assert!(!result.starts_with("./"), "Normalized path should not start with ./: {}", result);
    }

    #[test]
    fn normalize_path_is_idempotent(
        path in r"[a-zA-Z0-9_./]{0,60}",
        root in r"[a-zA-Z0-9_]{1,20}"
    ) {
        let root = PathBuf::from(&root);
        let first = normalize_path(&path, &root);
        let second = normalize_path(&first, &root);
        prop_assert_eq!(&first, &second, "normalize_path should be idempotent");
    }

    #[test]
    fn normalize_path_deterministic(
        path in r"[a-zA-Z0-9_./\\]{0,80}",
        root in r"[a-zA-Z0-9_]{1,20}"
    ) {
        let root = PathBuf::from(&root);
        let a = normalize_path(&path, &root);
        let b = normalize_path(&path, &root);
        prop_assert_eq!(a, b, "normalize_path must be deterministic");
    }
}

// ── path_depth properties ───────────────────────────────────────────────────

proptest! {
    #[test]
    fn path_depth_always_at_least_one(path in ".*") {
        prop_assert!(path_depth(&path) >= 1);
    }

    #[test]
    fn path_depth_equals_segment_count(
        segments in prop::collection::vec("[a-zA-Z0-9_]+", 1..10)
    ) {
        let path = segments.join("/");
        prop_assert_eq!(path_depth(&path), segments.len());
    }

    #[test]
    fn path_depth_unaffected_by_trailing_slash(
        segments in prop::collection::vec("[a-zA-Z0-9_]+", 1..8)
    ) {
        let clean = segments.join("/");
        let trailing = format!("{}/", clean);
        prop_assert_eq!(path_depth(&clean), path_depth(&trailing));
    }

    #[test]
    fn path_depth_unaffected_by_leading_slash(
        segments in prop::collection::vec("[a-zA-Z0-9_]+", 1..8)
    ) {
        let clean = segments.join("/");
        let leading = format!("/{}", clean);
        prop_assert_eq!(path_depth(&clean), path_depth(&leading));
    }

    #[test]
    fn path_depth_unaffected_by_double_slashes(
        segments in prop::collection::vec("[a-zA-Z0-9_]+", 1..6)
    ) {
        let clean = segments.join("/");
        let doubled = segments.join("//");
        prop_assert_eq!(path_depth(&clean), path_depth(&doubled));
    }
}

// ── is_test_path properties ─────────────────────────────────────────────────

proptest! {
    #[test]
    fn is_test_path_case_insensitive_for_test_dir(
        prefix in "[a-zA-Z0-9_/]{0,20}",
        suffix in "[a-zA-Z0-9_]+\\.rs"
    ) {
        let lower = format!("{}/test/{}", prefix, suffix);
        let upper = format!("{}/TEST/{}", prefix, suffix);
        prop_assert_eq!(
            is_test_path(&lower),
            is_test_path(&upper),
            "test dir detection should be case-insensitive"
        );
    }

    #[test]
    fn is_test_path_known_dirs_always_detected(
        dir in prop::sample::select(vec!["test", "tests", "__tests__", "spec", "specs"]),
        file in "[a-zA-Z][a-zA-Z0-9_]*\\.rs"
    ) {
        let path = format!("src/{}/{}", dir, file);
        prop_assert!(is_test_path(&path), "Should detect: {}", path);
    }

    #[test]
    fn is_test_path_known_file_patterns_always_detected(
        pattern in prop::sample::select(vec![
            "foo_test.rs", "test_foo.rs", "foo.test.js", "foo.spec.ts", "bar_test.rs"
        ])
    ) {
        let path = format!("src/{}", pattern);
        prop_assert!(is_test_path(&path), "Should detect file pattern: {}", path);
    }
}

// ── is_infra_lang properties ────────────────────────────────────────────────

proptest! {
    #[test]
    fn is_infra_lang_case_insensitive(
        lang in prop::sample::select(vec![
            "json", "yaml", "toml", "markdown", "xml", "html", "css",
            "scss", "less", "makefile", "dockerfile"
        ])
    ) {
        prop_assert!(is_infra_lang(&lang));
        prop_assert!(is_infra_lang(&lang.to_uppercase()));
        // Mixed case
        let mixed: String = lang
            .chars()
            .enumerate()
            .map(|(i, c)| if i % 2 == 0 { c.to_uppercase().next().unwrap() } else { c })
            .collect();
        prop_assert!(is_infra_lang(&mixed), "Mixed case should work: {}", mixed);
    }

    #[test]
    fn is_infra_lang_code_langs_never_infra(
        lang in prop::sample::select(vec![
            "rust", "python", "javascript", "typescript", "go", "java",
            "c", "cpp", "ruby", "swift", "kotlin", "scala", "haskell"
        ])
    ) {
        prop_assert!(!is_infra_lang(&lang));
    }
}

// ── Re-exported math function properties ────────────────────────────────────

proptest! {
    #[test]
    fn round_f64_result_is_finite(value in -1e10f64..1e10f64, decimals in 0u32..10) {
        let result = round_f64(value, decimals);
        prop_assert!(result.is_finite(), "round_f64 should always return finite");
    }

    #[test]
    fn round_f64_zero_is_zero(decimals in 0u32..10) {
        prop_assert_eq!(round_f64(0.0, decimals), 0.0);
    }

    #[test]
    fn safe_ratio_result_is_non_negative(numer in 0usize..10_000, denom in 0usize..10_000) {
        let result = safe_ratio(numer, denom);
        prop_assert!(result >= 0.0, "safe_ratio should never be negative");
    }

    #[test]
    fn safe_ratio_zero_denom_always_zero(numer in 0usize..10_000) {
        prop_assert_eq!(safe_ratio(numer, 0), 0.0);
    }

    #[test]
    fn safe_ratio_identity_when_equal(val in 1usize..10_000) {
        prop_assert_eq!(safe_ratio(val, val), 1.0);
    }

    #[test]
    fn safe_ratio_bounded_when_numer_leq_denom(
        numer in 0usize..10_000,
        denom in 1usize..10_000
    ) {
        prop_assume!(numer <= denom);
        let result = safe_ratio(numer, denom);
        prop_assert!(result <= 1.0, "ratio should be <= 1.0 when numer <= denom");
    }

    #[test]
    fn percentile_empty_always_zero(pct in 0.0f64..1.0) {
        prop_assert_eq!(percentile(&[], pct), 0.0);
    }

    #[test]
    fn percentile_single_returns_that_element(val in 0usize..10_000, pct in 0.0f64..1.0) {
        prop_assert_eq!(percentile(&[val], pct), val as f64);
    }

    #[test]
    fn percentile_result_within_bounds(
        mut values in prop::collection::vec(0usize..1000, 1..50),
        pct in 0.0f64..=1.0
    ) {
        values.sort();
        let result = percentile(&values, pct);
        let min = *values.first().unwrap() as f64;
        let max = *values.last().unwrap() as f64;
        prop_assert!(result >= min && result <= max,
            "percentile should be within [min, max]: {} not in [{}, {}]", result, min, max);
    }

    #[test]
    fn gini_coefficient_empty_is_zero(pct in 0.0f64..1.0) {
        let _ = pct; // unused, just to have a proptest input
        prop_assert_eq!(gini_coefficient(&[]), 0.0);
    }

    #[test]
    fn gini_coefficient_uniform_near_zero(val in 1usize..1000, len in 2usize..20) {
        let uniform: Vec<usize> = vec![val; len];
        let g = gini_coefficient(&uniform);
        prop_assert!(g.abs() < 1e-10, "Uniform distribution gini should be ~0, got {}", g);
    }

    #[test]
    fn gini_coefficient_all_zeros_is_zero(len in 1usize..20) {
        let zeros: Vec<usize> = vec![0; len];
        prop_assert_eq!(gini_coefficient(&zeros), 0.0);
    }

    #[test]
    fn gini_coefficient_bounded(
        mut values in prop::collection::vec(0usize..1000, 2..30)
    ) {
        values.sort();
        let g = gini_coefficient(&values);
        prop_assert!(g >= 0.0 && g <= 1.0,
            "Gini coefficient should be in [0, 1], got {}", g);
    }
}
