//! CLI output-format validation tests (w66).
//!
//! Verifies that each output format flag produces well-formed output:
//! JSON parses, TSV has consistent columns, Markdown contains table markers,
//! and --output writes to files correctly.

mod common;

use assert_cmd::Command;
use serde_json::Value;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

// ===========================================================================
// 1. lang format variants
// ===========================================================================

#[test]
fn lang_json_parses_as_valid_json() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)
        .expect("lang --format json should produce valid JSON");
    assert!(json.is_object());
}

#[test]
fn lang_json_has_required_envelope_fields() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["schema_version"].is_number(), "must have schema_version");
    assert!(json["rows"].is_array(), "must have rows array");
    assert!(json["mode"].is_string(), "must have mode field");
}

#[test]
fn lang_tsv_has_header_and_consistent_columns() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "tsv"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() >= 2, "need header + at least one data row");

    let header_cols = lines[0].split('\t').count();
    assert!(header_cols >= 3, "header should have at least 3 columns");

    for (i, line) in lines[1..].iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let cols = line.split('\t').count();
        assert_eq!(
            cols, header_cols,
            "row {} has {cols} cols, header has {header_cols}",
            i + 1
        );
    }
}

#[test]
fn lang_markdown_contains_table_markers() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "md"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains('|'),
        "Markdown output should contain pipe characters for tables"
    );
    assert!(
        stdout.contains("---"),
        "Markdown output should contain separator row"
    );
}

// ===========================================================================
// 2. module format variants
// ===========================================================================

#[test]
fn module_json_parses_and_has_rows() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)
        .expect("module --format json should produce valid JSON");
    assert!(json["rows"].is_array());
    assert_eq!(json["mode"], "module");
}

#[test]
fn module_tsv_has_consistent_columns() {
    let output = tokmd_cmd()
        .args(["module", "--format", "tsv"])
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
            cols, header_cols,
            "module TSV row {} has {cols} cols, header has {header_cols}",
            i + 1
        );
    }
}

#[test]
fn module_markdown_contains_table() {
    let output = tokmd_cmd()
        .args(["module", "--format", "md"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains('|'), "module markdown should have table");
}

// ===========================================================================
// 3. export format variants
// ===========================================================================

#[test]
fn export_json_parses_and_has_mode_export() {
    let output = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)
        .expect("export --format json should produce valid JSON");
    assert_eq!(json["mode"], "export");
    assert!(json["rows"].is_array());
}

#[test]
fn export_csv_has_header_and_consistent_columns() {
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
            cols, header_cols,
            "export CSV row {} has {cols} cols, header has {header_cols}",
            i + 1
        );
    }
}

#[test]
fn export_jsonl_each_line_is_valid_json() {
    let output = tokmd_cmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(!lines.is_empty(), "should have at least one line");

    for (i, line) in lines.iter().enumerate() {
        let _: Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("JSONL line {} is not valid JSON: {e}", i + 1));
    }
}

// ===========================================================================
// 4. analyze JSON output
// ===========================================================================

#[test]
fn analyze_json_parses_as_valid_json() {
    let output = tokmd_cmd()
        .args(["analyze", "--format", "json", "--preset", "receipt"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)
        .expect("analyze --format json should produce valid JSON");
    assert!(json.is_object());
}

#[test]
fn analyze_markdown_produces_readable_output() {
    let output = tokmd_cmd()
        .args(["analyze", "--format", "md", "--preset", "receipt"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.trim().is_empty(), "analyze md should produce output");
}

// ===========================================================================
// 5. badge output
// ===========================================================================

#[test]
fn badge_produces_valid_svg() {
    let output = tokmd_cmd()
        .args(["badge", "--metric", "lines"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("<svg"), "badge should produce SVG");
    assert!(stdout.contains("</svg>"), "badge SVG should be complete");
}

// ===========================================================================
// 6. --output flag writes to file
// ===========================================================================

#[test]
fn export_output_flag_writes_to_file() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let outfile = dir.path().join("export_output.csv");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd.args(["export", "--format", "csv", "--output"])
        .arg(outfile.as_os_str())
        .assert()
        .success();

    let content = std::fs::read_to_string(&outfile).expect("read output file");
    assert!(
        content.contains(','),
        "output file should contain CSV content"
    );
}

// ===========================================================================
// 7. --no-progress flag
// ===========================================================================

#[test]
fn no_progress_flag_suppresses_spinners() {
    tokmd_cmd()
        .args(["lang", "--no-progress"])
        .assert()
        .success();
}

#[test]
fn no_progress_with_json_produces_clean_output() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json", "--no-progress"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let _: Value = serde_json::from_slice(&output.stdout)
        .expect("JSON output with --no-progress should be valid");
}

// ===========================================================================
// 8. verbose flag (global, placed before subcommand)
// ===========================================================================

#[test]
fn verbose_flag_does_not_break_output() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd.args(["-v", "lang"]).assert().success();
}

// ===========================================================================
// 9. context JSON mode
// ===========================================================================

#[test]
fn context_json_parses() {
    let output = tokmd_cmd()
        .args(["context", "--mode", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let _: Value = serde_json::from_slice(&output.stdout)
        .expect("context --mode json should produce valid JSON");
}

#[test]
fn context_list_mode_produces_output() {
    let output = tokmd_cmd()
        .args(["context", "--mode", "list"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        !stdout.trim().is_empty(),
        "context list mode should produce output"
    );
}

// ===========================================================================
// 10. handoff writes to directory
// ===========================================================================

#[test]
fn handoff_writes_to_output_dir() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let out_dir = dir.path().join("handoff_out");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd.args(["handoff", "--out-dir"])
        .arg(out_dir.as_os_str())
        .args(["--force"])
        .assert()
        .success();

    assert!(out_dir.exists(), "handoff should create output directory");
}
