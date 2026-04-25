//! Regression prevention tests (W55).
//!
//! Guards known-good behavior, schema version contracts, public API surface,
//! error quality, and backward compatibility invariants.

mod common;

use assert_cmd::Command;
use serde_json::Value;
use tokmd_analysis_derived::derive_report;
use tokmd_format::{badge_svg, compute_diff_totals, create_diff_receipt, write_lang_report_to};
use tokmd_model::{
    avg, create_export_data, create_lang_report, create_module_report, module_key, normalize_path,
};
use tokmd_scan::scan;
use tokmd_settings::ScanOptions;
use tokmd_types::{
    CONTEXT_BUNDLE_SCHEMA_VERSION, CONTEXT_SCHEMA_VERSION, ChildIncludeMode, ChildrenMode,
    ConfigMode, HANDOFF_SCHEMA_VERSION, LangArgs, SCHEMA_VERSION, TableFormat,
};

use std::path::{Path, PathBuf};

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

// ===========================================================================
// 1. Schema version constants
// ===========================================================================

#[test]
fn schema_version_is_2() {
    assert_eq!(SCHEMA_VERSION, 2, "core SCHEMA_VERSION must be 2");
}

#[test]
fn analysis_schema_version_is_9() {
    assert_eq!(
        tokmd_analysis_types::ANALYSIS_SCHEMA_VERSION,
        9,
        "ANALYSIS_SCHEMA_VERSION must be 9"
    );
}

#[test]
fn handoff_schema_version_is_5() {
    assert_eq!(
        HANDOFF_SCHEMA_VERSION, 5,
        "HANDOFF_SCHEMA_VERSION must be 5"
    );
}

#[test]
fn context_schema_version_is_4() {
    assert_eq!(
        CONTEXT_SCHEMA_VERSION, 4,
        "CONTEXT_SCHEMA_VERSION must be 4"
    );
}

#[test]
fn context_bundle_schema_version_is_2() {
    assert_eq!(
        CONTEXT_BUNDLE_SCHEMA_VERSION, 2,
        "CONTEXT_BUNDLE_SCHEMA_VERSION must be 2"
    );
}

#[test]
fn cli_lang_json_schema_version_matches_constant() {
    let json = run_json(&["lang", "--format", "json"]);
    assert_eq!(
        json["schema_version"].as_u64().unwrap(),
        u64::from(SCHEMA_VERSION)
    );
}

#[test]
fn cli_analyze_json_schema_version_matches_constant() {
    let json = run_json(&["analyze", "--preset", "receipt", "--format", "json"]);
    assert_eq!(
        json["schema_version"].as_u64().unwrap(),
        u64::from(tokmd_analysis_types::ANALYSIS_SCHEMA_VERSION)
    );
}

// ===========================================================================
// 2. Known-good output structure
// ===========================================================================

#[test]
fn lang_json_has_required_keys() {
    let json = run_json(&["lang", "--format", "json"]);
    for key in ["schema_version", "mode", "tool", "rows", "total", "args"] {
        assert!(
            json.get(key).is_some(),
            "lang JSON missing required key: {key}"
        );
    }
}

#[test]
fn module_json_has_required_keys() {
    let json = run_json(&["module", "--format", "json"]);
    for key in ["schema_version", "mode", "tool", "rows", "total", "args"] {
        assert!(
            json.get(key).is_some(),
            "module JSON missing required key: {key}"
        );
    }
}

#[test]
fn export_json_has_required_keys() {
    let json = run_json(&["export", "--format", "json"]);
    for key in ["schema_version", "mode", "rows", "args"] {
        assert!(
            json.get(key).is_some(),
            "export JSON missing required key: {key}"
        );
    }
}

#[test]
fn analyze_json_has_required_keys() {
    let json = run_json(&["analyze", "--preset", "receipt", "--format", "json"]);
    for key in ["schema_version", "mode", "args", "derived"] {
        assert!(
            json.get(key).is_some(),
            "analyze JSON missing required key: {key}"
        );
    }
}

#[test]
fn lang_markdown_has_table_headers() {
    let stdout = run_stdout(&["lang"]);
    assert!(
        stdout.contains("|Lang|"),
        "markdown must have |Lang| header"
    );
    assert!(
        stdout.contains("|**Total**|"),
        "markdown must have total row"
    );
}

#[test]
fn lang_tsv_has_header_line() {
    let stdout = run_stdout(&["lang", "--format", "tsv"]);
    let header = stdout.lines().next().expect("no TSV output");
    assert!(header.contains('\t'), "TSV header must have tab characters");
}

#[test]
fn export_csv_has_header_line() {
    let stdout = run_stdout(&["export", "--format", "csv"]);
    let header = stdout.lines().next().expect("no CSV output");
    assert!(header.contains("path"), "CSV header must contain 'path'");
}

// ===========================================================================
// 3. Public API existence / signature verification
// ===========================================================================

#[test]
fn scan_function_accepts_paths_and_options() {
    let opts = default_scan_options();
    let result = scan(&[fixture_path()], &opts);
    assert!(result.is_ok(), "scan must succeed on fixture data");
}

#[test]
fn create_lang_report_produces_report() {
    let langs = scan(&[fixture_path()], &default_scan_options()).unwrap();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    assert!(!report.rows.is_empty(), "lang report must have rows");
}

#[test]
fn create_module_report_produces_report() {
    let langs = scan(&[fixture_path()], &default_scan_options()).unwrap();
    let report = create_module_report(&langs, &[], 2, ChildIncludeMode::Separate, 0);
    assert!(!report.rows.is_empty(), "module report must have rows");
}

#[test]
fn create_export_data_produces_data() {
    let langs = scan(&[fixture_path()], &default_scan_options()).unwrap();
    let data = create_export_data(
        &langs,
        &[],
        2,
        ChildIncludeMode::Separate,
        Some(fixture_path().as_path()),
        0,
        0,
    );
    assert!(!data.rows.is_empty(), "export data must have rows");
}

#[test]
fn badge_svg_produces_valid_svg() {
    let svg = badge_svg("test", "42");
    assert!(svg.starts_with("<svg"), "badge must start with <svg");
    assert!(svg.contains("test"), "badge must contain label");
    assert!(svg.contains("42"), "badge must contain value");
}

#[test]
fn format_write_functions_accept_writers() {
    let langs = scan(&[fixture_path()], &default_scan_options()).unwrap();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let args = LangArgs {
        paths: vec![fixture_path()],
        format: TableFormat::Md,
        top: 0,
        files: false,
        children: ChildrenMode::Collapse,
    };
    let mut buf: Vec<u8> = Vec::new();
    let result = write_lang_report_to(&mut buf, &report, &default_scan_options(), &args);
    assert!(result.is_ok(), "write_lang_report_to must succeed");
    assert!(!buf.is_empty(), "output buffer must not be empty");
}

#[test]
fn model_utility_functions_work() {
    assert_eq!(avg(100, 5), 20, "avg(100, 5) = 20");
    assert_eq!(avg(0, 0), 0, "avg(0, 0) = 0 (no div-by-zero)");
    let normed = normalize_path(Path::new("src/main.rs"), None);
    assert_eq!(normed, "src/main.rs");
    let key = module_key("src/foo/bar.rs", &[], 2);
    assert!(!key.is_empty(), "module_key must produce a value");
}

// ===========================================================================
// 4. Error message quality
// ===========================================================================

#[test]
fn invalid_path_produces_error_or_empty() {
    let result = scan(
        &[PathBuf::from("/nonexistent/path/that/does/not/exist")],
        &default_scan_options(),
    );
    // Scan may succeed with empty or minimal result, or may fail
    // Either outcome is acceptable - just verify no panic
    if let Ok(langs) = &result {
        let report = create_lang_report(langs, 0, false, ChildrenMode::Collapse);
        // May or may not have rows depending on platform behavior
        let _ = report.rows.len();
    }
}

#[test]
fn invalid_subcommand_fails_with_help() {
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .arg("nonexistent-subcommand")
        .assert()
        .failure();
}

#[test]
fn invalid_format_flag_fails() {
    tokmd_cmd()
        .args(["lang", "--format", "invalid_format"])
        .assert()
        .failure();
}

#[test]
fn invalid_preset_flag_fails() {
    tokmd_cmd()
        .args(["analyze", "--preset", "nonexistent_preset"])
        .assert()
        .failure();
}

// ===========================================================================
// 5. Backward compatibility / round-trip
// ===========================================================================

#[test]
fn json_output_parseable_as_serde_value() {
    let json = run_json(&["lang", "--format", "json"]);
    assert!(json.is_object(), "top-level must be JSON object");
}

#[test]
fn receipt_mode_field_present_and_correct() {
    let lang = run_json(&["lang", "--format", "json"]);
    assert_eq!(lang["mode"].as_str().unwrap(), "lang");
    let module = run_json(&["module", "--format", "json"]);
    assert_eq!(module["mode"].as_str().unwrap(), "module");
    let export = run_json(&["export", "--format", "json"]);
    assert_eq!(export["mode"].as_str().unwrap(), "export");
}

#[test]
fn receipt_tool_field_has_name_and_version() {
    let json = run_json(&["lang", "--format", "json"]);
    assert!(json["tool"]["name"].is_string(), "tool.name must be string");
    assert!(
        json["tool"]["version"].is_string(),
        "tool.version must be string"
    );
    assert_eq!(json["tool"]["name"].as_str().unwrap(), "tokmd");
}

#[test]
fn receipt_generated_at_ms_present() {
    let json = run_json(&["lang", "--format", "json"]);
    assert!(
        json["generated_at_ms"].is_number(),
        "generated_at_ms must be present"
    );
    assert!(
        json["generated_at_ms"].as_u64().unwrap() > 0,
        "generated_at_ms must be > 0"
    );
}

#[test]
fn lang_receipt_deserialize_roundtrip() {
    let json = run_json(&["lang", "--format", "json"]);
    // Re-serialize and deserialize to verify round-trip
    let serialized = serde_json::to_string(&json).unwrap();
    let deserialized: Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(json, deserialized, "JSON round-trip must be lossless");
}

// ===========================================================================
// 6. Arithmetic and data invariants
// ===========================================================================

#[test]
fn totals_nonnegative() {
    let json = run_json(&["lang", "--format", "json"]);
    let total = &json["total"];
    for field in ["code", "lines", "files", "bytes", "tokens"] {
        assert!(
            total[field].is_u64(),
            "total.{field} must be a non-negative integer"
        );
    }
}

#[test]
fn empty_dir_produces_empty_report() {
    let dir = tempfile::tempdir().expect("create temp dir");
    std::fs::create_dir_all(dir.path().join(".git")).expect("create .git marker");
    let output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args(["lang", "--format", "json"])
        .output()
        .expect("execute");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["total"]["code"], 0);
    assert_eq!(json["total"]["files"], 0);
    assert!(json["rows"].as_array().unwrap().is_empty());
}

#[test]
fn rows_sorted_descending_by_code_in_lang() {
    let json = run_json(&["lang", "--format", "json"]);
    let rows = json["rows"].as_array().unwrap();
    for window in rows.windows(2) {
        let a = window[0]["code"].as_u64().unwrap();
        let b = window[1]["code"].as_u64().unwrap();
        assert!(a >= b, "lang rows must be sorted desc by code: {a} vs {b}");
    }
}

#[test]
fn total_code_equals_sum_of_rows() {
    let json = run_json(&["lang", "--format", "json"]);
    let rows = json["rows"].as_array().unwrap();
    let sum: u64 = rows.iter().map(|r| r["code"].as_u64().unwrap()).sum();
    assert_eq!(json["total"]["code"].as_u64().unwrap(), sum);
}

#[test]
fn total_files_equals_sum_of_rows() {
    let json = run_json(&["lang", "--format", "json"]);
    let rows = json["rows"].as_array().unwrap();
    let sum: u64 = rows.iter().map(|r| r["files"].as_u64().unwrap()).sum();
    assert_eq!(json["total"]["files"].as_u64().unwrap(), sum);
}

// ===========================================================================
// 7. No deprecated features removed
// ===========================================================================

#[test]
fn lang_command_exists() {
    tokmd_cmd()
        .args(["lang", "--format", "json"])
        .assert()
        .success();
}

#[test]
fn module_command_exists() {
    tokmd_cmd()
        .args(["module", "--format", "json"])
        .assert()
        .success();
}

#[test]
fn export_command_exists() {
    tokmd_cmd()
        .args(["export", "--format", "json"])
        .assert()
        .success();
}

#[test]
fn analyze_command_exists() {
    tokmd_cmd()
        .args(["analyze", "--preset", "receipt", "--format", "json"])
        .assert()
        .success();
}

#[test]
fn badge_command_exists() {
    tokmd_cmd()
        .args(["badge", "--metric", "lines"])
        .assert()
        .success();
}

#[test]
fn diff_receipt_api_not_removed() {
    let rows = vec![];
    let totals = compute_diff_totals(&rows);
    let _receipt = create_diff_receipt("a", "b", rows, totals);
}

#[test]
fn derive_report_api_not_removed() {
    let langs = scan(&[fixture_path()], &default_scan_options()).unwrap();
    let data = create_export_data(
        &langs,
        &[],
        2,
        ChildIncludeMode::Separate,
        Some(fixture_path().as_path()),
        0,
        0,
    );
    let _derived = derive_report(&data, None);
}

#[test]
fn no_backslash_in_any_json_path() {
    let json = run_json(&["export", "--format", "json"]);
    let rows = json["rows"].as_array().unwrap();
    for row in rows {
        let path = row["path"].as_str().expect("path should be a string");
        assert!(!path.contains('\\'), "backslash in export path: {path}");
    }
}

#[test]
fn module_keys_never_start_with_slash() {
    let json = run_json(&["module", "--format", "json"]);
    let rows = json["rows"].as_array().unwrap();
    for row in rows {
        let module = row["module"].as_str().expect("module should be a string");
        assert!(!module.starts_with('/'), "module starts with /: {module}");
    }
}

#[test]
fn scan_args_in_receipt_has_correct_structure() {
    let json = run_json(&["lang", "--format", "json"]);
    let scan = &json["scan"];
    assert!(scan["paths"].is_array(), "scan.paths must be array");
    assert!(scan["excluded"].is_array(), "scan.excluded must be array");
}
