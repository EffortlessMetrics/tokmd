//! Contract tests for tokmd-analysis content helpers: deep coverage of entropy calculation,
//! tag extraction, hashing, complexity scoring, content reading, text detection,
//! determinism, and edge/boundary cases.

use std::fs::File;
use std::io::Write;

use crate::content::complexity::{
    analyze_functions, analyze_nesting_depth, estimate_cognitive_complexity,
    estimate_cyclomatic_complexity,
};
use crate::content::io::{
    count_tags, entropy_bits_per_byte, hash_bytes, hash_file, is_text_like, read_head,
    read_head_tail, read_lines, read_text_capped,
};

// ===========================================================================
// 1. Entropy calculation
// ===========================================================================

#[test]
fn entropy_empty_bytes_is_zero() {
    assert_eq!(entropy_bits_per_byte(&[]), 0.0);
}

#[test]
fn entropy_single_value_is_zero() {
    let buf = [42u8; 1000];
    let e = entropy_bits_per_byte(&buf);
    assert!(e.abs() < 1e-6, "constant bytes → 0 entropy, got {e}");
}

#[test]
fn entropy_two_values_is_one_bit() {
    let buf: Vec<u8> = (0..2000).map(|i| (i % 2) as u8).collect();
    let e = entropy_bits_per_byte(&buf);
    assert!((e - 1.0).abs() < 0.02, "expected ~1.0, got {e}");
}

#[test]
fn entropy_four_values_is_two_bits() {
    let buf: Vec<u8> = (0..4000).map(|i| (i % 4) as u8).collect();
    let e = entropy_bits_per_byte(&buf);
    assert!((e - 2.0).abs() < 0.02, "expected ~2.0, got {e}");
}

#[test]
fn entropy_full_byte_range_is_eight() {
    let buf: Vec<u8> = (0u8..=255).cycle().take(2560).collect();
    let e = entropy_bits_per_byte(&buf);
    assert!((e - 8.0).abs() < 0.02, "expected ~8.0, got {e}");
}

#[test]
fn entropy_sixteen_values_is_four() {
    let buf: Vec<u8> = (0..3200).map(|i| (i % 16) as u8).collect();
    let e = entropy_bits_per_byte(&buf);
    assert!((e - 4.0).abs() < 0.05, "expected ~4.0, got {e}");
}

#[test]
fn entropy_ascii_text_moderate() {
    let text = b"The quick brown fox jumps over the lazy dog. 0123456789";
    let e = entropy_bits_per_byte(text);
    // ASCII text typically 3–5 bits/byte
    assert!(
        (3.0..=6.0).contains(&e),
        "ASCII text entropy in 3-6, got {e}"
    );
}

#[test]
fn entropy_single_byte_input() {
    let e = entropy_bits_per_byte(&[0x42]);
    assert!(e.abs() < 1e-6, "single byte has 0 entropy");
}

// ===========================================================================
// 2. Property: entropy always in [0, 8]
// ===========================================================================

#[test]
fn property_entropy_bounded() {
    let test_inputs: &[&[u8]] = &[
        &[],
        &[0],
        &[0, 1],
        &[255; 100],
        b"hello world",
        &(0u8..=255).collect::<Vec<u8>>(),
    ];
    for input in test_inputs {
        let e = entropy_bits_per_byte(input);
        assert!(
            (0.0..=8.0).contains(&e),
            "entropy {e} out of [0,8] for input len {}",
            input.len()
        );
    }
}

#[test]
fn property_entropy_deterministic() {
    let inputs: &[&[u8]] = &[
        b"deterministic",
        &[0, 1, 2, 3],
        &(0u8..=255).collect::<Vec<u8>>(),
    ];
    for input in inputs {
        let a = entropy_bits_per_byte(input);
        let b = entropy_bits_per_byte(input);
        assert_eq!(a, b, "entropy should be deterministic");
    }
}

// ===========================================================================
// 3. Tag extraction
// ===========================================================================

#[test]
fn tags_todo_count() {
    let text = "// TODO: fix this\n// TODO: and this\nlet x = 1;";
    let tags = count_tags(text, &["TODO"]);
    assert_eq!(tags, [("TODO".to_string(), 2)]);
}

#[test]
fn tags_fixme_count() {
    let text = "// FIXME: broken\n// FIXME: also broken";
    let tags = count_tags(text, &["FIXME"]);
    assert_eq!(tags, [("FIXME".to_string(), 2)]);
}

#[test]
fn tags_multiple_types() {
    let text = "// TODO: a\n// FIXME: b\n// HACK: c\n// TODO: d";
    let tags = count_tags(text, &["TODO", "FIXME", "HACK"]);
    assert_eq!(tags[0], ("TODO".to_string(), 2));
    assert_eq!(tags[1], ("FIXME".to_string(), 1));
    assert_eq!(tags[2], ("HACK".to_string(), 1));
}

#[test]
fn tags_case_insensitive() {
    let text = "todo fixme Todo FIXME";
    let tags = count_tags(text, &["TODO", "FIXME"]);
    assert_eq!(tags[0].1, 2); // TODO matches "todo" and "Todo"
    assert_eq!(tags[1].1, 2); // FIXME matches "fixme" and "FIXME"
}

#[test]
fn tags_empty_text() {
    let tags = count_tags("", &["TODO", "FIXME"]);
    assert_eq!(tags[0].1, 0);
    assert_eq!(tags[1].1, 0);
}

#[test]
fn tags_no_matches() {
    let tags = count_tags("clean code here", &["TODO", "FIXME"]);
    assert_eq!(tags[0].1, 0);
    assert_eq!(tags[1].1, 0);
}

#[test]
fn tags_empty_tags_list() {
    let tags = count_tags("TODO FIXME", &[]);
    assert!(tags.is_empty());
}

// ===========================================================================
// 4. BLAKE3 hashing
// ===========================================================================

#[test]
fn hash_bytes_deterministic() {
    let a = hash_bytes(b"test content");
    let b = hash_bytes(b"test content");
    assert_eq!(a, b);
}

#[test]
fn hash_bytes_different_input() {
    let a = hash_bytes(b"hello");
    let b = hash_bytes(b"world");
    assert_ne!(a, b);
}

#[test]
fn hash_bytes_length_is_64() {
    let h = hash_bytes(b"anything");
    assert_eq!(h.len(), 64, "BLAKE3 hex output is 64 chars");
}

#[test]
fn hash_bytes_hex_chars_only() {
    let h = hash_bytes(b"data");
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn hash_bytes_empty() {
    let h = hash_bytes(b"");
    assert_eq!(h.len(), 64);
    // BLAKE3 of empty input is a known value
    let again = hash_bytes(b"");
    assert_eq!(h, again);
}

#[test]
fn hash_file_matches_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("hf.txt");
    let mut f = File::create(&path).unwrap();
    f.write_all(b"file content").unwrap();

    let file_hash = hash_file(&path, 10000).unwrap();
    let byte_hash = hash_bytes(b"file content");
    assert_eq!(file_hash, byte_hash);
}

#[test]
fn hash_file_respects_max_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("partial.txt");
    let mut f = File::create(&path).unwrap();
    f.write_all(b"abcdefghij").unwrap();

    let partial = hash_file(&path, 5).unwrap();
    let expected = hash_bytes(b"abcde");
    assert_eq!(partial, expected);

    let full = hash_file(&path, 100).unwrap();
    assert_ne!(partial, full);
}

// ===========================================================================
// 5. is_text_like
// ===========================================================================

#[test]
fn text_like_ascii() {
    assert!(is_text_like(b"Hello, World!"));
}

#[test]
fn text_like_utf8() {
    assert!(is_text_like("こんにちは".as_bytes()));
}

#[test]
fn text_like_empty() {
    assert!(is_text_like(b""));
}

#[test]
fn text_like_binary_null_byte() {
    assert!(!is_text_like(&[0x48, 0x65, 0x00, 0x6C, 0x6F]));
}

#[test]
fn text_like_pure_binary() {
    assert!(!is_text_like(&[0x00, 0xFF, 0x00, 0xFF]));
}

// ===========================================================================
// 6. read_head / read_head_tail / read_lines / read_text_capped
// ===========================================================================

#[test]
fn read_head_empty_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("empty");
    File::create(&path).unwrap();
    let bytes = read_head(&path, 100).unwrap();
    assert!(bytes.is_empty());
}

#[test]
fn read_head_exact_content() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("exact");
    let mut f = File::create(&path).unwrap();
    f.write_all(b"abcdef").unwrap();
    let bytes = read_head(&path, 100).unwrap();
    assert_eq!(bytes, b"abcdef");
}

#[test]
fn read_head_truncates() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("trunc");
    let mut f = File::create(&path).unwrap();
    f.write_all(b"abcdefghij").unwrap();
    let bytes = read_head(&path, 5).unwrap();
    assert_eq!(bytes, b"abcde");
}

#[test]
fn read_head_tail_small_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("small_ht");
    let mut f = File::create(&path).unwrap();
    f.write_all(b"tiny").unwrap();
    let bytes = read_head_tail(&path, 100).unwrap();
    assert_eq!(bytes, b"tiny");
}

#[test]
fn read_head_tail_combines_head_and_tail() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("headtail");
    let mut f = File::create(&path).unwrap();
    f.write_all(b"0123456789").unwrap();
    // max_bytes=4 → head=2, tail=2 → "01" + "89"
    let bytes = read_head_tail(&path, 4).unwrap();
    assert_eq!(bytes, b"0189");
}

#[test]
fn read_head_tail_zero_max() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("zero_ht");
    let mut f = File::create(&path).unwrap();
    f.write_all(b"content").unwrap();
    let bytes = read_head_tail(&path, 0).unwrap();
    assert!(bytes.is_empty());
}

#[test]
fn read_lines_basic() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("lines");
    let mut f = File::create(&path).unwrap();
    writeln!(f, "alpha").unwrap();
    writeln!(f, "beta").unwrap();
    writeln!(f, "gamma").unwrap();

    let lines = read_lines(&path, 10, 10000).unwrap();
    assert_eq!(lines, ["alpha", "beta", "gamma"]);
}

#[test]
fn read_lines_max_zero() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("lz");
    let mut f = File::create(&path).unwrap();
    writeln!(f, "x").unwrap();
    let lines = read_lines(&path, 0, 10000).unwrap();
    assert!(lines.is_empty());
}

#[test]
fn read_lines_max_bytes_zero() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("lbz");
    let mut f = File::create(&path).unwrap();
    writeln!(f, "x").unwrap();
    let lines = read_lines(&path, 100, 0).unwrap();
    assert!(lines.is_empty());
}

#[test]
fn read_text_capped_content() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("tc");
    let mut f = File::create(&path).unwrap();
    f.write_all(b"Rust is great").unwrap();
    let text = read_text_capped(&path, 100).unwrap();
    assert_eq!(text, "Rust is great");
}

#[test]
fn read_text_capped_limit() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("tcl");
    let mut f = File::create(&path).unwrap();
    f.write_all(b"abcdefghij").unwrap();
    let text = read_text_capped(&path, 4).unwrap();
    assert_eq!(text, "abcd");
}

// ===========================================================================
// 7. Complexity: analyze_functions
// ===========================================================================

#[test]
fn functions_rust_basic() {
    let code = r#"
fn main() {
    println!("hello");
}

fn helper() {
    let x = 1;
}
"#;
    let m = analyze_functions(code, "rust");
    assert_eq!(m.function_count, 2);
}

#[test]
fn functions_rust_pub_async() {
    let code = r#"
pub async fn serve() {
    loop {}
}
"#;
    let m = analyze_functions(code, "rust");
    assert_eq!(m.function_count, 1);
}

#[test]
fn functions_python_basic() {
    let code = r#"
def greet():
    print("hi")

def farewell():
    print("bye")
"#;
    let m = analyze_functions(code, "python");
    assert_eq!(m.function_count, 2);
}

#[test]
fn functions_go_basic() {
    let code = r#"
func main() {
    fmt.Println("hello")
}
"#;
    let m = analyze_functions(code, "go");
    assert_eq!(m.function_count, 1);
}

#[test]
fn functions_empty_code() {
    let m = analyze_functions("", "rust");
    assert_eq!(m.function_count, 0);
    assert_eq!(m.max_function_length, 0);
}

#[test]
fn functions_unsupported_language() {
    let m = analyze_functions("fn main() {}", "brainfuck");
    assert_eq!(m.function_count, 0);
}

#[test]
fn functions_max_length_tracks_longest() {
    let code = r#"
fn short() {
    1;
}

fn long() {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    let e = 5;
    let f = 6;
    let g = 7;
}
"#;
    let m = analyze_functions(code, "rust");
    assert_eq!(m.function_count, 2);
    assert!(m.max_function_length > m.avg_function_length as usize);
}

// ===========================================================================
// 8. Complexity: cyclomatic
// ===========================================================================

#[test]
fn cyclomatic_simple_function() {
    let code = r#"
fn simple() {
    println!("no branches");
}
"#;
    let cc = estimate_cyclomatic_complexity(code, "rust");
    assert_eq!(cc.function_count, 1);
    assert_eq!(cc.max_cc, 1); // base complexity
}

#[test]
fn cyclomatic_with_if() {
    let code = r#"
fn branching(x: i32) -> i32 {
    if x > 0 {
        x
    } else {
        -x
    }
}
"#;
    let cc = estimate_cyclomatic_complexity(code, "rust");
    assert!(cc.max_cc >= 2, "if/else adds at least 1, got {}", cc.max_cc);
}

#[test]
fn cyclomatic_empty() {
    let cc = estimate_cyclomatic_complexity("", "rust");
    assert_eq!(cc.function_count, 0);
    assert_eq!(cc.total_cc, 0);
}

#[test]
fn cyclomatic_python() {
    let code = r#"
def check(x):
    if x > 0:
        return True
    elif x < 0:
        return False
    return None
"#;
    let cc = estimate_cyclomatic_complexity(code, "python");
    assert_eq!(cc.function_count, 1);
    assert!(cc.max_cc >= 2);
}

// ===========================================================================
// 9. Complexity: cognitive
// ===========================================================================

#[test]
fn cognitive_simple_is_low() {
    let code = r#"
fn simple() {
    println!("hello");
}
"#;
    let cog = estimate_cognitive_complexity(code, "rust");
    assert_eq!(cog.function_count, 1);
    assert!(cog.max <= 1);
}

#[test]
fn cognitive_nested_is_higher() {
    let code = r#"
fn nested(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            if x > 100 {
                return x;
            }
        }
    }
    0
}
"#;
    let cog = estimate_cognitive_complexity(code, "rust");
    assert!(
        cog.max >= 3,
        "deeply nested → high cognitive, got {}",
        cog.max
    );
}

#[test]
fn cognitive_empty() {
    let cog = estimate_cognitive_complexity("", "rust");
    assert_eq!(cog.function_count, 0);
    assert_eq!(cog.total, 0);
}

// ===========================================================================
// 10. Complexity: nesting depth
// ===========================================================================

#[test]
fn nesting_flat_code() {
    let code = r#"
fn flat() {
    let x = 1;
    let y = 2;
}
"#;
    let n = analyze_nesting_depth(code, "rust");
    assert!(n.max_depth <= 2, "flat code low nesting: {}", n.max_depth);
}

#[test]
fn nesting_deep_code() {
    let code = r#"
fn deep() {
    if true {
        for i in 0..10 {
            if i > 5 {
                while true {
                    break;
                }
            }
        }
    }
}
"#;
    let n = analyze_nesting_depth(code, "rust");
    assert!(
        n.max_depth >= 4,
        "deep nesting expected >= 4, got {}",
        n.max_depth
    );
}

#[test]
fn nesting_empty() {
    let n = analyze_nesting_depth("", "rust");
    assert_eq!(n.max_depth, 0);
}

#[test]
fn nesting_python_indentation() {
    let code = r#"
def outer():
    if True:
        for i in range(10):
            if i > 5:
                pass
"#;
    let n = analyze_nesting_depth(code, "python");
    assert!(n.max_depth >= 3, "python nesting >= 3, got {}", n.max_depth);
}

// ===========================================================================
// 11. BDD: Given source file / When scanning / Then tags correct
// ===========================================================================

#[test]
fn bdd_given_source_with_todos_when_scanning_then_count_correct() {
    // Given
    let text = "// TODO: fix\n// TODO: refactor\n// FIXME: bug\nlet x = 1;";
    // When
    let tags = count_tags(text, &["TODO", "FIXME"]);
    // Then
    assert_eq!(tags[0], ("TODO".to_string(), 2));
    assert_eq!(tags[1], ("FIXME".to_string(), 1));
}

#[test]
fn bdd_given_clean_code_when_scanning_then_no_tags() {
    let text = "fn main() { println!(\"hello\"); }";
    let tags = count_tags(text, &["TODO", "FIXME", "HACK"]);
    for (tag, count) in &tags {
        assert_eq!(*count, 0, "no {tag} expected in clean code");
    }
}

#[test]
fn bdd_given_file_when_hashing_then_deterministic() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("bdd_hash.rs");
    let mut f = File::create(&path).unwrap();
    f.write_all(b"fn main() {}").unwrap();

    let h1 = hash_file(&path, 10000).unwrap();
    let h2 = hash_file(&path, 10000).unwrap();
    assert_eq!(h1, h2, "same file → same hash");
}

#[test]
fn bdd_given_rust_code_when_analyzing_then_functions_found() {
    let code = r#"
fn alpha() { 1; }
fn beta() { 2; }
fn gamma() { 3; }
"#;
    let m = analyze_functions(code, "rust");
    assert_eq!(m.function_count, 3);
}

// ===========================================================================
// 12. Edge: empty file
// ===========================================================================

#[test]
fn edge_entropy_empty_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("empty");
    File::create(&path).unwrap();
    let bytes = read_head(&path, 1000).unwrap();
    let e = entropy_bits_per_byte(&bytes);
    assert_eq!(e, 0.0);
}

#[test]
fn edge_hash_empty_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("empty_hash");
    File::create(&path).unwrap();
    let h = hash_file(&path, 1000).unwrap();
    assert_eq!(h.len(), 64);
    assert_eq!(h, hash_bytes(b""));
}

#[test]
fn edge_tags_empty_file() {
    let tags = count_tags("", &["TODO"]);
    assert_eq!(tags[0].1, 0);
}

#[test]
fn edge_functions_empty_string() {
    let m = analyze_functions("", "rust");
    assert_eq!(m.function_count, 0);
}

#[test]
fn edge_cyclomatic_empty_string() {
    let cc = estimate_cyclomatic_complexity("", "rust");
    assert_eq!(cc.function_count, 0);
}

// ===========================================================================
// 13. Edge: binary file
// ===========================================================================

#[test]
fn edge_binary_not_text() {
    assert!(!is_text_like(&[0x00, 0x01, 0x02, 0xFF]));
}

#[test]
fn edge_binary_entropy() {
    let binary: Vec<u8> = (0u8..=255).collect();
    let e = entropy_bits_per_byte(&binary);
    assert!((7.9..=8.0).contains(&e), "uniform binary → ~8 bits: {e}");
}

#[test]
fn edge_binary_hash() {
    let binary = vec![0u8; 100];
    let h = hash_bytes(&binary);
    assert_eq!(h.len(), 64);
}

// ===========================================================================
// 14. Edge: very large content
// ===========================================================================

#[test]
fn edge_large_entropy() {
    let large: Vec<u8> = (0..100_000).map(|i| (i % 256) as u8).collect();
    let e = entropy_bits_per_byte(&large);
    assert!(e.is_finite(), "entropy should be finite, got {e}");
    assert!(e >= 0.0, "entropy non-negative, got {e}");
    assert!(e <= 8.01, "entropy <= ~8, got {e}");
}

#[test]
fn edge_large_hash() {
    let large = vec![0xABu8; 1_000_000];
    let h = hash_bytes(&large);
    assert_eq!(h.len(), 64);
}

#[test]
fn edge_large_tag_count() {
    let text = "TODO ".repeat(10_000);
    let tags = count_tags(&text, &["TODO"]);
    assert_eq!(tags[0].1, 10_000);
}

// ===========================================================================
// 15. Boundary: max entropy, zero entropy
// ===========================================================================

#[test]
fn boundary_zero_entropy_constant() {
    let buf = [0xAAu8; 5000];
    let e = entropy_bits_per_byte(&buf);
    assert!(e.abs() < 1e-6, "constant → 0 entropy, got {e}");
}

#[test]
fn boundary_max_entropy_uniform() {
    let buf: Vec<u8> = (0u8..=255).cycle().take(25600).collect();
    let e = entropy_bits_per_byte(&buf);
    assert!((7.99..=8.01).contains(&e), "uniform → ~8.0, got {e}");
}

#[test]
fn boundary_entropy_monotonic_with_diversity() {
    // More distinct byte values → higher entropy
    let e1 = entropy_bits_per_byte(&[0u8; 1000]);
    let buf2: Vec<u8> = (0..1000).map(|i| (i % 4) as u8).collect();
    let e2 = entropy_bits_per_byte(&buf2);
    let buf3: Vec<u8> = (0..1000).map(|i| (i % 64) as u8).collect();
    let e3 = entropy_bits_per_byte(&buf3);
    assert!(e1 < e2, "e1({e1}) < e2({e2})");
    assert!(e2 < e3, "e2({e2}) < e3({e3})");
}

#[test]
fn boundary_hash_single_byte_change() {
    let h1 = hash_bytes(&[0, 1, 2, 3]);
    let h2 = hash_bytes(&[0, 1, 2, 4]);
    assert_ne!(h1, h2, "single byte change → different hash");
}

#[test]
fn boundary_read_head_max_bytes_one() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("one_byte");
    let mut f = File::create(&path).unwrap();
    f.write_all(b"XYZ").unwrap();
    let bytes = read_head(&path, 1).unwrap();
    assert_eq!(bytes, b"X");
}

#[test]
fn boundary_read_head_tail_max_one() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("ht_one");
    let mut f = File::create(&path).unwrap();
    f.write_all(b"ABCDE").unwrap();
    let bytes = read_head_tail(&path, 1).unwrap();
    assert_eq!(bytes.len(), 1);
}

// ===========================================================================
// 16. Determinism
// ===========================================================================

#[test]
fn deterministic_hash_bytes() {
    for _ in 0..10 {
        let h1 = hash_bytes(b"stable");
        let h2 = hash_bytes(b"stable");
        assert_eq!(h1, h2);
    }
}

#[test]
fn deterministic_count_tags() {
    let text = "TODO FIXME TODO";
    let t1 = count_tags(text, &["TODO", "FIXME"]);
    let t2 = count_tags(text, &["TODO", "FIXME"]);
    assert_eq!(t1, t2);
}

#[test]
fn deterministic_entropy() {
    let data = b"deterministic entropy test data";
    let e1 = entropy_bits_per_byte(data);
    let e2 = entropy_bits_per_byte(data);
    assert_eq!(e1, e2);
}

#[test]
fn deterministic_analyze_functions() {
    let code = "fn main() { let x = 1; }\nfn other() { 2; }";
    let m1 = analyze_functions(code, "rust");
    let m2 = analyze_functions(code, "rust");
    assert_eq!(m1.function_count, m2.function_count);
    assert_eq!(m1.max_function_length, m2.max_function_length);
}

#[test]
fn deterministic_cyclomatic() {
    let code = "fn f(x: i32) { if x > 0 { 1 } else { 0 } }";
    let c1 = estimate_cyclomatic_complexity(code, "rust");
    let c2 = estimate_cyclomatic_complexity(code, "rust");
    assert_eq!(c1.max_cc, c2.max_cc);
    assert_eq!(c1.total_cc, c2.total_cc);
}
