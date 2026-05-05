use tokmd_format::redact_path;

#[test]
fn test_redact_path_leak() {
    for leaked_data in ["super_secret_password_123", "passwd", "secret", "pass1234"] {
        let path = format!("file.{}", leaked_data);
        let redacted = redact_path(&path);
        assert!(
            !redacted.contains(leaked_data),
            "Path redaction leaked extension {leaked_data:?}: {redacted}"
        );
    }
}
