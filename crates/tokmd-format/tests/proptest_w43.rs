//! Wave 43 property-based tests for tokmd-format.
//!
//! Covers: Markdown header invariants, TSV column consistency, JSON validity,
//! rendering determinism, JSONL line validity, CSV structure, diff symmetry,
//! and export format preservation.

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

// =========================================================================
// Strategies
// =========================================================================

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
            avg_lines: lines.checked_div(files).unwrap_or(0),
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
            avg_lines: lines.checked_div(files).unwrap_or(0),
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

fn default_lang_args(fmt: TableFormat) -> LangArgs {
    LangArgs {
        paths: vec![PathBuf::from(".")],
        format: fmt,
        top: 0,
        files: false,
        children: ChildrenMode::Collapse,
    }
}

fn default_module_args(fmt: TableFormat) -> ModuleArgs {
    ModuleArgs {
        paths: vec![PathBuf::from(".")],
        format: fmt,
        top: 0,
        module_roots: vec!["crates".into()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

fn default_export_args(fmt: ExportFormat) -> ExportArgs {
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
        meta: false,
        strip_prefix: None,
    }
}

// =========================================================================
// 1. Markdown output always starts with expected header (pipe character)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn lang_md_starts_with_pipe(report in arb_lang_report()) {
        let mut buf = Vec::new();
        write_lang_report_to(&mut buf, &report, &default_global(), &default_lang_args(TableFormat::Md)).unwrap();
        let output = String::from_utf8(buf).unwrap();
        prop_assert!(output.starts_with('|'), "Lang MD should start with pipe");
    }

    #[test]
    fn module_md_starts_with_pipe(report in arb_module_report()) {
        let mut buf = Vec::new();
        write_module_report_to(&mut buf, &report, &default_global(), &default_module_args(TableFormat::Md)).unwrap();
        let output = String::from_utf8(buf).unwrap();
        prop_assert!(output.starts_with('|'), "Module MD should start with pipe");
    }
}

// =========================================================================
// 2. TSV output has consistent column count across rows
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn lang_tsv_consistent_columns(report in arb_lang_report()) {
        let mut buf = Vec::new();
        write_lang_report_to(&mut buf, &report, &default_global(), &default_lang_args(TableFormat::Tsv)).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        if !lines.is_empty() {
            let header_tabs = lines[0].matches('\t').count();
            for (i, line) in lines.iter().enumerate() {
                let tabs = line.matches('\t').count();
                prop_assert_eq!(tabs, header_tabs, "TSV line {} has {} tabs, header has {}", i, tabs, header_tabs);
            }
        }
    }

    #[test]
    fn module_tsv_consistent_columns(report in arb_module_report()) {
        let mut buf = Vec::new();
        write_module_report_to(&mut buf, &report, &default_global(), &default_module_args(TableFormat::Tsv)).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        if !lines.is_empty() {
            let header_tabs = lines[0].matches('\t').count();
            for (i, line) in lines.iter().enumerate() {
                let tabs = line.matches('\t').count();
                prop_assert_eq!(tabs, header_tabs, "Module TSV line {} has {} tabs, header has {}", i, tabs, header_tabs);
            }
        }
    }
}

// =========================================================================
// 3. JSON output is always valid JSON
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn lang_json_always_valid(report in arb_lang_report()) {
        let mut buf = Vec::new();
        write_lang_report_to(&mut buf, &report, &default_global(), &default_lang_args(TableFormat::Json)).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(output.trim());
        prop_assert!(parsed.is_ok(), "Lang JSON must be valid: {:?}", parsed.err());
    }

    #[test]
    fn module_json_always_valid(report in arb_module_report()) {
        let mut buf = Vec::new();
        write_module_report_to(&mut buf, &report, &default_global(), &default_module_args(TableFormat::Json)).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(output.trim());
        prop_assert!(parsed.is_ok(), "Module JSON must be valid: {:?}", parsed.err());
    }

    #[test]
    fn export_json_always_valid(rows in prop::collection::vec(arb_file_row(), 1..6)) {
        let data = ExportData {
            rows,
            module_roots: vec!["src".into()],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        };
        let mut buf = Vec::new();
        write_export_json_to(&mut buf, &data, &default_global(), &default_export_args(ExportFormat::Json)).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&output);
        prop_assert!(parsed.is_ok(), "Export JSON must be valid: {:?}", parsed.err());
    }
}

// =========================================================================
// 4. Rendering same data twice produces identical output (determinism)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn lang_md_deterministic(report in arb_lang_report()) {
        let args = default_lang_args(TableFormat::Md);
        let render = |r: &LangReport| -> Vec<u8> {
            let mut buf = Vec::new();
            write_lang_report_to(&mut buf, r, &default_global(), &args).unwrap();
            buf
        };
        prop_assert_eq!(render(&report), render(&report));
    }

    #[test]
    fn lang_tsv_deterministic(report in arb_lang_report()) {
        let args = default_lang_args(TableFormat::Tsv);
        let render = |r: &LangReport| -> Vec<u8> {
            let mut buf = Vec::new();
            write_lang_report_to(&mut buf, r, &default_global(), &args).unwrap();
            buf
        };
        prop_assert_eq!(render(&report), render(&report));
    }

    #[test]
    fn module_md_deterministic(report in arb_module_report()) {
        let args = default_module_args(TableFormat::Md);
        let render = |r: &ModuleReport| -> Vec<u8> {
            let mut buf = Vec::new();
            write_module_report_to(&mut buf, r, &default_global(), &args).unwrap();
            buf
        };
        prop_assert_eq!(render(&report), render(&report));
    }

    #[test]
    fn module_tsv_deterministic(report in arb_module_report()) {
        let args = default_module_args(TableFormat::Tsv);
        let render = |r: &ModuleReport| -> Vec<u8> {
            let mut buf = Vec::new();
            write_module_report_to(&mut buf, r, &default_global(), &args).unwrap();
            buf
        };
        prop_assert_eq!(render(&report), render(&report));
    }

    #[test]
    fn export_csv_deterministic(rows in prop::collection::vec(arb_file_row(), 1..6)) {
        let data = ExportData {
            rows,
            module_roots: vec!["src".into()],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        };
        let args = default_export_args(ExportFormat::Csv);
        let render = |d: &ExportData| -> Vec<u8> {
            let mut buf = Vec::new();
            write_export_csv_to(&mut buf, d, &args).unwrap();
            buf
        };
        prop_assert_eq!(render(&data), render(&data));
    }
}

// =========================================================================
// 5. Markdown pipe balance (all lines have same pipe count)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn lang_md_pipe_balance(report in arb_lang_report()) {
        let mut buf = Vec::new();
        write_lang_report_to(&mut buf, &report, &default_global(), &default_lang_args(TableFormat::Md)).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        if !lines.is_empty() {
            let expected = lines[0].matches('|').count();
            for (i, line) in lines.iter().enumerate() {
                prop_assert_eq!(line.matches('|').count(), expected, "Line {} pipe mismatch", i);
            }
        }
    }
}

// =========================================================================
// 6. Markdown line count: header + separator + rows + total
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn lang_md_line_count(report in arb_lang_report()) {
        let mut buf = Vec::new();
        write_lang_report_to(&mut buf, &report, &default_global(), &default_lang_args(TableFormat::Md)).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let expected = report.rows.len() + 3; // header + separator + total
        prop_assert_eq!(output.lines().count(), expected);
    }
}

// =========================================================================
// 7. JSONL: every line is valid JSON
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn export_jsonl_lines_valid(rows in prop::collection::vec(arb_file_row(), 1..6)) {
        let data = ExportData {
            rows,
            module_roots: vec!["src".into()],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        };
        let mut buf = Vec::new();
        write_export_jsonl_to(&mut buf, &data, &default_global(), &default_export_args(ExportFormat::Jsonl)).unwrap();
        let output = String::from_utf8(buf).unwrap();
        for (i, line) in output.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(line);
            prop_assert!(parsed.is_ok(), "JSONL line {} is not valid JSON: {}", i, line);
        }
    }
}

// =========================================================================
// 8. CSV: header + data rows line count
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn export_csv_line_count(rows in prop::collection::vec(arb_file_row(), 1..8)) {
        let n = rows.len();
        let data = ExportData {
            rows,
            module_roots: vec!["src".into()],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        };
        let mut buf = Vec::new();
        write_export_csv_to(&mut buf, &data, &default_export_args(ExportFormat::Csv)).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        prop_assert_eq!(lines.len(), n + 1, "CSV should have header + {} data rows", n);
    }
}

// =========================================================================
// 9. Diff: self-diff produces zero deltas
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn diff_self_zero_deltas(report in arb_lang_report()) {
        let rows = compute_diff_rows(&report, &report);
        for row in &rows {
            prop_assert_eq!(row.delta_code, 0);
            prop_assert_eq!(row.delta_lines, 0);
            prop_assert_eq!(row.delta_files, 0);
        }
        let totals = compute_diff_totals(&rows);
        prop_assert_eq!(totals.delta_code, 0);
    }
}

// =========================================================================
// 10. Diff: totals equal row sums
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn diff_totals_equal_row_sums(
        from in arb_lang_report(),
        to in arb_lang_report(),
    ) {
        let rows = compute_diff_rows(&from, &to);
        let totals = compute_diff_totals(&rows);
        let sum_delta_code: i64 = rows.iter().map(|r| r.delta_code).sum();
        let sum_delta_lines: i64 = rows.iter().map(|r| r.delta_lines).sum();
        prop_assert_eq!(totals.delta_code, sum_delta_code);
        prop_assert_eq!(totals.delta_lines, sum_delta_lines);
    }
}

// =========================================================================
// 11. Diff: anti-symmetry (diff(a,b) = -diff(b,a))
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn diff_anti_symmetric(
        from in arb_lang_report(),
        to in arb_lang_report(),
    ) {
        let totals_ab = compute_diff_totals(&compute_diff_rows(&from, &to));
        let totals_ba = compute_diff_totals(&compute_diff_rows(&to, &from));
        prop_assert_eq!(totals_ab.delta_code, -totals_ba.delta_code);
        prop_assert_eq!(totals_ab.delta_lines, -totals_ba.delta_lines);
        prop_assert_eq!(totals_ab.delta_files, -totals_ba.delta_files);
    }
}

// =========================================================================
// 12. Markdown contains separator line with dashes
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn lang_md_contains_separator(report in arb_lang_report()) {
        let mut buf = Vec::new();
        write_lang_report_to(&mut buf, &report, &default_global(), &default_lang_args(TableFormat::Md)).unwrap();
        let output = String::from_utf8(buf).unwrap();
        prop_assert!(output.contains("---"), "Markdown should contain separator dashes");
    }

    #[test]
    fn module_md_contains_separator(report in arb_module_report()) {
        let mut buf = Vec::new();
        write_module_report_to(&mut buf, &report, &default_global(), &default_module_args(TableFormat::Md)).unwrap();
        let output = String::from_utf8(buf).unwrap();
        prop_assert!(output.contains("---"), "Module MD should contain separator dashes");
    }
}

// =========================================================================
// 13. Export JSON preserves all paths
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn export_json_preserves_paths(file_rows in prop::collection::vec(arb_file_row(), 1..6)) {
        let mut seen = std::collections::HashSet::new();
        let rows: Vec<FileRow> = file_rows
            .into_iter()
            .filter(|r| seen.insert(r.path.clone()))
            .collect();
        let data = ExportData {
            rows: rows.clone(),
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        };
        let mut buf = Vec::new();
        write_export_json_to(&mut buf, &data, &default_global(), &ExportArgs {
            paths: vec![PathBuf::from(".")],
            format: ExportFormat::Json,
            output: None,
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
            min_code: 0,
            max_rows: 10_000,
            redact: RedactMode::None,
            meta: false,
            strip_prefix: None,
        }).unwrap();
        let output = String::from_utf8(buf).unwrap();
        for row in &rows {
            prop_assert!(output.contains(&row.path), "missing path {}", row.path);
        }
    }
}

// =========================================================================
// 14. JSONL determinism
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn export_jsonl_deterministic(rows in prop::collection::vec(arb_file_row(), 1..6)) {
        let data = ExportData {
            rows,
            module_roots: vec!["src".into()],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        };
        let args = default_export_args(ExportFormat::Jsonl);
        let render = |d: &ExportData| -> Vec<u8> {
            let mut buf = Vec::new();
            write_export_jsonl_to(&mut buf, d, &default_global(), &args).unwrap();
            buf
        };
        prop_assert_eq!(render(&data), render(&data));
    }
}
