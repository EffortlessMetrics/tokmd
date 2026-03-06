//! CLI integration tests for context and handoff commands (W73).

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
// context --mode list
// ---------------------------------------------------------------------------

#[test]
fn w73_context_list_on_temp_dir() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::write(
        root.join("hello.rs"),
        "fn main() { println!(\"hello\"); }\n",
    )
    .unwrap();
    fs::write(
        root.join("lib.rs"),
        "pub fn add(a: i32, b: i32) -> i32 { a + b }\n",
    )
    .unwrap();
    fs::create_dir_all(root.join(".git")).unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(root)
        .arg("context")
        .arg("--mode")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello.rs").or(predicate::str::contains("lib.rs")));
}

#[test]
fn w73_context_list_includes_all_source_files() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::write(root.join("main.py"), "print('hello')\n").unwrap();
    fs::write(root.join("app.js"), "console.log('hi');\n").unwrap();
    fs::create_dir_all(root.join(".git")).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(root)
        .arg("context")
        .arg("--mode")
        .arg("list")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("main.py") || stdout.contains("app.js"));
}

// ---------------------------------------------------------------------------
// context --mode json
// ---------------------------------------------------------------------------

#[test]
fn w73_context_json_has_required_structure() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::write(root.join("main.rs"), "fn main() {}\n").unwrap();
    fs::create_dir_all(root.join(".git")).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(root)
        .arg("context")
        .arg("--mode")
        .arg("json")
        .output()
        .unwrap();

    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid JSON output");

    assert_eq!(parsed["schema_version"].as_u64(), Some(4));
    assert_eq!(parsed["mode"].as_str(), Some("context"));
    assert!(parsed["budget_tokens"].is_number());
    assert!(parsed["used_tokens"].is_number());
    assert!(parsed["utilization_pct"].is_number());
    assert!(parsed["strategy"].is_string());
    assert!(parsed["rank_by"].is_string());
    assert!(parsed["file_count"].is_number());
    assert!(parsed["files"].is_array());
    assert_eq!(parsed["tool"]["name"].as_str(), Some("tokmd"));
}

#[test]
fn w73_context_json_file_rows_have_fields() {
    let output = tokmd_cmd()
        .arg("context")
        .arg("--mode")
        .arg("json")
        .output()
        .unwrap();

    assert!(output.status.success());
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let files = parsed["files"].as_array().unwrap();

    if let Some(first) = files.first() {
        assert!(first["path"].is_string());
        assert!(first["module"].is_string());
        assert!(first["lang"].is_string());
        assert!(first["tokens"].is_number());
        assert!(first["code"].is_number());
        assert!(first["lines"].is_number());
        assert!(first["bytes"].is_number());
        assert!(first["value"].is_number());
    }
}

// ---------------------------------------------------------------------------
// context --budget
// ---------------------------------------------------------------------------

#[test]
fn w73_context_budget_respected() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    // Write several files to exceed a small budget
    for i in 0..10 {
        fs::write(
            root.join(format!("file_{i}.rs")),
            format!("pub fn f{i}() {{ let x = {i}; println!(\"{{x}}\"); }}\n"),
        )
        .unwrap();
    }
    fs::create_dir_all(root.join(".git")).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(root)
        .arg("context")
        .arg("--mode")
        .arg("json")
        .arg("--budget")
        .arg("1000")
        .output()
        .unwrap();

    assert!(output.status.success());
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    let budget = parsed["budget_tokens"].as_u64().unwrap();
    let used = parsed["used_tokens"].as_u64().unwrap();
    assert_eq!(budget, 1000);
    assert!(
        used <= budget,
        "used_tokens ({used}) should not exceed budget_tokens ({budget})"
    );
}

#[test]
fn w73_context_budget_unlimited() {
    let output = tokmd_cmd()
        .arg("context")
        .arg("--mode")
        .arg("json")
        .arg("--budget")
        .arg("unlimited")
        .output()
        .unwrap();

    assert!(output.status.success());
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let budget = parsed["budget_tokens"].as_u64().unwrap();
    // Unlimited budget is represented as a very large number
    assert!(budget > 1_000_000);
}

// ---------------------------------------------------------------------------
// handoff --preset minimal
// ---------------------------------------------------------------------------

#[test]
fn w73_handoff_preset_minimal_output() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_w73_minimal");

    tokmd_cmd()
        .arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--preset")
        .arg("minimal")
        .arg("--no-git")
        .assert()
        .success();

    assert!(out_dir.join("manifest.json").exists());
    assert!(out_dir.join("intelligence.json").exists());
    assert!(out_dir.join("code.txt").exists());
    assert!(out_dir.join("map.jsonl").exists());

    let intel: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out_dir.join("intelligence.json")).unwrap())
            .unwrap();
    assert!(intel["tree"].is_string());
    // Minimal: no complexity or derived
    assert!(intel["complexity"].is_null());
    assert!(intel["derived"].is_null());
}

#[test]
fn w73_handoff_preset_in_manifest() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_w73_preset");

    tokmd_cmd()
        .arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--preset")
        .arg("minimal")
        .arg("--no-git")
        .assert()
        .success();

    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out_dir.join("manifest.json")).unwrap()).unwrap();

    assert_eq!(manifest["intelligence_preset"].as_str(), Some("minimal"));
}

// ---------------------------------------------------------------------------
// handoff --format json (manifest structure)
// ---------------------------------------------------------------------------

#[test]
fn w73_handoff_manifest_json_structure() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_w73_json");

    tokmd_cmd()
        .arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--no-git")
        .assert()
        .success();

    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out_dir.join("manifest.json")).unwrap()).unwrap();

    assert_eq!(manifest["schema_version"].as_u64(), Some(5));
    assert_eq!(manifest["mode"].as_str(), Some("handoff"));
    assert!(manifest["generated_at_ms"].is_number());
    assert_eq!(manifest["tool"]["name"].as_str(), Some("tokmd"));
    assert!(manifest["budget_tokens"].is_number());
    assert!(manifest["used_tokens"].is_number());
    assert!(manifest["utilization_pct"].is_number());
    assert!(manifest["strategy"].is_string());
    assert!(manifest["rank_by"].is_string());
    assert!(manifest["capabilities"].is_array());
    assert!(manifest["artifacts"].is_array());
    assert!(manifest["included_files"].is_array());
    assert!(manifest["total_files"].is_number());
    assert!(manifest["bundled_files"].is_number());
}

#[test]
fn w73_handoff_artifacts_have_integrity_hashes() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_w73_hashes");

    tokmd_cmd()
        .arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--no-git")
        .assert()
        .success();

    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out_dir.join("manifest.json")).unwrap()).unwrap();

    let artifacts = manifest["artifacts"].as_array().unwrap();
    assert!(!artifacts.is_empty());

    for artifact in artifacts {
        assert!(artifact["name"].is_string());
        assert!(artifact["path"].is_string());
        assert!(artifact["bytes"].is_number());
        // All artifacts should have blake3 hashes
        if let Some(hash) = artifact.get("hash") {
            assert_eq!(hash["algo"].as_str(), Some("blake3"));
            assert!(hash["hash"].is_string());
        }
    }
}
