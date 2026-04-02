//! BDD-style tests for tokmd-format rendering across all output formats.
//!
//! Naming convention: given_<precondition>_when_<action>_then_<expected>

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

// ===========================================================================
// Test helpers
// ===========================================================================

fn default_scan_options() -> ScanOptions {
    ScanOptions::default()
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
        module_depth: 2,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        redact: RedactMode::None,
        meta: true,
        strip_prefix: None,
    }
}

fn sample_lang_report() -> LangReport {
    LangReport {
        rows: vec![
            LangRow {
                lang: "Rust".to_string(),
                code: 5000,
                lines: 6200,
                files: 42,
                bytes: 180000,
                tokens: 12500,
                avg_lines: 148,
            },
            LangRow {
                lang: "TOML".to_string(),
                code: 200,
                lines: 260,
                files: 5,
                bytes: 8000,
                tokens: 500,
                avg_lines: 52,
            },
        ],
        total: Totals {
            code: 5200,
            lines: 6460,
            files: 47,
            bytes: 188000,
            tokens: 13000,
            avg_lines: 137,
        },
        with_files: true,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

fn single_lang_report() -> LangReport {
    LangReport {
        rows: vec![LangRow {
            lang: "Python".to_string(),
            code: 300,
            lines: 400,
            files: 3,
            bytes: 12000,
            tokens: 750,
            avg_lines: 133,
        }],
        total: Totals {
            code: 300,
            lines: 400,
            files: 3,
            bytes: 12000,
            tokens: 750,
            avg_lines: 133,
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
                lang: "HTML".to_string(),
                code: 800,
                lines: 1000,
                files: 10,
                bytes: 32000,
                tokens: 2000,
                avg_lines: 100,
            },
            LangRow {
                lang: "JavaScript (embedded)".to_string(),
                code: 200,
                lines: 250,
                files: 5,
                bytes: 8000,
                tokens: 500,
                avg_lines: 50,
            },
            LangRow {
                lang: "CSS (embedded)".to_string(),
                code: 100,
                lines: 120,
                files: 3,
                bytes: 4000,
                tokens: 250,
                avg_lines: 40,
            },
        ],
        total: Totals {
            code: 1100,
            lines: 1370,
            files: 18,
            bytes: 44000,
            tokens: 2750,
            avg_lines: 76,
        },
        with_files: true,
        children: ChildrenMode::Separate,
        top: 0,
    }
}

fn sample_module_report() -> ModuleReport {
    ModuleReport {
        rows: vec![
            ModuleRow {
                module: "crates/core".to_string(),
                code: 3000,
                lines: 3800,
                files: 25,
                bytes: 120000,
                tokens: 7500,
                avg_lines: 152,
            },
            ModuleRow {
                module: "crates/format".to_string(),
                code: 1500,
                lines: 1900,
                files: 12,
                bytes: 60000,
                tokens: 3750,
                avg_lines: 158,
            },
            ModuleRow {
                module: "crates/types".to_string(),
                code: 500,
                lines: 600,
                files: 4,
                bytes: 20000,
                tokens: 1250,
                avg_lines: 150,
            },
        ],
        total: Totals {
            code: 5000,
            lines: 6300,
            files: 41,
            bytes: 200000,
            tokens: 12500,
            avg_lines: 154,
        },
        module_roots: vec!["crates".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
        top: 0,
    }
}

fn nested_module_report() -> ModuleReport {
    ModuleReport {
        rows: vec![
            ModuleRow {
                module: "src".to_string(),
                code: 2000,
                lines: 2500,
                files: 15,
                bytes: 80000,
                tokens: 5000,
                avg_lines: 167,
            },
            ModuleRow {
                module: "src/api".to_string(),
                code: 800,
                lines: 1000,
                files: 6,
                bytes: 32000,
                tokens: 2000,
                avg_lines: 167,
            },
            ModuleRow {
                module: "src/api/v2".to_string(),
                code: 300,
                lines: 380,
                files: 3,
                bytes: 12000,
                tokens: 750,
                avg_lines: 127,
            },
        ],
        total: Totals {
            code: 3100,
            lines: 3880,
            files: 24,
            bytes: 124000,
            tokens: 7750,
            avg_lines: 162,
        },
        module_roots: vec!["src".to_string()],
        module_depth: 3,
        children: ChildIncludeMode::Separate,
        top: 0,
    }
}

fn sample_file_rows() -> Vec<FileRow> {
    vec![
        FileRow {
            path: "src/main.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 150,
            comments: 30,
            blanks: 20,
            lines: 200,
            bytes: 6000,
            tokens: 375,
        },
        FileRow {
            path: "src/lib.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 300,
            comments: 50,
            blanks: 30,
            lines: 380,
            bytes: 12000,
            tokens: 750,
        },
        FileRow {
            path: "tests/integration.rs".to_string(),
            module: "tests".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 80,
            comments: 10,
            blanks: 8,
            lines: 98,
            bytes: 3200,
            tokens: 200,
        },
    ]
}

fn sample_export_data() -> ExportData {
    ExportData {
        rows: sample_file_rows(),
        module_roots: vec!["src".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

fn render_to_string<F>(f: F) -> String
where
    F: FnOnce(&mut Vec<u8>) -> anyhow::Result<()>,
{
    let mut buf = Vec::new();
    f(&mut buf).expect("render should succeed");
    String::from_utf8(buf).expect("output should be valid UTF-8")
}

// ===========================================================================
// Markdown rendering
// ===========================================================================

#[test]
fn given_lang_receipt_when_rendered_as_markdown_then_has_header_and_table() {
    let report = sample_lang_report();
    let args = lang_args(TableFormat::Md);
    let output =
        render_to_string(|buf| write_lang_report_to(buf, &report, &default_scan_options(), &args));

    // Has markdown table header
    assert!(
        output.contains("|Lang|"),
        "should contain Lang header column"
    );
    assert!(
        output.contains("|Code|"),
        "should contain Code header column"
    );

    // Has separator row with alignment markers
    assert!(output.contains("|---|"), "should contain separator");
    assert!(
        output.contains("---:"),
        "should contain right-alignment marker"
    );

    // Has data rows
    assert!(output.contains("|Rust|"), "should contain Rust row");
    assert!(output.contains("|TOML|"), "should contain TOML row");

    // Has total row
    assert!(
        output.contains("|**Total**|"),
        "should contain bold Total row"
    );
}

#[test]
fn given_empty_receipt_when_rendered_then_minimal_output() {
    let report = empty_lang_report();
    let args = lang_args(TableFormat::Md);
    let output =
        render_to_string(|buf| write_lang_report_to(buf, &report, &default_scan_options(), &args));

    // Should still have header + separator + total (3 lines minimum)
    let lines: Vec<&str> = output.lines().collect();
    assert!(
        lines.len() >= 3,
        "empty receipt should still have header, separator, and total"
    );

    // Header present
    assert!(output.contains("|Lang|"), "should still have header");

    // Total row with zeros
    assert!(
        output.contains("|**Total**|0|0|0|0|"),
        "should have zero totals"
    );

    // No language data rows
    assert!(
        !output.contains("|Rust|"),
        "should not contain any language rows"
    );
}

#[test]
fn given_receipt_with_one_language_when_rendered_then_single_row() {
    let report = single_lang_report();
    let args = lang_args(TableFormat::Md);
    let output =
        render_to_string(|buf| write_lang_report_to(buf, &report, &default_scan_options(), &args));

    // Header + separator + 1 data row + total = 4 lines
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(
        lines.len(),
        4,
        "should have exactly 4 lines for single-language report"
    );

    // Data row present
    assert!(
        output.contains("|Python|300|400|3|12000|750|133|"),
        "should contain Python row"
    );
}

#[test]
fn given_receipt_with_embedded_languages_then_embedded_shown() {
    let report = embedded_lang_report();
    let args = lang_args(TableFormat::Md);
    let output =
        render_to_string(|buf| write_lang_report_to(buf, &report, &default_scan_options(), &args));

    // Embedded language rows visible
    assert!(
        output.contains("JavaScript (embedded)"),
        "should show embedded JavaScript"
    );
    assert!(
        output.contains("CSS (embedded)"),
        "should show embedded CSS"
    );

    // Parent language also present
    assert!(
        output.contains("|HTML|"),
        "should show parent HTML language"
    );

    // Total accounts for everything
    assert!(
        output.contains("|**Total**|1100|"),
        "total code should sum all rows"
    );
}

#[test]
fn given_lang_markdown_then_every_row_starts_and_ends_with_pipe() {
    let report = sample_lang_report();
    let args = lang_args(TableFormat::Md);
    let output =
        render_to_string(|buf| write_lang_report_to(buf, &report, &default_scan_options(), &args));

    for line in output.lines() {
        assert!(line.starts_with('|'), "line should start with pipe: {line}");
        assert!(line.ends_with('|'), "line should end with pipe: {line}");
    }
}

// ===========================================================================
// TSV rendering
// ===========================================================================

#[test]
fn given_lang_receipt_when_rendered_as_tsv_then_tab_separated() {
    let report = sample_lang_report();
    let args = lang_args(TableFormat::Tsv);
    let output =
        render_to_string(|buf| write_lang_report_to(buf, &report, &default_scan_options(), &args));

    // Every line uses tabs as separators
    for line in output.lines() {
        assert!(
            line.contains('\t'),
            "each line should be tab-separated: {line}"
        );
    }

    // No pipe characters (not Markdown)
    assert!(
        !output.contains('|'),
        "TSV should not contain pipe characters"
    );
}

#[test]
fn given_tsv_output_then_columns_are_correct() {
    let report = sample_lang_report();
    let args = lang_args(TableFormat::Tsv);
    let output =
        render_to_string(|buf| write_lang_report_to(buf, &report, &default_scan_options(), &args));

    let header = output.lines().next().expect("should have header");
    let cols: Vec<&str> = header.split('\t').collect();

    // with_files=true should have 7 columns
    assert_eq!(cols.len(), 7, "should have 7 columns with files");
    assert_eq!(cols[0], "Lang");
    assert_eq!(cols[1], "Code");
    assert_eq!(cols[2], "Lines");
    assert_eq!(cols[3], "Files");
    assert_eq!(cols[4], "Bytes");
    assert_eq!(cols[5], "Tokens");
    assert_eq!(cols[6], "Avg");
}

#[test]
fn given_empty_receipt_tsv_then_header_only() {
    let report = empty_lang_report();
    let args = lang_args(TableFormat::Tsv);
    let output =
        render_to_string(|buf| write_lang_report_to(buf, &report, &default_scan_options(), &args));

    // Should have header + total
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(
        lines.len(),
        2,
        "empty TSV should have header and total only"
    );
    assert!(
        lines[0].starts_with("Lang\t"),
        "first line should be header"
    );
    assert!(
        lines[1].starts_with("Total\t"),
        "second line should be total"
    );
}

#[test]
fn given_tsv_then_consistent_column_count() {
    let report = sample_lang_report();
    let args = lang_args(TableFormat::Tsv);
    let output =
        render_to_string(|buf| write_lang_report_to(buf, &report, &default_scan_options(), &args));

    let expected_tabs = output
        .lines()
        .next()
        .expect("output must have at least one line")
        .matches('\t')
        .count();
    for line in output.lines() {
        assert_eq!(
            line.matches('\t').count(),
            expected_tabs,
            "all lines should have same number of tabs: {line}"
        );
    }
}

// ===========================================================================
// JSON rendering
// ===========================================================================

#[test]
fn given_lang_receipt_when_rendered_as_json_then_valid_json() {
    let report = sample_lang_report();
    let args = lang_args(TableFormat::Json);
    let output =
        render_to_string(|buf| write_lang_report_to(buf, &report, &default_scan_options(), &args));

    let parsed: serde_json::Value =
        serde_json::from_str(output.trim()).expect("should be valid JSON");
    assert!(parsed.is_object(), "top-level should be a JSON object");
}

#[test]
fn given_json_output_then_has_schema_version() {
    let report = sample_lang_report();
    let args = lang_args(TableFormat::Json);
    let output =
        render_to_string(|buf| write_lang_report_to(buf, &report, &default_scan_options(), &args));

    let parsed: serde_json::Value =
        serde_json::from_str(output.trim()).expect("must parse valid JSON");
    let sv = parsed
        .get("schema_version")
        .expect("should have schema_version");
    assert!(sv.is_number(), "schema_version should be a number");
    assert_eq!(
        sv.as_u64().expect("must be a JSON integer"),
        u64::from(tokmd_types::SCHEMA_VERSION),
        "schema_version should match SCHEMA_VERSION constant"
    );
}

#[test]
fn given_json_output_then_has_metadata() {
    let report = sample_lang_report();
    let args = lang_args(TableFormat::Json);
    let output =
        render_to_string(|buf| write_lang_report_to(buf, &report, &default_scan_options(), &args));

    let parsed: serde_json::Value =
        serde_json::from_str(output.trim()).expect("must parse valid JSON");

    // Required envelope fields
    assert!(
        parsed.get("generated_at_ms").is_some(),
        "should have generated_at_ms"
    );
    assert!(parsed.get("tool").is_some(), "should have tool info");
    assert!(parsed.get("mode").is_some(), "should have mode");
    assert!(parsed.get("status").is_some(), "should have status");
    assert!(parsed.get("scan").is_some(), "should have scan args");
    // report is #[serde(flatten)]ed — rows/total appear at top level
    assert!(
        parsed.get("rows").is_some(),
        "should have rows (flattened from report)"
    );

    // Mode should be "lang"
    assert_eq!(
        parsed["mode"].as_str().expect("must be a JSON string"),
        "lang"
    );

    // generated_at_ms should be a positive number
    let ts = parsed["generated_at_ms"]
        .as_u64()
        .expect("generated_at_ms should be u64");
    assert!(ts > 0, "timestamp should be positive");
}

#[test]
fn given_json_output_then_report_contains_rows() {
    let report = sample_lang_report();
    let args = lang_args(TableFormat::Json);
    let output =
        render_to_string(|buf| write_lang_report_to(buf, &report, &default_scan_options(), &args));

    let parsed: serde_json::Value =
        serde_json::from_str(output.trim()).expect("must parse valid JSON");
    // report is #[serde(flatten)]ed — rows live at top level
    let rows = parsed["rows"].as_array().expect("rows should be an array");
    assert_eq!(rows.len(), 2, "should have 2 language rows");
    assert_eq!(
        rows[0]["lang"].as_str().expect("must be a JSON string"),
        "Rust"
    );
    assert_eq!(
        rows[1]["lang"].as_str().expect("must be a JSON string"),
        "TOML"
    );
}

#[test]
fn given_json_output_then_total_is_present() {
    let report = sample_lang_report();
    let args = lang_args(TableFormat::Json);
    let output =
        render_to_string(|buf| write_lang_report_to(buf, &report, &default_scan_options(), &args));

    let parsed: serde_json::Value =
        serde_json::from_str(output.trim()).expect("must parse valid JSON");
    // report is #[serde(flatten)]ed — total lives at top level
    let total = &parsed["total"];
    assert_eq!(
        total["code"].as_u64().expect("must be a JSON integer"),
        5200
    );
    assert_eq!(
        total["lines"].as_u64().expect("must be a JSON integer"),
        6460
    );
    assert_eq!(total["files"].as_u64().expect("must be a JSON integer"), 47);
}

// ===========================================================================
// Module rendering
// ===========================================================================

#[test]
fn given_module_receipt_when_rendered_then_shows_directory_tree() {
    let report = sample_module_report();
    let args = module_args(TableFormat::Md);
    let output = render_to_string(|buf| {
        write_module_report_to(buf, &report, &default_scan_options(), &args)
    });

    // Module header present
    assert!(
        output.contains("|Module|"),
        "should have Module column header"
    );

    // Each module path is represented
    assert!(
        output.contains("|crates/core|"),
        "should contain crates/core module"
    );
    assert!(
        output.contains("|crates/format|"),
        "should contain crates/format module"
    );
    assert!(
        output.contains("|crates/types|"),
        "should contain crates/types module"
    );

    // Total row
    assert!(
        output.contains("|**Total**|5000|"),
        "total code should be 5000"
    );
}

#[test]
fn given_nested_modules_then_proper_hierarchy() {
    let report = nested_module_report();
    let args = module_args(TableFormat::Md);
    let output = render_to_string(|buf| {
        write_module_report_to(buf, &report, &default_scan_options(), &args)
    });

    // All hierarchy levels present
    assert!(
        output.contains("|src|"),
        "should contain top-level src module"
    );
    assert!(
        output.contains("|src/api|"),
        "should contain src/api nested module"
    );
    assert!(
        output.contains("|src/api/v2|"),
        "should contain deeply nested src/api/v2 module"
    );

    // Rows appear in order (src before src/api before src/api/v2)
    let src_pos = output.find("|src|").expect("operation must succeed");
    let api_pos = output.find("|src/api|").expect("operation must succeed");
    let v2_pos = output.find("|src/api/v2|").expect("operation must succeed");
    assert!(src_pos < api_pos, "src should appear before src/api");
    assert!(api_pos < v2_pos, "src/api should appear before src/api/v2");
}

#[test]
fn given_module_receipt_when_rendered_as_json_then_has_module_metadata() {
    let report = sample_module_report();
    let args = module_args(TableFormat::Json);
    let output = render_to_string(|buf| {
        write_module_report_to(buf, &report, &default_scan_options(), &args)
    });

    let parsed: serde_json::Value =
        serde_json::from_str(output.trim()).expect("must parse valid JSON");
    assert_eq!(
        parsed["mode"].as_str().expect("must be a JSON string"),
        "module"
    );
    // report is #[serde(flatten)]ed — rows live at top level
    assert!(
        parsed.get("rows").is_some(),
        "should have rows (flattened from report)"
    );

    let module_rows = parsed["rows"].as_array().expect("must be a JSON array");
    assert_eq!(module_rows.len(), 3, "should have 3 module rows");
}

#[test]
fn given_module_receipt_when_rendered_as_tsv_then_tab_separated() {
    let report = sample_module_report();
    let args = module_args(TableFormat::Tsv);
    let output = render_to_string(|buf| {
        write_module_report_to(buf, &report, &default_scan_options(), &args)
    });

    let header = output
        .lines()
        .next()
        .expect("output must have at least one line");
    assert!(
        header.starts_with("Module\t"),
        "TSV header should start with Module"
    );

    for line in output.lines() {
        assert_eq!(
            line.matches('\t').count(),
            6,
            "each module TSV line should have 6 tabs"
        );
    }
}

// ===========================================================================
// Export rendering (JSONL)
// ===========================================================================

#[test]
fn given_export_receipt_when_rendered_as_jsonl_then_each_line_valid() {
    let data = sample_export_data();
    let args = export_args(ExportFormat::Jsonl);
    let output =
        render_to_string(|buf| write_export_jsonl_to(buf, &data, &default_scan_options(), &args));

    // Each line should parse as valid JSON
    for (i, line) in output.lines().enumerate() {
        let parsed: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("line {i} should be valid JSON: {e}"));
        assert!(parsed.is_object(), "line {i} should be a JSON object");
    }
}

#[test]
fn given_export_jsonl_with_meta_then_first_line_is_meta() {
    let data = sample_export_data();
    let args = export_args(ExportFormat::Jsonl);
    let output =
        render_to_string(|buf| write_export_jsonl_to(buf, &data, &default_scan_options(), &args));

    let first_line = output
        .lines()
        .next()
        .expect("should have at least one line");
    let parsed: serde_json::Value =
        serde_json::from_str(first_line).expect("operation must succeed");
    assert_eq!(
        parsed["type"].as_str().expect("must be a JSON string"),
        "meta",
        "first JSONL line should be meta"
    );
    assert!(
        parsed.get("schema_version").is_some(),
        "meta should have schema_version"
    );
}

#[test]
fn given_export_jsonl_then_data_rows_have_type_row() {
    let data = sample_export_data();
    let args = export_args(ExportFormat::Jsonl);
    let output =
        render_to_string(|buf| write_export_jsonl_to(buf, &data, &default_scan_options(), &args));

    // Skip meta line, check data rows
    for line in output.lines().skip(1) {
        let parsed: serde_json::Value = serde_json::from_str(line).expect("operation must succeed");
        assert_eq!(
            parsed["type"].as_str().expect("must be a JSON string"),
            "row",
            "non-meta lines should have type=row"
        );
        assert!(parsed.get("path").is_some(), "row should have path");
        assert!(parsed.get("lang").is_some(), "row should have lang");
        assert!(parsed.get("code").is_some(), "row should have code");
    }
}

#[test]
fn given_export_jsonl_then_row_count_matches_data() {
    let data = sample_export_data();
    let args = export_args(ExportFormat::Jsonl);
    let output =
        render_to_string(|buf| write_export_jsonl_to(buf, &data, &default_scan_options(), &args));

    // 1 meta + 3 data rows = 4 lines
    let line_count = output.lines().count();
    assert_eq!(line_count, 4, "should have 1 meta line + 3 data rows");
}

// ===========================================================================
// Export rendering (CSV)
// ===========================================================================

#[test]
fn given_export_receipt_when_rendered_as_csv_then_has_header() {
    let data = sample_export_data();
    let args = export_args(ExportFormat::Csv);
    let output = render_to_string(|buf| write_export_csv_to(buf, &data, &args));

    let header = output.lines().next().expect("should have header line");
    assert!(header.contains("path"), "header should contain 'path'");
    assert!(header.contains("module"), "header should contain 'module'");
    assert!(header.contains("lang"), "header should contain 'lang'");
    assert!(header.contains("code"), "header should contain 'code'");
    assert!(header.contains("lines"), "header should contain 'lines'");
    assert!(header.contains("bytes"), "header should contain 'bytes'");
    assert!(header.contains("tokens"), "header should contain 'tokens'");
}

#[test]
fn given_export_csv_then_data_rows_match_count() {
    let data = sample_export_data();
    let args = export_args(ExportFormat::Csv);
    let output = render_to_string(|buf| write_export_csv_to(buf, &data, &args));

    // header + 3 data rows = 4 lines
    let line_count = output.lines().count();
    assert_eq!(line_count, 4, "should have 1 header + 3 data rows");
}

#[test]
fn given_export_csv_then_columns_are_comma_separated() {
    let data = sample_export_data();
    let args = export_args(ExportFormat::Csv);
    let output = render_to_string(|buf| write_export_csv_to(buf, &data, &args));

    // Each line should contain commas (CSV separator)
    for (i, line) in output.lines().enumerate() {
        assert!(line.contains(','), "line {i} should be comma-separated");
    }
}

#[test]
fn given_export_csv_then_consistent_column_count() {
    let data = sample_export_data();
    let args = export_args(ExportFormat::Csv);
    let output = render_to_string(|buf| write_export_csv_to(buf, &data, &args));

    let expected = output
        .lines()
        .next()
        .expect("output must have at least one line")
        .matches(',')
        .count();
    for (i, line) in output.lines().enumerate() {
        assert_eq!(
            line.matches(',').count(),
            expected,
            "line {i} should have same number of commas as header"
        );
    }
}

// ===========================================================================
// Export rendering (JSON envelope)
// ===========================================================================

#[test]
fn given_export_json_with_meta_then_has_envelope() {
    let data = sample_export_data();
    let args = export_args(ExportFormat::Json);
    let output =
        render_to_string(|buf| write_export_json_to(buf, &data, &default_scan_options(), &args));

    let parsed: serde_json::Value =
        serde_json::from_str(output.trim()).expect("must parse valid JSON");
    assert!(
        parsed.get("schema_version").is_some(),
        "should have schema_version"
    );
    // data is #[serde(flatten)]ed — rows live at top level
    assert!(
        parsed.get("rows").is_some(),
        "should have rows (flattened from data)"
    );
    assert!(parsed.get("scan").is_some(), "should have scan metadata");
}

#[test]
fn given_export_json_then_data_rows_are_array() {
    let data = sample_export_data();
    let args = export_args(ExportFormat::Json);
    let output =
        render_to_string(|buf| write_export_json_to(buf, &data, &default_scan_options(), &args));

    let parsed: serde_json::Value =
        serde_json::from_str(output.trim()).expect("must parse valid JSON");
    // data is #[serde(flatten)]ed — rows live at top level
    let rows = parsed["rows"]
        .as_array()
        .expect("rows should be an array (flattened from data)");
    assert_eq!(rows.len(), 3, "should have 3 file rows");
}
