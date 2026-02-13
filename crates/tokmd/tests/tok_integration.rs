mod common;

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
#[cfg(feature = "alias-tok")]
fn tok_default_lang_output() {
    let bin_path = env!("CARGO_BIN_EXE_tok");
    if !std::path::Path::new(bin_path).exists() {
        return;
    }
    let mut cmd = Command::new(bin_path);
    cmd.current_dir(common::fixture_root())
        .assert()
        .success()
        .stdout(predicate::str::contains("|Rust|"));
}
