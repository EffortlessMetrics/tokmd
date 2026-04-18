//! W72 – JSON receipt contract validation tests.
//!
//! Verifies that every CLI command producing JSON output conforms to the
//! documented schema contracts: required fields, types, version numbers, and
//! structural invariants such as key ordering (BTreeMap guarantee).

mod common;

use assert_cmd::Command;
use serde_json::Value;

// Schema version constants mirrored from the crate sources.
const CORE_SCHEMA_VERSION: u32 = 2;
const ANALYSIS_SCHEMA_VERSION: u32 = 9;

// ── helpers ──────────────────────────────────────────────────────────────

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
        "tokmd {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    serde_json::from_str(&stdout).expect("output is not valid JSON")
}

fn run_raw(args: &[&str]) -> String {
    let output = tokmd_cmd()
        .args(args)
        .output()
        .expect("failed to execute tokmd");
    assert!(
        output.status.success(),
        "tokmd {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("invalid UTF-8")
}

/// Assert that all keys in a JSON object are in sorted order (BTreeMap guarantee).
fn assert_keys_sorted(val: &Value, context: &str) {
    if let Some(obj) = val.as_object() {
        let keys: Vec<&String> = obj.keys().collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted, "keys not sorted in {context}");

        for (k, v) in obj {
            assert_keys_sorted(v, &format!("{context}.{k}"));
        }
    } else if let Some(arr) = val.as_array() {
        for (i, v) in arr.iter().enumerate() {
            assert_keys_sorted(v, &format!("{context}[{i}]"));
        }
    }
}

/// Assert a value is not null (recursively checks required fields only at top level).
fn assert_not_null(val: &Value, field: &str) {
    assert!(!val.is_null(), "field '{field}' must not be null");
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. `tokmd lang --format json` – envelope & rows
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w72_lang_json_has_schema_version() {
    let json = run_json(&["lang", "--format", "json"]);
    assert_eq!(json["schema_version"], CORE_SCHEMA_VERSION);
}

#[test]
fn w72_lang_json_has_tool_metadata() {
    let json = run_json(&["lang", "--format", "json"]);
    let tool = &json["tool"];
    assert!(tool.is_object(), "tool should be an object");
    assert_eq!(tool["name"], "tokmd");
    assert!(
        tool["version"].is_string(),
        "tool.version should be a string"
    );
}

#[test]
fn w72_lang_json_has_required_envelope_fields() {
    let json = run_json(&["lang", "--format", "json"]);
    assert_eq!(json["mode"], "lang");
    assert!(json["generated_at_ms"].is_number());
    assert!(json["status"].is_string());
    assert!(json["warnings"].is_array());
    assert!(json["scan"].is_object());
    assert!(json["args"].is_object());
}

#[test]
fn w72_lang_json_rows_have_required_fields() {
    let json = run_json(&["lang", "--format", "json"]);
    let rows = json["rows"].as_array().expect("rows must be an array");
    assert!(!rows.is_empty());

    for row in rows {
        assert!(row["lang"].is_string(), "row.lang must be a string");
        assert!(row["code"].is_number(), "row.code must be a number");
        assert!(row["lines"].is_number(), "row.lines must be a number");
        assert!(row["files"].is_number(), "row.files must be a number");
        assert!(row["bytes"].is_number(), "row.bytes must be a number");
        assert!(row["tokens"].is_number(), "row.tokens must be a number");
        assert!(
            row["avg_lines"].is_number(),
            "row.avg_lines must be a number"
        );
    }
}

#[test]
fn w72_lang_json_has_total() {
    let json = run_json(&["lang", "--format", "json"]);
    let total = &json["total"];
    assert!(total.is_object(), "total must be present");
    assert!(total["code"].is_number());
    assert!(total["lines"].is_number());
    assert!(total["files"].is_number());
    assert!(total["bytes"].is_number());
    assert!(total["tokens"].is_number());
}

#[test]
fn w72_lang_json_non_null_required_fields() {
    let json = run_json(&["lang", "--format", "json"]);
    for field in &[
        "schema_version",
        "generated_at_ms",
        "tool",
        "mode",
        "status",
        "warnings",
        "scan",
        "args",
        "rows",
        "total",
    ] {
        assert_not_null(&json[*field], field);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. `tokmd module --format json` – envelope & rows
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w72_module_json_has_schema_version() {
    let json = run_json(&["module", "--format", "json"]);
    assert_eq!(json["schema_version"], CORE_SCHEMA_VERSION);
}

#[test]
fn w72_module_json_has_required_envelope_fields() {
    let json = run_json(&["module", "--format", "json"]);
    assert_eq!(json["mode"], "module");
    assert!(json["generated_at_ms"].is_number());
    assert!(json["tool"].is_object());
    assert!(json["status"].is_string());
    assert!(json["warnings"].is_array());
    assert!(json["scan"].is_object());
    assert!(json["args"].is_object());
}

#[test]
fn w72_module_json_rows_have_required_fields() {
    let json = run_json(&["module", "--format", "json"]);
    let rows = json["rows"].as_array().expect("rows must be an array");
    assert!(!rows.is_empty());

    for row in rows {
        assert!(row["module"].is_string(), "row.module must be a string");
        assert!(row["code"].is_number(), "row.code must be a number");
        assert!(row["lines"].is_number(), "row.lines must be a number");
        assert!(row["files"].is_number(), "row.files must be a number");
        assert!(row["bytes"].is_number(), "row.bytes must be a number");
        assert!(row["tokens"].is_number(), "row.tokens must be a number");
        assert!(
            row["avg_lines"].is_number(),
            "row.avg_lines must be a number"
        );
    }
}

#[test]
fn w72_module_json_has_total() {
    let json = run_json(&["module", "--format", "json"]);
    let total = &json["total"];
    assert!(total.is_object(), "total must be present");
    assert!(total["code"].is_number());
    assert!(total["lines"].is_number());
    assert!(total["files"].is_number());
}

#[test]
fn w72_module_json_has_root_module() {
    let json = run_json(&["module", "--format", "json"]);
    let rows = json["rows"].as_array().unwrap();
    let has_root = rows.iter().any(|r| r["module"] == "(root)");
    assert!(has_root, "fixture must produce a (root) module");
}

#[test]
fn w72_module_json_non_null_required_fields() {
    let json = run_json(&["module", "--format", "json"]);
    for field in &[
        "schema_version",
        "generated_at_ms",
        "tool",
        "mode",
        "status",
        "warnings",
        "scan",
        "args",
        "rows",
        "total",
    ] {
        assert_not_null(&json[*field], field);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. `tokmd export --format jsonl` – line-delimited JSON
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w72_export_jsonl_all_lines_valid_json() {
    let raw = run_raw(&["export", "--format", "jsonl"]);
    let lines: Vec<&str> = raw.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(lines.len() >= 2, "need meta + at least one data row");

    for (i, line) in lines.iter().enumerate() {
        let _: Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("line {} is not valid JSON: {e}", i + 1));
    }
}

#[test]
fn w72_export_jsonl_meta_has_schema_version() {
    let raw = run_raw(&["export", "--format", "jsonl"]);
    let first = raw.lines().next().expect("need at least one line");
    let meta: Value = serde_json::from_str(first).expect("meta line must be valid JSON");
    assert_eq!(meta["schema_version"], CORE_SCHEMA_VERSION);
    assert_eq!(meta["mode"], "export");
    assert!(meta["tool"].is_object());
}

#[test]
fn w72_export_jsonl_data_rows_have_required_fields() {
    let raw = run_raw(&["export", "--format", "jsonl"]);
    let lines: Vec<&str> = raw.lines().filter(|l| !l.trim().is_empty()).collect();

    for line in lines.iter().skip(1) {
        let row: Value = serde_json::from_str(line).unwrap();
        assert!(row["path"].is_string(), "data row must have path");
        assert!(row["lang"].is_string(), "data row must have lang");
        assert!(row["code"].is_number(), "data row must have code");
        assert!(row["lines"].is_number(), "data row must have lines");
        assert!(row["blanks"].is_number(), "data row must have blanks");
        assert!(row["comments"].is_number(), "data row must have comments");
        assert!(row["bytes"].is_number(), "data row must have bytes");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. `tokmd export --format json` – envelope
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w72_export_json_is_valid_envelope() {
    let json = run_json(&["export", "--format", "json"]);
    assert_eq!(json["mode"], "export");
    assert_eq!(json["schema_version"], CORE_SCHEMA_VERSION);
    assert!(json["generated_at_ms"].is_number());
    assert!(json["tool"].is_object());
    assert!(json["rows"].is_array());
}

#[test]
fn w72_export_json_rows_have_file_fields() {
    let json = run_json(&["export", "--format", "json"]);
    let rows = json["rows"].as_array().expect("rows must be array");
    assert!(!rows.is_empty());

    let first = &rows[0];
    assert!(first["path"].is_string());
    assert!(first["lang"].is_string());
    assert!(first["module"].is_string());
    assert!(first["code"].is_number());
    assert!(first["comments"].is_number());
    assert!(first["blanks"].is_number());
    assert!(first["lines"].is_number());
    assert!(first["bytes"].is_number());
    assert!(first["tokens"].is_number());
}

#[test]
fn w72_export_json_non_null_required_fields() {
    let json = run_json(&["export", "--format", "json"]);
    for field in &[
        "schema_version",
        "generated_at_ms",
        "tool",
        "mode",
        "status",
        "warnings",
        "scan",
        "args",
        "rows",
    ] {
        assert_not_null(&json[*field], field);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. `tokmd analyze --format json --preset receipt` – analysis envelope
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(feature = "analysis")]
#[test]
fn w72_analyze_receipt_has_schema_version() {
    let json = run_json(&["analyze", "--format", "json", "--preset", "receipt"]);
    assert_eq!(json["schema_version"], ANALYSIS_SCHEMA_VERSION);
}

#[cfg(feature = "analysis")]
#[test]
fn w72_analyze_receipt_has_required_envelope() {
    let json = run_json(&["analyze", "--format", "json", "--preset", "receipt"]);
    assert_eq!(json["mode"], "analysis");
    assert!(json["generated_at_ms"].is_number());
    assert!(json["tool"].is_object());
    assert!(json["status"].is_string());
    assert!(json["warnings"].is_array());
    assert!(json["source"].is_object());
    assert!(json["args"].is_object());
}

#[cfg(feature = "analysis")]
#[test]
fn w72_analyze_receipt_has_derived_section() {
    let json = run_json(&["analyze", "--format", "json", "--preset", "receipt"]);
    assert!(
        json["derived"].is_object(),
        "receipt preset must include derived section"
    );
}

#[cfg(feature = "analysis")]
#[test]
fn w72_analyze_receipt_tool_metadata() {
    let json = run_json(&["analyze", "--format", "json", "--preset", "receipt"]);
    assert_eq!(json["tool"]["name"], "tokmd");
    assert!(json["tool"]["version"].is_string());
}

#[cfg(feature = "analysis")]
#[test]
fn w72_analyze_receipt_args_preset() {
    let json = run_json(&["analyze", "--format", "json", "--preset", "receipt"]);
    assert_eq!(json["args"]["preset"], "receipt");
    assert_eq!(json["args"]["format"], "json");
}

#[cfg(feature = "analysis")]
#[test]
fn w72_analyze_receipt_non_null_required_fields() {
    let json = run_json(&["analyze", "--format", "json", "--preset", "receipt"]);
    for field in &[
        "schema_version",
        "generated_at_ms",
        "tool",
        "mode",
        "status",
        "warnings",
        "source",
        "args",
    ] {
        assert_not_null(&json[*field], field);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Envelope metadata consistency across commands
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(feature = "analysis")]
#[test]
fn w72_all_json_envelopes_contain_tool_name_tokmd() {
    let commands: &[&[&str]] = &[
        &["lang", "--format", "json"],
        &["module", "--format", "json"],
        &["export", "--format", "json"],
        &["analyze", "--format", "json", "--preset", "receipt"],
    ];
    for args in commands {
        let json = run_json(args);
        assert_eq!(
            json["tool"]["name"],
            "tokmd",
            "tool.name must be 'tokmd' for: {}",
            args.join(" ")
        );
    }
}

#[cfg(feature = "analysis")]
#[test]
fn w72_all_json_envelopes_have_timestamp() {
    let commands: &[&[&str]] = &[
        &["lang", "--format", "json"],
        &["module", "--format", "json"],
        &["export", "--format", "json"],
        &["analyze", "--format", "json", "--preset", "receipt"],
    ];
    for args in commands {
        let json = run_json(args);
        let ts = json["generated_at_ms"].as_u64();
        assert!(
            ts.is_some(),
            "generated_at_ms must be a positive number for: {}",
            args.join(" ")
        );
        assert!(
            ts.unwrap() > 0,
            "timestamp must be non-zero for: {}",
            args.join(" ")
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. All JSON outputs: valid JSON (parse succeeds)
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(feature = "analysis")]
#[test]
fn w72_all_json_commands_produce_valid_json() {
    let commands: &[&[&str]] = &[
        &["lang", "--format", "json"],
        &["module", "--format", "json"],
        &["export", "--format", "json"],
        &["analyze", "--format", "json", "--preset", "receipt"],
    ];
    for args in commands {
        let raw = run_raw(args);
        let parsed: Result<Value, _> = serde_json::from_str(&raw);
        assert!(
            parsed.is_ok(),
            "output must be valid JSON for: {}",
            args.join(" ")
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. All JSON outputs: keys are sorted (BTreeMap guarantee)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w72_lang_json_keys_sorted() {
    let json = run_json(&["lang", "--format", "json"]);
    assert_keys_sorted(&json, "lang");
}

#[test]
fn w72_module_json_keys_sorted() {
    let json = run_json(&["module", "--format", "json"]);
    assert_keys_sorted(&json, "module");
}

#[test]
fn w72_export_json_keys_sorted() {
    let json = run_json(&["export", "--format", "json"]);
    assert_keys_sorted(&json, "export");
}

#[cfg(feature = "analysis")]
#[test]
fn w72_analyze_json_keys_sorted() {
    let json = run_json(&["analyze", "--format", "json", "--preset", "receipt"]);
    assert_keys_sorted(&json, "analyze");
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. schema_version values match expected constants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w72_core_receipts_share_schema_version() {
    let commands: &[&[&str]] = &[
        &["lang", "--format", "json"],
        &["module", "--format", "json"],
        &["export", "--format", "json"],
    ];
    for args in commands {
        let json = run_json(args);
        assert_eq!(
            json["schema_version"].as_u64().unwrap(),
            u64::from(CORE_SCHEMA_VERSION),
            "schema_version mismatch for: {}",
            args.join(" ")
        );
    }
}

#[cfg(feature = "analysis")]
#[test]
fn w72_analysis_receipt_schema_version_matches() {
    let json = run_json(&["analyze", "--format", "json", "--preset", "receipt"]);
    assert_eq!(
        json["schema_version"].as_u64().unwrap(),
        u64::from(ANALYSIS_SCHEMA_VERSION),
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. Numeric fields are non-negative integers
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w72_lang_row_numeric_fields_non_negative() {
    let json = run_json(&["lang", "--format", "json"]);
    let rows = json["rows"].as_array().unwrap();
    for row in rows {
        for field in &["code", "lines", "files", "bytes", "tokens", "avg_lines"] {
            let v = row[*field].as_u64();
            assert!(v.is_some(), "row.{field} must be a non-negative integer");
        }
    }
}

#[test]
fn w72_export_row_numeric_fields_non_negative() {
    let json = run_json(&["export", "--format", "json"]);
    let rows = json["rows"].as_array().unwrap();
    for row in rows {
        for field in &["code", "comments", "blanks", "lines", "bytes", "tokens"] {
            let v = row[*field].as_u64();
            assert!(v.is_some(), "row.{field} must be a non-negative integer");
        }
    }
}
