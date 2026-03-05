//! Deep tests for tokmd-analysis-util (w70 wave).
//!
//! ~20 tests covering normalize_path, path_depth, is_test_path,
//! is_infra_lang, empty_file_row, AnalysisLimits, re-exported math helpers,
//! and determinism.

use std::path::PathBuf;

use tokmd_analysis_util::{
    AnalysisLimits, empty_file_row, gini_coefficient, is_infra_lang, is_test_path, normalize_path,
    path_depth, percentile, round_f64, safe_ratio,
};

// -- normalize_path --

#[test]
fn normalize_path_strips_root_prefix() {
    let root = PathBuf::from("myrepo");
    assert_eq!(normalize_path("myrepo/src/main.rs", &root), "src/main.rs");
}

#[test]
fn normalize_path_backslash_then_root_strip() {
    let root = PathBuf::from("repo");
    assert_eq!(normalize_path(r"repo\src\lib.rs", &root), "src/lib.rs");
}

#[test]
fn normalize_path_multiple_leading_dot_slash() {
    let root = PathBuf::from("x");
    assert_eq!(normalize_path("././src/a.rs", &root), "src/a.rs");
}

#[test]
fn normalize_path_no_match_root_passthrough() {
    let root = PathBuf::from("other");
    assert_eq!(normalize_path("src/lib.rs", &root), "src/lib.rs");
}

// -- path_depth --

#[test]
fn path_depth_single_segment() {
    assert_eq!(path_depth("lib.rs"), 1);
}

#[test]
fn path_depth_nested() {
    assert_eq!(path_depth("a/b/c/d.rs"), 4);
}

#[test]
fn path_depth_trailing_slash_ignored() {
    assert_eq!(path_depth("a/b/"), 2);
}

#[test]
fn path_depth_empty_string_returns_one() {
    assert_eq!(path_depth(""), 1);
}

// -- is_test_path --

#[test]
fn test_path_detects_tests_dir() {
    assert!(is_test_path("src/tests/unit.rs"));
}

#[test]
fn test_path_detects_test_suffix() {
    assert!(is_test_path("src/foo_test.rs"));
}

#[test]
fn test_path_detects_spec_dir() {
    assert!(is_test_path("app/spec/models/user_spec.rb"));
}

#[test]
fn test_path_detects_dunder_tests() {
    assert!(is_test_path("src/__tests__/App.test.js"));
}

#[test]
fn test_path_non_test_returns_false() {
    assert!(!is_test_path("src/lib.rs"));
    assert!(!is_test_path("src/main.rs"));
}

// -- is_infra_lang --

#[test]
fn infra_lang_recognises_all_known() {
    let known = [
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
    for lang in &known {
        assert!(is_infra_lang(lang), "Expected infra: {lang}");
    }
}

#[test]
fn infra_lang_rejects_code_languages() {
    for lang in &["rust", "python", "go", "java", "typescript", "c", "cpp"] {
        assert!(!is_infra_lang(lang), "Should not be infra: {lang}");
    }
}

#[test]
fn infra_lang_case_insensitive() {
    assert!(is_infra_lang("JSON"));
    assert!(is_infra_lang("Yaml"));
    assert!(is_infra_lang("TOML"));
}

// -- empty_file_row --

#[test]
fn empty_file_row_all_zeros_and_empty_strings() {
    let row = empty_file_row();
    assert!(row.path.is_empty());
    assert!(row.module.is_empty());
    assert!(row.lang.is_empty());
    assert_eq!(row.code, 0);
    assert_eq!(row.comments, 0);
    assert_eq!(row.blanks, 0);
    assert_eq!(row.lines, 0);
    assert_eq!(row.bytes, 0);
    assert_eq!(row.tokens, 0);
    assert_eq!(row.depth, 0);
    assert!(row.doc_pct.is_none());
    assert!(row.bytes_per_line.is_none());
}

// -- AnalysisLimits defaults --

#[test]
fn analysis_limits_default_all_none() {
    let limits = AnalysisLimits::default();
    assert!(limits.max_files.is_none());
    assert!(limits.max_bytes.is_none());
    assert!(limits.max_file_bytes.is_none());
    assert!(limits.max_commits.is_none());
    assert!(limits.max_commit_files.is_none());
}

// -- Re-exported math helpers --

#[test]
#[allow(clippy::approx_constant)]
fn round_f64_basic() {
    assert!((round_f64(3.14159, 2) - 3.14).abs() < f64::EPSILON);
    assert!((round_f64(2.005, 2) - 2.01).abs() < 0.001);
}

#[test]
fn round_f64_zero_decimals() {
    assert!((round_f64(9.9, 0) - 10.0).abs() < f64::EPSILON);
}

#[test]
fn safe_ratio_zero_denominator() {
    assert!((safe_ratio(10, 0) - 0.0).abs() < f64::EPSILON);
}

#[test]
fn safe_ratio_normal() {
    assert!((safe_ratio(1, 4) - 0.25).abs() < f64::EPSILON);
}

#[test]
fn percentile_median_of_five() {
    let data = vec![1, 2, 3, 4, 5];
    let p50 = percentile(&data, 0.5);
    assert!((p50 - 3.0).abs() < f64::EPSILON);
}

#[test]
fn percentile_empty_returns_zero() {
    let data: Vec<usize> = vec![];
    let p50 = percentile(&data, 0.5);
    assert!((p50 - 0.0).abs() < f64::EPSILON);
}

#[test]
fn gini_uniform_distribution_is_zero() {
    let data = vec![10, 10, 10, 10];
    assert!((gini_coefficient(&data) - 0.0).abs() < f64::EPSILON);
}

#[test]
fn gini_maximum_inequality() {
    let data = vec![0, 0, 0, 100];
    let g = gini_coefficient(&data);
    assert!(g > 0.5, "Gini={g} expected > 0.5 for max inequality");
}

// -- Determinism --

#[test]
fn normalize_path_deterministic() {
    let root = PathBuf::from("repo");
    let input = r".\src\main.rs";
    let a = normalize_path(input, &root);
    let b = normalize_path(input, &root);
    assert_eq!(a, b);
}

#[test]
fn path_depth_deterministic() {
    assert_eq!(path_depth("a/b/c"), path_depth("a/b/c"));
}
