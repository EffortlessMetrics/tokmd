//! Deep round-2 context and handoff CLI integration tests (W51).

mod common;

use assert_cmd::Command;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

// ---------------------------------------------------------------------------
// context --mode list
// ---------------------------------------------------------------------------

#[test]
fn context_list_mode_succeeds() {
    tokmd_cmd()
        .args(["context", "--mode", "list", "--budget", "1000"])
        .assert()
        .success();
}

#[test]
fn context_list_mode_contains_file_paths() {
    let output = tokmd_cmd()
        .args(["context", "--mode", "list", "--budget", "50000"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // list mode should contain at least one path with a file extension
    assert!(
        stdout.contains('.'),
        "list output should contain file paths: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// context --mode json
// ---------------------------------------------------------------------------

#[test]
fn context_json_mode_produces_valid_json() {
    let output = tokmd_cmd()
        .args(["context", "--mode", "json", "--budget", "50000"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("invalid JSON: {e}\nstdout: {stdout}"));
    assert!(parsed.is_object());
}

#[test]
fn context_json_has_schema_version() {
    let output = tokmd_cmd()
        .args(["context", "--mode", "json", "--budget", "50000"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(
        parsed["schema_version"].as_u64(),
        Some(4),
        "context schema_version should be 4"
    );
}

#[test]
fn context_json_has_required_fields() {
    let output = tokmd_cmd()
        .args(["context", "--mode", "json", "--budget", "50000"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(parsed["budget_tokens"].is_number());
    assert!(parsed["used_tokens"].is_number());
    assert!(parsed["files"].is_array());
    assert!(parsed["mode"].as_str() == Some("context"));
}

// ---------------------------------------------------------------------------
// context --budget
// ---------------------------------------------------------------------------

#[test]
fn context_budget_limits_tokens() {
    let output = tokmd_cmd()
        .args(["context", "--mode", "json", "--budget", "500"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let used = parsed["used_tokens"].as_u64().unwrap_or(0);
    let budget = parsed["budget_tokens"].as_u64().unwrap_or(0);
    assert!(
        used <= budget,
        "used_tokens ({used}) should not exceed budget_tokens ({budget})"
    );
}

#[test]
fn context_budget_k_suffix_works() {
    let output = tokmd_cmd()
        .args(["context", "--mode", "json", "--budget", "2k"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(parsed["budget_tokens"].as_u64(), Some(2000));
}

// ---------------------------------------------------------------------------
// context with filters
// ---------------------------------------------------------------------------

#[test]
fn context_exclude_filter_reduces_output() {
    let with_all = tokmd_cmd()
        .args(["context", "--mode", "json", "--budget", "50000"])
        .output()
        .expect("failed to run");
    let with_exclude = tokmd_cmd()
        .args([
            "context",
            "--mode",
            "json",
            "--budget",
            "50000",
            "--exclude",
            "*.md",
        ])
        .output()
        .expect("failed to run");

    assert!(with_all.status.success());
    assert!(with_exclude.status.success());

    let all_json: serde_json::Value = serde_json::from_slice(&with_all.stdout).unwrap();
    let excl_json: serde_json::Value = serde_json::from_slice(&with_exclude.stdout).unwrap();

    let all_count = all_json["file_count"].as_u64().unwrap_or(0);
    let excl_count = excl_json["file_count"].as_u64().unwrap_or(0);
    assert!(
        excl_count <= all_count,
        "excluding *.md should not increase file_count ({excl_count} vs {all_count})"
    );
}

// ---------------------------------------------------------------------------
// handoff
// ---------------------------------------------------------------------------

#[test]
fn handoff_minimal_preset_succeeds() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    tokmd_cmd()
        .args([
            "handoff",
            "--preset",
            "minimal",
            "--budget",
            "5000",
            "--out-dir",
        ])
        .arg(tmp.path().join("handoff"))
        .assert()
        .success();
}

#[test]
fn handoff_creates_manifest_json() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let out_dir = tmp.path().join("handoff");
    tokmd_cmd()
        .args([
            "handoff",
            "--preset",
            "minimal",
            "--budget",
            "5000",
            "--out-dir",
        ])
        .arg(&out_dir)
        .assert()
        .success();

    let manifest_path = out_dir.join("manifest.json");
    assert!(manifest_path.exists(), "manifest.json should exist");

    let manifest_str = std::fs::read_to_string(&manifest_path).unwrap();
    let manifest: serde_json::Value = serde_json::from_str(&manifest_str).unwrap();
    assert_eq!(
        manifest["schema_version"].as_u64(),
        Some(5),
        "handoff schema_version should be 5"
    );
}

#[test]
fn handoff_manifest_has_artifacts_list() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let out_dir = tmp.path().join("handoff");
    tokmd_cmd()
        .args([
            "handoff",
            "--preset",
            "minimal",
            "--budget",
            "5000",
            "--out-dir",
        ])
        .arg(&out_dir)
        .assert()
        .success();

    let manifest_str = std::fs::read_to_string(out_dir.join("manifest.json")).unwrap();
    let manifest: serde_json::Value = serde_json::from_str(&manifest_str).unwrap();
    assert!(manifest["artifacts"].is_array());
    assert!(
        !manifest["artifacts"].as_array().unwrap().is_empty(),
        "handoff should produce at least one artifact"
    );
}

#[test]
fn handoff_manifest_mode_is_handoff() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let out_dir = tmp.path().join("handoff");
    tokmd_cmd()
        .args([
            "handoff",
            "--preset",
            "minimal",
            "--budget",
            "5000",
            "--out-dir",
        ])
        .arg(&out_dir)
        .assert()
        .success();

    let manifest_str = std::fs::read_to_string(out_dir.join("manifest.json")).unwrap();
    let manifest: serde_json::Value = serde_json::from_str(&manifest_str).unwrap();
    assert_eq!(manifest["mode"].as_str(), Some("handoff"));
}

#[test]
fn handoff_standard_preset_succeeds() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    tokmd_cmd()
        .args([
            "handoff",
            "--preset",
            "standard",
            "--budget",
            "10000",
            "--out-dir",
        ])
        .arg(tmp.path().join("handoff"))
        .assert()
        .success();
}

#[test]
fn handoff_no_git_flag_accepted() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    tokmd_cmd()
        .args([
            "handoff",
            "--preset",
            "minimal",
            "--budget",
            "5000",
            "--no-git",
            "--out-dir",
        ])
        .arg(tmp.path().join("handoff"))
        .assert()
        .success();
}
