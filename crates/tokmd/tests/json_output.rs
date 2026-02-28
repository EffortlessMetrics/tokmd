//! E2E tests validating JSON output structure for all major CLI commands.
//!
//! These tests exercise `--format json` on each command and verify the
//! top-level shape of the emitted JSON using `serde_json`.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

// ---------------------------------------------------------------------------
// lang --format json
// ---------------------------------------------------------------------------

#[test]
fn json_output_lang_is_valid_json() {
    let output = tokmd_cmd()
        .arg("lang")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to execute tokmd lang");

    assert!(output.status.success(), "tokmd lang failed");

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("output is not valid JSON");

    assert_eq!(json["mode"], "lang", "mode should be 'lang'");
    assert!(
        json["schema_version"].is_number(),
        "schema_version should be present"
    );
    assert!(
        json["generated_at_ms"].is_number(),
        "generated_at_ms should be present"
    );
    assert!(json["tool"].is_object(), "tool metadata should be present");
    assert!(json["rows"].is_array(), "rows array should be present");
}

#[test]
fn json_output_lang_rows_have_expected_fields() {
    let output = tokmd_cmd()
        .arg("lang")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to execute tokmd lang");

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("output is not valid JSON");

    let rows = json["rows"].as_array().expect("rows is an array");
    assert!(!rows.is_empty(), "should detect at least one language");

    let first = &rows[0];
    assert!(first["lang"].is_string(), "row should have a lang field");
    assert!(first["code"].is_number(), "row should have code count");
    assert!(first["lines"].is_number(), "row should have lines count");
}

#[test]
fn json_output_lang_contains_rust() {
    let output = tokmd_cmd()
        .arg("lang")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to execute tokmd lang");

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("output is not valid JSON");

    let rows = json["rows"].as_array().expect("rows is an array");
    let has_rust = rows.iter().any(|r| r["lang"] == "Rust");
    assert!(has_rust, "fixture should contain Rust files");
}

#[test]
fn json_output_lang_total_present() {
    let output = tokmd_cmd()
        .arg("lang")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to execute tokmd lang");

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("output is not valid JSON");

    assert!(json["total"].is_object(), "total object should be present");
    assert!(
        json["total"]["code"].is_number(),
        "total should have code count"
    );
}

// ---------------------------------------------------------------------------
// module --format json
// ---------------------------------------------------------------------------

#[test]
fn json_output_module_is_valid_json() {
    let output = tokmd_cmd()
        .arg("module")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to execute tokmd module");

    assert!(output.status.success(), "tokmd module failed");

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("output is not valid JSON");

    assert_eq!(json["mode"], "module", "mode should be 'module'");
    assert!(
        json["schema_version"].is_number(),
        "schema_version should be present"
    );
    assert!(
        json["generated_at_ms"].is_number(),
        "generated_at_ms should be present"
    );
    assert!(json["tool"].is_object(), "tool metadata should be present");
    assert!(json["rows"].is_array(), "rows array should be present");
}

#[test]
fn json_output_module_rows_have_expected_fields() {
    let output = tokmd_cmd()
        .arg("module")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to execute tokmd module");

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("output is not valid JSON");

    let rows = json["rows"].as_array().expect("rows is an array");
    assert!(!rows.is_empty(), "should detect at least one module");

    let first = &rows[0];
    assert!(
        first["module"].is_string(),
        "row should have a module field"
    );
    assert!(first["code"].is_number(), "row should have code count");
}

#[test]
fn json_output_module_has_root() {
    let output = tokmd_cmd()
        .arg("module")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to execute tokmd module");

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("output is not valid JSON");

    let rows = json["rows"].as_array().expect("rows is an array");
    let has_root = rows.iter().any(|r| r["module"] == "(root)");
    assert!(has_root, "fixture should have a (root) module");
}

#[test]
fn json_output_module_total_present() {
    let output = tokmd_cmd()
        .arg("module")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to execute tokmd module");

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("output is not valid JSON");

    assert!(json["total"].is_object(), "total should be present");
}

// ---------------------------------------------------------------------------
// export --format json  (single JSON object)
// ---------------------------------------------------------------------------

#[test]
fn json_output_export_json_is_valid() {
    let output = tokmd_cmd()
        .arg("export")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to execute tokmd export");

    assert!(output.status.success(), "tokmd export --format json failed");

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("output is not valid JSON");

    assert_eq!(json["mode"], "export", "mode should be 'export'");
    assert!(
        json["schema_version"].is_number(),
        "schema_version should be present"
    );
    assert!(
        json["generated_at_ms"].is_number(),
        "generated_at_ms should be present"
    );
}

#[test]
fn json_output_export_json_has_rows() {
    let output = tokmd_cmd()
        .arg("export")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to execute tokmd export");

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("output is not valid JSON");

    assert!(json["rows"].is_array(), "rows array should be present");
    let rows = json["rows"].as_array().unwrap();
    assert!(!rows.is_empty(), "should have at least one file row");

    let first = &rows[0];
    assert!(first["path"].is_string(), "file row should have path");
    assert!(
        first["lang"].is_string(),
        "file row should have lang"
    );
    assert!(first["code"].is_number(), "file row should have code count");
}

// ---------------------------------------------------------------------------
// export --format jsonl  (newline-delimited JSON)
// ---------------------------------------------------------------------------

#[test]
fn json_output_export_jsonl_lines_are_valid() {
    let output = tokmd_cmd()
        .arg("export")
        .arg("--format")
        .arg("jsonl")
        .output()
        .expect("failed to execute tokmd export");

    assert!(
        output.status.success(),
        "tokmd export --format jsonl failed"
    );

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(lines.len() >= 2, "should have meta + at least one data row");

    // Every line must be valid JSON
    for (i, line) in lines.iter().enumerate() {
        let _: Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("line {} is not valid JSON: {}", i + 1, e));
    }

    // First line is the meta record
    let meta: Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(meta["mode"], "export", "meta record should have mode=export");
    assert!(
        meta["schema_version"].is_number(),
        "meta should have schema_version"
    );
}

#[test]
fn json_output_export_jsonl_data_rows_have_path() {
    let output = tokmd_cmd()
        .arg("export")
        .arg("--format")
        .arg("jsonl")
        .output()
        .expect("failed to execute tokmd export");

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();

    // Data rows (skip meta)
    for line in lines.iter().skip(1) {
        let row: Value = serde_json::from_str(line).unwrap();
        assert!(
            row["path"].is_string(),
            "data row should have a path field"
        );
    }
}

// ---------------------------------------------------------------------------
// analyze --format json --preset receipt
// ---------------------------------------------------------------------------

#[test]
fn json_output_analyze_receipt_is_valid() {
    let output = tokmd_cmd()
        .arg("analyze")
        .arg("--format")
        .arg("json")
        .arg("--preset")
        .arg("receipt")
        .output()
        .expect("failed to execute tokmd analyze");

    assert!(output.status.success(), "tokmd analyze failed");

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("output is not valid JSON");

    assert_eq!(json["mode"], "analysis", "mode should be 'analysis'");
    assert!(
        json["schema_version"].is_number(),
        "schema_version should be present"
    );
    assert!(
        json["generated_at_ms"].is_number(),
        "generated_at_ms should be present"
    );
    assert!(json["source"].is_object(), "source should be present");
    assert!(json["args"].is_object(), "args should be present");
}

#[test]
fn json_output_analyze_receipt_has_derived() {
    let output = tokmd_cmd()
        .arg("analyze")
        .arg("--format")
        .arg("json")
        .arg("--preset")
        .arg("receipt")
        .output()
        .expect("failed to execute tokmd analyze");

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("output is not valid JSON");

    assert!(
        json["derived"].is_object(),
        "derived section should be present"
    );
}

// ---------------------------------------------------------------------------
// badge (SVG output — no --format json, verify it still succeeds)
// ---------------------------------------------------------------------------

#[test]
fn json_output_badge_produces_svg() {
    tokmd_cmd()
        .arg("badge")
        .arg("--metric")
        .arg("lines")
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"));
}
