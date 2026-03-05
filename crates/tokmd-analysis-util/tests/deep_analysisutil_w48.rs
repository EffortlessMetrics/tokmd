//! Deep analysis-util tests (wave 48).
//!
//! Covers:
//! - Shared utility functions used by enrichers
//! - Statistical helper functions (gini, percentile, safe_ratio, round_f64)
//! - Data normalization helpers (normalize_path, path_depth, is_test_path, is_infra_lang)

use std::path::{Path, PathBuf};

use tokmd_analysis_util::{
    AnalysisLimits, empty_file_row, gini_coefficient, is_infra_lang, is_test_path, normalize_path,
    normalize_root, now_ms, path_depth, percentile, round_f64, safe_ratio,
};

// ═══════════════════════════════════════════════════════════════════════════
// 1. Shared utility — normalize_path edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn normalize_path_strips_root_prefix_when_matching() {
    let root = PathBuf::from("myproject");
    assert_eq!(
        normalize_path("myproject/src/main.rs", &root),
        "src/main.rs"
    );
}

#[test]
fn normalize_path_converts_all_backslashes() {
    let root = PathBuf::from("x");
    let result = normalize_path(r"a\b\c\d.rs", &root);
    assert!(!result.contains('\\'), "No backslashes should remain");
    assert_eq!(result, "a/b/c/d.rs");
}

#[test]
fn normalize_path_empty_input_returns_empty() {
    let root = PathBuf::from("repo");
    assert_eq!(normalize_path("", &root), "");
}

#[test]
fn normalize_path_only_dot_slash() {
    let root = PathBuf::from("x");
    assert_eq!(normalize_path("./", &root), "");
}

#[test]
fn normalize_path_deeply_nested_with_root_strip() {
    let root = PathBuf::from("repo");
    assert_eq!(
        normalize_path("repo/a/b/c/d/e/f.rs", &root),
        "a/b/c/d/e/f.rs"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Statistical helpers — safe_ratio
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn safe_ratio_zero_denominator_returns_zero() {
    assert_eq!(safe_ratio(100, 0), 0.0);
}

#[test]
fn safe_ratio_zero_numerator_returns_zero() {
    assert_eq!(safe_ratio(0, 100), 0.0);
}

#[test]
fn safe_ratio_equal_values_returns_one() {
    assert_eq!(safe_ratio(42, 42), 1.0);
}

#[test]
fn safe_ratio_known_fraction() {
    // 1/4 = 0.25 exactly
    assert_eq!(safe_ratio(1, 4), 0.25);
}

#[test]
fn safe_ratio_rounds_to_four_decimal_places() {
    // 1/7 = 0.142857... rounds to 0.1429
    assert_eq!(safe_ratio(1, 7), 0.1429);
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Statistical helpers — percentile
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn percentile_single_element() {
    assert_eq!(percentile(&[42], 0.5), 42.0);
}

#[test]
fn percentile_p50_of_even_count() {
    let vals = [10, 20, 30, 40];
    let p50 = percentile(&vals, 0.5);
    // percentile uses index-based lookup, so p50 of [10,20,30,40] is 30
    assert!(
        (p50 - 30.0).abs() < 1.0,
        "Median of [10,20,30,40] should be ~30, got {}",
        p50
    );
}

#[test]
fn percentile_p25_and_p75() {
    let vals: Vec<usize> = (1..=100).collect();
    let p25 = percentile(&vals, 0.25);
    let p75 = percentile(&vals, 0.75);
    assert!(p25 < p75, "p25 ({}) should be less than p75 ({})", p25, p75);
    assert!((p25 - 25.5).abs() < 2.0);
    assert!((p75 - 75.5).abs() < 2.0);
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Statistical helpers — gini_coefficient
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn gini_perfect_equality() {
    let vals = [100, 100, 100, 100, 100];
    let g = gini_coefficient(&vals);
    assert!(
        g.abs() < 1e-10,
        "Perfect equality should have gini ≈ 0, got {}",
        g
    );
}

#[test]
fn gini_single_element_is_zero() {
    assert_eq!(gini_coefficient(&[42]), 0.0);
}

#[test]
fn gini_two_elements_equal_is_zero() {
    let g = gini_coefficient(&[5, 5]);
    assert!(g.abs() < 1e-10);
}

#[test]
fn gini_range_is_zero_to_one() {
    let test_cases: Vec<Vec<usize>> = vec![
        vec![1, 2, 3, 4, 5],
        vec![1, 1, 1, 1, 100],
        vec![0, 0, 0, 1000],
        vec![10, 20],
    ];
    for vals in &test_cases {
        let g = gini_coefficient(vals);
        assert!(
            (0.0..=1.0).contains(&g),
            "Gini out of range: {} for {:?}",
            g,
            vals
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Statistical helpers — round_f64
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn round_f64_zero_decimals() {
    assert_eq!(round_f64(3.7, 0), 4.0);
    assert_eq!(round_f64(3.2, 0), 3.0);
}

#[test]
fn round_f64_two_decimals() {
    assert_eq!(round_f64(1.23456, 2), 1.23);
    assert_eq!(round_f64(2.005, 2), 2.01); // floating point behavior
}

#[test]
fn round_f64_negative_value() {
    assert_eq!(round_f64(-1.23456, 2), -1.23);
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Data normalization — path_depth
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn path_depth_empty_string_returns_one() {
    assert_eq!(path_depth(""), 1);
}

#[test]
fn path_depth_single_file() {
    assert_eq!(path_depth("lib.rs"), 1);
}

#[test]
fn path_depth_two_segments() {
    assert_eq!(path_depth("src/lib.rs"), 2);
}

#[test]
fn path_depth_deeply_nested() {
    assert_eq!(path_depth("a/b/c/d/e/f/g.rs"), 7);
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Data normalization — is_test_path
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn is_test_path_detects_tests_dir() {
    assert!(is_test_path("src/tests/unit.rs"));
}

#[test]
fn is_test_path_detects_test_dir() {
    assert!(is_test_path("src/test/integration.rs"));
}

#[test]
fn is_test_path_detects_spec_dir() {
    assert!(is_test_path("app/spec/model_spec.rb"));
}

#[test]
fn is_test_path_rejects_production_code() {
    assert!(!is_test_path("src/lib.rs"));
    assert!(!is_test_path("src/main.rs"));
    assert!(!is_test_path("index.js"));
}

#[test]
fn is_test_path_detects_dot_test_pattern() {
    assert!(is_test_path("src/App.test.tsx"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Data normalization — is_infra_lang
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn is_infra_lang_detects_common_infra() {
    for lang in ["json", "yaml", "toml", "markdown", "xml", "html", "css"] {
        assert!(is_infra_lang(lang), "Should detect {}", lang);
    }
}

#[test]
fn is_infra_lang_rejects_code_langs() {
    for lang in ["rust", "python", "javascript", "go", "java", "c"] {
        assert!(!is_infra_lang(lang), "Should reject {}", lang);
    }
}

#[test]
fn is_infra_lang_case_insensitive() {
    assert!(is_infra_lang("JSON"));
    assert!(is_infra_lang("Yaml"));
    assert!(is_infra_lang("TOML"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. empty_file_row and AnalysisLimits
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn empty_file_row_all_numeric_fields_zero() {
    let r = empty_file_row();
    assert_eq!(r.code, 0);
    assert_eq!(r.comments, 0);
    assert_eq!(r.blanks, 0);
    assert_eq!(r.lines, 0);
    assert_eq!(r.bytes, 0);
    assert_eq!(r.tokens, 0);
    assert_eq!(r.depth, 0);
}

#[test]
fn empty_file_row_string_fields_empty() {
    let r = empty_file_row();
    assert!(r.path.is_empty());
    assert!(r.module.is_empty());
    assert!(r.lang.is_empty());
}

#[test]
fn analysis_limits_default_all_none() {
    let limits = AnalysisLimits::default();
    assert!(limits.max_files.is_none());
    assert!(limits.max_bytes.is_none());
    assert!(limits.max_file_bytes.is_none());
    assert!(limits.max_commits.is_none());
    assert!(limits.max_commit_files.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. now_ms and normalize_root
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn now_ms_returns_nonzero() {
    assert!(now_ms() > 0);
}

#[test]
fn normalize_root_nonexistent_returns_input() {
    let fake = Path::new("nonexistent_path_xyz_abc_123");
    let result = normalize_root(fake);
    assert_eq!(result, PathBuf::from("nonexistent_path_xyz_abc_123"));
}
