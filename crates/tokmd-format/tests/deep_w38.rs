//! Deep tests for tokmd-format (wave 38).
//!
//! Covers Markdown rendering, TSV output, JSON/JSONL/CSV export,
//! empty data, special characters, deterministic ordering, diff
//! computation, and envelope metadata.

use std::path::PathBuf;

use tokmd_format::{
    DiffColorMode, DiffRenderOptions, compute_diff_rows, compute_diff_totals, create_diff_receipt,
    render_diff_md, render_diff_md_with_options, write_export_csv_to, write_export_json_to,
    write_export_jsonl_to, write_lang_report_to, write_module_report_to,
};
use tokmd_settings::ScanOptions;
use tokmd_types::{
    ChildIncludeMode, ChildrenMode, DiffRow, ExportArgs, ExportData, ExportFormat, FileKind,
    FileRow, LangArgs, LangReport, LangRow, ModuleArgs, ModuleReport, ModuleRow, RedactMode,
    TableFormat, Totals,
};

// ============================================================================
// Helpers
// ============================================================================

fn sample_lang_report(with_files: bool) -> LangReport {
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

fn sample_module_report() -> ModuleReport {
    ModuleReport {
        rows: vec![
            ModuleRow {
                module: "crates/foo".into(),
                code: 800,
                lines: 950,
                files: 8,
                bytes: 40000,
                tokens: 2000,
                avg_lines: 119,
            },
            ModuleRow {
                module: "crates/bar".into(),
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

fn sample_file_rows() -> Vec<FileRow> {
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
            path: "tests/test.rs".into(),
            module: "tests".into(),
            lang: "Rust".into(),
            kind: FileKind::Parent,
            code: 50,
            comments: 5,
            blanks: 5,
            lines: 60,
            bytes: 500,
            tokens: 125,
        },
    ]
}

fn sample_export_data() -> ExportData {
    ExportData {
        rows: sample_file_rows(),
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    }
}

fn default_scan_options() -> ScanOptions {
    ScanOptions::default()
}

fn default_lang_args(format: TableFormat) -> LangArgs {
    LangArgs {
        paths: vec![PathBuf::from(".")],
        format,
        top: 0,
        files: false,
        children: ChildrenMode::Collapse,
    }
}

fn default_module_args(format: TableFormat) -> ModuleArgs {
    ModuleArgs {
        paths: vec![PathBuf::from(".")],
        format,
        top: 0,
        module_roots: vec!["crates".into()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

fn default_export_args(format: ExportFormat) -> ExportArgs {
    ExportArgs {
        paths: vec![PathBuf::from(".")],
        format,
        output: None,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
        min_code: 0,
        max_rows: 0,
        redact: RedactMode::None,
        meta: true,
        strip_prefix: None,
    }
}

// ============================================================================
// 1. Markdown rendering — lang summary
// ============================================================================

#[test]
fn lang_md_without_files_has_correct_header() {
    let mut buf = Vec::new();
    let report = sample_lang_report(false);
    let global = default_scan_options();
    let args = default_lang_args(TableFormat::Md);
    write_lang_report_to(&mut buf, &report, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("|Lang|Code|Lines|Bytes|Tokens|"));
    assert!(!output.contains("|Files|"));
}

#[test]
fn lang_md_with_files_has_files_and_avg() {
    let mut buf = Vec::new();
    let report = sample_lang_report(true);
    let global = default_scan_options();
    let mut args = default_lang_args(TableFormat::Md);
    args.files = true;
    write_lang_report_to(&mut buf, &report, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("|Lang|Code|Lines|Files|Bytes|Tokens|Avg|"));
}

#[test]
fn lang_md_contains_row_data() {
    let mut buf = Vec::new();
    let report = sample_lang_report(false);
    let global = default_scan_options();
    let args = default_lang_args(TableFormat::Md);
    write_lang_report_to(&mut buf, &report, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("|Rust|1000|1200|50000|2500|"));
    assert!(output.contains("|TOML|50|60|1000|125|"));
}

#[test]
fn lang_md_contains_total_row() {
    let mut buf = Vec::new();
    let report = sample_lang_report(false);
    let global = default_scan_options();
    let args = default_lang_args(TableFormat::Md);
    write_lang_report_to(&mut buf, &report, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("|**Total**|1050|1260|51000|2625|"));
}

#[test]
fn lang_md_separator_row_present() {
    let mut buf = Vec::new();
    let report = sample_lang_report(false);
    let global = default_scan_options();
    let args = default_lang_args(TableFormat::Md);
    write_lang_report_to(&mut buf, &report, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("|---|---:|---:|---:|---:|"));
}

// ============================================================================
// 2. TSV output
// ============================================================================

#[test]
fn lang_tsv_tab_separated() {
    let mut buf = Vec::new();
    let report = sample_lang_report(false);
    let global = default_scan_options();
    let args = default_lang_args(TableFormat::Tsv);
    write_lang_report_to(&mut buf, &report, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("Lang\tCode\tLines\tBytes\tTokens"));
}

#[test]
fn lang_tsv_contains_data_rows() {
    let mut buf = Vec::new();
    let report = sample_lang_report(false);
    let global = default_scan_options();
    let args = default_lang_args(TableFormat::Tsv);
    write_lang_report_to(&mut buf, &report, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("Rust\t1000\t1200\t50000\t2500"));
    assert!(output.contains("TOML\t50\t60\t1000\t125"));
}

#[test]
fn lang_tsv_total_row() {
    let mut buf = Vec::new();
    let report = sample_lang_report(false);
    let global = default_scan_options();
    let args = default_lang_args(TableFormat::Tsv);
    write_lang_report_to(&mut buf, &report, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("Total\t1050\t1260\t51000\t2625"));
}

#[test]
fn lang_tsv_with_files_includes_files_column() {
    let mut buf = Vec::new();
    let report = sample_lang_report(true);
    let global = default_scan_options();
    let args = default_lang_args(TableFormat::Tsv);
    write_lang_report_to(&mut buf, &report, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("Lang\tCode\tLines\tFiles\tBytes\tTokens\tAvg"));
}

// ============================================================================
// 3. JSON output with envelope metadata
// ============================================================================

#[test]
fn lang_json_has_schema_version() {
    let mut buf = Vec::new();
    let report = sample_lang_report(false);
    let global = default_scan_options();
    let args = default_lang_args(TableFormat::Json);
    write_lang_report_to(&mut buf, &report, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let json: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    assert_eq!(json["schema_version"], 2);
}

#[test]
fn lang_json_has_tool_info() {
    let mut buf = Vec::new();
    let report = sample_lang_report(false);
    let global = default_scan_options();
    let args = default_lang_args(TableFormat::Json);
    write_lang_report_to(&mut buf, &report, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let json: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    assert_eq!(json["tool"]["name"], "tokmd");
    assert!(json["tool"]["version"].is_string());
}

#[test]
fn lang_json_has_mode_lang() {
    let mut buf = Vec::new();
    let report = sample_lang_report(false);
    let global = default_scan_options();
    let args = default_lang_args(TableFormat::Json);
    write_lang_report_to(&mut buf, &report, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let json: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    assert_eq!(json["mode"], "lang");
}

#[test]
fn lang_json_has_rows_data() {
    let mut buf = Vec::new();
    let report = sample_lang_report(false);
    let global = default_scan_options();
    let args = default_lang_args(TableFormat::Json);
    write_lang_report_to(&mut buf, &report, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let json: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    let rows = json["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["lang"], "Rust");
    assert_eq!(rows[0]["code"], 1000);
}

#[test]
fn module_json_has_mode_module() {
    let mut buf = Vec::new();
    let report = sample_module_report();
    let global = default_scan_options();
    let args = default_module_args(TableFormat::Json);
    write_module_report_to(&mut buf, &report, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let json: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    assert_eq!(json["mode"], "module");
}

// ============================================================================
// 4. Module Markdown rendering
// ============================================================================

#[test]
fn module_md_has_correct_header() {
    let mut buf = Vec::new();
    let report = sample_module_report();
    let global = default_scan_options();
    let args = default_module_args(TableFormat::Md);
    write_module_report_to(&mut buf, &report, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("|Module|Code|Lines|Files|Bytes|Tokens|Avg|"));
}

#[test]
fn module_md_contains_row_data() {
    let mut buf = Vec::new();
    let report = sample_module_report();
    let global = default_scan_options();
    let args = default_module_args(TableFormat::Md);
    write_module_report_to(&mut buf, &report, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("|crates/foo|800|950|8|40000|2000|119|"));
    assert!(output.contains("|crates/bar|200|250|2|10000|500|125|"));
}

#[test]
fn module_tsv_header() {
    let mut buf = Vec::new();
    let report = sample_module_report();
    let global = default_scan_options();
    let args = default_module_args(TableFormat::Tsv);
    write_module_report_to(&mut buf, &report, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("Module\tCode\tLines\tFiles\tBytes\tTokens\tAvg"));
}

// ============================================================================
// 5. CSV export format
// ============================================================================

#[test]
fn csv_export_has_header() {
    let mut buf = Vec::new();
    let export = sample_export_data();
    let args = default_export_args(ExportFormat::Csv);
    write_export_csv_to(&mut buf, &export, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.starts_with("path,module,lang,kind,code,comments,blanks,lines,bytes,tokens"));
}

#[test]
fn csv_export_row_count() {
    let mut buf = Vec::new();
    let export = sample_export_data();
    let args = default_export_args(ExportFormat::Csv);
    write_export_csv_to(&mut buf, &export, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let lines: Vec<&str> = output.lines().collect();
    // header + 2 data rows
    assert_eq!(lines.len(), 3);
}

#[test]
fn csv_export_contains_paths() {
    let mut buf = Vec::new();
    let export = sample_export_data();
    let args = default_export_args(ExportFormat::Csv);
    write_export_csv_to(&mut buf, &export, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("src/lib.rs"));
    assert!(output.contains("tests/test.rs"));
}

// ============================================================================
// 6. JSONL line-by-line format
// ============================================================================

#[test]
fn jsonl_export_each_line_is_valid_json() {
    let mut buf = Vec::new();
    let export = sample_export_data();
    let global = default_scan_options();
    let args = default_export_args(ExportFormat::Jsonl);
    write_export_jsonl_to(&mut buf, &export, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    for line in output.lines() {
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(line);
        assert!(parsed.is_ok(), "Invalid JSON line: {}", line);
    }
}

#[test]
fn jsonl_export_has_meta_line() {
    let mut buf = Vec::new();
    let export = sample_export_data();
    let global = default_scan_options();
    let args = default_export_args(ExportFormat::Jsonl);
    write_export_jsonl_to(&mut buf, &export, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let first_line = output.lines().next().unwrap();
    let json: serde_json::Value = serde_json::from_str(first_line).unwrap();
    assert_eq!(json["type"], "meta");
}

#[test]
fn jsonl_export_row_lines_have_type_row() {
    let mut buf = Vec::new();
    let export = sample_export_data();
    let global = default_scan_options();
    let args = default_export_args(ExportFormat::Jsonl);
    write_export_jsonl_to(&mut buf, &export, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    for line in output.lines().skip(1) {
        let json: serde_json::Value = serde_json::from_str(line).unwrap();
        assert_eq!(json["type"], "row");
    }
}

#[test]
fn jsonl_export_line_count() {
    let mut buf = Vec::new();
    let export = sample_export_data();
    let global = default_scan_options();
    let args = default_export_args(ExportFormat::Jsonl);
    write_export_jsonl_to(&mut buf, &export, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    // 1 meta + 2 data rows
    assert_eq!(output.lines().count(), 3);
}

// ============================================================================
// 7. JSON export (single object)
// ============================================================================

#[test]
fn json_export_is_valid_json() {
    let mut buf = Vec::new();
    let export = sample_export_data();
    let global = default_scan_options();
    let args = default_export_args(ExportFormat::Json);
    write_export_json_to(&mut buf, &export, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let json: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    assert!(json.is_object());
}

#[test]
fn json_export_has_schema_version() {
    let mut buf = Vec::new();
    let export = sample_export_data();
    let global = default_scan_options();
    let args = default_export_args(ExportFormat::Json);
    write_export_json_to(&mut buf, &export, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let json: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    assert_eq!(json["schema_version"], 2);
}

#[test]
fn json_export_has_rows_array() {
    let mut buf = Vec::new();
    let export = sample_export_data();
    let global = default_scan_options();
    let args = default_export_args(ExportFormat::Json);
    write_export_json_to(&mut buf, &export, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let json: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    assert!(json["rows"].is_array());
    assert_eq!(json["rows"].as_array().unwrap().len(), 2);
}

// ============================================================================
// 8. Empty data rendering
// ============================================================================

#[test]
fn empty_lang_report_md_still_has_header() {
    let mut buf = Vec::new();
    let report = LangReport {
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
    };
    let global = default_scan_options();
    let args = default_lang_args(TableFormat::Md);
    write_lang_report_to(&mut buf, &report, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("|Lang|Code|Lines|"));
    assert!(output.contains("|**Total**|0|0|0|0|"));
}

#[test]
fn empty_module_report_md_has_header() {
    let mut buf = Vec::new();
    let report = ModuleReport {
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
        children: ChildIncludeMode::ParentsOnly,
        top: 0,
    };
    let global = default_scan_options();
    let args = default_module_args(TableFormat::Md);
    write_module_report_to(&mut buf, &report, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("|Module|Code|Lines|"));
}

#[test]
fn empty_export_csv_has_only_header() {
    let mut buf = Vec::new();
    let export = ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    };
    let args = default_export_args(ExportFormat::Csv);
    write_export_csv_to(&mut buf, &export, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert_eq!(output.lines().count(), 1); // header only
}

#[test]
fn empty_export_jsonl_meta_only() {
    let mut buf = Vec::new();
    let export = ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    };
    let global = default_scan_options();
    let args = default_export_args(ExportFormat::Jsonl);
    write_export_jsonl_to(&mut buf, &export, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert_eq!(output.lines().count(), 1); // meta only
}

// ============================================================================
// 9. Special characters in language names
// ============================================================================

#[test]
fn lang_md_special_chars_in_name() {
    let report = LangReport {
        rows: vec![LangRow {
            lang: "C++".into(),
            code: 500,
            lines: 600,
            files: 5,
            bytes: 20000,
            tokens: 1000,
            avg_lines: 120,
        }],
        total: Totals {
            code: 500,
            lines: 600,
            files: 5,
            bytes: 20000,
            tokens: 1000,
            avg_lines: 120,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let mut buf = Vec::new();
    let global = default_scan_options();
    let args = default_lang_args(TableFormat::Md);
    write_lang_report_to(&mut buf, &report, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("|C++|500|600|20000|1000|"));
}

#[test]
fn lang_md_embedded_suffix() {
    let report = LangReport {
        rows: vec![LangRow {
            lang: "JavaScript (embedded)".into(),
            code: 100,
            lines: 120,
            files: 3,
            bytes: 5000,
            tokens: 250,
            avg_lines: 40,
        }],
        total: Totals {
            code: 100,
            lines: 120,
            files: 3,
            bytes: 5000,
            tokens: 250,
            avg_lines: 40,
        },
        with_files: false,
        children: ChildrenMode::Separate,
        top: 0,
    };
    let mut buf = Vec::new();
    let global = default_scan_options();
    let args = default_lang_args(TableFormat::Md);
    write_lang_report_to(&mut buf, &report, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("|JavaScript (embedded)|100|"));
}

// ============================================================================
// 10. Deterministic ordering of output
// ============================================================================

#[test]
fn lang_md_output_is_deterministic() {
    let report = sample_lang_report(false);
    let global = default_scan_options();
    let args = default_lang_args(TableFormat::Md);
    let mut buf1 = Vec::new();
    let mut buf2 = Vec::new();
    write_lang_report_to(&mut buf1, &report, &global, &args).unwrap();
    write_lang_report_to(&mut buf2, &report, &global, &args).unwrap();
    assert_eq!(buf1, buf2);
}

#[test]
fn csv_output_is_deterministic() {
    let export = sample_export_data();
    let args = default_export_args(ExportFormat::Csv);
    let mut buf1 = Vec::new();
    let mut buf2 = Vec::new();
    write_export_csv_to(&mut buf1, &export, &args).unwrap();
    write_export_csv_to(&mut buf2, &export, &args).unwrap();
    assert_eq!(buf1, buf2);
}

// ============================================================================
// 11. Diff computation
// ============================================================================

fn make_lang_report(rows: Vec<LangRow>) -> LangReport {
    let code: usize = rows.iter().map(|r| r.code).sum();
    let lines: usize = rows.iter().map(|r| r.lines).sum();
    let files: usize = rows.iter().map(|r| r.files).sum();
    let bytes: usize = rows.iter().map(|r| r.bytes).sum();
    let tokens: usize = rows.iter().map(|r| r.tokens).sum();
    LangReport {
        rows,
        total: Totals {
            code,
            lines,
            files,
            bytes,
            tokens,
            avg_lines: lines.checked_div(files).unwrap_or(0),
        },
        with_files: true,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

fn rust_row(code: usize) -> LangRow {
    LangRow {
        lang: "Rust".into(),
        code,
        lines: code + 50,
        files: 5,
        bytes: code * 10,
        tokens: code * 2,
        avg_lines: (code + 50) / 5,
    }
}

#[test]
fn diff_rows_same_report_is_empty() {
    let report = make_lang_report(vec![rust_row(100)]);
    let rows = compute_diff_rows(&report, &report);
    assert!(rows.is_empty(), "Same reports should produce no diff rows");
}

#[test]
fn diff_rows_code_increase() {
    let from = make_lang_report(vec![rust_row(100)]);
    let to = make_lang_report(vec![rust_row(200)]);
    let rows = compute_diff_rows(&from, &to);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].delta_code, 100);
    assert_eq!(rows[0].old_code, 100);
    assert_eq!(rows[0].new_code, 200);
}

#[test]
fn diff_rows_code_decrease() {
    let from = make_lang_report(vec![rust_row(200)]);
    let to = make_lang_report(vec![rust_row(100)]);
    let rows = compute_diff_rows(&from, &to);
    assert_eq!(rows[0].delta_code, -100);
}

#[test]
fn diff_rows_new_language_added() {
    let from = make_lang_report(vec![rust_row(100)]);
    let to = make_lang_report(vec![
        rust_row(100),
        LangRow {
            lang: "Python".into(),
            code: 50,
            lines: 60,
            files: 2,
            bytes: 500,
            tokens: 100,
            avg_lines: 30,
        },
    ]);
    let rows = compute_diff_rows(&from, &to);
    let python_row = rows.iter().find(|r| r.lang == "Python").unwrap();
    assert_eq!(python_row.old_code, 0);
    assert_eq!(python_row.new_code, 50);
    assert_eq!(python_row.delta_code, 50);
}

#[test]
fn diff_rows_language_removed() {
    let from = make_lang_report(vec![
        rust_row(100),
        LangRow {
            lang: "Python".into(),
            code: 50,
            lines: 60,
            files: 2,
            bytes: 500,
            tokens: 100,
            avg_lines: 30,
        },
    ]);
    let to = make_lang_report(vec![rust_row(100)]);
    let rows = compute_diff_rows(&from, &to);
    let python_row = rows.iter().find(|r| r.lang == "Python").unwrap();
    assert_eq!(python_row.delta_code, -50);
}

// ============================================================================
// 12. Diff totals
// ============================================================================

#[test]
fn diff_totals_empty_rows() {
    let totals = compute_diff_totals(&[]);
    assert_eq!(totals.delta_code, 0);
    assert_eq!(totals.delta_lines, 0);
    assert_eq!(totals.delta_files, 0);
}

#[test]
fn diff_totals_single_row() {
    let rows = vec![DiffRow {
        lang: "Rust".into(),
        old_code: 100,
        new_code: 200,
        delta_code: 100,
        old_lines: 150,
        new_lines: 300,
        delta_lines: 150,
        old_files: 5,
        new_files: 8,
        delta_files: 3,
        old_bytes: 4000,
        new_bytes: 8000,
        delta_bytes: 4000,
        old_tokens: 1000,
        new_tokens: 2000,
        delta_tokens: 1000,
    }];
    let totals = compute_diff_totals(&rows);
    assert_eq!(totals.delta_code, 100);
    assert_eq!(totals.delta_tokens, 1000);
}

#[test]
fn diff_totals_multiple_rows_sum() {
    let rows = vec![
        DiffRow {
            lang: "Rust".into(),
            old_code: 100,
            new_code: 200,
            delta_code: 100,
            old_lines: 0,
            new_lines: 0,
            delta_lines: 0,
            old_files: 0,
            new_files: 0,
            delta_files: 0,
            old_bytes: 0,
            new_bytes: 0,
            delta_bytes: 0,
            old_tokens: 0,
            new_tokens: 0,
            delta_tokens: 0,
        },
        DiffRow {
            lang: "Python".into(),
            old_code: 50,
            new_code: 30,
            delta_code: -20,
            old_lines: 0,
            new_lines: 0,
            delta_lines: 0,
            old_files: 0,
            new_files: 0,
            delta_files: 0,
            old_bytes: 0,
            new_bytes: 0,
            delta_bytes: 0,
            old_tokens: 0,
            new_tokens: 0,
            delta_tokens: 0,
        },
    ];
    let totals = compute_diff_totals(&rows);
    assert_eq!(totals.delta_code, 80); // 100 + (-20) = 80
}

// ============================================================================
// 13. Diff Markdown rendering
// ============================================================================

#[test]
fn diff_md_contains_header() {
    let from = make_lang_report(vec![rust_row(100)]);
    let to = make_lang_report(vec![rust_row(200)]);
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let output = render_diff_md("v1", "v2", &rows, &totals);
    assert!(output.contains("## Diff: v1 → v2"));
}

#[test]
fn diff_md_compact_mode() {
    let from = make_lang_report(vec![rust_row(100)]);
    let to = make_lang_report(vec![rust_row(200)]);
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let output = render_diff_md_with_options(
        "v1",
        "v2",
        &rows,
        &totals,
        DiffRenderOptions {
            compact: true,
            color: DiffColorMode::Off,
        },
    );
    assert!(output.contains("|From LOC|100|"));
    assert!(output.contains("|To LOC|200|"));
}

// ============================================================================
// 14. create_diff_receipt
// ============================================================================

#[test]
fn diff_receipt_has_correct_mode() {
    let from = make_lang_report(vec![rust_row(100)]);
    let to = make_lang_report(vec![rust_row(200)]);
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let receipt = create_diff_receipt("v1", "v2", rows, totals);
    assert_eq!(receipt.mode, "diff");
    assert_eq!(receipt.from_source, "v1");
    assert_eq!(receipt.to_source, "v2");
    assert_eq!(receipt.schema_version, 2);
}

// ============================================================================
// 15. JSONL without meta flag
// ============================================================================

#[test]
fn jsonl_no_meta_has_only_rows() {
    let mut buf = Vec::new();
    let export = sample_export_data();
    let global = default_scan_options();
    let mut args = default_export_args(ExportFormat::Jsonl);
    args.meta = false;
    write_export_jsonl_to(&mut buf, &export, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    // No meta line, only row lines
    assert_eq!(output.lines().count(), 2);
    for line in output.lines() {
        let json: serde_json::Value = serde_json::from_str(line).unwrap();
        assert_eq!(json["type"], "row");
    }
}

// ============================================================================
// 16. JSON without meta
// ============================================================================

#[test]
fn json_no_meta_is_array() {
    let mut buf = Vec::new();
    let export = sample_export_data();
    let global = default_scan_options();
    let mut args = default_export_args(ExportFormat::Json);
    args.meta = false;
    write_export_json_to(&mut buf, &export, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let json: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    assert!(json.is_array());
    assert_eq!(json.as_array().unwrap().len(), 2);
}

// ============================================================================
// 17. Module TSV rendering
// ============================================================================

#[test]
fn module_tsv_contains_data() {
    let mut buf = Vec::new();
    let report = sample_module_report();
    let global = default_scan_options();
    let args = default_module_args(TableFormat::Tsv);
    write_module_report_to(&mut buf, &report, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("crates/foo\t800\t950"));
    assert!(output.contains("crates/bar\t200\t250"));
}

// ============================================================================
// 18. CSV values are correct
// ============================================================================

#[test]
fn csv_values_match_input() {
    let mut buf = Vec::new();
    let export = sample_export_data();
    let args = default_export_args(ExportFormat::Csv);
    write_export_csv_to(&mut buf, &export, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    // Check first data row
    assert!(output.contains("src/lib.rs,src,Rust,parent,100,20,10,130,1000,250"));
}

// ============================================================================
// 19. FileKind in CSV
// ============================================================================

#[test]
fn csv_child_kind_rendered() {
    let mut buf = Vec::new();
    let export = ExportData {
        rows: vec![FileRow {
            path: "index.html".into(),
            module: "(root)".into(),
            lang: "JavaScript".into(),
            kind: FileKind::Child,
            code: 10,
            comments: 1,
            blanks: 1,
            lines: 12,
            bytes: 100,
            tokens: 25,
        }],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let args = default_export_args(ExportFormat::Csv);
    write_export_csv_to(&mut buf, &export, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains(",child,"));
}

// ============================================================================
// 20. Diff color mode off produces no ANSI codes
// ============================================================================

#[test]
fn diff_md_no_color_no_ansi() {
    let from = make_lang_report(vec![rust_row(100)]);
    let to = make_lang_report(vec![rust_row(200)]);
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let output = render_diff_md_with_options(
        "a",
        "b",
        &rows,
        &totals,
        DiffRenderOptions {
            compact: false,
            color: DiffColorMode::Off,
        },
    );
    assert!(
        !output.contains("\x1b["),
        "Should not contain ANSI escape codes"
    );
}

#[test]
fn diff_md_with_ansi_has_escape_codes() {
    let from = make_lang_report(vec![rust_row(100)]);
    let to = make_lang_report(vec![rust_row(200)]);
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let output = render_diff_md_with_options(
        "a",
        "b",
        &rows,
        &totals,
        DiffRenderOptions {
            compact: false,
            color: DiffColorMode::Ansi,
        },
    );
    assert!(
        output.contains("\x1b["),
        "ANSI mode should have escape codes"
    );
}
