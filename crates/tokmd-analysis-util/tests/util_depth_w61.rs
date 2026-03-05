//! W61 depth tests for tokmd-analysis-util: BDD edge cases, determinism, proptest.

use std::path::PathBuf;

use tokmd_analysis_util::{
    AnalysisLimits, empty_file_row, gini_coefficient, is_infra_lang, is_test_path, normalize_path,
    normalize_root, path_depth, percentile, round_f64, safe_ratio,
};

// ---------------------------------------------------------------------------
// BDD: normalize_path edge cases
// ---------------------------------------------------------------------------

#[test]
fn normalize_path_strips_multiple_leading_dot_slashes() {
    let root = PathBuf::from("repo");
    assert_eq!(normalize_path("././src/lib.rs", &root), "src/lib.rs");
}

#[test]
fn normalize_path_converts_backslashes() {
    let root = PathBuf::from("repo");
    assert_eq!(normalize_path(r"src\main\lib.rs", &root), "src/main/lib.rs");
}

#[test]
fn normalize_path_empty_string_returns_empty() {
    let root = PathBuf::from("repo");
    assert_eq!(normalize_path("", &root), "");
}

#[test]
fn normalize_path_plain_filename_unchanged() {
    let root = PathBuf::from("repo");
    assert_eq!(normalize_path("lib.rs", &root), "lib.rs");
}

#[test]
fn normalize_path_only_dot_slash_returns_empty() {
    let root = PathBuf::from("repo");
    // After stripping "./" we get ""
    assert_eq!(normalize_path("./", &root), "");
}

#[test]
fn normalize_path_mixed_separators() {
    let root = PathBuf::from("repo");
    assert_eq!(
        normalize_path(r".\src/main\lib.rs", &root),
        "src/main/lib.rs"
    );
}

// ---------------------------------------------------------------------------
// BDD: path_depth edge cases
// ---------------------------------------------------------------------------

#[test]
fn path_depth_single_file_is_one() {
    assert_eq!(path_depth("lib.rs"), 1);
}

#[test]
fn path_depth_deep_path() {
    assert_eq!(path_depth("a/b/c/d/e"), 5);
}

#[test]
fn path_depth_trailing_slash_ignored() {
    assert_eq!(path_depth("a/b/c/"), 3);
}

#[test]
fn path_depth_leading_slash_ignored() {
    assert_eq!(path_depth("/a/b"), 2);
}

#[test]
fn path_depth_double_slashes_ignored() {
    assert_eq!(path_depth("a//b"), 2);
}

#[test]
fn path_depth_empty_string_returns_one() {
    assert_eq!(path_depth(""), 1);
}

#[test]
fn path_depth_only_slashes_returns_one() {
    assert_eq!(path_depth("///"), 1);
}

// ---------------------------------------------------------------------------
// BDD: is_test_path edge cases
// ---------------------------------------------------------------------------

#[test]
fn is_test_path_with_test_dir() {
    assert!(is_test_path("src/test/foo.rs"));
}

#[test]
fn is_test_path_with_tests_dir() {
    assert!(is_test_path("src/tests/foo.rs"));
}

#[test]
fn is_test_path_with_dunder_tests() {
    assert!(is_test_path("src/__tests__/foo.js"));
}

#[test]
fn is_test_path_with_spec_dir() {
    assert!(is_test_path("src/spec/foo.rb"));
}

#[test]
fn is_test_path_with_specs_dir() {
    assert!(is_test_path("src/specs/foo.rb"));
}

#[test]
fn is_test_path_with_test_prefix_file() {
    assert!(is_test_path("src/test_main.py"));
}

#[test]
fn is_test_path_with_test_suffix_rs() {
    assert!(is_test_path("src/main_test.rs"));
}

#[test]
fn is_test_path_with_dot_test_js() {
    assert!(is_test_path("src/foo.test.js"));
}

#[test]
fn is_test_path_with_dot_spec_ts() {
    assert!(is_test_path("src/foo.spec.ts"));
}

#[test]
fn is_test_path_regular_source_is_false() {
    assert!(!is_test_path("src/main.rs"));
    assert!(!is_test_path("lib/utils.py"));
}

#[test]
fn is_test_path_case_insensitive() {
    assert!(is_test_path("src/TEST/foo.rs"));
    assert!(is_test_path("src/Tests/foo.rs"));
    assert!(is_test_path("src/__TESTS__/foo.js"));
}

// ---------------------------------------------------------------------------
// BDD: is_infra_lang
// ---------------------------------------------------------------------------

#[test]
fn infra_langs_detected_lowercase() {
    let infra = [
        "json",
        "yaml",
        "toml",
        "markdown",
        "xml",
        "html",
        "css",
        "scss",
        "less",
        "makefile",
        "dockerfile",
        "hcl",
        "terraform",
        "nix",
        "cmake",
        "ini",
        "properties",
        "gitignore",
        "gitconfig",
        "editorconfig",
        "csv",
        "tsv",
        "svg",
    ];
    for lang in &infra {
        assert!(is_infra_lang(lang), "{} should be infra", lang);
    }
}

#[test]
fn infra_langs_detected_uppercase() {
    assert!(is_infra_lang("JSON"));
    assert!(is_infra_lang("YAML"));
    assert!(is_infra_lang("TOML"));
}

#[test]
fn code_langs_are_not_infra() {
    let code = [
        "rust",
        "python",
        "javascript",
        "typescript",
        "go",
        "java",
        "c",
        "cpp",
        "ruby",
    ];
    for lang in &code {
        assert!(!is_infra_lang(lang), "{} should NOT be infra", lang);
    }
}

#[test]
fn empty_string_is_not_infra() {
    assert!(!is_infra_lang(""));
}

// ---------------------------------------------------------------------------
// BDD: AnalysisLimits
// ---------------------------------------------------------------------------

#[test]
fn analysis_limits_default_all_none() {
    let lim = AnalysisLimits::default();
    assert!(lim.max_files.is_none());
    assert!(lim.max_bytes.is_none());
    assert!(lim.max_file_bytes.is_none());
    assert!(lim.max_commits.is_none());
    assert!(lim.max_commit_files.is_none());
}

#[test]
fn analysis_limits_can_be_partially_set() {
    let lim = AnalysisLimits {
        max_files: Some(100),
        max_bytes: None,
        max_file_bytes: Some(4096),
        max_commits: None,
        max_commit_files: None,
    };
    assert_eq!(lim.max_files, Some(100));
    assert!(lim.max_bytes.is_none());
    assert_eq!(lim.max_file_bytes, Some(4096));
}

// ---------------------------------------------------------------------------
// BDD: empty_file_row
// ---------------------------------------------------------------------------

#[test]
fn empty_file_row_has_all_zeroes() {
    let row = empty_file_row();
    assert_eq!(row.code, 0);
    assert_eq!(row.comments, 0);
    assert_eq!(row.blanks, 0);
    assert_eq!(row.lines, 0);
    assert_eq!(row.bytes, 0);
    assert_eq!(row.tokens, 0);
    assert_eq!(row.depth, 0);
}

#[test]
fn empty_file_row_has_empty_strings() {
    let row = empty_file_row();
    assert!(row.path.is_empty());
    assert!(row.module.is_empty());
    assert!(row.lang.is_empty());
}

#[test]
fn empty_file_row_has_none_optionals() {
    let row = empty_file_row();
    assert!(row.doc_pct.is_none());
    assert!(row.bytes_per_line.is_none());
}

// ---------------------------------------------------------------------------
// BDD: math helpers (re-exported)
// ---------------------------------------------------------------------------

#[test]
#[allow(clippy::approx_constant)]
fn round_f64_zero_decimals() {
    assert_eq!(round_f64(3.14159, 0), 3.0);
}

#[test]
#[allow(clippy::approx_constant)]
fn round_f64_two_decimals() {
    assert_eq!(round_f64(3.14159, 2), 3.14);
}

#[test]
fn round_f64_negative_value() {
    let r = round_f64(-2.555, 2);
    assert!((r - -2.55).abs() < 0.02, "Expected near -2.55, got {}", r);
}

#[test]
fn round_f64_exact_value_unchanged() {
    assert_eq!(round_f64(1.0, 5), 1.0);
}

#[test]
fn safe_ratio_zero_denominator_returns_zero() {
    assert_eq!(safe_ratio(10, 0), 0.0);
}

#[test]
fn safe_ratio_normal_case() {
    assert!((safe_ratio(1, 2) - 0.5).abs() < f64::EPSILON);
}

#[test]
fn safe_ratio_equal_values_returns_one() {
    assert!((safe_ratio(5, 5) - 1.0).abs() < f64::EPSILON);
}

#[test]
fn gini_coefficient_uniform_is_zero() {
    let data = vec![10, 10, 10, 10];
    assert!((gini_coefficient(&data)).abs() < 1e-9);
}

#[test]
fn gini_coefficient_single_element_is_zero() {
    assert!((gini_coefficient(&[42])).abs() < 1e-9);
}

#[test]
fn gini_coefficient_empty_is_zero() {
    assert_eq!(gini_coefficient(&[]), 0.0);
}

#[test]
fn gini_coefficient_maximal_inequality() {
    // One person has everything, rest have zero
    let data = vec![0, 0, 0, 100];
    let g = gini_coefficient(&data);
    assert!(g > 0.5, "Gini should be high for maximal inequality: {}", g);
}

#[test]
fn percentile_median_of_sorted_list() {
    let data = vec![1, 2, 3, 4, 5];
    let p50 = percentile(&data, 50.0);
    // Must be within the data range
    assert!(
        (1.0..=5.0).contains(&p50),
        "Median should be in range [1,5]: {}",
        p50
    );
}

#[test]
fn percentile_p0_is_minimum() {
    let data = vec![10, 20, 30];
    let p0 = percentile(&data, 0.0);
    assert!((p0 - 10.0).abs() < 1.0, "P0 should be near min: {}", p0);
}

#[test]
fn percentile_p100_is_maximum() {
    let data = vec![10, 20, 30];
    let p100 = percentile(&data, 100.0);
    assert!(
        (p100 - 30.0).abs() < 1.0,
        "P100 should be near max: {}",
        p100
    );
}

// ---------------------------------------------------------------------------
// BDD: normalize_root
// ---------------------------------------------------------------------------

#[test]
fn normalize_root_nonexistent_returns_original() {
    let path = PathBuf::from("this_path_does_not_exist_w61");
    let result = normalize_root(&path);
    assert_eq!(result, path);
}

// ---------------------------------------------------------------------------
// Determinism tests
// ---------------------------------------------------------------------------

#[test]
fn normalize_path_deterministic() {
    let root = PathBuf::from("r");
    let input = r".\a\b\c.rs";
    assert_eq!(normalize_path(input, &root), normalize_path(input, &root));
}

#[test]
fn path_depth_deterministic() {
    let p = "a/b/c/d";
    assert_eq!(path_depth(p), path_depth(p));
}

#[test]
fn is_test_path_deterministic() {
    let p = "src/test/main.rs";
    assert_eq!(is_test_path(p), is_test_path(p));
}

#[test]
fn empty_file_row_deterministic() {
    let a = empty_file_row();
    let b = empty_file_row();
    assert_eq!(a.code, b.code);
    assert_eq!(a.path, b.path);
    assert_eq!(a.depth, b.depth);
}

// ---------------------------------------------------------------------------
// Proptest properties
// ---------------------------------------------------------------------------

mod properties {
    use proptest::prelude::*;
    use std::path::PathBuf;
    use tokmd_analysis_util::{
        is_infra_lang, is_test_path, normalize_path, path_depth, round_f64, safe_ratio,
    };

    proptest! {
        #[test]
        fn path_depth_at_least_one(s in "\\PC{0,100}") {
            prop_assert!(path_depth(&s) >= 1);
        }

        #[test]
        fn normalize_path_no_backslashes(input in "[a-zA-Z0-9_./\\\\]{0,50}") {
            let root = PathBuf::from("root");
            let out = normalize_path(&input, &root);
            prop_assert!(!out.contains('\\'), "Output should have no backslashes: {}", out);
        }

        #[test]
        fn normalize_path_no_leading_dot_slash(input in "[a-zA-Z0-9_./ ]{0,50}") {
            let root = PathBuf::from("root");
            let out = normalize_path(&input, &root);
            prop_assert!(!out.starts_with("./"), "Output should not start with ./: {}", out);
        }

        #[test]
        fn safe_ratio_non_negative(a in 0usize..10000, b in 0usize..10000) {
            let r = safe_ratio(a, b);
            prop_assert!(r >= 0.0, "safe_ratio should be non-negative: {}", r);
        }

        #[test]
        fn round_f64_idempotent(val in -1e6f64..1e6, decimals in 0u32..8) {
            let once = round_f64(val, decimals);
            let twice = round_f64(once, decimals);
            prop_assert!((once - twice).abs() < 1e-10, "round_f64 not idempotent: {} vs {}", once, twice);
        }

        #[test]
        fn is_test_path_does_not_panic(s in "\\PC{0,100}") {
            let _ = is_test_path(&s);
        }

        #[test]
        fn is_infra_lang_does_not_panic(s in "\\PC{0,50}") {
            let _ = is_infra_lang(&s);
        }
    }
}
