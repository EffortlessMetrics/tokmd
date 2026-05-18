use super::ErrorCode;
use serde::{Deserialize, Serialize};
use std::fmt;

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

    /// Create an invalid path error.
    pub fn invalid_path(message: impl Into<String>) -> Self {
        Self::with_suggestions(
            ErrorCode::InvalidPath,
            message.into(),
            vec![
                "Use paths inside the selected scan root".to_string(),
                "Avoid parent traversal (`..`) in root-relative paths".to_string(),
            ],
        )
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
        Self::with_details(
            ErrorCode::InvalidSettings,
            format!("Invalid value for '{}': expected {}", field, expected),
            field.to_string(),
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
