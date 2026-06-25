use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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
fn ci_gate_contract_help_documents_workflow_flag() {
    let (stdout, stderr, success) = run_xtask(&["ci-gate-contract", "--help"]);
    assert!(success, "ci-gate-contract --help failed: {stderr}");
    assert!(stdout.contains("--workflow"), "stdout: {stdout}");
    assert!(stdout.contains("--check"), "stdout: {stdout}");
}

#[test]
fn reference_fixture_satisfies_ub_review_gate_contract() {
    let (_, stderr, success) = run_xtask(&[
        "ci-gate-contract",
        "--check",
        "--workflow",
        "fixtures/ci-gate-contract/reference-ci.yml",
    ]);
    assert!(
        success,
        "reference fixture must satisfy contract. stderr: {stderr}"
    );
}

#[test]
fn live_ci_yml_satisfies_ub_review_gate_contract() {
    let (_, stderr, success) = run_xtask(&[
        "ci-gate-contract",
        "--check",
        "--workflow",
        ".github/workflows/ci.yml",
    ]);
    assert!(
        success,
        "live ci.yml must satisfy contract after phase 2. stderr: {stderr}"
    );
}

#[test]
fn checker_rejects_retired_multi_lane_marker() {
    let dir = tempfile::tempdir().expect("tempdir");
    let workflow = dir.path().join("bad-ci.yml");
    let mut text =
        fs::read_to_string(workspace_root().join("fixtures/ci-gate-contract/reference-ci.yml"))
            .expect("read reference fixture");
    text.push_str("\n# retired\nname: CI (Required)\n");
    fs::write(&workflow, text).expect("write bad workflow");

    let workflow_arg = workflow.to_string_lossy();
    let (_, stderr, success) =
        run_xtask(&["ci-gate-contract", "--check", "--workflow", &workflow_arg]);
    assert!(
        !success,
        "retired aggregator marker must fail contract. stderr: {stderr}"
    );
}
