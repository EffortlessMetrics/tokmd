//! BDD-style scenario tests for the `export` command.
//!
//! Each test follows the Given/When/Then pattern to verify key user-facing
//! workflows of the file-level export command.

mod common;

use assert_cmd::Command;
use serde_json::Value;
use tempfile::tempdir;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

// ---------------------------------------------------------------------------
// Scenario 1: JSONL export produces valid JSON on each line
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_export_jsonl_then_each_line_is_valid_json() {
    // Given: a project with source files
    // When: I export as JSONL
    let output = tokmd_cmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("failed to execute tokmd export --format jsonl");

    // Then: each line is valid JSON
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(
        lines.len() >= 2,
        "should have meta line + at least one data row"
    );

    for (i, line) in lines.iter().enumerate() {
        let _: Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("line {} is not valid JSON: {}", i + 1, e));
    }
}

// ---------------------------------------------------------------------------
// Scenario 2: JSONL meta line has schema_version
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_export_jsonl_then_meta_has_schema_version() {
    // Given: a project with source files
    // When: I export as JSONL
    let output = tokmd_cmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("failed to execute tokmd export --format jsonl");

    // Then: the first line (meta) has schema_version
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let first_line = stdout
        .lines()
        .next()
        .expect("should have at least one line");
    let meta: Value = serde_json::from_str(first_line).expect("meta line should be valid JSON");
    assert!(
        meta["schema_version"].is_number(),
        "meta line should have schema_version"
    );
}

// ---------------------------------------------------------------------------
// Scenario 3: CSV export has header with expected columns
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_export_csv_then_header_matches_expected_columns() {
    // Given: a project with source files
    // When: I export as CSV
    let output = tokmd_cmd()
        .args(["export", "--format", "csv"])
        .output()
        .expect("failed to execute tokmd export --format csv");

    // Then: header row has expected columns
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let header = stdout.lines().next().expect("should have header line");

    assert!(
        header.contains("path") || header.contains("file"),
        "CSV header should contain path/file column"
    );
    assert!(
        header.contains("language") || header.contains("lang"),
        "CSV header should contain language column"
    );
    assert!(
        header.contains("code"),
        "CSV header should contain code column"
    );
}

// ---------------------------------------------------------------------------
// Scenario 4: CSV rows have consistent column count
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_export_csv_then_rows_have_consistent_columns() {
    // Given: a project with source files
    // When: I export as CSV
    let output = tokmd_cmd()
        .args(["export", "--format", "csv"])
        .output()
        .expect("failed to execute tokmd export --format csv");

    // Then: all data rows have the same column count as the header
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() >= 2, "need header + at least one data row");

    let header_cols = lines[0].split(',').count();
    for (i, line) in lines[1..].iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let cols = line.split(',').count();
        assert_eq!(
            cols,
            header_cols,
            "row {} has {} cols, header has {}",
            i + 1,
            cols,
            header_cols
        );
    }
}

// ---------------------------------------------------------------------------
// Scenario 5: --redact paths hides real file paths
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_export_redact_paths_then_no_real_paths() {
    // Given: a project with source files (e.g., main.rs, script.js)
    // When: I export as JSON with --redact paths
    let output = tokmd_cmd()
        .args(["export", "--format", "json", "--redact", "paths"])
        .output()
        .expect("failed to execute tokmd export --redact paths");

    // Then: no real file paths appear in output
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows should be array");
    assert!(!rows.is_empty(), "should have rows");

    let known_filenames = ["main.rs", "script.js", "large.rs", "README.md", "mixed.md"];
    for row in rows {
        let path = row["path"].as_str().unwrap_or_default();
        for name in &known_filenames {
            assert!(
                !path.contains(name),
                "redacted output should not contain '{name}', found in path: {path}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 6: JSON export has mode and schema_version
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_export_json_then_has_mode_and_schema() {
    // Given: a project with source files
    // When: I export as JSON
    let output = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("failed to execute tokmd export --format json");

    // Then: output has mode="export" and schema_version
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["mode"], "export", "mode should be 'export'");
    assert!(
        json["schema_version"].is_number(),
        "should have schema_version"
    );
}

// ---------------------------------------------------------------------------
// Scenario 7: Export rows have path and language fields
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_export_json_then_rows_have_path_and_language() {
    // Given: a project with source files
    // When: I export as JSON
    let output = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("failed to execute tokmd export --format json");

    // Then: each row has path and language fields
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows should be array");
    assert!(!rows.is_empty());

    for row in rows {
        assert!(row["path"].is_string(), "each row should have path");
        assert!(row["lang"].is_string(), "each row should have lang");
        assert!(row["code"].is_number(), "each row should have code count");
    }
}

// ---------------------------------------------------------------------------
// Scenario 8: Empty directory export produces empty rows
// ---------------------------------------------------------------------------

#[test]
fn given_empty_dir_when_export_json_then_empty_rows() {
    // Given: an empty directory
    let dir = tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).expect("create .git marker");

    // When: I export as JSON
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd
        .current_dir(dir.path())
        .args(["export", "--format", "json"])
        .output()
        .expect("failed to execute tokmd export on empty dir");

    // Then: rows array is empty
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows should be array");
    assert!(rows.is_empty(), "empty dir should produce no export rows");
}
