//! Content module boundary tests.
//!
//! Verifies that tokmd-content functions handle empty files, binary files,
//! non-existent paths, entropy edge cases, and tag extraction robustly.

use std::io::Write;
use tempfile::NamedTempFile;
use tokmd_content::{
    count_tags, entropy_bits_per_byte, hash_bytes, hash_file, is_text_like, read_head,
    read_head_tail, read_lines, read_text_capped,
};

// ── entropy edge cases ───────────────────────────────────────────────

#[test]
fn entropy_empty_slice_returns_zero() {
    assert_eq!(entropy_bits_per_byte(&[]), 0.0);
}

#[test]
fn entropy_single_byte_returns_zero() {
    // One unique value repeated — zero entropy
    assert_eq!(entropy_bits_per_byte(&[42]), 0.0);
}

#[test]
fn entropy_uniform_bytes_returns_zero() {
    let data = vec![0xAA; 1024];
    assert_eq!(entropy_bits_per_byte(&data), 0.0);
}

#[test]
fn entropy_two_values_equal_distribution_near_one_bit() {
    let mut data = vec![0u8; 1000];
    for i in 0..500 {
        data[i] = 1;
    }
    let e = entropy_bits_per_byte(&data);
    assert!((e - 1.0).abs() < 0.01, "expected ~1.0, got {e}");
}

#[test]
fn entropy_all_256_values_near_eight_bits() {
    let data: Vec<u8> = (0..=255).collect();
    let e = entropy_bits_per_byte(&data);
    assert!((e - 8.0).abs() < 0.01, "expected ~8.0, got {e}");
}

#[test]
fn entropy_bounded_between_zero_and_eight() {
    let data = b"Hello, world! This is a test of entropy calculation.";
    let e = entropy_bits_per_byte(data);
    assert!(e >= 0.0 && e <= 8.0, "entropy out of range: {e}");
}

// ── tag extraction ───────────────────────────────────────────────────

#[test]
fn count_tags_empty_text_returns_zeros() {
    let result = count_tags("", &["TODO", "FIXME"]);
    assert_eq!(result.len(), 2);
    for (_, count) in &result {
        assert_eq!(*count, 0);
    }
}

#[test]
fn count_tags_case_insensitive() {
    let text = "todo: fix this\nTODO: and this\nToDo: also this";
    let result = count_tags(text, &["TODO"]);
    assert_eq!(result[0].1, 3, "should find all case variants");
}

#[test]
fn count_tags_multiple_tags() {
    let text = "TODO: fix this\nFIXME: broken\nHACK: workaround\nTODO: another";
    let result = count_tags(text, &["TODO", "FIXME", "HACK"]);
    assert_eq!(result[0].1, 2); // TODO
    assert_eq!(result[1].1, 1); // FIXME
    assert_eq!(result[2].1, 1); // HACK
}

#[test]
fn count_tags_no_matches() {
    let text = "fn main() { println!(\"hello\"); }";
    let result = count_tags(text, &["TODO", "FIXME"]);
    assert!(result.iter().all(|(_, c)| *c == 0));
}

#[test]
fn count_tags_empty_tag_list() {
    let result = count_tags("TODO: something", &[]);
    assert!(result.is_empty());
}

// ── binary detection ─────────────────────────────────────────────────

#[test]
fn is_text_like_empty_is_text() {
    assert!(is_text_like(&[]), "empty content should be text-like");
}

#[test]
fn is_text_like_valid_utf8_is_text() {
    assert!(is_text_like(b"Hello, world!"));
}

#[test]
fn is_text_like_null_bytes_is_binary() {
    let data = b"Hello\x00World";
    assert!(
        !is_text_like(data),
        "null bytes should be detected as binary"
    );
}

#[test]
fn is_text_like_pure_ascii_is_text() {
    let data: Vec<u8> = (32..127).collect();
    assert!(is_text_like(&data));
}

// ── read_head edge cases ─────────────────────────────────────────────

#[test]
fn read_head_empty_file() {
    let f = NamedTempFile::new().unwrap();
    let data = read_head(f.path(), 1024).unwrap();
    assert!(data.is_empty());
}

#[test]
fn read_head_caps_at_max_bytes() {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(&[b'A'; 1000]).unwrap();
    f.flush().unwrap();
    let data = read_head(f.path(), 100).unwrap();
    assert_eq!(data.len(), 100);
}

#[test]
fn read_head_nonexistent_path_returns_error() {
    let result = read_head(std::path::Path::new("__nonexistent_w53__"), 1024);
    assert!(result.is_err());
}

// ── read_head_tail edge cases ────────────────────────────────────────

#[test]
fn read_head_tail_empty_file() {
    let f = NamedTempFile::new().unwrap();
    let data = read_head_tail(f.path(), 1024).unwrap();
    assert!(data.is_empty());
}

#[test]
fn read_head_tail_small_file_returns_all() {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(b"short").unwrap();
    f.flush().unwrap();
    let data = read_head_tail(f.path(), 1024).unwrap();
    assert_eq!(data, b"short");
}

// ── read_lines edge cases ────────────────────────────────────────────

#[test]
fn read_lines_empty_file() {
    let f = NamedTempFile::new().unwrap();
    let lines = read_lines(f.path(), 100, 4096).unwrap();
    assert!(lines.is_empty());
}

#[test]
fn read_lines_respects_max_lines() {
    let mut f = NamedTempFile::new().unwrap();
    for i in 0..20 {
        writeln!(f, "line {i}").unwrap();
    }
    f.flush().unwrap();
    let lines = read_lines(f.path(), 5, 4096).unwrap();
    assert!(lines.len() <= 5, "should respect max_lines limit");
}

// ── read_text_capped edge cases ──────────────────────────────────────

#[test]
fn read_text_capped_empty_file() {
    let f = NamedTempFile::new().unwrap();
    let text = read_text_capped(f.path(), 4096).unwrap();
    assert!(text.is_empty());
}

// ── hashing ──────────────────────────────────────────────────────────

#[test]
fn hash_bytes_deterministic() {
    let h1 = hash_bytes(b"hello");
    let h2 = hash_bytes(b"hello");
    assert_eq!(h1, h2);
}

#[test]
fn hash_bytes_different_content_different_hash() {
    let h1 = hash_bytes(b"hello");
    let h2 = hash_bytes(b"world");
    assert_ne!(h1, h2);
}

#[test]
fn hash_bytes_empty_input() {
    let h = hash_bytes(b"");
    assert!(!h.is_empty(), "empty input should still produce a hash");
}

#[test]
fn hash_file_nonexistent_returns_error() {
    let result = hash_file(std::path::Path::new("__nonexistent_w53__"), 4096);
    assert!(result.is_err());
}

#[test]
fn hash_file_empty_file() {
    let f = NamedTempFile::new().unwrap();
    let h = hash_file(f.path(), 4096).unwrap();
    assert!(!h.is_empty());
}
