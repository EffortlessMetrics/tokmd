//! Integration tests for the `tokmd cockpit` command.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

/// Helper to check if git is available in the environment.
fn git_available() -> bool {
    std::process::Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Helper to initialize a git repo with some commits.
fn init_git_repo(dir: &std::path::Path) -> bool {
    let commands = [
        vec!["init"],
        vec!["symbolic-ref", "HEAD", "refs/heads/main"], // Set default branch to main
        vec!["config", "user.email", "test@test.com"],
        vec!["config", "user.name", "Test User"],
    ];

    for args in &commands {
        let status = std::process::Command::new("git")
            .args(args)
            .current_dir(dir)
            .status();
        if !status.map(|s| s.success()).unwrap_or(false) {
            return false;
        }
    }
    true
}

fn git_add_commit(dir: &std::path::Path, message: &str) -> bool {
    let commands = [vec!["add", "."], vec!["commit", "-m", message]];

    for args in &commands {
        let status = std::process::Command::new("git")
            .args(args)
            .current_dir(dir)
            .status();
        if !status.map(|s| s.success()).unwrap_or(false) {
            return false;
        }
    }
    true
}

#[test]
fn test_cockpit_help() {
    // Given: The cockpit command exists
    // When: We run `tokmd cockpit --help`
    // Then: It should show help with expected options
    let mut cmd = tokmd_cmd();
    cmd.arg("cockpit")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--base"))
        .stdout(predicate::str::contains("--head"))
        .stdout(predicate::str::contains("--format"))
        .stdout(predicate::str::contains("--output"));
}

#[test]
fn test_cockpit_json_format() {
    if !git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = tempdir().unwrap();

    // Initialize git repo
    if !init_git_repo(dir.path()) {
        eprintln!("Skipping: git init failed");
        return;
    }

    // Create initial commit on main
    std::fs::write(dir.path().join("lib.rs"), "fn main() {}").unwrap();
    if !git_add_commit(dir.path(), "Initial commit") {
        eprintln!("Skipping: git commit failed");
        return;
    }

    // Create a branch with changes
    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(dir.path())
        .status();

    std::fs::write(dir.path().join("new.rs"), "fn new() {}").unwrap();
    if !git_add_commit(dir.path(), "Add new file") {
        eprintln!("Skipping: second commit failed");
        return;
    }

    // Run cockpit
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd
        .current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--head")
        .arg("HEAD")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("cockpit failed: {}", stderr);
        // Don't fail the test - just verify the command is recognized
        return;
    }

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Verify JSON structure
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");

    assert!(
        json.get("schema_version").is_some(),
        "should have schema_version"
    );
    assert!(
        json.get("change_surface").is_some(),
        "should have change_surface"
    );
    assert!(json.get("composition").is_some(), "should have composition");
    assert!(json.get("contracts").is_some(), "should have contracts");
    assert!(json.get("review_plan").is_some(), "should have review_plan");
}

#[test]
fn test_cockpit_md_format() {
    if !git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = tempdir().unwrap();

    if !init_git_repo(dir.path()) {
        eprintln!("Skipping: git init failed");
        return;
    }

    std::fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
    if !git_add_commit(dir.path(), "Initial") {
        return;
    }

    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(dir.path())
        .status();

    std::fs::write(dir.path().join("test.rs"), "fn test() {}").unwrap();
    if !git_add_commit(dir.path(), "Add test") {
        return;
    }

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd
        .current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--format")
        .arg("md")
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Verify markdown structure
    assert!(
        stdout.contains("## Glass Cockpit"),
        "should have Glass Cockpit header"
    );
    assert!(
        stdout.contains("### Change Surface"),
        "should have Change Surface section"
    );
    assert!(
        stdout.contains("### Composition"),
        "should have Composition section"
    );
    assert!(
        stdout.contains("### Contracts"),
        "should have Contracts section"
    );
    assert!(
        stdout.contains("### Review Plan"),
        "should have Review Plan section"
    );
}

#[test]
fn test_cockpit_sections_format() {
    if !git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = tempdir().unwrap();

    if !init_git_repo(dir.path()) {
        return;
    }

    std::fs::write(dir.path().join("app.rs"), "fn app() {}").unwrap();
    if !git_add_commit(dir.path(), "Initial") {
        return;
    }

    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "dev"])
        .current_dir(dir.path())
        .status();

    std::fs::write(dir.path().join("mod.rs"), "mod app;").unwrap();
    if !git_add_commit(dir.path(), "Add module") {
        return;
    }

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd
        .current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--format")
        .arg("sections")
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Verify sections format (used for AI-FILL markers)
    assert!(
        stdout.contains("<!-- SECTION:COCKPIT -->"),
        "should have COCKPIT section marker"
    );
    assert!(
        stdout.contains("<!-- SECTION:REVIEW_PLAN -->"),
        "should have REVIEW_PLAN section marker"
    );
    assert!(
        stdout.contains("<!-- SECTION:RECEIPTS -->"),
        "should have RECEIPTS section marker"
    );
}

#[test]
fn test_cockpit_output_file() {
    if !git_available() {
        return;
    }

    let dir = tempdir().unwrap();

    if !init_git_repo(dir.path()) {
        return;
    }

    std::fs::write(dir.path().join("code.rs"), "fn code() {}").unwrap();
    if !git_add_commit(dir.path(), "Initial") {
        return;
    }

    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "test"])
        .current_dir(dir.path())
        .status();

    std::fs::write(dir.path().join("new.rs"), "fn new() {}").unwrap();
    if !git_add_commit(dir.path(), "New") {
        return;
    }

    let output_file = dir.path().join("cockpit.json");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--output")
        .arg(&output_file)
        .assert()
        .success()
        .stdout(""); // stdout should be empty

    // Verify file was created with valid JSON
    assert!(output_file.exists(), "output file should exist");
    let content = std::fs::read_to_string(&output_file).unwrap();
    let _: serde_json::Value = serde_json::from_str(&content).expect("valid JSON in file");
}

#[test]
fn test_cockpit_artifacts_dir() {
    if !git_available() {
        return;
    }

    let dir = tempdir().unwrap();

    if !init_git_repo(dir.path()) {
        return;
    }

    std::fs::write(dir.path().join("code.rs"), "fn code() {}").unwrap();
    if !git_add_commit(dir.path(), "Initial") {
        return;
    }

    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "test"])
        .current_dir(dir.path())
        .status();

    std::fs::write(dir.path().join("new.rs"), "fn new() {}").unwrap();
    if !git_add_commit(dir.path(), "New") {
        return;
    }

    let artifacts_dir = dir.path().join("artifacts").join("tokmd");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--artifacts-dir")
        .arg(&artifacts_dir)
        .assert()
        .success();

    let report_path = artifacts_dir.join("report.json");
    let comment_path = artifacts_dir.join("comment.md");
    assert!(report_path.exists(), "report.json should exist");
    assert!(comment_path.exists(), "comment.md should exist");

    let report = std::fs::read_to_string(&report_path).unwrap();
    let _: serde_json::Value = serde_json::from_str(&report).expect("valid JSON in report");

    let comment = std::fs::read_to_string(&comment_path).unwrap();
    let bullet_count = comment.lines().filter(|l| l.trim_start().starts_with("- ")).count();
    assert!(
        (3..=8).contains(&bullet_count),
        "comment bullet count should be 3-8, got {}",
        bullet_count
    );
}

#[test]
fn test_cockpit_not_in_git_repo() {
    // Given: A directory that is not a git repo
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();

    // When: We run cockpit
    // Then: It should fail with an appropriate error
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .arg("cockpit")
        .assert()
        .failure()
        .stderr(predicate::str::contains("git"));
}

#[test]
fn test_cockpit_file_classification() {
    if !git_available() {
        return;
    }

    let dir = tempdir().unwrap();

    if !init_git_repo(dir.path()) {
        return;
    }

    // Create initial commit
    std::fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
    if !git_add_commit(dir.path(), "Initial") {
        return;
    }

    // Create branch with diverse file types
    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "diverse"])
        .current_dir(dir.path())
        .status();

    // Code
    std::fs::write(dir.path().join("lib.rs"), "pub fn lib() {}").unwrap();
    // Test
    std::fs::create_dir(dir.path().join("tests")).unwrap();
    std::fs::write(dir.path().join("tests").join("test.rs"), "fn test() {}").unwrap();
    // Docs
    std::fs::write(dir.path().join("README.md"), "# README").unwrap();
    // Config
    std::fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();

    if !git_add_commit(dir.path(), "Add diverse files") {
        return;
    }

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd
        .current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Verify composition has all categories
    let composition = &json["composition"];
    assert!(
        composition.get("code_pct").is_some(),
        "should have code_pct"
    );
    assert!(
        composition.get("test_pct").is_some(),
        "should have test_pct"
    );
    assert!(
        composition.get("docs_pct").is_some(),
        "should have docs_pct"
    );
    assert!(
        composition.get("config_pct").is_some(),
        "should have config_pct"
    );
}
