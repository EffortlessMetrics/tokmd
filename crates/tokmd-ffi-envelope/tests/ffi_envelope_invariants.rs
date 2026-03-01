//! Deeper invariant tests for the FFI response envelope.

use serde_json::{Value, json};
use tokmd_ffi_envelope::{
    EnvelopeExtractError, extract_data, extract_data_from_json, extract_data_json,
    format_error_message, parse_envelope,
};

// ---------------------------------------------------------------------------
// Success envelope construction and serialization
// ---------------------------------------------------------------------------

#[test]
fn success_envelope_extracts_data_field() {
    let envelope = json!({"ok": true, "data": {"lang": "Rust", "lines": 42}});
    let data = extract_data(envelope).unwrap();
    assert_eq!(data["lang"], "Rust");
    assert_eq!(data["lines"], 42);
}

#[test]
fn success_envelope_without_data_returns_full_envelope() {
    let envelope = json!({"ok": true, "version": "1.0.0"});
    let data = extract_data(envelope.clone()).unwrap();
    assert_eq!(data, envelope);
}

#[test]
fn success_envelope_from_json_string() {
    let json_str = r#"{"ok": true, "data": {"count": 7}}"#;
    let data = extract_data_from_json(json_str).unwrap();
    assert_eq!(data["count"], 7);
}

#[test]
fn success_envelope_data_json_returns_serialized_string() {
    let json_str = r#"{"ok": true, "data": {"a": 1}}"#;
    let result = extract_data_json(json_str).unwrap();
    let parsed: Value = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed["a"], 1);
}

#[test]
fn success_envelope_with_nested_data() {
    let envelope = json!({
        "ok": true,
        "data": {
            "receipt": {
                "languages": [{"name": "Rust", "code": 1000}],
                "totals": {"code": 1000, "comments": 200}
            }
        }
    });
    let data = extract_data(envelope).unwrap();
    assert!(data["receipt"]["languages"].is_array());
    assert_eq!(data["receipt"]["totals"]["code"], 1000);
}

// ---------------------------------------------------------------------------
// Error envelope construction and serialization
// ---------------------------------------------------------------------------

#[test]
fn error_envelope_returns_upstream_error() {
    let envelope = json!({
        "ok": false,
        "error": {"code": "scan_error", "message": "No files found"}
    });
    let err = extract_data(envelope).unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
    assert_eq!(err.to_string(), "[scan_error] No files found");
}

#[test]
fn error_envelope_with_missing_error_object() {
    let envelope = json!({"ok": false});
    let err = extract_data(envelope).unwrap_err();
    assert_eq!(err.to_string(), "Unknown error");
}

#[test]
fn error_envelope_with_partial_error_object() {
    let envelope = json!({"ok": false, "error": {"code": "parse_error"}});
    let err = extract_data(envelope).unwrap_err();
    assert_eq!(err.to_string(), "[parse_error] Unknown error");
}

#[test]
fn error_envelope_with_only_message() {
    let envelope = json!({"ok": false, "error": {"message": "Something broke"}});
    let err = extract_data(envelope).unwrap_err();
    assert_eq!(err.to_string(), "[unknown] Something broke");
}

// ---------------------------------------------------------------------------
// Envelope always contains "ok" field logic
// ---------------------------------------------------------------------------

#[test]
fn missing_ok_field_treated_as_false() {
    let envelope = json!({"data": {"something": true}});
    let err = extract_data(envelope).unwrap_err();
    // Missing "ok" defaults to false → Upstream error
    assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
}

#[test]
fn ok_field_non_bool_treated_as_false() {
    let envelope = json!({"ok": "yes", "data": {"something": true}});
    let err = extract_data(envelope).unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
}

#[test]
fn ok_true_always_succeeds_even_with_error_present() {
    // If ok is true, data extraction succeeds regardless of error field
    let envelope = json!({"ok": true, "data": {"val": 1}, "error": {"code": "x", "message": "y"}});
    let data = extract_data(envelope).unwrap();
    assert_eq!(data["val"], 1);
}

// ---------------------------------------------------------------------------
// Data vs error mutual exclusivity
// ---------------------------------------------------------------------------

#[test]
fn ok_true_returns_data_not_error() {
    let envelope = json!({"ok": true, "data": {"result": "success"}});
    let result = extract_data(envelope);
    assert!(result.is_ok());
    assert_eq!(result.unwrap()["result"], "success");
}

#[test]
fn ok_false_returns_error_not_data() {
    let envelope = json!({
        "ok": false,
        "data": {"result": "should be ignored"},
        "error": {"code": "fail", "message": "failed"}
    });
    let result = extract_data(envelope);
    assert!(result.is_err());
}

#[test]
fn non_object_envelope_is_invalid_format() {
    for val in [json!([1, 2, 3]), json!("string"), json!(42), json!(null)] {
        let err = extract_data(val).unwrap_err();
        assert_eq!(err, EnvelopeExtractError::InvalidResponseFormat);
    }
}

// ---------------------------------------------------------------------------
// JSON roundtrip
// ---------------------------------------------------------------------------

#[test]
fn roundtrip_parse_extract_from_json() {
    let original = json!({"ok": true, "data": {"languages": ["Rust", "Python"], "total": 42}});
    let json_str = serde_json::to_string(&original).unwrap();
    let data = extract_data_from_json(&json_str).unwrap();
    assert_eq!(data["languages"][0], "Rust");
    assert_eq!(data["total"], 42);
}

#[test]
fn roundtrip_extract_data_json_preserves_values() {
    let original = json!({"ok": true, "data": {"nested": {"deep": true}, "count": 99}});
    let json_str = serde_json::to_string(&original).unwrap();
    let extracted = extract_data_json(&json_str).unwrap();
    let parsed: Value = serde_json::from_str(&extracted).unwrap();
    assert_eq!(parsed["nested"]["deep"], true);
    assert_eq!(parsed["count"], 99);
}

#[test]
fn parse_envelope_valid_json() {
    let json_str = r#"{"ok": true, "data": {}}"#;
    let value = parse_envelope(json_str).unwrap();
    assert!(value.is_object());
}

#[test]
fn parse_envelope_invalid_json() {
    let err = parse_envelope("not json").unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
}

// ---------------------------------------------------------------------------
// format_error_message edge cases
// ---------------------------------------------------------------------------

#[test]
fn format_error_message_none_returns_unknown() {
    assert_eq!(format_error_message(None), "Unknown error");
}

#[test]
fn format_error_message_non_object_returns_unknown() {
    assert_eq!(format_error_message(Some(&json!("string"))), "Unknown error");
    assert_eq!(format_error_message(Some(&json!(42))), "Unknown error");
    assert_eq!(format_error_message(Some(&json!([1]))), "Unknown error");
}

#[test]
fn format_error_message_complete_object() {
    let err = json!({"code": "io_error", "message": "File not found"});
    assert_eq!(
        format_error_message(Some(&err)),
        "[io_error] File not found"
    );
}

// ---------------------------------------------------------------------------
// Error variant equality and display
// ---------------------------------------------------------------------------

#[test]
fn error_variants_display_correctly() {
    let parse_err = EnvelopeExtractError::JsonParse("unexpected EOF".to_string());
    assert!(parse_err.to_string().contains("unexpected EOF"));

    let ser_err = EnvelopeExtractError::JsonSerialize("cycle detected".to_string());
    assert!(ser_err.to_string().contains("cycle detected"));

    let fmt_err = EnvelopeExtractError::InvalidResponseFormat;
    assert_eq!(fmt_err.to_string(), "Invalid response format");

    let upstream = EnvelopeExtractError::Upstream("[x] msg".to_string());
    assert_eq!(upstream.to_string(), "[x] msg");
}

#[test]
fn error_variants_eq() {
    assert_eq!(
        EnvelopeExtractError::InvalidResponseFormat,
        EnvelopeExtractError::InvalidResponseFormat
    );
    assert_ne!(
        EnvelopeExtractError::JsonParse("a".to_string()),
        EnvelopeExtractError::JsonParse("b".to_string()),
    );
}
