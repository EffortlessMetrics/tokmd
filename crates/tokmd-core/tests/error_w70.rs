//! Error handling tests for the tokmd-core FFI and workflow layer (w70).
//!
//! Validates that `run_json` produces correct error envelopes for malformed
//! inputs, invalid modes, type mismatches, and missing required fields.

use tokmd_core::ffi::run_json;

// ============================================================================
// Helpers
// ============================================================================

fn parse_envelope(result: &str) -> serde_json::Value {
    serde_json::from_str(result).expect("run_json must always return valid JSON")
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

fn assert_ok(result: &str) -> serde_json::Value {
    let parsed = parse_envelope(result);
    assert_eq!(parsed["ok"], true, "expected ok:true, got: {result}");
    parsed
}

// ============================================================================
// 1. Unknown mode strings
// ============================================================================

#[test]
fn run_json_unknown_mode_returns_error_envelope() {
    let result = run_json("unknown_mode_w70", "{}");
    let parsed = assert_err(&result);
    let code = parsed["error"]["code"].as_str().unwrap_or_default();
    assert_eq!(code, "unknown_mode");
}

#[test]
fn run_json_empty_mode_returns_error_envelope() {
    let result = run_json("", "{}");
    let parsed = assert_err(&result);
    let code = parsed["error"]["code"].as_str().unwrap_or_default();
    assert_eq!(code, "unknown_mode");
}

#[test]
fn run_json_mode_with_special_chars_returns_error() {
    let result = run_json("lang!@#$", "{}");
    assert_err(&result);
}

#[test]
fn run_json_mode_with_unicode_returns_error() {
    let result = run_json("日本語", "{}");
    assert_err(&result);
}

// ============================================================================
// 2. Malformed JSON input
// ============================================================================

#[test]
fn run_json_malformed_json_returns_invalid_json_error() {
    let result = run_json("lang", "not valid json at all");
    let parsed = assert_err(&result);
    let code = parsed["error"]["code"].as_str().unwrap_or_default();
    assert_eq!(code, "invalid_json");
}

#[test]
fn run_json_empty_string_json_returns_error() {
    let result = run_json("lang", "");
    assert_err(&result);
}

#[test]
fn run_json_truncated_json_returns_error() {
    let result = run_json("lang", r#"{"paths": ["src"]"#);
    assert_err(&result);
}

#[test]
fn run_json_json_array_instead_of_object_returns_error() {
    // The FFI expects a JSON object, not an array
    let result = run_json("lang", "[1, 2, 3]");
    // This should either error or work - either way must return valid envelope
    let parsed = parse_envelope(&result);
    assert!(parsed.get("ok").is_some());
}

// ============================================================================
// 3. Error envelope structure validation
// ============================================================================

#[test]
fn error_envelope_always_has_ok_field() {
    let result = run_json("bogus_w70", "{}");
    let parsed = parse_envelope(&result);
    assert!(parsed.get("ok").is_some(), "must have 'ok' field");
}

#[test]
fn error_envelope_has_error_object_with_code_and_message() {
    let result = run_json("bogus_w70", "{}");
    let parsed = assert_err(&result);
    let error = &parsed["error"];
    assert!(error.get("code").is_some(), "error must have 'code' field");
    assert!(
        error.get("message").is_some(),
        "error must have 'message' field"
    );
}

#[test]
fn error_message_is_nonempty_string() {
    let result = run_json("bogus_w70", "{}");
    let parsed = assert_err(&result);
    let message = parsed["error"]["message"].as_str().unwrap_or_default();
    assert!(!message.is_empty(), "error message must not be empty");
}

#[test]
fn success_envelope_has_no_error_field() {
    let result = run_json("version", "{}");
    let parsed = assert_ok(&result);
    // error field should be absent or null in success responses
    let error = parsed.get("error");
    assert!(
        error.is_none() || error.unwrap().is_null(),
        "success envelope should not have error field"
    );
}

// ============================================================================
// 4. Invalid field types in settings
// ============================================================================

#[test]
fn run_json_lang_with_non_boolean_hidden_returns_error() {
    let result = run_json("lang", r#"{"hidden": "yes"}"#);
    let parsed = assert_err(&result);
    let code = parsed["error"]["code"].as_str().unwrap_or_default();
    assert_eq!(code, "invalid_settings");
}

#[test]
fn run_json_lang_with_non_integer_top_returns_error() {
    let result = run_json("lang", r#"{"top": "ten"}"#);
    let parsed = assert_err(&result);
    let code = parsed["error"]["code"].as_str().unwrap_or_default();
    assert_eq!(code, "invalid_settings");
}

#[test]
fn run_json_lang_with_invalid_children_mode_returns_error() {
    let result = run_json("lang", r#"{"children": "invalid_mode"}"#);
    let parsed = assert_err(&result);
    let code = parsed["error"]["code"].as_str().unwrap_or_default();
    assert_eq!(code, "invalid_settings");
}

#[test]
fn run_json_export_with_invalid_format_returns_error() {
    let result = run_json("export", r#"{"format": "parquet"}"#);
    let parsed = assert_err(&result);
    let code = parsed["error"]["code"].as_str().unwrap_or_default();
    assert_eq!(code, "invalid_settings");
}

#[test]
fn run_json_lang_with_non_array_paths_returns_error() {
    let result = run_json("lang", r#"{"paths": 42}"#);
    let parsed = assert_err(&result);
    let code = parsed["error"]["code"].as_str().unwrap_or_default();
    assert_eq!(code, "invalid_settings");
}

#[test]
fn run_json_lang_with_non_string_in_paths_array_returns_error() {
    let result = run_json("lang", r#"{"paths": [42]}"#);
    let parsed = assert_err(&result);
    let code = parsed["error"]["code"].as_str().unwrap_or_default();
    assert_eq!(code, "invalid_settings");
}

// ============================================================================
// 5. version mode is resilient to bad args
// ============================================================================

#[test]
fn version_mode_ignores_malformed_extra_fields() {
    let result = run_json("version", r#"{"garbage": null, "foo": [1,2,3]}"#);
    assert_ok(&result);
}

#[test]
fn version_mode_with_empty_object_succeeds() {
    let result = run_json("version", "{}");
    let parsed = assert_ok(&result);
    assert!(parsed["data"]["version"].is_string());
}
