//! Property-based tests for tokmd-content functions.

use proptest::prelude::*;
use tokmd_content::{count_tags, entropy_bits_per_byte, hash_bytes, is_text_like};

proptest! {
    // ========================
    // Entropy Properties
    // ========================

    #[test]
    fn entropy_always_in_bounds(bytes in prop::collection::vec(any::<u8>(), 0..1024)) {
        let entropy = entropy_bits_per_byte(&bytes);
        prop_assert!(entropy >= 0.0, "Entropy must be non-negative: got {}", entropy);
        prop_assert!(entropy <= 8.0, "Entropy must be at most 8 bits/byte: got {}", entropy);
    }

    #[test]
    fn entropy_empty_is_zero(_dummy in 0..1u8) {
        let entropy = entropy_bits_per_byte(&[]);
        prop_assert_eq!(entropy, 0.0);
    }

    #[test]
    fn entropy_uniform_single_byte_is_zero(byte in any::<u8>(), len in 1usize..256) {
        // Uniform distribution of a single byte value should have zero entropy
        let bytes = vec![byte; len];
        let entropy = entropy_bits_per_byte(&bytes);
        prop_assert!(entropy.abs() < 0.0001, "Uniform bytes should have ~0 entropy: got {}", entropy);
    }

    #[test]
    fn entropy_two_values_max_one_bit(len in 2usize..256) {
        // Equal distribution of 0 and 1 should have ~1 bit of entropy
        let bytes: Vec<u8> = (0..len).map(|i| (i % 2) as u8).collect();
        let entropy = entropy_bits_per_byte(&bytes);
        prop_assert!(entropy <= 1.01, "Two-value distribution should have <=1 bit entropy: got {}", entropy);
    }

    #[test]
    fn entropy_random_bytes_high(seed in any::<u64>()) {
        // Bytes from 0-255 should have high entropy (~8 bits)
        let bytes: Vec<u8> = (0u8..=255).collect();
        let entropy = entropy_bits_per_byte(&bytes);
        prop_assert!(entropy > 7.9, "Full byte range should have ~8 bits entropy: got {}", entropy);

        // Use seed to avoid warning
        let _ = seed;
    }

    // ========================
    // Hash Properties
    // ========================

    #[test]
    fn hash_deterministic(bytes in prop::collection::vec(any::<u8>(), 0..512)) {
        let hash1 = hash_bytes(&bytes);
        let hash2 = hash_bytes(&bytes);
        prop_assert_eq!(hash1, hash2, "Same input should produce same hash");
    }

    #[test]
    fn hash_is_64_hex_chars(bytes in prop::collection::vec(any::<u8>(), 0..512)) {
        let hash = hash_bytes(&bytes);
        prop_assert_eq!(hash.len(), 64, "BLAKE3 hash should be 64 hex chars: got {}", hash.len());
        prop_assert!(hash.chars().all(|c| c.is_ascii_hexdigit()), "Hash should be hex: {}", hash);
    }

    #[test]
    fn hash_different_inputs_differ(bytes1 in prop::collection::vec(any::<u8>(), 1..256),
                                     bytes2 in prop::collection::vec(any::<u8>(), 1..256)) {
        // With high probability, different inputs produce different hashes
        prop_assume!(bytes1 != bytes2);
        let hash1 = hash_bytes(&bytes1);
        let hash2 = hash_bytes(&bytes2);
        prop_assert_ne!(hash1, hash2, "Different inputs should produce different hashes");
    }

    // ========================
    // Text Detection Properties
    // ========================

    #[test]
    fn is_text_like_no_nulls(bytes in prop::collection::vec(1u8..=255, 0..256)) {
        // Bytes without nulls might be text-like depending on UTF-8 validity
        let result = is_text_like(&bytes);
        // Just ensure it doesn't panic and returns a reasonable value
        let has_valid_utf8 = std::str::from_utf8(&bytes).is_ok();
        prop_assert_eq!(result, has_valid_utf8);
    }

    #[test]
    fn is_text_like_with_null_is_false(prefix in prop::collection::vec(any::<u8>(), 0..64),
                                        suffix in prop::collection::vec(any::<u8>(), 0..64)) {
        let mut bytes = prefix;
        bytes.push(0);
        bytes.extend(suffix);
        prop_assert!(!is_text_like(&bytes), "Bytes with null should not be text-like");
    }

    #[test]
    fn is_text_like_valid_utf8_strings(s in "\\PC*") {
        // Valid UTF-8 strings without nulls should be text-like
        if !s.contains('\0') {
            prop_assert!(is_text_like(s.as_bytes()), "Valid UTF-8 without null should be text-like");
        }
    }

    #[test]
    fn is_text_like_empty_is_true(_dummy in 0..1u8) {
        prop_assert!(is_text_like(&[]), "Empty bytes should be text-like");
    }

    // ========================
    // Tag Counting Properties
    // ========================

    #[test]
    fn count_tags_case_insensitive(text in "[a-zA-Z ]{0,64}", tag in "[a-zA-Z]{1,8}") {
        let lower_result = count_tags(&text.to_lowercase(), &[&tag.to_lowercase()]);
        let upper_result = count_tags(&text.to_uppercase(), &[&tag.to_uppercase()]);
        let mixed_result = count_tags(&text, &[&tag]);

        // All should find the same count
        let lower_count = lower_result.first().map(|(_, c)| *c).unwrap_or(0);
        let upper_count = upper_result.first().map(|(_, c)| *c).unwrap_or(0);
        let mixed_count = mixed_result.first().map(|(_, c)| *c).unwrap_or(0);

        prop_assert_eq!(lower_count, mixed_count, "Case insensitivity broken for lower");
        prop_assert_eq!(upper_count, mixed_count, "Case insensitivity broken for upper");
    }

    #[test]
    fn count_tags_returns_all_tags(text in "\\PC{0,64}", tags in prop::collection::vec("[a-zA-Z]{1,8}", 0..5)) {
        let tag_refs: Vec<&str> = tags.iter().map(|s| s.as_str()).collect();
        let result = count_tags(&text, &tag_refs);

        prop_assert_eq!(result.len(), tags.len(), "Should return one result per tag");
        for (i, (tag, _)) in result.iter().enumerate() {
            prop_assert_eq!(tag, &tags[i], "Tags should be in order");
        }
    }

    #[test]
    fn count_tags_known_counts(count in 0usize..10) {
        let text = "TODO ".repeat(count);
        let result = count_tags(&text, &["TODO"]);
        let found = result.first().map(|(_, c)| *c).unwrap_or(0);
        prop_assert_eq!(found, count, "Should find exact count of known tag");
    }

    #[test]
    fn count_tags_empty_text_zero_counts(tags in prop::collection::vec("[a-zA-Z]{1,8}", 1..5)) {
        let tag_refs: Vec<&str> = tags.iter().map(|s| s.as_str()).collect();
        let result = count_tags("", &tag_refs);

        for (tag, count) in result {
            prop_assert_eq!(count, 0, "Empty text should have zero count for tag: {}", tag);
        }
    }
}
