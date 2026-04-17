//! BDD-style scenario tests for the `module` command.
//!
//! Each test follows the Given/When/Then pattern to verify key user-facing
//! workflows of the module breakdown command.

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
// Scenario 1: Module keys use forward slashes
// ---------------------------------------------------------------------------

#[test]
fn given_nested_dirs_when_module_json_then_keys_use_forward_slashes() {
    // Given: a project with nested directories (fixture has src/)
    // When: I run `tokmd module --format json`
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("failed to execute tokmd module --format json");

    // Then: module keys use forward slashes (path normalization)
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows should be array");
    assert!(!rows.is_empty(), "should have at least one module");

    for row in rows {
        let module = row["module"].as_str().expect("module should be string");
        assert!(
            !module.contains('\\'),
            "module key should not contain backslashes: {module}"
        );
    }
}

// ---------------------------------------------------------------------------
// Scenario 2: --module-depth 0 produces only top-level modules
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_depth_0_then_only_top_level_modules() {
    // Given: a project with nested directories
    // When: I run `tokmd module --format json --module-depth 0`
    let output = tokmd_cmd()
        .args(["module", "--format", "json", "--module-depth", "0"])
        .output()
        .expect("failed to execute tokmd module --module-depth 0");

    // Then: only top-level modules appear (no nested slashes)
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        json["module_depth"].as_u64().unwrap(),
        0,
        "module_depth should be 0"
    );

    let rows = json["rows"].as_array().expect("rows should be array");
    for row in rows {
        let module = row["module"].as_str().expect("module should be string");
        assert!(
            !module.contains('/'),
            "depth 0 should not produce nested modules, got: {module}"
        );
    }
}

// ---------------------------------------------------------------------------
// Scenario 3: Single-file project produces one module entry
// ---------------------------------------------------------------------------

#[test]
fn given_single_file_project_when_module_then_one_entry() {
    // Given: a directory with a single source file
    let dir = tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).expect("create .git marker");
    std::fs::write(dir.path().join("main.rs"), "fn main() {}\n").expect("write main.rs");

    // When: I run `tokmd module --format json`
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd
        .current_dir(dir.path())
        .args(["module", "--format", "json"])
        .output()
        .expect("failed to execute tokmd module");

    // Then: exactly one module entry exists
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows should be array");
    assert_eq!(rows.len(), 1, "single-file project should have one module");
}

// ---------------------------------------------------------------------------
// Scenario 4: Module JSON has mode and total fields
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_module_json_then_has_mode_and_total() {
    // Given: a project with source files
    // When: I run `tokmd module --format json`
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("failed to execute tokmd module --format json");

    // Then: JSON has mode="module" and total object
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["mode"], "module", "mode should be 'module'");
    assert!(json["total"].is_object(), "total should be present");
    assert!(
        json["total"]["code"].is_number(),
        "total should have code count"
    );
}

// ---------------------------------------------------------------------------
// Scenario 5: Module rows have expected fields
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_module_json_then_rows_have_expected_fields() {
    // Given: a project with source files
    // When: I run `tokmd module --format json`
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("failed to execute tokmd module --format json");

    // Then: each row has module, code, lines fields
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows should be array");

    for row in rows {
        assert!(
            row["module"].is_string(),
            "each row should have module field"
        );
        assert!(row["code"].is_number(), "each row should have code field");
        assert!(row["lines"].is_number(), "each row should have lines field");
    }
}

// ---------------------------------------------------------------------------
// Scenario 6: Module with --children separate records the mode
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_module_children_separate_then_mode_recorded() {
    // Given: a project with source files
    // When: I run `tokmd module --format json --children separate`
    let output = tokmd_cmd()
        .args(["module", "--format", "json", "--children", "separate"])
        .output()
        .expect("failed to execute tokmd module --children separate");

    // Then: the children mode is recorded in args
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        json["args"]["children"].as_str().unwrap(),
        "separate",
        "args should record children=separate"
    );
}

// ---------------------------------------------------------------------------
// Scenario 7: Module TSV output is tab-separated
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_module_tsv_then_tab_separated() {
    // Given: a project with source files
    // When: I run `tokmd module --format tsv`
    let output = tokmd_cmd()
        .args(["module", "--format", "tsv"])
        .output()
        .expect("failed to execute tokmd module --format tsv");

    // Then: output is tab-separated with header
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() >= 2, "need header + data");

    let header = lines[0];
    assert!(header.contains('\t'), "TSV header should contain tabs");
}

// ---------------------------------------------------------------------------
// Scenario 8: Module with --children parents-only records the mode
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_module_children_parents_only_then_mode_recorded() {
    // Given: a project with source files
    // When: I run `tokmd module --format json --children parents-only`
    let output = tokmd_cmd()
        .args(["module", "--format", "json", "--children", "parents-only"])
        .output()
        .expect("failed to execute tokmd module --children parents-only");

    // Then: the children mode is recorded in args
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        json["args"]["children"].as_str().unwrap(),
        "parents-only",
        "args should record children=parents-only"
    );
}
