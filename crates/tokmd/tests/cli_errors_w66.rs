//! CLI error handling and edge-case tests (w66).
//!
//! Validates that tokmd produces helpful error messages for invalid arguments,
//! nonexistent paths, unknown subcommands, and that --help/--version work for
//! every subcommand.

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

fn nonexistent_path() -> std::path::PathBuf {
    std::env::temp_dir().join("tokmd_w66_nonexistent_path_that_does_not_exist")
}

// ===========================================================================
// 1. Invalid argument tests
// ===========================================================================

#[test]
fn invalid_format_value_for_lang_produces_error() {
    tokmd_cmd_fixture()
        .args(["lang", "--format", "yaml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn invalid_format_value_for_module_produces_error() {
    tokmd_cmd_fixture()
        .args(["module", "--format", "xml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn invalid_format_value_for_export_produces_error() {
    tokmd_cmd_fixture()
        .args(["export", "--format", "parquet"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn analyze_invalid_preset_value_produces_error() {
    tokmd_cmd_fixture()
        .args(["analyze", "--preset", "bogus_preset"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn non_numeric_top_flag_fails() {
    tokmd_cmd_fixture()
        .args(["lang", "--top", "abc"])
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn negative_top_flag_fails() {
    tokmd_cmd_fixture()
        .args(["lang", "--top", "-5"])
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn module_non_numeric_depth_fails() {
    tokmd_cmd_fixture()
        .args(["module", "--module-depth", "xyz"])
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn badge_invalid_metric_fails() {
    tokmd_cmd_fixture()
        .args(["badge", "--metric", "nonexistent_metric"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn completions_invalid_shell_fails() {
    tokmd_cmd()
        .args(["completions", "invalid_shell"])
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

// ===========================================================================
// 2. Non-existent path tests
// ===========================================================================

#[test]
fn lang_with_nonexistent_path_produces_error() {
    let p = nonexistent_path();
    tokmd_cmd()
        .arg(p.as_os_str())
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn module_with_nonexistent_path_produces_error() {
    let p = nonexistent_path();
    tokmd_cmd()
        .args(["module"])
        .arg(p.as_os_str())
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn export_with_nonexistent_path_produces_error() {
    let p = nonexistent_path();
    tokmd_cmd()
        .args(["export"])
        .arg(p.as_os_str())
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn analyze_with_nonexistent_path_produces_error() {
    let p = nonexistent_path();
    tokmd_cmd()
        .args(["analyze"])
        .arg(p.as_os_str())
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn run_with_nonexistent_path_produces_error() {
    let p = nonexistent_path();
    tokmd_cmd()
        .args(["run"])
        .arg(p.as_os_str())
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn diff_with_nonexistent_files_produces_error() {
    let p = nonexistent_path();
    tokmd_cmd()
        .args(["diff", "--from"])
        .arg(p.join("a.json").as_os_str())
        .arg("--to")
        .arg(p.join("b.json").as_os_str())
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

// ===========================================================================
// 3. --help works for every subcommand
// ===========================================================================

#[test]
fn help_flag_for_all_subcommands() {
    let subcommands = [
        "lang",
        "module",
        "export",
        "run",
        "analyze",
        "badge",
        "diff",
        "cockpit",
        "gate",
        "tools",
        "context",
        "init",
        "check-ignore",
        "completions",
        "baseline",
        "handoff",
        "sensor",
    ];
    for sub in subcommands {
        tokmd_cmd()
            .args([sub, "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Usage").or(predicate::str::contains("usage")));
    }
}

#[test]
fn root_help_mentions_subcommands() {
    tokmd_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("lang"))
        .stdout(predicate::str::contains("module"))
        .stdout(predicate::str::contains("export"))
        .stdout(predicate::str::contains("analyze"));
}

// ===========================================================================
// 4. --version output format
// ===========================================================================

#[test]
fn version_flag_contains_tokmd_and_semver() {
    let output = tokmd_cmd()
        .arg("--version")
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("tokmd"), "should contain 'tokmd'");
    // Semver pattern: at least X.Y.Z
    let re = regex::Regex::new(r"\d+\.\d+\.\d+").unwrap();
    assert!(
        re.is_match(&stdout),
        "version output should contain semver: {stdout}"
    );
}

// ===========================================================================
// 5. Unknown subcommand
// ===========================================================================

#[test]
fn unknown_subcommand_produces_helpful_error() {
    tokmd_cmd()
        .arg("not-a-real-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Path not found: not-a-real-command",
        ));
}

#[test]
fn empty_string_subcommand_fails_or_defaults() {
    tokmd_cmd()
        .arg("")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

// ===========================================================================
// 6. Commands that require arguments fail without them
// ===========================================================================

#[test]
fn diff_without_required_args_fails() {
    tokmd_cmd()
        .arg("diff")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn gate_without_required_args_fails() {
    tokmd_cmd()
        .arg("gate")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn cockpit_with_nonexistent_base_ref_fails() {
    tokmd_cmd_fixture()
        .args(["cockpit", "--base", "nonexistent_ref_w66_abc"])
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn unknown_global_flag_fails() {
    tokmd_cmd_fixture()
        .arg("--does-not-exist")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}
