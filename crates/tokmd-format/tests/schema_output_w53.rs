//! Schema output compliance tests for the format crate.
//!
//! These tests verify that rendered outputs (JSON, Markdown, TSV, CSV) conform
//! to expected structural properties.

use std::path::PathBuf;

use tokmd_format::write_lang_report_to;
use tokmd_format::write_module_report_to;
use tokmd_settings::ScanOptions;
use tokmd_types::{
    ChildIncludeMode, ChildrenMode, LangArgs, LangReport, LangRow, ModuleArgs, ModuleReport,
    ModuleRow, SCHEMA_VERSION, TableFormat, Totals,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn sample_totals() -> Totals {
    Totals {
        code: 100,
        lines: 150,
        files: 5,
        bytes: 5000,
        tokens: 1000,
        avg_lines: 30,
    }
}

fn sample_lang_report() -> LangReport {
    LangReport {
        rows: vec![
            LangRow {
                lang: "Rust".into(),
                code: 80,
                lines: 120,
                files: 3,
                bytes: 4000,
                tokens: 800,
                avg_lines: 40,
            },
            LangRow {
                lang: "TOML".into(),
                code: 20,
                lines: 30,
                files: 2,
                bytes: 1000,
                tokens: 200,
                avg_lines: 15,
            },
        ],
        total: sample_totals(),
        with_files: true,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

fn sample_module_report() -> ModuleReport {
    ModuleReport {
        rows: vec![ModuleRow {
            module: "src".into(),
            code: 100,
            lines: 150,
            files: 5,
            bytes: 5000,
            tokens: 1000,
            avg_lines: 30,
        }],
        total: sample_totals(),
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        top: 0,
    }
}

fn default_scan_options() -> ScanOptions {
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
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

// ---------------------------------------------------------------------------
// 1. JSON output is valid JSON
// ---------------------------------------------------------------------------

#[test]
fn lang_json_output_is_valid_json() {
    let report = sample_lang_report();
    let global = default_scan_options();
    let args = lang_args(TableFormat::Json);

    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &global, &args).expect("operation must succeed");

    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    let val: serde_json::Value =
        serde_json::from_str(output.trim()).expect("must parse valid JSON");
    assert!(val.is_object());
    assert_eq!(val["schema_version"], SCHEMA_VERSION);
}

#[test]
fn module_json_output_is_valid_json() {
    let report = sample_module_report();
    let global = default_scan_options();
    let args = module_args(TableFormat::Json);

    let mut buf = Vec::new();
    write_module_report_to(&mut buf, &report, &global, &args).expect("operation must succeed");

    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    let val: serde_json::Value =
        serde_json::from_str(output.trim()).expect("must parse valid JSON");
    assert!(val.is_object());
    assert_eq!(val["schema_version"], SCHEMA_VERSION);
}

#[test]
fn lang_json_has_rows_and_total() {
    let report = sample_lang_report();
    let global = default_scan_options();
    let args = lang_args(TableFormat::Json);

    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &global, &args).expect("operation must succeed");

    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    let val: serde_json::Value =
        serde_json::from_str(output.trim()).expect("must parse valid JSON");
    assert!(val["rows"].is_array());
    assert_eq!(
        val["rows"].as_array().expect("must be a JSON array").len(),
        2
    );
    assert!(val["total"].is_object());
}

// ---------------------------------------------------------------------------
// 2. Markdown output has expected headers
// ---------------------------------------------------------------------------

#[test]
fn lang_md_output_has_headers() {
    let report = sample_lang_report();
    let global = default_scan_options();
    let args = lang_args(TableFormat::Md);

    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &global, &args).expect("operation must succeed");

    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    assert!(output.contains("|Lang|"));
    assert!(output.contains("|Code|"));
    assert!(output.contains("|Lines|"));
    assert!(output.contains("|**Total**|"));
}

#[test]
fn module_md_output_has_headers() {
    let report = sample_module_report();
    let global = default_scan_options();
    let args = module_args(TableFormat::Md);

    let mut buf = Vec::new();
    write_module_report_to(&mut buf, &report, &global, &args).expect("operation must succeed");

    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    assert!(output.contains("|Module|"));
    assert!(output.contains("|Code|"));
    assert!(output.contains("|**Total**|"));
}

// ---------------------------------------------------------------------------
// 3. TSV output has correct column count
// ---------------------------------------------------------------------------

#[test]
fn lang_tsv_output_has_correct_columns() {
    let report = sample_lang_report();
    let global = default_scan_options();
    let args = lang_args(TableFormat::Tsv);

    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &global, &args).expect("operation must succeed");

    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    let lines: Vec<&str> = output.lines().collect();

    // Header line: Lang\tCode\tLines\tFiles\tBytes\tTokens\tAvg (7 columns)
    let header_cols = lines[0].split('\t').count();
    assert_eq!(header_cols, 7);

    // Each data row should have the same column count
    for line in &lines[1..] {
        let cols = line.split('\t').count();
        assert_eq!(cols, header_cols, "Row has wrong column count: {}", line);
    }
}

#[test]
fn module_tsv_output_has_correct_columns() {
    let report = sample_module_report();
    let global = default_scan_options();
    let args = module_args(TableFormat::Tsv);

    let mut buf = Vec::new();
    write_module_report_to(&mut buf, &report, &global, &args).expect("operation must succeed");

    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    let lines: Vec<&str> = output.lines().collect();

    // Header: Module\tCode\tLines\tFiles\tBytes\tTokens\tAvg (7 columns)
    let header_cols = lines[0].split('\t').count();
    assert_eq!(header_cols, 7);

    for line in &lines[1..] {
        assert_eq!(line.split('\t').count(), header_cols);
    }
}

// ---------------------------------------------------------------------------
// 4. Cross-format data consistency
// ---------------------------------------------------------------------------

#[test]
fn lang_all_formats_contain_same_totals() {
    let report = sample_lang_report();
    let global = default_scan_options();

    // JSON
    let mut json_buf = Vec::new();
    write_lang_report_to(
        &mut json_buf,
        &report,
        &global,
        &lang_args(TableFormat::Json),
    )
    .expect("operation must succeed");
    let json_str = String::from_utf8(json_buf).expect("operation must succeed");
    let json_val: serde_json::Value =
        serde_json::from_str(json_str.trim()).expect("must parse valid JSON");
    let json_total_code = json_val["total"]["code"]
        .as_u64()
        .expect("must be a JSON integer");

    // TSV
    let mut tsv_buf = Vec::new();
    write_lang_report_to(&mut tsv_buf, &report, &global, &lang_args(TableFormat::Tsv))
        .expect("operation must succeed");
    let tsv_str = String::from_utf8(tsv_buf).expect("operation must succeed");
    let tsv_last_line = tsv_str
        .lines()
        .last()
        .expect("output must have at least one line");
    let tsv_total_code: u64 = tsv_last_line
        .split('\t')
        .nth(1)
        .expect("operation must succeed")
        .parse()
        .expect("must parse value");

    // MD
    let mut md_buf = Vec::new();
    write_lang_report_to(&mut md_buf, &report, &global, &lang_args(TableFormat::Md))
        .expect("operation must succeed");
    let md_str = String::from_utf8(md_buf).expect("operation must succeed");
    let md_total_line = md_str
        .lines()
        .find(|l| l.contains("**Total**"))
        .expect("operation must succeed");
    let md_total_code: u64 = md_total_line
        .split('|')
        .nth(2)
        .expect("operation must succeed")
        .parse()
        .expect("must parse value");

    // All three formats report the same total code
    assert_eq!(json_total_code, report.total.code as u64);
    assert_eq!(tsv_total_code, report.total.code as u64);
    assert_eq!(md_total_code, report.total.code as u64);
}

// ---------------------------------------------------------------------------
// 5. JSON output has schema_version and tool
// ---------------------------------------------------------------------------

#[test]
fn json_output_has_schema_version() {
    let report = sample_lang_report();
    let global = default_scan_options();
    let args = lang_args(TableFormat::Json);

    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &global, &args).expect("operation must succeed");

    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    let val: serde_json::Value =
        serde_json::from_str(output.trim()).expect("must parse valid JSON");
    assert_eq!(val["schema_version"], SCHEMA_VERSION);
    assert!(val["tool"]["name"].is_string());
    assert!(val["tool"]["version"].is_string());
}

// ---------------------------------------------------------------------------
// 6. Markdown table structure
// ---------------------------------------------------------------------------

#[test]
fn md_table_has_separator_row() {
    let report = sample_lang_report();
    let global = default_scan_options();
    let args = lang_args(TableFormat::Md);

    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &global, &args).expect("operation must succeed");

    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    let lines: Vec<&str> = output.lines().collect();
    // Second line should be the separator row with ---
    assert!(lines[1].contains("---"));
}

// ---------------------------------------------------------------------------
// 7. Row count consistency across formats
// ---------------------------------------------------------------------------

#[test]
fn json_and_tsv_have_same_row_count() {
    let report = sample_lang_report();
    let global = default_scan_options();

    // JSON: count rows array
    let mut json_buf = Vec::new();
    write_lang_report_to(
        &mut json_buf,
        &report,
        &global,
        &lang_args(TableFormat::Json),
    )
    .expect("operation must succeed");
    let json_str = String::from_utf8(json_buf).expect("operation must succeed");
    let val: serde_json::Value =
        serde_json::from_str(json_str.trim()).expect("must parse valid JSON");
    let json_row_count = val["rows"].as_array().expect("must be a JSON array").len();

    // TSV: count data rows (total - header - totals)
    let mut tsv_buf = Vec::new();
    write_lang_report_to(&mut tsv_buf, &report, &global, &lang_args(TableFormat::Tsv))
        .expect("operation must succeed");
    let tsv_str = String::from_utf8(tsv_buf).expect("operation must succeed");
    let tsv_data_rows = tsv_str.lines().count() - 2; // minus header and total rows

    assert_eq!(json_row_count, tsv_data_rows);
    assert_eq!(json_row_count, report.rows.len());
}
