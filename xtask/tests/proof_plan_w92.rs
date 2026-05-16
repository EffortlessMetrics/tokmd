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
    assert!(stdout.contains("--run-required"), "stdout: {stdout}");
    assert!(stdout.contains("--proof-run-summary"), "stdout: {stdout}");
    assert!(
        stdout.contains("--allow-ci-required-execution"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("--allow-local-required-execution"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("--summary-md"), "stdout: {stdout}");
    assert!(stdout.contains("--plan-json"), "stdout: {stdout}");
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
fn proof_run_artifacts_check_help_mentions_summary_path() {
    let (stdout, stderr, success) = run_xtask(&["proof-run-artifacts-check", "--help"]);

    assert!(
        success,
        "proof-run-artifacts-check --help failed. stderr: {stderr}"
    );
    assert!(stdout.contains("--proof-run-summary"), "stdout: {stdout}");
}

#[test]
fn proof_run_observation_help_mentions_summary_path_and_output() {
    let (stdout, stderr, success) = run_xtask(&["proof-run-observation", "--help"]);

    assert!(
        success,
        "proof-run-observation --help failed. stderr: {stderr}"
    );
    assert!(stdout.contains("--proof-run-summary"), "stdout: {stdout}");
    assert!(stdout.contains("--output"), "stdout: {stdout}");
}

#[test]
fn proof_run_observations_summary_help_mentions_observation_paths() {
    let (stdout, stderr, success) = run_xtask(&["proof-run-observations-summary", "--help"]);

    assert!(
        success,
        "proof-run-observations-summary --help failed. stderr: {stderr}"
    );
    assert!(stdout.contains("--observation"), "stdout: {stdout}");
    assert!(stdout.contains("--observations-dir"), "stdout: {stdout}");
    assert!(stdout.contains("--source-runs-json"), "stdout: {stdout}");
    assert!(stdout.contains("--output"), "stdout: {stdout}");
    assert!(stdout.contains("--summary-md"), "stdout: {stdout}");
}

#[test]
fn proof_run_pr_policy_help_mentions_policy_and_output() {
    let (stdout, stderr, success) = run_xtask(&["proof-run-pr-policy", "--help"]);

    assert!(
        success,
        "proof-run-pr-policy --help failed. stderr: {stderr}"
    );
    assert!(stdout.contains("--proof-policy-json"), "stdout: {stdout}");
    assert!(stdout.contains("--github-output"), "stdout: {stdout}");
}

#[test]
fn proof_run_pr_policy_writes_github_output_artifact() {
    let root = workspace_root();
    let policy = root
        .join("target")
        .join("proof-run-pr-policy-w92")
        .join("proof-policy.json");
    let output = root
        .join("target")
        .join("proof-run-pr-policy-w92")
        .join("proof-run-pr.outputs");
    if output.exists() {
        fs::remove_file(&output).expect("stale proof-run PR output fixture should be removable");
    }

    let policy_arg = policy.to_string_lossy().to_string();
    let output_arg = output.to_string_lossy().to_string();
    let (_, stderr, success) = run_xtask(&["proof-policy", "--json-output", &policy_arg]);
    assert!(success, "proof-policy fixture failed. stderr: {stderr}");

    let (stdout, stderr, success) = run_xtask(&[
        "proof-run-pr-policy",
        "--proof-policy-json",
        &policy_arg,
        "--github-output",
        &output_arg,
    ]);

    assert!(
        success,
        "proof-run-pr-policy failed. stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("proof-run PR policy: wrote GitHub output"),
        "stdout: {stdout}"
    );

    let body = fs::read_to_string(output).expect("proof-run PR output should be readable");
    assert_eq!(body, "profile=fast\nartifact_name=fast-proof-run\n");
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
    assert!(stdout.contains("--source-runs-json"), "stdout: {stdout}");
    assert!(stdout.contains("--promotion-readiness"), "stdout: {stdout}");
    assert!(stdout.contains("--output"), "stdout: {stdout}");
    assert!(stdout.contains("--summary-md"), "stdout: {stdout}");
}

#[test]
fn proof_observation_thresholds_help_mentions_policy_and_overrides() {
    let (stdout, stderr, success) = run_xtask(&["proof-observation-thresholds", "--help"]);

    assert!(
        success,
        "proof-observation-thresholds --help failed. stderr: {stderr}"
    );
    assert!(stdout.contains("--proof-policy-json"), "stdout: {stdout}");
    assert!(stdout.contains("--env-output"), "stdout: {stdout}");
    assert!(stdout.contains("--run-limit"), "stdout: {stdout}");
    assert!(stdout.contains("--min-observations"), "stdout: {stdout}");
    assert!(
        stdout.contains("--min-passing-collector-runs"),
        "stdout: {stdout}"
    );
}

#[test]
fn proof_observation_thresholds_writes_env_artifact() {
    let root = workspace_root();
    let policy = root
        .join("target")
        .join("proof-thresholds-w92")
        .join("proof-policy.json");
    let env = root
        .join("target")
        .join("proof-thresholds-w92")
        .join("thresholds.env");
    if env.exists() {
        fs::remove_file(&env).expect("stale thresholds fixture should be removable");
    }

    let policy_arg = policy.to_string_lossy().to_string();
    let env_arg = env.to_string_lossy().to_string();
    let (_, stderr, success) = run_xtask(&["proof-policy", "--json-output", &policy_arg]);
    assert!(success, "proof-policy fixture failed. stderr: {stderr}");

    let (stdout, stderr, success) = run_xtask(&[
        "proof-observation-thresholds",
        "--proof-policy-json",
        &policy_arg,
        "--env-output",
        &env_arg,
        "--min-executed",
        "7",
    ]);

    assert!(
        success,
        "proof-observation-thresholds failed. stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("proof observation thresholds: wrote 6 threshold(s)"),
        "stdout: {stdout}"
    );

    let env_body = fs::read_to_string(&env).expect("threshold env artifact should be readable");
    assert!(env_body.contains("RUN_LIMIT=100"), "{env_body}");
    assert!(
        env_body.contains("RUN_LIMIT_SOURCE=ci/proof.toml"),
        "{env_body}"
    );
    assert!(env_body.contains("MIN_EXECUTED=7"), "{env_body}");
    assert!(
        env_body.contains("MIN_EXECUTED_SOURCE=workflow_dispatch"),
        "{env_body}"
    );
}

#[test]
fn proof_executor_pr_policy_help_mentions_policy_env_and_override() {
    let (stdout, stderr, success) = run_xtask(&["proof-executor-pr-policy", "--help"]);

    assert!(
        success,
        "proof-executor-pr-policy --help failed. stderr: {stderr}"
    );
    assert!(stdout.contains("--proof-policy-json"), "stdout: {stdout}");
    assert!(stdout.contains("--env-output"), "stdout: {stdout}");
    assert!(stdout.contains("--max-commands"), "stdout: {stdout}");
}

#[test]
fn proof_executor_pr_policy_writes_env_artifact() {
    let root = workspace_root();
    let policy = root
        .join("target")
        .join("proof-executor-pr-policy-w92")
        .join("proof-policy.json");
    let env = root
        .join("target")
        .join("proof-executor-pr-policy-w92")
        .join("proof-executor-pr.env");
    if env.exists() {
        fs::remove_file(&env).expect("stale executor PR policy fixture should be removable");
    }

    let policy_arg = policy.to_string_lossy().to_string();
    let env_arg = env.to_string_lossy().to_string();
    let (_, stderr, success) = run_xtask(&["proof-policy", "--json-output", &policy_arg]);
    assert!(success, "proof-policy fixture failed. stderr: {stderr}");

    let (stdout, stderr, success) = run_xtask(&[
        "proof-executor-pr-policy",
        "--proof-policy-json",
        &policy_arg,
        "--env-output",
        &env_arg,
        "--max-commands",
        "5",
    ]);

    assert!(
        success,
        "proof-executor-pr-policy failed. stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("proof executor PR policy: wrote executor PR env"),
        "stdout: {stdout}"
    );

    let env_body = fs::read_to_string(&env).expect("executor PR env artifact should be readable");
    assert!(
        env_body.contains("PROOF_EXECUTOR_MAX_COMMANDS=5"),
        "{env_body}"
    );
    assert!(
        env_body.contains("PROOF_EXECUTOR_MAX_COMMANDS_SOURCE=workflow_dispatch"),
        "{env_body}"
    );
    assert!(
        env_body.contains("PROOF_EXECUTOR_PR_DEFAULT_ENABLED=true"),
        "{env_body}"
    );
    assert!(
        env_body.contains("PROOF_EXECUTOR_PR_REQUIRED=false"),
        "{env_body}"
    );
    assert!(
        env_body.contains("PROOF_EXECUTOR_PR_CODECOV_UPLOAD=false"),
        "{env_body}"
    );
}

#[test]
fn proof_observation_run_ids_help_mentions_input_and_output() {
    let (stdout, stderr, success) = run_xtask(&["proof-observation-run-ids", "--help"]);

    assert!(
        success,
        "proof-observation-run-ids --help failed. stderr: {stderr}"
    );
    assert!(stdout.contains("--runs-json"), "stdout: {stdout}");
    assert!(stdout.contains("--output"), "stdout: {stdout}");
}

#[test]
fn proof_observation_run_ids_writes_ids_in_source_order() {
    let root = workspace_root();
    let dir = root.join("target").join("proof-run-ids-w92");
    let runs = dir.join("runs.json");
    let output = dir.join("run-ids.txt");
    fs::create_dir_all(&dir).expect("run id fixture dir should be creatable");
    fs::write(
        &runs,
        r#"[
  {"databaseId":333,"headSha":"c"},
  {"databaseId":"222","headSha":"b"},
  {"databaseId":111,"headSha":"a"}
]"#,
    )
    .expect("run-list fixture should be writable");
    if output.exists() {
        fs::remove_file(&output).expect("stale run ids fixture should be removable");
    }

    let runs_arg = runs.to_string_lossy().to_string();
    let output_arg = output.to_string_lossy().to_string();
    let (stdout, stderr, success) = run_xtask(&[
        "proof-observation-run-ids",
        "--runs-json",
        &runs_arg,
        "--output",
        &output_arg,
    ]);

    assert!(
        success,
        "proof-observation-run-ids failed. stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("proof observation run ids: wrote 3 id(s)"),
        "stdout: {stdout}"
    );
    let body = fs::read_to_string(output).expect("run ids artifact should be readable");
    assert_eq!(body, "333\n222\n111\n");
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
        ci.contains("--json-output target/proof/affected.json"),
        "affected-plan job should write affected.json through xtask instead of shell redirection"
    );
    assert!(
        !ci.contains("--json > target/proof/affected.json"),
        "affected-plan job should not capture affected.json with shell redirection"
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
fn proof_plan_json_writes_plan_report_artifact() {
    let root = workspace_root();
    let path = root
        .join("target")
        .join("proof-plan-w92")
        .join("proof-plan.json");
    if path.exists() {
        fs::remove_file(&path).expect("stale proof-plan fixture should be removable");
    }

    let path_arg = path.to_string_lossy().to_string();
    let (stdout, stderr, success) = run_xtask(&[
        "proof",
        "--profile",
        "affected",
        "--base",
        "HEAD",
        "--head",
        "HEAD",
        "--plan",
        "--plan-json",
        &path_arg,
    ]);

    assert!(success, "proof --plan-json failed. stderr: {stderr}");
    assert!(stdout.contains("\"schema\": \"tokmd.proof_plan.v1\""));
    assert!(path.exists(), "proof plan artifact should be written");

    let written = fs::read_to_string(&path).expect("proof plan artifact should be readable");
    let stdout_json: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout proof plan should be JSON");
    let written_json: serde_json::Value =
        serde_json::from_str(&written).expect("written proof plan should be JSON");

    assert_eq!(written_json["schema"], "tokmd.proof_plan.v1");
    assert_eq!(written_json, stdout_json);
}

#[test]
fn fast_proof_run_ci_job_is_advisory_and_verified() {
    let ci = fs::read_to_string(workspace_root().join(".github/workflows/ci.yml"))
        .expect("ci workflow should be readable");
    let required_section = ci
        .split("  ci-required:")
        .nth(1)
        .expect("CI workflow should define required aggregate");

    assert!(ci.contains("fast-proof-run:"), "fast proof job missing");
    assert!(
        ci.contains("Fast Proof Run (Advisory)"),
        "fast proof job should advertise advisory status"
    );
    assert!(
        ci.contains("cargo xtask proof-policy --json-output target/proof-run/proof-policy.json"),
        "fast proof job should resolve checked policy"
    );
    assert!(
        !ci.contains("cargo xtask proof-policy --json > target/proof-run/proof-policy.json"),
        "fast proof job should not capture proof-policy JSON with shell redirection"
    );
    assert!(
        ci.contains("cargo xtask proof-run-pr-policy"),
        "fast proof job should resolve proof-run PR policy through xtask"
    );
    assert!(
        ci.contains("--proof-policy-json target/proof-run/proof-policy.json"),
        "fast proof policy resolver should read the checked proof-policy JSON"
    );
    assert!(
        ci.contains("--github-output target/proof-run/proof-run-pr.outputs"),
        "fast proof policy resolver should write a stable GitHub output artifact"
    );
    assert!(
        ci.contains("cat target/proof-run/proof-run-pr.outputs >> \"$GITHUB_OUTPUT\""),
        "fast proof job should source Rust-resolved outputs"
    );
    assert!(
        !ci.contains("proof_run.pr.required must remain false"),
        "fast proof policy should not be enforced with inline Python"
    );
    assert!(
        !ci.contains("python - <<'PY' >> \"$GITHUB_OUTPUT\""),
        "fast proof job should not resolve policy with inline Python"
    );
    assert!(
        ci.contains("cargo xtask proof --profile \"${PROOF_RUN_PROFILE}\" --run-required --allow-ci-required-execution"),
        "fast proof job should use the policy-selected required proof runner"
    );
    assert!(
        ci.contains("--plan-json target/proof-run/proof-plan.json"),
        "fast proof job should write the proof plan as a Rust-owned JSON artifact"
    );
    assert!(
        ci.contains("cargo xtask proof-run-artifacts-check --proof-run-summary target/proof-run/proof-run-summary.json"),
        "fast proof job should verify the required-run summary"
    );
    assert!(
        ci.contains("cargo xtask proof-run-observation --proof-run-summary target/proof-run/proof-run-summary.json --output target/proof-run/proof-run-observation.json"),
        "fast proof job should emit a compact proof-run observation"
    );
    assert!(
        ci.contains("Fast proof-run artifact generation is advisory"),
        "fast proof job summary should state advisory status"
    );
    assert!(
        ci.contains(
            "name: ${{ steps.proof_run_policy.outputs.artifact_name || 'fast-proof-run' }}"
        ),
        "fast proof job should upload the policy-named artifact with a stable fallback"
    );
    assert!(
        !required_section.contains("- fast-proof-run"),
        "required CI aggregate must not depend on the advisory fast proof runner"
    );
}

#[test]
fn ci_mutation_job_uses_rust_owned_mutation_scope_selector() {
    let ci = fs::read_to_string(workspace_root().join(".github/workflows/ci.yml"))
        .expect("ci workflow should be readable");
    let mutation_section = ci
        .split("  mutation:")
        .nth(1)
        .and_then(|section| section.split("  ci-required:").next())
        .expect("CI workflow should define mutation and required aggregate jobs");

    assert!(
        mutation_section.contains("cargo xtask mutation-scope"),
        "CI mutation job should route file selection through xtask"
    );
    assert!(
        mutation_section.contains("--base-ref \"$BASE_REF\""),
        "CI mutation job should record the human base ref"
    );
    assert!(
        mutation_section.contains("--base \"origin/$BASE_REF\""),
        "CI mutation job should diff against the fetched base ref"
    );
    assert!(
        mutation_section.contains("--head HEAD"),
        "CI mutation job should diff against checked-out HEAD"
    );
    assert!(
        mutation_section.contains("--all-changed-files all_changed_files.txt"),
        "CI mutation job should preserve the all-changed-files evidence path"
    );
    assert!(
        mutation_section.contains("--changed-files changed_files.txt"),
        "CI mutation job should preserve the changed_files.txt execution input"
    );
    assert!(
        mutation_section.contains("--json-output target/mutation/mutation-scope.json"),
        "CI mutation job should emit the mutation scope JSON receipt"
    );
    assert!(
        mutation_section.contains("--github-output \"$GITHUB_OUTPUT\""),
        "CI mutation job should preserve workflow-compatible count/files outputs"
    );
    assert!(
        mutation_section.contains("steps.changed.outputs.count != '0'"),
        "CI mutation execution should still branch on the Rust-owned count output"
    );
    assert!(
        mutation_section.contains("done < changed_files.txt"),
        "CI mutation execution should keep consuming changed_files.txt"
    );
    assert!(
        mutation_section.contains("target/mutation/mutation-scope.json"),
        "CI mutation artifacts should include the mutation scope receipt"
    );
    assert!(
        mutation_section.contains("all_changed_files.txt"),
        "CI mutation artifacts should include all changed-file evidence"
    );
    assert!(
        !mutation_section.contains("git diff --name-only"),
        "CI mutation job should not keep the inline git diff classifier"
    );
    assert!(
        !mutation_section.contains("grep -v '/tests/'"),
        "CI mutation job should not keep duplicate shell filter logic"
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
        executor.contains("cargo xtask proof-policy --json-output target/proof/proof-policy.json"),
        "executor workflow should resolve PR defaults from proof policy"
    );
    assert!(
        !executor.contains("cargo xtask proof-policy --json > target/proof/proof-policy.json"),
        "executor workflow should not capture proof-policy JSON with shell redirection"
    );
    assert!(
        executor.contains("cargo xtask proof-executor-pr-policy"),
        "executor workflow should resolve PR policy through xtask"
    );
    assert!(
        executor.contains("--proof-policy-json target/proof/proof-policy.json"),
        "executor PR policy resolver should read the checked proof-policy JSON"
    );
    assert!(
        executor.contains("--env-output target/proof/proof-executor-pr.env"),
        "executor PR policy resolver should write a stable env artifact"
    );
    assert!(
        executor.contains("--max-commands \"${PROOF_EXECUTOR_MAX_COMMANDS_INPUT}\""),
        "executor PR policy resolver should keep the manual command override explicit"
    );
    assert!(
        executor.contains("cat target/proof/proof-executor-pr.env >> \"$GITHUB_ENV\""),
        "executor workflow should source Rust-resolved env output"
    );
    assert!(
        executor.contains("executor max commands source"),
        "executor summary should show whether max commands came from policy or manual dispatch"
    );
    assert!(
        executor.contains("--plan-json target/proof/proof-plan.json"),
        "executor workflow should write the proof plan as a Rust-owned JSON artifact"
    );
    assert!(
        executor.contains("--json-output target/proof/affected.json"),
        "executor workflow should write affected.json through xtask instead of shell redirection"
    );
    assert!(
        !executor.contains("--json > target/proof/affected.json"),
        "executor workflow should not capture affected.json with shell redirection"
    );
    assert!(
        !executor.contains("pr.get(\"default_enabled\") is not True"),
        "executor workflow should not enforce PR policy with inline Python"
    );
    assert!(
        !executor.contains("python - <<'PY'"),
        "executor workflow should not resolve PR policy with inline Python"
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
        collector.contains("cargo xtask proof-observation-run-ids"),
        "collector should derive run-id downloads through xtask"
    );
    assert!(
        collector.contains("--runs-json target/proof-observations/runs.json"),
        "collector should read the saved source-run window"
    );
    assert!(
        collector.contains("--output target/proof-observations/run-ids.txt"),
        "collector should write a stable run-id artifact"
    );
    assert!(
        !collector.contains("with open(\"target/proof-observations/runs.json\""),
        "collector should not parse run ids with inline Python"
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
            "cargo xtask proof-policy --json-output target/proof-observations/proof-policy.json"
        ),
        "collector should resolve default thresholds from the checked proof policy"
    );
    assert!(
        !collector.contains(
            "cargo xtask proof-policy --json > target/proof-observations/proof-policy.json"
        ),
        "collector should not capture proof-policy JSON with shell redirection"
    );
    assert!(
        collector.contains("cargo xtask proof-observation-thresholds"),
        "collector should resolve threshold env values through xtask"
    );
    assert!(
        collector.contains("--proof-policy-json target/proof-observations/proof-policy.json"),
        "collector should read executor promotion thresholds from proof-policy JSON"
    );
    assert!(
        collector.contains("--env-output target/proof-observations/thresholds.env"),
        "collector should write a stable threshold env artifact"
    );
    assert!(
        !collector.contains("promotion = json.load(handle)[\"executor\"][\"promotion\"]"),
        "collector should not resolve thresholds with inline Python"
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
        collector.contains("--source-runs-json target/proof-observations/runs.json"),
        "collector should feed the proof-executor source-run window into Rust-owned observation accounting"
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

    assert!(!success, "proof without an execution opt-in should fail");
    assert!(
        stderr.contains("--plan") && stderr.contains("--run-required"),
        "stderr: {stderr}"
    );
}

#[test]
fn proof_plan_refuses_required_execution_mode() {
    let (_stdout, stderr, success) = run_xtask(&[
        "proof",
        "--profile",
        "affected",
        "--base",
        "HEAD",
        "--head",
        "HEAD",
        "--plan",
        "--run-required",
    ]);

    assert!(!success, "proof --plan --run-required should fail");
    assert!(stderr.contains("--run-required"), "stderr: {stderr}");
    assert!(stderr.contains("--plan"), "stderr: {stderr}");
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
fn required_execution_refuses_advisory_executor_mode() {
    let (_stdout, stderr, success) = run_xtask(&[
        "proof",
        "--profile",
        "affected",
        "--base",
        "HEAD",
        "--head",
        "HEAD",
        "--run-required",
        "--executor-mode",
        "execute",
    ]);

    assert!(
        !success,
        "proof --run-required --executor-mode execute should fail"
    );
    assert!(stderr.contains("--run-required"), "stderr: {stderr}");
    assert!(
        stderr.contains("--executor-mode execute"),
        "stderr: {stderr}"
    );
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
fn local_required_execution_requires_explicit_local_opt_in() {
    let temp = tempfile::tempdir().expect("tempdir");
    let summary_path = temp.path().join("proof-run-summary.json");
    let summary_arg = summary_path.to_string_lossy().to_string();

    let (_stdout, stderr, success) = run_xtask(&[
        "proof",
        "--profile",
        "affected",
        "--base",
        "HEAD",
        "--head",
        "HEAD",
        "--run-required",
        "--proof-run-summary",
        &summary_arg,
    ]);

    assert!(
        !success,
        "local required execution should require explicit opt-in"
    );
    assert!(
        stderr.contains("--allow-local-required-execution"),
        "stderr: {stderr}"
    );
    assert!(
        !summary_path.exists(),
        "summary should not be written before guard opt-in"
    );
}

#[test]
fn ci_required_execution_requires_explicit_ci_opt_in() {
    let temp = tempfile::tempdir().expect("tempdir");
    let summary_path = temp.path().join("proof-run-summary.json");
    let summary_arg = summary_path.to_string_lossy().to_string();

    let (_stdout, stderr, success) = run_xtask_with_env(
        &[
            "proof",
            "--profile",
            "affected",
            "--base",
            "HEAD",
            "--head",
            "HEAD",
            "--run-required",
            "--proof-run-summary",
            &summary_arg,
        ],
        &[("CI", "true")],
    );

    assert!(
        !success,
        "CI required execution should require explicit CI opt-in"
    );
    assert!(
        stderr.contains("--allow-ci-required-execution"),
        "stderr: {stderr}"
    );
}

#[test]
fn local_required_execution_can_write_zero_command_summary() {
    let temp = tempfile::tempdir().expect("tempdir");
    let summary_path = temp.path().join("proof-run-summary.json");
    let summary_arg = summary_path.to_string_lossy().to_string();

    let (stdout, stderr, success) = run_xtask(&[
        "proof",
        "--profile",
        "affected",
        "--base",
        "HEAD",
        "--head",
        "HEAD",
        "--run-required",
        "--allow-local-required-execution",
        "--proof-run-summary",
        &summary_arg,
    ]);

    assert!(success, "local required execution failed. stderr: {stderr}");
    let value: serde_json::Value =
        serde_json::from_str(&stdout).expect("proof run should still emit JSON plan");
    assert_eq!(value["schema"], "tokmd.proof_plan.v1");

    let summary = fs::read_to_string(summary_path).expect("summary should be written");
    let summary: serde_json::Value =
        serde_json::from_str(&summary).expect("summary should be valid JSON");
    assert_eq!(summary["schema"], "tokmd.proof_run_summary.v1");
    assert_eq!(summary["status"], "passed");
    assert_eq!(summary["execution_status"], "executed");
    assert_eq!(summary["execution_guard"]["enabled"], true);
    assert_eq!(
        summary["execution_guard"]["reason"],
        "local_explicit_required_opt_in_enabled"
    );
    assert_eq!(summary["counts"]["commands_total"], 0);
    assert_eq!(summary["counts"]["required_planned"], 0);
    assert_eq!(summary["counts"]["advisory_skipped"], 0);
    assert_eq!(summary["counts"]["executed"], 0);
    assert_eq!(summary["counts"]["passed"], 0);
    assert_eq!(summary["counts"]["failed"], 0);
    assert!(summary["entries"].as_array().unwrap().is_empty());

    let (stdout, stderr, success) = run_xtask(&[
        "proof-run-artifacts-check",
        "--proof-run-summary",
        &summary_arg,
    ]);
    assert!(
        success,
        "proof-run-artifacts-check failed. stderr: {stderr}"
    );
    assert!(
        stdout.contains("Proof run artifacts OK: 0 executed required command(s)"),
        "stdout: {stdout}"
    );

    let observation_path = temp
        .path()
        .join("runs")
        .join("25502593070")
        .join("proof-run-observation.json");
    fs::create_dir_all(observation_path.parent().unwrap())
        .expect("proof-run observation parent should be writable");
    let observation_arg = observation_path.to_string_lossy().to_string();
    let (stdout, stderr, success) = run_xtask(&[
        "proof-run-observation",
        "--proof-run-summary",
        &summary_arg,
        "--output",
        &observation_arg,
    ]);
    assert!(success, "proof-run-observation failed. stderr: {stderr}");
    assert!(
        stdout.contains("Proof run observation OK: 0 executed required command(s)"),
        "stdout: {stdout}"
    );
    let observation =
        fs::read_to_string(observation_path).expect("proof-run observation should be written");
    let observation: serde_json::Value =
        serde_json::from_str(&observation).expect("observation should be valid JSON");
    assert_eq!(observation["schema"], "tokmd.proof_run_observation.v1");
    assert_eq!(observation["execution_status"], "executed");
    assert_eq!(observation["counts"]["executed"], 0);
    assert!(observation["scopes"].as_array().unwrap().is_empty());

    let (stdout, stderr, success) = run_xtask(&[
        "proof-run-observations-summary",
        "--observation",
        &observation_arg,
    ]);
    assert!(
        success,
        "proof-run-observations-summary failed. stderr: {stderr}"
    );
    let collection: serde_json::Value =
        serde_json::from_str(&stdout).expect("proof-run collection should be valid JSON");
    assert_eq!(
        collection["schema"],
        "tokmd.proof_run_observation_collection.v1"
    );
    assert_eq!(collection["counts"]["observations"], 1);
    assert_eq!(collection["counts"]["executed"], 0);
    assert!(collection["scopes"].as_array().unwrap().is_empty());
    assert_eq!(collection["profiles"][0]["profile"], "affected");
    assert_eq!(collection["guards"][0]["observations"], 1);
    assert!(
        collection.get("window").is_none(),
        "source-run window should be omitted unless requested: {collection}"
    );

    let source_runs_path = temp.path().join("proof-runs.json");
    fs::write(
        &source_runs_path,
        r#"[{"databaseId":25502593070,"event":"pull_request","headBranch":"main","headSha":"abc123","createdAt":"2026-05-07T14:46:00Z","url":"https://github.com/EffortlessMetrics/tokmd/actions/runs/25502593070"},{"databaseId":25502593071,"event":"pull_request","headBranch":"feature","headSha":"def456","createdAt":"2026-05-07T14:47:00Z","url":"https://github.com/EffortlessMetrics/tokmd/actions/runs/25502593071"}]"#,
    )
    .expect("source proof runs JSON should be writable");
    let source_runs_arg = source_runs_path.to_string_lossy().to_string();
    let (stdout, stderr, success) = run_xtask(&[
        "proof-run-observations-summary",
        "--observation",
        &observation_arg,
        "--source-runs-json",
        &source_runs_arg,
    ]);
    assert!(
        success,
        "proof-run-observations-summary --source-runs-json failed. stderr: {stderr}"
    );
    let collection: serde_json::Value =
        serde_json::from_str(&stdout).expect("source-run collection should be valid JSON");
    let expected_source = source_runs_path.to_string_lossy().replace('\\', "/");
    assert_eq!(collection["window"]["source"], expected_source);
    assert_eq!(collection["window"]["expected_runs"], 2);
    assert_eq!(collection["window"]["observed_runs"], 1);
    assert_eq!(collection["window"]["missing_runs"], 1);
    assert_eq!(collection["window"]["unmatched_observations"], 0);
    assert_eq!(
        collection["window"]["missing"][0]["database_id"],
        serde_json::json!(25502593071_u64)
    );

    let (stdout, stderr, success) = run_xtask(&[
        "proof-run-observations-summary",
        "--observations-dir",
        &temp.path().to_string_lossy(),
    ]);
    assert!(
        success,
        "proof-run-observations-summary --observations-dir failed. stderr: {stderr}"
    );
    let collection: serde_json::Value =
        serde_json::from_str(&stdout).expect("directory proof-run collection should be valid JSON");
    assert_eq!(
        collection["schema"],
        "tokmd.proof_run_observation_collection.v1"
    );
    assert_eq!(collection["counts"]["observations"], 1);

    let collection_path = temp.path().join("proof-run-observation-collection.json");
    let collection_arg = collection_path.to_string_lossy().to_string();
    let summary_md_path = temp.path().join("proof-run-observation-collection.md");
    let summary_md_arg = summary_md_path.to_string_lossy().to_string();
    let (stdout, stderr, success) = run_xtask(&[
        "proof-run-observations-summary",
        "--observation",
        &observation_arg,
        "--output",
        &collection_arg,
        "--summary-md",
        &summary_md_arg,
    ]);
    assert!(
        success,
        "proof-run-observations-summary --summary-md failed. stderr: {stderr}"
    );
    assert!(stdout.contains("wrote"), "stdout: {stdout}");
    let summary_md =
        fs::read_to_string(summary_md_path).expect("proof-run summary markdown should be written");
    assert!(
        summary_md.contains("# Proof Run Observation Collection"),
        "{summary_md}"
    );
    assert!(
        summary_md.contains("| Executed commands | 0 |"),
        "{summary_md}"
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

    let observation_path = temp
        .path()
        .join("runs")
        .join("25502593070")
        .join("proof-executor-observation.json");
    fs::create_dir_all(observation_path.parent().unwrap())
        .expect("observation parent should be writable");
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
    assert!(
        collection.get("window").is_none(),
        "source-run window should be omitted unless requested: {collection}"
    );

    let source_runs_path = temp.path().join("runs.json");
    fs::write(
        &source_runs_path,
        r#"[{"databaseId":25502593070,"event":"pull_request","headBranch":"main","headSha":"abc123","createdAt":"2026-05-07T14:46:00Z","url":"https://github.com/EffortlessMetrics/tokmd/actions/runs/25502593070"},{"databaseId":25502593071,"event":"pull_request","headBranch":"feature","headSha":"def456","createdAt":"2026-05-07T14:47:00Z","url":"https://github.com/EffortlessMetrics/tokmd/actions/runs/25502593071"}]"#,
    )
    .expect("source runs JSON should be writable");
    let source_runs_arg = source_runs_path.to_string_lossy().to_string();
    let (stdout, stderr, success) = run_xtask(&[
        "proof-execution-observations-summary",
        "--observation",
        &observation_arg,
        "--source-runs-json",
        &source_runs_arg,
    ]);
    assert!(
        success,
        "proof-execution-observations-summary --source-runs-json failed. stderr: {stderr}"
    );
    let collection: serde_json::Value =
        serde_json::from_str(&stdout).expect("source-run collection should be valid JSON");
    let expected_source = source_runs_path.to_string_lossy().replace('\\', "/");
    assert_eq!(collection["window"]["source"], expected_source);
    assert_eq!(collection["window"]["expected_runs"], 2);
    assert_eq!(collection["window"]["observed_runs"], 1);
    assert_eq!(collection["window"]["missing_runs"], 1);
    assert_eq!(collection["window"]["unmatched_observations"], 0);
    assert_eq!(
        collection["window"]["missing"][0]["database_id"],
        serde_json::json!(25502593071_u64)
    );

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
