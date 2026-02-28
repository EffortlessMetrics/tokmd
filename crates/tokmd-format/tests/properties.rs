//! Property-based tests for tokmd-format output determinism and structural
//! invariants.

use std::path::PathBuf;

use proptest::prelude::*;

use tokmd_format::{
    compute_diff_rows, compute_diff_totals, write_export_csv_to, write_export_json_to,
    write_lang_report_to, write_module_report_to,
};
use tokmd_settings::{ChildIncludeMode, ChildrenMode, ScanOptions};
use tokmd_types::{
    ExportArgs, ExportData, ExportFormat, FileKind, FileRow, LangArgs, LangReport, LangRow,
    ModuleArgs, ModuleReport, ModuleRow, RedactMode, TableFormat, Totals,
};

// ---------------------------------------------------------------------------
// Strategies
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
        // Deduplicate by language name – keep first occurrence
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
// Determinism: same input always produces byte-identical output
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn lang_md_deterministic(report in arb_lang_report()) {
        let args = LangArgs {
            paths: vec![PathBuf::from(".")],
            format: TableFormat::Md,
            top: 0,
            files: false,
            children: ChildrenMode::Collapse,
        };
        let render = |r: &LangReport| -> String {
            let mut buf = Vec::new();
            write_lang_report_to(&mut buf, r, &default_global(), &args).unwrap();
            String::from_utf8(buf).unwrap()
        };
        prop_assert_eq!(render(&report), render(&report));
    }

    #[test]
    fn lang_tsv_deterministic(report in arb_lang_report()) {
        let args = LangArgs {
            paths: vec![PathBuf::from(".")],
            format: TableFormat::Tsv,
            top: 0,
            files: false,
            children: ChildrenMode::Collapse,
        };
        let render = |r: &LangReport| -> String {
            let mut buf = Vec::new();
            write_lang_report_to(&mut buf, r, &default_global(), &args).unwrap();
            String::from_utf8(buf).unwrap()
        };
        prop_assert_eq!(render(&report), render(&report));
    }

    #[test]
    fn module_md_deterministic(report in arb_module_report()) {
        let args = ModuleArgs {
            paths: vec![PathBuf::from(".")],
            format: TableFormat::Md,
            top: 0,
            module_roots: vec!["crates".into()],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
        };
        let render = |r: &ModuleReport| -> String {
            let mut buf = Vec::new();
            write_module_report_to(&mut buf, r, &default_global(), &args).unwrap();
            String::from_utf8(buf).unwrap()
        };
        prop_assert_eq!(render(&report), render(&report));
    }

    #[test]
    fn module_tsv_deterministic(report in arb_module_report()) {
        let args = ModuleArgs {
            paths: vec![PathBuf::from(".")],
            format: TableFormat::Tsv,
            top: 0,
            module_roots: vec!["crates".into()],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
        };
        let render = |r: &ModuleReport| -> String {
            let mut buf = Vec::new();
            write_module_report_to(&mut buf, r, &default_global(), &args).unwrap();
            String::from_utf8(buf).unwrap()
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
        let render = |d: &ExportData| -> String {
            let mut buf = Vec::new();
            write_export_csv_to(&mut buf, d, &args).unwrap();
            String::from_utf8(buf).unwrap()
        };
        prop_assert_eq!(render(&data), render(&data));
    }
}

// ---------------------------------------------------------------------------
// Structural invariants: Markdown tables
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Every Markdown lang table has exactly (rows + 3) lines:
    /// header, separator, N data rows, total row.
    #[test]
    fn lang_md_line_count(report in arb_lang_report()) {
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
        let expected_lines = report.rows.len() + 3; // header + sep + total
        prop_assert_eq!(
            output.lines().count(),
            expected_lines,
            "expected {} lines, got {}",
            expected_lines,
            output.lines().count()
        );
    }

    /// Every TSV lang table has exactly (rows + 2) lines:
    /// header, N data rows, total row.
    #[test]
    fn lang_tsv_line_count(report in arb_lang_report()) {
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
        let expected_lines = report.rows.len() + 2; // header + total
        prop_assert_eq!(
            output.lines().count(),
            expected_lines,
            "expected {} lines, got {}",
            expected_lines,
            output.lines().count()
        );
    }

    /// Module Markdown always has header + sep + rows + total.
    #[test]
    fn module_md_line_count(report in arb_module_report()) {
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
        let expected_lines = report.rows.len() + 3;
        prop_assert_eq!(
            output.lines().count(),
            expected_lines,
        );
    }

    /// CSV export has header + one row per file.
    #[test]
    fn export_csv_line_count(rows in prop::collection::vec(arb_file_row(), 1..8)) {
        let n = rows.len();
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
        // CSV: header + N data rows (trailing newline means last split is empty)
        let lines: Vec<&str> = output.lines().collect();
        prop_assert_eq!(lines.len(), n + 1, "header + {} data rows", n);
    }
}

// ---------------------------------------------------------------------------
// JSON round-trip: lang JSON deserializes back into LangReceipt
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn lang_json_roundtrip(report in arb_lang_report()) {
        let args = LangArgs {
            paths: vec![PathBuf::from(".")],
            format: TableFormat::Json,
            top: 0,
            files: false,
            children: ChildrenMode::Collapse,
        };
        let mut buf = Vec::new();
        write_lang_report_to(&mut buf, &report, &default_global(), &args).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let receipt: tokmd_types::LangReceipt = serde_json::from_str(&output).unwrap();
        prop_assert_eq!(receipt.mode.as_str(), "lang");
        prop_assert_eq!(receipt.report.rows.len(), report.rows.len());
        prop_assert_eq!(receipt.report.total.code, report.total.code);
    }

    #[test]
    fn module_json_roundtrip(report in arb_module_report()) {
        let args = ModuleArgs {
            paths: vec![PathBuf::from(".")],
            format: TableFormat::Json,
            top: 0,
            module_roots: vec!["crates".into()],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
        };
        let mut buf = Vec::new();
        write_module_report_to(&mut buf, &report, &default_global(), &args).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let receipt: tokmd_types::ModuleReceipt = serde_json::from_str(&output).unwrap();
        prop_assert_eq!(receipt.mode.as_str(), "module");
        prop_assert_eq!(receipt.report.rows.len(), report.rows.len());
        prop_assert_eq!(receipt.report.total.code, report.total.code);
    }

    #[test]
    fn export_json_roundtrip(rows in prop::collection::vec(arb_file_row(), 1..6)) {
        let n = rows.len();
        let total_code: usize = rows.iter().map(|r| r.code).sum();
        let data = ExportData {
            rows,
            module_roots: vec!["src".into()],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        };
        let args = ExportArgs {
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
        };
        let mut buf = Vec::new();
        write_export_json_to(&mut buf, &data, &default_global(), &args).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let parsed: Vec<FileRow> = serde_json::from_str(&output).unwrap();
        prop_assert_eq!(parsed.len(), n);
        let rt_total: usize = parsed.iter().map(|r| r.code).sum();
        prop_assert_eq!(rt_total, total_code);
    }
}

// ---------------------------------------------------------------------------
// Diff: totals are consistent with rows
// ---------------------------------------------------------------------------

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
        let sum_delta_files: i64 = rows.iter().map(|r| r.delta_files).sum();

        prop_assert_eq!(totals.delta_code, sum_delta_code);
        prop_assert_eq!(totals.delta_lines, sum_delta_lines);
        prop_assert_eq!(totals.delta_files, sum_delta_files);
    }
}
