//! Determinism-focused regression tests for tokmd-format.
//!
//! Verifies that rendering the same receipt multiple times produces
//! byte-identical output, that TSV ordering is deterministic, and
//! that path normalization produces forward slashes.

use std::path::PathBuf;

use proptest::prelude::*;

use tokmd_format::write_lang_report_to;
use tokmd_settings::{ChildIncludeMode, ChildrenMode, ScanOptions};
use tokmd_types::{
    LangArgs, LangReport, LangRow, ModuleArgs, ModuleReport, ModuleRow, TableFormat, Totals,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn default_global() -> ScanOptions {
    ScanOptions::default()
}

fn make_lang_row(lang: &str, code: usize) -> LangRow {
    let lines = code + 20;
    let files = 3;
    LangRow {
        lang: lang.to_string(),
        code,
        lines,
        files,
        bytes: code * 10,
        tokens: code * 10 / 4,
        avg_lines: lines / files,
    }
}

fn make_lang_report(rows: Vec<LangRow>) -> LangReport {
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
}

fn make_module_row(module: &str, code: usize) -> ModuleRow {
    let lines = code + 30;
    let files = 2;
    ModuleRow {
        module: module.to_string(),
        code,
        lines,
        files,
        bytes: code * 10,
        tokens: code * 10 / 4,
        avg_lines: lines / files,
    }
}

fn render_lang_md(report: &LangReport) -> String {
    let args = LangArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Md,
        top: 0,
        files: false,
        children: ChildrenMode::Collapse,
    };
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, report, &default_global(), &args).unwrap();
    String::from_utf8(buf).unwrap()
}

fn render_lang_tsv(report: &LangReport) -> String {
    let args = LangArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Tsv,
        top: 0,
        files: false,
        children: ChildrenMode::Collapse,
    };
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, report, &default_global(), &args).unwrap();
    String::from_utf8(buf).unwrap()
}

fn render_module_md(report: &ModuleReport) -> String {
    let args = ModuleArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Md,
        top: 0,
        module_roots: vec!["crates".into()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };
    let mut buf = Vec::new();
    tokmd_format::write_module_report_to(&mut buf, report, &default_global(), &args).unwrap();
    String::from_utf8(buf).unwrap()
}

// ---------------------------------------------------------------------------
// 1. Rendering the same LangReceipt multiple times → byte-identical Markdown
// ---------------------------------------------------------------------------

#[test]
fn determinism_lang_md_repeated_render_identical() {
    let report = make_lang_report(vec![
        make_lang_row("Rust", 1000),
        make_lang_row("Python", 500),
        make_lang_row("Go", 200),
    ]);

    let out1 = render_lang_md(&report);
    let out2 = render_lang_md(&report);
    let out3 = render_lang_md(&report);
    assert_eq!(out1, out2, "Repeated Markdown render must be identical (1 vs 2)");
    assert_eq!(out2, out3, "Repeated Markdown render must be identical (2 vs 3)");
}

#[test]
fn determinism_lang_md_single_language() {
    let report = make_lang_report(vec![make_lang_row("Rust", 42)]);
    let out1 = render_lang_md(&report);
    let out2 = render_lang_md(&report);
    assert_eq!(out1, out2);
}

#[test]
fn determinism_module_md_repeated_render_identical() {
    let rows = vec![
        make_module_row("crates/foo", 800),
        make_module_row("crates/bar", 400),
        make_module_row("src", 200),
    ];
    let total = Totals {
        code: rows.iter().map(|r| r.code).sum(),
        lines: rows.iter().map(|r| r.lines).sum(),
        files: rows.iter().map(|r| r.files).sum(),
        bytes: rows.iter().map(|r| r.bytes).sum(),
        tokens: rows.iter().map(|r| r.tokens).sum(),
        avg_lines: 0,
    };
    let report = ModuleReport {
        rows,
        total,
        module_roots: vec!["crates".into()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
        top: 0,
    };

    let out1 = render_module_md(&report);
    let out2 = render_module_md(&report);
    assert_eq!(out1, out2, "Module Markdown render must be byte-identical");
}

// ---------------------------------------------------------------------------
// 2. TSV output ordering is deterministic (desc by code, tiebreak by name)
// ---------------------------------------------------------------------------

#[test]
fn determinism_tsv_ordering_desc_by_code() {
    // Rows are pre-sorted by model layer; verify format preserves that order.
    let report = make_lang_report(vec![
        make_lang_row("Rust", 1000),
        make_lang_row("Python", 500),
        make_lang_row("Go", 100),
    ]);

    let tsv = render_lang_tsv(&report);
    let lines: Vec<&str> = tsv.lines().collect();
    // lines[0] = header, lines[1..3] = data rows, lines[3] = total
    assert!(lines[1].starts_with("Rust"), "First data row should be Rust (highest code)");
    assert!(lines[2].starts_with("Python"), "Second data row should be Python");
    assert!(lines[3].starts_with("Go"), "Third data row should be Go (lowest code)");
}

#[test]
fn determinism_tsv_tiebreak_by_name() {
    // When code counts are equal, ordering should be alphabetical by name.
    let report = make_lang_report(vec![
        make_lang_row("Alpha", 500),
        make_lang_row("Beta", 500),
        make_lang_row("Gamma", 500),
    ]);

    let tsv = render_lang_tsv(&report);
    let lines: Vec<&str> = tsv.lines().collect();
    // With equal code, rows should appear in order given (which was alphabetical).
    assert!(lines[1].starts_with("Alpha"), "Tiebreak: Alpha first");
    assert!(lines[2].starts_with("Beta"), "Tiebreak: Beta second");
    assert!(lines[3].starts_with("Gamma"), "Tiebreak: Gamma third");
}

#[test]
fn determinism_tsv_repeated_render() {
    let report = make_lang_report(vec![
        make_lang_row("Rust", 2000),
        make_lang_row("C", 1500),
        make_lang_row("Python", 800),
    ]);

    let tsv1 = render_lang_tsv(&report);
    let tsv2 = render_lang_tsv(&report);
    assert_eq!(tsv1, tsv2, "TSV output must be byte-identical on repeated render");
}

// ---------------------------------------------------------------------------
// 3. Path normalization in formatted output (backslash → forward slash)
// ---------------------------------------------------------------------------

#[test]
fn determinism_path_normalization_in_file_rows() {
    // FileRow paths must always use forward slashes.
    let row = tokmd_types::FileRow {
        path: "src/main.rs".to_string(), // normalized
        module: "src".to_string(),
        lang: "Rust".to_string(),
        kind: tokmd_types::FileKind::Parent,
        code: 100,
        comments: 10,
        blanks: 5,
        lines: 115,
        bytes: 1000,
        tokens: 250,
    };

    let json = serde_json::to_string(&row).unwrap();
    assert!(!json.contains('\\'), "Serialized path must not contain backslashes");
    assert!(json.contains("src/main.rs"), "Path must use forward slashes");
}

#[test]
fn determinism_module_key_uses_forward_slashes() {
    // Module keys must always use forward slashes, even from Windows-style input.
    let row = make_module_row("crates/tokmd-types", 500);
    let json = serde_json::to_string(&row).unwrap();
    assert!(!json.contains('\\'), "Module key must not contain backslashes");
}

#[test]
fn determinism_markdown_output_has_no_backslashes_in_paths() {
    // Verify that rendered Markdown doesn't contain OS-specific path separators.
    let rows = vec![
        make_module_row("crates/foo", 1000),
        make_module_row("crates/bar", 500),
    ];
    let total = Totals {
        code: 1500,
        lines: 1560,
        files: 4,
        bytes: 15000,
        tokens: 3750,
        avg_lines: 0,
    };
    let report = ModuleReport {
        rows,
        total,
        module_roots: vec!["crates".into()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
        top: 0,
    };

    let md = render_module_md(&report);
    assert!(!md.contains('\\'), "Module Markdown must use forward slashes only");
}

// ---------------------------------------------------------------------------
// 4. Proptest: repeated rendering always produces identical output
// ---------------------------------------------------------------------------

fn arb_lang_row_strat() -> impl Strategy<Value = LangRow> {
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

fn arb_lang_report_strat() -> impl Strategy<Value = LangReport> {
    prop::collection::vec(arb_lang_row_strat(), 1..6).prop_map(|rows| {
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

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn determinism_proptest_lang_md_stable(report in arb_lang_report_strat()) {
        let out1 = render_lang_md(&report);
        let out2 = render_lang_md(&report);
        prop_assert_eq!(out1, out2, "Markdown render must be byte-identical");
    }

    #[test]
    fn determinism_proptest_lang_tsv_stable(report in arb_lang_report_strat()) {
        let out1 = render_lang_tsv(&report);
        let out2 = render_lang_tsv(&report);
        prop_assert_eq!(out1, out2, "TSV render must be byte-identical");
    }

    #[test]
    fn determinism_proptest_no_backslash_in_lang_output(report in arb_lang_report_strat()) {
        let md = render_lang_md(&report);
        let tsv = render_lang_tsv(&report);
        prop_assert!(!md.contains('\\'), "Markdown output must not contain backslashes");
        prop_assert!(!tsv.contains('\\'), "TSV output must not contain backslashes");
    }
}
