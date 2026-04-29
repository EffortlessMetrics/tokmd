//! Expanded insta snapshot tests for output formats (W58).
//!
//! Covers Markdown, TSV, and JSON renderings across lang, module, export,
//! and diff pipelines with various edge cases.

use std::path::PathBuf;

use tokmd_format::{
    DiffColorMode, DiffRenderOptions, compute_diff_rows, compute_diff_totals, render_diff_md,
    render_diff_md_with_options, write_export_csv_to, write_export_json_to, write_export_jsonl_to,
    write_lang_report_to, write_module_report_to,
};
use tokmd_settings::{ChildIncludeMode, ChildrenMode, ScanOptions};
use tokmd_types::{
    ExportArgs, ExportData, ExportFormat, FileKind, FileRow, LangArgs, LangReport, LangRow,
    ModuleArgs, ModuleReport, ModuleRow, RedactMode, TableFormat, Totals,
};

// ===========================================================================
// Fixtures
// ===========================================================================

fn lang_row(name: &str, code: usize) -> LangRow {
    LangRow {
        lang: name.to_string(),
        code,
        lines: code + code / 3,
        files: (code / 100).max(1),
        bytes: code * 35,
        tokens: code * 9,
        avg_lines: if code > 0 { 130 } else { 0 },
    }
}

fn totals_from_rows(rows: &[LangRow]) -> Totals {
    Totals {
        code: rows.iter().map(|r| r.code).sum(),
        lines: rows.iter().map(|r| r.lines).sum(),
        files: rows.iter().map(|r| r.files).sum(),
        bytes: rows.iter().map(|r| r.bytes).sum(),
        tokens: rows.iter().map(|r| r.tokens).sum(),
        avg_lines: if rows.is_empty() {
            0
        } else {
            let total_lines: usize = rows.iter().map(|r| r.lines).sum();
            let total_files: usize = rows.iter().map(|r| r.files).sum();
            if total_files > 0 {
                total_lines / total_files
            } else {
                0
            }
        },
    }
}

fn make_lang_report(names: &[(&str, usize)], with_files: bool) -> LangReport {
    let rows: Vec<LangRow> = names.iter().map(|(n, c)| lang_row(n, *c)).collect();
    let total = totals_from_rows(&rows);
    LangReport {
        rows,
        total,
        with_files,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

fn twenty_plus_lang_report() -> LangReport {
    let langs = [
        ("Rust", 8000),
        ("Python", 5000),
        ("JavaScript", 4500),
        ("TypeScript", 4000),
        ("Go", 3500),
        ("C", 3000),
        ("C++", 2500),
        ("Java", 2000),
        ("Ruby", 1500),
        ("Shell", 1200),
        ("Lua", 1000),
        ("Kotlin", 900),
        ("Swift", 800),
        ("Haskell", 700),
        ("Elixir", 600),
        ("Dart", 500),
        ("Scala", 400),
        ("OCaml", 300),
        ("Zig", 200),
        ("Nim", 150),
        ("Perl", 100),
    ];
    make_lang_report(&langs, true)
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
// Render helpers
// ===========================================================================

fn render_lang(report: &LangReport, format: TableFormat) -> String {
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, report, &default_scan(), &lang_args(format))
        .expect("operation must succeed");
    String::from_utf8(buf).expect("output must be valid UTF-8")
}

fn render_module(report: &ModuleReport, format: TableFormat) -> String {
    let mut buf = Vec::new();
    write_module_report_to(&mut buf, report, &default_scan(), &module_args(format))
        .expect("operation must succeed");
    String::from_utf8(buf).expect("output must be valid UTF-8")
}

fn render_export_csv(data: &ExportData) -> String {
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, data, &export_args(ExportFormat::Csv))
        .expect("operation must succeed");
    String::from_utf8(buf).expect("output must be valid UTF-8")
}

fn render_export_jsonl(data: &ExportData) -> String {
    let mut buf = Vec::new();
    write_export_jsonl_to(
        &mut buf,
        data,
        &default_scan(),
        &export_args(ExportFormat::Jsonl),
    )
    .expect("operation must succeed");
    String::from_utf8(buf).expect("output must be valid UTF-8")
}

fn render_export_json(data: &ExportData) -> String {
    let mut buf = Vec::new();
    write_export_json_to(
        &mut buf,
        data,
        &default_scan(),
        &export_args(ExportFormat::Json),
    )
    .expect("operation must succeed");
    String::from_utf8(buf).expect("output must be valid UTF-8")
}

fn normalize_json(raw: &str) -> String {
    let mut v: serde_json::Value = serde_json::from_str(raw).expect("operation must succeed");
    if let Some(obj) = v.as_object_mut() {
        obj.insert("generated_at_ms".into(), serde_json::json!(0));
        obj.insert(
            "tool".into(),
            serde_json::json!({"name": "tokmd", "version": "0.0.0-test"}),
        );
        obj.remove("scan");
    }
    serde_json::to_string_pretty(&v).expect("must serialize JSON")
}

// ===========================================================================
// Lang – Markdown
// ===========================================================================

#[test]
fn w58_lang_md_zero_languages() {
    let report = make_lang_report(&[], false);
    insta::assert_snapshot!(
        "w58_lang_md_zero_languages",
        render_lang(&report, TableFormat::Md)
    );
}

#[test]
fn w58_lang_md_single_with_files() {
    let report = make_lang_report(&[("Rust", 4200)], true);
    insta::assert_snapshot!(
        "w58_lang_md_single_with_files",
        render_lang(&report, TableFormat::Md)
    );
}

#[test]
fn w58_lang_md_five_languages() {
    let report = make_lang_report(
        &[
            ("Rust", 5000),
            ("Go", 3000),
            ("Python", 2000),
            ("TypeScript", 1000),
            ("Shell", 200),
        ],
        true,
    );
    insta::assert_snapshot!(
        "w58_lang_md_five_languages",
        render_lang(&report, TableFormat::Md)
    );
}

#[test]
fn w58_lang_md_twenty_plus_languages() {
    insta::assert_snapshot!(
        "w58_lang_md_twenty_plus_languages",
        render_lang(&twenty_plus_lang_report(), TableFormat::Md)
    );
}

#[test]
fn w58_lang_md_embedded_separate() {
    let report = LangReport {
        rows: vec![
            LangRow {
                lang: "HTML".into(),
                code: 600,
                lines: 750,
                files: 8,
                bytes: 18_000,
                tokens: 6_000,
                avg_lines: 93,
            },
            LangRow {
                lang: "JavaScript (embedded)".into(),
                code: 300,
                lines: 370,
                files: 8,
                bytes: 9_000,
                tokens: 3_000,
                avg_lines: 46,
            },
            LangRow {
                lang: "CSS (embedded)".into(),
                code: 150,
                lines: 180,
                files: 4,
                bytes: 4_500,
                tokens: 1_500,
                avg_lines: 45,
            },
        ],
        total: Totals {
            code: 1050,
            lines: 1300,
            files: 20,
            bytes: 31_500,
            tokens: 10_500,
            avg_lines: 65,
        },
        with_files: true,
        children: ChildrenMode::Separate,
        top: 0,
    };
    insta::assert_snapshot!(
        "w58_lang_md_embedded_separate",
        render_lang(&report, TableFormat::Md)
    );
}

#[test]
fn w58_lang_md_embedded_collapse() {
    let report = LangReport {
        rows: vec![LangRow {
            lang: "HTML".into(),
            code: 1050,
            lines: 1300,
            files: 8,
            bytes: 31_500,
            tokens: 10_500,
            avg_lines: 162,
        }],
        total: Totals {
            code: 1050,
            lines: 1300,
            files: 8,
            bytes: 31_500,
            tokens: 10_500,
            avg_lines: 162,
        },
        with_files: true,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    insta::assert_snapshot!(
        "w58_lang_md_embedded_collapse",
        render_lang(&report, TableFormat::Md)
    );
}

// ===========================================================================
// Lang – TSV
// ===========================================================================

#[test]
fn w58_lang_tsv_five_languages() {
    let report = make_lang_report(
        &[
            ("Rust", 5000),
            ("Go", 3000),
            ("Python", 2000),
            ("TypeScript", 1000),
            ("Shell", 200),
        ],
        true,
    );
    insta::assert_snapshot!(
        "w58_lang_tsv_five_languages",
        render_lang(&report, TableFormat::Tsv)
    );
}

#[test]
fn w58_lang_tsv_without_files() {
    let report = make_lang_report(&[("Rust", 3000), ("Go", 1500)], false);
    insta::assert_snapshot!(
        "w58_lang_tsv_without_files",
        render_lang(&report, TableFormat::Tsv)
    );
}

#[test]
fn w58_lang_tsv_zero_languages() {
    let report = make_lang_report(&[], true);
    insta::assert_snapshot!(
        "w58_lang_tsv_zero_languages",
        render_lang(&report, TableFormat::Tsv)
    );
}

// ===========================================================================
// Lang – JSON
// ===========================================================================

#[test]
fn w58_lang_json_five_languages() {
    let report = make_lang_report(
        &[
            ("Rust", 5000),
            ("Go", 3000),
            ("Python", 2000),
            ("TypeScript", 1000),
            ("Shell", 200),
        ],
        true,
    );
    let out = render_lang(&report, TableFormat::Json);
    insta::assert_snapshot!("w58_lang_json_five_languages", normalize_json(&out));
}

#[test]
fn w58_lang_json_single_language() {
    let report = make_lang_report(&[("Haskell", 999)], false);
    let out = render_lang(&report, TableFormat::Json);
    insta::assert_snapshot!("w58_lang_json_single_language", normalize_json(&out));
}

// ===========================================================================
// Module – Markdown / TSV / JSON
// ===========================================================================

fn deep_module_report() -> ModuleReport {
    ModuleReport {
        rows: vec![
            ModuleRow {
                module: "crates/tokmd-types".into(),
                code: 2500,
                lines: 3200,
                files: 12,
                bytes: 75_000,
                tokens: 20_000,
                avg_lines: 266,
            },
            ModuleRow {
                module: "crates/tokmd-scan".into(),
                code: 1800,
                lines: 2300,
                files: 8,
                bytes: 54_000,
                tokens: 14_400,
                avg_lines: 287,
            },
            ModuleRow {
                module: "crates/tokmd-format".into(),
                code: 1200,
                lines: 1500,
                files: 5,
                bytes: 36_000,
                tokens: 9_600,
                avg_lines: 300,
            },
            ModuleRow {
                module: "crates/tokmd-badge".into(),
                code: 200,
                lines: 260,
                files: 2,
                bytes: 6_000,
                tokens: 1_600,
                avg_lines: 130,
            },
            ModuleRow {
                module: "tests".into(),
                code: 600,
                lines: 750,
                files: 4,
                bytes: 18_000,
                tokens: 4_800,
                avg_lines: 187,
            },
        ],
        total: Totals {
            code: 6300,
            lines: 8010,
            files: 31,
            bytes: 189_000,
            tokens: 50_400,
            avg_lines: 258,
        },
        module_roots: vec!["crates".into()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
        top: 0,
    }
}

#[test]
fn w58_module_md_five_modules() {
    insta::assert_snapshot!(
        "w58_module_md_five_modules",
        render_module(&deep_module_report(), TableFormat::Md)
    );
}

#[test]
fn w58_module_tsv_five_modules() {
    insta::assert_snapshot!(
        "w58_module_tsv_five_modules",
        render_module(&deep_module_report(), TableFormat::Tsv)
    );
}

#[test]
fn w58_module_json_five_modules() {
    let out = render_module(&deep_module_report(), TableFormat::Json);
    insta::assert_snapshot!("w58_module_json_five_modules", normalize_json(&out));
}

// ===========================================================================
// Export – CSV / JSONL / JSON
// ===========================================================================

fn rich_export_data() -> ExportData {
    ExportData {
        rows: vec![
            FileRow {
                path: "src/engine.rs".into(),
                module: "src".into(),
                lang: "Rust".into(),
                kind: FileKind::Parent,
                code: 500,
                comments: 60,
                blanks: 40,
                lines: 600,
                bytes: 15_000,
                tokens: 5_000,
            },
            FileRow {
                path: "src/lib.rs".into(),
                module: "src".into(),
                lang: "Rust".into(),
                kind: FileKind::Parent,
                code: 250,
                comments: 30,
                blanks: 25,
                lines: 305,
                bytes: 7_500,
                tokens: 2_500,
            },
            FileRow {
                path: "src/template.html".into(),
                module: "src".into(),
                lang: "HTML".into(),
                kind: FileKind::Parent,
                code: 80,
                comments: 5,
                blanks: 10,
                lines: 95,
                bytes: 2_400,
                tokens: 800,
            },
            FileRow {
                path: "src/template.html".into(),
                module: "src".into(),
                lang: "JavaScript".into(),
                kind: FileKind::Child,
                code: 30,
                comments: 2,
                blanks: 3,
                lines: 35,
                bytes: 900,
                tokens: 300,
            },
        ],
        module_roots: vec!["src".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

#[test]
fn w58_export_csv_with_children() {
    insta::assert_snapshot!(
        "w58_export_csv_with_children",
        render_export_csv(&rich_export_data())
    );
}

#[test]
fn w58_export_jsonl_with_children() {
    insta::assert_snapshot!(
        "w58_export_jsonl_with_children",
        render_export_jsonl(&rich_export_data())
    );
}

#[test]
fn w58_export_json_rows_with_children() {
    insta::assert_snapshot!(
        "w58_export_json_rows_with_children",
        render_export_json(&rich_export_data())
    );
}

// ===========================================================================
// Diff rendering
// ===========================================================================

#[test]
fn w58_diff_md_growth() {
    let from = make_lang_report(&[("Rust", 1000), ("Python", 500)], true);
    let to = make_lang_report(&[("Rust", 1500), ("Python", 800), ("Go", 300)], true);
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    insta::assert_snapshot!(
        "w58_diff_md_growth",
        render_diff_md("v0.1.0", "v0.2.0", &rows, &totals)
    );
}

#[test]
fn w58_diff_md_compact() {
    let from = make_lang_report(&[("Rust", 2000)], true);
    let to = make_lang_report(&[("Rust", 1800)], true);
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let opts = DiffRenderOptions {
        compact: true,
        color: DiffColorMode::Off,
    };
    insta::assert_snapshot!(
        "w58_diff_md_compact",
        render_diff_md_with_options("before", "after", &rows, &totals, opts)
    );
}

#[test]
fn w58_diff_md_no_change() {
    let report = make_lang_report(&[("Rust", 1000)], true);
    let rows = compute_diff_rows(&report, &report);
    let totals = compute_diff_totals(&rows);
    insta::assert_snapshot!(
        "w58_diff_md_no_change",
        render_diff_md("a", "b", &rows, &totals)
    );
}
