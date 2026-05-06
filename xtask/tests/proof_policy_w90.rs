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
fn proof_policy_check_accepts_repo_policy() {
    let (stdout, stderr, success) = run_xtask(&["proof-policy", "--check"]);

    assert!(success, "proof-policy --check failed. stderr: {stderr}");
    assert!(stdout.contains("Proof policy OK"), "stdout: {stdout}");
    assert!(stdout.contains("ci/proof.toml"), "stdout: {stdout}");
}

#[test]
fn proof_policy_json_reports_current_schema() {
    let (stdout, stderr, success) = run_xtask(&["proof-policy", "--json"]);

    assert!(success, "proof-policy --json failed. stderr: {stderr}");
    let value: serde_json::Value =
        serde_json::from_str(&stdout).expect("proof-policy --json should emit JSON");

    assert_eq!(value["ok"], true);
    assert_eq!(value["schema"], "tokmd.proof_policy.v1");
    assert_eq!(value["scope_count"], 2);
    assert_eq!(value["allowlist_count"], 1);
    assert_eq!(value["fixture_blob_rule_count"], 1);
    assert_eq!(value["dependency_boundary_count"], 1);
}

#[test]
fn xtask_help_mentions_proof_policy() {
    let (stdout, stderr, success) = run_xtask(&["--help"]);

    assert!(success, "xtask --help failed. stderr: {stderr}");
    assert!(stdout.contains("proof-policy"), "stdout: {stdout}");
}
