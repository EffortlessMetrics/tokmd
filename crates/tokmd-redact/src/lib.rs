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

/// Compute a short (16-character) BLAKE3 hash of a string.
///
/// This is used for redacting sensitive strings like excluded patterns
/// or module names in receipts.
///
/// # Example
///
/// ```
/// use tokmd_redact::short_hash;
///
/// let hash = short_hash("my-secret-path");
/// assert_eq!(hash.len(), 16);
/// ```
pub fn short_hash(s: &str) -> String {
    let mut hex = blake3::hash(s.as_bytes()).to_hex().to_string();
    hex.truncate(16);
    hex
}

/// Redact a path by hashing it while preserving the file extension.
///
/// This allows redacted paths to still be recognizable by file type
/// while hiding the actual path structure.
///
/// # Example
///
/// ```
/// use tokmd_redact::redact_path;
///
/// let redacted = redact_path("src/secrets/config.json");
/// assert!(redacted.ends_with(".json"));
/// assert_eq!(redacted.len(), 16 + 1 + 4); // hash + dot + "json"
/// ```
pub fn redact_path(path: &str) -> String {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let mut out = short_hash(path);
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
}
