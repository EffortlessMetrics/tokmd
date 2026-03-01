//! Integration tests for the JSON API (FFI entrypoint).
//!
//! The FFI layer uses a consistent response envelope:
//! - Success: `{"ok": true, "data": {...receipt...}}`
//! - Error: `{"ok": false, "error": {"code": "...", "message": "..."}}`

use tokmd_core::ffi::{run_json, schema_version, version};

// ============================================================================
// Helper
// ============================================================================

/// Parse a run_json result and assert it is valid JSON with the expected `ok` value.
fn parse_envelope(result: &str) -> serde_json::Value {
    let parsed: serde_json::Value =
        serde_json::from_str(result).expect("run_json must always return valid JSON");
    assert!(
        parsed.get("ok").is_some(),
        "envelope must have 'ok' field: {result}"
    );
    parsed
}

fn assert_ok(result: &str) -> serde_json::Value {
    let parsed = parse_envelope(result);
    assert_eq!(parsed["ok"], true, "expected ok:true, got: {result}");
    parsed
}

fn assert_err(result: &str) -> serde_json::Value {
    let parsed = parse_envelope(result);
    assert_eq!(parsed["ok"], false, "expected ok:false, got: {result}");
    assert!(
        parsed.get("error").is_some(),
        "error envelope must have 'error' field"
    );
    parsed
}

// ============================================================================
// Version mode
// ============================================================================

#[test]
fn run_json_version_mode() {
    let result = run_json("version", "{}");
    let parsed = assert_ok(&result);

    let data = parsed.get("data").expect("should have data field");
    assert!(data.get("version").is_some());
    assert!(data.get("schema_version").is_some());
}

#[test]
fn version_data_contains_semver() {
    let result = run_json("version", "{}");
    let parsed = assert_ok(&result);

    let ver = parsed["data"]["version"]
        .as_str()
        .expect("version should be a string");
    let parts: Vec<&str> = ver.split('.').collect();
    assert!(parts.len() >= 2, "version should be semver-like: {ver}");
}

#[test]
fn version_schema_version_is_positive() {
    let result = run_json("version", "{}");
    let parsed = assert_ok(&result);

    let sv = parsed["data"]["schema_version"]
        .as_u64()
        .expect("schema_version should be a number");
    assert!(sv > 0, "schema_version should be > 0");
}

#[test]
fn version_ignores_extra_args() {
    // Extra keys in args should be silently ignored
    let result = run_json("version", r#"{"extra_key": 42, "another": true}"#);
    assert_ok(&result);
}

// ============================================================================
// Lang mode
// ============================================================================

#[test]
fn run_json_lang_mode() {
    let result = run_json("lang", r#"{"paths": ["src"]}"#);
    let parsed = assert_ok(&result);

    let data = &parsed["data"];
    assert_eq!(data["mode"].as_str(), Some("lang"));
    assert!(data.get("schema_version").is_some());
    assert!(data.get("rows").is_some());
}

#[test]
fn lang_mode_receipt_has_tool_info() {
    let result = run_json("lang", r#"{"paths": ["src"]}"#);
    let parsed = assert_ok(&result);

    let tool = &parsed["data"]["tool"];
    assert!(tool.get("name").is_some(), "receipt should have tool.name");
    assert!(
        tool.get("version").is_some(),
        "receipt should have tool.version"
    );
}

#[test]
fn lang_mode_generated_at_ms_is_recent() {
    let result = run_json("lang", r#"{"paths": ["src"]}"#);
    let parsed = assert_ok(&result);

    let ts = parsed["data"]["generated_at_ms"]
        .as_u64()
        .expect("generated_at_ms should be a number");
    // Should be a reasonable Unix timestamp in ms (after 2020-01-01)
    assert!(ts > 1_577_836_800_000, "timestamp looks too small: {ts}");
}

#[test]
fn lang_mode_finds_rust() {
    let result = run_json("lang", r#"{"paths": ["src"]}"#);
    let parsed = assert_ok(&result);

    let rows = parsed["data"]["rows"]
        .as_array()
        .expect("rows should be an array");
    let has_rust = rows.iter().any(|r| r["lang"].as_str() == Some("Rust"));
    assert!(has_rust, "should find Rust in this crate's src/");
}

#[test]
fn lang_mode_with_top_limits_rows() {
    let result = run_json("lang", r#"{"paths": ["src"], "top": 1}"#);
    let parsed = assert_ok(&result);

    let rows = parsed["data"]["rows"]
        .as_array()
        .expect("rows should be an array");
    // top=1 means at most 1 real row + optional "Other"
    assert!(
        rows.len() <= 2,
        "top=1 should yield at most 2 rows, got {}",
        rows.len()
    );
}

#[test]
fn lang_mode_with_files_flag() {
    let result = run_json("lang", r#"{"paths": ["src"], "files": true}"#);
    let parsed = assert_ok(&result);

    let args = &parsed["data"]["args"];
    assert_eq!(args["with_files"].as_bool(), Some(true));
}

#[test]
fn lang_mode_default_paths_uses_dot() {
    // Omitting "paths" should default to ["."]
    let result = run_json("lang", "{}");
    let parsed = assert_ok(&result);

    let scan = &parsed["data"]["scan"];
    // scan.paths should contain "."
    let paths = scan["paths"]
        .as_array()
        .expect("scan.paths should be an array");
    assert!(paths.iter().any(|p| p.as_str() == Some(".")));
}

#[test]
fn lang_mode_with_children_collapse() {
    let result = run_json("lang", r#"{"paths": ["src"], "children": "collapse"}"#);
    let parsed = assert_ok(&result);

    let args = &parsed["data"]["args"];
    assert_eq!(args["children"].as_str(), Some("collapse"));
}

#[test]
fn lang_mode_with_children_separate() {
    let result = run_json("lang", r#"{"paths": ["src"], "children": "separate"}"#);
    let parsed = assert_ok(&result);

    let args = &parsed["data"]["args"];
    assert_eq!(args["children"].as_str(), Some("separate"));
}

#[test]
fn lang_mode_status_is_complete() {
    let result = run_json("lang", r#"{"paths": ["src"]}"#);
    let parsed = assert_ok(&result);

    // status field should indicate completion
    let status = parsed["data"]["status"].as_str().unwrap_or("");
    assert!(
        status.to_lowercase().contains("complete"),
        "status should be complete, got: {status}"
    );
}

// ============================================================================
// Module mode
// ============================================================================

#[test]
fn run_json_module_mode() {
    let result = run_json("module", r#"{"paths": ["src"]}"#);
    let parsed = assert_ok(&result);

    assert_eq!(parsed["data"]["mode"].as_str(), Some("module"));
}

#[test]
fn module_mode_with_custom_depth() {
    let result = run_json("module", r#"{"paths": ["src"], "module_depth": 1}"#);
    let parsed = assert_ok(&result);

    let args = &parsed["data"]["args"];
    assert_eq!(args["module_depth"].as_u64(), Some(1));
}

#[test]
fn module_mode_with_custom_roots() {
    let result = run_json(
        "module",
        r#"{"paths": ["src"], "module_roots": ["src", "tests"]}"#,
    );
    let parsed = assert_ok(&result);

    let args = &parsed["data"]["args"];
    let roots = args["module_roots"]
        .as_array()
        .expect("module_roots should be an array");
    assert!(roots.iter().any(|r| r.as_str() == Some("src")));
    assert!(roots.iter().any(|r| r.as_str() == Some("tests")));
}

#[test]
fn module_mode_with_top_setting() {
    let result = run_json("module", r#"{"paths": ["src"], "top": 1}"#);
    let parsed = assert_ok(&result);

    let args = &parsed["data"]["args"];
    assert_eq!(args["top"].as_u64(), Some(1));
}

// ============================================================================
// Export mode
// ============================================================================

#[test]
fn export_mode_success() {
    let result = run_json("export", r#"{"paths": ["src"]}"#);
    let parsed = assert_ok(&result);

    assert_eq!(parsed["data"]["mode"].as_str(), Some("export"));
    // ExportData is flattened, so rows is at data.rows
    assert!(
        parsed["data"]["rows"].is_array(),
        "export should have data.rows"
    );
}

#[test]
fn export_mode_has_file_rows() {
    let result = run_json("export", r#"{"paths": ["src"]}"#);
    let parsed = assert_ok(&result);

    let rows = parsed["data"]["rows"]
        .as_array()
        .expect("rows should be an array");
    assert!(!rows.is_empty(), "export should find files");

    // Each row should have path and lang
    let first = &rows[0];
    assert!(first.get("path").is_some(), "row should have path");
    assert!(first.get("lang").is_some(), "row should have lang");
}

#[test]
fn export_mode_with_min_code_filter() {
    let result_all = run_json("export", r#"{"paths": ["src"], "min_code": 0}"#);
    let parsed_all = assert_ok(&result_all);
    let rows_all = parsed_all["data"]["rows"]
        .as_array()
        .expect("rows should be an array")
        .len();

    let result_filtered = run_json("export", r#"{"paths": ["src"], "min_code": 9999}"#);
    let parsed_filtered = assert_ok(&result_filtered);
    let rows_filtered = parsed_filtered["data"]["rows"]
        .as_array()
        .expect("rows should be an array")
        .len();

    assert!(
        rows_filtered <= rows_all,
        "min_code filter should reduce or equal row count"
    );
}

#[test]
fn export_mode_with_max_rows_limit() {
    let result = run_json("export", r#"{"paths": ["src"], "max_rows": 1}"#);
    let parsed = assert_ok(&result);

    let rows = parsed["data"]["rows"]
        .as_array()
        .expect("rows should be an array");
    assert!(
        rows.len() <= 1,
        "max_rows=1 should limit to at most 1 row, got {}",
        rows.len()
    );
}

#[test]
fn export_mode_with_redact_paths() {
    let result = run_json("export", r#"{"paths": ["src"], "redact": "paths"}"#);
    let parsed = assert_ok(&result);

    let args = &parsed["data"]["args"];
    assert_eq!(args["redact"].as_str(), Some("paths"));
}

#[test]
fn export_mode_with_format_json() {
    let result = run_json("export", r#"{"paths": ["src"], "format": "json"}"#);
    let parsed = assert_ok(&result);

    let args = &parsed["data"]["args"];
    assert_eq!(args["format"].as_str(), Some("json"));
}

#[test]
fn export_mode_with_format_csv() {
    let result = run_json("export", r#"{"paths": ["src"], "format": "csv"}"#);
    let parsed = assert_ok(&result);

    let args = &parsed["data"]["args"];
    assert_eq!(args["format"].as_str(), Some("csv"));
}

#[test]
fn export_mode_with_format_jsonl() {
    let result = run_json("export", r#"{"paths": ["src"], "format": "jsonl"}"#);
    assert_ok(&result);
}

// ============================================================================
// Diff mode
// ============================================================================

#[test]
fn diff_mode_missing_from_returns_error() {
    let result = run_json("diff", r#"{"to": "."}"#);
    let parsed = assert_err(&result);

    assert!(
        parsed["error"]["message"]
            .as_str()
            .expect("error message should be a string")
            .contains("from")
    );
}

#[test]
fn diff_mode_missing_to_returns_error() {
    let result = run_json("diff", r#"{"from": "."}"#);
    let parsed = assert_err(&result);

    assert!(
        parsed["error"]["message"]
            .as_str()
            .expect("error message should be a string")
            .contains("to")
    );
}

#[test]
fn diff_mode_both_missing_returns_error() {
    let result = run_json("diff", "{}");
    let parsed = assert_err(&result);

    // Should fail on the first missing field ("from")
    assert!(
        parsed["error"]["message"]
            .as_str()
            .expect("error message should be a string")
            .contains("from")
    );
}

#[test]
fn diff_mode_non_string_from_returns_error() {
    let result = run_json("diff", r#"{"from": 123, "to": "."}"#);
    let parsed = assert_err(&result);

    assert!(
        parsed["error"]["message"]
            .as_str()
            .expect("error message should be a string")
            .contains("from")
    );
}

#[test]
fn diff_mode_non_string_to_returns_error() {
    let result = run_json("diff", r#"{"from": ".", "to": false}"#);
    let parsed = assert_err(&result);

    assert!(
        parsed["error"]["message"]
            .as_str()
            .expect("error message should be a string")
            .contains("to")
    );
}

#[test]
fn diff_mode_with_directory_paths() {
    // Diff between two scans of the same directory should succeed
    let result = run_json("diff", r#"{"from": "src", "to": "src"}"#);
    let parsed = assert_ok(&result);

    // A self-diff should have zero or near-zero deltas
    assert!(parsed["data"].is_object());
}

// ============================================================================
// Analyze mode (feature-gated)
// ============================================================================

#[test]
#[cfg(not(feature = "analysis"))]
fn analyze_without_feature_returns_not_implemented() {
    let result = run_json("analyze", "{}");
    let parsed = assert_err(&result);

    assert_eq!(parsed["error"]["code"].as_str(), Some("not_implemented"));
}

#[test]
#[cfg(not(feature = "cockpit"))]
fn cockpit_without_feature_returns_not_implemented() {
    let result = run_json("cockpit", "{}");
    let parsed = assert_err(&result);

    assert_eq!(parsed["error"]["code"].as_str(), Some("not_implemented"));
}

// ============================================================================
// Unknown / error modes
// ============================================================================

#[test]
fn run_json_unknown_mode_returns_error() {
    let result = run_json("unknown_mode", "{}");
    let parsed = assert_err(&result);

    assert_eq!(parsed["error"]["code"].as_str(), Some("unknown_mode"));
    assert!(
        parsed["error"]["message"]
            .as_str()
            .expect("error message should be a string")
            .contains("unknown_mode")
    );
}

#[test]
fn run_json_empty_mode_returns_error() {
    let result = run_json("", "{}");
    let parsed = assert_err(&result);

    assert_eq!(parsed["error"]["code"].as_str(), Some("unknown_mode"));
}

#[test]
fn run_json_mode_is_case_sensitive() {
    // "Lang" should not match "lang"
    let result = run_json("Lang", "{}");
    let parsed = assert_err(&result);
    assert_eq!(parsed["error"]["code"].as_str(), Some("unknown_mode"));
}

#[test]
fn run_json_mode_with_whitespace() {
    let result = run_json(" lang ", "{}");
    let parsed = assert_err(&result);
    assert_eq!(parsed["error"]["code"].as_str(), Some("unknown_mode"));
}

// ============================================================================
// Invalid JSON input
// ============================================================================

#[test]
fn run_json_invalid_json_returns_error() {
    let result = run_json("lang", "not valid json");
    let parsed = assert_err(&result);

    assert_eq!(parsed["error"]["code"].as_str(), Some("invalid_json"));
}

#[test]
fn run_json_empty_string_args_returns_error() {
    let result = run_json("lang", "");
    let parsed = assert_err(&result);

    assert_eq!(parsed["error"]["code"].as_str(), Some("invalid_json"));
}

#[test]
fn run_json_array_as_args_returns_error_or_processes() {
    // A JSON array is not a valid args object
    let result = run_json("lang", "[]");
    // Should either error or treat as empty object — either way, valid envelope
    let _parsed = parse_envelope(&result);
}

#[test]
fn run_json_number_as_args_returns_error_or_processes() {
    let result = run_json("lang", "123");
    let _parsed = parse_envelope(&result);
}

#[test]
fn run_json_null_as_args_returns_error_or_processes() {
    let result = run_json("lang", "null");
    let _parsed = parse_envelope(&result);
}

// ============================================================================
// Invalid settings (type errors)
// ============================================================================

#[test]
fn run_json_invalid_children_mode_returns_error() {
    let result = run_json("lang", r#"{"children": "invalid"}"#);
    let parsed = assert_err(&result);

    assert_eq!(parsed["error"]["code"].as_str(), Some("invalid_settings"));
    assert!(
        parsed["error"]["message"]
            .as_str()
            .expect("error message should be a string")
            .contains("children")
    );
}

#[test]
fn run_json_invalid_format_returns_error() {
    let result = run_json("export", r#"{"format": "yaml"}"#);
    let parsed = assert_err(&result);

    assert_eq!(parsed["error"]["code"].as_str(), Some("invalid_settings"));
    assert!(
        parsed["error"]["message"]
            .as_str()
            .expect("error message should be a string")
            .contains("format")
    );
}

#[test]
fn run_json_top_as_string_returns_error() {
    let result = run_json("lang", r#"{"top": "ten"}"#);
    let parsed = assert_err(&result);

    assert_eq!(parsed["error"]["code"].as_str(), Some("invalid_settings"));
    assert!(
        parsed["error"]["message"]
            .as_str()
            .expect("error message should be a string")
            .contains("top")
    );
}

#[test]
fn run_json_top_as_negative_returns_error() {
    let result = run_json("lang", r#"{"top": -1}"#);
    let parsed = assert_err(&result);

    assert_eq!(parsed["error"]["code"].as_str(), Some("invalid_settings"));
}

#[test]
fn run_json_hidden_as_string_returns_error() {
    let result = run_json("lang", r#"{"hidden": "yes"}"#);
    let parsed = assert_err(&result);

    assert_eq!(parsed["error"]["code"].as_str(), Some("invalid_settings"));
    assert!(
        parsed["error"]["message"]
            .as_str()
            .expect("error message should be a string")
            .contains("hidden")
    );
}

#[test]
fn run_json_paths_as_string_returns_error() {
    let result = run_json("lang", r#"{"paths": "src"}"#);
    let parsed = assert_err(&result);

    assert_eq!(parsed["error"]["code"].as_str(), Some("invalid_settings"));
    assert!(
        parsed["error"]["message"]
            .as_str()
            .expect("error message should be a string")
            .contains("paths")
    );
}

#[test]
fn run_json_paths_with_non_string_element_returns_error() {
    let result = run_json("lang", r#"{"paths": ["src", 42]}"#);
    let parsed = assert_err(&result);

    assert_eq!(parsed["error"]["code"].as_str(), Some("invalid_settings"));
    // Should include index in message
    assert!(
        parsed["error"]["message"]
            .as_str()
            .expect("error message should be a string")
            .contains("paths[1]")
    );
}

#[test]
fn run_json_invalid_redact_returns_error() {
    let result = run_json("export", r#"{"redact": "maybe"}"#);
    let parsed = assert_err(&result);

    assert_eq!(parsed["error"]["code"].as_str(), Some("invalid_settings"));
    assert!(
        parsed["error"]["message"]
            .as_str()
            .expect("error message should be a string")
            .contains("redact")
    );
}

#[test]
fn run_json_invalid_config_mode_returns_error() {
    let result = run_json("lang", r#"{"config": "invalid"}"#);
    let parsed = assert_err(&result);

    assert_eq!(parsed["error"]["code"].as_str(), Some("invalid_settings"));
    assert!(
        parsed["error"]["message"]
            .as_str()
            .expect("error message should be a string")
            .contains("config")
    );
}

// ============================================================================
// Nested object parsing
// ============================================================================

#[test]
fn nested_scan_object_works() {
    let result = run_json("lang", r#"{"scan": {"paths": ["src"], "hidden": true}}"#);
    assert_ok(&result);
}

#[test]
fn nested_lang_object_works() {
    let result = run_json(
        "lang",
        r#"{"paths": ["src"], "lang": {"top": 3, "files": true}}"#,
    );
    let parsed = assert_ok(&result);

    let args = &parsed["data"]["args"];
    assert_eq!(args["top"].as_u64(), Some(3));
    assert_eq!(args["with_files"].as_bool(), Some(true));
}

#[test]
fn nested_scan_invalid_returns_error() {
    let result = run_json("lang", r#"{"scan": {"hidden": "yes"}}"#);
    let parsed = assert_err(&result);

    assert_eq!(parsed["error"]["code"].as_str(), Some("invalid_settings"));
}

// ============================================================================
// Null value handling
// ============================================================================

#[test]
fn null_values_use_defaults() {
    let result = run_json("lang", r#"{"paths": null, "top": null, "files": null}"#);
    let parsed = assert_ok(&result);

    let args = &parsed["data"]["args"];
    assert_eq!(args["top"].as_u64(), Some(0));
    assert_eq!(args["with_files"].as_bool(), Some(false));
}

// ============================================================================
// Version / schema version helpers
// ============================================================================

#[test]
fn version_returns_cargo_version() {
    let v = version();
    assert!(!v.is_empty());
    assert!(v.contains('.'), "version should contain dots");
}

#[test]
fn schema_version_matches_types() {
    let sv = schema_version();
    assert_eq!(sv, tokmd_types::SCHEMA_VERSION);
}

#[test]
fn run_json_with_settings() {
    let result = run_json(
        "lang",
        r#"{
            "paths": ["src"],
            "top": 5,
            "files": true
        }"#,
    );
    let parsed = assert_ok(&result);

    let args = &parsed["data"]["args"];
    assert_eq!(args["top"].as_u64(), Some(5));
    assert_eq!(args["with_files"].as_bool(), Some(true));
}

// ============================================================================
// Envelope totality: run_json ALWAYS returns valid JSON with `ok` field
// ============================================================================

#[test]
fn run_json_always_returns_valid_json_for_known_edge_cases() {
    let cases: Vec<(&str, &str)> = vec![
        ("", ""),
        ("lang", ""),
        ("lang", "null"),
        ("lang", "[]"),
        ("lang", "123"),
        ("lang", r#"{"paths": null}"#),
        ("lang", r#"{"top": -1}"#),
        ("\0", "{}"),
        ("lang", r#"{"paths": [1, 2, 3]}"#),
        ("export", r#"{"format": "invalid"}"#),
        ("unknown_mode", "{}"),
        ("version", "{}"),
        ("version", ""),
        ("diff", "{}"),
        ("module", r#"{"module_depth": -5}"#),
        ("export", r#"{"max_rows": "all"}"#),
    ];

    for (mode, args) in &cases {
        let result = run_json(mode, args);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap_or_else(|e| {
            panic!("Invalid JSON for mode={mode:?} args={args:?}: {e}\nraw: {result}")
        });
        assert!(
            parsed.get("ok").is_some(),
            "Missing 'ok' for mode={mode:?} args={args:?}"
        );
    }
}

// ============================================================================
// Envelope shape invariants
// ============================================================================

#[test]
fn success_envelope_has_data_no_error() {
    let result = run_json("version", "{}");
    let parsed = assert_ok(&result);

    assert!(parsed.get("data").is_some());
    // In the envelope, error should be absent (not just null)
    assert!(
        parsed.get("error").is_none(),
        "success envelope should not have error key"
    );
}

#[test]
fn error_envelope_has_error_no_data() {
    let result = run_json("unknown", "{}");
    let parsed = assert_err(&result);

    // data should be absent in error envelope
    assert!(
        parsed.get("data").is_none(),
        "error envelope should not have data key"
    );
    let err = &parsed["error"];
    assert!(err["code"].is_string(), "error.code should be string");
    assert!(err["message"].is_string(), "error.message should be string");
}

// ============================================================================
// Property tests: run_json envelope contract
// ============================================================================

mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn run_json_always_returns_valid_json_for_any_mode(
            mode in "\\PC{0,30}"
        ) {
            let result = run_json(&mode, "{}");
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(&result);
            prop_assert!(parsed.is_ok(), "Invalid JSON for mode={:?}: {}", mode, result);
            let val = parsed.expect("should parse as valid JSON");
            prop_assert!(val.get("ok").is_some(), "Missing 'ok' for mode={:?}", mode);
        }

        #[test]
        fn run_json_always_returns_valid_json_for_any_args(
            args in "\\PC{0,200}"
        ) {
            let result = run_json("lang", &args);
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(&result);
            prop_assert!(parsed.is_ok(), "Invalid JSON for args={:?}: {}", args, result);
            let val = parsed.expect("should parse as valid JSON");
            prop_assert!(val.get("ok").is_some(), "Missing 'ok' for args={:?}", args);
        }

        #[test]
        fn unknown_modes_always_produce_error_envelope(
            mode in "[a-z]{1,20}"
        ) {
            // Skip known modes
            if matches!(mode.as_str(), "lang" | "module" | "export" | "analyze" | "diff" | "version" | "cockpit") {
                return Ok(());
            }
            let result = run_json(&mode, "{}");
            let parsed: serde_json::Value = serde_json::from_str(&result).expect("should parse as valid JSON");
            prop_assert_eq!(&parsed["ok"], &serde_json::Value::Bool(false));
            prop_assert_eq!(parsed["error"]["code"].as_str(), Some("unknown_mode"));
        }

        #[test]
        fn version_mode_ignores_arbitrary_json_args(
            args in r#"\{"[a-z]+": (true|false|null|42|"x")\}"#
        ) {
            let result = run_json("version", &args);
            let parsed: serde_json::Value = serde_json::from_str(&result).expect("should parse as valid JSON");
            prop_assert_eq!(&parsed["ok"], &serde_json::Value::Bool(true));
        }

        #[test]
        fn invalid_json_always_produces_invalid_json_error(
            garbage in "[^{}\\[\\]\"0-9ntf]{1,50}"
        ) {
            let result = run_json("lang", &garbage);
            let parsed: serde_json::Value = serde_json::from_str(&result).expect("should parse as valid JSON");
            prop_assert_eq!(&parsed["ok"], &serde_json::Value::Bool(false));
            prop_assert_eq!(parsed["error"]["code"].as_str(), Some("invalid_json"));
        }
    }
}
