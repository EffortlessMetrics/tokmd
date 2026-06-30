#![cfg(feature = "ast")]

use assert_cmd::Command;
use serde_json::Value;
use tempfile::tempdir;

fn tokmd_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
}

fn syntax_fixture() -> &'static str {
    include_str!("../../../fixtures/syntax/typescript/native_boundary.ts")
}

#[test]
fn syntax_command_emits_scoped_review_signals() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("src").join("runtime");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("native_boundary.ts"), syntax_fixture()).unwrap();

    let output = tokmd_cmd()
        .current_dir(dir.path())
        .args(["syntax", "src/runtime"])
        .output()
        .expect("tokmd syntax should run");

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["schema"], "tokmd.syntax_receipts.v1");
    assert_eq!(json["status"], "complete");
    assert_eq!(json["paths"][0], "src/runtime");
    assert_eq!(
        json["receipts"][0]["path"],
        "src/runtime/native_boundary.ts"
    );
    assert_eq!(json["receipts"][0]["schema"], "tokmd.syntax_receipt.v1");
    assert!(
        json["receipts"][0]["review_signals"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| { entry["category"] == "native_boundary" && entry["severity"] == "high" })
    );
}

#[test]
fn syntax_command_honors_global_exclude_pattern() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("keep.rs"), "pub fn keep() {}\n").unwrap();
    std::fs::write(src.join("skip.rs"), "pub fn skip() {}\n").unwrap();

    let output = tokmd_cmd()
        .current_dir(dir.path())
        .args(["syntax", "--exclude", "**/skip.rs", "src"])
        .output()
        .expect("tokmd syntax should run");

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let paths: Vec<&str> = json["receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|receipt| receipt["path"].as_str())
        .collect();
    assert!(
        paths.contains(&"src/keep.rs"),
        "kept file should be present: {paths:?}"
    );
    assert!(
        !paths.iter().any(|path| path.ends_with("skip.rs")),
        "excluded file must not appear in receipts: {paths:?}"
    );
}

#[test]
fn syntax_command_records_advisory_receipts_as_partial() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("lib.rs"), "pub fn large_enough() {}\n").unwrap();
    std::fs::write(dir.path().join("README.md"), "# docs\n").unwrap();

    let output = tokmd_cmd()
        .current_dir(dir.path())
        .args(["syntax", "--max-bytes", "4", "."])
        .output()
        .expect("tokmd syntax should run");

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "partial");
    let warnings = json["warnings"].as_array().unwrap();
    assert!(
        warnings
            .iter()
            .any(|entry| entry.as_str().unwrap().contains("unsupported_language"))
    );
    assert!(
        warnings
            .iter()
            .any(|entry| entry.as_str().unwrap().contains("skipped_too_large"))
    );
}

#[test]
fn syntax_command_fails_after_printing_packet_for_missing_input() {
    let dir = tempdir().unwrap();

    let output = tokmd_cmd()
        .current_dir(dir.path())
        .args(["syntax", "missing.rs"])
        .output()
        .expect("tokmd syntax should run");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("syntax receipts failed"),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "failed");
    assert!(json["errors"][0].as_str().unwrap().contains("missing.rs"));
}
