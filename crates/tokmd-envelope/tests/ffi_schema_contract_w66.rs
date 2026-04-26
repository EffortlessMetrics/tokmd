//! Schema contract tests for `tokmd-envelope::ffi` (FFI response envelope).
//!
//! These tests verify that the FFI JSON envelope structure `{"ok": bool, "data": ..., "error": ...}`
//! is correct, stable, and backwards-compatible.

use serde_json::{Value, json};
use tokmd_envelope::ffi::*;

// ===========================================================================
// 1. Success envelope structure: {"ok": true, "data": ...}
// ===========================================================================

#[test]
fn success_envelope_has_ok_true() {
    let envelope = json!({"ok": true, "data": {"mode": "lang"}});
    let val = parse_envelope(&envelope.to_string()).unwrap();
    assert_eq!(val["ok"], true);
}

#[test]
fn success_envelope_data_extraction() {
    let envelope = json!({"ok": true, "data": {"schema_version": 2, "mode": "lang"}});
    let data = extract_data(envelope).unwrap();
    assert_eq!(data["schema_version"], 2);
    assert_eq!(data["mode"], "lang");
}

#[test]
fn success_envelope_without_data_returns_full_envelope() {
    let envelope = json!({"ok": true, "schema_version": 2});
    let data = extract_data(envelope.clone()).unwrap();
    assert_eq!(data, envelope);
}

// ===========================================================================
// 2. Error envelope structure: {"ok": false, "error": {"code": ..., "message": ...}}
// ===========================================================================

#[test]
fn error_envelope_has_ok_false() {
    let envelope = json!({
        "ok": false,
        "error": {"code": "scan_failed", "message": "Path not found"}
    });
    let err = extract_data(envelope).unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
}

#[test]
fn error_envelope_message_format() {
    let envelope = json!({
        "ok": false,
        "error": {"code": "unknown_mode", "message": "Unknown mode: nope"}
    });
    let err = extract_data(envelope).unwrap_err();
    assert_eq!(err.to_string(), "[unknown_mode] Unknown mode: nope");
}

#[test]
fn error_envelope_missing_code_defaults() {
    let err = json!({"message": "Something broke"});
    let msg = format_error_message(Some(&err));
    assert_eq!(msg, "[unknown] Something broke");
}

#[test]
fn error_envelope_missing_message_defaults() {
    let err = json!({"code": "my_code"});
    let msg = format_error_message(Some(&err));
    assert_eq!(msg, "[my_code] Unknown error");
}

#[test]
fn error_envelope_null_error_defaults() {
    let msg = format_error_message(None);
    assert_eq!(msg, "Unknown error");
}

// ===========================================================================
// 3. Invalid envelope handling
// ===========================================================================

#[test]
fn non_object_envelope_is_invalid_format() {
    let err = extract_data(json!(["not", "an", "envelope"])).unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::InvalidResponseFormat));
}

#[test]
fn non_json_input_returns_parse_error() {
    let err = parse_envelope("{invalid json").unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
}

#[test]
fn missing_ok_field_treated_as_false() {
    let envelope = json!({"data": {"mode": "lang"}});
    let err = extract_data(envelope).unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
}

// ===========================================================================
// 4. JSON output stability
// ===========================================================================

#[test]
fn extract_data_json_returns_valid_json() {
    let input = json!({"ok": true, "data": {"v": 1, "mode": "export"}});
    let json_str = extract_data_json(&input.to_string()).unwrap();
    let parsed: Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed["v"], 1);
    assert_eq!(parsed["mode"], "export");
}

#[test]
fn extract_data_from_json_convenience() {
    let input = r#"{"ok": true, "data": {"schema_version": 2}}"#;
    let data = extract_data_from_json(input).unwrap();
    assert_eq!(data["schema_version"], 2);
}

#[test]
fn error_envelope_error_variants_are_eq() {
    let err1 = EnvelopeExtractError::Upstream("boom".into());
    let err2 = EnvelopeExtractError::Upstream("boom".into());
    assert_eq!(err1, err2);

    let err3 = EnvelopeExtractError::InvalidResponseFormat;
    let err4 = EnvelopeExtractError::InvalidResponseFormat;
    assert_eq!(err3, err4);
}

#[test]
fn envelope_preserves_nested_data_types() {
    let envelope = json!({
        "ok": true,
        "data": {
            "schema_version": 2,
            "generated_at_ms": 1700000000000_u64,
            "rows": [{"lang": "Rust", "code": 100}],
            "nested": {"deep": true}
        }
    });
    let data = extract_data(envelope).unwrap();
    assert_eq!(data["schema_version"], 2);
    assert!(data["rows"].is_array());
    assert_eq!(data["nested"]["deep"], true);
}
