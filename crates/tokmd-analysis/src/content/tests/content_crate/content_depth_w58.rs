//! Depth tests for tokmd-analysis content helpers - W58.
//!
//! Exercises read_text_capped, entropy, is_text_like, hash determinism,
//! count_tags, and edge cases with binary/empty files.

use std::fs::File;
use std::io::Write;

use crate::content::io::{
    count_tags, entropy_bits_per_byte, hash_bytes, hash_file, is_text_like, read_head,
    read_head_tail, read_lines, read_text_capped,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn tmp_file(name: &str, content: &[u8]) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(name);
    let mut f = File::create(&path).unwrap();
    f.write_all(content).unwrap();
    (dir, path)
}

// ===========================================================================
// 1. read_text_capped
// ===========================================================================

#[test]
fn read_text_capped_returns_full_content_when_under_limit() {
    let (_dir, path) = tmp_file("small.txt", b"Hello, World!");
    let text = read_text_capped(&path, 1000).unwrap();
    assert_eq!(text, "Hello, World!");
}

#[test]
fn read_text_capped_truncates_at_limit() {
    let (_dir, path) = tmp_file("long.txt", b"The quick brown fox jumps");
    let text = read_text_capped(&path, 9).unwrap();
    assert_eq!(text, "The quick");
}

#[test]
fn read_text_capped_empty_file() {
    let (_dir, path) = tmp_file("empty.txt", b"");
    let text = read_text_capped(&path, 100).unwrap();
    assert_eq!(text, "");
}

#[test]
fn read_text_capped_binary_content_uses_lossy() {
    let (_dir, path) = tmp_file("bin.dat", &[0xFF, 0xFE, 0x00, 0x41]);
    let text = read_text_capped(&path, 100).unwrap();
    // Should contain replacement chars for invalid UTF-8 and 'A' (0x41)
    assert!(text.contains('A') || text.contains('\u{FFFD}'));
}

// ===========================================================================
// 2. Entropy calculation
// ===========================================================================

#[test]
fn entropy_empty_is_zero() {
    assert_eq!(entropy_bits_per_byte(&[]), 0.0);
}

#[test]
fn entropy_single_repeated_byte_is_zero() {
    let buf = vec![0x42u8; 1000];
    let e = entropy_bits_per_byte(&buf);
    assert!(e.abs() < 1e-6, "expected ~0.0, got {e}");
}

#[test]
fn entropy_two_values_is_one_bit() {
    let buf: Vec<u8> = (0..2000).map(|i| (i % 2) as u8).collect();
    let e = entropy_bits_per_byte(&buf);
    assert!((e - 1.0).abs() < 0.02, "expected ~1.0, got {e}");
}

#[test]
fn entropy_four_values_is_two_bits() {
    let buf: Vec<u8> = (0..2000).map(|i| (i % 4) as u8).collect();
    let e = entropy_bits_per_byte(&buf);
    assert!((e - 2.0).abs() < 0.02, "expected ~2.0, got {e}");
}

#[test]
fn entropy_full_byte_range_is_eight_bits() {
    let buf: Vec<u8> = (0u8..=255).cycle().take(2560).collect();
    let e = entropy_bits_per_byte(&buf);
    assert!((e - 8.0).abs() < 0.02, "expected ~8.0, got {e}");
}

#[test]
fn entropy_monotonically_increases_with_diversity() {
    let e1 = entropy_bits_per_byte(&vec![0xAA; 1000]);
    let e2 = {
        let buf: Vec<u8> = (0..1000).map(|i| (i % 4) as u8).collect();
        entropy_bits_per_byte(&buf)
    };
    let e3 = {
        let buf: Vec<u8> = (0..1000).map(|i| (i % 16) as u8).collect();
        entropy_bits_per_byte(&buf)
    };
    assert!(e1 < e2, "1 value < 4 values: {e1} < {e2}");
    assert!(e2 < e3, "4 values < 16 values: {e2} < {e3}");
}

// ===========================================================================
// 3. is_text_like detection
// ===========================================================================

#[test]
fn is_text_like_on_ascii() {
    assert!(is_text_like(b"Hello, World!"));
}

#[test]
fn is_text_like_on_utf8() {
    assert!(is_text_like("café résumé 日本語".as_bytes()));
}

#[test]
fn is_text_like_on_empty() {
    assert!(is_text_like(b""));
}

#[test]
fn is_text_like_rejects_null_bytes() {
    assert!(!is_text_like(&[0x48, 0x65, 0x00, 0x6C]));
}

#[test]
fn is_text_like_rejects_binary_blob() {
    let blob: Vec<u8> = (0u8..=255).collect();
    assert!(!is_text_like(&blob));
}

// ===========================================================================
// 4. Hash determinism
// ===========================================================================

#[test]
fn hash_bytes_deterministic() {
    let h1 = hash_bytes(b"hello");
    let h2 = hash_bytes(b"hello");
    assert_eq!(h1, h2);
}

#[test]
fn hash_bytes_different_inputs_differ() {
    let h1 = hash_bytes(b"hello");
    let h2 = hash_bytes(b"world");
    assert_ne!(h1, h2);
}

#[test]
fn hash_bytes_is_64_hex_chars() {
    let h = hash_bytes(b"test");
    assert_eq!(h.len(), 64, "BLAKE3 hex output should be 64 chars");
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn hash_file_matches_hash_bytes() {
    let (_dir, path) = tmp_file("hashme.txt", b"deterministic content");
    let file_hash = hash_file(&path, 10000).unwrap();
    let bytes_hash = hash_bytes(b"deterministic content");
    assert_eq!(file_hash, bytes_hash);
}

#[test]
fn hash_file_respects_max_bytes() {
    let (_dir, path) = tmp_file("partial.txt", b"abcdefghij");
    let h_partial = hash_file(&path, 5).unwrap();
    let h_full = hash_file(&path, 1000).unwrap();
    assert_ne!(h_partial, h_full, "partial hash should differ from full");
    assert_eq!(h_partial, hash_bytes(b"abcde"));
}

#[test]
fn hash_empty_file() {
    let (_dir, path) = tmp_file("empty.dat", b"");
    let h = hash_file(&path, 100).unwrap();
    assert_eq!(h, hash_bytes(b""));
}

// ===========================================================================
// 5. Tag counting (TODO, FIXME, etc.)
// ===========================================================================

#[test]
fn count_tags_finds_todo_and_fixme() {
    let text = "// TODO: fix this\n// FIXME: broken\n// TODO: another one\n";
    let tags = count_tags(text, &["TODO", "FIXME"]);
    let todo_count = tags.iter().find(|(t, _)| t == "TODO").map(|(_, c)| *c);
    let fixme_count = tags.iter().find(|(t, _)| t == "FIXME").map(|(_, c)| *c);
    assert_eq!(todo_count, Some(2));
    assert_eq!(fixme_count, Some(1));
}

#[test]
fn count_tags_case_insensitive() {
    let text = "todo Todo TODO";
    let tags = count_tags(text, &["TODO"]);
    assert_eq!(tags[0].1, 3, "should match case-insensitively");
}

#[test]
fn count_tags_no_matches() {
    let text = "clean code, no issues";
    let tags = count_tags(text, &["TODO", "FIXME", "HACK"]);
    assert!(tags.iter().all(|(_, c)| *c == 0));
}

#[test]
fn count_tags_empty_text() {
    let tags = count_tags("", &["TODO"]);
    assert_eq!(tags[0].1, 0);
}

#[test]
fn count_tags_preserves_tag_order() {
    let text = "FIXME TODO HACK";
    let tags = count_tags(text, &["TODO", "FIXME", "HACK"]);
    assert_eq!(tags[0].0, "TODO");
    assert_eq!(tags[1].0, "FIXME");
    assert_eq!(tags[2].0, "HACK");
}

// ===========================================================================
// 6. Binary files and edge cases
// ===========================================================================

#[test]
fn read_head_on_binary_file() {
    let (_dir, path) = tmp_file("binary.bin", &[0xFF, 0x00, 0xDE, 0xAD]);
    let bytes = read_head(&path, 100).unwrap();
    assert_eq!(bytes, &[0xFF, 0x00, 0xDE, 0xAD]);
}

#[test]
fn read_head_tail_on_small_file() {
    let (_dir, path) = tmp_file("tiny.txt", b"abc");
    let bytes = read_head_tail(&path, 100).unwrap();
    assert_eq!(bytes, b"abc");
}

#[test]
fn read_head_tail_splits_large_file() {
    let (_dir, path) = tmp_file("big.txt", b"0123456789");
    // max 4: head=2 ("01"), tail=2 ("89")
    let bytes = read_head_tail(&path, 4).unwrap();
    assert_eq!(bytes, b"0189");
}

#[test]
fn read_lines_empty_file() {
    let (_dir, path) = tmp_file("empty_lines.txt", b"");
    let lines = read_lines(&path, 100, 10000).unwrap();
    assert!(lines.is_empty());
}

#[test]
fn read_lines_respects_max_lines() {
    let content = "line1\nline2\nline3\nline4\nline5\n";
    let (_dir, path) = tmp_file("lines.txt", content.as_bytes());
    let lines = read_lines(&path, 2, 10000).unwrap();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "line1");
    assert_eq!(lines[1], "line2");
}

#[test]
fn read_lines_zero_max_returns_empty() {
    let (_dir, path) = tmp_file("nope.txt", b"content\n");
    let lines = read_lines(&path, 0, 10000).unwrap();
    assert!(lines.is_empty());
}
