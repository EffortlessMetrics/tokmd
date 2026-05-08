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
fn lint_policy_accepts_repo_policy() {
    let (stdout, stderr, success) = run_xtask(&["check-lint-policy"]);

    assert!(success, "check-lint-policy failed. stderr: {stderr}");
    assert!(stdout.contains("lint policy ok"), "stdout: {stdout}");
    assert!(stdout.contains("MSRV 1.95"), "stdout: {stdout}");
}

#[test]
fn xtask_help_mentions_lint_policy_gate() {
    let (stdout, stderr, success) = run_xtask(&["--help"]);

    assert!(success, "xtask --help failed. stderr: {stderr}");
    assert!(stdout.contains("check-lint-policy"), "stdout: {stdout}");
}
