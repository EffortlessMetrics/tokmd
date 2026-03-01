//! E2E tests verifying `tokmd --version` / `-V` output.

use assert_cmd::Command;
use predicates::prelude::*;

fn tokmd_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
}

// ── tokmd --version ──────────────────────────────────────────────────

#[test]
fn version_long_flag_exits_successfully() {
    tokmd_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\d+\.\d+\.\d+").unwrap());
}

// ── tokmd -V ─────────────────────────────────────────────────────────

#[test]
fn version_short_flag_exits_successfully() {
    tokmd_cmd()
        .arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\d+\.\d+\.\d+").unwrap());
}

// ── version matches Cargo.toml ───────────────────────────────────────

#[test]
fn version_matches_cargo_toml() {
    let expected = env!("CARGO_PKG_VERSION");

    tokmd_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(expected));
}
