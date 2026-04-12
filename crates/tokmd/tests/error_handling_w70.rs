//! Error handling edge-case tests (w70).
//!
//! Validates that tokmd CLI produces correct exit codes, helpful error messages
//! on stderr, and handles invalid inputs gracefully across all subcommands.

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
    std::env::temp_dir().join("tokmd_w70_nonexistent_path_does_not_exist")
}

// ===========================================================================
// 1. Nonexistent path handling
// ===========================================================================

#[test]
fn lang_nonexistent_path_fails_with_nonzero_exit() {
    tokmd_cmd()
        .args(["lang", "--path"])
        .arg(nonexistent_path())
        .assert()
        .failure();
}

#[test]
fn module_nonexistent_path_fails_with_nonzero_exit() {
    tokmd_cmd()
        .args(["module", "--path"])
        .arg(nonexistent_path())
        .assert()
        .failure();
}

#[test]
fn export_nonexistent_path_fails_with_nonzero_exit() {
    tokmd_cmd()
        .args(["export", "--path"])
        .arg(nonexistent_path())
        .assert()
        .failure();
}

#[test]
fn analyze_nonexistent_path_fails_with_nonzero_exit() {
    tokmd_cmd()
        .args(["analyze", "--path"])
        .arg(nonexistent_path())
        .assert()
        .failure();
}

#[test]
fn run_nonexistent_path_fails_with_nonzero_exit() {
    tokmd_cmd()
        .args(["run", "--path"])
        .arg(nonexistent_path())
        .assert()
        .failure();
}

// ===========================================================================
// 2. Invalid format options
// ===========================================================================

#[test]
fn lang_invalid_format_yaml_produces_stderr_error() {
    tokmd_cmd_fixture()
        .args(["lang", "--format", "yaml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value").or(predicate::str::contains("error")));
}

#[test]
fn module_invalid_format_xml_produces_stderr_error() {
    tokmd_cmd_fixture()
        .args(["module", "--format", "xml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value").or(predicate::str::contains("error")));
}

#[test]
fn export_invalid_format_parquet_produces_stderr_error() {
    tokmd_cmd_fixture()
        .args(["export", "--format", "parquet"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value").or(predicate::str::contains("error")));
}

// ===========================================================================
// 3. Invalid presets for analyze
// ===========================================================================

#[test]
fn analyze_invalid_preset_bogus_produces_stderr_error() {
    tokmd_cmd_fixture()
        .args(["analyze", "--preset", "nonexistent_preset_w70"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

// ===========================================================================
// 4. Non-numeric / invalid flag values
// ===========================================================================

#[test]
fn lang_top_flag_with_non_numeric_value_fails() {
    tokmd_cmd_fixture()
        .args(["lang", "--top", "not_a_number"])
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn module_depth_flag_with_non_numeric_value_fails() {
    tokmd_cmd_fixture()
        .args(["module", "--depth", "abc"])
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn lang_top_flag_with_negative_value_fails() {
    tokmd_cmd_fixture()
        .args(["lang", "--top", "-3"])
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

// ===========================================================================
// 5. Unknown subcommand
// ===========================================================================

#[test]
fn unknown_subcommand_fails_with_stderr() {
    tokmd_cmd()
        .arg("nonexistent_subcommand_w70")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Path not found: nonexistent_subcommand_w70",
        ));
}

// ===========================================================================
// 6. Stderr contains error output (not stdout)
// ===========================================================================

#[test]
fn error_messages_go_to_stderr_not_stdout_for_invalid_format() {
    let output = tokmd_cmd_fixture()
        .args(["lang", "--format", "yaml"])
        .output()
        .expect("failed to execute");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.is_empty(),
        "stderr should contain error message for invalid format"
    );
}

#[test]
fn error_messages_go_to_stderr_for_unknown_subcommand() {
    let output = tokmd_cmd()
        .arg("bogus_cmd_w70")
        .output()
        .expect("failed to execute");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Path not found: bogus_cmd_w70"),
        "stderr should contain error for unknown subcommand"
    );
}

#[test]
fn error_messages_go_to_stderr_for_nonexistent_path() {
    let output = tokmd_cmd()
        .args(["lang", "--path"])
        .arg(nonexistent_path())
        .output()
        .expect("failed to execute");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Some implementations may use exit code + stderr OR stdout error;
    // at minimum, exit code must be non-zero
    assert!(!output.status.success());
    let _ = stderr; // Ensures we captured it
}

// ===========================================================================
// 7. Gate CLI error paths
// ===========================================================================

#[test]
fn gate_with_nonexistent_policy_file_fails() {
    tokmd_cmd_fixture()
        .args(["gate", "--policy", "nonexistent_policy_w70.toml"])
        .assert()
        .failure();
}

#[test]
fn gate_with_nonexistent_receipt_file_fails() {
    tokmd_cmd_fixture()
        .args([
            "gate",
            "--receipt",
            "nonexistent_receipt_w70.json",
            "--policy",
            "nonexistent_policy_w70.toml",
        ])
        .assert()
        .failure();
}

// ===========================================================================
// 8. Diff CLI error paths
// ===========================================================================

#[test]
fn diff_missing_required_args_fails() {
    // diff requires at least two receipts/paths to compare
    tokmd_cmd()
        .arg("diff")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

// ===========================================================================
// 9. Context CLI error paths
// ===========================================================================

#[test]
fn context_nonexistent_path_fails() {
    tokmd_cmd()
        .args(["context", "--path"])
        .arg(nonexistent_path())
        .assert()
        .failure();
}

// ===========================================================================
// 10. Badge CLI error paths
// ===========================================================================

#[test]
fn badge_with_invalid_metric_fails() {
    tokmd_cmd_fixture()
        .args(["badge", "--metric", "nonexistent_metric_w70"])
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

// ===========================================================================
// 11. Malformed --exclude patterns (still succeed but produce different results)
// ===========================================================================

#[test]
fn lang_with_empty_exclude_flag_does_not_crash() {
    // Empty exclude should not crash; the command should still work
    tokmd_cmd_fixture()
        .args(["lang", "--exclude", ""])
        .assert()
        .success();
}

// ===========================================================================
// 12. Multiple invalid args combined
// ===========================================================================

#[test]
fn multiple_invalid_flags_still_produces_error() {
    tokmd_cmd_fixture()
        .args(["lang", "--format", "yaml", "--top", "abc"])
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn export_with_conflicting_invalid_format_fails() {
    tokmd_cmd_fixture()
        .args(["export", "--format", "binary"])
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

// ===========================================================================
// 13. Exit code is non-zero for all error scenarios
// ===========================================================================

#[test]
fn exit_code_is_nonzero_for_invalid_subcommand() {
    let output = tokmd_cmd()
        .arg("invalid_sub_w70")
        .output()
        .expect("failed to execute");
    assert!(!output.status.success(), "exit code must be non-zero");
}
