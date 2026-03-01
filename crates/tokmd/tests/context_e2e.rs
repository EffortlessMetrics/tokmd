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

// ---------------------------------------------------------------------------
// Mode: list (default)
// ---------------------------------------------------------------------------

#[test]
fn test_context_list_mode() {
    let mut cmd = tokmd_cmd();
    cmd.arg("context")
        .arg("--mode")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"));
}

#[test]
fn test_context_default_mode_is_list() {
    // Without --mode, default should behave like list
    let mut cmd = tokmd_cmd();
    cmd.arg("context")
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"));
}

// ---------------------------------------------------------------------------
// Mode: json
// ---------------------------------------------------------------------------

#[test]
fn test_context_json_mode() {
    let mut cmd = tokmd_cmd();
    let output = cmd
        .arg("context")
        .arg("--mode")
        .arg("json")
        .output()
        .expect("failed to run tokmd context --mode json");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("context JSON output should be valid JSON");

    assert_eq!(parsed["schema_version"].as_u64(), Some(4));
    assert_eq!(parsed["mode"].as_str(), Some("context"));
    assert!(parsed["budget_tokens"].is_number());
    assert!(parsed["used_tokens"].is_number());
    assert!(parsed["utilization_pct"].is_number());
    assert!(parsed["file_count"].is_number());
    assert!(parsed["files"].is_array());
    assert!(parsed["tool"]["name"].as_str() == Some("tokmd"));
    assert!(parsed["generated_at_ms"].is_number());

    let files = parsed["files"].as_array().unwrap();
    assert!(!files.is_empty(), "should include at least one file");
}

// ---------------------------------------------------------------------------
// Budget limiting
// ---------------------------------------------------------------------------

#[test]
fn test_context_budget_limiting() {
    let mut cmd = tokmd_cmd();
    let output = cmd
        .arg("context")
        .arg("--mode")
        .arg("json")
        .arg("--budget")
        .arg("1000")
        .output()
        .expect("failed to run tokmd context --budget 1000");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    let budget = parsed["budget_tokens"].as_u64().unwrap();
    let used = parsed["used_tokens"].as_u64().unwrap();

    assert_eq!(budget, 1000);
    assert!(
        used <= budget,
        "used_tokens ({}) should not exceed budget_tokens ({})",
        used,
        budget
    );
}

// ---------------------------------------------------------------------------
// Mode: bundle (directory output)
// ---------------------------------------------------------------------------

#[test]
fn test_context_bundle_mode() {
    let mut cmd = tokmd_cmd();
    cmd.arg("context")
        .arg("--mode")
        .arg("bundle")
        .assert()
        .success()
        // Bundle mode writes concatenated file contents to stdout
        .stdout(predicate::str::is_empty().not());
}

// ---------------------------------------------------------------------------
// --output flag (write to file)
// ---------------------------------------------------------------------------

#[test]
fn test_context_output_to_file_json() {
    let dir = tempdir().unwrap();
    let out_file = dir.path().join("context_output.json");

    let mut cmd = tokmd_cmd();
    cmd.arg("context")
        .arg("--mode")
        .arg("json")
        .arg("--output")
        .arg(&out_file)
        .assert()
        .success();

    assert!(out_file.exists(), "output file should be created");

    let content = fs::read_to_string(&out_file).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("output file should contain valid JSON");

    assert_eq!(parsed["schema_version"].as_u64(), Some(4));
    assert_eq!(parsed["mode"].as_str(), Some("context"));
    assert!(parsed["files"].is_array());
}

#[test]
fn test_context_output_to_file_list() {
    let dir = tempdir().unwrap();
    let out_file = dir.path().join("context_list.md");

    let mut cmd = tokmd_cmd();
    cmd.arg("context")
        .arg("--mode")
        .arg("list")
        .arg("--output")
        .arg(&out_file)
        .assert()
        .success();

    assert!(out_file.exists(), "output file should be created");

    let content = fs::read_to_string(&out_file).unwrap();
    assert!(
        content.contains("src/main.rs"),
        "list output should contain fixture file"
    );
}

// ---------------------------------------------------------------------------
// Strategy flag
// ---------------------------------------------------------------------------

#[test]
fn test_context_strategy_greedy() {
    let mut cmd = tokmd_cmd();
    let output = cmd
        .arg("context")
        .arg("--mode")
        .arg("json")
        .arg("--strategy")
        .arg("greedy")
        .output()
        .unwrap();

    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();
    assert_eq!(parsed["strategy"].as_str(), Some("greedy"));
}

#[test]
fn test_context_strategy_spread() {
    let mut cmd = tokmd_cmd();
    let output = cmd
        .arg("context")
        .arg("--mode")
        .arg("json")
        .arg("--strategy")
        .arg("spread")
        .output()
        .unwrap();

    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();
    assert_eq!(parsed["strategy"].as_str(), Some("spread"));
}

// ---------------------------------------------------------------------------
// JSON receipt file rows have expected fields
// ---------------------------------------------------------------------------

#[test]
fn test_context_json_file_rows_have_required_fields() {
    let mut cmd = tokmd_cmd();
    let output = cmd
        .arg("context")
        .arg("--mode")
        .arg("json")
        .output()
        .unwrap();

    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();

    let files = parsed["files"].as_array().unwrap();
    assert!(!files.is_empty());

    let first = &files[0];
    assert!(first["path"].is_string(), "file row should have path");
    assert!(
        first["tokens"].is_number(),
        "file row should have token count"
    );
}

// ---------------------------------------------------------------------------
// Bundle directory output (--bundle-dir)
// ---------------------------------------------------------------------------

#[test]
fn test_context_bundle_dir_creates_artifacts() {
    let dir = tempdir().unwrap();
    let bundle_dir = dir.path().join("context_bundle");

    let mut cmd = tokmd_cmd();
    cmd.arg("context")
        .arg("--bundle-dir")
        .arg(&bundle_dir)
        .assert()
        .success();

    // Bundle dir should contain manifest.json and bundle.txt
    assert!(
        bundle_dir.join("manifest.json").exists(),
        "bundle dir should contain manifest.json"
    );
    assert!(
        bundle_dir.join("bundle.txt").exists(),
        "bundle dir should contain bundle.txt"
    );

    // manifest.json should be valid JSON
    let manifest_content = fs::read_to_string(bundle_dir.join("manifest.json")).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&manifest_content).expect("manifest.json should be valid JSON");

    assert_eq!(parsed["schema_version"].as_u64(), Some(2));
    assert!(parsed["budget_tokens"].is_number());
    assert!(parsed["used_tokens"].is_number());
    assert!(parsed["included_files"].is_array());
}

// ---------------------------------------------------------------------------
// Zero budget edge case
// ---------------------------------------------------------------------------

#[test]
fn test_context_zero_budget() {
    let mut cmd = tokmd_cmd();
    let output = cmd
        .arg("context")
        .arg("--mode")
        .arg("json")
        .arg("--budget")
        .arg("0")
        .output()
        .expect("failed to run tokmd context --budget 0");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(parsed["budget_tokens"].as_u64().unwrap(), 0);
    assert_eq!(parsed["used_tokens"].as_u64().unwrap(), 0);
    // With zero budget, files may still appear with head_tail policy and effective_tokens=0
    let files = parsed["files"].as_array().unwrap();
    for f in files {
        assert_eq!(
            f["effective_tokens"].as_u64().unwrap(),
            0,
            "all files should have 0 effective tokens with zero budget"
        );
    }
}

// ---------------------------------------------------------------------------
// Budget with k/m suffix
// ---------------------------------------------------------------------------

#[test]
fn test_context_budget_k_suffix() {
    let mut cmd = tokmd_cmd();
    let output = cmd
        .arg("context")
        .arg("--mode")
        .arg("json")
        .arg("--budget")
        .arg("2k")
        .output()
        .expect("failed to run tokmd context --budget 2k");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(parsed["budget_tokens"].as_u64().unwrap(), 2000);
}

// ---------------------------------------------------------------------------
// Subdirectory scanning
// ---------------------------------------------------------------------------

#[test]
fn test_context_subdirectory() {
    let subdir = common::fixture_root().join("src");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(&subdir);

    let output = cmd
        .arg("context")
        .arg("--mode")
        .arg("json")
        .output()
        .expect("failed to run tokmd context in subdirectory");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert!(parsed["file_count"].as_u64().unwrap() >= 1);
}

// ---------------------------------------------------------------------------
// List mode contains table headers
// ---------------------------------------------------------------------------

#[test]
fn test_context_list_contains_table_headers() {
    let mut cmd = tokmd_cmd();
    cmd.arg("context")
        .arg("--mode")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("# Context Pack"))
        .stdout(predicate::str::contains("|Path|Module|Lang|Tokens|Code|"))
        .stdout(predicate::str::contains("|---|---|---|---:|---:|"));
}

// ---------------------------------------------------------------------------
// List mode with small budget
// ---------------------------------------------------------------------------

#[test]
fn test_context_list_with_small_budget() {
    let mut cmd = tokmd_cmd();
    cmd.arg("context")
        .arg("--mode")
        .arg("list")
        .arg("--budget")
        .arg("500")
        .assert()
        .success()
        .stdout(predicate::str::contains("Budget: 500 tokens"));
}

// ---------------------------------------------------------------------------
// Bundle mode includes file markers
// ---------------------------------------------------------------------------

#[test]
fn test_context_bundle_includes_file_markers() {
    let mut cmd = tokmd_cmd();
    cmd.arg("context")
        .arg("--mode")
        .arg("bundle")
        .assert()
        .success()
        .stdout(predicate::str::contains("// ==="));
}

// ---------------------------------------------------------------------------
// JSON receipt includes token_estimation
// ---------------------------------------------------------------------------

#[test]
fn test_context_json_has_token_estimation() {
    let mut cmd = tokmd_cmd();
    let output = cmd
        .arg("context")
        .arg("--mode")
        .arg("json")
        .output()
        .expect("failed to run tokmd context --mode json");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    let te = &parsed["token_estimation"];
    assert!(te.is_object(), "should have token_estimation object");
    assert!(te.get("source_bytes").is_some());
}
