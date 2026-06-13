#![cfg(feature = "git")]

//! BDD-style scenario tests for the `cockpit` command.
//!
//! Each test follows the Given/When/Then pattern to verify key user-facing
//! workflows of the cockpit PR summarization command.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn tokmd_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
}

/// Scaffold a two-branch git repo: main (initial commit) -> feature (with changes).
fn scaffold_git_repo(
    base_files: &[(&str, &str)],
    feature_files: &[(&str, &str)],
) -> tempfile::TempDir {
    let dir = tempdir().unwrap();

    if !common::git_available() || !common::init_git_repo(dir.path()) {
        panic!("git not available or init failed");
    }

    for (name, content) in base_files {
        let path = dir.path().join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, content).unwrap();
    }
    assert!(common::git_add_commit(dir.path(), "Initial commit"));

    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(dir.path())
        .status();

    for (name, content) in feature_files {
        let path = dir.path().join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, content).unwrap();
    }
    assert!(common::git_add_commit(dir.path(), "Feature commit"));

    dir
}

// ---------------------------------------------------------------------------
// Scenario 1: Cockpit Markdown generation
// ---------------------------------------------------------------------------

#[test]
fn given_git_branch_with_changes_when_cockpit_md_then_sections_generated() {
    if !common::git_available() {
        return;
    }

    // Given: A git repository with a feature branch containing changes
    let dir = scaffold_git_repo(
        &[("src/main.rs", "fn main() {}\n")],
        &[("src/main.rs", "fn main() { println!(\"hello\"); }\n")],
    );

    // When: I run the cockpit command with markdown format
    tokmd_cmd()
        .current_dir(dir.path())
        .args(["cockpit", "--base", "main", "--format", "md"])
        .assert()
        // Then: The markdown output contains expected Review Plan and Change Surface headers
        .success()
        .stdout(predicate::str::contains("## Glass Cockpit"))
        .stdout(predicate::str::contains("### Change Surface"))
        .stdout(predicate::str::contains("### Review Plan"));
}

// ---------------------------------------------------------------------------
// Scenario 2: Cockpit JSON structure
// ---------------------------------------------------------------------------

#[test]
fn given_git_branch_with_changes_when_cockpit_json_then_valid_schema() {
    if !common::git_available() {
        return;
    }

    // Given: A git repository with a feature branch containing changes
    let dir = scaffold_git_repo(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { println!(\"hi\"); }\n")],
    );

    // When: I run the cockpit command with json format
    let output = tokmd_cmd()
        .current_dir(dir.path())
        .args(["cockpit", "--base", "main", "--format", "json"])
        .output()
        .unwrap();

    assert!(output.status.success());

    // Then: The JSON output matches the expected envelope schema
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["mode"], "cockpit");
    assert!(json["schema_version"].is_number());
    assert!(json["change_surface"].is_object());
    assert!(json["review_plan"].is_array());
}
