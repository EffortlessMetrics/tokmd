//! Error handling tests for tokmd-core FFI layer.
//!
//! Tests invalid mode strings, malformed JSON args, non-existent paths,
//! and edge cases in the run_json envelope contract.

use serde_json::Value;
use tokmd_core::error::{ErrorCode, ResponseEnvelope, TokmdError};
use tokmd_core::ffi::run_json;

// =============================================================================
// Helper: parse run_json output and assert error envelope
// =============================================================================

fn assert_error_envelope(mode: &str, args: &str, expected_code: &str) -> Value {
    let result = run_json(mode, args);
    let parsed: Value = serde_json::from_str(&result).expect("run_json must return valid JSON");
    assert_eq!(
        parsed["ok"], false,
        "Expected error for mode={mode} args={args}"
    );
    assert_eq!(
        parsed["error"]["code"], expected_code,
        "Expected code={expected_code} for mode={mode}, got: {}",
        parsed["error"]["code"]
    );
    parsed
}

fn assert_success_envelope(mode: &str, args: &str) -> Value {
    let result = run_json(mode, args);
    let parsed: Value = serde_json::from_str(&result).expect("run_json must return valid JSON");
    assert_eq!(
        parsed["ok"], true,
        "Expected success for mode={mode} args={args}, got: {result}"
    );
    parsed
}

// =============================================================================
// Invalid mode strings
// =============================================================================

#[test]
fn ffi_empty_mode_returns_unknown_mode() {
    assert_error_envelope("", "{}", "unknown_mode");
}

#[test]
fn ffi_whitespace_mode_returns_unknown_mode() {
    assert_error_envelope("  ", "{}", "unknown_mode");
}

#[test]
fn ffi_case_sensitive_mode() {
    // "Lang" != "lang" - should be unknown
    assert_error_envelope("Lang", "{}", "unknown_mode");
}

#[test]
fn ffi_null_byte_mode_returns_unknown_mode() {
    assert_error_envelope("\0", "{}", "unknown_mode");
}

#[test]
fn ffi_numeric_mode_returns_unknown_mode() {
    assert_error_envelope("123", "{}", "unknown_mode");
}

// =============================================================================
// Malformed JSON args
// =============================================================================

#[test]
fn ffi_empty_args_string_returns_invalid_json() {
    assert_error_envelope("lang", "", "invalid_json");
}

#[test]
fn ffi_plain_text_args_returns_invalid_json() {
    assert_error_envelope("lang", "not valid json at all", "invalid_json");
}

#[test]
fn ffi_truncated_json_returns_invalid_json() {
    assert_error_envelope("lang", r#"{"paths": ["."#, "invalid_json");
}

#[test]
fn ffi_json_array_instead_of_object() {
    // JSON array is valid JSON but the parser expects an object for field extraction
    // The behavior depends on implementation - it should still return valid envelope
    let result = run_json("lang", "[]");
    let parsed: Value = serde_json::from_str(&result).expect("must return valid JSON");
    assert!(parsed.get("ok").is_some());
}

#[test]
fn ffi_json_null_arg() {
    let result = run_json("lang", "null");
    let parsed: Value = serde_json::from_str(&result).expect("must return valid JSON");
    assert!(parsed.get("ok").is_some());
}

#[test]
fn ffi_json_number_arg() {
    let result = run_json("lang", "123");
    let parsed: Value = serde_json::from_str(&result).expect("must return valid JSON");
    assert!(parsed.get("ok").is_some());
}

// =============================================================================
// Non-existent paths
// =============================================================================

#[test]
fn ffi_nonexistent_path_returns_error() {
    let result = run_json(
        "lang",
        r#"{"paths": ["/tmp/tokmd-errors-nonexistent-dir-xyz"]}"#,
    );
    let parsed: Value = serde_json::from_str(&result).expect("must return valid JSON");
    assert_eq!(parsed["ok"], false);
    // Should contain some indication of the path problem
    let msg = parsed["error"]["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("not found") || msg.contains("Path") || msg.contains("error"),
        "Error message should indicate path problem: {msg}"
    );
}

#[test]
fn ffi_nonexistent_path_in_module_mode() {
    let result = run_json(
        "module",
        r#"{"paths": ["/tmp/tokmd-errors-nonexistent-dir-xyz"]}"#,
    );
    let parsed: Value = serde_json::from_str(&result).expect("must return valid JSON");
    assert_eq!(parsed["ok"], false);
}

#[test]
fn ffi_nonexistent_path_in_export_mode() {
    let result = run_json(
        "export",
        r#"{"paths": ["/tmp/tokmd-errors-nonexistent-dir-xyz"]}"#,
    );
    let parsed: Value = serde_json::from_str(&result).expect("must return valid JSON");
    assert_eq!(parsed["ok"], false);
}

// =============================================================================
// Invalid field types in JSON args
// =============================================================================

#[test]
fn ffi_invalid_top_type_returns_error() {
    assert_error_envelope("lang", r#"{"top": "ten"}"#, "invalid_settings");
}

#[test]
fn ffi_invalid_hidden_type_returns_error() {
    assert_error_envelope("lang", r#"{"hidden": "yes"}"#, "invalid_settings");
}

#[test]
fn ffi_invalid_paths_type_returns_error() {
    assert_error_envelope("lang", r#"{"paths": "not-an-array"}"#, "invalid_settings");
}

#[test]
fn ffi_invalid_paths_element_type_returns_error() {
    assert_error_envelope("lang", r#"{"paths": [123]}"#, "invalid_settings");
}

#[test]
fn ffi_invalid_children_mode_returns_error() {
    assert_error_envelope("lang", r#"{"children": "invalid"}"#, "invalid_settings");
}

#[test]
fn ffi_invalid_redact_mode_returns_error() {
    assert_error_envelope("export", r#"{"redact": "invalid"}"#, "invalid_settings");
}

#[test]
fn ffi_invalid_export_format_returns_error() {
    assert_error_envelope("export", r#"{"format": "yaml"}"#, "invalid_settings");
}

#[test]
fn ffi_diff_missing_from_returns_error() {
    assert_error_envelope("diff", r#"{"to": "b.json"}"#, "invalid_settings");
}

#[test]
fn ffi_diff_missing_to_returns_error() {
    assert_error_envelope("diff", r#"{"from": "a.json"}"#, "invalid_settings");
}

#[test]
fn ffi_diff_wrong_type_from_returns_error() {
    assert_error_envelope(
        "diff",
        r#"{"from": 123, "to": "b.json"}"#,
        "invalid_settings",
    );
}

// =============================================================================
// Envelope contract invariants
// =============================================================================

#[test]
fn ffi_all_responses_are_valid_json_with_ok_field() {
    let cases = vec![
        ("", ""),
        ("", "{}"),
        ("lang", ""),
        ("lang", "null"),
        ("lang", "[]"),
        ("lang", "123"),
        ("lang", r#"{"paths": null}"#),
        ("module", r#"{"top": -1}"#),
        ("export", r#"{"format": "invalid"}"#),
        ("unknown", "{}"),
        ("\0\0\0", r#"{"a":1}"#),
    ];

    for (mode, args) in cases {
        let result = run_json(mode, args);
        let parsed: Result<Value, _> = serde_json::from_str(&result);
        assert!(
            parsed.is_ok(),
            "Invalid JSON for mode={mode:?} args={args:?}: {result}"
        );
        let parsed = parsed.unwrap();
        assert!(
            parsed.get("ok").is_some(),
            "Missing 'ok' field for mode={mode:?} args={args:?}"
        );
    }
}

#[test]
fn ffi_success_has_data_no_error() {
    let parsed = assert_success_envelope("version", "{}");
    assert!(parsed["data"].is_object());
    assert!(parsed.get("error").is_none() || parsed["error"].is_null());
}

#[test]
fn ffi_error_has_error_no_data() {
    let parsed = assert_error_envelope("unknown", "{}", "unknown_mode");
    assert!(parsed.get("data").is_none() || parsed["data"].is_null());
    assert!(parsed["error"].is_object());
    assert!(parsed["error"]["message"].is_string());
}

// =============================================================================
// TokmdError type construction tests
// =============================================================================

#[test]
fn tokmd_error_path_not_found_code() {
    let err = TokmdError::path_not_found("/some/path");
    assert_eq!(err.code, ErrorCode::PathNotFound);
    assert!(err.message.contains("/some/path"));
}

#[test]
fn tokmd_error_unknown_mode_code() {
    let err = TokmdError::unknown_mode("bogus");
    assert_eq!(err.code, ErrorCode::UnknownMode);
    assert!(err.message.contains("bogus"));
}

#[test]
fn tokmd_error_invalid_json_code() {
    let err = TokmdError::invalid_json("unexpected token");
    assert_eq!(err.code, ErrorCode::InvalidJson);
    assert!(err.message.contains("unexpected token"));
}

#[test]
fn tokmd_error_to_json_produces_valid_json() {
    let err = TokmdError::new(ErrorCode::ScanError, "test error");
    let json = err.to_json();
    let parsed: Value = serde_json::from_str(&json).expect("to_json must produce valid JSON");
    assert_eq!(parsed["code"], "scan_error");
    assert_eq!(parsed["message"], "test error");
}

#[test]
fn tokmd_error_display_with_details() {
    let err = TokmdError::with_details(
        ErrorCode::ConfigInvalid,
        "bad config",
        "missing required field",
    );
    let display = err.to_string();
    assert!(display.contains("config_invalid"));
    assert!(display.contains("bad config"));
    assert!(display.contains("missing required field"));
}

#[test]
fn response_envelope_error_to_json() {
    let err = TokmdError::unknown_mode("test");
    let envelope = ResponseEnvelope::error(&err);
    let json = envelope.to_json();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "unknown_mode");
}

#[test]
fn response_envelope_success_to_json() {
    let data = serde_json::json!({"version": "1.0.0"});
    let envelope = ResponseEnvelope::success(data);
    let json = envelope.to_json();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["data"]["version"], "1.0.0");
}

// =============================================================================
// From trait conversions for TokmdError
// =============================================================================

#[test]
fn tokmd_error_from_serde_json_error() {
    let serde_err = serde_json::from_str::<Value>("invalid").unwrap_err();
    let err: TokmdError = serde_err.into();
    assert_eq!(err.code, ErrorCode::InvalidJson);
}

#[test]
fn tokmd_error_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err: TokmdError = io_err.into();
    assert_eq!(err.code, ErrorCode::IoError);
}
