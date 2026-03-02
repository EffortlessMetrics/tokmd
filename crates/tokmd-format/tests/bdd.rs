//! BDD-style scenario tests for tokmd-format.
//!
//! Each test follows Given/When/Then structure covering:
//! - Language report rendering (Markdown, TSV, JSON)
//! - Module report rendering
//! - Diff computation and rendering
//! - Export output (CSV, JSONL, JSON, CycloneDX)

use std::path::PathBuf;

use tokmd_format::{
    compute_diff_rows, compute_diff_totals, render_diff_md, write_export_csv_to,
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

fn default_scan_options() -> ScanOptions {
    ScanOptions::default()
}

fn sample_lang_report(with_files: bool) -> LangReport {
    LangReport {
        rows: vec![
            LangRow {
                lang: "Rust".to_string(),
                code: 500,
                lines: 600,
                files: 5,
                bytes: 25000,
                tokens: 1250,
                avg_lines: 120,
            },
            LangRow {
                lang: "Python".to_string(),
                code: 200,
                lines: 250,
                files: 3,
                bytes: 10000,
                tokens: 500,
                avg_lines: 83,
            },
        ],
        total: Totals {
            code: 700,
            lines: 850,
            files: 8,
            bytes: 35000,
            tokens: 1750,
            avg_lines: 106,
        },
        with_files,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

fn sample_module_report() -> ModuleReport {
    ModuleReport {
        rows: vec![ModuleRow {
            module: "src".to_string(),
            code: 400,
            lines: 500,
            files: 4,
            bytes: 20000,
            tokens: 1000,
            avg_lines: 125,
        }],
        total: Totals {
            code: 400,
            lines: 500,
            files: 4,
            bytes: 20000,
            tokens: 1000,
            avg_lines: 125,
        },
        module_roots: vec!["src".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
        top: 0,
    }
}

fn sample_export_data() -> ExportData {
    ExportData {
        rows: vec![
            FileRow {
                path: "src/lib.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 100,
                comments: 20,
                blanks: 10,
                lines: 130,
                bytes: 5000,
                tokens: 250,
            },
            FileRow {
                path: "tests/test.rs".to_string(),
                module: "tests".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 50,
                comments: 5,
                blanks: 5,
                lines: 60,
                bytes: 2000,
                tokens: 100,
            },
        ],
        module_roots: vec!["src".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

fn default_export_args() -> ExportArgs {
    ExportArgs {
        paths: vec![PathBuf::from(".")],
        format: ExportFormat::Csv,
        output: None,
        module_roots: vec![],
        module_depth: 2,
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

// ============================================================================
// Scenario: Language report — Markdown rendering
// ============================================================================

mod given_lang_report_without_files {
    use super::*;

    #[test]
    fn when_rendered_as_markdown_then_output_contains_header_without_files_column() {
        let report = sample_lang_report(false);
        let args = LangArgs {
            paths: vec![PathBuf::from(".")],
            format: TableFormat::Md,
            top: 0,
            files: false,
            children: ChildrenMode::Collapse,
        };
        let mut buf = Vec::new();
        write_lang_report_to(&mut buf, &report, &default_scan_options(), &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("|Lang|Code|Lines|Bytes|Tokens|"));
        assert!(!output.contains("|Files|"));
        assert!(output.contains("|Rust|500|600|25000|1250|"));
        assert!(output.contains("|Python|200|250|10000|500|"));
        assert!(output.contains("|**Total**|700|850|35000|1750|"));
    }
}

mod given_lang_report_with_files {
    use super::*;

    #[test]
    fn when_rendered_as_markdown_then_output_contains_files_and_avg_columns() {
        let report = sample_lang_report(true);
        let args = LangArgs {
            paths: vec![PathBuf::from(".")],
            format: TableFormat::Md,
            top: 0,
            files: true,
            children: ChildrenMode::Collapse,
        };
        let mut buf = Vec::new();
        write_lang_report_to(&mut buf, &report, &default_scan_options(), &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("|Lang|Code|Lines|Files|Bytes|Tokens|Avg|"));
        assert!(output.contains("|Rust|500|600|5|25000|1250|120|"));
    }

    #[test]
    fn when_rendered_as_tsv_then_tab_separated_values_present() {
        let report = sample_lang_report(true);
        let args = LangArgs {
            paths: vec![PathBuf::from(".")],
            format: TableFormat::Tsv,
            top: 0,
            files: true,
            children: ChildrenMode::Collapse,
        };
        let mut buf = Vec::new();
        write_lang_report_to(&mut buf, &report, &default_scan_options(), &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("Lang\tCode\tLines\tFiles\tBytes\tTokens\tAvg"));
        assert!(output.contains("Rust\t500\t600\t5\t25000\t1250\t120"));
    }

    #[test]
    fn when_rendered_as_json_then_output_is_valid_json_with_schema_version() {
        let report = sample_lang_report(true);
        let args = LangArgs {
            paths: vec![PathBuf::from(".")],
            format: TableFormat::Json,
            top: 0,
            files: true,
            children: ChildrenMode::Collapse,
        };
        let mut buf = Vec::new();
        write_lang_report_to(&mut buf, &report, &default_scan_options(), &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        let v: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
        assert!(v.get("schema_version").is_some());
        assert_eq!(v["mode"], "lang");
        // report is #[serde(flatten)], so rows appear at top level
        assert!(v.get("rows").is_some(), "JSON receipt should have rows field (flattened)");
    }
}

// ============================================================================
// Scenario: Module report rendering
// ============================================================================

mod given_module_report {
    use super::*;

    #[test]
    fn when_rendered_as_markdown_then_module_column_present() {
        let report = sample_module_report();
        let args = ModuleArgs {
            paths: vec![PathBuf::from(".")],
            format: TableFormat::Md,
            top: 0,
            module_roots: vec!["src".to_string()],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
        };
        let mut buf = Vec::new();
        write_module_report_to(&mut buf, &report, &default_scan_options(), &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("|Module|Code|Lines|Files|Bytes|Tokens|Avg|"));
        assert!(output.contains("|src|400|500|4|20000|1000|125|"));
        assert!(output.contains("|**Total**|"));
    }

    #[test]
    fn when_rendered_as_json_then_mode_is_module() {
        let report = sample_module_report();
        let args = ModuleArgs {
            paths: vec![PathBuf::from(".")],
            format: TableFormat::Json,
            top: 0,
            module_roots: vec!["src".to_string()],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
        };
        let mut buf = Vec::new();
        write_module_report_to(&mut buf, &report, &default_scan_options(), &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        let v: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(v["mode"], "module");
    }
}

// ============================================================================
// Scenario: Diff computation
// ============================================================================

mod given_two_lang_reports {
    use super::*;

    #[test]
    fn when_diff_computed_then_delta_reflects_changes() {
        let from = LangReport {
            rows: vec![LangRow {
                lang: "Rust".to_string(),
                code: 100,
                lines: 120,
                files: 2,
                bytes: 5000,
                tokens: 250,
                avg_lines: 60,
            }],
            total: Totals {
                code: 100,
                lines: 120,
                files: 2,
                bytes: 5000,
                tokens: 250,
                avg_lines: 60,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };

        let to = LangReport {
            rows: vec![LangRow {
                lang: "Rust".to_string(),
                code: 150,
                lines: 180,
                files: 3,
                bytes: 7500,
                tokens: 375,
                avg_lines: 60,
            }],
            total: Totals {
                code: 150,
                lines: 180,
                files: 3,
                bytes: 7500,
                tokens: 375,
                avg_lines: 60,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };

        let rows = compute_diff_rows(&from, &to);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].delta_code, 50);
        assert_eq!(rows[0].delta_lines, 60);
        assert_eq!(rows[0].delta_files, 1);
    }

    #[test]
    fn when_new_language_added_then_old_values_are_zero() {
        let from = LangReport {
            rows: vec![],
            total: zero_totals(),
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };

        let to = LangReport {
            rows: vec![LangRow {
                lang: "Go".to_string(),
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
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };

        let rows = compute_diff_rows(&from, &to);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].old_code, 0);
        assert_eq!(rows[0].new_code, 50);
        assert_eq!(rows[0].delta_code, 50);
    }

    #[test]
    fn when_language_removed_then_new_values_are_zero() {
        let from = LangReport {
            rows: vec![LangRow {
                lang: "C".to_string(),
                code: 80,
                lines: 100,
                files: 2,
                bytes: 4000,
                tokens: 200,
                avg_lines: 50,
            }],
            total: Totals {
                code: 80,
                lines: 100,
                files: 2,
                bytes: 4000,
                tokens: 200,
                avg_lines: 50,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };

        let to = LangReport {
            rows: vec![],
            total: zero_totals(),
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };

        let rows = compute_diff_rows(&from, &to);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].new_code, 0);
        assert_eq!(rows[0].delta_code, -80);
    }

    #[test]
    fn when_no_changes_then_diff_rows_empty() {
        let report = sample_lang_report(false);
        let rows = compute_diff_rows(&report, &report);
        assert!(rows.is_empty(), "identical reports produce no diff rows");
    }
}

mod given_diff_rows {
    use super::*;
    use tokmd_types::DiffRow;

    #[test]
    fn when_totals_computed_then_sums_are_correct() {
        let rows = vec![
            DiffRow {
                lang: "Rust".to_string(),
                old_code: 100,
                new_code: 150,
                delta_code: 50,
                old_lines: 120,
                new_lines: 180,
                delta_lines: 60,
                old_files: 2,
                new_files: 3,
                delta_files: 1,
                old_bytes: 5000,
                new_bytes: 7500,
                delta_bytes: 2500,
                old_tokens: 250,
                new_tokens: 375,
                delta_tokens: 125,
            },
            DiffRow {
                lang: "Python".to_string(),
                old_code: 50,
                new_code: 30,
                delta_code: -20,
                old_lines: 60,
                new_lines: 35,
                delta_lines: -25,
                old_files: 1,
                new_files: 1,
                delta_files: 0,
                old_bytes: 2000,
                new_bytes: 1200,
                delta_bytes: -800,
                old_tokens: 100,
                new_tokens: 60,
                delta_tokens: -40,
            },
        ];

        let totals = compute_diff_totals(&rows);
        assert_eq!(totals.delta_code, 30); // 50 + (-20)
        assert_eq!(totals.delta_lines, 35); // 60 + (-25)
        assert_eq!(totals.old_code, 150); // 100 + 50
        assert_eq!(totals.new_code, 180); // 150 + 30
    }

    #[test]
    fn when_empty_rows_then_totals_are_zero() {
        let totals = compute_diff_totals(&[]);
        assert_eq!(totals.delta_code, 0);
        assert_eq!(totals.old_code, 0);
        assert_eq!(totals.new_code, 0);
    }

    #[test]
    fn when_rendered_as_markdown_then_contains_table_structure() {
        let from = sample_lang_report(false);
        let mut to = sample_lang_report(false);
        to.rows[0].code = 600; // increase Rust code
        to.total.code = 800;

        let rows = compute_diff_rows(&from, &to);
        let totals = compute_diff_totals(&rows);
        let md = render_diff_md("v1", "v2", &rows, &totals);

        assert!(md.contains("|Language|"), "diff table should have Language header");
        assert!(md.contains("Rust"));
        assert!(md.contains("v1"));
        assert!(md.contains("v2"));
    }
}

// ============================================================================
// Scenario: Export — CSV format
// ============================================================================

mod given_export_data {
    use super::*;

    #[test]
    fn when_written_as_csv_then_header_and_rows_present() {
        let export = sample_export_data();
        let args = default_export_args();
        let mut buf = Vec::new();
        write_export_csv_to(&mut buf, &export, &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.starts_with("path,module,lang,kind,code,comments,blanks,lines,bytes,tokens"));
        assert!(output.contains("src/lib.rs"));
        assert!(output.contains("tests/test.rs"));
    }

    #[test]
    fn when_written_as_csv_with_empty_data_then_only_header() {
        let export = ExportData {
            rows: vec![],
            module_roots: vec![],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
        };
        let args = default_export_args();
        let mut buf = Vec::new();
        write_export_csv_to(&mut buf, &export, &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 1, "empty export should have only header");
    }

    #[test]
    fn when_written_as_cyclonedx_then_valid_json_with_bom_format() {
        let export = sample_export_data();
        let mut buf = Vec::new();
        tokmd_format::write_export_cyclonedx_with_options(
            &mut buf,
            &export,
            RedactMode::None,
            Some("urn:uuid:test-123".to_string()),
            Some("2024-01-01T00:00:00Z".to_string()),
        )
        .unwrap();
        let output = String::from_utf8(buf).unwrap();

        let v: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(v["bomFormat"], "CycloneDX");
        assert!(v["components"].as_array().unwrap().len() > 0);
    }
}
