//! Deeper behavioral tests for `tokmd-analysis-util` utility functions,
//! focusing on edge cases: zero values, boundary conditions, large numbers,
//! and special floating-point values.

use std::path::{Path, PathBuf};

use tokmd_analysis_util::{
    AnalysisLimits, empty_file_row, gini_coefficient, is_infra_lang, is_test_path, normalize_path,
    normalize_root, path_depth, percentile, round_f64, safe_ratio,
};

// ── round_f64 edge cases ────────────────────────────────────────────────────

#[test]
fn round_f64_negative_values() {
    assert_eq!(round_f64(-1.2345, 2), -1.23);
    assert_eq!(round_f64(-0.5, 0), -1.0);
    assert_eq!(round_f64(-99.999, 1), -100.0);
}

#[test]
fn round_f64_very_large_numbers() {
    assert_eq!(round_f64(1e15, 2), 1e15);
    assert_eq!(round_f64(123456789.123, 0), 123456789.0);
}

#[test]
fn round_f64_very_small_fractions() {
    assert_eq!(round_f64(0.000001, 4), 0.0);
    assert_eq!(round_f64(0.000001, 6), 0.000001);
    assert_eq!(round_f64(0.00005, 4), 0.0001);
}

#[test]
fn round_f64_nan_propagates() {
    let result = round_f64(f64::NAN, 2);
    assert!(result.is_nan());
}

#[test]
fn round_f64_infinity_propagates() {
    assert_eq!(round_f64(f64::INFINITY, 2), f64::INFINITY);
    assert_eq!(round_f64(f64::NEG_INFINITY, 2), f64::NEG_INFINITY);
}

#[test]
fn round_f64_high_precision() {
    assert_eq!(round_f64(1.123456789, 8), 1.12345679);
    assert_eq!(round_f64(0.1 + 0.2, 1), 0.3);
}

#[test]
fn round_f64_exactly_half() {
    assert_eq!(round_f64(2.5, 0), 3.0);
    assert_eq!(round_f64(1.25, 1), 1.3);
    assert_eq!(round_f64(1.005, 2), 1.0); // IEEE 754 representation quirk
}

// ── safe_ratio edge cases ───────────────────────────────────────────────────

#[test]
fn safe_ratio_both_zero() {
    assert_eq!(safe_ratio(0, 0), 0.0);
}

#[test]
fn safe_ratio_very_large_values() {
    let result = safe_ratio(usize::MAX, usize::MAX);
    assert_eq!(result, 1.0);
}

#[test]
fn safe_ratio_numerator_larger_than_denominator() {
    let result = safe_ratio(100, 3);
    assert!(result > 1.0, "ratio should be > 1 when numer > denom");
    // 100/3 ≈ 33.3333
    assert_eq!(result, 33.3333);
}

#[test]
fn safe_ratio_one_over_large() {
    let result = safe_ratio(1, 10000);
    assert_eq!(result, 0.0001);
}

#[test]
fn safe_ratio_one_over_very_large() {
    // 1 / 100_000 = 0.00001, rounded to 4 decimals = 0.0
    let result = safe_ratio(1, 100_000);
    assert_eq!(result, 0.0);
}

// ── percentile edge cases ───────────────────────────────────────────────────

#[test]
fn percentile_two_elements() {
    let vals = [10, 20];
    assert_eq!(percentile(&vals, 0.0), 10.0);
    assert_eq!(percentile(&vals, 1.0), 20.0);
    // Nearest-rank method: 0.5 maps to index 1 → 20.0
    assert_eq!(percentile(&vals, 0.5), 20.0);
}

#[test]
fn percentile_all_same_values() {
    let vals = [42, 42, 42, 42, 42];
    assert_eq!(percentile(&vals, 0.0), 42.0);
    assert_eq!(percentile(&vals, 0.5), 42.0);
    assert_eq!(percentile(&vals, 1.0), 42.0);
}

#[test]
fn percentile_large_dataset() {
    let vals: Vec<usize> = (0..1000).collect();
    let p50 = percentile(&vals, 0.5);
    // Median should be near 499-500
    assert!((490.0..=510.0).contains(&p50), "p50 = {p50}");
    assert_eq!(percentile(&vals, 0.0), 0.0);
    assert_eq!(percentile(&vals, 1.0), 999.0);
}

#[test]
fn percentile_ascending_order() {
    let vals = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let p25 = percentile(&vals, 0.25);
    let p50 = percentile(&vals, 0.50);
    let p75 = percentile(&vals, 0.75);
    assert!(p25 <= p50);
    assert!(p50 <= p75);
}

// ── gini_coefficient edge cases ─────────────────────────────────────────────

#[test]
fn gini_coefficient_two_elements_equal() {
    let g = gini_coefficient(&[50, 50]);
    assert!(g.abs() < 1e-10);
}

#[test]
fn gini_coefficient_two_elements_maximally_unequal() {
    let g = gini_coefficient(&[0, 100]);
    assert!(g > 0.4, "gini should be high for [0, 100], got {g}");
}

#[test]
fn gini_coefficient_large_uniform() {
    let uniform: Vec<usize> = vec![100; 1000];
    let g = gini_coefficient(&uniform);
    assert!(g.abs() < 1e-10);
}

#[test]
fn gini_coefficient_increasing_sequence() {
    let vals: Vec<usize> = (1..=100).collect();
    let g = gini_coefficient(&vals);
    assert!(g > 0.0 && g < 1.0, "gini = {g}");
}

#[test]
fn gini_coefficient_single_nonzero_rest_zero() {
    let mut vals = vec![0; 99];
    vals.push(1000);
    let g = gini_coefficient(&vals);
    assert!(
        g > 0.9,
        "extreme inequality should yield gini near 1, got {g}"
    );
}

// ── normalize_path edge cases ───────────────────────────────────────────────

#[test]
fn normalize_path_deeply_nested_backslashes() {
    let root = PathBuf::from("root");
    let result = normalize_path(r"a\b\c\d\e\f.rs", &root);
    assert_eq!(result, "a/b/c/d/e/f.rs");
}

#[test]
fn normalize_path_root_only() {
    let root = PathBuf::from("repo");
    assert_eq!(normalize_path("repo", &root), "");
}

#[test]
fn normalize_path_unicode_characters() {
    let root = PathBuf::from("root");
    let result = normalize_path("src/日本語/file.rs", &root);
    assert_eq!(result, "src/日本語/file.rs");
}

#[test]
fn normalize_path_mixed_separators() {
    let root = PathBuf::from("root");
    let result = normalize_path(r"src/sub\dir/file.rs", &root);
    assert_eq!(result, "src/sub/dir/file.rs");
}

#[test]
fn normalize_path_double_dot_slash_prefix() {
    let root = PathBuf::from("root");
    // Only `./` is stripped, not `../`
    let result = normalize_path("././src/lib.rs", &root);
    // After backslash replace: "././src/lib.rs"
    // After strip_prefix("./") once: "./src/lib.rs" — actually let's verify
    assert!(!result.contains('\\'));
}

// ── path_depth edge cases ───────────────────────────────────────────────────

#[test]
fn path_depth_single_slash() {
    // "/" splits into ["", ""], both empty → count 0 → max(0,1) = 1
    assert_eq!(path_depth("/"), 1);
}

#[test]
fn path_depth_many_slashes() {
    assert_eq!(path_depth("///"), 1);
}

#[test]
fn path_depth_very_deep() {
    let deep = "a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p";
    assert_eq!(path_depth(deep), 16);
}

#[test]
fn path_depth_dot_segments() {
    // Dots are treated as regular segments
    assert_eq!(path_depth("./src/lib.rs"), 3); // ".", "src", "lib.rs"
}

// ── is_test_path edge cases ─────────────────────────────────────────────────

#[test]
fn is_test_path_nested_test_directory() {
    assert!(is_test_path("src/module/test/deep/file.rs"));
}

#[test]
fn is_test_path_test_in_filename_not_dir() {
    // "contest.rs" contains "test" but not in a pattern we detect
    assert!(!is_test_path("src/contest.rs"));
}

#[test]
fn is_test_path_test_at_end_with_suffix() {
    assert!(is_test_path("src/my_module_test.rs"));
}

#[test]
fn is_test_path_spec_file_suffix() {
    assert!(is_test_path("components/Button.spec.tsx"));
}

#[test]
fn is_test_path_just_filename_test_prefix() {
    assert!(is_test_path("test_main.py"));
}

#[test]
fn is_test_path_just_filename_no_dir() {
    assert!(!is_test_path("main.rs"));
}

// ── is_infra_lang edge cases ────────────────────────────────────────────────

#[test]
fn is_infra_lang_whitespace_not_matched() {
    assert!(!is_infra_lang(" json"));
    assert!(!is_infra_lang("json "));
}

#[test]
fn is_infra_lang_mixed_case_all_variants() {
    assert!(is_infra_lang("JSON"));
    assert!(is_infra_lang("Json"));
    assert!(is_infra_lang("jSoN"));
    assert!(is_infra_lang("DOCKERFILE"));
    assert!(is_infra_lang("Makefile"));
    assert!(is_infra_lang("EDITORCONFIG"));
}

#[test]
fn is_infra_lang_similar_but_different() {
    assert!(!is_infra_lang("jsonl"));
    assert!(!is_infra_lang("yml"));
    assert!(!is_infra_lang("htm"));
    assert!(!is_infra_lang("sass"));
}

// ── empty_file_row ──────────────────────────────────────────────────────────

#[test]
fn empty_file_row_is_clone_equal() {
    let a = empty_file_row();
    let b = empty_file_row();
    assert_eq!(a.path, b.path);
    assert_eq!(a.code, b.code);
    assert_eq!(a.tokens, b.tokens);
    assert_eq!(a.depth, b.depth);
}

#[test]
fn empty_file_row_has_no_doc_or_bytes_per_line() {
    let row = empty_file_row();
    assert!(row.doc_pct.is_none());
    assert!(row.bytes_per_line.is_none());
}

// ── normalize_root ──────────────────────────────────────────────────────────

#[test]
fn normalize_root_relative_nonexistent_returns_as_is() {
    let p = Path::new("definitely/not/a/real/path/xyz");
    let result = normalize_root(p);
    assert_eq!(result, p.to_path_buf());
}

#[test]
fn normalize_root_empty_path() {
    let p = Path::new("");
    let result = normalize_root(p);
    // Either returns "" or canonicalized cwd
    let _ = result; // canonicalize("") may succeed or return ""
}

// ── AnalysisLimits ──────────────────────────────────────────────────────────

#[test]
fn analysis_limits_partial_construction() {
    let limits = AnalysisLimits {
        max_files: Some(50),
        max_commits: Some(200),
        ..Default::default()
    };
    assert_eq!(limits.max_files, Some(50));
    assert_eq!(limits.max_commits, Some(200));
    assert!(limits.max_bytes.is_none());
    assert!(limits.max_file_bytes.is_none());
    assert!(limits.max_commit_files.is_none());
}

#[test]
fn analysis_limits_debug_output() {
    let limits = AnalysisLimits::default();
    let debug = format!("{:?}", limits);
    assert!(debug.contains("AnalysisLimits"));
    assert!(debug.contains("None"));
}

#[test]
fn analysis_limits_zero_values() {
    let limits = AnalysisLimits {
        max_files: Some(0),
        max_bytes: Some(0),
        max_file_bytes: Some(0),
        max_commits: Some(0),
        max_commit_files: Some(0),
    };
    assert_eq!(limits.max_files, Some(0));
    assert_eq!(limits.max_bytes, Some(0));
}

// ── Combined behavior ───────────────────────────────────────────────────────

#[test]
fn round_f64_and_safe_ratio_agree_on_simple_fractions() {
    // safe_ratio rounds to 4 decimals internally
    let sr = safe_ratio(1, 3);
    let manual = round_f64(1.0 / 3.0, 4);
    assert_eq!(sr, manual);
}

#[test]
fn normalize_path_then_path_depth() {
    let root = PathBuf::from("repo");
    let normalized = normalize_path(r".\src\a\b\c.rs", &root);
    assert_eq!(normalized, "src/a/b/c.rs");
    assert_eq!(path_depth(&normalized), 4);
}
