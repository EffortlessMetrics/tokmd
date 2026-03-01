//! E2E tests for the `tokmd cockpit` CLI command.
//!
//! Every test uses `--base HEAD --head HEAD` to avoid slow full-repo diffs.

#![cfg(feature = "git")]
mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::tempdir;

/// Create a minimal git repo with one commit and return its tempdir.
fn minimal_git_repo() -> tempfile::TempDir {
    let dir = tempdir().unwrap();
    assert!(common::init_git_repo(dir.path()), "git init failed");
    std::fs::write(dir.path().join("lib.rs"), "fn main() {}\n").unwrap();
    assert!(
        common::git_add_commit(dir.path(), "Initial commit"),
        "initial commit failed"
    );
    dir
}

// ---------------------------------------------------------------------------
// 1. Basic success — cockpit with HEAD..HEAD succeeds
// ---------------------------------------------------------------------------

#[test]
fn cockpit_head_to_head_succeeds() {
    if !common::git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = minimal_git_repo();

    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args(["cockpit", "--base", "HEAD", "--head", "HEAD"])
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// 2. JSON output is valid and has expected top-level fields
// ---------------------------------------------------------------------------

#[test]
fn cockpit_json_has_expected_fields() {
    if !common::git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = minimal_git_repo();

    let output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args([
            "cockpit", "--base", "HEAD", "--head", "HEAD", "--format", "json",
        ])
        .output()
        .expect("failed to execute");

    assert!(output.status.success(), "cockpit failed");

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("output is not valid JSON");

    // CockpitReceipt top-level fields
    for field in &[
        "schema_version",
        "mode",
        "generated_at_ms",
        "base_ref",
        "head_ref",
        "change_surface",
        "composition",
        "code_health",
        "risk",
        "contracts",
        "evidence",
        "review_plan",
    ] {
        assert!(json.get(*field).is_some(), "missing field: {field}");
    }

    assert_eq!(json["mode"], "cockpit");
}

// ---------------------------------------------------------------------------
// 3. schema_version is present and positive
// ---------------------------------------------------------------------------

#[test]
fn cockpit_receipt_has_schema_version() {
    if !common::git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = minimal_git_repo();

    let output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args([
            "cockpit", "--base", "HEAD", "--head", "HEAD", "--format", "json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());

    let json: Value = serde_json::from_slice(&output.stdout).expect("output is not valid JSON");

    let sv = json["schema_version"]
        .as_u64()
        .expect("schema_version should be a number");
    assert!(sv > 0, "schema_version should be positive, got {sv}");
}

// ---------------------------------------------------------------------------
// 4. Empty diff (HEAD..HEAD) produces valid but minimal metrics
// ---------------------------------------------------------------------------

#[test]
fn cockpit_empty_diff_has_zero_surface() {
    if !common::git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = minimal_git_repo();

    let output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args([
            "cockpit", "--base", "HEAD", "--head", "HEAD", "--format", "json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());

    let json: Value = serde_json::from_slice(&output.stdout).unwrap();

    let surface = &json["change_surface"];
    assert_eq!(
        surface["files_changed"].as_u64().unwrap_or(u64::MAX),
        0,
        "HEAD..HEAD should have 0 files_changed"
    );
    assert_eq!(
        surface["insertions"].as_u64().unwrap_or(u64::MAX),
        0,
        "HEAD..HEAD should have 0 insertions"
    );
    assert_eq!(
        surface["deletions"].as_u64().unwrap_or(u64::MAX),
        0,
        "HEAD..HEAD should have 0 deletions"
    );
}

// ---------------------------------------------------------------------------
// 5. Markdown output contains expected section headings
// ---------------------------------------------------------------------------

#[test]
fn cockpit_md_contains_expected_sections() {
    if !common::git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = minimal_git_repo();

    let output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args([
            "cockpit", "--base", "HEAD", "--head", "HEAD", "--format", "md",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();

    // The markdown renderer emits these section headers
    assert!(
        stdout.contains("Glass Cockpit"),
        "should contain Glass Cockpit header"
    );
    assert!(
        stdout.contains("Change Surface"),
        "should contain Change Surface section"
    );
    assert!(
        stdout.contains("Composition"),
        "should contain Composition section"
    );
    assert!(
        stdout.contains("Contracts"),
        "should contain Contracts section"
    );
}

// ---------------------------------------------------------------------------
// 6. Invalid ref fails gracefully
// ---------------------------------------------------------------------------

#[test]
fn cockpit_invalid_base_ref_fails() {
    if !common::git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = minimal_git_repo();

    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args([
            "cockpit",
            "--base",
            "nonexistent-ref-abc123",
            "--head",
            "HEAD",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent-ref-abc123"));
}

// ---------------------------------------------------------------------------
// 7. Not inside a git repo fails gracefully
// ---------------------------------------------------------------------------

#[test]
fn cockpit_outside_git_repo_fails() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();

    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args(["cockpit", "--base", "HEAD", "--head", "HEAD"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("git"));
}

// ---------------------------------------------------------------------------
// 8. JSON output for HEAD..HEAD has composition percentages
// ---------------------------------------------------------------------------

#[test]
fn cockpit_composition_has_percentage_fields() {
    if !common::git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = minimal_git_repo();

    let output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args([
            "cockpit", "--base", "HEAD", "--head", "HEAD", "--format", "json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());

    let json: Value = serde_json::from_slice(&output.stdout).unwrap();

    let comp = &json["composition"];
    for field in &["code_pct", "test_pct", "docs_pct", "config_pct"] {
        assert!(
            comp.get(*field).is_some(),
            "composition missing field: {field}"
        );
    }
}
