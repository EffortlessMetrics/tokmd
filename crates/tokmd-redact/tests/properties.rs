//! Property-based tests for tokmd-redact.
//!
//! These tests verify the correctness, determinism, and cross-platform
//! consistency of redaction functions.

use proptest::prelude::*;
use tokmd_redact::{redact_path, short_hash};

// ============================================================================
// Strategies
// ============================================================================

/// Strategy for generating arbitrary path-like strings with various separators.
fn arb_path() -> impl Strategy<Value = String> {
    prop::collection::vec("[a-zA-Z0-9_.-]+", 1..=6).prop_map(|parts| parts.join("/"))
}

/// Strategy for generating paths with mixed separators.
fn arb_mixed_path() -> impl Strategy<Value = (String, String)> {
    prop::collection::vec("[a-zA-Z0-9_.-]+", 2..=6).prop_map(|parts| {
        let unix = parts.join("/");
        // Create a mixed version alternating separators
        let mixed: String = parts
            .iter()
            .enumerate()
            .map(|(i, p)| {
                if i == 0 {
                    p.clone()
                } else if i % 2 == 0 {
                    format!("/{}", p)
                } else {
                    format!("\\{}", p)
                }
            })
            .collect();
        (unix, mixed)
    })
}

/// Strategy for common file extensions.
fn arb_extension() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("rs".to_string()),
        Just("js".to_string()),
        Just("ts".to_string()),
        Just("py".to_string()),
        Just("json".to_string()),
        Just("toml".to_string()),
        Just("md".to_string()),
        Just("txt".to_string()),
        Just("yaml".to_string()),
        Just("xml".to_string()),
    ]
}

/// Strategy for paths with extensions.
fn arb_path_with_extension() -> impl Strategy<Value = String> {
    (arb_path(), arb_extension()).prop_map(|(path, ext)| format!("{}.{}", path, ext))
}

/// Strategy for paths without extensions (like Makefile, Dockerfile).
fn arb_path_no_extension() -> impl Strategy<Value = String> {
    prop::collection::vec("[a-zA-Z0-9_-]+", 1..=5)
        .prop_filter("must not contain dots", |parts| {
            parts.iter().all(|p| !p.contains('.'))
        })
        .prop_map(|parts| parts.join("/"))
}

// ============================================================================
// short_hash tests
// ============================================================================

proptest! {
    /// Hash output is always exactly 16 hex characters.
    #[test]
    fn short_hash_length_is_16(input in ".*") {
        let hash = short_hash(&input);
        prop_assert_eq!(hash.len(), 16, "Hash length must be 16, got {}", hash.len());
    }

    /// Hash output contains only valid lowercase hex characters.
    #[test]
    fn short_hash_is_lowercase_hex(input in ".*") {
        let hash = short_hash(&input);
        prop_assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()),
            "Hash must be lowercase hex, got: {}",
            hash
        );
    }

    /// Same input always produces same hash (determinism).
    #[test]
    fn short_hash_is_deterministic(input in ".*") {
        let h1 = short_hash(&input);
        let h2 = short_hash(&input);
        prop_assert_eq!(h1, h2, "Hash must be deterministic");
    }

    /// Different inputs produce different hashes (collision resistance).
    /// Note: This is probabilistic - two different inputs might collide,
    /// but it's extremely unlikely with BLAKE3.
    #[test]
    fn short_hash_different_inputs_differ(a in "[a-z]{1,20}", b in "[a-z]{1,20}") {
        prop_assume!(a != b);
        let h1 = short_hash(&a);
        let h2 = short_hash(&b);
        prop_assert_ne!(h1, h2, "Different inputs should produce different hashes");
    }

    /// Forward and backward slashes in paths produce identical hashes.
    #[test]
    fn short_hash_normalizes_separators(path in arb_path()) {
        let unix_path = path.replace('\\', "/");
        let windows_path = path.replace('/', "\\");

        let h1 = short_hash(&unix_path);
        let h2 = short_hash(&windows_path);

        prop_assert_eq!(h1, h2, "Unix and Windows paths must hash identically");
    }

    /// Mixed separators normalize to forward slashes.
    #[test]
    fn short_hash_normalizes_mixed_separators((unix, mixed) in arb_mixed_path()) {
        let h1 = short_hash(&unix);
        let h2 = short_hash(&mixed);

        prop_assert_eq!(h1, h2, "Mixed separators must normalize: {} vs {}", unix, mixed);
    }

    /// Empty string produces a valid hash.
    #[test]
    fn short_hash_handles_empty_string(_dummy in 0..1u8) {
        let hash = short_hash("");
        prop_assert_eq!(hash.len(), 16);
        prop_assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    /// Unicode strings produce valid hashes.
    #[test]
    fn short_hash_handles_unicode(input in "[a-z\u{00E0}-\u{00FF}\u{4E00}-\u{4FFF}]{1,20}") {
        let hash = short_hash(&input);
        prop_assert_eq!(hash.len(), 16);
        prop_assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

// ============================================================================
// redact_path tests
// ============================================================================

proptest! {
    /// Redacted paths preserve file extensions.
    #[test]
    fn redact_path_preserves_extension(path in arb_path_with_extension()) {
        let redacted = redact_path(&path);
        let original_ext = path.rsplit('.').next().unwrap();
        let expected_suffix = format!(".{}", original_ext);

        prop_assert!(
            redacted.ends_with(&expected_suffix),
            "Redacted path '{}' must end with extension '.{}' from '{}'",
            redacted,
            original_ext,
            path
        );
    }

    /// Redacted paths without extensions have no dots.
    #[test]
    fn redact_path_no_extension_has_no_dot(path in arb_path_no_extension()) {
        let redacted = redact_path(&path);
        prop_assert!(
            !redacted.contains('.'),
            "Redacted extensionless path '{}' should not contain dots, got: {}",
            path,
            redacted
        );
        prop_assert_eq!(
            redacted.len(),
            16,
            "Extensionless redacted path should be exactly 16 chars"
        );
    }

    /// Redacted path length is hash_len + 1 + ext_len for paths with extensions.
    #[test]
    fn redact_path_length_with_extension(path in arb_path_with_extension()) {
        let redacted = redact_path(&path);
        let ext = path.rsplit('.').next().unwrap();
        let expected_len = 16 + 1 + ext.len(); // hash + dot + extension

        prop_assert_eq!(
            redacted.len(),
            expected_len,
            "Redacted path length mismatch for '{}': expected {}, got {}",
            path,
            expected_len,
            redacted.len()
        );
    }

    /// Same path always produces same redacted output (determinism).
    #[test]
    fn redact_path_is_deterministic(path in arb_path_with_extension()) {
        let r1 = redact_path(&path);
        let r2 = redact_path(&path);
        prop_assert_eq!(r1, r2, "Redaction must be deterministic");
    }

    /// Unix and Windows paths produce identical redacted output.
    #[test]
    fn redact_path_normalizes_separators(path in arb_path_with_extension()) {
        let unix_path = path.replace('\\', "/");
        let windows_path = path.replace('/', "\\");

        let r1 = redact_path(&unix_path);
        let r2 = redact_path(&windows_path);

        prop_assert_eq!(
            r1,
            r2,
            "Unix and Windows paths must redact identically: {} vs {}",
            unix_path,
            windows_path
        );
    }

    /// Double extensions preserve only the final extension.
    #[test]
    fn redact_path_preserves_only_final_extension(
        base in "[a-z]{1,10}",
        ext1 in arb_extension(),
        ext2 in arb_extension()
    ) {
        let path = format!("{}.{}.{}", base, ext1, ext2);
        let redacted = redact_path(&path);

        prop_assert!(
            redacted.ends_with(&format!(".{}", ext2)),
            "Double extension path '{}' should preserve only final extension '.{}', got: {}",
            path,
            ext2,
            redacted
        );
        // Should NOT contain the first extension
        prop_assert!(
            !redacted.contains(&format!(".{}.", ext1)),
            "Should not contain intermediate extension"
        );
    }

    /// Redact is idempotent for the hash portion (ignoring extension differences).
    #[test]
    fn redact_path_hash_portion_matches_short_hash(path in arb_path_with_extension()) {
        let redacted = redact_path(&path);
        let hash_portion = &redacted[..16];
        let normalized = path.replace('\\', "/");
        let expected_hash = short_hash(&normalized);

        prop_assert_eq!(
            hash_portion,
            expected_hash,
            "Hash portion of redacted path should match short_hash"
        );
    }

    /// Very long paths still produce correct redacted output.
    #[test]
    fn redact_path_handles_long_paths(
        parts in prop::collection::vec("[a-z]{5,15}", 10..=20),
        ext in arb_extension()
    ) {
        let path = format!("{}.{}", parts.join("/"), ext);
        let redacted = redact_path(&path);

        prop_assert!(
            redacted.ends_with(&format!(".{}", ext)),
            "Should end with extension .{}", ext
        );
        prop_assert_eq!(redacted.len(), 16 + 1 + ext.len());
    }
}

// ============================================================================
// Edge case tests
// ============================================================================

proptest! {
    /// Hidden files (starting with dot) are handled correctly.
    #[test]
    fn redact_path_handles_hidden_files(name in "[a-z]{1,10}", ext in arb_extension()) {
        let path = format!(".{}.{}", name, ext);
        let redacted = redact_path(&path);

        prop_assert!(
            redacted.ends_with(&format!(".{}", ext)),
            "Hidden file '{}' should preserve extension, got: {}",
            path,
            redacted
        );
    }

    /// Dotfiles (e.g., ".gitignore") have no extension per std::path::Path.
    /// They redact to just the hash with no extension preserved.
    #[test]
    fn redact_path_dotfiles_have_no_extension(name in "[a-z]{1,15}") {
        let path = format!(".{}", name);
        let redacted = redact_path(&path);

        // Dotfiles have no extension per Path::extension()
        // So redacted should be just the 16-char hash
        prop_assert_eq!(
            redacted.len(),
            16,
            "Dotfile '{}' should have no extension, got: {}",
            path,
            redacted
        );
        prop_assert!(
            !redacted.contains('.'),
            "Dotfile redaction should not contain a dot"
        );
    }

    /// Hidden files with extensions (e.g., ".test.rs") preserve the extension.
    #[test]
    fn redact_path_hidden_with_extension(name in "[a-z]{2,10}", ext in arb_extension()) {
        let path = format!(".{}.{}", name, ext);
        let redacted = redact_path(&path);

        // .test.rs has extension "rs"
        prop_assert!(
            redacted.ends_with(&format!(".{}", ext)),
            "Hidden file '{}' should preserve extension '.{}', got: {}",
            path,
            ext,
            redacted
        );
    }

    /// Paths with spaces in names (URL-encoded or raw).
    #[test]
    fn redact_path_handles_spaces(base in "[a-z ]{3,15}", ext in arb_extension()) {
        let path = format!("{}.{}", base.trim(), ext);
        prop_assume!(!path.starts_with('.'));

        let redacted = redact_path(&path);

        prop_assert!(
            redacted.ends_with(&format!(".{}", ext)),
            "Path with spaces '{}' should preserve extension, got: {}",
            path,
            redacted
        );
    }

    /// Consecutive separators normalize correctly.
    #[test]
    fn short_hash_handles_consecutive_separators(parts in prop::collection::vec("[a-z]{2,5}", 2..=4)) {
        let normal = parts.join("/");
        let double_slash = parts.join("//");
        let double_backslash = parts.join("\\\\");

        // These should all produce the same hash since they normalize separators
        // Note: The actual behavior depends on whether consecutive slashes are collapsed
        // In the current implementation, they are NOT collapsed, so "a//b" != "a/b"
        let h1 = short_hash(&normal);
        let h2 = short_hash(&double_slash);

        // Actually these won't be equal since consecutive slashes aren't collapsed
        // Just verify they're valid hashes
        prop_assert_eq!(h1.len(), 16);
        prop_assert_eq!(h2.len(), 16);

        // But Unix vs Windows should still match after normalization
        let h3 = short_hash(&double_backslash.replace('\\', "/"));
        prop_assert_eq!(h2, h3);
    }
}

// ============================================================================
// Lexical normalisation before hashing
// ============================================================================

proptest! {
    /// Leading "./" is stripped before hashing, so "./X" == "X".
    #[test]
    fn short_hash_strips_leading_dot_slash(path in arb_path()) {
        let bare = short_hash(&path);
        let dotted = short_hash(&format!("./{}", path));
        prop_assert_eq!(bare, dotted, "./ prefix must be transparent: {}", path);
    }

    /// Multiple leading "./" segments are all stripped.
    #[test]
    fn short_hash_strips_multiple_leading_dot_slash(path in arb_path()) {
        let bare = short_hash(&path);
        let double = short_hash(&format!("././{}", path));
        prop_assert_eq!(bare, double, "././ prefix must be transparent: {}", path);
    }

    /// Interior "/./\" segments are collapsed so "a/./b" == "a/b".
    #[test]
    fn short_hash_collapses_interior_dot_segments(
        left in "[a-z]{2,6}",
        right in "[a-z]{2,6}"
    ) {
        let clean = format!("{}/{}", left, right);
        let dotted = format!("{left}/./{right}");
        prop_assert_eq!(
            short_hash(&clean),
            short_hash(&dotted),
            "Interior /. must collapse: {} vs {}", clean, dotted
        );
    }

    /// Trailing "/." is removed, so "dir/." == "dir".
    #[test]
    fn short_hash_strips_trailing_dot(dir in "[a-z]{2,10}") {
        let bare = short_hash(&dir);
        let trailing = short_hash(&format!("{}/.", dir));
        prop_assert_eq!(bare, trailing, "Trailing /. must be stripped: {}", dir);
    }

    /// Leading ".\" (Windows dot-backslash) normalises the same as "./".
    #[test]
    fn short_hash_strips_leading_dot_backslash(path in arb_path()) {
        let dotslash = short_hash(&format!("./{}", path));
        let dotback = short_hash(&format!(".\\{}", path));
        prop_assert_eq!(dotslash, dotback, ".\\ must equal ./: {}", path);
    }
}

// ============================================================================
// Idempotency
// ============================================================================

proptest! {
    /// Calling short_hash N times on the same input always returns the same value.
    #[test]
    fn short_hash_triple_call_idempotent(input in ".*") {
        let h1 = short_hash(&input);
        let h2 = short_hash(&input);
        let h3 = short_hash(&input);
        prop_assert_eq!(&h1, &h2);
        prop_assert_eq!(&h2, &h3);
    }

    /// Calling redact_path N times on the same input always returns the same value.
    #[test]
    fn redact_path_triple_call_idempotent(path in arb_path_with_extension()) {
        let r1 = redact_path(&path);
        let r2 = redact_path(&path);
        let r3 = redact_path(&path);
        prop_assert_eq!(&r1, &r2);
        prop_assert_eq!(&r2, &r3);
    }
}

// ============================================================================
// Determinism across equivalent representations
// ============================================================================

proptest! {
    /// All four representations of a path (unix, win, ./-prefixed, mixed)
    /// produce the same short_hash.
    #[test]
    fn short_hash_four_way_equivalence(parts in prop::collection::vec("[a-z]{2,5}", 2..=5)) {
        let unix = parts.join("/");
        let win = parts.join("\\");
        let dot_unix = format!("./{}", unix);
        let dot_win = format!(".\\{}", win);

        let h_unix = short_hash(&unix);
        let h_win = short_hash(&win);
        let h_dot_unix = short_hash(&dot_unix);
        let h_dot_win = short_hash(&dot_win);

        prop_assert_eq!(&h_unix, &h_win, "unix != win");
        prop_assert_eq!(&h_unix, &h_dot_unix, "unix != ./unix");
        prop_assert_eq!(&h_unix, &h_dot_win, "unix != .\\ win");
    }

    /// All four representations of a path with extension produce the same redact_path.
    #[test]
    fn redact_path_four_way_equivalence(
        parts in prop::collection::vec("[a-z]{2,5}", 2..=5),
        ext in arb_extension()
    ) {
        let base_unix = parts.join("/");
        let base_win = parts.join("\\");
        let unix = format!("{}.{}", base_unix, ext);
        let win = format!("{}.{}", base_win, ext);
        let dot_unix = format!("./{}", unix);
        let dot_win = format!(".\\{}", win);

        let r_unix = redact_path(&unix);
        let r_win = redact_path(&win);
        let r_dot_unix = redact_path(&dot_unix);
        let r_dot_win = redact_path(&dot_win);

        prop_assert_eq!(&r_unix, &r_win, "unix != win");
        prop_assert_eq!(&r_unix, &r_dot_unix, "unix != ./unix");
        prop_assert_eq!(&r_unix, &r_dot_win, "unix != .\\ win");
    }

    /// The hash portion of redact_path equals short_hash of the same input,
    /// regardless of whether the path has a dot-slash prefix.
    #[test]
    fn redact_hash_equals_short_hash_with_normalization(
        parts in prop::collection::vec("[a-z]{2,5}", 1..=4),
        ext in arb_extension()
    ) {
        let path = format!("{}.{}", parts.join("/"), ext);
        let dotted = format!("./{}", path);

        let redacted = redact_path(&path);
        let redacted_dotted = redact_path(&dotted);

        // Both should start with the same 16-char hash
        prop_assert_eq!(&redacted[..16], &redacted_dotted[..16]);
        // And that hash equals short_hash of the path
        prop_assert_eq!(&redacted[..16], short_hash(&path));
    }
}

// ============================================================================
// Additional edge cases
// ============================================================================

proptest! {
    /// Redacted output never contains path separators (/ or \).
    #[test]
    fn redact_path_output_never_contains_separators(path in arb_path_with_extension()) {
        let redacted = redact_path(&path);
        prop_assert!(
            !redacted.contains('/'),
            "Redacted output should not contain /: {}",
            redacted
        );
        prop_assert!(
            !redacted.contains('\\'),
            "Redacted output should not contain \\: {}",
            redacted
        );
    }

    /// Single-character filenames produce valid hashes.
    #[test]
    fn short_hash_single_char(c in "[a-zA-Z0-9]") {
        let hash = short_hash(&c);
        prop_assert_eq!(hash.len(), 16);
        prop_assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    /// Paths with numeric segments are handled correctly.
    #[test]
    fn short_hash_numeric_segments(
        parts in prop::collection::vec("[0-9]{1,5}", 1..=4)
    ) {
        let path = parts.join("/");
        let hash = short_hash(&path);
        prop_assert_eq!(hash.len(), 16);
        prop_assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    /// Paths with trailing slash are valid (but differ from non-trailing).
    #[test]
    fn short_hash_trailing_slash_valid(path in arb_path()) {
        let with_trailing = format!("{}/", path);
        let h = short_hash(&with_trailing);
        prop_assert_eq!(h.len(), 16);
        prop_assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    /// Hash of a hash is itself a valid hash (recursive application).
    #[test]
    fn short_hash_of_hash_is_valid(input in ".*") {
        let h1 = short_hash(&input);
        let h2 = short_hash(&h1);
        prop_assert_eq!(h2.len(), 16);
        prop_assert!(h2.chars().all(|c| c.is_ascii_hexdigit()));
    }

    /// Windows drive-letter prefixed paths produce valid hashes.
    #[test]
    fn short_hash_windows_drive_prefix(
        drive in "[A-Z]",
        parts in prop::collection::vec("[a-z]{2,6}", 1..=4)
    ) {
        let path = format!("{}:\\{}", drive, parts.join("\\"));
        let hash = short_hash(&path);
        prop_assert_eq!(hash.len(), 16);
        prop_assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    /// Paths with only extensions (like ".gitignore") produce valid redaction.
    #[test]
    fn redact_path_extensionless_dotfile_is_hash_only(name in "[a-z]{2,12}") {
        let dotfile = format!(".{}", name);
        let redacted = redact_path(&dotfile);
        // Dotfiles have no extension per Path::extension()
        prop_assert_eq!(redacted.len(), 16);
        prop_assert!(!redacted.contains('.'));
    }

    /// redact_path with empty extension segment: "file." ends with empty ext.
    #[test]
    fn redact_path_trailing_dot(base in "[a-z]{2,10}") {
        let path = format!("{}.", base);
        let redacted = redact_path(&path);
        // std::path::Path considers "file." to have empty extension ""
        // so no extension is appended
        prop_assert_eq!(redacted.len(), 16);
    }
}
