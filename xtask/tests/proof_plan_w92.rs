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
    assert!(stdout.contains("--executor-manifest"), "stdout: {stdout}");
    assert!(stdout.contains("--executor-mode"), "stdout: {stdout}");
    assert!(
        stdout.contains("--executor-max-commands"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("--allow-ci-evidence-execution"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("--allow-local-evidence-execution"),
        "stdout: {stdout}"
    );
}

#[test]
fn proof_artifacts_check_help_mentions_executor_paths() {
    let (stdout, stderr, success) = run_xtask(&["proof-artifacts-check", "--help"]);

    assert!(
        success,
        "proof-artifacts-check --help failed. stderr: {stderr}"
    );
    assert!(stdout.contains("--executor-summary"), "stdout: {stdout}");
    assert!(stdout.contains("--executor-manifest"), "stdout: {stdout}");
}

#[test]
fn proof_execution_artifacts_check_help_mentions_executor_paths() {
    let (stdout, stderr, success) = run_xtask(&["proof-execution-artifacts-check", "--help"]);

    assert!(
        success,
        "proof-execution-artifacts-check --help failed. stderr: {stderr}"
    );
    assert!(stdout.contains("--executor-summary"), "stdout: {stdout}");
    assert!(stdout.contains("--executor-manifest"), "stdout: {stdout}");
}

#[test]
fn proof_execution_observation_help_mentions_executor_paths_and_output() {
    let (stdout, stderr, success) = run_xtask(&["proof-execution-observation", "--help"]);

    assert!(
        success,
        "proof-execution-observation --help failed. stderr: {stderr}"
    );
    assert!(stdout.contains("--executor-summary"), "stdout: {stdout}");
    assert!(stdout.contains("--executor-manifest"), "stdout: {stdout}");
    assert!(stdout.contains("--output"), "stdout: {stdout}");
}

#[test]
fn proof_execution_observations_summary_help_mentions_observation_paths() {
    let (stdout, stderr, success) = run_xtask(&["proof-execution-observations-summary", "--help"]);

    assert!(
        success,
        "proof-execution-observations-summary --help failed. stderr: {stderr}"
    );
    assert!(stdout.contains("--observation"), "stdout: {stdout}");
    assert!(stdout.contains("--observations-dir"), "stdout: {stdout}");
    assert!(stdout.contains("--min-observations"), "stdout: {stdout}");
    assert!(stdout.contains("--min-executed"), "stdout: {stdout}");
    assert!(stdout.contains("--min-scopes"), "stdout: {stdout}");
    assert!(stdout.contains("--min-artifacts"), "stdout: {stdout}");
    assert!(
        stdout.contains("--min-passing-collector-runs"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("--collector-runs-json"), "stdout: {stdout}");
    assert!(stdout.contains("--promotion-readiness"), "stdout: {stdout}");
    assert!(stdout.contains("--output"), "stdout: {stdout}");
    assert!(stdout.contains("--summary-md"), "stdout: {stdout}");
}

#[test]
fn affected_plan_ci_blocks_on_planner_generation_failures() {
    let ci = fs::read_to_string(workspace_root().join(".github/workflows/ci.yml"))
        .expect("ci workflow should be readable");

    assert!(
        ci.contains("Affected/proof-plan artifact generation is blocking"),
        "affected-plan summary should describe blocking planner artifacts"
    );
    assert!(
        ci.contains("executor command execution remains disabled"),
        "affected-plan summary should keep executor command execution disabled"
    );
    assert!(
        ci.contains(
            "if [ \"${affected_status}\" -ne 0 ]; then\n            exit \"${affected_status}\"\n          fi"
        ),
        "affected-plan job must fail when affected-scope generation fails"
    );
    assert!(
        ci.contains(
            "if [ \"${proof_plan_status}\" -ne 0 ]; then\n            exit \"${proof_plan_status}\"\n          fi"
        ),
        "affected-plan job must fail when proof-plan generation fails"
    );
    assert!(
        ci.contains(
            "if [ \"${proof_artifacts_status}\" -ne 0 ]; then\n            exit \"${proof_artifacts_status}\"\n          fi"
        ),
        "affected-plan job must still fail when proof artifact verification fails"
    );
}

#[test]
fn scoped_coverage_executor_is_pr_visible_but_not_required() {
    let root = workspace_root();
    let executor = fs::read_to_string(root.join(".github/workflows/proof-executor.yml"))
        .expect("proof executor workflow should be readable");
    let ci =
        fs::read_to_string(root.join(".github/workflows/ci.yml")).expect("ci workflow readable");

    assert!(
        executor.contains("pull_request:"),
        "proof executor should be visible on PRs"
    );
    assert!(
        executor.contains("Scoped Coverage Executor (Non-Required)"),
        "executor status name should make non-required status explicit"
    );
    assert!(
        executor.contains("explicitly non-required PR/manual experiment"),
        "executor summary should not imply required proof authority"
    );
    assert!(
        executor.contains(
            "github.event_name == 'workflow_dispatch' && github.event.inputs.upload_codecov == 'true'"
        ),
        "Codecov upload should remain manual-only"
    );
    assert!(
        executor.contains("proof-execution-observations-summary --observations-dir target/proof"),
        "executor should upload a Rust-generated observation collection summary"
    );
    assert!(
        executor.contains("proof-executor-observation-collection.json"),
        "executor collection summary artifact should have a stable name"
    );
    assert!(
        executor.contains("--summary-md target/proof/proof-executor-observation-collection.md"),
        "executor should append a Rust-generated Markdown collection summary"
    );
    assert!(
        executor.contains("--executor-max-commands \"${PROOF_EXECUTOR_MAX_COMMANDS}\""),
        "executor workflow should keep the command selection limit Rust-owned and manually tunable"
    );
    assert!(
        executor.contains("cargo xtask proof-policy --json > target/proof/proof-policy.json"),
        "executor workflow should resolve PR defaults from proof policy"
    );
    assert!(
        executor.contains("pr.get(\"default_enabled\") is not True"),
        "executor workflow should require policy-backed default PR observation"
    );
    assert!(
        executor.contains("PROOF_EXECUTOR_MAX_COMMANDS_INPUT"),
        "manual executor command override should be separate from the policy default"
    );
    assert!(
        !ci.contains("scoped-coverage-executor"),
        "required CI aggregate must not depend on the executor experiment"
    );
}

#[test]
fn proof_observation_collection_workflow_summarizes_downloaded_executor_runs() {
    let root = workspace_root();
    let collector =
        fs::read_to_string(root.join(".github/workflows/proof-observation-collection.yml"))
            .expect("proof observation collection workflow should be readable");
    let ci =
        fs::read_to_string(root.join(".github/workflows/ci.yml")).expect("ci workflow readable");

    assert!(
        collector.contains("workflow_dispatch:"),
        "collector should be manually dispatched"
    );
    assert!(
        collector.contains("actions: read"),
        "collector needs read-only workflow artifact access"
    );
    assert!(
        collector.contains("gh run list --workflow proof-executor.yml --status success"),
        "collector should enumerate successful proof executor runs"
    );
    assert!(
        collector.contains("gh run download \"${run_id}\" --name proof-executor-artifacts"),
        "collector should download the stable proof executor artifact"
    );
    assert!(
        collector
            .contains("gh run list --workflow proof-observation-collection.yml --status success"),
        "collector should enumerate recent passing manual collector runs"
    );
    assert!(
        collector.contains("cargo xtask proof-execution-observations-summary"),
        "collector should keep observation summarization Rust-owned"
    );
    assert!(
        collector.contains(
            "cargo xtask proof-policy --json > target/proof-observations/proof-policy.json"
        ),
        "collector should resolve default thresholds from the checked proof policy"
    );
    assert!(
        collector.contains("promotion = json.load(handle)[\"executor\"][\"promotion\"]"),
        "collector should read executor promotion thresholds from proof-policy JSON"
    );
    assert!(
        collector.contains("RUN_LIMIT_INPUT: ${{ github.event.inputs.run_limit || '' }}"),
        "collector should preserve workflow-dispatch threshold overrides"
    );
    assert!(
        collector.contains("--min-observations \"${MIN_OBSERVATIONS}\""),
        "collector should expose observation readiness thresholds"
    );
    assert!(
        collector.contains("--min-passing-collector-runs \"${MIN_PASSING_COLLECTOR_RUNS}\""),
        "collector should expose the passing collector-run promotion floor"
    );
    assert!(
        collector.contains("--collector-runs-json target/proof-observations/collector-runs.json"),
        "collector should feed GitHub run history into Rust-owned promotion readiness"
    );
    assert!(
        collector.contains("--promotion-readiness target/proof-observations/proof-executor-promotion-readiness.json"),
        "collector should emit a first-class promotion-readiness receipt"
    );
    assert!(
        collector.contains("MIN_EXECUTED_INPUT: ${{ github.event.inputs.min_executed || '' }}"),
        "collector should let blank executed thresholds fall back to the proof policy"
    );
    assert!(
        !collector.contains("MIN_EXECUTED: ${{ github.event.inputs.min_executed || '0' }}"),
        "collector should not keep stale hard-coded readiness defaults"
    );
    assert!(
        collector.contains("thresholds.env"),
        "collector should artifact the resolved threshold values"
    );
    assert!(
        collector.contains("| Input | Value | Source |"),
        "collector summary should show whether thresholds came from policy or manual inputs"
    );
    assert!(
        collector.contains("MIN_EXECUTED_SOURCE"),
        "collector summary should show the executed threshold source"
    );
    assert!(
        collector.contains("MIN_PASSING_COLLECTOR_RUNS_SOURCE"),
        "collector summary should show the collector-run threshold source"
    );
    assert!(
        collector.contains("proof-executor-observation-collection.md"),
        "collector should append the Rust-generated Markdown summary"
    );
    assert!(
        !collector.contains("proof --profile affected"),
        "collector must not execute new planner-selected evidence commands"
    );
    assert!(
        !ci.contains("proof-observation-collection"),
        "required CI aggregate must not depend on the manual collector"
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
fn proof_artifacts_check_accepts_generated_executor_artifacts() {
    let temp = tempfile::tempdir().expect("tempdir");
    let summary_path = temp.path().join("executor-summary.json");
    let manifest_path = temp.path().join("executor-manifest.json");
    let summary_arg = summary_path.to_string_lossy().to_string();
    let manifest_arg = manifest_path.to_string_lossy().to_string();

    let (_stdout, stderr, success) = run_xtask(&[
        "proof",
        "--profile",
        "affected",
        "--base",
        "HEAD",
        "--head",
        "HEAD",
        "--plan",
        "--executor-summary",
        &summary_arg,
        "--executor-manifest",
        &manifest_arg,
    ]);
    assert!(
        success,
        "proof artifact generation failed. stderr: {stderr}"
    );

    let (stdout, stderr, success) = run_xtask(&[
        "proof-artifacts-check",
        "--executor-summary",
        &summary_arg,
        "--executor-manifest",
        &manifest_arg,
    ]);

    assert!(success, "proof-artifacts-check failed. stderr: {stderr}");
    assert!(stdout.contains("Proof artifacts OK"), "stdout: {stdout}");
    assert!(stdout.contains("0 command(s)"), "stdout: {stdout}");
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
fn proof_plan_refuses_execute_executor_mode() {
    let (_stdout, stderr, success) = run_xtask(&[
        "proof",
        "--profile",
        "affected",
        "--base",
        "HEAD",
        "--head",
        "HEAD",
        "--plan",
        "--executor-mode",
        "execute",
    ]);

    assert!(!success, "proof --plan --executor-mode execute should fail");
    assert!(stderr.contains("--plan"), "stderr: {stderr}");
    assert!(stderr.contains("execute"), "stderr: {stderr}");
}

#[test]
fn proof_plan_rejects_zero_executor_max_commands() {
    let (_stdout, stderr, success) = run_xtask(&[
        "proof",
        "--profile",
        "affected",
        "--base",
        "HEAD",
        "--head",
        "HEAD",
        "--plan",
        "--executor-max-commands",
        "0",
    ]);

    assert!(
        !success,
        "proof --plan should reject zero executor command limit"
    );
    assert!(
        stderr.contains("--executor-max-commands"),
        "stderr: {stderr}"
    );
}

#[test]
fn local_execute_requires_explicit_local_opt_in() {
    let temp = tempfile::tempdir().expect("tempdir");
    let summary_path = temp.path().join("executor-summary.json");
    let manifest_path = temp.path().join("executor-manifest.json");
    let summary_arg = summary_path.to_string_lossy().to_string();
    let manifest_arg = manifest_path.to_string_lossy().to_string();

    let (_stdout, stderr, success) = run_xtask(&[
        "proof",
        "--profile",
        "affected",
        "--base",
        "HEAD",
        "--head",
        "HEAD",
        "--executor-mode",
        "execute",
        "--executor-summary",
        &summary_arg,
        "--executor-manifest",
        &manifest_arg,
    ]);

    assert!(!success, "local execute should require explicit opt-in");
    assert!(
        stderr.contains("--allow-local-evidence-execution"),
        "stderr: {stderr}"
    );
}

#[test]
fn local_execute_can_write_zero_command_executor_artifacts() {
    let temp = tempfile::tempdir().expect("tempdir");
    let summary_path = temp.path().join("executor-summary.json");
    let manifest_path = temp.path().join("executor-manifest.json");
    let summary_arg = summary_path.to_string_lossy().to_string();
    let manifest_arg = manifest_path.to_string_lossy().to_string();

    let (stdout, stderr, success) = run_xtask(&[
        "proof",
        "--profile",
        "affected",
        "--base",
        "HEAD",
        "--head",
        "HEAD",
        "--executor-mode",
        "execute",
        "--allow-local-evidence-execution",
        "--executor-summary",
        &summary_arg,
        "--executor-manifest",
        &manifest_arg,
    ]);

    assert!(success, "local execute failed. stderr: {stderr}");
    let value: serde_json::Value =
        serde_json::from_str(&stdout).expect("proof execute should still emit JSON plan");
    assert_eq!(value["schema"], "tokmd.proof_plan.v1");

    let summary = fs::read_to_string(summary_path).expect("summary should be written");
    let summary: serde_json::Value =
        serde_json::from_str(&summary).expect("summary should be valid JSON");
    assert_eq!(summary["mode"], "execute");
    assert_eq!(summary["execution_status"], "executed");
    assert_eq!(summary["execution_guard"]["enabled"], true);
    assert_eq!(
        summary["execution_guard"]["reason"],
        "local_explicit_opt_in_enabled"
    );
    assert_eq!(summary["counts"]["executed"], 0);
    assert_eq!(summary["counts"]["failed"], 0);

    let manifest = fs::read_to_string(manifest_path).expect("manifest should be written");
    let manifest: serde_json::Value =
        serde_json::from_str(&manifest).expect("manifest should be valid JSON");
    assert_eq!(manifest["mode"], "execute");
    assert_eq!(manifest["selection"]["selected"], 0);
    assert_eq!(manifest["selection"]["executed"], 0);

    let (stdout, stderr, success) = run_xtask(&[
        "proof-execution-artifacts-check",
        "--executor-summary",
        &summary_arg,
        "--executor-manifest",
        &manifest_arg,
    ]);
    assert!(
        success,
        "proof-execution-artifacts-check failed. stderr: {stderr}"
    );
    assert!(
        stdout.contains("Proof execution artifacts OK"),
        "stdout: {stdout}"
    );

    let observation_path = temp.path().join("proof-executor-observation.json");
    let observation_arg = observation_path.to_string_lossy().to_string();
    let (stdout, stderr, success) = run_xtask(&[
        "proof-execution-observation",
        "--executor-summary",
        &summary_arg,
        "--executor-manifest",
        &manifest_arg,
        "--output",
        &observation_arg,
    ]);
    assert!(
        success,
        "proof-execution-observation failed. stderr: {stderr}"
    );
    assert!(
        stdout.contains("Proof execution observation OK"),
        "stdout: {stdout}"
    );
    let observation = fs::read_to_string(observation_path).expect("observation should be written");
    let observation: serde_json::Value =
        serde_json::from_str(&observation).expect("observation should be valid JSON");
    assert_eq!(observation["schema"], "tokmd.proof_executor_observation.v1");
    assert_eq!(observation["execution_status"], "executed");
    assert_eq!(observation["counts"]["selected"], 0);
    assert_eq!(observation["counts"]["executed"], 0);
    assert!(observation["scopes"].as_array().unwrap().is_empty());

    let (stdout, stderr, success) = run_xtask(&[
        "proof-execution-observations-summary",
        "--observation",
        &observation_arg,
    ]);
    assert!(
        success,
        "proof-execution-observations-summary failed. stderr: {stderr}"
    );
    let collection: serde_json::Value =
        serde_json::from_str(&stdout).expect("collection should be valid JSON");
    assert_eq!(
        collection["schema"],
        "tokmd.proof_executor_observation_collection.v1"
    );
    assert_eq!(collection["counts"]["observations"], 1);
    assert_eq!(collection["counts"]["executed"], 0);
    assert!(collection["scopes"].as_array().unwrap().is_empty());
    assert_eq!(collection["sources"].as_array().unwrap().len(), 1);

    let (stdout, stderr, success) = run_xtask(&[
        "proof-execution-observations-summary",
        "--observations-dir",
        &temp.path().to_string_lossy(),
    ]);
    assert!(
        success,
        "proof-execution-observations-summary --observations-dir failed. stderr: {stderr}"
    );
    let collection: serde_json::Value =
        serde_json::from_str(&stdout).expect("directory collection should be valid JSON");
    assert_eq!(
        collection["schema"],
        "tokmd.proof_executor_observation_collection.v1"
    );
    assert_eq!(collection["counts"]["observations"], 1);
    assert_eq!(collection["counts"]["executed"], 0);

    let collection_path = temp
        .path()
        .join("proof-executor-observation-collection.json");
    let collection_arg = collection_path.to_string_lossy().to_string();
    let summary_path = temp.path().join("proof-executor-observation-collection.md");
    let summary_md_arg = summary_path.to_string_lossy().to_string();
    let (stdout, stderr, success) = run_xtask(&[
        "proof-execution-observations-summary",
        "--observation",
        &observation_arg,
        "--output",
        &collection_arg,
        "--summary-md",
        &summary_md_arg,
    ]);
    assert!(
        success,
        "proof-execution-observations-summary --summary-md failed. stderr: {stderr}"
    );
    assert!(stdout.contains("wrote"), "stdout: {stdout}");
    let summary_md = fs::read_to_string(summary_path).expect("summary markdown should be written");
    assert!(
        summary_md.contains("# Proof Executor Observation Collection"),
        "{summary_md}"
    );
    assert!(
        summary_md.contains("| Executed commands | 0 |"),
        "{summary_md}"
    );

    let collector_runs_path = temp.path().join("collector-runs.json");
    fs::write(
        &collector_runs_path,
        r#"[{"databaseId":25502593070,"event":"workflow_dispatch","headBranch":"main","headSha":"abc123","createdAt":"2026-05-07T14:46:00Z","url":"https://github.com/EffortlessMetrics/tokmd/actions/runs/25502593070"}]"#,
    )
    .expect("collector runs JSON should be writable");
    let collector_runs_arg = collector_runs_path.to_string_lossy().to_string();
    let readiness_path = temp.path().join("proof-executor-promotion-readiness.json");
    let readiness_arg = readiness_path.to_string_lossy().to_string();
    let (stdout, stderr, success) = run_xtask(&[
        "proof-execution-observations-summary",
        "--observation",
        &observation_arg,
        "--output",
        &collection_arg,
        "--collector-runs-json",
        &collector_runs_arg,
        "--min-passing-collector-runs",
        "1",
        "--promotion-readiness",
        &readiness_arg,
    ]);
    assert!(
        success,
        "proof-execution-observations-summary --promotion-readiness failed. stderr: {stderr}"
    );
    assert!(stdout.contains("promotion-readiness"), "stdout: {stdout}");
    let readiness =
        fs::read_to_string(readiness_path).expect("promotion readiness should be written");
    let readiness: serde_json::Value =
        serde_json::from_str(&readiness).expect("readiness should be valid JSON");
    assert_eq!(
        readiness["schema"],
        "tokmd.proof_executor_promotion_readiness.v1"
    );
    assert_eq!(readiness["actuals"]["passing_collector_runs"], 1);
    assert_eq!(readiness["thresholds"]["min_passing_collector_runs"], 1);

    let (_stdout, stderr, success) = run_xtask(&[
        "proof-execution-observations-summary",
        "--observation",
        &observation_arg,
        "--min-executed",
        "1",
    ]);
    assert!(
        !success,
        "collection threshold should reject zero executed observations"
    );
    assert!(stderr.contains("--min-executed 1"), "stderr: {stderr}");

    let (_stdout, stderr, success) = run_xtask(&[
        "proof-artifacts-check",
        "--executor-summary",
        &summary_arg,
        "--executor-manifest",
        &manifest_arg,
    ]);
    assert!(
        !success,
        "no-execution verifier should reject executed artifacts"
    );
    assert!(
        stderr.contains("proof-execution-artifacts-check"),
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
        "local_requires_--allow-local-evidence-execution"
    );
    assert_eq!(summary["family"], "coverage");
    assert_eq!(summary["required"], false);
    assert_eq!(summary["counts"]["selected"], 0);
    assert_eq!(summary["counts"]["dry_run"], 0);
    assert_eq!(summary["counts"]["executed"], 0);
    assert!(summary["entries"].as_array().unwrap().is_empty());
}

#[test]
fn proof_plan_writes_executor_manifest_artifact() {
    let temp = tempfile::tempdir().expect("tempdir");
    let manifest_path = temp.path().join("executor-manifest.json");
    let manifest_arg = manifest_path.to_string_lossy().to_string();
    let (stdout, stderr, success) = run_xtask(&[
        "proof",
        "--profile",
        "deep",
        "--plan",
        "--executor-manifest",
        &manifest_arg,
        "--executor-mode",
        "dry-run",
    ]);

    assert!(
        success,
        "proof --executor-manifest failed. stderr: {stderr}"
    );
    let value: serde_json::Value =
        serde_json::from_str(&stdout).expect("proof --plan should still emit JSON");
    assert_eq!(value["schema"], "tokmd.proof_plan.v1");
    assert_eq!(value["profile"], "deep");

    let manifest = fs::read_to_string(manifest_path).expect("executor manifest should be written");
    let manifest: serde_json::Value =
        serde_json::from_str(&manifest).expect("executor manifest should be valid JSON");
    assert_eq!(manifest["schema"], "tokmd.proof_executor_manifest.v1");
    assert_eq!(manifest["mode"], "dry_run");
    assert_eq!(manifest["family"], "coverage");
    assert_eq!(manifest["selection"]["source"], "proof_plan");
    assert_eq!(manifest["selection"]["max_dry_run_commands"], 1);
    assert_eq!(manifest["selection"]["required_included"], false);
    assert_eq!(manifest["selection"]["selected"], 0);
    assert_eq!(manifest["selection"]["executed"], 0);
    assert!(manifest["commands"].as_array().unwrap().is_empty());
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
