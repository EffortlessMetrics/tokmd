use tokmd_envelope::ffi::format_error_message;
use serde_json::json;

#[test]
fn format_error_message_includes_suggestions() {
    let err = json!({
        "code": "git_not_available",
        "message": "git is not available on PATH",
        "suggestions": [
            "Install git",
            "Ensure git is in PATH"
        ]
    });

    let msg = format_error_message(Some(&err));
    assert!(msg.contains("[git_not_available] git is not available on PATH"));
    assert!(msg.contains("Suggestions:"));
    assert!(msg.contains("  - Install git"));
    assert!(msg.contains("  - Ensure git is in PATH"));
}
