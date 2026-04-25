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
fn fixture_blobs_check_passes_on_clean_repo() {
    let (stdout, stderr, success) = run_xtask(&["fixture-blobs-check"]);
    assert!(success, "fixture-blobs-check failed. stderr: {stderr}");
    assert!(stdout.contains("No committed crypto fixture blobs found"));
}

#[test]
fn xtask_help_mentions_fixture_blobs_check() {
    let (stdout, stderr, success) = run_xtask(&["--help"]);
    assert!(success, "xtask --help failed. stderr: {stderr}");
    assert!(stdout.contains("fixture-blobs-check"));
}

#[test]
fn gate_invokes_fixture_blob_guardrail() {
    let src = std::fs::read_to_string(
        workspace_root()
            .join("xtask")
            .join("src")
            .join("tasks")
            .join("gate.rs"),
    )
    .expect("read gate.rs");

    assert!(src.contains("fixture_blobs_check::run"));
}
