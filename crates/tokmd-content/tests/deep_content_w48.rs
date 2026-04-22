//! Deep tests for tokmd-content (wave 48).
//!
//! Covers entropy calculation, high-entropy detection, import/tag extraction,
//! BLAKE3 hashing, property tests for entropy bounds and hash determinism,
//! and edge cases for empty/binary/large files.

use std::fs::File;
use std::io::Write;

use tokmd_content::{
    count_tags, entropy_bits_per_byte, hash_bytes, hash_file, is_text_like, read_head,
    read_head_tail, read_lines, read_text_capped,
};

// ============================================================================
// 1. Entropy calculation for known inputs
// ============================================================================

#[test]
fn entropy_empty_bytes_is_zero() {
    assert_eq!(entropy_bits_per_byte(&[]), 0.0);
}

#[test]
fn entropy_single_repeated_byte_is_zero() {
    let data = vec![0xAA; 1000];
    let e = entropy_bits_per_byte(&data);
    assert!(e.abs() < 1e-6, "All same byte → entropy ~0, got {e}");
}

#[test]
fn entropy_two_equal_values_is_one_bit() {
    let data: Vec<u8> = (0..1000).map(|i| (i % 2) as u8).collect();
    let e = entropy_bits_per_byte(&data);
    assert!(
        (e - 1.0).abs() < 0.01,
        "Two equally frequent bytes → ~1.0 bit, got {e}"
    );
}

#[test]
fn entropy_four_equal_values_is_two_bits() {
    let data: Vec<u8> = (0..1000).map(|i| (i % 4) as u8).collect();
    let e = entropy_bits_per_byte(&data);
    assert!(
        (e - 2.0).abs() < 0.02,
        "Four equally frequent bytes → ~2.0 bits, got {e}"
    );
}

#[test]
fn entropy_full_byte_range_near_eight() {
    let data: Vec<u8> = (0u8..=255).cycle().take(2560).collect();
    let e = entropy_bits_per_byte(&data);
    assert!(
        (e - 8.0).abs() < 0.01,
        "Full byte range → ~8.0 bits, got {e}"
    );
}

#[test]
fn entropy_ascii_english_text_moderate() {
    let text =
        b"The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor jugs.";
    let e = entropy_bits_per_byte(text);
    assert!(e > 3.0, "English text entropy should be > 3.0, got {e}");
    assert!(e < 6.0, "English text entropy should be < 6.0, got {e}");
}

#[test]
fn entropy_single_byte_input() {
    let e = entropy_bits_per_byte(&[42]);
    assert!(e.abs() < 1e-6, "Single byte → 0 entropy, got {e}");
}

// ============================================================================
// 2. High-entropy detection (random bytes vs normal code)
// ============================================================================

#[test]
fn high_entropy_random_looking_bytes() {
    // Simulated "random" data: full byte range shuffled
    let data: Vec<u8> = (0u8..=255).cycle().take(4096).collect();
    let e = entropy_bits_per_byte(&data);
    assert!(
        e > 7.5,
        "Random-like data should have high entropy, got {e}"
    );
}

#[test]
fn low_entropy_typical_source_code() {
    let code = br#"
fn main() {
    let x = 42;
    let y = x + 1;
    println!("Hello, world! {}", y);
    for i in 0..10 {
        println!("{}", i);
    }
}
"#;
    let e = entropy_bits_per_byte(code);
    assert!(e < 5.5, "Source code should have moderate entropy, got {e}");
    assert!(
        e > 2.0,
        "Source code should not have near-zero entropy, got {e}"
    );
}

#[test]
fn high_entropy_base64_like_content() {
    // Base64-encoded data has high entropy but not maximal
    let b64 = b"SGVsbG8gV29ybGQhIFRoaXMgaXMgYSB0ZXN0IG9mIGJhc2U2NCBlbmNvZGluZyB3aXRoIHNvbWUgcmVhbGx5IGxvbmcgdGV4dCB0aGF0IHNob3VsZCBiZSBlbm91Z2ggdG8gZ2V0IGEgZ29vZCBlbnRyb3B5IHJlYWRpbmc=";
    let e = entropy_bits_per_byte(b64);
    assert!(e > 4.0, "Base64 data should have elevated entropy, got {e}");
}

#[test]
fn entropy_compressed_data_high() {
    // Simulate compressed/encrypted bytes: every byte value present
    let mut data = Vec::with_capacity(512);
    for i in 0..512 {
        data.push((i * 37 + 13) as u8); // Pseudo-random distribution
    }
    let e = entropy_bits_per_byte(&data);
    assert!(
        e > 6.0,
        "Pseudo-random data should have high entropy, got {e}"
    );
}

// ============================================================================
// 3. Import/tag extraction from source files
// ============================================================================

#[test]
fn count_tags_finds_todo_in_code() {
    let code = r#"
fn main() {
    // TODO: implement this
    let x = 42;
    // TODO: add error handling
}
"#;
    let tags = count_tags(code, &["TODO"]);
    assert_eq!(tags[0].1, 2, "Should find 2 TODOs");
}

#[test]
fn count_tags_finds_fixme_and_hack() {
    let code = "// FIXME: broken\n// HACK: workaround\n// FIXME: another";
    let tags = count_tags(code, &["FIXME", "HACK"]);
    assert_eq!(tags[0].1, 2, "FIXME count");
    assert_eq!(tags[1].1, 1, "HACK count");
}

#[test]
fn count_tags_case_insensitive() {
    let code = "todo Todo TODO tOdO";
    let tags = count_tags(code, &["TODO"]);
    assert_eq!(tags[0].1, 4, "All case variants should match");
}

#[test]
fn count_tags_no_false_positives_on_clean_code() {
    let code = "fn main() {\n    let x = 42;\n    println!(\"hello\");\n}";
    let tags = count_tags(code, &["TODO", "FIXME", "HACK", "SAFETY"]);
    for (tag, count) in &tags {
        assert_eq!(*count, 0, "Tag {tag} should not be found in clean code");
    }
}

#[test]
fn count_tags_preserves_order_and_names() {
    let tags = count_tags("TODO FIXME", &["FIXME", "TODO", "HACK"]);
    assert_eq!(tags[0].0, "FIXME");
    assert_eq!(tags[1].0, "TODO");
    assert_eq!(tags[2].0, "HACK");
}

#[test]
fn count_tags_empty_text() {
    let tags = count_tags("", &["TODO", "FIXME"]);
    assert_eq!(tags[0].1, 0);
    assert_eq!(tags[1].1, 0);
}

#[test]
fn count_tags_adjacent_occurrences() {
    let text = "TODOTODOTODO";
    let tags = count_tags(text, &["TODO"]);
    assert_eq!(tags[0].1, 3, "Adjacent TODOs should be counted separately");
}

// ============================================================================
// 4. File hashing (BLAKE3)
// ============================================================================

#[test]
fn hash_bytes_returns_64_hex_chars() {
    let h = hash_bytes(b"test content");
    assert_eq!(h.len(), 64);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn hash_bytes_deterministic() {
    let a = hash_bytes(b"deterministic");
    let b = hash_bytes(b"deterministic");
    assert_eq!(a, b);
}

#[test]
fn hash_bytes_different_content_different_hash() {
    let a = hash_bytes(b"alpha");
    let b = hash_bytes(b"beta");
    assert_ne!(a, b);
}

#[test]
fn hash_bytes_empty_is_valid() {
    let h = hash_bytes(b"");
    assert_eq!(h.len(), 64);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn hash_file_matches_hash_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("file.txt");
    File::create(&path)
        .unwrap()
        .write_all(b"hello world")
        .unwrap();
    let file_hash = hash_file(&path, 1000).unwrap();
    let bytes_hash = hash_bytes(b"hello world");
    assert_eq!(file_hash, bytes_hash);
}

#[test]
fn hash_file_respects_max_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("large.txt");
    File::create(&path)
        .unwrap()
        .write_all(b"0123456789abcdef")
        .unwrap();
    let h5 = hash_file(&path, 5).unwrap();
    let expected = hash_bytes(b"01234");
    assert_eq!(h5, expected);
}

#[test]
fn hash_file_binary_content() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("binary.bin");
    let data: Vec<u8> = (0..=255).collect();
    File::create(&path).unwrap().write_all(&data).unwrap();
    let h = hash_file(&path, 10000).unwrap();
    let expected = hash_bytes(&data);
    assert_eq!(h, expected);
}

#[test]
fn hash_bytes_lowercase_hex() {
    let h = hash_bytes(b"check case");
    assert!(
        h.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
        "Hash should be lowercase hex: {h}"
    );
}

// ============================================================================
// 5. Property tests
// ============================================================================

mod properties {
    use proptest::prelude::*;
    use tokmd_content::{entropy_bits_per_byte, hash_bytes};

    proptest! {
        #[test]
        fn entropy_always_between_zero_and_eight(
            bytes in proptest::collection::vec(any::<u8>(), 0..2048)
        ) {
            let e = entropy_bits_per_byte(&bytes);
            prop_assert!(e >= 0.0, "Entropy must be non-negative, got {e}");
            prop_assert!(e <= 8.0 + 1e-6, "Entropy must be <= 8.0, got {e}");
        }

        #[test]
        fn entropy_is_finite(
            bytes in proptest::collection::vec(any::<u8>(), 0..1024)
        ) {
            let e = entropy_bits_per_byte(&bytes);
            prop_assert!(e.is_finite(), "Entropy must be finite, got {e}");
        }

        #[test]
        fn entropy_empty_always_zero(_i in 0..5u8) {
            let e = entropy_bits_per_byte(&[]);
            prop_assert_eq!(e, 0.0);
        }

        #[test]
        fn entropy_uniform_single_byte_always_zero(
            byte in any::<u8>(),
            len in 1usize..500
        ) {
            let data = vec![byte; len];
            let e = entropy_bits_per_byte(&data);
            prop_assert!(e.abs() < 1e-4, "Uniform single byte → ~0 entropy, got {e}");
        }

        #[test]
        fn hash_same_content_always_identical(
            bytes in proptest::collection::vec(any::<u8>(), 0..512)
        ) {
            let h1 = hash_bytes(&bytes);
            let h2 = hash_bytes(&bytes);
            prop_assert_eq!(h1, h2, "Hash of same content must be identical");
        }

        #[test]
        fn hash_always_64_hex(
            bytes in proptest::collection::vec(any::<u8>(), 0..512)
        ) {
            let h = hash_bytes(&bytes);
            prop_assert_eq!(h.len(), 64);
            prop_assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
        }

        #[test]
        fn hash_different_inputs_differ(
            a in proptest::collection::vec(any::<u8>(), 1..256),
            b in proptest::collection::vec(any::<u8>(), 1..256)
        ) {
            prop_assume!(a != b);
            let ha = hash_bytes(&a);
            let hb = hash_bytes(&b);
            prop_assert_ne!(ha, hb);
        }
    }
}

// ============================================================================
// 6. Edge cases: empty files
// ============================================================================

#[test]
fn edge_read_head_empty_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("empty.txt");
    File::create(&path).unwrap();
    let bytes = read_head(&path, 100).unwrap();
    assert!(bytes.is_empty());
}

#[test]
fn edge_hash_file_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("empty.txt");
    File::create(&path).unwrap();
    let h = hash_file(&path, 1000).unwrap();
    assert_eq!(
        h,
        hash_bytes(b""),
        "Empty file hash should match empty bytes hash"
    );
}

#[test]
fn edge_read_lines_empty_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("empty.txt");
    File::create(&path).unwrap();
    let lines = read_lines(&path, 100, 10000).unwrap();
    assert!(lines.is_empty());
}

#[test]
fn edge_read_head_tail_empty_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("empty.txt");
    File::create(&path).unwrap();
    let bytes = read_head_tail(&path, 100).unwrap();
    assert!(bytes.is_empty());
}

#[test]
fn edge_entropy_empty_file_content() {
    let e = entropy_bits_per_byte(b"");
    assert_eq!(e, 0.0);
}

// ============================================================================
// 7. Edge cases: binary files
// ============================================================================

#[test]
fn edge_is_text_like_binary_with_nulls() {
    let data = vec![0u8; 100];
    assert!(!is_text_like(&data), "Null bytes indicate binary content");
}

#[test]
fn edge_is_text_like_mixed_binary() {
    let mut data = b"hello".to_vec();
    data.push(0);
    data.extend_from_slice(b"world");
    assert!(!is_text_like(&data), "Embedded null means not text-like");
}

#[test]
fn edge_is_text_like_valid_utf8() {
    assert!(is_text_like(b"Hello, World!"));
    assert!(is_text_like("café résumé".as_bytes()));
}

#[test]
fn edge_is_text_like_empty() {
    assert!(is_text_like(b""), "Empty bytes should be text-like");
}

#[test]
fn edge_hash_file_all_zeros() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("zeros.bin");
    File::create(&path).unwrap().write_all(&[0u8; 256]).unwrap();
    let h = hash_file(&path, 10000).unwrap();
    let expected = hash_bytes(&[0u8; 256]);
    assert_eq!(h, expected);
}

// ============================================================================
// 8. Edge cases: very large single-line files
// ============================================================================

#[test]
fn edge_read_lines_single_long_line() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("long_line.txt");
    let line = "x".repeat(100_000);
    File::create(&path)
        .unwrap()
        .write_all(line.as_bytes())
        .unwrap();
    let lines = read_lines(&path, 100, 200_000).unwrap();
    assert_eq!(
        lines.len(),
        1,
        "Single long line should be read as one line"
    );
    assert_eq!(lines[0].len(), 100_000);
}

#[test]
fn edge_read_head_large_file_capped() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("big.txt");
    let content = "A".repeat(50_000);
    File::create(&path)
        .unwrap()
        .write_all(content.as_bytes())
        .unwrap();
    let bytes = read_head(&path, 100).unwrap();
    assert_eq!(bytes.len(), 100, "Should cap at max_bytes");
}

#[test]
fn edge_entropy_large_uniform_content() {
    let data = vec![b'A'; 100_000];
    let e = entropy_bits_per_byte(&data);
    assert!(e.abs() < 1e-6, "100k identical bytes → ~0 entropy, got {e}");
}

#[test]
fn edge_read_text_capped_large_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("big.txt");
    let content = "B".repeat(10_000);
    File::create(&path)
        .unwrap()
        .write_all(content.as_bytes())
        .unwrap();
    let text = read_text_capped(&path, 50).unwrap();
    assert_eq!(text.len(), 50);
    assert!(text.chars().all(|c| c == 'B'));
}

#[test]
fn edge_read_head_tail_large_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("ht.txt");
    // Write "AAAA...BBBB..." where first half is A, second half is B
    let mut content = vec![b'A'; 5000];
    content.extend(vec![b'B'; 5000]);
    File::create(&path).unwrap().write_all(&content).unwrap();
    let bytes = read_head_tail(&path, 10).unwrap();
    assert_eq!(bytes.len(), 10);
    // head should be A's, tail should be B's
    assert_eq!(bytes[0], b'A');
    assert_eq!(bytes[bytes.len() - 1], b'B');
}

#[test]
fn edge_read_lines_bytes_limit_on_long_line() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("long.txt");
    let line = "x".repeat(1000);
    let mut f = File::create(&path).unwrap();
    writeln!(f, "{line}").unwrap();
    writeln!(f, "second line").unwrap();
    // Byte limit 500 is less than the first line, so no lines fit.
    let lines = read_lines(&path, 100, 500).unwrap();
    assert_eq!(
        lines.len(),
        0,
        "Should stop before adding a line that exceeds the byte limit"
    );
}
