mod common;

use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use std::path::PathBuf;

/// "Docs as tests" - verify that the commands we recommend in README/Recipes actually work.
/// These run against `tests/data` to ensure stability.
fn tokmd() -> Command {
    let mut cmd: Command = cargo_bin_cmd!("tokmd");
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data");
    cmd.current_dir(&fixtures);
    cmd
}

#[test]
fn recipe_default_map() {
    // "tokmd module --top 20"
    tokmd()
        .arg("module")
        .arg("--top")
        .arg("20")
        .assert()
        .success();
}

#[test]
fn recipe_export_map_jsonl() {
    // "tokmd export --min-code 20 --max-rows 300 --redact paths > map.jsonl"
    tokmd()
        .arg("export")
        .arg("--min-code")
        .arg("0") // adjusted for small test data
        .arg("--max-rows")
        .arg("300")
        .arg("--redact")
        .arg("paths")
        .assert()
        .success();
}

#[test]
fn recipe_simple_lang_summary() {
    // "tokmd lang"
    tokmd().arg("lang").assert().success();
}

#[test]
fn recipe_module_summary_markdown() {
    // "tokmd module --format md"
    tokmd()
        .arg("module")
        .arg("--format")
        .arg("md")
        .assert()
        .success();
}

#[test]
fn recipe_export_full_json() {
    // "tokmd export --format json"
    tokmd()
        .arg("export")
        .arg("--format")
        .arg("json")
        .assert()
        .success();
}

#[test]
fn recipe_ci_workflow_snippet() {
    // From README: "tokmd module --format json > tokmd.module.json"
    // We don't redirect here, just check exit code.
    tokmd()
        .arg("module")
        .arg("--format")
        .arg("json")
        .assert()
        .success();
}

#[test]
fn recipe_generate_baseline() {
    // "tokmd baseline --output baseline.json"
    let tmp = tempfile::tempdir().unwrap();
    let baseline_path = tmp.path().join("baseline.json");
    tokmd()
        .arg("baseline")
        .arg("--output")
        .arg(&baseline_path)
        .assert()
        .success();
    assert!(baseline_path.exists());
}

#[test]
fn recipe_handoff_bundle() {
    // "tokmd handoff --out-dir .handoff"
    let tmp = tempfile::tempdir().unwrap();
    let handoff_dir = tmp.path().join(".handoff");
    tokmd()
        .arg("handoff")
        .arg("--out-dir")
        .arg(&handoff_dir)
        .assert()
        .success();
    assert!(handoff_dir.exists());
    assert!(handoff_dir.join("manifest.json").exists());
}

#[cfg(feature = "git")]
#[test]
fn recipe_sensor_json() {
    // "tokmd sensor --format json"
    // Requires a git repo context. We create a temporary one.
    if !common::git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let repo_dir = tmp.path();

    // Initialize git repo
    assert!(common::init_git_repo(repo_dir), "Failed to init git repo");

    // Add some files
    std::fs::write(repo_dir.join("main.rs"), "fn main() {}").unwrap();
    assert!(
        common::git_add_commit(repo_dir, "Initial commit"),
        "Failed to commit"
    );

    let report_path = repo_dir.join("report.json");

    // Run tokmd sensor in this repo
    let mut cmd = cargo_bin_cmd!("tokmd");
    cmd.current_dir(repo_dir)
        .env("TOKMD_GIT_BASE_REF", "HEAD")
        .arg("sensor")
        .arg("--format")
        .arg("json")
        .arg("--output")
        .arg(&report_path)
        .assert()
        .success();

    assert!(report_path.exists());
}

#[test]
fn recipe_gate_with_baseline() {
    // "tokmd gate --baseline baseline.json"
    let tmp = tempfile::tempdir().unwrap();
    let baseline_path = tmp.path().join("baseline.json");

    // First generate a baseline
    tokmd()
        .arg("baseline")
        .arg("--output")
        .arg(&baseline_path)
        .assert()
        .success();

    // Then gate against it (should pass since it's the same state)
    let ratchet_path = tmp.path().join("ratchet.toml");
    std::fs::write(
        &ratchet_path,
        r#"
[[rules]]
pointer = "/complexity/avg_cyclomatic"
max_increase_pct = 10.0
"#,
    )
    .unwrap();

    tokmd()
        .arg("gate")
        .arg("--baseline")
        .arg(&baseline_path)
        .arg("--ratchet-config")
        .arg(&ratchet_path)
        .assert()
        .success();
}
