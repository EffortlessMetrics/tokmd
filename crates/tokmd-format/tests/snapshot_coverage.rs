//! Extended snapshot coverage for output formatting.
//!
//! Supplements `snapshots.rs` with edge-case and breadth tests:
//! - various row counts (0, 1, 5, 20)
//! - TSV for module/export receipts
//! - JSON envelope completeness
//! - empty receipts, single-language repos, long language names
//! - ChildrenMode::Collapse vs Separate

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

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn global() -> ScanOptions {
    ScanOptions::default()
}

fn make_lang_row(lang: &str, code: usize) -> LangRow {
    LangRow {
        lang: lang.into(),
        code,
        lines: code + code / 5,
        files: std::cmp::max(1, code / 100),
        bytes: code * 50,
        tokens: code * 5 / 2,
        avg_lines: if code > 0 {
            (code + code / 5) / std::cmp::max(1, code / 100)
        } else {
            0
        },
    }
}

fn make_lang_report(rows: Vec<LangRow>, with_files: bool, children: ChildrenMode) -> LangReport {
    let total = Totals {
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
    };
    LangReport {
        rows,
        total,
        with_files,
        children,
        top: 0,
    }
}

fn make_module_row(module: &str, code: usize) -> ModuleRow {
    ModuleRow {
        module: module.into(),
        code,
        lines: code + code / 5,
        files: std::cmp::max(1, code / 100),
        bytes: code * 50,
        tokens: code * 5 / 2,
        avg_lines: if code > 0 {
            (code + code / 5) / std::cmp::max(1, code / 100)
        } else {
            0
        },
    }
}

fn make_module_report(rows: Vec<ModuleRow>, children: ChildIncludeMode) -> ModuleReport {
    let total = Totals {
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
    };
    ModuleReport {
        rows,
        total,
        module_roots: vec!["src".into()],
        module_depth: 2,
        children,
        top: 0,
    }
}

fn make_file_row(path: &str, lang: &str, kind: FileKind, code: usize) -> FileRow {
    FileRow {
        path: path.into(),
        module: path.rsplit_once('/').map_or(".", |p| p.0).into(),
        lang: lang.into(),
        kind,
        code,
        comments: code / 5,
        blanks: code / 10,
        lines: code + code / 5 + code / 10,
        bytes: code * 50,
        tokens: code * 5 / 2,
    }
}

fn make_export_data(rows: Vec<FileRow>, children: ChildIncludeMode) -> ExportData {
    ExportData {
        rows,
        module_roots: vec!["src".into()],
        module_depth: 1,
        children,
    }
}

fn lang_md_args(children: ChildrenMode) -> LangArgs {
    LangArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Md,
        top: 0,
        files: false,
        children,
    }
}

fn lang_tsv_args(children: ChildrenMode) -> LangArgs {
    LangArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Tsv,
        top: 0,
        files: false,
        children,
    }
}

fn lang_json_args(children: ChildrenMode) -> LangArgs {
    LangArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Json,
        top: 0,
        files: false,
        children,
    }
}

fn module_md_args(children: ChildIncludeMode) -> ModuleArgs {
    ModuleArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Md,
        top: 0,
        module_roots: vec!["src".into()],
        module_depth: 2,
        children,
    }
}

fn module_tsv_args(children: ChildIncludeMode) -> ModuleArgs {
    ModuleArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Tsv,
        top: 0,
        module_roots: vec!["src".into()],
        module_depth: 2,
        children,
    }
}

fn export_args(format: ExportFormat) -> ExportArgs {
    ExportArgs {
        paths: vec![PathBuf::from(".")],
        format,
        output: None,
        module_roots: vec!["src".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        redact: RedactMode::None,
        meta: false,
        strip_prefix: None,
    }
}

fn render_lang_md(report: &LangReport, children: ChildrenMode) -> String {
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, report, &global(), &lang_md_args(children)).unwrap();
    String::from_utf8(buf).unwrap()
}

fn render_lang_tsv(report: &LangReport, children: ChildrenMode) -> String {
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, report, &global(), &lang_tsv_args(children)).unwrap();
    String::from_utf8(buf).unwrap()
}

fn render_lang_json(report: &LangReport, children: ChildrenMode) -> String {
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, report, &global(), &lang_json_args(children)).unwrap();
    let raw = String::from_utf8(buf).unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    // Normalise non-deterministic fields
    v["generated_at_ms"] = serde_json::json!(0);
    v["tool"]["version"] = serde_json::json!("0.0.0");
    serde_json::to_string_pretty(&v).unwrap()
}

fn render_module_md(report: &ModuleReport, children: ChildIncludeMode) -> String {
    let mut buf = Vec::new();
    write_module_report_to(&mut buf, report, &global(), &module_md_args(children)).unwrap();
    String::from_utf8(buf).unwrap()
}

fn render_module_tsv(report: &ModuleReport, children: ChildIncludeMode) -> String {
    let mut buf = Vec::new();
    write_module_report_to(&mut buf, report, &global(), &module_tsv_args(children)).unwrap();
    String::from_utf8(buf).unwrap()
}

// =========================================================================
// 1. Markdown output — various row counts
// =========================================================================

#[test]
fn snapshot_lang_md_zero_rows() {
    let report = make_lang_report(vec![], false, ChildrenMode::Collapse);
    insta::assert_snapshot!(
        "lang_md_zero_rows",
        render_lang_md(&report, ChildrenMode::Collapse)
    );
}

#[test]
fn snapshot_lang_md_one_row() {
    let report = make_lang_report(
        vec![make_lang_row("Rust", 500)],
        false,
        ChildrenMode::Collapse,
    );
    insta::assert_snapshot!(
        "lang_md_one_row",
        render_lang_md(&report, ChildrenMode::Collapse)
    );
}

#[test]
fn snapshot_lang_md_five_rows() {
    let report = make_lang_report(
        vec![
            make_lang_row("Rust", 2000),
            make_lang_row("Python", 1500),
            make_lang_row("JavaScript", 800),
            make_lang_row("TOML", 200),
            make_lang_row("Markdown", 100),
        ],
        false,
        ChildrenMode::Collapse,
    );
    insta::assert_snapshot!(
        "lang_md_five_rows",
        render_lang_md(&report, ChildrenMode::Collapse)
    );
}

#[test]
fn snapshot_lang_md_twenty_rows() {
    let langs = [
        "Rust",
        "Python",
        "JavaScript",
        "TypeScript",
        "Go",
        "Java",
        "C",
        "C++",
        "C#",
        "Ruby",
        "Kotlin",
        "Swift",
        "Haskell",
        "OCaml",
        "Elixir",
        "Lua",
        "Shell",
        "TOML",
        "YAML",
        "JSON",
    ];
    let rows: Vec<LangRow> = langs
        .iter()
        .enumerate()
        .map(|(i, &l)| make_lang_row(l, 2000 - i * 90))
        .collect();
    let report = make_lang_report(rows, false, ChildrenMode::Collapse);
    insta::assert_snapshot!(
        "lang_md_twenty_rows",
        render_lang_md(&report, ChildrenMode::Collapse)
    );
}

#[test]
fn snapshot_lang_md_twenty_rows_with_files() {
    let langs = [
        "Rust",
        "Python",
        "JavaScript",
        "TypeScript",
        "Go",
        "Java",
        "C",
        "C++",
        "C#",
        "Ruby",
        "Kotlin",
        "Swift",
        "Haskell",
        "OCaml",
        "Elixir",
        "Lua",
        "Shell",
        "TOML",
        "YAML",
        "JSON",
    ];
    let rows: Vec<LangRow> = langs
        .iter()
        .enumerate()
        .map(|(i, &l)| make_lang_row(l, 2000 - i * 90))
        .collect();
    let report = make_lang_report(rows, true, ChildrenMode::Collapse);
    let mut buf = Vec::new();
    let args = LangArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Md,
        top: 0,
        files: true,
        children: ChildrenMode::Collapse,
    };
    write_lang_report_to(&mut buf, &report, &global(), &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    insta::assert_snapshot!("lang_md_twenty_rows_with_files", output);
}

// =========================================================================
// 2. TSV output — lang / module / export
// =========================================================================

#[test]
fn snapshot_lang_tsv_five_rows() {
    let report = make_lang_report(
        vec![
            make_lang_row("Rust", 2000),
            make_lang_row("Python", 1500),
            make_lang_row("JavaScript", 800),
            make_lang_row("TOML", 200),
            make_lang_row("Markdown", 100),
        ],
        false,
        ChildrenMode::Collapse,
    );
    insta::assert_snapshot!(
        "lang_tsv_five_rows",
        render_lang_tsv(&report, ChildrenMode::Collapse)
    );
}

#[test]
fn snapshot_module_tsv_multiple() {
    let report = make_module_report(
        vec![
            make_module_row("src/core", 1200),
            make_module_row("src/api", 600),
            make_module_row("src/util", 300),
        ],
        ChildIncludeMode::Separate,
    );
    insta::assert_snapshot!(
        "module_tsv_multiple",
        render_module_tsv(&report, ChildIncludeMode::Separate)
    );
}

#[test]
fn snapshot_export_csv_with_children() {
    let rows = vec![
        make_file_row("src/lib.rs", "Rust", FileKind::Parent, 200),
        make_file_row("src/lib.rs", "Markdown (Rust)", FileKind::Child, 30),
        make_file_row("src/util.rs", "Rust", FileKind::Parent, 100),
    ];
    let data = make_export_data(rows, ChildIncludeMode::Separate);
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &data, &export_args(ExportFormat::Csv)).unwrap();
    let output = String::from_utf8(buf).unwrap();
    insta::assert_snapshot!("export_csv_with_children", output);
}

#[test]
fn snapshot_export_jsonl_multiple_rows() {
    let rows = vec![
        make_file_row("src/lib.rs", "Rust", FileKind::Parent, 200),
        make_file_row("src/main.rs", "Rust", FileKind::Parent, 150),
        make_file_row("tests/test.rs", "Rust", FileKind::Parent, 80),
    ];
    let data = make_export_data(rows, ChildIncludeMode::Separate);
    let mut buf = Vec::new();
    write_export_jsonl_to(
        &mut buf,
        &data,
        &global(),
        &export_args(ExportFormat::Jsonl),
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    insta::assert_snapshot!("export_jsonl_multiple_rows", output);
}

// =========================================================================
// 3. JSON envelope — complete structure
// =========================================================================

#[test]
fn snapshot_lang_json_envelope_complete() {
    let report = make_lang_report(
        vec![make_lang_row("Rust", 1000), make_lang_row("Python", 400)],
        false,
        ChildrenMode::Collapse,
    );
    let json = render_lang_json(&report, ChildrenMode::Collapse);
    insta::assert_snapshot!("lang_json_envelope_complete", json);
}

#[test]
fn snapshot_module_json_envelope() {
    let report = make_module_report(
        vec![
            make_module_row("src/core", 800),
            make_module_row("src/api", 400),
        ],
        ChildIncludeMode::Separate,
    );
    let mut buf = Vec::new();
    let args = ModuleArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Json,
        top: 0,
        module_roots: vec!["src".into()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };
    write_module_report_to(&mut buf, &report, &global(), &args).unwrap();
    let raw = String::from_utf8(buf).unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    v["generated_at_ms"] = serde_json::json!(0);
    v["tool"]["version"] = serde_json::json!("0.0.0");
    let pretty = serde_json::to_string_pretty(&v).unwrap();
    insta::assert_snapshot!("module_json_envelope", pretty);
}

#[test]
fn snapshot_export_json_envelope() {
    let rows = vec![
        make_file_row("src/lib.rs", "Rust", FileKind::Parent, 200),
        make_file_row("src/util.rs", "Rust", FileKind::Parent, 100),
    ];
    let data = make_export_data(rows, ChildIncludeMode::Separate);
    let mut buf = Vec::new();
    write_export_json_to(&mut buf, &data, &global(), &export_args(ExportFormat::Json)).unwrap();
    let raw = String::from_utf8(buf).unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let pretty = serde_json::to_string_pretty(&v).unwrap();
    insta::assert_snapshot!("export_json_envelope", pretty);
}

// =========================================================================
// 4. Edge cases
// =========================================================================

#[test]
fn snapshot_lang_md_empty_receipt() {
    let report = make_lang_report(vec![], false, ChildrenMode::Collapse);
    insta::assert_snapshot!(
        "lang_md_empty_receipt",
        render_lang_md(&report, ChildrenMode::Collapse)
    );
}

#[test]
fn snapshot_lang_tsv_empty_receipt() {
    let report = make_lang_report(vec![], false, ChildrenMode::Collapse);
    insta::assert_snapshot!(
        "lang_tsv_empty_receipt",
        render_lang_tsv(&report, ChildrenMode::Collapse)
    );
}

#[test]
fn snapshot_lang_json_empty_receipt() {
    let report = make_lang_report(vec![], false, ChildrenMode::Collapse);
    let json = render_lang_json(&report, ChildrenMode::Collapse);
    insta::assert_snapshot!("lang_json_empty_receipt", json);
}

#[test]
fn snapshot_module_md_empty() {
    let report = make_module_report(vec![], ChildIncludeMode::Separate);
    insta::assert_snapshot!(
        "module_md_empty",
        render_module_md(&report, ChildIncludeMode::Separate)
    );
}

#[test]
fn snapshot_module_tsv_empty() {
    let report = make_module_report(vec![], ChildIncludeMode::Separate);
    insta::assert_snapshot!(
        "module_tsv_empty",
        render_module_tsv(&report, ChildIncludeMode::Separate)
    );
}

#[test]
fn snapshot_export_csv_empty() {
    let data = make_export_data(vec![], ChildIncludeMode::Separate);
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &data, &export_args(ExportFormat::Csv)).unwrap();
    let output = String::from_utf8(buf).unwrap();
    insta::assert_snapshot!("export_csv_empty", output);
}

#[test]
fn snapshot_export_jsonl_empty() {
    let data = make_export_data(vec![], ChildIncludeMode::Separate);
    let mut buf = Vec::new();
    write_export_jsonl_to(
        &mut buf,
        &data,
        &global(),
        &export_args(ExportFormat::Jsonl),
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    insta::assert_snapshot!("export_jsonl_empty", output);
}

#[test]
fn snapshot_lang_md_single_language() {
    let report = make_lang_report(
        vec![make_lang_row("Rust", 5000)],
        false,
        ChildrenMode::Collapse,
    );
    insta::assert_snapshot!(
        "lang_md_single_language",
        render_lang_md(&report, ChildrenMode::Collapse)
    );
}

#[test]
fn snapshot_lang_md_long_language_names() {
    let report = make_lang_report(
        vec![
            make_lang_row("Visual Basic .NET (Extremely Long Name)", 300),
            make_lang_row("Jupyter Notebook (Python Embedded)", 200),
            make_lang_row("C++ Header (Template Metaprogramming)", 100),
        ],
        false,
        ChildrenMode::Collapse,
    );
    insta::assert_snapshot!(
        "lang_md_long_names",
        render_lang_md(&report, ChildrenMode::Collapse)
    );
}

#[test]
fn snapshot_lang_tsv_long_language_names() {
    let report = make_lang_report(
        vec![
            make_lang_row("Visual Basic .NET (Extremely Long Name)", 300),
            make_lang_row("Jupyter Notebook (Python Embedded)", 200),
            make_lang_row("C++ Header (Template Metaprogramming)", 100),
        ],
        false,
        ChildrenMode::Collapse,
    );
    insta::assert_snapshot!(
        "lang_tsv_long_names",
        render_lang_tsv(&report, ChildrenMode::Collapse)
    );
}

// =========================================================================
// 5. Children mode — Collapse vs Separate
// =========================================================================

#[test]
fn snapshot_lang_md_children_collapse() {
    let report = make_lang_report(
        vec![make_lang_row("Rust", 1000), make_lang_row("HTML", 200)],
        false,
        ChildrenMode::Collapse,
    );
    insta::assert_snapshot!(
        "lang_md_children_collapse",
        render_lang_md(&report, ChildrenMode::Collapse)
    );
}

#[test]
fn snapshot_lang_md_children_separate() {
    let report = make_lang_report(
        vec![
            make_lang_row("Rust", 1000),
            make_lang_row("HTML (embedded)", 200),
        ],
        false,
        ChildrenMode::Separate,
    );
    insta::assert_snapshot!(
        "lang_md_children_separate",
        render_lang_md(&report, ChildrenMode::Separate)
    );
}

#[test]
fn snapshot_lang_tsv_children_collapse() {
    let report = make_lang_report(
        vec![make_lang_row("Rust", 1000), make_lang_row("HTML", 200)],
        false,
        ChildrenMode::Collapse,
    );
    insta::assert_snapshot!(
        "lang_tsv_children_collapse",
        render_lang_tsv(&report, ChildrenMode::Collapse)
    );
}

#[test]
fn snapshot_lang_tsv_children_separate() {
    let report = make_lang_report(
        vec![
            make_lang_row("Rust", 1000),
            make_lang_row("HTML (embedded)", 200),
        ],
        false,
        ChildrenMode::Separate,
    );
    insta::assert_snapshot!(
        "lang_tsv_children_separate",
        render_lang_tsv(&report, ChildrenMode::Separate)
    );
}

#[test]
fn snapshot_lang_json_children_collapse() {
    let report = make_lang_report(
        vec![make_lang_row("Rust", 1000), make_lang_row("HTML", 200)],
        false,
        ChildrenMode::Collapse,
    );
    let json = render_lang_json(&report, ChildrenMode::Collapse);
    insta::assert_snapshot!("lang_json_children_collapse", json);
}

#[test]
fn snapshot_lang_json_children_separate() {
    let report = make_lang_report(
        vec![
            make_lang_row("Rust", 1000),
            make_lang_row("HTML (embedded)", 200),
        ],
        false,
        ChildrenMode::Separate,
    );
    let json = render_lang_json(&report, ChildrenMode::Separate);
    insta::assert_snapshot!("lang_json_children_separate", json);
}

#[test]
fn snapshot_module_md_parents_only() {
    let report = make_module_report(
        vec![
            make_module_row("src/core", 800),
            make_module_row("src/api", 400),
        ],
        ChildIncludeMode::ParentsOnly,
    );
    insta::assert_snapshot!(
        "module_md_parents_only",
        render_module_md(&report, ChildIncludeMode::ParentsOnly)
    );
}

#[test]
fn snapshot_module_tsv_parents_only() {
    let report = make_module_report(
        vec![
            make_module_row("src/core", 800),
            make_module_row("src/api", 400),
        ],
        ChildIncludeMode::ParentsOnly,
    );
    insta::assert_snapshot!(
        "module_tsv_parents_only",
        render_module_tsv(&report, ChildIncludeMode::ParentsOnly)
    );
}

#[test]
fn snapshot_export_csv_parents_only() {
    let rows = vec![
        make_file_row("src/lib.rs", "Rust", FileKind::Parent, 200),
        make_file_row("src/util.rs", "Rust", FileKind::Parent, 100),
    ];
    let data = make_export_data(rows, ChildIncludeMode::ParentsOnly);
    let args = ExportArgs {
        paths: vec![PathBuf::from(".")],
        format: ExportFormat::Csv,
        output: None,
        module_roots: vec!["src".into()],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
        min_code: 0,
        max_rows: 0,
        redact: RedactMode::None,
        meta: false,
        strip_prefix: None,
    };
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &data, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    insta::assert_snapshot!("export_csv_parents_only", output);
}

#[test]
fn snapshot_export_jsonl_with_meta() {
    let rows = vec![
        make_file_row("src/lib.rs", "Rust", FileKind::Parent, 200),
        make_file_row("src/util.rs", "Rust", FileKind::Parent, 100),
    ];
    let data = make_export_data(rows, ChildIncludeMode::Separate);
    let mut args = export_args(ExportFormat::Jsonl);
    args.meta = true;
    let mut buf = Vec::new();
    write_export_jsonl_to(&mut buf, &data, &global(), &args).unwrap();
    let raw = String::from_utf8(buf).unwrap();
    // Normalise non-deterministic fields in the meta line
    let mut lines: Vec<String> = raw.lines().map(|l| l.to_string()).collect();
    if !lines.is_empty() {
        if let Ok(mut v) = serde_json::from_str::<serde_json::Value>(&lines[0]) {
            v["generated_at_ms"] = serde_json::json!(0);
            v["tool"]["version"] = serde_json::json!("0.0.0");
            lines[0] = serde_json::to_string(&v).unwrap();
        }
    }
    let normalised = lines.join("\n") + "\n";
    insta::assert_snapshot!("export_jsonl_with_meta", normalised);
}
