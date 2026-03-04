//! Edge-case and boundary-condition tests for tokmd-redact.

use tokmd_redact::{redact_path, short_hash};

// ---------------------------------------------------------------------------
// Empty string
// ---------------------------------------------------------------------------

#[test]
fn short_hash_empty_string() {
    let h = short_hash("");
    assert_eq!(h.len(), 16, "hash should be exactly 16 hex chars");
}

#[test]
fn redact_path_empty_string() {
    let r = redact_path("");
    // No extension, so just the hash
    assert_eq!(r.len(), 16);
}

// ---------------------------------------------------------------------------
// Path with only dots
// ---------------------------------------------------------------------------

#[test]
fn short_hash_dot_dot_path() {
    let h = short_hash("../..");
    assert_eq!(h.len(), 16);
    // Should be a valid hex string
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn redact_path_dot_dot_path() {
    let r = redact_path("../..");
    // ".." has no extension, so bare hash
    assert_eq!(r.len(), 16);
}

// ---------------------------------------------------------------------------
// Path with null bytes
// ---------------------------------------------------------------------------

#[test]
fn short_hash_with_null_bytes() {
    let h = short_hash("src/\0null\0.rs");
    assert_eq!(h.len(), 16);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn redact_path_with_null_bytes() {
    let r = redact_path("src/\0null\0.rs");
    // Should still preserve .rs extension
    assert!(r.ends_with(".rs"));
}

// ---------------------------------------------------------------------------
// Very long path
// ---------------------------------------------------------------------------

#[test]
fn short_hash_very_long_path() {
    let long_path = "a/".repeat(600) + "file.txt";
    let h = short_hash(&long_path);
    assert_eq!(h.len(), 16);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn redact_path_very_long_path() {
    let long_path = "a/".repeat(600) + "file.txt";
    let r = redact_path(&long_path);
    assert!(r.ends_with(".txt"));
    // Output should be compact regardless of input length
    assert!(r.len() < 25);
}

// ---------------------------------------------------------------------------
// Unicode path components
// ---------------------------------------------------------------------------

#[test]
fn short_hash_unicode_path() {
    let h = short_hash("src/日本語/ファイル.rs");
    assert_eq!(h.len(), 16);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn redact_path_unicode_preserves_extension() {
    let r = redact_path("src/日本語/ファイル.rs");
    assert!(r.ends_with(".rs"));
}

// ---------------------------------------------------------------------------
// Determinism: same path always gives same result
// ---------------------------------------------------------------------------

#[test]
fn short_hash_deterministic() {
    let path = "src/main.rs";
    let h1 = short_hash(path);
    let h2 = short_hash(path);
    let h3 = short_hash(path);
    assert_eq!(h1, h2);
    assert_eq!(h2, h3);
}

#[test]
fn redact_path_deterministic() {
    let path = "crates/foo/src/lib.rs";
    let r1 = redact_path(path);
    let r2 = redact_path(path);
    let r3 = redact_path(path);
    assert_eq!(r1, r2);
    assert_eq!(r2, r3);
}

// ---------------------------------------------------------------------------
// Backslash normalization
// ---------------------------------------------------------------------------

#[test]
fn short_hash_backslash_matches_forward_slash() {
    let forward = short_hash("src/lib.rs");
    let back = short_hash("src\\lib.rs");
    assert_eq!(forward, back);
}

#[test]
fn short_hash_dot_slash_normalized() {
    let without = short_hash("src/lib.rs");
    let with_dot = short_hash("./src/lib.rs");
    assert_eq!(without, with_dot);
}

// ---------------------------------------------------------------------------
// Double extension
// ---------------------------------------------------------------------------

#[test]
fn redact_path_double_extension_preserves_last() {
    let r = redact_path("archive.tar.gz");
    assert!(
        r.ends_with(".gz"),
        "should preserve last extension, got: {r}"
    );
    assert!(!r.contains("tar"), "should not expose inner extension");
}
