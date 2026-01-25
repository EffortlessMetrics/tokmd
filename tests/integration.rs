use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::cargo_bin("tokmd").unwrap();
    // Point to our test fixture
    cmd.current_dir("tests/data");
    cmd
}

fn redact_timestamps(output: &str) -> String {
    let re = regex::Regex::new(r#""generated_at_ms":\d+"#).unwrap();
    re.replace_all(output, r#""generated_at_ms":0"#).to_string()
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
        .stdout(predicate::str::contains("ignored.rs").not());
}

#[test]
fn test_ignore_override() {
    let mut cmd = tokmd_cmd();
    cmd.arg("--no-ignore")
        .arg("export")
        .assert()
        .success()
        .stdout(predicate::str::contains("ignored.rs"));
}

#[test]
fn test_golden_lang_json() {
    let mut cmd = tokmd_cmd();
    let output = cmd
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    let normalized = redact_timestamps(&stdout);
    
    insta::assert_snapshot!(normalized);
}

#[test]
fn test_golden_module_json() {
    let mut cmd = tokmd_cmd();
    let output = cmd
        .arg("module")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    let normalized = redact_timestamps(&stdout);
    
    insta::assert_snapshot!(normalized);
}

#[test]
fn test_golden_export_jsonl() {
    let mut cmd = tokmd_cmd();
    let output = cmd
        .arg("export")
        .output()
        .unwrap();
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    let normalized = redact_timestamps(&stdout);
    
    insta::assert_snapshot!(normalized);
}

#[test]
fn test_golden_export_redacted() {
    let mut cmd = tokmd_cmd();
    let output = cmd
        .arg("export")
        .arg("--redact")
        .arg("all")
        .output()
        .unwrap();
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    let normalized = redact_timestamps(&stdout);
    
    insta::assert_snapshot!(normalized);
}
