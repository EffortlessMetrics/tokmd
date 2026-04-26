//! Deep tests for tokmd-analysis content helpers – wave 69.
//!
//! Covers entropy calculation, tag counting, BLAKE3 hashing, text detection,
//! file reading helpers, and complexity metrics with determinism checks.

use std::fs::File;
use std::io::Write;

use crate::content::complexity::{
    analyze_functions, analyze_nesting_depth, estimate_cognitive_complexity,
    estimate_cyclomatic_complexity,
};
use crate::content::io::{
    count_tags, entropy_bits_per_byte, hash_bytes, hash_file, is_text_like, read_head,
    read_head_tail, read_lines,
};

// =========================================================================
// 1. Entropy calculation
// =========================================================================

#[test]
fn entropy_empty_is_zero() {
    assert_eq!(entropy_bits_per_byte(&[]), 0.0);
}

#[test]
fn entropy_uniform_single_byte_is_zero() {
    let data = vec![42u8; 500];
    let e = entropy_bits_per_byte(&data);
    assert!(e.abs() < 1e-6, "single byte repeated → ~0.0, got {e}");
}

#[test]
fn entropy_two_equal_frequencies_is_one() {
    let data: Vec<u8> = (0..1024).map(|i| (i % 2) as u8).collect();
    let e = entropy_bits_per_byte(&data);
    assert!(
        (e - 1.0).abs() < 0.01,
        "two bytes equally frequent → ~1.0, got {e}"
    );
}

#[test]
fn entropy_max_is_eight_bits() {
    let data: Vec<u8> = (0..256)
        .flat_map(|b| std::iter::repeat_n(b as u8, 100))
        .collect();
    let e = entropy_bits_per_byte(&data);
    assert!((e - 8.0).abs() < 0.01, "uniform 256 values → ~8.0, got {e}");
}

#[test]
fn entropy_deterministic() {
    let data = b"the quick brown fox jumps over the lazy dog";
    let a = entropy_bits_per_byte(data);
    let b = entropy_bits_per_byte(data);
    assert_eq!(a, b);
}

// =========================================================================
// 2. Tag counting (TODO, FIXME, etc.)
// =========================================================================

#[test]
fn count_tags_basic_detection() {
    let text = "// TODO: fix this\n// FIXME: broken\n// HACK: workaround";
    let result = count_tags(text, &["TODO", "FIXME", "HACK"]);
    assert_eq!(
        result,
        vec![
            ("TODO".to_string(), 1),
            ("FIXME".to_string(), 1),
            ("HACK".to_string(), 1),
        ]
    );
}

#[test]
fn count_tags_multiple_occurrences() {
    let text = "TODO first\nTODO second\nTODO third";
    let result = count_tags(text, &["TODO"]);
    assert_eq!(result[0].1, 3);
}

#[test]
fn count_tags_case_insensitive() {
    let text = "todo: lower\nTodo: mixed\nTODO: upper";
    let result = count_tags(text, &["TODO"]);
    assert_eq!(result[0].1, 3, "matching should be case-insensitive");
}

#[test]
fn count_tags_empty_text_and_empty_tags() {
    assert_eq!(count_tags("", &["TODO"])[0].1, 0);
    assert!(count_tags("TODO: something", &[]).is_empty());
}

// =========================================================================
// 3. BLAKE3 hashing
// =========================================================================

#[test]
fn hash_bytes_deterministic_and_hex() {
    let a = hash_bytes(b"hello world");
    let b = hash_bytes(b"hello world");
    assert_eq!(a, b);
    assert_eq!(a.len(), 64);
    assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn hash_bytes_different_input_different_hash() {
    assert_ne!(hash_bytes(b"aaa"), hash_bytes(b"bbb"));
}

// =========================================================================
// 4. is_text_like
// =========================================================================

#[test]
fn is_text_like_detects_text_and_binary() {
    assert!(is_text_like(b"Hello, world!"));
    assert!(is_text_like("café résumé".as_bytes()));
    assert!(is_text_like(b""));
    assert!(!is_text_like(b"has\x00null"));
}

// =========================================================================
// 5. File reading helpers
// =========================================================================

#[test]
fn read_head_respects_limit() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("file.txt");
    File::create(&path)
        .unwrap()
        .write_all(b"abcdefghij")
        .unwrap();
    let bytes = read_head(&path, 5).unwrap();
    assert_eq!(bytes, b"abcde");
}

#[test]
fn read_head_tail_small_file_returns_all() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("small.txt");
    File::create(&path).unwrap().write_all(b"tiny").unwrap();
    assert_eq!(read_head_tail(&path, 100).unwrap(), b"tiny");
    assert!(read_head_tail(&path, 0).unwrap().is_empty());
}

#[test]
fn read_lines_max_zero_returns_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("lines.txt");
    writeln!(File::create(&path).unwrap(), "line1").unwrap();
    assert!(read_lines(&path, 0, 10000).unwrap().is_empty());
}

#[test]
fn hash_file_matches_hash_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("hash.txt");
    File::create(&path)
        .unwrap()
        .write_all(b"deterministic")
        .unwrap();
    assert_eq!(
        hash_file(&path, 1000).unwrap(),
        hash_bytes(b"deterministic")
    );
}

// =========================================================================
// 6. Complexity – analyze_functions
// =========================================================================

#[test]
fn analyze_functions_empty_content() {
    let m = analyze_functions("", "rust");
    assert_eq!(m.function_count, 0);
    assert_eq!(m.max_function_length, 0);
}

#[test]
fn analyze_functions_rust_single_fn() {
    let src = "fn main() {\n    println!(\"hi\");\n}\n";
    let m = analyze_functions(src, "rust");
    assert_eq!(m.function_count, 1);
    assert!(m.max_function_length >= 2);
}

#[test]
fn analyze_functions_python_defs() {
    let src = "def foo():\n    pass\n\ndef bar():\n    return 1\n";
    let m = analyze_functions(src, "python");
    assert_eq!(m.function_count, 2);
}

// =========================================================================
// 7. Complexity – cyclomatic & cognitive
// =========================================================================

#[test]
fn cyclomatic_empty_and_simple() {
    let c = estimate_cyclomatic_complexity("", "rust");
    assert_eq!(c.total_cc, 0);
    let c = estimate_cyclomatic_complexity(
        "fn check(x: i32) {\n    if x > 0 {\n        println!(\"pos\");\n    }\n}\n",
        "rust",
    );
    assert!(c.total_cc >= 2, "base 1 + if = 2, got {}", c.total_cc);
}

#[test]
fn cognitive_empty_content() {
    let c = estimate_cognitive_complexity("", "rust");
    assert_eq!(c.total, 0);
}

// =========================================================================
// 8. Nesting depth
// =========================================================================

#[test]
fn nesting_empty_and_simple() {
    assert_eq!(analyze_nesting_depth("", "rust").max_depth, 0);
    let n = analyze_nesting_depth(
        "fn main() {\n    if true {\n        println!(\"hi\");\n    }\n}\n",
        "rust",
    );
    assert!(n.max_depth >= 1);
}
