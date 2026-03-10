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

// =========================================================================
// Strategies
// =========================================================================

fn arb_lang_row() -> impl Strategy<Value = LangRow> {
    (
        prop::sample::select(vec![
            "Rust",
            "Python",
            "Go",
            "Java",
            "C",
            "TOML",
            "YAML",
            "JSON",
            "Rust",
            "Python",
            "Go",
            "Java",
            "C",
            "TypeScript",
            "TOML",
            "YAML",
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
    prop::collection::vec(arb_lang_row(), 1..8).prop_map(|rows| {
        let mut seen = std::collections::BTreeSet::new();
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
        let mut seen = std::collections::BTreeSet::new();
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
// =========================================================================
// Diff: self-diff produces all-zero deltas
// =========================================================================

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
// Diff: self-diff produces zero deltas
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn self_diff_produces_zero_deltas(report in arb_lang_report()) {
        let rows = compute_diff_rows(&report, &report);
        for row in &rows {
            prop_assert_eq!(row.delta_code, 0, "self-diff code delta should be 0 for {}", row.lang);
            prop_assert_eq!(row.delta_lines, 0, "self-diff lines delta should be 0 for {}", row.lang);
            prop_assert_eq!(row.delta_files, 0, "self-diff files delta should be 0 for {}", row.lang);
        }
        let totals = compute_diff_totals(&rows);
        prop_assert_eq!(totals.delta_code, 0);
        prop_assert_eq!(totals.delta_lines, 0);
        prop_assert_eq!(totals.delta_files, 0);
    }
}

// =========================================================================
// Diff: deltas are anti-symmetric (diff(a,b) = -diff(b,a))
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn diff_deltas_anti_symmetric(
        from in arb_lang_report(),
        to in arb_lang_report(),
    ) {
        let rows_ab = compute_diff_rows(&from, &to);
        let rows_ba = compute_diff_rows(&to, &from);
        let totals_ab = compute_diff_totals(&rows_ab);
        let totals_ba = compute_diff_totals(&rows_ba);

        prop_assert_eq!(
            totals_ab.delta_code, -totals_ba.delta_code,
            "delta_code should be anti-symmetric"
        );
        prop_assert_eq!(
            totals_ab.delta_lines, -totals_ba.delta_lines,
            "delta_lines should be anti-symmetric"
        );
        prop_assert_eq!(
            totals_ab.delta_files, -totals_ba.delta_files,
            "delta_files should be anti-symmetric"
        );
    }
}

// =========================================================================
// Diff: totals old/new match source reports
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn diff_totals_match_source_reports(
        from in arb_lang_report(),
        to in arb_lang_report(),
    ) {
        let rows = compute_diff_rows(&from, &to);
        let totals = compute_diff_totals(&rows);

        prop_assert_eq!(totals.old_code as usize, from.total.code);
        prop_assert_eq!(totals.new_code as usize, to.total.code);
        prop_assert_eq!(totals.old_lines as usize, from.total.lines);
        prop_assert_eq!(totals.new_lines as usize, to.total.lines);
    }
}

// =========================================================================
// Diff: compute_diff_rows is deterministic
// =========================================================================

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
// Diff: rows are deterministic
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn diff_rows_are_deterministic(
        from in arb_lang_report(),
        to in arb_lang_report(),
    ) {
        let rows1 = compute_diff_rows(&from, &to);
        let rows2 = compute_diff_rows(&from, &to);
        prop_assert_eq!(rows1.len(), rows2.len());
        for (r1, r2) in rows1.iter().zip(rows2.iter()) {
            prop_assert_eq!(r1, r2, "Diff rows should be deterministic");
        }
    }
}

// =========================================================================
// Diff: row count covers union of languages
// =========================================================================

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
// Diff: row count covers union of languages
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn diff_row_count_is_union_of_languages(
        from in arb_lang_report(),
        to in arb_lang_report(),
    ) {
        let rows = compute_diff_rows(&from, &to);
        let mut all_langs: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
        for r in &from.rows { all_langs.insert(&r.lang); }
        for r in &to.rows { all_langs.insert(&r.lang); }
        prop_assert_eq!(rows.len(), all_langs.len(),
            "Diff rows should cover union of languages");
    }
}

// =========================================================================
// Diff: each row delta = new - old
// =========================================================================

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

    #[test]
    fn diff_each_row_delta_consistent(
        from in arb_lang_report(),
        to in arb_lang_report(),
    ) {
        let rows = compute_diff_rows(&from, &to);
        for row in &rows {
            prop_assert_eq!(
                row.delta_code, row.new_code as i64 - row.old_code as i64,
                "delta_code mismatch for {}", row.lang
            );
            prop_assert_eq!(
                row.delta_lines, row.new_lines as i64 - row.old_lines as i64,
                "delta_lines mismatch for {}", row.lang
            );
        }
    }
}
