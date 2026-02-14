mod common;

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn tok_default_lang_output() {
    // Only run if the binary exists (it's feature-gated and not built by default `cargo test`)
    let bin_path = std::path::Path::new(env!("CARGO_BIN_EXE_tok"));
    if !bin_path.exists() {
        eprintln!("Skipping tok_default_lang_output: binary not found at {:?}", bin_path);
        return;
    }

    let mut cmd = Command::new(bin_path);
    cmd.current_dir(common::fixture_root())
        .assert()
        .success()
        .stdout(predicate::str::contains("|Rust|"));
}
