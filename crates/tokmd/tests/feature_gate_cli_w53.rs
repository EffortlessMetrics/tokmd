//! CLI feature-gate boundary tests.
//!
//! Verifies that the tokmd CLI degrades gracefully when optional features
//! may be absent, that help text is accurate, and that JSON output always
//! has the expected structure.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;

fn tokmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
}

fn fixture() -> std::path::PathBuf {
    common::fixture_root().to_path_buf()
}

// ── help text tests ──────────────────────────────────────────────────

#[test]
fn help_text_exits_successfully() {
    tokmd().arg("--help").assert().success();
}

#[test]
fn help_text_mentions_analyze() {
    tokmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("analyze"));
}

#[test]
fn analyze_help_text_mentions_preset() {
    tokmd()
        .args(["analyze", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("preset"));
}

#[test]
fn export_help_text_exits_successfully() {
    tokmd().args(["export", "--help"]).assert().success();
}

#[test]
fn lang_help_text_exits_successfully() {
    tokmd().args(["lang", "--help"]).assert().success();
}

#[test]
fn module_help_text_exits_successfully() {
    tokmd().args(["module", "--help"]).assert().success();
}

// ── JSON output structure tests ──────────────────────────────────────

#[test]
fn lang_json_output_has_schema_version() {
    let output = tokmd()
        .current_dir(fixture())
        .args(["lang", "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        json.get("schema_version").is_some(),
        "JSON must include schema_version"
    );
}

#[test]
fn module_json_output_has_schema_version() {
    let output = tokmd()
        .current_dir(fixture())
        .args(["module", "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json.get("schema_version").is_some());
}

#[test]
fn analyze_receipt_json_has_schema_version() {
    let output = tokmd()
        .current_dir(fixture())
        .args(["analyze", "--preset", "receipt", "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json.get("schema_version").is_some());
}

#[test]
fn analyze_receipt_json_has_warnings_field() {
    let output = tokmd()
        .current_dir(fixture())
        .args(["analyze", "--preset", "receipt", "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        json.get("warnings").is_some(),
        "analyze JSON must include warnings array"
    );
}

#[test]
fn analyze_receipt_json_has_status_field() {
    let output = tokmd()
        .current_dir(fixture())
        .args(["analyze", "--preset", "receipt", "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        json.get("status").is_some(),
        "analyze JSON must include status"
    );
}

#[test]
fn analyze_receipt_json_has_tool_field() {
    let output = tokmd()
        .current_dir(fixture())
        .args(["analyze", "--preset", "receipt", "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let tool = json.get("tool").expect("must have tool field");
    assert_eq!(tool.get("name").unwrap().as_str().unwrap(), "tokmd");
}

// ── graceful degradation tests ───────────────────────────────────────

#[test]
fn analyze_with_unknown_preset_shows_error() {
    tokmd()
        .current_dir(fixture())
        .args(["analyze", "--preset", "nonexistent_w53"])
        .assert()
        .failure();
}

#[test]
fn export_json_produces_valid_json() {
    let output = tokmd()
        .current_dir(fixture())
        .args(["export", "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    // JSONL: each non-empty line should parse
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(line);
        assert!(parsed.is_ok(), "each line should be valid JSON: {line}");
    }
}

#[test]
fn version_flag_exits_successfully() {
    tokmd().arg("--version").assert().success();
}

// ── cockpit command feature gate ─────────────────────────────────────

#[cfg(feature = "git")]
#[test]
fn cockpit_help_mentions_base_and_head() {
    tokmd()
        .args(["cockpit", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--base"))
        .stdout(predicate::str::contains("--head"));
}

#[cfg(not(feature = "git"))]
#[test]
fn cockpit_without_git_feature_fails_with_message() {
    tokmd()
        .current_dir(fixture())
        .args(["cockpit"])
        .assert()
        .failure();
}
