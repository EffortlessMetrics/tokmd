//! Edge-case tests for tokmd-format rendering (W76).
//!
//! Covers: empty data, single entry, unicode, zero counts, large counts,
//! table alignment, TSV special chars, JSON key ordering, CSV quoting,
//! export formats with edge data, CycloneDX structure, diff edge cases.

use std::path::PathBuf;

use tokmd_format::{
    compute_diff_rows, compute_diff_totals, create_diff_receipt, render_diff_md,
    render_diff_md_with_options, write_export_csv_to, write_export_cyclonedx_with_options,
    write_export_json_to, write_export_jsonl_to, write_lang_report_to, write_module_report_to,
    DiffColorMode, DiffRenderOptions,
};
use tokmd_settings::{ChildIncludeMode, ChildrenMode, ScanOptions};
use tokmd_types::{
    ConfigMode, DiffTotals, ExportArgs, ExportData, ExportFormat, FileKind, FileRow, LangArgs,
    LangReport, LangRow, ModuleArgs, ModuleReport, ModuleRow, RedactMode, TableFormat, Totals,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn default_scan_options() -> ScanOptions {
    ScanOptions {
        excluded: vec![],
        config: ConfigMode::None,
        hidden: false,
        no_ignore: false,
        no_ignore_parent: false,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        treat_doc_strings_as_comments: false,
    }
}

fn default_lang_args() -> LangArgs {
    LangArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Md,
        top: 0,
        files: false,
        children: ChildrenMode::Collapse,
    }
}

fn default_module_args() -> ModuleArgs {
    ModuleArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Md,
        top: 0,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn default_export_args() -> ExportArgs {
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

fn zero_totals() -> Totals {
    Totals {
        code: 0,
        lines: 0,
        files: 0,
        bytes: 0,
        tokens: 0,
        avg_lines: 0,
    }
}

fn make_lang_row(lang: &str, code: usize, lines: usize, files: usize) -> LangRow {
    LangRow {
        lang: lang.to_string(),
        code,
        lines,
        files,
        bytes: code * 30,
        tokens: code * 4,
        avg_lines: if files > 0 { lines / files } else { 0 },
    }
}

fn make_module_row(module: &str, code: usize, lines: usize, files: usize) -> ModuleRow {
    ModuleRow {
        module: module.to_string(),
        code,
        lines,
        files,
        bytes: code * 30,
        tokens: code * 4,
        avg_lines: if files > 0 { lines / files } else { 0 },
    }
}

fn make_file_row(path: &str, lang: &str, code: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: String::new(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code,
        comments: 0,
        blanks: 0,
        lines: code,
        bytes: code * 30,
        tokens: code * 4,
    }
}

// ===========================================================================
// 1. Empty data
// ===========================================================================

#[test]
fn lang_report_empty_rows_produces_header_and_total() {
    let report = LangReport {
        rows: vec![],
        total: zero_totals(),
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &report,
        &default_scan_options(),
        &default_lang_args(),
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("Lang"), "should have header");
    assert!(output.contains("Total"), "should have total row");
}

#[test]
fn module_report_empty_rows_produces_header_and_total() {
    let report = ModuleReport {
        rows: vec![],
        total: zero_totals(),
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        top: 0,
    };
    let mut buf = Vec::new();
    write_module_report_to(
        &mut buf,
        &report,
        &default_scan_options(),
        &default_module_args(),
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("Module"), "should have header");
    assert!(output.contains("Total"), "should have total row");
}

#[test]
fn export_csv_empty_rows_produces_header_only() {
    let export = ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &export, &default_export_args()).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("path"), "CSV header should include path");
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 1, "only header line for empty export");
}

#[test]
fn export_jsonl_empty_rows_produces_nothing() {
    let export = ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let mut buf = Vec::new();
    write_export_jsonl_to(
        &mut buf,
        &export,
        &default_scan_options(),
        &default_export_args(),
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(
        output.trim().is_empty(),
        "JSONL with no rows should be empty"
    );
}

// ===========================================================================
// 2. Single entry
// ===========================================================================

#[test]
fn lang_report_single_row_md() {
    let report = LangReport {
        rows: vec![make_lang_row("Rust", 1000, 1500, 10)],
        total: Totals {
            code: 1000,
            lines: 1500,
            files: 10,
            bytes: 30000,
            tokens: 4000,
            avg_lines: 150,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &report,
        &default_scan_options(),
        &default_lang_args(),
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("Rust"), "should contain lang name");
    assert!(
        output.contains("1,000") || output.contains("1000"),
        "should contain code count"
    );
}

#[test]
fn module_report_single_row_md() {
    let report = ModuleReport {
        rows: vec![make_module_row("src", 500, 800, 5)],
        total: Totals {
            code: 500,
            lines: 800,
            files: 5,
            bytes: 15000,
            tokens: 2000,
            avg_lines: 160,
        },
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        top: 0,
    };
    let mut buf = Vec::new();
    write_module_report_to(
        &mut buf,
        &report,
        &default_scan_options(),
        &default_module_args(),
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("src"), "should contain module name");
}

// ===========================================================================
// 3. Unicode in language/module/path names
// ===========================================================================

#[test]
fn lang_report_unicode_language_name_md() {
    let report = LangReport {
        rows: vec![make_lang_row("\u{65E5}\u{672C}\u{8A9E}", 100, 200, 3)],
        total: Totals {
            code: 100,
            lines: 200,
            files: 3,
            bytes: 3000,
            tokens: 400,
            avg_lines: 66,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &report,
        &default_scan_options(),
        &default_lang_args(),
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(
        output.contains("\u{65E5}\u{672C}\u{8A9E}"),
        "Japanese chars must survive rendering"
    );
}

#[test]
fn lang_report_unicode_language_name_tsv() {
    let report = LangReport {
        rows: vec![make_lang_row("donn\u{00E9}es", 50, 80, 2)],
        total: Totals {
            code: 50,
            lines: 80,
            files: 2,
            bytes: 1500,
            tokens: 200,
            avg_lines: 40,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let mut args = default_lang_args();
    args.format = TableFormat::Tsv;
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &default_scan_options(), &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(
        output.contains("donn\u{00E9}es"),
        "French accented chars must survive TSV rendering"
    );
}

#[test]
fn module_report_unicode_module_name() {
    let report = ModuleReport {
        rows: vec![make_module_row("\u{00FC}ber/stra\u{00DF}e", 200, 300, 4)],
        total: Totals {
            code: 200,
            lines: 300,
            files: 4,
            bytes: 6000,
            tokens: 800,
            avg_lines: 75,
        },
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        top: 0,
    };
    let mut buf = Vec::new();
    write_module_report_to(
        &mut buf,
        &report,
        &default_scan_options(),
        &default_module_args(),
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(
        output.contains("\u{00FC}ber"),
        "German chars must survive rendering"
    );
}

#[test]
fn export_csv_unicode_path() {
    let export = ExportData {
        rows: vec![make_file_row("src/\u{65E5}\u{672C}\u{8A9E}.rs", "Rust", 50)],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &export, &default_export_args()).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(
        output.contains("\u{65E5}\u{672C}\u{8A9E}"),
        "Unicode path must appear in CSV"
    );
}

// ===========================================================================
// 4. Zero counts
// ===========================================================================

#[test]
fn lang_report_all_zero_counts() {
    let report = LangReport {
        rows: vec![make_lang_row("EmptyLang", 0, 0, 0)],
        total: zero_totals(),
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &report,
        &default_scan_options(),
        &default_lang_args(),
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(
        output.contains("EmptyLang"),
        "zero-count lang should appear"
    );
}

#[test]
fn export_csv_zero_code_file() {
    let export = ExportData {
        rows: vec![make_file_row("empty.txt", "Plain Text", 0)],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &export, &default_export_args()).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("empty.txt"), "zero-code file must appear");
}

// ===========================================================================
// 5. Large counts (overflow safety)
// ===========================================================================

#[test]
fn lang_report_large_counts_md() {
    let big = 999_999_999;
    let report = LangReport {
        rows: vec![make_lang_row("BigLang", big, big * 2, 1_000_000)],
        total: Totals {
            code: big,
            lines: big * 2,
            files: 1_000_000,
            bytes: big * 30,
            tokens: big * 4,
            avg_lines: big * 2,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &report,
        &default_scan_options(),
        &default_lang_args(),
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("BigLang"), "large count rendering succeeds");
}

#[test]
fn lang_report_large_counts_json() {
    let big = 999_999_999;
    let report = LangReport {
        rows: vec![make_lang_row("BigLang", big, big * 2, 1_000_000)],
        total: Totals {
            code: big,
            lines: big * 2,
            files: 1_000_000,
            bytes: big * 30,
            tokens: big * 4,
            avg_lines: big * 2,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let mut args = default_lang_args();
    args.format = TableFormat::Json;
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &default_scan_options(), &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let v: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(v["rows"][0]["code"], big, "large code count in JSON");
}

// ===========================================================================
// 6. Table alignment (markdown column separators)
// ===========================================================================

#[test]
fn lang_md_table_has_separator_row() {
    let report = LangReport {
        rows: vec![make_lang_row("Rust", 100, 200, 5)],
        total: Totals {
            code: 100,
            lines: 200,
            files: 5,
            bytes: 3000,
            tokens: 400,
            avg_lines: 40,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let mut buf = Vec::new();
    write_lang_report_to(
        &mut buf,
        &report,
        &default_scan_options(),
        &default_lang_args(),
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    let has_separator = output.lines().any(|l| l.contains("---"));
    assert!(has_separator, "Markdown table must have separator row");
}

#[test]
fn module_md_table_has_pipe_columns() {
    let report = ModuleReport {
        rows: vec![make_module_row("src", 100, 200, 5)],
        total: Totals {
            code: 100,
            lines: 200,
            files: 5,
            bytes: 3000,
            tokens: 400,
            avg_lines: 40,
        },
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        top: 0,
    };
    let mut buf = Vec::new();
    write_module_report_to(
        &mut buf,
        &report,
        &default_scan_options(),
        &default_module_args(),
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    for line in output.lines() {
        if !line.is_empty() {
            assert!(
                line.contains('|'),
                "every non-empty line should have pipe: {line}"
            );
        }
    }
}

// ===========================================================================
// 7. TSV special characters
// ===========================================================================

#[test]
fn lang_tsv_tab_separated_columns() {
    let report = LangReport {
        rows: vec![make_lang_row("Rust", 100, 200, 5)],
        total: Totals {
            code: 100,
            lines: 200,
            files: 5,
            bytes: 3000,
            tokens: 400,
            avg_lines: 40,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let mut args = default_lang_args();
    args.format = TableFormat::Tsv;
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &default_scan_options(), &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    for line in output.lines() {
        if !line.is_empty() {
            assert!(line.contains('\t'), "TSV line should have tabs: {line}");
        }
    }
}

#[test]
fn lang_tsv_no_pipe_separators() {
    let report = LangReport {
        rows: vec![make_lang_row("Rust", 100, 200, 5)],
        total: Totals {
            code: 100,
            lines: 200,
            files: 5,
            bytes: 3000,
            tokens: 400,
            avg_lines: 40,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let mut args = default_lang_args();
    args.format = TableFormat::Tsv;
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &default_scan_options(), &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(!output.contains('|'), "TSV should not contain pipe chars");
}

// ===========================================================================
// 8. JSON key ordering
// ===========================================================================

#[test]
fn lang_json_rows_are_array() {
    let report = LangReport {
        rows: vec![
            make_lang_row("Rust", 500, 800, 10),
            make_lang_row("Python", 300, 500, 5),
        ],
        total: Totals {
            code: 800,
            lines: 1300,
            files: 15,
            bytes: 24000,
            tokens: 3200,
            avg_lines: 86,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let mut args = default_lang_args();
    args.format = TableFormat::Json;
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &default_scan_options(), &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let v: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(v["rows"].is_array(), "rows must be a JSON array");
    assert_eq!(v["rows"].as_array().unwrap().len(), 2);
}

#[test]
fn lang_json_row_has_expected_keys() {
    let report = LangReport {
        rows: vec![make_lang_row("Go", 100, 150, 3)],
        total: Totals {
            code: 100,
            lines: 150,
            files: 3,
            bytes: 3000,
            tokens: 400,
            avg_lines: 50,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let mut args = default_lang_args();
    args.format = TableFormat::Json;
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &default_scan_options(), &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let v: serde_json::Value = serde_json::from_str(&output).unwrap();
    let row = &v["rows"][0];
    for key in &["lang", "code", "lines", "files", "bytes", "tokens"] {
        assert!(!row[key].is_null(), "row should have key: {key}");
    }
}

// ===========================================================================
// 9. CSV quoting
// ===========================================================================

#[test]
fn csv_quotes_path_with_comma() {
    let export = ExportData {
        rows: vec![make_file_row("src/hello,world.rs", "Rust", 42)],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &export, &default_export_args()).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(
        output.contains("\"src/hello,world.rs\""),
        "CSV must quote paths containing commas"
    );
}

#[test]
fn csv_quotes_path_with_double_quote() {
    let export = ExportData {
        rows: vec![make_file_row("src/say\"hi\".rs", "Rust", 10)],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &export, &default_export_args()).unwrap();
    let output = String::from_utf8(buf).unwrap();
    // CSV escapes quotes by doubling them
    assert!(
        output.contains("\"\""),
        "CSV must escape double quotes: {output}"
    );
}

#[test]
fn csv_quotes_path_with_newline() {
    let export = ExportData {
        rows: vec![make_file_row("src/line1\nline2.rs", "Rust", 5)],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &export, &default_export_args()).unwrap();
    let output = String::from_utf8(buf).unwrap();
    // The path with newline should be quoted in CSV
    assert!(
        output.contains("\"src/line1\nline2.rs\""),
        "CSV must quote paths with newlines"
    );
}

// ===========================================================================
// 10. Export formats with edge data
// ===========================================================================

#[test]
fn export_json_single_row() {
    let export = ExportData {
        rows: vec![make_file_row("main.rs", "Rust", 42)],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let mut args = default_export_args();
    args.format = ExportFormat::Json;
    let mut buf = Vec::new();
    write_export_json_to(&mut buf, &export, &default_scan_options(), &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let v: serde_json::Value = serde_json::from_str(&output).unwrap();
    // Without meta, export JSON is an array of file rows
    assert!(v.is_array(), "JSON export without meta must be an array");
    assert_eq!(v.as_array().unwrap().len(), 1);
}

#[test]
fn export_jsonl_single_row_is_one_line() {
    let export = ExportData {
        rows: vec![make_file_row("main.rs", "Rust", 42)],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let mut buf = Vec::new();
    write_export_jsonl_to(
        &mut buf,
        &export,
        &default_scan_options(),
        &default_export_args(),
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    let non_empty: Vec<&str> = output.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(non_empty.len(), 1, "one row = one JSONL line");
    let v: serde_json::Value = serde_json::from_str(non_empty[0]).unwrap();
    assert_eq!(v["path"], "main.rs");
}

#[test]
fn export_csv_multiple_rows_line_count() {
    let export = ExportData {
        rows: vec![
            make_file_row("a.rs", "Rust", 10),
            make_file_row("b.py", "Python", 20),
            make_file_row("c.go", "Go", 30),
        ],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &export, &default_export_args()).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 4, "header + 3 data rows");
}

// ===========================================================================
// 11. CycloneDX structure
// ===========================================================================

#[test]
fn cyclonedx_valid_json_structure() {
    let export = ExportData {
        rows: vec![make_file_row("src/lib.rs", "Rust", 100)],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let mut buf = Vec::new();
    write_export_cyclonedx_with_options(
        &mut buf,
        &export,
        RedactMode::None,
        Some("urn:uuid:test-serial".to_string()),
        Some("2024-01-01T00:00:00Z".to_string()),
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    let v: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(v["bomFormat"], "CycloneDX");
    assert!(v["components"].is_array());
}

#[test]
fn cyclonedx_empty_export() {
    let export = ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let mut buf = Vec::new();
    write_export_cyclonedx_with_options(
        &mut buf,
        &export,
        RedactMode::None,
        Some("urn:uuid:test".to_string()),
        Some("2024-01-01T00:00:00Z".to_string()),
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    let v: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(v["components"].as_array().unwrap().len(), 0);
}

#[test]
fn cyclonedx_with_redact_mode() {
    let export = ExportData {
        rows: vec![make_file_row("secret/path.rs", "Rust", 10)],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let mut buf_none = Vec::new();
    write_export_cyclonedx_with_options(
        &mut buf_none,
        &export,
        RedactMode::None,
        Some("urn:uuid:a".to_string()),
        Some("2024-01-01T00:00:00Z".to_string()),
    )
    .unwrap();
    let mut buf_redact = Vec::new();
    write_export_cyclonedx_with_options(
        &mut buf_redact,
        &export,
        RedactMode::Paths,
        Some("urn:uuid:b".to_string()),
        Some("2024-01-01T00:00:00Z".to_string()),
    )
    .unwrap();
    let out_none = String::from_utf8(buf_none).unwrap();
    let out_redact = String::from_utf8(buf_redact).unwrap();
    assert!(out_none.contains("secret/path.rs"));
    assert!(
        !out_redact.contains("secret/path.rs"),
        "redacted output should not contain raw path"
    );
}

// ===========================================================================
// 12. Diff edge cases
// ===========================================================================

#[test]
fn diff_empty_reports_produce_no_rows() {
    let from_report = LangReport {
        rows: vec![],
        total: zero_totals(),
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let to_report = from_report.clone();
    let rows = compute_diff_rows(&from_report, &to_report);
    assert!(rows.is_empty(), "diff of two empty reports = no rows");
}

#[test]
fn diff_identical_reports_filtered_out() {
    let report = LangReport {
        rows: vec![make_lang_row("Rust", 500, 800, 10)],
        total: Totals {
            code: 500,
            lines: 800,
            files: 10,
            bytes: 15000,
            tokens: 2000,
            avg_lines: 80,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let rows = compute_diff_rows(&report, &report);
    // Identical reports produce no diff rows (zero deltas are filtered out)
    assert!(rows.is_empty(), "identical reports should produce no diff rows");
}

#[test]
fn diff_new_language_appears() {
    let from_report = LangReport {
        rows: vec![],
        total: zero_totals(),
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let to_report = LangReport {
        rows: vec![make_lang_row("NewLang", 200, 300, 5)],
        total: Totals {
            code: 200,
            lines: 300,
            files: 5,
            bytes: 6000,
            tokens: 800,
            avg_lines: 60,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let rows = compute_diff_rows(&from_report, &to_report);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].lang, "NewLang");
    assert_eq!(rows[0].delta_code, 200);
    assert_eq!(rows[0].old_code, 0);
    assert_eq!(rows[0].new_code, 200);
}

#[test]
fn diff_language_removed() {
    let from_report = LangReport {
        rows: vec![make_lang_row("OldLang", 100, 200, 3)],
        total: Totals {
            code: 100,
            lines: 200,
            files: 3,
            bytes: 3000,
            tokens: 400,
            avg_lines: 66,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let to_report = LangReport {
        rows: vec![],
        total: zero_totals(),
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let rows = compute_diff_rows(&from_report, &to_report);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].lang, "OldLang");
    assert_eq!(rows[0].delta_code, -100);
    assert_eq!(rows[0].new_code, 0);
}

#[test]
fn diff_totals_default_is_zero() {
    let totals = DiffTotals::default();
    assert_eq!(totals.delta_code, 0);
    assert_eq!(totals.old_code, 0);
    assert_eq!(totals.new_code, 0);
}

#[test]
fn render_diff_md_empty_rows() {
    let output = render_diff_md("v1", "v2", &[], &DiffTotals::default());
    assert!(output.contains("v1"), "should mention from_source");
    assert!(output.contains("v2"), "should mention to_source");
}

#[test]
fn render_diff_md_with_arrow_separator() {
    let from = LangReport {
        rows: vec![make_lang_row("Rust", 100, 200, 5)],
        total: Totals {
            code: 100,
            lines: 200,
            files: 5,
            bytes: 3000,
            tokens: 400,
            avg_lines: 40,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let to = LangReport {
        rows: vec![make_lang_row("Rust", 150, 250, 6)],
        total: Totals {
            code: 150,
            lines: 250,
            files: 6,
            bytes: 4500,
            tokens: 600,
            avg_lines: 41,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let output = render_diff_md("old", "new", &rows, &totals);
    assert!(
        output.contains("\u{2192}") || output.contains("->"),
        "diff header should contain arrow"
    );
}

#[test]
fn render_diff_md_with_options_compact() {
    let rows = compute_diff_rows(
        &LangReport {
            rows: vec![make_lang_row("Rust", 100, 200, 5)],
            total: Totals {
                code: 100,
                lines: 200,
                files: 5,
                bytes: 3000,
                tokens: 400,
                avg_lines: 40,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        },
        &LangReport {
            rows: vec![make_lang_row("Rust", 200, 350, 7)],
            total: Totals {
                code: 200,
                lines: 350,
                files: 7,
                bytes: 6000,
                tokens: 800,
                avg_lines: 50,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        },
    );
    let totals = compute_diff_totals(&rows);
    let opts = DiffRenderOptions {
        compact: true,
        color: DiffColorMode::Off,
    };
    let output = render_diff_md_with_options("a", "b", &rows, &totals, opts);
    assert!(!output.is_empty(), "compact diff should produce output");
}

#[test]
fn create_diff_receipt_has_schema_version() {
    let receipt = create_diff_receipt("from", "to", vec![], DiffTotals::default());
    assert!(receipt.schema_version > 0);
    assert_eq!(receipt.from_source, "from");
    assert_eq!(receipt.to_source, "to");
    assert!(receipt.diff_rows.is_empty());
}
