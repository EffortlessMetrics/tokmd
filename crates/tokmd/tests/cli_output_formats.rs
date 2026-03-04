//! Cross-command output-format tests validating structural invariants of
//! JSON, TSV, CSV, and JSONL outputs.

mod common;

use assert_cmd::Command;
use serde_json::Value;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

// ===========================================================================
// lang --format json
// ===========================================================================

#[test]
fn lang_json_has_schema_version_field() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        json["schema_version"].is_number(),
        "JSON output must include schema_version"
    );
}

#[test]
fn lang_json_rows_sorted_descending_by_code() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows is array");

    // Filter out "Other" bucket which may break sort expectation
    let code_vals: Vec<u64> = rows
        .iter()
        .filter(|r| r["lang"].as_str() != Some("Other"))
        .filter_map(|r| r["code"].as_u64())
        .collect();

    for window in code_vals.windows(2) {
        assert!(
            window[0] >= window[1],
            "rows should be sorted descending by code: {} < {}",
            window[0],
            window[1]
        );
    }
}

#[test]
fn lang_json_has_args_metadata() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        json["args"].is_object(),
        "JSON should contain args metadata"
    );
}

// ===========================================================================
// lang --format tsv
// ===========================================================================

#[test]
fn lang_tsv_header_contains_expected_columns() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "tsv"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let header = stdout.lines().next().expect("should have header line");
    assert!(header.contains('\t'), "header should be tab-separated");
    assert!(
        header.contains("Code") || header.contains("code"),
        "header should contain Code column"
    );
}

#[test]
fn lang_tsv_data_rows_have_same_column_count_as_header() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "tsv"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() >= 2, "need header + data");

    let header_cols = lines[0].split('\t').count();
    for (i, line) in lines[1..].iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let cols = line.split('\t').count();
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

// ===========================================================================
// module --format json
// ===========================================================================

#[test]
fn module_json_rows_have_module_field() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows is array");

    for row in rows {
        assert!(
            row["module"].is_string(),
            "each row should have module field"
        );
        assert!(row["code"].is_number(), "each row should have code field");
    }
}

#[test]
fn module_json_has_mode_field() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["mode"], "module");
}

// ===========================================================================
// export --format csv
// ===========================================================================

#[test]
fn export_csv_header_contains_path_and_language() {
    let output = tokmd_cmd()
        .args(["export", "--format", "csv"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let header = stdout.lines().next().expect("should have header");
    assert!(
        header.contains("path") || header.contains("file"),
        "CSV header should have path/file column"
    );
    assert!(
        header.contains("language") || header.contains("lang"),
        "CSV header should have language column"
    );
}

#[test]
fn export_csv_rows_have_consistent_column_count() {
    let output = tokmd_cmd()
        .args(["export", "--format", "csv"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() >= 2, "need header + data");

    let header_cols = lines[0].split(',').count();
    for (i, line) in lines[1..].iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let cols = line.split(',').count();
        assert_eq!(
            cols,
            header_cols,
            "CSV row {} has {} cols, header has {}",
            i + 1,
            cols,
            header_cols
        );
    }
}

// ===========================================================================
// export --format jsonl
// ===========================================================================

#[test]
fn export_jsonl_meta_line_has_schema_version() {
    let output = tokmd_cmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let first = stdout.lines().next().expect("should have first line");
    let meta: Value = serde_json::from_str(first).expect("first line is JSON");
    assert!(
        meta["schema_version"].is_number(),
        "meta line should have schema_version"
    );
}

#[test]
fn export_jsonl_data_rows_have_path_field() {
    let output = tokmd_cmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(lines.len() >= 2, "need meta + at least one data row");

    // Skip first line (meta), check data rows
    for (i, line) in lines[1..].iter().enumerate() {
        let row: Value =
            serde_json::from_str(line).unwrap_or_else(|e| panic!("line {} invalid: {}", i + 2, e));
        assert!(
            row["path"].is_string() || row["file"].is_string(),
            "data row {} should have path or file field",
            i + 2
        );
    }
}

// ===========================================================================
// export --format json
// ===========================================================================

#[test]
fn export_json_envelope_has_mode_export() {
    let output = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["mode"], "export");
    assert!(json["schema_version"].is_number());
    assert!(json["rows"].is_array());
}

#[test]
fn export_json_rows_have_code_and_language() {
    let output = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows array");

    for row in rows {
        assert!(row["code"].is_number(), "each row should have code");
        assert!(
            row["language"].is_string() || row["lang"].is_string(),
            "each row should have language"
        );
    }
}
