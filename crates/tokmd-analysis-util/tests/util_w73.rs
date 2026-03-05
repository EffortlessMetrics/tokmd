//! W73 deep tests for `tokmd-analysis-util` shared utilities.
//!
//! Covers edge cases, boundary values, and interaction scenarios across:
//! - normalize_path with complex root/backslash combinations
//! - path_depth with special segments and delimiters
//! - is_test_path with tricky substring matches
//! - is_infra_lang boundary detection
//! - empty_file_row invariants
//! - AnalysisLimits field independence
//! - Math helpers: round_f64, safe_ratio, percentile, gini_coefficient

use std::path::{Path, PathBuf};

use tokmd_analysis_util::{
    AnalysisLimits, empty_file_row, gini_coefficient, is_infra_lang, is_test_path, normalize_path,
    normalize_root, now_ms, path_depth, percentile, round_f64, safe_ratio,
};

// ═══════════════════════════════════════════════════════════════════════════
// 1. normalize_path — complex root/path interactions
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn normalize_path_strips_root_then_dot_slash() {
    let root = PathBuf::from("repo");
    // Input doesn't start with "./" but matches root prefix
    assert_eq!(normalize_path("repo/src/lib.rs", &root), "src/lib.rs");
}

#[test]
fn normalize_path_only_backslashes() {
    let root = PathBuf::from("x");
    assert_eq!(normalize_path(r"a\b\c\d.rs", &root), "a/b/c/d.rs");
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

#[test]
fn normalize_path_deeply_nested_backslashes() {
    let root = PathBuf::from("project");
    let input = r"project\a\b\c\d\e\f.rs";
    assert_eq!(normalize_path(input, &root), "a/b/c/d/e/f.rs");
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. path_depth — special segment types
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn path_depth_empty_string_returns_one() {
    assert_eq!(path_depth(""), 1);
}

#[test]
fn path_depth_only_slashes() {
    // "///" has no non-empty segments → max(0, 1) = 1
    assert_eq!(path_depth("///"), 1);
}

#[test]
fn path_depth_single_segment_no_slash() {
    assert_eq!(path_depth("file.rs"), 1);
}

#[test]
fn path_depth_deeply_nested() {
    assert_eq!(path_depth("a/b/c/d/e/f/g/h/i/j"), 10);
}

#[test]
fn path_depth_trailing_slash_ignored() {
    assert_eq!(path_depth("a/b/c/"), 3);
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. is_test_path — tricky substring detection
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn is_test_path_rejects_protestant() {
    assert!(!is_test_path("src/protestant.rs"));
}

#[test]
fn is_test_path_detects_nested_tests_dir() {
    assert!(is_test_path("packages/core/tests/integration/setup.ts"));
}

#[test]
fn is_test_path_detects_spec_dir_case_insensitive() {
    assert!(is_test_path("app/Spec/models/user_spec.rb"));
}

#[test]
fn is_test_path_rejects_testimony_file() {
    assert!(!is_test_path("docs/testimony.md"));
}

#[test]
fn is_test_path_file_ending_with_test_dot_rs() {
    assert!(is_test_path("src/parser_test.rs"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. is_infra_lang — boundary values
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn is_infra_lang_rejects_empty_string() {
    assert!(!is_infra_lang(""));
}

#[test]
fn is_infra_lang_rejects_whitespace() {
    assert!(!is_infra_lang(" "));
    assert!(!is_infra_lang("\t"));
}

#[test]
fn is_infra_lang_accepts_all_known_infra_langs() {
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
        assert!(is_infra_lang(lang), "should accept: {}", lang);
    }
}

#[test]
fn is_infra_lang_rejects_programming_langs() {
    let code_langs = [
        "rust",
        "python",
        "javascript",
        "typescript",
        "go",
        "java",
        "c",
        "cpp",
        "ruby",
        "kotlin",
        "swift",
        "haskell",
    ];
    for lang in &code_langs {
        assert!(!is_infra_lang(lang), "should reject code lang: {}", lang);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. empty_file_row — field invariants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn empty_file_row_all_numeric_fields_zero() {
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
fn empty_file_row_all_string_fields_empty() {
    let row = empty_file_row();
    assert!(row.path.is_empty());
    assert!(row.module.is_empty());
    assert!(row.lang.is_empty());
}

#[test]
fn empty_file_row_optional_fields_none() {
    let row = empty_file_row();
    assert!(row.doc_pct.is_none());
    assert!(row.bytes_per_line.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. AnalysisLimits — defaults and field independence
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn analysis_limits_default_all_none() {
    let limits = AnalysisLimits::default();
    assert!(limits.max_files.is_none());
    assert!(limits.max_bytes.is_none());
    assert!(limits.max_file_bytes.is_none());
    assert!(limits.max_commits.is_none());
    assert!(limits.max_commit_files.is_none());
}

#[test]
fn analysis_limits_partial_override_preserves_others() {
    let limits = AnalysisLimits {
        max_files: Some(50),
        max_bytes: Some(1_000_000),
        ..Default::default()
    };
    assert_eq!(limits.max_files, Some(50));
    assert_eq!(limits.max_bytes, Some(1_000_000));
    assert!(limits.max_file_bytes.is_none());
    assert!(limits.max_commits.is_none());
    assert!(limits.max_commit_files.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Math helpers — edge cases
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
fn round_f64_zero_decimals() {
    assert_eq!(round_f64(3.7, 0), 4.0);
    assert_eq!(round_f64(3.2, 0), 3.0);
}

#[test]
fn round_f64_negative_values() {
    assert_eq!(round_f64(-2.345, 2), -2.35);
}

#[test]
fn percentile_single_element() {
    assert_eq!(percentile(&[42], 0.0), 42.0);
    assert_eq!(percentile(&[42], 0.5), 42.0);
    assert_eq!(percentile(&[42], 1.0), 42.0);
}

#[test]
fn gini_coefficient_perfect_equality() {
    let vals = [100, 100, 100, 100, 100];
    let g = gini_coefficient(&vals);
    assert!(
        g.abs() < 1e-10,
        "Perfect equality should have gini ≈ 0, got {}",
        g
    );
}

#[test]
fn gini_coefficient_single_element_is_zero() {
    let g = gini_coefficient(&[42]);
    assert_eq!(g, 0.0, "Single element should have gini = 0");
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. now_ms — temporal invariants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn now_ms_returns_positive() {
    assert!(now_ms() > 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. normalize_root — path canonicalization
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn normalize_root_existing_dir_is_absolute() {
    let result = normalize_root(Path::new("."));
    assert!(result.is_absolute());
}

#[test]
fn normalize_root_nonexistent_returns_original() {
    let fake = Path::new("definitely_not_a_real_directory_xyz123");
    let result = normalize_root(fake);
    assert_eq!(result, fake.to_path_buf());
}
