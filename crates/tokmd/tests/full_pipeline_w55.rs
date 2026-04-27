#![cfg(feature = "analysis")]

//! Cross-crate full-pipeline integration tests (W55).
//!
//! Exercises the complete data-flow across the tiered microcrate architecture:
//! types → scan → model → format → analysis, including CLI round-trips.

mod common;

use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tokmd_analysis::derive_report;
use tokmd_format::{
    badge_svg, compute_diff_rows, compute_diff_totals, create_diff_receipt, write_export_csv_to,
    write_export_json_to, write_export_jsonl_to, write_lang_report_to, write_module_report_to,
};
use tokmd_model::{create_export_data, create_lang_report, create_module_report};
use tokmd_scan::scan;
use tokmd_settings::{ScanOptions, TomlConfig};
use tokmd_types::{
    ChildIncludeMode, ChildrenMode, ConfigMode, DiffRow, ExportArgs, ExportFormat, LangArgs,
    ModuleArgs, RedactMode, SCHEMA_VERSION, TableFormat,
};

// ===========================================================================
// Helpers
// ===========================================================================

fn fixture_path() -> PathBuf {
    common::fixture_root().to_path_buf()
}

fn default_scan_options() -> ScanOptions {
    ScanOptions {
        config: ConfigMode::None,
        no_ignore_vcs: true,
        ..Default::default()
    }
}

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

fn run_json(args: &[&str]) -> Value {
    let output = tokmd_cmd()
        .args(args)
        .output()
        .expect("failed to execute tokmd");
    assert!(
        output.status.success(),
        "tokmd failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("stdout is not valid JSON")
}

fn run_stdout(args: &[&str]) -> String {
    let output = tokmd_cmd()
        .args(args)
        .output()
        .expect("failed to execute tokmd");
    assert!(
        output.status.success(),
        "tokmd failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("non-UTF-8 stdout")
}

fn default_lang_args(fmt: TableFormat) -> LangArgs {
    LangArgs {
        paths: vec![fixture_path()],
        format: fmt,
        top: 0,
        files: false,
        children: ChildrenMode::Collapse,
    }
}

fn default_module_args(fmt: TableFormat) -> ModuleArgs {
    ModuleArgs {
        paths: vec![fixture_path()],
        format: fmt,
        top: 0,
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

fn default_export_args(fmt: ExportFormat) -> ExportArgs {
    ExportArgs {
        paths: vec![fixture_path()],
        format: fmt,
        output: None,
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        redact: RedactMode::None,
        meta: false,
        strip_prefix: None,
    }
}

// ===========================================================================
// 1. Scan → Model → Format (Lang) pipeline
// ===========================================================================

#[test]
fn pipeline_lang_markdown_produces_table() {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let args = default_lang_args(TableFormat::Md);
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &default_scan_options(), &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("|Lang|"), "markdown must have table header");
    assert!(
        output.contains("|**Total**|"),
        "markdown must have total row"
    );
}

#[test]
fn pipeline_lang_tsv_has_correct_columns() {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let args = default_lang_args(TableFormat::Tsv);
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &default_scan_options(), &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let header = output.lines().next().expect("no TSV output");
    assert!(header.contains('\t'), "TSV header must have tabs");
}

#[test]
fn pipeline_lang_json_has_schema_version() {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let args = default_lang_args(TableFormat::Json);
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &default_scan_options(), &args).unwrap();
    let v: Value = serde_json::from_slice(&buf).unwrap();
    assert_eq!(
        v["schema_version"].as_u64().unwrap(),
        u64::from(SCHEMA_VERSION)
    );
    assert_eq!(v["mode"].as_str().unwrap(), "lang");
}

#[test]
fn pipeline_lang_with_files_includes_avg() {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let report = create_lang_report(&langs, 0, true, ChildrenMode::Collapse);
    assert!(
        report.with_files,
        "with_files flag must be set in the report"
    );
    // avg_lines should be computed for non-empty rows
    for row in &report.rows {
        if row.files > 0 {
            assert!(
                row.avg_lines > 0,
                "avg_lines should be > 0 for lang {} with {} files",
                row.lang,
                row.files
            );
        }
    }
}

#[test]
fn pipeline_lang_separate_children_has_embedded() {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let collapse = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let separate = create_lang_report(&langs, 0, false, ChildrenMode::Separate);
    assert!(
        separate.rows.len() >= collapse.rows.len(),
        "Separate mode should yield >= rows than Collapse"
    );
    assert_eq!(separate.children, ChildrenMode::Separate);
    assert_eq!(collapse.children, ChildrenMode::Collapse);
}

#[test]
fn pipeline_lang_top_limits_rows() {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let full = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    if full.rows.len() > 2 {
        let limited = create_lang_report(&langs, 1, false, ChildrenMode::Collapse);
        // top=1 keeps the #1 language + folds rest into "Other"
        assert_eq!(limited.rows.len(), 2, "top=1 should yield 1 lang + Other");
        assert_eq!(limited.rows[1].lang, "Other");
    }
}

#[test]
fn pipeline_lang_total_equals_row_sum() {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let sum_code: usize = report.rows.iter().map(|r| r.code).sum();
    let sum_files: usize = report.rows.iter().map(|r| r.files).sum();
    assert_eq!(report.total.code, sum_code, "total.code != row sum");
    assert_eq!(report.total.files, sum_files, "total.files != row sum");
}

// ===========================================================================
// 2. Scan → Module → Format pipeline
// ===========================================================================

#[test]
fn pipeline_module_markdown_produces_table() {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let report = create_module_report(&langs, &[], 2, ChildIncludeMode::Separate, 0);
    let args = default_module_args(TableFormat::Md);
    let mut buf = Vec::new();
    write_module_report_to(&mut buf, &report, &default_scan_options(), &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(
        output.contains('|'),
        "module markdown must contain table pipe chars"
    );
}

#[test]
fn pipeline_module_json_has_schema_version() {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let report = create_module_report(&langs, &[], 2, ChildIncludeMode::Separate, 0);
    let args = default_module_args(TableFormat::Json);
    let mut buf = Vec::new();
    write_module_report_to(&mut buf, &report, &default_scan_options(), &args).unwrap();
    let v: Value = serde_json::from_slice(&buf).unwrap();
    assert_eq!(
        v["schema_version"].as_u64().unwrap(),
        u64::from(SCHEMA_VERSION)
    );
    assert_eq!(v["mode"].as_str().unwrap(), "module");
}

#[test]
fn pipeline_module_custom_depth_changes_granularity() {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let depth1 = create_module_report(&langs, &[], 1, ChildIncludeMode::Separate, 0);
    let depth3 = create_module_report(&langs, &[], 3, ChildIncludeMode::Separate, 0);
    // Deeper depth should produce >= module rows (or equal for flat trees)
    assert!(
        depth3.rows.len() >= depth1.rows.len(),
        "depth=3 should yield >= rows than depth=1"
    );
}

// ===========================================================================
// 3. Scan → Export → Format pipeline
// ===========================================================================

#[test]
fn pipeline_export_csv_has_header_and_data() {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let data = create_export_data(
        &langs,
        &[],
        2,
        ChildIncludeMode::Separate,
        Some(fixture_path().as_path()),
        0,
        0,
    );
    let args = default_export_args(ExportFormat::Csv);
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &data, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let lines: Vec<&str> = output.lines().collect();
    assert!(lines.len() >= 2, "CSV must have header + data rows");
    assert!(lines[0].contains("path"), "CSV header must contain 'path'");
}

#[test]
fn pipeline_export_jsonl_valid_objects() {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let data = create_export_data(
        &langs,
        &[],
        2,
        ChildIncludeMode::Separate,
        Some(fixture_path().as_path()),
        0,
        0,
    );
    let args = default_export_args(ExportFormat::Jsonl);
    let mut buf = Vec::new();
    write_export_jsonl_to(&mut buf, &data, &default_scan_options(), &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    for (i, line) in output.lines().enumerate() {
        let v: Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("JSONL line {i} not valid JSON: {e}"));
        assert!(v.is_object(), "JSONL line {i} must be an object");
    }
}

#[test]
fn pipeline_export_json_with_meta_is_valid_receipt() {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let data = create_export_data(
        &langs,
        &[],
        2,
        ChildIncludeMode::Separate,
        Some(fixture_path().as_path()),
        0,
        0,
    );
    let mut args = default_export_args(ExportFormat::Json);
    args.meta = true;
    let mut buf = Vec::new();
    write_export_json_to(&mut buf, &data, &default_scan_options(), &args).unwrap();
    let v: Value = serde_json::from_slice(&buf).unwrap();
    assert_eq!(
        v["schema_version"].as_u64().unwrap(),
        u64::from(SCHEMA_VERSION)
    );
    assert!(v["rows"].is_array(), "export JSON must have rows array");
    assert_eq!(v["mode"].as_str().unwrap(), "export");
}

#[test]
fn pipeline_export_json_bare_is_array() {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let data = create_export_data(
        &langs,
        &[],
        2,
        ChildIncludeMode::Separate,
        Some(fixture_path().as_path()),
        0,
        0,
    );
    let args = default_export_args(ExportFormat::Json);
    let mut buf = Vec::new();
    write_export_json_to(&mut buf, &data, &default_scan_options(), &args).unwrap();
    let v: Value = serde_json::from_slice(&buf).unwrap();
    assert!(
        v.is_array(),
        "export JSON without meta must be a bare array"
    );
}

#[test]
fn pipeline_export_forward_slash_paths() {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let data = create_export_data(
        &langs,
        &[],
        2,
        ChildIncludeMode::Separate,
        Some(fixture_path().as_path()),
        0,
        0,
    );
    for row in &data.rows {
        assert!(
            !row.path.contains('\\'),
            "export path must use forward slashes: {}",
            row.path
        );
    }
}

#[test]
fn pipeline_export_csv_consistent_column_count() {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let data = create_export_data(
        &langs,
        &[],
        2,
        ChildIncludeMode::Separate,
        Some(fixture_path().as_path()),
        0,
        0,
    );
    let args = default_export_args(ExportFormat::Csv);
    let mut buf = Vec::new();
    write_export_csv_to(&mut buf, &data, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let mut col_counts = output.lines().map(|l| l.split(',').count());
    let first = col_counts.next().unwrap();
    for (i, count) in col_counts.enumerate() {
        assert_eq!(
            count, first,
            "CSV row {i} has {count} columns, expected {first}"
        );
    }
}

// ===========================================================================
// 4. CLI full pipeline tests
// ===========================================================================

#[test]
fn cli_lang_json_full_pipeline() {
    let json = run_json(&["lang", "--format", "json"]);
    assert_eq!(
        json["schema_version"].as_u64().unwrap(),
        u64::from(SCHEMA_VERSION)
    );
    assert!(json["rows"].is_array());
    assert!(json["total"].is_object());
    assert!(json["tool"]["version"].is_string());
}

#[test]
fn cli_module_json_full_pipeline() {
    let json = run_json(&["module", "--format", "json"]);
    assert_eq!(
        json["schema_version"].as_u64().unwrap(),
        u64::from(SCHEMA_VERSION)
    );
    assert!(json["rows"].is_array());
    assert!(json["total"].is_object());
    assert_eq!(json["mode"].as_str().unwrap(), "module");
}

#[test]
fn cli_export_csv_full_pipeline() {
    let stdout = run_stdout(&["export", "--format", "csv"]);
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() >= 2, "CSV must have header + data");
    assert!(lines[0].contains("path"), "CSV header must contain 'path'");
}

#[test]
fn cli_export_jsonl_full_pipeline() {
    let stdout = run_stdout(&["export", "--format", "jsonl"]);
    for (i, line) in stdout.lines().enumerate() {
        let _: Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("JSONL line {i} not valid JSON: {e}"));
    }
}

#[test]
fn cli_export_json_full_pipeline() {
    let json = run_json(&["export", "--format", "json"]);
    assert_eq!(
        json["schema_version"].as_u64().unwrap(),
        u64::from(SCHEMA_VERSION)
    );
    assert_eq!(json["mode"].as_str().unwrap(), "export");
    assert!(json["rows"].is_array());
}

#[test]
fn cli_analyze_receipt_json_pipeline() {
    let json = run_json(&["analyze", "--preset", "receipt", "--format", "json"]);
    assert_eq!(json["schema_version"].as_u64().unwrap(), 9);
    assert_eq!(json["mode"].as_str().unwrap(), "analysis");
    assert!(
        json["derived"].is_object(),
        "receipt preset must have derived"
    );
}

#[test]
fn cli_analyze_health_json_pipeline() {
    let json = run_json(&["analyze", "--preset", "health", "--format", "json"]);
    assert_eq!(json["schema_version"].as_u64().unwrap(), 9);
    assert_eq!(json["mode"].as_str().unwrap(), "analysis");
    assert!(json["derived"].is_object());
}

// ===========================================================================
// 5. Cross-crate type flow
// ===========================================================================

#[test]
fn types_flow_scan_to_model_to_format() {
    // Verify types produced by scan are consumed by model, then format
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    assert!(!report.rows.is_empty(), "fixture must produce rows");
    let args = default_lang_args(TableFormat::Json);
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &default_scan_options(), &args).unwrap();
    let v: Value = serde_json::from_slice(&buf).unwrap();
    assert_eq!(v["rows"].as_array().unwrap().len(), report.rows.len());
}

#[test]
fn lang_row_fields_preserved_through_pipeline() {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let args = default_lang_args(TableFormat::Json);
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &default_scan_options(), &args).unwrap();
    let v: Value = serde_json::from_slice(&buf).unwrap();
    let json_rows = v["rows"].as_array().unwrap();
    for (i, row) in report.rows.iter().enumerate() {
        let jr = &json_rows[i];
        assert_eq!(jr["lang"].as_str().unwrap(), row.lang);
        assert_eq!(jr["code"].as_u64().unwrap(), row.code as u64);
        assert_eq!(jr["files"].as_u64().unwrap(), row.files as u64);
        assert_eq!(jr["lines"].as_u64().unwrap(), row.lines as u64);
    }
}

#[test]
fn file_row_fields_preserved_through_export() {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let data = create_export_data(
        &langs,
        &[],
        2,
        ChildIncludeMode::Separate,
        Some(fixture_path().as_path()),
        0,
        0,
    );
    // Use bare JSON (no meta) which produces a plain array
    let args = default_export_args(ExportFormat::Json);
    let mut buf = Vec::new();
    write_export_json_to(&mut buf, &data, &default_scan_options(), &args).unwrap();
    let v: Value = serde_json::from_slice(&buf).unwrap();
    let json_rows = v.as_array().unwrap();
    for (i, row) in data.rows.iter().enumerate() {
        let jr = &json_rows[i];
        assert_eq!(jr["path"].as_str().unwrap(), row.path);
        assert_eq!(jr["code"].as_u64().unwrap(), row.code as u64);
    }
}

#[test]
fn totals_consistent_model_to_json() {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let args = default_lang_args(TableFormat::Json);
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &default_scan_options(), &args).unwrap();
    let v: Value = serde_json::from_slice(&buf).unwrap();
    assert_eq!(
        v["total"]["code"].as_u64().unwrap(),
        report.total.code as u64
    );
    assert_eq!(
        v["total"]["files"].as_u64().unwrap(),
        report.total.files as u64
    );
}

// ===========================================================================
// 6. Diff pipeline
// ===========================================================================

#[test]
fn diff_identical_reports_produces_no_rows() {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let diff_rows = compute_diff_rows(&report, &report);
    assert!(
        diff_rows.is_empty(),
        "diff of identical reports should be empty"
    );
}

#[test]
fn diff_totals_zero_for_identical() {
    let diff_rows: Vec<DiffRow> = vec![];
    let totals = compute_diff_totals(&diff_rows);
    assert_eq!(totals.delta_code, 0);
    assert_eq!(totals.delta_lines, 0);
    assert_eq!(totals.delta_files, 0);
}

#[test]
fn diff_receipt_has_correct_structure() {
    let rows = vec![DiffRow {
        lang: "Rust".into(),
        old_code: 100,
        new_code: 150,
        delta_code: 50,
        old_lines: 120,
        new_lines: 180,
        delta_lines: 60,
        old_files: 3,
        new_files: 4,
        delta_files: 1,
        old_bytes: 3000,
        new_bytes: 5000,
        delta_bytes: 2000,
        old_tokens: 800,
        new_tokens: 1200,
        delta_tokens: 400,
    }];
    let totals = compute_diff_totals(&rows);
    let receipt = create_diff_receipt("from.json", "to.json", rows, totals);
    assert_eq!(receipt.schema_version, SCHEMA_VERSION);
    assert_eq!(receipt.mode, "diff");
    assert_eq!(receipt.from_source, "from.json");
    assert_eq!(receipt.to_source, "to.json");
    assert_eq!(receipt.diff_rows.len(), 1);
    assert_eq!(receipt.totals.delta_code, 50);
}

#[test]
fn diff_receipt_serializes_to_valid_json() {
    let rows = vec![DiffRow {
        lang: "Python".into(),
        old_code: 50,
        new_code: 0,
        delta_code: -50,
        old_lines: 80,
        new_lines: 0,
        delta_lines: -80,
        old_files: 2,
        new_files: 0,
        delta_files: -2,
        old_bytes: 2000,
        new_bytes: 0,
        delta_bytes: -2000,
        old_tokens: 500,
        new_tokens: 0,
        delta_tokens: -500,
    }];
    let totals = compute_diff_totals(&rows);
    let receipt = create_diff_receipt("a.json", "b.json", rows, totals);
    let json = serde_json::to_value(&receipt).unwrap();
    assert_eq!(
        json["schema_version"].as_u64().unwrap(),
        u64::from(SCHEMA_VERSION)
    );
    assert!(json["diff_rows"].is_array());
    assert_eq!(json["diff_rows"][0]["delta_code"].as_i64().unwrap(), -50);
}

// ===========================================================================
// 7. Analysis pipeline
// ===========================================================================

#[test]
fn analysis_derived_from_export_data() {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let data = create_export_data(
        &langs,
        &[],
        2,
        ChildIncludeMode::Separate,
        Some(fixture_path().as_path()),
        0,
        0,
    );
    let derived = derive_report(&data, None);
    assert!(derived.totals.files > 0, "derived totals must have files");
    assert!(derived.totals.code > 0, "derived totals must have code");
}

#[test]
fn analysis_density_ratio_bounded() {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let data = create_export_data(
        &langs,
        &[],
        2,
        ChildIncludeMode::Separate,
        Some(fixture_path().as_path()),
        0,
        0,
    );
    let derived = derive_report(&data, None);
    let ratio = derived.doc_density.total.ratio;
    assert!(
        (0.0..=1.0).contains(&ratio),
        "doc_density ratio must be in [0,1], got {ratio}"
    );
}

#[test]
fn analysis_integrity_hash_present() {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let data = create_export_data(
        &langs,
        &[],
        2,
        ChildIncludeMode::Separate,
        Some(fixture_path().as_path()),
        0,
        0,
    );
    let derived = derive_report(&data, None);
    assert!(
        !derived.integrity.hash.is_empty(),
        "integrity hash must be present"
    );
}

// ===========================================================================
// 8. Badge generation pipeline
// ===========================================================================

#[test]
fn badge_from_scan_data_produces_svg() {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let svg = badge_svg("lines", &report.total.lines.to_string());
    assert!(svg.contains("<svg"), "badge must be valid SVG");
    assert!(svg.contains("lines"), "badge must contain label");
}

#[test]
fn cli_badge_produces_svg() {
    tokmd_cmd()
        .args(["badge", "--metric", "lines"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("lines"));
}

// ===========================================================================
// 9. Determinism
// ===========================================================================

#[test]
fn pipeline_deterministic_json_output() {
    let normalize = |s: &str| -> String {
        let re_ts = regex::Regex::new(r#""generated_at_ms"\s*:\s*\d+"#).unwrap();
        let re_ver = regex::Regex::new(r#""version"\s*:\s*"[^"]+""#).unwrap();
        let s = re_ts.replace_all(s, r#""generated_at_ms":0"#);
        let s = re_ver.replace_all(&s, r#""version":"0.0.0""#);
        s.to_string()
    };

    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let args = default_lang_args(TableFormat::Json);

    let mut buf1 = Vec::new();
    write_lang_report_to(&mut buf1, &report, &default_scan_options(), &args).unwrap();
    let mut buf2 = Vec::new();
    write_lang_report_to(&mut buf2, &report, &default_scan_options(), &args).unwrap();

    let out1 = normalize(&String::from_utf8(buf1).unwrap());
    let out2 = normalize(&String::from_utf8(buf2).unwrap());
    assert_eq!(out1, out2, "repeated formatting must be deterministic");
}

#[test]
fn pipeline_deterministic_markdown_output() {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let args = default_lang_args(TableFormat::Md);

    let mut buf1 = Vec::new();
    write_lang_report_to(&mut buf1, &report, &default_scan_options(), &args).unwrap();
    let mut buf2 = Vec::new();
    write_lang_report_to(&mut buf2, &report, &default_scan_options(), &args).unwrap();
    assert_eq!(buf1, buf2, "repeated markdown must be byte-identical");
}

// ===========================================================================
// 10. Config → Scan → Output pipeline
// ===========================================================================

#[test]
fn config_exclude_filters_languages() {
    let opts = ScanOptions {
        excluded: vec!["*.js".to_string()],
        config: ConfigMode::None,
        no_ignore_vcs: true,
        ..Default::default()
    };
    let langs = scan(&[fixture_path()], &opts).expect("scan");
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let has_js = report.rows.iter().any(|r| r.lang == "JavaScript");
    assert!(!has_js, "JavaScript should be excluded by *.js pattern");
}

#[test]
fn config_toml_parse_roundtrip() {
    let toml_str = r#"
[scan]
exclude = ["*.md", "vendor/**"]
"#;
    let config = TomlConfig::parse(toml_str).expect("valid TOML");
    let excludes = config.scan.exclude.unwrap_or_default();
    assert_eq!(excludes.len(), 2);
    assert!(excludes.contains(&"*.md".to_string()));
    assert!(excludes.contains(&"vendor/**".to_string()));
}

// ===========================================================================
// 11. Feature-gated paths graceful degradation
// ===========================================================================

#[test]
fn cli_analyze_without_git_range_succeeds() {
    // analyze with receipt preset doesn't require git range
    tokmd_cmd()
        .args(["analyze", "--preset", "receipt", "--format", "json"])
        .assert()
        .success();
}

#[test]
fn cli_help_succeeds() {
    tokmd_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("tokmd"));
}

#[test]
fn cli_lang_help_succeeds() {
    tokmd_cmd()
        .args(["lang", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("lang"));
}

// ===========================================================================
// 12. Export sort order determinism
// ===========================================================================

#[test]
fn export_rows_sorted_by_code_desc_then_path_asc() {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let data = create_export_data(
        &langs,
        &[],
        2,
        ChildIncludeMode::Separate,
        Some(fixture_path().as_path()),
        0,
        0,
    );
    for window in data.rows.windows(2) {
        let a = &window[0];
        let b = &window[1];
        let ok = a.code > b.code || (a.code == b.code && a.path <= b.path);
        assert!(
            ok,
            "rows must be sorted: {} ({}) vs {} ({})",
            a.path, a.code, b.path, b.code
        );
    }
}

#[test]
fn lang_rows_sorted_by_code_desc() {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan");
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    for window in report.rows.windows(2) {
        assert!(
            window[0].code >= window[1].code,
            "lang rows must be sorted desc by code: {} ({}) vs {} ({})",
            window[0].lang,
            window[0].code,
            window[1].lang,
            window[1].code
        );
    }
}
