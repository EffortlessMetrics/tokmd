//! Integration tests for the `tokmd gate` command.

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn tokmd() -> Command {
    cargo_bin_cmd!("tokmd")
}

/// Create a test receipt JSON file.
fn create_test_receipt(dir: &TempDir) -> std::path::PathBuf {
    let receipt = serde_json::json!({
        "schema_version": 2,
        "derived": {
            "totals": {
                "tokens": 100000,
                "code": 5000,
                "files": 50
            },
            "doc_density": {
                "total": {
                    "ratio": 0.15
                }
            }
        },
        "license": {
            "effective": "MIT"
        }
    });

    let path = dir.path().join("receipt.json");
    fs::write(&path, serde_json::to_string_pretty(&receipt).unwrap()).unwrap();
    path
}

/// Create a passing policy file.
fn create_passing_policy(dir: &TempDir) -> std::path::PathBuf {
    let policy = r#"
fail_fast = false

[[rules]]
name = "max_tokens"
pointer = "/derived/totals/tokens"
op = "lte"
value = 500000
level = "error"
message = "Codebase exceeds token budget"

[[rules]]
name = "min_code"
pointer = "/derived/totals/code"
op = "gte"
value = 100
level = "error"
"#;

    let path = dir.path().join("policy.toml");
    fs::write(&path, policy).unwrap();
    path
}

/// Create a failing policy file.
fn create_failing_policy(dir: &TempDir) -> std::path::PathBuf {
    let policy = r#"
fail_fast = false

[[rules]]
name = "max_tokens"
pointer = "/derived/totals/tokens"
op = "lte"
value = 1000
level = "error"
message = "Token budget exceeded"
"#;

    let path = dir.path().join("policy.toml");
    fs::write(&path, policy).unwrap();
    path
}

#[test]
fn test_gate_requires_policy() {
    // Gate without --policy should error
    tokmd()
        .args(["gate"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No policy specified"));
}

#[test]
fn test_gate_passing_policy() {
    let dir = TempDir::new().unwrap();
    let receipt = create_test_receipt(&dir);
    let policy = create_passing_policy(&dir);

    tokmd()
        .args([
            "gate",
            receipt.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("PASSED"));
}

#[test]
fn test_gate_failing_policy() {
    let dir = TempDir::new().unwrap();
    let receipt = create_test_receipt(&dir);
    let policy = create_failing_policy(&dir);

    tokmd()
        .args([
            "gate",
            receipt.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
        ])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("FAILED"));
}

#[test]
fn test_gate_json_output() {
    let dir = TempDir::new().unwrap();
    let receipt = create_test_receipt(&dir);
    let policy = create_passing_policy(&dir);

    let output = tokmd()
        .args([
            "gate",
            receipt.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("Should be valid JSON");

    assert!(parsed.get("passed").is_some());
    assert!(parsed.get("rule_results").is_some());
    assert!(parsed.get("errors").is_some());
    assert!(parsed.get("warnings").is_some());
}

#[test]
fn test_gate_invalid_policy_file() {
    let dir = TempDir::new().unwrap();
    let receipt = create_test_receipt(&dir);
    let policy_path = dir.path().join("nonexistent.toml");

    tokmd()
        .args([
            "gate",
            receipt.to_str().unwrap(),
            "--policy",
            policy_path.to_str().unwrap(),
        ])
        .assert()
        .failure();
}

#[test]
fn test_gate_operators() {
    let dir = TempDir::new().unwrap();
    let receipt = create_test_receipt(&dir);

    // Test various operators
    let policy = r#"
fail_fast = false

[[rules]]
name = "gt_test"
pointer = "/derived/totals/tokens"
op = "gt"
value = 50000
level = "error"

[[rules]]
name = "lt_test"
pointer = "/derived/totals/tokens"
op = "lt"
value = 500000
level = "error"

[[rules]]
name = "eq_test"
pointer = "/derived/totals/files"
op = "eq"
value = 50
level = "error"

[[rules]]
name = "exists_test"
pointer = "/license/effective"
op = "exists"
level = "error"

[[rules]]
name = "in_test"
pointer = "/license/effective"
op = "in"
values = ["MIT", "Apache-2.0", "BSD-3-Clause"]
level = "error"
"#;

    let policy_path = dir.path().join("operators.toml");
    fs::write(&policy_path, policy).unwrap();

    tokmd()
        .args([
            "gate",
            receipt.to_str().unwrap(),
            "--policy",
            policy_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("PASSED"));
}

#[test]
fn test_gate_warn_level() {
    let dir = TempDir::new().unwrap();
    let receipt = create_test_receipt(&dir);

    // Policy with only warn-level rules that fail
    let policy = r#"
[[rules]]
name = "warn_test"
pointer = "/derived/totals/tokens"
op = "lte"
value = 1000
level = "warn"
message = "Token count high"
"#;

    let policy_path = dir.path().join("warn.toml");
    fs::write(&policy_path, policy).unwrap();

    // Should pass (exit 0) because warnings don't fail the gate
    let output = tokmd()
        .args([
            "gate",
            receipt.to_str().unwrap(),
            "--policy",
            policy_path.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Warnings should not cause failure");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(parsed["passed"], true);
    assert_eq!(parsed["warnings"], 1);
    assert_eq!(parsed["errors"], 0);
}

#[test]
fn test_gate_negate() {
    let dir = TempDir::new().unwrap();
    let receipt = create_test_receipt(&dir);

    // Test negate - "secrets" should NOT exist
    let policy = r#"
[[rules]]
name = "no_secrets"
pointer = "/secrets"
op = "exists"
negate = true
level = "error"
"#;

    let policy_path = dir.path().join("negate.toml");
    fs::write(&policy_path, policy).unwrap();

    tokmd()
        .args([
            "gate",
            receipt.to_str().unwrap(),
            "--policy",
            policy_path.to_str().unwrap(),
        ])
        .assert()
        .success();
}
