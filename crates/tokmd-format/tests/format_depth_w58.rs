//! Depth tests for tokmd-format rendering paths.
//!
//! Covers: markdown table structure, TSV delimiters, JSON validity,
//! JSONL line-per-record, CSV quoting/escaping, format selection via
//! `write_lang_report_to` / `write_module_report_to`, edge cases
//! (zero rows, single row, many rows, unicode, long names).

use std::path::PathBuf;

use tokmd_format::{
    write_export_csv_to, write_export_json_to, write_export_jsonl_to, write_lang_report_to,
    write_module_report_to,
};
use tokmd_settings::ScanOptions;
use tokmd_types::{
    ChildIncludeMode, ChildrenMode, ExportArgs, ExportData, ExportFormat, FileKind, FileRow,
    LangArgs, LangReport, LangRow, ModuleArgs, ModuleReport, ModuleRow, RedactMode, TableFormat,
    Totals,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn lang_row(lang: &str, code: usize) -> LangRow {
    LangRow {
        lang: lang.to_string(),
        code,
        lines: code + code / 5,
        files: 1 + code / 100,
        bytes: code * 10,
        tokens: code * 3,
        avg_lines: if code > 0 { code + code / 5 } else { 0 },
    }
}

fn totals_from_lang_rows(rows: &[LangRow]) -> Totals {
    Totals {
        code: rows.iter().map(|r| r.code).sum(),
        lines: rows.iter().map(|r| r.lines).sum(),
        files: rows.iter().map(|r| r.files).sum(),
        bytes: rows.iter().map(|r| r.bytes).sum(),
        tokens: rows.iter().map(|r| r.tokens).sum(),
        avg_lines: 0,
    }
}

fn module_row(module: &str, code: usize) -> ModuleRow {
    ModuleRow {
        module: module.to_string(),
        code,
        lines: code + code / 5,
        files: 1 + code / 100,
        bytes: code * 10,
        tokens: code * 3,
        avg_lines: if code > 0 { code + code / 5 } else { 0 },
    }
}

fn totals_from_module_rows(rows: &[ModuleRow]) -> Totals {
    Totals {
        code: rows.iter().map(|r| r.code).sum(),
        lines: rows.iter().map(|r| r.lines).sum(),
        files: rows.iter().map(|r| r.files).sum(),
        bytes: rows.iter().map(|r| r.bytes).sum(),
        tokens: rows.iter().map(|r| r.tokens).sum(),
        avg_lines: 0,
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
        module_roots: vec!["crates".into()],
        module_depth: 2,
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

fn default_scan() -> ScanOptions {
    ScanOptions::default()
}

fn file_row(path: &str, module: &str, lang: &str, kind: FileKind, code: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: lang.to_string(),
        kind,
        code,
        comments: code / 5,
        blanks: code / 10,
        lines: code + code / 5 + code / 10,
        bytes: code * 10,
        tokens: code * 3,
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
// 1. Markdown table structure
// ===========================================================================

#[test]
fn md_lang_has_header_separator_and_total() {
    let rows = vec![lang_row("Rust", 500), lang_row("Python", 300)];
    let report = make_lang_report(rows, false);
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &report,
        &default_scan(),
        &lang_args(TableFormat::Md),
    )
    .expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    let lines: Vec<&str> = out.lines().collect();
    assert!(lines[0].starts_with("|Lang|"), "first line is header");
    assert!(lines[1].contains("---"), "second line is separator");
    assert!(
        lines
            .last()
            .expect("operation must succeed")
            .contains("**Total**"),
        "last line is total"
    );
}

#[test]
fn md_lang_with_files_has_seven_columns() {
    let report = make_lang_report(vec![lang_row("Go", 100)], true);
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &report,
        &default_scan(),
        &lang_args(TableFormat::Md),
    )
    .expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    let header = out
        .lines()
        .next()
        .expect("output must have at least one line");
    // |Lang|Code|Lines|Files|Bytes|Tokens|Avg| → 7 data columns, 8 pipes
    assert_eq!(header.matches('|').count(), 8, "7 columns = 8 pipes");
}

#[test]
fn md_lang_without_files_has_five_columns() {
    let report = make_lang_report(vec![lang_row("Go", 100)], false);
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &report,
        &default_scan(),
        &lang_args(TableFormat::Md),
    )
    .expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    let header = out
        .lines()
        .next()
        .expect("output must have at least one line");
    // |Lang|Code|Lines|Bytes|Tokens| → 5 data columns, 6 pipes
    assert_eq!(header.matches('|').count(), 6, "5 columns = 6 pipes");
}

#[test]
fn md_module_has_seven_columns() {
    let report = make_module_report(vec![module_row("src", 200)]);
    let mut buf = Vec::new();
    write_module_report_to(
        &mut buf,
        &report,
        &default_scan(),
        &module_args(TableFormat::Md),
    )
    .expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    let header = out
        .lines()
        .next()
        .expect("output must have at least one line");
    assert_eq!(header.matches('|').count(), 8, "7 columns = 8 pipes");
    assert!(header.starts_with("|Module|"));
}

#[test]
fn md_row_count_matches_data_plus_header_sep_total() {
    let rows = vec![
        lang_row("Rust", 500),
        lang_row("Python", 300),
        lang_row("Go", 100),
    ];
    let report = make_lang_report(rows, false);
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &report,
        &default_scan(),
        &lang_args(TableFormat::Md),
    )
    .expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    // header + separator + 3 data rows + total = 6
    assert_eq!(out.lines().count(), 6);
}

// ===========================================================================
// 2. TSV output
// ===========================================================================

#[test]
fn tsv_lang_uses_tab_delimiters() {
    let report = make_lang_report(vec![lang_row("Rust", 200)], false);
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &report,
        &default_scan(),
        &lang_args(TableFormat::Tsv),
    )
    .expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    for line in out.lines() {
        assert!(line.contains('\t'), "each TSV line has tabs: {line}");
    }
}

#[test]
fn tsv_lang_header_columns_match() {
    let report = make_lang_report(vec![lang_row("Rust", 200)], false);
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &report,
        &default_scan(),
        &lang_args(TableFormat::Tsv),
    )
    .expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    let header = out
        .lines()
        .next()
        .expect("output must have at least one line");
    let cols: Vec<&str> = header.split('\t').collect();
    assert_eq!(cols, vec!["Lang", "Code", "Lines", "Bytes", "Tokens"]);
}

#[test]
fn tsv_lang_with_files_header_has_seven_columns() {
    let report = make_lang_report(vec![lang_row("Rust", 200)], true);
    let mut buf = Vec::new();
    let mut args = lang_args(TableFormat::Tsv);
    args.files = true;
    write_lang_report_to(&mut buf, &report, &default_scan(), &args)
        .expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    let header = out
        .lines()
        .next()
        .expect("output must have at least one line");
    let cols: Vec<&str> = header.split('\t').collect();
    assert_eq!(
        cols,
        vec!["Lang", "Code", "Lines", "Files", "Bytes", "Tokens", "Avg"]
    );
}

#[test]
fn tsv_module_header_columns_match() {
    let report = make_module_report(vec![module_row("src", 100)]);
    let mut buf = Vec::new();
    write_module_report_to(
        &mut buf,
        &report,
        &default_scan(),
        &module_args(TableFormat::Tsv),
    )
    .expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    let header = out
        .lines()
        .next()
        .expect("output must have at least one line");
    let cols: Vec<&str> = header.split('\t').collect();
    assert_eq!(
        cols,
        vec!["Module", "Code", "Lines", "Files", "Bytes", "Tokens", "Avg"]
    );
}

// ===========================================================================
// 3. JSON output structure validity
// ===========================================================================

#[test]
fn json_lang_output_is_valid_json() {
    let report = make_lang_report(vec![lang_row("Rust", 500)], false);
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &report,
        &default_scan(),
        &lang_args(TableFormat::Json),
    )
    .expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("operation must succeed");
    assert!(v.is_object(), "JSON output is an object");
    assert!(v["schema_version"].is_number());
    assert_eq!(v["mode"], "lang");
}

#[test]
fn json_module_output_is_valid_json() {
    let report = make_module_report(vec![module_row("src", 200)]);
    let mut buf = Vec::new();
    write_module_report_to(
        &mut buf,
        &report,
        &default_scan(),
        &module_args(TableFormat::Json),
    )
    .expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("operation must succeed");
    assert!(v.is_object());
    assert_eq!(v["mode"], "module");
}

#[test]
fn json_lang_contains_rows_and_total() {
    let rows = vec![lang_row("Rust", 500), lang_row("Go", 200)];
    let report = make_lang_report(rows, false);
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &report,
        &default_scan(),
        &lang_args(TableFormat::Json),
    )
    .expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("operation must succeed");
    let json_rows = v["rows"].as_array().expect("must be a JSON array");
    assert_eq!(json_rows.len(), 2);
    assert!(v["total"].is_object());
}

// ===========================================================================
// 4. JSONL export: one line per record
// ===========================================================================

#[test]
fn jsonl_line_count_equals_row_count() {
    let rows = vec![
        file_row("src/a.rs", "src", "Rust", FileKind::Parent, 100),
        file_row("src/b.rs", "src", "Rust", FileKind::Parent, 50),
    ];
    let data = export_data(rows);
    let args = export_args(ExportFormat::Jsonl);
    let mut buf = Vec::new();

    write_export_jsonl_to(&mut buf, &data, &default_scan(), &args).expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    // meta=false → only data rows
    assert_eq!(out.lines().count(), 2);
}

#[test]
fn jsonl_each_line_parses_as_json() {
    let rows = vec![
        file_row("src/a.rs", "src", "Rust", FileKind::Parent, 100),
        file_row("src/b.rs", "src", "Rust", FileKind::Parent, 50),
    ];
    let data = export_data(rows);
    let args = export_args(ExportFormat::Jsonl);
    let mut buf = Vec::new();

    write_export_jsonl_to(&mut buf, &data, &default_scan(), &args).expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    for (i, line) in out.lines().enumerate() {
        assert!(
            serde_json::from_str::<serde_json::Value>(line).is_ok(),
            "line {i} is not valid JSON: {line}"
        );
    }
}

// ===========================================================================
// 5. CSV quoting and escaping
// ===========================================================================

#[test]
fn csv_header_row_present() {
    let data = export_data(vec![file_row(
        "src/lib.rs",
        "src",
        "Rust",
        FileKind::Parent,
        100,
    )]);
    let args = export_args(ExportFormat::Csv);
    let mut buf = Vec::new();

    write_export_csv_to(&mut buf, &data, &args).expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    let header = out
        .lines()
        .next()
        .expect("output must have at least one line");
    assert!(header.contains("path"), "CSV header has path column");
    assert!(header.contains("lang"), "CSV header has lang column");
    assert!(header.contains("code"), "CSV header has code column");
}

#[test]
fn csv_quotes_fields_with_commas() {
    let data = export_data(vec![file_row(
        "src/a,b.rs",
        "src",
        "Rust",
        FileKind::Parent,
        100,
    )]);
    let args = export_args(ExportFormat::Csv);
    let mut buf = Vec::new();

    write_export_csv_to(&mut buf, &data, &args).expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    // The path field with a comma should be quoted
    let data_line = out.lines().nth(1).expect("operation must succeed");
    assert!(
        data_line.contains("\"src/a,b.rs\""),
        "field with comma must be quoted: {data_line}"
    );
}

#[test]
fn csv_escapes_quotes_in_fields() {
    let data = export_data(vec![file_row(
        "src/a\"b.rs",
        "src",
        "Rust",
        FileKind::Parent,
        100,
    )]);
    let args = export_args(ExportFormat::Csv);
    let mut buf = Vec::new();

    write_export_csv_to(&mut buf, &data, &args).expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    // Embedded quotes should be escaped as ""
    assert!(
        out.contains("\"\""),
        "embedded quotes must be doubled: {out}"
    );
}

#[test]
fn csv_column_count_consistent() {
    let data = export_data(vec![
        file_row("src/a.rs", "src", "Rust", FileKind::Parent, 100),
        file_row("src/b.rs", "src", "Rust", FileKind::Parent, 50),
    ]);
    let args = export_args(ExportFormat::Csv);
    let mut buf = Vec::new();

    write_export_csv_to(&mut buf, &data, &args).expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    let lines: Vec<&str> = out.lines().collect();
    let header_cols = lines[0].split(',').count();
    for (i, line) in lines.iter().enumerate().skip(1) {
        // CSV fields with commas are quoted, so simple split may overcount;
        // use the csv crate reader for precise check
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(line.as_bytes());
        let record = rdr
            .records()
            .next()
            .expect("operation must succeed")
            .expect("operation must succeed");
        assert_eq!(record.len(), header_cols, "line {i} column count mismatch");
    }
}

// ===========================================================================
// 6. Format selection logic
// ===========================================================================

#[test]
fn format_selection_md_produces_pipe_tables() {
    let report = make_lang_report(vec![lang_row("Rust", 100)], false);
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &report,
        &default_scan(),
        &lang_args(TableFormat::Md),
    )
    .expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");
    assert!(out.contains('|'), "Md format uses pipe tables");
    assert!(!out.contains('\t'), "Md format has no tabs");
}

#[test]
fn format_selection_tsv_produces_tabs() {
    let report = make_lang_report(vec![lang_row("Rust", 100)], false);
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &report,
        &default_scan(),
        &lang_args(TableFormat::Tsv),
    )
    .expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");
    assert!(out.contains('\t'), "TSV format uses tabs");
}

#[test]
fn format_selection_json_produces_json() {
    let report = make_lang_report(vec![lang_row("Rust", 100)], false);
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &report,
        &default_scan(),
        &lang_args(TableFormat::Json),
    )
    .expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");
    assert!(
        serde_json::from_str::<serde_json::Value>(out.trim()).is_ok(),
        "JSON format produces valid JSON"
    );
}

// ===========================================================================
// 7. Zero rows, single row, many rows
// ===========================================================================

#[test]
fn md_with_zero_rows_still_has_header_and_total() {
    let report = make_lang_report(vec![], false);
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &report,
        &default_scan(),
        &lang_args(TableFormat::Md),
    )
    .expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    let lines: Vec<&str> = out.lines().collect();
    // header + separator + total = 3
    assert_eq!(lines.len(), 3, "zero rows: header + sep + total");
}

#[test]
fn tsv_with_zero_rows_still_has_header_and_total() {
    let report = make_lang_report(vec![], false);
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &report,
        &default_scan(),
        &lang_args(TableFormat::Tsv),
    )
    .expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    let lines: Vec<&str> = out.lines().collect();
    // header + total = 2
    assert_eq!(lines.len(), 2, "zero rows: header + total");
}

#[test]
fn single_row_lang_report_md() {
    let report = make_lang_report(vec![lang_row("Rust", 500)], false);
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &report,
        &default_scan(),
        &lang_args(TableFormat::Md),
    )
    .expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    assert_eq!(out.lines().count(), 4, "header + sep + 1 row + total");
    assert!(out.contains("Rust"));
}

#[test]
fn many_rows_lang_report_md() {
    let rows: Vec<LangRow> = (0..50)
        .map(|i| lang_row(&format!("Lang{i}"), i * 10))
        .collect();
    let report = make_lang_report(rows, false);
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &report,
        &default_scan(),
        &lang_args(TableFormat::Md),
    )
    .expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    // header + sep + 50 rows + total = 53
    assert_eq!(out.lines().count(), 53);
}

#[test]
fn jsonl_with_zero_rows_produces_empty() {
    let data = export_data(vec![]);
    let args = export_args(ExportFormat::Jsonl);
    let mut buf = Vec::new();

    write_export_jsonl_to(&mut buf, &data, &default_scan(), &args).expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    assert!(out.trim().is_empty(), "zero rows → empty JSONL: {out:?}");
}

#[test]
fn csv_with_zero_rows_has_only_header() {
    let data = export_data(vec![]);
    let args = export_args(ExportFormat::Csv);
    let mut buf = Vec::new();

    write_export_csv_to(&mut buf, &data, &args).expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 1, "zero rows: header only");
}

#[test]
fn json_export_with_zero_rows_no_meta_is_empty_array() {
    let data = export_data(vec![]);
    let args = export_args(ExportFormat::Json);
    let mut buf = Vec::new();

    write_export_json_to(&mut buf, &data, &default_scan(), &args).expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("operation must succeed");
    let arr = v.as_array().expect("must be a JSON array");
    assert!(arr.is_empty(), "zero rows without meta → empty array");
}

#[test]
fn json_export_with_meta_is_object() {
    let data = export_data(vec![file_row(
        "src/a.rs",
        "src",
        "Rust",
        FileKind::Parent,
        10,
    )]);
    let mut args = export_args(ExportFormat::Json);
    args.meta = true;
    let mut buf = Vec::new();

    write_export_json_to(&mut buf, &data, &default_scan(), &args).expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("operation must succeed");
    assert!(v.is_object(), "with meta → receipt object");
    assert!(v["schema_version"].is_number());
}

// ===========================================================================
// 8. Column headers match expected order
// ===========================================================================

#[test]
fn md_lang_header_order_without_files() {
    let report = make_lang_report(vec![lang_row("Rust", 1)], false);
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &report,
        &default_scan(),
        &lang_args(TableFormat::Md),
    )
    .expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    let header = out
        .lines()
        .next()
        .expect("output must have at least one line");
    assert_eq!(header, "|Lang|Code|Lines|Bytes|Tokens|");
}

#[test]
fn md_lang_header_order_with_files() {
    let report = make_lang_report(vec![lang_row("Rust", 1)], true);
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &report,
        &default_scan(),
        &lang_args(TableFormat::Md),
    )
    .expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    let header = out
        .lines()
        .next()
        .expect("output must have at least one line");
    assert_eq!(header, "|Lang|Code|Lines|Files|Bytes|Tokens|Avg|");
}

#[test]
fn md_module_header_order() {
    let report = make_module_report(vec![module_row("src", 1)]);
    let mut buf = Vec::new();
    write_module_report_to(
        &mut buf,
        &report,
        &default_scan(),
        &module_args(TableFormat::Md),
    )
    .expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    let header = out
        .lines()
        .next()
        .expect("output must have at least one line");
    assert_eq!(header, "|Module|Code|Lines|Files|Bytes|Tokens|Avg|");
}

// ===========================================================================
// 9. Unicode and long language names
// ===========================================================================

#[test]
fn md_unicode_language_name() {
    let report = make_lang_report(vec![lang_row("日本語", 100)], false);
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &report,
        &default_scan(),
        &lang_args(TableFormat::Md),
    )
    .expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    assert!(out.contains("日本語"), "unicode name preserved");
}

#[test]
fn tsv_unicode_language_name() {
    let report = make_lang_report(vec![lang_row("Ñoño", 50)], false);
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &report,
        &default_scan(),
        &lang_args(TableFormat::Tsv),
    )
    .expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    assert!(out.contains("Ñoño"), "unicode name preserved in TSV");
}

#[test]
fn md_very_long_language_name() {
    let long_name = "A".repeat(200);
    let report = make_lang_report(vec![lang_row(&long_name, 100)], false);
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &report,
        &default_scan(),
        &lang_args(TableFormat::Md),
    )
    .expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    assert!(out.contains(&long_name), "long language name not truncated");
}

#[test]
fn json_unicode_roundtrip() {
    let report = make_lang_report(vec![lang_row("中文", 100)], false);
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &report,
        &default_scan(),
        &lang_args(TableFormat::Json),
    )
    .expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("operation must succeed");
    let rows = v["rows"].as_array().expect("must be a JSON array");
    assert_eq!(rows[0]["lang"], "中文");
}

#[test]
fn csv_unicode_file_path() {
    let data = export_data(vec![file_row(
        "src/données.rs",
        "src",
        "Rust",
        FileKind::Parent,
        100,
    )]);
    let args = export_args(ExportFormat::Csv);
    let mut buf = Vec::new();

    write_export_csv_to(&mut buf, &data, &args).expect("operation must succeed");
    let out = String::from_utf8(buf).expect("output must be valid UTF-8");

    assert!(out.contains("données"), "unicode path preserved in CSV");
}
