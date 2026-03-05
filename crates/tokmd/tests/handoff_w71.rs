//! W71 deep handoff CLI integration tests.
//!
//! Tests cover: rank-by variants, preset deep, max-file-pct/max-file-tokens,
//! no-smart-exclude, module-depth, manifest field completeness, and
//! intelligence preset recording.

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

/// Helper: run handoff and return parsed manifest JSON.
fn run_handoff(extra: &[&str]) -> serde_json::Value {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("ho");

    let mut cmd = tokmd_cmd();
    cmd.arg("handoff").arg("--out-dir").arg(&out_dir);
    for a in extra {
        cmd.arg(a);
    }
    cmd.assert().success();

    let manifest = fs::read_to_string(out_dir.join("manifest.json")).unwrap();
    serde_json::from_str(&manifest).unwrap()
}

// ===========================================================================
// 1. Rank-by variants
// ===========================================================================

#[test]
fn handoff_rank_by_code() {
    let parsed = run_handoff(&["--rank-by", "code", "--budget", "10k"]);
    assert_eq!(parsed["rank_by"].as_str(), Some("code"));
}

#[test]
fn handoff_rank_by_tokens() {
    let parsed = run_handoff(&["--rank-by", "tokens", "--budget", "10k"]);
    assert_eq!(parsed["rank_by"].as_str(), Some("tokens"));
}

#[test]
fn handoff_rank_by_hotspot_no_git_fallback() {
    // With --no-git, hotspot ranking should gracefully fallback
    let parsed = run_handoff(&["--rank-by", "hotspot", "--no-git", "--budget", "10k"]);
    // Should succeed even without git; rank_by or rank_by_effective recorded
    assert!(parsed["budget_tokens"].is_number());
}

#[test]
fn handoff_rank_by_churn_no_git_fallback() {
    let parsed = run_handoff(&["--rank-by", "churn", "--no-git", "--budget", "10k"]);
    assert!(parsed["budget_tokens"].is_number());
}

// ===========================================================================
// 2. Preset deep
// ===========================================================================

#[test]
fn handoff_preset_deep_succeeds() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("ho_deep");

    tokmd_cmd()
        .args(["handoff", "--preset", "deep", "--budget", "20k", "--out-dir"])
        .arg(&out_dir)
        .assert()
        .success();

    assert!(out_dir.join("manifest.json").exists());
    assert!(out_dir.join("intelligence.json").exists());
}

#[test]
fn handoff_preset_deep_intelligence_has_derived() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("ho_deep_i");

    tokmd_cmd()
        .args(["handoff", "--preset", "deep", "--budget", "20k", "--out-dir"])
        .arg(&out_dir)
        .assert()
        .success();

    let intel = fs::read_to_string(out_dir.join("intelligence.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&intel).unwrap();

    // Deep preset should include tree and derived metrics
    assert!(parsed["tree"].is_string());
    assert!(parsed["derived"].is_object());
}

#[test]
fn handoff_manifest_records_intelligence_preset() {
    let parsed = run_handoff(&["--preset", "minimal", "--budget", "5k"]);
    assert_eq!(
        parsed["intelligence_preset"].as_str(),
        Some("minimal"),
        "manifest should record the intelligence preset"
    );
}

#[test]
fn handoff_manifest_records_deep_preset() {
    let parsed = run_handoff(&["--preset", "deep", "--budget", "20k"]);
    assert_eq!(parsed["intelligence_preset"].as_str(), Some("deep"));
}

// ===========================================================================
// 3. Budget fields in manifest
// ===========================================================================

#[test]
fn handoff_manifest_has_total_and_bundled_files() {
    let parsed = run_handoff(&["--budget", "10k"]);

    assert!(
        parsed["total_files"].is_number(),
        "manifest should have total_files"
    );
    assert!(
        parsed["bundled_files"].is_number(),
        "manifest should have bundled_files"
    );

    let total = parsed["total_files"].as_u64().unwrap();
    let bundled = parsed["bundled_files"].as_u64().unwrap();
    assert!(
        bundled <= total,
        "bundled_files ({bundled}) should be <= total_files ({total})"
    );
}

#[test]
fn handoff_manifest_utilization_pct_consistent() {
    let parsed = run_handoff(&["--budget", "10k"]);

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
// 4. Max-file-pct / max-file-tokens
// ===========================================================================

#[test]
fn handoff_max_file_pct_accepted() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("ho_mfp");

    tokmd_cmd()
        .args([
            "handoff",
            "--budget",
            "10k",
            "--max-file-pct",
            "0.05",
            "--out-dir",
        ])
        .arg(&out_dir)
        .assert()
        .success();

    assert!(out_dir.join("manifest.json").exists());
}

#[test]
fn handoff_max_file_tokens_accepted() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("ho_mft");

    tokmd_cmd()
        .args([
            "handoff",
            "--budget",
            "10k",
            "--max-file-tokens",
            "100",
            "--out-dir",
        ])
        .arg(&out_dir)
        .assert()
        .success();

    assert!(out_dir.join("manifest.json").exists());
}

// ===========================================================================
// 5. No-smart-exclude
// ===========================================================================

#[test]
fn handoff_no_smart_exclude_accepted() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("ho_nse");

    tokmd_cmd()
        .args([
            "handoff",
            "--no-smart-exclude",
            "--budget",
            "10k",
            "--out-dir",
        ])
        .arg(&out_dir)
        .assert()
        .success();

    assert!(out_dir.join("manifest.json").exists());
}

#[test]
fn handoff_smart_excluded_files_field_present() {
    let parsed = run_handoff(&["--budget", "10k"]);
    // smart_excluded_files should be an array (possibly empty for fixture)
    assert!(
        parsed["smart_excluded_files"].is_array(),
        "manifest should have smart_excluded_files array"
    );
}

// ===========================================================================
// 6. Module depth
// ===========================================================================

#[test]
fn handoff_module_depth_accepted() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("ho_md");

    tokmd_cmd()
        .args([
            "handoff",
            "--module-depth",
            "1",
            "--budget",
            "10k",
            "--out-dir",
        ])
        .arg(&out_dir)
        .assert()
        .success();

    assert!(out_dir.join("manifest.json").exists());
}

// ===========================================================================
// 7. Included-files row completeness
// ===========================================================================

#[test]
fn handoff_included_files_rows_have_expected_fields() {
    let parsed = run_handoff(&["--budget", "50k"]);

    let included = parsed["included_files"].as_array().unwrap();
    if included.is_empty() {
        return; // Fixture too small for budget – skip
    }

    let first = &included[0];
    assert!(first["path"].is_string(), "should have path");
    assert!(first["tokens"].is_number(), "should have tokens");
}

// ===========================================================================
// 8. Determinism: rank-by does not break determinism
// ===========================================================================

#[test]
fn handoff_rank_by_code_deterministic() {
    let get_files = || {
        let parsed = run_handoff(&["--rank-by", "code", "--budget", "10k"]);
        parsed["included_files"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["path"].as_str().unwrap().to_string())
            .collect::<Vec<_>>()
    };

    let run1 = get_files();
    let run2 = get_files();
    assert_eq!(run1, run2, "rank-by code should be deterministic");
}

// ===========================================================================
// 9. Invalid preset rejected
// ===========================================================================

#[test]
fn handoff_invalid_preset_rejected() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("ho_bad");

    tokmd_cmd()
        .args([
            "handoff",
            "--preset",
            "nonexistent",
            "--out-dir",
        ])
        .arg(&out_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

// ===========================================================================
// 10. Compress with preset
// ===========================================================================

#[test]
fn handoff_compress_with_deep_preset() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("ho_c_d");

    tokmd_cmd()
        .args([
            "handoff",
            "--preset",
            "deep",
            "--compress",
            "--budget",
            "20k",
            "--out-dir",
        ])
        .arg(&out_dir)
        .assert()
        .success();

    let code = fs::read_to_string(out_dir.join("code.txt")).unwrap();
    // Compressed code should still have file markers
    assert!(code.contains("// ==="));
}

// ===========================================================================
// 11. Tool metadata in manifest
// ===========================================================================

#[test]
fn handoff_manifest_tool_has_version() {
    let parsed = run_handoff(&["--budget", "5k"]);
    assert_eq!(parsed["tool"]["name"].as_str(), Some("tokmd"));
    assert!(
        parsed["tool"]["version"].is_string(),
        "tool should have version"
    );
}
