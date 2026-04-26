//! Comprehensive depth tests for tokmd-analysis content helpers (W55).
//!
//! Covers: entropy, hashing, text detection, tag counting, read helpers,
//! complexity metrics, and property-based invariants.

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

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

fn write_file(dir: &tempfile::TempDir, name: &str, content: &[u8]) -> std::path::PathBuf {
    let p = dir.path().join(name);
    let mut f = File::create(&p).unwrap();
    f.write_all(content).unwrap();
    p
}

// ===========================================================================
// 1. Entropy
// ===========================================================================

#[test]
fn entropy_empty_bytes() {
    assert_eq!(entropy_bits_per_byte(&[]), 0.0);
}

#[test]
fn entropy_single_byte_is_zero() {
    // All same bytes → zero entropy
    assert_eq!(entropy_bits_per_byte(&[0x41; 100]), 0.0);
}

#[test]
fn entropy_two_equally_likely() {
    // 50/50 distribution → 1.0 bit
    let data: Vec<u8> = (0..1000).map(|i| if i % 2 == 0 { 0 } else { 1 }).collect();
    let e = entropy_bits_per_byte(&data);
    assert!((e - 1.0).abs() < 0.01, "expected ~1.0, got {e}");
}

#[test]
fn entropy_uniform_256() {
    // All 256 byte values equally → 8 bits
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
fn entropy_non_negative() {
    for input in [b"abc".as_slice(), b"\x00\xFF", b"aaaa", b""] {
        assert!(entropy_bits_per_byte(input) >= 0.0);
    }
}

#[test]
fn entropy_max_eight_bits() {
    let data: Vec<u8> = (0..=255).collect();
    let e = entropy_bits_per_byte(&data);
    assert!(e <= 8.0 + 0.001);
}

#[test]
fn entropy_deterministic() {
    let data = b"hello world entropy test";
    let e1 = entropy_bits_per_byte(data);
    let e2 = entropy_bits_per_byte(data);
    assert_eq!(e1, e2);
}

// ===========================================================================
// 2. Hashing
// ===========================================================================

#[test]
fn hash_bytes_empty() {
    let h = hash_bytes(&[]);
    assert_eq!(h.len(), 64);
}

#[test]
fn hash_bytes_deterministic() {
    assert_eq!(hash_bytes(b"test"), hash_bytes(b"test"));
}

#[test]
fn hash_bytes_different_inputs_different_hashes() {
    assert_ne!(hash_bytes(b"aaa"), hash_bytes(b"bbb"));
}

#[test]
fn hash_bytes_hex_chars_only() {
    let h = hash_bytes(b"sample");
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn hash_file_matches_hash_bytes() {
    let d = tmp();
    let p = write_file(&d, "f.txt", b"hello");
    let fh = hash_file(&p, 1000).unwrap();
    assert_eq!(fh, hash_bytes(b"hello"));
}

#[test]
fn hash_file_respects_limit() {
    let d = tmp();
    let p = write_file(&d, "f.txt", b"ABCDE12345");
    let limited = hash_file(&p, 5).unwrap();
    assert_eq!(limited, hash_bytes(b"ABCDE"));
}

#[test]
fn hash_file_empty_file() {
    let d = tmp();
    let p = write_file(&d, "empty.txt", b"");
    let h = hash_file(&p, 100).unwrap();
    assert_eq!(h, hash_bytes(b""));
}

// ===========================================================================
// 3. Text detection
// ===========================================================================

#[test]
fn text_like_ascii() {
    assert!(is_text_like(b"hello world"));
}

#[test]
fn text_like_utf8() {
    assert!(is_text_like("日本語".as_bytes()));
}

#[test]
fn not_text_with_null_byte() {
    assert!(!is_text_like(b"hello\x00world"));
}

#[test]
fn text_like_empty() {
    assert!(is_text_like(b""));
}

#[test]
fn not_text_binary_blob() {
    assert!(!is_text_like(&[0x00, 0xFF, 0x00, 0xFE]));
}

// ===========================================================================
// 4. Tag counting (TODO / FIXME)
// ===========================================================================

#[test]
fn count_tags_basic() {
    let text = "// TODO: fix this\n// FIXME: and this\n// TODO: another";
    let tags = count_tags(text, &["TODO", "FIXME"]);
    assert_eq!(tags.len(), 2);
    assert_eq!(tags[0], ("TODO".to_string(), 2));
    assert_eq!(tags[1], ("FIXME".to_string(), 1));
}

#[test]
fn count_tags_case_insensitive() {
    let text = "todo Todo TODO";
    let tags = count_tags(text, &["TODO"]);
    assert_eq!(tags[0].1, 3);
}

#[test]
fn count_tags_none_found() {
    let text = "clean code without any markers";
    let tags = count_tags(text, &["TODO", "FIXME", "HACK"]);
    for (_, count) in &tags {
        assert_eq!(*count, 0);
    }
}

#[test]
fn count_tags_empty_text() {
    let tags = count_tags("", &["TODO"]);
    assert_eq!(tags[0].1, 0);
}

#[test]
fn count_tags_empty_tags_list() {
    let tags = count_tags("TODO FIXME HACK", &[]);
    assert!(tags.is_empty());
}

#[test]
fn count_tags_multiple_on_one_line() {
    let text = "TODO TODO TODO";
    let tags = count_tags(text, &["TODO"]);
    assert_eq!(tags[0].1, 3);
}

// ===========================================================================
// 5. read_head / read_head_tail / read_lines / read_text_capped
// ===========================================================================

#[test]
fn read_head_zero_limit() {
    let d = tmp();
    let p = write_file(&d, "f.txt", b"data");
    let bytes = read_head(&p, 0).unwrap();
    assert!(bytes.is_empty());
}

#[test]
fn read_head_tail_zero_limit() {
    let d = tmp();
    let p = write_file(&d, "f.txt", b"data");
    let bytes = read_head_tail(&p, 0).unwrap();
    assert!(bytes.is_empty());
}

#[test]
fn read_head_exact_size() {
    let d = tmp();
    let p = write_file(&d, "f.txt", b"12345");
    let bytes = read_head(&p, 5).unwrap();
    assert_eq!(bytes, b"12345");
}

#[test]
fn read_head_tail_exact_size() {
    let d = tmp();
    let p = write_file(&d, "f.txt", b"12345");
    let bytes = read_head_tail(&p, 5).unwrap();
    assert_eq!(bytes, b"12345");
}

#[test]
fn read_head_tail_split() {
    let d = tmp();
    // "0123456789" → head 3 bytes "012", tail 3 bytes "789"
    let p = write_file(&d, "f.txt", b"0123456789");
    let bytes = read_head_tail(&p, 6).unwrap();
    assert_eq!(bytes, b"012789");
}

#[test]
fn read_lines_zero_lines() {
    let d = tmp();
    let p = write_file(&d, "f.txt", b"line1\nline2\n");
    let lines = read_lines(&p, 0, 10000).unwrap();
    assert!(lines.is_empty());
}

#[test]
fn read_lines_zero_bytes() {
    let d = tmp();
    let p = write_file(&d, "f.txt", b"line1\nline2\n");
    let lines = read_lines(&p, 100, 0).unwrap();
    assert!(lines.is_empty());
}

#[test]
fn read_text_capped_short_file() {
    let d = tmp();
    let p = write_file(&d, "f.txt", b"short");
    let text = read_text_capped(&p, 1000).unwrap();
    assert_eq!(text, "short");
}

#[test]
fn read_text_capped_truncates() {
    let d = tmp();
    let p = write_file(&d, "f.txt", b"abcdefghij");
    let text = read_text_capped(&p, 4).unwrap();
    assert_eq!(text, "abcd");
}

// ===========================================================================
// 6. Complexity: analyze_functions
// ===========================================================================

#[test]
fn analyze_functions_empty() {
    let m = analyze_functions("", "rust");
    assert_eq!(m.function_count, 0);
    assert_eq!(m.max_function_length, 0);
}

#[test]
fn analyze_functions_rust_single() {
    let code = "fn main() {\n    println!(\"hi\");\n}\n";
    let m = analyze_functions(code, "rust");
    assert_eq!(m.function_count, 1);
    assert!(m.max_function_length >= 2);
}

#[test]
fn analyze_functions_rust_multiple() {
    let code = "fn a() {\n}\nfn b() {\n    x();\n    y();\n}\n";
    let m = analyze_functions(code, "rust");
    assert_eq!(m.function_count, 2);
}

#[test]
fn analyze_functions_python() {
    let code = "def foo():\n    pass\n\ndef bar():\n    return 1\n";
    let m = analyze_functions(code, "python");
    assert_eq!(m.function_count, 2);
}

#[test]
fn analyze_functions_go() {
    let code = "func main() {\n    fmt.Println(\"hi\")\n}\n";
    let m = analyze_functions(code, "go");
    assert_eq!(m.function_count, 1);
}

#[test]
fn analyze_functions_unsupported_lang() {
    let code = "some random content";
    let m = analyze_functions(code, "brainfuck");
    assert_eq!(m.function_count, 0);
}

#[test]
fn analyze_functions_avg_length() {
    // Two functions: 2 lines and 4 lines → avg 3.0
    let code = "fn a() {\n}\nfn b() {\n    x();\n    y();\n}\n";
    let m = analyze_functions(code, "rust");
    assert!(m.avg_function_length > 0.0);
}

// ===========================================================================
// 7. Complexity: cyclomatic
// ===========================================================================

#[test]
fn cyclomatic_empty() {
    let r = estimate_cyclomatic_complexity("", "rust");
    assert_eq!(r.function_count, 0);
    assert_eq!(r.total_cc, 0);
}

#[test]
fn cyclomatic_simple_function() {
    let code = "fn simple() {\n    println!(\"no branches\");\n}\n";
    let r = estimate_cyclomatic_complexity(code, "rust");
    assert_eq!(r.function_count, 1);
    // Base complexity = 1
    assert!(r.max_cc >= 1);
}

#[test]
fn cyclomatic_with_if() {
    let code = "fn f(x: i32) {\n    if x > 0 {\n        return;\n    }\n}\n";
    let r = estimate_cyclomatic_complexity(code, "rust");
    // 1 (base) + 1 (if) = 2
    assert!(r.max_cc >= 2, "expected >= 2, got {}", r.max_cc);
}

#[test]
fn cyclomatic_unsupported_lang() {
    let r = estimate_cyclomatic_complexity("anything", "cobol");
    assert_eq!(r.function_count, 0);
}

// ===========================================================================
// 8. Complexity: cognitive
// ===========================================================================

#[test]
fn cognitive_empty() {
    let r = estimate_cognitive_complexity("", "rust");
    assert_eq!(r.function_count, 0);
    assert_eq!(r.total, 0);
}

#[test]
fn cognitive_simple_function() {
    let code = "fn simple() {\n    println!(\"hi\");\n}\n";
    let r = estimate_cognitive_complexity(code, "rust");
    assert_eq!(r.function_count, 1);
    assert_eq!(r.max, 0); // No control flow
}

#[test]
fn cognitive_nested_ifs_higher() {
    let code = "\
fn f() {
    if true {
        if true {
            println!(\"deep\");
        }
    }
}
";
    let r = estimate_cognitive_complexity(code, "rust");
    assert!(
        r.max >= 2,
        "nested ifs should increase cognitive complexity"
    );
}

// ===========================================================================
// 9. Complexity: nesting depth
// ===========================================================================

#[test]
fn nesting_empty() {
    let r = analyze_nesting_depth("", "rust");
    assert_eq!(r.max_depth, 0);
}

#[test]
fn nesting_flat_code() {
    let code = "fn main() {\n    println!(\"flat\");\n}\n";
    let r = analyze_nesting_depth(code, "rust");
    assert!(r.max_depth >= 1);
}

#[test]
fn nesting_deep_braces() {
    let code =
        "fn f() {\n    if true {\n        for i in 0..1 {\n            x();\n        }\n    }\n}\n";
    let r = analyze_nesting_depth(code, "rust");
    assert!(r.max_depth >= 3, "expected >= 3, got {}", r.max_depth);
}

#[test]
fn nesting_python_indentation() {
    let code = "def f():\n    if True:\n        for i in range(1):\n            pass\n";
    let r = analyze_nesting_depth(code, "python");
    assert!(r.max_depth >= 2);
}

// ===========================================================================
// 10. Edge cases
// ===========================================================================

#[test]
fn binary_file_not_text() {
    let d = tmp();
    let p = write_file(&d, "bin", &[0x00, 0x01, 0x02, 0xFF]);
    let bytes = read_head(&p, 100).unwrap();
    assert!(!is_text_like(&bytes));
}

#[test]
fn large_file_hash() {
    let d = tmp();
    let data = vec![0x42u8; 1_000_000];
    let p = write_file(&d, "big.bin", &data);
    let h = hash_file(&p, 1_000_000).unwrap();
    assert_eq!(h.len(), 64);
    assert_eq!(h, hash_bytes(&data));
}

#[test]
fn empty_file_entropy() {
    let d = tmp();
    let p = write_file(&d, "empty", b"");
    let bytes = read_head(&p, 100).unwrap();
    assert_eq!(entropy_bits_per_byte(&bytes), 0.0);
}

#[test]
fn nonexistent_file_errors() {
    let d = tmp();
    let p = d.path().join("no_such_file");
    assert!(read_head(&p, 100).is_err());
    assert!(hash_file(&p, 100).is_err());
    assert!(read_lines(&p, 10, 1000).is_err());
    assert!(read_text_capped(&p, 100).is_err());
    assert!(read_head_tail(&p, 100).is_err());
}

// ===========================================================================
// 11. Property-based tests
// ===========================================================================

mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn entropy_always_in_range(data in proptest::collection::vec(any::<u8>(), 0..500)) {
            let e = entropy_bits_per_byte(&data);
            assert!(e >= 0.0);
            assert!(e <= 8.0 + 0.001);
        }

        #[test]
        fn hash_deterministic(data in proptest::collection::vec(any::<u8>(), 0..200)) {
            assert_eq!(hash_bytes(&data), hash_bytes(&data));
        }

        #[test]
        fn hash_length_always_64(data in proptest::collection::vec(any::<u8>(), 0..200)) {
            assert_eq!(hash_bytes(&data).len(), 64);
        }

        #[test]
        fn is_text_like_no_null_implies_utf8_check(data in proptest::collection::vec(1u8..=127, 0..200)) {
            // ASCII range 1-127 with no nulls is always text-like
            assert!(is_text_like(&data));
        }

        #[test]
        fn count_tags_count_non_negative(
            text in "[a-zA-Z ]{0,100}",
        ) {
            let tags = count_tags(&text, &["TODO", "FIXME"]);
            for (_, count) in &tags {
                assert!(*count <= text.len());
            }
        }
    }
}
