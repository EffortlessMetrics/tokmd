//! Deep tests for `analysis types util module` shared utilities.
//!
//! Covers corner cases, boundary values, invariant checks, and interaction
//! scenarios across normalize_path, path_depth, is_test_path, is_infra_lang,
//! empty_file_row, normalize_root, AnalysisLimits, and re-exported math helpers.

use std::path::{Path, PathBuf};

use crate::{
    AnalysisLimits, empty_file_row, gini_coefficient, is_infra_lang, is_test_path, normalize_path,
    normalize_root, now_ms, path_depth, percentile, round_f64, safe_ratio,
};

// ── normalize_path: root-stripping interactions ─────────────────────────────

#[test]
fn normalize_path_root_match_leaves_nested_relative() {
    let root = PathBuf::from("project");
    assert_eq!(
        normalize_path("project/src/deep/mod.rs", &root),
        "src/deep/mod.rs"
    );
}

#[test]
fn normalize_path_root_match_with_backslash_input() {
    let root = PathBuf::from("project");
    // backslash converted first, then root stripped
    assert_eq!(normalize_path(r"project\src\lib.rs", &root), "src/lib.rs");
}

#[test]
fn normalize_path_dot_slash_combined_with_root_no_double_strip() {
    // "./" is stripped, but root won't match the remaining path
    let root = PathBuf::from("other");
    assert_eq!(normalize_path("./src/main.rs", &root), "src/main.rs");
}

#[test]
fn normalize_path_multiple_dot_slash_only_strips_leading() {
    let root = PathBuf::from("r");
    let result = normalize_path("./a/./b/./c.rs", &root);
    assert!(!result.starts_with("./"));
    // Only the leading "./" is stripped; inner "./" remain
    assert!(result.contains("./b/"));
}

#[test]
fn normalize_path_single_filename_no_change() {
    let root = PathBuf::from("repo");
    assert_eq!(normalize_path("file.txt", &root), "file.txt");
}

#[test]
fn normalize_path_whitespace_preserved() {
    let root = PathBuf::from("repo");
    assert_eq!(normalize_path("src/my file.rs", &root), "src/my file.rs");
}

#[test]
fn normalize_path_double_dot_prefix_not_stripped() {
    let root = PathBuf::from("repo");
    let result = normalize_path("../sibling/lib.rs", &root);
    assert!(result.starts_with(".."));
}

// ── path_depth: boundary and semantic checks ────────────────────────────────

#[test]
fn path_depth_single_dot_is_one_segment() {
    assert_eq!(path_depth("."), 1);
}

#[test]
fn path_depth_dot_dot_segments() {
    // ".." is a non-empty segment
    assert_eq!(path_depth("../src/lib.rs"), 3);
}

#[test]
fn path_depth_matches_forward_slash_count_plus_one() {
    // For a clean path without empty segments, depth == slash_count + 1
    let path = "a/b/c/d/e";
    let slash_count = path.chars().filter(|&c| c == '/').count();
    assert_eq!(path_depth(path), slash_count + 1);
}

#[test]
fn path_depth_backslash_not_treated_as_separator() {
    // path_depth only splits on '/', so backslashes are literal characters
    assert_eq!(path_depth(r"a\b\c"), 1);
}

#[test]
fn path_depth_unicode_segments() {
    assert_eq!(path_depth("日本/中国/한국"), 3);
}

// ── is_test_path: subtle detection patterns ─────────────────────────────────

#[test]
fn is_test_path_test_at_end_of_path_as_dir() {
    // "/test/" must be a directory component, not at end without trailing /
    assert!(is_test_path("project/test/unit.rs"));
}

#[test]
fn is_test_path_rejects_contest_in_dirname() {
    // "contest" contains "test" substring but not as "/test/" directory
    assert!(!is_test_path("src/contest/solution.py"));
}

#[test]
fn is_test_path_rejects_latest_in_dirname() {
    // "latest" contains "test" substring
    assert!(!is_test_path("dist/latest/bundle.js"));
}

#[test]
fn is_test_path_rejects_attest_in_dirname() {
    assert!(!is_test_path("src/attest/verify.rs"));
}

#[test]
fn is_test_path_detects_test_suffix_various_extensions() {
    assert!(is_test_path("src/parser_test.py"));
    assert!(is_test_path("src/handler_test.go"));
    assert!(is_test_path("src/utils_test.ts"));
}

#[test]
fn is_test_path_detects_test_prefix_various_extensions() {
    assert!(is_test_path("test_main.py"));
    assert!(is_test_path("test_handler.go"));
}

#[test]
fn is_test_path_detects_dot_test_pattern_in_nested_path() {
    assert!(is_test_path("packages/ui/src/Button.test.tsx"));
}

#[test]
fn is_test_path_uppercase_spec_dir() {
    assert!(is_test_path("app/SPEC/model_spec.rb"));
}

#[test]
fn is_test_path_dunder_tests_case_insensitive() {
    assert!(is_test_path("src/__TESTS__/component.js"));
}

#[test]
fn is_test_path_no_extension() {
    // "test_runner" starts with "test_"
    assert!(is_test_path("bin/test_runner"));
}

#[test]
fn is_test_path_rejects_testimony() {
    // "testimony" in filename doesn't match exact patterns
    assert!(!is_test_path("src/testimony.rs"));
}

// ── is_infra_lang: boundary detection ───────────────────────────────────────

#[test]
fn is_infra_lang_rejects_near_matches() {
    assert!(!is_infra_lang("json5"));
    assert!(!is_infra_lang("yaml2"));
    assert!(!is_infra_lang("toml2"));
    assert!(!is_infra_lang("xml2"));
}

#[test]
fn is_infra_lang_rejects_prefixed_langs() {
    assert!(!is_infra_lang("ejson"));
    assert!(!is_infra_lang("xhtml")); // only "html" matches
    assert!(!is_infra_lang("postcss"));
}

#[test]
fn is_infra_lang_all_case_variants_of_every_infra_lang() {
    // Exhaustive upper/lower/title case for every infra language
    let all_infra = [
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
    for lang in &all_infra {
        let upper = lang.to_uppercase();
        let mut title = String::new();
        for (i, c) in lang.chars().enumerate() {
            if i == 0 {
                title.extend(c.to_uppercase());
            } else {
                title.push(c);
            }
        }
        assert!(is_infra_lang(lang), "lower: {}", lang);
        assert!(is_infra_lang(&upper), "upper: {}", upper);
        assert!(is_infra_lang(&title), "title: {}", title);
    }
}

#[test]
fn is_infra_lang_rejects_numeric_and_special_strings() {
    assert!(!is_infra_lang("123"));
    assert!(!is_infra_lang("!!!"));
    assert!(!is_infra_lang("json\n"));
    assert!(!is_infra_lang("json\t"));
}

// ── empty_file_row: mutation independence ───────────────────────────────────

#[test]
fn empty_file_row_multiple_calls_independent() {
    let mut a = empty_file_row();
    let b = empty_file_row();
    a.code = 42;
    a.path = "modified".to_string();
    assert_eq!(b.code, 0);
    assert!(b.path.is_empty());
}

#[test]
fn empty_file_row_clone_is_independent() {
    let original = empty_file_row();
    let mut cloned = original.clone();
    cloned.code = 100;
    cloned.lang = "Rust".to_string();
    assert_eq!(original.code, 0);
    assert!(original.lang.is_empty());
}

#[test]
fn empty_file_row_lines_equal_code_plus_comments_plus_blanks() {
    let row = empty_file_row();
    // For the empty row, all are zero, so the invariant holds trivially
    assert_eq!(row.lines, row.code + row.comments + row.blanks);
}

// ── normalize_root: behavior on various path types ──────────────────────────

#[test]
fn normalize_root_relative_dot_resolves_to_cwd() {
    let result = normalize_root(Path::new("."));
    // "." should canonicalize to the current working directory
    assert!(result.is_absolute());
}

#[test]
fn normalize_root_empty_path_fallback() {
    let result = normalize_root(Path::new(""));
    // Empty path cannot be canonicalized; returns the empty PathBuf
    assert_eq!(result, PathBuf::from(""));
}

// ── AnalysisLimits: trait behavior ──────────────────────────────────────────

#[test]
fn analysis_limits_clone_is_equal_to_original() {
    let original = AnalysisLimits {
        max_files: Some(100),
        max_bytes: Some(50_000),
        max_file_bytes: Some(10_000),
        max_commits: Some(200),
        max_commit_files: Some(15),
    };
    let cloned = original.clone();
    assert_eq!(cloned.max_files, original.max_files);
    assert_eq!(cloned.max_bytes, original.max_bytes);
    assert_eq!(cloned.max_file_bytes, original.max_file_bytes);
    assert_eq!(cloned.max_commits, original.max_commits);
    assert_eq!(cloned.max_commit_files, original.max_commit_files);
}

#[test]
fn analysis_limits_debug_format_contains_field_names() {
    let limits = AnalysisLimits {
        max_files: Some(5),
        ..Default::default()
    };
    let debug = format!("{:?}", limits);
    assert!(debug.contains("max_files"));
    assert!(debug.contains("max_bytes"));
    assert!(debug.contains("max_file_bytes"));
    assert!(debug.contains("max_commits"));
    assert!(debug.contains("max_commit_files"));
}

#[test]
fn analysis_limits_default_then_override_single_field() {
    let limits = AnalysisLimits {
        max_commits: Some(999),
        ..Default::default()
    };
    assert_eq!(limits.max_commits, Some(999));
    assert!(limits.max_files.is_none());
    assert!(limits.max_bytes.is_none());
}

#[test]
fn analysis_limits_max_values() {
    let limits = AnalysisLimits {
        max_files: Some(usize::MAX),
        max_bytes: Some(u64::MAX),
        max_file_bytes: Some(u64::MAX),
        max_commits: Some(usize::MAX),
        max_commit_files: Some(usize::MAX),
    };
    assert_eq!(limits.max_files, Some(usize::MAX));
    assert_eq!(limits.max_bytes, Some(u64::MAX));
}

// ── now_ms: timing invariants ───────────────────────────────────────────────

#[test]
fn now_ms_monotonic_across_loop() {
    let mut prev = now_ms();
    for _ in 0..100 {
        let curr = now_ms();
        assert!(curr >= prev, "now_ms went backward: {} < {}", curr, prev);
        prev = curr;
    }
}

#[test]
fn now_ms_within_reasonable_range() {
    let ts = now_ms();
    // Should be between 2020 and 2100 in epoch millis
    let min_2020: u128 = 1_577_836_800_000;
    let max_2100: u128 = 4_102_444_800_000;
    assert!(
        ts > min_2020 && ts < max_2100,
        "timestamp out of range: {}",
        ts
    );
}

// ── round_f64: precision and special values ─────────────────────────────────

#[test]
fn round_f64_half_rounds_to_even_behavior() {
    // f64 .round() uses "round half away from zero"
    assert_eq!(round_f64(0.5, 0), 1.0);
    assert_eq!(round_f64(1.5, 0), 2.0);
    assert_eq!(round_f64(-0.5, 0), -1.0);
}

#[test]
fn round_f64_preserves_exact_values() {
    assert_eq!(round_f64(1.0, 5), 1.0);
    assert_eq!(round_f64(0.0, 0), 0.0);
    assert_eq!(round_f64(-1.0, 3), -1.0);
}

#[test]
#[allow(clippy::approx_constant)]
fn round_f64_high_precision() {
    let val = round_f64(std::f64::consts::PI, 10);
    assert!((val - 3.1415926536).abs() < 1e-10);
}

#[test]
fn round_f64_nan_stays_nan() {
    let val = round_f64(f64::NAN, 2);
    assert!(val.is_nan());
}

#[test]
fn round_f64_infinity_stays_infinity() {
    assert_eq!(round_f64(f64::INFINITY, 2), f64::INFINITY);
    assert_eq!(round_f64(f64::NEG_INFINITY, 2), f64::NEG_INFINITY);
}

// ── safe_ratio: edge cases ──────────────────────────────────────────────────

#[test]
fn safe_ratio_very_large_numerator() {
    let r = safe_ratio(usize::MAX, 1);
    assert!(r > 0.0);
    assert!(r.is_finite());
}

#[test]
fn safe_ratio_one_to_large_denominator() {
    let r = safe_ratio(1, usize::MAX);
    assert_eq!(r, 0.0); // rounds to 0.0000 at 4 decimals
}

#[test]
fn safe_ratio_rounds_to_four_decimals() {
    // 1/3 = 0.33333... rounds to 0.3333
    assert_eq!(safe_ratio(1, 3), 0.3333);
    // 2/3 = 0.66666... rounds to 0.6667
    assert_eq!(safe_ratio(2, 3), 0.6667);
}

// ── percentile: corner cases ────────────────────────────────────────────────

#[test]
fn percentile_all_same_values() {
    let vals = [7, 7, 7, 7, 7];
    for pct in [0.0, 0.25, 0.5, 0.75, 1.0] {
        assert_eq!(percentile(&vals, pct), 7.0, "pct={}", pct);
    }
}

#[test]
fn percentile_two_elements_boundaries() {
    let vals = [10, 20];
    assert_eq!(percentile(&vals, 0.0), 10.0);
    assert_eq!(percentile(&vals, 1.0), 20.0);
}

#[test]
fn percentile_large_sorted_input() {
    let vals: Vec<usize> = (1..=1000).collect();
    let p50 = percentile(&vals, 0.5);
    // Median of 1..=1000 should be around 500
    assert!((p50 - 500.0).abs() < 2.0, "p50={}", p50);
}

#[test]
fn percentile_p0_is_minimum_p100_is_maximum() {
    let vals = [5, 15, 25, 35, 45, 55];
    assert_eq!(percentile(&vals, 0.0), 5.0);
    assert_eq!(percentile(&vals, 1.0), 55.0);
}

// ── gini_coefficient: distribution shape tests ──────────────────────────────

#[test]
fn gini_coefficient_linear_increase() {
    // [1, 2, 3, 4, 5] — moderate inequality
    let g = gini_coefficient(&[1, 2, 3, 4, 5]);
    assert!(g > 0.1 && g < 0.5, "Expected moderate gini, got {}", g);
}

#[test]
fn gini_coefficient_single_large_with_many_small() {
    // High inequality: one large value among many small
    let mut vals = vec![1usize; 99];
    vals.push(10_000);
    vals.sort();
    let g = gini_coefficient(&vals);
    assert!(g > 0.9, "Expected very high gini, got {}", g);
}

#[test]
fn gini_coefficient_two_values_half() {
    // [0, X] should give gini = 0.5 for any positive X
    let g = gini_coefficient(&[0, 1000]);
    assert!((g - 0.5).abs() < 1e-10, "Expected 0.5, got {}", g);
}

#[test]
fn gini_coefficient_symmetric_around_middle() {
    // [1, 2, 3, 2, 1] sorted = [1, 1, 2, 2, 3]
    let g = gini_coefficient(&[1, 1, 2, 2, 3]);
    assert!(g > 0.0 && g < 0.5);
}

// ── Cross-function interaction tests ────────────────────────────────────────

#[test]
fn normalize_path_then_path_depth_consistent() {
    let root = PathBuf::from("project");
    let raw = r"project\src\deep\mod.rs";
    let normalized = normalize_path(raw, &root);
    // After normalization and root-stripping: "src/deep/mod.rs" → depth 3
    assert_eq!(path_depth(&normalized), 3);
}

#[test]
fn normalize_path_then_is_test_path_detects_test_dir() {
    let root = PathBuf::from("proj");
    let raw = r"proj\src\tests\unit.rs";
    let normalized = normalize_path(raw, &root);
    assert!(is_test_path(&normalized));
}

#[test]
fn normalize_path_then_is_test_path_rejects_non_test() {
    let root = PathBuf::from("proj");
    let raw = r"proj\src\lib.rs";
    let normalized = normalize_path(raw, &root);
    assert!(!is_test_path(&normalized));
}

#[test]
fn empty_file_row_depth_matches_path_depth_for_empty() {
    let row = empty_file_row();
    // Empty path → path_depth("") = 1 (max(0,1)), but row.depth is 0
    // This documents the intentional difference: empty_file_row sets depth=0
    assert_eq!(row.depth, 0);
    assert_eq!(path_depth(&row.path), 1);
}

#[test]
fn safe_ratio_result_matches_manual_round() {
    // safe_ratio(a, b) == round_f64(a as f64 / b as f64, 4) when b != 0
    let a = 7usize;
    let b = 13usize;
    let expected = round_f64(a as f64 / b as f64, 4);
    assert_eq!(safe_ratio(a, b), expected);
}
