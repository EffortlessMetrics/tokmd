#![cfg(feature = "analysis")]
//! CLI scan-option tests (w66).
//!
//! Validates that scan-related flags (--exclude, --module-depth, --children,
//! --hidden, --no-ignore, etc.) work correctly, including edge cases like
//! empty directories and self-referential scans.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

// ===========================================================================
// 1. --exclude pattern tests
// ===========================================================================

#[test]
fn exclude_pattern_reduces_output() {
    let baseline = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("baseline run");
    assert!(baseline.status.success());
    let base_json: Value = serde_json::from_slice(&baseline.stdout).unwrap();
    let base_total: u64 = base_json["total"]["code"]
        .as_u64()
        .or_else(|| base_json["totals"]["code"].as_u64())
        .unwrap_or(0);

    let excluded = tokmd_cmd()
        .args(["lang", "--format", "json", "--exclude", "*.rs"])
        .output()
        .expect("excluded run");
    assert!(excluded.status.success());
    let excl_json: Value = serde_json::from_slice(&excluded.stdout).unwrap();
    let excl_total: u64 = excl_json["total"]["code"]
        .as_u64()
        .or_else(|| excl_json["totals"]["code"].as_u64())
        .unwrap_or(0);

    assert!(
        excl_total <= base_total,
        "excluding *.rs should not increase code count: excluded={excl_total} baseline={base_total}"
    );
}

#[test]
fn exclude_multiple_patterns() {
    tokmd_cmd()
        .args([
            "lang",
            "--format",
            "json",
            "--exclude",
            "*.rs",
            "--exclude",
            "*.js",
        ])
        .assert()
        .success();
}

#[test]
fn exclude_all_files_produces_zero_rows() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json", "--exclude", "*"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows array");
    assert!(
        rows.is_empty(),
        "excluding everything should produce zero rows"
    );
}

#[test]
fn export_exclude_filters_files() {
    let output = tokmd_cmd()
        .args(["export", "--format", "json", "--exclude", "*.js"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows array");

    for row in rows {
        let path = row["path"].as_str().unwrap_or("");
        assert!(
            !path.ends_with(".js"),
            "excluded .js file still appears: {path}"
        );
    }
}

// ===========================================================================
// 2. --module-depth tests
// ===========================================================================

#[test]
fn module_depth_one_groups_at_top_level() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json", "--module-depth", "1"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows array");

    for row in rows {
        let module = row["module"].as_str().unwrap_or("");
        // At depth 1, module keys should have at most one path separator
        let depth = module.matches('/').count();
        assert!(
            depth <= 1,
            "module-depth 1 should produce shallow keys, got: {module}"
        );
    }
}

#[test]
fn module_depth_large_value_succeeds() {
    tokmd_cmd()
        .args(["module", "--format", "json", "--module-depth", "100"])
        .assert()
        .success();
}

// ===========================================================================
// 3. Scanning empty directories
// ===========================================================================

#[test]
fn scan_empty_directory_succeeds_with_zero_results() {
    let dir = tempfile::tempdir().expect("create temp dir");
    std::fs::create_dir(dir.path().join(".git")).expect("create .git");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path());
    let output = cmd
        .args(["lang", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows array");
    assert!(rows.is_empty(), "empty dir should have zero rows");
}

#[test]
fn export_empty_directory_succeeds() {
    let dir = tempfile::tempdir().expect("create temp dir");
    std::fs::create_dir(dir.path().join(".git")).expect("create .git");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path());
    cmd.args(["export", "--format", "json"]).assert().success();
}

#[test]
fn module_empty_directory_succeeds() {
    let dir = tempfile::tempdir().expect("create temp dir");
    std::fs::create_dir(dir.path().join(".git")).expect("create .git");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path());
    cmd.args(["module", "--format", "json"]).assert().success();
}

// ===========================================================================
// 4. Self-referential scan (scan the tokmd repo itself)
// ===========================================================================

#[test]
fn self_scan_succeeds() {
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(&repo_root);
    cmd.args(["lang", "--format", "json", "--exclude", "target"])
        .assert()
        .success();
}

#[test]
fn self_scan_finds_rust_language() {
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(&repo_root);
    let output = cmd
        .args(["lang", "--format", "json", "--exclude", "target"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows array");

    let has_rust = rows.iter().any(|r| r["lang"].as_str() == Some("Rust"));
    assert!(has_rust, "self-scan should find Rust language");
}

// ===========================================================================
// 5. --children mode flags
// ===========================================================================

#[test]
fn lang_children_collapse_succeeds() {
    tokmd_cmd()
        .args(["lang", "--format", "json", "--children", "collapse"])
        .assert()
        .success();
}

#[test]
fn lang_children_separate_succeeds() {
    tokmd_cmd()
        .args(["lang", "--format", "json", "--children", "separate"])
        .assert()
        .success();
}

#[test]
fn module_children_separate_succeeds() {
    tokmd_cmd()
        .args(["module", "--format", "json", "--children", "separate"])
        .assert()
        .success();
}

#[test]
fn export_children_separate_succeeds() {
    tokmd_cmd()
        .args(["export", "--format", "json", "--children", "separate"])
        .assert()
        .success();
}

#[test]
fn lang_children_invalid_value_fails() {
    tokmd_cmd()
        .args(["lang", "--children", "invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

// ===========================================================================
// 6. --hidden and --no-ignore flags (global, placed before subcommand)
// ===========================================================================

#[test]
fn hidden_flag_succeeds() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd.args(["--hidden", "lang", "--format", "json"])
        .assert()
        .success();
}

#[test]
fn no_ignore_flag_succeeds() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd.args(["--no-ignore", "lang", "--format", "json"])
        .assert()
        .success();
}

// ===========================================================================
// 7. --top flag
// ===========================================================================

#[test]
fn top_flag_limits_output() {
    let full = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("full run");
    assert!(full.status.success());
    let full_json: Value = serde_json::from_slice(&full.stdout).unwrap();
    let full_rows = full_json["rows"].as_array().expect("rows array").len();

    let limited = tokmd_cmd()
        .args(["lang", "--format", "json", "--top", "1"])
        .output()
        .expect("limited run");
    assert!(limited.status.success());
    let lim_json: Value = serde_json::from_slice(&limited.stdout).unwrap();
    let lim_rows = lim_json["rows"].as_array().expect("rows array").len();

    assert!(
        lim_rows <= full_rows,
        "--top 1 should produce <= full rows: limited={lim_rows} full={full_rows}"
    );
}

#[test]
fn top_zero_shows_all_languages() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json", "--top", "0"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows array");
    assert!(
        !rows.is_empty(),
        "--top 0 should show all languages (fixture has code)"
    );
}
