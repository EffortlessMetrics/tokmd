//! End-to-end tests for `tokmd completions` — shell-specific output
//! validation for bash, zsh, fish, powershell, and elvish.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

// ---------------------------------------------------------------------------
// Bash
// ---------------------------------------------------------------------------

#[test]
fn completions_bash_contains_tokmd_reference() {
    tokmd_cmd()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tokmd"));
}

#[test]
fn completions_bash_contains_complete_keyword() {
    tokmd_cmd()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete"));
}

// ---------------------------------------------------------------------------
// Zsh
// ---------------------------------------------------------------------------

#[test]
fn completions_zsh_contains_tokmd_reference() {
    tokmd_cmd()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tokmd"));
}

#[test]
fn completions_zsh_contains_compdef_or_arguments() {
    tokmd_cmd()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#compdef").or(predicate::str::contains("_arguments")));
}

// ---------------------------------------------------------------------------
// Fish
// ---------------------------------------------------------------------------

#[test]
fn completions_fish_contains_complete_command() {
    tokmd_cmd()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete"));
}

#[test]
fn completions_fish_references_tokmd() {
    tokmd_cmd()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tokmd"));
}

// ---------------------------------------------------------------------------
// PowerShell
// ---------------------------------------------------------------------------

#[test]
fn completions_powershell_contains_register() {
    tokmd_cmd()
        .args(["completions", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Register-ArgumentCompleter"));
}

#[test]
fn completions_powershell_references_tokmd() {
    tokmd_cmd()
        .args(["completions", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tokmd"));
}

// ---------------------------------------------------------------------------
// Elvish
// ---------------------------------------------------------------------------

#[test]
fn completions_elvish_produces_non_empty_output() {
    tokmd_cmd()
        .args(["completions", "elvish"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_elvish_references_tokmd() {
    tokmd_cmd()
        .args(["completions", "elvish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tokmd"));
}

// ---------------------------------------------------------------------------
// Error case
// ---------------------------------------------------------------------------

#[test]
fn completions_invalid_shell_fails() {
    tokmd_cmd()
        .args(["completions", "tcsh"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}
