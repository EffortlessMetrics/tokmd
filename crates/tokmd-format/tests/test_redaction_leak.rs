use tokmd_format::redact_path;

#[test]
fn test_redact_path_leak() {
    let leaked_data = "super_secret_password_123";
    let path = format!("file.{}", leaked_data);
    let redacted = redact_path(&path);
    assert!(
        !redacted.contains(leaked_data),
        "Path redaction leaked the extension!"
    );
}
