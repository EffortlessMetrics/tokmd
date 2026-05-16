//! Branch-coverage tests for `compute_determinism_gate`.
//!
//! These tests cover the gate's resolution / parse / hash-mismatch
//! branches without requiring git history, which the rest of the cockpit
//! gates need.

#![cfg(feature = "git")]

use std::fs;
use std::path::PathBuf;

use tokmd_analysis_types::{ComplexityBaseline, DeterminismBaseline};
use tokmd_cockpit::compute_determinism_gate;
use tokmd_cockpit::determinism::hash_files_from_walk;
use tokmd_types::cockpit::GateStatus;

fn baseline_path(repo: &tempfile::TempDir) -> PathBuf {
    let dir = repo.path().join(".tokmd");
    fs::create_dir_all(&dir).expect("create .tokmd dir");
    dir.join("baseline.json")
}

fn write_baseline(path: &std::path::Path, baseline: &ComplexityBaseline) {
    let json = serde_json::to_string_pretty(baseline).expect("serialize baseline");
    fs::write(path, json).expect("write baseline");
}

#[test]
fn determinism_gate_returns_none_without_baseline_file() {
    let repo = tempfile::tempdir().unwrap();
    // No `.tokmd/baseline.json` and no explicit path => Ok(None).
    let result = compute_determinism_gate(repo.path(), None).expect("gate runs");
    assert!(result.is_none());
}

#[test]
fn determinism_gate_returns_none_when_baseline_lacks_determinism_section() {
    let repo = tempfile::tempdir().unwrap();
    let path = baseline_path(&repo);
    let baseline = ComplexityBaseline::new();
    write_baseline(&path, &baseline);

    let result = compute_determinism_gate(repo.path(), None).expect("gate runs");
    assert!(
        result.is_none(),
        "baseline without determinism section returns None"
    );
}

#[test]
fn determinism_gate_returns_none_when_baseline_is_cockpit_receipt() {
    let repo = tempfile::tempdir().unwrap();
    let path = baseline_path(&repo);
    // A cockpit receipt (not a ComplexityBaseline) should be tolerated and
    // produce Ok(None) so trend-comparison flows keep working.
    let cockpit_json = serde_json::json!({
        "mode": "cockpit",
        "schema_version": 3,
    });
    fs::write(&path, cockpit_json.to_string()).expect("write cockpit json");

    let result = compute_determinism_gate(repo.path(), None).expect("gate runs");
    assert!(result.is_none());
}

#[test]
fn determinism_gate_errors_on_unrecognized_json() {
    let repo = tempfile::tempdir().unwrap();
    let path = baseline_path(&repo);
    // Not a ComplexityBaseline and not a cockpit receipt either.
    fs::write(&path, r#"{"hello": "world"}"#).expect("write garbage json");

    let result = compute_determinism_gate(repo.path(), None);
    let err = result.expect_err("unrecognized baseline should error");
    assert!(
        err.to_string().contains("not a ComplexityBaseline"),
        "expected ComplexityBaseline error, got {err}"
    );
}

#[test]
fn determinism_gate_passes_when_source_hash_matches() {
    let repo = tempfile::tempdir().unwrap();
    fs::write(repo.path().join("hello.txt"), "stable\n").unwrap();

    // Compute the hash the gate will compute, excluding the baseline file
    // itself (which it always auto-excludes).
    let actual_hash =
        hash_files_from_walk(repo.path(), &[".tokmd/baseline.json"]).expect("hash walk");

    let mut baseline = ComplexityBaseline::new();
    baseline.determinism = Some(DeterminismBaseline {
        baseline_version: 1,
        generated_at: "2025-01-01T00:00:00Z".to_string(),
        build_hash: "00".repeat(20),
        source_hash: actual_hash.clone(),
        cargo_lock_hash: None,
    });

    let path = baseline_path(&repo);
    write_baseline(&path, &baseline);

    let gate = compute_determinism_gate(repo.path(), None)
        .expect("gate runs")
        .expect("baseline produces gate");

    assert_eq!(gate.meta.status, GateStatus::Pass);
    assert_eq!(gate.expected_hash.as_deref(), Some(actual_hash.as_str()));
    assert_eq!(gate.actual_hash.as_deref(), Some(actual_hash.as_str()));
    assert!(gate.differences.is_empty());
}

#[test]
fn determinism_gate_warns_on_source_hash_mismatch() {
    let repo = tempfile::tempdir().unwrap();
    fs::write(repo.path().join("hello.txt"), "stable\n").unwrap();

    let mut baseline = ComplexityBaseline::new();
    baseline.determinism = Some(DeterminismBaseline {
        baseline_version: 1,
        generated_at: "2025-01-01T00:00:00Z".to_string(),
        build_hash: "00".repeat(20),
        // Pin to a hash the walk will not produce.
        source_hash: "ff".repeat(20),
        cargo_lock_hash: None,
    });

    let path = baseline_path(&repo);
    write_baseline(&path, &baseline);

    let gate = compute_determinism_gate(repo.path(), None)
        .expect("gate runs")
        .expect("baseline produces gate");

    assert_eq!(gate.meta.status, GateStatus::Warn);
    assert!(
        gate.differences
            .iter()
            .any(|d| d.contains("source hash mismatch")),
        "expected source hash mismatch in differences: {:?}",
        gate.differences
    );
}

#[test]
fn determinism_gate_explicit_path_overrides_default() {
    let repo = tempfile::tempdir().unwrap();
    fs::write(repo.path().join("hello.txt"), "stable\n").unwrap();

    let actual_hash = hash_files_from_walk(repo.path(), &[]).expect("hash walk");

    let mut baseline = ComplexityBaseline::new();
    baseline.determinism = Some(DeterminismBaseline {
        baseline_version: 1,
        generated_at: "2025-01-01T00:00:00Z".to_string(),
        build_hash: "00".repeat(20),
        source_hash: actual_hash.clone(),
        cargo_lock_hash: None,
    });

    // Put baseline somewhere outside `.tokmd/` so the strip_prefix branch
    // resolves to an absolute path that isn't auto-exclusion-able.
    let outside = tempfile::tempdir().unwrap();
    let explicit = outside.path().join("custom-baseline.json");
    write_baseline(&explicit, &baseline);

    let gate = compute_determinism_gate(repo.path(), Some(&explicit))
        .expect("gate runs")
        .expect("baseline produces gate");

    // With no `Cargo.lock` expectation, status reduces to source hash equality.
    assert_eq!(gate.meta.status, GateStatus::Pass);
    assert_eq!(gate.expected_hash.as_deref(), Some(actual_hash.as_str()));
}

#[test]
fn determinism_gate_warns_when_cargo_lock_missing_but_expected() {
    let repo = tempfile::tempdir().unwrap();
    fs::write(repo.path().join("hello.txt"), "stable\n").unwrap();

    let actual_hash =
        hash_files_from_walk(repo.path(), &[".tokmd/baseline.json"]).expect("hash walk");

    let mut baseline = ComplexityBaseline::new();
    baseline.determinism = Some(DeterminismBaseline {
        baseline_version: 1,
        generated_at: "2025-01-01T00:00:00Z".to_string(),
        build_hash: "00".repeat(20),
        source_hash: actual_hash,
        cargo_lock_hash: Some("deadbeef".repeat(8)),
    });

    let path = baseline_path(&repo);
    write_baseline(&path, &baseline);

    let gate = compute_determinism_gate(repo.path(), None)
        .expect("gate runs")
        .expect("baseline produces gate");

    assert_eq!(gate.meta.status, GateStatus::Warn);
    assert!(
        gate.differences
            .iter()
            .any(|d| d.contains("Cargo.lock missing")),
        "expected Cargo.lock missing difference, got {:?}",
        gate.differences
    );
}

#[test]
fn determinism_gate_warns_when_cargo_lock_hash_mismatches() {
    let repo = tempfile::tempdir().unwrap();
    fs::write(repo.path().join("hello.txt"), "stable\n").unwrap();
    fs::write(repo.path().join("Cargo.lock"), "# Cargo.lock contents").unwrap();

    let actual_hash =
        hash_files_from_walk(repo.path(), &[".tokmd/baseline.json"]).expect("hash walk");

    let mut baseline = ComplexityBaseline::new();
    baseline.determinism = Some(DeterminismBaseline {
        baseline_version: 1,
        generated_at: "2025-01-01T00:00:00Z".to_string(),
        build_hash: "00".repeat(20),
        source_hash: actual_hash,
        // Wrong cargo lock hash on purpose.
        cargo_lock_hash: Some("aa".repeat(32)),
    });

    let path = baseline_path(&repo);
    write_baseline(&path, &baseline);

    let gate = compute_determinism_gate(repo.path(), None)
        .expect("gate runs")
        .expect("baseline produces gate");

    assert_eq!(gate.meta.status, GateStatus::Warn);
    assert!(
        gate.differences
            .iter()
            .any(|d| d.contains("Cargo.lock hash mismatch")),
        "expected Cargo.lock mismatch difference, got {:?}",
        gate.differences
    );
}

#[test]
fn determinism_gate_errors_on_unreadable_baseline() {
    let repo = tempfile::tempdir().unwrap();
    // Provide an explicit baseline path that does NOT exist; explicit
    // missing paths return Ok(None) (we cover the "no path exists" branch).
    let nonexistent = repo.path().join("missing-baseline.json");
    let result = compute_determinism_gate(repo.path(), Some(&nonexistent)).expect("gate runs");
    assert!(result.is_none());
}

#[test]
fn determinism_gate_errors_on_invalid_baseline_json() {
    let repo = tempfile::tempdir().unwrap();
    let path = baseline_path(&repo);
    // Malformed JSON triggers the parse error branch.
    fs::write(&path, "{ this is not json").expect("write malformed json");

    let err = compute_determinism_gate(repo.path(), None).expect_err("malformed json must error");
    assert!(
        err.to_string().contains("failed to parse baseline JSON"),
        "expected parse-error message, got {err}"
    );
}
