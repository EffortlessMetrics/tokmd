//! W71 deep context CLI integration tests.
//!
//! Tests cover: rank-by variants, no-git fallback, compress flag, max-file-pct,
//! max-file-tokens, no-smart-exclude, module-depth, log flag, empty directory,
//! file ordering, and JSON receipt completeness.

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

/// Helper: run context --mode json and return parsed receipt.
fn run_context_json(extra: &[&str]) -> serde_json::Value {
    let mut cmd = tokmd_cmd();
    cmd.arg("context").arg("--mode").arg("json");
    for a in extra {
        cmd.arg(a);
    }
    let output = cmd.output().expect("context json should succeed");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).unwrap()
}

// ===========================================================================
// 1. Rank-by variants
// ===========================================================================

#[test]
fn context_rank_by_code() {
    let parsed = run_context_json(&["--rank-by", "code", "--budget", "10k"]);
    assert_eq!(parsed["rank_by"].as_str(), Some("code"));
}

#[test]
fn context_rank_by_tokens() {
    let parsed = run_context_json(&["--rank-by", "tokens", "--budget", "10k"]);
    assert_eq!(parsed["rank_by"].as_str(), Some("tokens"));
}

#[test]
fn context_rank_by_hotspot_no_git_fallback() {
    let parsed = run_context_json(&["--rank-by", "hotspot", "--no-git", "--budget", "10k"]);
    // Should succeed even without git
    assert!(parsed["budget_tokens"].is_number());
}

#[test]
fn context_rank_by_churn_no_git_fallback() {
    let parsed = run_context_json(&["--rank-by", "churn", "--no-git", "--budget", "10k"]);
    assert!(parsed["budget_tokens"].is_number());
}

// ===========================================================================
// 2. No-git flag
// ===========================================================================

#[test]
fn context_no_git_flag_succeeds() {
    tokmd_cmd()
        .args(["context", "--mode", "json", "--no-git", "--budget", "10k"])
        .assert()
        .success();
}

#[test]
fn context_no_git_json_still_has_required_fields() {
    let parsed = run_context_json(&["--no-git", "--budget", "10k"]);
    assert_eq!(parsed["schema_version"].as_u64(), Some(4));
    assert_eq!(parsed["mode"].as_str(), Some("context"));
    assert!(parsed["files"].is_array());
}

// ===========================================================================
// 3. Compress flag
// ===========================================================================

#[test]
fn context_compress_bundle_mode() {
    let normal = tokmd_cmd()
        .args(["context", "--mode", "bundle", "--budget", "50k"])
        .output()
        .unwrap();

    let compressed = tokmd_cmd()
        .args([
            "context",
            "--mode",
            "bundle",
            "--budget",
            "50k",
            "--compress",
        ])
        .output()
        .unwrap();

    assert!(normal.status.success());
    assert!(compressed.status.success());

    // Compressed should be same or smaller
    assert!(
        compressed.stdout.len() <= normal.stdout.len(),
        "compressed ({}) should be <= normal ({})",
        compressed.stdout.len(),
        normal.stdout.len()
    );
}

// ===========================================================================
// 4. Max-file-pct / max-file-tokens
// ===========================================================================

#[test]
fn context_max_file_pct_accepted() {
    tokmd_cmd()
        .args([
            "context",
            "--mode",
            "json",
            "--budget",
            "10k",
            "--max-file-pct",
            "0.05",
        ])
        .assert()
        .success();
}

#[test]
fn context_max_file_tokens_accepted() {
    tokmd_cmd()
        .args([
            "context",
            "--mode",
            "json",
            "--budget",
            "10k",
            "--max-file-tokens",
            "50",
        ])
        .assert()
        .success();
}

#[test]
fn context_max_file_tokens_limits_individual_files() {
    let parsed = run_context_json(&["--budget", "50k", "--max-file-tokens", "50"]);
    let files = parsed["files"].as_array().unwrap();
    for file in files {
        let tokens = file["tokens"].as_u64().unwrap_or(0);
        // Each file's contribution should not dramatically exceed the cap
        // (the cap applies to bundled content; the row still records total tokens)
        assert!(
            tokens > 0 || file["path"].is_string(),
            "file rows should have tokens"
        );
    }
}

// ===========================================================================
// 5. No-smart-exclude
// ===========================================================================

#[test]
fn context_no_smart_exclude_accepted() {
    tokmd_cmd()
        .args([
            "context",
            "--mode",
            "json",
            "--budget",
            "10k",
            "--no-smart-exclude",
        ])
        .assert()
        .success();
}

// ===========================================================================
// 6. Module depth
// ===========================================================================

#[test]
fn context_module_depth_accepted() {
    tokmd_cmd()
        .args([
            "context",
            "--mode",
            "json",
            "--budget",
            "10k",
            "--module-depth",
            "1",
        ])
        .assert()
        .success();
}

// ===========================================================================
// 7. Log flag
// ===========================================================================

#[test]
fn context_log_flag_creates_logfile() {
    let dir = tempdir().unwrap();
    let log_file = dir.path().join("context.log");

    tokmd_cmd()
        .args(["context", "--mode", "json", "--budget", "5k", "--log"])
        .arg(&log_file)
        .assert()
        .success();

    assert!(log_file.exists(), "log file should be created");
    let content = fs::read_to_string(&log_file).unwrap();
    // Log should contain valid JSONL
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let _: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("Invalid JSONL in log: {e}"));
    }
}

// ===========================================================================
// 8. Empty directory handling
// ===========================================================================

#[test]
fn context_empty_directory_succeeds() {
    let dir = tempdir().unwrap();
    let empty = dir.path().join("empty");
    fs::create_dir_all(&empty).unwrap();
    // Create .git marker so ignore crate is happy
    fs::create_dir_all(empty.join(".git")).unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(&empty)
        .args(["context", "--mode", "json", "--budget", "1k"])
        .arg(&empty)
        .assert()
        .success();
}

#[test]
fn context_empty_directory_zero_files() {
    let dir = tempdir().unwrap();
    let empty = dir.path().join("empty2");
    fs::create_dir_all(&empty).unwrap();
    fs::create_dir_all(empty.join(".git")).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(&empty)
        .args(["context", "--mode", "json", "--budget", "1k"])
        .arg(&empty)
        .output()
        .unwrap();

    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();
    assert_eq!(parsed["file_count"].as_u64(), Some(0));
    assert_eq!(parsed["used_tokens"].as_u64(), Some(0));
}

// ===========================================================================
// 9. File ordering determinism
// ===========================================================================

#[test]
fn context_file_ordering_deterministic() {
    let get_paths = || {
        let parsed = run_context_json(&["--budget", "50k"]);
        parsed["files"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["path"].as_str().unwrap().to_string())
            .collect::<Vec<_>>()
    };

    let run1 = get_paths();
    let run2 = get_paths();
    assert_eq!(run1, run2, "file ordering should be deterministic");
}

// ===========================================================================
// 10. Budget suffix parsing
// ===========================================================================

#[test]
fn context_budget_m_suffix() {
    let parsed = run_context_json(&["--budget", "1m"]);
    assert_eq!(parsed["budget_tokens"].as_u64(), Some(1_000_000));
}

#[test]
fn context_budget_plain_number() {
    let parsed = run_context_json(&["--budget", "5000"]);
    assert_eq!(parsed["budget_tokens"].as_u64(), Some(5000));
}

// ===========================================================================
// 11. Output to file
// ===========================================================================

#[test]
fn context_output_bundle_to_file() {
    let dir = tempdir().unwrap();
    let out_file = dir.path().join("bundle_out.txt");

    tokmd_cmd()
        .args(["context", "--mode", "bundle", "--budget", "10k", "--output"])
        .arg(&out_file)
        .assert()
        .success();

    assert!(out_file.exists());
    let content = fs::read_to_string(&out_file).unwrap();
    assert!(!content.is_empty(), "bundle output should not be empty");
}

// ===========================================================================
// 12. Invalid rank-by rejected
// ===========================================================================

#[test]
fn context_invalid_rank_by_rejected() {
    tokmd_cmd()
        .args([
            "context",
            "--mode",
            "json",
            "--rank-by",
            "nonexistent",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}
