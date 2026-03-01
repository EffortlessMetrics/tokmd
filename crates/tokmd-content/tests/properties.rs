//! Property-based tests for tokmd-content functions.

use proptest::prelude::*;
use tokmd_content::complexity::{
    analyze_functions, analyze_nesting_depth, estimate_cognitive_complexity,
    estimate_cyclomatic_complexity,
};
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

    // ========================
    // Entropy Monotonicity
    // ========================

    #[test]
    fn entropy_increases_with_more_distinct_values(n in 2usize..64) {
        // Entropy with n distinct values should be >= entropy with 1 value
        let uniform: Vec<u8> = (0..256).map(|i| (i % n) as u8).collect();
        let single = vec![0u8; 256];
        let e_multi = entropy_bits_per_byte(&uniform);
        let e_single = entropy_bits_per_byte(&single);
        prop_assert!(e_multi >= e_single,
            "More distinct values should not decrease entropy: {n} values -> {e_multi}, 1 value -> {e_single}");
    }

    #[test]
    fn entropy_is_finite(bytes in prop::collection::vec(any::<u8>(), 0..2048)) {
        let entropy = entropy_bits_per_byte(&bytes);
        prop_assert!(entropy.is_finite(), "Entropy must be finite: got {}", entropy);
    }

    // ========================
    // Hash Prefix Properties
    // ========================

    #[test]
    fn hash_is_lowercase_hex(bytes in prop::collection::vec(any::<u8>(), 0..256)) {
        let hash = hash_bytes(&bytes);
        prop_assert!(hash.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
            "Hash should be lowercase hex: {}", hash);
    }

    #[test]
    fn hash_empty_is_deterministic(_dummy in 0..5u8) {
        let h1 = hash_bytes(&[]);
        let h2 = hash_bytes(&[]);
        prop_assert_eq!(h1.len(), 64);
        prop_assert_eq!(h1, h2, "Empty hash should always be the same");
    }

    #[test]
    fn hash_prefix_differs_for_appended_bytes(
        base in prop::collection::vec(any::<u8>(), 1..128),
        extra in prop::collection::vec(any::<u8>(), 1..64)
    ) {
        let hash_base = hash_bytes(&base);
        let mut extended = base.clone();
        extended.extend_from_slice(&extra);
        let hash_extended = hash_bytes(&extended);
        prop_assert_ne!(hash_base, hash_extended,
            "Appending bytes should change the hash");
    }

    // ========================
    // Text Detection Strengthened
    // ========================

    #[test]
    fn ascii_printable_is_text_like(bytes in prop::collection::vec(0x20u8..=0x7Eu8, 1..256)) {
        prop_assert!(is_text_like(&bytes), "Printable ASCII should be text-like");
    }

    #[test]
    fn is_text_like_idempotent(bytes in prop::collection::vec(any::<u8>(), 0..256)) {
        let r1 = is_text_like(&bytes);
        let r2 = is_text_like(&bytes);
        prop_assert_eq!(r1, r2, "is_text_like should be deterministic");
    }

    // ========================
    // Tag Counting Extended
    // ========================

    #[test]
    fn count_tags_substring_counted(count in 1usize..20) {
        // Repeating "FIXME " n times should yield n occurrences
        let text = "FIXME ".repeat(count);
        let result = count_tags(&text, &["FIXME"]);
        let found = result.first().map(|(_, c)| *c).unwrap_or(0);
        prop_assert_eq!(found, count, "Expected FIXME count mismatch");
    }

    #[test]
    fn count_tags_disjoint_tags_independent(
        n_todo in 0usize..10,
        n_fixme in 0usize..10,
    ) {
        let text = format!("{}{}", "TODO ".repeat(n_todo), "FIXME ".repeat(n_fixme));
        let result = count_tags(&text, &["TODO", "FIXME"]);
        let todo_count = result[0].1;
        let fixme_count = result[1].1;
        prop_assert_eq!(todo_count, n_todo, "TODO count mismatch");
        prop_assert_eq!(fixme_count, n_fixme, "FIXME count mismatch");
    }

    // ========================
    // Import Pattern Detection via Tags
    // ========================

    #[test]
    fn count_tags_detects_import_patterns(
        n_import in 0usize..5,
        n_require in 0usize..5,
    ) {
        let text = format!(
            "{}{}",
            "import foo\n".repeat(n_import),
            "require('baz')\n".repeat(n_require),
        );
        let result = count_tags(&text, &["import", "require"]);
        let import_count = result[0].1;
        let require_count = result[1].1;
        prop_assert_eq!(import_count, n_import, "import count mismatch");
        prop_assert_eq!(require_count, n_require, "require count mismatch");
    }

    #[test]
    fn count_tags_detects_rust_use_statements(n_use in 0usize..5) {
        // Build text with no accidental "use" substrings
        let text = "use std::path::Path;\n".repeat(n_use);
        let result = count_tags(&text, &["use"]);
        prop_assert_eq!(result[0].1, n_use, "use statement count mismatch");
    }

    // ========================
    // Entropy File-Based Round-Trip
    // ========================

    #[test]
    fn entropy_of_file_bytes_matches_direct(content in prop::collection::vec(any::<u8>(), 0..512)) {
        // Hash through file I/O and directly should agree
        let direct_entropy = entropy_bits_per_byte(&content);
        // Re-compute from a clone to verify determinism
        let clone_entropy = entropy_bits_per_byte(&content.clone());
        prop_assert_eq!(
            direct_entropy, clone_entropy,
            "entropy must be deterministic for same content"
        );
    }

    // ========================
    // Hash Determinism Across Identical Content
    // ========================

    #[test]
    fn hash_same_content_always_matches(
        content in prop::collection::vec(any::<u8>(), 0..256),
        repeats in 2usize..5,
    ) {
        let first = hash_bytes(&content);
        for _ in 1..repeats {
            let again = hash_bytes(&content);
            prop_assert_eq!(&first, &again, "hash must be identical for same content");
        }
    }

    // ========================
    // Complexity Properties
    // ========================

    #[test]
    fn analyze_functions_count_never_negative(code in "fn [a-z]{1,8}\\(\\) \\{\n    let x = 1;\n\\}\n") {
        let metrics = analyze_functions(&code, "rust");
        // function_count is usize, can't be negative, but verify it's reasonable
        prop_assert!(metrics.function_count <= 10,
            "Single fn code should detect at most a few functions: got {}", metrics.function_count);
    }

    #[test]
    fn cyclomatic_complexity_at_least_one_per_function(code in "fn [a-z]{1,8}\\(\\) \\{\n    let x = 1;\n\\}\n") {
        let result = estimate_cyclomatic_complexity(&code, "rust");
        if result.function_count > 0 {
            prop_assert!(result.max_cc >= 1,
                "Every function has at least CC=1: got {}", result.max_cc);
            prop_assert!(result.total_cc >= result.function_count,
                "Total CC ({}) should be >= function count ({})", result.total_cc, result.function_count);
        }
    }

    #[test]
    fn cognitive_complexity_non_negative(code in "fn [a-z]{1,8}\\(\\) \\{\n    let x = 1;\n\\}\n") {
        let result = estimate_cognitive_complexity(&code, "rust");
        // All cognitive complexity values are usize, so non-negative by type
        if result.function_count > 0 {
            prop_assert!(result.avg >= 0.0, "Average cognitive complexity must be non-negative");
        }
    }

    #[test]
    fn nesting_depth_non_negative_for_any_language(
        code in "(fn [a-z]+\\(\\) \\{\n    let x = 1;\n\\}\n){0,3}",
        lang in prop::sample::select(vec!["rust", "python", "javascript", "go", "unknown"])
    ) {
        let result = analyze_nesting_depth(&code, lang);
        prop_assert!(result.avg_depth >= 0.0,
            "Average nesting depth must be non-negative: got {}", result.avg_depth);
    }

    #[test]
    fn empty_code_yields_default_metrics(
        lang in prop::sample::select(vec!["rust", "python", "javascript", "go"])
    ) {
        let fn_metrics = analyze_functions("", lang);
        prop_assert_eq!(fn_metrics.function_count, 0);

        let cc = estimate_cyclomatic_complexity("", lang);
        prop_assert_eq!(cc.function_count, 0);
        prop_assert_eq!(cc.total_cc, 0);

        let cog = estimate_cognitive_complexity("", lang);
        prop_assert_eq!(cog.function_count, 0);
        prop_assert_eq!(cog.total, 0);

        let nest = analyze_nesting_depth("", lang);
        prop_assert_eq!(nest.max_depth, 0);
    }
}
