//! Error handling and edge case tests for tokmd-format.

use std::path::PathBuf;
use tokmd_format::{
    compute_diff_rows, compute_diff_totals, create_diff_receipt, render_diff_md,
    write_lang_report_to, write_module_report_to,
};
use tokmd_settings::{ChildIncludeMode, ChildrenMode, ScanOptions};
use tokmd_types::{
    ConfigMode, DiffRow, LangArgs, LangReport, LangRow, ModuleArgs, ModuleReport, ModuleRow,
    TableFormat, Totals,
};

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

// ── Empty receipt formatting ───────────────────────────────────────

#[test]
fn format_empty_lang_report_md() {
    let report = LangReport {
        rows: vec![],
        total: zero_totals(),
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let mut buf = Vec::new();
    let result = write_lang_report_to(
        &mut buf,
        &report,
        &default_scan_options(),
        &default_lang_args(),
    );
    assert!(result.is_ok());
    let output = String::from_utf8(buf).unwrap();
    // Should produce a valid table with header + totals even with no rows
    assert!(output.contains("Total"));
}

#[test]
fn format_empty_lang_report_tsv() {
    let report = LangReport {
        rows: vec![],
        total: zero_totals(),
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let args = LangArgs {
        format: TableFormat::Tsv,
        ..default_lang_args()
    };
    let mut buf = Vec::new();
    let result = write_lang_report_to(&mut buf, &report, &default_scan_options(), &args);
    assert!(result.is_ok());
}

#[test]
fn format_empty_lang_report_json() {
    let report = LangReport {
        rows: vec![],
        total: zero_totals(),
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let args = LangArgs {
        format: TableFormat::Json,
        ..default_lang_args()
    };
    let mut buf = Vec::new();
    let result = write_lang_report_to(&mut buf, &report, &default_scan_options(), &args);
    assert!(result.is_ok());
    let output = String::from_utf8(buf).unwrap();
    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(parsed.is_object());
}

#[test]
fn format_empty_module_report_md() {
    let report = ModuleReport {
        rows: vec![],
        total: zero_totals(),
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        top: 0,
    };
    let mut buf = Vec::new();
    let result = write_module_report_to(
        &mut buf,
        &report,
        &default_scan_options(),
        &default_module_args(),
    );
    assert!(result.is_ok());
}

// ── Receipt with zero languages ────────────────────────────────────

#[test]
fn diff_rows_from_two_empty_reports() {
    let from = LangReport {
        rows: vec![],
        total: zero_totals(),
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let to = from.clone();
    let rows = compute_diff_rows(&from, &to);
    assert!(rows.is_empty());
}

#[test]
fn diff_totals_from_empty_rows() {
    let totals = compute_diff_totals(&[]);
    assert_eq!(totals.delta_code, 0);
    assert_eq!(totals.delta_lines, 0);
    assert_eq!(totals.delta_files, 0);
}

#[test]
fn render_diff_md_with_no_changes() {
    let totals = compute_diff_totals(&[]);
    let output = render_diff_md("v1", "v2", &[], &totals);
    assert!(output.contains("v1"));
    assert!(output.contains("v2"));
}

// ── Receipt with very long language names ──────────────────────────

#[test]
fn format_lang_report_with_very_long_name() {
    let long_name = "A".repeat(500);
    let report = LangReport {
        rows: vec![LangRow {
            lang: long_name.clone(),
            code: 100,
            lines: 200,
            files: 5,
            bytes: 5000,
            tokens: 400,
            avg_lines: 40,
        }],
        total: Totals {
            code: 100,
            lines: 200,
            files: 5,
            bytes: 5000,
            tokens: 400,
            avg_lines: 40,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let mut buf = Vec::new();
    let result = write_lang_report_to(
        &mut buf,
        &report,
        &default_scan_options(),
        &default_lang_args(),
    );
    assert!(result.is_ok());
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains(&long_name));
}

#[test]
fn format_module_report_with_very_long_module_name() {
    let long_module = "src/".repeat(100);
    let report = ModuleReport {
        rows: vec![ModuleRow {
            module: long_module.clone(),
            code: 50,
            lines: 100,
            files: 2,
            bytes: 2000,
            tokens: 200,
            avg_lines: 50,
        }],
        total: Totals {
            code: 50,
            lines: 100,
            files: 2,
            bytes: 2000,
            tokens: 200,
            avg_lines: 50,
        },
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        top: 0,
    };
    let mut buf = Vec::new();
    let result = write_module_report_to(
        &mut buf,
        &report,
        &default_scan_options(),
        &default_module_args(),
    );
    assert!(result.is_ok());
}

// ── Receipt with unicode in names ──────────────────────────────────

#[test]
fn format_lang_report_with_unicode_name() {
    let report = LangReport {
        rows: vec![LangRow {
            lang: "日本語コード".to_string(),
            code: 10,
            lines: 20,
            files: 1,
            bytes: 500,
            tokens: 40,
            avg_lines: 20,
        }],
        total: Totals {
            code: 10,
            lines: 20,
            files: 1,
            bytes: 500,
            tokens: 40,
            avg_lines: 20,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let mut buf = Vec::new();
    let result = write_lang_report_to(
        &mut buf,
        &report,
        &default_scan_options(),
        &default_lang_args(),
    );
    assert!(result.is_ok());
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("日本語コード"));
}

#[test]
fn format_lang_report_with_emoji_name() {
    let report = LangReport {
        rows: vec![LangRow {
            lang: "🦀 Rust".to_string(),
            code: 42,
            lines: 84,
            files: 3,
            bytes: 2000,
            tokens: 168,
            avg_lines: 28,
        }],
        total: Totals {
            code: 42,
            lines: 84,
            files: 3,
            bytes: 2000,
            tokens: 168,
            avg_lines: 28,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let mut buf = Vec::new();
    let result = write_lang_report_to(
        &mut buf,
        &report,
        &default_scan_options(),
        &default_lang_args(),
    );
    assert!(result.is_ok());
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("🦀 Rust"));
}

#[test]
fn diff_with_unicode_language_names() {
    let from = LangReport {
        rows: vec![LangRow {
            lang: "C++".to_string(),
            code: 100,
            lines: 200,
            files: 5,
            bytes: 5000,
            tokens: 400,
            avg_lines: 40,
        }],
        total: Totals {
            code: 100,
            lines: 200,
            files: 5,
            bytes: 5000,
            tokens: 400,
            avg_lines: 40,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let to = LangReport {
        rows: vec![LangRow {
            lang: "C++".to_string(),
            code: 150,
            lines: 300,
            files: 7,
            bytes: 7500,
            tokens: 600,
            avg_lines: 43,
        }],
        total: Totals {
            code: 150,
            lines: 300,
            files: 7,
            bytes: 7500,
            tokens: 600,
            avg_lines: 43,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let rows = compute_diff_rows(&from, &to);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].lang, "C++");
    assert_eq!(rows[0].delta_code, 50);
}

// ── Receipt with max values ────────────────────────────────────────

#[test]
fn format_lang_report_with_large_values() {
    let big = usize::MAX / 2;
    let report = LangReport {
        rows: vec![LangRow {
            lang: "BigLang".to_string(),
            code: big,
            lines: big,
            files: big,
            bytes: big,
            tokens: big,
            avg_lines: big,
        }],
        total: Totals {
            code: big,
            lines: big,
            files: big,
            bytes: big,
            tokens: big,
            avg_lines: big,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let mut buf = Vec::new();
    let result = write_lang_report_to(
        &mut buf,
        &report,
        &default_scan_options(),
        &default_lang_args(),
    );
    assert!(result.is_ok());
}

#[test]
fn format_lang_report_json_with_large_values() {
    let big = usize::MAX / 2;
    let report = LangReport {
        rows: vec![LangRow {
            lang: "BigLang".to_string(),
            code: big,
            lines: big,
            files: big,
            bytes: big,
            tokens: big,
            avg_lines: big,
        }],
        total: Totals {
            code: big,
            lines: big,
            files: big,
            bytes: big,
            tokens: big,
            avg_lines: big,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let args = LangArgs {
        format: TableFormat::Json,
        ..default_lang_args()
    };
    let mut buf = Vec::new();
    let result = write_lang_report_to(&mut buf, &report, &default_scan_options(), &args);
    assert!(result.is_ok());
    let output = String::from_utf8(buf).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(parsed.is_object());
}

// ── Diff edge cases ────────────────────────────────────────────────

#[test]
fn diff_new_language_added() {
    let from = LangReport {
        rows: vec![],
        total: zero_totals(),
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let to = LangReport {
        rows: vec![LangRow {
            lang: "Rust".to_string(),
            code: 100,
            lines: 200,
            files: 5,
            bytes: 5000,
            tokens: 400,
            avg_lines: 40,
        }],
        total: Totals {
            code: 100,
            lines: 200,
            files: 5,
            bytes: 5000,
            tokens: 400,
            avg_lines: 40,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let rows = compute_diff_rows(&from, &to);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].old_code, 0);
    assert_eq!(rows[0].new_code, 100);
    assert_eq!(rows[0].delta_code, 100);
}

#[test]
fn diff_language_removed() {
    let from = LangReport {
        rows: vec![LangRow {
            lang: "Python".to_string(),
            code: 200,
            lines: 400,
            files: 10,
            bytes: 10000,
            tokens: 800,
            avg_lines: 40,
        }],
        total: Totals {
            code: 200,
            lines: 400,
            files: 10,
            bytes: 10000,
            tokens: 800,
            avg_lines: 40,
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
    assert_eq!(rows[0].old_code, 200);
    assert_eq!(rows[0].new_code, 0);
    assert_eq!(rows[0].delta_code, -200);
}

#[test]
fn diff_identical_reports_yields_no_rows() {
    let report = LangReport {
        rows: vec![LangRow {
            lang: "Go".to_string(),
            code: 50,
            lines: 100,
            files: 3,
            bytes: 2500,
            tokens: 200,
            avg_lines: 33,
        }],
        total: Totals {
            code: 50,
            lines: 100,
            files: 3,
            bytes: 2500,
            tokens: 200,
            avg_lines: 33,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let rows = compute_diff_rows(&report, &report);
    assert!(
        rows.is_empty(),
        "identical reports should produce no diff rows"
    );
}

#[test]
fn create_diff_receipt_has_correct_sources() {
    let totals = compute_diff_totals(&[]);
    let receipt = create_diff_receipt("before", "after", vec![], totals);
    assert_eq!(receipt.from_source, "before");
    assert_eq!(receipt.to_source, "after");
}

// ── Single row edge cases ──────────────────────────────────────────

#[test]
fn format_single_language_report() {
    let report = LangReport {
        rows: vec![LangRow {
            lang: "Rust".to_string(),
            code: 1,
            lines: 1,
            files: 1,
            bytes: 10,
            tokens: 4,
            avg_lines: 1,
        }],
        total: Totals {
            code: 1,
            lines: 1,
            files: 1,
            bytes: 10,
            tokens: 4,
            avg_lines: 1,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let mut buf = Vec::new();
    let result = write_lang_report_to(
        &mut buf,
        &report,
        &default_scan_options(),
        &default_lang_args(),
    );
    assert!(result.is_ok());
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("Rust"));
}

#[test]
fn format_multiple_languages_report() {
    let report = LangReport {
        rows: vec![
            LangRow {
                lang: "Rust".to_string(),
                code: 500,
                lines: 1000,
                files: 10,
                bytes: 25000,
                tokens: 2000,
                avg_lines: 100,
            },
            LangRow {
                lang: "Python".to_string(),
                code: 300,
                lines: 600,
                files: 5,
                bytes: 15000,
                tokens: 1200,
                avg_lines: 120,
            },
            LangRow {
                lang: "TOML".to_string(),
                code: 50,
                lines: 100,
                files: 3,
                bytes: 2500,
                tokens: 200,
                avg_lines: 33,
            },
        ],
        total: Totals {
            code: 850,
            lines: 1700,
            files: 18,
            bytes: 42500,
            tokens: 3400,
            avg_lines: 94,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let mut buf = Vec::new();
    let result = write_lang_report_to(
        &mut buf,
        &report,
        &default_scan_options(),
        &default_lang_args(),
    );
    assert!(result.is_ok());
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("Rust"));
    assert!(output.contains("Python"));
    assert!(output.contains("TOML"));
}

// ── Special characters in names ────────────────────────────────────

#[test]
fn format_lang_report_with_special_chars() {
    let report = LangReport {
        rows: vec![
            LangRow {
                lang: "C#".to_string(),
                code: 100,
                lines: 200,
                files: 5,
                bytes: 5000,
                tokens: 400,
                avg_lines: 40,
            },
            LangRow {
                lang: "Objective-C++".to_string(),
                code: 50,
                lines: 100,
                files: 2,
                bytes: 2500,
                tokens: 200,
                avg_lines: 50,
            },
        ],
        total: Totals {
            code: 150,
            lines: 300,
            files: 7,
            bytes: 7500,
            tokens: 600,
            avg_lines: 43,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let mut buf = Vec::new();
    let result = write_lang_report_to(
        &mut buf,
        &report,
        &default_scan_options(),
        &default_lang_args(),
    );
    assert!(result.is_ok());
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("C#"));
    assert!(output.contains("Objective-C++"));
}

#[test]
fn diff_row_with_negative_deltas() {
    let row = DiffRow {
        lang: "Rust".to_string(),
        old_code: 200,
        new_code: 100,
        delta_code: -100,
        old_lines: 400,
        new_lines: 200,
        delta_lines: -200,
        old_files: 10,
        new_files: 5,
        delta_files: -5,
        old_bytes: 10000,
        new_bytes: 5000,
        delta_bytes: -5000,
        old_tokens: 800,
        new_tokens: 400,
        delta_tokens: -400,
    };
    let totals = compute_diff_totals(&[row]);
    assert_eq!(totals.delta_code, -100);
    assert_eq!(totals.delta_files, -5);
}
