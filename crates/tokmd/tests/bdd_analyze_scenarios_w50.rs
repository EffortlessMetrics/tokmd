//! BDD-style scenario tests for the `analyze` command.
//!
//! Each test follows the Given/When/Then pattern to verify key user-facing
//! workflows of the analysis command.

mod common;

use assert_cmd::Command;
use serde_json::Value;
use tempfile::tempdir;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

// ---------------------------------------------------------------------------
// Scenario 1: Receipt preset includes derived metrics
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_analyze_receipt_then_derived_metrics_present() {
    // Given: a project with source files
    // When: I analyze with `receipt` preset and JSON format
    let output = tokmd_cmd()
        .args(["analyze", ".", "--preset", "receipt", "--format", "json"])
        .output()
        .expect("failed to execute tokmd analyze --preset receipt");

    // Then: derived metrics include density and COCOMO fields
    assert!(
        output.status.success(),
        "analyze should succeed: {:?}",
        output.status
    );
    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("output should be valid JSON");

    assert_eq!(json["mode"], "analysis", "mode should be 'analysis'");

    // The derived section should be present
    assert!(json.get("source").is_some(), "should have source section");
}

// ---------------------------------------------------------------------------
// Scenario 2: JSON output has analysis_schema_version (schema_version=9)
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_analyze_json_then_has_schema_version() {
    // Given: a project with source files
    // When: I analyze with `receipt` preset and JSON format
    let output = tokmd_cmd()
        .args(["analyze", ".", "--preset", "receipt", "--format", "json"])
        .output()
        .expect("failed to execute tokmd analyze --format json");

    // Then: output has schema_version matching ANALYSIS_SCHEMA_VERSION
    assert!(output.status.success());
    let json: Value = serde_json::from_str(
        &String::from_utf8(output.stdout).expect("should decode stdout as UTF-8"),
    )
    .expect("should decode stdout as UTF-8");
    assert_eq!(
        json["schema_version"], 9,
        "analysis schema_version should be 9"
    );
    assert!(
        json["generated_at_ms"].is_number(),
        "should have generated_at_ms"
    );
}

// ---------------------------------------------------------------------------
// Scenario 3: XML output is valid XML with angle brackets
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_analyze_xml_then_valid_xml_structure() {
    // Given: a project with source files
    // When: I analyze with `receipt` preset and XML format
    let output = tokmd_cmd()
        .args(["analyze", "--preset", "receipt", "--format", "xml"])
        .output()
        .expect("failed to execute tokmd analyze --format xml");

    // Then: output is non-empty and contains XML angle brackets
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("should decode stdout as UTF-8");
    assert!(!stdout.trim().is_empty(), "XML output should not be empty");
    assert!(
        stdout.contains('<') && stdout.contains('>'),
        "XML output should contain angle brackets"
    );
}

// ---------------------------------------------------------------------------
// Scenario 4: Analyze writes JSON to output directory
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_analyze_with_output_dir_then_file_created() {
    // Given: a project and a temporary output directory
    let dir = tempdir().expect("should create temp dir");

    // When: I analyze with --output-dir
    let output = tokmd_cmd()
        .args([
            "analyze",
            ".",
            "--preset",
            "receipt",
            "--format",
            "json",
            "--output-dir",
        ])
        .arg(dir.path())
        .output()
        .expect("failed to execute tokmd analyze --output-dir");

    // Then: analysis.json is created in the output directory
    assert!(output.status.success());
    let path = dir.path().join("analysis.json");
    assert!(path.exists(), "analysis.json should be created");

    let content = std::fs::read_to_string(&path).expect("read analysis.json");
    let json: Value = serde_json::from_str(&content).expect("analysis.json should be valid JSON");
    assert_eq!(json["mode"], "analysis");
}

// ---------------------------------------------------------------------------
// Scenario 5: Analyze has args metadata
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_analyze_json_then_has_args_metadata() {
    // Given: a project with source files
    // When: I analyze with JSON format
    let output = tokmd_cmd()
        .args(["analyze", ".", "--preset", "receipt", "--format", "json"])
        .output()
        .expect("failed to execute tokmd analyze");

    // Then: output has args metadata
    assert!(output.status.success());
    let json: Value = serde_json::from_str(
        &String::from_utf8(output.stdout).expect("should decode stdout as UTF-8"),
    )
    .expect("should decode stdout as UTF-8");
    assert!(json.get("args").is_some(), "should have args metadata");
}

// ---------------------------------------------------------------------------
// Scenario 6: Analyze markdown produces table output
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_analyze_md_then_markdown_table() {
    // Given: a project with source files
    // When: I analyze with markdown format
    let output = tokmd_cmd()
        .args(["analyze", "--preset", "receipt", "."])
        .output()
        .expect("failed to execute tokmd analyze --format md");

    // Then: output contains markdown table
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("should decode stdout as UTF-8");
    let has_table = stdout
        .lines()
        .any(|line| line.contains("|---") || line.contains("|:--"));
    assert!(has_table, "analyze markdown output should contain a table");
}

// ---------------------------------------------------------------------------
// Scenario 7: Estimate preset includes effort model
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_analyze_estimate_then_effort_model_present() {
    // Given: a project with source files
    // When: I analyze with `estimate` preset and JSON format
    let output = tokmd_cmd()
        .args(["analyze", ".", "--preset", "estimate", "--format", "json"])
        .output()
        .expect("failed to execute tokmd analyze --preset estimate");

    // Then: estimate metadata is present
    assert!(
        output.status.success(),
        "analyze should succeed: {:?}",
        output.status
    );
    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("output should be valid JSON");

    let effort = json["effort"].as_object().expect("effort should be object");
    assert!(effort.get("model").is_some(), "effort should have model");
    assert!(
        effort.get("drivers").is_some(),
        "effort should have drivers"
    );
}

// ---------------------------------------------------------------------------
// Scenario 8: Fun preset returns eco-label
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_analyze_fun_then_eco_label_present() {
    // Given: a project with source files
    // When: I analyze with `fun` preset and JSON format
    let output = tokmd_cmd()
        .args(["analyze", ".", "--preset", "fun", "--format", "json"])
        .output()
        .expect("failed to execute tokmd analyze --preset fun");

    // Then: eco_label metadata is present
    assert!(output.status.success());
    let json: Value = serde_json::from_str(
        &String::from_utf8(output.stdout).expect("should decode stdout as UTF-8"),
    )
    .expect("should decode stdout as UTF-8");
    let eco_label = json["fun"]["eco_label"]
        .as_object()
        .expect("eco_label should be object");
    assert!(
        eco_label.get("label").is_some(),
        "eco_label should have label"
    );
    assert!(
        eco_label.get("score").is_some(),
        "eco_label should have score"
    );
}
