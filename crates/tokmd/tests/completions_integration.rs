mod common;

use assert_cmd::Command;
use predicates::prelude::*;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

#[test]
fn completions_bash_output_contains_command() {
    let mut cmd = tokmd_cmd();
    cmd.arg("completions")
        .arg("bash")
        .assert()
        .success()
        .stdout(predicate::str::contains("tokmd"));
}
