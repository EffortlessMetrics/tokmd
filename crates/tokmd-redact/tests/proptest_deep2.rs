//! Additional deep property-based tests for tokmd-redact.
//!
//! Covers: batch uniqueness, prefix independence, case sensitivity,
//! extension-only files, and composition properties.

use proptest::prelude::*;
use tokmd_redact::{redact_path, short_hash};

// =========================================================================
// Strategies
// =========================================================================

fn arb_extension() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("rs".to_string()),
        Just("py".to_string()),
        Just("go".to_string()),
        Just("js".to_string()),
        Just("ts".to_string()),
        Just("json".to_string()),
        Just("toml".to_string()),
    ]
}

// =========================================================================
// Batch uniqueness: N distinct paths produce N distinct hashes
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn batch_distinct_paths_distinct_hashes(
        paths in prop::collection::hash_set("[a-z]{3,12}/[a-z]{3,8}", 2..10)
    ) {
        let hashes: std::collections::BTreeSet<String> =
            paths.iter().map(|p| short_hash(p)).collect();
        prop_assert_eq!(
            hashes.len(), paths.len(),
            "Expected {} unique hashes for {} unique paths, got {}",
            paths.len(), paths.len(), hashes.len()
        );
    }

    #[test]
    fn batch_distinct_paths_distinct_redactions(
        paths in prop::collection::hash_set("[a-z]{3,8}/[a-z]{3,8}\\.rs", 2..8)
    ) {
        let redacted: std::collections::BTreeSet<String> =
            paths.iter().map(|p| redact_path(p)).collect();
        prop_assert_eq!(
            redacted.len(), paths.len(),
            "Expected {} unique redactions for {} unique paths",
            paths.len(), paths.len()
        );
    }
}

// =========================================================================
// Prefix independence: adding a directory prefix changes the hash
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn different_prefix_different_hash(
        base in "[a-z]{3,8}/[a-z]{3,8}",
        prefix in "[a-z]{3,8}",
    ) {
        let prefixed = format!("{}/{}", prefix, base);
        let h_base = short_hash(&base);
        let h_prefixed = short_hash(&prefixed);
        prop_assert_ne!(
            h_base, h_prefixed,
            "Prefix should change hash: '{}' vs '{}'", base, prefixed
        );
    }
}

// =========================================================================
// Case sensitivity: differing case produces different hashes
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn case_sensitivity_in_hashing(input in "[a-z]{4,15}") {
        let upper: String = input.to_uppercase();
        prop_assume!(input != upper);
        let h_lower = short_hash(&input);
        let h_upper = short_hash(&upper);
        prop_assert_ne!(
            h_lower, h_upper,
            "Case should affect hash: '{}' vs '{}'", input, upper
        );
    }
}

// =========================================================================
// Extension handling edge cases
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn same_stem_different_ext_different_hash(
        stem in "[a-z]{3,10}",
        ext1 in arb_extension(),
        ext2 in arb_extension(),
    ) {
        prop_assume!(ext1 != ext2);
        let path1 = format!("{}.{}", stem, ext1);
        let path2 = format!("{}.{}", stem, ext2);
        let h1 = short_hash(&path1);
        let h2 = short_hash(&path2);
        prop_assert_ne!(h1, h2, "Different extensions should give different hashes");
    }

    #[test]
    fn redact_path_same_ext_same_length(
        stem1 in "[a-z]{3,10}",
        stem2 in "[a-z]{3,10}",
        ext in arb_extension(),
    ) {
        let path1 = format!("{}.{}", stem1, ext);
        let path2 = format!("{}.{}", stem2, ext);
        let r1 = redact_path(&path1);
        let r2 = redact_path(&path2);
        prop_assert_eq!(r1.len(), r2.len(),
            "Same extension should give same redacted length");
    }

    #[test]
    fn redact_path_different_ext_different_suffix(
        stem in "[a-z]{3,10}",
        ext1 in arb_extension(),
        ext2 in arb_extension(),
    ) {
        prop_assume!(ext1 != ext2);
        let path1 = format!("{}.{}", stem, ext1);
        let path2 = format!("{}.{}", stem, ext2);
        let r1 = redact_path(&path1);
        let r2 = redact_path(&path2);
        let expected1 = format!(".{}", ext1);
        let expected2 = format!(".{}", ext2);
        prop_assert!(r1.ends_with(&expected1));
        prop_assert!(r2.ends_with(&expected2));
    }
}

// =========================================================================
// Hash of concatenation differs from hash of parts
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn hash_of_concat_differs_from_parts(
        a in "[a-z]{3,10}",
        b in "[a-z]{3,10}",
    ) {
        let combined = format!("{}{}", a, b);
        let h_combined = short_hash(&combined);
        let h_a = short_hash(&a);
        let h_b = short_hash(&b);
        // The hash of the concatenation should differ from either part's hash
        prop_assert_ne!(h_combined.clone(), h_a);
        prop_assert_ne!(h_combined, h_b);
    }
}

// =========================================================================
// Redact output is purely ASCII hex + extension
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn redact_path_ascii_hex_hash_portion(
        parts in prop::collection::vec("[a-z]{2,6}", 1..5),
        ext in arb_extension(),
    ) {
        let path = format!("{}.{}", parts.join("/"), ext);
        let redacted = redact_path(&path);
        let hash_part = &redacted[..16];
        prop_assert!(
            hash_part.chars().all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()),
            "Hash portion '{}' should be lowercase hex", hash_part
        );
    }

    #[test]
    fn short_hash_never_empty(input in ".*") {
        let hash = short_hash(&input);
        prop_assert!(!hash.is_empty());
        prop_assert_eq!(hash.len(), 16);
    }
}
