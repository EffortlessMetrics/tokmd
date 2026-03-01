//! E2E tests for the `tokmd diff` CLI command.
//!
//! Tests exercise diff with lang.json receipt files and verify
//! both markdown and JSON output formats.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

fn tokmd_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
}

/// Generate a lang.json receipt by scanning the given path.
fn generate_lang_receipt(scan_path: &std::path::Path, out_file: &std::path::Path) {
    let output = tokmd_cmd()
        .current_dir(scan_path)
        .arg("lang")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to execute tokmd lang");
    assert!(
        output.status.success(),
        "tokmd lang failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    fs::write(out_file, &output.stdout).expect("failed to write receipt");
}

// ---------------------------------------------------------------------------
// diff with --from / --to (lang.json files)
// ---------------------------------------------------------------------------

#[test]
fn diff_identical_receipts_produces_valid_json() {
    let dir = tempdir().unwrap();
    let receipt = dir.path().join("lang.json");

    generate_lang_receipt(common::fixture_root(), &receipt);

    let output = tokmd_cmd()
        .arg("diff")
        .arg("--format")
        .arg("json")
        .arg("--from")
        .arg(&receipt)
        .arg("--to")
        .arg(&receipt)
        .output()
        .expect("failed to execute tokmd diff");

    assert!(output.status.success(), "tokmd diff failed");

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("diff output is not valid JSON");

    assert_eq!(json["mode"], "diff");
    assert!(json["schema_version"].is_number());
    assert!(json["generated_at_ms"].is_number());
    assert!(json["tool"].is_object());
    assert!(json["diff_rows"].is_array());
    assert!(json["totals"].is_object());
}

#[test]
fn diff_identical_receipts_has_zero_deltas() {
    let dir = tempdir().unwrap();
    let receipt = dir.path().join("lang.json");

    generate_lang_receipt(common::fixture_root(), &receipt);

    let output = tokmd_cmd()
        .arg("diff")
        .arg("--format")
        .arg("json")
        .arg("--from")
        .arg(&receipt)
        .arg("--to")
        .arg(&receipt)
        .output()
        .expect("failed to execute tokmd diff");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout).unwrap();

    let totals = &json["totals"];
    assert_eq!(
        totals["delta_code"], 0,
        "identical receipts should have zero delta_code"
    );
    assert_eq!(totals["delta_lines"], 0);
    assert_eq!(totals["delta_files"], 0);

    let rows = json["diff_rows"].as_array().unwrap();
    for row in rows {
        assert_eq!(row["delta_code"], 0, "each row should have zero delta_code");
        assert_eq!(row["delta_lines"], 0);
    }
}

#[test]
fn diff_different_receipts_detects_changes() {
    let dir = tempdir().unwrap();

    // Receipt 1: from fixture data
    let receipt1 = dir.path().join("old.json");
    generate_lang_receipt(common::fixture_root(), &receipt1);

    // Receipt 2: from a temp dir with different content
    let alt_dir = dir.path().join("alt_source");
    fs::create_dir_all(alt_dir.join(".git")).unwrap();
    fs::write(
        alt_dir.join("hello.py"),
        "#!/usr/bin/env python3\nprint('hello')\n",
    )
    .unwrap();
    let receipt2 = dir.path().join("new.json");
    generate_lang_receipt(&alt_dir, &receipt2);

    let output = tokmd_cmd()
        .arg("diff")
        .arg("--format")
        .arg("json")
        .arg("--from")
        .arg(&receipt1)
        .arg("--to")
        .arg(&receipt2)
        .output()
        .expect("failed to execute tokmd diff");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout).unwrap();

    let rows = json["diff_rows"].as_array().unwrap();
    assert!(!rows.is_empty(), "diff should have at least one row");

    // The diff should contain rows with language names
    let langs: Vec<&str> = rows.iter().filter_map(|r| r["lang"].as_str()).collect();
    assert!(!langs.is_empty(), "diff rows should have lang fields");
}

#[test]
fn diff_json_rows_have_expected_fields() {
    let dir = tempdir().unwrap();

    // Create two different receipts so diff_rows is non-empty
    let receipt1 = dir.path().join("old.json");
    generate_lang_receipt(common::fixture_root(), &receipt1);

    let alt_dir = dir.path().join("alt");
    fs::create_dir_all(alt_dir.join(".git")).unwrap();
    fs::write(
        alt_dir.join("app.py"),
        "#!/usr/bin/env python3\ndef main():\n    pass\n",
    )
    .unwrap();
    let receipt2 = dir.path().join("new.json");
    generate_lang_receipt(&alt_dir, &receipt2);

    let output = tokmd_cmd()
        .arg("diff")
        .arg("--format")
        .arg("json")
        .arg("--from")
        .arg(&receipt1)
        .arg("--to")
        .arg(&receipt2)
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout).unwrap();

    let rows = json["diff_rows"].as_array().unwrap();
    assert!(
        !rows.is_empty(),
        "different receipts should produce diff rows"
    );

    let first = &rows[0];
    assert!(first["lang"].is_string(), "row should have lang");
    assert!(first["old_code"].is_number(), "row should have old_code");
    assert!(first["new_code"].is_number(), "row should have new_code");
    assert!(
        first["delta_code"].is_number(),
        "row should have delta_code"
    );
    assert!(first["old_lines"].is_number(), "row should have old_lines");
    assert!(first["new_lines"].is_number(), "row should have new_lines");
    assert!(
        first["delta_lines"].is_number(),
        "row should have delta_lines"
    );
}

// ---------------------------------------------------------------------------
// diff with positional arguments
// ---------------------------------------------------------------------------

#[test]
fn diff_positional_args_work() {
    let dir = tempdir().unwrap();
    let receipt = dir.path().join("lang.json");
    generate_lang_receipt(common::fixture_root(), &receipt);

    tokmd_cmd()
        .arg("diff")
        .arg("--format")
        .arg("json")
        .arg(&receipt)
        .arg(&receipt)
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// diff markdown output
// ---------------------------------------------------------------------------

#[test]
fn diff_default_format_is_markdown() {
    let dir = tempdir().unwrap();
    let receipt = dir.path().join("lang.json");
    generate_lang_receipt(common::fixture_root(), &receipt);

    tokmd_cmd()
        .arg("diff")
        .arg("--from")
        .arg(&receipt)
        .arg("--to")
        .arg(&receipt)
        .assert()
        .success()
        .stdout(predicate::str::contains("## Diff:"));
}

#[test]
fn diff_compact_mode_omits_breakdown() {
    let dir = tempdir().unwrap();
    let receipt = dir.path().join("lang.json");
    generate_lang_receipt(common::fixture_root(), &receipt);

    tokmd_cmd()
        .arg("diff")
        .arg("--compact")
        .arg("--from")
        .arg(&receipt)
        .arg("--to")
        .arg(&receipt)
        .assert()
        .success()
        .stdout(predicate::str::contains("Languages changed"))
        .stdout(predicate::str::contains("Language Breakdown").not());
}

// ---------------------------------------------------------------------------
// diff sources are recorded
// ---------------------------------------------------------------------------

#[test]
fn diff_json_records_source_paths() {
    let dir = tempdir().unwrap();
    let old = dir.path().join("old_lang.json");
    let new = dir.path().join("new_lang.json");
    generate_lang_receipt(common::fixture_root(), &old);
    fs::copy(&old, &new).unwrap();

    let output = tokmd_cmd()
        .arg("diff")
        .arg("--format")
        .arg("json")
        .arg("--from")
        .arg(&old)
        .arg("--to")
        .arg(&new)
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout).unwrap();

    assert!(
        json["from_source"].is_string(),
        "from_source should be present"
    );
    assert!(json["to_source"].is_string(), "to_source should be present");
}

// ---------------------------------------------------------------------------
// error cases
// ---------------------------------------------------------------------------

#[test]
fn diff_missing_args_fails() {
    tokmd_cmd().arg("diff").assert().failure();
}

#[test]
fn diff_nonexistent_file_fails() {
    tokmd_cmd()
        .arg("diff")
        .arg("--from")
        .arg("nonexistent_old.json")
        .arg("--to")
        .arg("nonexistent_new.json")
        .assert()
        .failure();
}
