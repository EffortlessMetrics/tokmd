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
fn proof_help_mentions_profile_and_plan() {
    let (stdout, stderr, success) = run_xtask(&["proof", "--help"]);

    assert!(success, "proof --help failed. stderr: {stderr}");
    assert!(stdout.contains("--profile"), "stdout: {stdout}");
    assert!(stdout.contains("--plan"), "stdout: {stdout}");
    assert!(stdout.contains("--summary-md"), "stdout: {stdout}");
    assert!(stdout.contains("--evidence-json"), "stdout: {stdout}");
}

#[test]
fn affected_proof_plan_reports_no_changes_for_same_ref() {
    let (stdout, stderr, success) = run_xtask(&[
        "proof",
        "--profile",
        "affected",
        "--base",
        "HEAD",
        "--head",
        "HEAD",
        "--plan",
    ]);

    assert!(success, "proof --plan failed. stderr: {stderr}");
    let value: serde_json::Value =
        serde_json::from_str(&stdout).expect("proof --plan should emit JSON");

    assert_eq!(value["schema"], "tokmd.proof_plan.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["profile"], "affected");
    assert_eq!(value["base"], "HEAD");
    assert_eq!(value["head"], "HEAD");
    assert!(value["commands"].as_array().unwrap().is_empty());
    assert!(value["unknown_files"].as_array().unwrap().is_empty());
}

#[test]
fn proof_without_plan_refuses_to_execute() {
    let (_stdout, stderr, success) = run_xtask(&["proof", "--profile", "affected"]);

    assert!(!success, "proof without --plan should fail for now");
    assert!(
        stderr.contains("--plan") || stderr.contains("not implemented"),
        "stderr: {stderr}"
    );
}

#[test]
fn fast_proof_plan_includes_policy_and_guardrails() {
    let (stdout, stderr, success) = run_xtask(&["proof", "--profile", "fast", "--plan"]);

    assert!(success, "proof fast --plan failed. stderr: {stderr}");
    let value: serde_json::Value =
        serde_json::from_str(&stdout).expect("proof fast --plan should emit JSON");
    let commands = value["commands"]
        .as_array()
        .expect("commands should be array");

    assert_eq!(value["profile"], "fast");
    assert!(
        commands
            .iter()
            .any(|cmd| cmd["command"] == "cargo xtask proof-policy --check")
    );
    assert!(
        commands
            .iter()
            .any(|cmd| cmd["command"] == "cargo xtask fixture-blobs-check")
    );
    assert!(
        commands
            .iter()
            .any(|cmd| cmd["command"] == "cargo xtask boundaries-check")
    );
}

#[test]
fn proof_plan_writes_markdown_summary_artifact() {
    let temp = tempfile::tempdir().expect("tempdir");
    let summary_path = temp.path().join("proof-plan.md");
    let summary_arg = summary_path.to_string_lossy().to_string();
    let (stdout, stderr, success) = run_xtask(&[
        "proof",
        "--profile",
        "affected",
        "--base",
        "HEAD",
        "--head",
        "HEAD",
        "--plan",
        "--summary-md",
        &summary_arg,
    ]);

    assert!(success, "proof --summary-md failed. stderr: {stderr}");
    let value: serde_json::Value =
        serde_json::from_str(&stdout).expect("proof --plan should still emit JSON");
    assert_eq!(value["schema"], "tokmd.proof_plan.v1");

    let summary = fs::read_to_string(summary_path).expect("summary should be written");
    assert!(summary.contains("## Proof Plan Summary"));
    assert!(summary.contains("No proof commands planned."));
}

#[test]
fn proof_plan_writes_evidence_json_artifact() {
    let temp = tempfile::tempdir().expect("tempdir");
    let evidence_path = temp.path().join("proof-evidence.json");
    let evidence_arg = evidence_path.to_string_lossy().to_string();
    let (stdout, stderr, success) = run_xtask(&[
        "proof",
        "--profile",
        "affected",
        "--base",
        "HEAD",
        "--head",
        "HEAD",
        "--plan",
        "--evidence-json",
        &evidence_arg,
    ]);

    assert!(success, "proof --evidence-json failed. stderr: {stderr}");
    let value: serde_json::Value =
        serde_json::from_str(&stdout).expect("proof --plan should still emit JSON");
    assert_eq!(value["schema"], "tokmd.proof_plan.v1");

    let evidence = fs::read_to_string(evidence_path).expect("evidence should be written");
    let evidence: serde_json::Value =
        serde_json::from_str(&evidence).expect("evidence should be valid JSON");
    assert_eq!(evidence["schema"], "tokmd.proof_evidence_plan.v1");
    assert_eq!(evidence["status"], "planned");
    assert_eq!(evidence["execution_status"], "not_executed");
    assert_eq!(evidence["counts"]["coverage"]["executed"], 0);
    assert_eq!(evidence["counts"]["mutation"]["executed"], 0);
    assert!(evidence["entries"].as_array().unwrap().is_empty());
}
