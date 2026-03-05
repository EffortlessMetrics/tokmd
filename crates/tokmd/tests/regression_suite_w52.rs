//! Regression test suite – W52
//!
//! Captures known-good behavior to prevent future regressions in receipt
//! structure, output formats, path handling, arithmetic invariants, and
//! serialization round-trips.

mod common;

use assert_cmd::Command;
use serde_json::Value;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

fn run_json(args: &[&str]) -> Value {
    let output = tokmd_cmd()
        .args(args)
        .output()
        .expect("failed to execute tokmd");
    assert!(
        output.status.success(),
        "tokmd failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("stdout is not valid JSON")
}

fn run_stdout(args: &[&str]) -> String {
    let output = tokmd_cmd()
        .args(args)
        .output()
        .expect("failed to execute tokmd");
    assert!(
        output.status.success(),
        "tokmd failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("non-UTF-8 stdout")
}

// ===========================================================================
// 1. Receipt structure regression
// ===========================================================================

#[test]
fn lang_json_receipt_has_required_keys() {
    let json = run_json(&["lang", "--format", "json"]);
    assert!(json["schema_version"].is_number(), "missing schema_version");
    assert!(json["rows"].is_array(), "missing rows");
    assert!(json["total"].is_object(), "missing total");
    assert!(json["args"].is_object(), "missing args");
    assert!(
        json["generated_at_ms"].is_number(),
        "missing generated_at_ms"
    );
}

#[test]
fn module_json_receipt_has_required_keys() {
    let json = run_json(&["module", "--format", "json"]);
    assert!(json["schema_version"].is_number(), "missing schema_version");
    assert!(json["rows"].is_array(), "missing rows");
    assert!(json["total"].is_object(), "missing total");
    assert!(json["args"].is_object(), "missing args");
}

#[test]
fn export_json_receipt_has_required_keys() {
    let json = run_json(&["export", "--format", "json"]);
    assert!(json["schema_version"].is_number(), "missing schema_version");
    assert!(json["rows"].is_array(), "missing rows");
    assert!(json["args"].is_object(), "missing args");
}

#[test]
fn analysis_json_receipt_has_required_keys() {
    let json = run_json(&["analyze", "--preset", "receipt", "--format", "json"]);
    assert!(json["schema_version"].is_number(), "missing schema_version");
    assert!(json["warnings"].is_array(), "missing warnings");
    assert!(
        json["mode"].as_str() == Some("analysis"),
        "mode != analysis"
    );
    assert!(json["args"].is_object(), "missing args");
    assert!(json["source"].is_object(), "missing source");
    // receipt preset always produces derived metrics
    assert!(
        json["derived"].is_object(),
        "missing derived with receipt preset"
    );
}

#[test]
fn all_schema_versions_are_numeric() {
    let lang = run_json(&["lang", "--format", "json"]);
    let module = run_json(&["module", "--format", "json"]);
    let export = run_json(&["export", "--format", "json"]);
    let analysis = run_json(&["analyze", "--preset", "receipt", "--format", "json"]);

    for (name, json) in [
        ("lang", &lang),
        ("module", &module),
        ("export", &export),
        ("analysis", &analysis),
    ] {
        assert!(
            json["schema_version"].is_u64(),
            "{name} schema_version is not a u64"
        );
    }
}

#[test]
fn all_receipts_include_tool_version() {
    let lang = run_json(&["lang", "--format", "json"]);
    let module = run_json(&["module", "--format", "json"]);
    let export = run_json(&["export", "--format", "json"]);
    let analysis = run_json(&["analyze", "--preset", "receipt", "--format", "json"]);

    for (name, json) in [
        ("lang", &lang),
        ("module", &module),
        ("export", &export),
        ("analysis", &analysis),
    ] {
        let version = &json["tool"]["version"];
        assert!(version.is_string(), "{name} missing tool.version");
        assert!(
            !version.as_str().unwrap().is_empty(),
            "{name} tool.version is empty"
        );
    }
}

#[test]
fn totals_values_are_non_negative() {
    let json = run_json(&["lang", "--format", "json"]);
    let total = &json["total"];
    for field in ["code", "lines", "files", "bytes", "tokens"] {
        assert!(
            total[field].is_u64(),
            "total.{field} is not a non-negative integer"
        );
    }
}

#[test]
fn empty_dir_produces_valid_receipt_with_zero_values() {
    let dir = tempfile::tempdir().expect("create temp dir");
    std::fs::create_dir_all(dir.path().join(".git")).expect("create .git marker");

    let output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args(["lang", "--format", "json"])
        .output()
        .expect("failed to execute");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).expect("not valid JSON");
    assert_eq!(json["total"]["code"], 0);
    assert_eq!(json["total"]["files"], 0);
    assert!(json["rows"].as_array().unwrap().is_empty());
}

// ===========================================================================
// 2. Output format regression
// ===========================================================================

#[test]
fn markdown_output_starts_with_header_or_table() {
    let stdout = run_stdout(&["lang"]);
    let first_line = stdout.lines().next().expect("no output");
    assert!(
        first_line.starts_with('#') || first_line.starts_with('|'),
        "markdown first line doesn't start with # or |: {first_line}"
    );
}

#[test]
fn tsv_first_line_is_header() {
    let stdout = run_stdout(&["lang", "--format", "tsv"]);
    let header = stdout.lines().next().expect("no TSV output");
    assert!(header.contains('\t'), "TSV header has no tabs: {header}");
    let lower = header.to_lowercase();
    assert!(
        lower.contains("language") || lower.contains("lang") || lower.contains("code"),
        "TSV header missing expected column names: {header}"
    );
}

#[test]
fn csv_consistent_column_count() {
    let stdout = run_stdout(&["export", "--format", "csv"]);
    let mut counts = stdout.lines().map(|l| l.split(',').count());
    let first = counts.next().expect("no CSV output");
    assert!(first > 1, "CSV has only one column");
    for (i, count) in counts.enumerate() {
        assert_eq!(
            count, first,
            "CSV row {i} has {count} columns, expected {first}"
        );
    }
}

#[test]
fn jsonl_each_line_is_valid_json() {
    let stdout = run_stdout(&["export", "--format", "jsonl"]);
    for (i, line) in stdout.lines().enumerate() {
        let parsed: Result<Value, _> = serde_json::from_str(line);
        assert!(parsed.is_ok(), "JSONL line {i} is not valid JSON: {line}");
    }
}

#[test]
fn json_top_level_has_expected_keys() {
    let json = run_json(&["lang", "--format", "json"]);
    let obj = json.as_object().expect("top-level is not an object");
    for key in ["schema_version", "mode", "tool", "rows", "total"] {
        assert!(obj.contains_key(key), "missing top-level key: {key}");
    }
}

#[test]
fn xml_output_has_valid_root_element() {
    let stdout = run_stdout(&["analyze", "--preset", "receipt", "--format", "xml"]);
    let trimmed = stdout.trim();
    assert!(
        trimmed.starts_with('<'),
        "XML output does not start with <: {}",
        &trimmed[..trimmed.len().min(80)]
    );
    assert!(trimmed.ends_with('>'), "XML output does not end with >");
}

// ===========================================================================
// 3. Path handling regression
// ===========================================================================

#[test]
fn no_backslash_paths_in_json_output() {
    let json = run_json(&["export", "--format", "json"]);
    let rows = json["rows"].as_array().expect("rows is not an array");
    for row in rows {
        let path = row["path"].as_str().unwrap_or_default();
        assert!(
            !path.contains('\\'),
            "backslash found in export path: {path}"
        );
    }
}

#[test]
fn module_keys_never_start_with_slash() {
    let json = run_json(&["module", "--format", "json"]);
    let rows = json["rows"].as_array().expect("rows is not an array");
    for row in rows {
        let module = row["module"].as_str().unwrap_or_default();
        assert!(
            !module.starts_with('/'),
            "module key starts with /: {module}"
        );
    }
}

#[test]
fn export_paths_are_relative() {
    let json = run_json(&["export", "--format", "json"]);
    let rows = json["rows"].as_array().expect("rows is not an array");
    for row in rows {
        let path = row["path"].as_str().unwrap_or_default();
        assert!(
            !path.starts_with('/'),
            "export path is absolute (starts with /): {path}"
        );
        // Windows absolute paths
        assert!(
            !(path.len() >= 2 && path.as_bytes()[1] == b':'),
            "export path is absolute (drive letter): {path}"
        );
    }
}

#[test]
fn export_rows_sorted_by_code_desc_then_path() {
    let json = run_json(&["export", "--format", "json"]);
    let rows = json["rows"].as_array().expect("rows is not an array");
    for window in rows.windows(2) {
        let code_a = window[0]["code"].as_u64().unwrap();
        let code_b = window[1]["code"].as_u64().unwrap();
        let path_a = window[0]["path"].as_str().unwrap();
        let path_b = window[1]["path"].as_str().unwrap();
        assert!(
            code_a > code_b || (code_a == code_b && path_a <= path_b),
            "rows not sorted (code desc, path asc): ({code_a}, {path_a}) before ({code_b}, {path_b})"
        );
    }
}

// ===========================================================================
// 4. Arithmetic regression
// ===========================================================================

#[test]
fn total_code_equals_sum_of_lang_code() {
    let json = run_json(&["lang", "--format", "json"]);
    let total_code = json["total"]["code"].as_u64().expect("total.code");
    let sum: u64 = json["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["code"].as_u64().unwrap())
        .sum();
    assert_eq!(
        total_code, sum,
        "total.code ({total_code}) != sum of rows ({sum})"
    );
}

#[test]
fn file_lines_gte_code_plus_comments_plus_blanks() {
    let json = run_json(&["export", "--format", "json"]);
    let rows = json["rows"].as_array().expect("rows is not an array");
    for row in rows {
        let code = row["code"].as_u64().unwrap_or(0);
        let comments = row["comments"].as_u64().unwrap_or(0);
        let blanks = row["blanks"].as_u64().unwrap_or(0);
        let lines = row["lines"].as_u64().unwrap_or(0);
        assert!(
            lines >= code + comments + blanks,
            "lines ({lines}) < code ({code}) + comments ({comments}) + blanks ({blanks}) for {}",
            row["path"].as_str().unwrap_or("?")
        );
    }
}

#[test]
fn no_negative_values_in_any_numeric_field() {
    let json = run_json(&["lang", "--format", "json"]);
    for row in json["rows"].as_array().unwrap() {
        for field in ["code", "lines", "files", "bytes", "tokens"] {
            assert!(
                row[field].as_u64().is_some(),
                "row field {field} is negative or not a number: {:?}",
                row[field]
            );
        }
    }
    let total = &json["total"];
    for field in ["code", "lines", "files", "bytes", "tokens"] {
        assert!(
            total[field].as_u64().is_some(),
            "total.{field} is negative or not a number: {:?}",
            total[field]
        );
    }
}

#[test]
fn top_n_other_row_preserves_remaining_totals() {
    // Get full results first
    let full = run_json(&["lang", "--format", "json"]);
    let full_rows = full["rows"].as_array().unwrap();
    if full_rows.len() <= 1 {
        // Not enough languages to test top-N
        return;
    }

    let top = run_json(&["lang", "--format", "json", "--top", "1"]);
    let top_rows = top["rows"].as_array().unwrap();
    let top_total = top["total"]["code"].as_u64().unwrap();

    // Sum of all top rows (including potential "Other") should equal total
    let top_sum: u64 = top_rows.iter().map(|r| r["code"].as_u64().unwrap()).sum();
    assert_eq!(
        top_sum, top_total,
        "top-1 rows sum ({top_sum}) != total ({top_total})"
    );

    // Total should match full scan total
    let full_total = full["total"]["code"].as_u64().unwrap();
    assert_eq!(top_total, full_total, "top-1 total differs from full total");
}

// ===========================================================================
// 5. Serde regression
// ===========================================================================

#[test]
fn lang_receipt_roundtrips_through_json() {
    let json = run_json(&["lang", "--format", "json"]);
    let serialized = serde_json::to_string(&json).expect("re-serialize");
    let roundtrip: Value = serde_json::from_str(&serialized).expect("re-parse");
    assert_eq!(
        json, roundtrip,
        "lang receipt did not roundtrip through JSON"
    );
}

#[test]
fn receipt_enums_serialize_to_expected_strings() {
    let lang = run_json(&["lang", "--format", "json"]);
    assert_eq!(lang["mode"].as_str(), Some("lang"), "lang mode");
    assert_eq!(lang["status"].as_str(), Some("complete"), "lang status");

    let module = run_json(&["module", "--format", "json"]);
    assert_eq!(module["mode"].as_str(), Some("module"), "module mode");

    let export = run_json(&["export", "--format", "json"]);
    assert_eq!(export["mode"].as_str(), Some("export"), "export mode");

    let analysis = run_json(&["analyze", "--preset", "receipt", "--format", "json"]);
    assert_eq!(analysis["mode"].as_str(), Some("analysis"), "analysis mode");
}

#[test]
fn unknown_format_does_not_crash() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "bogus"])
        .output()
        .expect("failed to execute");
    // Should fail gracefully (non-zero exit), not panic
    assert!(
        !output.status.success(),
        "expected failure for unknown format"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    // clap produces an error message, not a panic backtrace
    assert!(
        !stderr.contains("panicked"),
        "tokmd panicked on unknown format: {stderr}"
    );
}

#[test]
fn flatten_puts_report_fields_at_top_level() {
    let json = run_json(&["lang", "--format", "json"]);
    let obj = json.as_object().unwrap();
    // rows and total come from flattened LangReport
    assert!(obj.contains_key("rows"), "rows not at top level");
    assert!(obj.contains_key("total"), "total not at top level");
    // Should NOT be nested under a "report" key
    assert!(
        !obj.contains_key("report"),
        "report field should be flattened"
    );

    let module = run_json(&["module", "--format", "json"]);
    let mobj = module.as_object().unwrap();
    assert!(mobj.contains_key("rows"), "module rows not at top level");
    assert!(mobj.contains_key("total"), "module total not at top level");
    assert!(
        !mobj.contains_key("report"),
        "module report should be flattened"
    );

    let export = run_json(&["export", "--format", "json"]);
    let eobj = export.as_object().unwrap();
    assert!(eobj.contains_key("rows"), "export rows not at top level");
    assert!(
        !eobj.contains_key("data"),
        "export data should be flattened"
    );
}
