//! W71 deep baseline CLI integration tests.
//!
//! Tests cover: metrics field validation, custom output paths, error cases,
//! determinism, commit field, empty project, and force-overwrite semantics.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

/// Helper: run baseline and return parsed JSON.
fn run_baseline(extra: &[&str]) -> serde_json::Value {
    let dir = tempdir().unwrap();
    let out_file = dir.path().join("baseline.json");

    let mut cmd = tokmd_cmd();
    cmd.arg("--no-progress")
        .arg("baseline")
        .arg("--output")
        .arg(&out_file)
        .arg("--force");
    for a in extra {
        cmd.arg(a);
    }
    cmd.assert().success();

    let content = fs::read_to_string(&out_file).unwrap();
    serde_json::from_str(&content).unwrap()
}

// ===========================================================================
// 1. Metrics field validation
// ===========================================================================

#[test]
fn baseline_metrics_has_total_files() {
    let parsed = run_baseline(&[]);
    let total = parsed["metrics"]["total_files"].as_u64();
    assert!(total.is_some(), "metrics should have total_files");
    assert!(total.unwrap() > 0, "fixture should have at least one file");
}

#[test]
fn baseline_metrics_has_function_count() {
    let parsed = run_baseline(&[]);
    assert!(
        parsed["metrics"]["function_count"].is_number(),
        "metrics should have function_count"
    );
}

#[test]
fn baseline_metrics_has_avg_cyclomatic() {
    let parsed = run_baseline(&[]);
    assert!(
        parsed["metrics"]["avg_cyclomatic"].is_number(),
        "metrics should have avg_cyclomatic"
    );
}

#[test]
fn baseline_metrics_has_max_cyclomatic() {
    let parsed = run_baseline(&[]);
    assert!(
        parsed["metrics"]["max_cyclomatic"].is_number(),
        "metrics should have max_cyclomatic"
    );
}

#[test]
fn baseline_metrics_avg_le_max_cyclomatic() {
    let parsed = run_baseline(&[]);
    let avg = parsed["metrics"]["avg_cyclomatic"].as_f64().unwrap_or(0.0);
    let max = parsed["metrics"]["max_cyclomatic"].as_f64().unwrap_or(0.0);
    assert!(
        avg <= max,
        "avg_cyclomatic ({avg}) should be <= max_cyclomatic ({max})"
    );
}

// ===========================================================================
// 2. Custom output path
// ===========================================================================

#[test]
fn baseline_custom_output_path() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let custom_path = dir.path().join("subdir").join("my_baseline.json");

    // Parent dir must exist for the command to write
    fs::create_dir_all(custom_path.parent().unwrap())?;

    tokmd_cmd()
        .arg("--no-progress")
        .arg("baseline")
        .arg("--output")
        .arg(&custom_path)
        .arg("--force")
        .assert()
        .success();

    assert!(custom_path.exists(), "baseline should be written to custom path");

    let content = fs::read_to_string(&custom_path)?;
    let parsed: serde_json::Value = serde_json::from_str(&content)?;
    assert_eq!(parsed["baseline_version"].as_u64(), Some(1));
    Ok(())
}

// ===========================================================================
// 3. Force overwrite semantics
// ===========================================================================

#[test]
fn baseline_without_force_on_existing_file_fails() {
    let dir = tempdir().unwrap();
    let out_file = dir.path().join("baseline.json");

    // First run with --force
    tokmd_cmd()
        .arg("--no-progress")
        .arg("baseline")
        .arg("--output")
        .arg(&out_file)
        .arg("--force")
        .assert()
        .success();

    assert!(out_file.exists());

    // Second run without --force should fail
    tokmd_cmd()
        .arg("--no-progress")
        .arg("baseline")
        .arg("--output")
        .arg(&out_file)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("exists")
                .or(predicate::str::contains("--force"))
                .or(predicate::str::contains("overwrite")),
        );
}

#[test]
fn baseline_force_overwrites_existing() {
    let dir = tempdir().unwrap();
    let out_file = dir.path().join("baseline.json");

    // First run
    tokmd_cmd()
        .arg("--no-progress")
        .arg("baseline")
        .arg("--output")
        .arg(&out_file)
        .arg("--force")
        .assert()
        .success();

    // Second run with --force should succeed
    tokmd_cmd()
        .arg("--no-progress")
        .arg("baseline")
        .arg("--output")
        .arg(&out_file)
        .arg("--force")
        .assert()
        .success();
}

// ===========================================================================
// 4. Determinism
// ===========================================================================

#[test]
fn baseline_deterministic_metrics() {
    let run1 = run_baseline(&[]);
    let run2 = run_baseline(&[]);

    assert_eq!(
        run1["metrics"]["total_files"],
        run2["metrics"]["total_files"],
        "total_files should be deterministic"
    );
    assert_eq!(
        run1["metrics"]["function_count"],
        run2["metrics"]["function_count"],
        "function_count should be deterministic"
    );
}

// ===========================================================================
// 5. Commit field
// ===========================================================================

#[test]
fn baseline_has_commit_field() {
    let parsed = run_baseline(&[]);
    // commit field may be null if fixture has no real git history,
    // but the key should be present
    assert!(
        parsed.get("commit").is_some(),
        "baseline should have commit field"
    );
}

// ===========================================================================
// 6. Empty project
// ===========================================================================

#[test]
fn baseline_empty_project() {
    let dir = tempdir().unwrap();
    let empty = dir.path().join("empty_proj");
    fs::create_dir_all(&empty).unwrap();
    fs::create_dir_all(empty.join(".git")).unwrap();

    let out_file = dir.path().join("empty_baseline.json");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(&empty)
        .arg("--no-progress")
        .arg("baseline")
        .arg(&empty)
        .arg("--output")
        .arg(&out_file)
        .arg("--force")
        .assert()
        .success();

    let content = fs::read_to_string(&out_file).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed["baseline_version"].as_u64(), Some(1));
    assert_eq!(parsed["metrics"]["total_files"].as_u64(), Some(0));
}

// ===========================================================================
// 7. Baseline version field
// ===========================================================================

#[test]
fn baseline_version_is_one() {
    let parsed = run_baseline(&[]);
    assert_eq!(parsed["baseline_version"].as_u64(), Some(1));
}

// ===========================================================================
// 8. Determinism flag (feature-gated)
// ===========================================================================

#[test]
#[cfg(feature = "git")]
fn baseline_determinism_flag_source_hash_format() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let out_file = dir.path().join("det_baseline.json");

    tokmd_cmd()
        .arg("--no-progress")
        .arg("baseline")
        .arg("--determinism")
        .arg("--output")
        .arg(&out_file)
        .arg("--force")
        .assert()
        .success();

    let content = fs::read_to_string(&out_file)?;
    let parsed: serde_json::Value = serde_json::from_str(&content)?;

    let det = parsed
        .get("determinism")
        .expect("determinism section should be present");

    // source_hash should be 64 hex chars (BLAKE3)
    let hash = det["source_hash"].as_str().unwrap();
    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

    // generated_at should be present
    assert!(det["generated_at"].is_string());

    Ok(())
}

#[test]
#[cfg(feature = "git")]
fn baseline_determinism_flag_deterministic_hash() {
    let get_hash = || {
        let dir = tempdir().unwrap();
        let out_file = dir.path().join("det.json");

        tokmd_cmd()
            .arg("--no-progress")
            .arg("baseline")
            .arg("--determinism")
            .arg("--output")
            .arg(&out_file)
            .arg("--force")
            .assert()
            .success();

        let content = fs::read_to_string(&out_file).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        parsed["determinism"]["source_hash"]
            .as_str()
            .unwrap()
            .to_string()
    };

    let hash1 = get_hash();
    let hash2 = get_hash();
    assert_eq!(hash1, hash2, "source_hash should be deterministic");
}

// ===========================================================================
// 9. Metrics non-negative
// ===========================================================================

#[test]
fn baseline_metrics_values_non_negative() {
    let parsed = run_baseline(&[]);
    let m = &parsed["metrics"];
    assert!(m["total_files"].as_u64().is_some(), "total_files should be a non-negative integer");
    assert!(m["function_count"].as_u64().is_some(), "function_count should be a non-negative integer");
    assert!(m["avg_cyclomatic"].as_f64().unwrap_or(0.0) >= 0.0);
    assert!(m["max_cyclomatic"].as_f64().unwrap_or(0.0) >= 0.0);
}
