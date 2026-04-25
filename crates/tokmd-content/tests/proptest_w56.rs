use proptest::prelude::*;

proptest! {
    /// Entropy of any byte slice is in [0, 8].
    #[test]
    fn entropy_in_valid_range(bytes in proptest::collection::vec(any::<u8>(), 0..1024)) {
        let e = tokmd_content::entropy_bits_per_byte(&bytes);
        prop_assert!(e >= 0.0, "Entropy should be >= 0, got {}", e);
        prop_assert!(e <= 8.0, "Entropy should be <= 8, got {}", e);
    }

    /// Entropy of empty byte slice is exactly 0.
    #[test]
    fn entropy_of_empty_is_zero(_dummy in 0..1u8) {
        let e = tokmd_content::entropy_bits_per_byte(&[]);
        prop_assert_eq!(e, 0.0);
    }

    /// Entropy of single-value repeated bytes is 0.
    #[test]
    fn entropy_of_uniform_bytes_is_zero(byte_val in any::<u8>(), len in 1usize..256) {
        let bytes = vec![byte_val; len];
        let e = tokmd_content::entropy_bits_per_byte(&bytes);
        prop_assert!((e - 0.0).abs() < 1e-6,
            "Uniform bytes should have entropy ~0, got {}", e);
    }

    /// Entropy is deterministic.
    #[test]
    fn entropy_is_deterministic(bytes in proptest::collection::vec(any::<u8>(), 0..512)) {
        let e1 = tokmd_content::entropy_bits_per_byte(&bytes);
        let e2 = tokmd_content::entropy_bits_per_byte(&bytes);
        prop_assert_eq!(e1, e2);
    }

    /// hash_bytes produces a 64-char hex string (BLAKE3).
    #[test]
    fn hash_bytes_is_64_hex_chars(bytes in proptest::collection::vec(any::<u8>(), 0..512)) {
        let hash = tokmd_content::hash_bytes(&bytes);
        prop_assert_eq!(hash.len(), 64, "BLAKE3 hash should be 64 hex chars");
        prop_assert!(hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash should be hex: {}", hash);
    }

    /// hash_bytes is deterministic.
    #[test]
    fn hash_bytes_is_deterministic(bytes in proptest::collection::vec(any::<u8>(), 0..256)) {
        let h1 = tokmd_content::hash_bytes(&bytes);
        let h2 = tokmd_content::hash_bytes(&bytes);
        prop_assert_eq!(h1, h2);
    }

    /// Different inputs (usually) produce different hashes.
    #[test]
    fn hash_bytes_different_inputs_differ(
        a in proptest::collection::vec(any::<u8>(), 1..128),
        b in proptest::collection::vec(any::<u8>(), 1..128),
    ) {
        prop_assume!(a != b);
        let h1 = tokmd_content::hash_bytes(&a);
        let h2 = tokmd_content::hash_bytes(&b);
        prop_assert_ne!(h1, h2,
            "Different inputs should produce different hashes");
    }

    /// is_text_like returns false for bytes containing null.
    #[test]
    fn is_text_like_rejects_null_bytes(
        prefix in proptest::collection::vec(b'a'..=b'z', 0..32),
        suffix in proptest::collection::vec(b'a'..=b'z', 0..32),
    ) {
        let mut bytes = prefix;
        bytes.push(0);
        bytes.extend(suffix);
        prop_assert!(!tokmd_content::is_text_like(&bytes),
            "Bytes containing null should not be text-like");
    }

    /// is_text_like accepts valid ASCII strings.
    #[test]
    fn is_text_like_accepts_ascii(input in "[a-zA-Z0-9 \t\n]{0,100}") {
        prop_assert!(tokmd_content::is_text_like(input.as_bytes()),
            "Valid ASCII should be text-like");
    }

    /// count_tags returns non-negative counts.
    #[test]
    fn count_tags_nonnegative(text in "\\PC{0,200}") {
        let tags = &["TODO", "FIXME", "HACK"];
        let results = tokmd_content::count_tags(&text, tags);
        prop_assert_eq!(results.len(), 3);
        for (tag, count) in &results {
            prop_assert!(!tag.is_empty());
            // count is usize, always >= 0; just verify structure.
            let _ = count;
        }
    }

    /// count_tags is deterministic.
    #[test]
    fn count_tags_is_deterministic(text in "[a-zA-Z TODO FIXME HACK ]{0,100}") {
        let tags = &["TODO", "FIXME"];
        let r1 = tokmd_content::count_tags(&text, tags);
        let r2 = tokmd_content::count_tags(&text, tags);
        prop_assert_eq!(r1, r2);
    }

    /// count_tags is case-insensitive.
    #[test]
    fn count_tags_case_insensitive(
        prefix in "[a-zA-Z ]{0,20}",
        tag_case in prop::sample::select(vec!["todo", "Todo", "TODO", "tOdO"]),
        suffix in "[a-zA-Z ]{0,20}",
    ) {
        let text = format!("{} {} {}", prefix, tag_case, suffix);
        let results = tokmd_content::count_tags(&text, &["TODO"]);
        prop_assert!(results[0].1 >= 1,
            "Should find at least 1 TODO in '{}', got {}", text, results[0].1);
    }
}
