//! Deep edge-case tests for tokmd-format – wave 63.
//!
//! ~80 tests covering:
//! - Markdown table alignment with varying column widths
//! - TSV output with special characters (tabs in data, newlines)
//! - JSON output with nested structures
//! - JSONL output with many rows
//! - CSV output with escaping (commas, quotes in data)
//! - CycloneDX SBOM format correctness
//! - Diff rendering with zero/negative/large changes
//! - Empty data for every format
//! - Unicode in language names and file paths
//! - Very long path truncation behavior
//! - Determinism: same data → same formatted output for all formats
//! - Property tests: all JSON output parses back as valid JSON
//! - Snapshot tests with insta for key format outputs
//!
//! Run with: `cargo test -p tokmd-format --test format_depth_w63`

use std::path::PathBuf;

use proptest::prelude::*;

use tokmd_format::{
    DiffColorMode, DiffRenderOptions, compute_diff_rows, compute_diff_totals, create_diff_receipt,
    render_diff_md, render_diff_md_with_options, write_export_csv_to,
    write_export_cyclonedx_with_options, write_export_json_to, write_export_jsonl_to,
    write_lang_report_to, write_module_report_to,
};
use tokmd_settings::{ChildIncludeMode, ChildrenMode, ScanOptions};
use tokmd_types::{
    ExportArgs, ExportData, ExportFormat, FileKind, FileRow, LangArgs, LangReport, LangRow,
    ModuleArgs, ModuleReport, ModuleRow, RedactMode, TableFormat, Totals,
};

// ============================================================================
// Helpers
// ============================================================================

fn lang_report(rows: Vec<LangRow>, with_files: bool) -> LangReport {
    let total = Totals {
        code: rows.iter().map(|r| r.code).sum(),
        lines: rows.iter().map(|r| r.lines).sum(),
        files: rows.iter().map(|r| r.files).sum(),
        bytes: rows.iter().map(|r| r.bytes).sum(),
        tokens: rows.iter().map(|r| r.tokens).sum(),
        avg_lines: if rows.is_empty() {
            0
        } else {
            rows.iter().map(|r| r.lines).sum::<usize>() / rows.len()
        },
    };
    LangReport {
        rows,
        total,
        with_files,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

fn module_report(rows: Vec<ModuleRow>) -> ModuleReport {
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
        module_roots: vec!["src".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        top: 0,
    }
}

fn export_data(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec!["src".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
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
        module_roots: vec!["src".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn export_args(format: ExportFormat) -> ExportArgs {
    ExportArgs {
        paths: vec![PathBuf::from(".")],
        format,
        output: None,
        module_roots: vec!["src".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        redact: RedactMode::None,
        meta: true,
        strip_prefix: None,
    }
}

fn export_args_no_meta(format: ExportFormat) -> ExportArgs {
    ExportArgs {
        meta: false,
        ..export_args(format)
    }
}

fn globals() -> ScanOptions {
    ScanOptions::default()
}

fn make_lang_row(lang: &str, code: usize) -> LangRow {
    LangRow {
        lang: lang.into(),
        code,
        lines: code + code / 5,
        files: (code / 100).max(1),
        bytes: code * 40,
        tokens: code * 2,
        avg_lines: if code > 0 { 120 } else { 0 },
    }
}

fn make_file_row(path: &str, lang: &str, code: usize) -> FileRow {
    FileRow {
        path: path.into(),
        module: path.rsplit_once('/').map(|(m, _)| m).unwrap_or(".").into(),
        lang: lang.into(),
        kind: FileKind::Parent,
        code,
        comments: code / 5,
        blanks: code / 10,
        lines: code + code / 5 + code / 10,
        bytes: code * 40,
        tokens: code * 2,
    }
}

fn make_module_row(module: &str, code: usize) -> ModuleRow {
    ModuleRow {
        module: module.into(),
        code,
        lines: code + code / 5,
        files: (code / 100).max(1),
        bytes: code * 40,
        tokens: code * 2,
        avg_lines: if code > 0 { 120 } else { 0 },
    }
}

fn render_to_string<F>(f: F) -> String
where
    F: FnOnce(&mut Vec<u8>) -> anyhow::Result<()>,
{
    let mut buf = Vec::new();
    f(&mut buf).expect("render failed");
    String::from_utf8(buf).expect("invalid utf-8")
}

// ============================================================================
// 1. Markdown table alignment with varying column widths
// ============================================================================

#[test]
fn md_lang_single_digit_values() {
    let report = lang_report(vec![make_lang_row("C", 1)], true);
    let out = render_to_string(|buf| {
        write_lang_report_to(buf, &report, &globals(), &lang_args(TableFormat::Md))
    });
    assert!(out.contains("|C|1|"));
    assert!(out.starts_with('|'));
}

#[test]
fn md_lang_large_values_aligned() {
    let report = lang_report(
        vec![make_lang_row("Rust", 999_999), make_lang_row("C", 1)],
        true,
    );
    let out = render_to_string(|buf| {
        write_lang_report_to(buf, &report, &globals(), &lang_args(TableFormat::Md))
    });
    assert!(out.contains("|Rust|999999|"));
    assert!(out.contains("|C|1|"));
    // Separator has right-alignment markers
    assert!(out.contains("---:"));
}

#[test]
fn md_module_separator_row_present() {
    let report = module_report(vec![make_module_row("src", 100)]);
    let out = render_to_string(|buf| {
        write_module_report_to(buf, &report, &globals(), &module_args(TableFormat::Md))
    });
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines.len() >= 3);
    assert!(lines[1].starts_with("|---"));
}

#[test]
fn md_lang_total_row_bold() {
    let report = lang_report(vec![make_lang_row("Go", 500)], true);
    let out = render_to_string(|buf| {
        write_lang_report_to(buf, &report, &globals(), &lang_args(TableFormat::Md))
    });
    assert!(out.contains("|**Total**|"));
}

#[test]
fn md_module_total_row_bold() {
    let report = module_report(vec![make_module_row("lib", 200)]);
    let out = render_to_string(|buf| {
        write_module_report_to(buf, &report, &globals(), &module_args(TableFormat::Md))
    });
    assert!(out.contains("|**Total**|"));
}

// ============================================================================
// 2. TSV output with special characters
// ============================================================================

#[test]
fn tsv_lang_tab_count_without_files() {
    let args = LangArgs {
        files: false,
        ..lang_args(TableFormat::Tsv)
    };
    let report = lang_report(vec![make_lang_row("Rust", 100)], false);
    let out = render_to_string(|buf| write_lang_report_to(buf, &report, &globals(), &args));
    // Without files: Lang, Code, Lines, Bytes, Tokens = 4 tabs per line
    for line in out.lines() {
        assert_eq!(line.matches('\t').count(), 4, "line: {}", line);
    }
}

#[test]
fn tsv_lang_tab_count_with_files() {
    let report = lang_report(vec![make_lang_row("Python", 200)], true);
    let out = render_to_string(|buf| {
        write_lang_report_to(buf, &report, &globals(), &lang_args(TableFormat::Tsv))
    });
    // With files: Lang, Code, Lines, Files, Bytes, Tokens, Avg = 6 tabs per line
    for line in out.lines() {
        assert_eq!(line.matches('\t').count(), 6, "line: {}", line);
    }
}

#[test]
fn tsv_module_tab_count() {
    let report = module_report(vec![make_module_row("mod_a", 100)]);
    let out = render_to_string(|buf| {
        write_module_report_to(buf, &report, &globals(), &module_args(TableFormat::Tsv))
    });
    for line in out.lines() {
        assert_eq!(line.matches('\t').count(), 6, "line: {}", line);
    }
}

#[test]
fn tsv_no_trailing_tab() {
    let report = lang_report(vec![make_lang_row("Rust", 100)], true);
    let out = render_to_string(|buf| {
        write_lang_report_to(buf, &report, &globals(), &lang_args(TableFormat::Tsv))
    });
    for line in out.lines() {
        assert!(!line.ends_with('\t'), "trailing tab in: {}", line);
    }
}

#[test]
fn tsv_header_matches_data_columns() {
    let report = lang_report(
        vec![make_lang_row("Rust", 100), make_lang_row("Go", 50)],
        true,
    );
    let out = render_to_string(|buf| {
        write_lang_report_to(buf, &report, &globals(), &lang_args(TableFormat::Tsv))
    });
    let lines: Vec<&str> = out.lines().collect();
    let header_cols = lines[0].split('\t').count();
    for line in &lines[1..] {
        assert_eq!(
            line.split('\t').count(),
            header_cols,
            "column mismatch: {}",
            line
        );
    }
}

// ============================================================================
// 3. JSON output with nested structures
// ============================================================================

#[test]
fn json_lang_receipt_valid() {
    let report = lang_report(vec![make_lang_row("Rust", 500)], true);
    let out = render_to_string(|buf| {
        write_lang_report_to(buf, &report, &globals(), &lang_args(TableFormat::Json))
    });
    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("invalid JSON");
    assert_eq!(v["mode"], "lang");
    assert!(v["schema_version"].is_number());
    // LangReport is #[serde(flatten)] so rows are at top level
    assert!(v["rows"].is_array());
    assert_eq!(v["rows"][0]["lang"], "Rust");
}

#[test]
fn json_module_receipt_valid() {
    let report = module_report(vec![make_module_row("core", 300)]);
    let out = render_to_string(|buf| {
        write_module_report_to(buf, &report, &globals(), &module_args(TableFormat::Json))
    });
    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("invalid JSON");
    assert_eq!(v["mode"], "module");
    // ModuleReport is #[serde(flatten)] so rows are at top level
    assert!(v["rows"].is_array());
}

#[test]
fn json_lang_receipt_has_tool_info() {
    let report = lang_report(vec![make_lang_row("Rust", 100)], false);
    let out = render_to_string(|buf| {
        write_lang_report_to(buf, &report, &globals(), &lang_args(TableFormat::Json))
    });
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert!(v["tool"]["name"].is_string());
    assert!(v["tool"]["version"].is_string());
}

#[test]
fn json_export_receipt_has_data_array() {
    let data = export_data(vec![make_file_row("src/main.rs", "Rust", 100)]);
    let out = render_to_string(|buf| {
        write_export_json_to(buf, &data, &globals(), &export_args(ExportFormat::Json))
    });
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    // ExportData is #[serde(flatten)] so rows are at top level
    assert!(v["rows"].is_array());
    assert_eq!(v["rows"].as_array().unwrap().len(), 1);
}

#[test]
fn json_export_no_meta_is_array() {
    let data = export_data(vec![make_file_row("src/lib.rs", "Rust", 50)]);
    let out = render_to_string(|buf| {
        write_export_json_to(
            buf,
            &data,
            &globals(),
            &export_args_no_meta(ExportFormat::Json),
        )
    });
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert!(v.is_array(), "without meta should be a bare array");
}

// ============================================================================
// 4. JSONL output with many rows
// ============================================================================

#[test]
fn jsonl_each_line_valid_json() {
    let rows: Vec<FileRow> = (0..20)
        .map(|i| make_file_row(&format!("src/f{}.rs", i), "Rust", 10 + i))
        .collect();
    let data = export_data(rows);
    let out = render_to_string(|buf| {
        write_export_jsonl_to(buf, &data, &globals(), &export_args(ExportFormat::Jsonl))
    });
    for line in out.lines() {
        let _: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("line not valid JSON: {e}\n{line}"));
    }
}

#[test]
fn jsonl_first_line_is_meta() {
    let data = export_data(vec![make_file_row("a.rs", "Rust", 10)]);
    let out = render_to_string(|buf| {
        write_export_jsonl_to(buf, &data, &globals(), &export_args(ExportFormat::Jsonl))
    });
    let first: serde_json::Value = serde_json::from_str(out.lines().next().unwrap()).unwrap();
    assert_eq!(first["type"], "meta");
}

#[test]
fn jsonl_row_lines_have_type_row() {
    let data = export_data(vec![
        make_file_row("a.rs", "Rust", 10),
        make_file_row("b.py", "Python", 20),
    ]);
    let out = render_to_string(|buf| {
        write_export_jsonl_to(buf, &data, &globals(), &export_args(ExportFormat::Jsonl))
    });
    for line in out.lines().skip(1) {
        let v: serde_json::Value = serde_json::from_str(line).unwrap();
        assert_eq!(v["type"], "row");
    }
}

#[test]
fn jsonl_no_meta_skips_first_line() {
    let data = export_data(vec![make_file_row("a.rs", "Rust", 10)]);
    let out = render_to_string(|buf| {
        write_export_jsonl_to(
            buf,
            &data,
            &globals(),
            &export_args_no_meta(ExportFormat::Jsonl),
        )
    });
    // Only row lines, no meta
    assert_eq!(out.lines().count(), 1);
    let v: serde_json::Value = serde_json::from_str(out.lines().next().unwrap()).unwrap();
    assert_eq!(v["type"], "row");
}

#[test]
fn jsonl_100_rows_correct_count() {
    let rows: Vec<FileRow> = (0..100)
        .map(|i| make_file_row(&format!("src/f{}.rs", i), "Rust", i + 1))
        .collect();
    let data = export_data(rows);
    let out = render_to_string(|buf| {
        write_export_jsonl_to(buf, &data, &globals(), &export_args(ExportFormat::Jsonl))
    });
    // 1 meta + 100 row lines
    assert_eq!(out.lines().count(), 101);
}

// ============================================================================
// 5. CSV output with escaping
// ============================================================================

#[test]
fn csv_header_columns() {
    let data = export_data(vec![make_file_row("a.rs", "Rust", 10)]);
    let out =
        render_to_string(|buf| write_export_csv_to(buf, &data, &export_args(ExportFormat::Csv)));
    let header = out.lines().next().unwrap();
    assert_eq!(
        header,
        "path,module,lang,kind,code,comments,blanks,lines,bytes,tokens"
    );
}

#[test]
fn csv_commas_in_path_are_quoted() {
    let row = FileRow {
        path: "src/hello,world.rs".into(),
        module: "src".into(),
        lang: "Rust".into(),
        kind: FileKind::Parent,
        code: 10,
        comments: 2,
        blanks: 1,
        lines: 13,
        bytes: 400,
        tokens: 20,
    };
    let data = export_data(vec![row]);
    let out =
        render_to_string(|buf| write_export_csv_to(buf, &data, &export_args(ExportFormat::Csv)));
    // CSV lib should quote the field containing a comma
    let data_line = out.lines().nth(1).unwrap();
    assert!(
        data_line.contains("\"src/hello,world.rs\""),
        "path with comma should be quoted: {}",
        data_line
    );
}

#[test]
fn csv_quotes_in_path_are_escaped() {
    let row = FileRow {
        path: "src/say\"hi\".rs".into(),
        module: "src".into(),
        lang: "Rust".into(),
        kind: FileKind::Parent,
        code: 5,
        comments: 1,
        blanks: 0,
        lines: 6,
        bytes: 200,
        tokens: 10,
    };
    let data = export_data(vec![row]);
    let out =
        render_to_string(|buf| write_export_csv_to(buf, &data, &export_args(ExportFormat::Csv)));
    let data_line = out.lines().nth(1).unwrap();
    // CSV escapes double-quotes by doubling them
    assert!(
        data_line.contains("\"\""),
        "quotes should be escaped: {}",
        data_line
    );
}

#[test]
fn csv_child_kind_column() {
    let row = FileRow {
        path: "src/main.rs".into(),
        module: "src".into(),
        lang: "Rust".into(),
        kind: FileKind::Child,
        code: 10,
        comments: 2,
        blanks: 1,
        lines: 13,
        bytes: 400,
        tokens: 20,
    };
    let data = export_data(vec![row]);
    let out =
        render_to_string(|buf| write_export_csv_to(buf, &data, &export_args(ExportFormat::Csv)));
    assert!(out.contains(",child,"), "kind column should show 'child'");
}

#[test]
fn csv_parent_kind_column() {
    let data = export_data(vec![make_file_row("a.rs", "Rust", 10)]);
    let out =
        render_to_string(|buf| write_export_csv_to(buf, &data, &export_args(ExportFormat::Csv)));
    assert!(out.contains(",parent,"), "kind column should show 'parent'");
}

#[test]
fn csv_data_row_count() {
    let rows: Vec<FileRow> = (0..5)
        .map(|i| make_file_row(&format!("f{}.rs", i), "Rust", 10))
        .collect();
    let data = export_data(rows);
    let out =
        render_to_string(|buf| write_export_csv_to(buf, &data, &export_args(ExportFormat::Csv)));
    // 1 header + 5 data
    assert_eq!(out.lines().count(), 6);
}

// ============================================================================
// 6. CycloneDX SBOM format correctness
// ============================================================================

#[test]
fn cyclonedx_bom_format_field() {
    let data = export_data(vec![make_file_row("src/lib.rs", "Rust", 100)]);
    let out = render_to_string(|buf| {
        write_export_cyclonedx_with_options(
            buf,
            &data,
            RedactMode::None,
            Some("urn:uuid:test-1234".into()),
            Some("2024-01-01T00:00:00Z".into()),
        )
    });
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert_eq!(v["bomFormat"], "CycloneDX");
    assert_eq!(v["specVersion"], "1.6");
    assert_eq!(v["version"], 1);
}

#[test]
fn cyclonedx_serial_number_present() {
    let data = export_data(vec![make_file_row("a.rs", "Rust", 10)]);
    let out = render_to_string(|buf| {
        write_export_cyclonedx_with_options(
            buf,
            &data,
            RedactMode::None,
            Some("urn:uuid:abc-123".into()),
            Some("2024-06-01T00:00:00Z".into()),
        )
    });
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert_eq!(v["serialNumber"], "urn:uuid:abc-123");
}

#[test]
fn cyclonedx_components_match_rows() {
    let rows = vec![
        make_file_row("a.rs", "Rust", 10),
        make_file_row("b.py", "Python", 20),
    ];
    let data = export_data(rows);
    let out = render_to_string(|buf| {
        write_export_cyclonedx_with_options(
            buf,
            &data,
            RedactMode::None,
            Some("urn:uuid:test".into()),
            Some("2024-01-01T00:00:00Z".into()),
        )
    });
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert_eq!(v["components"].as_array().unwrap().len(), 2);
}

#[test]
fn cyclonedx_component_type_is_file() {
    let data = export_data(vec![make_file_row("src/lib.rs", "Rust", 50)]);
    let out = render_to_string(|buf| {
        write_export_cyclonedx_with_options(
            buf,
            &data,
            RedactMode::None,
            Some("urn:uuid:x".into()),
            Some("2024-01-01T00:00:00Z".into()),
        )
    });
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert_eq!(v["components"][0]["type"], "file");
}

#[test]
fn cyclonedx_child_kind_property() {
    let mut row = make_file_row("src/embed.html", "HTML", 10);
    row.kind = FileKind::Child;
    let data = export_data(vec![row]);
    let out = render_to_string(|buf| {
        write_export_cyclonedx_with_options(
            buf,
            &data,
            RedactMode::None,
            Some("urn:uuid:x".into()),
            Some("2024-01-01T00:00:00Z".into()),
        )
    });
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    let props = v["components"][0]["properties"].as_array().unwrap();
    let kind_prop = props.iter().find(|p| p["name"] == "tokmd:kind");
    assert!(
        kind_prop.is_some(),
        "child rows should have tokmd:kind property"
    );
    assert_eq!(kind_prop.unwrap()["value"], "child");
}

#[test]
fn cyclonedx_parent_no_kind_property() {
    let data = export_data(vec![make_file_row("src/lib.rs", "Rust", 50)]);
    let out = render_to_string(|buf| {
        write_export_cyclonedx_with_options(
            buf,
            &data,
            RedactMode::None,
            Some("urn:uuid:x".into()),
            Some("2024-01-01T00:00:00Z".into()),
        )
    });
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    let props = v["components"][0]["properties"].as_array().unwrap();
    let kind_prop = props.iter().find(|p| p["name"] == "tokmd:kind");
    assert!(
        kind_prop.is_none(),
        "parent rows should not have tokmd:kind"
    );
}

#[test]
fn cyclonedx_metadata_timestamp() {
    let data = export_data(vec![make_file_row("a.rs", "Rust", 10)]);
    let out = render_to_string(|buf| {
        write_export_cyclonedx_with_options(
            buf,
            &data,
            RedactMode::None,
            Some("urn:uuid:x".into()),
            Some("2024-12-25T12:00:00Z".into()),
        )
    });
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert_eq!(v["metadata"]["timestamp"], "2024-12-25T12:00:00Z");
}

#[test]
fn cyclonedx_tool_vendor() {
    let data = export_data(vec![make_file_row("a.rs", "Rust", 10)]);
    let out = render_to_string(|buf| {
        write_export_cyclonedx_with_options(
            buf,
            &data,
            RedactMode::None,
            Some("urn:uuid:x".into()),
            Some("2024-01-01T00:00:00Z".into()),
        )
    });
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert_eq!(v["metadata"]["tools"][0]["vendor"], "tokmd");
    assert_eq!(v["metadata"]["tools"][0]["name"], "tokmd");
}

// ============================================================================
// 7. Diff rendering with zero/negative/large changes
// ============================================================================

#[test]
fn diff_zero_delta() {
    let from = lang_report(vec![make_lang_row("Rust", 100)], true);
    let to = lang_report(vec![make_lang_row("Rust", 100)], true);
    let rows = compute_diff_rows(&from, &to);
    // No change means the row is omitted
    assert!(rows.is_empty());
}

#[test]
fn diff_negative_delta() {
    let from = lang_report(vec![make_lang_row("Rust", 500)], true);
    let to = lang_report(vec![make_lang_row("Rust", 200)], true);
    let rows = compute_diff_rows(&from, &to);
    assert_eq!(rows.len(), 1);
    assert!(rows[0].delta_code < 0);
}

#[test]
fn diff_large_increase() {
    let from = lang_report(vec![make_lang_row("Rust", 1)], true);
    let to = lang_report(vec![make_lang_row("Rust", 1_000_000)], true);
    let rows = compute_diff_rows(&from, &to);
    assert_eq!(rows[0].delta_code, 999_999);
}

#[test]
fn diff_language_added() {
    let from = lang_report(vec![], true);
    let to = lang_report(vec![make_lang_row("Go", 100)], true);
    let rows = compute_diff_rows(&from, &to);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].lang, "Go");
    assert_eq!(rows[0].old_code, 0);
    assert_eq!(rows[0].new_code, 100);
}

#[test]
fn diff_language_removed() {
    let from = lang_report(vec![make_lang_row("Perl", 500)], true);
    let to = lang_report(vec![], true);
    let rows = compute_diff_rows(&from, &to);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].new_code, 0);
    assert!(rows[0].delta_code < 0);
}

#[test]
fn diff_totals_aggregate_correctly() {
    let from = lang_report(
        vec![make_lang_row("Rust", 100), make_lang_row("Go", 200)],
        true,
    );
    let to = lang_report(
        vec![make_lang_row("Rust", 150), make_lang_row("Go", 50)],
        true,
    );
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    assert_eq!(totals.delta_code, (150 - 100) + (50 - 200));
}

#[test]
fn diff_md_contains_heading() {
    let rows = vec![tokmd_types::DiffRow {
        lang: "Rust".into(),
        old_code: 100,
        new_code: 200,
        delta_code: 100,
        old_lines: 120,
        new_lines: 240,
        delta_lines: 120,
        old_files: 5,
        new_files: 8,
        delta_files: 3,
        old_bytes: 4000,
        new_bytes: 8000,
        delta_bytes: 4000,
        old_tokens: 200,
        new_tokens: 400,
        delta_tokens: 200,
    }];
    let totals = compute_diff_totals(&rows);
    let md = render_diff_md("v1", "v2", &rows, &totals);
    assert!(md.contains("## Diff: v1 → v2"));
    assert!(md.contains("### Summary"));
    assert!(md.contains("### Language Breakdown"));
}

#[test]
fn diff_md_compact_mode() {
    let rows = vec![tokmd_types::DiffRow {
        lang: "Rust".into(),
        old_code: 100,
        new_code: 200,
        delta_code: 100,
        old_lines: 120,
        new_lines: 240,
        delta_lines: 120,
        old_files: 5,
        new_files: 8,
        delta_files: 3,
        old_bytes: 4000,
        new_bytes: 8000,
        delta_bytes: 4000,
        old_tokens: 200,
        new_tokens: 400,
        delta_tokens: 200,
    }];
    let totals = compute_diff_totals(&rows);
    let md = render_diff_md_with_options(
        "a",
        "b",
        &rows,
        &totals,
        DiffRenderOptions {
            compact: true,
            color: DiffColorMode::Off,
        },
    );
    assert!(md.contains("|Metric|Value|"));
    assert!(md.contains("|From LOC|"));
    // Compact mode should NOT have "Language Breakdown" section
    assert!(!md.contains("### Language Breakdown"));
}

#[test]
fn diff_md_color_mode_ansi_codes() {
    let rows = vec![tokmd_types::DiffRow {
        lang: "Rust".into(),
        old_code: 100,
        new_code: 200,
        delta_code: 100,
        old_lines: 120,
        new_lines: 240,
        delta_lines: 120,
        old_files: 5,
        new_files: 8,
        delta_files: 3,
        old_bytes: 4000,
        new_bytes: 8000,
        delta_bytes: 4000,
        old_tokens: 200,
        new_tokens: 400,
        delta_tokens: 200,
    }];
    let totals = compute_diff_totals(&rows);
    let md = render_diff_md_with_options(
        "a",
        "b",
        &rows,
        &totals,
        DiffRenderOptions {
            compact: false,
            color: DiffColorMode::Ansi,
        },
    );
    // Positive deltas should have green ANSI escape
    assert!(md.contains("\x1b[32m"), "should contain green ANSI code");
}

#[test]
fn diff_md_negative_delta_red() {
    let rows = vec![tokmd_types::DiffRow {
        lang: "Go".into(),
        old_code: 500,
        new_code: 200,
        delta_code: -300,
        old_lines: 600,
        new_lines: 240,
        delta_lines: -360,
        old_files: 10,
        new_files: 4,
        delta_files: -6,
        old_bytes: 20000,
        new_bytes: 8000,
        delta_bytes: -12000,
        old_tokens: 1000,
        new_tokens: 400,
        delta_tokens: -600,
    }];
    let totals = compute_diff_totals(&rows);
    let md = render_diff_md_with_options(
        "old",
        "new",
        &rows,
        &totals,
        DiffRenderOptions {
            compact: false,
            color: DiffColorMode::Ansi,
        },
    );
    assert!(
        md.contains("\x1b[31m"),
        "negative delta should have red ANSI"
    );
}

// ============================================================================
// 8. Empty data for every format
// ============================================================================

#[test]
fn empty_lang_md() {
    let report = lang_report(vec![], true);
    let out = render_to_string(|buf| {
        write_lang_report_to(buf, &report, &globals(), &lang_args(TableFormat::Md))
    });
    assert!(out.contains("|**Total**|0|0|0|0|0|0|"));
}

#[test]
fn empty_lang_tsv() {
    let report = lang_report(vec![], true);
    let out = render_to_string(|buf| {
        write_lang_report_to(buf, &report, &globals(), &lang_args(TableFormat::Tsv))
    });
    assert!(out.contains("Total\t0\t0\t0\t0\t0\t0"));
}

#[test]
fn empty_lang_json() {
    let report = lang_report(vec![], true);
    let out = render_to_string(|buf| {
        write_lang_report_to(buf, &report, &globals(), &lang_args(TableFormat::Json))
    });
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert!(v["rows"].as_array().unwrap().is_empty());
}

#[test]
fn empty_module_md() {
    let report = module_report(vec![]);
    let out = render_to_string(|buf| {
        write_module_report_to(buf, &report, &globals(), &module_args(TableFormat::Md))
    });
    assert!(out.contains("|**Total**|0|0|0|0|0|0|"));
}

#[test]
fn empty_module_tsv() {
    let report = module_report(vec![]);
    let out = render_to_string(|buf| {
        write_module_report_to(buf, &report, &globals(), &module_args(TableFormat::Tsv))
    });
    assert!(out.contains("Total\t0\t0\t0\t0\t0\t0"));
}

#[test]
fn empty_export_csv() {
    let data = export_data(vec![]);
    let out =
        render_to_string(|buf| write_export_csv_to(buf, &data, &export_args(ExportFormat::Csv)));
    // Just header, no data rows
    assert_eq!(out.lines().count(), 1);
    assert!(out.starts_with("path,"));
}

#[test]
fn empty_export_jsonl() {
    let data = export_data(vec![]);
    let out = render_to_string(|buf| {
        write_export_jsonl_to(buf, &data, &globals(), &export_args(ExportFormat::Jsonl))
    });
    // Just meta line
    assert_eq!(out.lines().count(), 1);
}

#[test]
fn empty_export_json() {
    let data = export_data(vec![]);
    let out = render_to_string(|buf| {
        write_export_json_to(buf, &data, &globals(), &export_args(ExportFormat::Json))
    });
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert!(v["rows"].as_array().unwrap().is_empty());
}

#[test]
fn empty_cyclonedx() {
    let data = export_data(vec![]);
    let out = render_to_string(|buf| {
        write_export_cyclonedx_with_options(
            buf,
            &data,
            RedactMode::None,
            Some("urn:uuid:empty".into()),
            Some("2024-01-01T00:00:00Z".into()),
        )
    });
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert!(v["components"].as_array().unwrap().is_empty());
}

#[test]
fn empty_diff_rows() {
    let from = lang_report(vec![], true);
    let to = lang_report(vec![], true);
    let rows = compute_diff_rows(&from, &to);
    assert!(rows.is_empty());
    let totals = compute_diff_totals(&rows);
    assert_eq!(totals.delta_code, 0);
}

#[test]
fn empty_diff_md() {
    let totals = compute_diff_totals(&[]);
    let md = render_diff_md("a", "b", &[], &totals);
    assert!(md.contains("## Diff:"));
    // Should still render the structure even with no rows
    assert!(md.contains("### Language Breakdown"));
}

// ============================================================================
// 9. Unicode in language names and file paths
// ============================================================================

#[test]
fn unicode_lang_name_md() {
    let report = lang_report(vec![make_lang_row("Ñ_ünïcödé", 100)], true);
    let out = render_to_string(|buf| {
        write_lang_report_to(buf, &report, &globals(), &lang_args(TableFormat::Md))
    });
    assert!(out.contains("|Ñ_ünïcödé|"));
}

#[test]
fn unicode_lang_name_tsv() {
    let report = lang_report(vec![make_lang_row("日本語", 50)], true);
    let out = render_to_string(|buf| {
        write_lang_report_to(buf, &report, &globals(), &lang_args(TableFormat::Tsv))
    });
    assert!(out.contains("日本語\t"));
}

#[test]
fn unicode_lang_name_json() {
    let report = lang_report(vec![make_lang_row("中文", 75)], true);
    let out = render_to_string(|buf| {
        write_lang_report_to(buf, &report, &globals(), &lang_args(TableFormat::Json))
    });
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    // LangReport is #[serde(flatten)] so rows are at top level
    assert_eq!(v["rows"][0]["lang"], "中文");
}

#[test]
fn unicode_file_path_csv() {
    let row = FileRow {
        path: "src/café/données.rs".into(),
        module: "src/café".into(),
        lang: "Rust".into(),
        kind: FileKind::Parent,
        code: 10,
        comments: 2,
        blanks: 1,
        lines: 13,
        bytes: 400,
        tokens: 20,
    };
    let data = export_data(vec![row]);
    let out =
        render_to_string(|buf| write_export_csv_to(buf, &data, &export_args(ExportFormat::Csv)));
    assert!(out.contains("café"));
    assert!(out.contains("données"));
}

#[test]
fn unicode_module_name_md() {
    let report = module_report(vec![make_module_row("ソース/コア", 200)]);
    let out = render_to_string(|buf| {
        write_module_report_to(buf, &report, &globals(), &module_args(TableFormat::Md))
    });
    assert!(out.contains("|ソース/コア|"));
}

#[test]
fn unicode_path_cyclonedx() {
    let row = FileRow {
        path: "src/λ/μ.rs".into(),
        module: "src/λ".into(),
        lang: "Rust".into(),
        kind: FileKind::Parent,
        code: 10,
        comments: 2,
        blanks: 1,
        lines: 13,
        bytes: 400,
        tokens: 20,
    };
    let data = export_data(vec![row]);
    let out = render_to_string(|buf| {
        write_export_cyclonedx_with_options(
            buf,
            &data,
            RedactMode::None,
            Some("urn:uuid:u".into()),
            Some("2024-01-01T00:00:00Z".into()),
        )
    });
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert_eq!(v["components"][0]["name"], "src/λ/μ.rs");
    assert_eq!(v["components"][0]["group"], "src/λ");
}

#[test]
fn unicode_diff_lang_name() {
    let from = lang_report(vec![make_lang_row("Ελληνικά", 100)], true);
    let to = lang_report(vec![make_lang_row("Ελληνικά", 200)], true);
    let rows = compute_diff_rows(&from, &to);
    assert_eq!(rows[0].lang, "Ελληνικά");
}

// ============================================================================
// 10. Very long path handling
// ============================================================================

#[test]
fn long_path_csv_preserves_full() {
    let long_path = format!("src/{}/file.rs", "a".repeat(500));
    let row = make_file_row(&long_path, "Rust", 10);
    let data = export_data(vec![row]);
    let out =
        render_to_string(|buf| write_export_csv_to(buf, &data, &export_args(ExportFormat::Csv)));
    assert!(out.contains(&"a".repeat(500)));
}

#[test]
fn long_path_jsonl_preserves_full() {
    let long_path = format!("deep/{}/file.py", "nested/".repeat(50));
    let row = make_file_row(&long_path, "Python", 10);
    let data = export_data(vec![row]);
    let out = render_to_string(|buf| {
        write_export_jsonl_to(
            buf,
            &data,
            &globals(),
            &export_args_no_meta(ExportFormat::Jsonl),
        )
    });
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert_eq!(v["path"], long_path);
}

#[test]
fn long_module_name_md() {
    let module_name = format!("crates/{}", "deep_module_name_".repeat(20));
    let report = module_report(vec![make_module_row(&module_name, 100)]);
    let out = render_to_string(|buf| {
        write_module_report_to(buf, &report, &globals(), &module_args(TableFormat::Md))
    });
    assert!(out.contains(&module_name));
}

// ============================================================================
// 11. Determinism: same data → same formatted output
// ============================================================================

#[test]
fn determinism_lang_md() {
    let report = lang_report(
        vec![make_lang_row("Rust", 500), make_lang_row("Go", 200)],
        true,
    );
    let out1 = render_to_string(|buf| {
        write_lang_report_to(buf, &report, &globals(), &lang_args(TableFormat::Md))
    });
    let out2 = render_to_string(|buf| {
        write_lang_report_to(buf, &report, &globals(), &lang_args(TableFormat::Md))
    });
    assert_eq!(out1, out2);
}

#[test]
fn determinism_lang_tsv() {
    let report = lang_report(vec![make_lang_row("C", 300)], false);
    let out1 = render_to_string(|buf| {
        write_lang_report_to(buf, &report, &globals(), &lang_args(TableFormat::Tsv))
    });
    let out2 = render_to_string(|buf| {
        write_lang_report_to(buf, &report, &globals(), &lang_args(TableFormat::Tsv))
    });
    assert_eq!(out1, out2);
}

#[test]
fn determinism_module_md() {
    let report = module_report(vec![
        make_module_row("src/core", 400),
        make_module_row("src/util", 100),
    ]);
    let out1 = render_to_string(|buf| {
        write_module_report_to(buf, &report, &globals(), &module_args(TableFormat::Md))
    });
    let out2 = render_to_string(|buf| {
        write_module_report_to(buf, &report, &globals(), &module_args(TableFormat::Md))
    });
    assert_eq!(out1, out2);
}

#[test]
fn determinism_csv() {
    let data = export_data(vec![
        make_file_row("a.rs", "Rust", 100),
        make_file_row("b.rs", "Rust", 200),
    ]);
    let out1 =
        render_to_string(|buf| write_export_csv_to(buf, &data, &export_args(ExportFormat::Csv)));
    let out2 =
        render_to_string(|buf| write_export_csv_to(buf, &data, &export_args(ExportFormat::Csv)));
    assert_eq!(out1, out2);
}

#[test]
fn determinism_cyclonedx() {
    let data = export_data(vec![make_file_row("x.rs", "Rust", 50)]);
    let out1 = render_to_string(|buf| {
        write_export_cyclonedx_with_options(
            buf,
            &data,
            RedactMode::None,
            Some("urn:uuid:det".into()),
            Some("2024-01-01T00:00:00Z".into()),
        )
    });
    let out2 = render_to_string(|buf| {
        write_export_cyclonedx_with_options(
            buf,
            &data,
            RedactMode::None,
            Some("urn:uuid:det".into()),
            Some("2024-01-01T00:00:00Z".into()),
        )
    });
    assert_eq!(out1, out2);
}

#[test]
fn determinism_diff_md() {
    let from = lang_report(vec![make_lang_row("Rust", 100)], true);
    let to = lang_report(vec![make_lang_row("Rust", 200)], true);
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let md1 = render_diff_md("a", "b", &rows, &totals);
    let md2 = render_diff_md("a", "b", &rows, &totals);
    assert_eq!(md1, md2);
}

#[test]
fn determinism_diff_receipt() {
    let r1 = create_diff_receipt("a", "b", vec![], compute_diff_totals(&[]));
    let r2 = create_diff_receipt("a", "b", vec![], compute_diff_totals(&[]));
    // Receipt fields that are stable (mode, source, rows, totals)
    assert_eq!(r1.mode, r2.mode);
    assert_eq!(r1.from_source, r2.from_source);
    assert_eq!(r1.diff_rows.len(), r2.diff_rows.len());
}

// ============================================================================
// 12. Property tests: JSON output always parses back as valid JSON
// ============================================================================

proptest! {
    #[test]
    fn prop_lang_json_always_valid(code in 0usize..100_000, lines in 0usize..200_000) {
        let row = LangRow {
            lang: "TestLang".into(),
            code,
            lines,
            files: 1,
            bytes: code * 40,
            tokens: code * 2,
            avg_lines: lines,
        };
        let report = lang_report(vec![row], true);
        let out = render_to_string(|buf| {
            write_lang_report_to(buf, &report, &globals(), &lang_args(TableFormat::Json))
        });
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(out.trim());
        prop_assert!(parsed.is_ok(), "JSON parse failed for code={}, lines={}", code, lines);
    }

    #[test]
    fn prop_module_json_always_valid(code in 0usize..100_000) {
        let row = make_module_row("test_mod", code);
        let report = module_report(vec![row]);
        let out = render_to_string(|buf| {
            write_module_report_to(buf, &report, &globals(), &module_args(TableFormat::Json))
        });
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(out.trim());
        prop_assert!(parsed.is_ok(), "JSON parse failed for code={}", code);
    }

    #[test]
    fn prop_csv_row_count_matches(n in 0usize..50) {
        let rows: Vec<FileRow> = (0..n)
            .map(|i| make_file_row(&format!("f{}.rs", i), "Rust", i + 1))
            .collect();
        let data = export_data(rows);
        let out = render_to_string(|buf| {
            write_export_csv_to(buf, &data, &export_args(ExportFormat::Csv))
        });
        // header + n data rows
        prop_assert_eq!(out.lines().count(), n + 1);
    }

    #[test]
    fn prop_jsonl_line_count_with_meta(n in 0usize..50) {
        let rows: Vec<FileRow> = (0..n)
            .map(|i| make_file_row(&format!("f{}.rs", i), "Rust", i + 1))
            .collect();
        let data = export_data(rows);
        let out = render_to_string(|buf| {
            write_export_jsonl_to(buf, &data, &globals(), &export_args(ExportFormat::Jsonl))
        });
        // 1 meta + n row lines
        prop_assert_eq!(out.lines().count(), n + 1);
    }

    #[test]
    fn prop_diff_totals_delta_matches(
        old_code in 0usize..100_000,
        new_code in 0usize..100_000
    ) {
        let row = tokmd_types::DiffRow {
            lang: "X".into(),
            old_code,
            new_code,
            delta_code: new_code as i64 - old_code as i64,
            old_lines: old_code, new_lines: new_code,
            delta_lines: new_code as i64 - old_code as i64,
            old_files: 1, new_files: 1, delta_files: 0,
            old_bytes: old_code * 40, new_bytes: new_code * 40,
            delta_bytes: (new_code as i64 - old_code as i64) * 40,
            old_tokens: old_code * 2, new_tokens: new_code * 2,
            delta_tokens: (new_code as i64 - old_code as i64) * 2,
        };
        let totals = compute_diff_totals(&[row]);
        prop_assert_eq!(totals.delta_code, new_code as i64 - old_code as i64);
    }

    #[test]
    fn prop_tsv_no_trailing_tabs(code in 1usize..10_000) {
        let report = lang_report(vec![make_lang_row("Rust", code)], true);
        let out = render_to_string(|buf| {
            write_lang_report_to(buf, &report, &globals(), &lang_args(TableFormat::Tsv))
        });
        for line in out.lines() {
            prop_assert!(!line.ends_with('\t'), "trailing tab: {}", line);
        }
    }

    #[test]
    fn prop_cyclonedx_component_count_matches(n in 0usize..30) {
        let rows: Vec<FileRow> = (0..n)
            .map(|i| make_file_row(&format!("src/f{}.rs", i), "Rust", i + 1))
            .collect();
        let data = export_data(rows);
        let out = render_to_string(|buf| {
            write_export_cyclonedx_with_options(
                buf,
                &data,
                RedactMode::None,
                Some("urn:uuid:prop".into()),
                Some("2024-01-01T00:00:00Z".into()),
            )
        });
        let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
        prop_assert_eq!(v["components"].as_array().unwrap().len(), n);
    }
}

// ============================================================================
// 13. Snapshot tests with insta for key format outputs
// ============================================================================

#[test]
fn snapshot_w63_lang_md_multi_lang() {
    let report = lang_report(
        vec![
            make_lang_row("Rust", 1000),
            make_lang_row("Python", 500),
            make_lang_row("Go", 200),
        ],
        true,
    );
    let out = render_to_string(|buf| {
        write_lang_report_to(buf, &report, &globals(), &lang_args(TableFormat::Md))
    });
    insta::assert_snapshot!(out);
}

#[test]
fn snapshot_w63_lang_tsv_no_files() {
    let report = lang_report(vec![make_lang_row("Rust", 100)], false);
    let args = LangArgs {
        files: false,
        ..lang_args(TableFormat::Tsv)
    };
    let out = render_to_string(|buf| write_lang_report_to(buf, &report, &globals(), &args));
    insta::assert_snapshot!(out);
}

#[test]
fn snapshot_w63_module_md() {
    let report = module_report(vec![
        make_module_row("crates/core", 800),
        make_module_row("crates/format", 400),
        make_module_row("crates/types", 200),
    ]);
    let out = render_to_string(|buf| {
        write_module_report_to(buf, &report, &globals(), &module_args(TableFormat::Md))
    });
    insta::assert_snapshot!(out);
}

#[test]
fn snapshot_w63_csv_basic() {
    let data = export_data(vec![
        make_file_row("src/lib.rs", "Rust", 100),
        make_file_row("src/main.rs", "Rust", 50),
    ]);
    let out =
        render_to_string(|buf| write_export_csv_to(buf, &data, &export_args(ExportFormat::Csv)));
    insta::assert_snapshot!(out);
}

#[test]
fn snapshot_w63_cyclonedx_basic() {
    let data = export_data(vec![make_file_row("src/lib.rs", "Rust", 100)]);
    let out = render_to_string(|buf| {
        write_export_cyclonedx_with_options(
            buf,
            &data,
            RedactMode::None,
            Some("urn:uuid:snapshot-test".into()),
            Some("2024-01-15T10:30:00Z".into()),
        )
    });
    insta::assert_snapshot!(out);
}

#[test]
fn snapshot_w63_diff_md_full() {
    let from = lang_report(
        vec![make_lang_row("Rust", 1000), make_lang_row("Go", 500)],
        true,
    );
    let to = lang_report(
        vec![
            make_lang_row("Rust", 1200),
            make_lang_row("Go", 300),
            make_lang_row("Python", 100),
        ],
        true,
    );
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let md = render_diff_md("v1.0", "v2.0", &rows, &totals);
    insta::assert_snapshot!(md);
}

#[test]
fn snapshot_w63_diff_compact() {
    let from = lang_report(vec![make_lang_row("Rust", 500)], true);
    let to = lang_report(vec![make_lang_row("Rust", 800)], true);
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let md = render_diff_md_with_options(
        "before",
        "after",
        &rows,
        &totals,
        DiffRenderOptions {
            compact: true,
            color: DiffColorMode::Off,
        },
    );
    insta::assert_snapshot!(md);
}

#[test]
fn snapshot_w63_empty_lang_md() {
    let report = lang_report(vec![], true);
    let out = render_to_string(|buf| {
        write_lang_report_to(buf, &report, &globals(), &lang_args(TableFormat::Md))
    });
    insta::assert_snapshot!(out);
}

// ============================================================================
// 14. Additional edge cases
// ============================================================================

#[test]
fn cyclonedx_empty_module_omits_group() {
    let row = FileRow {
        path: "top_level.rs".into(),
        module: "".into(),
        lang: "Rust".into(),
        kind: FileKind::Parent,
        code: 10,
        comments: 2,
        blanks: 1,
        lines: 13,
        bytes: 400,
        tokens: 20,
    };
    let data = export_data(vec![row]);
    let out = render_to_string(|buf| {
        write_export_cyclonedx_with_options(
            buf,
            &data,
            RedactMode::None,
            Some("urn:uuid:g".into()),
            Some("2024-01-01T00:00:00Z".into()),
        )
    });
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    // Empty module → group should be null/absent
    assert!(
        v["components"][0]["group"].is_null(),
        "empty module should skip group"
    );
}

#[test]
fn diff_receipt_mode_is_diff() {
    let receipt = create_diff_receipt("a", "b", vec![], compute_diff_totals(&[]));
    assert_eq!(receipt.mode, "diff");
}

#[test]
fn diff_receipt_schema_version_present() {
    let receipt = create_diff_receipt("x", "y", vec![], compute_diff_totals(&[]));
    assert!(receipt.schema_version > 0);
}

#[test]
fn csv_newline_in_lang_name_quoted() {
    // While unusual, ensure the CSV writer handles embedded newlines
    let row = FileRow {
        path: "a.rs".into(),
        module: "src".into(),
        lang: "Rust\nNext".into(),
        kind: FileKind::Parent,
        code: 10,
        comments: 2,
        blanks: 1,
        lines: 13,
        bytes: 400,
        tokens: 20,
    };
    let data = export_data(vec![row]);
    let out =
        render_to_string(|buf| write_export_csv_to(buf, &data, &export_args(ExportFormat::Csv)));
    // CSV writer should quote fields containing newlines
    assert!(
        out.contains("\"Rust\nNext\""),
        "newline in field should be quoted"
    );
}

#[test]
fn many_languages_diff() {
    let from_rows: Vec<LangRow> = (0..20)
        .map(|i| make_lang_row(&format!("Lang{}", i), (i + 1) * 100))
        .collect();
    let to_rows: Vec<LangRow> = (0..20)
        .map(|i| make_lang_row(&format!("Lang{}", i), (i + 1) * 150))
        .collect();
    let from = lang_report(from_rows, true);
    let to = lang_report(to_rows, true);
    let rows = compute_diff_rows(&from, &to);
    assert_eq!(rows.len(), 20);
    for row in &rows {
        assert!(row.delta_code > 0);
    }
}

#[test]
fn diff_md_language_movement_counts() {
    let from = lang_report(
        vec![make_lang_row("Rust", 100), make_lang_row("C", 50)],
        true,
    );
    let to = lang_report(
        vec![make_lang_row("Rust", 200), make_lang_row("Go", 100)],
        true,
    );
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let md = render_diff_md("v1", "v2", &rows, &totals);
    assert!(md.contains("|Added|1|"));
    assert!(md.contains("|Removed|1|"));
    assert!(md.contains("|Modified|1|"));
}
