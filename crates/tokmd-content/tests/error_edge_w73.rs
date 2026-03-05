//! Error handling and edge case tests for tokmd-content (W73).
//!
//! Tests empty files, binary files, very large lines, entropy edge cases,
//! complexity analysis on non-code input, and boundary conditions.

use std::fs::File;
use std::io::Write;
use tokmd_content::complexity::{
    analyze_functions, analyze_nesting_depth, estimate_cognitive_complexity,
    estimate_cyclomatic_complexity,
};
use tokmd_content::{
    count_tags, entropy_bits_per_byte, hash_bytes, hash_file, is_text_like, read_head,
    read_head_tail, read_lines, read_text_capped,
};

// =============================================================================
// Empty file edge cases
// =============================================================================

#[test]
fn read_head_on_empty_file_returns_empty_vec() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("empty.txt");
    File::create(&path).unwrap();

    let bytes = read_head(&path, 1024).unwrap();
    assert!(bytes.is_empty());
}

#[test]
fn read_head_tail_on_empty_file_returns_empty_vec() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("empty.txt");
    File::create(&path).unwrap();

    let bytes = read_head_tail(&path, 1024).unwrap();
    assert!(bytes.is_empty());
}

#[test]
fn read_lines_on_empty_file_returns_empty_vec() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("empty.txt");
    File::create(&path).unwrap();

    let lines = read_lines(&path, 100, 10000).unwrap();
    assert!(lines.is_empty());
}

#[test]
fn read_text_capped_on_empty_file_returns_empty_string() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("empty.txt");
    File::create(&path).unwrap();

    let text = read_text_capped(&path, 1024).unwrap();
    assert!(text.is_empty());
}

#[test]
fn hash_file_on_empty_file_returns_blake3_of_empty() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("empty.txt");
    File::create(&path).unwrap();

    let hash = hash_file(&path, 1024).unwrap();
    let expected = hash_bytes(&[]);
    assert_eq!(hash, expected);
    assert_eq!(hash.len(), 64); // BLAKE3 hex length
}

// =============================================================================
// Binary file handling
// =============================================================================

#[test]
fn is_text_like_detects_binary_with_null_bytes() {
    let binary = vec![0x00, 0x01, 0x02, 0xFF, 0xFE];
    assert!(!is_text_like(&binary));
}

#[test]
fn is_text_like_accepts_valid_utf8() {
    assert!(is_text_like(b"Hello, world!"));
    assert!(is_text_like("café résumé".as_bytes()));
}

#[test]
fn is_text_like_empty_input_is_text() {
    assert!(is_text_like(&[]));
}

#[test]
fn read_head_on_binary_file_returns_raw_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("binary.bin");
    let mut f = File::create(&path).unwrap();
    let binary_data: Vec<u8> = (0..=255).collect();
    f.write_all(&binary_data).unwrap();

    let bytes = read_head(&path, 256).unwrap();
    assert_eq!(bytes.len(), 256);
    assert_eq!(bytes[0], 0x00);
    assert_eq!(bytes[255], 0xFF);
}

#[test]
fn read_text_capped_on_binary_file_uses_lossy_conversion() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("binary.bin");
    let mut f = File::create(&path).unwrap();
    f.write_all(&[0xFF, 0xFE, 0x00, 0x41]).unwrap(); // invalid UTF-8 + 'A'

    let text = read_text_capped(&path, 1024).unwrap();
    // from_utf8_lossy replaces invalid sequences with U+FFFD
    assert!(text.contains('\u{FFFD}') || text.contains('A'));
}

// =============================================================================
// Very large lines
// =============================================================================

#[test]
fn read_lines_with_very_long_line_respects_byte_limit() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("long_line.txt");
    let mut f = File::create(&path).unwrap();
    let long_line = "x".repeat(10_000);
    writeln!(f, "{}", long_line).unwrap();
    writeln!(f, "short").unwrap();

    let lines = read_lines(&path, 100, 5000).unwrap();
    assert_eq!(
        lines.len(),
        1,
        "byte limit should stop after first long line"
    );
    assert_eq!(lines[0].len(), 10_000);
}

#[test]
fn read_head_with_zero_max_bytes_returns_empty() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.txt");
    let mut f = File::create(&path).unwrap();
    f.write_all(b"content").unwrap();

    let bytes = read_head(&path, 0).unwrap();
    assert!(bytes.is_empty());
}

#[test]
fn read_head_tail_with_zero_max_bytes_returns_empty() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.txt");
    let mut f = File::create(&path).unwrap();
    f.write_all(b"content").unwrap();

    let bytes = read_head_tail(&path, 0).unwrap();
    assert!(bytes.is_empty());
}

#[test]
fn read_lines_with_zero_max_lines_returns_empty() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.txt");
    let mut f = File::create(&path).unwrap();
    writeln!(f, "line1").unwrap();

    let lines = read_lines(&path, 0, 10000).unwrap();
    assert!(lines.is_empty());
}

// =============================================================================
// Entropy edge cases
// =============================================================================

#[test]
fn entropy_empty_input_returns_zero() {
    assert_eq!(entropy_bits_per_byte(&[]), 0.0);
}

#[test]
fn entropy_single_byte_returns_zero() {
    assert_eq!(entropy_bits_per_byte(&[42]), 0.0);
}

#[test]
fn entropy_all_same_bytes_returns_zero() {
    let data = vec![0xAB; 10_000];
    let e = entropy_bits_per_byte(&data);
    assert!(
        e.abs() < 1e-6,
        "uniform bytes should have ~0 entropy, got {e}"
    );
}

#[test]
fn entropy_maximum_for_uniform_256_values() {
    let data: Vec<u8> = (0..=255).cycle().take(256 * 100).collect();
    let e = entropy_bits_per_byte(&data);
    assert!(
        (e - 8.0).abs() < 0.01,
        "uniform 256 values should yield ~8.0 bits, got {e}"
    );
}

// =============================================================================
// Non-existent file errors
// =============================================================================

#[test]
fn read_head_nonexistent_file_returns_error() {
    let result = read_head(std::path::Path::new("/tmp/tokmd_w73_no_such_file.txt"), 100);
    assert!(result.is_err());
}

#[test]
fn hash_file_nonexistent_returns_error() {
    let result = hash_file(std::path::Path::new("/tmp/tokmd_w73_no_such_file.txt"), 100);
    assert!(result.is_err());
}

// =============================================================================
// count_tags edge cases
// =============================================================================

#[test]
fn count_tags_empty_text_returns_all_zeros() {
    let result = count_tags("", &["TODO", "FIXME", "HACK"]);
    assert_eq!(result.len(), 3);
    for (_, count) in &result {
        assert_eq!(*count, 0);
    }
}

#[test]
fn count_tags_empty_tag_list_returns_empty() {
    let result = count_tags("TODO FIXME HACK", &[]);
    assert!(result.is_empty());
}

#[test]
fn count_tags_case_insensitive() {
    let text = "todo Todo TODO tOdO";
    let result = count_tags(text, &["TODO"]);
    assert_eq!(result[0].1, 4, "count_tags should be case-insensitive");
}

// =============================================================================
// Complexity analysis on non-code / edge inputs
// =============================================================================

#[test]
fn analyze_functions_empty_string_returns_defaults() {
    let m = analyze_functions("", "rust");
    assert_eq!(m.function_count, 0);
    assert_eq!(m.max_function_length, 0);
    assert_eq!(m.avg_function_length, 0.0);
}

#[test]
fn analyze_functions_unknown_language_returns_defaults() {
    let code = "function foo() { return 42; }";
    let m = analyze_functions(code, "brainfuck");
    assert_eq!(m.function_count, 0);
}

#[test]
fn cyclomatic_complexity_empty_string_returns_defaults() {
    let cc = estimate_cyclomatic_complexity("", "rust");
    assert_eq!(cc.function_count, 0);
    assert_eq!(cc.total_cc, 0);
    assert_eq!(cc.max_cc, 0);
    assert_eq!(cc.avg_cc, 0.0);
}

#[test]
fn cognitive_complexity_empty_string_returns_defaults() {
    let cc = estimate_cognitive_complexity("", "rust");
    assert_eq!(cc.function_count, 0);
    assert_eq!(cc.total, 0);
    assert_eq!(cc.max, 0);
}

#[test]
fn nesting_depth_empty_string_returns_defaults() {
    let n = analyze_nesting_depth("", "rust");
    assert_eq!(n.max_depth, 0);
    assert_eq!(n.avg_depth, 0.0);
    assert!(n.max_depth_lines.is_empty());
}

#[test]
fn complexity_on_plain_text_returns_no_functions() {
    let prose = "This is just plain English text.\nNo functions here.\nJust sentences.";
    let m = analyze_functions(prose, "rust");
    assert_eq!(m.function_count, 0);

    let cc = estimate_cyclomatic_complexity(prose, "python");
    assert_eq!(cc.function_count, 0);
}
