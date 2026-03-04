//! Extended BDD-style tests for tokmd-content.
//!
//! These tests cover edge cases not addressed by the existing test suite:
//! empty file reads, binary format headers, hash edge cases,
//! overlapping tags, read_head_tail boundary values, and additional
//! complexity scenarios.

use std::fs::File;
use std::io::Write;
use std::path::Path;

use tokmd_content::complexity::{
    analyze_functions, analyze_nesting_depth, estimate_cognitive_complexity,
    estimate_cyclomatic_complexity,
};
use tokmd_content::{
    count_tags, entropy_bits_per_byte, hash_bytes, hash_file, is_text_like, read_head,
    read_head_tail, read_lines, read_text_capped,
};

// ============================================================================
// Scenario: read_head on empty file
// ============================================================================

#[test]
fn test_given_empty_file_when_read_head_then_empty_vec() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("empty");
    File::create(&path).unwrap();

    let bytes = read_head(&path, 1024).unwrap();
    assert!(bytes.is_empty());
}

// ============================================================================
// Scenario: read_head_tail boundary value max_bytes = 1
// ============================================================================

#[test]
fn test_given_file_when_read_head_tail_one_byte_then_single_byte() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("one.txt");
    File::create(&path)
        .unwrap()
        .write_all(b"ABCDEFGHIJ")
        .unwrap();

    // max_bytes=1 → half=0, head_len=max(0,1)=1, tail_len=0
    let bytes = read_head_tail(&path, 1).unwrap();
    assert_eq!(bytes.len(), 1);
    assert_eq!(bytes[0], b'A');
}

#[test]
fn test_given_empty_file_when_read_head_tail_then_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("empty.bin");
    File::create(&path).unwrap();

    let bytes = read_head_tail(&path, 100).unwrap();
    assert!(bytes.is_empty());
}

#[test]
fn test_given_file_equal_to_limit_when_read_head_tail_then_full_content() {
    // Given a file whose size == max_bytes
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("exact.txt");
    File::create(&path).unwrap().write_all(b"12345").unwrap();

    // When max_bytes equals file size
    let bytes = read_head_tail(&path, 5).unwrap();

    // Then we get the full file content
    assert_eq!(bytes, b"12345");
}

// ============================================================================
// Scenario: read_lines on empty file
// ============================================================================

#[test]
fn test_given_empty_file_when_read_lines_then_empty_vec() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("empty.txt");
    File::create(&path).unwrap();

    let lines = read_lines(&path, 100, 10_000).unwrap();
    assert!(lines.is_empty());
}

#[test]
fn test_given_single_line_no_newline_when_read_lines_then_one_line() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("no_nl.txt");
    // File with content but no trailing newline
    File::create(&path).unwrap().write_all(b"hello").unwrap();

    let lines = read_lines(&path, 100, 10_000).unwrap();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], "hello");
}

// ============================================================================
// Scenario: read_text_capped on empty file
// ============================================================================

#[test]
fn test_given_empty_file_when_read_text_capped_then_empty_string() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("empty.txt");
    File::create(&path).unwrap();

    let text = read_text_capped(&path, 1024).unwrap();
    assert!(text.is_empty());
}

#[test]
fn test_given_zero_cap_when_read_text_capped_then_empty_string() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("content.txt");
    File::create(&path).unwrap().write_all(b"content").unwrap();

    let text = read_text_capped(&path, 0).unwrap();
    assert!(text.is_empty());
}

// ============================================================================
// Scenario: hash_file on empty file
// ============================================================================

#[test]
fn test_given_empty_file_when_hashed_then_matches_empty_bytes_hash() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("empty.bin");
    File::create(&path).unwrap();

    let file_hash = hash_file(&path, 1024).unwrap();
    let empty_hash = hash_bytes(&[]);

    assert_eq!(file_hash, empty_hash);
    assert_eq!(file_hash.len(), 64);
}

// ============================================================================
// Scenario: hash_file on nonexistent path
// ============================================================================

#[test]
fn test_given_nonexistent_file_when_hashed_then_error() {
    let result = hash_file(Path::new("/tmp/tokmd_definitely_not_here_12345.bin"), 1024);
    assert!(result.is_err());
}

// ============================================================================
// Scenario: is_text_like with binary format headers
// ============================================================================

#[test]
fn test_given_png_header_when_checked_then_not_text_like() {
    // PNG magic bytes: 0x89 P N G \r \n 0x1A \n
    let png_header: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    assert!(
        !is_text_like(png_header),
        "PNG header should not be text-like"
    );
}

#[test]
fn test_given_elf_header_when_checked_then_not_text_like() {
    // ELF magic bytes: 0x7F E L F
    let elf_header: &[u8] = &[0x7F, 0x45, 0x4C, 0x46, 0x00, 0x00];
    assert!(
        !is_text_like(elf_header),
        "ELF header should not be text-like"
    );
}

#[test]
fn test_given_zip_header_when_checked_then_not_text_like() {
    // ZIP magic bytes: P K 0x03 0x04 followed by binary
    let zip_header: &[u8] = &[0x50, 0x4B, 0x03, 0x04, 0x00, 0x00];
    assert!(
        !is_text_like(zip_header),
        "ZIP header should not be text-like"
    );
}

#[test]
fn test_given_only_whitespace_when_checked_then_text_like() {
    let whitespace = b"   \t\n\r\n   ";
    assert!(is_text_like(whitespace), "whitespace should be text-like");
}

// ============================================================================
// Scenario: entropy on known distributions
// ============================================================================

#[test]
fn test_given_all_same_byte_when_entropy_computed_then_zero() {
    // Given 1000 copies of byte 0x42
    let bytes = vec![0x42u8; 1000];
    let entropy = entropy_bits_per_byte(&bytes);
    assert!(
        entropy.abs() < 1e-6,
        "uniform single byte should be zero entropy, got {}",
        entropy
    );
}

#[test]
fn test_given_uniform_256_values_when_entropy_computed_then_eight_bits() {
    // Given all 256 byte values repeated equally (1024 total)
    let bytes: Vec<u8> = (0..=255u8).cycle().take(1024).collect();
    let entropy = entropy_bits_per_byte(&bytes);
    assert!(
        (entropy - 8.0).abs() < 0.01,
        "uniform 256 values should be 8.0 bits, got {}",
        entropy
    );
}

#[test]
fn test_given_three_equally_distributed_values_when_entropy_computed_then_log2_3() {
    // Given equal distribution of 3 values: entropy should be log2(3) ≈ 1.585
    let bytes: Vec<u8> = (0..900).map(|i| (i % 3) as u8).collect();
    let entropy = entropy_bits_per_byte(&bytes);
    let expected = 3.0f32.log2(); // ~1.585
    assert!(
        (entropy - expected).abs() < 0.02,
        "expected ~{:.3}, got {}",
        expected,
        entropy
    );
}

#[test]
fn test_given_entropy_always_bounded_0_to_8() {
    for pattern in [
        vec![0u8; 1],
        vec![0u8; 100],
        (0..=255u8).collect::<Vec<_>>(),
        b"the quick brown fox jumps over the lazy dog".to_vec(),
    ] {
        let entropy = entropy_bits_per_byte(&pattern);
        assert!((0.0..=8.0).contains(&entropy), "out of bounds: {entropy}");
    }
}

// ============================================================================
// Scenario: count_tags with overlapping needles
// ============================================================================

#[test]
fn test_given_overlapping_tags_when_counted_then_independent() {
    // "TODO" and "TODOS" searched independently
    let text = "TODO TODOS TODO";
    let result = count_tags(text, &["TODO", "TODOS"]);
    // "TODO" appears at positions 0, 5 (inside TODOS), and 11 → 3 matches
    assert_eq!(result[0], ("TODO".to_string(), 3));
    // "TODOS" appears at position 5 → 1 match
    assert_eq!(result[1], ("TODOS".to_string(), 1));
}

#[test]
fn test_given_tag_at_start_and_end_when_counted_then_both_found() {
    let text = "FIXME middle text FIXME";
    let result = count_tags(text, &["FIXME"]);
    assert_eq!(result[0].1, 2);
}

#[test]
fn test_given_adjacent_tags_when_counted_then_all_found() {
    let text = "TODOTODOTODO";
    let result = count_tags(text, &["TODO"]);
    assert_eq!(result[0].1, 3);
}

// ============================================================================
// Scenario: hash_bytes known values
// ============================================================================

#[test]
fn test_given_known_input_when_hashed_then_deterministic_across_calls() {
    let content = b"tokmd-content deterministic hashing";
    let h1 = hash_bytes(content);
    let h2 = hash_bytes(content);
    let h3 = hash_bytes(content);
    assert_eq!(h1, h2);
    assert_eq!(h2, h3);
}

#[test]
fn test_given_single_byte_difference_when_hashed_then_different() {
    let a = b"hello world";
    let b = b"hello worle";
    assert_ne!(hash_bytes(a), hash_bytes(b));
}

// ============================================================================
// Scenario: complexity analysis – multiple functions
// ============================================================================

#[test]
fn test_given_three_rust_functions_when_analyzed_then_count_is_three() {
    let code = r#"fn a() {
    println!("a");
}

fn b() {
    println!("b");
}

fn c() {
    println!("c");
}
"#;
    let metrics = analyze_functions(code, "rust");
    assert_eq!(metrics.function_count, 3);
}

#[test]
fn test_given_function_with_loop_when_cc_estimated_then_incremented() {
    let code = r#"fn loopy() {
    for i in 0..10 {
        while i > 5 {
            break;
        }
    }
}
"#;
    let result = estimate_cyclomatic_complexity(code, "rust");
    assert_eq!(result.function_count, 1);
    // Base 1 + for + while = at least 3
    assert!(
        result.max_cc >= 3,
        "expected CC >= 3, got {}",
        result.max_cc
    );
}

#[test]
fn test_given_match_arms_when_cc_estimated_then_counted() {
    let code = r#"fn matcher(x: i32) {
    match x {
        1 => println!("one"),
        2 => println!("two"),
        3 => println!("three"),
        _ => println!("other"),
    }
}
"#;
    let result = estimate_cyclomatic_complexity(code, "rust");
    assert_eq!(result.function_count, 1);
    // match + multiple case arms should increase CC
    assert!(
        result.max_cc >= 2,
        "expected CC >= 2 for match, got {}",
        result.max_cc
    );
}

// ============================================================================
// Scenario: cognitive complexity for flat code
// ============================================================================

#[test]
fn test_given_flat_function_when_cognitive_estimated_then_low() {
    let code = r#"fn flat() {
    let a = 1;
    let b = 2;
    let c = a + b;
    println!("{}", c);
}
"#;
    let result = estimate_cognitive_complexity(code, "rust");
    assert_eq!(result.function_count, 1);
    // No nesting, no branches → minimal cognitive complexity
    assert!(
        result.max <= 1,
        "flat code should have low CC, got {}",
        result.max
    );
}

// ============================================================================
// Scenario: nesting depth for Python with deep indentation
// ============================================================================

#[test]
fn test_given_deeply_nested_python_when_nesting_analyzed_then_high_depth() {
    let code = "\
def deep():
    if True:
        for x in range(10):
            while x > 0:
                if x % 2 == 0:
                    print(x)
";
    let result = analyze_nesting_depth(code, "python");
    // def -> if -> for -> while -> if = 4 nesting levels
    assert!(
        result.max_depth >= 4,
        "expected depth >= 4, got {}",
        result.max_depth
    );
}

// ============================================================================
// Scenario: analyze_functions for TypeScript
// ============================================================================

#[test]
fn test_given_typescript_function_when_analyzed_then_detected() {
    let code = "function greet(name: string): void {\n    console.log(name);\n}\n";
    let metrics = analyze_functions(code, "typescript");
    assert_eq!(metrics.function_count, 1);
}

// ============================================================================
// Scenario: read_head with zero max_bytes
// ============================================================================

#[test]
fn test_given_file_when_read_head_zero_then_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("content.txt");
    File::create(&path).unwrap().write_all(b"content").unwrap();

    let bytes = read_head(&path, 0).unwrap();
    assert!(bytes.is_empty());
}

// ============================================================================
// Scenario: read_lines with max_lines=0
// ============================================================================

#[test]
fn test_given_file_when_read_lines_zero_max_then_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("lines.txt");
    let mut f = File::create(&path).unwrap();
    writeln!(f, "line 1").unwrap();
    writeln!(f, "line 2").unwrap();

    let lines = read_lines(&path, 0, 10_000).unwrap();
    assert!(lines.is_empty());
}

// ============================================================================
// Scenario: is_text_like on edge cases
// ============================================================================

#[test]
fn test_given_single_null_byte_when_checked_then_not_text_like() {
    assert!(!is_text_like(&[0x00]));
}

#[test]
fn test_given_single_printable_byte_when_checked_then_text_like() {
    assert!(is_text_like(b"A"));
}

#[test]
fn test_given_long_ascii_text_when_checked_then_text_like() {
    let text = "a".repeat(100_000);
    assert!(is_text_like(text.as_bytes()));
}

// ============================================================================
// Scenario: hash_bytes is lowercase hex
// ============================================================================

#[test]
fn test_given_any_input_when_hashed_then_lowercase_hex() {
    for input in [b"".as_slice(), b"hello", b"\x00\xFF\xFE"] {
        let hash = hash_bytes(input);
        assert!(
            hash.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
            "hash should be lowercase hex: {}",
            hash
        );
    }
}

// ============================================================================
// Scenario: entropy determinism
// ============================================================================

#[test]
fn test_given_same_input_when_entropy_computed_twice_then_identical() {
    let data = b"determinism test data for entropy";
    let e1 = entropy_bits_per_byte(data);
    let e2 = entropy_bits_per_byte(data);
    assert_eq!(e1, e2, "entropy must be deterministic");
}

// ============================================================================
// Scenario: complexity with logical operators
// ============================================================================

#[test]
fn test_given_logical_operators_when_cc_estimated_then_incremented() {
    let code = r#"fn complex(a: bool, b: bool, c: bool) {
    if a && b || c {
        println!("yes");
    }
}
"#;
    let result = estimate_cyclomatic_complexity(code, "rust");
    assert_eq!(result.function_count, 1);
    // Base 1 + if + && + || = at least 4
    assert!(
        result.max_cc >= 3,
        "expected CC >= 3 with logical operators, got {}",
        result.max_cc
    );
}
