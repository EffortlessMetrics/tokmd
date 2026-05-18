use super::TokmdError;
use serde::{Deserialize, Serialize};

/// JSON error response wrapper for FFI.
///
/// DEPRECATED: Use ResponseEnvelope instead for new code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Always `true` for error responses.
    pub error: bool,
    /// The error code.
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// Optional details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl From<TokmdError> for ErrorResponse {
    fn from(err: TokmdError) -> Self {
        Self {
            error: true,
            code: err.code.to_string(),
            message: err.message,
            details: err.details,
        }
    }
}

impl ErrorResponse {
    /// Convert to JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            format!(
                r#"{{"error":true,"code":"{}","message":"{}"}}"#,
                self.code, self.message
            )
        })
    }
}
