//! Depth tests for tokmd-model aggregation logic.
//!
//! Covers: duplicate language aggregation, sorting invariants,
//! children mode (Collapse vs Separate), module aggregation at
//! various depths, file-level export rows, empty scan results,
//! deterministic output, and BTreeMap ordering.

use std::path::PathBuf;
use tokei::{Config, Languages};
use tokmd_model::{
    avg, collect_file_rows, create_export_data, create_lang_report, create_module_report,
    module_key,
};
use tokmd_types::{ChildIncludeMode, ChildrenMode, FileKind};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn scan(path: &str) -> Languages {
    let mut languages = Languages::new();
    languages.get_statistics(&[PathBuf::from(path)], &[], &Config::default());
    languages
}

fn crate_src() -> String {
    format!("{}/src", env!("CARGO_MANIFEST_DIR"))
}

fn empty_languages() -> Languages {
    Languages::new()
}

// ===========================================================================
// 1. Aggregation with duplicate languages
// ===========================================================================

#[test]
fn lang_report_aggregates_all_files_of_same_language() {
    let langs = scan(&crate_src());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);

    // The model crate is pure Rust — all files should aggregate into one row
    let rust_rows: Vec<_> = report.rows.iter().filter(|r| r.lang == "Rust").collect();
    assert_eq!(
        rust_rows.len(),
        1,
        "all Rust files should aggregate into a single row"
    );
    assert!(
        rust_rows[0].files > 1 || rust_rows[0].code > 0,
        "aggregated Rust row should have data"
    );
}

#[test]
fn lang_report_total_code_matches_sum_of_rows() {
    let langs = scan(&crate_src());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);

    let sum: usize = report.rows.iter().map(|r| r.code).sum();
    assert_eq!(
        report.total.code, sum,
        "total code must equal sum of row codes"
    );
}

#[test]
fn lang_report_total_lines_matches_sum_of_rows() {
    let langs = scan(&crate_src());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);

    let sum: usize = report.rows.iter().map(|r| r.lines).sum();
    assert_eq!(report.total.lines, sum);
}

// ===========================================================================
// 2. Sorting: code descending, name ascending tie-break
// ===========================================================================

#[test]
fn lang_report_sorted_by_code_descending() {
    let langs = scan(&crate_src());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);

    for w in report.rows.windows(2) {
        assert!(
            w[0].code >= w[1].code,
            "rows must be sorted by code descending: {} ({}) >= {} ({})",
            w[0].lang,
            w[0].code,
            w[1].lang,
            w[1].code
        );
    }
}

#[test]
fn lang_report_name_tiebreak_ascending() {
    let langs = scan(&crate_src());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);

    for w in report.rows.windows(2) {
        if w[0].code == w[1].code {
            assert!(
                w[0].lang <= w[1].lang,
                "equal-code rows must sort by name ascending: '{}' <= '{}'",
                w[0].lang,
                w[1].lang
            );
        }
    }
}

#[test]
fn module_report_sorted_by_code_descending() {
    let langs = scan(&crate_src());
    let report = create_module_report(
        &langs,
        &["crates".to_string()],
        2,
        ChildIncludeMode::Separate,
        0,
    );

    for w in report.rows.windows(2) {
        assert!(
            w[0].code >= w[1].code,
            "module rows must be sorted by code descending: {} ({}) >= {} ({})",
            w[0].module,
            w[0].code,
            w[1].module,
            w[1].code
        );
    }
}

#[test]
fn export_rows_sorted_by_code_descending() {
    let langs = scan(&crate_src());
    let data = create_export_data(
        &langs,
        &["crates".to_string()],
        2,
        ChildIncludeMode::Separate,
        None,
        0,
        0,
    );

    for w in data.rows.windows(2) {
        assert!(
            w[0].code >= w[1].code,
            "export rows must be sorted by code descending: {} ({}) >= {} ({})",
            w[0].path,
            w[0].code,
            w[1].path,
            w[1].code
        );
    }
}

// ===========================================================================
// 3. Children mode collapse: embedded langs merge into parent
// ===========================================================================

#[test]
fn collapse_mode_no_embedded_rows() {
    let langs = scan(&crate_src());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);

    for row in &report.rows {
        assert!(
            !row.lang.contains("(embedded)"),
            "collapse mode must not produce embedded rows: '{}'",
            row.lang
        );
    }
}

#[test]
fn collapse_mode_all_rows_have_bytes() {
    let langs = scan(&crate_src());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);

    for row in &report.rows {
        if row.code > 0 {
            assert!(
                row.bytes > 0,
                "collapse mode: row '{}' with code > 0 should have bytes > 0",
                row.lang
            );
        }
    }
}

// ===========================================================================
// 4. Children mode separate: embedded shown as "(embedded)"
// ===========================================================================

#[test]
fn separate_mode_embedded_rows_have_zero_bytes() {
    let langs = scan(&crate_src());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Separate);

    for row in &report.rows {
        if row.lang.contains("(embedded)") {
            assert_eq!(
                row.bytes, 0,
                "embedded row '{}' must have 0 bytes",
                row.lang
            );
            assert_eq!(
                row.tokens, 0,
                "embedded row '{}' must have 0 tokens",
                row.lang
            );
        }
    }
}

#[test]
fn separate_mode_parent_rows_have_bytes() {
    let langs = scan(&crate_src());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Separate);

    for row in &report.rows {
        if !row.lang.contains("(embedded)") && row.code > 0 {
            assert!(
                row.bytes > 0,
                "parent row '{}' with code > 0 should have bytes > 0",
                row.lang
            );
        }
    }
}

#[test]
fn separate_mode_total_code_still_matches_row_sum() {
    let langs = scan(&crate_src());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Separate);

    let sum: usize = report.rows.iter().map(|r| r.code).sum();
    assert_eq!(
        report.total.code, sum,
        "separate mode: total code = sum of rows"
    );
}

// ===========================================================================
// 5. Module aggregation at different depths
// ===========================================================================

#[test]
fn module_key_root_file_is_root() {
    assert_eq!(
        module_key("Cargo.toml", &["crates".to_string()], 2),
        "(root)"
    );
}

#[test]
fn module_key_depth_one() {
    assert_eq!(
        module_key("crates/tokmd-model/src/lib.rs", &["crates".to_string()], 1),
        "crates"
    );
}

#[test]
fn module_key_depth_two() {
    assert_eq!(
        module_key("crates/tokmd-model/src/lib.rs", &["crates".to_string()], 2),
        "crates/tokmd-model"
    );
}

#[test]
fn module_key_non_root_dir() {
    assert_eq!(module_key("src/main.rs", &["crates".to_string()], 2), "src");
}

#[test]
fn module_report_at_depth_two_groups_by_subcrate() {
    let langs = scan(&crate_src());
    let report = create_module_report(
        &langs,
        &["crates".to_string()],
        2,
        ChildIncludeMode::Separate,
        0,
    );

    // Scanning the model crate's src, everything should be in one module
    assert!(
        !report.rows.is_empty(),
        "module report should have at least one row"
    );
}

// ===========================================================================
// 6. File-level export rows
// ===========================================================================

#[test]
fn export_rows_have_parent_kind() {
    let langs = scan(&crate_src());
    let data = create_export_data(
        &langs,
        &["crates".to_string()],
        2,
        ChildIncludeMode::ParentsOnly,
        None,
        0,
        0,
    );

    for row in &data.rows {
        assert_eq!(
            row.kind,
            FileKind::Parent,
            "ParentsOnly mode should only produce Parent rows, got {:?} for '{}'",
            row.kind,
            row.path
        );
    }
}

#[test]
fn export_rows_lines_equals_code_plus_comments_plus_blanks() {
    let langs = scan(&crate_src());
    let data = create_export_data(
        &langs,
        &["crates".to_string()],
        2,
        ChildIncludeMode::Separate,
        None,
        0,
        0,
    );

    for row in &data.rows {
        assert_eq!(
            row.lines,
            row.code + row.comments + row.blanks,
            "lines = code + comments + blanks for '{}'",
            row.path
        );
    }
}

#[test]
fn collect_file_rows_returns_nonempty_for_real_code() {
    let langs = scan(&crate_src());
    let rows = collect_file_rows(
        &langs,
        &["crates".to_string()],
        2,
        ChildIncludeMode::Separate,
        None,
    );
    assert!(!rows.is_empty(), "collect_file_rows should find files");
}

// ===========================================================================
// 7. Empty scan results
// ===========================================================================

#[test]
fn empty_scan_lang_report_has_zero_rows() {
    let langs = empty_languages();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);

    assert!(report.rows.is_empty());
    assert_eq!(report.total.code, 0);
    assert_eq!(report.total.files, 0);
}

#[test]
fn empty_scan_module_report_has_zero_rows() {
    let langs = empty_languages();
    let report = create_module_report(
        &langs,
        &["crates".to_string()],
        2,
        ChildIncludeMode::Separate,
        0,
    );

    assert!(report.rows.is_empty());
    assert_eq!(report.total.code, 0);
}

#[test]
fn empty_scan_export_data_has_zero_rows() {
    let langs = empty_languages();
    let data = create_export_data(
        &langs,
        &["crates".to_string()],
        2,
        ChildIncludeMode::Separate,
        None,
        0,
        0,
    );

    assert!(data.rows.is_empty());
}

#[test]
fn avg_zero_files_returns_zero() {
    assert_eq!(avg(100, 0), 0);
}

#[test]
fn avg_rounds_correctly() {
    // 7 lines / 2 files = 3.5 → rounded to 4
    assert_eq!(avg(7, 2), 4);
    // 10 lines / 3 files = 3.33 → rounded to 3
    assert_eq!(avg(10, 3), 3);
}

// ===========================================================================
// 8. Deterministic output across multiple aggregations
// ===========================================================================

#[test]
fn lang_report_deterministic() {
    let langs1 = scan(&crate_src());
    let langs2 = scan(&crate_src());

    let report1 = create_lang_report(&langs1, 0, false, ChildrenMode::Collapse);
    let report2 = create_lang_report(&langs2, 0, false, ChildrenMode::Collapse);

    assert_eq!(report1.rows.len(), report2.rows.len());
    for (a, b) in report1.rows.iter().zip(report2.rows.iter()) {
        assert_eq!(a.lang, b.lang, "language names must match");
        assert_eq!(a.code, b.code, "code counts must match for {}", a.lang);
        assert_eq!(a.lines, b.lines, "line counts must match for {}", a.lang);
    }
}

#[test]
fn module_report_deterministic() {
    let langs1 = scan(&crate_src());
    let langs2 = scan(&crate_src());

    let r1 = create_module_report(
        &langs1,
        &["crates".to_string()],
        2,
        ChildIncludeMode::Separate,
        0,
    );
    let r2 = create_module_report(
        &langs2,
        &["crates".to_string()],
        2,
        ChildIncludeMode::Separate,
        0,
    );

    assert_eq!(r1.rows.len(), r2.rows.len());
    for (a, b) in r1.rows.iter().zip(r2.rows.iter()) {
        assert_eq!(a.module, b.module);
        assert_eq!(a.code, b.code);
    }
}

#[test]
fn export_data_deterministic() {
    let langs1 = scan(&crate_src());
    let langs2 = scan(&crate_src());

    let d1 = create_export_data(
        &langs1,
        &["crates".to_string()],
        2,
        ChildIncludeMode::Separate,
        None,
        0,
        0,
    );
    let d2 = create_export_data(
        &langs2,
        &["crates".to_string()],
        2,
        ChildIncludeMode::Separate,
        None,
        0,
        0,
    );

    assert_eq!(d1.rows.len(), d2.rows.len());
    for (a, b) in d1.rows.iter().zip(d2.rows.iter()) {
        assert_eq!(a.path, b.path);
        assert_eq!(a.code, b.code);
    }
}

// ===========================================================================
// 9. BTreeMap ordering in aggregated results
// ===========================================================================

#[test]
fn file_rows_are_grouped_by_path_lang_kind() {
    let langs = scan(&crate_src());
    let rows = collect_file_rows(
        &langs,
        &["crates".to_string()],
        2,
        ChildIncludeMode::Separate,
        None,
    );

    // No duplicate (path, lang, kind) combinations
    let mut seen = std::collections::BTreeSet::new();
    for r in &rows {
        let key = (r.path.clone(), r.lang.clone(), format!("{:?}", r.kind));
        assert!(seen.insert(key.clone()), "duplicate file row: {:?}", key);
    }
}

#[test]
fn file_rows_paths_use_forward_slashes() {
    let langs = scan(&crate_src());
    let rows = collect_file_rows(
        &langs,
        &["crates".to_string()],
        2,
        ChildIncludeMode::Separate,
        None,
    );

    for r in &rows {
        assert!(
            !r.path.contains('\\'),
            "path must use forward slashes: '{}'",
            r.path
        );
    }
}
