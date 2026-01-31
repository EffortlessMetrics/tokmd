//! Fuzz target for entropy calculation and content analysis.
//!
//! Tests `entropy_bits_per_byte()`, `is_text_like()`, `count_tags()`,
//! and `hash_bytes()` with arbitrary input to find panics or unexpected behavior.

#![no_main]
use libfuzzer_sys::fuzz_target;
use tokmd_content::{count_tags, entropy_bits_per_byte, hash_bytes, is_text_like};

/// Max input size - entropy calculation is O(n) so we can be more generous
const MAX_INPUT_SIZE: usize = 256 * 1024; // 256KB

fuzz_target!(|data: &[u8]| {
    if data.len() > MAX_INPUT_SIZE {
        return;
    }
    // Test entropy calculation - should never panic, always return valid f32
    let entropy = entropy_bits_per_byte(data);
    assert!(entropy >= 0.0, "Entropy should be non-negative");
    assert!(entropy <= 8.0, "Entropy should be at most 8 bits per byte");
    assert!(!entropy.is_nan(), "Entropy should not be NaN");

    // Test text detection - should never panic
    let _ = is_text_like(data);

    // Test hashing - should never panic
    let hash = hash_bytes(data);
    assert_eq!(hash.len(), 64, "BLAKE3 hash should be 64 hex chars");

    // Test tag counting with arbitrary text
    if let Ok(text) = std::str::from_utf8(data) {
        let tags = ["TODO", "FIXME", "HACK", "XXX", "BUG"];
        let counts = count_tags(text, &tags);
        assert_eq!(counts.len(), tags.len());
        for (tag, count) in &counts {
            assert!(tags.contains(&tag.as_str()));
            // Count should be non-negative (it's usize so always is)
            let _ = count;
        }
    }
});
