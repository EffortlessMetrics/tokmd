//! Targeted tests for mutation testing coverage gaps (W50).
//!
//! Each test catches common mutations: replacing operators,
//! negating conditions, removing statements.

use std::path::PathBuf;

use tokmd_format::write_lang_report_to;
use tokmd_settings::ScanOptions;
use tokmd_types::{ChildrenMode, LangArgs, LangReport, LangRow, TableFormat, Totals};

fn default_scan_options() -> ScanOptions {
    ScanOptions::default()
}

fn sample_lang_report(with_files: bool) -> LangReport {
    LangReport {
        rows: vec![
            LangRow {
                lang: "Rust".to_string(),
                code: 500,
                lines: 700,
                files: 5,
                bytes: 20000,
                tokens: 5000,
                avg_lines: 140,
            },
            LangRow {
                lang: "Python".to_string(),
                code: 200,
                lines: 300,
                files: 3,
                bytes: 8000,
                tokens: 2000,
                avg_lines: 100,
            },
        ],
        total: Totals {
            code: 700,
            lines: 1000,
            files: 8,
            bytes: 28000,
            tokens: 7000,
            avg_lines: 125,
        },
        with_files,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

fn lang_args(fmt: TableFormat) -> LangArgs {
    LangArgs {
        paths: vec![PathBuf::from(".")],
        format: fmt,
        top: 0,
        files: true,
        children: ChildrenMode::Collapse,
    }
}

// ---------------------------------------------------------------------------
// 1. Markdown output contains pipe-delimited header markers
// ---------------------------------------------------------------------------

#[test]
fn markdown_contains_header_markers() {
    let report = sample_lang_report(true);
    let args = lang_args(TableFormat::Md);
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &default_scan_options(), &args)
        .expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");

    assert!(output.contains("|Lang|"), "should contain |Lang| header");
    assert!(output.contains("|---|"), "should contain separator row");
    assert!(
        output.contains("|**Total**|"),
        "should contain bold Total row"
    );
}

// ---------------------------------------------------------------------------
// 2. Markdown without files omits Files column
// ---------------------------------------------------------------------------

#[test]
fn markdown_without_files_omits_files_column() {
    let report = sample_lang_report(false);
    let mut args = lang_args(TableFormat::Md);
    args.files = false;
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &default_scan_options(), &args)
        .expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");

    assert!(
        !output.contains("|Files|"),
        "should not contain Files header when with_files=false"
    );
    assert!(output.contains("|Lang|Code|Lines|Bytes|Tokens|"));
}

// ---------------------------------------------------------------------------
// 3. TSV has tab separators (not spaces or commas)
// ---------------------------------------------------------------------------

#[test]
fn tsv_has_tab_separators() {
    let report = sample_lang_report(true);
    let args = lang_args(TableFormat::Tsv);
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &default_scan_options(), &args)
        .expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");

    assert!(output.contains('\t'), "TSV output must contain tabs");
    // Header should be tab-separated
    assert!(
        output.contains("Lang\tCode\t"),
        "header should be tab-separated"
    );
    // Should NOT contain pipe separators
    assert!(
        !output.contains('|'),
        "TSV should not contain pipe characters"
    );
}

// ---------------------------------------------------------------------------
// 4. JSON output contains "schema_version" key
// ---------------------------------------------------------------------------

#[test]
fn json_contains_schema_version() {
    let report = sample_lang_report(true);
    let args = lang_args(TableFormat::Json);
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &default_scan_options(), &args)
        .expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");

    assert!(
        output.contains("\"schema_version\""),
        "JSON output must contain schema_version key"
    );
    // Parse and verify the value
    let json: serde_json::Value =
        serde_json::from_str(output.trim()).expect("must parse valid JSON");
    assert_eq!(json["schema_version"], 2);
}

// ---------------------------------------------------------------------------
// 5. JSON output contains mode field
// ---------------------------------------------------------------------------

#[test]
fn json_contains_mode_field() {
    let report = sample_lang_report(true);
    let args = lang_args(TableFormat::Json);
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &default_scan_options(), &args)
        .expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(output.trim()).expect("must parse valid JSON");

    assert_eq!(json["mode"], "lang");
}

// ---------------------------------------------------------------------------
// 6. Markdown contains actual data values
// ---------------------------------------------------------------------------

#[test]
fn markdown_contains_data_values() {
    let report = sample_lang_report(true);
    let args = lang_args(TableFormat::Md);
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &default_scan_options(), &args)
        .expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");

    assert!(output.contains("Rust"), "should contain language name Rust");
    assert!(output.contains("500"), "should contain code count 500");
    assert!(
        output.contains("Python"),
        "should contain language name Python"
    );
}

// ---------------------------------------------------------------------------
// 7. TSV total row is present
// ---------------------------------------------------------------------------

#[test]
fn tsv_total_row_present() {
    let report = sample_lang_report(true);
    let args = lang_args(TableFormat::Tsv);
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &default_scan_options(), &args)
        .expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");

    assert!(output.contains("Total\t"), "TSV should have a Total row");
}

// ---------------------------------------------------------------------------
// 8. JSON output includes tool info
// ---------------------------------------------------------------------------

#[test]
fn json_includes_tool_info() {
    let report = sample_lang_report(true);
    let args = lang_args(TableFormat::Json);
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &default_scan_options(), &args)
        .expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(output.trim()).expect("must parse valid JSON");

    assert_eq!(json["tool"]["name"], "tokmd");
    assert!(json["tool"]["version"].is_string());
}

// ---------------------------------------------------------------------------
// 9. Markdown row count matches report rows + header + separator + total
// ---------------------------------------------------------------------------

#[test]
fn markdown_line_count() {
    let report = sample_lang_report(true);
    let args = lang_args(TableFormat::Md);
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &default_scan_options(), &args)
        .expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");

    let line_count = output.lines().count();
    // header + separator + 2 data rows + total = 5
    assert_eq!(
        line_count, 5,
        "expected 5 lines: header + sep + 2 rows + total"
    );
}

// ---------------------------------------------------------------------------
// 10. JSON rows array length matches report rows
// ---------------------------------------------------------------------------

#[test]
fn json_rows_length_matches() {
    let report = sample_lang_report(true);
    let args = lang_args(TableFormat::Json);
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &default_scan_options(), &args)
        .expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(output.trim()).expect("must parse valid JSON");

    let rows = json["rows"].as_array().expect("must be a JSON array");
    assert_eq!(rows.len(), 2, "should have 2 language rows");
}
