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

    // New test case to demonstrate the leak
    let leaked_data_short = "secret12";
    let path = format!("file.{}", leaked_data_short);
    let redacted = redact_path(&path);
    assert!(
        !redacted.contains(leaked_data_short),
        "Path redaction leaked the extension!"
    );
}
