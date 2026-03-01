//! Additional format snapshot tests covering edge cases, mixed-format
//! consistency, top-N truncation, export metadata lines, and diff
//! with negative deltas.
//!
//! Supplements `snapshots.rs` and `snapshot_coverage.rs`.

use std::path::PathBuf;

use tokmd_format::{
    DiffColorMode, DiffRenderOptions, compute_diff_rows, compute_diff_totals, create_diff_receipt,
    render_diff_md_with_options, write_export_csv_to, write_export_json_to, write_export_jsonl_to,
    write_lang_report_to, write_module_report_to,
};
use tokmd_settings::{ChildIncludeMode, ChildrenMode, ScanOptions};
use tokmd_types::{
    ExportArgs, ExportData, ExportFormat, FileKind, FileRow, LangArgs, LangReport, LangRow,
    ModuleArgs, ModuleReport, ModuleRow, RedactMode, TableFormat, Totals,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn global() -> ScanOptions {
    ScanOptions::default()
}

fn make_totals(rows_code: &[usize], rows_lines: &[usize], rows_files: &[usize]) -> Totals {
    let code: usize = rows_code.iter().sum();
    let lines: usize = rows_lines.iter().sum();
    let files: usize = rows_files.iter().sum();
    Totals {
        code,
        lines,
        files,
        bytes: code * 50,
        tokens: code * 5 / 2,
        avg_lines: if files > 0 { lines / files } else { 0 },
    }
}

fn lang_row(lang: &str, code: usize) -> LangRow {
    LangRow {
        lang: lang.into(),
        code,
        lines: code + code / 5,
        files: std::cmp::max(1, code / 100),
        bytes: code * 50,
        tokens: code * 5 / 2,
        avg_lines: if code > 0 {
            (code + code / 5) / std::cmp::max(1, code / 100)
        } else {
            0
        },
    }
}

fn module_row(module: &str, code: usize) -> ModuleRow {
    ModuleRow {
        module: module.into(),
        code,
        lines: code + code / 5,
        files: std::cmp::max(1, code / 100),
        bytes: code * 50,
        tokens: code * 5 / 2,
        avg_lines: if code > 0 {
            (code + code / 5) / std::cmp::max(1, code / 100)
        } else {
            0
        },
    }
}

fn file_row(path: &str, lang: &str, kind: FileKind, code: usize) -> FileRow {
    FileRow {
        path: path.into(),
        module: path.rsplit_once('/').map_or(".", |p| p.0).into(),
        lang: lang.into(),
        kind,
        code,
        comments: code / 5,
        blanks: code / 10,
        lines: code + code / 5 + code / 10,
        bytes: code * 50,
        tokens: code * 5 / 2,
    }
}

fn report_from_lang_rows(
    rows: Vec<LangRow>,
    with_files: bool,
    children: ChildrenMode,
) -> LangReport {
    let codes: Vec<usize> = rows.iter().map(|r| r.code).collect();
    let lines: Vec<usize> = rows.iter().map(|r| r.lines).collect();
    let files: Vec<usize> = rows.iter().map(|r| r.files).collect();
    LangReport {
        rows,
        total: make_totals(&codes, &lines, &files),
        with_files,
        children,
        top: 0,
    }
}

fn report_from_module_rows(rows: Vec<ModuleRow>, children: ChildIncludeMode) -> ModuleReport {
    let codes: Vec<usize> = rows.iter().map(|r| r.code).collect();
    let lines: Vec<usize> = rows.iter().map(|r| r.lines).collect();
    let files: Vec<usize> = rows.iter().map(|r| r.files).collect();
    ModuleReport {
        rows,
        total: make_totals(&codes, &lines, &files),
        module_roots: vec!["src".into()],
        module_depth: 2,
        children,
        top: 0,
    }
}

fn normalize_json(raw: &str) -> String {
    let mut v: serde_json::Value = serde_json::from_str(raw).unwrap();
    v["generated_at_ms"] = serde_json::json!(0);
    v["tool"]["version"] = serde_json::json!("0.0.0");
    serde_json::to_string_pretty(&v).unwrap()
}

// =========================================================================
// 1. Lang top-N truncation
// =========================================================================

#[test]
fn snapshot_lang_md_top_2() {
    let report = LangReport {
        top: 2,
        ..report_from_lang_rows(
            vec![
                lang_row("Rust", 3000),
                lang_row("Python", 1500),
                lang_row("Go", 500),
                lang_row("TOML", 100),
            ],
            false,
            ChildrenMode::Collapse,
        )
    };
    let mut buf = Vec::new();
    let args = LangArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Md,
        top: 2,
        files: false,
        children: ChildrenMode::Collapse,
    };
    write_lang_report_to(&mut buf, &report, &global(), &args).unwrap();
    insta::assert_snapshot!("lang_md_top_2", String::from_utf8(buf).unwrap());
}

#[test]
fn snapshot_lang_tsv_top_2() {
    let report = LangReport {
        top: 2,
        ..report_from_lang_rows(
            vec![
                lang_row("Rust", 3000),
                lang_row("Python", 1500),
                lang_row("Go", 500),
                lang_row("TOML", 100),
            ],
            false,
            ChildrenMode::Collapse,
        )
    };
    let mut buf = Vec::new();
    let args = LangArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Tsv,
        top: 2,
        files: false,
        children: ChildrenMode::Collapse,
    };
    write_lang_report_to(&mut buf, &report, &global(), &args).unwrap();
    insta::assert_snapshot!("lang_tsv_top_2", String::from_utf8(buf).unwrap());
}

// =========================================================================
// 2. Module — top-N and JSON envelope with parents-only
// =========================================================================

#[test]
fn snapshot_module_md_top_1() {
    let report = ModuleReport {
        top: 1,
        ..report_from_module_rows(
            vec![
                module_row("src/core", 1200),
                module_row("src/api", 600),
                module_row("src/util", 200),
            ],
            ChildIncludeMode::ParentsOnly,
        )
    };
    let mut buf = Vec::new();
    let args = ModuleArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Md,
        top: 1,
        module_roots: vec!["src".into()],
        module_depth: 2,
        children: ChildIncludeMode::ParentsOnly,
    };
    write_module_report_to(&mut buf, &report, &global(), &args).unwrap();
    insta::assert_snapshot!("module_md_top_1", String::from_utf8(buf).unwrap());
}

#[test]
fn snapshot_module_json_parents_only() {
    let report = report_from_module_rows(
        vec![module_row("src/core", 800), module_row("src/api", 400)],
        ChildIncludeMode::ParentsOnly,
    );
    let mut buf = Vec::new();
    let args = ModuleArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Json,
        top: 0,
        module_roots: vec!["src".into()],
        module_depth: 2,
        children: ChildIncludeMode::ParentsOnly,
    };
    write_module_report_to(&mut buf, &report, &global(), &args).unwrap();
    let raw = String::from_utf8(buf).unwrap();
    insta::assert_snapshot!("module_json_parents_only", normalize_json(&raw));
}

// =========================================================================
// 3. Export — min_code filtering and max_rows
// =========================================================================

#[test]
fn snapshot_export_csv_min_code() {
    let rows = vec![
        file_row("src/big.rs", "Rust", FileKind::Parent, 500),
        file_row("src/tiny.rs", "Rust", FileKind::Parent, 5),
        file_row("src/medium.rs", "Rust", FileKind::Parent, 100),
    ];
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
        min_code: 50,
        max_rows: 0,
        redact: RedactMode::None,
        meta: false,
        strip_prefix: None,
    };
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &data, &args).unwrap();
    insta::assert_snapshot!("export_csv_min_code_50", String::from_utf8(buf).unwrap());
}

#[test]
fn snapshot_export_jsonl_max_rows() {
    let rows = vec![
        file_row("src/a.rs", "Rust", FileKind::Parent, 300),
        file_row("src/b.rs", "Rust", FileKind::Parent, 200),
        file_row("src/c.rs", "Rust", FileKind::Parent, 100),
        file_row("src/d.rs", "Rust", FileKind::Parent, 50),
    ];
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
        max_rows: 2,
        redact: RedactMode::None,
        meta: false,
        strip_prefix: None,
    };
    let mut buf = Vec::new();
    write_export_jsonl_to(&mut buf, &data, &global(), &args).unwrap();
    insta::assert_snapshot!("export_jsonl_max_rows_2", String::from_utf8(buf).unwrap());
}

// =========================================================================
// 4. Export — JSON with meta envelope
// =========================================================================

#[test]
fn snapshot_export_json_with_meta() {
    let rows = vec![file_row("src/lib.rs", "Rust", FileKind::Parent, 200)];
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
        meta: true,
        strip_prefix: None,
    };
    let mut buf = Vec::new();
    write_export_json_to(&mut buf, &data, &global(), &args).unwrap();
    let raw = String::from_utf8(buf).unwrap();
    insta::assert_snapshot!("export_json_with_meta", normalize_json(&raw));
}

// =========================================================================
// 5. Diff — negative deltas (shrinking codebase)
// =========================================================================

#[test]
fn snapshot_diff_md_shrinking() {
    let from = report_from_lang_rows(
        vec![lang_row("Rust", 2000), lang_row("Python", 1000)],
        false,
        ChildrenMode::Collapse,
    );
    let to = report_from_lang_rows(vec![lang_row("Rust", 1500)], false, ChildrenMode::Collapse);
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let md = render_diff_md_with_options(
        "v2.0.0",
        "v3.0.0",
        &rows,
        &totals,
        DiffRenderOptions::default(),
    );
    insta::assert_snapshot!("diff_md_shrinking", md);
}

#[test]
fn snapshot_diff_md_no_change() {
    let report = report_from_lang_rows(vec![lang_row("Rust", 1000)], false, ChildrenMode::Collapse);
    let rows = compute_diff_rows(&report, &report);
    let totals = compute_diff_totals(&rows);
    let md =
        render_diff_md_with_options("same", "same", &rows, &totals, DiffRenderOptions::default());
    insta::assert_snapshot!("diff_md_no_change", md);
}

#[test]
fn snapshot_diff_md_empty() {
    let empty = report_from_lang_rows(vec![], false, ChildrenMode::Collapse);
    let rows = compute_diff_rows(&empty, &empty);
    let totals = compute_diff_totals(&rows);
    let md = render_diff_md_with_options(
        "empty-a",
        "empty-b",
        &rows,
        &totals,
        DiffRenderOptions::default(),
    );
    insta::assert_snapshot!("diff_md_empty", md);
}

// =========================================================================
// 6. Diff — JSON receipt with negative deltas (timestamp-redacted)
// =========================================================================

#[test]
fn snapshot_diff_json_shrinking() {
    let from = report_from_lang_rows(
        vec![lang_row("Rust", 2000), lang_row("Go", 800)],
        false,
        ChildrenMode::Collapse,
    );
    let to = report_from_lang_rows(vec![lang_row("Rust", 1500)], false, ChildrenMode::Collapse);
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let receipt = create_diff_receipt("v1", "v2", rows, totals);
    let raw = serde_json::to_string(&receipt).unwrap();
    insta::assert_snapshot!("diff_json_shrinking", normalize_json(&raw));
}

// =========================================================================
// 7. Multi-language mix with child rows in export
// =========================================================================

#[test]
fn snapshot_export_csv_multi_lang_children() {
    let rows = vec![
        file_row("src/app.rs", "Rust", FileKind::Parent, 400),
        file_row("src/app.rs", "Markdown (Rust)", FileKind::Child, 50),
        file_row("web/index.html", "HTML", FileKind::Parent, 120),
        file_row("web/index.html", "JavaScript", FileKind::Child, 80),
        file_row("web/index.html", "CSS", FileKind::Child, 30),
    ];
    let data = ExportData {
        rows,
        module_roots: vec!["src".into(), "web".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let args = ExportArgs {
        paths: vec![PathBuf::from(".")],
        format: ExportFormat::Csv,
        output: None,
        module_roots: vec!["src".into(), "web".into()],
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
    insta::assert_snapshot!(
        "export_csv_multi_lang_children",
        String::from_utf8(buf).unwrap()
    );
}

// =========================================================================
// 8. Diff — compact with many languages (breadth)
// =========================================================================

#[test]
fn snapshot_diff_md_compact_many_langs() {
    let from = report_from_lang_rows(
        vec![
            lang_row("Rust", 2000),
            lang_row("Python", 1000),
            lang_row("Go", 500),
            lang_row("Java", 300),
        ],
        false,
        ChildrenMode::Collapse,
    );
    let to = report_from_lang_rows(
        vec![
            lang_row("Rust", 2500),
            lang_row("Python", 800),
            lang_row("TypeScript", 400),
        ],
        false,
        ChildrenMode::Collapse,
    );
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let md = render_diff_md_with_options(
        "v1",
        "v2",
        &rows,
        &totals,
        DiffRenderOptions {
            compact: true,
            color: DiffColorMode::Off,
        },
    );
    insta::assert_snapshot!("diff_md_compact_many_langs", md);
}
