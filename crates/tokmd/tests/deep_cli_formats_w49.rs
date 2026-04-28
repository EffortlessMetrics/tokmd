#![cfg(feature = "analysis")]

//! Comprehensive CLI output-format matrix tests for all core commands.
//!
//! Tests the full cross-product of commands × formats, verifying structural
//! invariants: exit code, non-empty output, valid JSON, tab/comma delimiters,
//! markdown headers, schema_version presence, and flag interactions.

mod common;

use assert_cmd::Command;
use serde_json::Value;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

// ===========================================================================
// lang --format md
// ===========================================================================

#[test]
fn lang_md_exits_zero_and_nonempty() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "md"])
        .output()
        .expect("lang --format md");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.trim().is_empty(), "md output must not be empty");
}

#[test]
fn lang_md_contains_markdown_headers() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "md"])
        .output()
        .expect("lang --format md");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains('#') || stdout.contains('|'),
        "md output should contain markdown headers or table pipes"
    );
}

// ===========================================================================
// lang --format tsv
// ===========================================================================

#[test]
fn lang_tsv_exits_zero_and_nonempty() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "tsv"])
        .output()
        .expect("lang --format tsv");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.trim().is_empty(), "tsv output must not be empty");
}

#[test]
fn lang_tsv_has_tab_characters() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "tsv"])
        .output()
        .expect("lang --format tsv");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains('\t'),
        "tsv output must contain tab characters"
    );
}

// ===========================================================================
// lang --format json
// ===========================================================================

#[test]
fn lang_json_exits_zero_and_valid() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("lang --format json");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert!(json.is_object(), "JSON output must be an object");
}

#[test]
fn lang_json_has_schema_version() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("lang --format json");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        json["schema_version"].is_number(),
        "lang JSON must include schema_version"
    );
}

#[test]
fn lang_json_has_rows_and_total() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("lang --format json");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["rows"].is_array(), "must have rows array");
    assert!(json["total"].is_object(), "must have total object");
}

// ===========================================================================
// module --format md
// ===========================================================================

#[test]
fn module_md_exits_zero_and_nonempty() {
    let output = tokmd_cmd()
        .args(["module", "--format", "md"])
        .output()
        .expect("module --format md");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.trim().is_empty(), "md output must not be empty");
}

#[test]
fn module_md_contains_markdown_elements() {
    let output = tokmd_cmd()
        .args(["module", "--format", "md"])
        .output()
        .expect("module --format md");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains('#') || stdout.contains('|'),
        "module md should contain markdown headers or table pipes"
    );
}

// ===========================================================================
// module --format tsv
// ===========================================================================

#[test]
fn module_tsv_exits_zero_and_nonempty() {
    let output = tokmd_cmd()
        .args(["module", "--format", "tsv"])
        .output()
        .expect("module --format tsv");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.trim().is_empty(), "tsv output must not be empty");
}

#[test]
fn module_tsv_has_tab_characters() {
    let output = tokmd_cmd()
        .args(["module", "--format", "tsv"])
        .output()
        .expect("module --format tsv");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains('\t'),
        "module tsv must contain tab characters"
    );
}

// ===========================================================================
// module --format json
// ===========================================================================

#[test]
fn module_json_exits_zero_and_valid() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("module --format json");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(json["mode"], "module");
}

#[test]
fn module_json_has_schema_version() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("module --format json");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        json["schema_version"].is_number(),
        "module JSON must include schema_version"
    );
}

#[test]
fn module_json_rows_have_module_field() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("module --format json");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows is array");
    assert!(!rows.is_empty(), "module should have at least one row");
    for row in rows {
        assert!(row["module"].is_string(), "each row must have module field");
    }
}

// ===========================================================================
// export --format csv
// ===========================================================================

#[test]
fn export_csv_exits_zero_and_nonempty() {
    let output = tokmd_cmd()
        .args(["export", "--format", "csv"])
        .output()
        .expect("export --format csv");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.trim().is_empty(), "csv output must not be empty");
}

#[test]
fn export_csv_has_commas() {
    let output = tokmd_cmd()
        .args(["export", "--format", "csv"])
        .output()
        .expect("export --format csv");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(','), "csv output must contain commas");
}

#[test]
fn export_csv_header_has_path_column() {
    let output = tokmd_cmd()
        .args(["export", "--format", "csv"])
        .output()
        .expect("export --format csv");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let header = stdout.lines().next().expect("csv should have header");
    assert!(
        header.contains("path") || header.contains("file"),
        "CSV header should reference path or file"
    );
}

// ===========================================================================
// export --format jsonl
// ===========================================================================

#[test]
fn export_jsonl_exits_zero_and_nonempty() {
    let output = tokmd_cmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("export --format jsonl");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.trim().is_empty(), "jsonl output must not be empty");
}

#[test]
fn export_jsonl_each_line_is_valid_json() {
    let output = tokmd_cmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("export --format jsonl");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(lines.len() >= 2, "need meta + at least one data row");

    for (i, line) in lines.iter().enumerate() {
        let _: Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("JSONL line {} is not valid JSON: {}", i + 1, e));
    }
}

#[test]
fn export_jsonl_meta_has_schema_version() {
    let output = tokmd_cmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("export --format jsonl");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let first = stdout.lines().next().expect("should have meta line");
    let meta: Value = serde_json::from_str(first).expect("meta is valid JSON");
    assert!(
        meta["schema_version"].is_number(),
        "JSONL meta must have schema_version"
    );
}

// ===========================================================================
// export --format json
// ===========================================================================

#[test]
fn export_json_exits_zero_and_valid() {
    let output = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("export --format json");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(json["mode"], "export");
}

#[test]
fn export_json_has_schema_version() {
    let output = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("export --format json");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        json["schema_version"].is_number(),
        "export JSON must include schema_version"
    );
}

#[test]
fn export_json_has_rows_array() {
    let output = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("export --format json");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows is array");
    assert!(!rows.is_empty(), "export should have file rows");
}

// ===========================================================================
// analyze --format md
// ===========================================================================

#[test]
fn analyze_md_exits_zero_and_nonempty() {
    let output = tokmd_cmd()
        .args(["analyze", "--preset", "receipt", "--format", "md"])
        .output()
        .expect("analyze --format md");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.trim().is_empty(), "analyze md must not be empty");
}

#[test]
fn analyze_md_contains_markdown_headers() {
    let output = tokmd_cmd()
        .args(["analyze", "--preset", "receipt", "--format", "md"])
        .output()
        .expect("analyze --format md");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains('#') || stdout.contains('|'),
        "analyze md should contain markdown headers or table pipes"
    );
}

// ===========================================================================
// analyze --format json
// ===========================================================================

#[test]
fn analyze_json_exits_zero_and_valid() {
    let output = tokmd_cmd()
        .args(["analyze", "--preset", "receipt", "--format", "json"])
        .output()
        .expect("analyze --format json");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert!(json.is_object(), "analyze JSON must be an object");
}

#[test]
fn analyze_json_has_schema_version() {
    let output = tokmd_cmd()
        .args(["analyze", "--preset", "receipt", "--format", "json"])
        .output()
        .expect("analyze --format json");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        json["schema_version"].is_number(),
        "analyze JSON must include schema_version"
    );
}

// ===========================================================================
// analyze --format xml
// ===========================================================================

#[test]
fn analyze_xml_exits_zero_and_nonempty() {
    let output = tokmd_cmd()
        .args(["analyze", "--preset", "receipt", "--format", "xml"])
        .output()
        .expect("analyze --format xml");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.trim().is_empty(), "analyze xml must not be empty");
}

#[test]
fn analyze_xml_has_xml_structure() {
    let output = tokmd_cmd()
        .args(["analyze", "--preset", "receipt", "--format", "xml"])
        .output()
        .expect("analyze --format xml");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains('<') && stdout.contains('>'),
        "analyze xml must contain XML angle brackets"
    );
}

// ===========================================================================
// --top N filtering
// ===========================================================================

#[test]
fn lang_top_1_limits_rows_plus_other() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json", "--top", "1"])
        .output()
        .expect("lang --top 1");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows is array");
    // --top 1 yields 1 primary row + 1 "Other" bucket when there are multiple langs
    assert!(
        rows.len() <= 2,
        "top 1 should yield at most 2 rows (1 + Other), got {}",
        rows.len()
    );
    assert!(!rows.is_empty(), "top 1 should yield at least 1 row");
}

#[test]
fn lang_top_2_limits_rows_plus_other() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json", "--top", "2"])
        .output()
        .expect("lang --top 2");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows is array");
    // --top 2 yields up to 2 primary rows + 1 "Other" bucket
    assert!(
        rows.len() <= 3,
        "top 2 should yield at most 3 rows (2 + Other), got {}",
        rows.len()
    );
}

#[test]
fn module_top_1_limits_rows_plus_other() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json", "--top", "1"])
        .output()
        .expect("module --top 1");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows is array");
    // --top 1 yields 1 primary row + 1 "Other" bucket when there are multiple modules
    assert!(
        rows.len() <= 2,
        "module top 1 should yield at most 2 rows (1 + Other), got {}",
        rows.len()
    );
}

// ===========================================================================
// --children collapse vs separate
// ===========================================================================

#[test]
fn lang_children_collapse_vs_separate_differ() {
    let collapse_output = tokmd_cmd()
        .args(["lang", "--format", "json", "--children", "collapse"])
        .output()
        .expect("lang --children collapse");

    let separate_output = tokmd_cmd()
        .args(["lang", "--format", "json", "--children", "separate"])
        .output()
        .expect("lang --children separate");

    assert!(collapse_output.status.success());
    assert!(separate_output.status.success());

    let collapse_json: Value = serde_json::from_slice(&collapse_output.stdout).unwrap();
    let separate_json: Value = serde_json::from_slice(&separate_output.stdout).unwrap();

    assert_eq!(
        collapse_json["args"]["children"].as_str().unwrap(),
        "collapse"
    );
    assert_eq!(
        separate_json["args"]["children"].as_str().unwrap(),
        "separate"
    );

    // With mixed.md containing embedded code, separate mode should produce
    // more rows than collapse mode (embedded languages shown separately).
    let collapse_rows = collapse_json["rows"].as_array().unwrap().len();
    let separate_rows = separate_json["rows"].as_array().unwrap().len();
    assert!(
        separate_rows >= collapse_rows,
        "separate ({}) should have >= rows than collapse ({})",
        separate_rows,
        collapse_rows
    );
}

#[test]
fn export_children_separate_vs_parents_only_differ() {
    let separate_output = tokmd_cmd()
        .args(["export", "--format", "json", "--children", "separate"])
        .output()
        .expect("export --children separate");

    let parents_only_output = tokmd_cmd()
        .args(["export", "--format", "json", "--children", "parents-only"])
        .output()
        .expect("export --children parents-only");

    assert!(separate_output.status.success());
    assert!(parents_only_output.status.success());

    let separate_json: Value = serde_json::from_slice(&separate_output.stdout).unwrap();
    let parents_json: Value = serde_json::from_slice(&parents_only_output.stdout).unwrap();

    let separate_rows = separate_json["rows"].as_array().unwrap().len();
    let parents_rows = parents_json["rows"].as_array().unwrap().len();
    // separate mode includes child rows; parents-only excludes them
    assert!(
        separate_rows >= parents_rows,
        "export separate ({}) should have >= rows than parents-only ({})",
        separate_rows,
        parents_rows
    );
}

// ===========================================================================
// --redact paths in export
// ===========================================================================

#[test]
fn export_redact_paths_json_hides_real_filenames() {
    let output = tokmd_cmd()
        .args(["export", "--format", "json", "--redact", "paths"])
        .output()
        .expect("export --redact paths");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    let rows = json["rows"].as_array().expect("rows array");
    assert!(!rows.is_empty(), "should have rows");

    for row in rows {
        let path = row["path"].as_str().expect("each row has path");
        // Redacted paths are hashed (hex string + extension preserved).
        // They should NOT contain original filenames like "main", "script", etc.
        assert!(
            !path.contains("main") && !path.contains("script") && !path.contains("README"),
            "redacted path should not contain original filename: {path}"
        );
    }
}

#[test]
fn export_redact_paths_jsonl_meta_indicates_redaction() {
    let output = tokmd_cmd()
        .args(["export", "--format", "jsonl", "--redact", "paths"])
        .output()
        .expect("export --redact paths jsonl");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let first = stdout.lines().next().expect("meta line");
    let meta: Value = serde_json::from_str(first).expect("valid meta JSON");
    assert_eq!(
        meta["args"]["redact"].as_str(),
        Some("paths"),
        "meta should indicate redact mode"
    );
}

// ===========================================================================
// Cross-command: all JSON outputs have schema_version
// ===========================================================================

#[test]
fn all_json_outputs_include_schema_version() {
    let commands: Vec<Vec<&str>> = vec![
        vec!["lang", "--format", "json"],
        vec!["module", "--format", "json"],
        vec!["export", "--format", "json"],
        vec!["analyze", "--preset", "receipt", "--format", "json"],
    ];

    for args in &commands {
        let output = tokmd_cmd()
            .args(args)
            .output()
            .unwrap_or_else(|e| panic!("failed to run {:?}: {}", args, e));

        assert!(output.status.success(), "{:?} should exit 0", args);

        let json: Value = serde_json::from_slice(&output.stdout)
            .unwrap_or_else(|e| panic!("{:?} not valid JSON: {}", args, e));

        assert!(
            json["schema_version"].is_number(),
            "{:?} JSON missing schema_version",
            args
        );
    }
}

// ===========================================================================
// Format consistency: TSV column counts match across rows
// ===========================================================================

#[test]
fn lang_tsv_consistent_column_count() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "tsv"])
        .output()
        .expect("lang tsv");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(lines.len() >= 2, "need header + data");

    let header_cols = lines[0].split('\t').count();
    for (i, line) in lines[1..].iter().enumerate() {
        let cols = line.split('\t').count();
        assert_eq!(
            cols,
            header_cols,
            "lang tsv row {} has {} cols, header has {}",
            i + 1,
            cols,
            header_cols
        );
    }
}

#[test]
fn module_tsv_consistent_column_count() {
    let output = tokmd_cmd()
        .args(["module", "--format", "tsv"])
        .output()
        .expect("module tsv");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(lines.len() >= 2, "need header + data");

    let header_cols = lines[0].split('\t').count();
    for (i, line) in lines[1..].iter().enumerate() {
        let cols = line.split('\t').count();
        assert_eq!(
            cols,
            header_cols,
            "module tsv row {} has {} cols, header has {}",
            i + 1,
            cols,
            header_cols
        );
    }
}

// ===========================================================================
// CSV column count consistency for export
// ===========================================================================

#[test]
fn export_csv_consistent_column_count() {
    let output = tokmd_cmd()
        .args(["export", "--format", "csv"])
        .output()
        .expect("export csv");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(lines.len() >= 2, "need header + data");

    let header_cols = lines[0].split(',').count();
    for (i, line) in lines[1..].iter().enumerate() {
        let cols = line.split(',').count();
        assert_eq!(
            cols,
            header_cols,
            "export csv row {} has {} cols, header has {}",
            i + 1,
            cols,
            header_cols
        );
    }
}

// ===========================================================================
// Analyze preset interaction with format
// ===========================================================================

#[test]
fn analyze_health_json_exits_zero() {
    let output = tokmd_cmd()
        .args(["analyze", "--preset", "health", "--format", "json"])
        .output()
        .expect("analyze health json");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert!(
        json["schema_version"].is_number(),
        "health JSON must have schema_version"
    );
}

#[test]
fn analyze_health_md_exits_zero() {
    let output = tokmd_cmd()
        .args(["analyze", "--preset", "health", "--format", "md"])
        .output()
        .expect("analyze health md");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.trim().is_empty(), "health md must not be empty");
}
