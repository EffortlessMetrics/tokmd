//! Depth tests for tokmd-content scanning, entropy, tags, hashing, and complexity.
//!
//! 60+ BDD-style and property-based tests covering edge cases, binary handling,
//! UTF-8 boundaries, determinism, and large/small file behaviour.

use std::fs::File;
use std::io::Write;

use proptest::prelude::*;
use tokmd_content::complexity::{
    analyze_functions, analyze_nesting_depth, estimate_cognitive_complexity,
    estimate_cyclomatic_complexity,
};
use tokmd_content::{
    count_tags, entropy_bits_per_byte, hash_bytes, hash_file, is_text_like, read_head,
    read_head_tail, read_lines, read_text_capped,
};

// ============================================================================
// 1. Entropy edge cases
// ============================================================================

mod entropy_edge_cases {
    use super::*;

    #[test]
    fn empty_slice_returns_zero() {
        assert_eq!(entropy_bits_per_byte(&[]), 0.0);
    }

    #[test]
    fn single_byte_returns_zero() {
        // Only one symbol → no uncertainty
        assert_eq!(entropy_bits_per_byte(&[42]), 0.0);
    }

    #[test]
    fn uniform_buffer_returns_zero() {
        let buf = vec![0xFFu8; 4096];
        let e = entropy_bits_per_byte(&buf);
        assert!(e.abs() < 1e-6, "uniform bytes should yield ~0, got {e}");
    }

    #[test]
    fn two_equal_values_yield_one_bit() {
        let buf: Vec<u8> = (0..2048).map(|i| (i % 2) as u8).collect();
        let e = entropy_bits_per_byte(&buf);
        assert!(
            (e - 1.0).abs() < 0.02,
            "expected ~1.0 bit, got {e}"
        );
    }

    #[test]
    fn four_equal_values_yield_two_bits() {
        let buf: Vec<u8> = (0..2048).map(|i| (i % 4) as u8).collect();
        let e = entropy_bits_per_byte(&buf);
        assert!(
            (e - 2.0).abs() < 0.02,
            "expected ~2.0 bits, got {e}"
        );
    }

    #[test]
    fn eight_equal_values_yield_three_bits() {
        let buf: Vec<u8> = (0..2048).map(|i| (i % 8) as u8).collect();
        let e = entropy_bits_per_byte(&buf);
        assert!(
            (e - 3.0).abs() < 0.05,
            "expected ~3.0 bits, got {e}"
        );
    }

    #[test]
    fn full_byte_range_yields_eight_bits() {
        let buf: Vec<u8> = (0u8..=255).cycle().take(4096).collect();
        let e = entropy_bits_per_byte(&buf);
        assert!(
            (e - 8.0).abs() < 0.01,
            "expected ~8.0 bits, got {e}"
        );
    }

    #[test]
    fn skewed_distribution_lower_than_uniform() {
        // 90% byte-0, 10% byte-1
        let mut buf = vec![0u8; 900];
        buf.extend(vec![1u8; 100]);
        let e = entropy_bits_per_byte(&buf);
        assert!(e > 0.0, "must be positive");
        assert!(e < 1.0, "highly skewed should be well below 1 bit, got {e}");
    }

    #[test]
    fn entropy_monotonically_increases_with_distinct_values() {
        let mut prev = 0.0f32;
        for n in [1, 2, 4, 8, 16, 32, 64, 128, 256] {
            let buf: Vec<u8> = (0..2048).map(|i| (i % n) as u8).collect();
            let e = entropy_bits_per_byte(&buf);
            assert!(
                e >= prev - 0.01,
                "entropy should not decrease: n={n}, prev={prev}, cur={e}"
            );
            prev = e;
        }
    }

    #[test]
    fn entropy_finite_for_all_byte_value_255() {
        let buf = vec![255u8; 1];
        let e = entropy_bits_per_byte(&buf);
        assert!(e.is_finite());
    }

    #[test]
    fn random_like_data_has_high_entropy() {
        // Pseudo-random via simple LCG
        let mut v = Vec::with_capacity(1024);
        let mut x: u32 = 12345;
        for _ in 0..1024 {
            x = x.wrapping_mul(1103515245).wrapping_add(12345);
            v.push((x >> 16) as u8);
        }
        let e = entropy_bits_per_byte(&v);
        assert!(e > 7.0, "pseudo-random data should have high entropy, got {e}");
    }
}

// ============================================================================
// 2. Import / dependency tag extraction
// ============================================================================

mod import_extraction {
    use super::*;

    #[test]
    fn rust_use_statements() {
        let code = "use std::io;\nuse std::fs::File;\nuse crate::model;";
        let result = count_tags(code, &["use"]);
        assert_eq!(result[0].1, 3);
    }

    #[test]
    fn python_import_and_from() {
        let code = "import os\nfrom pathlib import Path\nimport json\nfrom sys import argv";
        let result = count_tags(code, &["import"]);
        // "import" appears 4 times (each line has at least one)
        assert_eq!(result[0].1, 4);
    }

    #[test]
    fn javascript_require() {
        let code = "const fs = require('fs');\nconst p = require('path');";
        let result = count_tags(code, &["require"]);
        assert_eq!(result[0].1, 2);
    }

    #[test]
    fn go_import_keyword() {
        let code = "import (\n  \"fmt\"\n  \"os\"\n)\nimport \"strings\"";
        let result = count_tags(code, &["import"]);
        assert_eq!(result[0].1, 2);
    }

    #[test]
    fn java_import_statements() {
        let code = "import java.util.List;\nimport java.io.File;";
        let result = count_tags(code, &["import"]);
        assert_eq!(result[0].1, 2);
    }

    #[test]
    fn mixed_language_imports() {
        let code = "use foo;\nimport bar;\nconst x = require('baz');";
        let result = count_tags(code, &["use", "import", "require"]);
        assert_eq!(result[0].1, 1, "use");
        assert_eq!(result[1].1, 1, "import");
        assert_eq!(result[2].1, 1, "require");
    }

    #[test]
    fn no_imports_in_plain_text() {
        let text = "Hello world, this is a plain sentence.";
        let result = count_tags(text, &["use", "import", "require"]);
        assert_eq!(result[0].1, 0);
        assert_eq!(result[1].1, 0);
        assert_eq!(result[2].1, 0);
    }
}

// ============================================================================
// 3. TODO / FIXME tag detection
// ============================================================================

mod tag_detection {
    use super::*;

    #[test]
    fn detects_todo_fixme_hack() {
        let code = "// TODO: implement\n// FIXME: crash\n// HACK: temp";
        let result = count_tags(code, &["TODO", "FIXME", "HACK"]);
        assert_eq!(result[0].1, 1);
        assert_eq!(result[1].1, 1);
        assert_eq!(result[2].1, 1);
    }

    #[test]
    fn case_insensitive_matching() {
        let text = "todo Todo TODO toDo";
        let result = count_tags(text, &["TODO"]);
        assert_eq!(result[0].1, 4);
    }

    #[test]
    fn no_tags_in_empty_text() {
        let result = count_tags("", &["TODO", "FIXME"]);
        assert_eq!(result[0].1, 0);
        assert_eq!(result[1].1, 0);
    }

    #[test]
    fn adjacent_tags_all_counted() {
        // "TODOTODO" contains "TODO" starting at 0 and at 4
        let result = count_tags("TODOTODO", &["TODO"]);
        assert_eq!(result[0].1, 2);
    }

    #[test]
    fn overlapping_tags_counted_by_matches() {
        // str::matches is non-overlapping
        let text = "TODOTODOTODO";
        let result = count_tags(text, &["TODO"]);
        assert_eq!(result[0].1, 3);
    }

    #[test]
    fn tags_in_python_comments() {
        let code = "# TODO: fix later\n# FIXME: broken\ndef foo(): pass";
        let result = count_tags(code, &["TODO", "FIXME"]);
        assert_eq!(result[0].1, 1);
        assert_eq!(result[1].1, 1);
    }

    #[test]
    fn tags_in_multiline_rust_code() {
        let code = "\
fn main() {
    // TODO: first
    let x = 1;
    // FIXME: second
    // TODO: third
}";
        let result = count_tags(code, &["TODO", "FIXME"]);
        assert_eq!(result[0].1, 2, "TODO count");
        assert_eq!(result[1].1, 1, "FIXME count");
    }

    #[test]
    fn empty_tag_list_returns_empty_results() {
        let result = count_tags("some text", &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn tags_preserves_input_order() {
        let result = count_tags("TODO FIXME HACK", &["HACK", "FIXME", "TODO"]);
        assert_eq!(result[0].0, "HACK");
        assert_eq!(result[1].0, "FIXME");
        assert_eq!(result[2].0, "TODO");
    }
}

// ============================================================================
// 4. File hashing determinism
// ============================================================================

mod hashing {
    use super::*;

    #[test]
    fn hash_bytes_deterministic() {
        let h1 = hash_bytes(b"deterministic content");
        let h2 = hash_bytes(b"deterministic content");
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_bytes_64_hex_chars() {
        let h = hash_bytes(b"test");
        assert_eq!(h.len(), 64);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn hash_bytes_lowercase() {
        let h = hash_bytes(b"test");
        assert!(h.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));
    }

    #[test]
    fn hash_bytes_different_inputs_differ() {
        let h1 = hash_bytes(b"alpha");
        let h2 = hash_bytes(b"beta");
        assert_ne!(h1, h2);
    }

    #[test]
    fn hash_empty_is_well_defined() {
        let h = hash_bytes(b"");
        assert_eq!(h.len(), 64);
        // BLAKE3 hash of empty input is a specific constant
        let h2 = hash_bytes(b"");
        assert_eq!(h, h2);
    }

    #[test]
    fn hash_file_matches_hash_bytes() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("match.txt");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"hello hash").unwrap();

        let file_hash = hash_file(&path, 1024).unwrap();
        let bytes_hash = hash_bytes(b"hello hash");
        assert_eq!(file_hash, bytes_hash);
    }

    #[test]
    fn hash_file_respects_max_bytes() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("limited.txt");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"abcdefghij").unwrap();

        let hash_5 = hash_file(&path, 5).unwrap();
        let expected = hash_bytes(b"abcde");
        assert_eq!(hash_5, expected);
    }

    #[test]
    fn hash_file_empty_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("empty");
        File::create(&path).unwrap();

        let h = hash_file(&path, 1024).unwrap();
        assert_eq!(h, hash_bytes(b""));
    }
}

// ============================================================================
// 5. Binary file handling / is_text_like
// ============================================================================

mod binary_handling {
    use super::*;

    #[test]
    fn null_byte_means_not_text() {
        assert!(!is_text_like(&[0]));
        assert!(!is_text_like(b"hello\x00world"));
    }

    #[test]
    fn empty_bytes_are_text_like() {
        assert!(is_text_like(&[]));
    }

    #[test]
    fn pure_ascii_is_text_like() {
        assert!(is_text_like(b"Hello, World! 123"));
    }

    #[test]
    fn valid_utf8_multibyte_is_text_like() {
        assert!(is_text_like("こんにちは".as_bytes()));
        assert!(is_text_like("émojis 🎉🚀".as_bytes()));
    }

    #[test]
    fn invalid_utf8_without_null_is_not_text_like() {
        // 0xFF alone is invalid UTF-8
        assert!(!is_text_like(&[0xFF]));
        // Incomplete multi-byte sequence
        assert!(!is_text_like(&[0xC0, 0x20]));
    }

    #[test]
    fn binary_exe_header_is_not_text_like() {
        // Simulated PE header bytes
        let header = [0x4D, 0x5A, 0x90, 0x00, 0x03, 0x00, 0x00, 0x00];
        assert!(!is_text_like(&header));
    }

    #[test]
    fn all_printable_ascii_is_text() {
        let buf: Vec<u8> = (0x20..=0x7E).collect();
        assert!(is_text_like(&buf));
    }
}

// ============================================================================
// 6. UTF-8 edge cases
// ============================================================================

mod utf8_edges {
    use super::*;

    #[test]
    fn read_text_capped_handles_multibyte_utf8() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("utf8.txt");
        let mut f = File::create(&path).unwrap();
        // Each Kanji char is 3 bytes in UTF-8
        f.write_all("漢字テスト".as_bytes()).unwrap();

        let text = read_text_capped(&path, 1024).unwrap();
        assert_eq!(text, "漢字テスト");
    }

    #[test]
    fn read_text_capped_lossy_on_truncated_multibyte() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("trunc.txt");
        let mut f = File::create(&path).unwrap();
        // "漢" is 3 bytes: E6 BC A2. Reading only 2 bytes truncates mid-char.
        f.write_all("漢字".as_bytes()).unwrap();

        let text = read_text_capped(&path, 2).unwrap();
        // from_utf8_lossy replaces invalid sequence
        assert!(text.contains('\u{FFFD}') || text.len() <= 6);
    }

    #[test]
    fn entropy_of_utf8_text() {
        let text = "Ünïcödé tëxt wïth dîäcrïtïcs";
        let e = entropy_bits_per_byte(text.as_bytes());
        assert!(e > 2.0, "UTF-8 text should have moderate entropy, got {e}");
        assert!(e < 8.0);
    }

    #[test]
    fn hash_utf8_string_deterministic() {
        let s = "日本語テスト";
        let h1 = hash_bytes(s.as_bytes());
        let h2 = hash_bytes(s.as_bytes());
        assert_eq!(h1, h2);
    }
}

// ============================================================================
// 7. Very large and very small files
// ============================================================================

mod file_sizes {
    use super::*;

    #[test]
    fn read_head_on_zero_byte_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("empty");
        File::create(&path).unwrap();

        let data = read_head(&path, 4096).unwrap();
        assert!(data.is_empty());
    }

    #[test]
    fn read_head_with_zero_max() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("zero_max.txt");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"some content").unwrap();

        let data = read_head(&path, 0).unwrap();
        assert!(data.is_empty());
    }

    #[test]
    fn read_head_tail_zero_max_bytes() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("ht_zero.txt");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"data").unwrap();

        let data = read_head_tail(&path, 0).unwrap();
        assert!(data.is_empty());
    }

    #[test]
    fn read_lines_zero_max_lines() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("zero_lines.txt");
        let mut f = File::create(&path).unwrap();
        writeln!(f, "line1").unwrap();

        let lines = read_lines(&path, 0, 4096).unwrap();
        assert!(lines.is_empty());
    }

    #[test]
    fn read_lines_zero_max_bytes() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("zero_bytes.txt");
        let mut f = File::create(&path).unwrap();
        writeln!(f, "line1").unwrap();

        let lines = read_lines(&path, 100, 0).unwrap();
        assert!(lines.is_empty());
    }

    #[test]
    fn large_file_head_tail_selects_extremes() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("big.bin");
        let mut f = File::create(&path).unwrap();
        // 10000 bytes: 0x00..0x00 then middle then 0xFF..0xFF
        let mut data = vec![0xAAu8; 5000];
        data.extend(vec![0xBBu8; 5000]);
        f.write_all(&data).unwrap();

        // Read head(5) + tail(5) = 10 bytes from a 10000 byte file
        let result = read_head_tail(&path, 10).unwrap();
        assert_eq!(result.len(), 10);
        // First 5 should be 0xAA (head)
        assert_eq!(&result[..5], &[0xAA; 5]);
        // Last 5 should be 0xBB (tail)
        assert_eq!(&result[5..], &[0xBB; 5]);
    }

    #[test]
    fn entropy_large_uniform_buffer() {
        let buf = vec![42u8; 100_000];
        let e = entropy_bits_per_byte(&buf);
        assert!(e.abs() < 1e-6, "uniform 100K bytes should have ~0 entropy");
    }

    #[test]
    fn hash_file_large_content() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("big.txt");
        let mut f = File::create(&path).unwrap();
        let data = vec![b'x'; 50_000];
        f.write_all(&data).unwrap();

        let h = hash_file(&path, 50_000).unwrap();
        assert_eq!(h.len(), 64);
        let expected = hash_bytes(&data);
        assert_eq!(h, expected);
    }
}

// ============================================================================
// 8. Complexity analysis
// ============================================================================

mod complexity_tests {
    use super::*;

    #[test]
    fn empty_code_yields_default_metrics() {
        let m = analyze_functions("", "rust");
        assert_eq!(m.function_count, 0);
        assert_eq!(m.max_function_length, 0);
    }

    #[test]
    fn single_rust_function_detected() {
        let code = "\
fn hello() {
    println!(\"hi\");
}
";
        let m = analyze_functions(code, "rust");
        assert_eq!(m.function_count, 1);
        assert!(m.max_function_length >= 2);
    }

    #[test]
    fn multiple_rust_functions_counted() {
        let code = "\
fn a() {
    let x = 1;
}

fn b() {
    let y = 2;
}

fn c() {
    let z = 3;
}
";
        let m = analyze_functions(code, "rust");
        assert_eq!(m.function_count, 3);
    }

    #[test]
    fn python_def_detected() {
        let code = "\
def greet(name):
    print(f'Hello {name}')

def farewell():
    print('Bye')
";
        let m = analyze_functions(code, "python");
        assert_eq!(m.function_count, 2);
    }

    #[test]
    fn go_func_detected() {
        let code = "\
func main() {
    fmt.Println(\"hello\")
}

func helper() {
    return
}
";
        let m = analyze_functions(code, "go");
        assert_eq!(m.function_count, 2);
    }

    #[test]
    fn javascript_function_detected() {
        let code = "\
function greet() {
    console.log('hi');
}
";
        let m = analyze_functions(code, "javascript");
        assert_eq!(m.function_count, 1);
    }

    #[test]
    fn unknown_language_returns_zero_functions() {
        let code = "proc do_thing\n  puts 'hello'\nend";
        let m = analyze_functions(code, "ruby");
        assert_eq!(m.function_count, 0);
    }

    #[test]
    fn cyclomatic_complexity_simple_function() {
        let code = "\
fn simple() {
    let x = 1;
}
";
        let cc = estimate_cyclomatic_complexity(code, "rust");
        assert_eq!(cc.function_count, 1);
        assert_eq!(cc.max_cc, 1, "no branches → CC=1");
    }

    #[test]
    fn cyclomatic_complexity_with_branches() {
        let code = "\
fn branchy(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            return 100;
        }
        return x;
    }
    0
}
";
        let cc = estimate_cyclomatic_complexity(code, "rust");
        assert_eq!(cc.function_count, 1);
        assert!(cc.max_cc >= 3, "two if branches → CC >= 3, got {}", cc.max_cc);
    }

    #[test]
    fn cognitive_complexity_empty_code() {
        let cog = estimate_cognitive_complexity("", "rust");
        assert_eq!(cog.function_count, 0);
        assert_eq!(cog.total, 0);
    }

    #[test]
    fn cognitive_complexity_nested_ifs() {
        let code = "\
fn nested(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            return x;
        }
    }
    0
}
";
        let cog = estimate_cognitive_complexity(code, "rust");
        assert_eq!(cog.function_count, 1);
        assert!(cog.max >= 2, "nested ifs should contribute cognitive load, got {}", cog.max);
    }

    #[test]
    fn nesting_depth_empty() {
        let n = analyze_nesting_depth("", "rust");
        assert_eq!(n.max_depth, 0);
        assert_eq!(n.avg_depth, 0.0);
    }

    #[test]
    fn nesting_depth_flat_code() {
        let code = "let x = 1;\nlet y = 2;\n";
        let n = analyze_nesting_depth(code, "rust");
        assert_eq!(n.max_depth, 0);
    }

    #[test]
    fn nesting_depth_one_level() {
        let code = "\
fn main() {
    let x = 1;
}
";
        let n = analyze_nesting_depth(code, "rust");
        assert!(n.max_depth >= 1);
    }

    #[test]
    fn nesting_depth_python_indentation() {
        let code = "\
def foo():
    if True:
        for i in range(10):
            print(i)
";
        let n = analyze_nesting_depth(code, "python");
        assert!(n.max_depth >= 2, "deeply indented Python, got {}", n.max_depth);
    }
}

// ============================================================================
// 9. read_head_tail specifics
// ============================================================================

mod read_head_tail_tests {
    use super::*;

    #[test]
    fn small_file_returns_all_content() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("small.txt");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"abc").unwrap();

        let data = read_head_tail(&path, 100).unwrap();
        assert_eq!(data, b"abc");
    }

    #[test]
    fn exact_size_returns_all() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("exact.txt");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"12345").unwrap();

        let data = read_head_tail(&path, 5).unwrap();
        assert_eq!(data, b"12345");
    }

    #[test]
    fn odd_max_bytes_splits_correctly() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("odd.txt");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"ABCDEFGHIJ").unwrap(); // 10 bytes

        // max_bytes=3: half=1, head=1("A"), tail=2("IJ")
        let data = read_head_tail(&path, 3).unwrap();
        assert_eq!(data.len(), 3);
        assert_eq!(data[0], b'A');
    }

    #[test]
    fn max_bytes_one_returns_head_only() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("one.txt");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"XYZW").unwrap();

        // max_bytes=1: half=0, head=max(0,1)=1, tail=0
        let data = read_head_tail(&path, 1).unwrap();
        assert_eq!(data, b"X");
    }
}

// ============================================================================
// 10. read_lines edge cases
// ============================================================================

mod read_lines_tests {
    use super::*;

    #[test]
    fn empty_file_returns_no_lines() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("empty.txt");
        File::create(&path).unwrap();

        let lines = read_lines(&path, 100, 10_000).unwrap();
        assert!(lines.is_empty());
    }

    #[test]
    fn single_line_no_newline() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("single.txt");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"no newline").unwrap();

        let lines = read_lines(&path, 100, 10_000).unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "no newline");
    }

    #[test]
    fn respects_max_lines_limit() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("many.txt");
        let mut f = File::create(&path).unwrap();
        for i in 0..20 {
            writeln!(f, "line {i}").unwrap();
        }

        let lines = read_lines(&path, 5, 100_000).unwrap();
        assert_eq!(lines.len(), 5);
    }
}

// ============================================================================
// 11. Property-based tests
// ============================================================================

proptest! {
    #[test]
    fn prop_entropy_bounded(bytes in prop::collection::vec(any::<u8>(), 0..2048)) {
        let e = entropy_bits_per_byte(&bytes);
        prop_assert!(e >= 0.0);
        prop_assert!(e <= 8.0 + 1e-6);
        prop_assert!(e.is_finite());
    }

    #[test]
    fn prop_entropy_single_value_is_zero(byte in any::<u8>(), len in 1usize..512) {
        let buf = vec![byte; len];
        let e = entropy_bits_per_byte(&buf);
        prop_assert!(e.abs() < 1e-5, "single value entropy should be ~0, got {}", e);
    }

    #[test]
    fn prop_hash_deterministic(data in prop::collection::vec(any::<u8>(), 0..1024)) {
        let h1 = hash_bytes(&data);
        let h2 = hash_bytes(&data);
        prop_assert_eq!(h1, h2);
    }

    #[test]
    fn prop_hash_length_always_64(data in prop::collection::vec(any::<u8>(), 0..512)) {
        let h = hash_bytes(&data);
        prop_assert_eq!(h.len(), 64);
    }

    #[test]
    fn prop_hash_hex_only(data in prop::collection::vec(any::<u8>(), 0..512)) {
        let h = hash_bytes(&data);
        prop_assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn prop_is_text_like_no_null_implies_utf8_check(bytes in prop::collection::vec(1u8..=255, 0..256)) {
        let result = is_text_like(&bytes);
        let valid_utf8 = std::str::from_utf8(&bytes).is_ok();
        prop_assert_eq!(result, valid_utf8);
    }

    #[test]
    fn prop_null_byte_makes_not_text(
        prefix in prop::collection::vec(any::<u8>(), 0..32),
        suffix in prop::collection::vec(any::<u8>(), 0..32),
    ) {
        let mut data = prefix;
        data.push(0);
        data.extend(suffix);
        prop_assert!(!is_text_like(&data));
    }

    #[test]
    fn prop_count_tags_returns_correct_length(
        text in "\\PC{0,64}",
        n_tags in 0usize..6,
    ) {
        let tags: Vec<String> = (0..n_tags).map(|i| format!("TAG{i}")).collect();
        let tag_refs: Vec<&str> = tags.iter().map(|s| s.as_str()).collect();
        let result = count_tags(&text, &tag_refs);
        prop_assert_eq!(result.len(), n_tags);
    }

    #[test]
    fn prop_count_tags_known_repetition(count in 0usize..20) {
        let text = "FIXME ".repeat(count);
        let result = count_tags(&text, &["FIXME"]);
        prop_assert_eq!(result[0].1, count);
    }

    #[test]
    fn prop_read_text_capped_length_bounded(content in "[a-zA-Z0-9 ]{0,200}", cap in 1usize..50) {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("prop.txt");
        let mut f = File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        drop(f);

        let text = read_text_capped(&path, cap).unwrap();
        prop_assert!(text.len() <= cap, "capped text too long: {} > {}", text.len(), cap);
    }

    #[test]
    fn prop_hash_different_data_different_hash(
        a in prop::collection::vec(any::<u8>(), 1..128),
        b in prop::collection::vec(any::<u8>(), 1..128),
    ) {
        prop_assume!(a != b);
        let h1 = hash_bytes(&a);
        let h2 = hash_bytes(&b);
        prop_assert_ne!(h1, h2);
    }

    #[test]
    fn prop_entropy_more_values_at_least_as_high(n in 2usize..64) {
        let multi: Vec<u8> = (0..512).map(|i| (i % n) as u8).collect();
        let single = vec![0u8; 512];
        let e_multi = entropy_bits_per_byte(&multi);
        let e_single = entropy_bits_per_byte(&single);
        prop_assert!(e_multi >= e_single - 0.001,
            "n={}: multi={} should >= single={}", n, e_multi, e_single);
    }

    #[test]
    fn prop_analyze_functions_empty_any_lang(
        lang in prop::sample::select(vec!["rust", "python", "javascript", "go", "unknown"]),
    ) {
        let m = analyze_functions("", lang);
        prop_assert_eq!(m.function_count, 0);
    }

    #[test]
    fn prop_nesting_depth_non_negative(
        lang in prop::sample::select(vec!["rust", "python", "javascript", "go"]),
    ) {
        let n = analyze_nesting_depth("", lang);
        prop_assert!(n.avg_depth >= 0.0);
        prop_assert_eq!(n.max_depth, 0);
    }
}
