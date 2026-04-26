//! Deep tests for tokmd-envelope::ffi — W69
//!
//! Covers: parse_envelope, extract_data, extract_data_from_json,
//! extract_data_json, format_error_message, error variants, and
//! determinism properties.

use proptest::prelude::*;
use serde_json::{Value, json};
use tokmd_envelope::ffi::{
    EnvelopeExtractError, extract_data, extract_data_from_json, extract_data_json,
    format_error_message, parse_envelope,
};

// ── parse_envelope ──────────────────────────────────────────────────

#[test]
fn parse_envelope_valid_object() -> Result<(), Box<dyn std::error::Error>> {
    let v = parse_envelope(r#"{"ok": true}"#)?;
    assert_eq!(v, json!({"ok": true}));
    Ok(())
}

#[test]
fn parse_envelope_valid_array() -> Result<(), Box<dyn std::error::Error>> {
    let v = parse_envelope("[1,2,3]")?;
    assert_eq!(v, json!([1, 2, 3]));
    Ok(())
}

#[test]
fn parse_envelope_empty_string_is_error() -> Result<(), Box<dyn std::error::Error>> {
    let err = match parse_envelope("") {
        Ok(_) => return Err("Expected error".into()),
        Err(e) => e,
    };
    assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
    Ok(())
}

#[test]
fn parse_envelope_truncated_json_is_error() -> Result<(), Box<dyn std::error::Error>> {
    let err = match parse_envelope(r#"{"ok": tru"#) {
        Ok(_) => return Err("Expected error".into()),
        Err(e) => e,
    };
    assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
    Ok(())
}

#[test]
fn parse_envelope_error_display_contains_context() -> Result<(), Box<dyn std::error::Error>> {
    let err = match parse_envelope("not-json") {
        Ok(_) => return Err("Expected error".into()),
        Err(e) => e,
    };
    let msg = err.to_string();
    assert!(msg.contains("JSON parse error"), "got: {msg}");
    Ok(())
}

// ── extract_data ────────────────────────────────────────────────────

#[test]
fn extract_data_ok_true_returns_data_field() -> Result<(), Box<dyn std::error::Error>> {
    let envelope = json!({"ok": true, "data": {"lang": "Rust"}});
    let data = extract_data(envelope)?;
    assert_eq!(data, json!({"lang": "Rust"}));
    Ok(())
}

#[test]
fn extract_data_ok_true_null_data_returns_null() -> Result<(), Box<dyn std::error::Error>> {
    let envelope = json!({"ok": true, "data": null});
    let data = extract_data(envelope)?;
    assert_eq!(data, Value::Null);
    Ok(())
}

#[test]
fn extract_data_ok_true_no_data_key_returns_full_envelope() -> Result<(), Box<dyn std::error::Error>>
{
    let envelope = json!({"ok": true, "extra": 42});
    let data = extract_data(envelope.clone())?;
    assert_eq!(data, envelope);
    Ok(())
}

#[test]
fn extract_data_ok_false_yields_upstream_error() -> Result<(), Box<dyn std::error::Error>> {
    let envelope = json!({"ok": false, "error": {"code": "e1", "message": "bad"}});
    let err = match extract_data(envelope) {
        Ok(_) => return Err("Expected error".into()),
        Err(e) => e,
    };
    assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
    assert_eq!(err.to_string(), "[e1] bad");
    Ok(())
}

#[test]
fn extract_data_missing_ok_treated_as_false() -> Result<(), Box<dyn std::error::Error>> {
    let envelope = json!({"data": 1});
    let err = match extract_data(envelope) {
        Ok(_) => return Err("Expected error".into()),
        Err(e) => e,
    };
    assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
    Ok(())
}

#[test]
fn extract_data_ok_non_bool_treated_as_false() -> Result<(), Box<dyn std::error::Error>> {
    let envelope = json!({"ok": "yes", "data": 1});
    let err = match extract_data(envelope) {
        Ok(_) => return Err("Expected error".into()),
        Err(e) => e,
    };
    assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
    Ok(())
}

#[test]
fn extract_data_string_is_invalid_format() -> Result<(), Box<dyn std::error::Error>> {
    let err = match extract_data(json!("just a string")) {
        Ok(_) => return Err("Expected error".into()),
        Err(e) => e,
    };
    assert_eq!(err, EnvelopeExtractError::InvalidResponseFormat);
    Ok(())
}

#[test]
fn extract_data_number_is_invalid_format() -> Result<(), Box<dyn std::error::Error>> {
    let err = match extract_data(json!(42)) {
        Ok(_) => return Err("Expected error".into()),
        Err(e) => e,
    };
    assert_eq!(err, EnvelopeExtractError::InvalidResponseFormat);
    Ok(())
}

#[test]
fn extract_data_null_is_invalid_format() -> Result<(), Box<dyn std::error::Error>> {
    let err = match extract_data(Value::Null) {
        Ok(_) => return Err("Expected error".into()),
        Err(e) => e,
    };
    assert_eq!(err, EnvelopeExtractError::InvalidResponseFormat);
    Ok(())
}

// ── format_error_message ────────────────────────────────────────────

#[test]
fn format_error_message_full_object() -> Result<(), Box<dyn std::error::Error>> {
    let err = json!({"code": "scan_failed", "message": "Path not found"});
    assert_eq!(
        format_error_message(Some(&err)),
        "[scan_failed] Path not found"
    );
    Ok(())
}

#[test]
fn format_error_message_code_only() -> Result<(), Box<dyn std::error::Error>> {
    let err = json!({"code": "timeout"});
    assert_eq!(format_error_message(Some(&err)), "[timeout] Unknown error");
    Ok(())
}

#[test]
fn format_error_message_message_only() -> Result<(), Box<dyn std::error::Error>> {
    let err = json!({"message": "oops"});
    assert_eq!(format_error_message(Some(&err)), "[unknown] oops");
    Ok(())
}

#[test]
fn format_error_message_non_string_fields() -> Result<(), Box<dyn std::error::Error>> {
    let err = json!({"code": 123, "message": true});
    assert_eq!(format_error_message(Some(&err)), "[unknown] Unknown error");
    Ok(())
}

// ── extract_data_from_json ──────────────────────────────────────────

#[test]
fn extract_data_from_json_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let input = r#"{"ok":true,"data":{"count":7}}"#;
    let data = extract_data_from_json(input)?;
    assert_eq!(data["count"], 7);
    Ok(())
}

#[test]
fn extract_data_from_json_error_envelope() -> Result<(), Box<dyn std::error::Error>> {
    let input = r#"{"ok":false,"error":{"code":"c","message":"m"}}"#;
    let err = match extract_data_from_json(input) {
        Ok(_) => return Err("Expected error".into()),
        Err(e) => e,
    };
    assert_eq!(err.to_string(), "[c] m");
    Ok(())
}

// ── extract_data_json ───────────────────────────────────────────────

#[test]
fn extract_data_json_returns_valid_json_string() -> Result<(), Box<dyn std::error::Error>> {
    let input = r#"{"ok":true,"data":{"a":1,"b":"two"}}"#;
    let json_str = extract_data_json(input)?;
    let parsed: Value = serde_json::from_str(&json_str)?;
    assert_eq!(parsed["a"], 1);
    assert_eq!(parsed["b"], "two");
    Ok(())
}

#[test]
fn extract_data_json_propagates_parse_error() -> Result<(), Box<dyn std::error::Error>> {
    let err = match extract_data_json("{bad") {
        Ok(_) => return Err("Expected error".into()),
        Err(e) => e,
    };
    assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
    Ok(())
}

// ── determinism ─────────────────────────────────────────────────────

#[test]
fn extract_data_json_deterministic_across_calls() -> Result<(), Box<dyn std::error::Error>> {
    let input = r#"{"ok":true,"data":{"z":1,"a":2}}"#;
    let a = extract_data_json(input)?;
    let b = extract_data_json(input)?;
    assert_eq!(a, b, "output must be byte-identical across calls");
    Ok(())
}

// ── error equality ──────────────────────────────────────────────────

#[test]
fn error_variants_eq_clone() -> Result<(), Box<dyn std::error::Error>> {
    let a = EnvelopeExtractError::InvalidResponseFormat;
    let b = a.clone();
    assert_eq!(a, b);

    let c = EnvelopeExtractError::JsonParse("x".into());
    let d = c.clone();
    assert_eq!(c, d);

    let e = EnvelopeExtractError::Upstream("u".into());
    assert_ne!(c, e);
    Ok(())
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
        let data = match extract_data(envelope) { Ok(d) => d, Err(_) => { prop_assert!(false, "Expected success"); unreachable!() } };
        prop_assert_eq!(data, json!(v));
    }
}
