//! Depth tests for tokmd-analysis content helpers (wave 63).
//!
//! Covers: entropy accuracy, import extraction patterns, TODO/FIXME counting
//! edge cases, BLAKE3 hashing determinism, empty file handling, binary detection,
//! large file handling, and property-based invariants.

use crate::content::complexity::{
    analyze_functions, analyze_nesting_depth, estimate_cognitive_complexity,
    estimate_cyclomatic_complexity,
};
use crate::content::io::{
    count_tags, entropy_bits_per_byte, hash_bytes, hash_file, is_text_like, read_head,
    read_head_tail, read_lines, read_text_capped,
};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

fn write_file(dir: &tempfile::TempDir, name: &str, content: &[u8]) -> PathBuf {
    let p = dir.path().join(name);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let mut f = File::create(&p).unwrap();
    f.write_all(content).unwrap();
    p
}

// ============================================================================
// 1. Entropy calculation accuracy
// ============================================================================

#[test]
fn entropy_empty_is_zero() {
    assert_eq!(entropy_bits_per_byte(&[]), 0.0);
}

#[test]
fn entropy_uniform_single_byte_is_zero() {
    assert_eq!(entropy_bits_per_byte(&[0xAA; 1000]), 0.0);
}

#[test]
fn entropy_two_values_equally_distributed() {
    let data: Vec<u8> = (0..1000).map(|i| if i % 2 == 0 { 0 } else { 1 }).collect();
    let e = entropy_bits_per_byte(&data);
    assert!((e - 1.0).abs() < 0.01, "expected ~1.0 bit, got {e}");
}

#[test]
fn entropy_four_values_equally_distributed() {
    let data: Vec<u8> = (0..1000).map(|i| (i % 4) as u8).collect();
    let e = entropy_bits_per_byte(&data);
    assert!((e - 2.0).abs() < 0.05, "expected ~2.0 bits, got {e}");
}

#[test]
fn entropy_uniform_256_near_eight() {
    let mut data = Vec::with_capacity(256 * 100);
    for _ in 0..100 {
        for b in 0u8..=255 {
            data.push(b);
        }
    }
    let e = entropy_bits_per_byte(&data);
    assert!((e - 8.0).abs() < 0.01, "expected ~8.0, got {e}");
}

#[test]
fn entropy_single_byte_input() {
    // Single byte → only one value → 0 entropy
    assert_eq!(entropy_bits_per_byte(&[42]), 0.0);
}

#[test]
fn entropy_two_distinct_bytes() {
    let e = entropy_bits_per_byte(&[0, 1]);
    assert!((e - 1.0).abs() < 0.01, "expected ~1.0, got {e}");
}

#[test]
fn entropy_skewed_distribution() {
    // 90% zeros, 10% ones
    let mut data = vec![0u8; 900];
    data.extend(vec![1u8; 100]);
    let e = entropy_bits_per_byte(&data);
    // H = -0.9*log2(0.9) - 0.1*log2(0.1) ≈ 0.469
    assert!(e > 0.4 && e < 0.5, "expected ~0.469, got {e}");
}

#[test]
fn entropy_ascii_text_typical_range() {
    let text =
        b"The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor jugs.";
    let e = entropy_bits_per_byte(text);
    // Typical English text: 3.5-5.0 bits per byte
    assert!(e > 3.0 && e < 6.0, "text entropy should be 3-6, got {e}");
}

#[test]
fn entropy_monotonically_nondecreasing_with_more_values() {
    let e1 = entropy_bits_per_byte(&[0; 100]);
    let e2 = entropy_bits_per_byte(&{
        let mut d = vec![0u8; 50];
        d.extend(vec![1u8; 50]);
        d
    });
    assert!(e2 >= e1, "more distinct values should yield >= entropy");
}

// ============================================================================
// 2. Tag counting (TODO/FIXME) edge cases
// ============================================================================

#[test]
fn count_tags_basic() {
    let text = "// TODO: fix this\n// FIXME: broken\n";
    let tags = count_tags(text, &["TODO", "FIXME"]);
    assert_eq!(tags[0], ("TODO".to_string(), 1));
    assert_eq!(tags[1], ("FIXME".to_string(), 1));
}

#[test]
fn count_tags_case_insensitive() {
    let text = "// todo: lowercase\n// Todo: mixed\n// TODO: upper\n";
    let tags = count_tags(text, &["TODO"]);
    assert_eq!(tags[0].1, 3);
}

#[test]
fn count_tags_multiple_on_same_line() {
    let text = "// TODO TODO TODO\n";
    let tags = count_tags(text, &["TODO"]);
    assert_eq!(tags[0].1, 3);
}

#[test]
fn count_tags_no_matches() {
    let text = "fn main() { println!(\"hello\"); }";
    let tags = count_tags(text, &["TODO", "FIXME", "HACK"]);
    for (_, count) in &tags {
        assert_eq!(*count, 0);
    }
}

#[test]
fn count_tags_empty_text() {
    let tags = count_tags("", &["TODO", "FIXME"]);
    assert_eq!(tags[0].1, 0);
    assert_eq!(tags[1].1, 0);
}

#[test]
fn count_tags_empty_tag_list() {
    let tags = count_tags("TODO FIXME", &[]);
    assert!(tags.is_empty());
}

#[test]
fn count_tags_tag_in_string_literal() {
    let text = r#"let s = "TODO: this is in a string";"#;
    let tags = count_tags(text, &["TODO"]);
    // count_tags doesn't distinguish strings — it counts all occurrences
    assert_eq!(tags[0].1, 1);
}

#[test]
fn count_tags_custom_markers() {
    let text = "// HACK: workaround\n// XXX: dangerous\n// NOTE: important\n";
    let tags = count_tags(text, &["HACK", "XXX", "NOTE"]);
    assert_eq!(tags[0].1, 1); // HACK
    assert_eq!(tags[1].1, 1); // XXX
    assert_eq!(tags[2].1, 1); // NOTE
}

#[test]
fn count_tags_partial_match_counted() {
    // "TODOLIST" contains "TODO"
    let text = "TODOLIST is not a real tag\n";
    let tags = count_tags(text, &["TODO"]);
    // Substring match — "TODO" appears in "TODOLIST"
    assert_eq!(tags[0].1, 1);
}

// ============================================================================
// 3. BLAKE3 hashing determinism
// ============================================================================

#[test]
fn hash_bytes_deterministic() {
    let h1 = hash_bytes(b"hello world");
    let h2 = hash_bytes(b"hello world");
    assert_eq!(h1, h2);
}

#[test]
fn hash_bytes_always_64_hex_chars() {
    let h = hash_bytes(b"test");
    assert_eq!(h.len(), 64);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn hash_bytes_different_inputs_different_hashes() {
    let h1 = hash_bytes(b"alpha");
    let h2 = hash_bytes(b"beta");
    assert_ne!(h1, h2);
}

#[test]
fn hash_bytes_empty_input() {
    let h = hash_bytes(b"");
    assert_eq!(h.len(), 64);
    // Empty input still produces a valid hash
    assert!(!h.is_empty());
}

#[test]
fn hash_file_matches_hash_bytes() {
    let dir = tmp();
    let path = write_file(&dir, "test.txt", b"consistent content");
    let file_hash = hash_file(&path, 1000).unwrap();
    let bytes_hash = hash_bytes(b"consistent content");
    assert_eq!(file_hash, bytes_hash);
}

#[test]
fn hash_file_respects_max_bytes() {
    let dir = tmp();
    let path = write_file(&dir, "long.txt", b"abcdefghij");
    let h5 = hash_file(&path, 5).unwrap();
    let h10 = hash_file(&path, 10).unwrap();
    assert_ne!(
        h5, h10,
        "different byte limits should produce different hashes"
    );
    assert_eq!(h5, hash_bytes(b"abcde"));
}

#[test]
fn hash_file_nonexistent_errors() {
    let dir = tmp();
    let result = hash_file(&dir.path().join("nope.txt"), 1000);
    assert!(result.is_err());
}

// ============================================================================
// 4. Empty file handling
// ============================================================================

#[test]
fn read_head_empty_file() {
    let dir = tmp();
    let path = write_file(&dir, "empty.txt", b"");
    let bytes = read_head(&path, 100).unwrap();
    assert!(bytes.is_empty());
}

#[test]
fn read_head_tail_empty_file() {
    let dir = tmp();
    let path = write_file(&dir, "empty.txt", b"");
    let bytes = read_head_tail(&path, 100).unwrap();
    assert!(bytes.is_empty());
}

#[test]
fn read_lines_empty_file() {
    let dir = tmp();
    let path = write_file(&dir, "empty.txt", b"");
    let lines = read_lines(&path, 100, 10000).unwrap();
    assert!(lines.is_empty());
}

#[test]
fn read_text_capped_empty_file() {
    let dir = tmp();
    let path = write_file(&dir, "empty.txt", b"");
    let text = read_text_capped(&path, 100).unwrap();
    assert!(text.is_empty());
}

#[test]
fn hash_file_empty_file() {
    let dir = tmp();
    let path = write_file(&dir, "empty.txt", b"");
    let h = hash_file(&path, 100).unwrap();
    assert_eq!(h, hash_bytes(b""));
}

#[test]
fn entropy_empty_file_content() {
    assert_eq!(entropy_bits_per_byte(b""), 0.0);
}

// ============================================================================
// 5. Binary file detection
// ============================================================================

#[test]
fn is_text_like_ascii() {
    assert!(is_text_like(b"hello world"));
}

#[test]
fn is_text_like_utf8() {
    assert!(is_text_like("héllo wörld".as_bytes()));
}

#[test]
fn is_text_like_null_byte_makes_binary() {
    assert!(!is_text_like(b"hello\x00world"));
}

#[test]
fn is_text_like_empty_is_text() {
    assert!(is_text_like(b""));
}

#[test]
fn is_text_like_just_null() {
    assert!(!is_text_like(b"\x00"));
}

#[test]
fn is_text_like_binary_header() {
    // ELF header
    assert!(!is_text_like(b"\x7fELF\x00\x01\x01"));
}

#[test]
fn is_text_like_invalid_utf8_without_null() {
    // Invalid UTF-8 sequence but no null bytes
    let bytes = vec![0xFF, 0xFE, 0x41, 0x42];
    // This should be false because it's not valid UTF-8
    assert!(!is_text_like(&bytes));
}

#[test]
fn is_text_like_newlines_and_tabs() {
    assert!(is_text_like(b"line1\nline2\ttab"));
}

// ============================================================================
// 6. Very large file handling
// ============================================================================

#[test]
fn read_head_large_file_capped() {
    let dir = tmp();
    let data = vec![b'x'; 100_000];
    let path = write_file(&dir, "big.txt", &data);
    let head = read_head(&path, 1024).unwrap();
    assert_eq!(head.len(), 1024);
}

#[test]
fn read_head_tail_large_file() {
    let dir = tmp();
    let mut data = Vec::with_capacity(10_000);
    for i in 0..10_000u16 {
        data.push((i % 256) as u8);
    }
    let path = write_file(&dir, "big.bin", &data);
    let result = read_head_tail(&path, 100).unwrap();
    assert_eq!(result.len(), 100);
    // First 50 bytes should be head, last 50 should be tail
    assert_eq!(result[0], 0); // first byte of file
}

#[test]
fn hash_file_large_with_limit() {
    let dir = tmp();
    let data = vec![b'A'; 1_000_000];
    let path = write_file(&dir, "huge.txt", &data);
    let h = hash_file(&path, 4096).unwrap();
    assert_eq!(h.len(), 64);
    assert_eq!(h, hash_bytes(&vec![b'A'; 4096]));
}

#[test]
fn entropy_large_uniform_data() {
    let data = vec![0x42; 100_000];
    let e = entropy_bits_per_byte(&data);
    assert_eq!(e, 0.0, "uniform data should have zero entropy");
}

// ============================================================================
// 7. read_head_tail boundary conditions
// ============================================================================

#[test]
fn read_head_tail_zero_max_returns_empty() {
    let dir = tmp();
    let path = write_file(&dir, "f.txt", b"hello");
    let result = read_head_tail(&path, 0).unwrap();
    assert!(result.is_empty());
}

#[test]
fn read_head_tail_max_one() {
    let dir = tmp();
    let path = write_file(&dir, "f.txt", b"abcdefghij");
    let result = read_head_tail(&path, 1).unwrap();
    // max_bytes=1, half=0, head_len=max(0,1)=1, tail_len=0
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], b'a');
}

#[test]
fn read_head_tail_exact_file_size() {
    let dir = tmp();
    let path = write_file(&dir, "f.txt", b"abcde");
    let result = read_head_tail(&path, 5).unwrap();
    assert_eq!(result, b"abcde");
}

#[test]
fn read_head_tail_larger_than_file() {
    let dir = tmp();
    let path = write_file(&dir, "f.txt", b"abc");
    let result = read_head_tail(&path, 1000).unwrap();
    assert_eq!(result, b"abc");
}

// ============================================================================
// 8. read_lines edge cases
// ============================================================================

#[test]
fn read_lines_zero_max_lines() {
    let dir = tmp();
    let path = write_file(&dir, "f.txt", b"line1\nline2\n");
    let lines = read_lines(&path, 0, 10000).unwrap();
    assert!(lines.is_empty());
}

#[test]
fn read_lines_zero_max_bytes() {
    let dir = tmp();
    let path = write_file(&dir, "f.txt", b"line1\nline2\n");
    let lines = read_lines(&path, 100, 0).unwrap();
    assert!(lines.is_empty());
}

#[test]
fn read_lines_single_line_no_newline() {
    let dir = tmp();
    let path = write_file(&dir, "f.txt", b"no newline at end");
    let lines = read_lines(&path, 100, 10000).unwrap();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], "no newline at end");
}

#[test]
fn read_lines_preserves_blank_lines() {
    let dir = tmp();
    let path = write_file(&dir, "f.txt", b"a\n\nb\n\nc\n");
    let lines = read_lines(&path, 100, 10000).unwrap();
    assert_eq!(lines.len(), 5);
    assert_eq!(lines[1], "");
    assert_eq!(lines[3], "");
}

// ============================================================================
// 9. Complexity metric edge cases
// ============================================================================

#[test]
fn complexity_empty_code() {
    let m = analyze_functions("", "rust");
    assert_eq!(m.function_count, 0);
}

#[test]
fn complexity_no_functions() {
    let code = "// just a comment\nlet x = 42;\n";
    let m = analyze_functions(code, "rust");
    assert_eq!(m.function_count, 0);
}

#[test]
fn complexity_single_rust_fn() {
    let code = "fn main() {\n    println!(\"hello\");\n}\n";
    let m = analyze_functions(code, "rust");
    assert_eq!(m.function_count, 1);
    assert_eq!(m.max_function_length, 3);
}

#[test]
fn complexity_multiple_rust_fns() {
    let code = r#"
fn foo() {
    // do stuff
}

fn bar() {
    // do other stuff
}

fn baz() {
    // do more stuff
}
"#;
    let m = analyze_functions(code, "rust");
    assert_eq!(m.function_count, 3);
}

#[test]
fn complexity_python_def() {
    let code = "def hello():\n    print('hello')\n\ndef world():\n    print('world')\n";
    let m = analyze_functions(code, "python");
    assert_eq!(m.function_count, 2);
}

#[test]
fn complexity_unsupported_language() {
    let code = "fn main() { }";
    let m = analyze_functions(code, "brainfuck");
    assert_eq!(m.function_count, 0);
}

#[test]
fn cyclomatic_empty() {
    let cc = estimate_cyclomatic_complexity("", "rust");
    assert_eq!(cc.function_count, 0);
    assert_eq!(cc.total_cc, 0);
}

#[test]
fn cyclomatic_simple_function() {
    let code = "fn simple() {\n    println!(\"hello\");\n}\n";
    let cc = estimate_cyclomatic_complexity(code, "rust");
    assert_eq!(cc.function_count, 1);
    assert_eq!(cc.max_cc, 1); // base complexity only
}

#[test]
fn cyclomatic_if_branch() {
    let code = r#"
fn decide(x: i32) {
    if x > 0 {
        println!("positive");
    }
}
"#;
    let cc = estimate_cyclomatic_complexity(code, "rust");
    assert!(cc.max_cc >= 2, "if adds at least 1: got {}", cc.max_cc);
}

#[test]
fn cognitive_empty() {
    let cc = estimate_cognitive_complexity("", "rust");
    assert_eq!(cc.function_count, 0);
    assert_eq!(cc.total, 0);
}

#[test]
fn cognitive_nested_ifs_higher_than_flat() {
    let flat = r#"
fn flat(x: i32) {
    if x > 0 { }
    if x > 1 { }
}
"#;
    let nested = r#"
fn nested(x: i32) {
    if x > 0 {
        if x > 1 { }
    }
}
"#;
    let flat_cc = estimate_cognitive_complexity(flat, "rust");
    let nested_cc = estimate_cognitive_complexity(nested, "rust");
    assert!(
        nested_cc.max >= flat_cc.max,
        "nested should be >= flat: {} vs {}",
        nested_cc.max,
        flat_cc.max
    );
}

#[test]
fn nesting_depth_empty() {
    let n = analyze_nesting_depth("", "rust");
    assert_eq!(n.max_depth, 0);
}

#[test]
fn nesting_depth_single_level() {
    let code = "fn main() {\n    println!(\"hi\");\n}\n";
    let n = analyze_nesting_depth(code, "rust");
    assert!(n.max_depth >= 1);
}

#[test]
fn nesting_depth_deep_nesting() {
    let code = r#"
fn deep() {
    if true {
        if true {
            if true {
                println!("deep");
            }
        }
    }
}
"#;
    let n = analyze_nesting_depth(code, "rust");
    assert!(n.max_depth >= 3, "expected depth >= 3, got {}", n.max_depth);
}

// ============================================================================
// 10. Property-based tests
// ============================================================================

mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn entropy_in_range(data in proptest::collection::vec(any::<u8>(), 0..500)) {
            let e = entropy_bits_per_byte(&data);
            prop_assert!(e >= 0.0, "entropy must be >= 0: {e}");
            prop_assert!(e <= 8.0, "entropy must be <= 8: {e}");
        }

        #[test]
        fn hash_length_always_64(data in proptest::collection::vec(any::<u8>(), 0..500)) {
            let h = hash_bytes(&data);
            prop_assert_eq!(h.len(), 64, "hash must be 64 hex chars");
        }

        #[test]
        fn hash_is_hex_only(data in proptest::collection::vec(any::<u8>(), 0..200)) {
            let h = hash_bytes(&data);
            prop_assert!(h.chars().all(|c| c.is_ascii_hexdigit()), "hash must be hex: {h}");
        }

        #[test]
        fn hash_deterministic(data in proptest::collection::vec(any::<u8>(), 0..200)) {
            let h1 = hash_bytes(&data);
            let h2 = hash_bytes(&data);
            prop_assert_eq!(h1, h2, "hash must be deterministic");
        }

        #[test]
        fn entropy_uniform_single_byte_zero(byte in any::<u8>()) {
            let data = vec![byte; 100];
            let e = entropy_bits_per_byte(&data);
            prop_assert!((e - 0.0).abs() < f32::EPSILON, "single-byte uniform = 0: {e}");
        }

        #[test]
        fn count_tags_non_negative(text in "[a-zA-Z ]{0,100}") {
            let tags = count_tags(&text, &["TODO", "FIXME"]);
            // Verify we get exactly the requested tags back
            prop_assert!(tags.len() <= 2);
        }

        #[test]
        fn is_text_like_consistent(data in proptest::collection::vec(any::<u8>(), 0..100)) {
            let r1 = is_text_like(&data);
            let r2 = is_text_like(&data);
            prop_assert_eq!(r1, r2, "is_text_like must be deterministic");
        }

        #[test]
        fn read_head_never_exceeds_max(len in 1usize..500, max in 1usize..500) {
            let dir = tmp();
            let data = vec![b'x'; len];
            let path = write_file(&dir, "f.txt", &data);
            let head = read_head(&path, max).unwrap();
            prop_assert!(head.len() <= max, "head {} > max {}", head.len(), max);
            prop_assert!(head.len() <= len, "head {} > file len {}", head.len(), len);
        }

        #[test]
        fn read_head_tail_never_exceeds_max(len in 1usize..500, max in 1usize..500) {
            let dir = tmp();
            let data = vec![b'y'; len];
            let path = write_file(&dir, "f.txt", &data);
            let result = read_head_tail(&path, max).unwrap();
            prop_assert!(result.len() <= max, "head_tail {} > max {}", result.len(), max);
        }
    }
}
