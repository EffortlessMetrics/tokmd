use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .expect("workspace parent")
        .to_path_buf()
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
fn ci_actuals_help_mentions_receipt_inputs() {
    let (stdout, stderr, success) = run_xtask(&["ci-actuals", "--help"]);

    assert!(success, "ci-actuals --help failed. stderr: {stderr}");
    assert!(stdout.contains("--needs"), "stdout: {stdout}");
    assert!(stdout.contains("--timings"), "stdout: {stdout}");
    assert!(stdout.contains("--output"), "stdout: {stdout}");
}

#[test]
fn ci_actuals_writes_schema_stable_receipt() {
    let temp = tempfile::tempdir().expect("tempdir");
    let needs = temp.path().join("needs.json");
    let timings = temp.path().join("timings.json");
    let output = temp.path().join("ci-actuals.json");
    fs::write(
        &needs,
        r#"{
          "docs-check": {"result": "success", "outputs": {"docs": "ok"}},
          "mutation": {"result": "skipped", "outputs": {}}
        }"#,
    )
    .expect("needs json");
    fs::write(
        &timings,
        r#"{
          "docs-check": {"duration_seconds": 75.0, "runner": "ubuntu-latest", "cache_hit": true}
        }"#,
    )
    .expect("timings json");

    let (stdout, stderr, success) = run_xtask(&[
        "ci-actuals",
        "--needs",
        needs.to_str().expect("needs path"),
        "--timings",
        timings.to_str().expect("timings path"),
        "--output",
        output.to_str().expect("output path"),
        "--sha",
        "abc123",
    ]);

    assert!(success, "ci-actuals failed. stderr: {stderr}");
    assert!(
        stdout.contains("CI actuals receipt written"),
        "stdout: {stdout}"
    );
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).expect("receipt body"))
            .expect("receipt json");
    assert_eq!(value["schema"], "tokmd.ci_actuals.v1");
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["sha"], "abc123");
    assert_eq!(value["status"]["job_count"], 2);
    assert_eq!(value["status"]["timed_job_count"], 1);
    assert_eq!(value["status"]["missing_timing"][0], "mutation");
    assert_eq!(value["jobs"][0]["name"], "docs-check");
    assert_eq!(value["jobs"][0]["timing_status"], "measured");
    assert_eq!(value["jobs"][1]["name"], "mutation");
    assert_eq!(value["jobs"][1]["timing_status"], "missing");
}
