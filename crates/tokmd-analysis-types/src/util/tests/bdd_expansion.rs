//! Expanded BDD-style tests for analysis types util module.
//!
//! Covers edge cases and boundary values for utility functions: normalize_path,
//! path_depth, is_test_path, is_infra_lang, math re-exports, and empty_file_row.

use std::path::PathBuf;

use crate::{
    empty_file_row, gini_coefficient, is_infra_lang, is_test_path, normalize_path, path_depth,
    percentile, round_f64, safe_ratio,
};

// ── normalize_path boundary scenarios ───────────────────────────────────

mod normalize_path_boundaries {
    use super::*;

    #[test]
    fn given_double_dot_slash_prefix_then_not_stripped() {
        let root = PathBuf::from("repo");
        // "../" is not a "./" prefix, so it should be left alone
        let result = normalize_path("../other/lib.rs", &root);
        assert!(result.contains("../other/lib.rs") || result.starts_with("../"));
    }

    #[test]
    fn given_path_equal_to_root_then_empty_string() {
        let root = PathBuf::from("myrepo");
        let result = normalize_path("myrepo", &root);
        assert_eq!(result, "");
    }

    #[test]
    fn given_unicode_path_then_normalized_without_crash() {
        let root = PathBuf::from("repo");
        let result = normalize_path("src/日本語/ファイル.rs", &root);
        assert!(!result.contains('\\'));
        assert!(result.contains("日本語"));
    }

    #[test]
    fn given_path_with_consecutive_backslashes_then_normalized_to_forward() {
        let root = PathBuf::from("repo");
        let result = normalize_path(r"src\\nested\\file.rs", &root);
        assert!(!result.contains('\\'));
        assert!(result.contains("src//nested//file.rs") || result.contains("src/"));
    }

    #[test]
    fn given_only_dot_then_returns_dot_or_empty() {
        let root = PathBuf::from("repo");
        let result = normalize_path(".", &root);
        // "." may be left as-is if not matching "./"
        assert!(result == "." || result.is_empty());
    }
}

// ── path_depth boundary scenarios ───────────────────────────────────────

mod path_depth_boundaries {
    use super::*;

    #[test]
    fn given_path_with_only_dots_then_counts_as_segments() {
        // "." and ".." are non-empty segments
        assert_eq!(path_depth("./.."), 2);
    }

    #[test]
    fn given_very_deep_path_then_correct_count() {
        let parts: Vec<&str> = (0..100).map(|_| "dir").collect();
        let deep = parts.join("/");
        assert_eq!(path_depth(&deep), 100);
    }

    #[test]
    fn given_path_with_mixed_separators_then_only_forward_slash_counted() {
        // path_depth splits on '/' only; backslashes are not separators
        let depth = path_depth(r"a\b\c");
        // "a\b\c" is a single segment when splitting on '/'
        assert_eq!(depth, 1);
    }

    #[test]
    fn given_single_slash_then_depth_is_one() {
        assert_eq!(path_depth("/"), 1);
    }

    #[test]
    fn given_path_with_spaces_then_counted_correctly() {
        assert_eq!(path_depth("my dir/sub dir/file.txt"), 3);
    }
}

// ── is_test_path additional edge cases ──────────────────────────────────

mod is_test_path_edge_cases {
    use super::*;

    #[test]
    fn given_path_with_test_only_in_middle_segment_then_detected() {
        assert!(is_test_path("app/test/helpers/setup.py"));
    }

    #[test]
    fn given_file_ending_with_test_dot_rs_then_detected() {
        assert!(is_test_path("src/parser_test.rs"));
    }

    #[test]
    fn given_file_with_test_in_directory_name_but_not_test_dir_then_not_detected() {
        // "testing/" is not a recognized test directory
        assert!(!is_test_path("testing/helper.rs"));
    }

    #[test]
    fn given_file_named_test_without_underscore_then_not_detected_as_file_pattern() {
        // "test.rs" doesn't match _test or test_ patterns, but may match via
        // dir patterns if "test" is in path. As a standalone basename, check behavior.
        // "test.rs" → name = "test.rs", doesn't start with "test_" or contain "_test"
        // or ".test." or ".spec."
        let result = is_test_path("test.rs");
        // The function checks name.starts_with("test_") — "test.rs" does not start with "test_"
        assert!(!result);
    }

    #[test]
    fn given_deeply_nested_spec_dir_then_detected() {
        assert!(is_test_path("packages/core/spec/unit/model_spec.rb"));
    }

    #[test]
    fn given_dunder_tests_in_python_project_then_detected() {
        assert!(is_test_path("src/components/__tests__/Button.test.jsx"));
    }
}

// ── is_infra_lang additional edge cases ─────────────────────────────────

mod is_infra_lang_edge_cases {
    use super::*;

    #[test]
    fn given_lang_with_leading_or_trailing_whitespace_then_not_detected() {
        assert!(!is_infra_lang(" json"));
        assert!(!is_infra_lang("json "));
        assert!(!is_infra_lang(" json "));
    }

    #[test]
    fn given_similar_but_non_matching_lang_then_not_detected() {
        assert!(!is_infra_lang("jsonl"));
        assert!(!is_infra_lang("yml")); // only "yaml" matches
        assert!(!is_infra_lang("htm")); // only "html" matches
    }

    #[test]
    fn given_all_uppercase_infra_lang_then_detected() {
        assert!(is_infra_lang("DOCKERFILE"));
        assert!(is_infra_lang("MAKEFILE"));
        assert!(is_infra_lang("TERRAFORM"));
    }

    #[test]
    fn given_shell_scripting_lang_then_not_infra() {
        assert!(!is_infra_lang("bash"));
        assert!(!is_infra_lang("sh"));
        assert!(!is_infra_lang("powershell"));
    }
}

// ── Math function edge cases ────────────────────────────────────────────

mod math_edge_cases {
    use super::*;

    #[test]
    fn given_round_f64_with_very_small_number_then_rounds_correctly() {
        assert_eq!(round_f64(0.000001, 4), 0.0);
        assert_eq!(round_f64(0.000001, 6), 0.000001);
    }

    #[test]
    fn given_safe_ratio_with_numerator_larger_than_denominator_then_exceeds_one() {
        let r = safe_ratio(10, 3);
        assert!(r > 1.0);
    }

    #[test]
    fn given_percentile_with_two_elements_at_median_then_interpolated() {
        let vals = [10, 20];
        let med = percentile(&vals, 0.5);
        // Should be between 10 and 20
        assert!((10.0..=20.0).contains(&med));
    }

    #[test]
    fn given_gini_with_single_nonzero_in_many_zeros_then_high_inequality() {
        let g = gini_coefficient(&[0, 0, 0, 0, 100]);
        assert!(
            g > 0.7,
            "Expected high gini for extreme inequality, got {g}"
        );
    }

    #[test]
    fn given_gini_with_two_equal_values_then_zero() {
        let g = gini_coefficient(&[50, 50]);
        assert!(
            g.abs() < 1e-10,
            "Expected gini ~0 for equal values, got {g}"
        );
    }
}

// ── empty_file_row validation ───────────────────────────────────────────

mod empty_file_row_scenarios {
    use super::*;

    #[test]
    fn given_empty_file_row_then_all_numeric_fields_sum_to_zero() {
        let row = empty_file_row();
        let total = row.code + row.comments + row.blanks + row.lines;
        assert_eq!(total, 0);
    }

    #[test]
    fn given_empty_file_row_then_tokens_and_bytes_are_zero() {
        let row = empty_file_row();
        assert_eq!(row.tokens, 0);
        assert_eq!(row.bytes, 0);
    }
}
