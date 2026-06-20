#![cfg(feature = "analysis")]

//! BDD tests for error paths and hints.

use assert_cmd::Command;
use predicates::prelude::*;

fn tokmd_bare() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
}

#[test]
fn given_unrecognized_bare_token_when_run_then_reports_as_unrecognized_subcommand() {
    // Given: an unrecognized token that looks like a mistyped subcommand (e.g. 'anolyze')
    // When: invoking the CLI
    let mut cmd = tokmd_bare();
    let result = cmd.arg("anolyze").assert();

    // Then: it suggests the correct subcommand and reports as unrecognized subcommand
    result
        .failure()
        .stderr(predicate::str::contains(
            "Error: Unrecognized subcommand 'anolyze'",
        ))
        .stderr(predicate::str::contains(
            "Did you mean the subcommand `analyze`?",
        ))
        .stderr(predicate::str::contains("Error: Path not found: anolyze").not());
}

#[test]
fn given_path_looking_string_when_run_then_reports_as_path_not_found() {
    // Given: a string that looks like a path (e.g. 'missing/dir')
    // When: invoking the CLI
    let mut cmd = tokmd_bare();
    let result = cmd.arg("missing/dir").assert();

    // Then: it reports as path not found, not unrecognized subcommand
    result
        .failure()
        .stderr(predicate::str::contains(
            "Error: Path not found: missing/dir",
        ))
        .stderr(predicate::str::contains("Unrecognized subcommand").not());
}
