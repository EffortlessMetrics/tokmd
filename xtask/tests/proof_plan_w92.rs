use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().unwrap().to_path_buf()
}

fn run_xtask(args: &[&str]) -> (String, String, bool) {
    run_xtask_with_env(args, &[])
}

fn run_xtask_with_env(args: &[&str], envs: &[(&str, &str)]) -> (String, String, bool) {
    let root = workspace_root();
    let mut command = Command::new("cargo");
    command
        .arg("run")
        .arg("-q")
        .arg("-p")
        .arg("xtask")
        .arg("--")
        .args(args)
        .current_dir(&root)
        .env_remove("CI");
    for (key, value) in envs {
        command.env(key, value);
    }

    let output = command.output().expect("failed to run cargo xtask");
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
    assert!(stdout.contains("--executor-summary"), "stdout: {stdout}");
    assert!(stdout.contains("--executor-mode"), "stdout: {stdout}");
    assert!(
        stdout.contains("--allow-ci-evidence-execution"),
        "stdout: {stdout}"
    );
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
    assert!(!summary.contains("### Executor Guard"));
}

#[test]
fn proof_plan_markdown_summary_includes_executor_guard_when_requested() {
    let temp = tempfile::tempdir().expect("tempdir");
    let summary_path = temp.path().join("proof-plan.md");
    let summary_arg = summary_path.to_string_lossy().to_string();
    let executor_path = temp.path().join("executor-summary.json");
    let executor_arg = executor_path.to_string_lossy().to_string();
    let (_stdout, stderr, success) = run_xtask_with_env(
        &[
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
            "--executor-summary",
            &executor_arg,
            "--executor-mode",
            "dry-run",
        ],
        &[("CI", "true")],
    );

    assert!(
        success,
        "proof --summary-md with executor summary failed. stderr: {stderr}"
    );
    let summary = fs::read_to_string(summary_path).expect("summary should be written");
    assert!(summary.contains("### Executor Guard"));
    assert!(summary.contains("| Mode | `dry_run` |"));
    assert!(summary.contains("| Guard enabled | `false` |"));
    assert!(summary.contains("| CI | `true` |"));
    assert!(summary.contains("ci_requires_--allow-ci-evidence-execution"));
    assert!(summary.contains("| Executed commands | 0 |"));

    let executor = fs::read_to_string(executor_path).expect("executor summary should be written");
    let executor: serde_json::Value =
        serde_json::from_str(&executor).expect("executor summary should be valid JSON");
    assert_eq!(executor["execution_guard"]["enabled"], false);
    assert_eq!(executor["counts"]["executed"], 0);
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

#[test]
fn proof_plan_writes_executor_summary_artifact() {
    let temp = tempfile::tempdir().expect("tempdir");
    let executor_path = temp.path().join("executor-summary.json");
    let executor_arg = executor_path.to_string_lossy().to_string();
    let (stdout, stderr, success) = run_xtask(&[
        "proof",
        "--profile",
        "affected",
        "--base",
        "HEAD",
        "--head",
        "HEAD",
        "--plan",
        "--executor-summary",
        &executor_arg,
    ]);

    assert!(success, "proof --executor-summary failed. stderr: {stderr}");
    let value: serde_json::Value =
        serde_json::from_str(&stdout).expect("proof --plan should still emit JSON");
    assert_eq!(value["schema"], "tokmd.proof_plan.v1");

    let summary = fs::read_to_string(executor_path).expect("executor summary should be written");
    let summary: serde_json::Value =
        serde_json::from_str(&summary).expect("executor summary should be valid JSON");
    assert_eq!(summary["schema"], "tokmd.proof_executor_summary.v1");
    assert_eq!(summary["mode"], "prototype");
    assert_eq!(summary["status"], "prototype");
    assert_eq!(summary["execution_status"], "not_executed");
    assert_eq!(summary["execution_guard"]["required"], true);
    assert_eq!(summary["execution_guard"]["enabled"], false);
    assert_eq!(summary["execution_guard"]["ci"], false);
    assert_eq!(
        summary["execution_guard"]["allow_ci_evidence_execution"],
        false
    );
    assert_eq!(
        summary["execution_guard"]["reason"],
        "not_ci_and_no_--allow-ci-evidence-execution"
    );
    assert_eq!(summary["family"], "coverage");
    assert_eq!(summary["required"], false);
    assert_eq!(summary["counts"]["selected"], 0);
    assert_eq!(summary["counts"]["dry_run"], 0);
    assert_eq!(summary["counts"]["executed"], 0);
    assert!(summary["entries"].as_array().unwrap().is_empty());
}

#[test]
fn proof_plan_writes_dry_run_executor_summary_artifact() {
    let temp = tempfile::tempdir().expect("tempdir");
    let executor_path = temp.path().join("executor-summary.json");
    let executor_arg = executor_path.to_string_lossy().to_string();
    let (stdout, stderr, success) = run_xtask(&[
        "proof",
        "--profile",
        "deep",
        "--plan",
        "--executor-summary",
        &executor_arg,
        "--executor-mode",
        "dry-run",
    ]);

    assert!(
        success,
        "proof --executor-mode dry-run failed. stderr: {stderr}"
    );
    let value: serde_json::Value =
        serde_json::from_str(&stdout).expect("proof --plan should still emit JSON");
    assert_eq!(value["schema"], "tokmd.proof_plan.v1");
    assert_eq!(value["profile"], "deep");

    let summary = fs::read_to_string(executor_path).expect("executor summary should be written");
    let summary: serde_json::Value =
        serde_json::from_str(&summary).expect("executor summary should be valid JSON");
    assert_eq!(summary["schema"], "tokmd.proof_executor_summary.v1");
    assert_eq!(summary["mode"], "dry_run");
    assert_eq!(summary["status"], "dry_run");
    assert_eq!(summary["execution_status"], "dry_run");
    assert_eq!(summary["execution_guard"]["enabled"], false);
    assert_eq!(
        summary["execution_guard"]["allow_ci_evidence_execution"],
        false
    );
    assert_eq!(summary["family"], "coverage");
    assert_eq!(summary["counts"]["family_planned"], 1);
    assert_eq!(summary["counts"]["selected"], 0);
    assert_eq!(summary["counts"]["required_excluded"], 1);
    assert_eq!(summary["counts"]["executed"], 0);
    assert!(summary["entries"].as_array().unwrap().is_empty());
}

#[test]
fn ci_executor_summary_requires_explicit_evidence_execution_opt_in() {
    let temp = tempfile::tempdir().expect("tempdir");
    let blocked_path = temp.path().join("blocked-executor-summary.json");
    let blocked_arg = blocked_path.to_string_lossy().to_string();
    let (stdout, stderr, success) = run_xtask_with_env(
        &[
            "proof",
            "--profile",
            "affected",
            "--base",
            "HEAD",
            "--head",
            "HEAD",
            "--plan",
            "--executor-summary",
            &blocked_arg,
        ],
        &[("CI", "true")],
    );

    assert!(
        success,
        "CI proof executor summary failed. stderr: {stderr}"
    );
    let value: serde_json::Value =
        serde_json::from_str(&stdout).expect("proof --plan should still emit JSON");
    assert_eq!(value["schema"], "tokmd.proof_plan.v1");

    let blocked = fs::read_to_string(blocked_path).expect("executor summary should be written");
    let blocked: serde_json::Value =
        serde_json::from_str(&blocked).expect("executor summary should be valid JSON");
    assert_eq!(blocked["execution_guard"]["ci"], true);
    assert_eq!(blocked["execution_guard"]["enabled"], false);
    assert_eq!(
        blocked["execution_guard"]["reason"],
        "ci_requires_--allow-ci-evidence-execution"
    );
    assert_eq!(blocked["counts"]["executed"], 0);

    let enabled_path = temp.path().join("enabled-executor-summary.json");
    let enabled_arg = enabled_path.to_string_lossy().to_string();
    let (_stdout, stderr, success) = run_xtask_with_env(
        &[
            "proof",
            "--profile",
            "affected",
            "--base",
            "HEAD",
            "--head",
            "HEAD",
            "--plan",
            "--executor-summary",
            &enabled_arg,
            "--allow-ci-evidence-execution",
        ],
        &[("CI", "true")],
    );

    assert!(
        success,
        "CI proof executor summary with opt-in failed. stderr: {stderr}"
    );
    let enabled = fs::read_to_string(enabled_path).expect("executor summary should be written");
    let enabled: serde_json::Value =
        serde_json::from_str(&enabled).expect("executor summary should be valid JSON");
    assert_eq!(enabled["execution_guard"]["ci"], true);
    assert_eq!(enabled["execution_guard"]["enabled"], true);
    assert_eq!(
        enabled["execution_guard"]["reason"],
        "ci_explicit_opt_in_enabled"
    );
    assert_eq!(enabled["counts"]["executed"], 0);
}
