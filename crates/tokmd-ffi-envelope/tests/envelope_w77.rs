//! W77 deep tests for tokmd-ffi-envelope: envelope construction, error
//! formatting, serialization, and extraction edge cases.

use serde_json::{Value, json};
use tokmd_ffi_envelope::{
    EnvelopeExtractError, extract_data, extract_data_from_json, extract_data_json,
    format_error_message, parse_envelope,
};

// ============================================================================
// Envelope construction / parsing
// ============================================================================

#[test]
fn parse_envelope_success_roundtrip() {
    let input = json!({"ok": true, "data": {"v": 1}});
    let parsed = parse_envelope(&input.to_string()).unwrap();
    assert_eq!(parsed, input);
}

#[test]
fn parse_envelope_error_roundtrip() {
    let input = json!({"ok": false, "error": {"code": "e", "message": "m"}});
    let parsed = parse_envelope(&input.to_string()).unwrap();
    assert_eq!(parsed, input);
}

#[test]
fn parse_envelope_rejects_invalid_json() {
    let err = parse_envelope("{{{").unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
    assert!(err.to_string().contains("JSON parse error"));
}

// ============================================================================
// Error envelope construction
// ============================================================================

#[test]
fn error_envelope_extract_returns_upstream_error() {
    let envelope = json!({"ok": false, "error": {"code": "scan_failed", "message": "No files"}});
    let err = extract_data(envelope).unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
    assert!(err.to_string().contains("scan_failed"));
    assert!(err.to_string().contains("No files"));
}

#[test]
fn error_envelope_missing_ok_treated_as_false() {
    // `ok` absent → defaults to false via unwrap_or(false)
    let envelope = json!({"error": {"code": "x", "message": "y"}});
    let err = extract_data(envelope).unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
}

#[test]
fn format_error_message_with_full_error() {
    let err_obj = json!({"code": "bad_input", "message": "path missing"});
    assert_eq!(
        format_error_message(Some(&err_obj)),
        "[bad_input] path missing"
    );
}

#[test]
fn format_error_message_with_none() {
    assert_eq!(format_error_message(None), "Unknown error");
}

#[test]
fn format_error_message_with_non_object() {
    assert_eq!(
        format_error_message(Some(&json!("string_err"))),
        "Unknown error"
    );
}

// ============================================================================
// JSON serialization of envelopes
// ============================================================================

#[test]
fn extract_data_json_serializes_data_to_json_string() {
    let input = json!({"ok": true, "data": {"count": 42}}).to_string();
    let json_str = extract_data_json(&input).unwrap();
    let v: Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(v["count"], 42);
}

#[test]
fn extract_data_from_json_returns_data_value() {
    let input = json!({"ok": true, "data": [1, 2, 3]}).to_string();
    let data = extract_data_from_json(&input).unwrap();
    assert!(data.is_array());
    assert_eq!(data.as_array().unwrap().len(), 3);
}

#[test]
fn extract_data_non_object_envelope_is_invalid() {
    let err = extract_data(json!("just a string")).unwrap_err();
    assert_eq!(err, EnvelopeExtractError::InvalidResponseFormat);
}
