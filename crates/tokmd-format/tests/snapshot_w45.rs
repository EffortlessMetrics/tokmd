//! Snapshot tests for tokmd-format – wave 45.
//!
//! Covers output format gaps: module TSV/JSON single-row, export CSV
//! single-file, diff JSON no-changes, and top-N lang filtering.

use std::path::PathBuf;

use tokmd_format::{
    compute_diff_rows, compute_diff_totals, create_diff_receipt, render_diff_md,
    write_export_csv_to, write_export_cyclonedx_with_options, write_lang_report_to,
    write_module_report_to,
};
use tokmd_settings::{ChildIncludeMode, ChildrenMode, ScanOptions};
use tokmd_types::{
    ExportArgs, ExportData, ExportFormat, FileKind, FileRow, LangArgs, LangReport, LangRow,
    ModuleArgs, ModuleReport, ModuleRow, RedactMode, TableFormat, Totals,
};

// ---------------------------------------------------------------------------
// Shared fixtures
// ---------------------------------------------------------------------------

fn single_module_report() -> ModuleReport {
    ModuleReport {
        rows: vec![ModuleRow {
            module: "src".into(),
            code: 800,
            lines: 1000,
            files: 8,
            bytes: 32000,
            tokens: 2000,
            avg_lines: 125,
        }],
        total: Totals {
            code: 800,
            lines: 1000,
            files: 8,
            bytes: 32000,
            tokens: 2000,
            avg_lines: 125,
        },
        module_roots: vec!["src".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        top: 0,
    }
}

fn multi_lang_report() -> LangReport {
    LangReport {
        rows: vec![
            LangRow {
                lang: "Rust".into(),
                code: 3000,
                lines: 3600,
                files: 30,
                bytes: 120000,
                tokens: 7500,
                avg_lines: 120,
            },
            LangRow {
                lang: "Python".into(),
                code: 1500,
                lines: 1800,
                files: 12,
                bytes: 54000,
                tokens: 3750,
                avg_lines: 150,
            },
            LangRow {
                lang: "TOML".into(),
                code: 200,
                lines: 250,
                files: 5,
                bytes: 6000,
                tokens: 500,
                avg_lines: 50,
            },
            LangRow {
                lang: "Markdown".into(),
                code: 100,
                lines: 150,
                files: 3,
                bytes: 4500,
                tokens: 250,
                avg_lines: 50,
            },
        ],
        total: Totals {
            code: 4800,
            lines: 5800,
            files: 50,
            bytes: 184500,
            tokens: 12000,
            avg_lines: 116,
        },
        with_files: true,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

fn single_file_export() -> ExportData {
    ExportData {
        rows: vec![FileRow {
            path: "src/main.rs".into(),
            module: "src".into(),
            lang: "Rust".into(),
            kind: FileKind::Parent,
            code: 120,
            comments: 15,
            blanks: 10,
            lines: 145,
            bytes: 4350,
            tokens: 300,
        }],
        module_roots: vec!["src".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn multi_file_export() -> ExportData {
    ExportData {
        rows: vec![
            FileRow {
                path: "src/lib.rs".into(),
                module: "src".into(),
                lang: "Rust".into(),
                kind: FileKind::Parent,
                code: 500,
                comments: 80,
                blanks: 40,
                lines: 620,
                bytes: 18600,
                tokens: 1250,
            },
            FileRow {
                path: "src/utils.rs".into(),
                module: "src".into(),
                lang: "Rust".into(),
                kind: FileKind::Parent,
                code: 200,
                comments: 30,
                blanks: 20,
                lines: 250,
                bytes: 7500,
                tokens: 500,
            },
            FileRow {
                path: "tests/integration.rs".into(),
                module: "tests".into(),
                lang: "Rust".into(),
                kind: FileKind::Parent,
                code: 150,
                comments: 10,
                blanks: 15,
                lines: 175,
                bytes: 5250,
                tokens: 375,
            },
        ],
        module_roots: vec!["src".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

// ── module TSV single row ─────────────────────────────────────────────

#[test]
fn snapshot_module_tsv_single() {
    let report = single_module_report();
    let args = ModuleArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Tsv,
        top: 0,
        module_roots: vec!["src".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let mut buf = Vec::new();
    write_module_report_to(&mut buf, &report, &ScanOptions::default(), &args).unwrap();
    insta::assert_snapshot!(String::from_utf8(buf).unwrap());
}

// ── module JSON single row ────────────────────────────────────────────

#[test]
fn snapshot_module_json_single() {
    let report = single_module_report();
    let args = ModuleArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Json,
        top: 0,
        module_roots: vec!["src".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let mut buf = Vec::new();
    write_module_report_to(&mut buf, &report, &ScanOptions::default(), &args).unwrap();
    let raw = String::from_utf8(buf).unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    v["generated_at_ms"] = serde_json::json!(0);
    v["tool"]["version"] = serde_json::json!("0.0.0-test");
    insta::assert_json_snapshot!(v);
}

// ── export CSV single file ────────────────────────────────────────────

#[test]
fn snapshot_export_csv_single_file() {
    let data = single_file_export();
    let args = ExportArgs {
        paths: vec![PathBuf::from(".")],
        format: ExportFormat::Csv,
        output: None,
        module_roots: vec!["src".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        redact: RedactMode::None,
        meta: false,
        strip_prefix: None,
    };
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &data, &args).unwrap();
    insta::assert_snapshot!(String::from_utf8(buf).unwrap());
}

// ── export CSV multi-file ─────────────────────────────────────────────

#[test]
fn snapshot_export_csv_multi_file() {
    let data = multi_file_export();
    let args = ExportArgs {
        paths: vec![PathBuf::from(".")],
        format: ExportFormat::Csv,
        output: None,
        module_roots: vec!["src".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        redact: RedactMode::None,
        meta: false,
        strip_prefix: None,
    };
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &data, &args).unwrap();
    insta::assert_snapshot!(String::from_utf8(buf).unwrap());
}

// ── export CycloneDX single file ──────────────────────────────────────

#[test]
fn snapshot_export_cyclonedx_single_file() {
    let data = single_file_export();
    let mut buf = Vec::new();
    write_export_cyclonedx_with_options(
        &mut buf,
        &data,
        RedactMode::None,
        Some("urn:uuid:00000000-0000-0000-0000-000000000000".to_string()),
        Some("2024-01-01T00:00:00Z".to_string()),
    )
    .unwrap();
    insta::assert_snapshot!(String::from_utf8(buf).unwrap());
}

// ── diff JSON no changes ──────────────────────────────────────────────

#[test]
fn snapshot_diff_json_no_changes() {
    let report = LangReport {
        rows: vec![LangRow {
            lang: "Rust".into(),
            code: 500,
            lines: 600,
            files: 5,
            bytes: 25000,
            tokens: 1250,
            avg_lines: 120,
        }],
        total: Totals {
            code: 500,
            lines: 600,
            files: 5,
            bytes: 25000,
            tokens: 1250,
            avg_lines: 120,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let rows = compute_diff_rows(&report, &report);
    let totals = compute_diff_totals(&rows);
    let receipt = create_diff_receipt("v1.0.0", "v1.0.0", rows, totals);
    let raw = serde_json::to_string(&receipt).unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    v["generated_at_ms"] = serde_json::json!(0);
    v["tool"]["version"] = serde_json::json!("0.0.0-test");
    let pretty = serde_json::to_string_pretty(&v).unwrap();
    insta::assert_snapshot!(pretty);
}

// ── lang top-N filtering ──────────────────────────────────────────────

#[test]
fn snapshot_lang_md_top2() {
    let report = multi_lang_report();
    let args = LangArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Md,
        top: 2,
        files: true,
        children: ChildrenMode::Collapse,
    };
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &ScanOptions::default(), &args).unwrap();
    insta::assert_snapshot!(String::from_utf8(buf).unwrap());
}

// ── diff md with new language added ───────────────────────────────────

#[test]
fn snapshot_diff_md_new_language() {
    let from = LangReport {
        rows: vec![LangRow {
            lang: "Rust".into(),
            code: 1000,
            lines: 1200,
            files: 10,
            bytes: 40000,
            tokens: 2500,
            avg_lines: 120,
        }],
        total: Totals {
            code: 1000,
            lines: 1200,
            files: 10,
            bytes: 40000,
            tokens: 2500,
            avg_lines: 120,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let to = LangReport {
        rows: vec![
            LangRow {
                lang: "Rust".into(),
                code: 1200,
                lines: 1440,
                files: 12,
                bytes: 48000,
                tokens: 3000,
                avg_lines: 120,
            },
            LangRow {
                lang: "Python".into(),
                code: 300,
                lines: 360,
                files: 3,
                bytes: 10800,
                tokens: 750,
                avg_lines: 120,
            },
        ],
        total: Totals {
            code: 1500,
            lines: 1800,
            files: 15,
            bytes: 58800,
            tokens: 3750,
            avg_lines: 120,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let md = render_diff_md("v1.0.0", "v2.0.0", &rows, &totals);
    insta::assert_snapshot!(md);
}
