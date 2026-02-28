//! BDD-style scenario tests for tokmd-content.
//!
//! Covers entropy calculation, hash computation, text detection,
//! tag counting, and complexity analysis with Given/When/Then structure.

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

// ============================================================================
// Entropy Calculation Scenarios
// ============================================================================

mod entropy {
    use super::*;

    #[test]
    fn scenario_empty_input_yields_zero_entropy() {
        // Given an empty byte slice
        let bytes: &[u8] = &[];
        // When we compute entropy
        let entropy = entropy_bits_per_byte(bytes);
        // Then entropy is exactly zero
        assert_eq!(entropy, 0.0);
    }

    #[test]
    fn scenario_single_repeated_byte_yields_zero_entropy() {
        // Given a buffer of identical bytes
        let bytes = vec![0xAA; 1000];
        // When we compute entropy
        let entropy = entropy_bits_per_byte(&bytes);
        // Then entropy is zero (no uncertainty)
        assert!(entropy.abs() < 1e-6, "got {entropy}");
    }

    #[test]
    fn scenario_two_equally_distributed_values_yield_one_bit() {
        // Given alternating 0 and 1 bytes (equal distribution)
        let bytes: Vec<u8> = (0..1000).map(|i| (i % 2) as u8).collect();
        // When we compute entropy
        let entropy = entropy_bits_per_byte(&bytes);
        // Then entropy ≈ 1.0 bit per byte
        assert!(
            (entropy - 1.0).abs() < 0.01,
            "expected ~1.0 bit, got {entropy}"
        );
    }

    #[test]
    fn scenario_four_equally_distributed_values_yield_two_bits() {
        // Given alternating 0,1,2,3 bytes
        let bytes: Vec<u8> = (0..1000).map(|i| (i % 4) as u8).collect();
        // When we compute entropy
        let entropy = entropy_bits_per_byte(&bytes);
        // Then entropy ≈ 2.0 bits per byte
        assert!(
            (entropy - 2.0).abs() < 0.01,
            "expected ~2.0 bits, got {entropy}"
        );
    }

    #[test]
    fn scenario_full_byte_range_yields_eight_bits() {
        // Given all 256 byte values equally represented
        let bytes: Vec<u8> = (0u8..=255).collect();
        // When we compute entropy
        let entropy = entropy_bits_per_byte(&bytes);
        // Then entropy is exactly 8.0 bits per byte (max)
        assert!(
            (entropy - 8.0).abs() < 0.01,
            "expected ~8.0 bits, got {entropy}"
        );
    }

    #[test]
    fn scenario_skewed_distribution_yields_low_entropy() {
        // Given a buffer that's 99% one byte and 1% another
        let mut bytes = vec![0x00; 990];
        bytes.extend(vec![0xFF; 10]);
        // When we compute entropy
        let entropy = entropy_bits_per_byte(&bytes);
        // Then entropy is low (close to 0, far from 8)
        assert!(entropy < 0.15, "expected low entropy, got {entropy}");
    }

    #[test]
    fn scenario_natural_text_has_moderate_entropy() {
        // Given typical English text
        let text = b"The quick brown fox jumps over the lazy dog. \
                     This is a sample of natural language text that \
                     should have moderate Shannon entropy.";
        // When we compute entropy
        let entropy = entropy_bits_per_byte(text);
        // Then entropy is moderate (between 3 and 5 bits for English)
        assert!(
            entropy > 3.0 && entropy < 5.5,
            "expected 3-5.5 bits for English text, got {entropy}"
        );
    }

    #[test]
    fn scenario_single_byte_input_yields_zero_entropy() {
        // Given a one-byte input
        let bytes: &[u8] = &[42];
        // When we compute entropy
        let entropy = entropy_bits_per_byte(bytes);
        // Then entropy is zero (only one symbol, no uncertainty)
        assert!(entropy.abs() < 1e-6, "got {entropy}");
    }

    #[test]
    fn scenario_entropy_is_non_negative_for_any_input() {
        // Given various byte patterns
        for pattern in [
            vec![0u8; 100],
            vec![255u8; 100],
            (0u8..=255).collect::<Vec<_>>(),
            b"hello world".to_vec(),
        ] {
            // When we compute entropy
            let entropy = entropy_bits_per_byte(&pattern);
            // Then it is always >= 0
            assert!(entropy >= 0.0, "entropy should be non-negative: {entropy}");
        }
    }
}

// ============================================================================
// Hash Computation Scenarios
// ============================================================================

mod hashing {
    use super::*;

    #[test]
    fn scenario_hash_is_64_hex_characters() {
        // Given some content
        let content = b"deterministic hashing test";
        // When we hash it
        let hash = hash_bytes(content);
        // Then we get a 64-character hex string (BLAKE3)
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn scenario_same_content_produces_same_hash() {
        // Given the same content
        let content = b"reproducible";
        // When we hash it twice
        let h1 = hash_bytes(content);
        let h2 = hash_bytes(content);
        // Then both hashes are identical
        assert_eq!(h1, h2);
    }

    #[test]
    fn scenario_different_content_produces_different_hash() {
        // Given two different inputs
        let a = b"alpha";
        let b = b"bravo";
        // When we hash both
        let ha = hash_bytes(a);
        let hb = hash_bytes(b);
        // Then hashes differ
        assert_ne!(ha, hb);
    }

    #[test]
    fn scenario_empty_input_produces_valid_hash() {
        // Given empty input
        let hash = hash_bytes(&[]);
        // Then we still get a valid 64-char hex hash
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn scenario_hash_file_matches_hash_bytes() {
        // Given a file with known content
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("hashtest.bin");
        let content = b"file hash consistency check";
        File::create(&path)
            .unwrap()
            .write_all(content)
            .unwrap();

        // When we hash the file and hash bytes directly
        let file_hash = hash_file(&path, 10000).unwrap();
        let bytes_hash = hash_bytes(content);

        // Then both match
        assert_eq!(file_hash, bytes_hash);
    }

    #[test]
    fn scenario_hash_file_respects_max_bytes() {
        // Given a file with content "abcdefghij"
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("truncated.bin");
        File::create(&path)
            .unwrap()
            .write_all(b"abcdefghij")
            .unwrap();

        // When we hash only the first 5 bytes
        let truncated = hash_file(&path, 5).unwrap();
        let full = hash_file(&path, 100).unwrap();

        // Then the truncated hash equals hash_bytes of "abcde"
        assert_eq!(truncated, hash_bytes(b"abcde"));
        // And the full hash differs from the truncated one
        assert_ne!(truncated, full);
    }

    #[test]
    fn scenario_one_bit_flip_changes_hash() {
        // Given content that differs by one bit
        let a = b"aaaa";
        let mut b = *a;
        b[0] ^= 0x01; // flip one bit
        // When we hash both
        let ha = hash_bytes(a);
        let hb = hash_bytes(&b);
        // Then hashes are different (avalanche effect)
        assert_ne!(ha, hb);
    }
}

// ============================================================================
// Text Detection Scenarios
// ============================================================================

mod text_detection {
    use super::*;

    #[test]
    fn scenario_empty_input_is_text_like() {
        // Given an empty byte slice
        // When we check if it's text-like
        // Then it returns true
        assert!(is_text_like(&[]));
    }

    #[test]
    fn scenario_valid_ascii_is_text_like() {
        // Given plain ASCII text
        let bytes = b"Hello, World! 123";
        // When we check if it's text-like
        // Then it returns true
        assert!(is_text_like(bytes));
    }

    #[test]
    fn scenario_valid_utf8_is_text_like() {
        // Given valid UTF-8 with multi-byte characters
        let bytes = "こんにちは世界 🌍".as_bytes();
        // When we check if it's text-like
        // Then it returns true
        assert!(is_text_like(bytes));
    }

    #[test]
    fn scenario_null_byte_makes_it_not_text_like() {
        // Given bytes containing a null byte
        let bytes = b"hello\x00world";
        // When we check if it's text-like
        // Then it returns false
        assert!(!is_text_like(bytes));
    }

    #[test]
    fn scenario_pure_binary_with_nulls_not_text_like() {
        // Given binary data with embedded nulls
        let bytes: Vec<u8> = vec![0x00, 0xFF, 0x00, 0xFE, 0x00];
        // When we check if it's text-like
        // Then it returns false
        assert!(!is_text_like(&bytes));
    }

    #[test]
    fn scenario_invalid_utf8_without_nulls_not_text_like() {
        // Given invalid UTF-8 sequences (no null bytes)
        let bytes: &[u8] = &[0xFF, 0xFE, 0xFD];
        // When we check if it's text-like
        // Then it returns false (invalid UTF-8)
        assert!(!is_text_like(bytes));
    }

    #[test]
    fn scenario_newlines_and_tabs_are_text_like() {
        // Given text with whitespace characters
        let bytes = b"line1\nline2\ttabbed\r\nwindows";
        // When we check if it's text-like
        // Then it returns true
        assert!(is_text_like(bytes));
    }
}

// ============================================================================
// Tag Counting Scenarios
// ============================================================================

mod tag_counting {
    use super::*;

    #[test]
    fn scenario_counts_todo_tags() {
        // Given code with TODO comments
        let text = "// TODO: fix this\n// TODO: also this\nlet x = 1;";
        // When we count TODO tags
        let result = count_tags(text, &["TODO"]);
        // Then we find 2
        assert_eq!(result, vec![("TODO".to_string(), 2)]);
    }

    #[test]
    fn scenario_case_insensitive_matching() {
        // Given text with mixed-case tags
        let text = "todo Todo TODO tOdO";
        // When we count "TODO"
        let result = count_tags(text, &["TODO"]);
        // Then all 4 are found (case-insensitive)
        assert_eq!(result[0].1, 4);
    }

    #[test]
    fn scenario_multiple_tags_counted_independently() {
        // Given text with TODO and FIXME
        let text = "// TODO: implement\n// FIXME: broken\n// TODO: refactor";
        // When we count both
        let result = count_tags(text, &["TODO", "FIXME"]);
        // Then TODO=2, FIXME=1
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("TODO".to_string(), 2));
        assert_eq!(result[1], ("FIXME".to_string(), 1));
    }

    #[test]
    fn scenario_empty_text_yields_zero_counts() {
        // Given empty text
        let result = count_tags("", &["TODO", "FIXME", "HACK"]);
        // Then all counts are zero
        for (_, count) in &result {
            assert_eq!(*count, 0);
        }
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn scenario_no_tags_yields_empty_results() {
        // Given text but no tags to search for
        let result = count_tags("some text here", &[]);
        // Then result is empty
        assert!(result.is_empty());
    }

    #[test]
    fn scenario_tag_not_present_yields_zero() {
        // Given text without the searched tag
        let result = count_tags("no markers here", &["TODO"]);
        // Then count is zero
        assert_eq!(result[0].1, 0);
    }

    #[test]
    fn scenario_preserves_tag_order_in_results() {
        // Given a specific tag order
        let tags = &["FIXME", "TODO", "HACK", "NOTE"];
        let result = count_tags("TODO FIXME", tags);
        // Then results preserve the input order
        assert_eq!(result[0].0, "FIXME");
        assert_eq!(result[1].0, "TODO");
        assert_eq!(result[2].0, "HACK");
        assert_eq!(result[3].0, "NOTE");
    }
}

// ============================================================================
// File Reading Scenarios
// ============================================================================

mod file_reading {
    use super::*;

    #[test]
    fn scenario_read_head_returns_first_n_bytes() {
        // Given a file with content "abcdefghij"
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("head.txt");
        File::create(&path)
            .unwrap()
            .write_all(b"abcdefghij")
            .unwrap();
        // When we read the first 5 bytes
        let bytes = read_head(&path, 5).unwrap();
        // Then we get "abcde"
        assert_eq!(bytes, b"abcde");
    }

    #[test]
    fn scenario_read_head_tail_captures_boundaries() {
        // Given a 10-byte file "0123456789"
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("ht.txt");
        File::create(&path)
            .unwrap()
            .write_all(b"0123456789")
            .unwrap();
        // When we request 6 bytes head+tail
        let bytes = read_head_tail(&path, 6).unwrap();
        // Then we get first 3 + last 3 = "012789"
        assert_eq!(bytes, b"012789");
    }

    #[test]
    fn scenario_read_head_tail_zero_bytes_returns_empty() {
        // Given any file
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("zero.txt");
        File::create(&path)
            .unwrap()
            .write_all(b"content")
            .unwrap();
        // When we request 0 bytes
        let bytes = read_head_tail(&path, 0).unwrap();
        // Then we get empty
        assert!(bytes.is_empty());
    }

    #[test]
    fn scenario_read_head_tail_file_smaller_than_limit() {
        // Given a 5-byte file
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("small.txt");
        File::create(&path)
            .unwrap()
            .write_all(b"hello")
            .unwrap();
        // When we request 100 bytes
        let bytes = read_head_tail(&path, 100).unwrap();
        // Then we get entire file
        assert_eq!(bytes, b"hello");
    }

    #[test]
    fn scenario_read_lines_limits_by_count() {
        // Given a multi-line file
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("lines.txt");
        let mut f = File::create(&path).unwrap();
        for i in 0..20 {
            writeln!(f, "line {i}").unwrap();
        }
        // When we request at most 5 lines
        let lines = read_lines(&path, 5, usize::MAX).unwrap();
        // Then we get exactly 5
        assert_eq!(lines.len(), 5);
        assert_eq!(lines[0], "line 0");
        assert_eq!(lines[4], "line 4");
    }

    #[test]
    fn scenario_read_text_capped_handles_truncation_gracefully() {
        // Given a file with UTF-8 content
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("utf8.txt");
        File::create(&path)
            .unwrap()
            .write_all("Hello 🌍 World".as_bytes())
            .unwrap();
        // When we cap at a byte boundary that splits a multi-byte char
        let text = read_text_capped(&path, 7).unwrap();
        // Then it uses lossy conversion (no panic, valid string)
        assert!(!text.is_empty());
        assert!(text.starts_with("Hello "));
    }
}

// ============================================================================
// Complexity Analysis Scenarios
// ============================================================================

mod complexity_scenarios {
    use super::*;

    #[test]
    fn scenario_empty_code_yields_zero_functions() {
        // Given empty source code
        let metrics = analyze_functions("", "rust");
        // Then function count is zero
        assert_eq!(metrics.function_count, 0);
        assert_eq!(metrics.max_function_length, 0);
        assert_eq!(metrics.avg_function_length, 0.0);
    }

    #[test]
    fn scenario_single_rust_function_detected() {
        // Given a single Rust function
        let code = "fn greet() {\n    println!(\"hi\");\n}\n";
        // When we analyze it
        let metrics = analyze_functions(code, "rust");
        // Then we detect 1 function of length 3
        assert_eq!(metrics.function_count, 1);
        assert_eq!(metrics.max_function_length, 3);
    }

    #[test]
    fn scenario_python_def_detected() {
        // Given a Python function
        let code = "def greet():\n    print('hi')\n    print('bye')\n";
        // When we analyze it
        let metrics = analyze_functions(code, "python");
        // Then we detect 1 function
        assert_eq!(metrics.function_count, 1);
    }

    #[test]
    fn scenario_javascript_function_detected() {
        // Given a JavaScript function
        let code = "function greet() {\n    console.log('hi');\n}\n";
        // When we analyze it
        let metrics = analyze_functions(code, "javascript");
        // Then we detect 1 function
        assert_eq!(metrics.function_count, 1);
    }

    #[test]
    fn scenario_go_func_detected() {
        // Given a Go function
        let code = "func main() {\n    fmt.Println(\"hello\")\n}\n";
        // When we analyze it
        let metrics = analyze_functions(code, "go");
        // Then we detect 1 function
        assert_eq!(metrics.function_count, 1);
    }

    #[test]
    fn scenario_unsupported_language_yields_zero() {
        // Given code in an unsupported language
        let code = "some code here";
        // When we analyze with "brainfuck"
        let metrics = analyze_functions(code, "brainfuck");
        // Then we detect nothing
        assert_eq!(metrics.function_count, 0);
    }

    #[test]
    fn scenario_cyclomatic_complexity_simple_function() {
        // Given a simple Rust function with no branches
        let code = "fn simple() {\n    println!(\"hello\");\n}\n";
        // When we estimate CC
        let result = estimate_cyclomatic_complexity(code, "rust");
        // Then CC = 1 (base only)
        assert_eq!(result.function_count, 1);
        assert_eq!(result.max_cc, 1);
    }

    #[test]
    fn scenario_cyclomatic_complexity_with_branch() {
        // Given a Rust function with one if-else
        let code = r#"fn check(x: i32) {
    if x > 0 {
        println!("positive");
    } else {
        println!("non-positive");
    }
}
"#;
        // When we estimate CC
        let result = estimate_cyclomatic_complexity(code, "rust");
        // Then CC > 1 (has at least one decision point)
        assert_eq!(result.function_count, 1);
        assert!(result.max_cc >= 2, "expected CC >= 2, got {}", result.max_cc);
    }

    #[test]
    fn scenario_cyclomatic_complexity_empty_code() {
        // Given empty code
        let result = estimate_cyclomatic_complexity("", "rust");
        // Then everything is zero/default
        assert_eq!(result.function_count, 0);
        assert_eq!(result.total_cc, 0);
        assert_eq!(result.max_cc, 0);
    }

    #[test]
    fn scenario_cognitive_complexity_nested_ifs() {
        // Given nested control structures
        let code = r#"fn nested(x: i32) {
    if x > 0 {
        if x > 10 {
            if x > 100 {
                println!("big");
            }
        }
    }
}
"#;
        // When we estimate cognitive complexity
        let result = estimate_cognitive_complexity(code, "rust");
        // Then complexity is high due to nesting penalty
        assert_eq!(result.function_count, 1);
        assert!(
            result.max >= 3,
            "expected high cognitive complexity for nested ifs, got {}",
            result.max
        );
    }

    #[test]
    fn scenario_nesting_depth_increases_with_braces() {
        // Given deeply nested Rust code
        let code = r#"fn deep() {
    if true {
        for i in 0..10 {
            if i > 5 {
                println!("{}", i);
            }
        }
    }
}
"#;
        // When we analyze nesting depth
        let result = analyze_nesting_depth(code, "rust");
        // Then max depth >= 4 (fn, if, for, if)
        assert!(
            result.max_depth >= 4,
            "expected depth >= 4, got {}",
            result.max_depth
        );
    }

    #[test]
    fn scenario_nesting_analysis_empty_code() {
        // Given empty code
        let result = analyze_nesting_depth("", "rust");
        // Then defaults
        assert_eq!(result.max_depth, 0);
        assert_eq!(result.avg_depth, 0.0);
    }

    #[test]
    fn scenario_python_nesting_by_indentation() {
        // Given nested Python code
        let code = "def f():\n    if True:\n        for x in range(10):\n            print(x)\n";
        // When we analyze nesting depth for python
        let result = analyze_nesting_depth(code, "python");
        // Then max depth >= 2 (if + for inside def)
        assert!(
            result.max_depth >= 2,
            "expected depth >= 2, got {}",
            result.max_depth
        );
    }

    #[test]
    fn scenario_multiple_functions_avg_length() {
        // Given two functions of different lengths
        let code = r#"fn short() {
    println!("hi");
}

fn longer() {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    println!("{}", a + b + c + d);
}
"#;
        // When we analyze functions
        let metrics = analyze_functions(code, "rust");
        // Then average is between the two lengths
        assert_eq!(metrics.function_count, 2);
        assert!(metrics.avg_function_length > 2.0);
        assert!(metrics.avg_function_length < metrics.max_function_length as f64);
    }

    #[test]
    fn scenario_high_complexity_function_flagged() {
        // Given a function with many branches (CC > 10)
        let mut code = String::from("fn branchy(x: i32) {\n");
        for i in 0..12 {
            code.push_str(&format!("    if x == {i} {{ println!(\"{i}\"); }}\n"));
        }
        code.push_str("}\n");

        // When we estimate CC
        let result = estimate_cyclomatic_complexity(&code, "rust");
        // Then it's flagged as high complexity
        assert_eq!(result.function_count, 1);
        assert!(
            result.max_cc > 10,
            "expected CC > 10, got {}",
            result.max_cc
        );
        assert!(
            !result.high_complexity_functions.is_empty(),
            "expected high complexity flag"
        );
    }
}
