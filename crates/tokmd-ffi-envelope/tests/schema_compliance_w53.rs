//! Schema compliance tests for FFI envelope types.
//!
//! These tests verify that the FFI response envelope correctly handles
//! success/error responses and data extraction.

use serde_json::{Value, json};
use tokmd_ffi_envelope::{
    EnvelopeExtractError, extract_data, extract_data_from_json, extract_data_json,
    format_error_message, parse_envelope,
};

// ---------------------------------------------------------------------------
// 1. Success response has ok=true, data={...}
// ---------------------------------------------------------------------------

#[test]
fn success_response_has_ok_true_and_data() {
    let envelope = json!({
        "ok": true,
        "data": { "schema_version": 2, "mode": "lang" }
    });
    let data = extract_data(envelope).unwrap();
    assert_eq!(data["schema_version"], 2);
    assert_eq!(data["mode"], "lang");
}

#[test]
fn success_response_without_data_returns_envelope() {
    let envelope = json!({
        "ok": true,
        "schema_version": 2
    });
    let data = extract_data(envelope.clone()).unwrap();
    assert_eq!(data, envelope);
}

// ---------------------------------------------------------------------------
// 2. Error response has ok=false, error={message: "..."}
// ---------------------------------------------------------------------------

#[test]
fn error_response_has_ok_false_and_error() {
    let envelope = json!({
        "ok": false,
        "error": { "code": "scan_failed", "message": "Path not found" }
    });
    let err = extract_data(envelope).unwrap_err();
    match err {
        EnvelopeExtractError::Upstream(msg) => {
            assert!(msg.contains("scan_failed"));
            assert!(msg.contains("Path not found"));
        }
        _ => panic!("Expected Upstream error"),
    }
}

#[test]
fn error_response_with_missing_error_object() {
    let envelope = json!({ "ok": false });
    let err = extract_data(envelope).unwrap_err();
    match err {
        EnvelopeExtractError::Upstream(msg) => {
            assert!(msg.contains("Unknown error"));
        }
        _ => panic!("Expected Upstream error"),
    }
}

// ---------------------------------------------------------------------------
// 3. Data payload matches inner receipt structure
// ---------------------------------------------------------------------------

#[test]
fn data_payload_preserves_inner_receipt() {
    let inner = json!({
        "schema_version": 2,
        "generated_at_ms": 1_700_000_000_000_u64,
        "tool": { "name": "tokmd", "version": "1.0.0" },
        "mode": "lang",
        "rows": [{ "lang": "Rust", "code": 100 }],
        "total": { "code": 100, "lines": 150, "files": 5, "bytes": 5000, "tokens": 1000 }
    });
    let envelope = json!({ "ok": true, "data": inner.clone() });

    let data = extract_data(envelope).unwrap();
    assert_eq!(data, inner);
    assert_eq!(data["rows"][0]["lang"], "Rust");
}

// ---------------------------------------------------------------------------
// 4. Version response format
// ---------------------------------------------------------------------------

#[test]
fn version_response_format() {
    let envelope = json!({
        "ok": true,
        "data": { "version": "1.5.0", "schema_version": 2 }
    });
    let data = extract_data(envelope).unwrap();
    assert!(data["version"].is_string());
    assert!(data["schema_version"].is_number());
}

// ---------------------------------------------------------------------------
// 5. parse_envelope and extract_data_from_json
// ---------------------------------------------------------------------------

#[test]
fn parse_envelope_valid_json() {
    let val = parse_envelope(r#"{"ok": true, "data": 42}"#).unwrap();
    assert_eq!(val["ok"], true);
    assert_eq!(val["data"], 42);
}

#[test]
fn parse_envelope_invalid_json() {
    let err = parse_envelope("{invalid}").unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
}

#[test]
fn extract_data_from_json_success() {
    let data = extract_data_from_json(r#"{"ok": true, "data": {"v": 1}}"#).unwrap();
    assert_eq!(data["v"], 1);
}

#[test]
fn extract_data_from_json_error() {
    let err = extract_data_from_json(r#"{"ok": false, "error": {"code": "e", "message": "fail"}}"#)
        .unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
}

// ---------------------------------------------------------------------------
// 6. extract_data_json returns valid JSON string
// ---------------------------------------------------------------------------

#[test]
fn extract_data_json_returns_json_string() {
    let json_str = extract_data_json(r#"{"ok": true, "data": {"count": 5}}"#).unwrap();
    let parsed: Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed["count"], 5);
}

// ---------------------------------------------------------------------------
// 7. format_error_message
// ---------------------------------------------------------------------------

#[test]
fn format_error_message_with_code_and_message() {
    let err = json!({"code": "invalid_mode", "message": "Unknown mode"});
    assert_eq!(
        format_error_message(Some(&err)),
        "[invalid_mode] Unknown mode"
    );
}

#[test]
fn format_error_message_none_returns_default() {
    assert_eq!(format_error_message(None), "Unknown error");
}

// ---------------------------------------------------------------------------
// 8. Non-object envelope
// ---------------------------------------------------------------------------

#[test]
fn non_object_envelope_returns_invalid_format() {
    let err = extract_data(json!(42)).unwrap_err();
    assert_eq!(err, EnvelopeExtractError::InvalidResponseFormat);
}

#[test]
fn array_envelope_returns_invalid_format() {
    let err = extract_data(json!([1, 2, 3])).unwrap_err();
    assert_eq!(err, EnvelopeExtractError::InvalidResponseFormat);
}
