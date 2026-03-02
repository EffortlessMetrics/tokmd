//! Unit tests for `tokmd-analysis-util` utility functions and edge cases.

use std::path::{Path, PathBuf};

use tokmd_analysis_util::{
    AnalysisLimits, empty_file_row, gini_coefficient, is_infra_lang, is_test_path, normalize_path,
    normalize_root, now_ms, path_depth, percentile, round_f64, safe_ratio,
};

// ── normalize_path edge cases ───────────────────────────────────────────────

#[test]
fn normalize_path_handles_only_backslashes() {
    let root = PathBuf::from("r");
    assert_eq!(normalize_path(r"\a\b\c", &root), "/a/b/c");
}

#[test]
fn normalize_path_mixed_separators() {
    let root = PathBuf::from("r");
    assert_eq!(normalize_path(r"a/b\c/d\e", &root), "a/b/c/d/e");
}

#[test]
fn normalize_path_dot_slash_only_strips_prefix() {
    let root = PathBuf::from("r");
    // Inner "./" is not stripped, only leading
    let result = normalize_path("./a/./b", &root);
    assert!(!result.starts_with("./"));
    assert!(result.starts_with("a/"));
}

#[test]
fn normalize_path_root_prefix_stripped() {
    let root = PathBuf::from("myrepo");
    assert_eq!(normalize_path("myrepo/src/main.rs", &root), "src/main.rs");
}

#[test]
fn normalize_path_root_no_match_leaves_unchanged() {
    let root = PathBuf::from("other");
    assert_eq!(
        normalize_path("myrepo/src/main.rs", &root),
        "myrepo/src/main.rs"
    );
}

// ── path_depth edge cases ───────────────────────────────────────────────────

#[test]
fn path_depth_root_slash_only() {
    // "/" has no non-empty segments, so max(0, 1) = 1
    assert_eq!(path_depth("/"), 1);
}

#[test]
fn path_depth_many_slashes() {
    assert_eq!(path_depth("///"), 1);
}

#[test]
fn path_depth_single_segment_no_slash() {
    assert_eq!(path_depth("README.md"), 1);
}

#[test]
fn path_depth_three_segments() {
    assert_eq!(path_depth("a/b/c"), 3);
}

// ── is_test_path edge cases ─────────────────────────────────────────────────

#[test]
fn is_test_path_nested_test_dir() {
    assert!(is_test_path("project/src/tests/unit/foo.rs"));
}

#[test]
fn is_test_path_ends_with_test_rs() {
    assert!(is_test_path("crate/src/parser_test.rs"));
}

#[test]
fn is_test_path_starts_with_test_underscore() {
    assert!(is_test_path("test_parser.py"));
}

#[test]
fn is_test_path_false_for_production_code() {
    assert!(!is_test_path("src/lib.rs"));
    assert!(!is_test_path("src/utils/helper.ts"));
    assert!(!is_test_path("main.go"));
}

#[test]
fn is_test_path_spec_file_pattern() {
    assert!(is_test_path("components/button.spec.tsx"));
}

#[test]
fn is_test_path_test_file_pattern_js() {
    assert!(is_test_path("components/button.test.jsx"));
}

// ── is_infra_lang edge cases ────────────────────────────────────────────────

#[test]
fn is_infra_lang_all_known_detected() {
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
        assert!(is_infra_lang(lang), "'{}' should be infra", lang);
    }
}

#[test]
fn is_infra_lang_mixed_case() {
    assert!(is_infra_lang("JSON"));
    assert!(is_infra_lang("Yaml"));
    assert!(is_infra_lang("Makefile"));
    assert!(is_infra_lang("HCL"));
}

#[test]
fn is_infra_lang_rejects_programming_languages() {
    for lang in &[
        "rust", "python", "go", "java", "c", "cpp", "haskell", "elixir",
    ] {
        assert!(!is_infra_lang(lang), "'{}' should NOT be infra", lang);
    }
}

#[test]
fn is_infra_lang_empty_and_whitespace() {
    assert!(!is_infra_lang(""));
    assert!(!is_infra_lang(" "));
    assert!(!is_infra_lang("  json  "));
}

// ── empty_file_row ──────────────────────────────────────────────────────────

#[test]
fn empty_file_row_has_zero_metrics() {
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
fn empty_file_row_optional_fields_are_none() {
    let row = empty_file_row();
    assert!(row.doc_pct.is_none());
    assert!(row.bytes_per_line.is_none());
}

// ── normalize_root ──────────────────────────────────────────────────────────

#[test]
fn normalize_root_nonexistent_returns_input() {
    let fake = Path::new("z:\\nonexistent\\path\\abc");
    let result = normalize_root(fake);
    assert_eq!(result, fake.to_path_buf());
}

#[test]
fn normalize_root_cwd_returns_absolute() {
    let cwd = std::env::current_dir().unwrap();
    let result = normalize_root(&cwd);
    assert!(result.is_absolute());
}

// ── now_ms ──────────────────────────────────────────────────────────────────

#[test]
fn now_ms_returns_reasonable_epoch_millis() {
    let ts = now_ms();
    // Should be well past year 2020 (~1577836800000 ms)
    assert!(ts > 1_577_836_800_000);
}

#[test]
fn now_ms_two_calls_non_decreasing() {
    let a = now_ms();
    let b = now_ms();
    assert!(b >= a);
}

// ── AnalysisLimits ──────────────────────────────────────────────────────────

#[test]
fn analysis_limits_partial_construction() {
    let limits = AnalysisLimits {
        max_files: Some(50),
        max_bytes: None,
        max_file_bytes: Some(10_000),
        ..Default::default()
    };
    assert_eq!(limits.max_files, Some(50));
    assert!(limits.max_bytes.is_none());
    assert_eq!(limits.max_file_bytes, Some(10_000));
    assert!(limits.max_commits.is_none());
    assert!(limits.max_commit_files.is_none());
}

#[test]
fn analysis_limits_debug_is_non_empty() {
    let limits = AnalysisLimits::default();
    let debug = format!("{:?}", limits);
    assert!(!debug.is_empty());
}

// ── Re-exported math: round_f64 ─────────────────────────────────────────────

#[test]
fn round_f64_negative_values() {
    assert_eq!(round_f64(-1.555, 2), -1.56);
    assert_eq!(round_f64(-0.001, 2), 0.0);
}

#[test]
fn round_f64_large_decimals() {
    let val = round_f64(1.123456789, 8);
    assert!((val - 1.12345679).abs() < 1e-10);
}

// ── Re-exported math: safe_ratio ────────────────────────────────────────────

#[test]
fn safe_ratio_large_values() {
    // safe_ratio rounds to 4 decimals, so 999_999/1_000_000 rounds to 1.0
    let r = safe_ratio(999_999, 1_000_000);
    assert_eq!(r, 1.0);
}

#[test]
fn safe_ratio_both_zero() {
    assert_eq!(safe_ratio(0, 0), 0.0);
}

// ── Re-exported math: percentile ────────────────────────────────────────────

#[test]
fn percentile_median_of_odd_count() {
    let vals = [10, 20, 30, 40, 50];
    let med = percentile(&vals, 0.5);
    assert!((med - 30.0).abs() < 1e-10);
}

#[test]
fn percentile_boundary_values() {
    let vals = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    assert_eq!(percentile(&vals, 0.0), 1.0);
    assert_eq!(percentile(&vals, 1.0), 10.0);
}

// ── Re-exported math: gini_coefficient ──────────────────────────────────────

#[test]
fn gini_coefficient_two_elements_extreme() {
    let g = gini_coefficient(&[0, 100]);
    // Maximum inequality for 2 elements: gini = 0.5
    assert!((g - 0.5).abs() < 1e-10);
}

#[test]
fn gini_coefficient_increasing_sequence() {
    let g = gini_coefficient(&[1, 2, 3, 4, 5]);
    assert!(g > 0.0 && g < 1.0, "expected gini in (0,1), got {}", g);
}
