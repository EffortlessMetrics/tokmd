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
        "format_analysis_rendering",
        "format_core_outputs",
        "format_redaction_scan_args",
        "fuzz_harnesses",
        "jules_workspace",
        "model_scan_path_normalization",
        "no_panic_policy",
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

    assert!(no_panic_proof.contains("cargo xtask check-no-panic-family"));
    assert!(no_panic_proof.contains("cargo test -p xtask no_panic --verbose"));

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

    assert!(fuzz_paths.contains("fuzz/Cargo.toml"));
    assert!(fuzz_paths.contains("fuzz/corpus/**"));
    assert!(fuzz_paths.contains("fuzz/dict/**"));
    assert!(fuzz_paths.contains("fuzz/fuzz_targets/**"));
    assert!(fuzz_proof.contains("cargo +nightly fuzz list"));

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
    assert!(tokmd_cli_paths.contains("crates/tokmd/tests/error_handling_w70.rs"));
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
}

#[test]
fn proof_policy_json_reports_current_schema() {
    let (stdout, stderr, success) = run_xtask(&["proof-policy", "--json"]);

    assert!(success, "proof-policy --json failed. stderr: {stderr}");
    let value: serde_json::Value =
        serde_json::from_str(&stdout).expect("proof-policy --json should emit JSON");

    assert_eq!(value["ok"], true);
    assert_eq!(value["schema"], "tokmd.proof_policy.v1");
    assert_eq!(value["scope_count"], 35);
    assert_eq!(value["allowlist_count"], 1);
    assert_eq!(value["fixture_blob_rule_count"], 1);
    assert_eq!(value["dependency_boundary_count"], 1);
    assert_eq!(value["executor"]["family"], "coverage");
    assert_eq!(value["executor"]["ci_execution"], "explicit_opt_in");
    assert_eq!(value["executor"]["max_dry_run_commands"], 1);
}

#[test]
fn xtask_help_mentions_proof_policy() {
    let (stdout, stderr, success) = run_xtask(&["--help"]);

    assert!(success, "xtask --help failed. stderr: {stderr}");
    assert!(stdout.contains("proof-policy"), "stdout: {stdout}");
}
