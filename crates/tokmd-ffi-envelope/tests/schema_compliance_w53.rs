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
fn success_response_has_ok_true_and_data() -> Result<(), Box<dyn std::error::Error>> {
    let envelope = json!({
        "ok": true,
        "data": { "schema_version": 2, "mode": "lang" }
    });
    let data = extract_data(envelope)?;
    assert_eq!(data["schema_version"], 2);
    assert_eq!(data["mode"], "lang");
    Ok(())
}

#[test]
fn success_response_without_data_returns_envelope() -> Result<(), Box<dyn std::error::Error>> {
    let envelope = json!({
        "ok": true,
        "schema_version": 2
    });
    let data = extract_data(envelope.clone())?;
    assert_eq!(data, envelope);
    Ok(())
}

// ---------------------------------------------------------------------------
// 2. Error response has ok=false, error={message: "..."}
// ---------------------------------------------------------------------------

#[test]
fn error_response_has_ok_false_and_error() -> Result<(), Box<dyn std::error::Error>> {
    let envelope = json!({
        "ok": false,
        "error": { "code": "scan_failed", "message": "Path not found" }
    });
    let err = match extract_data(envelope) {
        Ok(_) => return Err("Expected error".into()),
        Err(e) => e,
    };
    match err {
        EnvelopeExtractError::Upstream(msg) => {
            assert!(msg.contains("scan_failed"));
            assert!(msg.contains("Path not found"));
        }
        _ => return Err("Expected Upstream error".into()),
    }
    Ok(())
}

#[test]
fn error_response_with_missing_error_object() -> Result<(), Box<dyn std::error::Error>> {
    let envelope = json!({ "ok": false });
    let err = match extract_data(envelope) {
        Ok(_) => return Err("Expected error".into()),
        Err(e) => e,
    };
    match err {
        EnvelopeExtractError::Upstream(msg) => {
            assert!(msg.contains("Unknown error"));
        }
        _ => return Err("Expected Upstream error".into()),
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 3. Data payload matches inner receipt structure
// ---------------------------------------------------------------------------

#[test]
fn data_payload_preserves_inner_receipt() -> Result<(), Box<dyn std::error::Error>> {
    let inner = json!({
        "schema_version": 2,
        "generated_at_ms": 1_700_000_000_000_u64,
        "tool": { "name": "tokmd", "version": "1.0.0" },
        "mode": "lang",
        "rows": [{ "lang": "Rust", "code": 100 }],
        "total": { "code": 100, "lines": 150, "files": 5, "bytes": 5000, "tokens": 1000 }
    });
    let envelope = json!({ "ok": true, "data": inner.clone() });

    let data = extract_data(envelope)?;
    assert_eq!(data, inner);
    assert_eq!(data["rows"][0]["lang"], "Rust");
    Ok(())
}

// ---------------------------------------------------------------------------
// 4. Version response format
// ---------------------------------------------------------------------------

#[test]
fn version_response_format() -> Result<(), Box<dyn std::error::Error>> {
    let envelope = json!({
        "ok": true,
        "data": { "version": "1.5.0", "schema_version": 2 }
    });
    let data = extract_data(envelope)?;
    assert!(data["version"].is_string());
    assert!(data["schema_version"].is_number());
    Ok(())
}

// ---------------------------------------------------------------------------
// 5. parse_envelope and extract_data_from_json
// ---------------------------------------------------------------------------

#[test]
fn parse_envelope_valid_json() -> Result<(), Box<dyn std::error::Error>> {
    let val = parse_envelope(r#"{"ok": true, "data": 42}"#)?;
    assert_eq!(val["ok"], true);
    assert_eq!(val["data"], 42);
    Ok(())
}

#[test]
fn parse_envelope_invalid_json() -> Result<(), Box<dyn std::error::Error>> {
    let err = match parse_envelope("{invalid}") {
        Ok(_) => return Err("Expected error".into()),
        Err(e) => e,
    };
    assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
    Ok(())
}

#[test]
fn extract_data_from_json_success() -> Result<(), Box<dyn std::error::Error>> {
    let data = extract_data_from_json(r#"{"ok": true, "data": {"v": 1}}"#)?;
    assert_eq!(data["v"], 1);
    Ok(())
}

#[test]
fn extract_data_from_json_error() -> Result<(), Box<dyn std::error::Error>> {
    let err =
        match extract_data_from_json(r#"{"ok": false, "error": {"code": "e", "message": "fail"}}"#)
        {
            Ok(_) => return Err("Expected error".into()),
            Err(e) => e,
        };
    assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
    Ok(())
}

// ---------------------------------------------------------------------------
// 6. extract_data_json returns valid JSON string
// ---------------------------------------------------------------------------

#[test]
fn extract_data_json_returns_json_string() -> Result<(), Box<dyn std::error::Error>> {
    let json_str = extract_data_json(r#"{"ok": true, "data": {"count": 5}}"#)?;
    let parsed: Value = serde_json::from_str(&json_str)?;
    assert_eq!(parsed["count"], 5);
    Ok(())
}

// ---------------------------------------------------------------------------
// 7. format_error_message
// ---------------------------------------------------------------------------

#[test]
fn format_error_message_with_code_and_message() -> Result<(), Box<dyn std::error::Error>> {
    let err = json!({"code": "invalid_mode", "message": "Unknown mode"});
    assert_eq!(
        format_error_message(Some(&err)),
        "[invalid_mode] Unknown mode"
    );
    Ok(())
}

#[test]
fn format_error_message_none_returns_default() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(format_error_message(None), "Unknown error");
    Ok(())
}

// ---------------------------------------------------------------------------
// 8. Non-object envelope
// ---------------------------------------------------------------------------

#[test]
fn non_object_envelope_returns_invalid_format() -> Result<(), Box<dyn std::error::Error>> {
    let err = match extract_data(json!(42)) {
        Ok(_) => return Err("Expected error".into()),
        Err(e) => e,
    };
    assert_eq!(err, EnvelopeExtractError::InvalidResponseFormat);
    Ok(())
}

#[test]
fn array_envelope_returns_invalid_format() -> Result<(), Box<dyn std::error::Error>> {
    let err = match extract_data(json!([1, 2, 3])) {
        Ok(_) => return Err("Expected error".into()),
        Err(e) => e,
    };
    assert_eq!(err, EnvelopeExtractError::InvalidResponseFormat);
    Ok(())
}
