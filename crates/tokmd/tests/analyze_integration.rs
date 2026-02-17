mod common;

use assert_cmd::Command;
use serde_json::Value;
use tempfile::tempdir;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    // Point to hermetic copy of test fixtures with .git/ marker
    cmd.current_dir(common::fixture_root());
    cmd
}

#[test]
fn analyze_receipt_preset_json_smoke() {
    let output = tokmd_cmd()
        .arg("analyze")
        .arg(".")
        .arg("--preset")
        .arg("receipt")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to execute tokmd analyze");

    assert!(
        output.status.success(),
        "tokmd analyze failed: {:?}",
        output.status
    );

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("invalid JSON output");

    assert_eq!(json["mode"], "analysis");
    assert_eq!(json["schema_version"], 5);
    assert!(json["generated_at_ms"].is_number());

    // A couple of stable "shape" checks
    assert!(json.get("source").is_some());
    assert!(json.get("args").is_some());
}

#[test]
fn analyze_writes_json_to_output_dir() {
    let dir = tempdir().unwrap();
    let out = dir.path();

    let output = tokmd_cmd()
        .arg("analyze")
        .arg(".")
        .arg("--preset")
        .arg("receipt")
        .arg("--format")
        .arg("json")
        .arg("--output-dir")
        .arg(out)
        .output()
        .expect("failed to execute tokmd analyze");

    assert!(
        output.status.success(),
        "tokmd analyze failed: {:?}",
        output.status
    );

    let path = out.join("analysis.json");
    assert!(path.exists(), "expected analysis.json at {:?}", path);

    let content = std::fs::read_to_string(&path).expect("failed to read analysis.json");
    let json: Value = serde_json::from_str(&content).expect("analysis.json is not valid JSON");
    assert_eq!(json["mode"], "analysis");
}

#[test]
fn analyze_explain_known_metric() {
    let output = tokmd_cmd()
        .arg("analyze")
        .arg("--explain")
        .arg("avg_cyclomatic")
        .output()
        .expect("failed to execute tokmd analyze --explain");

    assert!(
        output.status.success(),
        "tokmd analyze --explain failed: {:?}",
        output.status
    );

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    assert!(stdout.contains("avg_cyclomatic"));
    assert!(stdout.contains("complexity"));
}

#[test]
fn analyze_explain_list() {
    let output = tokmd_cmd()
        .arg("analyze")
        .arg("--explain")
        .arg("list")
        .output()
        .expect("failed to execute tokmd analyze --explain list");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    assert!(stdout.contains("Available metric/finding keys:"));
    assert!(stdout.contains("maintainability_index"));
}

#[test]
fn analyze_explain_unknown_metric_fails() {
    let output = tokmd_cmd()
        .arg("analyze")
        .arg("--explain")
        .arg("not_a_metric")
        .output()
        .expect("failed to execute tokmd analyze --explain unknown");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("invalid UTF-8");
    assert!(stderr.contains("Unknown metric/finding key"));
    assert!(stderr.contains("--explain list"));
}
