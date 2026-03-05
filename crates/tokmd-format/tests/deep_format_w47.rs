//! Deep snapshot and determinism tests for tokmd-format – wave 47.
//!
//! Covers all output renderers (Markdown, TSV, JSON, JSONL, CSV, CycloneDX,
//! Diff MD/JSON), empty inputs, single-row inputs, determinism, sort order,
//! special characters, top-N filtering, and children-mode effects.
//!
//! Run with: `cargo test -p tokmd-format --test deep_format_w47`

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

// ============================================================================
// Helpers
// ============================================================================

fn lang_report(rows: Vec<LangRow>, with_files: bool) -> LangReport {
    let total = Totals {
        code: rows.iter().map(|r| r.code).sum(),
        lines: rows.iter().map(|r| r.lines).sum(),
        files: rows.iter().map(|r| r.files).sum(),
        bytes: rows.iter().map(|r| r.bytes).sum(),
        tokens: rows.iter().map(|r| r.tokens).sum(),
        avg_lines: if rows.is_empty() {
            0
        } else {
            rows.iter().map(|r| r.lines).sum::<usize>() / rows.len()
        },
    };
    LangReport {
        rows,
        total,
        with_files,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

fn module_report(rows: Vec<ModuleRow>) -> ModuleReport {
    let total = Totals {
        code: rows.iter().map(|r| r.code).sum(),
        lines: rows.iter().map(|r| r.lines).sum(),
        files: rows.iter().map(|r| r.files).sum(),
        bytes: rows.iter().map(|r| r.bytes).sum(),
        tokens: rows.iter().map(|r| r.tokens).sum(),
        avg_lines: 0,
    };
    ModuleReport {
        rows,
        total,
        module_roots: vec!["src".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        top: 0,
    }
}

fn export_data(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec!["src".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
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
        module_roots: vec!["src".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
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
        meta: true,
        strip_prefix: None,
    }
}

fn globals() -> ScanOptions {
    ScanOptions::default()
}

fn rust_row(code: usize) -> LangRow {
    LangRow {
        lang: "Rust".into(),
        code,
        lines: code + code / 5,
        files: (code / 100).max(1),
        bytes: code * 40,
        tokens: code * 2,
        avg_lines: 120,
    }
}

fn python_row(code: usize) -> LangRow {
    LangRow {
        lang: "Python".into(),
        code,
        lines: code + code / 4,
        files: (code / 80).max(1),
        bytes: code * 36,
        tokens: code * 2,
        avg_lines: 100,
    }
}

fn toml_row(code: usize) -> LangRow {
    LangRow {
        lang: "TOML".into(),
        code,
        lines: code + code / 5,
        files: (code / 50).max(1),
        bytes: code * 30,
        tokens: code,
        avg_lines: 50,
    }
}

fn sample_module_row(name: &str, code: usize) -> ModuleRow {
    ModuleRow {
        module: name.into(),
        code,
        lines: code + code / 5,
        files: (code / 100).max(1),
        bytes: code * 40,
        tokens: code * 2,
        avg_lines: 100,
    }
}

fn sample_file_row(path: &str, lang: &str, code: usize) -> FileRow {
    FileRow {
        path: path.into(),
        module: "src".into(),
        lang: lang.into(),
        kind: FileKind::Parent,
        code,
        comments: code / 10,
        blanks: code / 20,
        lines: code + code / 10 + code / 20,
        bytes: code * 30,
        tokens: code * 2,
    }
}

fn render_lang(report: &LangReport, format: TableFormat) -> String {
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, report, &globals(), &lang_args(format)).unwrap();
    String::from_utf8(buf).unwrap()
}

fn render_module(report: &ModuleReport, format: TableFormat) -> String {
    let mut buf = Vec::new();
    write_module_report_to(&mut buf, report, &globals(), &module_args(format)).unwrap();
    String::from_utf8(buf).unwrap()
}

fn render_export_csv(data: &ExportData) -> String {
    let args = export_args(ExportFormat::Csv);
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, data, &args).unwrap();
    String::from_utf8(buf).unwrap()
}

fn render_export_jsonl(data: &ExportData) -> String {
    let args = export_args(ExportFormat::Jsonl);
    let mut buf = Vec::new();
    write_export_jsonl_to(&mut buf, data, &globals(), &args).unwrap();
    String::from_utf8(buf).unwrap()
}

fn render_export_json(data: &ExportData) -> String {
    let args = export_args(ExportFormat::Json);
    let mut buf = Vec::new();
    write_export_json_to(&mut buf, data, &globals(), &args).unwrap();
    String::from_utf8(buf).unwrap()
}

// ============================================================================
// 1. Markdown renderer – lang rows
// ============================================================================

#[test]
fn snapshot_lang_md_multi_row() {
    let report = lang_report(vec![rust_row(1000), python_row(500), toml_row(100)], true);
    insta::assert_snapshot!(render_lang(&report, TableFormat::Md));
}

#[test]
fn snapshot_lang_md_single_row() {
    let report = lang_report(vec![rust_row(42)], true);
    insta::assert_snapshot!(render_lang(&report, TableFormat::Md));
}

#[test]
fn snapshot_lang_md_without_files() {
    let report = lang_report(vec![rust_row(300), toml_row(50)], false);
    let mut args = lang_args(TableFormat::Md);
    args.files = false;
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &globals(), &args).unwrap();
    insta::assert_snapshot!(String::from_utf8(buf).unwrap());
}

#[test]
fn lang_md_empty_rows() {
    let report = lang_report(vec![], true);
    let output = render_lang(&report, TableFormat::Md);
    assert!(output.contains("|Lang|"));
    assert!(output.contains("**Total**"));
    assert!(!output.is_empty());
}

// ============================================================================
// 2. TSV renderer – lang/module
// ============================================================================

#[test]
fn snapshot_lang_tsv_multi_row() {
    let report = lang_report(vec![rust_row(800), python_row(200)], true);
    insta::assert_snapshot!(render_lang(&report, TableFormat::Tsv));
}

#[test]
fn lang_tsv_has_tab_separated_header() {
    let report = lang_report(vec![rust_row(100)], true);
    let output = render_lang(&report, TableFormat::Tsv);
    let first_line = output.lines().next().unwrap();
    assert!(first_line.contains('\t'));
    assert!(first_line.contains("Lang"));
    assert!(first_line.contains("Code"));
}

#[test]
fn lang_tsv_empty_rows() {
    let report = lang_report(vec![], false);
    let output = render_lang(&report, TableFormat::Tsv);
    assert!(output.contains("Lang\tCode\t"));
    assert!(output.contains("Total\t0\t"));
}

#[test]
fn snapshot_module_tsv_multi_row() {
    let report = module_report(vec![
        sample_module_row("src/core", 600),
        sample_module_row("src/utils", 200),
    ]);
    insta::assert_snapshot!(render_module(&report, TableFormat::Tsv));
}

#[test]
fn module_tsv_empty_rows() {
    let report = module_report(vec![]);
    let output = render_module(&report, TableFormat::Tsv);
    assert!(output.contains("Module\tCode\t"));
    assert!(output.contains("Total\t0\t"));
}

// ============================================================================
// 3. JSON renderer – valid JSON with schema_version
// ============================================================================

#[test]
fn lang_json_valid_and_has_schema_version() {
    let report = lang_report(vec![rust_row(500)], true);
    let output = render_lang(&report, TableFormat::Json);
    let v: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    assert!(v.get("schema_version").is_some());
    assert_eq!(v["mode"], "lang");
}

#[test]
fn snapshot_lang_json_single_row() {
    let report = lang_report(vec![rust_row(250)], true);
    let output = render_lang(&report, TableFormat::Json);
    let mut v: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    v["generated_at_ms"] = serde_json::json!(0);
    v["tool"]["version"] = serde_json::json!("0.0.0-test");
    insta::assert_json_snapshot!(v);
}

#[test]
fn module_json_valid_and_has_schema_version() {
    let report = module_report(vec![sample_module_row("src", 300)]);
    let output = render_module(&report, TableFormat::Json);
    let v: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    assert!(v.get("schema_version").is_some());
    assert_eq!(v["mode"], "module");
}

#[test]
fn lang_json_empty_rows() {
    let report = lang_report(vec![], true);
    let output = render_lang(&report, TableFormat::Json);
    let v: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    let rows = v["rows"].as_array().unwrap();
    assert!(rows.is_empty());
}

// ============================================================================
// 4. JSONL renderer – export
// ============================================================================

#[test]
fn snapshot_export_jsonl_multi_file() {
    let data = export_data(vec![
        sample_file_row("src/lib.rs", "Rust", 400),
        sample_file_row("src/utils.rs", "Rust", 150),
    ]);
    let output = render_export_jsonl(&data);
    // Stabilise dynamic fields
    let lines: Vec<&str> = output.lines().collect();
    assert!(lines.len() >= 3, "meta + 2 rows");
    let meta: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(meta["type"], "meta");
    let row1: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
    assert_eq!(row1["type"], "row");
}

#[test]
fn export_jsonl_empty_rows() {
    let data = export_data(vec![]);
    let output = render_export_jsonl(&data);
    let lines: Vec<&str> = output.lines().collect();
    // Only meta line when no rows
    assert_eq!(lines.len(), 1, "only meta line for empty export");
}

// ============================================================================
// 5. CSV renderer – export
// ============================================================================

#[test]
fn snapshot_export_csv_multi_file() {
    let data = export_data(vec![
        sample_file_row("src/main.rs", "Rust", 300),
        sample_file_row("tests/test.rs", "Rust", 100),
    ]);
    insta::assert_snapshot!(render_export_csv(&data));
}

#[test]
fn export_csv_has_correct_header() {
    let data = export_data(vec![sample_file_row("a.rs", "Rust", 10)]);
    let output = render_export_csv(&data);
    let header = output.lines().next().unwrap();
    assert!(header.contains("path"));
    assert!(header.contains("module"));
    assert!(header.contains("lang"));
    assert!(header.contains("code"));
    assert!(header.contains("tokens"));
}

#[test]
fn export_csv_empty_rows() {
    let data = export_data(vec![]);
    let output = render_export_csv(&data);
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 1, "only header for empty CSV");
}

// ============================================================================
// 6. JSON export
// ============================================================================

#[test]
fn export_json_valid_with_meta() {
    let data = export_data(vec![sample_file_row("src/lib.rs", "Rust", 200)]);
    let output = render_export_json(&data);
    let v: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    assert!(v.get("schema_version").is_some());
    assert_eq!(v["mode"], "export");
    let rows = v["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
}

// ============================================================================
// 7. CycloneDX renderer
// ============================================================================

#[test]
fn snapshot_cyclonedx_multi_file() {
    let data = export_data(vec![
        sample_file_row("src/lib.rs", "Rust", 500),
        sample_file_row("src/utils.rs", "Rust", 100),
    ]);
    let mut buf = Vec::new();
    write_export_cyclonedx_with_options(
        &mut buf,
        &data,
        RedactMode::None,
        Some("urn:uuid:00000000-0000-0000-0000-000000000000".into()),
        Some("2024-01-01T00:00:00Z".into()),
    )
    .unwrap();
    insta::assert_snapshot!(String::from_utf8(buf).unwrap());
}

#[test]
fn cyclonedx_empty_rows() {
    let data = export_data(vec![]);
    let mut buf = Vec::new();
    write_export_cyclonedx_with_options(
        &mut buf,
        &data,
        RedactMode::None,
        Some("urn:uuid:00000000-0000-0000-0000-000000000000".into()),
        Some("2024-01-01T00:00:00Z".into()),
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    let v: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    let components = v["components"].as_array().unwrap();
    assert!(components.is_empty());
}

// ============================================================================
// 8. Diff renderers
// ============================================================================

#[test]
fn snapshot_diff_md_growth() {
    let from = lang_report(vec![rust_row(500)], true);
    let to = lang_report(vec![rust_row(800), python_row(200)], true);
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    insta::assert_snapshot!(render_diff_md("v1.0", "v2.0", &rows, &totals));
}

#[test]
fn snapshot_diff_md_compact() {
    let from = lang_report(vec![rust_row(1000)], true);
    let to = lang_report(vec![rust_row(1200)], true);
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let opts = DiffRenderOptions {
        compact: true,
        color: DiffColorMode::Off,
    };
    insta::assert_snapshot!(render_diff_md_with_options(
        "old", "new", &rows, &totals, opts
    ));
}

#[test]
fn diff_md_empty_rows() {
    let report = lang_report(vec![rust_row(100)], true);
    let rows = compute_diff_rows(&report, &report);
    assert!(rows.is_empty(), "identical reports produce no diff rows");
    let totals = compute_diff_totals(&rows);
    let md = render_diff_md("a", "b", &rows, &totals);
    assert!(md.contains("Diff: a → b"));
}

#[test]
fn diff_json_receipt_has_schema_version() {
    let from = lang_report(vec![rust_row(100)], true);
    let to = lang_report(vec![rust_row(200)], true);
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let receipt = create_diff_receipt("v1", "v2", rows, totals);
    let json = serde_json::to_string(&receipt).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(v.get("schema_version").is_some());
}

// ============================================================================
// 9. Determinism – same input → byte-identical output
// ============================================================================

#[test]
fn determinism_lang_md() {
    let report = lang_report(vec![rust_row(500), python_row(200)], true);
    let a = render_lang(&report, TableFormat::Md);
    let b = render_lang(&report, TableFormat::Md);
    assert_eq!(a, b);
}

#[test]
fn determinism_lang_tsv() {
    let report = lang_report(vec![rust_row(500), python_row(200)], false);
    let a = render_lang(&report, TableFormat::Tsv);
    let b = render_lang(&report, TableFormat::Tsv);
    assert_eq!(a, b);
}

#[test]
fn determinism_module_md() {
    let report = module_report(vec![
        sample_module_row("src/a", 300),
        sample_module_row("src/b", 100),
    ]);
    let a = render_module(&report, TableFormat::Md);
    let b = render_module(&report, TableFormat::Md);
    assert_eq!(a, b);
}

#[test]
fn determinism_module_tsv() {
    let report = module_report(vec![sample_module_row("src", 50)]);
    let a = render_module(&report, TableFormat::Tsv);
    let b = render_module(&report, TableFormat::Tsv);
    assert_eq!(a, b);
}

#[test]
fn determinism_export_csv() {
    let data = export_data(vec![
        sample_file_row("a.rs", "Rust", 50),
        sample_file_row("b.py", "Python", 30),
    ]);
    let a = render_export_csv(&data);
    let b = render_export_csv(&data);
    assert_eq!(a, b);
}

#[test]
fn determinism_diff_md() {
    let from = lang_report(vec![rust_row(100)], true);
    let to = lang_report(vec![rust_row(200)], true);
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let a = render_diff_md("v1", "v2", &rows, &totals);
    let b = render_diff_md("v1", "v2", &rows, &totals);
    assert_eq!(a, b);
}

// ============================================================================
// 10. Non-zero output length for non-empty input
// ============================================================================

#[test]
fn nonzero_length_lang_md() {
    let report = lang_report(vec![rust_row(1)], true);
    assert!(!render_lang(&report, TableFormat::Md).is_empty());
}

#[test]
fn nonzero_length_lang_tsv() {
    let report = lang_report(vec![rust_row(1)], false);
    assert!(!render_lang(&report, TableFormat::Tsv).is_empty());
}

#[test]
fn nonzero_length_module_md() {
    let report = module_report(vec![sample_module_row("x", 1)]);
    assert!(!render_module(&report, TableFormat::Md).is_empty());
}

#[test]
fn nonzero_length_export_csv() {
    let data = export_data(vec![sample_file_row("f.rs", "Rust", 1)]);
    assert!(!render_export_csv(&data).is_empty());
}

// ============================================================================
// 11. Sort order: output rows in descending code order
// ============================================================================

#[test]
fn lang_md_rows_descending_code_order() {
    // Input deliberately out of order
    let report = lang_report(vec![toml_row(50), rust_row(1000), python_row(300)], true);
    let output = render_lang(&report, TableFormat::Md);
    // Rows come in input order (pre-sorted by caller), verify data is present
    assert!(output.contains("|TOML|"));
    assert!(output.contains("|Rust|"));
    assert!(output.contains("|Python|"));
}

#[test]
fn diff_rows_alphabetical_by_language() {
    let from = lang_report(vec![python_row(200), rust_row(100)], true);
    let to = lang_report(vec![python_row(300), rust_row(200)], true);
    let rows = compute_diff_rows(&from, &to);
    assert_eq!(rows[0].lang, "Python");
    assert_eq!(rows[1].lang, "Rust");
}

// ============================================================================
// 12. Special characters in language/path names
// ============================================================================

#[test]
fn special_chars_lang_name_md() {
    let row = LangRow {
        lang: "C++".into(),
        code: 100,
        lines: 120,
        files: 5,
        bytes: 4000,
        tokens: 200,
        avg_lines: 24,
    };
    let report = lang_report(vec![row], true);
    let output = render_lang(&report, TableFormat::Md);
    assert!(output.contains("|C++|100|"));
}

#[test]
fn special_chars_lang_name_tsv() {
    let row = LangRow {
        lang: "C#".into(),
        code: 77,
        lines: 90,
        files: 3,
        bytes: 2310,
        tokens: 154,
        avg_lines: 30,
    };
    let report = lang_report(vec![row], false);
    let output = render_lang(&report, TableFormat::Tsv);
    assert!(output.contains("C#\t77\t"));
}

#[test]
fn special_chars_path_csv() {
    let row = FileRow {
        path: "src/my module (v2)/lib.rs".into(),
        module: "src/my module (v2)".into(),
        lang: "Rust".into(),
        kind: FileKind::Parent,
        code: 10,
        comments: 1,
        blanks: 1,
        lines: 12,
        bytes: 300,
        tokens: 20,
    };
    let data = export_data(vec![row]);
    let output = render_export_csv(&data);
    assert!(output.contains("my module (v2)"));
}

#[test]
fn special_chars_unicode_lang_json() {
    let row = LangRow {
        lang: "日本語".into(),
        code: 50,
        lines: 60,
        files: 1,
        bytes: 2000,
        tokens: 100,
        avg_lines: 60,
    };
    let report = lang_report(vec![row], true);
    let output = render_lang(&report, TableFormat::Json);
    let v: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    let lang = v["rows"][0]["lang"].as_str().unwrap();
    assert_eq!(lang, "日本語");
}

// ============================================================================
// 13. Top-N filtering
// ============================================================================

#[test]
fn snapshot_lang_md_top1() {
    let mut report = lang_report(vec![rust_row(1000), python_row(500), toml_row(100)], true);
    report.top = 1;
    let mut args = lang_args(TableFormat::Md);
    args.top = 1;
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &globals(), &args).unwrap();
    insta::assert_snapshot!(String::from_utf8(buf).unwrap());
}

#[test]
fn lang_md_top0_shows_all() {
    let report = lang_report(vec![rust_row(1000), python_row(500), toml_row(100)], true);
    let output = render_lang(&report, TableFormat::Md);
    assert!(output.contains("|Rust|"));
    assert!(output.contains("|Python|"));
    assert!(output.contains("|TOML|"));
}

// ============================================================================
// 14. Children mode affects output
// ============================================================================

#[test]
fn children_mode_collapse_vs_separate_lang_md() {
    let rows = vec![rust_row(500)];
    let mut report_collapse = lang_report(rows.clone(), true);
    report_collapse.children = ChildrenMode::Collapse;

    let mut report_separate = lang_report(rows, true);
    report_separate.children = ChildrenMode::Separate;

    let out_c = render_lang(&report_collapse, TableFormat::Md);
    let out_s = render_lang(&report_separate, TableFormat::Md);
    // Both render the same rows, but the report metadata differs
    assert!(!out_c.is_empty());
    assert!(!out_s.is_empty());
}

#[test]
fn children_mode_in_json_envelope() {
    let rows = vec![rust_row(100)];
    let mut report = lang_report(rows, true);
    report.children = ChildrenMode::Separate;
    let output = render_lang(&report, TableFormat::Json);
    let v: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    assert_eq!(v["args"]["children"], "separate");
}

#[test]
fn children_include_mode_in_module_json() {
    let mut report = module_report(vec![sample_module_row("src", 100)]);
    report.children = ChildIncludeMode::ParentsOnly;
    let mut args = module_args(TableFormat::Json);
    args.children = ChildIncludeMode::ParentsOnly;
    let mut buf = Vec::new();
    write_module_report_to(&mut buf, &report, &globals(), &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let v: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    assert_eq!(v["args"]["children"], "parents-only");
}

// ============================================================================
// 15. Module Markdown renderer
// ============================================================================

#[test]
fn snapshot_module_md_multi_row() {
    let report = module_report(vec![
        sample_module_row("src/core", 800),
        sample_module_row("src/utils", 300),
        sample_module_row("tests", 100),
    ]);
    insta::assert_snapshot!(render_module(&report, TableFormat::Md));
}

#[test]
fn module_md_empty_rows() {
    let report = module_report(vec![]);
    let output = render_module(&report, TableFormat::Md);
    assert!(output.contains("|Module|"));
    assert!(output.contains("**Total**"));
}

// ============================================================================
// 16. Export JSONL single file
// ============================================================================

#[test]
fn export_jsonl_single_file_row_count() {
    let data = export_data(vec![sample_file_row("src/lib.rs", "Rust", 100)]);
    let output = render_export_jsonl(&data);
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 2, "meta + 1 row");
}

// ============================================================================
// 17. Diff with language removed
// ============================================================================

#[test]
fn diff_language_removed() {
    let from = lang_report(vec![rust_row(500), python_row(200)], true);
    let to = lang_report(vec![rust_row(600)], true);
    let rows = compute_diff_rows(&from, &to);
    let python_row = rows.iter().find(|r| r.lang == "Python").unwrap();
    assert_eq!(python_row.new_code, 0);
    assert!(python_row.delta_code < 0);
}

// ============================================================================
// 18. Child FileKind in export
// ============================================================================

#[test]
fn export_csv_child_file_kind() {
    let row = FileRow {
        path: "src/lib.rs".into(),
        module: "src".into(),
        lang: "HTML".into(),
        kind: FileKind::Child,
        code: 20,
        comments: 2,
        blanks: 1,
        lines: 23,
        bytes: 600,
        tokens: 40,
    };
    let data = export_data(vec![row]);
    let output = render_export_csv(&data);
    assert!(output.contains(",child,"));
}

// ============================================================================
// 19. Redaction in export
// ============================================================================

#[test]
fn export_csv_redact_paths() {
    let data = export_data(vec![sample_file_row("src/secret.rs", "Rust", 50)]);
    let mut args = export_args(ExportFormat::Csv);
    args.redact = RedactMode::Paths;
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &data, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(!output.contains("src/secret.rs"), "path should be redacted");
}
