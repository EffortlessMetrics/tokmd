//! Feature-stability tests for WASM readiness seams.
//!
//! These tests verify that tokmd-format works correctly WITHOUT optional
//! features. They must NOT use `#[cfg(feature = ...)]` guards.

use std::path::PathBuf;
use tokmd_format::*;
use tokmd_types::*;

// ── Helpers ───────────────────────────────────────────────────────────

fn empty_lang_report() -> LangReport {
    LangReport {
        rows: vec![],
        total: Totals {
            code: 0,
            lines: 0,
            files: 0,
            bytes: 0,
            tokens: 0,
            avg_lines: 0,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

fn sample_lang_report() -> LangReport {
    LangReport {
        rows: vec![LangRow {
            lang: "Rust".into(),
            code: 100,
            lines: 150,
            files: 3,
            bytes: 5000,
            tokens: 1200,
            avg_lines: 50,
        }],
        total: Totals {
            code: 100,
            lines: 150,
            files: 3,
            bytes: 5000,
            tokens: 1200,
            avg_lines: 50,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

fn empty_module_report() -> ModuleReport {
    ModuleReport {
        rows: vec![],
        total: Totals {
            code: 0,
            lines: 0,
            files: 0,
            bytes: 0,
            tokens: 0,
            avg_lines: 0,
        },
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
        top: 0,
    }
}

fn empty_export_data() -> ExportData {
    ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

fn md_lang_args() -> LangArgs {
    LangArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Md,
        top: 0,
        files: false,
        children: ChildrenMode::Collapse,
    }
}

fn tsv_lang_args() -> LangArgs {
    LangArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Tsv,
        top: 0,
        files: false,
        children: ChildrenMode::Collapse,
    }
}

fn json_lang_args() -> LangArgs {
    LangArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Json,
        top: 0,
        files: false,
        children: ChildrenMode::Collapse,
    }
}

fn md_module_args() -> ModuleArgs {
    ModuleArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Md,
        top: 0,
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

fn default_scan_options() -> tokmd_settings::ScanOptions {
    tokmd_settings::ScanOptions::default()
}

fn csv_export_args() -> ExportArgs {
    ExportArgs {
        paths: vec![PathBuf::from(".")],
        format: ExportFormat::Csv,
        output: None,
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        redact: RedactMode::None,
        meta: false,
        strip_prefix: None,
    }
}

fn jsonl_export_args() -> ExportArgs {
    ExportArgs {
        format: ExportFormat::Jsonl,
        ..csv_export_args()
    }
}

fn json_export_args() -> ExportArgs {
    ExportArgs {
        format: ExportFormat::Json,
        ..csv_export_args()
    }
}

// ── Markdown rendering ────────────────────────────────────────────────

#[test]
fn markdown_lang_empty_data() {
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &empty_lang_report(),
        &default_scan_options(),
        &md_lang_args(),
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    // Empty report produces output (may be header-only or empty)
    // The key assertion: no panic, valid UTF-8
    let _ = output;
}

#[test]
fn markdown_lang_with_data() {
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &sample_lang_report(),
        &default_scan_options(),
        &md_lang_args(),
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("Rust"));
}

#[test]
fn markdown_module_empty_data() {
    let mut buf = Vec::new();
    write_module_report_to(
        &mut buf,
        &empty_module_report(),
        &default_scan_options(),
        &md_module_args(),
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    // Empty report produces output (may be header-only or empty)
    let _ = output;
}

// ── TSV rendering ─────────────────────────────────────────────────────

#[test]
fn tsv_lang_empty_data() {
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &empty_lang_report(),
        &default_scan_options(),
        &tsv_lang_args(),
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    // TSV output should have header with tab separators
    assert!(output.contains('\t'));
}

#[test]
fn tsv_lang_with_data() {
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &sample_lang_report(),
        &default_scan_options(),
        &tsv_lang_args(),
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("Rust"));
    assert!(output.contains('\t'));
}

// ── JSON rendering ────────────────────────────────────────────────────

#[test]
fn json_lang_empty_data() {
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &empty_lang_report(),
        &default_scan_options(),
        &json_lang_args(),
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(parsed.is_object());
}

#[test]
fn json_lang_with_data() {
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &sample_lang_report(),
        &default_scan_options(),
        &json_lang_args(),
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(parsed.is_object());
}

// ── Export rendering ──────────────────────────────────────────────────

#[test]
fn csv_export_empty_data() {
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &empty_export_data(), &csv_export_args()).unwrap();
    let output = String::from_utf8(buf).unwrap();
    // CSV should at least produce a header line
    assert!(output.contains("path"));
}

#[test]
fn jsonl_export_empty_data() {
    let mut buf = Vec::new();
    write_export_jsonl_to(
        &mut buf,
        &empty_export_data(),
        &default_scan_options(),
        &jsonl_export_args(),
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    // Empty data produces no JSONL lines
    assert!(output.is_empty());
}

#[test]
fn json_export_empty_data() {
    let mut buf = Vec::new();
    write_export_json_to(
        &mut buf,
        &empty_export_data(),
        &default_scan_options(),
        &json_export_args(),
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    // Empty export may produce empty string or valid JSON
    if !output.is_empty() {
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.is_object() || parsed.is_array());
    }
}

// ── Diff functions ────────────────────────────────────────────────────

#[test]
fn compute_diff_rows_empty_reports() {
    let rows = compute_diff_rows(&empty_lang_report(), &empty_lang_report());
    assert!(rows.is_empty());
}

#[test]
fn compute_diff_totals_empty_rows() {
    let totals = compute_diff_totals(&[]);
    assert_eq!(totals.delta_code, 0);
}

#[test]
fn render_diff_md_empty() {
    let output = render_diff_md("a.json", "b.json", &[], &DiffTotals::default());
    assert!(output.contains("a.json"));
    assert!(output.contains("b.json"));
}

#[test]
fn create_diff_receipt_construction() {
    let receipt = create_diff_receipt("from.json", "to.json", vec![], DiffTotals::default());
    assert_eq!(receipt.schema_version, SCHEMA_VERSION);
    assert_eq!(receipt.from_source, "from.json");
    assert_eq!(receipt.to_source, "to.json");
}
