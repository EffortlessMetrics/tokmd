//! Integration tests for the `tokmd gate` command.

use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
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
        .stderr(predicate::str::contains("No policy or ratchet rules specified"));
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
    // New structure has policy.rule_results instead of flat rule_results
    assert!(parsed.get("policy").is_some());
    assert!(parsed["policy"].get("rule_results").is_some());
    assert!(parsed.get("total_errors").is_some());
    assert!(parsed.get("total_warnings").is_some());
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
    // New structure uses total_warnings and total_errors
    assert_eq!(parsed["total_warnings"], 1);
    assert_eq!(parsed["total_errors"], 0);
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

// =============================================================================
// Ratchet Tests
// =============================================================================

/// Create a baseline JSON file for ratchet tests.
fn create_test_baseline(dir: &TempDir) -> std::path::PathBuf {
    let baseline = serde_json::json!({
        "baseline_version": 1,
        "generated_at": "1700000000:000000000",
        "metrics": {
            "total_code_lines": 5000,
            "total_files": 50,
            "avg_cyclomatic": 5.0,
            "max_cyclomatic": 20,
            "avg_cognitive": 3.0,
            "max_cognitive": 15,
            "avg_nesting_depth": 2.5,
            "max_nesting_depth": 6,
            "function_count": 100,
            "avg_function_length": 25.0
        },
        "files": [],
        "complexity": {
            "total_functions": 100,
            "avg_function_length": 25.0,
            "max_function_length": 150,
            "avg_cyclomatic": 5.0,
            "max_cyclomatic": 20,
            "avg_cognitive": 3.0,
            "max_cognitive": 15,
            "avg_nesting_depth": 2.5,
            "max_nesting_depth": 6,
            "high_risk_files": 5
        }
    });

    let path = dir.path().join("baseline.json");
    fs::write(&path, serde_json::to_string_pretty(&baseline).unwrap()).unwrap();
    path
}

/// Create a current receipt for ratchet tests (slight increase from baseline).
fn create_current_receipt_slight_increase(dir: &TempDir) -> std::path::PathBuf {
    let receipt = serde_json::json!({
        "schema_version": 4,
        "complexity": {
            "total_functions": 105,
            "avg_function_length": 26.0,
            "max_function_length": 155,
            "avg_cyclomatic": 5.2,  // 4% increase (under 10% threshold)
            "max_cyclomatic": 21,
            "avg_cognitive": 3.1,
            "max_cognitive": 16,
            "avg_nesting_depth": 2.6,
            "max_nesting_depth": 6,
            "high_risk_files": 5
        },
        "derived": {
            "totals": {
                "tokens": 100000,
                "code": 5200,
                "files": 52
            }
        }
    });

    let path = dir.path().join("current.json");
    fs::write(&path, serde_json::to_string_pretty(&receipt).unwrap()).unwrap();
    path
}

/// Create a current receipt for ratchet tests (significant increase from baseline).
fn create_current_receipt_large_increase(dir: &TempDir) -> std::path::PathBuf {
    let receipt = serde_json::json!({
        "schema_version": 4,
        "complexity": {
            "total_functions": 150,
            "avg_function_length": 35.0,
            "max_function_length": 200,
            "avg_cyclomatic": 7.5,  // 50% increase (over 10% threshold)
            "max_cyclomatic": 35,
            "avg_cognitive": 5.0,
            "max_cognitive": 25,
            "avg_nesting_depth": 4.0,
            "max_nesting_depth": 10,
            "high_risk_files": 12
        },
        "derived": {
            "totals": {
                "tokens": 150000,
                "code": 7500,
                "files": 75
            }
        }
    });

    let path = dir.path().join("current.json");
    fs::write(&path, serde_json::to_string_pretty(&receipt).unwrap()).unwrap();
    path
}

/// Create a ratchet config file.
fn create_ratchet_config(dir: &TempDir) -> std::path::PathBuf {
    let config = r#"
fail_fast = false
allow_missing_baseline = false
allow_missing_current = false

[[rules]]
pointer = "/complexity/avg_cyclomatic"
max_increase_pct = 10.0
description = "Average cyclomatic complexity"
level = "error"

[[rules]]
pointer = "/complexity/max_cyclomatic"
max_value = 30.0
description = "Max cyclomatic complexity ceiling"
level = "error"
"#;

    let path = dir.path().join("ratchet.toml");
    fs::write(&path, config).unwrap();
    path
}

#[test]
fn test_gate_ratchet_passing() {
    let dir = TempDir::new().unwrap();
    let baseline = create_test_baseline(&dir);
    let current = create_current_receipt_slight_increase(&dir);
    let ratchet = create_ratchet_config(&dir);

    tokmd()
        .args([
            "gate",
            current.to_str().unwrap(),
            "--baseline",
            baseline.to_str().unwrap(),
            "--ratchet-config",
            ratchet.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("PASSED"))
        .stdout(predicate::str::contains("Ratchet Rules"));
}

#[test]
fn test_gate_ratchet_failing_percentage() {
    let dir = TempDir::new().unwrap();
    let baseline = create_test_baseline(&dir);
    let current = create_current_receipt_large_increase(&dir);
    let ratchet = create_ratchet_config(&dir);

    tokmd()
        .args([
            "gate",
            current.to_str().unwrap(),
            "--baseline",
            baseline.to_str().unwrap(),
            "--ratchet-config",
            ratchet.to_str().unwrap(),
        ])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("FAILED"))
        .stdout(predicate::str::contains("exceeds"));
}

#[test]
fn test_gate_ratchet_failing_max_value() {
    let dir = TempDir::new().unwrap();
    let baseline = create_test_baseline(&dir);
    let current = create_current_receipt_large_increase(&dir);
    let ratchet = create_ratchet_config(&dir);

    // Current has max_cyclomatic = 35, which exceeds the max_value = 30 ceiling
    tokmd()
        .args([
            "gate",
            current.to_str().unwrap(),
            "--baseline",
            baseline.to_str().unwrap(),
            "--ratchet-config",
            ratchet.to_str().unwrap(),
        ])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("exceeds maximum"));
}

#[test]
fn test_gate_ratchet_json_output() {
    let dir = TempDir::new().unwrap();
    let baseline = create_test_baseline(&dir);
    let current = create_current_receipt_slight_increase(&dir);
    let ratchet = create_ratchet_config(&dir);

    let output = tokmd()
        .args([
            "gate",
            current.to_str().unwrap(),
            "--baseline",
            baseline.to_str().unwrap(),
            "--ratchet-config",
            ratchet.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("Should be valid JSON");

    assert!(parsed.get("passed").is_some());
    assert!(parsed.get("ratchet").is_some());
    assert!(parsed["ratchet"].get("ratchet_results").is_some());
    assert!(parsed.get("total_errors").is_some());
    assert!(parsed.get("total_warnings").is_some());
}

#[test]
fn test_gate_ratchet_warn_level() {
    let dir = TempDir::new().unwrap();
    let baseline = create_test_baseline(&dir);
    let current = create_current_receipt_large_increase(&dir);

    // Ratchet config with warn level
    let config = r#"
[[rules]]
pointer = "/complexity/avg_cyclomatic"
max_increase_pct = 10.0
description = "Average cyclomatic complexity"
level = "warn"
"#;

    let ratchet_path = dir.path().join("ratchet_warn.toml");
    fs::write(&ratchet_path, config).unwrap();

    // Should pass (exit 0) because warnings don't fail the gate
    let output = tokmd()
        .args([
            "gate",
            current.to_str().unwrap(),
            "--baseline",
            baseline.to_str().unwrap(),
            "--ratchet-config",
            ratchet_path.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Warnings should not cause failure");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(parsed["passed"], true);
    assert_eq!(parsed["total_warnings"], 1);
    assert_eq!(parsed["total_errors"], 0);
}

#[test]
fn test_gate_combined_policy_and_ratchet() {
    let dir = TempDir::new().unwrap();
    let baseline = create_test_baseline(&dir);
    let current = create_current_receipt_slight_increase(&dir);
    let policy = create_passing_policy(&dir);
    let ratchet = create_ratchet_config(&dir);

    // Both policy and ratchet should pass
    tokmd()
        .args([
            "gate",
            current.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--baseline",
            baseline.to_str().unwrap(),
            "--ratchet-config",
            ratchet.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("PASSED"))
        .stdout(predicate::str::contains("Policy Rules"))
        .stdout(predicate::str::contains("Ratchet Rules"));
}

#[test]
fn test_gate_ratchet_no_baseline() {
    let dir = TempDir::new().unwrap();
    let current = create_current_receipt_slight_increase(&dir);
    let ratchet = create_ratchet_config(&dir);

    // Ratchet without baseline should error
    tokmd()
        .args([
            "gate",
            current.to_str().unwrap(),
            "--ratchet-config",
            ratchet.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No policy or ratchet rules"));
}
