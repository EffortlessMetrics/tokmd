//! Structured error types for binding-friendly API.
//!
//! These error types are designed to be easily converted to JSON
//! for FFI boundaries while providing rich error information.

use serde::{Deserialize, Serialize};
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
}

impl TokmdError {
    /// Create a new error with the given code and message.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
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
        }
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
        Self::new(ErrorCode::AnalysisError, format!("Analysis failed: {}", err))
    }

    /// Create an I/O error.
    pub fn io_error(err: impl fmt::Display) -> Self {
        Self::new(ErrorCode::IoError, format!("I/O error: {}", err))
    }

    /// Create an internal error.
    pub fn internal(err: impl fmt::Display) -> Self {
        Self::new(ErrorCode::InternalError, format!("Internal error: {}", err))
    }

    /// Convert to JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            format!(
                r#"{{"code":"{}","message":"{}"}}"#,
                self.code, self.message
            )
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

/// JSON error response wrapper for FFI.
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
}
