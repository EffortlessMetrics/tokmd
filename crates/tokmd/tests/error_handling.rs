//! E2E tests for CLI error handling scenarios.
//!
//! Validates that tokmd returns non-zero exit codes and helpful error messages
//! when invoked with invalid arguments, nonexistent paths, or other error
//! conditions.  Where the CLI intentionally succeeds gracefully (e.g. empty
//! directories yielding zero-row output) we verify the shape of that output.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn tokmd_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
}

// ---------------------------------------------------------------------------
// Invalid --format values (caught by clap value_enum)
// ---------------------------------------------------------------------------

#[test]
fn lang_invalid_format_fails() {
    tokmd_cmd()
        .arg("lang")
        .arg("--format")
        .arg("invalid_format")
        .arg(".")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value 'invalid_format'"));
}

#[test]
fn module_invalid_format_fails() {
    tokmd_cmd()
        .arg("module")
        .arg("--format")
        .arg("invalid_format")
        .arg(".")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value 'invalid_format'"));
}

#[test]
fn export_invalid_format_fails() {
    tokmd_cmd()
        .arg("export")
        .arg("--format")
        .arg("invalid_format")
        .arg(".")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value 'invalid_format'"));
}

#[test]
fn analyze_invalid_format_fails() {
    tokmd_cmd()
        .arg("analyze")
        .arg("--format")
        .arg("invalid_format")
        .arg(".")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value 'invalid_format'"));
}

// ---------------------------------------------------------------------------
// Invalid --preset value for analyze / badge
// ---------------------------------------------------------------------------

#[test]
fn analyze_invalid_preset_fails() {
    tokmd_cmd()
        .arg("analyze")
        .arg("--preset")
        .arg("nonexistent")
        .arg(".")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value 'nonexistent'"));
}

#[test]
fn badge_invalid_preset_fails() {
    tokmd_cmd()
        .arg("badge")
        .arg("--preset")
        .arg("nonexistent")
        .arg(".")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value 'nonexistent'"));
}

// ---------------------------------------------------------------------------
// Unknown subcommand
// ---------------------------------------------------------------------------

#[test]
fn unknown_subcommand_fails() {
    tokmd_cmd()
        .arg("this-subcommand-does-not-exist")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

// ---------------------------------------------------------------------------
// Empty directory — CLI succeeds gracefully with zero totals
// ---------------------------------------------------------------------------

#[test]
fn lang_empty_directory_succeeds_with_zero_totals() {
    let dir = tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();

    tokmd_cmd()
        .arg("lang")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("|**Total**|0|0|0|0|"));
}

#[test]
fn module_empty_directory_succeeds_with_zero_totals() {
    let dir = tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();

    tokmd_cmd()
        .arg("module")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("|**Total**|0|0|0|0|0|0|"));
}

#[test]
fn export_empty_directory_succeeds_with_meta_only() {
    let dir = tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();

    // JSONL export should emit only the meta record and no data rows
    let output = tokmd_cmd()
        .arg("export")
        .current_dir(dir.path())
        .output()
        .expect("execute tokmd export");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("valid UTF-8");
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "expected only the meta record");
    assert!(lines[0].contains(r#""type":"meta""#));
}

// ---------------------------------------------------------------------------
// Gate command: missing required arguments
// ---------------------------------------------------------------------------

#[test]
fn gate_missing_receipt_arg_fails() {
    tokmd_cmd()
        .arg("gate")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

// ---------------------------------------------------------------------------
// Diff command: missing required arguments
// ---------------------------------------------------------------------------

#[test]
fn diff_missing_args_fails() {
    tokmd_cmd()
        .arg("diff")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

// ---------------------------------------------------------------------------
// Conflicting / unrecognised flags
// ---------------------------------------------------------------------------

#[test]
fn unknown_flag_fails() {
    tokmd_cmd()
        .arg("lang")
        .arg("--this-flag-does-not-exist")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument"));
}

// ---------------------------------------------------------------------------
// Gate: nonexistent receipt file
// ---------------------------------------------------------------------------

#[test]
fn gate_nonexistent_receipt_file_fails() {
    let dir = tempdir().unwrap();
    let policy = dir.path().join("policy.json");
    std::fs::write(&policy, r#"{"rules":[]}"#).unwrap();

    tokmd_cmd()
        .arg("gate")
        .arg("--receipt")
        .arg(dir.path().join("does_not_exist.json"))
        .arg("--policy")
        .arg(&policy)
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

// ---------------------------------------------------------------------------
// Diff: nonexistent input files
// ---------------------------------------------------------------------------

#[test]
fn diff_nonexistent_files_fails() {
    tokmd_cmd()
        .arg("diff")
        .arg("--before")
        .arg("/tmp/no_such_file_a.json")
        .arg("--after")
        .arg("/tmp/no_such_file_b.json")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

// ---------------------------------------------------------------------------
// Lang: invalid --children value
// ---------------------------------------------------------------------------

#[test]
fn lang_invalid_children_mode_fails() {
    tokmd_cmd()
        .arg("lang")
        .arg("--children")
        .arg("invalid_mode")
        .arg(".")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value 'invalid_mode'"));
}
