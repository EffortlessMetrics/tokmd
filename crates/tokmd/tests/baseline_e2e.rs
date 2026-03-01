//! End-to-end tests for the `tokmd baseline` CLI command.

use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::path::PathBuf;

/// Build a `tokmd` command rooted at the test fixtures directory.
fn tokmd() -> Command {
    let mut cmd: Command = cargo_bin_cmd!("tokmd");
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data");
    cmd.current_dir(&fixtures);
    cmd
}

/// Build a `tokmd` command rooted at a custom directory.
fn tokmd_at(dir: &std::path::Path) -> Command {
    let mut cmd: Command = cargo_bin_cmd!("tokmd");
    cmd.current_dir(dir);
    cmd
}

// ── Helper: write sample Rust source files into a directory ──────────

fn write_sample_files(dir: &std::path::Path) {
    std::fs::write(
        dir.join("main.rs"),
        r#"fn main() {
    let x = 1;
    if x > 0 {
        println!("positive");
    } else {
        println!("non-positive");
    }
}

fn helper(a: i32, b: i32) -> i32 {
    if a > b {
        a
    } else {
        b
    }
}
"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("lib.rs"),
        r#"pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn is_even(n: i32) -> bool {
    n % 2 == 0
}
"#,
    )
    .unwrap();
}

// ── 1. Baseline on temp dir with sample files ────────────────────────

#[test]
fn baseline_on_temp_dir_with_sample_files() {
    let tmp = tempfile::tempdir().unwrap();
    write_sample_files(tmp.path());

    let output_path = tmp.path().join("baseline.json");
    tokmd_at(tmp.path())
        .arg("baseline")
        .arg("--output")
        .arg(&output_path)
        .assert()
        .success();

    assert!(output_path.exists(), "baseline file should be created");
    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(!content.is_empty());
}

// ── 2. Baseline output is valid JSON ─────────────────────────────────

#[test]
fn baseline_output_is_valid_json() {
    let tmp = tempfile::tempdir().unwrap();
    write_sample_files(tmp.path());

    let output_path = tmp.path().join("baseline.json");
    tokmd_at(tmp.path())
        .arg("baseline")
        .arg("--output")
        .arg(&output_path)
        .assert()
        .success();

    let content = std::fs::read_to_string(&output_path).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("baseline output should be valid JSON");
    assert!(parsed.is_object());
}

// ── 3. JSON has expected fields ──────────────────────────────────────

#[test]
fn baseline_json_has_expected_fields() {
    let tmp = tempfile::tempdir().unwrap();
    write_sample_files(tmp.path());

    let output_path = tmp.path().join("baseline.json");
    tokmd_at(tmp.path())
        .arg("baseline")
        .arg("--output")
        .arg(&output_path)
        .assert()
        .success();

    let content = std::fs::read_to_string(&output_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Top-level fields
    assert!(
        json["baseline_version"].is_number(),
        "should have baseline_version"
    );
    assert!(
        json["generated_at"].is_string(),
        "should have generated_at timestamp"
    );
    assert!(json["metrics"].is_object(), "should have metrics object");

    // Metrics fields
    let metrics = &json["metrics"];
    assert!(
        metrics["total_files"].is_number(),
        "metrics should have total_files"
    );
    assert!(
        metrics["total_code_lines"].is_number(),
        "metrics should have total_code_lines"
    );
    assert!(
        metrics["avg_cyclomatic"].is_number(),
        "metrics should have avg_cyclomatic"
    );
    assert!(
        metrics["max_cyclomatic"].is_number(),
        "metrics should have max_cyclomatic"
    );
    assert!(
        metrics["function_count"].is_number(),
        "metrics should have function_count"
    );
}

// ── 4. Baseline on empty directory ───────────────────────────────────

#[test]
fn baseline_on_empty_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let output_path = tmp.path().join("baseline.json");

    // Empty directory should still succeed (zero counts)
    tokmd_at(tmp.path())
        .arg("baseline")
        .arg("--output")
        .arg(&output_path)
        .assert()
        .success();

    assert!(output_path.exists());
    let content = std::fs::read_to_string(&output_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Metrics should be zero
    assert_eq!(json["metrics"]["total_files"], 0);
    assert_eq!(json["metrics"]["total_code_lines"], 0);
    assert_eq!(json["metrics"]["function_count"], 0);
}

// ── 5. Baseline with explicit path argument ──────────────────────────

#[test]
fn baseline_with_explicit_path_argument() {
    let tmp = tempfile::tempdir().unwrap();
    let src_dir = tmp.path().join("src");
    std::fs::create_dir(&src_dir).unwrap();
    write_sample_files(&src_dir);

    let output_path = tmp.path().join("baseline.json");

    // Pass the source directory as the positional argument
    tokmd_at(tmp.path())
        .arg("baseline")
        .arg(&src_dir)
        .arg("--output")
        .arg(&output_path)
        .assert()
        .success();

    assert!(output_path.exists());
    let content = std::fs::read_to_string(&output_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(json["metrics"]["total_files"].as_u64().unwrap() > 0);
}

// ── 6. Baseline output contains complexity metrics ───────────────────

#[test]
fn baseline_contains_complexity_metrics() {
    let tmp = tempfile::tempdir().unwrap();
    write_sample_files(tmp.path());

    let output_path = tmp.path().join("baseline.json");
    tokmd_at(tmp.path())
        .arg("baseline")
        .arg("--output")
        .arg(&output_path)
        .assert()
        .success();

    let content = std::fs::read_to_string(&output_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    let metrics = &json["metrics"];
    // With sample files containing if/else, we should see non-trivial metrics
    assert!(
        metrics["total_code_lines"].as_u64().unwrap() > 0,
        "should have code lines"
    );
    assert!(
        metrics["avg_cyclomatic"].is_number(),
        "should have avg_cyclomatic"
    );
    assert!(
        metrics["avg_cognitive"].is_number(),
        "should have avg_cognitive"
    );
    assert!(
        metrics["max_nesting_depth"].is_number(),
        "should have max_nesting_depth"
    );
    assert!(
        metrics["avg_function_length"].is_number(),
        "should have avg_function_length"
    );
}

// ── 7. Two runs produce same results (determinism) ───────────────────

#[test]
fn baseline_determinism_two_runs_same_result() {
    let tmp = tempfile::tempdir().unwrap();
    write_sample_files(tmp.path());

    let output1 = tmp.path().join("baseline1.json");
    let output2 = tmp.path().join("baseline2.json");

    tokmd_at(tmp.path())
        .arg("baseline")
        .arg("--output")
        .arg(&output1)
        .assert()
        .success();

    tokmd_at(tmp.path())
        .arg("baseline")
        .arg("--output")
        .arg(&output2)
        .assert()
        .success();

    let json1: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&output1).unwrap()).unwrap();
    let json2: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&output2).unwrap()).unwrap();

    // Metrics and files should be identical between runs
    assert_eq!(json1["metrics"], json2["metrics"], "metrics should match");
    assert_eq!(json1["files"], json2["files"], "file entries should match");
    assert_eq!(
        json1["baseline_version"], json2["baseline_version"],
        "version should match"
    );
}

// ── 8. Invalid path fails gracefully ─────────────────────────────────

#[test]
fn baseline_invalid_path_fails_gracefully() {
    let tmp = tempfile::tempdir().unwrap();
    let output_path = tmp.path().join("baseline.json");

    tokmd()
        .arg("baseline")
        .arg("this/path/does/not/exist/at/all")
        .arg("--output")
        .arg(&output_path)
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

// ── 9. Force overwrite existing baseline ─────────────────────────────

#[test]
fn baseline_force_overwrite() {
    let tmp = tempfile::tempdir().unwrap();
    write_sample_files(tmp.path());

    let output_path = tmp.path().join("baseline.json");

    // First run
    tokmd_at(tmp.path())
        .arg("baseline")
        .arg("--output")
        .arg(&output_path)
        .assert()
        .success();

    // Second run without --force should fail
    tokmd_at(tmp.path())
        .arg("baseline")
        .arg("--output")
        .arg(&output_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));

    // Third run with --force should succeed
    tokmd_at(tmp.path())
        .arg("baseline")
        .arg("--output")
        .arg(&output_path)
        .arg("--force")
        .assert()
        .success();
}

// ── 10. Baseline with test fixtures ──────────────────────────────────

#[test]
fn baseline_on_test_fixtures() {
    let tmp = tempfile::tempdir().unwrap();
    let output_path = tmp.path().join("baseline.json");

    tokmd()
        .arg("baseline")
        .arg("--output")
        .arg(&output_path)
        .assert()
        .success();

    let content = std::fs::read_to_string(&output_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Fixtures have source files, so we should see non-zero counts
    assert!(json["metrics"]["total_files"].as_u64().unwrap() > 0);
    assert!(json["baseline_version"].as_u64().unwrap() >= 1);
}
