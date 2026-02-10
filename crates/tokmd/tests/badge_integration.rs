mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

#[test]
fn badge_lines_svg_stdout() {
    let mut cmd = tokmd_cmd();
    cmd.arg("badge")
        .arg("--metric")
        .arg("lines")
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("lines"));
}

#[test]
fn badge_writes_to_file() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let out_file = dir.path().join("badge.svg");

    let mut cmd = tokmd_cmd();
    cmd.arg("badge")
        .arg("--metric")
        .arg("bytes")
        .arg("--out")
        .arg(&out_file)
        .assert()
        .success()
        .stdout("");

    let content = std::fs::read_to_string(&out_file)?;
    assert!(content.contains("<svg"));
    assert!(content.contains("bytes"));
    Ok(())
}
