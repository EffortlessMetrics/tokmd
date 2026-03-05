//! Pipeline integration tests for `tokmd-format` (w67).
//!
//! Verifies Markdown, TSV, JSON, and CSV rendering for lang/module/export
//! reports. Covers edge cases, determinism, and envelope structure.

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

fn render_to_string<F>(f: F) -> String
where
    F: FnOnce(&mut Vec<u8>),
{
    let mut buf = Vec::new();
    f(&mut buf);
    String::from_utf8(buf).expect("valid UTF-8")
}

// ===========================================================================
// 1. Markdown output — lang summary
// ===========================================================================

#[test]
fn lang_md_without_files_header() {
    let output = render_to_string(|buf| {
        let report = sample_lang_report(false);
        let args = default_lang_args(TableFormat::Md);
        write_lang_report_to(buf, &report, &default_scan_options(), &args).unwrap();
    });
    assert!(output.contains("|Lang|Code|Lines|Bytes|Tokens|"));
    assert!(!output.contains("|Files|"));
}

#[test]
fn lang_md_with_files_header() {
    let output = render_to_string(|buf| {
        let report = sample_lang_report(true);
        let mut args = default_lang_args(TableFormat::Md);
        args.files = true;
        write_lang_report_to(buf, &report, &default_scan_options(), &args).unwrap();
    });
    assert!(output.contains("|Lang|Code|Lines|Files|Bytes|Tokens|Avg|"));
}

#[test]
fn lang_md_contains_rust_row() {
    let output = render_to_string(|buf| {
        let report = sample_lang_report(false);
        let args = default_lang_args(TableFormat::Md);
        write_lang_report_to(buf, &report, &default_scan_options(), &args).unwrap();
    });
    assert!(output.contains("|Rust|1000|1200|50000|2500|"));
}

#[test]
fn lang_md_contains_total_row() {
    let output = render_to_string(|buf| {
        let report = sample_lang_report(false);
        let args = default_lang_args(TableFormat::Md);
        write_lang_report_to(buf, &report, &default_scan_options(), &args).unwrap();
    });
    assert!(output.contains("|**Total**|1050|1260|51000|2625|"));
}

#[test]
fn lang_md_has_separator_row() {
    let output = render_to_string(|buf| {
        let report = sample_lang_report(false);
        let args = default_lang_args(TableFormat::Md);
        write_lang_report_to(buf, &report, &default_scan_options(), &args).unwrap();
    });
    assert!(output.contains("|---|---:|---:|---:|---:|"));
}

// ===========================================================================
// 2. TSV output format
// ===========================================================================

#[test]
fn lang_tsv_tab_separated_header() {
    let output = render_to_string(|buf| {
        let report = sample_lang_report(false);
        let args = default_lang_args(TableFormat::Tsv);
        write_lang_report_to(buf, &report, &default_scan_options(), &args).unwrap();
    });
    assert!(output.starts_with("Lang\tCode\tLines\tBytes\tTokens\n"));
}

#[test]
fn lang_tsv_rows_use_tabs() {
    let output = render_to_string(|buf| {
        let report = sample_lang_report(false);
        let args = default_lang_args(TableFormat::Tsv);
        write_lang_report_to(buf, &report, &default_scan_options(), &args).unwrap();
    });
    assert!(output.contains("Rust\t1000\t1200\t50000\t2500"));
}

#[test]
fn lang_tsv_with_files_has_extra_columns() {
    let output = render_to_string(|buf| {
        let report = sample_lang_report(true);
        let mut args = default_lang_args(TableFormat::Tsv);
        args.files = true;
        write_lang_report_to(buf, &report, &default_scan_options(), &args).unwrap();
    });
    assert!(output.starts_with("Lang\tCode\tLines\tFiles\tBytes\tTokens\tAvg\n"));
}

#[test]
fn module_tsv_tab_separated() {
    let output = render_to_string(|buf| {
        let report = sample_module_report();
        let args = default_module_args(TableFormat::Tsv);
        write_module_report_to(buf, &report, &default_scan_options(), &args).unwrap();
    });
    assert!(output.starts_with("Module\tCode\tLines\tFiles\tBytes\tTokens\tAvg\n"));
    assert!(output.contains("crates/foo\t800\t950\t8\t40000\t2000\t119"));
}

// ===========================================================================
// 3. JSON output validity
// ===========================================================================

#[test]
fn lang_json_is_valid_json() {
    let output = render_to_string(|buf| {
        let report = sample_lang_report(false);
        let args = default_lang_args(TableFormat::Json);
        write_lang_report_to(buf, &report, &default_scan_options(), &args).unwrap();
    });
    let parsed: serde_json::Value = serde_json::from_str(output.trim()).expect("valid JSON");
    assert!(parsed.is_object());
}

#[test]
fn lang_json_has_schema_version() {
    let output = render_to_string(|buf| {
        let report = sample_lang_report(false);
        let args = default_lang_args(TableFormat::Json);
        write_lang_report_to(buf, &report, &default_scan_options(), &args).unwrap();
    });
    let parsed: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    assert!(parsed["schema_version"].is_number());
}

#[test]
fn lang_json_has_rows_field() {
    let output = render_to_string(|buf| {
        let report = sample_lang_report(false);
        let args = default_lang_args(TableFormat::Json);
        write_lang_report_to(buf, &report, &default_scan_options(), &args).unwrap();
    });
    let parsed: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    // LangReport is #[serde(flatten)]'d so rows appear at top level
    assert!(parsed["rows"].is_array(), "rows should be at top level (flattened)");
}

#[test]
fn module_json_is_valid() {
    let output = render_to_string(|buf| {
        let report = sample_module_report();
        let args = default_module_args(TableFormat::Json);
        write_module_report_to(buf, &report, &default_scan_options(), &args).unwrap();
    });
    let parsed: serde_json::Value = serde_json::from_str(output.trim()).expect("valid JSON");
    // ModuleReport is #[serde(flatten)]'d so rows appear at top level
    assert!(parsed["rows"].is_array());
}

// ===========================================================================
// 4. CSV output
// ===========================================================================

#[test]
fn csv_has_header_row() {
    let output = render_to_string(|buf| {
        let data = sample_export_data();
        let args = default_export_args(ExportFormat::Csv);
        write_export_csv_to(buf, &data, &args).unwrap();
    });
    let first_line = output.lines().next().unwrap();
    assert_eq!(
        first_line,
        "path,module,lang,kind,code,comments,blanks,lines,bytes,tokens"
    );
}

#[test]
fn csv_row_count_matches_data() {
    let output = render_to_string(|buf| {
        let data = sample_export_data();
        let args = default_export_args(ExportFormat::Csv);
        write_export_csv_to(buf, &data, &args).unwrap();
    });
    // header + 2 data rows
    let line_count = output.lines().count();
    assert_eq!(line_count, 3, "header + 2 rows");
}

#[test]
fn csv_contains_file_paths() {
    let output = render_to_string(|buf| {
        let data = sample_export_data();
        let args = default_export_args(ExportFormat::Csv);
        write_export_csv_to(buf, &data, &args).unwrap();
    });
    assert!(output.contains("src/lib.rs"));
    assert!(output.contains("tests/test.rs"));
}

// ===========================================================================
// 5. JSONL export
// ===========================================================================

#[test]
fn jsonl_each_line_is_valid_json() {
    let output = render_to_string(|buf| {
        let data = sample_export_data();
        let args = default_export_args(ExportFormat::Jsonl);
        write_export_jsonl_to(buf, &data, &default_scan_options(), &args).unwrap();
    });
    for line in output.lines() {
        let _: serde_json::Value = serde_json::from_str(line).expect("each line must be JSON");
    }
}

#[test]
fn jsonl_first_line_is_meta() {
    let output = render_to_string(|buf| {
        let data = sample_export_data();
        let args = default_export_args(ExportFormat::Jsonl);
        write_export_jsonl_to(buf, &data, &default_scan_options(), &args).unwrap();
    });
    let first: serde_json::Value = serde_json::from_str(output.lines().next().unwrap()).unwrap();
    assert_eq!(first["type"], "meta");
}

#[test]
fn jsonl_data_rows_have_type_row() {
    let output = render_to_string(|buf| {
        let data = sample_export_data();
        let args = default_export_args(ExportFormat::Jsonl);
        write_export_jsonl_to(buf, &data, &default_scan_options(), &args).unwrap();
    });
    for line in output.lines().skip(1) {
        let v: serde_json::Value = serde_json::from_str(line).unwrap();
        assert_eq!(v["type"], "row");
    }
}

// ===========================================================================
// 6. JSON export
// ===========================================================================

#[test]
fn json_export_is_valid() {
    let output = render_to_string(|buf| {
        let data = sample_export_data();
        let args = default_export_args(ExportFormat::Json);
        write_export_json_to(buf, &data, &default_scan_options(), &args).unwrap();
    });
    let parsed: serde_json::Value = serde_json::from_str(output.trim()).expect("valid JSON");
    // ExportData is #[serde(flatten)]'d so rows appear at top level
    assert!(parsed["rows"].is_array());
}

#[test]
fn json_export_has_schema_version() {
    let output = render_to_string(|buf| {
        let data = sample_export_data();
        let args = default_export_args(ExportFormat::Json);
        write_export_json_to(buf, &data, &default_scan_options(), &args).unwrap();
    });
    let parsed: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    assert!(parsed["schema_version"].is_number());
}

// ===========================================================================
// 7. Edge cases
// ===========================================================================

#[test]
fn empty_lang_report_renders_without_panic() {
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
    let output = render_to_string(|buf| {
        let args = default_lang_args(TableFormat::Md);
        write_lang_report_to(buf, &report, &default_scan_options(), &args).unwrap();
    });
    assert!(output.contains("|**Total**|0|0|0|0|"));
}

#[test]
fn single_row_lang_report() {
    let report = LangReport {
        rows: vec![LangRow {
            lang: "Go".into(),
            code: 42,
            lines: 50,
            files: 1,
            bytes: 200,
            tokens: 50,
            avg_lines: 50,
        }],
        total: Totals {
            code: 42,
            lines: 50,
            files: 1,
            bytes: 200,
            tokens: 50,
            avg_lines: 50,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let output = render_to_string(|buf| {
        let args = default_lang_args(TableFormat::Md);
        write_lang_report_to(buf, &report, &default_scan_options(), &args).unwrap();
    });
    assert!(output.contains("|Go|42|50|200|50|"));
}

#[test]
fn empty_export_csv() {
    let data = ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    };
    let output = render_to_string(|buf| {
        let args = default_export_args(ExportFormat::Csv);
        write_export_csv_to(buf, &data, &args).unwrap();
    });
    // Only header line
    assert_eq!(output.lines().count(), 1);
}

#[test]
fn empty_module_report_renders() {
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
    let output = render_to_string(|buf| {
        let args = default_module_args(TableFormat::Md);
        write_module_report_to(buf, &report, &default_scan_options(), &args).unwrap();
    });
    assert!(output.contains("|**Total**|0|0|0|0|0|0|"));
}

// ===========================================================================
// 8. Unicode in language names
// ===========================================================================

#[test]
fn unicode_lang_name_in_md() {
    let report = LangReport {
        rows: vec![LangRow {
            lang: "C++".into(),
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
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let output = render_to_string(|buf| {
        let args = default_lang_args(TableFormat::Md);
        write_lang_report_to(buf, &report, &default_scan_options(), &args).unwrap();
    });
    assert!(output.contains("|C++|100|120|5000|250|"));
}

#[test]
fn special_chars_in_module_name() {
    let report = ModuleReport {
        rows: vec![ModuleRow {
            module: "crates/my-lib".into(),
            code: 50,
            lines: 60,
            files: 1,
            bytes: 2000,
            tokens: 100,
            avg_lines: 60,
        }],
        total: Totals {
            code: 50,
            lines: 60,
            files: 1,
            bytes: 2000,
            tokens: 100,
            avg_lines: 60,
        },
        module_roots: vec!["crates".into()],
        module_depth: 2,
        children: ChildIncludeMode::ParentsOnly,
        top: 0,
    };
    let output = render_to_string(|buf| {
        let args = default_module_args(TableFormat::Tsv);
        write_module_report_to(buf, &report, &default_scan_options(), &args).unwrap();
    });
    assert!(output.contains("crates/my-lib\t50\t60\t1\t2000\t100\t60"));
}

// ===========================================================================
// 9. Output determinism
// ===========================================================================

#[test]
fn lang_md_deterministic() {
    let report = sample_lang_report(false);
    let global = default_scan_options();
    let args = default_lang_args(TableFormat::Md);
    let out1 = render_to_string(|buf| {
        write_lang_report_to(buf, &report, &global, &args).unwrap();
    });
    let out2 = render_to_string(|buf| {
        write_lang_report_to(buf, &report, &global, &args).unwrap();
    });
    assert_eq!(out1, out2, "same input must produce identical output");
}

#[test]
fn lang_tsv_deterministic() {
    let report = sample_lang_report(true);
    let global = default_scan_options();
    let args = default_lang_args(TableFormat::Tsv);
    let out1 = render_to_string(|buf| {
        write_lang_report_to(buf, &report, &global, &args).unwrap();
    });
    let out2 = render_to_string(|buf| {
        write_lang_report_to(buf, &report, &global, &args).unwrap();
    });
    assert_eq!(out1, out2);
}

#[test]
fn csv_deterministic() {
    let data = sample_export_data();
    let args = default_export_args(ExportFormat::Csv);
    let out1 = render_to_string(|buf| {
        write_export_csv_to(buf, &data, &args).unwrap();
    });
    let out2 = render_to_string(|buf| {
        write_export_csv_to(buf, &data, &args).unwrap();
    });
    assert_eq!(out1, out2);
}

#[test]
fn module_md_deterministic() {
    let report = sample_module_report();
    let global = default_scan_options();
    let args = default_module_args(TableFormat::Md);
    let out1 = render_to_string(|buf| {
        write_module_report_to(buf, &report, &global, &args).unwrap();
    });
    let out2 = render_to_string(|buf| {
        write_module_report_to(buf, &report, &global, &args).unwrap();
    });
    assert_eq!(out1, out2);
}

// ===========================================================================
// 10. Module markdown output
// ===========================================================================

#[test]
fn module_md_has_correct_header() {
    let output = render_to_string(|buf| {
        let report = sample_module_report();
        let args = default_module_args(TableFormat::Md);
        write_module_report_to(buf, &report, &default_scan_options(), &args).unwrap();
    });
    assert!(output.contains("|Module|Code|Lines|Files|Bytes|Tokens|Avg|"));
}

#[test]
fn module_md_contains_data_rows() {
    let output = render_to_string(|buf| {
        let report = sample_module_report();
        let args = default_module_args(TableFormat::Md);
        write_module_report_to(buf, &report, &default_scan_options(), &args).unwrap();
    });
    assert!(output.contains("|crates/foo|800|950|8|40000|2000|119|"));
    assert!(output.contains("|crates/bar|200|250|2|10000|500|125|"));
}
