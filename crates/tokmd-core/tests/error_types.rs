//! BDD-style tests for the tokmd-core error module.
//!
//! Focuses on areas not covered by existing test files:
//! - ErrorCode serde roundtrip for ALL variants
//! - TokmdError factory methods and conversion traits
//! - ResponseEnvelope serialization contract and fallbacks
//! - CORE_SCHEMA_VERSION constant validation
//! - version() format validation

use serde_json::json;
use tokmd_core::error::{ErrorCode, ErrorDetails, ErrorResponse, ResponseEnvelope, TokmdError};

// =========================================================================
// Scenario: ErrorCode serde roundtrip for all variants
// =========================================================================

mod error_code_serde {
    use super::*;

    const ALL_CODES: &[ErrorCode] = &[
        ErrorCode::PathNotFound,
        ErrorCode::InvalidPath,
        ErrorCode::ScanError,
        ErrorCode::AnalysisError,
        ErrorCode::InvalidJson,
        ErrorCode::UnknownMode,
        ErrorCode::InvalidSettings,
        ErrorCode::IoError,
        ErrorCode::InternalError,
        ErrorCode::NotImplemented,
        ErrorCode::GitNotAvailable,
        ErrorCode::NotGitRepository,
        ErrorCode::GitOperationFailed,
        ErrorCode::ConfigNotFound,
        ErrorCode::ConfigInvalid,
    ];

    #[test]
    fn all_error_codes_roundtrip_through_json() {
        for code in ALL_CODES {
            let json = serde_json::to_string(code).unwrap();
            let back: ErrorCode = serde_json::from_str(&json).unwrap();
            assert_eq!(*code, back, "ErrorCode roundtrip failed for {:?}", code);
        }
    }

    #[test]
    fn all_error_codes_serialize_to_snake_case() {
        for code in ALL_CODES {
            let json = serde_json::to_string(code).unwrap();
            let inner = json.trim_matches('"');
            assert!(
                !inner.chars().any(|c| c.is_uppercase()),
                "ErrorCode should serialize to snake_case, got: {}",
                inner
            );
        }
    }

    #[test]
    fn error_code_display_matches_serde() {
        // Display impl should produce the same string as serde serialization
        for code in ALL_CODES {
            let display = code.to_string();
            let serde = serde_json::to_string(code).unwrap();
            let serde_inner = serde.trim_matches('"');
            assert_eq!(
                display, serde_inner,
                "Display and serde disagree for {:?}",
                code
            );
        }
    }
}

// =========================================================================
// Scenario: TokmdError factory methods produce correct codes and messages
// =========================================================================

mod error_factory_methods {
    use super::*;

    #[test]
    fn path_not_found_error() {
        let err = TokmdError::path_not_found("/some/missing/path");
        assert_eq!(err.code, ErrorCode::PathNotFound);
        assert!(err.message.contains("/some/missing/path"));
        assert!(err.details.is_none());
        assert!(err.suggestions.is_none());
    }

    #[test]
    fn path_not_found_with_suggestions_error() {
        let err = TokmdError::path_not_found_with_suggestions("/bad/path");
        assert_eq!(err.code, ErrorCode::PathNotFound);
        assert!(err.message.contains("/bad/path"));
        assert!(err.details.is_some());
        assert!(err.suggestions.is_some());
        assert!(!err.suggestions.unwrap().is_empty());
    }

    #[test]
    fn invalid_json_error() {
        let err = TokmdError::invalid_json("unexpected token");
        assert_eq!(err.code, ErrorCode::InvalidJson);
        assert!(err.message.contains("unexpected token"));
    }

    #[test]
    fn unknown_mode_error() {
        let err = TokmdError::unknown_mode("bogus");
        assert_eq!(err.code, ErrorCode::UnknownMode);
        assert!(err.message.contains("bogus"));
    }

    #[test]
    fn scan_error() {
        let err = TokmdError::scan_error("permission denied");
        assert_eq!(err.code, ErrorCode::ScanError);
        assert!(err.message.contains("permission denied"));
    }

    #[test]
    fn analysis_error() {
        let err = TokmdError::analysis_error("out of memory");
        assert_eq!(err.code, ErrorCode::AnalysisError);
        assert!(err.message.contains("out of memory"));
    }

    #[test]
    fn io_error() {
        let err = TokmdError::io_error("disk full");
        assert_eq!(err.code, ErrorCode::IoError);
        assert!(err.message.contains("disk full"));
    }

    #[test]
    fn internal_error() {
        let err = TokmdError::internal("unexpected state");
        assert_eq!(err.code, ErrorCode::InternalError);
        assert!(err.message.contains("unexpected state"));
    }

    #[test]
    fn not_implemented_error() {
        let err = TokmdError::not_implemented("streaming mode");
        assert_eq!(err.code, ErrorCode::NotImplemented);
        assert_eq!(err.message, "streaming mode");
    }

    #[test]
    fn invalid_field_error() {
        let err = TokmdError::invalid_field("format", "'md' or 'json'");
        assert_eq!(err.code, ErrorCode::InvalidSettings);
        assert!(err.message.contains("format"));
        assert!(err.message.contains("'md' or 'json'"));
    }

    #[test]
    fn git_not_available_has_suggestions() {
        let err = TokmdError::git_not_available();
        assert_eq!(err.code, ErrorCode::GitNotAvailable);
        let suggestions = err.suggestions.expect("should have suggestions");
        assert!(suggestions.len() >= 2, "should have multiple suggestions");
    }

    #[test]
    fn not_git_repository_includes_path() {
        let err = TokmdError::not_git_repository("/tmp/not-a-repo");
        assert_eq!(err.code, ErrorCode::NotGitRepository);
        assert!(err.message.contains("/tmp/not-a-repo"));
        assert!(err.details.is_some());
        assert!(err.suggestions.is_some());
    }

    #[test]
    fn git_operation_failed_includes_details() {
        let err = TokmdError::git_operation_failed("git log", "fatal: not a repository");
        assert_eq!(err.code, ErrorCode::GitOperationFailed);
        assert!(err.message.contains("git log"));
        assert!(
            err.details
                .as_ref()
                .unwrap()
                .contains("fatal: not a repository")
        );
    }

    #[test]
    fn config_not_found_has_suggestions() {
        let err = TokmdError::config_not_found("tokmd.toml");
        assert_eq!(err.code, ErrorCode::ConfigNotFound);
        assert!(err.message.contains("tokmd.toml"));
        let suggestions = err.suggestions.expect("should have suggestions");
        assert!(suggestions.iter().any(|s| s.contains("tokmd init")));
    }

    #[test]
    fn config_invalid_includes_reason() {
        let err = TokmdError::config_invalid("tokmd.toml", "bad syntax");
        assert_eq!(err.code, ErrorCode::ConfigInvalid);
        assert!(err.details.as_ref().unwrap().contains("bad syntax"));
        assert!(err.suggestions.is_some());
    }
}

// =========================================================================
// Scenario: TokmdError conversion traits
// =========================================================================

mod error_conversions {
    use super::*;

    #[test]
    fn from_anyhow_error() {
        let anyhow_err = anyhow::anyhow!("something broke");
        let err: TokmdError = anyhow_err.into();
        assert_eq!(err.code, ErrorCode::InternalError);
        assert!(err.message.contains("something broke"));
    }

    #[test]
    fn from_serde_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let err: TokmdError = json_err.into();
        assert_eq!(err.code, ErrorCode::InvalidJson);
    }

    #[test]
    fn from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err: TokmdError = io_err.into();
        assert_eq!(err.code, ErrorCode::IoError);
        assert!(err.message.contains("file missing"));
    }

    #[test]
    fn error_display_without_details() {
        let err = TokmdError::new(ErrorCode::ScanError, "scan failed");
        let display = err.to_string();
        assert!(display.contains("[scan_error]"));
        assert!(display.contains("scan failed"));
        // Without details, no colon separator for details
        assert!(!display.ends_with(": "));
    }

    #[test]
    fn error_display_with_details() {
        let err = TokmdError::with_details(ErrorCode::ScanError, "scan failed", "timeout");
        let display = err.to_string();
        assert!(display.contains("[scan_error]"));
        assert!(display.contains("scan failed"));
        assert!(display.contains("timeout"));
    }

    #[test]
    fn error_implements_std_error() {
        let err = TokmdError::new(ErrorCode::ScanError, "test");
        // Verify std::error::Error is implemented
        let _: &dyn std::error::Error = &err;
    }
}

// =========================================================================
// Scenario: TokmdError to_json serialization
// =========================================================================

mod error_json_serialization {
    use super::*;

    #[test]
    fn to_json_includes_code_and_message() {
        let err = TokmdError::path_not_found("/path");
        let json = err.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["code"], "path_not_found");
        assert!(parsed["message"].as_str().unwrap().contains("/path"));
    }

    #[test]
    fn to_json_skips_none_details() {
        let err = TokmdError::new(ErrorCode::ScanError, "msg");
        let json = err.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("details").is_none());
        assert!(parsed.get("suggestions").is_none());
    }

    #[test]
    fn to_json_includes_details_when_present() {
        let err = TokmdError::with_details(ErrorCode::ScanError, "msg", "detail");
        let json = err.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["details"], "detail");
    }

    #[test]
    fn to_json_includes_suggestions_when_present() {
        let err = TokmdError::git_not_available();
        let json = err.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["suggestions"].is_array());
        assert!(!parsed["suggestions"].as_array().unwrap().is_empty());
    }
}

// =========================================================================
// Scenario: ResponseEnvelope contract
// =========================================================================

mod response_envelope {
    use super::*;

    #[test]
    fn success_envelope_has_correct_shape() {
        let data = json!({"rows": [1, 2, 3]});
        let envelope = ResponseEnvelope::success(data.clone());
        assert!(envelope.ok);
        assert_eq!(envelope.data, Some(data));
        assert!(envelope.error.is_none());
    }

    #[test]
    fn error_envelope_has_correct_shape() {
        let err = TokmdError::unknown_mode("bogus");
        let envelope = ResponseEnvelope::error(&err);
        assert!(!envelope.ok);
        assert!(envelope.data.is_none());
        let error_details = envelope.error.as_ref().unwrap();
        assert_eq!(error_details.code, "unknown_mode");
        assert!(error_details.message.contains("bogus"));
    }

    #[test]
    fn success_envelope_to_json_is_valid() {
        let envelope = ResponseEnvelope::success(json!({"count": 42}));
        let json = envelope.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["ok"], true);
        assert_eq!(parsed["data"]["count"], 42);
        // error should not be present
        assert!(parsed.get("error").is_none());
    }

    #[test]
    fn error_envelope_to_json_is_valid() {
        let err = TokmdError::scan_error("timeout");
        let envelope = ResponseEnvelope::error(&err);
        let json = envelope.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["ok"], false);
        assert!(parsed.get("data").is_none());
        assert_eq!(parsed["error"]["code"], "scan_error");
    }

    #[test]
    fn error_details_from_tokmd_error() {
        let err = TokmdError::with_details(ErrorCode::ConfigInvalid, "bad config", "syntax error");
        let details: ErrorDetails = ErrorDetails::from(&err);
        assert_eq!(details.code, "config_invalid");
        assert_eq!(details.message, "bad config");
        assert_eq!(details.details, Some("syntax error".to_string()));
    }

    #[test]
    fn error_response_from_tokmd_error() {
        let err = TokmdError::unknown_mode("xyz");
        let resp: ErrorResponse = err.into();
        assert!(resp.error);
        assert_eq!(resp.code, "unknown_mode");
        assert!(resp.message.contains("xyz"));
    }

    #[test]
    fn error_response_to_json_is_valid() {
        let err = TokmdError::path_not_found("/missing");
        let resp: ErrorResponse = err.into();
        let json = resp.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["error"], true);
        assert_eq!(parsed["code"], "path_not_found");
    }

    #[test]
    fn envelope_roundtrip_through_json() {
        // Success envelope
        let success = ResponseEnvelope::success(json!({"x": 1}));
        let json = serde_json::to_string(&success).unwrap();
        let back: ResponseEnvelope = serde_json::from_str(&json).unwrap();
        assert!(back.ok);
        assert_eq!(back.data, Some(json!({"x": 1})));

        // Error envelope
        let err = TokmdError::scan_error("oops");
        let error = ResponseEnvelope::error(&err);
        let json = serde_json::to_string(&error).unwrap();
        let back: ResponseEnvelope = serde_json::from_str(&json).unwrap();
        assert!(!back.ok);
        assert_eq!(back.error.unwrap().code, "scan_error");
    }
}

// =========================================================================
// Scenario: CORE_SCHEMA_VERSION and version() constants
// =========================================================================

mod constants {
    use tokmd_core::CORE_SCHEMA_VERSION;
    use tokmd_types::SCHEMA_VERSION;

    #[test]
    fn core_schema_version_matches_types() {
        assert_eq!(
            CORE_SCHEMA_VERSION, SCHEMA_VERSION,
            "CORE_SCHEMA_VERSION should re-export SCHEMA_VERSION from tokmd-types"
        );
    }

    #[test]
    fn core_schema_version_is_positive() {
        assert!(CORE_SCHEMA_VERSION > 0, "Schema version should be > 0");
    }

    #[test]
    fn version_is_valid_semver() {
        let v = tokmd_core::version();
        assert!(!v.is_empty());
        let parts: Vec<&str> = v.split('.').collect();
        assert!(
            parts.len() >= 2,
            "Version should be semver-like (at least major.minor), got: {}",
            v
        );
        // Major and minor should be numeric
        assert!(
            parts[0].parse::<u32>().is_ok(),
            "Major version should be numeric, got: {}",
            parts[0]
        );
        assert!(
            parts[1].parse::<u32>().is_ok(),
            "Minor version should be numeric, got: {}",
            parts[1]
        );
    }
}
