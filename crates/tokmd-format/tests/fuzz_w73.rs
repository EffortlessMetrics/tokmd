//! Fuzz-like property tests for tokmd-format.
//!
//! These tests exercise formatting functions with large, random input spaces
//! to ensure no panics occur regardless of input content.

use std::path::PathBuf;

use proptest::prelude::*;

use tokmd_format::{
    DiffColorMode, DiffRenderOptions, compute_diff_rows, compute_diff_totals, render_diff_md,
    render_diff_md_with_options, write_export_csv_to, write_export_json_to, write_export_jsonl_to,
    write_lang_report_to, write_module_report_to,
};
use tokmd_settings::{ChildIncludeMode, ChildrenMode, ScanOptions};
use tokmd_types::{
    DiffRow, DiffTotals, ExportArgs, ExportData, ExportFormat, FileKind, FileRow, LangArgs,
    LangReport, LangRow, ModuleArgs, ModuleReport, ModuleRow, RedactMode, TableFormat, Totals,
};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_unicode_string() -> impl Strategy<Value = String> {
    prop::string::string_regex(".{0,100}").unwrap()
}

fn arb_lang_row_unicode() -> impl Strategy<Value = LangRow> {
    (
        arb_unicode_string(),
        0usize..50_000,
        0usize..100_000,
        0usize..500,
    )
        .prop_map(|(lang, code, lines, files)| {
            let files = files.max(1);
            LangRow {
                lang,
                code,
                lines: lines.max(code),
                files,
                bytes: code.saturating_mul(10),
                tokens: code / 4,
                avg_lines: lines.checked_div(files).unwrap_or(0),
            }
        })
}

fn arb_lang_report_unicode() -> impl Strategy<Value = LangReport> {
    prop::collection::vec(arb_lang_row_unicode(), 0..10).prop_map(|rows| {
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

fn arb_module_row_unicode() -> impl Strategy<Value = ModuleRow> {
    (
        arb_unicode_string(),
        0usize..50_000,
        0usize..100_000,
        0usize..200,
    )
        .prop_map(|(module, code, lines, files)| {
            let files = files.max(1);
            ModuleRow {
                module,
                code,
                lines: lines.max(code),
                files,
                bytes: code.saturating_mul(10),
                tokens: code / 4,
                avg_lines: lines.checked_div(files).unwrap_or(0),
            }
        })
}

fn arb_module_report_unicode() -> impl Strategy<Value = ModuleReport> {
    prop::collection::vec(arb_module_row_unicode(), 0..8).prop_map(|rows| {
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

fn arb_file_row_unicode() -> impl Strategy<Value = FileRow> {
    (
        arb_unicode_string(),
        arb_unicode_string(),
        arb_unicode_string(),
        0usize..10_000,
        0usize..2_000,
        0usize..1_000,
    )
        .prop_map(|(path, module, lang, code, comments, blanks)| FileRow {
            path,
            module,
            lang,
            kind: FileKind::Parent,
            code,
            comments,
            blanks,
            lines: code + comments + blanks,
            bytes: code.saturating_mul(10),
            tokens: code / 4,
        })
}

fn arb_diff_row() -> impl Strategy<Value = DiffRow> {
    (
        arb_unicode_string(),
        0usize..50_000,
        0usize..50_000,
        0usize..50_000,
        0usize..50_000,
        0usize..500,
        0usize..500,
    )
        .prop_map(
            |(lang, old_code, new_code, old_lines, new_lines, old_files, new_files)| DiffRow {
                lang,
                old_code,
                new_code,
                delta_code: new_code as i64 - old_code as i64,
                old_lines,
                new_lines,
                delta_lines: new_lines as i64 - old_lines as i64,
                old_files,
                new_files,
                delta_files: new_files as i64 - old_files as i64,
                old_bytes: old_code * 10,
                new_bytes: new_code * 10,
                delta_bytes: (new_code as i64 - old_code as i64) * 10,
                old_tokens: old_code / 4,
                new_tokens: new_code / 4,
                delta_tokens: new_code as i64 / 4 - old_code as i64 / 4,
            },
        )
}

fn arb_diff_totals() -> impl Strategy<Value = DiffTotals> {
    (0usize..100_000, 0usize..100_000).prop_map(|(old_code, new_code)| DiffTotals {
        old_code,
        new_code,
        delta_code: new_code as i64 - old_code as i64,
        old_lines: old_code * 2,
        new_lines: new_code * 2,
        delta_lines: (new_code as i64 - old_code as i64) * 2,
        old_files: 10,
        new_files: 12,
        delta_files: 2,
        old_bytes: old_code * 10,
        new_bytes: new_code * 10,
        delta_bytes: (new_code as i64 - old_code as i64) * 10,
        old_tokens: old_code / 4,
        new_tokens: new_code / 4,
        delta_tokens: new_code as i64 / 4 - old_code as i64 / 4,
    })
}

fn default_global() -> ScanOptions {
    ScanOptions::default()
}

// ---------------------------------------------------------------------------
// 1. Markdown rendering never panics with arbitrary Unicode
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn fuzz_lang_md_no_panic(report in arb_lang_report_unicode()) {
        let args = LangArgs {
            paths: vec![PathBuf::from(".")],
            format: TableFormat::Md,
            top: 0,
            files: false,
            children: ChildrenMode::Collapse,
        };
        let mut buf = Vec::new();
        let _ = write_lang_report_to(&mut buf, &report, &default_global(), &args);
    }

    #[test]
    fn fuzz_lang_tsv_no_panic(report in arb_lang_report_unicode()) {
        let args = LangArgs {
            paths: vec![PathBuf::from(".")],
            format: TableFormat::Tsv,
            top: 0,
            files: false,
            children: ChildrenMode::Collapse,
        };
        let mut buf = Vec::new();
        let _ = write_lang_report_to(&mut buf, &report, &default_global(), &args);
    }

    #[test]
    fn fuzz_lang_json_no_panic(report in arb_lang_report_unicode()) {
        let args = LangArgs {
            paths: vec![PathBuf::from(".")],
            format: TableFormat::Json,
            top: 0,
            files: false,
            children: ChildrenMode::Collapse,
        };
        let mut buf = Vec::new();
        let _ = write_lang_report_to(&mut buf, &report, &default_global(), &args);
    }

    #[test]
    fn fuzz_module_md_no_panic(report in arb_module_report_unicode()) {
        let args = ModuleArgs {
            paths: vec![PathBuf::from(".")],
            format: TableFormat::Md,
            top: 0,
            module_roots: vec!["crates".into()],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
        };
        let mut buf = Vec::new();
        let _ = write_module_report_to(&mut buf, &report, &default_global(), &args);
    }

    #[test]
    fn fuzz_module_tsv_no_panic(report in arb_module_report_unicode()) {
        let args = ModuleArgs {
            paths: vec![PathBuf::from(".")],
            format: TableFormat::Tsv,
            top: 0,
            module_roots: vec!["crates".into()],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
        };
        let mut buf = Vec::new();
        let _ = write_module_report_to(&mut buf, &report, &default_global(), &args);
    }

    #[test]
    fn fuzz_module_json_no_panic(report in arb_module_report_unicode()) {
        let args = ModuleArgs {
            paths: vec![PathBuf::from(".")],
            format: TableFormat::Json,
            top: 0,
            module_roots: vec!["crates".into()],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
        };
        let mut buf = Vec::new();
        let _ = write_module_report_to(&mut buf, &report, &default_global(), &args);
    }
}

// ---------------------------------------------------------------------------
// 2. TSV with tabs/newlines in language names
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn fuzz_tsv_with_tab_newline_names(
        name in "[a-z\t\n\r]{1,30}",
        code in 0usize..5_000,
    ) {
        let report = LangReport {
            rows: vec![LangRow {
                lang: name,
                code,
                lines: code + 10,
                files: 1,
                bytes: code * 10,
                tokens: code / 4,
                avg_lines: code + 10,
            }],
            total: Totals {
                code,
                lines: code + 10,
                files: 1,
                bytes: code * 10,
                tokens: code / 4,
                avg_lines: code + 10,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };
        let args = LangArgs {
            paths: vec![PathBuf::from(".")],
            format: TableFormat::Tsv,
            top: 0,
            files: false,
            children: ChildrenMode::Collapse,
        };
        let mut buf = Vec::new();
        let _ = write_lang_report_to(&mut buf, &report, &default_global(), &args);
    }
}

// ---------------------------------------------------------------------------
// 3. Export formats never panic with random Unicode
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn fuzz_export_csv_no_panic(rows in prop::collection::vec(arb_file_row_unicode(), 0..8)) {
        let data = ExportData {
            rows,
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        };
        let args = ExportArgs {
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
        };
        let mut buf = Vec::new();
        let _ = write_export_csv_to(&mut buf, &data, &args);
    }

    #[test]
    fn fuzz_export_json_no_panic(rows in prop::collection::vec(arb_file_row_unicode(), 0..8)) {
        let data = ExportData {
            rows,
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        };
        let args = ExportArgs {
            paths: vec![PathBuf::from(".")],
            format: ExportFormat::Json,
            output: None,
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
            min_code: 0,
            max_rows: 0,
            redact: RedactMode::None,
            meta: false,
            strip_prefix: None,
        };
        let mut buf = Vec::new();
        let _ = write_export_json_to(&mut buf, &data, &default_global(), &args);
    }

    #[test]
    fn fuzz_export_jsonl_no_panic(rows in prop::collection::vec(arb_file_row_unicode(), 0..8)) {
        let data = ExportData {
            rows,
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        };
        let args = ExportArgs {
            paths: vec![PathBuf::from(".")],
            format: ExportFormat::Jsonl,
            output: None,
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
            min_code: 0,
            max_rows: 0,
            redact: RedactMode::None,
            meta: false,
            strip_prefix: None,
        };
        let mut buf = Vec::new();
        let _ = write_export_jsonl_to(&mut buf, &data, &default_global(), &args);
    }
}

// ---------------------------------------------------------------------------
// 4. Diff rendering never panics with arbitrary input
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn fuzz_diff_md_no_panic(
        from in arb_unicode_string(),
        to in arb_unicode_string(),
        rows in prop::collection::vec(arb_diff_row(), 0..10),
        totals in arb_diff_totals(),
    ) {
        let _ = render_diff_md(&from, &to, &rows, &totals);
    }

    #[test]
    fn fuzz_diff_md_compact_no_panic(
        from in arb_unicode_string(),
        to in arb_unicode_string(),
        rows in prop::collection::vec(arb_diff_row(), 0..10),
        totals in arb_diff_totals(),
    ) {
        let opts = DiffRenderOptions {
            compact: true,
            color: DiffColorMode::Off,
        };
        let _ = render_diff_md_with_options(&from, &to, &rows, &totals, opts);
    }

    #[test]
    fn fuzz_diff_md_ansi_no_panic(
        from in arb_unicode_string(),
        to in arb_unicode_string(),
        rows in prop::collection::vec(arb_diff_row(), 0..10),
        totals in arb_diff_totals(),
    ) {
        let opts = DiffRenderOptions {
            compact: false,
            color: DiffColorMode::Ansi,
        };
        let _ = render_diff_md_with_options(&from, &to, &rows, &totals, opts);
    }

    #[test]
    fn fuzz_compute_diff_rows_no_panic(
        from in arb_lang_report_unicode(),
        to in arb_lang_report_unicode(),
    ) {
        let rows = compute_diff_rows(&from, &to);
        let _ = compute_diff_totals(&rows);
    }
}
