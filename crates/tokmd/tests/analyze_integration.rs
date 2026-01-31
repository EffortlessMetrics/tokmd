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
    assert_eq!(json["schema_version"], 2);
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
