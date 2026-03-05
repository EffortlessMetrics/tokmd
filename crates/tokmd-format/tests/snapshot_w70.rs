//! Golden snapshot tests for output formats (W70).
//!
//! Captures Markdown, TSV, JSON, JSONL, and CSV renderings so that any
//! accidental change to the output surface is caught at review time.

use std::path::PathBuf;

use tokmd_format::{
    write_export_csv_to, write_export_jsonl_to, write_lang_report_to, write_module_report_to,
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
                code: 4200,
                lines: 5500,
                files: 35,
                bytes: 150_000,
                tokens: 38_000,
                avg_lines: 157,
            },
            LangRow {
                lang: "Python".into(),
                code: 1800,
                lines: 2300,
                files: 12,
                bytes: 54_000,
                tokens: 18_000,
                avg_lines: 191,
            },
        ],
        total: Totals {
            code: 6000,
            lines: 7800,
            files: 47,
            bytes: 204_000,
            tokens: 56_000,
            avg_lines: 165,
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
            code: 750,
            lines: 950,
            files: 6,
            bytes: 22_500,
            tokens: 7_500,
            avg_lines: 158,
        }],
        total: Totals {
            code: 750,
            lines: 950,
            files: 6,
            bytes: 22_500,
            tokens: 7_500,
            avg_lines: 158,
        },
        with_files: true,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

fn four_lang_report() -> LangReport {
    let langs = [
        ("Rust", 4000),
        ("TypeScript", 2500),
        ("Python", 1200),
        ("Shell", 300),
    ];
    let rows: Vec<LangRow> = langs
        .iter()
        .map(|(name, code)| LangRow {
            lang: name.to_string(),
            code: *code,
            lines: code + code / 5,
            files: code / 200,
            bytes: code * 25,
            tokens: code * 7,
            avg_lines: 140,
        })
        .collect();
    let total = Totals {
        code: rows.iter().map(|r| r.code).sum(),
        lines: rows.iter().map(|r| r.lines).sum(),
        files: rows.iter().map(|r| r.files).sum(),
        bytes: rows.iter().map(|r| r.bytes).sum(),
        tokens: rows.iter().map(|r| r.tokens).sum(),
        avg_lines: 140,
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
                code: 350,
                lines: 450,
                files: 4,
                bytes: 10_500,
                tokens: 3_500,
                avg_lines: 112,
            },
            LangRow {
                lang: "JavaScript (embedded)".into(),
                code: 180,
                lines: 220,
                files: 4,
                bytes: 5_400,
                tokens: 1_800,
                avg_lines: 55,
            },
            LangRow {
                lang: "CSS (embedded)".into(),
                code: 90,
                lines: 110,
                files: 3,
                bytes: 2_700,
                tokens: 900,
                avg_lines: 36,
            },
        ],
        total: Totals {
            code: 620,
            lines: 780,
            files: 11,
            bytes: 18_600,
            tokens: 6_200,
            avg_lines: 70,
        },
        with_files: true,
        children: ChildrenMode::Separate,
        top: 0,
    }
}

fn top_limited_report() -> LangReport {
    let mut report = four_lang_report();
    report.top = 2;
    report
}

fn three_module_report() -> ModuleReport {
    ModuleReport {
        rows: vec![
            ModuleRow {
                module: "crates/engine".into(),
                code: 2800,
                lines: 3500,
                files: 14,
                bytes: 84_000,
                tokens: 22_400,
                avg_lines: 250,
            },
            ModuleRow {
                module: "crates/api".into(),
                code: 1400,
                lines: 1750,
                files: 8,
                bytes: 42_000,
                tokens: 11_200,
                avg_lines: 218,
            },
            ModuleRow {
                module: "tests".into(),
                code: 900,
                lines: 1100,
                files: 5,
                bytes: 27_000,
                tokens: 7_200,
                avg_lines: 220,
            },
        ],
        total: Totals {
            code: 5100,
            lines: 6350,
            files: 27,
            bytes: 153_000,
            tokens: 40_800,
            avg_lines: 235,
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
            path: "src/engine.rs".into(),
            module: "src".into(),
            lang: "Rust".into(),
            kind: FileKind::Parent,
            code: 320,
            comments: 45,
            blanks: 35,
            lines: 400,
            bytes: 9_600,
            tokens: 3_200,
        },
        FileRow {
            path: "src/api.rs".into(),
            module: "src".into(),
            lang: "Rust".into(),
            kind: FileKind::Parent,
            code: 210,
            comments: 30,
            blanks: 25,
            lines: 265,
            bytes: 6_300,
            tokens: 2_100,
        },
        FileRow {
            path: "tests/integration.rs".into(),
            module: "tests".into(),
            lang: "Rust".into(),
            kind: FileKind::Parent,
            code: 85,
            comments: 10,
            blanks: 12,
            lines: 107,
            bytes: 2_550,
            tokens: 850,
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

// ===========================================================================
// Lang – Markdown
// ===========================================================================

#[test]
fn w70_lang_md_two_langs_with_files() {
    let out = render_lang(&two_lang_report(true), TableFormat::Md);
    insta::assert_snapshot!("w70_lang_md_two_langs_with_files", out);
}

#[test]
fn w70_lang_md_two_langs_without_files() {
    let out = render_lang(&two_lang_report(false), TableFormat::Md);
    insta::assert_snapshot!("w70_lang_md_two_langs_without_files", out);
}

#[test]
fn w70_lang_md_single_lang() {
    let out = render_lang(&single_lang_report(), TableFormat::Md);
    insta::assert_snapshot!("w70_lang_md_single_lang", out);
}

#[test]
fn w70_lang_md_four_langs() {
    let out = render_lang(&four_lang_report(), TableFormat::Md);
    insta::assert_snapshot!("w70_lang_md_four_langs", out);
}

#[test]
fn w70_lang_md_empty() {
    let out = render_lang(&empty_lang_report(), TableFormat::Md);
    insta::assert_snapshot!("w70_lang_md_empty", out);
}

#[test]
fn w70_lang_md_embedded_separate() {
    let out = render_lang(&embedded_lang_report_separate(), TableFormat::Md);
    insta::assert_snapshot!("w70_lang_md_embedded_separate", out);
}

#[test]
fn w70_lang_md_top_limited() {
    let out = render_lang(&top_limited_report(), TableFormat::Md);
    insta::assert_snapshot!("w70_lang_md_top_limited", out);
}

// ===========================================================================
// Lang – TSV
// ===========================================================================

#[test]
fn w70_lang_tsv_with_files() {
    let out = render_lang(&two_lang_report(true), TableFormat::Tsv);
    insta::assert_snapshot!("w70_lang_tsv_with_files", out);
}

#[test]
fn w70_lang_tsv_without_files() {
    let out = render_lang(&two_lang_report(false), TableFormat::Tsv);
    insta::assert_snapshot!("w70_lang_tsv_without_files", out);
}

#[test]
fn w70_lang_tsv_empty() {
    let out = render_lang(&empty_lang_report(), TableFormat::Tsv);
    insta::assert_snapshot!("w70_lang_tsv_empty", out);
}

#[test]
fn w70_lang_tsv_four_langs() {
    let out = render_lang(&four_lang_report(), TableFormat::Tsv);
    insta::assert_snapshot!("w70_lang_tsv_four_langs", out);
}

// ===========================================================================
// Lang – JSON
// ===========================================================================

#[test]
fn w70_lang_json_two_langs() {
    let out = render_lang(&two_lang_report(true), TableFormat::Json);
    insta::assert_snapshot!("w70_lang_json_two_langs", normalize_json(&out));
}

#[test]
fn w70_lang_json_empty() {
    let out = render_lang(&empty_lang_report(), TableFormat::Json);
    insta::assert_snapshot!("w70_lang_json_empty", normalize_json(&out));
}

#[test]
fn w70_lang_json_single_lang() {
    let out = render_lang(&single_lang_report(), TableFormat::Json);
    insta::assert_snapshot!("w70_lang_json_single_lang", normalize_json(&out));
}

// ===========================================================================
// Module – Markdown
// ===========================================================================

#[test]
fn w70_module_md_three_modules() {
    let out = render_module(&three_module_report(), TableFormat::Md);
    insta::assert_snapshot!("w70_module_md_three_modules", out);
}

#[test]
fn w70_module_md_empty() {
    let out = render_module(&empty_module_report(), TableFormat::Md);
    insta::assert_snapshot!("w70_module_md_empty", out);
}

// ===========================================================================
// Module – TSV
// ===========================================================================

#[test]
fn w70_module_tsv_three_modules() {
    let out = render_module(&three_module_report(), TableFormat::Tsv);
    insta::assert_snapshot!("w70_module_tsv_three_modules", out);
}

// ===========================================================================
// Module – JSON
// ===========================================================================

#[test]
fn w70_module_json_three_modules() {
    let out = render_module(&three_module_report(), TableFormat::Json);
    insta::assert_snapshot!("w70_module_json_three_modules", normalize_json(&out));
}

// ===========================================================================
// Export – CSV
// ===========================================================================

#[test]
fn w70_export_csv_with_children() {
    let out = render_export_csv(&sample_export_data());
    insta::assert_snapshot!("w70_export_csv_with_children", out);
}

// ===========================================================================
// Export – JSONL
// ===========================================================================

#[test]
fn w70_export_jsonl_rows() {
    let out = render_export_jsonl(&sample_export_data());
    insta::assert_snapshot!("w70_export_jsonl_rows", out);
}
