//! Mutation-hardening tests for `tokmd-redact`.
//!
//! Targets: constant-replacement, boolean-flip, and comparison-swap
//! mutation survivors in hashing and path-cleaning logic.

use tokmd_redact::{redact_path, short_hash};

// ── short_hash basics ────────────────────────────────────────────────

#[test]
fn hash_always_16_chars() {
    for input in ["", "a", "long/nested/path/to/file.rs", "🦀"] {
        assert_eq!(short_hash(input).len(), 16, "failed for input: {input}");
    }
}

#[test]
fn hash_deterministic() {
    assert_eq!(short_hash("hello"), short_hash("hello"));
}

#[test]
fn hash_empty_string_is_deterministic_nonzero() {
    let h = short_hash("");
    assert_eq!(h.len(), 16);
    // Must not be all zeros (that would indicate the hash was skipped)
    assert!(
        h.chars().any(|c| c != '0'),
        "hash of empty string is all zeros"
    );
}

#[test]
fn hash_single_char_difference_produces_different_hash() {
    assert_ne!(short_hash("a"), short_hash("b"));
    assert_ne!(short_hash("file.rs"), short_hash("file.rr"));
    assert_ne!(short_hash("src/lib"), short_hash("src/lib "));
}

// ── separator normalization ──────────────────────────────────────────

#[test]
fn hash_backslash_equals_forward_slash() {
    assert_eq!(short_hash("a\\b\\c"), short_hash("a/b/c"));
}

#[test]
fn hash_mixed_separators_normalize() {
    assert_eq!(short_hash("a/b\\c/d"), short_hash("a/b/c/d"));
}

// ── dot-segment normalization ────────────────────────────────────────

#[test]
fn hash_leading_dot_slash_stripped() {
    assert_eq!(short_hash("./foo"), short_hash("foo"));
    assert_eq!(short_hash("././foo"), short_hash("foo"));
}

#[test]
fn hash_interior_dot_segment_removed() {
    assert_eq!(short_hash("a/./b"), short_hash("a/b"));
}

#[test]
fn hash_trailing_dot_segment_removed() {
    assert_eq!(short_hash("a/."), short_hash("a"));
}

// ── redact_path ──────────────────────────────────────────────────────

#[test]
fn redact_never_returns_original_path() {
    let paths = [
        "src/main.rs",
        "Cargo.toml",
        "deeply/nested/path/to/module.py",
        "",
        "Makefile",
    ];
    for p in paths {
        let redacted = redact_path(p);
        if !p.is_empty() {
            assert_ne!(redacted, p, "redact_path returned original for: {p}");
        }
    }
}

#[test]
fn redact_preserves_extension() {
    assert!(redact_path("file.json").ends_with(".json"));
    assert!(redact_path("a/b/c.tar.gz").ends_with(".gz"));
    assert!(redact_path("index.html").ends_with(".html"));
}

#[test]
fn redact_no_extension_bare_hash() {
    let r = redact_path("Makefile");
    assert_eq!(r.len(), 16);
    assert!(!r.contains('.'));
}

#[test]
fn redact_cross_platform_consistency() {
    assert_eq!(redact_path("src\\main.rs"), redact_path("src/main.rs"));
    assert_eq!(
        redact_path("crates\\tokmd\\src\\lib.rs"),
        redact_path("crates/tokmd/src/lib.rs")
    );
}

#[test]
fn redact_dot_prefix_normalization() {
    assert_eq!(redact_path("./src/lib.rs"), redact_path("src/lib.rs"));
}

#[test]
fn redact_different_paths_produce_different_hashes() {
    assert_ne!(redact_path("src/a.rs"), redact_path("src/b.rs"));
    assert_ne!(redact_path("file.rs"), redact_path("file.py"));
}
