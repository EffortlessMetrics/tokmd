//! Deep tests for tokmd-analysis content helpers (wave 43).
//!
//! Covers read_lines edge cases, tag extraction, import parsing,
//! hash computation, entropy calculation, and is_text_like.

use crate::content::io::{
    count_tags, entropy_bits_per_byte, hash_bytes, hash_file, is_text_like, read_head,
    read_head_tail, read_lines, read_text_capped,
};
use std::fs::File;
use std::io::Write;
use std::path::Path;

// ============================================================================
// 1. read_lines — edge cases
// ============================================================================

#[test]
fn read_lines_max_bytes_zero_returns_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("data.txt");
    let mut f = File::create(&path).unwrap();
    writeln!(f, "line 1").unwrap();
    let lines = read_lines(&path, 100, 0).unwrap();
    assert!(lines.is_empty(), "max_bytes=0 should return empty Vec");
}

#[test]
fn read_lines_preserves_leading_whitespace() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("indent.txt");
    let mut f = File::create(&path).unwrap();
    writeln!(f, "    indented").unwrap();
    writeln!(f, "\ttabbed").unwrap();
    let lines = read_lines(&path, 10, 10_000).unwrap();
    assert_eq!(lines[0], "    indented");
    assert_eq!(lines[1], "\ttabbed");
}

#[test]
fn read_lines_preserves_trailing_spaces() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("trailing.txt");
    let mut f = File::create(&path).unwrap();
    // BufReader::lines() strips the newline, but NOT trailing spaces
    write!(f, "hello   \nworld\n").unwrap();
    let lines = read_lines(&path, 10, 10_000).unwrap();
    assert_eq!(lines[0], "hello   ");
    assert_eq!(lines[1], "world");
}

#[test]
fn read_lines_empty_lines_counted() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("blanks.txt");
    let mut f = File::create(&path).unwrap();
    write!(f, "a\n\n\nb\n").unwrap();
    let lines = read_lines(&path, 100, 10_000).unwrap();
    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0], "a");
    assert_eq!(lines[1], "");
    assert_eq!(lines[2], "");
    assert_eq!(lines[3], "b");
}

#[test]
fn read_lines_max_one_line() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("multi.txt");
    let mut f = File::create(&path).unwrap();
    writeln!(f, "first").unwrap();
    writeln!(f, "second").unwrap();
    writeln!(f, "third").unwrap();
    let lines = read_lines(&path, 1, 10_000).unwrap();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], "first");
}

// ============================================================================
// 2. read_head / read_head_tail — edge cases
// ============================================================================

#[test]
fn read_head_max_bytes_zero_returns_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("data.txt");
    File::create(&path).unwrap().write_all(b"hello").unwrap();
    let bytes = read_head(&path, 0).unwrap();
    assert!(bytes.is_empty());
}

#[test]
fn read_head_tail_max_bytes_one() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("data.txt");
    File::create(&path)
        .unwrap()
        .write_all(b"abcdefghij")
        .unwrap();
    // max_bytes=1, half=0, head_len=max(0,1)=1, tail_len=1-1=0
    let bytes = read_head_tail(&path, 1).unwrap();
    assert_eq!(bytes.len(), 1);
    assert_eq!(bytes[0], b'a');
}

#[test]
fn read_head_tail_exact_file_size() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("exact.txt");
    File::create(&path).unwrap().write_all(b"12345").unwrap();
    // When max_bytes >= file size, returns full content
    let bytes = read_head_tail(&path, 5).unwrap();
    assert_eq!(bytes, b"12345");
}

// ============================================================================
// 3. count_tags — tag extraction
// ============================================================================

#[test]
fn count_tags_multiple_occurrences_per_line() {
    let text = "TODO: first TODO: second TODO: third";
    let tags = count_tags(text, &["TODO"]);
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].1, 3);
}

#[test]
fn count_tags_mixed_case_matching() {
    let text = "todo fixme Todo FIXME FiXmE";
    let tags = count_tags(text, &["TODO", "FIXME"]);
    // "todo" and "Todo" both match "TODO" (case-insensitive)
    assert_eq!(tags[0].1, 2); // TODO
    assert_eq!(tags[1].1, 3); // FIXME
}

#[test]
fn count_tags_hack_and_safety() {
    let text = "// HACK: workaround\n// SAFETY: validated\n// HACK again";
    let tags = count_tags(text, &["HACK", "SAFETY"]);
    assert_eq!(tags[0].1, 2); // HACK
    assert_eq!(tags[1].1, 1); // SAFETY
}

#[test]
fn count_tags_with_custom_tags() {
    let text = "PERF: optimize this\nDEPRECATED: remove later\nPERF again";
    let tags = count_tags(text, &["PERF", "DEPRECATED"]);
    assert_eq!(tags[0].1, 2);
    assert_eq!(tags[1].1, 1);
}

#[test]
fn count_tags_no_matches_returns_zeros() {
    let text = "clean code with no markers";
    let tags = count_tags(text, &["TODO", "FIXME", "HACK"]);
    for (_, count) in &tags {
        assert_eq!(*count, 0);
    }
}

#[test]
fn count_tags_empty_text_all_zeros() {
    let tags = count_tags("", &["TODO", "FIXME"]);
    assert_eq!(tags[0].1, 0);
    assert_eq!(tags[1].1, 0);
}

#[test]
fn count_tags_preserves_tag_name_in_output() {
    let text = "TODO item";
    let tags = count_tags(text, &["TODO", "FIXME", "HACK"]);
    assert_eq!(tags[0].0, "TODO");
    assert_eq!(tags[1].0, "FIXME");
    assert_eq!(tags[2].0, "HACK");
}

// ============================================================================
// 4. hash_bytes / hash_file
// ============================================================================

#[test]
fn hash_bytes_empty_is_valid_hex() {
    let h = hash_bytes(b"");
    assert_eq!(h.len(), 64);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn hash_bytes_single_bit_difference() {
    let h1 = hash_bytes(&[0x00]);
    let h2 = hash_bytes(&[0x01]);
    assert_ne!(
        h1, h2,
        "Single bit difference must produce different hashes"
    );
}

#[test]
fn hash_file_binary_content() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("binary.bin");
    let data: Vec<u8> = (0..=255).collect();
    File::create(&path).unwrap().write_all(&data).unwrap();
    let h = hash_file(&path, 1000).unwrap();
    let expected = hash_bytes(&data);
    assert_eq!(h, expected);
}

#[test]
fn hash_file_cap_smaller_than_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("large.txt");
    let content = "a".repeat(500);
    File::create(&path)
        .unwrap()
        .write_all(content.as_bytes())
        .unwrap();
    let h_capped = hash_file(&path, 100).unwrap();
    let h_full = hash_file(&path, 1000).unwrap();
    assert_ne!(h_capped, h_full, "Capped hash should differ from full");
    assert_eq!(h_capped, hash_bytes(&content.as_bytes()[..100]));
}

// ============================================================================
// 5. entropy_bits_per_byte
// ============================================================================

#[test]
fn entropy_all_same_byte_is_zero() {
    let data = vec![42u8; 1000];
    let e = entropy_bits_per_byte(&data);
    assert!(
        e.abs() < 0.001,
        "All same byte should have ~0 entropy, got {}",
        e
    );
}

#[test]
fn entropy_two_values_equal_frequency_is_one() {
    let mut data = Vec::new();
    for _ in 0..500 {
        data.push(0u8);
        data.push(1u8);
    }
    let e = entropy_bits_per_byte(&data);
    assert!(
        (e - 1.0).abs() < 0.01,
        "Two equally frequent bytes should have ~1.0 bit entropy, got {}",
        e
    );
}

#[test]
fn entropy_increases_with_distinct_values() {
    let e1 = entropy_bits_per_byte(&[0, 0, 0, 0, 1, 1, 1, 1]);
    let e2 = entropy_bits_per_byte(&[0, 0, 1, 1, 2, 2, 3, 3]);
    assert!(
        e2 > e1,
        "More distinct values should have higher entropy: {} vs {}",
        e2,
        e1
    );
}

#[test]
fn entropy_ascii_text_moderate() {
    let text = b"The quick brown fox jumps over the lazy dog. 1234567890!";
    let e = entropy_bits_per_byte(text);
    assert!(e > 3.0, "English text entropy should be >3 bits, got {}", e);
    assert!(e < 8.0, "Entropy cannot exceed 8 bits, got {}", e);
}

// ============================================================================
// 6. is_text_like
// ============================================================================

#[test]
fn is_text_like_utf8_emoji() {
    assert!(is_text_like("Hello 🌍".as_bytes()));
}

#[test]
fn is_text_like_null_in_middle() {
    assert!(!is_text_like(b"hello\x00world"));
}

#[test]
fn is_text_like_only_whitespace() {
    assert!(is_text_like(b"   \t\n\r  "));
}

// ============================================================================
// 7. read_text_capped
// ============================================================================

#[test]
fn read_text_capped_zero_returns_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("data.txt");
    File::create(&path).unwrap().write_all(b"content").unwrap();
    let text = read_text_capped(&path, 0).unwrap();
    assert!(text.is_empty());
}

#[test]
fn read_text_capped_nonexistent_file_errors() {
    let result = read_text_capped(Path::new("nonexistent_w43_file.txt"), 100);
    assert!(result.is_err());
}
