//! Snapshot tests for tokmd-format – wave 50.
//!
//! Covers: lang Markdown with 5+ languages, module with nested modules,
//! TSV output, CSV export header, JSON envelope, top-N truncation,
//! JSONL export, diff markdown, and multi-module reports.

use std::path::PathBuf;

use tokmd_format::{
    compute_diff_rows, compute_diff_totals, render_diff_md, write_export_csv_to,
    write_export_json_to, write_export_jsonl_to, write_lang_report_to, write_module_report_to,
};
use tokmd_settings::{ChildIncludeMode, ChildrenMode, ScanOptions};
use tokmd_types::{
    ExportArgs, ExportData, ExportFormat, FileKind, FileRow, LangArgs, LangReport, LangRow,
    ModuleArgs, ModuleReport, ModuleRow, RedactMode, TableFormat, Totals,
};

// ---------------------------------------------------------------------------
// Shared fixtures
// ---------------------------------------------------------------------------

fn five_lang_report() -> LangReport {
    LangReport {
        rows: vec![
            LangRow {
                lang: "Rust".into(),
                code: 5000,
                lines: 6000,
                files: 40,
                bytes: 200000,
                tokens: 12500,
                avg_lines: 150,
            },
            LangRow {
                lang: "Python".into(),
                code: 2000,
                lines: 2500,
                files: 15,
                bytes: 72000,
                tokens: 5000,
                avg_lines: 167,
            },
            LangRow {
                lang: "JavaScript".into(),
                code: 1500,
                lines: 1800,
                files: 12,
                bytes: 54000,
                tokens: 3750,
                avg_lines: 150,
            },
            LangRow {
                lang: "TOML".into(),
                code: 400,
                lines: 500,
                files: 8,
                bytes: 12000,
                tokens: 1000,
                avg_lines: 63,
            },
            LangRow {
                lang: "Markdown".into(),
                code: 300,
                lines: 400,
                files: 6,
                bytes: 12000,
                tokens: 750,
                avg_lines: 67,
            },
            LangRow {
                lang: "YAML".into(),
                code: 100,
                lines: 120,
                files: 3,
                bytes: 3600,
                tokens: 250,
                avg_lines: 40,
            },
        ],
        total: Totals {
            code: 9300,
            lines: 11320,
            files: 84,
            bytes: 353600,
            tokens: 23250,
            avg_lines: 135,
        },
        with_files: true,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

fn nested_module_report() -> ModuleReport {
    ModuleReport {
        rows: vec![
            ModuleRow {
                module: "src".into(),
                code: 3000,
                lines: 3600,
                files: 20,
                bytes: 120000,
                tokens: 7500,
                avg_lines: 180,
            },
            ModuleRow {
                module: "src/parser".into(),
                code: 1200,
                lines: 1440,
                files: 8,
                bytes: 48000,
                tokens: 3000,
                avg_lines: 180,
            },
            ModuleRow {
                module: "src/format".into(),
                code: 800,
                lines: 960,
                files: 5,
                bytes: 32000,
                tokens: 2000,
                avg_lines: 192,
            },
            ModuleRow {
                module: "tests".into(),
                code: 600,
                lines: 720,
                files: 4,
                bytes: 24000,
                tokens: 1500,
                avg_lines: 180,
            },
        ],
        total: Totals {
            code: 5600,
            lines: 6720,
            files: 37,
            bytes: 224000,
            tokens: 14000,
            avg_lines: 182,
        },
        module_roots: vec!["src".into()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
        top: 0,
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
                path: "src/parser.rs".into(),
                module: "src".into(),
                lang: "Rust".into(),
                kind: FileKind::Parent,
                code: 350,
                comments: 50,
                blanks: 30,
                lines: 430,
                bytes: 12900,
                tokens: 875,
            },
            FileRow {
                path: "src/format.rs".into(),
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

fn default_scan_options() -> ScanOptions {
    ScanOptions::default()
}

// ── 1. Lang Markdown with 5+ languages ───────────────────────────────

#[test]
fn snapshot_lang_md_five_languages() {
    let report = five_lang_report();
    let args = LangArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Md,
        top: 0,
        files: true,
        children: ChildrenMode::Collapse,
    };
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &default_scan_options(), &args)
        .expect("operation must succeed");
    insta::assert_snapshot!(String::from_utf8(buf).expect("output must be valid UTF-8"));
}

// ── 2. Module with nested modules (Markdown) ─────────────────────────

#[test]
fn snapshot_module_md_nested() {
    let report = nested_module_report();
    let args = ModuleArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Md,
        top: 0,
        module_roots: vec!["src".into()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };
    let mut buf = Vec::new();
    write_module_report_to(&mut buf, &report, &default_scan_options(), &args)
        .expect("operation must succeed");
    insta::assert_snapshot!(String::from_utf8(buf).expect("output must be valid UTF-8"));
}

// ── 3. TSV output for lang ───────────────────────────────────────────

#[test]
fn snapshot_lang_tsv_five_languages() {
    let report = five_lang_report();
    let args = LangArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Tsv,
        top: 0,
        files: true,
        children: ChildrenMode::Collapse,
    };
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &default_scan_options(), &args)
        .expect("operation must succeed");
    insta::assert_snapshot!(String::from_utf8(buf).expect("output must be valid UTF-8"));
}

// ── 4. Export CSV header line ────────────────────────────────────────

#[test]
fn snapshot_export_csv_header() {
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
    write_export_csv_to(&mut buf, &data, &args).expect("operation must succeed");
    insta::assert_snapshot!(String::from_utf8(buf).expect("output must be valid UTF-8"));
}

// ── 5. JSON envelope with metadata ───────────────────────────────────

#[test]
fn snapshot_export_json_envelope() {
    let data = multi_file_export();
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
    let mut buf = Vec::new();
    write_export_json_to(&mut buf, &data, &default_scan_options(), &args)
        .expect("operation must succeed");
    let raw = String::from_utf8(buf).expect("output must be valid UTF-8");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("must parse valid JSON");
    insta::assert_json_snapshot!(v);
}

// ── 6. Top-N truncation (show Other row) ─────────────────────────────

#[test]
fn snapshot_lang_md_top3_with_other() {
    let report = five_lang_report();
    let args = LangArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Md,
        top: 3,
        files: true,
        children: ChildrenMode::Collapse,
    };
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &default_scan_options(), &args)
        .expect("operation must succeed");
    insta::assert_snapshot!(String::from_utf8(buf).expect("output must be valid UTF-8"));
}

// ── 7. JSONL export ──────────────────────────────────────────────────

#[test]
fn snapshot_export_jsonl() {
    let data = multi_file_export();
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
    let mut buf = Vec::new();
    write_export_jsonl_to(&mut buf, &data, &default_scan_options(), &args)
        .expect("operation must succeed");
    insta::assert_snapshot!(String::from_utf8(buf).expect("output must be valid UTF-8"));
}

// ── 8. Module TSV output ─────────────────────────────────────────────

#[test]
fn snapshot_module_tsv_nested() {
    let report = nested_module_report();
    let args = ModuleArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Tsv,
        top: 0,
        module_roots: vec!["src".into()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };
    let mut buf = Vec::new();
    write_module_report_to(&mut buf, &report, &default_scan_options(), &args)
        .expect("operation must succeed");
    insta::assert_snapshot!(String::from_utf8(buf).expect("output must be valid UTF-8"));
}

// ── 9. Module JSON output ────────────────────────────────────────────

#[test]
fn snapshot_module_json_nested() {
    let report = nested_module_report();
    let args = ModuleArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Json,
        top: 0,
        module_roots: vec!["src".into()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };
    let mut buf = Vec::new();
    write_module_report_to(&mut buf, &report, &default_scan_options(), &args)
        .expect("operation must succeed");
    let raw = String::from_utf8(buf).expect("output must be valid UTF-8");
    let mut v: serde_json::Value = serde_json::from_str(&raw).expect("must parse valid JSON");
    v["generated_at_ms"] = serde_json::json!(0);
    v["tool"]["version"] = serde_json::json!("0.0.0-test");
    insta::assert_json_snapshot!(v);
}

// ── 10. Diff markdown with removed language ──────────────────────────

#[test]
fn snapshot_diff_md_removed_language() {
    let from = LangReport {
        rows: vec![
            LangRow {
                lang: "Rust".into(),
                code: 2000,
                lines: 2400,
                files: 20,
                bytes: 80000,
                tokens: 5000,
                avg_lines: 120,
            },
            LangRow {
                lang: "Python".into(),
                code: 500,
                lines: 600,
                files: 5,
                bytes: 18000,
                tokens: 1250,
                avg_lines: 120,
            },
        ],
        total: Totals {
            code: 2500,
            lines: 3000,
            files: 25,
            bytes: 98000,
            tokens: 6250,
            avg_lines: 120,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let to = LangReport {
        rows: vec![LangRow {
            lang: "Rust".into(),
            code: 2500,
            lines: 3000,
            files: 25,
            bytes: 100000,
            tokens: 6250,
            avg_lines: 120,
        }],
        total: Totals {
            code: 2500,
            lines: 3000,
            files: 25,
            bytes: 100000,
            tokens: 6250,
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

// ── 11. Lang Markdown without files column ───────────────────────────

#[test]
fn snapshot_lang_md_no_files() {
    let mut report = five_lang_report();
    report.with_files = false;
    let args = LangArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Md,
        top: 0,
        files: false,
        children: ChildrenMode::Collapse,
    };
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &default_scan_options(), &args)
        .expect("operation must succeed");
    insta::assert_snapshot!(String::from_utf8(buf).expect("output must be valid UTF-8"));
}

// ── 12. Export CSV with min_code filter ──────────────────────────────

#[test]
fn snapshot_export_csv_min_code() {
    let data = multi_file_export();
    let args = ExportArgs {
        paths: vec![PathBuf::from(".")],
        format: ExportFormat::Csv,
        output: None,
        module_roots: vec!["src".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        min_code: 300,
        max_rows: 0,
        redact: RedactMode::None,
        meta: false,
        strip_prefix: None,
    };
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &data, &args).expect("operation must succeed");
    insta::assert_snapshot!(String::from_utf8(buf).expect("output must be valid UTF-8"));
}
