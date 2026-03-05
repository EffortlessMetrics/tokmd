//! Pipeline integration tests for `tokmd-model` (w67).
//!
//! Verifies language aggregation, module aggregation, file-row generation,
//! deterministic sorting, totals invariants, and property-based checks.

use proptest::prelude::*;
use std::path::PathBuf;
use tokei::{Config, Languages};
use tokmd_model::{
    avg, collect_file_rows, create_export_data, create_lang_report, create_module_report,
    normalize_path, unique_parent_file_count,
};
use tokmd_types::{ChildIncludeMode, ChildrenMode};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn scan_path(path: &str) -> Languages {
    let mut languages = Languages::new();
    let paths = vec![PathBuf::from(path)];
    let cfg = Config::default();
    languages.get_statistics(&paths, &[], &cfg);
    languages
}

fn scan_self() -> Languages {
    scan_path(&format!("{}/src", env!("CARGO_MANIFEST_DIR")))
}

fn scan_workspace() -> Languages {
    scan_path(env!("CARGO_MANIFEST_DIR"))
}

// ===========================================================================
// 1. Language aggregation — single language
// ===========================================================================

#[test]
fn single_lang_collapse_has_one_row() {
    let langs = scan_self();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    assert_eq!(report.rows.len(), 1, "pure Rust crate → 1 language");
    assert_eq!(report.rows[0].lang, "Rust");
}

#[test]
fn single_lang_totals_equal_sole_row() {
    let langs = scan_self();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    assert_eq!(report.total.code, report.rows[0].code);
    assert_eq!(report.total.lines, report.rows[0].lines);
}

#[test]
fn single_lang_positive_metrics() {
    let langs = scan_self();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    assert!(report.total.code > 0);
    assert!(report.total.files > 0);
    assert!(report.total.bytes > 0);
}

// ===========================================================================
// 2. Multi-language aggregation
// ===========================================================================

#[test]
fn multi_lang_rows_sum_to_totals_code() {
    let langs = scan_workspace();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let sum_code: usize = report.rows.iter().map(|r| r.code).sum();
    assert_eq!(sum_code, report.total.code);
}

#[test]
fn multi_lang_rows_sum_to_totals_lines() {
    let langs = scan_workspace();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let sum_lines: usize = report.rows.iter().map(|r| r.lines).sum();
    assert_eq!(sum_lines, report.total.lines);
}

#[test]
fn multi_lang_rows_sum_to_totals_bytes() {
    let langs = scan_workspace();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let sum_bytes: usize = report.rows.iter().map(|r| r.bytes).sum();
    assert_eq!(sum_bytes, report.total.bytes);
}

// ===========================================================================
// 3. Module aggregation at different depths
// ===========================================================================

#[test]
fn module_report_depth_1_groups_top_dirs() {
    let langs = scan_workspace();
    let report = create_module_report(&langs, &[], 1, ChildIncludeMode::ParentsOnly, 0);
    // All module keys should be either top-level dirs or "(root)"
    for row in &report.rows {
        assert!(
            !row.module.contains('/') || row.module == "(root)",
            "depth 1 should not have nested modules: {}",
            row.module
        );
    }
}

#[test]
fn module_report_depth_2_with_roots() {
    let langs = scan_workspace();
    let roots = vec!["tests".to_string()];
    let report = create_module_report(&langs, &roots, 2, ChildIncludeMode::ParentsOnly, 0);
    assert!(!report.rows.is_empty());
    assert!(report.total.code > 0);
}

#[test]
fn module_report_totals_are_positive() {
    let langs = scan_self();
    let report = create_module_report(&langs, &[], 1, ChildIncludeMode::ParentsOnly, 0);
    assert!(report.total.code > 0);
    assert!(report.total.files > 0);
}

#[test]
fn module_report_code_sum_matches_total() {
    let langs = scan_workspace();
    let report = create_module_report(&langs, &[], 1, ChildIncludeMode::ParentsOnly, 0);
    let sum: usize = report.rows.iter().map(|r| r.code).sum();
    assert_eq!(sum, report.total.code);
}

// ===========================================================================
// 4. File row generation
// ===========================================================================

#[test]
fn file_rows_have_forward_slash_paths() {
    let langs = scan_self();
    let rows = collect_file_rows(&langs, &[], 1, ChildIncludeMode::ParentsOnly, None);
    for row in &rows {
        assert!(
            !row.path.contains('\\'),
            "paths must use forward slashes: {}",
            row.path
        );
    }
}

#[test]
fn file_rows_have_positive_code() {
    let langs = scan_self();
    let rows = collect_file_rows(&langs, &[], 1, ChildIncludeMode::ParentsOnly, None);
    assert!(!rows.is_empty());
    // At least the lib.rs should have code
    let has_code = rows.iter().any(|r| r.code > 0);
    assert!(has_code);
}

#[test]
fn file_rows_lines_equal_sum_of_parts() {
    let langs = scan_self();
    let rows = collect_file_rows(&langs, &[], 1, ChildIncludeMode::ParentsOnly, None);
    for row in &rows {
        assert_eq!(
            row.lines,
            row.code + row.comments + row.blanks,
            "lines = code + comments + blanks for {}",
            row.path
        );
    }
}

#[test]
fn unique_parent_file_count_matches_report_files() {
    let langs = scan_self();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let count = unique_parent_file_count(&langs);
    assert_eq!(report.total.files, count);
}

// ===========================================================================
// 5. Sorting: descending by code, then by name
// ===========================================================================

#[test]
fn lang_report_rows_sorted_desc_by_code() {
    let langs = scan_workspace();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    for window in report.rows.windows(2) {
        assert!(
            window[0].code >= window[1].code,
            "rows must be sorted desc by code: {} ({}) vs {} ({})",
            window[0].lang,
            window[0].code,
            window[1].lang,
            window[1].code,
        );
    }
}

#[test]
fn lang_report_tied_rows_sorted_by_name() {
    // Create a synthetic scenario with tied code counts
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(dir.path().join("alpha.py"), "x = 1\n").unwrap();
    std::fs::write(dir.path().join("beta.rb"), "x = 1\n").unwrap();

    let langs = scan_path(dir.path().to_str().unwrap());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    for window in report.rows.windows(2) {
        if window[0].code == window[1].code {
            assert!(
                window[0].lang <= window[1].lang,
                "tied rows must be sorted by name: {} vs {}",
                window[0].lang,
                window[1].lang,
            );
        }
    }
}

#[test]
fn module_report_rows_sorted_desc_by_code() {
    let langs = scan_workspace();
    let report = create_module_report(&langs, &[], 1, ChildIncludeMode::ParentsOnly, 0);
    for window in report.rows.windows(2) {
        assert!(
            window[0].code >= window[1].code,
            "module rows must be sorted desc by code"
        );
    }
}

#[test]
fn export_data_rows_sorted_desc_by_code() {
    let langs = scan_self();
    let data = create_export_data(&langs, &[], 1, ChildIncludeMode::ParentsOnly, None, 0, 0);
    for window in data.rows.windows(2) {
        assert!(
            window[0].code >= window[1].code,
            "export rows must be sorted desc by code"
        );
    }
}

// ===========================================================================
// 6. Top-N truncation
// ===========================================================================

#[test]
fn lang_report_top_n_truncates() {
    let langs = scan_workspace();
    let report = create_lang_report(&langs, 1, false, ChildrenMode::Collapse);
    // top=1 → 1 real row + 1 "Other" row (if there are more languages)
    assert!(report.rows.len() <= 2, "top=1 should truncate to ≤2 rows");
    if report.rows.len() == 2 {
        assert_eq!(report.rows[1].lang, "Other");
    }
}

#[test]
fn module_report_top_n_truncates() {
    let langs = scan_workspace();
    let report = create_module_report(&langs, &[], 1, ChildIncludeMode::ParentsOnly, 1);
    assert!(report.rows.len() <= 2);
    if report.rows.len() == 2 {
        assert_eq!(report.rows[1].module, "Other");
    }
}

// ===========================================================================
// 7. Path normalization
// ===========================================================================

#[test]
fn normalize_path_converts_backslashes() {
    let p = std::path::Path::new("src\\main.rs");
    assert_eq!(normalize_path(p, None), "src/main.rs");
}

#[test]
fn normalize_path_strips_prefix() {
    let p = std::path::Path::new("project/src/lib.rs");
    let prefix = std::path::Path::new("project");
    assert_eq!(normalize_path(p, Some(prefix)), "src/lib.rs");
}

// ===========================================================================
// 8. avg() utility
// ===========================================================================

#[test]
fn avg_zero_files_returns_zero() {
    assert_eq!(avg(100, 0), 0);
}

#[test]
fn avg_rounds_to_nearest() {
    assert_eq!(avg(7, 2), 4); // 3.5 rounds up
    assert_eq!(avg(300, 3), 100);
}

// ===========================================================================
// 9. Proptest: aggregation totals match individual file totals
// ===========================================================================

proptest! {
    #[test]
    fn prop_avg_never_exceeds_lines(lines in 0..10_000usize, files in 1..1_000usize) {
        let result = avg(lines, files);
        // Average should never be more than lines (since files >= 1)
        prop_assert!(result <= lines, "avg({}, {}) = {} > lines", lines, files, result);
    }

    #[test]
    fn prop_avg_zero_lines_is_zero(files in 1..1_000usize) {
        prop_assert_eq!(avg(0, files), 0);
    }
}
