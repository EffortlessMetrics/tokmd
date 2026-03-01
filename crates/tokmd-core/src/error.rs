//! Structured error types for binding-friendly API.
//!
//! These error types are designed to be easily converted to JSON
//! for FFI boundaries while providing rich error information.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

/// Error codes for tokmd operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// Path does not exist or is not accessible.
    PathNotFound,
    /// Invalid path format.
    InvalidPath,
    /// Scan operation failed.
    ScanError,
    /// Analysis operation failed.
    AnalysisError,
    /// Invalid JSON input.
    InvalidJson,
    /// Unknown operation mode.
    UnknownMode,
    /// Invalid settings/arguments.
    InvalidSettings,
    /// I/O error during operation.
    IoError,
    /// Internal error (unexpected state).
    InternalError,
    /// Feature not yet implemented.
    NotImplemented,
    /// Git is not available on PATH.
    GitNotAvailable,
    /// Not inside a git repository.
    NotGitRepository,
    /// Git operation failed.
    GitOperationFailed,
    /// Configuration file not found.
    ConfigNotFound,
    /// Configuration file invalid.
    ConfigInvalid,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCode::PathNotFound => write!(f, "path_not_found"),
            ErrorCode::InvalidPath => write!(f, "invalid_path"),
            ErrorCode::ScanError => write!(f, "scan_error"),
            ErrorCode::AnalysisError => write!(f, "analysis_error"),
            ErrorCode::InvalidJson => write!(f, "invalid_json"),
            ErrorCode::UnknownMode => write!(f, "unknown_mode"),
            ErrorCode::InvalidSettings => write!(f, "invalid_settings"),
            ErrorCode::IoError => write!(f, "io_error"),
            ErrorCode::InternalError => write!(f, "internal_error"),
            ErrorCode::NotImplemented => write!(f, "not_implemented"),
            ErrorCode::GitNotAvailable => write!(f, "git_not_available"),
            ErrorCode::NotGitRepository => write!(f, "not_git_repository"),
            ErrorCode::GitOperationFailed => write!(f, "git_operation_failed"),
            ErrorCode::ConfigNotFound => write!(f, "config_not_found"),
            ErrorCode::ConfigInvalid => write!(f, "config_invalid"),
        }
    }
}

/// Structured error for FFI-friendly error reporting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokmdError {
    /// Error code for programmatic handling.
    pub code: ErrorCode,
    /// Human-readable error message.
    pub message: String,
    /// Optional additional details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    /// Optional helpful suggestions for resolving the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestions: Option<Vec<String>>,
}

impl TokmdError {
    /// Create a new error with given code and message.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
            suggestions: None,
        }
    }

    /// Create an error with additional details.
    pub fn with_details(
        code: ErrorCode,
        message: impl Into<String>,
        details: impl Into<String>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            details: Some(details.into()),
            suggestions: None,
        }
    }

    /// Create an error with suggestions.
    pub fn with_suggestions(
        code: ErrorCode,
        message: impl Into<String>,
        suggestions: Vec<String>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
            suggestions: Some(suggestions),
        }
    }

    /// Create an error with both details and suggestions.
    pub fn with_details_and_suggestions(
        code: ErrorCode,
        message: impl Into<String>,
        details: impl Into<String>,
        suggestions: Vec<String>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            details: Some(details.into()),
            suggestions: Some(suggestions),
        }
    }

    /// Create a git not available error.
    pub fn git_not_available() -> Self {
        Self::with_suggestions(
            ErrorCode::GitNotAvailable,
            "git is not available on PATH".to_string(),
            vec![
                "Install git from https://git-scm.com/downloads".to_string(),
                "Ensure git is in your system PATH".to_string(),
                "Verify installation by running: git --version".to_string(),
            ],
        )
    }

    /// Create a not git repository error.
    pub fn not_git_repository(path: &str) -> Self {
        Self::with_details_and_suggestions(
            ErrorCode::NotGitRepository,
            format!("Not inside a git repository: {}", path),
            "The current directory is not a git repository".to_string(),
            vec![
                "Initialize a git repository: git init".to_string(),
                "Navigate to a git repository directory".to_string(),
                "Use --no-git flag to disable git features".to_string(),
            ],
        )
    }

    /// Create a git operation failed error.
    pub fn git_operation_failed(operation: &str, reason: &str) -> Self {
        Self::with_details(
            ErrorCode::GitOperationFailed,
            format!("Git operation failed: {}", operation),
            format!("Reason: {}", reason),
        )
    }

    /// Create a config not found error.
    pub fn config_not_found(path: &str) -> Self {
        Self::with_suggestions(
            ErrorCode::ConfigNotFound,
            format!("Configuration file not found: {}", path),
            vec![
                "Create a tokmd.toml configuration file".to_string(),
                "Run 'tokmd init' to generate a template".to_string(),
                "Use default settings by omitting --config flag".to_string(),
            ],
        )
    }

    /// Create a config invalid error.
    pub fn config_invalid(path: &str, reason: &str) -> Self {
        Self::with_details_and_suggestions(
            ErrorCode::ConfigInvalid,
            format!("Invalid configuration file: {}", path),
            format!("Reason: {}", reason),
            vec![
                "Check the configuration file syntax".to_string(),
                "Refer to documentation for valid options".to_string(),
                "Run 'tokmd init' to generate a valid template".to_string(),
            ],
        )
    }

    /// Create a path not found error with suggestions.
    pub fn path_not_found_with_suggestions(path: &str) -> Self {
        Self::with_details_and_suggestions(
            ErrorCode::PathNotFound,
            format!("Path not found: {}", path),
            "The specified path does not exist or is not accessible".to_string(),
            vec![
                "Check the path spelling".to_string(),
                "Verify the path exists: ls -la".to_string(),
                "Ensure you have read permissions".to_string(),
            ],
        )
    }

    /// Create a path not found error.
    pub fn path_not_found(path: &str) -> Self {
        Self::new(ErrorCode::PathNotFound, format!("Path not found: {}", path))
    }

    /// Create an invalid JSON error.
    pub fn invalid_json(err: impl fmt::Display) -> Self {
        Self::new(ErrorCode::InvalidJson, format!("Invalid JSON: {}", err))
    }

    /// Create an unknown mode error.
    pub fn unknown_mode(mode: &str) -> Self {
        Self::new(ErrorCode::UnknownMode, format!("Unknown mode: {}", mode))
    }

    /// Create a scan error from an anyhow error.
    pub fn scan_error(err: impl fmt::Display) -> Self {
        Self::new(ErrorCode::ScanError, format!("Scan failed: {}", err))
    }

    /// Create an analysis error from an anyhow error.
    pub fn analysis_error(err: impl fmt::Display) -> Self {
        Self::new(
            ErrorCode::AnalysisError,
            format!("Analysis failed: {}", err),
        )
    }

    /// Create an I/O error.
    pub fn io_error(err: impl fmt::Display) -> Self {
        Self::new(ErrorCode::IoError, format!("I/O error: {}", err))
    }

    /// Create an internal error.
    pub fn internal(err: impl fmt::Display) -> Self {
        Self::new(ErrorCode::InternalError, format!("Internal error: {}", err))
    }

    /// Create a not implemented error.
    pub fn not_implemented(feature: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotImplemented, feature)
    }

    /// Create an invalid settings error for a specific field.
    pub fn invalid_field(field: &str, expected: &str) -> Self {
        Self::new(
            ErrorCode::InvalidSettings,
            format!("Invalid value for '{}': expected {}", field, expected),
        )
    }

    /// Convert to JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            format!(r#"{{"code":"{}","message":"{}"}}"#, self.code, self.message)
        })
    }
}

impl fmt::Display for TokmdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(details) = &self.details {
            write!(f, "[{}] {}: {}", self.code, self.message, details)
        } else {
            write!(f, "[{}] {}", self.code, self.message)
        }
    }
}

impl std::error::Error for TokmdError {}

impl From<anyhow::Error> for TokmdError {
    fn from(err: anyhow::Error) -> Self {
        Self::internal(err)
    }
}

impl From<serde_json::Error> for TokmdError {
    fn from(err: serde_json::Error) -> Self {
        Self::invalid_json(err)
    }
}

impl From<std::io::Error> for TokmdError {
    fn from(err: std::io::Error) -> Self {
        Self::io_error(err)
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_codes_serialize_to_snake_case() {
        let err = TokmdError::path_not_found("/some/path");
        let json = err.to_json();
        assert!(json.contains("\"code\":\"path_not_found\""));
    }

    #[test]
    fn error_response_has_error_true() {
        let err = TokmdError::unknown_mode("foo");
        let resp: ErrorResponse = err.into();
        assert!(resp.error);
        assert_eq!(resp.code, "unknown_mode");
    }

    #[test]
    fn error_display_includes_code() {
        let err = TokmdError::new(ErrorCode::ScanError, "test message");
        let display = err.to_string();
        assert!(display.contains("[scan_error]"));
        assert!(display.contains("test message"));
    }

    #[test]
    fn invalid_field_error() {
        let err = TokmdError::invalid_field("children", "'collapse' or 'separate'");
        assert_eq!(err.code, ErrorCode::InvalidSettings);
        assert!(err.message.contains("children"));
        assert!(err.message.contains("'collapse' or 'separate'"));
    }

    #[test]
    fn response_envelope_success() {
        let data = serde_json::json!({"rows": []});
        let envelope = ResponseEnvelope::success(data.clone());
        assert!(envelope.ok);
        assert!(envelope.error.is_none());
        assert_eq!(envelope.data, Some(data));
    }

    #[test]
    fn error_with_suggestions() {
        let err = TokmdError::git_not_available();
        assert_eq!(err.code, ErrorCode::GitNotAvailable);
        assert!(err.suggestions.is_some());
        let suggestions = err.suggestions.expect("should have suggestions");
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn error_with_details_and_suggestions() {
        let err = TokmdError::not_git_repository("/some/path");
        assert_eq!(err.code, ErrorCode::NotGitRepository);
        assert!(err.details.is_some());
        assert!(err.suggestions.is_some());
    }
}
