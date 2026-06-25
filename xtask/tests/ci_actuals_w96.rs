use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
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
fn ci_actuals_help_mentions_receipt_inputs() {
    let (stdout, stderr, success) = run_xtask(&["ci-actuals", "--help"]);

    assert!(success, "ci-actuals --help failed. stderr: {stderr}");
    assert!(stdout.contains("--needs"), "stdout: {stdout}");
    assert!(stdout.contains("--timings"), "stdout: {stdout}");
    assert!(stdout.contains("--output"), "stdout: {stdout}");
}

#[test]
fn ci_actuals_writes_schema_stable_receipt() {
    let temp = tempfile::tempdir().expect("tempdir");
    let needs = temp.path().join("needs.json");
    let timings = temp.path().join("timings.json");
    let output = temp.path().join("ci-actuals.json");
    let summary = temp.path().join("step-summary.md");
    fs::write(
        &needs,
        r#"{
          "build": {"result": "failure", "outputs": {"actual_lem": "13"}},
          "docs-check": {"result": "success", "outputs": {"docs": "ok", "route_target": "hosted", "estimated_lem": "3", "estimate_source": "static"}},
          "mutation": {"result": "skipped", "outputs": {"skip_reason": "not_selected_by_policy"}}
        }"#,
    )
    .expect("needs json");
    fs::write(
        &timings,
        r#"{
          "build": {"duration_seconds": 180.0, "queue_seconds": 12.0},
          "docs-check": {"duration_seconds": 75.0, "actual_lem": 1.5, "runner": "ubuntu-latest", "cache_hit": true}
        }"#,
    )
    .expect("timings json");

    let (stdout, stderr, success) = run_xtask(&[
        "ci-actuals",
        "--needs",
        needs.to_str().expect("needs path"),
        "--timings",
        timings.to_str().expect("timings path"),
        "--output",
        output.to_str().expect("output path"),
        "--github-summary",
        summary.to_str().expect("summary path"),
        "--sha",
        "abc123",
    ]);

    assert!(success, "ci-actuals failed. stderr: {stderr}");
    assert!(
        stdout.contains("CI actuals receipt written"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("CI actuals summary appended"),
        "stdout: {stdout}"
    );
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).expect("receipt body"))
            .expect("receipt json");
    assert_eq!(value["schema"], "tokmd.ci_actuals.v3");
    assert_eq!(value["schema_version"], 3);
    assert_eq!(value["sha"], "abc123");
    assert_eq!(value["status"]["job_count"], 3);
    assert_eq!(value["status"]["timed_job_count"], 2);
    assert_eq!(value["status"]["missing_timing"][0], "mutation");
    let jobs = value["jobs"].as_array().expect("jobs array");
    let build = jobs
        .iter()
        .find(|job| job["name"] == "build")
        .expect("build job");
    assert_eq!(build["lane_id"], "build_test_linux");
    assert_eq!(build["selected"].as_bool(), Some(true));
    assert_eq!(build["result"], "failure");
    assert_eq!(build["queue_seconds"].as_f64(), Some(12.0));
    assert_eq!(build["actual_lem"].as_f64(), Some(13.0));

    let docs = jobs
        .iter()
        .find(|job| job["name"] == "docs-check")
        .expect("docs-check job");
    assert_eq!(docs["summary_key"], "docs-check");
    assert_eq!(docs["lane_id"], "docs_check");
    assert_eq!(docs["aliases"][0], "docs-check");
    assert_eq!(docs["aliases"][1], "docs_check");
    assert_eq!(docs["selected"].as_bool(), Some(true));
    assert_eq!(docs["route_target"], "hosted");
    assert_eq!(docs["estimated_lem"].as_f64(), Some(3.0));
    assert_eq!(docs["actual_lem"].as_f64(), Some(1.5));
    assert_eq!(docs["estimate_source"], "static");
    assert_eq!(docs["timing_status"], "measured");

    let mutation = jobs
        .iter()
        .find(|job| job["name"] == "mutation")
        .expect("mutation job");
    assert_eq!(mutation["lane_id"], "mutation_required");
    assert_eq!(mutation["selected"].as_bool(), Some(false));
    assert_eq!(mutation["skip_reason"], "not_selected_by_policy");
    assert_eq!(mutation["timing_status"], "missing");

    let summary_body = fs::read_to_string(summary).expect("summary body");
    assert!(summary_body.contains("## CI Actuals (advisory)"));
    assert!(
        summary_body.contains(
            "| `docs_check` | `success` | yes | 3 | 1.5 | 75s | unknown | hosted | no (`static`) |"
        ),
        "{summary_body}"
    );
    assert!(
        summary_body
            .contains("| `mutation` skip reason |  |  |  |  |  |  |  | not_selected_by_policy |"),
        "{summary_body}"
    );
}

#[test]
fn ci_actuals_uploads_receipt_before_advisory_summary() {
    let workflow = fs::read_to_string(workspace_root().join(".github/workflows/ci.yml"))
        .expect("read ci workflow");
    let ci_actuals_idx = workflow
        .find("  ci-actuals:")
        .expect("CI actuals job should exist");
    let ci_actuals = &workflow[ci_actuals_idx..];

    let checkout_idx = ci_actuals
        .find("actions/checkout@v6.0.2")
        .expect("CI actuals checkout step");
    let toolchain_idx = ci_actuals
        .find("dtolnay/rust-toolchain@stable")
        .expect("CI actuals toolchain step");
    let cache_idx = ci_actuals
        .find("Swatinem/rust-cache@v2")
        .expect("CI actuals cache step");
    let timings_idx = ci_actuals
        .find("Generate CI timings sidecar")
        .expect("generate CI timings sidecar step");
    let generate_idx = ci_actuals
        .find("Generate CI actuals receipt")
        .expect("generate CI actuals receipt step");
    let upload_idx = ci_actuals
        .find("Upload CI actuals receipt")
        .expect("upload CI actuals receipt step");
    let summary_idx = ci_actuals
        .find("Publish advisory lane summary")
        .expect("publish advisory lane summary step");

    assert!(
        checkout_idx < toolchain_idx && toolchain_idx < cache_idx && cache_idx < timings_idx,
        "setup should happen before timing collection"
    );
    assert!(
        timings_idx < generate_idx,
        "timing collection should happen before receipt generation"
    );
    assert!(
        generate_idx < upload_idx,
        "upload must follow receipt generation"
    );
    assert!(
        upload_idx < summary_idx,
        "receipt upload must happen before advisory summary"
    );
    let setup_block = &ci_actuals[checkout_idx..timings_idx];
    let timings_block = &ci_actuals[timings_idx..generate_idx];
    let generate_block = &ci_actuals[generate_idx..upload_idx];
    let upload_block = &ci_actuals[upload_idx..summary_idx];
    let summary_block = &ci_actuals[summary_idx..];

    assert_eq!(
        setup_block.matches("continue-on-error: true").count(),
        3,
        "checkout, toolchain, and cache should be best-effort telemetry setup"
    );
    assert!(
        ci_actuals.contains("permissions:\n      contents: read\n      actions: read"),
        "CI actuals job should request only read permissions needed for checkout and job timing lookup"
    );
    assert!(ci_actuals.contains("continue-on-error: true"));
    assert!(timings_block.contains("if: always()"));
    assert!(timings_block.contains("continue-on-error: true"));
    assert!(
        timings_block.contains("GITHUB_TOKEN: ${{ github.token }}"),
        "timing lookup should use the ephemeral workflow token"
    );
    assert!(
        timings_block.contains("/attempts/{run_attempt}/jobs"),
        "timing lookup should read the current run attempt"
    );
    assert!(
        timings_block.contains("if job.get(\"conclusion\") != \"success\":"),
        "timing sidecar should only collect successful job durations"
    );
    assert!(
        timings_block.contains("\"Tokmd Rust Result\": \"tokmd-rust-result\"")
            && timings_block.contains("\"Docs Check\": \"docs-check\""),
        "timing lookup should map hosted job display names back to needs keys"
    );
    assert!(
        timings_block.contains("target/ci/timings.json"),
        "workflow should persist the timing sidecar"
    );
    assert!(
        timings_block.contains("record = {\"duration_seconds\": duration_seconds}")
            && timings_block.contains("record[\"runner\"] = runner"),
        "timing sidecar should preserve duration and runner label observations"
    );
    assert!(generate_block.contains("if: always()"));
    assert!(generate_block.contains("continue-on-error: true"));
    assert!(
        generate_block.contains("printf '%s\\n' \"${NEEDS_JSON}\" > target/ci/needs.json"),
        "workflow should persist the raw needs payload"
    );
    assert!(
        generate_block.contains("cargo xtask ci-actuals"),
        "workflow should call the ci-actuals command"
    );
    assert!(
        generate_block.contains("timing_args=()")
            && generate_block.contains("if [ -s target/ci/timings.json ]; then")
            && generate_block.contains("timing_args=(--timings target/ci/timings.json)")
            && generate_block.contains("\"${timing_args[@]}\""),
        "workflow should use the timing sidecar only when it exists"
    );
    assert!(
        generate_block.contains("--output target/ci/ci-actuals.json"),
        "workflow should write the stable ci-actuals path"
    );
    assert!(
        generate_block.contains("--github-summary \"$GITHUB_STEP_SUMMARY\""),
        "workflow should publish a human-readable CI actuals table"
    );
    assert!(upload_block.contains("if: always()"));
    assert!(upload_block.contains("continue-on-error: true"));
    assert!(
        upload_block.contains("name: ci-actuals"),
        "workflow should upload a named ci-actuals artifact"
    );
    assert!(
        upload_block.contains("target/ci/needs.json")
            && upload_block.contains("target/ci/timings.json")
            && upload_block.contains("target/ci/ci-actuals.json"),
        "workflow should upload CI actuals inputs and receipt"
    );
    assert!(
        upload_block.contains("if-no-files-found: warn"),
        "receipt upload should tolerate missing timing sidecars"
    );
    assert!(
        summary_block.contains("continue-on-error: true"),
        "advisory lane summary must not block merge"
    );
    assert!(
        summary_block.contains("Branch protection uses `Tokmd Rust Result` only"),
        "advisory summary should document the single required check"
    );
    assert!(
        !summary_block.contains("sys.exit(1)"),
        "advisory summary must not fail the job"
    );
}

#[test]
fn pr_plan_downloads_ci_actuals_cache_before_planning() {
    let workflow = fs::read_to_string(workspace_root().join(".github/workflows/pr-plan.yml"))
        .expect("read PR Plan workflow");

    assert!(
        workflow.contains("permissions:\n  actions: read\n  contents: read\n  pull-requests: read"),
        "PR Plan should request only read permissions needed to inspect prior CI actual artifacts"
    );

    let fetch_idx = workflow
        .find("Fetch base ref")
        .expect("fetch base step should exist");
    let download_idx = workflow
        .find("Download recent CI actuals cache")
        .expect("actuals cache download step should exist");
    let plan_idx = workflow
        .find("Generate PR plan")
        .expect("generate PR plan step should exist");
    let verify_idx = workflow
        .find("Verify PR plan receipts")
        .expect("verify PR plan receipts step should exist");

    assert!(
        fetch_idx < download_idx && download_idx < plan_idx && plan_idx < verify_idx,
        "PR Plan should fetch refs, download actuals cache, plan, then verify receipts"
    );

    let download_block = &workflow[download_idx..plan_idx];
    let plan_block = &workflow[plan_idx..verify_idx];

    assert!(download_block.contains("continue-on-error: true"));
    assert!(
        download_block.contains("GH_TOKEN: ${{ github.token }}"),
        "actuals cache download should use the ephemeral workflow token"
    );
    assert!(
        download_block.contains("target/ci/actuals-cache"),
        "actuals cache should be local to target/ci"
    );
    assert!(
        download_block
            .contains("gh run list --workflow CI --branch main --status success --limit 5")
            && download_block.contains("gh run download \"${run_id}\" --name ci-actuals"),
        "PR Plan should read recent successful main CI actuals artifacts"
    );
    assert!(
        download_block.contains("Downloaded ${count} CI actuals receipt(s)"),
        "workflow summary should expose how many actuals receipts were available"
    );
    assert!(
        download_block.contains("## CI actuals cache (advisory)")
            && download_block.contains("Downloaded ${count} recent \\`ci-actuals\\` receipt(s)")
            && download_block.contains("falls back to static \\`base_lem\\` estimates")
            && download_block.contains(">> \"$GITHUB_STEP_SUMMARY\""),
        "PR Plan step summary should make actuals-cache availability visible"
    );

    assert!(
        plan_block.contains("actuals_args=()")
            && plan_block.contains("if [ -d target/ci/actuals-cache ] && find target/ci/actuals-cache -name '*.json' -print -quit | grep -q .; then")
            && plan_block.contains("actuals_args=(--actuals-dir target/ci/actuals-cache)")
            && plan_block.contains("\"${actuals_args[@]}\""),
        "ci-plan should receive --actuals-dir only when cached receipts exist"
    );
}

#[test]
fn ci_actuals_docs_explain_receipt_status_and_timing_semantics() {
    let root = workspace_root();
    let artifacts = fs::read_to_string(root.join("docs/artifacts.md"))
        .expect("artifact glossary should be readable");
    let ci_actuals = fs::read_to_string(root.join("docs/ci/ci-actuals.md"))
        .expect("CI actuals docs should be readable");
    let pr_plan = fs::read_to_string(root.join("docs/ci/pr-plan.md"))
        .expect("PR Plan docs should be readable");
    let learned_estimates = fs::read_to_string(root.join("docs/ci/learned-estimates.md"))
        .expect("learned estimates docs should be readable");
    let budget_guard = fs::read_to_string(root.join("docs/ci/budget-guard.md"))
        .expect("budget guard docs should be readable");

    assert!(
        artifacts.contains("Read CI actuals")
            && artifacts.contains("target/ci/ci-actuals.json")
            && artifacts.contains("target/ci/needs.json")
            && artifacts.contains("target/ci/timings.json"),
        "artifact glossary should name the CI actuals receipt and source inputs as the first reading surface"
    );
    assert!(
        artifacts.contains("canonical lane id")
            && artifacts.contains("selected/skipped status")
            && artifacts.contains("optional skip reason"),
        "artifact glossary should summarize stabilized CI actuals fields"
    );

    for text in [
        "`summary_key`",
        "`lane_id`",
        "`aliases`",
        "`selected`",
        "`skip_reason`",
        "`route_target`",
        "`estimated_lem`",
        "`actual_lem`",
        "`queue_seconds`",
        "`estimate_source`",
        "This is an execution skip reason, not proof-policy authorization",
    ] {
        assert!(
            ci_actuals.contains(text),
            "CI actuals docs should explain stabilized field `{text}`"
        );
    }

    for text in [
        "`status.ok` means the receipt was generated successfully",
        "every CI job passed",
        "`jobs[].result` is the per-required-job result",
        "`jobs[].summary_key`",
        "`jobs[].lane_id`",
        "`jobs[].aliases`",
        "`jobs[].selected`",
        "`skip_reason`",
        "`route_target`",
        "`estimated_lem`",
        "`actual_lem`",
        "`queue_seconds`",
        "`estimate_source`",
        "`status.missing_timing` means timing telemetry was unavailable",
        "It is not a zero-second duration",
        "`duration_seconds`",
        "`duration_minutes`",
        "`runner`",
        "`cache_hit`",
        "They do not promote learned estimates",
        "`status.unused_timing` records timing sidecar entries",
        "downloads recent successful `CI` run",
        "`ci-actuals` artifacts from `main`",
        "falls back to static `base_lem` values",
    ] {
        assert!(
            pr_plan.contains(text),
            "PR Plan docs should explain CI actuals reader guidance `{text}`"
        );
    }

    for text in [
        "best-effort cache of recent successful",
        "passes `--actuals-dir target/ci/actuals-cache`",
        "When no receipt is available",
        "`base_lem` estimates remain the fallback",
    ] {
        assert!(
            learned_estimates.contains(text),
            "learned-estimates docs should explain hosted actuals-cache behavior `{text}`"
        );
    }

    for text in [
        "best-effort cache of recent successful `main` CI",
        "When no cache receipt is available",
        "the static floor is",
        "the estimate",
    ] {
        assert!(
            budget_guard.contains(text),
            "budget guard docs should explain learned/static estimate fallback `{text}`"
        );
    }

    for stale in [
        "hosted PR Plan workflow currently uses static estimates",
        "hosted PR Plan workflow currently uses static `base_lem`",
        "must wire that directory in before hosted PRs use learned estimates",
        "unless a future workflow change provides",
    ] {
        assert!(
            !pr_plan.contains(stale)
                && !learned_estimates.contains(stale)
                && !budget_guard.contains(stale),
            "docs should not retain stale hosted-static wording: {stale}"
        );
    }
}
