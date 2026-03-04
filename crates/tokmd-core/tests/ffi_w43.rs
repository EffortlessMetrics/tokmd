//! Wave 43: Deep FFI entrypoint tests for `run_json`.
//!
//! Focuses on:
//! - Response envelope contract (ok/data/error fields)
//! - All modes: version, lang, module, export, diff
//! - Error handling: invalid JSON, unknown mode, bad settings
//! - Schema version propagation
//! - JSON validity invariant (every call returns parseable JSON)
//! - Idempotency of error envelopes
//! - Nested settings parsing through FFI boundary
//! - Mode-specific receipt field validation
//! - Edge-case inputs: empty strings, unicode, null fields

use serde_json::Value;
use tokmd_core::ffi::run_json;

// ═══════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════

fn parse(result: &str) -> Value {
    serde_json::from_str(result)
        .unwrap_or_else(|e| panic!("run_json must return valid JSON: {e}\nraw: {result}"))
}

fn ok(result: &str) -> Value {
    let v = parse(result);
    assert_eq!(v["ok"], true, "expected ok=true: {result}");
    assert!(v.get("data").is_some(), "ok envelope must have 'data'");
    assert!(
        v.get("error").is_none(),
        "ok envelope must not have 'error'"
    );
    v
}

fn err(result: &str) -> Value {
    let v = parse(result);
    assert_eq!(v["ok"], false, "expected ok=false: {result}");
    assert!(v.get("error").is_some(), "error envelope must have 'error'");
    assert!(
        v.get("data").is_none(),
        "error envelope must not have 'data'"
    );
    v
}

// ═══════════════════════════════════════════════════════════════════
// 1. Version mode
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w43_version_returns_ok() {
    ok(&run_json("version", "{}"));
}

#[test]
fn w43_version_has_version_string() {
    let v = ok(&run_json("version", "{}"));
    let ver = v["data"]["version"].as_str().expect("version is string");
    assert!(!ver.is_empty());
    assert!(ver.contains('.'), "should be semver: {ver}");
}

#[test]
fn w43_version_has_schema_version() {
    let v = ok(&run_json("version", "{}"));
    let sv = v["data"]["schema_version"]
        .as_u64()
        .expect("schema_version");
    assert_eq!(sv, u64::from(tokmd_types::SCHEMA_VERSION));
}

#[test]
fn w43_version_ignores_extra_fields() {
    let v = ok(&run_json("version", r#"{"extra": "stuff", "foo": 42}"#));
    assert!(v["data"]["version"].is_string());
}

#[test]
fn w43_version_idempotent() {
    let r1 = run_json("version", "{}");
    let r2 = run_json("version", "{}");
    assert_eq!(r1, r2, "version should be byte-identical across calls");
}

// ═══════════════════════════════════════════════════════════════════
// 2. Lang mode
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w43_lang_default_args_returns_ok() {
    ok(&run_json("lang", "{}"));
}

#[test]
fn w43_lang_receipt_has_mode_field() {
    let v = ok(&run_json("lang", "{}"));
    assert_eq!(v["data"]["mode"].as_str(), Some("lang"));
}

#[test]
fn w43_lang_receipt_has_schema_version() {
    let v = ok(&run_json("lang", "{}"));
    let sv = v["data"]["schema_version"].as_u64().unwrap();
    assert_eq!(sv, u64::from(tokmd_types::SCHEMA_VERSION));
}

#[test]
fn w43_lang_receipt_has_generated_at_ms() {
    let v = ok(&run_json("lang", "{}"));
    let ts = v["data"]["generated_at_ms"].as_u64().unwrap();
    // Should be after 2020-01-01
    assert!(ts > 1_577_836_800_000);
}

#[test]
fn w43_lang_receipt_has_tool_info() {
    let v = ok(&run_json("lang", "{}"));
    assert!(v["data"]["tool"]["name"].is_string());
    assert!(v["data"]["tool"]["version"].is_string());
}

#[test]
fn w43_lang_receipt_has_rows_array() {
    let v = ok(&run_json("lang", "{}"));
    assert!(v["data"]["rows"].is_array());
}

#[test]
fn w43_lang_receipt_has_scan_metadata() {
    let v = ok(&run_json("lang", "{}"));
    assert!(v["data"]["scan"].is_object());
}

#[test]
fn w43_lang_receipt_has_args_metadata() {
    let v = ok(&run_json("lang", "{}"));
    assert!(v["data"]["args"].is_object());
}

#[test]
fn w43_lang_with_top_parameter() {
    let v = ok(&run_json("lang", r#"{"top": 1}"#));
    let rows = v["data"]["rows"].as_array().unwrap();
    // top=1 means at most 1 language row + optional "Other"
    assert!(rows.len() <= 2);
}

#[test]
fn w43_lang_with_files_parameter() {
    let v = ok(&run_json("lang", r#"{"files": true}"#));
    assert_eq!(v["data"]["args"]["with_files"], true);
}

// ═══════════════════════════════════════════════════════════════════
// 3. Module mode
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w43_module_default_args_returns_ok() {
    ok(&run_json("module", "{}"));
}

#[test]
fn w43_module_receipt_has_mode() {
    let v = ok(&run_json("module", "{}"));
    assert_eq!(v["data"]["mode"].as_str(), Some("module"));
}

#[test]
fn w43_module_receipt_has_schema_version() {
    let v = ok(&run_json("module", "{}"));
    let sv = v["data"]["schema_version"].as_u64().unwrap();
    assert_eq!(sv, u64::from(tokmd_types::SCHEMA_VERSION));
}

#[test]
fn w43_module_receipt_has_rows() {
    let v = ok(&run_json("module", "{}"));
    assert!(v["data"]["rows"].is_array());
}

// ═══════════════════════════════════════════════════════════════════
// 4. Export mode
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w43_export_default_args_returns_ok() {
    ok(&run_json("export", "{}"));
}

#[test]
fn w43_export_receipt_has_mode() {
    let v = ok(&run_json("export", "{}"));
    assert_eq!(v["data"]["mode"].as_str(), Some("export"));
}

#[test]
fn w43_export_receipt_has_schema_version() {
    let v = ok(&run_json("export", "{}"));
    let sv = v["data"]["schema_version"].as_u64().unwrap();
    assert_eq!(sv, u64::from(tokmd_types::SCHEMA_VERSION));
}

#[test]
fn w43_export_receipt_has_rows() {
    let v = ok(&run_json("export", "{}"));
    assert!(v["data"]["rows"].is_array());
    let rows = v["data"]["rows"].as_array().unwrap();
    assert!(!rows.is_empty(), "should find files in cwd");
}

#[test]
fn w43_export_rows_have_path_and_lang() {
    let v = ok(&run_json("export", "{}"));
    let rows = v["data"]["rows"].as_array().unwrap();
    for row in rows.iter().take(5) {
        assert!(row["path"].is_string(), "row must have path: {row}");
        assert!(row["lang"].is_string(), "row must have lang: {row}");
    }
}

// ═══════════════════════════════════════════════════════════════════
// 5. Invalid mode
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w43_invalid_mode_returns_error_envelope() {
    let v = err(&run_json("invalid_mode", "{}"));
    assert_eq!(v["error"]["code"].as_str(), Some("unknown_mode"));
}

#[test]
fn w43_invalid_mode_message_includes_mode_name() {
    let v = err(&run_json("foobar", "{}"));
    let msg = v["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("foobar"),
        "error should mention the mode: {msg}"
    );
}

#[test]
fn w43_empty_mode_is_unknown() {
    let v = err(&run_json("", "{}"));
    assert_eq!(v["error"]["code"].as_str(), Some("unknown_mode"));
}

#[test]
fn w43_case_sensitive_mode() {
    // "Lang" is not "lang"
    let v = err(&run_json("Lang", "{}"));
    assert_eq!(v["error"]["code"].as_str(), Some("unknown_mode"));
}

#[test]
fn w43_mode_with_spaces() {
    let v = err(&run_json(" lang ", "{}"));
    assert_eq!(v["error"]["code"].as_str(), Some("unknown_mode"));
}

// ═══════════════════════════════════════════════════════════════════
// 6. Invalid JSON input
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w43_invalid_json_returns_error_envelope() {
    let v = err(&run_json("lang", "invalid json"));
    assert_eq!(v["error"]["code"].as_str(), Some("invalid_json"));
}

#[test]
fn w43_empty_string_input_returns_error() {
    let v = err(&run_json("lang", ""));
    assert_eq!(v["error"]["code"].as_str(), Some("invalid_json"));
}

#[test]
fn w43_truncated_json_returns_error() {
    let v = err(&run_json("lang", r#"{"paths": ["sr"#));
    assert_eq!(v["error"]["code"].as_str(), Some("invalid_json"));
}

#[test]
fn w43_json_array_is_parseable_but_may_succeed_or_fail() {
    // A JSON array parses fine but field extraction uses defaults
    let result = run_json("lang", "[]");
    let v = parse(&result);
    assert!(v["ok"].is_boolean());
}

// ═══════════════════════════════════════════════════════════════════
// 7. Envelope contract invariants
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w43_every_response_has_ok_boolean() {
    let cases = [
        ("version", "{}"),
        ("lang", "{}"),
        ("module", "{}"),
        ("export", "{}"),
        ("bogus", "{}"),
        ("lang", "bad"),
        ("diff", "{}"),
    ];
    for (mode, args) in &cases {
        let result = run_json(mode, args);
        let v = parse(&result);
        assert!(
            v["ok"].is_boolean(),
            "ok must be bool for mode={mode} args={args}"
        );
    }
}

#[test]
fn w43_success_envelope_has_data_not_error() {
    let v = ok(&run_json("version", "{}"));
    assert!(v.get("data").is_some());
    assert!(v.get("error").is_none());
}

#[test]
fn w43_error_envelope_has_error_not_data() {
    let v = err(&run_json("bogus", "{}"));
    assert!(v.get("error").is_some());
    assert!(v.get("data").is_none());
}

#[test]
fn w43_error_envelope_has_code_and_message() {
    let v = err(&run_json("bogus", "{}"));
    let e = &v["error"];
    assert!(e["code"].is_string(), "error.code must be string");
    assert!(e["message"].is_string(), "error.message must be string");
}

#[test]
fn w43_all_responses_are_valid_json() {
    let cases = [
        ("", ""),
        ("lang", ""),
        ("lang", "null"),
        ("lang", "42"),
        ("\0", "{}"),
        ("lang", r#"{"top": -1}"#),
        ("export", r#"{"format": "yaml"}"#),
        ("module", r#"{"module_depth": "deep"}"#),
        ("diff", r#"{"from":"a"}"#),
    ];
    for (mode, args) in &cases {
        let result = run_json(mode, args);
        let parsed: Result<Value, _> = serde_json::from_str(&result);
        assert!(
            parsed.is_ok(),
            "Invalid JSON for mode={mode:?} args={args:?}: {result}"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════
// 8. Strict settings validation through FFI
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w43_lang_invalid_top_type_returns_error() {
    let v = err(&run_json("lang", r#"{"top": "ten"}"#));
    assert_eq!(v["error"]["code"].as_str(), Some("invalid_settings"));
    assert!(v["error"]["message"].as_str().unwrap().contains("top"));
}

#[test]
fn w43_lang_invalid_files_type_returns_error() {
    let v = err(&run_json("lang", r#"{"files": "yes"}"#));
    assert_eq!(v["error"]["code"].as_str(), Some("invalid_settings"));
    assert!(v["error"]["message"].as_str().unwrap().contains("files"));
}

#[test]
fn w43_lang_invalid_children_returns_error() {
    let v = err(&run_json("lang", r#"{"children": "merge"}"#));
    assert_eq!(v["error"]["code"].as_str(), Some("invalid_settings"));
    assert!(v["error"]["message"].as_str().unwrap().contains("children"));
}

#[test]
fn w43_export_invalid_format_returns_error() {
    let v = err(&run_json("export", r#"{"format": "xml"}"#));
    assert_eq!(v["error"]["code"].as_str(), Some("invalid_settings"));
    assert!(v["error"]["message"].as_str().unwrap().contains("format"));
}

#[test]
fn w43_export_invalid_redact_returns_error() {
    let v = err(&run_json("export", r#"{"redact": "partial"}"#));
    assert_eq!(v["error"]["code"].as_str(), Some("invalid_settings"));
    assert!(v["error"]["message"].as_str().unwrap().contains("redact"));
}

#[test]
fn w43_scan_invalid_hidden_type_returns_error() {
    let v = err(&run_json("lang", r#"{"hidden": 1}"#));
    assert_eq!(v["error"]["code"].as_str(), Some("invalid_settings"));
}

#[test]
fn w43_scan_invalid_paths_type_returns_error() {
    let v = err(&run_json("lang", r#"{"paths": "not-array"}"#));
    assert_eq!(v["error"]["code"].as_str(), Some("invalid_settings"));
}

#[test]
fn w43_scan_invalid_config_returns_error() {
    let v = err(&run_json("lang", r#"{"config": "invalid"}"#));
    assert_eq!(v["error"]["code"].as_str(), Some("invalid_settings"));
}

// ═══════════════════════════════════════════════════════════════════
// 9. Nested scan object settings through FFI
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w43_nested_scan_object_works() {
    let v = ok(&run_json("lang", r#"{"scan": {"paths": ["."]}}"#));
    assert_eq!(v["data"]["mode"].as_str(), Some("lang"));
}

#[test]
fn w43_nested_scan_invalid_field_returns_error() {
    let v = err(&run_json("lang", r#"{"scan": {"hidden": "nope"}}"#));
    assert_eq!(v["error"]["code"].as_str(), Some("invalid_settings"));
}

// ═══════════════════════════════════════════════════════════════════
// 10. Diff mode errors
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w43_diff_missing_from_returns_error() {
    let v = err(&run_json("diff", r#"{"to": "."}"#));
    assert!(v["error"]["message"].as_str().unwrap().contains("from"));
}

#[test]
fn w43_diff_missing_to_returns_error() {
    let v = err(&run_json("diff", r#"{"from": "."}"#));
    assert!(v["error"]["message"].as_str().unwrap().contains("to"));
}

#[test]
fn w43_diff_missing_both_returns_error() {
    err(&run_json("diff", "{}"));
}

#[test]
fn w43_diff_self_returns_zero_deltas() {
    let v = ok(&run_json("diff", r#"{"from": ".", "to": "."}"#));
    if let Some(rows) = v["data"]["diff_rows"].as_array() {
        for row in rows {
            assert_eq!(row["delta_code"].as_i64(), Some(0));
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// 11. Unicode and special characters
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w43_unicode_in_mode_returns_error() {
    let v = err(&run_json("日本語", "{}"));
    assert_eq!(v["error"]["code"].as_str(), Some("unknown_mode"));
}

#[test]
fn w43_null_args_json_produces_valid_json() {
    let result = run_json("lang", "null");
    let v = parse(&result);
    assert!(v["ok"].is_boolean());
}

// ═══════════════════════════════════════════════════════════════════
// 12. Feature-gated modes
// ═══════════════════════════════════════════════════════════════════

#[test]
#[cfg(not(feature = "analysis"))]
fn w43_analyze_without_feature_returns_not_implemented() {
    let v = err(&run_json("analyze", "{}"));
    assert_eq!(v["error"]["code"].as_str(), Some("not_implemented"));
}

#[test]
#[cfg(not(feature = "cockpit"))]
fn w43_cockpit_without_feature_returns_not_implemented() {
    let v = err(&run_json("cockpit", "{}"));
    assert_eq!(v["error"]["code"].as_str(), Some("not_implemented"));
}

// ═══════════════════════════════════════════════════════════════════
// 13. Error code consistency
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w43_error_codes_are_snake_case() {
    let cases = [
        ("bogus", "{}"),
        ("lang", "bad json"),
        ("lang", r#"{"top": "x"}"#),
        ("export", r#"{"format": "yaml"}"#),
    ];
    for (mode, args) in &cases {
        let v = err(&run_json(mode, args));
        let code = v["error"]["code"].as_str().unwrap();
        assert!(
            code.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
            "error code must be snake_case: {code} for mode={mode}"
        );
    }
}

#[test]
fn w43_error_messages_are_non_empty() {
    let cases = [("bogus", "{}"), ("lang", ""), ("diff", "{}")];
    for (mode, args) in &cases {
        let v = err(&run_json(mode, args));
        let msg = v["error"]["message"].as_str().unwrap();
        assert!(
            !msg.is_empty(),
            "error message must not be empty for mode={mode}"
        );
    }
}
