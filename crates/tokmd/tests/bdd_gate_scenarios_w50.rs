#![cfg(feature = "analysis")]

//! BDD-style scenario tests for the `gate` command.
//!
//! Each test follows the Given/When/Then pattern to verify key user-facing
//! workflows of the CI gate command, specifically around policies and ratchets.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::{TempDir, tempdir};

fn tokmd_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
}

// Helper: create a baseline receipt
fn create_baseline(dir: &TempDir) -> std::path::PathBuf {
    let receipt = serde_json::json!({
        "schema_version": 4,
        "complexity": {
            "avg_cyclomatic": 5.0,
            "max_cyclomatic": 20
        }
    });
    let path = dir.path().join("baseline.json");
    fs::write(&path, serde_json::to_string(&receipt).unwrap()).unwrap();
    path
}

// Helper: create a current receipt with higher complexity
fn create_current_high(dir: &TempDir) -> std::path::PathBuf {
    let receipt = serde_json::json!({
        "schema_version": 4,
        "complexity": {
            "avg_cyclomatic": 7.0, // > 10% increase from 5.0
            "max_cyclomatic": 25
        }
    });
    let path = dir.path().join("current.json");
    fs::write(&path, serde_json::to_string(&receipt).unwrap()).unwrap();
    path
}

// Helper: create a current receipt with lower complexity
fn create_current_low(dir: &TempDir) -> std::path::PathBuf {
    let receipt = serde_json::json!({
        "schema_version": 4,
        "complexity": {
            "avg_cyclomatic": 4.5,
            "max_cyclomatic": 18
        }
    });
    let path = dir.path().join("current.json");
    fs::write(&path, serde_json::to_string(&receipt).unwrap()).unwrap();
    path
}

// Helper: create a strict ratchet config
fn create_ratchet(dir: &TempDir) -> std::path::PathBuf {
    let config = r#"
[[rules]]
pointer = "/complexity/avg_cyclomatic"
max_increase_pct = 10.0
level = "error"
"#;
    let path = dir.path().join("ratchet.toml");
    fs::write(&path, config).unwrap();
    path
}

// ---------------------------------------------------------------------------
// Scenario 1: Ratchet passing
// ---------------------------------------------------------------------------

#[test]
fn given_improved_metrics_when_gate_evaluated_then_gate_passes() {
    // Given: A baseline receipt and a new receipt with improved (lower) complexity
    let dir = tempdir().unwrap();
    let baseline = create_baseline(&dir);
    let current = create_current_low(&dir);
    let ratchet = create_ratchet(&dir);

    // When: I run the gate command with a 10% tolerance ratchet
    tokmd_cmd()
        .args([
            "gate",
            current.to_str().unwrap(),
            "--baseline",
            baseline.to_str().unwrap(),
            "--ratchet-config",
            ratchet.to_str().unwrap(),
        ])
        .assert()
        // Then: The gate succeeds and outputs PASSED
        .success()
        .stdout(predicate::str::contains("PASSED"));
}

// ---------------------------------------------------------------------------
// Scenario 2: Ratchet failing
// ---------------------------------------------------------------------------

#[test]
fn given_degraded_metrics_when_gate_evaluated_then_gate_fails() {
    // Given: A baseline receipt and a new receipt with degraded complexity exceeding the 10% threshold
    let dir = tempdir().unwrap();
    let baseline = create_baseline(&dir);
    let current = create_current_high(&dir);
    let ratchet = create_ratchet(&dir);

    // When: I run the gate command
    tokmd_cmd()
        .args([
            "gate",
            current.to_str().unwrap(),
            "--baseline",
            baseline.to_str().unwrap(),
            "--ratchet-config",
            ratchet.to_str().unwrap(),
        ])
        .assert()
        // Then: The gate fails with a non-zero exit code and outputs FAILED
        .failure()
        .stdout(predicate::str::contains("FAILED"))
        .stdout(predicate::str::contains("exceeds maximum allowed increase"));
}

// ---------------------------------------------------------------------------
// Scenario 3: Missing baseline with fail_fast=false
// ---------------------------------------------------------------------------

#[test]
fn given_missing_baseline_when_gate_evaluated_then_it_errors() {
    // Given: A current receipt and a ratchet config, but NO baseline
    let dir = tempdir().unwrap();
    let current = create_current_low(&dir);
    let ratchet = create_ratchet(&dir);

    // When: I run the gate command without providing a baseline
    tokmd_cmd()
        .args([
            "gate",
            current.to_str().unwrap(),
            "--ratchet-config",
            ratchet.to_str().unwrap(),
        ])
        .assert()
        // Then: It fails gracefully explaining that ratchet rules require a baseline
        .failure()
        .stderr(predicate::str::contains("No policy or ratchet rules"));
}
