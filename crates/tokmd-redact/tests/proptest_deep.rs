//! Deep property-based tests for tokmd-redact.
//!
//! Covers leak detection, collision resistance, hash consistency,
//! pure function verification, and path normalization equivalence.

use proptest::prelude::*;
use tokmd_redact::{redact_path, short_hash};

// =========================================================================
// Strategies
// =========================================================================

fn arb_path_with_ext() -> impl Strategy<Value = String> {
    (
        prop::collection::vec("[a-zA-Z0-9_-]+", 1..=5),
        prop::sample::select(vec!["rs", "py", "go", "js", "toml", "md", ""]),
    )
        .prop_map(|(parts, ext)| {
            let base = parts.join("/");
            if ext.is_empty() {
                base
            } else {
                format!("{}.{}", base, ext)
            }
        })
}

fn arb_directory_segment() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_-]{2,19}"
}

// =========================================================================
// Redacted output never leaks original directory segments
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn redacted_output_never_contains_original_directory(
        segments in prop::collection::vec(arb_directory_segment(), 2..=5),
        ext in prop::sample::select(vec!["rs", "py", "go", "js"]),
    ) {
        let path = format!("{}.{}", segments.join("/"), ext);
        let redacted = redact_path(&path);

        // No original directory segment should appear in the redacted output
        for seg in &segments {
            prop_assert!(
                !redacted.contains(seg.as_str()),
                "Redacted output '{}' should not contain segment '{}'",
                redacted, seg
            );
        }
    }
}

// =========================================================================
// Redacted output is hex characters + optional extension
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn redacted_output_is_hex_plus_extension(path in arb_path_with_ext()) {
        let redacted = redact_path(&path);

        // Split by last dot to separate hash from extension
        if let Some(dot_pos) = redacted.rfind('.') {
            let hash_part = &redacted[..dot_pos];
            prop_assert!(
                hash_part.chars().all(|c| c.is_ascii_hexdigit()),
                "Hash part '{}' should be all hex digits",
                hash_part
            );
        }
        // Without extension: entire output should be hex
        else {
            prop_assert!(
                redacted.chars().all(|c| c.is_ascii_hexdigit()),
                "Redacted '{}' should be all hex digits",
                redacted
            );
        }
    }
}

// =========================================================================
// Collision resistance: different paths produce different hashes
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn different_paths_different_hashes(
        a in "[a-zA-Z]{1,10}/[a-zA-Z]{1,10}",
        b in "[a-zA-Z]{1,10}/[a-zA-Z]{1,10}",
    ) {
        prop_assume!(a != b);
        let ha = short_hash(&a);
        let hb = short_hash(&b);
        prop_assert_ne!(ha, hb, "Different inputs should produce different hashes");
    }

    #[test]
    fn different_paths_different_redactions(
        a in "[a-zA-Z]{1,8}/[a-zA-Z]{1,8}.rs",
        b in "[a-zA-Z]{1,8}/[a-zA-Z]{1,8}.rs",
    ) {
        prop_assume!(a != b);
        let ra = redact_path(&a);
        let rb = redact_path(&b);
        prop_assert_ne!(ra, rb, "Different paths should have different redactions");
    }
}

// =========================================================================
// Hash length is constant across all inputs
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn hash_length_constant_across_inputs(
        a in ".{1,200}",
        b in ".{1,200}",
    ) {
        let ha = short_hash(&a);
        let hb = short_hash(&b);
        prop_assert_eq!(
            ha.len(), hb.len(),
            "Hash lengths should be equal: '{}' ({}) vs '{}' ({})",
            ha, ha.len(), hb, hb.len()
        );
    }

    #[test]
    fn redaction_length_consistent_for_same_extension(
        a_parts in prop::collection::vec("[a-zA-Z]{1,5}", 1..=4),
        b_parts in prop::collection::vec("[a-zA-Z]{1,5}", 1..=4),
    ) {
        let a = format!("{}.rs", a_parts.join("/"));
        let b = format!("{}.rs", b_parts.join("/"));
        prop_assume!(a != b);
        let ra = redact_path(&a);
        let rb = redact_path(&b);
        prop_assert_eq!(
            ra.len(), rb.len(),
            "Same extension paths should have same redaction length"
        );
    }
}

// =========================================================================
// Pure function: calling N times gives same result
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn short_hash_is_pure(input in ".{1,100}") {
        let results: Vec<String> = (0..5).map(|_| short_hash(&input)).collect();
        for r in &results[1..] {
            prop_assert_eq!(&results[0], r, "short_hash should be pure");
        }
    }

    #[test]
    fn redact_path_is_pure(path in arb_path_with_ext()) {
        let results: Vec<String> = (0..5).map(|_| redact_path(&path)).collect();
        for r in &results[1..] {
            prop_assert_eq!(&results[0], r, "redact_path should be pure");
        }
    }
}

// =========================================================================
// Backslash and forward-slash normalization equivalence
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn backslash_forward_slash_equivalence(
        parts in prop::collection::vec("[a-zA-Z0-9_]{1,8}", 2..=5),
        ext in prop::sample::select(vec!["rs", "py", "go"]),
    ) {
        let unix_path = format!("{}.{}", parts.join("/"), ext);
        let win_path = format!("{}.{}", parts.join("\\"), ext);
        let mixed = parts.iter().enumerate().map(|(i, p)| {
            if i == 0 { p.clone() } else if i % 2 == 0 { format!("/{}", p) } else { format!("\\{}", p) }
        }).collect::<String>();
        let mixed_path = format!("{}.{}", mixed, ext);

        let r_unix = redact_path(&unix_path);
        let r_win = redact_path(&win_path);
        let r_mixed = redact_path(&mixed_path);

        prop_assert_eq!(&r_unix, &r_win, "Unix and Windows paths should redact identically");
        prop_assert_eq!(&r_unix, &r_mixed, "Mixed separators should redact identically");
    }
}

// =========================================================================
// Idempotency: redacting the same input always gives the same output
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn redact_path_idempotent_across_calls(path in arb_path_with_ext()) {
        let first = redact_path(&path);
        let second = redact_path(&path);
        let third = redact_path(&path);
        prop_assert_eq!(&first, &second);
        prop_assert_eq!(&second, &third);
    }

    #[test]
    fn short_hash_always_hex(input in ".{1,200}") {
        let hash = short_hash(&input);
        prop_assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "short_hash must produce only hex chars, got: {}", hash
        );
    }

    #[test]
    fn short_hash_length_is_16(input in ".{0,500}") {
        let hash = short_hash(&input);
        prop_assert_eq!(hash.len(), 16, "short_hash must be 16 chars, got {}", hash.len());
    }
}

// =========================================================================
// Dot-prefix normalization: ./path and path produce same redaction
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn dot_prefix_normalizes_for_redact(
        parts in prop::collection::vec("[a-zA-Z0-9_]{1,8}", 1..=4),
        ext in prop::sample::select(vec!["rs", "py", "go", "js"]),
    ) {
        let plain = format!("{}.{}", parts.join("/"), ext);
        let dotted = format!("./{}.{}", parts.join("/"), ext);
        prop_assert_eq!(
            redact_path(&plain),
            redact_path(&dotted),
            "Dot-prefix should be normalized away"
        );
    }

    #[test]
    fn dot_prefix_normalizes_for_short_hash(
        parts in prop::collection::vec("[a-zA-Z0-9_]{1,8}", 1..=4),
    ) {
        let plain = parts.join("/");
        let dotted = format!("./{}", plain);
        prop_assert_eq!(
            short_hash(&plain),
            short_hash(&dotted),
            "Dot-prefix should be normalized for short_hash"
        );
    }
}
