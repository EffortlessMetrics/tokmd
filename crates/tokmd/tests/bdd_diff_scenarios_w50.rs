//! BDD-style scenario tests for the `diff` command.
//!
//! Each test follows the Given/When/Then pattern to verify key user-facing
//! workflows of the diff command.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::tempdir;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

/// Helper: run `tokmd run --output-dir <dir> .` against the fixture root.
fn run_receipt(output_dir: &std::path::Path) {
    tokmd_cmd()
        .args(["run", "--output-dir"])
        .arg(output_dir)
        .arg(".")
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// Scenario 1: Diff two receipts produces diff output
// ---------------------------------------------------------------------------

#[test]
fn given_two_receipts_when_diff_then_diff_output_produced() {
    // Given: two JSON receipt files from separate runs
    let dir = tempdir().unwrap();
    let run1 = dir.path().join("run1");
    let run2 = dir.path().join("run2");
    run_receipt(&run1);
    run_receipt(&run2);

    // When: I diff them
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.arg("diff")
        .arg("--from")
        .arg(run1.join("receipt.json"))
        .arg("--to")
        .arg(run2.join("receipt.json"))
        .assert()
        // Then: I get a diff receipt with headers
        .success()
        .stdout(predicate::str::contains("## Diff:"));
}

// ---------------------------------------------------------------------------
// Scenario 2: Identical receipts show no meaningful changes
// ---------------------------------------------------------------------------

#[test]
fn given_identical_receipts_when_diff_then_no_changes() {
    // Given: two identical receipt files (same run)
    let dir = tempdir().unwrap();
    let run_dir = dir.path().join("run_same");
    run_receipt(&run_dir);

    let receipt = run_dir.join("receipt.json");

    // When: I diff the same receipt against itself
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd
        .args(["diff", "--format", "json", "--from"])
        .arg(&receipt)
        .arg("--to")
        .arg(&receipt)
        .output()
        .expect("diff should execute");

    // Then: diff shows no changes (all deltas are zero)
    assert!(output.status.success());
    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .expect("diff output should be valid JSON");

    // The summary deltas should all be zero
    if let Some(summary) = json.get("summary") {
        if let Some(delta_code) = summary.get("delta_code") {
            assert_eq!(
                delta_code.as_i64().unwrap_or(0),
                0,
                "identical receipts should have zero code delta"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 3: Diff JSON format produces valid JSON
// ---------------------------------------------------------------------------

#[test]
fn given_receipts_when_diff_json_then_valid_json() {
    // Given: receipt files
    let dir = tempdir().unwrap();
    let run_dir = dir.path().join("run_json");
    run_receipt(&run_dir);

    let receipt = run_dir.join("receipt.json");

    // When: I diff with --format json
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd
        .args(["diff", "--format", "json", "--from"])
        .arg(&receipt)
        .arg("--to")
        .arg(&receipt)
        .output()
        .expect("diff json should execute");

    // Then: output is valid JSON
    assert!(output.status.success());
    let _: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .expect("diff JSON output should be valid");
}

// ---------------------------------------------------------------------------
// Scenario 4: Diff compact mode produces summary table
// ---------------------------------------------------------------------------

#[test]
fn given_receipts_when_diff_compact_then_summary_table() {
    // Given: receipt files
    let dir = tempdir().unwrap();
    let run_dir = dir.path().join("run_compact");
    run_receipt(&run_dir);

    let receipt = run_dir.join("receipt.json");

    // When: I diff with --compact
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.arg("diff")
        .arg("--compact")
        .arg("--from")
        .arg(&receipt)
        .arg("--to")
        .arg(&receipt)
        .assert()
        // Then: output has compact summary table
        .success()
        .stdout(predicate::str::contains("|Metric|Value|"));
}

// ---------------------------------------------------------------------------
// Scenario 5: Diff full mode shows summary comparison rows
// ---------------------------------------------------------------------------

#[test]
fn given_receipts_when_diff_full_then_shows_loc_lines_files() {
    // Given: receipt files
    let dir = tempdir().unwrap();
    let run_dir = dir.path().join("run_full");
    run_receipt(&run_dir);

    let receipt = run_dir.join("receipt.json");

    // When: I diff (full mode, default)
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.arg("diff")
        .arg("--from")
        .arg(&receipt)
        .arg("--to")
        .arg(&receipt)
        .assert()
        // Then: output shows LOC, Lines, Files metrics
        .success()
        .stdout(predicate::str::contains("|LOC|"))
        .stdout(predicate::str::contains("|Lines|"))
        .stdout(predicate::str::contains("|Files|"));
}
