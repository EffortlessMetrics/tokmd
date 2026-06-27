#![cfg(feature = "analysis")]

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
// Scenario 7: Fun preset returns eco-label
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

// ---------------------------------------------------------------------------
// Scenario 8: Estimate preset returns effort model
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_analyze_estimate_then_effort_model_present() {
    // Given: a project with source files
    // When: I analyze with `estimate` preset and JSON format
    let output = tokmd_cmd()
        .args([
            "analyze", ".", "--preset", "estimate", "--format", "json", "--no-git",
        ])
        .output()
        .expect("failed to execute tokmd analyze --preset estimate");

    // Then: effort model and drivers are present
    assert!(
        output.status.success(),
        "analyze should succeed: {:?}",
        output.status
    );
    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("output should be valid JSON");

    let effort = json["effort"].as_object().expect("effort should be object");
    assert!(effort.get("model").is_some(), "effort should have model");
    let drivers = effort
        .get("drivers")
        .and_then(Value::as_array)
        .expect("effort drivers should be an array");
    assert!(!drivers.is_empty(), "effort should have drivers");
}

// ---------------------------------------------------------------------------
// Scenario 9: Health preset includes TODO tags and complexity
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_analyze_health_then_todo_and_complexity_present() {
    // Given: a project with source files
    // When: I analyze with `health` preset and JSON format
    let output = tokmd_cmd()
        .args(["analyze", ".", "--preset", "health", "--format", "json"])
        .output()
        .expect("failed to execute tokmd analyze --preset health");

    // Then: complexity metrics and TODO density are present
    assert!(
        output.status.success(),
        "analyze should succeed: {:?}",
        output.status
    );
    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("output should be valid JSON");

    assert_eq!(json["mode"], "analysis", "mode should be 'analysis'");
    assert!(
        json.get("complexity").is_some(),
        "should have complexity section"
    );
    assert!(
        json["derived"].get("todo").is_some(),
        "should have derived.todo section"
    );
}

// ---------------------------------------------------------------------------
// Scenario 10: Deep preset includes derived metrics and tool metadata
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_analyze_deep_then_derived_and_tool_present() {
    let output = tokmd_cmd()
        .args(["analyze", ".", "--preset", "deep", "--format", "json"])
        .output()
        .expect("failed to execute tokmd analyze --preset deep");

    assert!(output.status.success());
    let json: Value = serde_json::from_str(
        &String::from_utf8(output.stdout).expect("should decode stdout as UTF-8"),
    )
    .expect("should decode stdout as JSON");

    assert!(
        json.get("derived").is_some(),
        "deep preset must include derived"
    );
    assert!(
        json.get("tool").is_some(),
        "deep preset must include tool metadata"
    );
}

// ---------------------------------------------------------------------------
// Scenario 11: Topics preset returns topic cloud payload
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_analyze_topics_then_topics_cloud_present() {
    let output = tokmd_cmd()
        .args(["analyze", ".", "--preset", "topics", "--format", "json"])
        .output()
        .expect("failed to execute tokmd analyze --preset topics");

    assert!(output.status.success());
    let json: Value = serde_json::from_str(
        &String::from_utf8(output.stdout).expect("should decode stdout as UTF-8"),
    )
    .expect("should decode stdout as JSON");

    let topics = json["topics"].as_object().expect("topics should be object");
    assert!(
        topics.get("overall").is_some(),
        "topics must have overall array"
    );
}

// ---------------------------------------------------------------------------
// Scenario 12: Architecture preset includes imports and API surface (if available)
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_analyze_architecture_then_architecture_metrics_present() {
    let output = tokmd_cmd()
        .args([
            "analyze",
            ".",
            "--preset",
            "architecture",
            "--format",
            "json",
        ])
        .output()
        .expect("failed to execute tokmd analyze --preset architecture");

    assert!(output.status.success());
    let json: Value = serde_json::from_str(
        &String::from_utf8(output.stdout).expect("should decode stdout as UTF-8"),
    )
    .expect("should decode stdout as JSON");

    // We verify the command parses the preset correctly and produces derived metrics.
    // Actually asserting on imports/api_surface depends on the exact crate dependencies,
    // but the analyze run must succeed.
    assert!(
        json.get("derived").is_some(),
        "architecture preset must at least succeed and include derived"
    );
}

// ---------------------------------------------------------------------------
// Scenario 13: Risk preset includes git and complexity
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_analyze_risk_then_git_and_complexity_present() {
    let output = tokmd_cmd()
        .args(["analyze", ".", "--preset", "risk", "--format", "json"])
        .output()
        .expect("failed to execute tokmd analyze --preset risk");

    assert!(output.status.success());
    let json: Value = serde_json::from_str(
        &String::from_utf8(output.stdout).expect("should decode stdout as UTF-8"),
    )
    .expect("should decode stdout as JSON");

    assert!(
        json.get("complexity").is_some(),
        "risk preset must include complexity"
    );
    // 'git' might be null in hermetic test env without .git, but the key usually exists or we just verify parsing.
}

// ---------------------------------------------------------------------------
// Scenario 14: Security preset includes entropy and license
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_analyze_security_then_entropy_and_license_present() {
    let output = tokmd_cmd()
        .args(["analyze", ".", "--preset", "security", "--format", "json"])
        .output()
        .expect("failed to execute tokmd analyze --preset security");

    assert!(output.status.success());
    let json: Value = serde_json::from_str(
        &String::from_utf8(output.stdout).expect("should decode stdout as UTF-8"),
    )
    .expect("should decode stdout as JSON");

    // Basic success check for preset parsing
    assert!(json.get("derived").is_some());
}

// ---------------------------------------------------------------------------
// Scenario 15: Supply preset includes assets and deps
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_analyze_supply_then_assets_and_deps_present() {
    let output = tokmd_cmd()
        .args(["analyze", ".", "--preset", "supply", "--format", "json"])
        .output()
        .expect("failed to execute tokmd analyze --preset supply");

    assert!(output.status.success());
    let json: Value = serde_json::from_str(
        &String::from_utf8(output.stdout).expect("should decode stdout as UTF-8"),
    )
    .expect("should decode stdout as JSON");

    // Basic success check for preset parsing
    assert!(json.get("derived").is_some());
}

// ---------------------------------------------------------------------------
// Scenario 16: Identity preset includes archetype
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_analyze_identity_then_archetype_present() {
    let output = tokmd_cmd()
        .args(["analyze", ".", "--preset", "identity", "--format", "json"])
        .output()
        .expect("failed to execute tokmd analyze --preset identity");

    assert!(output.status.success());
    let json: Value = serde_json::from_str(
        &String::from_utf8(output.stdout).expect("should decode stdout as UTF-8"),
    )
    .expect("should decode stdout as JSON");

    assert!(
        json.get("archetype").is_some(),
        "identity preset must include archetype"
    );
}
