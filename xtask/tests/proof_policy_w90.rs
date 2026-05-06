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
}

#[test]
fn proof_policy_includes_analysis_and_format_module_scopes() {
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
    assert_eq!(value["scope_count"], 29);
    assert_eq!(value["allowlist_count"], 1);
    assert_eq!(value["fixture_blob_rule_count"], 1);
    assert_eq!(value["dependency_boundary_count"], 1);
}

#[test]
fn xtask_help_mentions_proof_policy() {
    let (stdout, stderr, success) = run_xtask(&["--help"]);

    assert!(success, "xtask --help failed. stderr: {stderr}");
    assert!(stdout.contains("proof-policy"), "stdout: {stdout}");
}
