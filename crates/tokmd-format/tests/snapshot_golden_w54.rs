//! Golden snapshot tests for output formats (W54).
//!
//! Pins Markdown, TSV, JSON, JSONL, and CSV renderings so that any
//! accidental change to the output surface is caught at review time.

use std::path::PathBuf;

use tokmd_format::{
    write_export_csv_to, write_export_json_to, write_export_jsonl_to, write_lang_report_to,
    write_module_report_to,
};
use tokmd_settings::{ChildIncludeMode, ChildrenMode, ScanOptions};
use tokmd_types::{
    ExportArgs, ExportData, ExportFormat, FileKind, FileRow, LangArgs, LangReport, LangRow,
    ModuleArgs, ModuleReport, ModuleRow, RedactMode, TableFormat, Totals,
};

// ===========================================================================
// Fixtures
// ===========================================================================

fn two_lang_report(with_files: bool) -> LangReport {
    LangReport {
        rows: vec![
            LangRow {
                lang: "Rust".into(),
                code: 5000,
                lines: 6500,
                files: 42,
                bytes: 180_000,
                tokens: 45_000,
                avg_lines: 154,
            },
            LangRow {
                lang: "Python".into(),
                code: 1200,
                lines: 1500,
                files: 8,
                bytes: 36_000,
                tokens: 12_000,
                avg_lines: 187,
            },
        ],
        total: Totals {
            code: 6200,
            lines: 8000,
            files: 50,
            bytes: 216_000,
            tokens: 57_000,
            avg_lines: 160,
        },
        with_files,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

fn single_lang_report() -> LangReport {
    LangReport {
        rows: vec![LangRow {
            lang: "Go".into(),
            code: 300,
            lines: 400,
            files: 3,
            bytes: 9000,
            tokens: 3000,
            avg_lines: 133,
        }],
        total: Totals {
            code: 300,
            lines: 400,
            files: 3,
            bytes: 9000,
            tokens: 3000,
            avg_lines: 133,
        },
        with_files: true,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

fn many_lang_report() -> LangReport {
    let langs = [
        ("Rust", 5000),
        ("Python", 3000),
        ("JavaScript", 2000),
        ("TypeScript", 1500),
        ("Go", 1000),
        ("C", 800),
        ("Shell", 400),
    ];
    let rows: Vec<LangRow> = langs
        .iter()
        .map(|(name, code)| LangRow {
            lang: name.to_string(),
            code: *code,
            lines: code + code / 4,
            files: code / 100,
            bytes: code * 30,
            tokens: code * 8,
            avg_lines: 125,
        })
        .collect();
    let total = Totals {
        code: rows.iter().map(|r| r.code).sum(),
        lines: rows.iter().map(|r| r.lines).sum(),
        files: rows.iter().map(|r| r.files).sum(),
        bytes: rows.iter().map(|r| r.bytes).sum(),
        tokens: rows.iter().map(|r| r.tokens).sum(),
        avg_lines: 125,
    };
    LangReport {
        rows,
        total,
        with_files: true,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

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

fn embedded_lang_report_separate() -> LangReport {
    LangReport {
        rows: vec![
            LangRow {
                lang: "HTML".into(),
                code: 400,
                lines: 500,
                files: 5,
                bytes: 12_000,
                tokens: 4_000,
                avg_lines: 100,
            },
            LangRow {
                lang: "JavaScript (embedded)".into(),
                code: 200,
                lines: 250,
                files: 5,
                bytes: 6_000,
                tokens: 2_000,
                avg_lines: 50,
            },
            LangRow {
                lang: "CSS (embedded)".into(),
                code: 100,
                lines: 120,
                files: 3,
                bytes: 3_000,
                tokens: 1_000,
                avg_lines: 40,
            },
        ],
        total: Totals {
            code: 700,
            lines: 870,
            files: 13,
            bytes: 21_000,
            tokens: 7_000,
            avg_lines: 66,
        },
        with_files: true,
        children: ChildrenMode::Separate,
        top: 0,
    }
}

fn embedded_lang_report_collapse() -> LangReport {
    LangReport {
        rows: vec![LangRow {
            lang: "HTML".into(),
            code: 700,
            lines: 870,
            files: 5,
            bytes: 21_000,
            tokens: 7_000,
            avg_lines: 174,
        }],
        total: Totals {
            code: 700,
            lines: 870,
            files: 5,
            bytes: 21_000,
            tokens: 7_000,
            avg_lines: 174,
        },
        with_files: true,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

fn sample_module_report() -> ModuleReport {
    ModuleReport {
        rows: vec![
            ModuleRow {
                module: "crates/core".into(),
                code: 3000,
                lines: 3800,
                files: 15,
                bytes: 90_000,
                tokens: 24_000,
                avg_lines: 253,
            },
            ModuleRow {
                module: "crates/cli".into(),
                code: 1500,
                lines: 1900,
                files: 7,
                bytes: 45_000,
                tokens: 12_000,
                avg_lines: 271,
            },
            ModuleRow {
                module: "tests".into(),
                code: 800,
                lines: 1000,
                files: 4,
                bytes: 24_000,
                tokens: 6_400,
                avg_lines: 250,
            },
        ],
        total: Totals {
            code: 5300,
            lines: 6700,
            files: 26,
            bytes: 159_000,
            tokens: 42_400,
            avg_lines: 257,
        },
        module_roots: vec!["crates".into()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
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
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        top: 0,
    }
}

fn sample_file_rows() -> Vec<FileRow> {
    vec![
        FileRow {
            path: "src/main.rs".into(),
            module: "src".into(),
            lang: "Rust".into(),
            kind: FileKind::Parent,
            code: 250,
            comments: 40,
            blanks: 30,
            lines: 320,
            bytes: 8000,
            tokens: 2500,
        },
        FileRow {
            path: "src/lib.rs".into(),
            module: "src".into(),
            lang: "Rust".into(),
            kind: FileKind::Parent,
            code: 180,
            comments: 25,
            blanks: 20,
            lines: 225,
            bytes: 5400,
            tokens: 1800,
        },
        FileRow {
            path: "tests/smoke.rs".into(),
            module: "tests".into(),
            lang: "Rust".into(),
            kind: FileKind::Parent,
            code: 60,
            comments: 5,
            blanks: 8,
            lines: 73,
            bytes: 1800,
            tokens: 600,
        },
    ]
}

fn sample_export_data() -> ExportData {
    ExportData {
        rows: sample_file_rows(),
        module_roots: vec!["src".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn default_scan() -> ScanOptions {
    ScanOptions::default()
}

fn lang_args(format: TableFormat) -> LangArgs {
    LangArgs {
        paths: vec![PathBuf::from(".")],
        format,
        top: 0,
        files: true,
        children: ChildrenMode::Collapse,
    }
}

fn module_args(format: TableFormat) -> ModuleArgs {
    ModuleArgs {
        paths: vec![PathBuf::from(".")],
        format,
        top: 0,
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

fn export_args(format: ExportFormat) -> ExportArgs {
    ExportArgs {
        paths: vec![PathBuf::from(".")],
        format,
        output: None,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        redact: RedactMode::None,
        meta: false,
        strip_prefix: None,
    }
}

// ===========================================================================
// Helper: render to String
// ===========================================================================

fn render_lang(report: &LangReport, format: TableFormat) -> String {
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, report, &default_scan(), &lang_args(format)).unwrap();
    String::from_utf8(buf).unwrap()
}

fn render_module(report: &ModuleReport, format: TableFormat) -> String {
    let mut buf = Vec::new();
    write_module_report_to(&mut buf, report, &default_scan(), &module_args(format)).unwrap();
    String::from_utf8(buf).unwrap()
}

fn render_export_csv(data: &ExportData) -> String {
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, data, &export_args(ExportFormat::Csv)).unwrap();
    String::from_utf8(buf).unwrap()
}

fn render_export_jsonl(data: &ExportData) -> String {
    let mut buf = Vec::new();
    write_export_jsonl_to(
        &mut buf,
        data,
        &default_scan(),
        &export_args(ExportFormat::Jsonl),
    )
    .unwrap();
    String::from_utf8(buf).unwrap()
}

// ===========================================================================
// Lang – Markdown
// ===========================================================================

#[test]
fn w54_lang_md_with_files() {
    let out = render_lang(&two_lang_report(true), TableFormat::Md);
    insta::assert_snapshot!("w54_lang_md_with_files", out);
}

#[test]
fn w54_lang_md_without_files() {
    let out = render_lang(&two_lang_report(false), TableFormat::Md);
    insta::assert_snapshot!("w54_lang_md_without_files", out);
}

#[test]
fn w54_lang_md_single_language() {
    let out = render_lang(&single_lang_report(), TableFormat::Md);
    insta::assert_snapshot!("w54_lang_md_single_language", out);
}

#[test]
fn w54_lang_md_many_languages() {
    let out = render_lang(&many_lang_report(), TableFormat::Md);
    insta::assert_snapshot!("w54_lang_md_many_languages", out);
}

#[test]
fn w54_lang_md_empty() {
    let out = render_lang(&empty_lang_report(), TableFormat::Md);
    insta::assert_snapshot!("w54_lang_md_empty", out);
}

#[test]
fn w54_lang_md_embedded_separate() {
    let out = render_lang(&embedded_lang_report_separate(), TableFormat::Md);
    insta::assert_snapshot!("w54_lang_md_embedded_separate", out);
}

#[test]
fn w54_lang_md_embedded_collapse() {
    let out = render_lang(&embedded_lang_report_collapse(), TableFormat::Md);
    insta::assert_snapshot!("w54_lang_md_embedded_collapse", out);
}

// ===========================================================================
// Lang – TSV
// ===========================================================================

#[test]
fn w54_lang_tsv_with_files() {
    let out = render_lang(&two_lang_report(true), TableFormat::Tsv);
    insta::assert_snapshot!("w54_lang_tsv_with_files", out);
}

#[test]
fn w54_lang_tsv_without_files() {
    let out = render_lang(&two_lang_report(false), TableFormat::Tsv);
    insta::assert_snapshot!("w54_lang_tsv_without_files", out);
}

#[test]
fn w54_lang_tsv_empty() {
    let out = render_lang(&empty_lang_report(), TableFormat::Tsv);
    insta::assert_snapshot!("w54_lang_tsv_empty", out);
}

#[test]
fn w54_lang_tsv_single_language() {
    let out = render_lang(&single_lang_report(), TableFormat::Tsv);
    insta::assert_snapshot!("w54_lang_tsv_single_language", out);
}

// ===========================================================================
// Lang – JSON (structure only; timestamps redacted via insta settings)
// ===========================================================================

fn normalize_json(raw: &str) -> String {
    let mut v: serde_json::Value = serde_json::from_str(raw).unwrap();
    if let Some(obj) = v.as_object_mut() {
        obj.insert("generated_at_ms".into(), serde_json::json!(0));
        obj.insert(
            "tool".into(),
            serde_json::json!({"name": "tokmd", "version": "0.0.0-test"}),
        );
        obj.remove("scan");
    }
    serde_json::to_string_pretty(&v).unwrap()
}

#[test]
fn w54_lang_json_two_langs() {
    let out = render_lang(&two_lang_report(true), TableFormat::Json);
    insta::assert_snapshot!("w54_lang_json_two_langs", normalize_json(&out));
}

#[test]
fn w54_lang_json_empty() {
    let out = render_lang(&empty_lang_report(), TableFormat::Json);
    insta::assert_snapshot!("w54_lang_json_empty", normalize_json(&out));
}

// ===========================================================================
// Module – Markdown
// ===========================================================================

#[test]
fn w54_module_md() {
    let out = render_module(&sample_module_report(), TableFormat::Md);
    insta::assert_snapshot!("w54_module_md", out);
}

#[test]
fn w54_module_md_empty() {
    let out = render_module(&empty_module_report(), TableFormat::Md);
    insta::assert_snapshot!("w54_module_md_empty", out);
}

// ===========================================================================
// Module – TSV
// ===========================================================================

#[test]
fn w54_module_tsv() {
    let out = render_module(&sample_module_report(), TableFormat::Tsv);
    insta::assert_snapshot!("w54_module_tsv", out);
}

#[test]
fn w54_module_tsv_empty() {
    let out = render_module(&empty_module_report(), TableFormat::Tsv);
    insta::assert_snapshot!("w54_module_tsv_empty", out);
}

// ===========================================================================
// Module – JSON
// ===========================================================================

#[test]
fn w54_module_json() {
    let out = render_module(&sample_module_report(), TableFormat::Json);
    insta::assert_snapshot!("w54_module_json", normalize_json(&out));
}

// ===========================================================================
// Export – CSV
// ===========================================================================

#[test]
fn w54_export_csv() {
    let out = render_export_csv(&sample_export_data());
    insta::assert_snapshot!("w54_export_csv", out);
}

#[test]
fn w54_export_csv_empty() {
    let data = ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let out = render_export_csv(&data);
    insta::assert_snapshot!("w54_export_csv_empty", out);
}

// ===========================================================================
// Export – JSONL (no meta)
// ===========================================================================

#[test]
fn w54_export_jsonl() {
    let out = render_export_jsonl(&sample_export_data());
    insta::assert_snapshot!("w54_export_jsonl", out);
}

#[test]
fn w54_export_jsonl_empty() {
    let data = ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let out = render_export_jsonl(&data);
    insta::assert_snapshot!("w54_export_jsonl_empty", out);
}

// ===========================================================================
// Export – JSON (no meta, rows-only mode)
// ===========================================================================

#[test]
fn w54_export_json_rows() {
    let mut buf = Vec::new();
    write_export_json_to(
        &mut buf,
        &sample_export_data(),
        &default_scan(),
        &export_args(ExportFormat::Json),
    )
    .unwrap();
    let out = String::from_utf8(buf).unwrap();
    // rows-only mode (meta=false) emits a plain JSON array – no timestamps.
    insta::assert_snapshot!("w54_export_json_rows", out);
}
