//! W73: Cross-crate integration tests for tokmd-scan → tokmd-model pipeline.
//!
//! These tests verify that scanning temp directories and feeding the results
//! into tokmd-model produces correct, deterministic aggregations.

use std::fs;
use tempfile::TempDir;

use tokmd_settings::ScanOptions;
use tokmd_types::{ChildIncludeMode, ChildrenMode, ConfigMode};

fn default_scan_options() -> ScanOptions {
    ScanOptions {
        excluded: vec![],
        config: ConfigMode::None,
        hidden: false,
        no_ignore: false,
        no_ignore_parent: false,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        treat_doc_strings_as_comments: false,
    }
}

/// Scaffold a temp dir with known files for deterministic tests.
fn scaffold() -> TempDir {
    let dir = TempDir::new().expect("create temp dir");

    // 3-line Rust file (1 blank, 2 code)
    fs::write(
        dir.path().join("main.rs"),
        "fn main() {\n    println!(\"hello\");\n}\n",
    )
    .unwrap();

    // 2-line Python file
    fs::write(dir.path().join("script.py"), "def run():\n    pass\n").unwrap();

    // Nested directory
    let sub = dir.path().join("src");
    fs::create_dir_all(&sub).unwrap();
    fs::write(
        sub.join("lib.rs"),
        "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
    )
    .unwrap();

    dir
}

// ===========================================================================
// Scan → Model: lang aggregation
// ===========================================================================

#[test]
fn scan_then_lang_report_finds_languages() {
    let dir = scaffold();
    let opts = default_scan_options();
    let paths = vec![dir.path().to_path_buf()];

    let languages = tokmd_scan::scan(&paths, &opts).expect("scan succeeds");
    let report = tokmd_model::create_lang_report(&languages, 0, false, ChildrenMode::Collapse);

    assert!(!report.rows.is_empty(), "should have language rows");
    let langs: Vec<&str> = report.rows.iter().map(|r| r.lang.as_str()).collect();
    assert!(langs.contains(&"Rust"), "should find Rust");
    assert!(langs.contains(&"Python"), "should find Python");
}

#[test]
fn scan_then_lang_report_totals_are_positive() {
    let dir = scaffold();
    let opts = default_scan_options();
    let paths = vec![dir.path().to_path_buf()];

    let languages = tokmd_scan::scan(&paths, &opts).unwrap();
    let report = tokmd_model::create_lang_report(&languages, 0, false, ChildrenMode::Collapse);

    assert!(report.total.code > 0, "total code > 0");
    assert!(report.total.files > 0, "total files > 0");
    assert!(report.total.lines > 0, "total lines > 0");
}

#[test]
fn scan_then_lang_report_rows_sum_to_total() {
    let dir = scaffold();
    let opts = default_scan_options();
    let paths = vec![dir.path().to_path_buf()];

    let languages = tokmd_scan::scan(&paths, &opts).unwrap();
    let report = tokmd_model::create_lang_report(&languages, 0, false, ChildrenMode::Collapse);

    let row_code_sum: usize = report.rows.iter().map(|r| r.code).sum();
    assert_eq!(row_code_sum, report.total.code);
}

// ===========================================================================
// Scan → Model: module aggregation
// ===========================================================================

#[test]
fn scan_then_module_report_finds_modules() {
    let dir = scaffold();
    let opts = default_scan_options();
    let paths = vec![dir.path().to_path_buf()];

    let languages = tokmd_scan::scan(&paths, &opts).unwrap();
    let report =
        tokmd_model::create_module_report(&languages, &[], 1, ChildIncludeMode::ParentsOnly, 0);

    assert!(!report.rows.is_empty(), "should have module rows");
    assert!(report.total.code > 0);
}

#[test]
fn scan_then_module_report_totals_match_lang() {
    let dir = scaffold();
    let opts = default_scan_options();
    let paths = vec![dir.path().to_path_buf()];

    let languages = tokmd_scan::scan(&paths, &opts).unwrap();
    let lang = tokmd_model::create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    let module =
        tokmd_model::create_module_report(&languages, &[], 1, ChildIncludeMode::ParentsOnly, 0);

    assert_eq!(
        lang.total.code, module.total.code,
        "lang and module totals should match"
    );
    assert_eq!(lang.total.files, module.total.files);
}

// ===========================================================================
// Scan → Model: export (file-level) data
// ===========================================================================

#[test]
fn scan_then_export_data_lists_all_files() {
    let dir = scaffold();
    let opts = default_scan_options();
    let paths = vec![dir.path().to_path_buf()];

    let languages = tokmd_scan::scan(&paths, &opts).unwrap();
    let data = tokmd_model::create_export_data(
        &languages,
        &[],
        1,
        ChildIncludeMode::ParentsOnly,
        None,
        0,
        0,
    );

    // We created 3 files: main.rs, script.py, src/lib.rs
    assert_eq!(data.rows.len(), 3, "should export exactly 3 file rows");
}

#[test]
fn scan_then_export_data_paths_are_forward_slash() {
    let dir = scaffold();
    let opts = default_scan_options();
    let paths = vec![dir.path().to_path_buf()];

    let languages = tokmd_scan::scan(&paths, &opts).unwrap();
    let data = tokmd_model::create_export_data(
        &languages,
        &[],
        1,
        ChildIncludeMode::ParentsOnly,
        None,
        0,
        0,
    );

    for row in &data.rows {
        assert!(
            !row.path.contains('\\'),
            "paths should be forward-slash normalized: {}",
            row.path
        );
    }
}

// ===========================================================================
// Scan with various file types
// ===========================================================================

#[test]
fn scan_javascript_file() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("app.js"),
        "function greet() {\n  return 'hi';\n}\n",
    )
    .unwrap();

    let opts = default_scan_options();
    let paths = vec![dir.path().to_path_buf()];

    let languages = tokmd_scan::scan(&paths, &opts).unwrap();
    let report = tokmd_model::create_lang_report(&languages, 0, false, ChildrenMode::Collapse);

    let langs: Vec<&str> = report.rows.iter().map(|r| r.lang.as_str()).collect();
    assert!(langs.contains(&"JavaScript"));
}

#[test]
fn scan_empty_directory_produces_empty_report() {
    let dir = TempDir::new().unwrap();
    let opts = default_scan_options();
    let paths = vec![dir.path().to_path_buf()];

    let languages = tokmd_scan::scan(&paths, &opts).unwrap();
    let report = tokmd_model::create_lang_report(&languages, 0, false, ChildrenMode::Collapse);

    assert!(report.rows.is_empty(), "empty dir should have no rows");
    assert_eq!(report.total.code, 0);
    assert_eq!(report.total.files, 0);
}
