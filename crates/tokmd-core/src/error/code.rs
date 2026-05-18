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
