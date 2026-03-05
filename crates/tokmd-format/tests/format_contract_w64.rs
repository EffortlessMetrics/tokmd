//! Contract tests for tokmd-format (w64).
//!
//! Coverage:
//! - All output format variants (Markdown, TSV, JSON, JSONL, CSV)
//! - LangRow and ModuleRow rendering determinism
//! - Totals formatting
//! - Table alignment
//! - Property: render(parse(render(x))) == render(x) for JSON
//! - BDD: Given rows / When formatting / Then structure correct
//! - Edge: empty rows, single row, unicode language names
//! - Boundary: very long file paths, max column widths
//! - Snapshot: key format outputs via insta

use std::path::PathBuf;

use tokmd_format::{
    compute_diff_rows, compute_diff_totals, create_diff_receipt, render_diff_md,
    render_diff_md_with_options, write_export_csv_to, write_export_cyclonedx_with_options,
    write_export_json_to, write_export_jsonl_to, write_lang_report_to, write_module_report_to,
    DiffColorMode, DiffRenderOptions,
};
use tokmd_settings::ScanOptions;
use tokmd_types::{
    ChildIncludeMode, ChildrenMode, DiffRow, DiffTotals, ExportArgs, ExportData, ExportFormat,
    FileKind, FileRow, LangArgs, LangReceipt, LangReport, LangRow, ModuleArgs, ModuleReport,
    ModuleRow, RedactMode, TableFormat, Totals,
};

// ============================================================================
// Helpers
// ============================================================================

fn default_scan() -> ScanOptions {
    ScanOptions::default()
}

fn lang_row(lang: &str, code: usize) -> LangRow {
    LangRow {
        lang: lang.to_string(),
        code,
        lines: code + code / 5,
        files: code.checked_div(100).unwrap_or(1).max(1),
        bytes: code * 50,
        tokens: code * 3,
        avg_lines: code.checked_div(code.checked_div(100).unwrap_or(1).max(1)).unwrap_or(0),
    }
}

fn module_row(module: &str, code: usize) -> ModuleRow {
    ModuleRow {
        module: module.to_string(),
        code,
        lines: code + code / 4,
        files: code.checked_div(80).unwrap_or(1).max(1),
        bytes: code * 45,
        tokens: code * 2,
        avg_lines: code.checked_div(code.checked_div(80).unwrap_or(1).max(1)).unwrap_or(0),
    }
}

fn totals_from_lang_rows(rows: &[LangRow]) -> Totals {
    let code: usize = rows.iter().map(|r| r.code).sum();
    let lines: usize = rows.iter().map(|r| r.lines).sum();
    let files: usize = rows.iter().map(|r| r.files).sum();
    let bytes: usize = rows.iter().map(|r| r.bytes).sum();
    let tokens: usize = rows.iter().map(|r| r.tokens).sum();
    Totals {
        code,
        lines,
        files,
        bytes,
        tokens,
        avg_lines: code.checked_div(files).unwrap_or(0),
    }
}

fn totals_from_module_rows(rows: &[ModuleRow]) -> Totals {
    let code: usize = rows.iter().map(|r| r.code).sum();
    let lines: usize = rows.iter().map(|r| r.lines).sum();
    let files: usize = rows.iter().map(|r| r.files).sum();
    let bytes: usize = rows.iter().map(|r| r.bytes).sum();
    let tokens: usize = rows.iter().map(|r| r.tokens).sum();
    Totals {
        code,
        lines,
        files,
        bytes,
        tokens,
        avg_lines: code.checked_div(files).unwrap_or(0),
    }
}

fn make_lang_report(rows: Vec<LangRow>, with_files: bool) -> LangReport {
    let total = totals_from_lang_rows(&rows);
    LangReport {
        rows,
        total,
        with_files,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

fn make_module_report(rows: Vec<ModuleRow>) -> ModuleReport {
    let total = totals_from_module_rows(&rows);
    ModuleReport {
        rows,
        total,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        top: 0,
    }
}

fn lang_args(format: TableFormat) -> LangArgs {
    LangArgs {
        paths: vec![PathBuf::from(".")],
        format,
        top: 0,
        files: false,
        children: ChildrenMode::Collapse,
    }
}

fn module_args(format: TableFormat) -> ModuleArgs {
    ModuleArgs {
        paths: vec![PathBuf::from(".")],
        format,
        top: 0,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn file_row(path: &str, lang: &str, code: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: "src".to_string(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code,
        comments: code / 5,
        blanks: code / 10,
        lines: code + code / 5 + code / 10,
        bytes: code * 40,
        tokens: code * 3,
    }
}

fn export_data(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn export_args_csv() -> ExportArgs {
    ExportArgs {
        paths: vec![PathBuf::from(".")],
        format: ExportFormat::Csv,
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

fn export_args_jsonl() -> ExportArgs {
    ExportArgs {
        paths: vec![PathBuf::from(".")],
        format: ExportFormat::Jsonl,
        output: None,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        redact: RedactMode::None,
        meta: true,
        strip_prefix: None,
    }
}

fn export_args_json() -> ExportArgs {
    ExportArgs {
        paths: vec![PathBuf::from(".")],
        format: ExportFormat::Json,
        output: None,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        redact: RedactMode::None,
        meta: true,
        strip_prefix: None,
    }
}

fn render_lang_md(report: &LangReport) -> String {
    let args = lang_args(TableFormat::Md);
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, report, &default_scan(), &args).unwrap();
    String::from_utf8(buf).unwrap()
}

fn render_lang_tsv(report: &LangReport) -> String {
    let args = lang_args(TableFormat::Tsv);
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, report, &default_scan(), &args).unwrap();
    String::from_utf8(buf).unwrap()
}

fn render_lang_json(report: &LangReport) -> String {
    let args = lang_args(TableFormat::Json);
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, report, &default_scan(), &args).unwrap();
    String::from_utf8(buf).unwrap()
}

fn render_module_md(report: &ModuleReport) -> String {
    let args = module_args(TableFormat::Md);
    let mut buf = Vec::new();
    write_module_report_to(&mut buf, report, &default_scan(), &args).unwrap();
    String::from_utf8(buf).unwrap()
}

fn render_module_tsv(report: &ModuleReport) -> String {
    let args = module_args(TableFormat::Tsv);
    let mut buf = Vec::new();
    write_module_report_to(&mut buf, report, &default_scan(), &args).unwrap();
    String::from_utf8(buf).unwrap()
}

fn render_module_json(report: &ModuleReport) -> String {
    let args = module_args(TableFormat::Json);
    let mut buf = Vec::new();
    write_module_report_to(&mut buf, report, &default_scan(), &args).unwrap();
    String::from_utf8(buf).unwrap()
}

// ============================================================================
// 1. Lang Markdown rendering
// ============================================================================

#[test]
fn lang_md_header_without_files() {
    let report = make_lang_report(vec![lang_row("Rust", 100)], false);
    let out = render_lang_md(&report);
    assert!(out.starts_with("|Lang|Code|Lines|Bytes|Tokens|\n"));
    assert!(out.contains("|---|---:|---:|---:|---:|\n"));
}

#[test]
fn lang_md_header_with_files() {
    let report = make_lang_report(vec![lang_row("Rust", 100)], true);
    let out = render_lang_md(&report);
    assert!(out.starts_with("|Lang|Code|Lines|Files|Bytes|Tokens|Avg|\n"));
    assert!(out.contains("|---|---:|---:|---:|---:|---:|---:|\n"));
}

#[test]
fn lang_md_contains_row_data() {
    let report = make_lang_report(vec![lang_row("Rust", 500)], false);
    let out = render_lang_md(&report);
    assert!(out.contains("|Rust|500|"));
}

#[test]
fn lang_md_contains_total_row() {
    let report = make_lang_report(vec![lang_row("Rust", 500)], false);
    let out = render_lang_md(&report);
    assert!(out.contains("|**Total**|500|"));
}

#[test]
fn lang_md_multiple_rows_preserved_order() {
    let rows = vec![lang_row("Rust", 500), lang_row("Python", 200)];
    let report = make_lang_report(rows, false);
    let out = render_lang_md(&report);
    let rust_pos = out.find("|Rust|").unwrap();
    let py_pos = out.find("|Python|").unwrap();
    assert!(rust_pos < py_pos, "Rust should appear before Python");
}

#[test]
fn lang_md_pipe_delimited_rows() {
    let report = make_lang_report(vec![lang_row("Go", 300)], false);
    let out = render_lang_md(&report);
    for line in out.lines().skip(2) {
        // Data rows and total row start and end with pipe
        assert!(line.starts_with('|'), "Line should start with pipe: {line}");
        assert!(line.ends_with('|'), "Line should end with pipe: {line}");
    }
}

// ============================================================================
// 2. Lang TSV rendering
// ============================================================================

#[test]
fn lang_tsv_header_without_files() {
    let report = make_lang_report(vec![lang_row("Rust", 100)], false);
    let out = render_lang_tsv(&report);
    assert!(out.starts_with("Lang\tCode\tLines\tBytes\tTokens\n"));
}

#[test]
fn lang_tsv_header_with_files() {
    let report = make_lang_report(vec![lang_row("Rust", 100)], true);
    let out = render_lang_tsv(&report);
    assert!(out.starts_with("Lang\tCode\tLines\tFiles\tBytes\tTokens\tAvg\n"));
}

#[test]
fn lang_tsv_tab_separated_data() {
    let report = make_lang_report(vec![lang_row("Rust", 100)], false);
    let out = render_lang_tsv(&report);
    let data_line = out.lines().nth(1).unwrap();
    let fields: Vec<&str> = data_line.split('\t').collect();
    assert_eq!(fields[0], "Rust");
    assert_eq!(fields[1], "100");
}

#[test]
fn lang_tsv_total_row() {
    let report = make_lang_report(vec![lang_row("Rust", 100)], false);
    let out = render_lang_tsv(&report);
    let last_line = out.lines().last().unwrap();
    assert!(last_line.starts_with("Total\t"));
}

// ============================================================================
// 3. Lang JSON rendering
// ============================================================================

#[test]
fn lang_json_is_valid_json() {
    let report = make_lang_report(vec![lang_row("Rust", 100)], false);
    let out = render_lang_json(&report);
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert!(v.is_object());
}

#[test]
fn lang_json_has_schema_version() {
    let report = make_lang_report(vec![lang_row("Rust", 100)], false);
    let out = render_lang_json(&report);
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert!(v["schema_version"].is_number());
    assert_eq!(v["schema_version"].as_u64().unwrap(), 2);
}

#[test]
fn lang_json_has_tool_info() {
    let report = make_lang_report(vec![lang_row("Rust", 100)], false);
    let out = render_lang_json(&report);
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert_eq!(v["tool"]["name"].as_str().unwrap(), "tokmd");
}

#[test]
fn lang_json_contains_rows() {
    let report = make_lang_report(vec![lang_row("Rust", 100)], false);
    let out = render_lang_json(&report);
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    let rows = v["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["lang"].as_str().unwrap(), "Rust");
}

#[test]
fn lang_json_roundtrip_parse_render_idempotent() {
    let report = make_lang_report(vec![lang_row("Rust", 500), lang_row("Python", 200)], true);
    let out = render_lang_json(&report);
    let parsed: LangReceipt = serde_json::from_str(out.trim()).unwrap();
    let rendered_again = serde_json::to_string(&parsed).unwrap();
    let reparsed: LangReceipt = serde_json::from_str(&rendered_again).unwrap();
    let rendered_third = serde_json::to_string(&reparsed).unwrap();
    assert_eq!(rendered_again, rendered_third, "render(parse(render(x))) == render(x)");
}

#[test]
fn lang_json_mode_is_lang() {
    let report = make_lang_report(vec![lang_row("Rust", 100)], false);
    let out = render_lang_json(&report);
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert_eq!(v["mode"].as_str().unwrap(), "lang");
}

// ============================================================================
// 4. Module Markdown rendering
// ============================================================================

#[test]
fn module_md_header() {
    let report = make_module_report(vec![module_row("src", 100)]);
    let out = render_module_md(&report);
    assert!(out.starts_with("|Module|Code|Lines|Files|Bytes|Tokens|Avg|\n"));
}

#[test]
fn module_md_data_row() {
    let report = make_module_report(vec![module_row("src/lib", 400)]);
    let out = render_module_md(&report);
    assert!(out.contains("|src/lib|400|"));
}

#[test]
fn module_md_total_row() {
    let report = make_module_report(vec![module_row("src", 400)]);
    let out = render_module_md(&report);
    assert!(out.contains("|**Total**|400|"));
}

// ============================================================================
// 5. Module TSV rendering
// ============================================================================

#[test]
fn module_tsv_header() {
    let report = make_module_report(vec![module_row("src", 100)]);
    let out = render_module_tsv(&report);
    assert!(out.starts_with("Module\tCode\tLines\tFiles\tBytes\tTokens\tAvg\n"));
}

#[test]
fn module_tsv_data_row() {
    let report = make_module_report(vec![module_row("src", 400)]);
    let out = render_module_tsv(&report);
    let data_line = out.lines().nth(1).unwrap();
    assert!(data_line.starts_with("src\t400\t"));
}

// ============================================================================
// 6. Module JSON rendering
// ============================================================================

#[test]
fn module_json_valid() {
    let report = make_module_report(vec![module_row("src", 100)]);
    let out = render_module_json(&report);
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert_eq!(v["mode"].as_str().unwrap(), "module");
}

#[test]
fn module_json_has_schema_version() {
    let report = make_module_report(vec![module_row("src", 100)]);
    let out = render_module_json(&report);
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert_eq!(v["schema_version"].as_u64().unwrap(), 2);
}

#[test]
fn module_json_contains_rows() {
    let report = make_module_report(vec![module_row("src", 100)]);
    let out = render_module_json(&report);
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    let rows = v["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["module"].as_str().unwrap(), "src");
}

// ============================================================================
// 7. Export CSV
// ============================================================================

#[test]
fn export_csv_header_row() {
    let data = export_data(vec![file_row("src/main.rs", "Rust", 100)]);
    let args = export_args_csv();
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &data, &args).unwrap();
    let out = String::from_utf8(buf).unwrap();
    let header = out.lines().next().unwrap();
    assert!(header.contains("path"));
    assert!(header.contains("lang"));
    assert!(header.contains("code"));
}

#[test]
fn export_csv_data_row_count() {
    let data = export_data(vec![
        file_row("src/a.rs", "Rust", 100),
        file_row("src/b.rs", "Rust", 200),
    ]);
    let args = export_args_csv();
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &data, &args).unwrap();
    let out = String::from_utf8(buf).unwrap();
    // header + 2 data rows
    assert_eq!(out.lines().count(), 3);
}

#[test]
fn export_csv_contains_path() {
    let data = export_data(vec![file_row("src/main.rs", "Rust", 100)]);
    let args = export_args_csv();
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &data, &args).unwrap();
    let out = String::from_utf8(buf).unwrap();
    assert!(out.contains("src/main.rs"));
}

// ============================================================================
// 8. Export JSONL
// ============================================================================

#[test]
fn export_jsonl_first_line_is_meta() {
    let data = export_data(vec![file_row("src/main.rs", "Rust", 100)]);
    let args = export_args_jsonl();
    let mut buf = Vec::new();
    write_export_jsonl_to(&mut buf, &data, &default_scan(), &args).unwrap();
    let out = String::from_utf8(buf).unwrap();
    let first_line = out.lines().next().unwrap();
    let v: serde_json::Value = serde_json::from_str(first_line).unwrap();
    assert_eq!(v["type"].as_str().unwrap(), "meta");
}

#[test]
fn export_jsonl_data_lines_are_rows() {
    let data = export_data(vec![file_row("src/main.rs", "Rust", 100)]);
    let args = export_args_jsonl();
    let mut buf = Vec::new();
    write_export_jsonl_to(&mut buf, &data, &default_scan(), &args).unwrap();
    let out = String::from_utf8(buf).unwrap();
    let second_line = out.lines().nth(1).unwrap();
    let v: serde_json::Value = serde_json::from_str(second_line).unwrap();
    assert_eq!(v["type"].as_str().unwrap(), "row");
    assert_eq!(v["path"].as_str().unwrap(), "src/main.rs");
}

#[test]
fn export_jsonl_each_line_valid_json() {
    let data = export_data(vec![
        file_row("src/a.rs", "Rust", 100),
        file_row("src/b.py", "Python", 50),
    ]);
    let args = export_args_jsonl();
    let mut buf = Vec::new();
    write_export_jsonl_to(&mut buf, &data, &default_scan(), &args).unwrap();
    let out = String::from_utf8(buf).unwrap();
    for line in out.lines() {
        let _: serde_json::Value =
            serde_json::from_str(line).expect("Every JSONL line must be valid JSON");
    }
}

// ============================================================================
// 9. Export JSON
// ============================================================================

#[test]
fn export_json_valid() {
    let data = export_data(vec![file_row("src/main.rs", "Rust", 100)]);
    let args = export_args_json();
    let mut buf = Vec::new();
    write_export_json_to(&mut buf, &data, &default_scan(), &args).unwrap();
    let out = String::from_utf8(buf).unwrap();
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert!(v.is_object());
}

#[test]
fn export_json_has_rows_array() {
    let data = export_data(vec![file_row("src/main.rs", "Rust", 100)]);
    let args = export_args_json();
    let mut buf = Vec::new();
    write_export_json_to(&mut buf, &data, &default_scan(), &args).unwrap();
    let out = String::from_utf8(buf).unwrap();
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert!(v["rows"].is_array());
}

#[test]
fn export_json_without_meta() {
    let data = export_data(vec![file_row("src/main.rs", "Rust", 100)]);
    let mut args = export_args_json();
    args.meta = false;
    let mut buf = Vec::new();
    write_export_json_to(&mut buf, &data, &default_scan(), &args).unwrap();
    let out = String::from_utf8(buf).unwrap();
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    // Without meta, output is a bare array of rows
    assert!(v.is_array());
}

// ============================================================================
// 10. Export CycloneDX
// ============================================================================

#[test]
fn export_cyclonedx_has_bom_format() {
    let data = export_data(vec![file_row("src/main.rs", "Rust", 100)]);
    let mut buf = Vec::new();
    write_export_cyclonedx_with_options(
        &mut buf,
        &data,
        RedactMode::None,
        Some("urn:uuid:test-1234".to_string()),
        Some("2024-01-01T00:00:00Z".to_string()),
    )
    .unwrap();
    let out = String::from_utf8(buf).unwrap();
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert_eq!(v["bomFormat"].as_str().unwrap(), "CycloneDX");
}

#[test]
fn export_cyclonedx_components_match_rows() {
    let rows = vec![
        file_row("src/a.rs", "Rust", 100),
        file_row("src/b.rs", "Rust", 200),
    ];
    let data = export_data(rows);
    let mut buf = Vec::new();
    write_export_cyclonedx_with_options(
        &mut buf,
        &data,
        RedactMode::None,
        Some("urn:uuid:test".to_string()),
        Some("2024-01-01T00:00:00Z".to_string()),
    )
    .unwrap();
    let out = String::from_utf8(buf).unwrap();
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    let components = v["components"].as_array().unwrap();
    assert_eq!(components.len(), 2);
}

// ============================================================================
// 11. Determinism: same input => same output
// ============================================================================

#[test]
fn lang_md_deterministic() {
    let report = make_lang_report(vec![lang_row("Rust", 500), lang_row("Go", 300)], true);
    let a = render_lang_md(&report);
    let b = render_lang_md(&report);
    assert_eq!(a, b, "Markdown output must be deterministic");
}

#[test]
fn lang_tsv_deterministic() {
    let report = make_lang_report(vec![lang_row("Rust", 500), lang_row("Go", 300)], true);
    let a = render_lang_tsv(&report);
    let b = render_lang_tsv(&report);
    assert_eq!(a, b, "TSV output must be deterministic");
}

#[test]
fn module_md_deterministic() {
    let report = make_module_report(vec![module_row("src", 400), module_row("tests", 100)]);
    let a = render_module_md(&report);
    let b = render_module_md(&report);
    assert_eq!(a, b, "Module Markdown must be deterministic");
}

#[test]
fn export_csv_deterministic() {
    let data = export_data(vec![
        file_row("src/a.rs", "Rust", 100),
        file_row("src/b.py", "Python", 50),
    ]);
    let args = export_args_csv();
    let render = || {
        let mut buf = Vec::new();
        write_export_csv_to(&mut buf, &data, &args).unwrap();
        String::from_utf8(buf).unwrap()
    };
    assert_eq!(render(), render(), "CSV output must be deterministic");
}

// ============================================================================
// 12. Diff computation
// ============================================================================

#[test]
fn diff_rows_detect_changes() {
    let from = make_lang_report(vec![lang_row("Rust", 100)], false);
    let to = make_lang_report(vec![lang_row("Rust", 200)], false);
    let rows = compute_diff_rows(&from, &to);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].delta_code, 100);
}

#[test]
fn diff_rows_skip_unchanged() {
    let from = make_lang_report(vec![lang_row("Rust", 100)], false);
    let to = make_lang_report(vec![lang_row("Rust", 100)], false);
    let rows = compute_diff_rows(&from, &to);
    assert!(rows.is_empty(), "Unchanged languages should be omitted");
}

#[test]
fn diff_rows_new_language_appears() {
    let from = make_lang_report(vec![], false);
    let to = make_lang_report(vec![lang_row("Rust", 100)], false);
    let rows = compute_diff_rows(&from, &to);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].old_code, 0);
    assert_eq!(rows[0].new_code, 100);
}

#[test]
fn diff_rows_language_removed() {
    let from = make_lang_report(vec![lang_row("Rust", 100)], false);
    let to = make_lang_report(vec![], false);
    let rows = compute_diff_rows(&from, &to);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].delta_code, -100);
}

#[test]
fn diff_totals_aggregate() {
    let rows = [
        DiffRow {
            lang: "Rust".into(),
            old_code: 100,
            new_code: 200,
            delta_code: 100,
            old_lines: 120,
            new_lines: 240,
            delta_lines: 120,
            old_files: 5,
            new_files: 8,
            delta_files: 3,
            old_bytes: 5000,
            new_bytes: 10000,
            delta_bytes: 5000,
            old_tokens: 300,
            new_tokens: 600,
            delta_tokens: 300,
        },
        DiffRow {
            lang: "Python".into(),
            old_code: 50,
            new_code: 30,
            delta_code: -20,
            old_lines: 60,
            new_lines: 36,
            delta_lines: -24,
            old_files: 2,
            new_files: 1,
            delta_files: -1,
            old_bytes: 2500,
            new_bytes: 1500,
            delta_bytes: -1000,
            old_tokens: 150,
            new_tokens: 90,
            delta_tokens: -60,
        },
    ];
    let totals = compute_diff_totals(&rows);
    assert_eq!(totals.delta_code, 80);
    assert_eq!(totals.delta_files, 2);
    assert_eq!(totals.delta_tokens, 240);
}

#[test]
fn diff_totals_empty() {
    let totals = compute_diff_totals(&[]);
    assert_eq!(totals, DiffTotals::default());
}

// ============================================================================
// 13. Diff rendering
// ============================================================================

#[test]
fn diff_md_contains_source_labels() {
    let from = make_lang_report(vec![lang_row("Rust", 100)], false);
    let to = make_lang_report(vec![lang_row("Rust", 200)], false);
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let md = render_diff_md("v0.1", "v0.2", &rows, &totals);
    assert!(md.contains("v0.1"));
    assert!(md.contains("v0.2"));
}

#[test]
fn diff_md_contains_delta_with_sign() {
    let from = make_lang_report(vec![lang_row("Rust", 100)], false);
    let to = make_lang_report(vec![lang_row("Rust", 200)], false);
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let md = render_diff_md("a", "b", &rows, &totals);
    assert!(md.contains("+100"), "Positive delta should have + prefix");
}

#[test]
fn diff_receipt_has_schema_version() {
    let receipt = create_diff_receipt("a", "b", vec![], DiffTotals::default());
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
}

#[test]
fn diff_render_options_compact() {
    let from = make_lang_report(vec![lang_row("Rust", 100)], false);
    let to = make_lang_report(vec![lang_row("Rust", 200)], false);
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let opts = DiffRenderOptions {
        compact: true,
        color: DiffColorMode::Off,
    };
    let md = render_diff_md_with_options("a", "b", &rows, &totals, opts);
    assert!(!md.is_empty());
}

#[test]
fn diff_render_colored_contains_ansi() {
    let from = make_lang_report(vec![lang_row("Rust", 100)], false);
    let to = make_lang_report(vec![lang_row("Rust", 200)], false);
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let opts = DiffRenderOptions {
        compact: false,
        color: DiffColorMode::Ansi,
    };
    let md = render_diff_md_with_options("a", "b", &rows, &totals, opts);
    assert!(md.contains("\x1b["), "ANSI color codes expected");
}

// ============================================================================
// 14. BDD: Given/When/Then
// ============================================================================

#[test]
fn bdd_given_two_langs_when_md_then_correct_table() {
    // Given
    let rows = vec![lang_row("Rust", 500), lang_row("Python", 200)];
    let report = make_lang_report(rows, true);

    // When
    let md = render_lang_md(&report);

    // Then: header + separator + 2 data rows + total = 5 lines
    let lines: Vec<&str> = md.lines().collect();
    assert_eq!(lines.len(), 5, "Expected header + sep + 2 rows + total");
    assert!(lines[0].contains("Lang"));
    assert!(lines[2].contains("Rust"));
    assert!(lines[3].contains("Python"));
    assert!(lines[4].contains("**Total**"));
}

#[test]
fn bdd_given_module_rows_when_tsv_then_tab_delimited() {
    // Given
    let report = make_module_report(vec![module_row("src", 400), module_row("tests", 100)]);

    // When
    let tsv = render_module_tsv(&report);

    // Then: each line has correct number of tabs
    for line in tsv.lines() {
        let tab_count = line.matches('\t').count();
        assert_eq!(tab_count, 6, "Expected 6 tabs per line, got {tab_count}: {line}");
    }
}

#[test]
fn bdd_given_file_rows_when_csv_then_parseable() {
    // Given
    let data = export_data(vec![
        file_row("src/main.rs", "Rust", 500),
        file_row("src/lib.rs", "Rust", 300),
    ]);

    // When
    let args = export_args_csv();
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &data, &args).unwrap();
    let out = String::from_utf8(buf).unwrap();

    // Then: parseable CSV with correct column count
    let mut rdr = csv::Reader::from_reader(out.as_bytes());
    let headers = rdr.headers().unwrap();
    assert_eq!(headers.len(), 10);
    let records: Vec<_> = rdr.records().map(|r| r.unwrap()).collect();
    assert_eq!(records.len(), 2);
}

// ============================================================================
// 15. Edge cases: empty rows
// ============================================================================

#[test]
fn lang_md_empty_rows() {
    let report = make_lang_report(vec![], false);
    let out = render_lang_md(&report);
    // Header + separator + total = 3 lines
    assert_eq!(out.lines().count(), 3);
    assert!(out.contains("|**Total**|0|"));
}

#[test]
fn lang_tsv_empty_rows() {
    let report = make_lang_report(vec![], false);
    let out = render_lang_tsv(&report);
    // Header + total = 2 lines
    assert_eq!(out.lines().count(), 2);
}

#[test]
fn module_md_empty_rows() {
    let report = make_module_report(vec![]);
    let out = render_module_md(&report);
    assert_eq!(out.lines().count(), 3);
}

#[test]
fn export_csv_empty_rows() {
    let data = export_data(vec![]);
    let args = export_args_csv();
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &data, &args).unwrap();
    let out = String::from_utf8(buf).unwrap();
    // Just the header
    assert_eq!(out.lines().count(), 1);
}

#[test]
fn diff_rows_both_empty() {
    let from = make_lang_report(vec![], false);
    let to = make_lang_report(vec![], false);
    let rows = compute_diff_rows(&from, &to);
    assert!(rows.is_empty());
}

// ============================================================================
// 16. Edge: single row
// ============================================================================

#[test]
fn lang_md_single_row_structure() {
    let report = make_lang_report(vec![lang_row("Rust", 1000)], false);
    let out = render_lang_md(&report);
    assert_eq!(out.lines().count(), 4); // header + sep + 1 row + total
}

#[test]
fn module_md_single_row_structure() {
    let report = make_module_report(vec![module_row("src", 500)]);
    let out = render_module_md(&report);
    assert_eq!(out.lines().count(), 4);
}

// ============================================================================
// 17. Edge: unicode language names
// ============================================================================

#[test]
fn lang_md_unicode_name() {
    let report = make_lang_report(vec![lang_row("日本語", 100)], false);
    let out = render_lang_md(&report);
    assert!(out.contains("|日本語|"));
}

#[test]
fn lang_json_unicode_roundtrip() {
    let report = make_lang_report(vec![lang_row("中文", 100)], false);
    let out = render_lang_json(&report);
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert_eq!(v["rows"][0]["lang"].as_str().unwrap(), "中文");
}

#[test]
fn module_md_unicode_name() {
    let report = make_module_report(vec![module_row("プロジェクト/src", 200)]);
    let out = render_module_md(&report);
    assert!(out.contains("|プロジェクト/src|"));
}

// ============================================================================
// 18. Boundary: very long file paths
// ============================================================================

#[test]
fn export_csv_long_path() {
    let long_path = format!("src/{}/main.rs", "deep/".repeat(50));
    let data = export_data(vec![file_row(&long_path, "Rust", 100)]);
    let args = export_args_csv();
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &data, &args).unwrap();
    let out = String::from_utf8(buf).unwrap();
    assert!(out.contains(&long_path));
}

#[test]
fn export_jsonl_long_path() {
    let long_path = format!("src/{}/main.rs", "nested/".repeat(100));
    let data = export_data(vec![file_row(&long_path, "Rust", 100)]);
    let args = export_args_jsonl();
    let mut buf = Vec::new();
    write_export_jsonl_to(&mut buf, &data, &default_scan(), &args).unwrap();
    let out = String::from_utf8(buf).unwrap();
    let row_line = out.lines().nth(1).unwrap();
    let v: serde_json::Value = serde_json::from_str(row_line).unwrap();
    assert_eq!(v["path"].as_str().unwrap(), long_path);
}

// ============================================================================
// 19. Boundary: large code values
// ============================================================================

#[test]
fn lang_md_large_numbers() {
    let report = make_lang_report(vec![lang_row("Rust", 999_999_999)], false);
    let out = render_lang_md(&report);
    assert!(out.contains("999999999"));
}

#[test]
fn lang_json_large_numbers() {
    let report = make_lang_report(vec![lang_row("Rust", 999_999_999)], false);
    let out = render_lang_json(&report);
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert_eq!(v["rows"][0]["code"].as_u64().unwrap(), 999_999_999);
}

// ============================================================================
// 20. Boundary: zero values
// ============================================================================

#[test]
fn lang_md_zero_code() {
    let report = make_lang_report(vec![lang_row("Rust", 0)], false);
    let out = render_lang_md(&report);
    assert!(out.contains("|Rust|0|"));
}

#[test]
fn export_csv_zero_values() {
    let data = export_data(vec![file_row("empty.rs", "Rust", 0)]);
    let args = export_args_csv();
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &data, &args).unwrap();
    let out = String::from_utf8(buf).unwrap();
    assert!(out.contains("empty.rs"));
}

// ============================================================================
// 21. Redaction modes
// ============================================================================

#[test]
fn export_csv_redact_paths() {
    let data = export_data(vec![file_row("src/secret.rs", "Rust", 100)]);
    let mut args = export_args_csv();
    args.redact = RedactMode::Paths;
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &data, &args).unwrap();
    let out = String::from_utf8(buf).unwrap();
    assert!(!out.contains("src/secret.rs"), "Path should be redacted");
}

#[test]
fn export_csv_redact_all() {
    let data = export_data(vec![file_row("src/secret.rs", "Rust", 100)]);
    let mut args = export_args_csv();
    args.redact = RedactMode::All;
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &data, &args).unwrap();
    let out = String::from_utf8(buf).unwrap();
    assert!(!out.contains("src/secret.rs"), "Path should be redacted");
    assert!(!out.contains(",src,"), "Module should be redacted");
}

// ============================================================================
// 22. with_files toggle
// ============================================================================

#[test]
fn lang_md_with_files_shows_files_column() {
    let report = make_lang_report(vec![lang_row("Rust", 500)], true);
    let md = render_lang_md(&report);
    assert!(md.contains("|Files|"));
    assert!(md.contains("|Avg|"));
}

#[test]
fn lang_md_without_files_omits_files_column() {
    let report = make_lang_report(vec![lang_row("Rust", 500)], false);
    let md = render_lang_md(&report);
    assert!(!md.contains("|Files|"));
    assert!(!md.contains("|Avg|"));
}

#[test]
fn lang_tsv_with_files_has_seven_columns() {
    let report = make_lang_report(vec![lang_row("Rust", 500)], true);
    let tsv = render_lang_tsv(&report);
    let header_cols = tsv.lines().next().unwrap().split('\t').count();
    assert_eq!(header_cols, 7);
}

#[test]
fn lang_tsv_without_files_has_five_columns() {
    let report = make_lang_report(vec![lang_row("Rust", 500)], false);
    let tsv = render_lang_tsv(&report);
    let header_cols = tsv.lines().next().unwrap().split('\t').count();
    assert_eq!(header_cols, 5);
}

// ============================================================================
// 23. ChildrenMode variants
// ============================================================================

#[test]
fn lang_json_children_collapse() {
    let mut report = make_lang_report(vec![lang_row("Rust", 100)], false);
    report.children = ChildrenMode::Collapse;
    let out = render_lang_json(&report);
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert_eq!(v["children"].as_str().unwrap(), "collapse");
}

#[test]
fn lang_json_children_separate() {
    let mut report = make_lang_report(vec![lang_row("Rust", 100)], false);
    report.children = ChildrenMode::Separate;
    let out = render_lang_json(&report);
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert_eq!(v["children"].as_str().unwrap(), "separate");
}

// ============================================================================
// 24. Snapshot tests (insta)
// ============================================================================

#[test]
fn snapshot_lang_md_two_langs() {
    let report = make_lang_report(
        vec![lang_row("Rust", 500), lang_row("Python", 200)],
        true,
    );
    let out = render_lang_md(&report);
    insta::assert_snapshot!("w64_lang_md_two_langs", out);
}

#[test]
fn snapshot_module_md_two_modules() {
    let report = make_module_report(vec![module_row("src", 400), module_row("tests", 100)]);
    let out = render_module_md(&report);
    insta::assert_snapshot!("w64_module_md_two_modules", out);
}

#[test]
fn snapshot_lang_tsv_with_files() {
    let report = make_lang_report(
        vec![lang_row("Rust", 500), lang_row("Go", 300)],
        true,
    );
    let out = render_lang_tsv(&report);
    insta::assert_snapshot!("w64_lang_tsv_with_files", out);
}
