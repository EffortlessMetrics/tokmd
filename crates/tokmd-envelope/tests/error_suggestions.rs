use serde_json::json;
use tokmd_envelope::ffi::format_error_message;

#[test]
fn test_suggestions() {
    let err = json!({
        "code": "scan_failed",
        "message": "Path not found",
        "suggestions": ["Check path", "Use absolute path"]
    });
    let formatted = format_error_message(Some(&err));
    assert!(
        formatted.contains("Check path"),
        "Missing suggestion in: {}",
        formatted
    );
}
