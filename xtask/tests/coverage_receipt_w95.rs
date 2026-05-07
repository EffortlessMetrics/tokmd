use std::fs;
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
fn coverage_receipt_help_mentions_artifact_paths() {
    let (stdout, stderr, success) = run_xtask(&["coverage-receipt", "--help"]);

    assert!(success, "coverage-receipt --help failed. stderr: {stderr}");
    assert!(stdout.contains("--coverage-json"), "stdout: {stdout}");
    assert!(stdout.contains("--coverage-text"), "stdout: {stdout}");
    assert!(stdout.contains("--lcov"), "stdout: {stdout}");
    assert!(stdout.contains("--output"), "stdout: {stdout}");
}

#[test]
fn coverage_receipt_writes_receipt_for_non_empty_artifacts() {
    let temp = tempfile::tempdir().expect("tempdir");
    let coverage_json = temp.path().join("coverage.json");
    let coverage_text = temp.path().join("coverage.txt");
    let lcov = temp.path().join("lcov.info");
    let output = temp.path().join("coverage-receipt.json");
    fs::write(&coverage_json, "{}\n").expect("coverage json");
    fs::write(&coverage_text, "coverage\n").expect("coverage text");
    fs::write(&lcov, "TN:\nSF:src/lib.rs\nend_of_record\n").expect("lcov");

    let (stdout, stderr, success) = run_xtask(&[
        "coverage-receipt",
        "--coverage-json",
        coverage_json.to_str().expect("coverage json path"),
        "--coverage-text",
        coverage_text.to_str().expect("coverage text path"),
        "--lcov",
        lcov.to_str().expect("lcov path"),
        "--output",
        output.to_str().expect("output path"),
        "--sha",
        "abc123",
    ]);

    assert!(success, "coverage-receipt failed. stderr: {stderr}");
    assert!(
        stdout.contains("coverage receipt written"),
        "stdout: {stdout}"
    );
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).expect("receipt body"))
            .expect("receipt json");
    assert_eq!(value["schema"], "tokmd.coverage_receipt.v1");
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["sha"], "abc123");
    assert_eq!(value["status"]["ok"], true);
    assert_eq!(value["artifacts"].as_array().expect("artifacts").len(), 3);
}
