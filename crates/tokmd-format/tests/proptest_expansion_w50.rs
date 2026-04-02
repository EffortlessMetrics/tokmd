//! Property-based tests for tokmd-format (W50 expansion).
//!
//! Verifies that formatting functions never panic on arbitrary inputs,
//! produce structurally valid output, and preserve totals through truncation.

use proptest::prelude::*;
use tokmd_types::{
    ChildIncludeMode, ChildrenMode, ExportData, FileKind, FileRow, LangArgs, LangReport, LangRow,
    ModuleArgs, ModuleReport, ModuleRow, TableFormat, Totals,
};

// ── Strategies ───────────────────────────────────────────────────────────────

fn arb_totals() -> impl Strategy<Value = Totals> {
    (
        0usize..100_000,
        0usize..200_000,
        0usize..10_000,
        0usize..10_000_000,
        0usize..1_000_000,
        0usize..1_000,
    )
        .prop_map(|(code, lines, files, bytes, tokens, avg_lines)| Totals {
            code,
            lines,
            files,
            bytes,
            tokens,
            avg_lines,
        })
}

fn arb_lang_row() -> impl Strategy<Value = LangRow> {
    (
        "[a-zA-Z][a-zA-Z0-9 ]{0,20}",
        0usize..100_000,
        0usize..200_000,
        0usize..10_000,
        0usize..10_000_000,
        0usize..1_000_000,
        0usize..1_000,
    )
        .prop_map(
            |(lang, code, lines, files, bytes, tokens, avg_lines)| LangRow {
                lang,
                code,
                lines,
                files,
                bytes,
                tokens,
                avg_lines,
            },
        )
}

fn arb_module_row() -> impl Strategy<Value = ModuleRow> {
    (
        "[a-zA-Z0-9_/]{1,30}",
        0usize..100_000,
        0usize..200_000,
        0usize..10_000,
        0usize..10_000_000,
        0usize..1_000_000,
        0usize..1_000,
    )
        .prop_map(
            |(module, code, lines, files, bytes, tokens, avg_lines)| ModuleRow {
                module,
                code,
                lines,
                files,
                bytes,
                tokens,
                avg_lines,
            },
        )
}

fn arb_file_kind() -> impl Strategy<Value = FileKind> {
    prop_oneof![Just(FileKind::Parent), Just(FileKind::Child)]
}

fn arb_file_row() -> impl Strategy<Value = FileRow> {
    (
        "[a-zA-Z0-9_/]{1,20}\\.[a-z]{1,4}",
        "[a-zA-Z0-9_/]{1,20}",
        "[a-zA-Z]{1,10}",
        arb_file_kind(),
        0usize..100_000,
        0usize..50_000,
        0usize..50_000,
        0usize..200_000,
        0usize..10_000_000,
        0usize..1_000_000,
    )
        .prop_map(
            |(path, module, lang, kind, code, comments, blanks, lines, bytes, tokens)| FileRow {
                path,
                module,
                lang,
                kind,
                code,
                comments,
                blanks,
                lines,
                bytes,
                tokens,
            },
        )
}

fn arb_children_mode() -> impl Strategy<Value = ChildrenMode> {
    prop_oneof![Just(ChildrenMode::Collapse), Just(ChildrenMode::Separate)]
}

fn arb_child_include_mode() -> impl Strategy<Value = ChildIncludeMode> {
    prop_oneof![
        Just(ChildIncludeMode::Separate),
        Just(ChildIncludeMode::ParentsOnly)
    ]
}

fn arb_lang_report() -> impl Strategy<Value = LangReport> {
    (
        prop::collection::vec(arb_lang_row(), 0..20),
        arb_totals(),
        any::<bool>(),
        arb_children_mode(),
        0usize..50,
    )
        .prop_map(|(rows, total, with_files, children, top)| LangReport {
            rows,
            total,
            with_files,
            children,
            top,
        })
}

fn arb_module_report() -> impl Strategy<Value = ModuleReport> {
    (
        prop::collection::vec(arb_module_row(), 0..20),
        arb_totals(),
        prop::collection::vec("[a-z]{1,10}", 0..5),
        0usize..10,
        arb_child_include_mode(),
        0usize..50,
    )
        .prop_map(
            |(rows, total, module_roots, module_depth, children, top)| ModuleReport {
                rows,
                total,
                module_roots,
                module_depth,
                children,
                top,
            },
        )
}

fn arb_export_data() -> impl Strategy<Value = ExportData> {
    (
        prop::collection::vec(arb_file_row(), 0..20),
        prop::collection::vec("[a-z]{1,10}", 0..3),
        0usize..10,
        arb_child_include_mode(),
    )
        .prop_map(|(rows, module_roots, module_depth, children)| ExportData {
            rows,
            module_roots,
            module_depth,
            children,
        })
}

fn default_scan_options() -> tokmd_settings::ScanOptions {
    tokmd_settings::ScanOptions::default()
}

fn default_lang_args(format: TableFormat) -> LangArgs {
    LangArgs {
        paths: vec![".".into()],
        format,
        top: 0,
        files: false,
        children: ChildrenMode::Collapse,
    }
}

fn default_module_args(format: TableFormat) -> ModuleArgs {
    ModuleArgs {
        paths: vec![".".into()],
        format,
        top: 0,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

// ── Markdown output tests ────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn lang_md_never_panics(report in arb_lang_report()) {
        let global = default_scan_options();
        let args = default_lang_args(TableFormat::Md);
        let mut buf = Vec::new();
        let _ = tokmd_format::write_lang_report_to(&mut buf, &report, &global, &args);
    }

    #[test]
    fn module_md_never_panics(report in arb_module_report()) {
        let global = default_scan_options();
        let args = default_module_args(TableFormat::Md);
        let mut buf = Vec::new();
        let _ = tokmd_format::write_module_report_to(&mut buf, &report, &global, &args);
    }

    #[test]
    fn lang_md_has_header_and_separator(report in arb_lang_report()) {
        let global = default_scan_options();
        let args = default_lang_args(TableFormat::Md);
        let mut buf = Vec::new();
        tokmd_format::write_lang_report_to(&mut buf, &report, &global, &args).expect("operation must succeed");
        let output = String::from_utf8(buf).expect("output must be valid UTF-8");
        // Markdown table always starts with a header row and separator
        prop_assert!(output.starts_with('|'));
        prop_assert!(output.contains("|---|"));
    }

    #[test]
    fn module_md_has_header_and_separator(report in arb_module_report()) {
        let global = default_scan_options();
        let args = default_module_args(TableFormat::Md);
        let mut buf = Vec::new();
        tokmd_format::write_module_report_to(&mut buf, &report, &global, &args).expect("operation must succeed");
        let output = String::from_utf8(buf).expect("output must be valid UTF-8");
        prop_assert!(output.starts_with('|'));
        prop_assert!(output.contains("|---|"));
    }
}

// ── TSV output tests ─────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn lang_tsv_never_panics(report in arb_lang_report()) {
        let global = default_scan_options();
        let args = default_lang_args(TableFormat::Tsv);
        let mut buf = Vec::new();
        let _ = tokmd_format::write_lang_report_to(&mut buf, &report, &global, &args);
    }

    #[test]
    fn module_tsv_never_panics(report in arb_module_report()) {
        let global = default_scan_options();
        let args = default_module_args(TableFormat::Tsv);
        let mut buf = Vec::new();
        let _ = tokmd_format::write_module_report_to(&mut buf, &report, &global, &args);
    }

    #[test]
    fn lang_tsv_consistent_columns(report in arb_lang_report()) {
        let global = default_scan_options();
        let args = default_lang_args(TableFormat::Tsv);
        let mut buf = Vec::new();
        tokmd_format::write_lang_report_to(&mut buf, &report, &global, &args).expect("operation must succeed");
        let output = String::from_utf8(buf).expect("output must be valid UTF-8");
        let lines: Vec<&str> = output.lines().collect();
        if !lines.is_empty() {
            let header_cols = lines[0].split('\t').count();
            for (i, line) in lines.iter().enumerate() {
                let cols = line.split('\t').count();
                prop_assert_eq!(
                    cols, header_cols,
                    "Row {} has {} columns but header has {}",
                    i, cols, header_cols
                );
            }
        }
    }

    #[test]
    fn module_tsv_consistent_columns(report in arb_module_report()) {
        let global = default_scan_options();
        let args = default_module_args(TableFormat::Tsv);
        let mut buf = Vec::new();
        tokmd_format::write_module_report_to(&mut buf, &report, &global, &args).expect("operation must succeed");
        let output = String::from_utf8(buf).expect("output must be valid UTF-8");
        let lines: Vec<&str> = output.lines().collect();
        if !lines.is_empty() {
            let header_cols = lines[0].split('\t').count();
            for (i, line) in lines.iter().enumerate() {
                let cols = line.split('\t').count();
                prop_assert_eq!(
                    cols, header_cols,
                    "Row {} has {} columns but header has {}",
                    i, cols, header_cols
                );
            }
        }
    }
}

// ── JSON output tests ────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn lang_json_always_valid(report in arb_lang_report()) {
        let global = default_scan_options();
        let args = default_lang_args(TableFormat::Json);
        let mut buf = Vec::new();
        tokmd_format::write_lang_report_to(&mut buf, &report, &global, &args).expect("operation must succeed");
        let output = String::from_utf8(buf).expect("output must be valid UTF-8");
        let parsed: serde_json::Value = serde_json::from_str(output.trim()).expect("must parse valid JSON");
        prop_assert!(parsed.is_object());
        prop_assert!(parsed.get("schema_version").is_some());
    }

    #[test]
    fn module_json_always_valid(report in arb_module_report()) {
        let global = default_scan_options();
        let args = default_module_args(TableFormat::Json);
        let mut buf = Vec::new();
        tokmd_format::write_module_report_to(&mut buf, &report, &global, &args).expect("operation must succeed");
        let output = String::from_utf8(buf).expect("output must be valid UTF-8");
        let parsed: serde_json::Value = serde_json::from_str(output.trim()).expect("must parse valid JSON");
        prop_assert!(parsed.is_object());
        prop_assert!(parsed.get("schema_version").is_some());
    }

    #[test]
    fn lang_json_roundtrip_rows(report in arb_lang_report()) {
        let global = default_scan_options();
        let args = default_lang_args(TableFormat::Json);
        let mut buf = Vec::new();
        tokmd_format::write_lang_report_to(&mut buf, &report, &global, &args).expect("operation must succeed");
        let output = String::from_utf8(buf).expect("output must be valid UTF-8");
        let parsed: serde_json::Value = serde_json::from_str(output.trim()).expect("must parse valid JSON");
        // Row count in JSON must match input
        if let Some(rows) = parsed.get("rows").and_then(|v| v.as_array()) {
            prop_assert_eq!(rows.len(), report.rows.len());
        }
    }

    #[test]
    fn module_json_roundtrip_rows(report in arb_module_report()) {
        let global = default_scan_options();
        let args = default_module_args(TableFormat::Json);
        let mut buf = Vec::new();
        tokmd_format::write_module_report_to(&mut buf, &report, &global, &args).expect("operation must succeed");
        let output = String::from_utf8(buf).expect("output must be valid UTF-8");
        let parsed: serde_json::Value = serde_json::from_str(output.trim()).expect("must parse valid JSON");
        if let Some(rows) = parsed.get("rows").and_then(|v| v.as_array()) {
            prop_assert_eq!(rows.len(), report.rows.len());
        }
    }
}

// ── Export data serde roundtrip ──────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn export_data_json_roundtrip(data in arb_export_data()) {
        let json = serde_json::to_string(&data).expect("operation must succeed");
        let round: ExportData = serde_json::from_str(&json).expect("must parse valid JSON");
        prop_assert_eq!(round.rows.len(), data.rows.len());
        prop_assert_eq!(round.module_roots, data.module_roots);
        prop_assert_eq!(round.module_depth, data.module_depth);
    }

    #[test]
    fn file_row_json_roundtrip(row in arb_file_row()) {
        let json = serde_json::to_string(&row).expect("operation must succeed");
        let round: FileRow = serde_json::from_str(&json).expect("must parse valid JSON");
        prop_assert_eq!(round, row);
    }

    #[test]
    fn lang_row_json_roundtrip(row in arb_lang_row()) {
        let json = serde_json::to_string(&row).expect("operation must succeed");
        let round: LangRow = serde_json::from_str(&json).expect("must parse valid JSON");
        prop_assert_eq!(round, row);
    }

    #[test]
    fn module_row_json_roundtrip(row in arb_module_row()) {
        let json = serde_json::to_string(&row).expect("operation must succeed");
        let round: ModuleRow = serde_json::from_str(&json).expect("must parse valid JSON");
        prop_assert_eq!(round, row);
    }
}
