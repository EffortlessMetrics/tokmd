//! Deep snapshot tests for tokmd-format rendering edge cases.
//!
//! Complements `snapshots.rs` by covering:
//! - Embedded / "(embedded)" language rows
//! - ChildrenMode::Separate in lang reports
//! - Unicode language and module names
//! - Zero-value rows (all zeros)
//! - Deterministic ordering across repeated calls
//! - Module paths with forward-slash normalisation
//! - Many-module reports (MD / TSV / JSON)
//! - Export with FileKind::Child rows
//! - Export CSV with special characters in paths
//! - Diff with many languages
//! - CycloneDX with redacted paths

use std::path::PathBuf;

use tokmd_format::{
    compute_diff_rows, compute_diff_totals, render_diff_md, write_export_csv_to,
    write_export_cyclonedx_with_options, write_export_json_to, write_export_jsonl_to,
    write_lang_report_to, write_module_report_to,
};
use tokmd_settings::{ChildIncludeMode, ChildrenMode, ScanOptions};
use tokmd_types::{
    ExportArgs, ExportData, ExportFormat, FileKind, FileRow, LangArgs, LangReport, LangRow,
    ModuleArgs, ModuleReport, ModuleRow, RedactMode, TableFormat, Totals,
};

// ===========================================================================
// Helpers
// ===========================================================================

fn global() -> ScanOptions {
    ScanOptions::default()
}

fn lang_args(format: TableFormat) -> LangArgs {
    LangArgs {
        paths: vec![PathBuf::from(".")],
        format,
        top: 0,
        files: false,
        children: ChildrenMode::Collapse,
    }
}

fn module_args(format: TableFormat) -> ModuleArgs {
    ModuleArgs {
        paths: vec![PathBuf::from(".")],
        format,
        top: 0,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn export_args(format: ExportFormat) -> ExportArgs {
    ExportArgs {
        paths: vec![PathBuf::from(".")],
        format,
        output: None,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        redact: RedactMode::None,
        meta: false,
        strip_prefix: None,
    }
}

fn render_lang(report: &LangReport, format: TableFormat) -> String {
    let mut buf = Vec::new();
    let args = lang_args(format);
    write_lang_report_to(&mut buf, report, &global(), &args).unwrap();
    String::from_utf8(buf).unwrap()
}

fn render_module(report: &ModuleReport, format: TableFormat) -> String {
    let mut buf = Vec::new();
    let args = module_args(format);
    write_module_report_to(&mut buf, report, &global(), &args).unwrap();
    String::from_utf8(buf).unwrap()
}

/// Normalise non-deterministic JSON fields for snapshot stability.
fn normalise_json(raw: &str) -> String {
    let mut v: serde_json::Value = serde_json::from_str(raw).unwrap();
    if v.get("generated_at_ms").is_some() {
        v["generated_at_ms"] = serde_json::json!(0);
    }
    if v.pointer("/tool/version").is_some() {
        v["tool"]["version"] = serde_json::json!("0.0.0");
    }
    serde_json::to_string_pretty(&v).unwrap()
}

// ===========================================================================
// 1. Embedded language rows – "(embedded)" suffix
// ===========================================================================

fn embedded_lang_report() -> LangReport {
    LangReport {
        rows: vec![
            LangRow {
                lang: "HTML".into(),
                code: 400,
                lines: 500,
                files: 5,
                bytes: 20000,
                tokens: 1000,
                avg_lines: 100,
            },
            LangRow {
                lang: "JavaScript (embedded)".into(),
                code: 120,
                lines: 150,
                files: 5,
                bytes: 6000,
                tokens: 300,
                avg_lines: 30,
            },
            LangRow {
                lang: "CSS (embedded)".into(),
                code: 80,
                lines: 100,
                files: 3,
                bytes: 4000,
                tokens: 200,
                avg_lines: 33,
            },
        ],
        total: Totals {
            code: 600,
            lines: 750,
            files: 13,
            bytes: 30000,
            tokens: 1500,
            avg_lines: 58,
        },
        with_files: true,
        children: ChildrenMode::Separate,
        top: 0,
    }
}

#[test]
fn snapshot_lang_md_embedded_rows() {
    let output = render_lang(&embedded_lang_report(), TableFormat::Md);
    insta::assert_snapshot!("deep_lang_md_embedded", output);
}

#[test]
fn snapshot_lang_tsv_embedded_rows() {
    let output = render_lang(&embedded_lang_report(), TableFormat::Tsv);
    insta::assert_snapshot!("deep_lang_tsv_embedded", output);
}

#[test]
fn snapshot_lang_json_embedded_rows() {
    let mut buf = Vec::new();
    let args = LangArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Json,
        top: 0,
        files: true,
        children: ChildrenMode::Separate,
    };
    write_lang_report_to(&mut buf, &embedded_lang_report(), &global(), &args).unwrap();
    let pretty = normalise_json(&String::from_utf8(buf).unwrap());
    insta::assert_snapshot!("deep_lang_json_embedded", pretty);
}

// ===========================================================================
// 2. Unicode language and module names
// ===========================================================================

fn unicode_lang_report() -> LangReport {
    LangReport {
        rows: vec![
            LangRow {
                lang: "日本語".into(),
                code: 300,
                lines: 400,
                files: 3,
                bytes: 15000,
                tokens: 750,
                avg_lines: 133,
            },
            LangRow {
                lang: "Ελληνικά".into(),
                code: 200,
                lines: 250,
                files: 2,
                bytes: 10000,
                tokens: 500,
                avg_lines: 125,
            },
            LangRow {
                lang: "Ру́сский".into(),
                code: 100,
                lines: 120,
                files: 1,
                bytes: 5000,
                tokens: 250,
                avg_lines: 120,
            },
        ],
        total: Totals {
            code: 600,
            lines: 770,
            files: 6,
            bytes: 30000,
            tokens: 1500,
            avg_lines: 128,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

#[test]
fn snapshot_lang_md_unicode() {
    let output = render_lang(&unicode_lang_report(), TableFormat::Md);
    insta::assert_snapshot!("deep_lang_md_unicode", output);
}

#[test]
fn snapshot_lang_tsv_unicode() {
    let output = render_lang(&unicode_lang_report(), TableFormat::Tsv);
    insta::assert_snapshot!("deep_lang_tsv_unicode", output);
}

fn unicode_module_report() -> ModuleReport {
    ModuleReport {
        rows: vec![
            ModuleRow {
                module: "src/コンポーネント".into(),
                code: 250,
                lines: 300,
                files: 3,
                bytes: 12500,
                tokens: 625,
                avg_lines: 100,
            },
            ModuleRow {
                module: "src/données".into(),
                code: 150,
                lines: 180,
                files: 2,
                bytes: 7500,
                tokens: 375,
                avg_lines: 90,
            },
        ],
        total: Totals {
            code: 400,
            lines: 480,
            files: 5,
            bytes: 20000,
            tokens: 1000,
            avg_lines: 96,
        },
        module_roots: vec!["src".into()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
        top: 0,
    }
}

#[test]
fn snapshot_module_md_unicode() {
    let output = render_module(&unicode_module_report(), TableFormat::Md);
    insta::assert_snapshot!("deep_module_md_unicode", output);
}

#[test]
fn snapshot_module_tsv_unicode() {
    let output = render_module(&unicode_module_report(), TableFormat::Tsv);
    insta::assert_snapshot!("deep_module_tsv_unicode", output);
}

// ===========================================================================
// 3. Zero-value rows (all zeros)
// ===========================================================================

fn zero_value_lang_report() -> LangReport {
    LangReport {
        rows: vec![
            LangRow {
                lang: "Rust".into(),
                code: 0,
                lines: 0,
                files: 0,
                bytes: 0,
                tokens: 0,
                avg_lines: 0,
            },
            LangRow {
                lang: "Python".into(),
                code: 0,
                lines: 0,
                files: 0,
                bytes: 0,
                tokens: 0,
                avg_lines: 0,
            },
        ],
        total: Totals {
            code: 0,
            lines: 0,
            files: 0,
            bytes: 0,
            tokens: 0,
            avg_lines: 0,
        },
        with_files: true,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

#[test]
fn snapshot_lang_md_zero_values() {
    let output = render_lang(&zero_value_lang_report(), TableFormat::Md);
    insta::assert_snapshot!("deep_lang_md_zeros", output);
}

#[test]
fn snapshot_lang_tsv_zero_values() {
    let output = render_lang(&zero_value_lang_report(), TableFormat::Tsv);
    insta::assert_snapshot!("deep_lang_tsv_zeros", output);
}

fn zero_value_module_report() -> ModuleReport {
    ModuleReport {
        rows: vec![ModuleRow {
            module: "empty_mod".into(),
            code: 0,
            lines: 0,
            files: 0,
            bytes: 0,
            tokens: 0,
            avg_lines: 0,
        }],
        total: Totals {
            code: 0,
            lines: 0,
            files: 0,
            bytes: 0,
            tokens: 0,
            avg_lines: 0,
        },
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        top: 0,
    }
}

#[test]
fn snapshot_module_md_zero_values() {
    let output = render_module(&zero_value_module_report(), TableFormat::Md);
    insta::assert_snapshot!("deep_module_md_zeros", output);
}

#[test]
fn snapshot_module_tsv_zero_values() {
    let output = render_module(&zero_value_module_report(), TableFormat::Tsv);
    insta::assert_snapshot!("deep_module_tsv_zeros", output);
}

// ===========================================================================
// 4. Deterministic ordering – repeated calls produce identical output
// ===========================================================================

fn multi_lang_report() -> LangReport {
    LangReport {
        rows: vec![
            LangRow {
                lang: "Rust".into(),
                code: 500,
                lines: 600,
                files: 5,
                bytes: 25000,
                tokens: 1250,
                avg_lines: 120,
            },
            LangRow {
                lang: "Go".into(),
                code: 300,
                lines: 360,
                files: 3,
                bytes: 15000,
                tokens: 750,
                avg_lines: 120,
            },
            LangRow {
                lang: "Python".into(),
                code: 200,
                lines: 240,
                files: 2,
                bytes: 10000,
                tokens: 500,
                avg_lines: 120,
            },
        ],
        total: Totals {
            code: 1000,
            lines: 1200,
            files: 10,
            bytes: 50000,
            tokens: 2500,
            avg_lines: 120,
        },
        with_files: true,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

#[test]
fn determinism_lang_md_identical_across_calls() {
    let report = multi_lang_report();
    let a = render_lang(&report, TableFormat::Md);
    let b = render_lang(&report, TableFormat::Md);
    let c = render_lang(&report, TableFormat::Md);
    assert_eq!(a, b, "first and second calls must be identical");
    assert_eq!(b, c, "second and third calls must be identical");
}

#[test]
fn determinism_lang_tsv_identical_across_calls() {
    let report = multi_lang_report();
    let a = render_lang(&report, TableFormat::Tsv);
    let b = render_lang(&report, TableFormat::Tsv);
    assert_eq!(a, b);
}

#[test]
fn determinism_module_md_identical_across_calls() {
    let report = unicode_module_report();
    let a = render_module(&report, TableFormat::Md);
    let b = render_module(&report, TableFormat::Md);
    assert_eq!(a, b);
}

#[test]
fn determinism_module_tsv_identical_across_calls() {
    let report = unicode_module_report();
    let a = render_module(&report, TableFormat::Tsv);
    let b = render_module(&report, TableFormat::Tsv);
    assert_eq!(a, b);
}

#[test]
fn determinism_export_csv_identical_across_calls() {
    let data = ExportData {
        rows: sample_file_rows(),
        module_roots: vec!["src".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let args = export_args(ExportFormat::Csv);
    let render = || {
        let mut buf = Vec::new();
        write_export_csv_to(&mut buf, &data, &args).unwrap();
        String::from_utf8(buf).unwrap()
    };
    let a = render();
    let b = render();
    assert_eq!(a, b);
}

// ===========================================================================
// 5. Forward-slash path normalisation in module names
// ===========================================================================

#[test]
fn snapshot_module_md_forward_slash_paths() {
    let report = ModuleReport {
        rows: vec![
            ModuleRow {
                module: "crates/tokmd-format/src".into(),
                code: 400,
                lines: 500,
                files: 4,
                bytes: 20000,
                tokens: 1000,
                avg_lines: 125,
            },
            ModuleRow {
                module: "crates/tokmd-types/src".into(),
                code: 300,
                lines: 360,
                files: 3,
                bytes: 15000,
                tokens: 750,
                avg_lines: 120,
            },
        ],
        total: Totals {
            code: 700,
            lines: 860,
            files: 7,
            bytes: 35000,
            tokens: 1750,
            avg_lines: 123,
        },
        module_roots: vec!["crates".into()],
        module_depth: 3,
        children: ChildIncludeMode::Separate,
        top: 0,
    };
    let output = render_module(&report, TableFormat::Md);
    // All paths must use forward slashes
    assert!(
        !output.contains('\\'),
        "output must not contain backslashes"
    );
    insta::assert_snapshot!("deep_module_md_fwd_slash", output);
}

// ===========================================================================
// 6. Many-module reports
// ===========================================================================

fn many_module_report() -> ModuleReport {
    let rows: Vec<ModuleRow> = (0..10)
        .map(|i| ModuleRow {
            module: format!("crates/mod_{:02}", i),
            code: (10 - i) * 100,
            lines: (10 - i) * 120,
            files: 10 - i,
            bytes: (10 - i) * 5000,
            tokens: (10 - i) * 250,
            avg_lines: 120,
        })
        .collect();
    let total = Totals {
        code: rows.iter().map(|r| r.code).sum(),
        lines: rows.iter().map(|r| r.lines).sum(),
        files: rows.iter().map(|r| r.files).sum(),
        bytes: rows.iter().map(|r| r.bytes).sum(),
        tokens: rows.iter().map(|r| r.tokens).sum(),
        avg_lines: 120,
    };
    ModuleReport {
        rows,
        total,
        module_roots: vec!["crates".into()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
        top: 0,
    }
}

#[test]
fn snapshot_module_md_many() {
    let output = render_module(&many_module_report(), TableFormat::Md);
    insta::assert_snapshot!("deep_module_md_many", output);
}

#[test]
fn snapshot_module_tsv_many() {
    let output = render_module(&many_module_report(), TableFormat::Tsv);
    insta::assert_snapshot!("deep_module_tsv_many", output);
}

#[test]
fn snapshot_module_json_many() {
    let report = many_module_report();
    let mut buf = Vec::new();
    let args = ModuleArgs {
        paths: vec![PathBuf::from(".")],
        format: TableFormat::Json,
        top: 0,
        module_roots: vec!["crates".into()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };
    write_module_report_to(&mut buf, &report, &global(), &args).unwrap();
    let pretty = normalise_json(&String::from_utf8(buf).unwrap());
    insta::assert_snapshot!("deep_module_json_many", pretty);
}

// ===========================================================================
// 7. Export with FileKind::Child (embedded) rows
// ===========================================================================

fn sample_file_rows() -> Vec<FileRow> {
    vec![
        FileRow {
            path: "src/lib.rs".into(),
            module: "src".into(),
            lang: "Rust".into(),
            kind: FileKind::Parent,
            code: 200,
            comments: 40,
            blanks: 20,
            lines: 260,
            bytes: 10000,
            tokens: 500,
        },
        FileRow {
            path: "templates/index.html".into(),
            module: "templates".into(),
            lang: "HTML".into(),
            kind: FileKind::Parent,
            code: 80,
            comments: 5,
            blanks: 10,
            lines: 95,
            bytes: 4000,
            tokens: 200,
        },
        FileRow {
            path: "templates/index.html".into(),
            module: "templates".into(),
            lang: "JavaScript".into(),
            kind: FileKind::Child,
            code: 30,
            comments: 2,
            blanks: 3,
            lines: 35,
            bytes: 1500,
            tokens: 75,
        },
        FileRow {
            path: "templates/index.html".into(),
            module: "templates".into(),
            lang: "CSS".into(),
            kind: FileKind::Child,
            code: 20,
            comments: 1,
            blanks: 2,
            lines: 23,
            bytes: 1000,
            tokens: 50,
        },
    ]
}

#[test]
fn snapshot_export_csv_with_children() {
    let data = ExportData {
        rows: sample_file_rows(),
        module_roots: vec!["src".into(), "templates".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &data, &export_args(ExportFormat::Csv)).unwrap();
    let output = String::from_utf8(buf).unwrap();
    insta::assert_snapshot!("deep_export_csv_children", output);
}

#[test]
fn snapshot_export_jsonl_with_children() {
    let data = ExportData {
        rows: sample_file_rows(),
        module_roots: vec!["src".into(), "templates".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let mut buf = Vec::new();
    write_export_jsonl_to(
        &mut buf,
        &data,
        &global(),
        &export_args(ExportFormat::Jsonl),
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    insta::assert_snapshot!("deep_export_jsonl_children", output);
}

#[test]
fn snapshot_export_json_with_children() {
    let data = ExportData {
        rows: sample_file_rows(),
        module_roots: vec!["src".into(), "templates".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let mut buf = Vec::new();
    write_export_json_to(&mut buf, &data, &global(), &export_args(ExportFormat::Json)).unwrap();
    let raw = String::from_utf8(buf).unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let pretty = serde_json::to_string_pretty(&v).unwrap();
    insta::assert_snapshot!("deep_export_json_children", pretty);
}

// ===========================================================================
// 8. Export CSV with special characters in paths
// ===========================================================================

#[test]
fn snapshot_export_csv_special_chars() {
    let data = ExportData {
        rows: vec![
            FileRow {
                path: "src/my file, with commas.rs".into(),
                module: "src".into(),
                lang: "Rust".into(),
                kind: FileKind::Parent,
                code: 50,
                comments: 5,
                blanks: 3,
                lines: 58,
                bytes: 2500,
                tokens: 125,
            },
            FileRow {
                path: r#"src/has "quotes".rs"#.into(),
                module: "src".into(),
                lang: "Rust".into(),
                kind: FileKind::Parent,
                code: 30,
                comments: 2,
                blanks: 1,
                lines: 33,
                bytes: 1500,
                tokens: 75,
            },
        ],
        module_roots: vec!["src".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &data, &export_args(ExportFormat::Csv)).unwrap();
    let output = String::from_utf8(buf).unwrap();
    insta::assert_snapshot!("deep_export_csv_special_chars", output);
}

// ===========================================================================
// 9. CycloneDX with redacted paths
// ===========================================================================

#[test]
fn snapshot_cyclonedx_redacted() {
    let data = ExportData {
        rows: vec![FileRow {
            path: "src/secret/internal.rs".into(),
            module: "src/secret".into(),
            lang: "Rust".into(),
            kind: FileKind::Parent,
            code: 100,
            comments: 10,
            blanks: 5,
            lines: 115,
            bytes: 5000,
            tokens: 250,
        }],
        module_roots: vec!["src".into()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };
    let mut buf = Vec::new();
    write_export_cyclonedx_with_options(
        &mut buf,
        &data,
        RedactMode::Paths,
        Some("urn:uuid:00000000-0000-0000-0000-000000000000".into()),
        Some("1970-01-01T00:00:00Z".into()),
    )
    .unwrap();
    let raw = String::from_utf8(buf).unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    v["metadata"]["tools"][0]["version"] = serde_json::json!("0.0.0");
    let pretty = serde_json::to_string_pretty(&v).unwrap();
    insta::assert_snapshot!("deep_cyclonedx_redacted", pretty);
}

#[test]
fn snapshot_cyclonedx_with_children() {
    let data = ExportData {
        rows: sample_file_rows(),
        module_roots: vec!["src".into(), "templates".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let mut buf = Vec::new();
    write_export_cyclonedx_with_options(
        &mut buf,
        &data,
        RedactMode::None,
        Some("urn:uuid:00000000-0000-0000-0000-000000000000".into()),
        Some("1970-01-01T00:00:00Z".into()),
    )
    .unwrap();
    let raw = String::from_utf8(buf).unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    v["metadata"]["tools"][0]["version"] = serde_json::json!("0.0.0");
    let pretty = serde_json::to_string_pretty(&v).unwrap();
    insta::assert_snapshot!("deep_cyclonedx_children", pretty);
}

// ===========================================================================
// 10. Diff with many languages (added, removed, modified)
// ===========================================================================

#[test]
fn snapshot_diff_many_languages() {
    let from = LangReport {
        rows: vec![
            LangRow {
                lang: "C".into(),
                code: 1000,
                lines: 1200,
                files: 10,
                bytes: 50000,
                tokens: 2500,
                avg_lines: 120,
            },
            LangRow {
                lang: "C++".into(),
                code: 800,
                lines: 960,
                files: 8,
                bytes: 40000,
                tokens: 2000,
                avg_lines: 120,
            },
            LangRow {
                lang: "Makefile".into(),
                code: 50,
                lines: 60,
                files: 2,
                bytes: 2500,
                tokens: 125,
                avg_lines: 30,
            },
            LangRow {
                lang: "Shell".into(),
                code: 30,
                lines: 36,
                files: 1,
                bytes: 1500,
                tokens: 75,
                avg_lines: 36,
            },
        ],
        total: Totals {
            code: 1880,
            lines: 2256,
            files: 21,
            bytes: 94000,
            tokens: 4700,
            avg_lines: 107,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let to = LangReport {
        rows: vec![
            LangRow {
                lang: "C".into(),
                code: 1200,
                lines: 1440,
                files: 12,
                bytes: 60000,
                tokens: 3000,
                avg_lines: 120,
            },
            LangRow {
                lang: "Rust".into(),
                code: 500,
                lines: 600,
                files: 5,
                bytes: 25000,
                tokens: 1250,
                avg_lines: 120,
            },
            LangRow {
                lang: "Makefile".into(),
                code: 50,
                lines: 60,
                files: 2,
                bytes: 2500,
                tokens: 125,
                avg_lines: 30,
            },
            LangRow {
                lang: "CMake".into(),
                code: 40,
                lines: 48,
                files: 1,
                bytes: 2000,
                tokens: 100,
                avg_lines: 48,
            },
        ],
        total: Totals {
            code: 1790,
            lines: 2148,
            files: 20,
            bytes: 89500,
            tokens: 4475,
            avg_lines: 107,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let md = render_diff_md("v1.0", "v2.0", &rows, &totals);
    insta::assert_snapshot!("deep_diff_many_langs", md);
}

// ===========================================================================
// 11. Lang report with_files=true and Separate children mode
// ===========================================================================

#[test]
fn snapshot_lang_md_with_files_separate() {
    let mut args = lang_args(TableFormat::Md);
    args.files = true;
    args.children = ChildrenMode::Separate;
    let report = embedded_lang_report();
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &global(), &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    insta::assert_snapshot!("deep_lang_md_with_files_separate", output);
}

// ===========================================================================
// 12. Export empty with all formats
// ===========================================================================

fn empty_export_data() -> ExportData {
    ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

#[test]
fn snapshot_export_jsonl_empty() {
    let mut buf = Vec::new();
    write_export_jsonl_to(
        &mut buf,
        &empty_export_data(),
        &global(),
        &export_args(ExportFormat::Jsonl),
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    insta::assert_snapshot!("deep_export_jsonl_empty", output);
}

#[test]
fn snapshot_export_json_empty() {
    let mut buf = Vec::new();
    write_export_json_to(
        &mut buf,
        &empty_export_data(),
        &global(),
        &export_args(ExportFormat::Json),
    )
    .unwrap();
    let raw = String::from_utf8(buf).unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let pretty = serde_json::to_string_pretty(&v).unwrap();
    insta::assert_snapshot!("deep_export_json_empty", pretty);
}

#[test]
fn snapshot_cyclonedx_empty() {
    let mut buf = Vec::new();
    write_export_cyclonedx_with_options(
        &mut buf,
        &empty_export_data(),
        RedactMode::None,
        Some("urn:uuid:00000000-0000-0000-0000-000000000000".into()),
        Some("1970-01-01T00:00:00Z".into()),
    )
    .unwrap();
    let raw = String::from_utf8(buf).unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    v["metadata"]["tools"][0]["version"] = serde_json::json!("0.0.0");
    let pretty = serde_json::to_string_pretty(&v).unwrap();
    insta::assert_snapshot!("deep_cyclonedx_empty", pretty);
}

// ===========================================================================
// 13. Diff — all languages removed (empty target)
// ===========================================================================

#[test]
fn snapshot_diff_all_removed() {
    let from = multi_lang_report();
    let to = LangReport {
        rows: vec![],
        total: Totals {
            code: 0,
            lines: 0,
            files: 0,
            bytes: 0,
            tokens: 0,
            avg_lines: 0,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let md = render_diff_md("v1.0", "empty", &rows, &totals);
    insta::assert_snapshot!("deep_diff_all_removed", md);
}

#[test]
fn snapshot_diff_all_added() {
    let from = LangReport {
        rows: vec![],
        total: Totals {
            code: 0,
            lines: 0,
            files: 0,
            bytes: 0,
            tokens: 0,
            avg_lines: 0,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let to = multi_lang_report();
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let md = render_diff_md("empty", "v1.0", &rows, &totals);
    insta::assert_snapshot!("deep_diff_all_added", md);
}

// ===========================================================================
// 14. Export with RedactMode::All
// ===========================================================================

#[test]
fn snapshot_export_csv_redact_all() {
    let data = ExportData {
        rows: sample_file_rows(),
        module_roots: vec!["src".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let mut args = export_args(ExportFormat::Csv);
    args.redact = RedactMode::All;
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &data, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    // Redacted output should not contain original paths
    assert!(
        !output.contains("src/lib.rs"),
        "redacted CSV must not contain raw paths"
    );
    assert!(
        !output.contains("templates/index.html"),
        "redacted CSV must not contain raw paths"
    );
    insta::assert_snapshot!("deep_export_csv_redact_all", output);
}

// ===========================================================================
// 15. Lang with_files=true, many rows — verify file/avg columns present
// ===========================================================================

#[test]
fn snapshot_lang_md_many_with_files() {
    let rows: Vec<LangRow> = ["Rust", "Python", "Go", "Java", "TypeScript"]
        .iter()
        .enumerate()
        .map(|(i, lang)| LangRow {
            lang: lang.to_string(),
            code: (5 - i) * 200,
            lines: (5 - i) * 240,
            files: (5 - i) * 2,
            bytes: (5 - i) * 10000,
            tokens: (5 - i) * 500,
            avg_lines: 120,
        })
        .collect();
    let total = Totals {
        code: rows.iter().map(|r| r.code).sum(),
        lines: rows.iter().map(|r| r.lines).sum(),
        files: rows.iter().map(|r| r.files).sum(),
        bytes: rows.iter().map(|r| r.bytes).sum(),
        tokens: rows.iter().map(|r| r.tokens).sum(),
        avg_lines: 120,
    };
    let report = LangReport {
        rows,
        total,
        with_files: true,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let mut buf = Vec::new();
    let mut args = lang_args(TableFormat::Md);
    args.files = true;
    write_lang_report_to(&mut buf, &report, &global(), &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    // Verify Files and Avg columns are present
    assert!(
        output.contains("|Files|"),
        "with_files=true must show Files column"
    );
    assert!(
        output.contains("|Avg|"),
        "with_files=true must show Avg column"
    );
    insta::assert_snapshot!("deep_lang_md_many_with_files", output);
}
