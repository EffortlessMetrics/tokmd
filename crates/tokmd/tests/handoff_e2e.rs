//! E2E integration tests for `tokmd handoff` CLI command.
//!
//! These tests exercise the handoff subcommand end-to-end, verifying artifact
//! creation, JSON schema compliance, preset selection, and error handling.

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
// Basic output: artifacts exist and manifest is valid JSON
// ---------------------------------------------------------------------------

#[test]
fn handoff_creates_output_dir_with_valid_json_manifest() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_e2e");

    tokmd_cmd()
        .arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--no-git")
        .assert()
        .success();

    // All four artifacts must exist
    assert!(out_dir.join("manifest.json").exists());
    assert!(out_dir.join("map.jsonl").exists());
    assert!(out_dir.join("intelligence.json").exists());
    assert!(out_dir.join("code.txt").exists());

    // manifest.json must be valid JSON
    let content = fs::read_to_string(out_dir.join("manifest.json")).unwrap();
    let _: serde_json::Value =
        serde_json::from_str(&content).expect("manifest.json should be valid JSON");
}

// ---------------------------------------------------------------------------
// Schema field verification
// ---------------------------------------------------------------------------

#[test]
fn handoff_manifest_has_expected_schema_fields() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_schema");

    tokmd_cmd()
        .arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--no-git")
        .assert()
        .success();

    let content = fs::read_to_string(out_dir.join("manifest.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Required top-level fields
    assert!(
        parsed["schema_version"].is_number(),
        "manifest should have schema_version"
    );
    assert_eq!(
        parsed["schema_version"].as_u64(),
        Some(5),
        "schema_version should be 5"
    );
    assert!(
        parsed["generated_at_ms"].is_number(),
        "manifest should have generated_at_ms"
    );
    assert_eq!(
        parsed["tool"]["name"].as_str(),
        Some("tokmd"),
        "tool.name should be 'tokmd'"
    );
    assert_eq!(
        parsed["mode"].as_str(),
        Some("handoff"),
        "mode should be 'handoff'"
    );
    assert!(
        parsed["budget_tokens"].is_number(),
        "manifest should have budget_tokens"
    );
    assert!(
        parsed["used_tokens"].is_number(),
        "manifest should have used_tokens"
    );
    assert!(
        parsed["output_dir"].is_string(),
        "manifest should have output_dir"
    );

    // Collection fields
    assert!(
        parsed["capabilities"].is_array(),
        "manifest should have capabilities array"
    );
    assert!(
        parsed["artifacts"].is_array(),
        "manifest should have artifacts array"
    );
    assert!(
        parsed["included_files"].is_array(),
        "manifest should have included_files array"
    );
    assert!(
        parsed["excluded_paths"].is_array(),
        "manifest should have excluded_paths array"
    );
    assert!(
        parsed["excluded_patterns"].is_array(),
        "manifest should have excluded_patterns array"
    );
}

#[test]
fn handoff_manifest_artifacts_have_integrity_hashes() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_hashes");

    tokmd_cmd()
        .arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--no-git")
        .assert()
        .success();

    let content = fs::read_to_string(out_dir.join("manifest.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

    let artifacts = parsed["artifacts"].as_array().unwrap();
    assert!(!artifacts.is_empty(), "artifacts should not be empty");

    let map_artifact = artifacts.iter().find(|a| a["name"] == "map").unwrap();
    assert_eq!(
        map_artifact["hash"]["algo"].as_str(),
        Some("blake3"),
        "map artifact should use blake3 hash"
    );
    assert!(
        map_artifact["hash"]["hash"].is_string(),
        "map artifact should have a hash value"
    );
}

#[test]
fn handoff_included_files_reference_fixture_content() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_files");

    tokmd_cmd()
        .arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--no-git")
        .assert()
        .success();

    let content = fs::read_to_string(out_dir.join("manifest.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

    let files = parsed["included_files"].as_array().unwrap();
    assert!(!files.is_empty(), "included_files should not be empty");

    // At least one file from the fixture should have a path containing "main.rs"
    let has_main = files.iter().any(|f| {
        f["path"]
            .as_str()
            .map(|p| p.contains("main.rs"))
            .unwrap_or(false)
    });
    assert!(has_main, "included_files should reference main.rs");
}

// ---------------------------------------------------------------------------
// Preset selection: --preset minimal
// ---------------------------------------------------------------------------

#[test]
fn handoff_preset_minimal_produces_lighter_intelligence() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_preset_minimal");

    tokmd_cmd()
        .arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--preset")
        .arg("minimal")
        .arg("--no-git")
        .assert()
        .success();

    // All artifacts should still be created
    assert!(out_dir.join("manifest.json").exists());
    assert!(out_dir.join("intelligence.json").exists());

    let intel = fs::read_to_string(out_dir.join("intelligence.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&intel).unwrap();

    // Minimal preset: tree present, but complexity and derived absent
    assert!(parsed["tree"].is_string(), "minimal should include tree");
    assert!(
        parsed["complexity"].is_null(),
        "minimal should not include complexity"
    );
    assert!(
        parsed["derived"].is_null(),
        "minimal should not include derived"
    );
}

// ---------------------------------------------------------------------------
// Preset selection: --preset deep
// ---------------------------------------------------------------------------

#[test]
fn handoff_preset_deep_includes_rich_intelligence() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_preset_deep");

    tokmd_cmd()
        .arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--preset")
        .arg("deep")
        .arg("--no-git")
        .assert()
        .success();

    let intel = fs::read_to_string(out_dir.join("intelligence.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&intel).unwrap();

    assert!(parsed["tree"].is_string(), "deep should include tree");
    assert!(
        parsed["complexity"].is_object(),
        "deep should include complexity"
    );
    assert!(parsed["derived"].is_object(), "deep should include derived");
}

// ---------------------------------------------------------------------------
// Map JSONL: every line is valid JSON
// ---------------------------------------------------------------------------

#[test]
fn handoff_map_jsonl_lines_are_valid_json() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_map_jsonl");

    tokmd_cmd()
        .arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--no-git")
        .assert()
        .success();

    let map_content = fs::read_to_string(out_dir.join("map.jsonl")).unwrap();
    let mut line_count = 0;
    for line in map_content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let _: serde_json::Value = serde_json::from_str(line).unwrap_or_else(|e| {
            panic!("invalid JSON on line {}: {}\n  {}", line_count + 1, e, line)
        });
        line_count += 1;
    }
    assert!(line_count > 0, "map.jsonl should have at least one line");
}

// ---------------------------------------------------------------------------
// Budget tokens: used <= budget
// ---------------------------------------------------------------------------

#[test]
fn handoff_respects_token_budget() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_budget");

    tokmd_cmd()
        .arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--budget")
        .arg("1k")
        .arg("--no-git")
        .assert()
        .success();

    let content = fs::read_to_string(out_dir.join("manifest.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

    let budget = parsed["budget_tokens"].as_u64().unwrap();
    let used = parsed["used_tokens"].as_u64().unwrap();
    assert!(
        used <= budget,
        "used_tokens ({}) should not exceed budget_tokens ({})",
        used,
        budget
    );
}

// ---------------------------------------------------------------------------
// Error handling: nonexistent path
// ---------------------------------------------------------------------------

#[test]
fn handoff_nonexistent_path_fails() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_noexist");

    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .arg("handoff")
        .arg("/this/path/definitely/does/not/exist")
        .arg("--out-dir")
        .arg(&out_dir)
        .assert()
        .failure();
}

// ---------------------------------------------------------------------------
// Error handling: invalid preset value
// ---------------------------------------------------------------------------

#[test]
fn handoff_invalid_preset_fails() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_bad_preset");

    tokmd_cmd()
        .arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--preset")
        .arg("nonexistent_preset")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "invalid value 'nonexistent_preset'",
        ));
}

// ---------------------------------------------------------------------------
// Error handling: output dir already exists without --force
// ---------------------------------------------------------------------------

#[test]
fn handoff_existing_dir_without_force_fails() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_exists");

    // First run succeeds
    tokmd_cmd()
        .arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--no-git")
        .assert()
        .success();

    // Second run without --force fails
    tokmd_cmd()
        .arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--no-git")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not empty").or(predicate::str::contains("--force")));
}

#[test]
fn handoff_existing_dir_with_force_succeeds() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_force");

    // First run
    tokmd_cmd()
        .arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--no-git")
        .assert()
        .success();

    // Second run with --force
    tokmd_cmd()
        .arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--force")
        .arg("--no-git")
        .assert()
        .success();

    // Artifacts still present after overwrite
    assert!(out_dir.join("manifest.json").exists());
}
