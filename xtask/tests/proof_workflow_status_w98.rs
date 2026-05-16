use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().unwrap().to_path_buf()
}

fn run_xtask(args: &[&str]) -> (String, String, bool) {
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

    let output = command.output().expect("failed to run cargo xtask");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (stdout, stderr, output.status.success())
}

#[test]
fn proof_workflow_status_help_mentions_fast_inputs() {
    let (stdout, stderr, success) = run_xtask(&["proof-workflow-status", "--help"]);

    assert!(
        success,
        "proof-workflow-status --help failed. stderr: {stderr}"
    );
    assert!(stdout.contains("--workflow-kind"), "stdout: {stdout}");
    assert!(stdout.contains("--status"), "stdout: {stdout}");
    assert!(stdout.contains("--proof-policy"), "stdout: {stdout}");
    assert!(stdout.contains("--proof-plan"), "stdout: {stdout}");
    assert!(stdout.contains("--proof-run-summary"), "stdout: {stdout}");
    assert!(
        stdout.contains("--proof-run-artifacts-check"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("--proof-run-observation"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("--summary-md"), "stdout: {stdout}");
    assert!(stdout.contains("--env-output"), "stdout: {stdout}");
}

#[test]
fn proof_workflow_status_check_help_mentions_receipt_output() {
    let (stdout, stderr, success) = run_xtask(&["proof-workflow-status-check", "--help"]);

    assert!(
        success,
        "proof-workflow-status-check --help failed. stderr: {stderr}"
    );
    assert!(stdout.contains("--status"), "stdout: {stdout}");
    assert!(stdout.contains("--json"), "stdout: {stdout}");
}

#[test]
fn proof_workflow_status_writes_and_checks_fast_packet() {
    let paths = FixturePaths::new("proof-workflow-status-ok");
    paths.reset();
    paths.write_sources();

    let (stdout, stderr, success) = run_xtask(&[
        "proof-workflow-status",
        "--workflow-kind",
        "fast-proof-run",
        "--status",
        "proof_run_status=0",
        "--status",
        "proof_run_artifacts_status=0",
        "--status",
        "proof_run_observation_status=0",
        "--proof-policy",
        paths.proof_policy_arg(),
        "--proof-plan",
        paths.proof_plan_arg(),
        "--proof-run-summary",
        paths.proof_run_summary_arg(),
        "--proof-run-artifacts-check",
        paths.proof_run_artifacts_check_arg(),
        "--proof-run-observation",
        paths.proof_run_observation_arg(),
        "--json",
        paths.status_arg(),
        "--summary-md",
        paths.summary_md_arg(),
        "--env-output",
        paths.env_output_arg(),
    ]);

    assert!(
        success,
        "proof-workflow-status failed. stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("recommended_exit_code=0"),
        "stdout: {stdout}"
    );

    let packet = read_json(&paths.status);
    assert_eq!(packet["schema"], "tokmd.proof_workflow_status.v1");
    assert_eq!(packet["mode"], "workflow_status_only");
    assert_eq!(packet["workflow_kind"], "fast_proof_run");
    assert_eq!(packet["advisory"], true);
    assert_eq!(packet["required"], false);
    assert_eq!(packet["policy_guardrails"]["required_gate"], false);
    assert_eq!(packet["policy_guardrails"]["codecov_default_upload"], false);
    assert_eq!(packet["recommended_exit_code"], 0);
    assert_eq!(packet["source_artifacts"].as_array().unwrap().len(), 5);
    assert_eq!(packet["command_statuses"].as_array().unwrap().len(), 3);

    let markdown = fs::read_to_string(&paths.summary_md).expect("summary should be readable");
    assert!(markdown.contains("## Fast Proof Run"), "{markdown}");
    assert!(
        markdown.contains("Fast proof-run artifact generation is advisory"),
        "{markdown}"
    );
    assert!(
        markdown.contains("Recommended workflow exit code: 0"),
        "{markdown}"
    );

    let env_output = fs::read_to_string(&paths.env_output).expect("env output should be readable");
    assert_eq!(
        env_output,
        "ok=true\nrecommended_exit_code=0\nworkflow_kind=fast_proof_run\n"
    );

    let (stdout, stderr, success) = run_xtask(&[
        "proof-workflow-status-check",
        "--status",
        paths.status_arg(),
        "--json",
        paths.check_arg(),
    ]);

    assert!(
        success,
        "proof-workflow-status-check failed. stdout: {stdout}\nstderr: {stderr}"
    );
    let receipt = read_json(&paths.check);
    assert_eq!(receipt["schema"], "tokmd.proof_workflow_status_check.v1");
    assert_eq!(receipt["ok"], true);
    assert_eq!(receipt["source_artifacts"], 5);
    assert_eq!(receipt["command_statuses"], 3);
    assert_eq!(receipt["recommended_exit_code"], 0);
}

#[test]
fn proof_workflow_status_preserves_first_nonzero_fast_status_priority() {
    let paths = FixturePaths::new("proof-workflow-status-nonzero");
    paths.reset();
    paths.write_sources();

    let (stdout, stderr, success) = run_xtask(&[
        "proof-workflow-status",
        "--status",
        "proof_run_status=0",
        "--status",
        "proof_run_artifacts_status=7",
        "--status",
        "proof_run_observation_status=3",
        "--proof-policy",
        paths.proof_policy_arg(),
        "--proof-plan",
        paths.proof_plan_arg(),
        "--proof-run-summary",
        paths.proof_run_summary_arg(),
        "--proof-run-artifacts-check",
        paths.proof_run_artifacts_check_arg(),
        "--proof-run-observation",
        paths.proof_run_observation_arg(),
        "--json",
        paths.status_arg(),
    ]);

    assert!(
        success,
        "proof-workflow-status failed. stdout: {stdout}\nstderr: {stderr}"
    );

    let packet = read_json(&paths.status);
    assert_eq!(packet["recommended_exit_code"], 7);
}

#[test]
fn proof_workflow_status_check_rejects_absolute_source_paths() {
    let paths = FixturePaths::new("proof-workflow-status-bad-path");
    paths.reset();
    paths.write_sources();

    let (_, stderr, success) = run_xtask(&[
        "proof-workflow-status",
        "--status",
        "proof_run_status=0",
        "--status",
        "proof_run_artifacts_status=0",
        "--status",
        "proof_run_observation_status=0",
        "--proof-policy",
        paths.proof_policy_arg(),
        "--proof-plan",
        paths.proof_plan_arg(),
        "--proof-run-summary",
        paths.proof_run_summary_arg(),
        "--proof-run-artifacts-check",
        paths.proof_run_artifacts_check_arg(),
        "--proof-run-observation",
        paths.proof_run_observation_arg(),
        "--json",
        paths.status_arg(),
    ]);
    assert!(
        success,
        "proof-workflow-status fixture failed. stderr: {stderr}"
    );

    let mut packet = read_json(&paths.status);
    packet["source_artifacts"][0]["path"] = json!(if cfg!(windows) {
        "C:/tmp/proof-policy.json"
    } else {
        "/tmp/proof-policy.json"
    });
    write_json(&paths.status, &packet);

    let (stdout, stderr, success) = run_xtask(&[
        "proof-workflow-status-check",
        "--status",
        paths.status_arg(),
    ]);

    assert!(!success, "check unexpectedly passed. stdout: {stdout}");
    assert!(stderr.contains("repo-relative"), "stderr: {stderr}");
}

struct FixturePaths {
    root: PathBuf,
    proof_policy: PathBuf,
    proof_plan: PathBuf,
    proof_run_summary: PathBuf,
    proof_run_artifacts_check: PathBuf,
    proof_run_observation: PathBuf,
    status: PathBuf,
    summary_md: PathBuf,
    env_output: PathBuf,
    check: PathBuf,
}

impl FixturePaths {
    fn new(name: &str) -> Self {
        let root = workspace_root().join("target").join(name);
        Self {
            proof_policy: root.join("proof-policy.json"),
            proof_plan: root.join("proof-plan.json"),
            proof_run_summary: root.join("proof-run-summary.json"),
            proof_run_artifacts_check: root.join("proof-run-artifacts-check.json"),
            proof_run_observation: root.join("proof-run-observation.json"),
            status: root.join("proof-workflow-status.json"),
            summary_md: root.join("proof-workflow-status.md"),
            env_output: root.join("proof-workflow-status.env"),
            check: root.join("proof-workflow-status-check.json"),
            root,
        }
    }

    fn reset(&self) {
        if self.root.exists() {
            fs::remove_dir_all(&self.root).expect("stale fixture dir should be removable");
        }
        fs::create_dir_all(&self.root).expect("fixture dir should be creatable");
    }

    fn write_sources(&self) {
        write_json(
            &self.proof_policy,
            &json!({
                "schema": "tokmd.proof_policy.v1",
                "proof_run": {
                    "pr": {
                        "default_enabled": true,
                        "profile": "fast",
                        "required": false,
                        "artifact_name": "fast-proof-run"
                    }
                },
                "executor": {
                    "pr": {"codecov_upload": false},
                    "promotion": {"default_codecov_upload": false}
                }
            }),
        );
        write_json(&self.proof_plan, &json!({"schema": "tokmd.proof_plan.v1"}));
        write_json(
            &self.proof_run_summary,
            &json!({"schema": "tokmd.proof_run_summary.v1", "entries": []}),
        );
        write_json(
            &self.proof_run_artifacts_check,
            &json!({"schema": "tokmd.proof_run_artifacts_check.v1", "ok": true}),
        );
        write_json(
            &self.proof_run_observation,
            &json!({"schema": "tokmd.proof_run_observation.v1", "entries": []}),
        );
    }

    fn proof_policy_arg(&self) -> &str {
        rel_str(&self.proof_policy)
    }

    fn proof_plan_arg(&self) -> &str {
        rel_str(&self.proof_plan)
    }

    fn proof_run_summary_arg(&self) -> &str {
        rel_str(&self.proof_run_summary)
    }

    fn proof_run_artifacts_check_arg(&self) -> &str {
        rel_str(&self.proof_run_artifacts_check)
    }

    fn proof_run_observation_arg(&self) -> &str {
        rel_str(&self.proof_run_observation)
    }

    fn status_arg(&self) -> &str {
        rel_str(&self.status)
    }

    fn summary_md_arg(&self) -> &str {
        rel_str(&self.summary_md)
    }

    fn env_output_arg(&self) -> &str {
        rel_str(&self.env_output)
    }

    fn check_arg(&self) -> &str {
        rel_str(&self.check)
    }
}

fn read_json(path: &Path) -> Value {
    let raw = fs::read_to_string(path).expect("json fixture should be readable");
    serde_json::from_str(&raw).expect("json fixture should parse")
}

fn write_json(path: &Path, value: &Value) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("json parent should be creatable");
    }
    fs::write(
        path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(value).expect("fixture should serialize")
        ),
    )
    .expect("json fixture should be writable");
}

fn rel_str(path: &Path) -> &str {
    path.strip_prefix(workspace_root())
        .expect("fixture path should be under workspace")
        .to_str()
        .expect("fixture path should be utf-8")
}
