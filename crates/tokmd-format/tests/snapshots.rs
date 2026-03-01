//! Integration-level snapshot tests for every public format renderer.
//!
//! Each test exercises the public `write_*` API and pins the output
//! with an insta snapshot so regressions are caught at review time.

use std::path::PathBuf;

use tokmd_format::{
    DiffColorMode, DiffRenderOptions, compute_diff_rows, compute_diff_totals, create_diff_receipt,
    render_diff_md, render_diff_md_with_options, write_export_csv_to,
    write_export_cyclonedx_with_options, write_export_json_to, write_export_jsonl_to,
    write_lang_report_to, write_module_report_to,
};
use tokmd_settings::{ChildIncludeMode, ChildrenMode, ScanOptions};
use tokmd_types::{
    ExportArgs, ExportData, ExportFormat, FileKind, FileRow, LangArgs, LangReport, LangRow,
    ModuleArgs, ModuleReport, ModuleRow, RedactMode, TableFormat, Totals,
};

// ---------------------------------------------------------------------------
// Shared fixtures
// ---------------------------------------------------------------------------

fn lang_report(with_files: bool) -> LangReport {
    LangReport {
        rows: vec![
            LangRow {
                lang: "Rust".into(),
                code: 1000,
                lines: 1200,
                files: 10,
                bytes: 50000,
                tokens: 2500,
                avg_lines: 120,
            },
            LangRow {
                lang: "TOML".into(),
                code: 50,
                lines: 60,
                files: 2,
                bytes: 1000,
                tokens: 125,
                avg_lines: 30,
            },
        ],
        total: Totals {
            code: 1050,
            lines: 1260,
            files: 12,
            bytes: 51000,
            tokens: 2625,
            avg_lines: 105,
        },
        with_files,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

fn module_report() -> ModuleReport {
    ModuleReport {
        rows: vec![
            ModuleRow {
                module: "crates/alpha".into(),
                code: 800,
                lines: 950,
                files: 8,
                bytes: 40000,
                tokens: 2000,
                avg_lines: 119,
            },
            ModuleRow {
                module: "crates/beta".into(),
                code: 200,
                lines: 250,
                files: 2,
                bytes: 10000,
                tokens: 500,
                avg_lines: 125,
            },
        ],
        total: Totals {
            code: 1000,
            lines: 1200,
            files: 10,
            bytes: 50000,
            tokens: 2500,
            avg_lines: 120,
        },
        module_roots: vec!["crates".into()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
        top: 0,
    }
}

fn file_rows() -> Vec<FileRow> {
    vec![
        FileRow {
            path: "src/lib.rs".into(),
            module: "src".into(),
            lang: "Rust".into(),
            kind: FileKind::Parent,
            code: 100,
            comments: 20,
            blanks: 10,
            lines: 130,
            bytes: 1000,
            tokens: 250,
        },
        FileRow {
            path: "src/util.rs".into(),
            module: "src".into(),
            lang: "Rust".into(),
            kind: FileKind::Parent,
            code: 60,
            comments: 5,
            blanks: 5,
            lines: 70,
            bytes: 600,
            tokens: 150,
        },
        FileRow {
            path: "tests/smoke.rs".into(),
            module: "tests".into(),
            lang: "Rust".into(),
            kind: FileKind::Parent,
            code: 40,
            comments: 2,
            blanks: 3,
            lines: 45,
            bytes: 400,
            tokens: 100,
        },
    ]
}

fn export_data() -> ExportData {
    ExportData {
        rows: file_rows(),
        module_roots: vec!["src".into(), "tests".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn global() -> ScanOptions {
    ScanOptions::default()
}

// ---------------------------------------------------------------------------
// Lang — Markdown
// ---------------------------------------------------------------------------

#[test]
fn snapshot_lang_md_without_files() {
    let mut buf = Vec::new();
    let args = LangArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Md,
        top: 0,
        files: false,
        children: ChildrenMode::Collapse,
    };
    write_lang_report_to(&mut buf, &lang_report(false), &global(), &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    insta::assert_snapshot!("lang_md_no_files", output);
}

#[test]
fn snapshot_lang_md_with_files() {
    let mut buf = Vec::new();
    let args = LangArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Md,
        top: 0,
        files: true,
        children: ChildrenMode::Collapse,
    };
    write_lang_report_to(&mut buf, &lang_report(true), &global(), &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    insta::assert_snapshot!("lang_md_with_files", output);
}

// ---------------------------------------------------------------------------
// Lang — TSV
// ---------------------------------------------------------------------------

#[test]
fn snapshot_lang_tsv_without_files() {
    let mut buf = Vec::new();
    let args = LangArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Tsv,
        top: 0,
        files: false,
        children: ChildrenMode::Collapse,
    };
    write_lang_report_to(&mut buf, &lang_report(false), &global(), &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    insta::assert_snapshot!("lang_tsv_no_files", output);
}

#[test]
fn snapshot_lang_tsv_with_files() {
    let mut buf = Vec::new();
    let args = LangArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Tsv,
        top: 0,
        files: true,
        children: ChildrenMode::Collapse,
    };
    write_lang_report_to(&mut buf, &lang_report(true), &global(), &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    insta::assert_snapshot!("lang_tsv_with_files", output);
}

// ---------------------------------------------------------------------------
// Lang — JSON (timestamp-redacted)
// ---------------------------------------------------------------------------

#[test]
fn snapshot_lang_json() {
    let mut buf = Vec::new();
    let args = LangArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Json,
        top: 0,
        files: false,
        children: ChildrenMode::Collapse,
    };
    write_lang_report_to(&mut buf, &lang_report(false), &global(), &args).unwrap();
    let raw = String::from_utf8(buf).unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    // Normalise non-deterministic fields
    v["generated_at_ms"] = serde_json::json!(0);
    v["tool"]["version"] = serde_json::json!("0.0.0");
    let pretty = serde_json::to_string_pretty(&v).unwrap();
    insta::assert_snapshot!("lang_json", pretty);
}

// ---------------------------------------------------------------------------
// Module — Markdown
// ---------------------------------------------------------------------------

#[test]
fn snapshot_module_md() {
    let mut buf = Vec::new();
    let args = ModuleArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Md,
        top: 0,
        module_roots: vec!["crates".into()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };
    write_module_report_to(&mut buf, &module_report(), &global(), &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    insta::assert_snapshot!("module_md", output);
}

// ---------------------------------------------------------------------------
// Module — TSV
// ---------------------------------------------------------------------------

#[test]
fn snapshot_module_tsv() {
    let mut buf = Vec::new();
    let args = ModuleArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Tsv,
        top: 0,
        module_roots: vec!["crates".into()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };
    write_module_report_to(&mut buf, &module_report(), &global(), &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    insta::assert_snapshot!("module_tsv", output);
}

// ---------------------------------------------------------------------------
// Module — JSON (timestamp-redacted)
// ---------------------------------------------------------------------------

#[test]
fn snapshot_module_json() {
    let mut buf = Vec::new();
    let args = ModuleArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Json,
        top: 0,
        module_roots: vec!["crates".into()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };
    write_module_report_to(&mut buf, &module_report(), &global(), &args).unwrap();
    let raw = String::from_utf8(buf).unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    v["generated_at_ms"] = serde_json::json!(0);
    v["tool"]["version"] = serde_json::json!("0.0.0");
    let pretty = serde_json::to_string_pretty(&v).unwrap();
    insta::assert_snapshot!("module_json", pretty);
}

// ---------------------------------------------------------------------------
// Export — CSV
// ---------------------------------------------------------------------------

#[test]
fn snapshot_export_csv() {
    let mut buf = Vec::new();
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
    write_export_csv_to(&mut buf, &export_data(), &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    insta::assert_snapshot!("export_csv", output);
}

// ---------------------------------------------------------------------------
// Export — JSONL (no meta)
// ---------------------------------------------------------------------------

#[test]
fn snapshot_export_jsonl_no_meta() {
    let mut buf = Vec::new();
    let args = ExportArgs {
        paths: vec![PathBuf::from(".")],
        format: ExportFormat::Jsonl,
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
    write_export_jsonl_to(&mut buf, &export_data(), &global(), &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    insta::assert_snapshot!("export_jsonl_no_meta", output);
}

// ---------------------------------------------------------------------------
// Export — JSON (no meta, rows only)
// ---------------------------------------------------------------------------

#[test]
fn snapshot_export_json_no_meta() {
    let mut buf = Vec::new();
    let args = ExportArgs {
        paths: vec![PathBuf::from(".")],
        format: ExportFormat::Json,
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
    write_export_json_to(&mut buf, &export_data(), &global(), &args).unwrap();
    let raw = String::from_utf8(buf).unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let pretty = serde_json::to_string_pretty(&v).unwrap();
    insta::assert_snapshot!("export_json_no_meta", pretty);
}

// ---------------------------------------------------------------------------
// Export — CycloneDX (deterministic serial+timestamp)
// ---------------------------------------------------------------------------

#[test]
fn snapshot_export_cyclonedx() {
    let mut buf = Vec::new();
    write_export_cyclonedx_with_options(
        &mut buf,
        &export_data(),
        RedactMode::None,
        Some("urn:uuid:00000000-0000-0000-0000-000000000000".into()),
        Some("1970-01-01T00:00:00Z".into()),
    )
    .unwrap();
    let raw = String::from_utf8(buf).unwrap();
    // Normalise tool version
    let mut v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    v["metadata"]["tools"][0]["version"] = serde_json::json!("0.0.0");
    let pretty = serde_json::to_string_pretty(&v).unwrap();
    insta::assert_snapshot!("export_cyclonedx", pretty);
}

// ---------------------------------------------------------------------------
// Diff — Markdown (full + compact)
// ---------------------------------------------------------------------------

fn diff_reports() -> (LangReport, LangReport) {
    let from = LangReport {
        rows: vec![
            LangRow {
                lang: "Rust".into(),
                code: 500,
                lines: 600,
                files: 5,
                bytes: 25000,
                tokens: 1250,
                avg_lines: 120,
            },
            LangRow {
                lang: "Go".into(),
                code: 200,
                lines: 240,
                files: 3,
                bytes: 8000,
                tokens: 500,
                avg_lines: 80,
            },
        ],
        total: Totals {
            code: 700,
            lines: 840,
            files: 8,
            bytes: 33000,
            tokens: 1750,
            avg_lines: 105,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let to = LangReport {
        rows: vec![
            LangRow {
                lang: "Rust".into(),
                code: 600,
                lines: 720,
                files: 6,
                bytes: 30000,
                tokens: 1500,
                avg_lines: 120,
            },
            LangRow {
                lang: "Python".into(),
                code: 150,
                lines: 180,
                files: 4,
                bytes: 7500,
                tokens: 375,
                avg_lines: 45,
            },
        ],
        total: Totals {
            code: 750,
            lines: 900,
            files: 10,
            bytes: 37500,
            tokens: 1875,
            avg_lines: 90,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    (from, to)
}

#[test]
fn snapshot_diff_md_full() {
    let (from, to) = diff_reports();
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let md = render_diff_md("v1.0.0", "v2.0.0", &rows, &totals);
    insta::assert_snapshot!("diff_md_full", md);
}

#[test]
fn snapshot_diff_md_compact() {
    let (from, to) = diff_reports();
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let md = render_diff_md_with_options(
        "v1.0.0",
        "v2.0.0",
        &rows,
        &totals,
        DiffRenderOptions {
            compact: true,
            color: DiffColorMode::Off,
        },
    );
    insta::assert_snapshot!("diff_md_compact", md);
}

// ---------------------------------------------------------------------------
// Diff — JSON receipt (timestamp-redacted)
// ---------------------------------------------------------------------------

#[test]
fn snapshot_diff_json() {
    let (from, to) = diff_reports();
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let receipt = create_diff_receipt("v1.0.0", "v2.0.0", rows, totals);
    let raw = serde_json::to_string(&receipt).unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    v["generated_at_ms"] = serde_json::json!(0);
    v["tool"]["version"] = serde_json::json!("0.0.0");
    let pretty = serde_json::to_string_pretty(&v).unwrap();
    insta::assert_snapshot!("diff_json", pretty);
}
