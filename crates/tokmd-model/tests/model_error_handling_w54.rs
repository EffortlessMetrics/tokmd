//! Comprehensive error handling and edge case tests for tokmd-model.

use std::path::Path;
use tokei::Languages;
use tokmd_model::{
    avg, collect_file_rows, create_export_data, create_lang_report, create_module_report,
    module_key, normalize_path, unique_parent_file_count,
};
use tokmd_types::{ChildIncludeMode, ChildrenMode};

// ── avg() edge cases ──────────────────────────────────────────────────

#[test]
fn avg_zero_files_returns_zero() {
    assert_eq!(avg(1000, 0), 0);
}

#[test]
fn avg_zero_lines_returns_zero() {
    assert_eq!(avg(0, 10), 0);
}

#[test]
fn avg_both_zero_returns_zero() {
    assert_eq!(avg(0, 0), 0);
}

#[test]
fn avg_rounds_to_nearest() {
    // 7 / 2 = 3.5 → rounds to 4
    assert_eq!(avg(7, 2), 4);
    // 5 / 3 = 1.67 → rounds to 2
    assert_eq!(avg(5, 3), 2);
}

#[test]
fn avg_exact_division() {
    assert_eq!(avg(300, 3), 100);
    assert_eq!(avg(1000, 10), 100);
}

// ── normalize_path edge cases ─────────────────────────────────────────

#[test]
fn normalize_path_strips_leading_dot_slash() {
    let p = Path::new("./src/lib.rs");
    assert_eq!(normalize_path(p, None), "src/lib.rs");
}

#[test]
fn normalize_path_backslashes_to_forward() {
    let p = Path::new("src\\main.rs");
    assert_eq!(normalize_path(p, None), "src/main.rs");
}

#[test]
fn normalize_path_already_normalized() {
    let p = Path::new("src/lib.rs");
    assert_eq!(normalize_path(p, None), "src/lib.rs");
}

#[test]
fn normalize_path_with_strip_prefix() {
    let p = Path::new("project/src/lib.rs");
    let prefix = Path::new("project");
    assert_eq!(normalize_path(p, Some(prefix)), "src/lib.rs");
}

#[test]
fn normalize_path_strip_prefix_not_matching() {
    let p = Path::new("other/src/lib.rs");
    let prefix = Path::new("project");
    // Prefix doesn't match → path returned unchanged (normalized slashes)
    assert_eq!(normalize_path(p, Some(prefix)), "other/src/lib.rs");
}

#[test]
fn normalize_path_root_file() {
    let p = Path::new("Cargo.toml");
    assert_eq!(normalize_path(p, None), "Cargo.toml");
}

#[test]
fn normalize_path_empty_path() {
    let p = Path::new("");
    let result = normalize_path(p, None);
    assert!(result.is_empty() || result == ".");
}

// ── module_key edge cases ─────────────────────────────────────────────

#[test]
fn module_key_root_file() {
    let roots = vec!["crates".to_string()];
    assert_eq!(module_key("Cargo.toml", &roots, 2), "(root)");
}

#[test]
fn module_key_inside_module_root() {
    let roots = vec!["crates".to_string()];
    assert_eq!(module_key("crates/foo/src/lib.rs", &roots, 2), "crates/foo");
}

#[test]
fn module_key_not_in_roots() {
    let roots = vec!["crates".to_string()];
    assert_eq!(module_key("src/lib.rs", &roots, 2), "src");
}

#[test]
fn module_key_empty_roots() {
    let roots: Vec<String> = vec![];
    assert_eq!(module_key("src/lib.rs", &roots, 2), "src");
    assert_eq!(module_key("Cargo.toml", &roots, 2), "(root)");
}

#[test]
fn module_key_depth_one() {
    let roots = vec!["crates".to_string()];
    assert_eq!(module_key("crates/foo/src/lib.rs", &roots, 1), "crates");
}

// ── Empty Languages (empty scan results) ──────────────────────────────

#[test]
fn create_lang_report_empty_languages_collapse() {
    let langs = Languages::new();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    assert!(report.rows.is_empty());
    assert_eq!(report.total.code, 0);
    assert_eq!(report.total.files, 0);
    assert_eq!(report.total.bytes, 0);
    assert_eq!(report.total.tokens, 0);
}

#[test]
fn create_lang_report_empty_languages_separate() {
    let langs = Languages::new();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Separate);
    assert!(report.rows.is_empty());
    assert_eq!(report.total.code, 0);
}

#[test]
fn create_lang_report_empty_with_files_flag() {
    let langs = Languages::new();
    let report = create_lang_report(&langs, 0, true, ChildrenMode::Collapse);
    assert!(report.rows.is_empty());
    assert!(report.with_files);
}

#[test]
fn create_module_report_empty_languages() {
    let langs = Languages::new();
    let report = create_module_report(&langs, &[], 2, ChildIncludeMode::ParentsOnly, 0);
    assert!(report.rows.is_empty());
    assert_eq!(report.total.code, 0);
    assert_eq!(report.total.files, 0);
}

#[test]
fn create_module_report_empty_with_roots() {
    let langs = Languages::new();
    let roots = vec!["crates".to_string(), "packages".to_string()];
    let report = create_module_report(&langs, &roots, 2, ChildIncludeMode::ParentsOnly, 0);
    assert!(report.rows.is_empty());
    assert_eq!(report.module_roots, roots);
}

#[test]
fn create_export_data_empty_languages() {
    let langs = Languages::new();
    let data = create_export_data(&langs, &[], 2, ChildIncludeMode::ParentsOnly, None, 0, 0);
    assert!(data.rows.is_empty());
}

#[test]
fn create_export_data_with_min_code_filter() {
    let langs = Languages::new();
    let data = create_export_data(&langs, &[], 2, ChildIncludeMode::ParentsOnly, None, 100, 0);
    assert!(data.rows.is_empty());
}

#[test]
fn create_export_data_with_max_rows() {
    let langs = Languages::new();
    let data = create_export_data(&langs, &[], 2, ChildIncludeMode::ParentsOnly, None, 0, 5);
    assert!(data.rows.is_empty());
}

// ── unique_parent_file_count ──────────────────────────────────────────

#[test]
fn unique_parent_file_count_empty() {
    let langs = Languages::new();
    assert_eq!(unique_parent_file_count(&langs), 0);
}

// ── collect_file_rows ─────────────────────────────────────────────────

#[test]
fn collect_file_rows_empty_languages() {
    let langs = Languages::new();
    let rows = collect_file_rows(&langs, &[], 2, ChildIncludeMode::ParentsOnly, None);
    assert!(rows.is_empty());
}

#[test]
fn collect_file_rows_empty_with_separate_mode() {
    let langs = Languages::new();
    let rows = collect_file_rows(&langs, &[], 2, ChildIncludeMode::Separate, None);
    assert!(rows.is_empty());
}

// ── top-N truncation edge cases ───────────────────────────────────────

#[test]
fn create_lang_report_top_zero_means_no_limit() {
    let langs = Languages::new();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    // top=0 means show all; with empty data, no rows
    assert!(report.rows.is_empty());
    assert_eq!(report.top, 0);
}

#[test]
fn create_module_report_top_zero_means_no_limit() {
    let langs = Languages::new();
    let report = create_module_report(&langs, &[], 2, ChildIncludeMode::ParentsOnly, 0);
    assert!(report.rows.is_empty());
    assert_eq!(report.top, 0);
}

// ── create_lang_report with large top (> rows) ───────────────────────

#[test]
fn create_lang_report_top_larger_than_rows() {
    let langs = Languages::new();
    let report = create_lang_report(&langs, 100, false, ChildrenMode::Collapse);
    // top > actual rows → no "Other" row created
    assert!(report.rows.is_empty());
}
