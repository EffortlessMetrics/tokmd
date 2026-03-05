//! Golden snapshot tests for output formats (W74).
//!
//! Comprehensive insta snapshot coverage for Markdown, TSV, JSON, CSV, and JSONL
//! renderings across lang, module, and export commands.

use std::path::PathBuf;

use tokmd_format::{
    write_export_csv_to, write_export_jsonl_to, write_export_json_to,
    write_export_cyclonedx_with_options, write_lang_report_to, write_module_report_to,
};
use tokmd_settings::{ChildIncludeMode, ChildrenMode, ScanOptions};
use tokmd_types::{
    ExportArgs, ExportData, ExportFormat, FileKind, FileRow, LangArgs, LangReport, LangRow,
    ModuleArgs, ModuleReport, ModuleRow, RedactMode, TableFormat, Totals,
};

// ===========================================================================
// Fixtures
// ===========================================================================

fn three_lang_report(with_files: bool) -> LangReport {
    LangReport {
        rows: vec![
            LangRow {
                lang: "Rust".into(),
                code: 5200,
                lines: 6800,
                files: 40,
                bytes: 182_000,
                tokens: 46_000,
                avg_lines: 170,
            },
            LangRow {
                lang: "TypeScript".into(),
                code: 3100,
                lines: 4000,
                files: 25,
                bytes: 93_000,
                tokens: 28_000,
                avg_lines: 160,
            },
            LangRow {
                lang: "TOML".into(),
                code: 400,
                lines: 520,
                files: 8,
                bytes: 12_000,
                tokens: 3_200,
                avg_lines: 65,
            },
        ],
        total: Totals {
            code: 8700,
            lines: 11320,
            files: 73,
            bytes: 287_000,
            tokens: 77_200,
            avg_lines: 155,
        },
        with_files,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

fn single_lang_report() -> LangReport {
    LangReport {
        rows: vec![LangRow {
            lang: "Python".into(),
            code: 920,
            lines: 1180,
            files: 7,
            bytes: 27_600,
            tokens: 9_200,
            avg_lines: 168,
        }],
        total: Totals {
            code: 920,
            lines: 1180,
            files: 7,
            bytes: 27_600,
            tokens: 9_200,
            avg_lines: 168,
        },
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

fn embedded_lang_report() -> LangReport {
    LangReport {
        rows: vec![
            LangRow {
                lang: "HTML".into(),
                code: 420,
                lines: 540,
                files: 5,
                bytes: 12_600,
                tokens: 4_200,
                avg_lines: 108,
            },
            LangRow {
                lang: "JavaScript (embedded)".into(),
                code: 210,
                lines: 260,
                files: 5,
                bytes: 6_300,
                tokens: 2_100,
                avg_lines: 52,
            },
            LangRow {
                lang: "CSS (embedded)".into(),
                code: 130,
                lines: 160,
                files: 4,
                bytes: 3_900,
                tokens: 1_300,
                avg_lines: 40,
            },
        ],
        total: Totals {
            code: 760,
            lines: 960,
            files: 14,
            bytes: 22_800,
            tokens: 7_600,
            avg_lines: 68,
        },
        with_files: true,
        children: ChildrenMode::Separate,
        top: 0,
    }
}

fn three_module_report() -> ModuleReport {
    ModuleReport {
        rows: vec![
            ModuleRow {
                module: "crates/core".into(),
                code: 3200,
                lines: 4100,
                files: 16,
                bytes: 96_000,
                tokens: 25_600,
                avg_lines: 256,
            },
            ModuleRow {
                module: "crates/api".into(),
                code: 1800,
                lines: 2250,
                files: 10,
                bytes: 54_000,
                tokens: 14_400,
                avg_lines: 225,
            },
            ModuleRow {
                module: "tests".into(),
                code: 1100,
                lines: 1350,
                files: 6,
                bytes: 33_000,
                tokens: 8_800,
                avg_lines: 225,
            },
        ],
        total: Totals {
            code: 6100,
            lines: 7700,
            files: 32,
            bytes: 183_000,
            tokens: 48_800,
            avg_lines: 240,
        },
        module_roots: vec!["crates".into()],
        module_depth: 2,
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
            code: 280,
            comments: 40,
            blanks: 30,
            lines: 350,
            bytes: 8_400,
            tokens: 2_800,
        },
        FileRow {
            path: "src/lib.rs".into(),
            module: "src".into(),
            lang: "Rust".into(),
            kind: FileKind::Parent,
            code: 450,
            comments: 65,
            blanks: 45,
            lines: 560,
            bytes: 13_500,
            tokens: 4_500,
        },
        FileRow {
            path: "tests/smoke.rs".into(),
            module: "tests".into(),
            lang: "Rust".into(),
            kind: FileKind::Parent,
            code: 95,
            comments: 12,
            blanks: 15,
            lines: 122,
            bytes: 2_850,
            tokens: 950,
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

// ===========================================================================
// Args helpers
// ===========================================================================

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
// Render helpers
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

fn render_export_json(data: &ExportData) -> String {
    let mut buf = Vec::new();
    write_export_json_to(
        &mut buf,
        data,
        &default_scan(),
        &export_args(ExportFormat::Json),
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
// Lang – Markdown (6 tests)
// ===========================================================================

#[test]
fn w74_lang_md_three_langs_with_files() {
    let out = render_lang(&three_lang_report(true), TableFormat::Md);
    insta::assert_snapshot!("w74_lang_md_three_langs_with_files", out);
}

#[test]
fn w74_lang_md_three_langs_without_files() {
    let out = render_lang(&three_lang_report(false), TableFormat::Md);
    insta::assert_snapshot!("w74_lang_md_three_langs_without_files", out);
}

#[test]
fn w74_lang_md_single_lang() {
    let out = render_lang(&single_lang_report(), TableFormat::Md);
    insta::assert_snapshot!("w74_lang_md_single_lang", out);
}

#[test]
fn w74_lang_md_empty() {
    let out = render_lang(&empty_lang_report(), TableFormat::Md);
    insta::assert_snapshot!("w74_lang_md_empty", out);
}

#[test]
fn w74_lang_md_embedded_separate() {
    let out = render_lang(&embedded_lang_report(), TableFormat::Md);
    insta::assert_snapshot!("w74_lang_md_embedded_separate", out);
}

#[test]
fn w74_lang_md_empty_with_files() {
    let mut report = empty_lang_report();
    report.with_files = true;
    let out = render_lang(&report, TableFormat::Md);
    insta::assert_snapshot!("w74_lang_md_empty_with_files", out);
}

// ===========================================================================
// Lang – TSV (4 tests)
// ===========================================================================

#[test]
fn w74_lang_tsv_three_langs() {
    let out = render_lang(&three_lang_report(true), TableFormat::Tsv);
    insta::assert_snapshot!("w74_lang_tsv_three_langs", out);
}

#[test]
fn w74_lang_tsv_without_files() {
    let out = render_lang(&three_lang_report(false), TableFormat::Tsv);
    insta::assert_snapshot!("w74_lang_tsv_without_files", out);
}

#[test]
fn w74_lang_tsv_single_lang() {
    let out = render_lang(&single_lang_report(), TableFormat::Tsv);
    insta::assert_snapshot!("w74_lang_tsv_single_lang", out);
}

#[test]
fn w74_lang_tsv_empty() {
    let out = render_lang(&empty_lang_report(), TableFormat::Tsv);
    insta::assert_snapshot!("w74_lang_tsv_empty", out);
}

// ===========================================================================
// Lang – JSON (3 tests)
// ===========================================================================

#[test]
fn w74_lang_json_three_langs() {
    let out = render_lang(&three_lang_report(true), TableFormat::Json);
    insta::assert_snapshot!("w74_lang_json_three_langs", normalize_json(&out));
}

#[test]
fn w74_lang_json_single_lang() {
    let out = render_lang(&single_lang_report(), TableFormat::Json);
    insta::assert_snapshot!("w74_lang_json_single_lang", normalize_json(&out));
}

#[test]
fn w74_lang_json_empty() {
    let out = render_lang(&empty_lang_report(), TableFormat::Json);
    insta::assert_snapshot!("w74_lang_json_empty", normalize_json(&out));
}

// ===========================================================================
// Module – Markdown / TSV / JSON (3 tests)
// ===========================================================================

#[test]
fn w74_module_md_three_modules() {
    let out = render_module(&three_module_report(), TableFormat::Md);
    insta::assert_snapshot!("w74_module_md_three_modules", out);
}

#[test]
fn w74_module_tsv_three_modules() {
    let out = render_module(&three_module_report(), TableFormat::Tsv);
    insta::assert_snapshot!("w74_module_tsv_three_modules", out);
}

#[test]
fn w74_module_json_three_modules() {
    let out = render_module(&three_module_report(), TableFormat::Json);
    insta::assert_snapshot!("w74_module_json_three_modules", normalize_json(&out));
}

// ===========================================================================
// Export – CSV / JSONL / JSON / CycloneDX (4 tests)
// ===========================================================================

#[test]
fn w74_export_csv() {
    let out = render_export_csv(&sample_export_data());
    insta::assert_snapshot!("w74_export_csv", out);
}

#[test]
fn w74_export_jsonl() {
    let out = render_export_jsonl(&sample_export_data());
    insta::assert_snapshot!("w74_export_jsonl", out);
}

#[test]
fn w74_export_json() {
    let raw = render_export_json(&sample_export_data());
    insta::assert_snapshot!("w74_export_json", normalize_json(&raw));
}

#[test]
fn w74_export_cyclonedx() {
    let mut buf = Vec::new();
    write_export_cyclonedx_with_options(
        &mut buf,
        &sample_export_data(),
        RedactMode::None,
        Some("urn:uuid:00000000-0000-0000-0000-000000000000".into()),
        Some("2024-01-01T00:00:00Z".into()),
    )
    .unwrap();
    let out = String::from_utf8(buf).unwrap();
    let pretty: serde_json::Value = serde_json::from_str(&out).unwrap();
    insta::assert_snapshot!(
        "w74_export_cyclonedx",
        serde_json::to_string_pretty(&pretty).unwrap()
    );
}
