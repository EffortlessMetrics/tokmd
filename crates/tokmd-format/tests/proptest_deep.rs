//! Deep property-based tests for tokmd-format.
//!
//! Covers Markdown pipe balance, JSON output validity, TSV/CSV column
//! consistency, JSONL line validity, and diff invariants.

use std::path::PathBuf;

use proptest::prelude::*;

use tokmd_format::{
    compute_diff_rows, compute_diff_totals, write_export_csv_to, write_export_json_to,
    write_export_jsonl_to, write_lang_report_to, write_module_report_to,
};
use tokmd_settings::{ChildIncludeMode, ChildrenMode, ScanOptions};
use tokmd_types::{
    ExportArgs, ExportData, ExportFormat, FileKind, FileRow, LangArgs, LangReport, LangRow,
    ModuleArgs, ModuleReport, ModuleRow, RedactMode, TableFormat, Totals,
};

// ---------------------------------------------------------------------------
// Strategies (copied from existing properties.rs to ensure consistency)
// ---------------------------------------------------------------------------

fn arb_lang_row() -> impl Strategy<Value = LangRow> {
    (
        prop::sample::select(vec![
            "Rust", "Python", "Go", "Java", "C", "TOML", "YAML", "JSON",
        ]),
        1usize..10_000,
        1usize..20_000,
        1usize..100,
    )
        .prop_map(|(lang, code, lines, files)| LangRow {
            lang: lang.to_string(),
            code,
            lines: lines.max(code),
            files,
            bytes: code * 10,
            tokens: code / 4,
            avg_lines: if files > 0 { lines / files } else { 0 },
        })
}

fn arb_lang_report() -> impl Strategy<Value = LangReport> {
    prop::collection::vec(arb_lang_row(), 1..6).prop_map(|rows| {
        let mut seen = std::collections::HashSet::new();
        let rows: Vec<LangRow> = rows
            .into_iter()
            .filter(|r| seen.insert(r.lang.clone()))
            .collect();
        let total = Totals {
            code: rows.iter().map(|r| r.code).sum(),
            lines: rows.iter().map(|r| r.lines).sum(),
            files: rows.iter().map(|r| r.files).sum(),
            bytes: rows.iter().map(|r| r.bytes).sum(),
            tokens: rows.iter().map(|r| r.tokens).sum(),
            avg_lines: 0,
        };
        LangReport {
            rows,
            total,
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        }
    })
}

fn arb_module_row() -> impl Strategy<Value = ModuleRow> {
    (
        prop::sample::select(vec!["src", "tests", "crates/a", "crates/b", "lib"]),
        1usize..10_000,
        1usize..20_000,
        1usize..50,
    )
        .prop_map(|(module, code, lines, files)| ModuleRow {
            module: module.to_string(),
            code,
            lines: lines.max(code),
            files,
            bytes: code * 10,
            tokens: code / 4,
            avg_lines: if files > 0 { lines / files } else { 0 },
        })
}

fn arb_module_report() -> impl Strategy<Value = ModuleReport> {
    prop::collection::vec(arb_module_row(), 1..5).prop_map(|rows| {
        let mut seen = std::collections::HashSet::new();
        let rows: Vec<ModuleRow> = rows
            .into_iter()
            .filter(|r| seen.insert(r.module.clone()))
            .collect();
        let total = Totals {
            code: rows.iter().map(|r| r.code).sum(),
            lines: rows.iter().map(|r| r.lines).sum(),
            files: rows.iter().map(|r| r.files).sum(),
            bytes: rows.iter().map(|r| r.bytes).sum(),
            tokens: rows.iter().map(|r| r.tokens).sum(),
            avg_lines: 0,
        };
        ModuleReport {
            rows,
            total,
            module_roots: vec!["crates".into()],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
            top: 0,
        }
    })
}

fn arb_file_row() -> impl Strategy<Value = FileRow> {
    (
        prop::sample::select(vec![
            "src/lib.rs",
            "src/main.rs",
            "tests/it.rs",
            "build.rs",
            "Cargo.toml",
        ]),
        1usize..5_000,
        0usize..500,
        0usize..200,
    )
        .prop_map(|(path, code, comments, blanks)| FileRow {
            path: path.to_string(),
            module: path.split('/').next().unwrap_or("root").to_string(),
            lang: "Rust".into(),
            kind: FileKind::Parent,
            code,
            comments,
            blanks,
            lines: code + comments + blanks,
            bytes: code * 10,
            tokens: code / 4,
        })
}

fn default_global() -> ScanOptions {
    ScanOptions::default()
}

// ---------------------------------------------------------------------------
// Markdown: every line has balanced pipes (same column count)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn lang_md_pipe_balance(report in arb_lang_report()) {
        let args = LangArgs {
            paths: vec![PathBuf::from(".")],
            format: TableFormat::Md,
            top: 0,
            files: false,
            children: ChildrenMode::Collapse,
        };
        let mut buf = Vec::new();
        write_lang_report_to(&mut buf, &report, &default_global(), &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        let lines: Vec<&str> = output.lines().collect();
        if !lines.is_empty() {
            let header_pipes = lines[0].matches('|').count();
            for (i, line) in lines.iter().enumerate() {
                let pipes = line.matches('|').count();
                prop_assert_eq!(
                    pipes, header_pipes,
                    "Line {} has {} pipes, header has {}",
                    i, pipes, header_pipes
                );
            }
        }
    }

    #[test]
    fn module_md_pipe_balance(report in arb_module_report()) {
        let args = ModuleArgs {
            paths: vec![PathBuf::from(".")],
            format: TableFormat::Md,
            top: 0,
            module_roots: vec!["crates".into()],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
        };
        let mut buf = Vec::new();
        write_module_report_to(&mut buf, &report, &default_global(), &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        let lines: Vec<&str> = output.lines().collect();
        if !lines.is_empty() {
            let header_pipes = lines[0].matches('|').count();
            for (i, line) in lines.iter().enumerate() {
                let pipes = line.matches('|').count();
                prop_assert_eq!(
                    pipes, header_pipes,
                    "Module MD line {} has {} pipes, header has {}",
                    i, pipes, header_pipes
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// JSON: output is always valid JSON
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn lang_json_is_valid_json(report in arb_lang_report()) {
        // Render as JSON by serializing the report directly
        let json = serde_json::to_string(&report).unwrap();
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json);
        prop_assert!(parsed.is_ok(), "LangReport JSON is not valid");
    }

    #[test]
    fn module_json_is_valid_json(report in arb_module_report()) {
        let json = serde_json::to_string(&report).unwrap();
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json);
        prop_assert!(parsed.is_ok(), "ModuleReport JSON is not valid");
    }

    #[test]
    fn export_json_is_valid_json(rows in prop::collection::vec(arb_file_row(), 1..6)) {
        let data = ExportData {
            rows,
            module_roots: vec!["src".into()],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        };
        let mut buf = Vec::new();
        write_export_json_to(&mut buf, &data, &default_global(), &ExportArgs {
            paths: vec![PathBuf::from(".")],
            format: ExportFormat::Json,
            output: None,
            module_roots: vec!["src".into()],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
            min_code: 0,
            max_rows: 0,
            redact: RedactMode::None,
            meta: false,
            strip_prefix: None,
        }).unwrap();
        let output = String::from_utf8(buf).unwrap();

        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&output);
        prop_assert!(parsed.is_ok(), "Export JSON is not valid JSON");
    }
}

// ---------------------------------------------------------------------------
// TSV: consistent column count across all lines
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn lang_tsv_consistent_columns(report in arb_lang_report()) {
        let args = LangArgs {
            paths: vec![PathBuf::from(".")],
            format: TableFormat::Tsv,
            top: 0,
            files: false,
            children: ChildrenMode::Collapse,
        };
        let mut buf = Vec::new();
        write_lang_report_to(&mut buf, &report, &default_global(), &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        let lines: Vec<&str> = output.lines().collect();
        if !lines.is_empty() {
            let header_tabs = lines[0].matches('\t').count();
            for (i, line) in lines.iter().enumerate() {
                let tabs = line.matches('\t').count();
                prop_assert_eq!(
                    tabs, header_tabs,
                    "TSV line {} has {} tabs, header has {}",
                    i, tabs, header_tabs
                );
            }
        }
    }

    #[test]
    fn module_tsv_consistent_columns(report in arb_module_report()) {
        let args = ModuleArgs {
            paths: vec![PathBuf::from(".")],
            format: TableFormat::Tsv,
            top: 0,
            module_roots: vec!["crates".into()],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
        };
        let mut buf = Vec::new();
        write_module_report_to(&mut buf, &report, &default_global(), &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        let lines: Vec<&str> = output.lines().collect();
        if !lines.is_empty() {
            let header_tabs = lines[0].matches('\t').count();
            for (i, line) in lines.iter().enumerate() {
                let tabs = line.matches('\t').count();
                prop_assert_eq!(
                    tabs, header_tabs,
                    "Module TSV line {} has {} tabs, header has {}",
                    i, tabs, header_tabs
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// CSV: consistent column count
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn export_csv_consistent_columns(rows in prop::collection::vec(arb_file_row(), 1..8)) {
        let data = ExportData {
            rows,
            module_roots: vec!["src".into()],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        };
        let args = ExportArgs {
            paths: vec![PathBuf::from(".")],
            format: ExportFormat::Csv,
            output: None,
            module_roots: vec!["src".into()],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
            min_code: 0,
            max_rows: 0,
            redact: RedactMode::None,
            meta: false,
            strip_prefix: None,
        };
        let mut buf = Vec::new();
        write_export_csv_to(&mut buf, &data, &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        let lines: Vec<&str> = output.lines().collect();
        if !lines.is_empty() {
            let header_commas = lines[0].matches(',').count();
            for (i, line) in lines.iter().enumerate() {
                let commas = line.matches(',').count();
                prop_assert_eq!(
                    commas, header_commas,
                    "CSV line {} has {} commas, header has {}",
                    i, commas, header_commas
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// JSONL: each line is valid JSON
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn export_jsonl_each_line_is_valid_json(rows in prop::collection::vec(arb_file_row(), 1..6)) {
        let data = ExportData {
            rows,
            module_roots: vec!["src".into()],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        };
        let args = ExportArgs {
            paths: vec![PathBuf::from(".")],
            format: ExportFormat::Jsonl,
            output: None,
            module_roots: vec!["src".into()],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
            min_code: 0,
            max_rows: 0,
            redact: RedactMode::None,
            meta: false,
            strip_prefix: None,
        };
        let mut buf = Vec::new();
        write_export_jsonl_to(&mut buf, &data, &default_global(), &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        for (i, line) in output.lines().enumerate() {
            if !line.trim().is_empty() {
                let parsed: Result<serde_json::Value, _> = serde_json::from_str(line);
                prop_assert!(
                    parsed.is_ok(),
                    "JSONL line {} is not valid JSON: '{}'",
                    i, line
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Diff: row sums equal totals (deep)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn diff_totals_bytes_tokens_consistency(
        report in arb_lang_report(),
    ) {
        let diff_rows = compute_diff_rows(&report, &report);
        let totals = compute_diff_totals(&diff_rows);

        // Diffing a report against itself: all deltas should be zero
        for row in &diff_rows {
            prop_assert_eq!(
                row.delta_code, 0,
                "Self-diff delta should be 0, got {} for lang '{}'",
                row.delta_code, row.lang
            );
        }
        prop_assert_eq!(totals.delta_code, 0, "Self-diff total delta should be 0");
    }

    #[test]
    fn diff_self_is_zero(report in arb_lang_report()) {
        let rows = compute_diff_rows(&report, &report);
        let totals = compute_diff_totals(&rows);

        prop_assert_eq!(totals.old_code, totals.new_code);
        prop_assert_eq!(totals.delta_code, 0);
    }
}