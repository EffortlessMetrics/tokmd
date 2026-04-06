//! Deep tests for export formatting paths (JSONL, CSV, JSON, CycloneDX).
//!
//! Covers: field escaping, empty data for every format, newline-delimited
//! JSONL invariants, CycloneDX structural validity, redaction in CycloneDX,
//! and determinism of serialized output.

use std::path::PathBuf;

use tokmd_format::{
    write_export_csv_to, write_export_cyclonedx_to, write_export_cyclonedx_with_options,
    write_export_json_to, write_export_jsonl_to,
};
use tokmd_settings::{ChildIncludeMode, ScanOptions};
use tokmd_types::{ExportArgs, ExportData, ExportFormat, FileKind, FileRow, RedactMode};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn file_row(path: &str, module: &str, lang: &str, kind: FileKind, code: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: lang.to_string(),
        kind,
        code,
        comments: code / 5,
        blanks: code / 10,
        lines: code + code / 5 + code / 10,
        bytes: code * 10,
        tokens: code * 3,
    }
}

fn sample_rows() -> Vec<FileRow> {
    vec![
        file_row("src/lib.rs", "src", "Rust", FileKind::Parent, 200),
        file_row("tests/it.rs", "tests", "Rust", FileKind::Parent, 80),
        file_row("src/main.rs", "src", "Rust", FileKind::Parent, 50),
    ]
}

fn export_data(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec!["src".into()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn default_args(format: ExportFormat) -> ExportArgs {
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
        meta: true,
        strip_prefix: None,
    }
}

fn default_scan() -> ScanOptions {
    ScanOptions::default()
}

// ===========================================================================
// 1. JSONL: each line is valid JSON, newline-delimited
// ===========================================================================

#[test]
fn jsonl_each_line_is_valid_json_with_meta() {
    let data = export_data(sample_rows());
    let args = default_args(ExportFormat::Jsonl);
    let scan = default_scan();
    let mut buf = Vec::new();

    write_export_jsonl_to(&mut buf, &data, &scan, &args).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");

    for (i, line) in output.lines().enumerate() {
        assert!(
            serde_json::from_str::<serde_json::Value>(line).is_ok(),
            "line {i} is not valid JSON: {line}"
        );
    }
}

#[test]
fn jsonl_without_meta_has_only_data_rows() {
    let data = export_data(sample_rows());
    let mut args = default_args(ExportFormat::Jsonl);
    args.meta = false;
    let scan = default_scan();
    let mut buf = Vec::new();

    write_export_jsonl_to(&mut buf, &data, &scan, &args).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    let lines: Vec<&str> = output.lines().collect();

    // No meta line → all lines are data rows
    assert_eq!(lines.len(), 3);
    for line in &lines {
        let v: serde_json::Value = serde_json::from_str(line).expect("operation must succeed");
        assert_eq!(v["type"], "row");
    }
}

#[test]
fn jsonl_with_meta_first_line_is_meta() {
    let data = export_data(sample_rows());
    let args = default_args(ExportFormat::Jsonl);
    let scan = default_scan();
    let mut buf = Vec::new();

    write_export_jsonl_to(&mut buf, &data, &scan, &args).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    let first_line = output
        .lines()
        .next()
        .expect("output must have at least one line");
    let v: serde_json::Value = serde_json::from_str(first_line).expect("operation must succeed");

    assert_eq!(v["type"], "meta");
    assert!(v["schema_version"].is_number());
}

#[test]
fn jsonl_row_count_equals_data_rows() {
    let rows = sample_rows();
    let expected = rows.len();
    let data = export_data(rows);
    let mut args = default_args(ExportFormat::Jsonl);
    args.meta = false;
    let scan = default_scan();
    let mut buf = Vec::new();

    write_export_jsonl_to(&mut buf, &data, &scan, &args).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");

    assert_eq!(output.lines().count(), expected);
}

#[test]
fn jsonl_ends_with_newline() {
    let data = export_data(sample_rows());
    let args = default_args(ExportFormat::Jsonl);
    let scan = default_scan();
    let mut buf = Vec::new();

    write_export_jsonl_to(&mut buf, &data, &scan, &args).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");

    assert!(output.ends_with('\n'), "JSONL must end with newline");
}

// ===========================================================================
// 2. CSV: headers, field escaping, column consistency
// ===========================================================================

#[test]
fn csv_first_line_is_header() {
    let data = export_data(sample_rows());
    let args = default_args(ExportFormat::Csv);
    let mut buf = Vec::new();

    write_export_csv_to(&mut buf, &data, &args).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    let header = output
        .lines()
        .next()
        .expect("output must have at least one line");

    assert_eq!(
        header,
        "path,module,lang,kind,code,comments,blanks,lines,bytes,tokens"
    );
}

#[test]
fn csv_column_count_consistent_across_all_rows() {
    let data = export_data(sample_rows());
    let args = default_args(ExportFormat::Csv);
    let mut buf = Vec::new();

    write_export_csv_to(&mut buf, &data, &args).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");

    let expected_cols = 10;
    for (i, line) in output.lines().enumerate() {
        // CSV parser handles quoting; a simple count works for non-quoted data
        let cols = line.split(',').count();
        assert_eq!(
            cols, expected_cols,
            "line {i} has {cols} columns, expected {expected_cols}"
        );
    }
}

#[test]
fn csv_escapes_commas_in_path() {
    let rows = vec![file_row("src/a,b.rs", "src", "Rust", FileKind::Parent, 100)];
    let data = export_data(rows);
    let args = default_args(ExportFormat::Csv);
    let mut buf = Vec::new();

    write_export_csv_to(&mut buf, &data, &args).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");

    // CSV library should quote fields containing commas
    assert!(
        output.contains("\"src/a,b.rs\""),
        "comma in path must be quoted: {output}"
    );
}

#[test]
fn csv_escapes_quotes_in_path() {
    let rows = vec![file_row(
        "src/a\"b.rs",
        "src",
        "Rust",
        FileKind::Parent,
        100,
    )];
    let data = export_data(rows);
    let args = default_args(ExportFormat::Csv);
    let mut buf = Vec::new();

    write_export_csv_to(&mut buf, &data, &args).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");

    // CSV escapes double-quotes by doubling them inside a quoted field
    assert!(
        output.contains("\"\""),
        "quote in path must be escaped: {output}"
    );
}

#[test]
fn csv_includes_child_file_kind() {
    let rows = vec![
        file_row("src/lib.rs", "src", "Rust", FileKind::Parent, 100),
        file_row("src/inline.html", "src", "HTML", FileKind::Child, 20),
    ];
    let data = export_data(rows);
    let args = default_args(ExportFormat::Csv);
    let mut buf = Vec::new();

    write_export_csv_to(&mut buf, &data, &args).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");

    assert!(output.contains(",parent,"));
    assert!(output.contains(",child,"));
}

// ===========================================================================
// 3. CycloneDX: structural validity
// ===========================================================================

#[test]
fn cyclonedx_output_is_valid_json() {
    let data = export_data(sample_rows());
    let mut buf = Vec::new();

    write_export_cyclonedx_to(&mut buf, &data, RedactMode::None).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");

    let v: serde_json::Value = serde_json::from_str(&output).expect("must parse valid JSON");
    assert_eq!(v["bomFormat"], "CycloneDX");
    assert_eq!(v["specVersion"], "1.6");
}

#[test]
fn cyclonedx_component_count_matches_rows() {
    let rows = sample_rows();
    let expected = rows.len();
    let data = export_data(rows);
    let mut buf = Vec::new();

    write_export_cyclonedx_to(&mut buf, &data, RedactMode::None).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    let v: serde_json::Value = serde_json::from_str(&output).expect("must parse valid JSON");

    assert_eq!(
        v["components"]
            .as_array()
            .expect("must be a JSON array")
            .len(),
        expected
    );
}

#[test]
fn cyclonedx_components_have_required_properties() {
    let data = export_data(sample_rows());
    let mut buf = Vec::new();

    write_export_cyclonedx_to(&mut buf, &data, RedactMode::None).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    let v: serde_json::Value = serde_json::from_str(&output).expect("must parse valid JSON");

    for comp in v["components"].as_array().expect("must be a JSON array") {
        assert_eq!(comp["type"], "file");
        assert!(comp["name"].is_string());
        let props = comp["properties"].as_array().expect("must be a JSON array");
        let prop_names: Vec<&str> = props
            .iter()
            .map(|p| p["name"].as_str().expect("must be a JSON string"))
            .collect();
        assert!(prop_names.contains(&"tokmd:lang"));
        assert!(prop_names.contains(&"tokmd:code"));
        assert!(prop_names.contains(&"tokmd:lines"));
        assert!(prop_names.contains(&"tokmd:tokens"));
    }
}

#[test]
fn cyclonedx_child_kind_property_only_on_children() {
    let rows = vec![
        file_row("src/lib.rs", "src", "Rust", FileKind::Parent, 100),
        file_row("src/inline.html", "src", "HTML", FileKind::Child, 20),
    ];
    let data = export_data(rows);
    let mut buf = Vec::new();

    write_export_cyclonedx_to(&mut buf, &data, RedactMode::None).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    let v: serde_json::Value = serde_json::from_str(&output).expect("must parse valid JSON");

    let components = v["components"].as_array().expect("must be a JSON array");

    // Parent should NOT have tokmd:kind
    let parent_props: Vec<&str> = components[0]["properties"]
        .as_array()
        .expect("operation must succeed")
        .iter()
        .map(|p| p["name"].as_str().expect("must be a JSON string"))
        .collect();
    assert!(!parent_props.contains(&"tokmd:kind"));

    // Child should have tokmd:kind = "child"
    let child_props = components[1]["properties"]
        .as_array()
        .expect("must be a JSON array");
    let kind_prop = child_props
        .iter()
        .find(|p| p["name"] == "tokmd:kind")
        .expect("operation must succeed");
    assert_eq!(kind_prop["value"], "child");
}

#[test]
fn cyclonedx_with_fixed_serial_and_timestamp_is_deterministic() {
    let data = export_data(sample_rows());
    let serial = "urn:uuid:00000000-0000-0000-0000-000000000000".to_string();
    let ts = "2024-01-01T00:00:00Z".to_string();

    let mut buf1 = Vec::new();
    write_export_cyclonedx_with_options(
        &mut buf1,
        &data,
        RedactMode::None,
        Some(serial.clone()),
        Some(ts.clone()),
    )
    .expect("operation must succeed");

    let mut buf2 = Vec::new();
    write_export_cyclonedx_with_options(&mut buf2, &data, RedactMode::None, Some(serial), Some(ts))
        .expect("operation must succeed");

    assert_eq!(
        buf1, buf2,
        "CycloneDX must be deterministic with fixed params"
    );
}

#[test]
fn cyclonedx_redact_paths_hashes_component_names() {
    let data = export_data(sample_rows());
    let mut buf = Vec::new();

    write_export_cyclonedx_to(&mut buf, &data, RedactMode::Paths).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    let v: serde_json::Value = serde_json::from_str(&output).expect("must parse valid JSON");

    for comp in v["components"].as_array().expect("must be a JSON array") {
        let name = comp["name"].as_str().expect("must be a JSON string");
        // Redacted paths should NOT contain the original path segments
        assert!(
            !name.starts_with("src/"),
            "redacted path should not start with src/: {name}"
        );
    }
}

#[test]
fn cyclonedx_metadata_contains_tool_info() {
    let data = export_data(sample_rows());
    let mut buf = Vec::new();

    write_export_cyclonedx_to(&mut buf, &data, RedactMode::None).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    let v: serde_json::Value = serde_json::from_str(&output).expect("must parse valid JSON");

    let tools = v["metadata"]["tools"]
        .as_array()
        .expect("must be a JSON array");
    assert!(!tools.is_empty());
    assert_eq!(tools[0]["name"], "tokmd");
}

// ===========================================================================
// 4. Empty data handling for all formats
// ===========================================================================

#[test]
fn jsonl_empty_data_with_meta_produces_only_meta_line() {
    let data = export_data(vec![]);
    let args = default_args(ExportFormat::Jsonl);
    let scan = default_scan();
    let mut buf = Vec::new();

    write_export_jsonl_to(&mut buf, &data, &scan, &args).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    let lines: Vec<&str> = output.lines().collect();

    assert_eq!(lines.len(), 1, "empty data with meta = 1 line");
    let v: serde_json::Value = serde_json::from_str(lines[0]).expect("operation must succeed");
    assert_eq!(v["type"], "meta");
}

#[test]
fn jsonl_empty_data_without_meta_produces_no_output() {
    let data = export_data(vec![]);
    let mut args = default_args(ExportFormat::Jsonl);
    args.meta = false;
    let scan = default_scan();
    let mut buf = Vec::new();

    write_export_jsonl_to(&mut buf, &data, &scan, &args).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");

    assert!(output.is_empty(), "empty data without meta = no output");
}

#[test]
fn csv_empty_data_produces_only_header() {
    let data = export_data(vec![]);
    let args = default_args(ExportFormat::Csv);
    let mut buf = Vec::new();

    write_export_csv_to(&mut buf, &data, &args).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    let lines: Vec<&str> = output.lines().collect();

    assert_eq!(lines.len(), 1, "empty data CSV = header only");
    assert!(lines[0].starts_with("path,"));
}

#[test]
fn json_empty_data_with_meta_produces_valid_envelope() {
    let data = export_data(vec![]);
    let args = default_args(ExportFormat::Json);
    let scan = default_scan();
    let mut buf = Vec::new();

    write_export_json_to(&mut buf, &data, &scan, &args).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    let v: serde_json::Value = serde_json::from_str(&output).expect("must parse valid JSON");

    assert!(v["schema_version"].is_number());
    assert_eq!(v["rows"].as_array().expect("must be a JSON array").len(), 0);
}

#[test]
fn json_empty_data_without_meta_produces_empty_array() {
    let data = export_data(vec![]);
    let mut args = default_args(ExportFormat::Json);
    args.meta = false;
    let scan = default_scan();
    let mut buf = Vec::new();

    write_export_json_to(&mut buf, &data, &scan, &args).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    let v: serde_json::Value = serde_json::from_str(&output).expect("must parse valid JSON");

    assert!(v.is_array());
    assert_eq!(v.as_array().expect("must be a JSON array").len(), 0);
}

#[test]
fn cyclonedx_empty_data_produces_valid_bom_with_no_components() {
    let data = export_data(vec![]);
    let mut buf = Vec::new();

    write_export_cyclonedx_to(&mut buf, &data, RedactMode::None).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    let v: serde_json::Value = serde_json::from_str(&output).expect("must parse valid JSON");

    assert_eq!(v["bomFormat"], "CycloneDX");
    assert_eq!(
        v["components"]
            .as_array()
            .expect("must be a JSON array")
            .len(),
        0
    );
}

// ===========================================================================
// 5. Determinism: same input → identical output
// ===========================================================================

#[test]
fn csv_output_is_deterministic() {
    let data = export_data(sample_rows());
    let args = default_args(ExportFormat::Csv);

    let mut buf1 = Vec::new();
    write_export_csv_to(&mut buf1, &data, &args).expect("operation must succeed");

    let mut buf2 = Vec::new();
    write_export_csv_to(&mut buf2, &data, &args).expect("operation must succeed");

    assert_eq!(buf1, buf2, "CSV output must be deterministic");
}

#[test]
fn jsonl_without_meta_is_deterministic() {
    let data = export_data(sample_rows());
    let mut args = default_args(ExportFormat::Jsonl);
    args.meta = false;
    let scan = default_scan();

    let mut buf1 = Vec::new();
    write_export_jsonl_to(&mut buf1, &data, &scan, &args).expect("operation must succeed");

    let mut buf2 = Vec::new();
    write_export_jsonl_to(&mut buf2, &data, &scan, &args).expect("operation must succeed");

    assert_eq!(buf1, buf2, "JSONL (no meta) output must be deterministic");
}

// ===========================================================================
// 6. JSONL row field integrity
// ===========================================================================

#[test]
fn jsonl_rows_contain_all_file_row_fields() {
    let data = export_data(sample_rows());
    let mut args = default_args(ExportFormat::Jsonl);
    args.meta = false;
    let scan = default_scan();
    let mut buf = Vec::new();

    write_export_jsonl_to(&mut buf, &data, &scan, &args).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");

    let first: serde_json::Value = serde_json::from_str(
        output
            .lines()
            .next()
            .expect("output must have at least one line"),
    )
    .expect("operation must succeed");
    assert!(first["path"].is_string());
    assert!(first["module"].is_string());
    assert!(first["lang"].is_string());
    assert!(first["kind"].is_string());
    assert!(first["code"].is_number());
    assert!(first["comments"].is_number());
    assert!(first["blanks"].is_number());
    assert!(first["lines"].is_number());
    assert!(first["bytes"].is_number());
    assert!(first["tokens"].is_number());
}

#[test]
fn jsonl_row_values_match_input_data() {
    let rows = vec![file_row("src/lib.rs", "src", "Rust", FileKind::Parent, 100)];
    let data = export_data(rows);
    let mut args = default_args(ExportFormat::Jsonl);
    args.meta = false;
    let scan = default_scan();
    let mut buf = Vec::new();

    write_export_jsonl_to(&mut buf, &data, &scan, &args).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");

    let v: serde_json::Value = serde_json::from_str(
        output
            .lines()
            .next()
            .expect("output must have at least one line"),
    )
    .expect("operation must succeed");
    assert_eq!(v["path"], "src/lib.rs");
    assert_eq!(v["lang"], "Rust");
    assert_eq!(v["code"], 100);
    assert_eq!(v["kind"], "parent");
}

// ===========================================================================
// 7. CycloneDX: group field presence
// ===========================================================================

#[test]
fn cyclonedx_empty_module_omits_group_field() {
    let rows = vec![file_row("README.md", "", "Markdown", FileKind::Parent, 10)];
    let data = export_data(rows);
    let mut buf = Vec::new();

    write_export_cyclonedx_to(&mut buf, &data, RedactMode::None).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    let v: serde_json::Value = serde_json::from_str(&output).expect("must parse valid JSON");

    let comp = &v["components"][0];
    assert!(comp.get("group").is_none() || comp["group"].is_null());
}

#[test]
fn cyclonedx_nonempty_module_includes_group_field() {
    let rows = vec![file_row("src/lib.rs", "src", "Rust", FileKind::Parent, 100)];
    let data = export_data(rows);
    let mut buf = Vec::new();

    write_export_cyclonedx_to(&mut buf, &data, RedactMode::None).expect("operation must succeed");
    let output = String::from_utf8(buf).expect("output must be valid UTF-8");
    let v: serde_json::Value = serde_json::from_str(&output).expect("must parse valid JSON");

    let comp = &v["components"][0];
    assert_eq!(comp["group"], "src");
}
