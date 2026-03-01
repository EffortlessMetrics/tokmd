//! End-to-end tests for the `tokmd sensor` command.
//!
//! All tests use `--base HEAD --head HEAD` to avoid slow full-repo diffs.

#![cfg(feature = "git")]
mod common;

use assert_cmd::Command;
use tempfile::tempdir;

/// Build a `tokmd` command pointed at the given directory.
fn tokmd_at(dir: &std::path::Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir);
    cmd
}

/// Skip guard: returns `true` when git is available and repo init succeeds.
fn setup_git_repo(dir: &std::path::Path) -> bool {
    if !common::git_available() {
        eprintln!("Skipping: git not available");
        return false;
    }
    if !common::init_git_repo(dir) {
        eprintln!("Skipping: git init failed");
        return false;
    }
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/lib.rs"), "fn main() {}\n").unwrap();
    if !common::git_add_commit(dir, "Initial commit") {
        eprintln!("Skipping: git commit failed");
        return false;
    }
    true
}

// ---------------------------------------------------------------------------
// 1. Basic success: `tokmd sensor --base HEAD --head HEAD` succeeds
// ---------------------------------------------------------------------------

#[test]
fn sensor_base_head_succeeds() {
    let dir = tempdir().unwrap();
    if !setup_git_repo(dir.path()) {
        return;
    }

    tokmd_at(dir.path())
        .args([
            "sensor", "--base", "HEAD", "--head", "HEAD", "--format", "json",
        ])
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// 2. JSON output is valid JSON with expected envelope fields
// ---------------------------------------------------------------------------

#[test]
fn sensor_json_has_envelope_fields() {
    let dir = tempdir().unwrap();
    if !setup_git_repo(dir.path()) {
        return;
    }

    let output = tokmd_at(dir.path())
        .args([
            "sensor", "--base", "HEAD", "--head", "HEAD", "--format", "json",
        ])
        .output()
        .expect("run tokmd sensor");

    assert!(output.status.success(), "sensor exited with error");

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("stdout must be valid JSON");

    assert_eq!(json["schema"], "sensor.report.v1");
    assert!(json.get("tool").is_some(), "missing 'tool' field");
    assert!(json.get("verdict").is_some(), "missing 'verdict' field");
    assert!(json.get("summary").is_some(), "missing 'summary' field");
    assert!(
        json.get("generated_at").is_some(),
        "missing 'generated_at' field"
    );
    assert!(json.get("findings").is_some(), "missing 'findings' field");
    assert!(json.get("data").is_some(), "missing 'data' field");
    assert!(json.get("artifacts").is_some(), "missing 'artifacts' field");
}

// ---------------------------------------------------------------------------
// 3. Sensor report contains schema_version (via the "schema" field)
// ---------------------------------------------------------------------------

#[test]
fn sensor_report_contains_schema_version() {
    let dir = tempdir().unwrap();
    if !setup_git_repo(dir.path()) {
        return;
    }

    let output = tokmd_at(dir.path())
        .args([
            "sensor", "--base", "HEAD", "--head", "HEAD", "--format", "json",
        ])
        .output()
        .expect("run tokmd sensor");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    let schema = json["schema"].as_str().expect("schema must be a string");
    assert_eq!(
        schema, "sensor.report.v1",
        "schema must be sensor.report.v1"
    );
}

// ---------------------------------------------------------------------------
// 4. Report has sensor metadata (tool name, version)
// ---------------------------------------------------------------------------

#[test]
fn sensor_report_has_tool_metadata() {
    let dir = tempdir().unwrap();
    if !setup_git_repo(dir.path()) {
        return;
    }

    let output = tokmd_at(dir.path())
        .args([
            "sensor", "--base", "HEAD", "--head", "HEAD", "--format", "json",
        ])
        .output()
        .expect("run tokmd sensor");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    let tool = &json["tool"];
    assert_eq!(tool["name"], "tokmd", "tool name must be 'tokmd'");
    assert!(
        tool.get("version").and_then(|v| v.as_str()).is_some(),
        "tool must have a version string"
    );
    assert_eq!(tool["mode"], "sensor", "tool mode must be 'sensor'");
}

// ---------------------------------------------------------------------------
// 5. Report on temp dir with sample files — verify artifacts on disk
// ---------------------------------------------------------------------------

#[test]
fn sensor_writes_artifacts_to_disk() {
    let dir = tempdir().unwrap();
    if !setup_git_repo(dir.path()) {
        return;
    }

    let output_path = dir
        .path()
        .join("artifacts")
        .join("tokmd")
        .join("report.json");

    let output = tokmd_at(dir.path())
        .args([
            "sensor", "--base", "HEAD", "--head", "HEAD", "--format", "json", "--output",
        ])
        .arg(&output_path)
        .output()
        .expect("run tokmd sensor");

    assert!(output.status.success(), "sensor exited with error");

    // Report file must exist
    assert!(output_path.exists(), "report.json must be written");

    // Sidecar cockpit receipt must exist
    let extras_dir = output_path.parent().unwrap().join("extras");
    let cockpit_path = extras_dir.join("cockpit_receipt.json");
    assert!(
        cockpit_path.exists(),
        "extras/cockpit_receipt.json must exist"
    );

    // Comment markdown must exist
    let comment_path = output_path.parent().unwrap().join("comment.md");
    assert!(comment_path.exists(), "comment.md must exist");

    // Verify on-disk report is valid JSON
    let disk_content = std::fs::read_to_string(&output_path).unwrap();
    let disk_json: serde_json::Value =
        serde_json::from_str(&disk_content).expect("on-disk report must be valid JSON");
    assert_eq!(disk_json["schema"], "sensor.report.v1");

    // Cockpit sidecar must be valid JSON with schema_version
    let cockpit_content = std::fs::read_to_string(&cockpit_path).unwrap();
    let cockpit_json: serde_json::Value =
        serde_json::from_str(&cockpit_content).expect("cockpit sidecar must be valid JSON");
    assert!(
        cockpit_json.get("schema_version").is_some(),
        "cockpit sidecar must have schema_version"
    );
}

// ---------------------------------------------------------------------------
// 6. Invalid base ref fails gracefully
// ---------------------------------------------------------------------------

#[test]
fn sensor_invalid_base_ref_fails_gracefully() {
    let dir = tempdir().unwrap();
    if !setup_git_repo(dir.path()) {
        return;
    }

    let output = tokmd_at(dir.path())
        .args([
            "sensor",
            "--base",
            "nonexistent-ref-abc123",
            "--head",
            "HEAD",
            "--format",
            "json",
        ])
        .output()
        .expect("run tokmd sensor");

    assert!(
        !output.status.success(),
        "sensor must fail for invalid base ref"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found") || stderr.contains("not a valid") || stderr.contains("error"),
        "stderr should explain the failure, got: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// 7. Markdown format outputs expected header
// ---------------------------------------------------------------------------

#[test]
fn sensor_md_format_outputs_header() {
    let dir = tempdir().unwrap();
    if !setup_git_repo(dir.path()) {
        return;
    }

    let output = tokmd_at(dir.path())
        .args([
            "sensor", "--base", "HEAD", "--head", "HEAD", "--format", "md",
        ])
        .output()
        .expect("run tokmd sensor");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(
        stdout.contains("## Sensor Report: tokmd"),
        "markdown output must contain sensor report header"
    );
    assert!(
        stdout.contains("**Verdict**"),
        "markdown output must contain verdict"
    );
}

// ---------------------------------------------------------------------------
// 8. JSON data section contains gates and summary_metrics
// ---------------------------------------------------------------------------

#[test]
fn sensor_json_data_has_gates_and_metrics() {
    let dir = tempdir().unwrap();
    if !setup_git_repo(dir.path()) {
        return;
    }

    let output = tokmd_at(dir.path())
        .args([
            "sensor", "--base", "HEAD", "--head", "HEAD", "--format", "json",
        ])
        .output()
        .expect("run tokmd sensor");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    let data = &json["data"];
    assert!(data.get("gates").is_some(), "data must contain 'gates'");
    assert!(
        data.get("summary_metrics").is_some(),
        "data must contain 'summary_metrics'"
    );

    // Verify gates structure
    let gates = &data["gates"];
    assert!(gates.get("status").is_some(), "gates must have 'status'");
    assert!(
        gates.get("items").and_then(|v| v.as_array()).is_some(),
        "gates must have 'items' array"
    );

    // Verify summary_metrics has expected keys
    let metrics = &data["summary_metrics"];
    for key in [
        "files_changed",
        "insertions",
        "deletions",
        "health_score",
        "risk_level",
        "risk_score",
    ] {
        assert!(
            metrics.get(key).is_some(),
            "summary_metrics must contain '{key}'"
        );
    }
}
