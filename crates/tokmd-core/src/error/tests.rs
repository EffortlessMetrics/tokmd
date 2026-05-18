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
    assert_eq!(err.details, Some("children".to_string()));
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

#[test]
fn anyhow_path_not_found_maps_to_path_not_found() {
    let err: TokmdError = anyhow::anyhow!("Path not found: missing-dir").into();
    assert_eq!(err.code, ErrorCode::PathNotFound);
    assert!(err.message.contains("missing-dir"));
    assert!(err.suggestions.is_some());
}

#[test]
fn anyhow_parent_traversal_maps_to_invalid_path() {
    let err: TokmdError =
        anyhow::anyhow!("Bounded path must not contain parent traversal: ../secret.txt").into();
    assert_eq!(err.code, ErrorCode::InvalidPath);
    assert!(err.message.contains("parent traversal"));
    assert!(err.suggestions.is_some());
}

#[test]
fn anyhow_root_escape_maps_to_invalid_path() {
    let err: TokmdError =
        anyhow::anyhow!("Bounded path escapes scan root C:/repo: C:/secret.txt").into();
    assert_eq!(err.code, ErrorCode::InvalidPath);
    assert!(err.message.contains("escapes scan root"));
}

#[test]
fn anyhow_scan_root_resolve_failure_stays_internal() {
    let err: TokmdError =
        anyhow::anyhow!("Failed to resolve scan root C:/repo: permission denied").into();
    assert_eq!(err.code, ErrorCode::InternalError);
    assert!(err.message.contains("Failed to resolve scan root"));
    assert!(err.suggestions.is_none());
}

#[test]
fn anyhow_bounded_path_resolve_failure_stays_internal() {
    let err: TokmdError =
        anyhow::anyhow!("Failed to resolve bounded path src/lib.rs: permission denied").into();
    assert_eq!(err.code, ErrorCode::InternalError);
    assert!(err.message.contains("Failed to resolve bounded path"));
    assert!(err.suggestions.is_none());
}

#[test]
fn generic_anyhow_stays_internal() {
    let err: TokmdError = anyhow::anyhow!("unexpected failure").into();
    assert_eq!(err.code, ErrorCode::InternalError);
}
