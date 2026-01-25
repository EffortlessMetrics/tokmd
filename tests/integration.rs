use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::cargo_bin("tokmd").unwrap();
    // Point to our test fixture
    cmd.current_dir("tests/data");
    cmd
}

#[test]
fn test_default_lang_output() {
    let mut cmd = tokmd_cmd();
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("|Rust|"));
}

#[test]
fn test_module_output() {
    let mut cmd = tokmd_cmd();
    cmd.arg("module")
        .assert()
        .success()
        .stdout(predicate::str::contains("|(root)|"))
        .stdout(predicate::str::contains("|src|"));
}

#[test]
fn test_export_jsonl() {
    let mut cmd = tokmd_cmd();
    cmd.arg("export")
        .arg("--format")
        .arg("jsonl")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""mode":"export""#)) // Meta record
        .stdout(predicate::str::contains(r#""path":"src/main.rs""#)); // Data row
}

#[test]
fn test_export_redaction() {
    let mut cmd = tokmd_cmd();
    cmd.arg("export")
        .arg("--redact")
        .arg("paths")
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs").not());
}

#[test]
fn test_ignore_file_respected() {
    let mut cmd = tokmd_cmd();
    cmd.arg("export")
        .assert()
        .success()
        .stdout(predicate::str::contains(".ignored").not());
}

#[test]
fn test_ignore_override() {
    let mut cmd = tokmd_cmd();
    cmd.arg("export")
        .arg("--no-ignore")
        .assert()
        .success()
        .stdout(predicate::str::contains(".ignored"));
}
