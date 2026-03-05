//! Fuzz-like property tests for tokmd-redact.
//!
//! These tests exercise hashing and redaction functions with large random
//! input spaces to ensure no panics and that structural invariants hold.

use proptest::prelude::*;
use tokmd_redact::{redact_path, short_hash};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_arbitrary_bytes_as_str() -> impl Strategy<Value = String> {
    prop::string::string_regex(".{0,200}").unwrap()
}

fn arb_path_like() -> impl Strategy<Value = String> {
    prop::collection::vec("[^\0]{0,30}", 1..=8).prop_map(|parts| parts.join("/"))
}

fn arb_path_with_mixed_seps() -> impl Strategy<Value = String> {
    prop::collection::vec("[a-zA-Z0-9_.\\-]{1,15}", 1..=6).prop_map(|parts| {
        parts
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
            .collect::<String>()
    })
}

// ---------------------------------------------------------------------------
// 1. short_hash never panics on any input
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn fuzz_short_hash_never_panics(input in arb_arbitrary_bytes_as_str()) {
        let _ = short_hash(&input);
    }

    #[test]
    fn fuzz_short_hash_never_panics_pathlike(input in arb_path_like()) {
        let _ = short_hash(&input);
    }
}

// ---------------------------------------------------------------------------
// 2. Hash output is always exactly 16 hex characters
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn fuzz_short_hash_length_always_16(input in arb_arbitrary_bytes_as_str()) {
        let hash = short_hash(&input);
        prop_assert_eq!(hash.len(), 16, "Expected 16 chars, got {} for input {:?}", hash.len(), input);
    }

    #[test]
    fn fuzz_short_hash_all_hex(input in arb_arbitrary_bytes_as_str()) {
        let hash = short_hash(&input);
        prop_assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash contains non-hex chars: {:?}",
            hash
        );
    }
}

// ---------------------------------------------------------------------------
// 3. redact_path never panics on any input
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn fuzz_redact_path_never_panics(input in arb_arbitrary_bytes_as_str()) {
        let _ = redact_path(&input);
    }

    #[test]
    fn fuzz_redact_path_never_panics_pathlike(input in arb_path_like()) {
        let _ = redact_path(&input);
    }
}

// ---------------------------------------------------------------------------
// 4. Redacted path length invariants
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn fuzz_redact_path_min_length_16(input in arb_arbitrary_bytes_as_str()) {
        let redacted = redact_path(&input);
        prop_assert!(
            redacted.len() >= 16,
            "Redacted path too short: {} (len={}) for input {:?}",
            redacted,
            redacted.len(),
            input
        );
    }

    #[test]
    fn fuzz_redact_path_preserves_extension(
        stem in "[a-zA-Z0-9_]{1,30}",
        ext in "[a-zA-Z]{1,8}",
    ) {
        let path = format!("{}.{}", stem, ext);
        let redacted = redact_path(&path);
        prop_assert!(
            redacted.ends_with(&format!(".{}", ext)),
            "Expected extension .{} in {:?} for path {:?}",
            ext,
            redacted,
            path
        );
    }
}

// ---------------------------------------------------------------------------
// 5. Determinism: same input always produces same output
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn fuzz_short_hash_deterministic(input in arb_arbitrary_bytes_as_str()) {
        prop_assert_eq!(short_hash(&input), short_hash(&input));
    }

    #[test]
    fn fuzz_redact_path_deterministic(input in arb_arbitrary_bytes_as_str()) {
        prop_assert_eq!(redact_path(&input), redact_path(&input));
    }
}

// ---------------------------------------------------------------------------
// 6. Cross-platform normalization: backslash == forward slash
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn fuzz_separator_normalization(path in arb_path_with_mixed_seps()) {
        let canonical = path.replace('\\', "/");
        prop_assert_eq!(
            short_hash(&path),
            short_hash(&canonical),
            "Hash differs for {:?} vs {:?}",
            path,
            canonical
        );
    }
}
