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
/// Path separators are normalized to forward slashes before hashing
/// to ensure consistent hashes across operating systems.
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
    let normalized = s.replace('\\', "/");
    let mut hex = blake3::hash(normalized.as_bytes()).to_hex().to_string();
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
    let normalized = path.replace('\\', "/");
    let ext = Path::new(&normalized)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let mut out = short_hash(&normalized);
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
}
