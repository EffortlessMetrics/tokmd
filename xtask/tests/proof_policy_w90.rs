use std::collections::BTreeSet;
use std::fs;
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

fn repo_policy() -> toml::Value {
    let policy_path = workspace_root().join("ci").join("proof.toml");
    let policy = fs::read_to_string(policy_path).expect("repo proof policy should be readable");
    toml::from_str(&policy).expect("repo proof policy should be valid TOML")
}

#[test]
fn proof_policy_check_accepts_repo_policy() {
    let (stdout, stderr, success) = run_xtask(&["proof-policy", "--check"]);

    assert!(success, "proof-policy --check failed. stderr: {stderr}");
    assert!(stdout.contains("Proof policy OK"), "stdout: {stdout}");
    assert!(stdout.contains("ci/proof.toml"), "stdout: {stdout}");
    assert!(
        stdout.contains("executor coverage/explicit_opt_in/max-dry-run-1"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("pr-default-on"), "stdout: {stdout}");
    assert!(stdout.contains("pr-required-off"), "stdout: {stdout}");
    assert!(stdout.contains("pr-max-commands-2"), "stdout: {stdout}");
    assert!(stdout.contains("pr-codecov-upload-off"), "stdout: {stdout}");
    assert!(
        stdout.contains("promotion-window-last_successful_runs"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("run-limit-100"), "stdout: {stdout}");
    assert!(stdout.contains("min-scopes-4"), "stdout: {stdout}");
    assert!(
        stdout.contains("min-passing-collector-runs-1"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("required-gate-off"), "stdout: {stdout}");
    assert!(
        stdout.contains("proof-run pr-default-on"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("pr-profile-fast"), "stdout: {stdout}");
    assert!(
        stdout.contains("pr-artifact-fast-proof-run"),
        "stdout: {stdout}"
    );
}

#[test]
fn proof_policy_help_mentions_json_output() {
    let (stdout, stderr, success) = run_xtask(&["proof-policy", "--help"]);

    assert!(success, "proof-policy --help failed. stderr: {stderr}");
    assert!(stdout.contains("--check"), "stdout: {stdout}");
    assert!(stdout.contains("--json"), "stdout: {stdout}");
    assert!(stdout.contains("--json-output"), "stdout: {stdout}");
}

#[test]
fn proof_policy_includes_current_product_scopes() {
    let value = repo_policy();
    let scopes = value["scope"]
        .as_array()
        .expect("repo policy should expose scope array");
    let names = scopes
        .iter()
        .filter_map(|scope| scope["name"].as_str())
        .collect::<BTreeSet<_>>();

    for expected in [
        "analysis_api_surface",
        "analysis_complexity",
        "analysis_derived",
        "analysis_receipt_types",
        "analysis_types_api_surface",
        "analysis_types_baseline",
        "analysis_types_complexity",
        "analysis_types_derived",
        "analysis_types_effort",
        "analysis_types_source",
        "analysis_types_topics",
        "doc_artifacts_policy",
        "format_analysis_rendering",
        "format_core_outputs",
        "format_redaction_scan_args",
        "fuzz_harnesses",
        "jules_workspace",
        "model_scan_path_normalization",
        "no_panic_policy",
        "project_readme",
        "project_truth_docs",
        "tokmd_cli",
        "workspace_dependency_graph",
    ] {
        assert!(
            names.contains(expected),
            "missing expected proof scope {expected}"
        );
    }

    let analysis_rendering = scopes
        .iter()
        .find(|scope| scope["name"].as_str() == Some("format_analysis_rendering"))
        .expect("format_analysis_rendering scope should exist");
    let proof = analysis_rendering["proof"]
        .as_array()
        .expect("format_analysis_rendering should expose proof commands")
        .iter()
        .filter_map(toml::Value::as_str)
        .collect::<BTreeSet<_>>();

    assert!(proof.contains("cargo test -p tokmd-format --test analysis_format --verbose"));
    assert!(proof.contains("cargo test -p tokmd-format --test analysis_html --verbose"));

    let no_panic = scopes
        .iter()
        .find(|scope| scope["name"].as_str() == Some("no_panic_policy"))
        .expect("no_panic_policy scope should exist");
    let no_panic_proof = no_panic["proof"]
        .as_array()
        .expect("no_panic_policy should expose proof commands")
        .iter()
        .filter_map(toml::Value::as_str)
        .collect::<BTreeSet<_>>();

    assert!(no_panic_proof.contains("cargo xtask check-no-panic-family --strict"));
    assert!(no_panic_proof.contains("cargo test -p xtask no_panic --verbose"));

    let proof_control_plane = scopes
        .iter()
        .find(|scope| scope["name"].as_str() == Some("proof_control_plane"))
        .expect("proof_control_plane scope should exist");
    let proof_control_paths = proof_control_plane["paths"]
        .as_array()
        .expect("proof_control_plane should expose path globs")
        .iter()
        .filter_map(toml::Value::as_str)
        .collect::<BTreeSet<_>>();
    let proof_control_proof = proof_control_plane["proof"]
        .as_array()
        .expect("proof_control_plane should expose proof commands")
        .iter()
        .filter_map(toml::Value::as_str)
        .collect::<BTreeSet<_>>();

    assert!(proof_control_paths.contains(".github/workflows/mutants.yml"));
    assert!(proof_control_paths.contains(".github/workflows/nix-full.yml"));
    assert!(proof_control_paths.contains(".github/workflows/nix-macos.yml"));
    assert!(proof_control_paths.contains("xtask/src/tasks/workspace.rs"));
    assert!(proof_control_proof.contains("cargo xtask ci-lane-whitelist"));

    let user_guides = scopes
        .iter()
        .find(|scope| scope["name"].as_str() == Some("user_guides"))
        .expect("user_guides scope should exist");
    let user_guide_paths = user_guides["paths"]
        .as_array()
        .expect("user_guides should expose path globs")
        .iter()
        .filter_map(toml::Value::as_str)
        .collect::<BTreeSet<_>>();

    assert!(user_guide_paths.contains("docs/analyze/**"));
    assert!(user_guide_paths.contains("docs/integrations/**"));
    assert!(user_guide_paths.contains("docs/packet-workflows.md"));
    assert!(user_guide_paths.contains("docs/recipes.md"));
    assert!(user_guide_paths.contains("docs/user-paths.md"));

    let project_truth_docs = scopes
        .iter()
        .find(|scope| scope["name"].as_str() == Some("project_truth_docs"))
        .expect("project_truth_docs scope should exist");
    let project_truth_paths = project_truth_docs["paths"]
        .as_array()
        .expect("project_truth_docs should expose path globs")
        .iter()
        .filter_map(toml::Value::as_str)
        .collect::<BTreeSet<_>>();

    assert!(project_truth_paths.contains("CONTRIBUTING.md"));
    assert!(project_truth_paths.contains("docs/agent-workflows/**"));
    assert!(project_truth_paths.contains("docs/contributor-guide.md"));
    assert!(project_truth_paths.contains("docs/debugging.md"));
    assert!(project_truth_paths.contains("docs/ROADMAP.md"));

    let doc_artifacts_policy = scopes
        .iter()
        .find(|scope| scope["name"].as_str() == Some("doc_artifacts_policy"))
        .expect("doc_artifacts_policy scope should exist");
    let doc_artifacts_paths = doc_artifacts_policy["paths"]
        .as_array()
        .expect("doc_artifacts_policy should expose path globs")
        .iter()
        .filter_map(toml::Value::as_str)
        .collect::<BTreeSet<_>>();
    let doc_artifacts_proof = doc_artifacts_policy["proof"]
        .as_array()
        .expect("doc_artifacts_policy should expose proof commands")
        .iter()
        .filter_map(toml::Value::as_str)
        .collect::<BTreeSet<_>>();

    for expected in [
        ".jules/goals/**",
        "docs/adr/**",
        "docs/agent-workflows/**",
        "docs/plans/**",
        "docs/proposals/**",
        "docs/source-of-truth.md",
        "docs/specs/**",
        "docs/templates/**",
        "policy/doc-artifacts.toml",
        "xtask/src/tasks/doc_artifacts.rs",
    ] {
        assert!(
            doc_artifacts_paths.contains(expected),
            "doc_artifacts_policy missing path {expected}"
        );
    }
    assert!(doc_artifacts_proof.contains("cargo xtask doc-artifacts --check"));
    assert!(doc_artifacts_proof.contains("cargo xtask docs --check"));
    assert!(doc_artifacts_proof.contains("cargo xtask proof-policy --check"));
    assert!(doc_artifacts_proof.contains("cargo test -p xtask doc_artifacts --verbose"));

    let dependency_graph = scopes
        .iter()
        .find(|scope| scope["name"].as_str() == Some("workspace_dependency_graph"))
        .expect("workspace_dependency_graph scope should exist");
    let dependency_paths = dependency_graph["paths"]
        .as_array()
        .expect("workspace_dependency_graph should expose path globs")
        .iter()
        .filter_map(toml::Value::as_str)
        .collect::<BTreeSet<_>>();
    let dependency_proof = dependency_graph["proof"]
        .as_array()
        .expect("workspace_dependency_graph should expose proof commands")
        .iter()
        .filter_map(toml::Value::as_str)
        .collect::<BTreeSet<_>>();

    assert!(dependency_paths.contains("Cargo.lock"));
    assert!(dependency_paths.contains("Cargo.toml"));
    assert!(dependency_proof.contains("cargo deny --all-features check"));
    assert!(dependency_proof.contains("cargo xtask boundaries-check"));
    assert!(dependency_proof.contains("cargo xtask publish-surface --json"));

    let fuzz_harnesses = scopes
        .iter()
        .find(|scope| scope["name"].as_str() == Some("fuzz_harnesses"))
        .expect("fuzz_harnesses scope should exist");
    let fuzz_paths = fuzz_harnesses["paths"]
        .as_array()
        .expect("fuzz_harnesses should expose path globs")
        .iter()
        .filter_map(toml::Value::as_str)
        .collect::<BTreeSet<_>>();
    let fuzz_proof = fuzz_harnesses["proof"]
        .as_array()
        .expect("fuzz_harnesses should expose proof commands")
        .iter()
        .filter_map(toml::Value::as_str)
        .collect::<BTreeSet<_>>();

    assert!(fuzz_paths.contains(".github/workflows/fuzz.yml"));
    assert!(fuzz_paths.contains("fuzz/Cargo.toml"));
    assert!(fuzz_paths.contains("fuzz/corpus/**"));
    assert!(fuzz_paths.contains("fuzz/dict/**"));
    assert!(fuzz_paths.contains("fuzz/fuzz_targets/**"));
    assert!(fuzz_paths.contains("xtask/tests/fuzz_dict_w100.rs"));
    assert!(fuzz_proof.contains("cargo +nightly fuzz list"));
    assert!(
        fuzz_proof.contains(
            "cargo test -p xtask fuzz_dictionaries_do_not_define_empty_tokens -- --exact"
        )
    );

    let tokmd_cli = scopes
        .iter()
        .find(|scope| scope["name"].as_str() == Some("tokmd_cli"))
        .expect("tokmd_cli scope should exist");
    let tokmd_cli_paths = tokmd_cli["paths"]
        .as_array()
        .expect("tokmd_cli should expose path globs")
        .iter()
        .filter_map(toml::Value::as_str)
        .collect::<BTreeSet<_>>();

    assert!(tokmd_cli_paths.contains("crates/tokmd/tests/cli_*.rs"));
    assert!(tokmd_cli_paths.contains("crates/tokmd/tests/evidence_packet_*.rs"));
    assert!(tokmd_cli_paths.contains("crates/tokmd/src/analysis_utils.rs"));
    assert!(tokmd_cli_paths.contains("crates/tokmd/tests/snapshots/cli_*.snap"));
    assert!(tokmd_cli_paths.contains("crates/tokmd/tests/error_handling.rs"));
    assert!(tokmd_cli_paths.contains("crates/tokmd/tests/error_handling_w70.rs"));

    let ci_economics_docs = scopes
        .iter()
        .find(|scope| scope["name"].as_str() == Some("ci_economics_docs"))
        .expect("ci_economics_docs scope should exist");
    let ci_economics_paths = ci_economics_docs["paths"]
        .as_array()
        .expect("ci_economics_docs should expose path globs")
        .iter()
        .filter_map(toml::Value::as_str)
        .collect::<BTreeSet<_>>();

    assert!(ci_economics_paths.contains(".github/workflows/sync-labels.yml"));

    let release_metadata = scopes
        .iter()
        .find(|scope| scope["name"].as_str() == Some("release_metadata"))
        .expect("release_metadata scope should exist");
    let release_paths = release_metadata["paths"]
        .as_array()
        .expect("release_metadata should expose path globs")
        .iter()
        .filter_map(toml::Value::as_str)
        .collect::<BTreeSet<_>>();
    let release_proof = release_metadata["proof"]
        .as_array()
        .expect("release_metadata should expose proof commands")
        .iter()
        .filter_map(toml::Value::as_str)
        .collect::<BTreeSet<_>>();

    assert!(release_paths.contains(".github/workflows/release.yml"));
    assert!(release_proof.contains("cargo xtask publish-surface --json --verify-publish"));
}

#[test]
fn proof_policy_declares_coverage_executor_promotion_rule() {
    let value = repo_policy();
    let executor = value["executor"]
        .as_table()
        .expect("repo policy should expose executor policy");

    assert_eq!(executor["family"].as_str(), Some("coverage"));
    assert_eq!(executor["ci_execution"].as_str(), Some("explicit_opt_in"));
    assert_eq!(executor["max_dry_run_commands"].as_integer(), Some(1));

    let pr = executor["pr"]
        .as_table()
        .expect("repo policy should expose executor PR defaults");
    assert_eq!(pr["default_enabled"].as_bool(), Some(true));
    assert_eq!(pr["required"].as_bool(), Some(false));
    assert_eq!(pr["max_commands"].as_integer(), Some(2));
    assert_eq!(pr["codecov_upload"].as_bool(), Some(false));

    let promotion = executor["promotion"]
        .as_table()
        .expect("repo policy should expose executor promotion criteria");
    assert_eq!(promotion["window"].as_str(), Some("last_successful_runs"));
    assert_eq!(promotion["run_limit"].as_integer(), Some(100));
    assert_eq!(promotion["min_observations"].as_integer(), Some(1));
    assert_eq!(promotion["min_executed"].as_integer(), Some(4));
    assert_eq!(promotion["min_scopes"].as_integer(), Some(4));
    assert_eq!(promotion["min_artifacts"].as_integer(), Some(4));
    assert_eq!(
        promotion["min_passing_collector_runs"].as_integer(),
        Some(1)
    );
    assert_eq!(promotion["required_gate"].as_bool(), Some(false));
    assert_eq!(promotion["default_codecov_upload"].as_bool(), Some(false));
}

#[test]
fn proof_policy_declares_advisory_fast_proof_run_rule() {
    let value = repo_policy();
    let proof_run = value["proof_run"]
        .as_table()
        .expect("repo policy should expose proof_run policy");
    let pr = proof_run["pr"]
        .as_table()
        .expect("repo policy should expose proof_run PR defaults");

    assert_eq!(pr["default_enabled"].as_bool(), Some(true));
    assert_eq!(pr["profile"].as_str(), Some("fast"));
    assert_eq!(pr["required"].as_bool(), Some(false));
    assert_eq!(pr["artifact_name"].as_str(), Some("fast-proof-run"));
}

#[test]
fn proof_policy_json_reports_current_schema() {
    let (stdout, stderr, success) = run_xtask(&["proof-policy", "--json"]);

    assert!(success, "proof-policy --json failed. stderr: {stderr}");
    let value: serde_json::Value =
        serde_json::from_str(&stdout).expect("proof-policy --json should emit JSON");

    assert_eq!(value["ok"], true);
    assert_eq!(value["schema"], "tokmd.proof_policy.v1");
    let scope_count = value["scope_count"]
        .as_u64()
        .expect("scope_count should be a JSON number");
    assert!(scope_count > 0, "scope_count should report active scopes");
    assert_eq!(value["allowlist_count"], 1);
    assert_eq!(value["fixture_blob_rule_count"], 1);
    assert_eq!(value["dependency_boundary_count"], 1);
    assert_eq!(value["executor"]["family"], "coverage");
    assert_eq!(value["executor"]["ci_execution"], "explicit_opt_in");
    assert_eq!(value["executor"]["max_dry_run_commands"], 1);
    assert_eq!(value["executor"]["pr"]["default_enabled"], true);
    assert_eq!(value["executor"]["pr"]["required"], false);
    assert_eq!(value["executor"]["pr"]["max_commands"], 2);
    assert_eq!(value["executor"]["pr"]["codecov_upload"], false);
    assert_eq!(
        value["executor"]["promotion"]["window"],
        "last_successful_runs"
    );
    assert_eq!(value["executor"]["promotion"]["run_limit"], 100);
    assert_eq!(value["executor"]["promotion"]["min_observations"], 1);
    assert_eq!(value["executor"]["promotion"]["min_executed"], 4);
    assert_eq!(value["executor"]["promotion"]["min_scopes"], 4);
    assert_eq!(value["executor"]["promotion"]["min_artifacts"], 4);
    assert_eq!(
        value["executor"]["promotion"]["min_passing_collector_runs"],
        1
    );
    assert_eq!(value["executor"]["promotion"]["required_gate"], false);
    assert_eq!(
        value["executor"]["promotion"]["default_codecov_upload"],
        false
    );
    assert_eq!(value["proof_run"]["pr"]["default_enabled"], true);
    assert_eq!(value["proof_run"]["pr"]["profile"], "fast");
    assert_eq!(value["proof_run"]["pr"]["required"], false);
    assert_eq!(value["proof_run"]["pr"]["artifact_name"], "fast-proof-run");
}

#[test]
fn proof_policy_json_output_writes_report_artifact() {
    let root = workspace_root();
    let path = root
        .join("target")
        .join("proof-policy-w90")
        .join("proof-policy.json");
    if path.exists() {
        fs::remove_file(&path).expect("stale proof-policy fixture should be removable");
    }

    let path_arg = path.to_string_lossy().to_string();
    let (stdout, stderr, success) =
        run_xtask(&["proof-policy", "--json", "--json-output", &path_arg]);

    assert!(
        success,
        "proof-policy --json-output failed. stderr: {stderr}"
    );
    assert!(stdout.contains("\"schema\": \"tokmd.proof_policy.v1\""));
    assert!(path.exists(), "proof-policy artifact should be written");

    let written = fs::read_to_string(&path).expect("proof-policy artifact should be readable");
    let stdout_json: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout proof-policy report should be JSON");
    let written_json: serde_json::Value =
        serde_json::from_str(&written).expect("written proof-policy report should be JSON");

    assert_eq!(written_json["schema"], "tokmd.proof_policy.v1");
    assert_eq!(written_json, stdout_json);
}

#[test]
fn xtask_help_mentions_proof_policy() {
    let (stdout, stderr, success) = run_xtask(&["--help"]);

    assert!(success, "xtask --help failed. stderr: {stderr}");
    assert!(stdout.contains("proof-policy"), "stdout: {stdout}");
}
