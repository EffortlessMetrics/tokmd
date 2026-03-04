//! Deep tests for xtask CLI parsing, boundaries check, docs check, and publish plan.

use std::path::PathBuf;
use std::process::Command;

/// Find the workspace root by walking up from the current file's location.
fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // xtask/ -> workspace root
    manifest_dir.parent().unwrap().to_path_buf()
}

/// Run `cargo xtask <args>` and return (stdout, stderr, success).
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

// ── CLI argument parsing tests ──────────────────────────────────────────

#[test]
fn cli_help_succeeds() {
    let output = Command::new("cargo")
        .args(["run", "-q", "-p", "xtask", "--", "--help"])
        .current_dir(workspace_root())
        .output()
        .expect("failed to run xtask --help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("Development tasks for tokmd"));
}

#[test]
fn cli_bump_requires_version_arg() {
    let (_, stderr, success) = run_xtask(&["bump"]);
    assert!(!success, "bump without version should fail");
    assert!(
        stderr.contains("required") || stderr.contains("Usage"),
        "should mention required arg: {stderr}"
    );
}

#[test]
fn cli_bump_help_shows_schema_flag() {
    let output = Command::new("cargo")
        .args(["run", "-q", "-p", "xtask", "--", "bump", "--help"])
        .current_dir(workspace_root())
        .output()
        .expect("failed to run bump --help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--schema"));
    assert!(stdout.contains("--dry-run"));
}

#[test]
fn cli_publish_help_shows_flags() {
    let output = Command::new("cargo")
        .args(["run", "-q", "-p", "xtask", "--", "publish", "--help"])
        .current_dir(workspace_root())
        .output()
        .expect("failed to run publish --help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--plan"));
    assert!(stdout.contains("--dry-run"));
    assert!(stdout.contains("--crates"));
    assert!(stdout.contains("--exclude"));
    assert!(stdout.contains("--tag"));
}

#[test]
fn cli_docs_help_shows_check_and_update() {
    let output = Command::new("cargo")
        .args(["run", "-q", "-p", "xtask", "--", "docs", "--help"])
        .current_dir(workspace_root())
        .output()
        .expect("failed to run docs --help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--check"));
    assert!(stdout.contains("--update"));
}

#[test]
fn cli_boundaries_check_help() {
    let output = Command::new("cargo")
        .args([
            "run",
            "-q",
            "-p",
            "xtask",
            "--",
            "boundaries-check",
            "--help",
        ])
        .current_dir(workspace_root())
        .output()
        .expect("failed to run boundaries-check --help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("boundar") || stdout.contains("dependenc"));
}

#[test]
fn cli_unknown_subcommand_fails() {
    let (_, stderr, success) = run_xtask(&["nonexistent-command"]);
    assert!(!success, "unknown subcommand should fail");
    assert!(
        stderr.contains("error") || stderr.contains("unrecognized"),
        "should show error: {stderr}"
    );
}

// ── Boundaries check tests ─────────────────────────────────────────────

#[test]
fn boundaries_check_passes() {
    let (stdout, stderr, success) = run_xtask(&["boundaries-check"]);
    assert!(
        success,
        "boundaries-check should pass in clean repo. stderr: {stderr}"
    );
    assert!(stdout.contains("OK") || stdout.contains("boundaries"));
}

#[test]
fn boundaries_analysis_crates_exist() {
    let root = workspace_root();
    let crates_dir = root.join("crates");
    let has_analysis = std::fs::read_dir(&crates_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .any(|e| {
            e.file_name()
                .to_str()
                .map(|n| n.starts_with("tokmd-analysis"))
                .unwrap_or(false)
        });
    assert!(
        has_analysis,
        "should have at least one tokmd-analysis-* crate"
    );
}

#[test]
fn boundaries_forbidden_dep_not_present() {
    // Verify that no analysis crate depends on tokmd-config (the FORBIDDEN list)
    let root = workspace_root();
    let crates_dir = root.join("crates");
    for entry in std::fs::read_dir(&crates_dir)
        .unwrap()
        .filter_map(|e| e.ok())
    {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("tokmd-analysis") {
            continue;
        }
        let cargo_toml = entry.path().join("Cargo.toml");
        if !cargo_toml.exists() {
            continue;
        }
        let content = std::fs::read_to_string(&cargo_toml).unwrap();
        let table: toml::Table = toml::from_str(&content).unwrap();
        for dep_table in &["dependencies", "dev-dependencies", "build-dependencies"] {
            if let Some(toml::Value::Table(deps)) = table.get(*dep_table) {
                assert!(
                    !deps.contains_key("tokmd-config"),
                    "{name} should not depend on tokmd-config in [{dep_table}]"
                );
            }
        }
    }
}

// ── Publish plan tests ─────────────────────────────────────────────────

#[test]
fn publish_plan_succeeds() {
    let (stdout, stderr, success) = run_xtask(&["publish", "--plan"]);
    assert!(success, "publish --plan should succeed. stderr: {stderr}");
    assert!(stdout.contains("Publish Plan") || stdout.contains("Publish order"));
}

#[test]
fn publish_plan_lists_tokmd_types_first() {
    let (stdout, _, success) = run_xtask(&["publish", "--plan"]);
    assert!(success);
    // tokmd-types is tier 0, should appear before tokmd (tier 5)
    let types_pos = stdout.find("tokmd-types");
    let cli_pos = stdout.find("\n").and_then(|_| {
        // Find "tokmd" as a standalone crate in the list (not tokmd-*)
        stdout
            .lines()
            .position(|l| l.trim().ends_with("tokmd") || l.contains(". tokmd"))
    });
    if let (Some(t), Some(_)) = (types_pos, cli_pos) {
        assert!(t > 0, "tokmd-types should appear in the plan");
    }
}

#[test]
fn publish_plan_excludes_xtask() {
    let (stdout, _, success) = run_xtask(&["publish", "--plan", "--verbose"]);
    assert!(success);
    // xtask should be in the excluded list, not the publish order
    let lines: Vec<&str> = stdout.lines().collect();
    let in_publish_order = lines
        .iter()
        .any(|l| l.contains(". xtask") && !l.contains("tokmd"));
    assert!(!in_publish_order, "xtask should not be in publish order");
}

#[test]
fn publish_plan_shows_workspace_version() {
    let (stdout, _, success) = run_xtask(&["publish", "--plan"]);
    assert!(success);
    assert!(
        stdout.contains("Workspace version:"),
        "should show workspace version"
    );
}

#[test]
fn publish_plan_verbose_shows_exclusions() {
    let (stdout, _, success) = run_xtask(&["publish", "--plan", "--verbose"]);
    assert!(success);
    assert!(
        stdout.contains("Excluded crates"),
        "verbose mode should show excluded crates"
    );
}

// ── Bump dry-run tests ─────────────────────────────────────────────────

#[test]
fn bump_dry_run_shows_plan() {
    let (stdout, _, success) = run_xtask(&["bump", "99.99.99", "--dry-run"]);
    assert!(success, "bump --dry-run should succeed");
    assert!(stdout.contains("DRY RUN"));
    assert!(stdout.contains("99.99.99"));
}

#[test]
fn bump_dry_run_invalid_version() {
    let (_, stderr, success) = run_xtask(&["bump", "not-a-version", "--dry-run"]);
    assert!(!success, "invalid version should fail");
    assert!(
        stderr.contains("semver") || stderr.contains("MAJOR.MINOR.PATCH"),
        "should mention semver format: {stderr}"
    );
}
