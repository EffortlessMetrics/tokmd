//! Insta snapshot tests for tokmd-model aggregated output.
//!
//! These tests capture the *shape* and determinism of receipts so that
//! any accidental structural change is caught.

use std::path::PathBuf;
use tokei::{Config, Languages};
use tokmd_model::{
    collect_file_rows, create_export_data, create_lang_report, create_module_report,
};
use tokmd_types::{ChildIncludeMode, ChildrenMode};

/// Scan a directory and return Languages data.
fn scan(path: &str) -> Languages {
    let mut languages = Languages::new();
    languages.get_statistics(&[PathBuf::from(path)], &[], &Config::default());
    languages
}

fn crate_src() -> String {
    format!("{}/src", env!("CARGO_MANIFEST_DIR"))
}

// ─── Helpers to produce redacted (line-count-stable) snapshots ───

/// Strip volatile values (bytes, tokens, absolute counts) and keep only
/// the structural shape (field names, ordering, relative ranking).
fn redact_lang_rows(
    report: &tokmd_types::LangReport,
) -> Vec<serde_json::Value> {
    report
        .rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "lang": r.lang,
                "code_gt_zero": r.code > 0,
                "lines_gte_code": r.lines >= r.code,
                "files_gt_zero": r.files > 0,
            })
        })
        .collect()
}

fn redact_module_rows(
    report: &tokmd_types::ModuleReport,
) -> Vec<serde_json::Value> {
    report
        .rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "module": r.module,
                "code_gt_zero": r.code > 0,
                "lines_gte_code": r.lines >= r.code,
                "files_gt_zero": r.files > 0,
            })
        })
        .collect()
}

fn redact_file_rows(
    rows: &[tokmd_types::FileRow],
) -> Vec<serde_json::Value> {
    rows.iter()
        .map(|r| {
            serde_json::json!({
                "path": r.path,
                "module": r.module,
                "lang": r.lang,
                "kind": format!("{:?}", r.kind),
                "code_gt_zero": r.code > 0,
                "lines_eq_sum": r.lines == r.code + r.comments + r.blanks,
            })
        })
        .collect()
}

// ─── Lang report snapshots ───

#[test]
fn snapshot_lang_report_collapse_shape() {
    let langs = scan(&crate_src());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let redacted = redact_lang_rows(&report);
    insta::assert_json_snapshot!("lang_collapse_shape", redacted);
}

#[test]
fn snapshot_lang_report_separate_shape() {
    let langs = scan(&crate_src());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Separate);
    let redacted = redact_lang_rows(&report);
    insta::assert_json_snapshot!("lang_separate_shape", redacted);
}

#[test]
fn snapshot_lang_report_top1() {
    let langs = scan(&crate_src());
    let report = create_lang_report(&langs, 1, false, ChildrenMode::Collapse);
    let redacted = redact_lang_rows(&report);
    insta::assert_json_snapshot!("lang_top1", redacted);
}

// ─── Module report snapshots ───

#[test]
fn snapshot_module_report_shape() {
    let langs = scan(&crate_src());
    let report = create_module_report(&langs, &[], 2, ChildIncludeMode::ParentsOnly, 0);
    let redacted = redact_module_rows(&report);
    insta::assert_json_snapshot!("module_shape", redacted);
}

// ─── Export / file-row snapshots ───

#[test]
fn snapshot_file_rows_parents_only() {
    let langs = scan(&crate_src());
    let rows = collect_file_rows(&langs, &[], 2, ChildIncludeMode::ParentsOnly, None);
    let redacted = redact_file_rows(&rows);
    insta::assert_json_snapshot!("file_rows_parents_only", redacted);
}

#[test]
fn snapshot_file_rows_separate() {
    let langs = scan(&crate_src());
    let rows = collect_file_rows(&langs, &[], 2, ChildIncludeMode::Separate, None);
    let redacted = redact_file_rows(&rows);
    insta::assert_json_snapshot!("file_rows_separate", redacted);
}

// ─── Empty input snapshots ───

#[test]
fn snapshot_empty_lang_report() {
    let langs = Languages::new();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    insta::assert_json_snapshot!("empty_lang_report", serde_json::json!({
        "rows": report.rows.len(),
        "total_code": report.total.code,
        "total_lines": report.total.lines,
        "total_bytes": report.total.bytes,
        "total_tokens": report.total.tokens,
        "total_files": report.total.files,
    }));
}

#[test]
fn snapshot_empty_module_report() {
    let langs = Languages::new();
    let report = create_module_report(&langs, &[], 2, ChildIncludeMode::ParentsOnly, 0);
    insta::assert_json_snapshot!("empty_module_report", serde_json::json!({
        "rows": report.rows.len(),
        "total_code": report.total.code,
        "total_files": report.total.files,
    }));
}

#[test]
fn snapshot_empty_export() {
    let langs = Languages::new();
    let data = create_export_data(&langs, &[], 2, ChildIncludeMode::ParentsOnly, None, 0, 0);
    insta::assert_json_snapshot!("empty_export", serde_json::json!({
        "rows": data.rows.len(),
    }));
}
