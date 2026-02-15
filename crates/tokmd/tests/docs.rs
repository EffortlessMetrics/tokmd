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
    // Setup a fresh git repo to run sensor against.
    // This ensures reliability in CI environments (Nix, Tarballs) where .git might be missing.
    let tmp = tempfile::tempdir().unwrap();
    let repo_path = tmp.path();

    // Initialize git repo
    std::process::Command::new("git")
        .arg("init")
        .current_dir(repo_path)
        .output()
        .expect("failed to init git");

    std::process::Command::new("git")
        .args(&["config", "user.email", "you@example.com"])
        .current_dir(repo_path)
        .output()
        .expect("failed to config git email");

    std::process::Command::new("git")
        .args(&["config", "user.name", "Your Name"])
        .current_dir(repo_path)
        .output()
        .expect("failed to config git name");

    // Add a file
    let file_path = repo_path.join("test.rs");
    std::fs::write(&file_path, "fn main() {}").unwrap();

    std::process::Command::new("git")
        .args(&["add", "."])
        .current_dir(repo_path)
        .output()
        .expect("failed to git add");

    std::process::Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .output()
        .expect("failed to git commit");

    let report_path = repo_path.join("report.json");

    // "tokmd sensor --format json"
    // We construct the command manually to run in the temp repo
    let mut cmd = cargo_bin_cmd!("tokmd");
    cmd.current_dir(repo_path);
    cmd.arg("sensor")
        .arg("--base")
        .arg("HEAD")
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
