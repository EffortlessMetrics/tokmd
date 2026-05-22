use std::path::PathBuf;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use tempfile::TempDir;

fn xtask_run_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().unwrap().to_path_buf()
}

fn xtask_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_xtask"))
}

fn run_xtask(args: &[&str]) -> (String, String, bool) {
    let _guard = xtask_run_lock()
        .lock()
        .expect("xtask run lock should not be poisoned");
    let root = workspace_root();
    let output = Command::new(xtask_bin())
        .args(args)
        .current_dir(&root)
        .output()
        .expect("failed to run xtask");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (stdout, stderr, output.status.success())
}

fn run_xtask_in_dir(args: &[&str], current_dir: &std::path::Path) -> (String, String, bool) {
    let _guard = xtask_run_lock()
        .lock()
        .expect("xtask run lock should not be poisoned");
    let output = Command::new(xtask_bin())
        .args(args)
        .current_dir(current_dir)
        .output()
        .expect("failed to run xtask");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (stdout, stderr, output.status.success())
}

fn git(repo: &std::path::Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .unwrap_or_else(|_| panic!("failed to run git {}", args.join(" ")));

    assert!(
        output.status.success(),
        "git {} failed\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn write_file(repo: &std::path::Path, path: &str, body: &str) {
    std::fs::write(repo.join(path), body).expect("fixture file should be writable");
}

fn commit(repo: &std::path::Path, message: &str) {
    git(repo, &["add", "."]);
    git(repo, &["commit", "-m", message]);
}

fn init_repo() -> TempDir {
    let temp = TempDir::new().expect("temporary git repo should be creatable");
    let repo = temp.path();
    git(repo, &["init", "-b", "main"]);
    git(
        repo,
        &["config", "user.email", "repo-graph@example.invalid"],
    );
    git(repo, &["config", "user.name", "Repo Graph Test"]);
    write_file(repo, "file.txt", "base\n");
    commit(repo, "base");
    git(repo, &["branch", "publication"]);
    git(repo, &["branch", "swarm"]);
    temp
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
    assert_eq!(
        value["next_action"],
        "graph is aligned; no publication or swarm fast-forward action is needed"
    );
    assert_eq!(value["ahead_behind"]["publication_ahead"], 0);
    assert_eq!(value["ahead_behind"]["swarm_ahead"], 0);
}

#[test]
fn repo_graph_reports_swarm_ahead_in_real_git_repo() {
    let temp = init_repo();
    let repo = temp.path();

    git(repo, &["switch", "swarm"]);
    write_file(repo, "file.txt", "base\nswarm\n");
    commit(repo, "swarm");

    let (stdout, stderr, success) = run_xtask_in_dir(
        &[
            "repo-graph",
            "--publication",
            "publication",
            "--swarm",
            "swarm",
            "--expect",
            "swarm-descends-publication",
        ],
        repo,
    );

    assert!(success, "repo-graph swarm-ahead failed. stderr: {stderr}");
    assert!(stdout.contains("SwarmAhead"), "stdout: {stdout}");
    assert!(stdout.contains("publication_ahead=0"), "stdout: {stdout}");
    assert!(stdout.contains("swarm_ahead=1"), "stdout: {stdout}");
}

#[test]
fn repo_graph_reports_publication_ahead_in_real_git_repo() {
    let temp = init_repo();
    let repo = temp.path();

    git(repo, &["switch", "publication"]);
    write_file(repo, "file.txt", "base\npublication\n");
    commit(repo, "publication");

    let (stdout, stderr, success) = run_xtask_in_dir(
        &[
            "repo-graph",
            "--publication",
            "publication",
            "--swarm",
            "swarm",
            "--expect",
            "publication-descends-swarm",
        ],
        repo,
    );

    assert!(
        success,
        "repo-graph publication-ahead failed. stderr: {stderr}"
    );
    assert!(stdout.contains("PublicationAhead"), "stdout: {stdout}");
    assert!(stdout.contains("publication_ahead=1"), "stdout: {stdout}");
    assert!(stdout.contains("swarm_ahead=0"), "stdout: {stdout}");
}

#[test]
fn repo_graph_no_divergence_accepts_swarm_ahead_in_real_git_repo() {
    let temp = init_repo();
    let repo = temp.path();

    git(repo, &["switch", "swarm"]);
    write_file(repo, "file.txt", "base\nswarm\n");
    commit(repo, "swarm");

    let (stdout, stderr, success) = run_xtask_in_dir(
        &[
            "repo-graph",
            "--publication",
            "publication",
            "--swarm",
            "swarm",
            "--expect",
            "no-divergence",
        ],
        repo,
    );

    assert!(
        success,
        "repo-graph no-divergence swarm-ahead failed. stderr: {stderr}"
    );
    assert!(stdout.contains("SwarmAhead"), "stdout: {stdout}");
    assert!(stdout.contains("ok=true"), "stdout: {stdout}");
    assert!(
        stdout.contains("open a tokmd publication PR"),
        "stdout: {stdout}"
    );
}

#[test]
fn repo_graph_no_divergence_accepts_publication_ahead_in_real_git_repo() {
    let temp = init_repo();
    let repo = temp.path();

    git(repo, &["switch", "publication"]);
    write_file(repo, "file.txt", "base\npublication\n");
    commit(repo, "publication");

    let (stdout, stderr, success) = run_xtask_in_dir(
        &[
            "repo-graph",
            "--publication",
            "publication",
            "--swarm",
            "swarm",
            "--expect",
            "no-divergence",
        ],
        repo,
    );

    assert!(
        success,
        "repo-graph no-divergence publication-ahead failed. stderr: {stderr}"
    );
    assert!(stdout.contains("PublicationAhead"), "stdout: {stdout}");
    assert!(stdout.contains("ok=true"), "stdout: {stdout}");
    assert!(
        stdout.contains("fast-forward tokmd-swarm/main"),
        "stdout: {stdout}"
    );
}

#[test]
fn repo_graph_rejects_swarm_ahead_when_publication_must_descend_from_swarm() {
    let temp = init_repo();
    let repo = temp.path();

    git(repo, &["switch", "swarm"]);
    write_file(repo, "file.txt", "base\nswarm\n");
    commit(repo, "swarm");

    let (stdout, stderr, success) = run_xtask_in_dir(
        &[
            "repo-graph",
            "--publication",
            "publication",
            "--swarm",
            "swarm",
            "--expect",
            "publication-descends-swarm",
        ],
        repo,
    );

    assert!(
        !success,
        "swarm-ahead refs should fail publication-descends-swarm"
    );
    assert!(stdout.contains("SwarmAhead"), "stdout: {stdout}");
    assert!(
        stderr.contains("repo graph expectation publication-descends-swarm was not met"),
        "stderr: {stderr}"
    );
    assert!(
        stderr.contains("publication has not imported the swarm head"),
        "stderr: {stderr}"
    );
    assert!(stderr.contains("merge commit"), "stderr: {stderr}");
}

#[test]
fn repo_graph_rejects_diverged_refs_in_real_git_repo() {
    let temp = init_repo();
    let repo = temp.path();

    git(repo, &["switch", "publication"]);
    write_file(repo, "publication.txt", "publication\n");
    commit(repo, "publication");
    git(repo, &["switch", "swarm"]);
    write_file(repo, "swarm.txt", "swarm\n");
    commit(repo, "swarm");

    let (stdout, stderr, success) = run_xtask_in_dir(
        &[
            "repo-graph",
            "--publication",
            "publication",
            "--swarm",
            "swarm",
            "--expect",
            "no-divergence",
        ],
        repo,
    );

    assert!(!success, "diverged refs should fail");
    assert!(stdout.contains("Diverged"), "stdout: {stdout}");
    assert!(
        stderr.contains("repo graph expectation no-divergence was not met"),
        "stderr: {stderr}"
    );
    assert!(stderr.contains("explicit sync merge"), "stderr: {stderr}");
}

#[test]
fn repo_graph_rejects_unrelated_refs_in_real_git_repo() {
    let temp = init_repo();
    let repo = temp.path();

    git(repo, &["checkout", "--orphan", "orphan"]);
    git(repo, &["rm", "-rf", "."]);
    write_file(repo, "orphan.txt", "orphan\n");
    commit(repo, "orphan");

    let (stdout, stderr, success) = run_xtask_in_dir(
        &[
            "repo-graph",
            "--publication",
            "publication",
            "--swarm",
            "orphan",
            "--expect",
            "no-divergence",
        ],
        repo,
    );

    assert!(!success, "unrelated refs should fail");
    assert!(stdout.contains("Unrelated"), "stdout: {stdout}");
    assert!(
        stderr.contains("repo graph expectation no-divergence was not met"),
        "stderr: {stderr}"
    );
    assert!(stderr.contains("admin realignment"), "stderr: {stderr}");
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
