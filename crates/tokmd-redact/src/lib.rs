//! # tokmd-redact
//!
//! **Tier 0.5 (Utilities)**
//!
//! This crate provides redaction utilities for `tokmd` receipts.
//! It's the canonical source for hashing functions used to redact sensitive
//! information (paths, patterns) in output while preserving useful structure.
//!
//! ## What belongs here
//! * Path redaction (hash while preserving extension)
//! * String hashing for redaction
//!
//! ## What does NOT belong here
//! * General-purpose file hashing (see `tokmd-content`)
//! * Integrity hashing (see `tokmd-analysis`)

use std::path::Path;

/// Clean a path by normalizing separators and resolving `.` and `./` segments.
///
/// This ensures that logically identical paths produce the same hash.
/// For example, `./src/lib.rs` and `src/lib.rs` will produce the same hash.
fn clean_path(s: &str) -> String {
    let mut normalized = s.replace('\\', "/");
    // Strip leading ./
    while let Some(stripped) = normalized.strip_prefix("./") {
        normalized = stripped.to_string();
    }
    // Remove interior /./
    while normalized.contains("/./") {
        normalized = normalized.replace("/./", "/");
    }
    // Remove trailing /.
    if normalized.ends_with("/.") {
        normalized.truncate(normalized.len() - 2);
    }
    normalized
}

/// Compute a short (16-character) BLAKE3 hash of a string.
///
/// This is used for redacting sensitive strings like excluded patterns
/// or module names in receipts.
///
/// Path separators are normalized to forward slashes before hashing
/// to ensure consistent hashes across operating systems. Redundant `.`
/// segments are also resolved so that logically identical paths hash
/// identically.
///
/// # Example
///
/// ```
/// use tokmd_redact::short_hash;
///
/// let hash = short_hash("my-secret-path");
/// assert_eq!(hash.len(), 16);
///
/// // Cross-platform consistency: same hash regardless of separator
/// assert_eq!(short_hash("src\\lib"), short_hash("src/lib"));
/// ```
pub fn short_hash(s: &str) -> String {
    let cleaned = clean_path(s);
    let mut hex = blake3::hash(cleaned.as_bytes()).to_hex().to_string();
    hex.truncate(16);
    hex
}

/// Redact a path by hashing it while preserving the file extension.
///
/// This allows redacted paths to still be recognizable by file type
/// while hiding the actual path structure.
///
/// Path separators are normalized to forward slashes before hashing
/// to ensure consistent hashes across operating systems.
///
/// # Example
///
/// ```
/// use tokmd_redact::redact_path;
///
/// let redacted = redact_path("src/secrets/config.json");
/// assert!(redacted.ends_with(".json"));
/// assert_eq!(redacted.len(), 16 + 1 + 4); // hash + dot + "json"
///
/// // Cross-platform consistency: same hash regardless of separator
/// assert_eq!(redact_path("src\\main.rs"), redact_path("src/main.rs"));
/// ```
pub fn redact_path(path: &str) -> String {
    let cleaned = clean_path(path);
    let ext = Path::new(&cleaned)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let mut out = short_hash(&cleaned);
    if !ext.is_empty() {
        out.push('.');
        out.push_str(ext);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_hash_length() {
        let hash = short_hash("test");
        assert_eq!(hash.len(), 16);
    }

    #[test]
    fn test_short_hash_deterministic() {
        let h1 = short_hash("same input");
        let h2 = short_hash("same input");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_short_hash_different_inputs() {
        let h1 = short_hash("input1");
        let h2 = short_hash("input2");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_redact_path_preserves_extension() {
        let redacted = redact_path("src/lib.rs");
        assert!(redacted.ends_with(".rs"));
    }

    #[test]
    fn test_redact_path_no_extension() {
        let redacted = redact_path("Makefile");
        assert_eq!(redacted.len(), 16);
        assert!(!redacted.contains('.'));
    }

    #[test]
    fn test_redact_path_double_extension() {
        // Only preserves final extension
        let redacted = redact_path("archive.tar.gz");
        assert!(redacted.ends_with(".gz"));
    }

    #[test]
    fn test_redact_path_deterministic() {
        let r1 = redact_path("src/main.rs");
        let r2 = redact_path("src/main.rs");
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_short_hash_normalizes_separators() {
        // Same logical path with different separators should hash identically
        let h1 = short_hash("src/lib");
        let h2 = short_hash("src\\lib");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_short_hash_normalizes_mixed_separators() {
        let h1 = short_hash("crates/foo/src/lib");
        let h2 = short_hash("crates\\foo\\src\\lib");
        let h3 = short_hash("crates/foo\\src/lib");
        assert_eq!(h1, h2);
        assert_eq!(h2, h3);
    }

    #[test]
    fn test_redact_path_normalizes_separators() {
        let r1 = redact_path("src/main.rs");
        let r2 = redact_path("src\\main.rs");
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_redact_path_normalizes_deep_paths() {
        let r1 = redact_path("crates/tokmd/src/commands/run.rs");
        let r2 = redact_path("crates\\tokmd\\src\\commands\\run.rs");
        assert_eq!(r1, r2);
        assert!(r1.ends_with(".rs"));
    }

    #[test]
    fn test_short_hash_normalizes_dot_prefix() {
        assert_eq!(short_hash("src/lib.rs"), short_hash("./src/lib.rs"));
    }

    #[test]
    fn test_short_hash_normalizes_interior_dot_segments() {
        assert_eq!(
            short_hash("crates/foo/./src/lib.rs"),
            short_hash("crates/foo/src/lib.rs")
        );
    }

    #[test]
    fn test_redact_path_normalizes_dot_prefix() {
        assert_eq!(redact_path("src/main.rs"), redact_path("./src/main.rs"));
    }

    // ========================
    // clean_path edge cases
    // ========================

    #[test]
    fn clean_path_empty_string() {
        assert_eq!(clean_path(""), "");
    }

    #[test]
    fn clean_path_multiple_dot_slash_prefix() {
        // Stripping repeated ./././
        assert_eq!(clean_path("./././src/lib.rs"), "src/lib.rs");
    }

    #[test]
    fn clean_path_interior_dot_segments() {
        assert_eq!(clean_path("a/./b/./c"), "a/b/c");
    }

    #[test]
    fn clean_path_trailing_dot() {
        assert_eq!(clean_path("src/."), "src");
    }

    #[test]
    fn clean_path_backslash_and_dot() {
        assert_eq!(clean_path(".\\src\\.\\lib.rs"), "src/lib.rs");
    }

    // ========================
    // Hash output invariants
    // ========================

    #[test]
    fn short_hash_is_hex_only() {
        let h = short_hash("anything");
        assert!(
            h.chars().all(|c| c.is_ascii_hexdigit()),
            "hash should only contain hex chars: {}",
            h
        );
    }

    #[test]
    fn short_hash_empty_input() {
        let h = short_hash("");
        assert_eq!(h.len(), 16);
        // Deterministic: empty string always produces the same hash
        assert_eq!(h, short_hash(""));
    }

    #[test]
    fn redact_path_hidden_dotfile() {
        let r = redact_path(".gitignore");
        // Rust's Path::extension() returns None for dotfiles like .gitignore
        assert_eq!(r.len(), 16);
        assert!(!r.contains('.'));
    }

    #[test]
    fn redact_path_deeply_nested() {
        let r = redact_path("a/b/c/d/e/f/g.txt");
        assert!(r.ends_with(".txt"));
        assert_eq!(r.len(), 16 + 4); // hash + ".txt"
    }
}
