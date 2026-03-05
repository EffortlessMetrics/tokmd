//! W52 – Format roundtrip and cross-format consistency tests.
//!
//! Verifies:
//! 1. JSON serialisation round-trips (serialize → deserialize → equality).
//! 2. Cross-format consistency (row counts, totals, ordering).
//! 3. Format-specific structural invariants (Markdown tables, TSV lines, CSV headers, JSONL lines).

use std::path::PathBuf;

use tokmd_format::{
    scan_args, write_export_csv_to, write_export_json_to, write_export_jsonl_to,
    write_lang_report_to, write_module_report_to,
};
use tokmd_settings::ScanOptions;
use tokmd_types::{
    ChildIncludeMode, ChildrenMode, ExportArgs, ExportData, ExportFormat, ExportReceipt, FileKind,
    FileRow, LangArgs, LangReceipt, LangReport, LangRow, ModuleArgs, ModuleReceipt, ModuleReport,
    ModuleRow, RedactMode, SCHEMA_VERSION, ScanStatus, TableFormat, ToolInfo, Totals,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn sample_lang_report() -> LangReport {
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
        with_files: true,
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
        module_roots: vec!["src".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn default_scan_options() -> ScanOptions {
    ScanOptions::default()
}

fn make_lang_receipt(report: &LangReport) -> LangReceipt {
    LangReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: ToolInfo {
            name: "tokmd".into(),
            version: "0.0.0-test".into(),
        },
        mode: "lang".into(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: scan_args(&[PathBuf::from(".")], &default_scan_options(), None),
        args: tokmd_types::LangArgsMeta {
            format: "json".into(),
            top: report.top,
            with_files: report.with_files,
            children: report.children,
        },
        report: report.clone(),
    }
}

fn make_module_receipt(report: &ModuleReport) -> ModuleReceipt {
    ModuleReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: ToolInfo {
            name: "tokmd".into(),
            version: "0.0.0-test".into(),
        },
        mode: "module".into(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: scan_args(&[PathBuf::from(".")], &default_scan_options(), None),
        args: tokmd_types::ModuleArgsMeta {
            format: "json".into(),
            top: report.top,
            module_roots: report.module_roots.clone(),
            module_depth: report.module_depth,
            children: report.children,
        },
        report: report.clone(),
    }
}

fn make_export_receipt(data: &ExportData) -> ExportReceipt {
    ExportReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: ToolInfo {
            name: "tokmd".into(),
            version: "0.0.0-test".into(),
        },
        mode: "export".into(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: scan_args(&[PathBuf::from(".")], &default_scan_options(), None),
        args: tokmd_types::ExportArgsMeta {
            format: ExportFormat::Json,
            module_roots: data.module_roots.clone(),
            module_depth: data.module_depth,
            children: data.children,
            min_code: 0,
            max_rows: 0,
            redact: RedactMode::None,
            strip_prefix: None,
            strip_prefix_redacted: false,
        },
        data: data.clone(),
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

fn module_args(fmt: TableFormat) -> ModuleArgs {
    ModuleArgs {
        paths: vec![PathBuf::from(".")],
        format: fmt,
        top: 0,
        module_roots: vec!["crates".into()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

fn export_args(fmt: ExportFormat) -> ExportArgs {
    ExportArgs {
        paths: vec![PathBuf::from(".")],
        format: fmt,
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

fn render_lang(fmt: TableFormat) -> String {
    let report = sample_lang_report();
    let global = default_scan_options();
    let args = lang_args(fmt);
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &global, &args).unwrap();
    String::from_utf8(buf).unwrap()
}

fn render_module(fmt: TableFormat) -> String {
    let report = sample_module_report();
    let global = default_scan_options();
    let args = module_args(fmt);
    let mut buf = Vec::new();
    write_module_report_to(&mut buf, &report, &global, &args).unwrap();
    String::from_utf8(buf).unwrap()
}

// ========================================================================
// 1. JSON roundtrip tests
// ========================================================================

#[test]
fn json_roundtrip_lang_receipt() {
    let receipt = make_lang_receipt(&sample_lang_report());
    let json = serde_json::to_string(&receipt).unwrap();
    let back: LangReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, receipt.schema_version);
    assert_eq!(back.mode, receipt.mode);
    assert_eq!(back.report.rows.len(), receipt.report.rows.len());
    assert_eq!(back.report.total, receipt.report.total);
    for (a, b) in back.report.rows.iter().zip(&receipt.report.rows) {
        assert_eq!(a, b);
    }
}

#[test]
fn json_roundtrip_module_receipt() {
    let receipt = make_module_receipt(&sample_module_report());
    let json = serde_json::to_string(&receipt).unwrap();
    let back: ModuleReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, receipt.schema_version);
    assert_eq!(back.mode, receipt.mode);
    assert_eq!(back.report.rows.len(), receipt.report.rows.len());
    assert_eq!(back.report.total, receipt.report.total);
    for (a, b) in back.report.rows.iter().zip(&receipt.report.rows) {
        assert_eq!(a, b);
    }
}

#[test]
fn json_roundtrip_export_receipt() {
    let receipt = make_export_receipt(&sample_export_data());
    let json = serde_json::to_string(&receipt).unwrap();
    let back: ExportReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, receipt.schema_version);
    assert_eq!(back.mode, receipt.mode);
    assert_eq!(back.data.rows.len(), receipt.data.rows.len());
    for (a, b) in back.data.rows.iter().zip(&receipt.data.rows) {
        assert_eq!(a, b);
    }
}

#[test]
fn json_roundtrip_empty_rows() {
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
    let receipt = make_lang_receipt(&report);
    let json = serde_json::to_string(&receipt).unwrap();
    let back: LangReceipt = serde_json::from_str(&json).unwrap();
    assert!(back.report.rows.is_empty());
    assert_eq!(back.report.total.code, 0);
}

#[test]
fn json_roundtrip_max_field_values() {
    let report = LangReport {
        rows: vec![LangRow {
            lang: "MaxLang".into(),
            code: usize::MAX,
            lines: usize::MAX,
            files: usize::MAX,
            bytes: usize::MAX,
            tokens: usize::MAX,
            avg_lines: usize::MAX,
        }],
        total: Totals {
            code: usize::MAX,
            lines: usize::MAX,
            files: usize::MAX,
            bytes: usize::MAX,
            tokens: usize::MAX,
            avg_lines: usize::MAX,
        },
        with_files: true,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let receipt = make_lang_receipt(&report);
    let json = serde_json::to_string(&receipt).unwrap();
    let back: LangReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.report.rows[0].code, usize::MAX);
    assert_eq!(back.report.total.code, usize::MAX);
}

#[test]
fn json_roundtrip_special_characters_in_lang() {
    let report = LangReport {
        rows: vec![LangRow {
            lang: "C++ (embedded)".into(),
            code: 42,
            lines: 50,
            files: 1,
            bytes: 200,
            tokens: 100,
            avg_lines: 50,
        }],
        total: Totals {
            code: 42,
            lines: 50,
            files: 1,
            bytes: 200,
            tokens: 100,
            avg_lines: 50,
        },
        with_files: true,
        children: ChildrenMode::Separate,
        top: 0,
    };
    let receipt = make_lang_receipt(&report);
    let json = serde_json::to_string(&receipt).unwrap();
    let back: LangReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.report.rows[0].lang, "C++ (embedded)");
}

#[test]
fn json_deterministic_output() {
    let receipt = make_lang_receipt(&sample_lang_report());
    let json1 = serde_json::to_string(&receipt).unwrap();
    let json2 = serde_json::to_string(&receipt).unwrap();
    assert_eq!(
        json1, json2,
        "same input must produce identical JSON strings"
    );
}

#[test]
fn json_nested_structure_correctness() {
    let receipt = make_lang_receipt(&sample_lang_report());
    let val: serde_json::Value = serde_json::to_value(&receipt).unwrap();

    assert_eq!(val["schema_version"], SCHEMA_VERSION);
    assert_eq!(val["mode"], "lang");
    assert!(val["rows"].is_array());
    assert_eq!(val["rows"].as_array().unwrap().len(), 2);
    assert_eq!(val["rows"][0]["lang"], "Rust");
    assert!(val["total"].is_object());
    assert_eq!(val["total"]["code"], 1050);
    assert!(val["scan"].is_object());
    assert!(val["args"].is_object());
}

// ========================================================================
// 2. Cross-format consistency tests
// ========================================================================

#[test]
fn lang_row_count_consistent_across_formats() {
    let report = sample_lang_report();
    let n = report.rows.len();

    let md = render_lang(TableFormat::Md);
    let tsv = render_lang(TableFormat::Tsv);
    let json_str = render_lang(TableFormat::Json);

    // Markdown: header + separator + N data rows + totals
    let md_data_rows = md.lines().count() - 3; // header, separator, total
    assert_eq!(md_data_rows, n);

    // TSV: header + N data rows + totals
    let tsv_data_rows = tsv.lines().count() - 2; // header, total
    assert_eq!(tsv_data_rows, n);

    // JSON: parse and count rows array
    let val: serde_json::Value = serde_json::from_str(json_str.trim()).unwrap();
    assert_eq!(val["rows"].as_array().unwrap().len(), n);
}

#[test]
fn module_row_count_consistent_across_formats() {
    let report = sample_module_report();
    let n = report.rows.len();

    let md = render_module(TableFormat::Md);
    let tsv = render_module(TableFormat::Tsv);
    let json_str = render_module(TableFormat::Json);

    let md_data_rows = md.lines().count() - 3;
    assert_eq!(md_data_rows, n);

    let tsv_data_rows = tsv.lines().count() - 2;
    assert_eq!(tsv_data_rows, n);

    let val: serde_json::Value = serde_json::from_str(json_str.trim()).unwrap();
    assert_eq!(val["rows"].as_array().unwrap().len(), n);
}

#[test]
fn total_code_lines_consistent_across_formats() {
    let report = sample_lang_report();
    let expected_code = report.total.code;

    // Markdown – totals row
    let md = render_lang(TableFormat::Md);
    let total_line = md.lines().last().unwrap();
    let total_code_md: usize = total_line.split('|').nth(2).unwrap().parse().unwrap();
    assert_eq!(total_code_md, expected_code);

    // TSV – totals row
    let tsv = render_lang(TableFormat::Tsv);
    let total_line = tsv.lines().last().unwrap();
    let total_code_tsv: usize = total_line.split('\t').nth(1).unwrap().parse().unwrap();
    assert_eq!(total_code_tsv, expected_code);

    // JSON
    let json_str = render_lang(TableFormat::Json);
    let val: serde_json::Value = serde_json::from_str(json_str.trim()).unwrap();
    assert_eq!(
        val["total"]["code"].as_u64().unwrap() as usize,
        expected_code
    );
}

#[test]
fn language_ordering_consistent_across_formats() {
    let md = render_lang(TableFormat::Md);
    let tsv = render_lang(TableFormat::Tsv);
    let json_str = render_lang(TableFormat::Json);

    // Extract language names from Markdown (skip header, separator, totals)
    let md_langs: Vec<&str> = md
        .lines()
        .skip(2)
        .filter(|l| !l.contains("**Total**"))
        .map(|l| l.split('|').nth(1).unwrap())
        .collect();

    // Extract from TSV (skip header, totals)
    let tsv_langs: Vec<&str> = tsv
        .lines()
        .skip(1)
        .filter(|l| !l.starts_with("Total\t"))
        .map(|l| l.split('\t').next().unwrap())
        .collect();

    // Extract from JSON
    let val: serde_json::Value = serde_json::from_str(json_str.trim()).unwrap();
    let json_langs: Vec<String> = val["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["lang"].as_str().unwrap().to_string())
        .collect();

    assert_eq!(md_langs, tsv_langs);
    assert_eq!(
        md_langs.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        json_langs
    );
}

#[test]
fn top_n_produces_same_count_in_all_formats() {
    // Build a report with 3 rows but top=1 to simulate limited output.
    // The format functions use the rows as-is (top is metadata, not a filter
    // applied during rendering), so we pre-slice to simulate the CLI behaviour.
    let mut report = sample_lang_report();
    report.top = 1;
    report.rows.truncate(1);

    let global = default_scan_options();

    let mut md_buf = Vec::new();
    let md_args = lang_args(TableFormat::Md);
    write_lang_report_to(&mut md_buf, &report, &global, &md_args).unwrap();
    let md = String::from_utf8(md_buf).unwrap();

    let mut tsv_buf = Vec::new();
    let tsv_args = lang_args(TableFormat::Tsv);
    write_lang_report_to(&mut tsv_buf, &report, &global, &tsv_args).unwrap();
    let tsv = String::from_utf8(tsv_buf).unwrap();

    let mut json_buf = Vec::new();
    let json_args = lang_args(TableFormat::Json);
    write_lang_report_to(&mut json_buf, &report, &global, &json_args).unwrap();
    let json_str = String::from_utf8(json_buf).unwrap();

    let md_data = md.lines().count() - 3; // header, sep, total
    let tsv_data = tsv.lines().count() - 2; // header, total
    let val: serde_json::Value = serde_json::from_str(json_str.trim()).unwrap();
    let json_data = val["rows"].as_array().unwrap().len();

    assert_eq!(md_data, 1);
    assert_eq!(tsv_data, 1);
    assert_eq!(json_data, 1);
}

#[test]
fn module_total_code_consistent_across_formats() {
    let report = sample_module_report();
    let expected = report.total.code;

    let md = render_module(TableFormat::Md);
    let total_line = md.lines().last().unwrap();
    let md_code: usize = total_line.split('|').nth(2).unwrap().parse().unwrap();
    assert_eq!(md_code, expected);

    let tsv = render_module(TableFormat::Tsv);
    let total_line = tsv.lines().last().unwrap();
    let tsv_code: usize = total_line.split('\t').nth(1).unwrap().parse().unwrap();
    assert_eq!(tsv_code, expected);

    let json_str = render_module(TableFormat::Json);
    let val: serde_json::Value = serde_json::from_str(json_str.trim()).unwrap();
    assert_eq!(val["total"]["code"].as_u64().unwrap() as usize, expected);
}

// ========================================================================
// 3. Format-specific invariants
// ========================================================================

#[test]
fn markdown_contains_table_markers() {
    let md = render_lang(TableFormat::Md);
    assert!(md.contains('|'), "Markdown must contain pipe characters");
    assert!(md.contains("---"), "Markdown must contain separator dashes");
}

#[test]
fn tsv_line_count_equals_header_plus_data() {
    let report = sample_lang_report();
    let n = report.rows.len();
    let tsv = render_lang(TableFormat::Tsv);
    // header + N data rows + 1 totals row = N + 2
    assert_eq!(tsv.lines().count(), n + 2);
}

#[test]
fn csv_header_has_expected_columns() {
    let data = sample_export_data();
    let args = export_args(ExportFormat::Csv);
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &data, &args).unwrap();
    let csv = String::from_utf8(buf).unwrap();
    let header = csv.lines().next().unwrap();
    for col in [
        "path", "module", "lang", "kind", "code", "comments", "blanks", "lines", "bytes", "tokens",
    ] {
        assert!(header.contains(col), "CSV header missing column: {col}");
    }
}

#[test]
fn jsonl_each_line_is_valid_json() {
    let data = sample_export_data();
    let global = default_scan_options();
    let args = export_args(ExportFormat::Jsonl);
    let mut buf = Vec::new();
    write_export_jsonl_to(&mut buf, &data, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();

    for (i, line) in output.lines().enumerate() {
        assert!(
            serde_json::from_str::<serde_json::Value>(line).is_ok(),
            "JSONL line {i} is not valid JSON: {line}"
        );
    }
}

#[test]
fn json_export_is_valid_with_schema_version() {
    let data = sample_export_data();
    let global = default_scan_options();
    let args = export_args(ExportFormat::Json);
    let mut buf = Vec::new();
    write_export_json_to(&mut buf, &data, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();

    let val: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    assert!(val.is_object(), "JSON export must be an object");
    assert!(
        val.get("schema_version").is_some(),
        "JSON export must have schema_version"
    );
    assert_eq!(
        val["schema_version"].as_u64().unwrap() as u32,
        SCHEMA_VERSION
    );
}

#[test]
fn markdown_totals_row_is_last() {
    let md = render_lang(TableFormat::Md);
    let last = md.lines().last().unwrap();
    assert!(
        last.contains("**Total**"),
        "last Markdown row must be the totals row"
    );
}

#[test]
fn csv_line_count_equals_header_plus_data() {
    let data = sample_export_data();
    let n = data.rows.len();
    let args = export_args(ExportFormat::Csv);
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &data, &args).unwrap();
    let csv = String::from_utf8(buf).unwrap();
    // CSV: header + N data rows
    let line_count = csv.lines().count();
    assert_eq!(
        line_count,
        n + 1,
        "CSV must have header + {n} data rows, got {line_count}"
    );
}

#[test]
fn jsonl_line_count_matches_meta_plus_data() {
    let data = sample_export_data();
    let global = default_scan_options();
    let args = export_args(ExportFormat::Jsonl);
    let mut buf = Vec::new();
    write_export_jsonl_to(&mut buf, &data, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();

    // With meta=true: 1 meta line + N data lines
    let n = data.rows.len();
    assert_eq!(output.lines().count(), n + 1);

    // First line should be meta
    let first: serde_json::Value = serde_json::from_str(output.lines().next().unwrap()).unwrap();
    assert_eq!(first["type"], "meta");
}

#[test]
fn module_markdown_totals_row_is_last() {
    let md = render_module(TableFormat::Md);
    let last = md.lines().last().unwrap();
    assert!(
        last.contains("**Total**"),
        "last module Markdown row must be the totals row"
    );
}

#[test]
fn json_lang_output_is_single_line() {
    let json_str = render_lang(TableFormat::Json);
    assert_eq!(
        json_str.trim().lines().count(),
        1,
        "JSON lang output should be a single compact line"
    );
}
