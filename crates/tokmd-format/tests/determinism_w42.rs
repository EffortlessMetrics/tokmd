//! Determinism and snapshot tests for tokmd-format – wave 42.
//!
//! Verifies deterministic output across rendering functions, tie-breaking,
//! BTreeMap key ordering in JSON, and path normalisation.
//!
//! Run with: `cargo test -p tokmd-format --test determinism_w42`

use std::path::PathBuf;

use tokmd_format::{
    compute_diff_rows, compute_diff_totals, render_diff_md, write_export_csv_to,
    write_lang_report_to, write_module_report_to,
};
use tokmd_settings::ScanOptions;
use tokmd_types::{
    ChildIncludeMode, ChildrenMode, DiffRow, ExportArgs, ExportData, ExportFormat, FileKind,
    FileRow, LangArgs, LangReport, LangRow, ModuleArgs, ModuleReport, ModuleRow, RedactMode,
    TableFormat, Totals,
};

// =========================================================================
// Helpers
// =========================================================================

fn lang_report_with_rows(rows: Vec<LangRow>, with_files: bool) -> LangReport {
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
        with_files,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

fn module_report_with_rows(rows: Vec<ModuleRow>) -> ModuleReport {
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
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::ParentsOnly,
        top: 0,
    }
}

fn default_global() -> ScanOptions {
    ScanOptions::default()
}

fn lang_args(format: TableFormat) -> LangArgs {
    LangArgs {
        paths: vec![PathBuf::from(".")],
        format,
        top: 0,
        files: true,
        children: ChildrenMode::Collapse,
    }
}

fn module_args(format: TableFormat) -> ModuleArgs {
    ModuleArgs {
        paths: vec![PathBuf::from(".")],
        format,
        top: 0,
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::ParentsOnly,
    }
}

fn export_args_csv() -> ExportArgs {
    ExportArgs {
        paths: vec![PathBuf::from(".")],
        format: ExportFormat::Csv,
        output: None,
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::ParentsOnly,
        min_code: 0,
        max_rows: 0,
        redact: RedactMode::None,
        meta: false,
        strip_prefix: None,
    }
}

fn render_lang(report: &LangReport, format: TableFormat) -> String {
    let global = default_global();
    let args = lang_args(format);
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, report, &global, &args).expect("operation must succeed");
    String::from_utf8(buf).expect("output must be valid UTF-8")
}

fn render_module(report: &ModuleReport, format: TableFormat) -> String {
    let global = default_global();
    let args = module_args(format);
    let mut buf = Vec::new();
    write_module_report_to(&mut buf, report, &global, &args).expect("operation must succeed");
    String::from_utf8(buf).expect("output must be valid UTF-8")
}

fn render_csv(export: &ExportData) -> String {
    let args = export_args_csv();
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, export, &args).expect("operation must succeed");
    String::from_utf8(buf).expect("output must be valid UTF-8")
}

fn two_lang_rows() -> Vec<LangRow> {
    vec![
        LangRow {
            lang: "Rust".into(),
            code: 500,
            lines: 700,
            files: 10,
            bytes: 20000,
            tokens: 5000,
            avg_lines: 70,
        },
        LangRow {
            lang: "Python".into(),
            code: 300,
            lines: 450,
            files: 5,
            bytes: 12000,
            tokens: 3000,
            avg_lines: 90,
        },
    ]
}

// =========================================================================
// 1. Same LangRow data → same Markdown output on repeated calls
// =========================================================================

#[test]
fn lang_md_repeated_calls_identical() {
    let report = lang_report_with_rows(two_lang_rows(), true);
    let a = render_lang(&report, TableFormat::Md);
    let b = render_lang(&report, TableFormat::Md);
    let c = render_lang(&report, TableFormat::Md);
    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn lang_md_without_files_repeated_calls_identical() {
    let report = lang_report_with_rows(two_lang_rows(), false);
    let a = render_lang(&report, TableFormat::Md);
    let b = render_lang(&report, TableFormat::Md);
    assert_eq!(a, b);
}

#[test]
fn lang_md_single_row_deterministic() {
    let rows = vec![LangRow {
        lang: "Go".into(),
        code: 42,
        lines: 55,
        files: 1,
        bytes: 1000,
        tokens: 250,
        avg_lines: 55,
    }];
    let report = lang_report_with_rows(rows, true);
    let a = render_lang(&report, TableFormat::Md);
    let b = render_lang(&report, TableFormat::Md);
    assert_eq!(a, b);
    assert!(a.contains("|Go|42|"));
}

// =========================================================================
// 2. Same ModuleRow data → same TSV output
// =========================================================================

#[test]
fn module_tsv_repeated_calls_identical() {
    let rows = vec![
        ModuleRow {
            module: "src/core".into(),
            code: 400,
            lines: 600,
            files: 6,
            bytes: 16000,
            tokens: 4000,
            avg_lines: 100,
        },
        ModuleRow {
            module: "src/utils".into(),
            code: 200,
            lines: 300,
            files: 4,
            bytes: 8000,
            tokens: 2000,
            avg_lines: 75,
        },
    ];
    let report = module_report_with_rows(rows);
    let a = render_module(&report, TableFormat::Tsv);
    let b = render_module(&report, TableFormat::Tsv);
    let c = render_module(&report, TableFormat::Tsv);
    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn module_tsv_single_row_deterministic() {
    let rows = vec![ModuleRow {
        module: "lib".into(),
        code: 10,
        lines: 15,
        files: 1,
        bytes: 500,
        tokens: 120,
        avg_lines: 15,
    }];
    let report = module_report_with_rows(rows);
    let a = render_module(&report, TableFormat::Tsv);
    let b = render_module(&report, TableFormat::Tsv);
    assert_eq!(a, b);
    assert!(a.contains("lib\t10\t"));
}

#[test]
fn module_md_repeated_calls_identical() {
    let rows = vec![
        ModuleRow {
            module: "src/a".into(),
            code: 100,
            lines: 150,
            files: 3,
            bytes: 4000,
            tokens: 1000,
            avg_lines: 50,
        },
        ModuleRow {
            module: "src/b".into(),
            code: 100,
            lines: 150,
            files: 3,
            bytes: 4000,
            tokens: 1000,
            avg_lines: 50,
        },
    ];
    let report = module_report_with_rows(rows);
    let a = render_module(&report, TableFormat::Md);
    let b = render_module(&report, TableFormat::Md);
    assert_eq!(a, b);
}

// =========================================================================
// 3. Sorting stability with equal code line counts (tie-break by name)
// =========================================================================

#[test]
fn lang_md_tie_break_preserves_input_order() {
    // Both rows have identical code counts – output must preserve input order
    let rows = vec![
        LangRow {
            lang: "Alpha".into(),
            code: 100,
            lines: 200,
            files: 2,
            bytes: 4000,
            tokens: 1000,
            avg_lines: 100,
        },
        LangRow {
            lang: "Beta".into(),
            code: 100,
            lines: 200,
            files: 2,
            bytes: 4000,
            tokens: 1000,
            avg_lines: 100,
        },
    ];
    let report = lang_report_with_rows(rows, true);
    let output = render_lang(&report, TableFormat::Md);
    let alpha_pos = output.find("Alpha").expect("operation must succeed");
    let beta_pos = output.find("Beta").expect("operation must succeed");
    assert!(alpha_pos < beta_pos, "Alpha should appear before Beta");
}

#[test]
fn lang_tsv_tie_break_preserves_input_order() {
    let rows = vec![
        LangRow {
            lang: "Zebra".into(),
            code: 50,
            lines: 80,
            files: 1,
            bytes: 1500,
            tokens: 400,
            avg_lines: 80,
        },
        LangRow {
            lang: "Apple".into(),
            code: 50,
            lines: 80,
            files: 1,
            bytes: 1500,
            tokens: 400,
            avg_lines: 80,
        },
    ];
    let report = lang_report_with_rows(rows, true);
    let output = render_lang(&report, TableFormat::Tsv);
    let zebra_pos = output.find("Zebra").expect("operation must succeed");
    let apple_pos = output.find("Apple").expect("operation must succeed");
    assert!(zebra_pos < apple_pos, "Zebra should appear before Apple");
}

#[test]
fn module_md_tie_break_preserves_input_order() {
    let rows = vec![
        ModuleRow {
            module: "z_mod".into(),
            code: 200,
            lines: 300,
            files: 3,
            bytes: 8000,
            tokens: 2000,
            avg_lines: 100,
        },
        ModuleRow {
            module: "a_mod".into(),
            code: 200,
            lines: 300,
            files: 3,
            bytes: 8000,
            tokens: 2000,
            avg_lines: 100,
        },
    ];
    let report = module_report_with_rows(rows);
    let output = render_module(&report, TableFormat::Md);
    let z_pos = output.find("z_mod").expect("operation must succeed");
    let a_pos = output.find("a_mod").expect("operation must succeed");
    assert!(z_pos < a_pos, "z_mod should appear before a_mod");
}

// =========================================================================
// 4. BTreeMap ordering preserved in JSON output
// =========================================================================

#[test]
fn lang_json_top_level_keys_sorted() {
    let report = lang_report_with_rows(two_lang_rows(), true);
    let global = default_global();
    let args = lang_args(TableFormat::Json);
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &global, &args).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(output.trim()).expect("must parse valid JSON");
    let map = json.as_object().expect("must be a JSON object");
    let keys: Vec<&String> = map.keys().collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(keys, sorted, "top-level JSON keys must be sorted");
}

#[test]
fn module_json_top_level_keys_sorted() {
    let rows = vec![ModuleRow {
        module: "src".into(),
        code: 100,
        lines: 150,
        files: 3,
        bytes: 4000,
        tokens: 1000,
        avg_lines: 50,
    }];
    let report = module_report_with_rows(rows);
    let global = default_global();
    let args = module_args(TableFormat::Json);
    let mut buf = Vec::new();
    write_module_report_to(&mut buf, &report, &global, &args).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(output.trim()).expect("must parse valid JSON");
    let map = json.as_object().expect("must be a JSON object");
    let keys: Vec<&String> = map.keys().collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(keys, sorted, "module JSON top-level keys must be sorted");
}

#[test]
fn lang_json_row_keys_sorted() {
    let report = lang_report_with_rows(two_lang_rows(), true);
    let global = default_global();
    let args = lang_args(TableFormat::Json);
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &global, &args).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(output.trim()).expect("must parse valid JSON");
    if let Some(rows) = json
        .get("report")
        .and_then(|r| r.get("rows"))
        .and_then(|v| v.as_array())
    {
        for (i, row) in rows.iter().enumerate() {
            let map = row.as_object().expect("must be a JSON object");
            let keys: Vec<&String> = map.keys().collect();
            let mut sorted = keys.clone();
            sorted.sort();
            assert_eq!(keys, sorted, "row[{i}] keys must be sorted");
        }
    }
}

// =========================================================================
// 5. Path normalisation in output (forward slashes)
// =========================================================================

#[test]
fn csv_paths_use_forward_slashes() {
    let export = ExportData {
        rows: vec![FileRow {
            path: "src/core/main.rs".into(),
            module: "src/core".into(),
            lang: "Rust".into(),
            kind: FileKind::Parent,
            code: 100,
            comments: 10,
            blanks: 5,
            lines: 115,
            bytes: 4000,
            tokens: 1000,
        }],
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::ParentsOnly,
    };
    let output = render_csv(&export);
    assert!(
        output.contains("src/core/main.rs"),
        "paths use forward slashes"
    );
    assert!(!output.contains("src\\core\\main.rs"), "no backslashes");
}

#[test]
fn module_md_paths_use_forward_slashes() {
    let rows = vec![ModuleRow {
        module: "src/deep/nested".into(),
        code: 50,
        lines: 70,
        files: 2,
        bytes: 2000,
        tokens: 500,
        avg_lines: 35,
    }];
    let report = module_report_with_rows(rows);
    let output = render_module(&report, TableFormat::Md);
    assert!(output.contains("src/deep/nested"));
}

// =========================================================================
// 6. CSV determinism
// =========================================================================

#[test]
fn csv_repeated_calls_identical() {
    let export = ExportData {
        rows: vec![
            FileRow {
                path: "a.rs".into(),
                module: "root".into(),
                lang: "Rust".into(),
                kind: FileKind::Parent,
                code: 10,
                comments: 2,
                blanks: 1,
                lines: 13,
                bytes: 500,
                tokens: 120,
            },
            FileRow {
                path: "b.py".into(),
                module: "root".into(),
                lang: "Python".into(),
                kind: FileKind::Parent,
                code: 20,
                comments: 4,
                blanks: 2,
                lines: 26,
                bytes: 1000,
                tokens: 240,
            },
        ],
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::ParentsOnly,
    };
    let a = render_csv(&export);
    let b = render_csv(&export);
    let c = render_csv(&export);
    assert_eq!(a, b);
    assert_eq!(b, c);
}

// =========================================================================
// 7. Diff rendering determinism
// =========================================================================

#[test]
fn diff_md_repeated_calls_identical() {
    let rows = vec![DiffRow {
        lang: "Rust".into(),
        old_code: 100,
        new_code: 200,
        delta_code: 100,
        old_lines: 150,
        new_lines: 300,
        delta_lines: 150,
        old_files: 5,
        new_files: 8,
        delta_files: 3,
        old_bytes: 4000,
        new_bytes: 8000,
        delta_bytes: 4000,
        old_tokens: 1000,
        new_tokens: 2000,
        delta_tokens: 1000,
    }];
    let totals = compute_diff_totals(&rows);
    let a = render_diff_md("v1", "v2", &rows, &totals);
    let b = render_diff_md("v1", "v2", &rows, &totals);
    assert_eq!(a, b);
}

#[test]
fn compute_diff_rows_deterministic_ordering() {
    let from = lang_report_with_rows(
        vec![
            LangRow {
                lang: "C".into(),
                code: 100,
                lines: 150,
                files: 3,
                bytes: 4000,
                tokens: 1000,
                avg_lines: 50,
            },
            LangRow {
                lang: "Rust".into(),
                code: 200,
                lines: 300,
                files: 5,
                bytes: 8000,
                tokens: 2000,
                avg_lines: 60,
            },
        ],
        true,
    );
    let to = lang_report_with_rows(
        vec![
            LangRow {
                lang: "Rust".into(),
                code: 300,
                lines: 400,
                files: 5,
                bytes: 12000,
                tokens: 3000,
                avg_lines: 80,
            },
            LangRow {
                lang: "C".into(),
                code: 150,
                lines: 200,
                files: 3,
                bytes: 6000,
                tokens: 1500,
                avg_lines: 67,
            },
        ],
        true,
    );
    let a = compute_diff_rows(&from, &to);
    let b = compute_diff_rows(&from, &to);
    assert_eq!(a.len(), b.len());
    for (ra, rb) in a.iter().zip(b.iter()) {
        assert_eq!(ra.lang, rb.lang);
        assert_eq!(ra.delta_code, rb.delta_code);
    }
    // Languages should be alphabetically ordered
    assert_eq!(a[0].lang, "C");
    assert_eq!(a[1].lang, "Rust");
}
