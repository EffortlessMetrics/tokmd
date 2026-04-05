//! JSON envelope parsing/extraction helpers for tokmd FFI bindings.
//!
//! This crate centralizes handling of the `{"ok": bool, "data": ..., "error": ...}`
//! response envelope used by `tokmd_core::ffi::run_json`.

#![forbid(unsafe_code)]

use serde_json::Value;

/// Errors produced while parsing or extracting a response envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvelopeExtractError {
    /// Input could not be parsed as JSON.
    JsonParse(String),
    /// Extracted value could not be serialized back to JSON.
    JsonSerialize(String),
    /// Envelope is not a JSON object.
    InvalidResponseFormat,
    /// Upstream returned `{ "ok": false, "error": ... }`.
    Upstream(String),
}

impl std::fmt::Display for EnvelopeExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::JsonParse(err) => write!(f, "JSON parse error: {err}"),
            Self::JsonSerialize(err) => write!(f, "JSON serialize error: {err}"),
            Self::InvalidResponseFormat => write!(f, "Invalid response format"),
            Self::Upstream(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for EnvelopeExtractError {}

/// Parse a JSON envelope.
///
/// # Examples
///
/// ```
/// use tokmd_ffi_envelope::parse_envelope;
///
/// let val = parse_envelope(r#"{"ok": true, "data": 42}"#).expect("valid JSON envelope should parse");
/// assert_eq!(val["ok"], true);
/// assert_eq!(val["data"], 42);
///
/// // Invalid JSON returns an error
/// assert!(parse_envelope("{not json").is_err());
/// ```
pub fn parse_envelope(result_json: &str) -> Result<Value, EnvelopeExtractError> {
    serde_json::from_str(result_json)
        .map_err(|err| EnvelopeExtractError::JsonParse(err.to_string()))
}

/// Format an upstream error object into a stable message.
///
/// Expected shape: `{"code": "...", "message": "..."}`.
/// Falls back to `"Unknown error"` when missing or invalid.
///
/// # Examples
///
/// ```
/// use tokmd_ffi_envelope::format_error_message;
/// use serde_json::json;
///
/// let err = json!({"code": "scan_failed", "message": "Path not found"});
/// assert_eq!(format_error_message(Some(&err)), "[scan_failed] Path not found");
///
/// // Missing fields fall back to defaults
/// assert_eq!(format_error_message(None), "Unknown error");
/// ```
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
///
/// # Examples
///
/// ```
/// use tokmd_ffi_envelope::extract_data;
/// use serde_json::json;
///
/// let envelope = json!({"ok": true, "data": {"count": 5}});
/// let data = extract_data(envelope).expect("valid OK envelope should extract data");
/// assert_eq!(data["count"], 5);
///
/// // An error envelope returns Err
/// let fail = json!({"ok": false, "error": {"code": "e", "message": "boom"}});
/// assert!(extract_data(fail).is_err());
/// ```
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
///
/// # Examples
///
/// ```
/// use tokmd_ffi_envelope::extract_data_from_json;
///
/// let json_str = r#"{"ok": true, "data": {"mode": "lang"}}"#;
/// let data = extract_data_from_json(json_str).expect("valid JSON should extract successfully");
/// assert_eq!(data["mode"], "lang");
/// ```
pub fn extract_data_from_json(result_json: &str) -> Result<Value, EnvelopeExtractError> {
    let envelope = parse_envelope(result_json)?;
    extract_data(envelope)
}

/// Parse and extract, returning a JSON-encoded data payload.
///
/// # Examples
///
/// ```
/// use tokmd_ffi_envelope::extract_data_json;
///
/// let input = r#"{"ok": true, "data": {"v": 1}}"#;
/// let json_out = extract_data_json(input).expect("valid JSON should extract successfully");
/// let parsed: serde_json::Value = serde_json::from_str(&json_out).expect("serialized JSON should parse");
/// assert_eq!(parsed["v"], 1);
/// ```
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
        let err = parse_envelope("{").expect_err("invalid JSON should fail to parse");
        assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
        assert!(err.to_string().contains("JSON parse error"));
    }

    #[test]
    fn extract_data_success_returns_data() {
        let envelope = json!({
            "ok": true,
            "data": { "mode": "version" }
        });
        let data = extract_data(envelope).expect("valid envelope should extract data");
        assert_eq!(data["mode"], "version");
    }

    #[test]
    fn extract_data_success_without_data_returns_envelope() {
        let envelope = json!({
            "ok": true,
            "mode": "version"
        });
        let data = extract_data(envelope.clone()).expect("valid envelope without data should return itself");
        assert_eq!(data, envelope);
    }

    #[test]
    fn extract_data_error_formats_message() {
        let envelope = json!({
            "ok": false,
            "error": { "code": "unknown_mode", "message": "Unknown mode: nope" }
        });
        let err = extract_data(envelope).expect_err("error envelope should fail to extract data");
        assert_eq!(
            err,
            EnvelopeExtractError::Upstream("[unknown_mode] Unknown mode: nope".to_string())
        );
    }

    #[test]
    fn extract_data_non_object_is_invalid_format() {
        let err = extract_data(json!(["not", "an", "envelope"])).expect_err("non-object JSON should fail to extract data");
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
        let encoded = extract_data_json(&envelope.to_string()).expect("valid JSON should encode successfully");
        let parsed: Value = serde_json::from_str(&encoded).expect("encoded JSON should parse");
        assert_eq!(parsed["a"], 1);
        assert_eq!(parsed["b"], true);
    }
}
