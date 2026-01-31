//! Fuzz target for path redaction and hashing utilities.
//!
//! Tests `short_hash()` and `redact_path()` with arbitrary input to verify:
//! - Determinism (same input produces same output)
//! - Output format invariants (hash length, hex characters)
//! - Extension preservation
//! - Cross-platform separator normalization

#![no_main]
use libfuzzer_sys::fuzz_target;
use tokmd_redact::{redact_path, short_hash};

/// Max input size - redaction is O(n) so we can be generous
const MAX_INPUT_SIZE: usize = 64 * 1024; // 64KB

fuzz_target!(|data: &[u8]| {
    if data.len() > MAX_INPUT_SIZE {
        return;
    }

    // Only process valid UTF-8 strings (paths must be valid strings)
    let Ok(input) = std::str::from_utf8(data) else {
        return;
    };

    // Test short_hash invariants
    let hash = short_hash(input);

    // Invariant: Hash is always exactly 16 characters
    assert_eq!(
        hash.len(),
        16,
        "short_hash must produce exactly 16 characters"
    );

    // Invariant: Hash contains only lowercase hex characters
    assert!(
        hash.chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()),
        "short_hash must produce lowercase hex, got: {}",
        hash
    );

    // Invariant: Same input produces same hash (determinism)
    let hash2 = short_hash(input);
    assert_eq!(hash, hash2, "short_hash must be deterministic");

    // Invariant: Separator normalization - Unix and Windows paths hash identically
    let unix_path = input.replace('\\', "/");
    let windows_path = input.replace('/', "\\");
    assert_eq!(
        short_hash(&unix_path),
        short_hash(&windows_path),
        "short_hash must normalize path separators"
    );

    // Test redact_path invariants
    let redacted = redact_path(input);

    // Invariant: Same input produces same redacted output (determinism)
    let redacted2 = redact_path(input);
    assert_eq!(redacted, redacted2, "redact_path must be deterministic");

    // Invariant: Separator normalization for redact_path
    assert_eq!(
        redact_path(&unix_path),
        redact_path(&windows_path),
        "redact_path must normalize path separators"
    );

    // Invariant: Hash portion (first 16 chars) matches short_hash output
    // (redact_path uses the normalized path for hashing)
    let normalized = input.replace('\\', "/");
    let expected_hash = short_hash(&normalized);
    assert!(
        redacted.starts_with(&expected_hash),
        "redact_path hash portion should match short_hash"
    );

    // Invariant: Extension handling
    // Get the extension from the normalized path using std::path::Path
    let ext = std::path::Path::new(&normalized)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    if ext.is_empty() {
        // No extension: redacted path is just the 16-char hash
        assert_eq!(
            redacted.len(),
            16,
            "redact_path without extension should be 16 chars"
        );
        assert!(
            !redacted.contains('.'),
            "redact_path without extension should have no dots"
        );
    } else {
        // Has extension: redacted path is hash + '.' + extension
        let expected_suffix = format!(".{}", ext);
        assert!(
            redacted.ends_with(&expected_suffix),
            "redact_path should preserve extension '.{}', got: {}",
            ext,
            redacted
        );
        assert_eq!(
            redacted.len(),
            16 + 1 + ext.len(),
            "redact_path length should be 16 + 1 + ext.len()"
        );
    }
});
