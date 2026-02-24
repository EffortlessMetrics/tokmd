//! JSON envelope parsing/extraction helpers for tokmd FFI bindings.
//!
//! This crate centralizes handling of the `{"ok": bool, "data": ..., "error": ...}`
//! response envelope used by `tokmd_core::ffi::run_json`.

#![forbid(unsafe_code)]

use serde_json::Value;
use thiserror::Error;

/// Errors produced while parsing or extracting a response envelope.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EnvelopeExtractError {
    /// Input could not be parsed as JSON.
    #[error("JSON parse error: {0}")]
    JsonParse(String),
    /// Extracted value could not be serialized back to JSON.
    #[error("JSON serialize error: {0}")]
    JsonSerialize(String),
    /// Envelope is not a JSON object.
    #[error("Invalid response format")]
    InvalidResponseFormat,
    /// Upstream returned `{ "ok": false, "error": ... }`.
    #[error("{0}")]
    Upstream(String),
}

/// Parse a JSON envelope.
pub fn parse_envelope(result_json: &str) -> Result<Value, EnvelopeExtractError> {
    serde_json::from_str(result_json)
        .map_err(|err| EnvelopeExtractError::JsonParse(err.to_string()))
}

/// Format an upstream error object into a stable message.
///
/// Expected shape: `{"code": "...", "message": "..."}`.
/// Falls back to `"Unknown error"` when missing or invalid.
pub fn format_error_message(error_obj: Option<&Value>) -> String {
    let Some(error_obj) = error_obj else {
        return "Unknown error".to_string();
    };
    let Some(error_obj) = error_obj.as_object() else {
        return "Unknown error".to_string();
    };

    let code = error_obj
        .get("code")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let message = error_obj
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("Unknown error");
    format!("[{code}] {message}")
}

/// Extract `data` from an already-parsed envelope.
///
/// Rules:
/// - If `ok` is true and `data` exists, return `data`.
/// - If `ok` is true and `data` is missing, return the full envelope unchanged.
/// - Otherwise return an `Upstream` error with a normalized message.
pub fn extract_data(envelope: Value) -> Result<Value, EnvelopeExtractError> {
    let Some(obj) = envelope.as_object() else {
        return Err(EnvelopeExtractError::InvalidResponseFormat);
    };

    let ok = obj.get("ok").and_then(Value::as_bool).unwrap_or(false);
    if ok {
        if let Some(data) = obj.get("data") {
            return Ok(data.clone());
        }
        return Ok(envelope);
    }

    Err(EnvelopeExtractError::Upstream(format_error_message(
        obj.get("error"),
    )))
}

/// Parse and extract from a JSON envelope string.
pub fn extract_data_from_json(result_json: &str) -> Result<Value, EnvelopeExtractError> {
    let envelope = parse_envelope(result_json)?;
    extract_data(envelope)
}

/// Parse and extract, returning a JSON-encoded data payload.
pub fn extract_data_json(result_json: &str) -> Result<String, EnvelopeExtractError> {
    let data = extract_data_from_json(result_json)?;
    serde_json::to_string(&data).map_err(|err| EnvelopeExtractError::JsonSerialize(err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_envelope_invalid_json_errors() {
        let err = parse_envelope("{").unwrap_err();
        assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
        assert!(err.to_string().contains("JSON parse error"));
    }

    #[test]
    fn extract_data_success_returns_data() {
        let envelope = json!({
            "ok": true,
            "data": { "mode": "version" }
        });
        let data = extract_data(envelope).unwrap();
        assert_eq!(data["mode"], "version");
    }

    #[test]
    fn extract_data_success_without_data_returns_envelope() {
        let envelope = json!({
            "ok": true,
            "mode": "version"
        });
        let data = extract_data(envelope.clone()).unwrap();
        assert_eq!(data, envelope);
    }

    #[test]
    fn extract_data_error_formats_message() {
        let envelope = json!({
            "ok": false,
            "error": { "code": "unknown_mode", "message": "Unknown mode: nope" }
        });
        let err = extract_data(envelope).unwrap_err();
        assert_eq!(
            err,
            EnvelopeExtractError::Upstream("[unknown_mode] Unknown mode: nope".to_string())
        );
    }

    #[test]
    fn extract_data_non_object_is_invalid_format() {
        let err = extract_data(json!(["not", "an", "envelope"])).unwrap_err();
        assert_eq!(err, EnvelopeExtractError::InvalidResponseFormat);
    }

    #[test]
    fn format_error_message_defaults_when_missing_fields() {
        let missing = json!({});
        assert_eq!(
            format_error_message(Some(&missing)),
            "[unknown] Unknown error"
        );
        assert_eq!(format_error_message(None), "Unknown error");
        assert_eq!(format_error_message(Some(&json!("boom"))), "Unknown error");
    }

    #[test]
    fn extract_data_json_serializes_payload() {
        let envelope = json!({
            "ok": true,
            "data": { "a": 1, "b": true }
        });
        let encoded = extract_data_json(&envelope.to_string()).unwrap();
        let parsed: Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(parsed["a"], 1);
        assert_eq!(parsed["b"], true);
    }
}
