//! CLI error-path and help-output integration tests.
//!
//! Validates that tokmd produces user-friendly error messages for invalid
//! arguments, nonexistent paths, and other edge cases, and that help/version
//! output is well-formed.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;

fn tokmd_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
}

fn tokmd_cmd_fixture() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

/// Returns a path that reliably does not exist on any platform.
fn nonexistent_path() -> std::path::PathBuf {
    std::env::temp_dir().join("tokmd_w51_nonexistent_path_that_does_not_exist")
}

// ===========================================================================
// 1. Invalid argument tests
// ===========================================================================

#[test]
fn invalid_format_on_default_command_fails() {
    tokmd_cmd_fixture()
        .args(["--format", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn negative_top_value_fails() {
    tokmd_cmd_fixture()
        .args(["--top", "-1"])
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn module_negative_depth_fails() {
    tokmd_cmd_fixture()
        .args(["module", "--module-depth", "-1"])
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn analyze_invalid_preset_fails() {
    tokmd_cmd_fixture()
        .args(["analyze", "--preset", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn diff_no_arguments_fails() {
    tokmd_cmd()
        .arg("diff")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn gate_no_arguments_fails() {
    tokmd_cmd()
        .arg("gate")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn context_budget_zero_succeeds_with_zero_tokens() {
    tokmd_cmd_fixture()
        .args(["context", "--budget", "0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Budget: 0 tokens"));
}

#[test]
fn export_invalid_format_fails() {
    tokmd_cmd_fixture()
        .args(["export", "--format", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

// ===========================================================================
// 2. Missing / invalid path tests
// ===========================================================================

#[test]
fn lang_nonexistent_path_fails() {
    let p = nonexistent_path();
    tokmd_cmd()
        .arg(p.as_os_str())
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn module_nonexistent_path_fails() {
    let p = nonexistent_path();
    tokmd_cmd()
        .arg("module")
        .arg(p.as_os_str())
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn export_nonexistent_path_fails() {
    let p = nonexistent_path();
    tokmd_cmd()
        .arg("export")
        .arg(p.as_os_str())
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn analyze_nonexistent_path_fails() {
    let p = nonexistent_path();
    tokmd_cmd()
        .arg("analyze")
        .arg(p.as_os_str())
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn diff_nonexistent_before_after_fails() {
    let p = nonexistent_path();
    let a = p.join("a.json");
    let b = p.join("b.json");
    tokmd_cmd()
        .arg("diff")
        .arg("--from")
        .arg(a.as_os_str())
        .arg("--to")
        .arg(b.as_os_str())
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

// ===========================================================================
// 3. Help and version output
// ===========================================================================

#[test]
fn help_flag_succeeds_with_usage() {
    tokmd_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn version_flag_succeeds_with_version() {
    tokmd_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("tokmd"));
}

#[test]
fn lang_help_succeeds() {
    tokmd_cmd()
        .args(["lang", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn analyze_help_lists_presets() {
    tokmd_cmd()
        .args(["analyze", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("preset"));
}

#[test]
fn completions_help_mentions_shells() {
    tokmd_cmd()
        .args(["completions", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("bash").or(predicate::str::contains("shell")));
}

// ===========================================================================
// 4. Subcommand discovery
// ===========================================================================

#[test]
fn unknown_subcommand_fails() {
    tokmd_cmd()
        .arg("nonexistent-subcommand")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

/// Verify that every expected subcommand responds to --help without error.
#[test]
fn known_subcommands_respond_to_help() {
    let subcommands = [
        "lang", "module", "export", "analyze", "diff", "badge", "gate", "context", "handoff",
    ];
    for sub in subcommands {
        tokmd_cmd()
            .args([sub, "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Usage"));
    }
}

#[test]
fn run_subcommand_responds_to_help() {
    tokmd_cmd()
        .args(["run", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn cockpit_subcommand_responds_to_help() {
    tokmd_cmd()
        .args(["cockpit", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}
