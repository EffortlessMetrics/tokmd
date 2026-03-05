//! Deep tests for tokmd-ffi-envelope — W69
//!
//! Covers: parse_envelope, extract_data, extract_data_from_json,
//! extract_data_json, format_error_message, error variants, and
//! determinism properties.

use proptest::prelude::*;
use serde_json::{json, Value};
use tokmd_ffi_envelope::{
    extract_data, extract_data_from_json, extract_data_json, format_error_message, parse_envelope,
    EnvelopeExtractError,
};

// ── parse_envelope ──────────────────────────────────────────────────

#[test]
fn parse_envelope_valid_object() {
    let v = parse_envelope(r#"{"ok": true}"#).unwrap();
    assert_eq!(v, json!({"ok": true}));
}

#[test]
fn parse_envelope_valid_array() {
    let v = parse_envelope("[1,2,3]").unwrap();
    assert_eq!(v, json!([1, 2, 3]));
}

#[test]
fn parse_envelope_empty_string_is_error() {
    let err = parse_envelope("").unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
}

#[test]
fn parse_envelope_truncated_json_is_error() {
    let err = parse_envelope(r#"{"ok": tru"#).unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
}

#[test]
fn parse_envelope_error_display_contains_context() {
    let err = parse_envelope("not-json").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("JSON parse error"), "got: {msg}");
}

// ── extract_data ────────────────────────────────────────────────────

#[test]
fn extract_data_ok_true_returns_data_field() {
    let envelope = json!({"ok": true, "data": {"lang": "Rust"}});
    let data = extract_data(envelope).unwrap();
    assert_eq!(data, json!({"lang": "Rust"}));
}

#[test]
fn extract_data_ok_true_null_data_returns_null() {
    let envelope = json!({"ok": true, "data": null});
    let data = extract_data(envelope).unwrap();
    assert_eq!(data, Value::Null);
}

#[test]
fn extract_data_ok_true_no_data_key_returns_full_envelope() {
    let envelope = json!({"ok": true, "extra": 42});
    let data = extract_data(envelope.clone()).unwrap();
    assert_eq!(data, envelope);
}

#[test]
fn extract_data_ok_false_yields_upstream_error() {
    let envelope = json!({"ok": false, "error": {"code": "e1", "message": "bad"}});
    let err = extract_data(envelope).unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
    assert_eq!(err.to_string(), "[e1] bad");
}

#[test]
fn extract_data_missing_ok_treated_as_false() {
    let envelope = json!({"data": 1});
    let err = extract_data(envelope).unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
}

#[test]
fn extract_data_ok_non_bool_treated_as_false() {
    let envelope = json!({"ok": "yes", "data": 1});
    let err = extract_data(envelope).unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
}

#[test]
fn extract_data_string_is_invalid_format() {
    let err = extract_data(json!("just a string")).unwrap_err();
    assert_eq!(err, EnvelopeExtractError::InvalidResponseFormat);
}

#[test]
fn extract_data_number_is_invalid_format() {
    let err = extract_data(json!(42)).unwrap_err();
    assert_eq!(err, EnvelopeExtractError::InvalidResponseFormat);
}

#[test]
fn extract_data_null_is_invalid_format() {
    let err = extract_data(Value::Null).unwrap_err();
    assert_eq!(err, EnvelopeExtractError::InvalidResponseFormat);
}

// ── format_error_message ────────────────────────────────────────────

#[test]
fn format_error_message_full_object() {
    let err = json!({"code": "scan_failed", "message": "Path not found"});
    assert_eq!(
        format_error_message(Some(&err)),
        "[scan_failed] Path not found"
    );
}

#[test]
fn format_error_message_code_only() {
    let err = json!({"code": "timeout"});
    assert_eq!(format_error_message(Some(&err)), "[timeout] Unknown error");
}

#[test]
fn format_error_message_message_only() {
    let err = json!({"message": "oops"});
    assert_eq!(format_error_message(Some(&err)), "[unknown] oops");
}

#[test]
fn format_error_message_non_string_fields() {
    let err = json!({"code": 123, "message": true});
    assert_eq!(
        format_error_message(Some(&err)),
        "[unknown] Unknown error"
    );
}

// ── extract_data_from_json ──────────────────────────────────────────

#[test]
fn extract_data_from_json_round_trip() {
    let input = r#"{"ok":true,"data":{"count":7}}"#;
    let data = extract_data_from_json(input).unwrap();
    assert_eq!(data["count"], 7);
}

#[test]
fn extract_data_from_json_error_envelope() {
    let input = r#"{"ok":false,"error":{"code":"c","message":"m"}}"#;
    let err = extract_data_from_json(input).unwrap_err();
    assert_eq!(err.to_string(), "[c] m");
}

// ── extract_data_json ───────────────────────────────────────────────

#[test]
fn extract_data_json_returns_valid_json_string() {
    let input = r#"{"ok":true,"data":{"a":1,"b":"two"}}"#;
    let json_str = extract_data_json(input).unwrap();
    let parsed: Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed["a"], 1);
    assert_eq!(parsed["b"], "two");
}

#[test]
fn extract_data_json_propagates_parse_error() {
    let err = extract_data_json("{bad").unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
}

// ── determinism ─────────────────────────────────────────────────────

#[test]
fn extract_data_json_deterministic_across_calls() {
    let input = r#"{"ok":true,"data":{"z":1,"a":2}}"#;
    let a = extract_data_json(input).unwrap();
    let b = extract_data_json(input).unwrap();
    assert_eq!(a, b, "output must be byte-identical across calls");
}

// ── error equality ──────────────────────────────────────────────────

#[test]
fn error_variants_eq_clone() {
    let a = EnvelopeExtractError::InvalidResponseFormat;
    let b = a.clone();
    assert_eq!(a, b);

    let c = EnvelopeExtractError::JsonParse("x".into());
    let d = c.clone();
    assert_eq!(c, d);

    let e = EnvelopeExtractError::Upstream("u".into());
    assert_ne!(c, e);
}

// ── proptest ────────────────────────────────────────────────────────

proptest! {
    #[test]
    fn parse_envelope_never_panics(s in "\\PC{0,200}") {
        let _ = parse_envelope(&s);
    }

    #[test]
    fn ok_true_envelope_always_succeeds(v in prop::num::i64::ANY) {
        let envelope = json!({"ok": true, "data": v});
        let data = extract_data(envelope).unwrap();
        assert_eq!(data, json!(v));
    }
}
