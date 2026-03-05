//! Targeted tests for mutation testing coverage gaps (W50).
//!
//! Each test catches common mutations: replacing operators,
//! negating conditions, removing statements.

use tokmd_redact::{redact_path, short_hash};

// ---------------------------------------------------------------------------
// 1. Different paths produce different hashes
// ---------------------------------------------------------------------------

#[test]
fn different_paths_produce_different_hashes() {
    let h1 = short_hash("src/main.rs");
    let h2 = short_hash("src/lib.rs");
    let h3 = short_hash("tests/integration.rs");

    assert_ne!(h1, h2, "distinct paths must hash differently");
    assert_ne!(h2, h3);
    assert_ne!(h1, h3);
}

// ---------------------------------------------------------------------------
// 2. Same path always produces same hash (determinism)
// ---------------------------------------------------------------------------

#[test]
fn same_path_always_produces_same_hash() {
    let path = "crates/tokmd-redact/src/lib.rs";
    let h1 = short_hash(path);
    let h2 = short_hash(path);
    let h3 = short_hash(path);
    assert_eq!(h1, h2);
    assert_eq!(h2, h3);
}

// ---------------------------------------------------------------------------
// 3. redact_path vs raw path: output differs
// ---------------------------------------------------------------------------

#[test]
fn redact_path_differs_from_raw_path() {
    let raw = "src/secrets/config.json";
    let redacted = redact_path(raw);
    assert_ne!(redacted, raw, "redacted output must differ from original");
    assert!(
        !redacted.contains("secrets"),
        "redacted must not contain original directory"
    );
    assert!(
        !redacted.contains("config"),
        "redacted must not contain original filename stem"
    );
}

// ---------------------------------------------------------------------------
// 4. BLAKE3 hash length is always 16 characters
// ---------------------------------------------------------------------------

#[test]
fn blake3_hash_length_always_16() {
    for input in &[
        "",
        "a",
        "hello world",
        "src/very/deep/nested/path/file.rs",
        "🦀",
    ] {
        let h = short_hash(input);
        assert_eq!(
            h.len(),
            16,
            "hash of {:?} should be 16 chars, got {}",
            input,
            h.len()
        );
    }
}

// ---------------------------------------------------------------------------
// 5. Hash is valid hex (lowercase)
// ---------------------------------------------------------------------------

#[test]
fn hash_is_valid_lowercase_hex() {
    let h = short_hash("some/path/to/file.py");
    assert!(
        h.chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
        "hash should be lowercase hex, got: {h}"
    );
}

// ---------------------------------------------------------------------------
// 6. redact_path preserves extension correctly
// ---------------------------------------------------------------------------

#[test]
fn redact_path_preserves_extension() {
    assert!(redact_path("foo.rs").ends_with(".rs"));
    assert!(redact_path("bar.json").ends_with(".json"));
    assert!(redact_path("baz.tar.gz").ends_with(".gz"));
}

// ---------------------------------------------------------------------------
// 7. redact_path without extension: bare 16-char hash
// ---------------------------------------------------------------------------

#[test]
fn redact_path_no_extension_bare_hash() {
    let r = redact_path("Makefile");
    assert_eq!(r.len(), 16);
    assert!(!r.contains('.'));
}

// ---------------------------------------------------------------------------
// 8. Path cleaning: dot-prefix and interior dots are normalized
// ---------------------------------------------------------------------------

#[test]
fn path_cleaning_normalizes_dots() {
    // Leading "./" stripped
    assert_eq!(short_hash("./src/lib.rs"), short_hash("src/lib.rs"));
    // Interior "/." resolved
    assert_eq!(short_hash("crates/./foo"), short_hash("crates/foo"));
    // Trailing "/." resolved
    assert_eq!(short_hash("crates/."), short_hash("crates"));
}

// ---------------------------------------------------------------------------
// 9. Cross-platform separator normalization
// ---------------------------------------------------------------------------

#[test]
fn cross_platform_separator_normalization() {
    assert_eq!(short_hash("a/b/c"), short_hash("a\\b\\c"));
    assert_eq!(redact_path("a/b/c.rs"), redact_path("a\\b\\c.rs"));
}

// ---------------------------------------------------------------------------
// 10. redact_path length: hash + dot + ext
// ---------------------------------------------------------------------------

#[test]
fn redact_path_length_with_extension() {
    let r = redact_path("any/path/here.toml");
    // 16 (hash) + 1 (dot) + 4 (toml)
    assert_eq!(r.len(), 16 + 1 + 4);
}
