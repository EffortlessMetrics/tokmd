//! Deep coverage tests for `tokmd-envelope::ffi`.
//!
//! Exercises envelope creation with various payloads, serialization validity,
//! error envelope format, version field consistency, and edge cases.

use serde_json::{Value, json};
use tokmd_envelope::ffi::{
    EnvelopeExtractError, extract_data, extract_data_from_json, extract_data_json,
    format_error_message, parse_envelope,
};

// ===========================================================================
// Envelope creation with various payloads
// ===========================================================================

#[test]
fn envelope_with_nested_object_payload() {
    let envelope = json!({
        "ok": true,
        "data": {
            "receipt": {
                "total_code": 1000,
                "languages": {"Rust": 800, "Python": 200}
            }
        }
    });
    let data = extract_data(envelope).unwrap();
    assert_eq!(data["receipt"]["total_code"], 1000);
    assert_eq!(data["receipt"]["languages"]["Rust"], 800);
}

#[test]
fn envelope_with_mixed_type_array_payload() {
    let envelope = json!({
        "ok": true,
        "data": [1, "two", true, null, {"nested": 3}]
    });
    let data = extract_data(envelope).unwrap();
    let arr = data.as_array().unwrap();
    assert_eq!(arr.len(), 5);
    assert_eq!(arr[0], 1);
    assert_eq!(arr[1], "two");
    assert_eq!(arr[2], true);
    assert!(arr[3].is_null());
    assert_eq!(arr[4]["nested"], 3);
}

#[test]
fn envelope_with_numeric_payload() {
    let envelope = json!({"ok": true, "data": 3.15});
    let data = extract_data(envelope).unwrap();
    assert!((data.as_f64().unwrap() - 3.15).abs() < 0.001);
}

#[test]
fn envelope_with_large_string_payload() {
    let big = "x".repeat(100_000);
    let envelope = json!({"ok": true, "data": big});
    let data = extract_data(envelope).unwrap();
    assert_eq!(data.as_str().unwrap().len(), 100_000);
}

#[test]
fn envelope_with_empty_data_object() {
    let envelope = json!({"ok": true, "data": {}});
    let data = extract_data(envelope).unwrap();
    assert!(data.as_object().unwrap().is_empty());
}

#[test]
fn envelope_with_empty_data_array() {
    let envelope = json!({"ok": true, "data": []});
    let data = extract_data(envelope).unwrap();
    assert!(data.as_array().unwrap().is_empty());
}

// ===========================================================================
// Serialization produces valid JSON
// ===========================================================================

#[test]
fn extract_data_json_produces_valid_json() {
    let input = r#"{"ok":true,"data":{"count":42,"items":[1,2,3]}}"#;
    let output = extract_data_json(input).unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["count"], 42);
    assert_eq!(parsed["items"].as_array().unwrap().len(), 3);
}

#[test]
fn extract_data_json_preserves_nested_structure() {
    let input = r#"{"ok":true,"data":{"a":{"b":{"c":true}}}}"#;
    let output = extract_data_json(input).unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["a"]["b"]["c"], true);
}

#[test]
fn extract_data_json_scalar_payload() {
    let input = r#"{"ok":true,"data":"hello world"}"#;
    let output = extract_data_json(input).unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed, "hello world");
}

#[test]
fn extract_data_json_null_payload() {
    let input = r#"{"ok":true,"data":null}"#;
    let output = extract_data_json(input).unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    assert!(parsed.is_null());
}

// ===========================================================================
// Error envelope format
// ===========================================================================

#[test]
fn error_envelope_with_all_fields() {
    let envelope = json!({
        "ok": false,
        "error": {"code": "scan_failed", "message": "No files found"}
    });
    let err = extract_data(envelope).unwrap_err();
    match err {
        EnvelopeExtractError::Upstream(msg) => {
            assert!(msg.contains("scan_failed"));
            assert!(msg.contains("No files found"));
            assert!(msg.starts_with("["));
        }
        other => panic!("Expected Upstream, got {other:?}"),
    }
}

#[test]
fn error_envelope_missing_error_field() {
    let envelope = json!({"ok": false});
    let err = extract_data(envelope).unwrap_err();
    match err {
        EnvelopeExtractError::Upstream(msg) => {
            assert_eq!(msg, "Unknown error");
        }
        other => panic!("Expected Upstream, got {other:?}"),
    }
}

#[test]
fn error_envelope_empty_error_object() {
    let envelope = json!({"ok": false, "error": {}});
    let err = extract_data(envelope).unwrap_err();
    match err {
        EnvelopeExtractError::Upstream(msg) => {
            assert_eq!(msg, "[unknown] Unknown error");
        }
        other => panic!("Expected Upstream, got {other:?}"),
    }
}

#[test]
fn error_envelope_with_extra_fields_ignored() {
    let envelope = json!({
        "ok": false,
        "error": {"code": "e", "message": "m", "trace": "stacktrace", "debug": true}
    });
    let err = extract_data(envelope).unwrap_err();
    match err {
        EnvelopeExtractError::Upstream(msg) => {
            assert_eq!(msg, "[e] m");
        }
        other => panic!("Expected Upstream, got {other:?}"),
    }
}

#[test]
fn error_format_non_object_error_field() {
    let envelope = json!({"ok": false, "error": "just a string"});
    let err = extract_data(envelope).unwrap_err();
    match err {
        EnvelopeExtractError::Upstream(msg) => {
            assert_eq!(msg, "Unknown error");
        }
        other => panic!("Expected Upstream, got {other:?}"),
    }
}

// ===========================================================================
// Version field consistency (ok field handling)
// ===========================================================================

#[test]
fn ok_true_extracts_successfully() {
    let envelope = json!({"ok": true, "data": 1});
    assert!(extract_data(envelope).is_ok());
}

#[test]
fn ok_false_always_errors() {
    let envelope = json!({"ok": false, "data": 1});
    assert!(extract_data(envelope).is_err());
}

#[test]
fn ok_missing_treated_as_false() {
    let envelope = json!({"data": {"x": 1}});
    assert!(extract_data(envelope).is_err());
}

#[test]
fn ok_null_treated_as_false() {
    let envelope = json!({"ok": null, "data": 1});
    assert!(extract_data(envelope).is_err());
}

#[test]
fn ok_string_true_treated_as_false() {
    let envelope = json!({"ok": "true", "data": 1});
    assert!(extract_data(envelope).is_err());
}

#[test]
fn ok_integer_one_treated_as_false() {
    let envelope = json!({"ok": 1, "data": 1});
    assert!(extract_data(envelope).is_err());
}

// ===========================================================================
// parse_envelope edge cases
// ===========================================================================

#[test]
fn parse_deeply_nested_json() {
    let input = r#"{"a":{"b":{"c":{"d":{"e":{"f":42}}}}}}"#;
    let val = parse_envelope(input).unwrap();
    assert_eq!(val["a"]["b"]["c"]["d"]["e"]["f"], 42);
}

#[test]
fn parse_unicode_content() {
    let input = r#"{"ok":true,"data":"こんにちは🦀"}"#;
    let val = parse_envelope(input).unwrap();
    assert_eq!(val["data"], "こんにちは🦀");
}

#[test]
fn parse_whitespace_padded_json() {
    let val = parse_envelope("  \n  { \"ok\": true }  \n  ").unwrap();
    assert_eq!(val["ok"], true);
}

// ===========================================================================
// extract_data_from_json combined behavior
// ===========================================================================

#[test]
fn from_json_ok_without_data_returns_full_envelope() {
    let input = r#"{"ok":true,"version":"1.0"}"#;
    let data = extract_data_from_json(input).unwrap();
    assert_eq!(data["ok"], true);
    assert_eq!(data["version"], "1.0");
}

#[test]
fn from_json_error_envelope_message_preserved() {
    let input = r#"{"ok":false,"error":{"code":"timeout","message":"Request timed out"}}"#;
    let err = extract_data_from_json(input).unwrap_err();
    assert!(err.to_string().contains("timeout"));
    assert!(err.to_string().contains("Request timed out"));
}

// ===========================================================================
// Error variant Display consistency
// ===========================================================================

#[test]
fn json_parse_error_display_prefix() {
    let err = parse_envelope("<<<").unwrap_err();
    let msg = err.to_string();
    assert!(msg.starts_with("JSON parse error:"));
}

#[test]
fn invalid_format_display() {
    let err = extract_data(json!("string")).unwrap_err();
    assert_eq!(err.to_string(), "Invalid response format");
}

#[test]
fn upstream_error_display_passthrough() {
    let err = EnvelopeExtractError::Upstream("custom message".to_string());
    assert_eq!(err.to_string(), "custom message");
}

// ===========================================================================
// Determinism across invocations
// ===========================================================================

#[test]
fn parse_and_extract_deterministic() {
    let input = r#"{"ok":true,"data":{"k":"v","n":1,"a":[true]}}"#;
    let r1 = extract_data_from_json(input).unwrap();
    let r2 = extract_data_from_json(input).unwrap();
    assert_eq!(r1, r2);
}

#[test]
fn extract_data_json_deterministic() {
    let input = r#"{"ok":true,"data":{"z":1,"a":2}}"#;
    let o1 = extract_data_json(input).unwrap();
    let o2 = extract_data_json(input).unwrap();
    assert_eq!(o1, o2);
}

// ===========================================================================
// format_error_message edge cases
// ===========================================================================

#[test]
fn format_error_with_empty_strings() {
    let err = json!({"code": "", "message": ""});
    assert_eq!(format_error_message(Some(&err)), "[] ");
}

#[test]
fn format_error_with_special_characters() {
    let err = json!({"code": "err/path", "message": "file \"not\" found <here>"});
    let msg = format_error_message(Some(&err));
    assert!(msg.contains("err/path"));
    assert!(msg.contains("file \"not\" found <here>"));
}

#[test]
fn format_error_with_boolean_value() {
    assert_eq!(format_error_message(Some(&json!(true))), "Unknown error");
    assert_eq!(format_error_message(Some(&json!(false))), "Unknown error");
}

#[test]
fn format_error_with_number_value() {
    assert_eq!(format_error_message(Some(&json!(42))), "Unknown error");
}
