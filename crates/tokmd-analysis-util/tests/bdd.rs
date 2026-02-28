//! BDD-style scenario tests for `tokmd-analysis-util` public API.

use std::path::{Path, PathBuf};

use tokmd_analysis_util::{
    AnalysisLimits, empty_file_row, gini_coefficient, is_infra_lang, is_test_path, normalize_path,
    normalize_root, now_ms, path_depth, percentile, round_f64, safe_ratio,
};

// ── AnalysisLimits ──────────────────────────────────────────────────────────

#[test]
fn analysis_limits_default_has_all_none() {
    let limits = AnalysisLimits::default();
    assert!(limits.max_files.is_none());
    assert!(limits.max_bytes.is_none());
    assert!(limits.max_file_bytes.is_none());
    assert!(limits.max_commits.is_none());
    assert!(limits.max_commit_files.is_none());
}

#[test]
fn analysis_limits_can_be_constructed_with_values() {
    let limits = AnalysisLimits {
        max_files: Some(100),
        max_bytes: Some(1_000_000),
        max_file_bytes: Some(50_000),
        max_commits: Some(500),
        max_commit_files: Some(20),
    };
    assert_eq!(limits.max_files, Some(100));
    assert_eq!(limits.max_bytes, Some(1_000_000));
    assert_eq!(limits.max_file_bytes, Some(50_000));
    assert_eq!(limits.max_commits, Some(500));
    assert_eq!(limits.max_commit_files, Some(20));
}

#[test]
fn analysis_limits_clone_produces_equal_copy() {
    let limits = AnalysisLimits {
        max_files: Some(10),
        ..Default::default()
    };
    let cloned = limits.clone();
    assert_eq!(cloned.max_files, Some(10));
    assert!(cloned.max_bytes.is_none());
}

// ── now_ms ──────────────────────────────────────────────────────────────────

#[test]
fn now_ms_returns_positive_timestamp() {
    let ts = now_ms();
    assert!(ts > 0, "Timestamp should be positive");
}

#[test]
fn now_ms_is_monotonically_non_decreasing() {
    let t1 = now_ms();
    let t2 = now_ms();
    assert!(t2 >= t1);
}

// ── normalize_path ──────────────────────────────────────────────────────────

#[test]
fn normalize_path_replaces_backslashes() {
    let root = PathBuf::from("repo");
    assert_eq!(normalize_path(r"src\main.rs", &root), "src/main.rs");
}

#[test]
fn normalize_path_strips_leading_dot_slash() {
    let root = PathBuf::from("repo");
    assert_eq!(normalize_path("./src/lib.rs", &root), "src/lib.rs");
}

#[test]
fn normalize_path_strips_both_backslash_and_dot() {
    let root = PathBuf::from("repo");
    assert_eq!(normalize_path(r".\src\lib.rs", &root), "src/lib.rs");
}

#[test]
fn normalize_path_strips_root_prefix() {
    let root = PathBuf::from("repo");
    assert_eq!(normalize_path("repo/src/lib.rs", &root), "src/lib.rs");
}

#[test]
fn normalize_path_leaves_clean_path_unchanged() {
    let root = PathBuf::from("other");
    assert_eq!(normalize_path("src/lib.rs", &root), "src/lib.rs");
}

#[test]
fn normalize_path_empty_input() {
    let root = PathBuf::from("repo");
    assert_eq!(normalize_path("", &root), "");
}

#[test]
fn normalize_path_just_dot_slash() {
    let root = PathBuf::from("repo");
    assert_eq!(normalize_path("./", &root), "");
}

// ── path_depth ──────────────────────────────────────────────────────────────

#[test]
fn path_depth_single_file_is_one() {
    assert_eq!(path_depth("file.rs"), 1);
}

#[test]
fn path_depth_nested_path() {
    assert_eq!(path_depth("src/main.rs"), 2);
    assert_eq!(path_depth("a/b/c/d.txt"), 4);
}

#[test]
fn path_depth_trailing_slash_ignored() {
    assert_eq!(path_depth("src/"), 1);
}

#[test]
fn path_depth_leading_slash_ignored() {
    assert_eq!(path_depth("/src/lib.rs"), 2);
}

#[test]
fn path_depth_empty_string_returns_one() {
    assert_eq!(path_depth(""), 1);
}

#[test]
fn path_depth_double_slashes_ignored() {
    assert_eq!(path_depth("src//lib.rs"), 2);
}

#[test]
fn path_depth_deeply_nested() {
    assert_eq!(path_depth("a/b/c/d/e/f/g"), 7);
}

// ── is_test_path ────────────────────────────────────────────────────────────

#[test]
fn is_test_path_detects_test_directory() {
    assert!(is_test_path("src/test/foo.rs"));
}

#[test]
fn is_test_path_detects_tests_directory() {
    assert!(is_test_path("src/tests/integration.rs"));
}

#[test]
fn is_test_path_detects_dunder_tests() {
    assert!(is_test_path("src/__tests__/foo.js"));
}

#[test]
fn is_test_path_detects_spec_directory() {
    assert!(is_test_path("src/spec/helper.rb"));
}

#[test]
fn is_test_path_detects_specs_directory() {
    assert!(is_test_path("src/specs/unit.rb"));
}

#[test]
fn is_test_path_detects_suffix_test_rs() {
    assert!(is_test_path("src/foo_test.rs"));
}

#[test]
fn is_test_path_detects_prefix_test_underscore() {
    assert!(is_test_path("src/test_foo.rs"));
}

#[test]
fn is_test_path_detects_dot_test_dot() {
    assert!(is_test_path("src/component.test.js"));
}

#[test]
fn is_test_path_detects_dot_spec_dot() {
    assert!(is_test_path("src/component.spec.ts"));
}

#[test]
fn is_test_path_case_insensitive() {
    assert!(is_test_path("src/TEST/foo.rs"));
    assert!(is_test_path("src/Tests/foo.rs"));
    assert!(is_test_path("src/__TESTS__/foo.js"));
}

#[test]
fn is_test_path_rejects_normal_source() {
    assert!(!is_test_path("src/main.rs"));
    assert!(!is_test_path("lib/utils.js"));
    assert!(!is_test_path("app/model.py"));
}

#[test]
fn is_test_path_rejects_partial_match_in_filename() {
    // "testing" contains "test" but doesn't match patterns exactly
    // The function looks for "_test" or "test_" prefix, so "testing.rs" won't match
    assert!(!is_test_path("src/testing_helpers.rs"));
}

#[test]
fn is_test_path_empty_string() {
    assert!(!is_test_path(""));
}

// ── is_infra_lang ───────────────────────────────────────────────────────────

#[test]
fn is_infra_lang_detects_all_known_infra_languages() {
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
        assert!(is_infra_lang(lang), "Expected '{}' to be infra", lang);
    }
}

#[test]
fn is_infra_lang_case_insensitive() {
    assert!(is_infra_lang("JSON"));
    assert!(is_infra_lang("Yaml"));
    assert!(is_infra_lang("TOML"));
    assert!(is_infra_lang("HtMl"));
}

#[test]
fn is_infra_lang_rejects_code_languages() {
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
        "swift",
        "kotlin",
    ];
    for lang in &code {
        assert!(!is_infra_lang(lang), "Expected '{}' to NOT be infra", lang);
    }
}

#[test]
fn is_infra_lang_empty_string() {
    assert!(!is_infra_lang(""));
}

// ── empty_file_row ──────────────────────────────────────────────────────────

#[test]
fn empty_file_row_all_zeros_and_empty() {
    let row = empty_file_row();
    assert_eq!(row.path, "");
    assert_eq!(row.module, "");
    assert_eq!(row.lang, "");
    assert_eq!(row.code, 0);
    assert_eq!(row.comments, 0);
    assert_eq!(row.blanks, 0);
    assert_eq!(row.lines, 0);
    assert_eq!(row.bytes, 0);
    assert_eq!(row.tokens, 0);
    assert!(row.doc_pct.is_none());
    assert!(row.bytes_per_line.is_none());
    assert_eq!(row.depth, 0);
}

// ── normalize_root ──────────────────────────────────────────────────────────

#[test]
fn normalize_root_returns_path_for_nonexistent_dir() {
    let fake = Path::new("/nonexistent/path/xyz123");
    let result = normalize_root(fake);
    assert_eq!(result, fake.to_path_buf());
}

#[test]
fn normalize_root_returns_canonical_for_existing_dir() {
    let cwd = std::env::current_dir().unwrap();
    let result = normalize_root(&cwd);
    // Should be a valid absolute path
    assert!(result.is_absolute());
}

// ── Re-exported math functions ──────────────────────────────────────────────

#[test]
fn round_f64_basic() {
    assert_eq!(round_f64(1.2345, 2), 1.23);
    assert_eq!(round_f64(1.2355, 2), 1.24);
    assert_eq!(round_f64(0.0, 4), 0.0);
}

#[test]
fn round_f64_zero_decimals() {
    assert_eq!(round_f64(3.7, 0), 4.0);
    assert_eq!(round_f64(3.2, 0), 3.0);
}

#[test]
fn safe_ratio_zero_denominator() {
    assert_eq!(safe_ratio(42, 0), 0.0);
}

#[test]
fn safe_ratio_normal() {
    assert_eq!(safe_ratio(1, 2), 0.5);
    assert_eq!(safe_ratio(1, 3), 0.3333);
    assert_eq!(safe_ratio(0, 5), 0.0);
}

#[test]
fn safe_ratio_identity() {
    assert_eq!(safe_ratio(10, 10), 1.0);
}

#[test]
fn percentile_empty_slice() {
    assert_eq!(percentile(&[], 0.5), 0.0);
}

#[test]
fn percentile_single_element() {
    assert_eq!(percentile(&[42], 0.0), 42.0);
    assert_eq!(percentile(&[42], 0.5), 42.0);
    assert_eq!(percentile(&[42], 1.0), 42.0);
}

#[test]
fn percentile_sorted_values() {
    let vals = [10, 20, 30, 40, 50];
    assert_eq!(percentile(&vals, 0.0), 10.0);
    assert_eq!(percentile(&vals, 1.0), 50.0);
}

#[test]
fn gini_coefficient_empty() {
    assert_eq!(gini_coefficient(&[]), 0.0);
}

#[test]
fn gini_coefficient_uniform() {
    let val = gini_coefficient(&[5, 5, 5, 5]);
    assert!(
        val.abs() < 1e-10,
        "Uniform distribution should have gini ~0"
    );
}

#[test]
fn gini_coefficient_all_zeros() {
    assert_eq!(gini_coefficient(&[0, 0, 0]), 0.0);
}

#[test]
fn gini_coefficient_single_element() {
    assert_eq!(gini_coefficient(&[100]), 0.0);
}

#[test]
fn gini_coefficient_skewed() {
    let val = gini_coefficient(&[0, 0, 0, 100]);
    assert!(
        val > 0.5,
        "Highly skewed distribution should have high gini"
    );
}
