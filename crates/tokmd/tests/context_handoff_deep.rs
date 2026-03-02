//! Deep integration tests for context and handoff CLI pipelines.
//!
//! Covers: context JSON receipt structure, bundle assembly, handoff manifest
//! validation, determinism (same input → identical file selection), budget
//! enforcement, and edge cases.

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

// ===========================================================================
// 1. Context JSON receipt: structure validation
// ===========================================================================

#[test]
fn context_json_receipt_has_all_required_fields() {
    let output = tokmd_cmd()
        .args(["context", "--mode", "json"])
        .output()
        .expect("context json should succeed");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Top-level fields
    assert_eq!(parsed["schema_version"].as_u64(), Some(4));
    assert!(parsed["generated_at_ms"].is_number());
    assert_eq!(parsed["tool"]["name"].as_str(), Some("tokmd"));
    assert_eq!(parsed["mode"].as_str(), Some("context"));
    assert!(parsed["budget_tokens"].is_number());
    assert!(parsed["used_tokens"].is_number());
    assert!(parsed["utilization_pct"].is_number());
    assert!(parsed["strategy"].is_string());
    assert!(parsed["rank_by"].is_string());
    assert!(parsed["file_count"].is_number());
    assert!(parsed["files"].is_array());
}

#[test]
fn context_json_file_rows_have_complete_fields() {
    let output = tokmd_cmd()
        .args(["context", "--mode", "json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();

    let files = parsed["files"].as_array().unwrap();
    assert!(!files.is_empty());

    for file in files {
        assert!(file["path"].is_string(), "file should have path");
        assert!(file["module"].is_string(), "file should have module");
        assert!(file["lang"].is_string(), "file should have lang");
        assert!(file["tokens"].is_number(), "file should have tokens");
        assert!(file["code"].is_number(), "file should have code");
        assert!(file["lines"].is_number(), "file should have lines");
        assert!(file["bytes"].is_number(), "file should have bytes");
        assert!(file["value"].is_number(), "file should have value");
    }
}

#[test]
fn context_json_file_paths_use_forward_slashes() {
    let output = tokmd_cmd()
        .args(["context", "--mode", "json"])
        .output()
        .unwrap();

    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();

    let files = parsed["files"].as_array().unwrap();
    for file in files {
        let path = file["path"].as_str().unwrap();
        assert!(
            !path.contains('\\'),
            "path should use forward slashes: {path}"
        );
    }
}

// ===========================================================================
// 2. Budget enforcement: used_tokens ≤ budget_tokens
// ===========================================================================

#[test]
fn context_budget_is_respected_with_small_budget() {
    let output = tokmd_cmd()
        .args(["context", "--mode", "json", "--budget", "500"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();

    let budget = parsed["budget_tokens"].as_u64().unwrap();
    let used = parsed["used_tokens"].as_u64().unwrap();

    assert_eq!(budget, 500);
    assert!(
        used <= budget,
        "used ({used}) must not exceed budget ({budget})"
    );
}

#[test]
fn context_budget_is_respected_with_large_budget() {
    let output = tokmd_cmd()
        .args(["context", "--mode", "json", "--budget", "1m"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();

    let budget = parsed["budget_tokens"].as_u64().unwrap();
    let used = parsed["used_tokens"].as_u64().unwrap();

    assert_eq!(budget, 1_000_000);
    assert!(used <= budget);
}

#[test]
fn context_utilization_pct_is_consistent() {
    let output = tokmd_cmd()
        .args(["context", "--mode", "json", "--budget", "10000"])
        .output()
        .unwrap();

    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();

    let budget = parsed["budget_tokens"].as_f64().unwrap();
    let used = parsed["used_tokens"].as_f64().unwrap();
    let pct = parsed["utilization_pct"].as_f64().unwrap();

    if budget > 0.0 {
        let expected_pct = (used / budget) * 100.0;
        assert!(
            (pct - expected_pct).abs() < 0.1,
            "utilization_pct ({pct}) should be ~{expected_pct}"
        );
    }
}

// ===========================================================================
// 3. Context bundle assembly
// ===========================================================================

#[test]
fn context_bundle_dir_creates_manifest_and_bundle() {
    let dir = tempdir().unwrap();
    let bundle_dir = dir.path().join("ctx_bundle");

    tokmd_cmd()
        .args(["context", "--bundle-dir"])
        .arg(&bundle_dir)
        .assert()
        .success();

    assert!(bundle_dir.join("manifest.json").exists());
    assert!(bundle_dir.join("bundle.txt").exists());
}

#[test]
fn context_bundle_manifest_has_valid_schema() {
    let dir = tempdir().unwrap();
    let bundle_dir = dir.path().join("ctx_schema");

    tokmd_cmd()
        .args(["context", "--bundle-dir"])
        .arg(&bundle_dir)
        .assert()
        .success();

    let manifest = fs::read_to_string(bundle_dir.join("manifest.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&manifest).unwrap();

    assert_eq!(parsed["schema_version"].as_u64(), Some(2));
    assert!(parsed["generated_at_ms"].is_number());
    assert!(parsed["budget_tokens"].is_number());
    assert!(parsed["used_tokens"].is_number());
    assert!(parsed["utilization_pct"].is_number());
    assert!(parsed["file_count"].is_number());
    assert!(parsed["bundle_bytes"].is_number());
    assert!(parsed["included_files"].is_array());
}

#[test]
fn context_bundle_included_files_match_file_count() {
    let dir = tempdir().unwrap();
    let bundle_dir = dir.path().join("ctx_count");

    tokmd_cmd()
        .args(["context", "--bundle-dir"])
        .arg(&bundle_dir)
        .assert()
        .success();

    let manifest = fs::read_to_string(bundle_dir.join("manifest.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&manifest).unwrap();

    let file_count = parsed["file_count"].as_u64().unwrap();
    let included_files = parsed["included_files"].as_array().unwrap();
    assert_eq!(
        file_count,
        included_files.len() as u64,
        "file_count should match included_files length"
    );
}

#[test]
fn context_bundle_budget_respected() {
    let dir = tempdir().unwrap();
    let bundle_dir = dir.path().join("ctx_budget");

    tokmd_cmd()
        .args(["context", "--bundle-dir"])
        .arg(&bundle_dir)
        .args(["--budget", "1000"])
        .assert()
        .success();

    let manifest = fs::read_to_string(bundle_dir.join("manifest.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&manifest).unwrap();

    let budget = parsed["budget_tokens"].as_u64().unwrap();
    let used = parsed["used_tokens"].as_u64().unwrap();
    assert!(used <= budget);
}

// ===========================================================================
// 4. Handoff manifest structure validation
// ===========================================================================

#[test]
fn handoff_manifest_has_all_required_fields() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("hoff_fields");

    tokmd_cmd()
        .args(["handoff", "--out-dir"])
        .arg(&out_dir)
        .assert()
        .success();

    let manifest = fs::read_to_string(out_dir.join("manifest.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&manifest).unwrap();

    assert_eq!(parsed["schema_version"].as_u64(), Some(5));
    assert!(parsed["generated_at_ms"].is_number());
    assert_eq!(parsed["tool"]["name"].as_str(), Some("tokmd"));
    assert_eq!(parsed["mode"].as_str(), Some("handoff"));
    assert!(parsed["inputs"].is_array());
    assert!(parsed["output_dir"].is_string());
    assert!(parsed["budget_tokens"].is_number());
    assert!(parsed["used_tokens"].is_number());
    assert!(parsed["utilization_pct"].is_number());
    assert!(parsed["strategy"].is_string());
    assert!(parsed["rank_by"].is_string());
    assert!(parsed["capabilities"].is_array());
    assert!(parsed["artifacts"].is_array());
    assert!(parsed["included_files"].is_array());
    assert!(parsed["excluded_paths"].is_array());
    assert!(parsed["excluded_patterns"].is_array());
}

#[test]
fn handoff_artifacts_have_expected_names() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("hoff_artifacts");

    tokmd_cmd()
        .args(["handoff", "--out-dir"])
        .arg(&out_dir)
        .assert()
        .success();

    let manifest = fs::read_to_string(out_dir.join("manifest.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&manifest).unwrap();

    let artifacts = parsed["artifacts"].as_array().unwrap();
    let names: Vec<&str> = artifacts
        .iter()
        .map(|a| a["name"].as_str().unwrap())
        .collect();

    assert!(names.contains(&"map"), "should have map artifact");
    assert!(
        names.contains(&"intelligence"),
        "should have intelligence artifact"
    );
    assert!(names.contains(&"code"), "should have code artifact");
}

#[test]
fn handoff_artifact_map_has_blake3_hash() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("hoff_hash");

    tokmd_cmd()
        .args(["handoff", "--out-dir"])
        .arg(&out_dir)
        .assert()
        .success();

    let manifest = fs::read_to_string(out_dir.join("manifest.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&manifest).unwrap();

    let artifacts = parsed["artifacts"].as_array().unwrap();
    let map_artifact = artifacts.iter().find(|a| a["name"] == "map").unwrap();

    assert_eq!(map_artifact["hash"]["algo"].as_str(), Some("blake3"));
    let hash = map_artifact["hash"]["hash"].as_str().unwrap();
    assert!(!hash.is_empty(), "hash should not be empty");
    assert_eq!(hash.len(), 64, "blake3 hex hash should be 64 chars");
}

#[test]
fn handoff_produces_all_four_files() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("hoff_files");

    tokmd_cmd()
        .args(["handoff", "--out-dir"])
        .arg(&out_dir)
        .assert()
        .success();

    assert!(out_dir.join("manifest.json").exists());
    assert!(out_dir.join("map.jsonl").exists());
    assert!(out_dir.join("intelligence.json").exists());
    assert!(out_dir.join("code.txt").exists());
}

#[test]
fn handoff_map_jsonl_lines_are_valid_json() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("hoff_jsonl");

    tokmd_cmd()
        .args(["handoff", "--out-dir"])
        .arg(&out_dir)
        .assert()
        .success();

    let map = fs::read_to_string(out_dir.join("map.jsonl")).unwrap();
    for (i, line) in map.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        serde_json::from_str::<serde_json::Value>(line)
            .unwrap_or_else(|e| panic!("Invalid JSON on line {}: {e}", i + 1));
    }
}

#[test]
fn handoff_budget_enforcement() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("hoff_budget");

    tokmd_cmd()
        .args(["handoff", "--out-dir"])
        .arg(&out_dir)
        .args(["--budget", "1k"])
        .assert()
        .success();

    let manifest = fs::read_to_string(out_dir.join("manifest.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&manifest).unwrap();

    let budget = parsed["budget_tokens"].as_u64().unwrap();
    let used = parsed["used_tokens"].as_u64().unwrap();
    assert!(
        used <= budget,
        "used ({used}) must not exceed budget ({budget})"
    );
}

#[test]
fn handoff_included_files_paths_use_forward_slashes() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("hoff_paths");

    tokmd_cmd()
        .args(["handoff", "--out-dir"])
        .arg(&out_dir)
        .assert()
        .success();

    let manifest = fs::read_to_string(out_dir.join("manifest.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&manifest).unwrap();

    let included = parsed["included_files"].as_array().unwrap();
    for file in included {
        let path = file["path"].as_str().unwrap();
        assert!(
            !path.contains('\\'),
            "path should use forward slashes: {path}"
        );
    }
}

// ===========================================================================
// 5. Determinism: repeated runs produce identical file selection
// ===========================================================================

#[test]
fn context_json_determinism_same_file_selection() {
    let get_files = || {
        let output = tokmd_cmd()
            .args(["context", "--mode", "json", "--budget", "5000"])
            .output()
            .unwrap();

        let parsed: serde_json::Value =
            serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();

        let files = parsed["files"].as_array().unwrap().clone();
        let paths: Vec<String> = files
            .iter()
            .map(|f| f["path"].as_str().unwrap().to_string())
            .collect();
        paths
    };

    let run1 = get_files();
    let run2 = get_files();
    assert_eq!(run1, run2, "file selection should be deterministic");
}

#[test]
fn context_json_determinism_same_token_counts() {
    let get_receipt = || {
        let output = tokmd_cmd()
            .args(["context", "--mode", "json", "--budget", "10000"])
            .output()
            .unwrap();

        let parsed: serde_json::Value =
            serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();

        let used = parsed["used_tokens"].as_u64().unwrap();
        let file_count = parsed["file_count"].as_u64().unwrap();
        (used, file_count)
    };

    let (used1, count1) = get_receipt();
    let (used2, count2) = get_receipt();
    assert_eq!(used1, used2, "used_tokens should be deterministic");
    assert_eq!(count1, count2, "file_count should be deterministic");
}

#[test]
fn handoff_determinism_same_manifest_structure() {
    let get_manifest = || {
        let dir = tempdir().unwrap();
        let out_dir = dir.path().join("hoff_det");

        tokmd_cmd()
            .args(["handoff", "--out-dir"])
            .arg(&out_dir)
            .args(["--budget", "5k"])
            .assert()
            .success();

        let manifest = fs::read_to_string(out_dir.join("manifest.json")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&manifest).unwrap();

        let files: Vec<String> = parsed["included_files"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["path"].as_str().unwrap().to_string())
            .collect();

        let used = parsed["used_tokens"].as_u64().unwrap();
        (files, used)
    };

    let (files1, used1) = get_manifest();
    let (files2, used2) = get_manifest();
    assert_eq!(
        files1, files2,
        "handoff file selection should be deterministic"
    );
    assert_eq!(used1, used2, "handoff used_tokens should be deterministic");
}

// ===========================================================================
// 6. Strategy variations
// ===========================================================================

#[test]
fn context_greedy_strategy_records_strategy_field() {
    let output = tokmd_cmd()
        .args(["context", "--mode", "json", "--strategy", "greedy"])
        .output()
        .unwrap();

    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();
    assert_eq!(parsed["strategy"].as_str(), Some("greedy"));
}

#[test]
fn context_spread_strategy_records_strategy_field() {
    let output = tokmd_cmd()
        .args(["context", "--mode", "json", "--strategy", "spread"])
        .output()
        .unwrap();

    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();
    assert_eq!(parsed["strategy"].as_str(), Some("spread"));
}

#[test]
fn handoff_greedy_strategy_records_strategy_field() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("hoff_greedy");

    tokmd_cmd()
        .args(["handoff", "--out-dir"])
        .arg(&out_dir)
        .args(["--strategy", "greedy"])
        .assert()
        .success();

    let manifest = fs::read_to_string(out_dir.join("manifest.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&manifest).unwrap();
    assert_eq!(parsed["strategy"].as_str(), Some("greedy"));
}

// ===========================================================================
// 7. Edge cases
// ===========================================================================

#[test]
fn context_with_very_small_budget_still_succeeds() {
    tokmd_cmd()
        .args(["context", "--mode", "json", "--budget", "1"])
        .assert()
        .success();
}

#[test]
fn context_with_very_small_budget_respects_limit() {
    let output = tokmd_cmd()
        .args(["context", "--mode", "json", "--budget", "1"])
        .output()
        .unwrap();

    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();

    let used = parsed["used_tokens"].as_u64().unwrap();
    assert!(used <= 1, "used_tokens ({used}) should be ≤ 1");
}

#[test]
fn handoff_with_no_git_flag_succeeds_and_marks_capability() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("hoff_nogit");

    tokmd_cmd()
        .args(["handoff", "--out-dir"])
        .arg(&out_dir)
        .arg("--no-git")
        .assert()
        .success();

    let manifest = fs::read_to_string(out_dir.join("manifest.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&manifest).unwrap();

    let caps = parsed["capabilities"].as_array().unwrap();
    let git_cap = caps.iter().find(|c| c["name"] == "git").unwrap();
    assert_eq!(git_cap["status"].as_str(), Some("skipped"));
}

#[test]
fn context_output_to_file_writes_valid_json() {
    let dir = tempdir().unwrap();
    let out_file = dir.path().join("ctx_out.json");

    tokmd_cmd()
        .args(["context", "--mode", "json", "--output"])
        .arg(&out_file)
        .assert()
        .success();

    let content = fs::read_to_string(&out_file).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed["schema_version"].as_u64(), Some(4));
}

#[test]
fn handoff_force_flag_allows_overwrite() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("hoff_force");

    // First run
    tokmd_cmd()
        .args(["handoff", "--out-dir"])
        .arg(&out_dir)
        .assert()
        .success();

    // Second run without --force fails
    tokmd_cmd()
        .args(["handoff", "--out-dir"])
        .arg(&out_dir)
        .assert()
        .failure();

    // Third run with --force succeeds
    tokmd_cmd()
        .args(["handoff", "--out-dir"])
        .arg(&out_dir)
        .arg("--force")
        .assert()
        .success();
}

#[test]
fn context_bundle_mode_outputs_to_stdout() {
    tokmd_cmd()
        .args(["context", "--mode", "bundle"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ===========================================================================
// 8. Intelligence file structure (handoff)
// ===========================================================================

#[test]
fn handoff_intelligence_json_has_expected_fields() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("hoff_intel");

    tokmd_cmd()
        .args(["handoff", "--out-dir"])
        .arg(&out_dir)
        .assert()
        .success();

    let intel = fs::read_to_string(out_dir.join("intelligence.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&intel).unwrap();

    // Intelligence fields
    assert!(parsed["tree"].is_string());
    assert!(parsed["tree_depth"].is_number());
    assert!(parsed["warnings"].is_array());

    // Should NOT have envelope fields (those belong to manifest)
    assert!(parsed.get("schema_version").is_none());
    assert!(parsed.get("generated_at_ms").is_none());
}

#[test]
fn handoff_code_txt_contains_file_markers() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("hoff_code");

    tokmd_cmd()
        .args(["handoff", "--out-dir"])
        .arg(&out_dir)
        .assert()
        .success();

    let code = fs::read_to_string(out_dir.join("code.txt")).unwrap();
    assert!(
        code.contains("// ==="),
        "code.txt should contain file separator markers"
    );
}
