//! Wave 42 property-based tests for tokmd-redact.
//!
//! Covers: non-emptiness, determinism, collision resistance, extension
//! preservation, cross-platform normalization, and idempotency.

use proptest::prelude::*;
use tokmd_redact::{redact_path, short_hash};

// ============================================================================
// Strategies
// ============================================================================

fn arb_path() -> impl Strategy<Value = String> {
    prop::collection::vec("[a-zA-Z0-9_.-]+", 1..=6).prop_map(|parts| parts.join("/"))
}

fn arb_extension() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("rs".to_string()),
        Just("py".to_string()),
        Just("go".to_string()),
        Just("js".to_string()),
        Just("ts".to_string()),
        Just("json".to_string()),
        Just("toml".to_string()),
        Just("md".to_string()),
    ]
}

fn arb_path_with_ext() -> impl Strategy<Value = String> {
    (arb_path(), arb_extension()).prop_map(|(p, ext)| format!("{}.{}", p, ext))
}

fn arb_extensionless_path() -> impl Strategy<Value = String> {
    prop::collection::vec("[a-zA-Z0-9_-]+", 1..=5)
        .prop_filter("no dots", |parts| parts.iter().all(|p| !p.contains('.')))
        .prop_map(|parts| parts.join("/"))
}

// ============================================================================
// short_hash: non-emptiness and format
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// Redacted paths are never empty.
    #[test]
    fn redact_path_never_empty(path in arb_path_with_ext()) {
        let r = redact_path(&path);
        prop_assert!(!r.is_empty(), "Redacted path must not be empty");
    }

    /// short_hash always returns exactly 16 lowercase hex chars.
    #[test]
    fn short_hash_always_16_hex(input in ".*") {
        let h = short_hash(&input);
        prop_assert_eq!(h.len(), 16);
        prop_assert!(h.chars().all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()),
            "Not lowercase hex: {}", h);
    }

    /// short_hash output is never empty.
    #[test]
    fn short_hash_never_empty(input in ".*") {
        prop_assert!(!short_hash(&input).is_empty());
    }
}

// ============================================================================
// Determinism: same input → same output
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// short_hash is deterministic.
    #[test]
    fn short_hash_deterministic(input in ".*") {
        prop_assert_eq!(short_hash(&input), short_hash(&input));
    }

    /// redact_path is deterministic.
    #[test]
    fn redact_path_deterministic(path in arb_path_with_ext()) {
        prop_assert_eq!(redact_path(&path), redact_path(&path));
    }

    /// Triple invocation returns same result.
    #[test]
    fn short_hash_triple_stable(input in "\\PC{0,50}") {
        let h1 = short_hash(&input);
        let h2 = short_hash(&input);
        let h3 = short_hash(&input);
        prop_assert_eq!(&h1, &h2);
        prop_assert_eq!(&h2, &h3);
    }
}

// ============================================================================
// Collision resistance: different inputs → different outputs
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// Different short strings produce different hashes.
    #[test]
    fn different_inputs_different_hashes(a in "[a-z]{1,20}", b in "[a-z]{1,20}") {
        prop_assume!(a != b);
        prop_assert_ne!(short_hash(&a), short_hash(&b),
            "Collision: '{}' and '{}' hash the same", a, b);
    }

    /// Different paths produce different redacted outputs.
    #[test]
    fn different_paths_different_redactions(
        a in "[a-z]{2,10}/[a-z]{2,10}\\.[a-z]{1,4}",
        b in "[a-z]{2,10}/[a-z]{2,10}\\.[a-z]{1,4}"
    ) {
        prop_assume!(a != b);
        // The redacted outputs may still collide if only the extension differs
        // and the base path is the same, but for truly different paths this holds.
        let ra = redact_path(&a);
        let rb = redact_path(&b);
        // At minimum, the hash portion should differ
        prop_assert_ne!(&ra[..16], &rb[..16],
            "Hash collision for '{}' and '{}'", a, b);
    }
}

// ============================================================================
// Extension preservation
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    /// redact_path preserves the final file extension.
    #[test]
    fn redact_preserves_extension(path in arb_path_with_ext()) {
        let ext = path.rsplit('.').next().unwrap();
        let r = redact_path(&path);
        prop_assert!(r.ends_with(&format!(".{}", ext)),
            "'{}' should end with '.{}', got '{}'", path, ext, r);
    }

    /// Extensionless paths redact to exactly 16 chars with no dot.
    #[test]
    fn redact_extensionless_no_dot(path in arb_extensionless_path()) {
        let r = redact_path(&path);
        prop_assert_eq!(r.len(), 16);
        prop_assert!(!r.contains('.'), "Extensionless '{}' → '{}' contains dot", path, r);
    }

    /// Redacted path length = 16 + 1 + ext.len() for paths with extensions.
    #[test]
    fn redact_length_matches(path in arb_path_with_ext()) {
        let ext = path.rsplit('.').next().unwrap();
        let r = redact_path(&path);
        prop_assert_eq!(r.len(), 16 + 1 + ext.len(),
            "Length mismatch for '{}': got {}", path, r.len());
    }
}

// ============================================================================
// Cross-platform normalization
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    /// Unix and Windows paths produce identical hashes.
    #[test]
    fn hash_cross_platform(path in arb_path()) {
        let unix = path.replace('\\', "/");
        let win = path.replace('/', "\\");
        prop_assert_eq!(short_hash(&unix), short_hash(&win),
            "Cross-platform mismatch: {} vs {}", unix, win);
    }

    /// Unix and Windows paths produce identical redacted output.
    #[test]
    fn redact_cross_platform(path in arb_path_with_ext()) {
        let unix = path.replace('\\', "/");
        let win = path.replace('/', "\\");
        prop_assert_eq!(redact_path(&unix), redact_path(&win));
    }

    /// Leading "./" is transparent to hashing.
    #[test]
    fn hash_leading_dot_slash_transparent(path in arb_path()) {
        prop_assert_eq!(short_hash(&path), short_hash(&format!("./{}", path)));
    }
}
