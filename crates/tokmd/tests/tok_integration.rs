mod common;

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn tok_default_lang_output() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tok"));
    cmd.current_dir(common::fixture_root())
        .assert()
        .success()
        .stdout(predicate::str::contains("|Rust|"));
}
