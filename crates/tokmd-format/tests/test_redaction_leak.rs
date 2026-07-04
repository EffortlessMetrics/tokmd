use tokmd_format::{redact_path, short_hash};

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

#[test]
fn redaction_preserves_known_compound_archive_suffix() {
    let redacted = redact_path("archive.tar.gz");
    assert!(redacted.ends_with(".tar.gz"));
}

#[test]
fn redaction_preserves_only_final_extension_for_unknown_safe_chains() {
    let redacted = redact_path("fixture.json.rs");
    assert!(redacted.ends_with(".rs"));
    assert!(!redacted.ends_with(".json.rs"));
}

#[test]
fn redaction_drops_suffixes_when_final_extension_is_unsafe() {
    let redacted = redact_path("secret.rs.bak");
    assert_eq!(redacted.len(), 16);
    assert!(!redacted.contains(".rs"));
    assert!(!redacted.contains(".bak"));
}

#[test]
fn redaction_normalizes_safe_extension_case() {
    let redacted = redact_path("file.JSON");
    assert!(redacted.ends_with(".json"));
    assert!(!redacted.ends_with(".JSON"));
}

#[test]
fn redaction_normalizes_known_compound_archive_suffix_case() {
    let redacted = redact_path("archive.TAR.GZ");
    assert!(redacted.ends_with(".tar.gz"));
    assert!(!redacted.ends_with(".TAR.GZ"));
}

#[test]
fn strip_prefix_redaction_must_use_short_hash_not_redact_path() {
    let prefix = "src/secret.rs";
    let via_redact_path = redact_path(prefix);
    let via_short_hash = short_hash(prefix);

    assert!(
        via_redact_path.ends_with(".rs"),
        "redact_path preserves a file extension that must not leak for strip_prefix: {via_redact_path}"
    );
    assert_eq!(via_short_hash.len(), 16);
    assert!(!via_short_hash.contains('.'));
    assert_ne!(
        via_redact_path, via_short_hash,
        "strip_prefix redaction must use short_hash semantics, not redact_path"
    );
}
