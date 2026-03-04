//! Edge-case CLI tests for the tokmd binary.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

// ---------------------------------------------------------------------------
// Default (no arguments) behavior
// ---------------------------------------------------------------------------

#[test]
fn cli_no_arguments_defaults_to_lang() {
    tokmd_cmd().assert().success();
}

// ---------------------------------------------------------------------------
// Non-existent target path
// ---------------------------------------------------------------------------

#[test]
fn cli_nonexistent_path_fails() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.arg("/this/path/does/not/exist/at/all");
    cmd.assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

// ---------------------------------------------------------------------------
// Invalid format flag
// ---------------------------------------------------------------------------

#[test]
fn cli_invalid_format_fails() {
    tokmd_cmd()
        .args(["lang", "--format", "invalid_format_xyz"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

// ---------------------------------------------------------------------------
// --top 0 edge case
// ---------------------------------------------------------------------------

#[test]
fn cli_top_zero_shows_all() {
    tokmd_cmd().args(["lang", "--top", "0"]).assert().success();
}

// ---------------------------------------------------------------------------
// Very long path argument
// ---------------------------------------------------------------------------

#[test]
fn cli_very_long_path_argument() {
    let long_path = "a/".repeat(500) + "fake";
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.arg(&long_path);
    cmd.assert().failure();
}

// ---------------------------------------------------------------------------
// Help output
// ---------------------------------------------------------------------------

#[test]
fn cli_help_contains_expected_sections() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage"))
        .stdout(predicate::str::contains("Commands"));
}

#[test]
fn cli_lang_help_mentions_format() {
    tokmd_cmd()
        .args(["lang", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--format"));
}

// ---------------------------------------------------------------------------
// Version output
// ---------------------------------------------------------------------------

#[test]
fn cli_version_is_nonempty() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ---------------------------------------------------------------------------
// Conflicting or unusual flag combos
// ---------------------------------------------------------------------------

#[test]
fn cli_top_with_format_json() {
    tokmd_cmd()
        .args(["lang", "--top", "3", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("schema_version"));
}

#[test]
fn cli_module_depth_zero() {
    tokmd_cmd()
        .args(["module", "--module-depth", "0"])
        .assert()
        .success();
}

#[test]
fn cli_export_min_code_max() {
    tokmd_cmd()
        .args(["export", "--min-code", "999999"])
        .assert()
        .success();
}
