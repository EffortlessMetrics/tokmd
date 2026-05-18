use super::TokmdError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Error details for response envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    /// The error code.
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// Optional additional details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl From<&TokmdError> for ErrorDetails {
    fn from(err: &TokmdError) -> Self {
        Self {
            code: err.code.to_string(),
            message: err.message.clone(),
            details: err.details.clone(),
        }
    }
}

/// Stable JSON response envelope for FFI.
///
/// Success: `{"ok": true, "data": {...}}`
/// Error: `{"ok": false, "error": {"code": "...", "message": "...", "details": ...}}`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseEnvelope {
    /// Whether the operation succeeded.
    pub ok: bool,
    /// The result data (present when ok=true).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    /// The error details (present when ok=false).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorDetails>,
}

impl ResponseEnvelope {
    /// Create a success response with given data.
    pub fn success(data: Value) -> Self {
        Self {
            ok: true,
            data: Some(data),
            error: None,
        }
    }

    /// Create an error response from a TokmdError.
    pub fn error(err: &TokmdError) -> Self {
        Self {
            ok: false,
            data: None,
            error: Some(ErrorDetails::from(err)),
        }
    }

    /// Convert to JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            if self.ok {
                r#"{"ok":true,"data":null}"#.to_string()
            } else {
                let (code, message) = self
                    .error
                    .as_ref()
                    .map(|e| (e.code.as_str(), e.message.as_str()))
                    .unwrap_or(("internal_error", "serialization failed"));
                format!(
                    r#"{{"ok":false,"error":{{"code":"{}","message":"{}"}}}}"#,
                    code, message
                )
            }
        })
    }
}
