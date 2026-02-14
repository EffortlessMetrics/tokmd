mod common;

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn tok_default_lang_output() {
    let bin = env!("CARGO_BIN_EXE_tok");
    if !std::path::Path::new(bin).exists() {
        eprintln!("Skipping test: tok binary not found at {}", bin);
        return;
    }

    let mut cmd = Command::new(bin);
    cmd.current_dir(common::fixture_root())
        .assert()
        .success()
        .stdout(predicate::str::contains("|Rust|"));
}
