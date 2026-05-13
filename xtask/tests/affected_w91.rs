use std::path::PathBuf;
use std::process::Command;

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().unwrap().to_path_buf()
}

fn run_xtask(args: &[&str]) -> (String, String, bool) {
    let root = workspace_root();
    let output = Command::new("cargo")
        .arg("run")
        .arg("-q")
        .arg("-p")
        .arg("xtask")
        .arg("--")
        .args(args)
        .current_dir(&root)
        .output()
        .expect("failed to run cargo xtask");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (stdout, stderr, output.status.success())
}

#[test]
fn affected_help_mentions_base_head_and_json() {
    let (stdout, stderr, success) = run_xtask(&["affected", "--help"]);

    assert!(success, "affected --help failed. stderr: {stderr}");
    assert!(stdout.contains("--base"), "stdout: {stdout}");
    assert!(stdout.contains("--head"), "stdout: {stdout}");
    assert!(stdout.contains("--json"), "stdout: {stdout}");
    assert!(stdout.contains("--json-output"), "stdout: {stdout}");
}

#[test]
fn affected_json_reports_no_changes_for_same_ref() {
    let (stdout, stderr, success) =
        run_xtask(&["affected", "--base", "HEAD", "--head", "HEAD", "--json"]);

    assert!(success, "affected --json failed. stderr: {stderr}");
    let value: serde_json::Value =
        serde_json::from_str(&stdout).expect("affected --json should emit JSON");

    assert_eq!(value["schema"], "tokmd.affected.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["base"], "HEAD");
    assert_eq!(value["head"], "HEAD");
    assert!(value["changed_files"].as_array().unwrap().is_empty());
    assert!(value["scopes"].as_array().unwrap().is_empty());
    assert!(value["unknown_files"].as_array().unwrap().is_empty());
}

#[test]
fn affected_json_output_writes_report_artifact() {
    let root = workspace_root();
    let path = root
        .join("target")
        .join("affected-w91")
        .join("affected.json");
    if path.exists() {
        std::fs::remove_file(&path).expect("stale affected fixture should be removable");
    }

    let path_arg = path.to_string_lossy().to_string();
    let (stdout, stderr, success) = run_xtask(&[
        "affected",
        "--base",
        "HEAD",
        "--head",
        "HEAD",
        "--json",
        "--json-output",
        &path_arg,
    ]);

    assert!(success, "affected --json-output failed. stderr: {stderr}");
    assert!(stdout.contains("\"schema\": \"tokmd.affected.v1\""));
    assert!(path.exists(), "affected artifact should be written");

    let written = std::fs::read_to_string(&path).expect("affected artifact should be readable");
    let stdout_json: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout affected report should be JSON");
    let written_json: serde_json::Value =
        serde_json::from_str(&written).expect("written affected report should be JSON");

    assert_eq!(written_json["schema"], "tokmd.affected.v1");
    assert_eq!(written_json, stdout_json);
}

#[test]
fn affected_bad_base_reports_git_error() {
    let (_stdout, stderr, success) = run_xtask(&[
        "affected",
        "--base",
        "definitely-not-a-real-ref",
        "--head",
        "HEAD",
        "--json",
    ]);

    assert!(!success, "affected should fail for an invalid base ref");
    assert!(
        stderr.contains("git diff") || stderr.contains("bad revision"),
        "stderr: {stderr}"
    );
}
