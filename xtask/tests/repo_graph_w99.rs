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
fn repo_graph_help_mentions_refs_expectation_and_json() {
    let (stdout, stderr, success) = run_xtask(&["repo-graph", "--help"]);

    assert!(success, "repo-graph --help failed. stderr: {stderr}");
    assert!(stdout.contains("--publication"), "stdout: {stdout}");
    assert!(stdout.contains("--swarm"), "stdout: {stdout}");
    assert!(stdout.contains("--expect"), "stdout: {stdout}");
    assert!(stdout.contains("--json"), "stdout: {stdout}");
}

#[test]
fn repo_graph_head_to_head_is_aligned() {
    let root = workspace_root();
    let path = root
        .join("target")
        .join("repo-graph-w99")
        .join("aligned.json");
    if path.exists() {
        std::fs::remove_file(&path).expect("stale repo-graph fixture should be removable");
    }

    let path_arg = path.to_string_lossy().to_string();
    let (stdout, stderr, success) = run_xtask(&[
        "repo-graph",
        "--publication",
        "HEAD",
        "--swarm",
        "HEAD",
        "--json",
        &path_arg,
    ]);

    assert!(success, "repo-graph HEAD HEAD failed. stderr: {stderr}");
    assert!(stdout.contains("Aligned"), "stdout: {stdout}");
    assert!(stdout.contains("ok=true"), "stdout: {stdout}");
    assert!(path.exists(), "repo-graph JSON receipt should be written");

    let value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&path).expect("repo-graph JSON receipt should be readable"),
    )
    .expect("repo-graph JSON receipt should parse");

    assert_eq!(value["schema"], "tokmd.repo_graph.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["relation"], "aligned");
    assert_eq!(value["ahead_behind"]["publication_ahead"], 0);
    assert_eq!(value["ahead_behind"]["swarm_ahead"], 0);
}

#[test]
fn repo_graph_invalid_ref_fails_with_git_error() {
    let (_stdout, stderr, success) = run_xtask(&[
        "repo-graph",
        "--publication",
        "definitely-not-a-real-ref",
        "--swarm",
        "HEAD",
    ]);

    assert!(!success, "invalid publication ref should fail");
    assert!(
        stderr.contains("git rev-parse") || stderr.contains("unknown revision"),
        "stderr: {stderr}"
    );
}
