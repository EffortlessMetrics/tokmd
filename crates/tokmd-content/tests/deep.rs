//! Deep tests for tokmd-content scanning functions.
//!
//! Covers entropy computation, high/low entropy detection, import-like tag scanning,
//! TODO/FIXME scanning, BLAKE3 hashing, empty/binary/unicode file handling,
//! deterministic output, and serialization-style roundtrips.

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
// 1. Entropy computation on known inputs
// ============================================================================

#[test]
fn entropy_empty_is_zero() {
    assert_eq!(entropy_bits_per_byte(&[]), 0.0);
}

#[test]
fn entropy_single_byte_is_zero() {
    assert!(entropy_bits_per_byte(&[0x42]).abs() < 1e-6);
}

#[test]
fn entropy_uniform_pair_is_one_bit() {
    // Equal distribution of two values → log2(2) = 1.0
    let buf: Vec<u8> = (0..2000).map(|i| (i % 2) as u8).collect();
    let e = entropy_bits_per_byte(&buf);
    assert!((e - 1.0).abs() < 0.01, "expected ~1.0, got {e}");
}

#[test]
fn entropy_uniform_four_is_two_bits() {
    let buf: Vec<u8> = (0..2000).map(|i| (i % 4) as u8).collect();
    let e = entropy_bits_per_byte(&buf);
    assert!((e - 2.0).abs() < 0.01, "expected ~2.0, got {e}");
}

#[test]
fn entropy_full_byte_range_is_eight_bits() {
    let buf: Vec<u8> = (0u8..=255).cycle().take(2048).collect();
    let e = entropy_bits_per_byte(&buf);
    assert!((e - 8.0).abs() < 0.01, "expected ~8.0, got {e}");
}

#[test]
fn entropy_sixteen_values_is_four_bits() {
    let buf: Vec<u8> = (0..1600).map(|i| (i % 16) as u8).collect();
    let e = entropy_bits_per_byte(&buf);
    assert!((e - 4.0).abs() < 0.05, "expected ~4.0, got {e}");
}

#[test]
fn entropy_is_always_non_negative_and_bounded() {
    for data in [
        vec![0u8; 1],
        vec![255u8; 500],
        b"hello world".to_vec(),
        (0u8..=255).collect::<Vec<_>>(),
    ] {
        let e = entropy_bits_per_byte(&data);
        assert!(e >= 0.0, "negative: {e}");
        assert!(e <= 8.0 + 1e-6, "exceeds 8 bits: {e}");
    }
}

// ============================================================================
// 2. High vs low entropy detection
// ============================================================================

#[test]
fn high_entropy_random_like_data() {
    // Simulated high-entropy (all 256 values equally represented)
    let buf: Vec<u8> = (0u8..=255).cycle().take(4096).collect();
    let e = entropy_bits_per_byte(&buf);
    assert!(e > 7.9, "expected high entropy, got {e}");
}

#[test]
fn low_entropy_repeated_ascii() {
    let buf = b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let e = entropy_bits_per_byte(buf);
    assert!(e < 0.01, "expected near-zero entropy, got {e}");
}

#[test]
fn moderate_entropy_english_text() {
    let text = b"The quick brown fox jumps over the lazy dog repeatedly many times.";
    let e = entropy_bits_per_byte(text);
    assert!(e > 3.0 && e < 5.5, "expected moderate entropy, got {e}");
}

#[test]
fn high_entropy_vs_low_entropy_ordering() {
    let low = entropy_bits_per_byte(b"aaaaaaaaaaaa");
    let high = entropy_bits_per_byte(&(0u8..=255).collect::<Vec<_>>());
    assert!(
        high > low,
        "high-entropy data ({high}) should exceed low-entropy ({low})"
    );
}

// ============================================================================
// 3. Import-like pattern scanning via count_tags
// ============================================================================

#[test]
fn rust_use_statements_detected() {
    let code = "use std::io;\nuse std::fs;\nlet x = use_something();";
    let result = count_tags(code, &["use"]);
    // "use" appears in all three lines (case-insensitive match)
    assert!(
        result[0].1 >= 3,
        "expected >=3 'use' matches, got {}",
        result[0].1
    );
}

#[test]
fn python_import_statements_detected() {
    let code = "import os\nfrom pathlib import Path\nimport sys";
    let result = count_tags(code, &["import"]);
    assert_eq!(result[0].1, 3, "expected 3 'import' matches");
}

#[test]
fn js_require_detected() {
    let code = "const fs = require('fs');\nconst path = require('path');";
    let result = count_tags(code, &["require"]);
    assert_eq!(result[0].1, 2, "expected 2 'require' matches");
}

#[test]
fn multiple_import_styles_counted_together() {
    let code = "\
use std::io;
import os
const x = require('x');
from foo import bar;
use crate::something;
";
    let result = count_tags(code, &["use", "import", "require"]);
    assert_eq!(result[0].1, 2, "use count"); // "use" appears twice
    assert_eq!(result[1].1, 2, "import count"); // "import" appears twice
    assert_eq!(result[2].1, 1, "require count");
}

// ============================================================================
// 4. TODO/FIXME tag scanning
// ============================================================================

#[test]
fn todo_fixme_hack_scanned() {
    let code = "// TO\x44O: fix\n// FI\x58ME: broken\n// HA\x43K: workaround\nlet x = 1;";
    let result = count_tags(code, &["TODO", "FIXME", "HACK"]);
    assert_eq!(result[0].1, 1);
    assert_eq!(result[1].1, 1);
    assert_eq!(result[2].1, 1);
}

#[test]
fn tags_case_insensitive() {
    let text = "to\x64o To\x64o TO\x44O tO\x64O";
    let result = count_tags(text, &["TODO"]);
    assert_eq!(result[0].1, 4);
}

#[test]
fn tags_in_multiline_code() {
    let code = "\
fn main() {
    // TO\x44O: first item
    let x = 42;
    // FI\x58ME: second item
    // TO\x44O: third item
    println!(\"{}\", x);
}
";
    let result = count_tags(code, &["TODO", "FIXME"]);
    assert_eq!(result[0].1, 2, "TODO");
    assert_eq!(result[1].1, 1, "FIXME");
}

#[test]
fn adjacent_tags_counted() {
    let text = "TO\x44OTO\x44OTO\x44O";
    let result = count_tags(text, &["TODO"]);
    assert_eq!(result[0].1, 3);
}

#[test]
fn no_tags_in_clean_code() {
    let code = "fn main() { println!(\"hello\"); }";
    let result = count_tags(code, &["TODO", "FIXME", "HACK"]);
    for (_tag, count) in &result {
        assert_eq!(*count, 0);
    }
}

#[test]
fn empty_tag_list_returns_empty() {
    let result = count_tags("some text", &[]);
    assert!(result.is_empty());
}

#[test]
fn tag_order_preserved_in_results() {
    let tags = &["HACK", "TODO", "FIXME", "NOTE"];
    let result = count_tags("TODO", tags);
    assert_eq!(result[0].0, "HACK");
    assert_eq!(result[1].0, "TODO");
    assert_eq!(result[2].0, "FIXME");
    assert_eq!(result[3].0, "NOTE");
}

// ============================================================================
// 5. File hashing (BLAKE3)
// ============================================================================

#[test]
fn hash_bytes_is_64_hex_lowercase() {
    let h = hash_bytes(b"test");
    assert_eq!(h.len(), 64);
    assert!(
        h.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    );
}

#[test]
fn hash_bytes_empty_input() {
    let h = hash_bytes(&[]);
    assert_eq!(h.len(), 64);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn hash_file_matches_hash_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("data.bin");
    let content = b"hello world for hashing";
    File::create(&path).unwrap().write_all(content).unwrap();

    assert_eq!(hash_file(&path, 10000).unwrap(), hash_bytes(content));
}

#[test]
fn hash_file_respects_max_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("long.bin");
    File::create(&path)
        .unwrap()
        .write_all(b"abcdefghijklmnop")
        .unwrap();

    let h5 = hash_file(&path, 5).unwrap();
    let hfull = hash_file(&path, 10000).unwrap();
    assert_eq!(h5, hash_bytes(b"abcde"));
    assert_ne!(h5, hfull);
}

#[test]
fn hash_single_bit_flip_differs() {
    let a = b"AAAA";
    let mut b_buf = *a;
    b_buf[0] ^= 0x01;
    assert_ne!(hash_bytes(a), hash_bytes(&b_buf));
}

// ============================================================================
// 6. Empty file handling
// ============================================================================

#[test]
fn read_head_empty_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("empty");
    File::create(&path).unwrap();
    assert!(read_head(&path, 100).unwrap().is_empty());
}

#[test]
fn read_head_tail_empty_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("empty");
    File::create(&path).unwrap();
    assert!(read_head_tail(&path, 100).unwrap().is_empty());
}

#[test]
fn read_lines_empty_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("empty");
    File::create(&path).unwrap();
    assert!(read_lines(&path, 100, 10_000).unwrap().is_empty());
}

#[test]
fn read_text_capped_empty_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("empty");
    File::create(&path).unwrap();
    assert!(read_text_capped(&path, 100).unwrap().is_empty());
}

#[test]
fn hash_file_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("empty");
    File::create(&path).unwrap();
    assert_eq!(hash_file(&path, 100).unwrap(), hash_bytes(&[]));
}

#[test]
fn entropy_empty_bytes() {
    assert_eq!(entropy_bits_per_byte(&[]), 0.0);
}

#[test]
fn count_tags_empty_text() {
    let result = count_tags("", &["TODO", "FIXME"]);
    assert_eq!(result[0].1, 0);
    assert_eq!(result[1].1, 0);
}

// ============================================================================
// 7. Binary file detection
// ============================================================================

#[test]
fn null_byte_makes_non_text() {
    assert!(!is_text_like(b"hello\x00world"));
}

#[test]
fn png_header_not_text() {
    let header: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    assert!(!is_text_like(header));
}

#[test]
fn elf_header_not_text() {
    let header: &[u8] = &[0x7F, 0x45, 0x4C, 0x46, 0x00, 0x00];
    assert!(!is_text_like(header));
}

#[test]
fn pure_binary_bytes_not_text() {
    let data: Vec<u8> = vec![0x00, 0xFF, 0x00, 0xFE, 0x00];
    assert!(!is_text_like(&data));
}

#[test]
fn invalid_utf8_without_null_not_text() {
    let data: &[u8] = &[0xFF, 0xFE, 0xFD];
    assert!(!is_text_like(data));
}

#[test]
fn valid_ascii_is_text() {
    assert!(is_text_like(b"Hello, World! 123\n\ttabs"));
}

#[test]
fn empty_bytes_is_text() {
    assert!(is_text_like(&[]));
}

// ============================================================================
// 8. Large file handling
// ============================================================================

#[test]
fn read_head_large_file_truncated() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("large.bin");
    let data = vec![b'X'; 100_000];
    File::create(&path).unwrap().write_all(&data).unwrap();

    let head = read_head(&path, 1024).unwrap();
    assert_eq!(head.len(), 1024);
    assert!(head.iter().all(|&b| b == b'X'));
}

#[test]
fn read_head_tail_large_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("large.bin");
    let mut data = vec![b'A'; 50_000];
    data.extend(vec![b'Z'; 50_000]);
    File::create(&path).unwrap().write_all(&data).unwrap();

    let result = read_head_tail(&path, 100).unwrap();
    assert_eq!(result.len(), 100);
    // First 50 bytes from head (all 'A'), last 50 from tail (all 'Z')
    assert!(result[..50].iter().all(|&b| b == b'A'));
    assert!(result[50..].iter().all(|&b| b == b'Z'));
}

#[test]
fn hash_file_large_with_cap() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("large.bin");
    let data = vec![b'M'; 200_000];
    File::create(&path).unwrap().write_all(&data).unwrap();

    let h1 = hash_file(&path, 1024).unwrap();
    let h2 = hash_file(&path, 2048).unwrap();
    // Different caps produce different hashes (data is homogeneous, but length differs)
    assert_ne!(h1, h2);
}

#[test]
fn entropy_large_uniform() {
    let data = vec![0x42u8; 1_000_000];
    let e = entropy_bits_per_byte(&data);
    assert!(
        e.abs() < 1e-6,
        "uniform large data should have zero entropy"
    );
}

// ============================================================================
// 9. Unicode content
// ============================================================================

#[test]
fn is_text_like_valid_utf8_multibyte() {
    let text = "こんにちは世界 🌍 café résumé naïve";
    assert!(is_text_like(text.as_bytes()));
}

#[test]
fn read_text_capped_unicode_lossy() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("unicode.txt");
    // "Hello 🌍" - the emoji is 4 bytes, truncating in the middle should use lossy conversion
    File::create(&path)
        .unwrap()
        .write_all("Hello 🌍 World".as_bytes())
        .unwrap();

    let text = read_text_capped(&path, 7).unwrap();
    assert!(!text.is_empty());
    assert!(text.starts_with("Hello "));
}

#[test]
fn entropy_unicode_text() {
    let text = "日本語テキストの例文です。これはエントロピーのテストです。";
    let e = entropy_bits_per_byte(text.as_bytes());
    // UTF-8 encoded CJK text has relatively high byte-level entropy
    assert!(e > 3.0, "CJK text should have moderate+ entropy, got {e}");
}

#[test]
fn hash_bytes_unicode_deterministic() {
    let text = "🦀 Rust is awesome! 日本語";
    let h1 = hash_bytes(text.as_bytes());
    let h2 = hash_bytes(text.as_bytes());
    assert_eq!(h1, h2);
}

#[test]
fn count_tags_in_unicode_text() {
    let code = "// TO\x44O: 日本語コメント\n// FI\x58ME: ñoño\nlet x = 1;";
    let result = count_tags(code, &["TODO", "FIXME"]);
    assert_eq!(result[0].1, 1);
    assert_eq!(result[1].1, 1);
}

// ============================================================================
// 10. Multiple import styles (via count_tags heuristic)
// ============================================================================

#[test]
fn es6_import_detected() {
    let code = "import React from 'react';\nimport { useState } from 'react';";
    let result = count_tags(code, &["import"]);
    assert_eq!(result[0].1, 2);
}

#[test]
fn go_import_detected() {
    let code = "import (\n\t\"fmt\"\n\t\"os\"\n)";
    let result = count_tags(code, &["import"]);
    assert_eq!(result[0].1, 1);
}

#[test]
fn mixed_lang_imports() {
    let code = "\
use serde::Serialize;
import numpy as np
const fs = require('fs');
from typing import List
use tokmd_types::Receipt;
import React from 'react';
require 'json'
";
    let result = count_tags(code, &["use", "import", "require"]);
    assert_eq!(result[0].1, 2, "'use' count");
    assert_eq!(result[1].1, 3, "'import' count");
    assert_eq!(result[2].1, 2, "'require' count");
}

// ============================================================================
// 11. Deterministic output
// ============================================================================

#[test]
fn entropy_deterministic() {
    let data = b"the quick brown fox jumps over the lazy dog";
    let e1 = entropy_bits_per_byte(data);
    let e2 = entropy_bits_per_byte(data);
    assert_eq!(e1, e2);
}

#[test]
fn hash_bytes_deterministic() {
    let data = b"determinism check 12345";
    let h1 = hash_bytes(data);
    let h2 = hash_bytes(data);
    let h3 = hash_bytes(data);
    assert_eq!(h1, h2);
    assert_eq!(h2, h3);
}

#[test]
fn count_tags_deterministic() {
    let text = "TO\x44O FI\x58ME HA\x43K TO\x44O";
    let r1 = count_tags(text, &["TODO", "FIXME", "HACK"]);
    let r2 = count_tags(text, &["TODO", "FIXME", "HACK"]);
    assert_eq!(r1, r2);
}

#[test]
fn hash_file_deterministic() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("det.bin");
    File::create(&path)
        .unwrap()
        .write_all(b"stable content")
        .unwrap();

    let h1 = hash_file(&path, 10000).unwrap();
    let h2 = hash_file(&path, 10000).unwrap();
    assert_eq!(h1, h2);
}

#[test]
fn is_text_like_deterministic() {
    let data = b"hello world";
    assert_eq!(is_text_like(data), is_text_like(data));
}

#[test]
fn complexity_deterministic() {
    let code = "fn foo() {\n    if true { println!(\"yes\"); }\n}\n";
    let m1 = analyze_functions(code, "rust");
    let m2 = analyze_functions(code, "rust");
    assert_eq!(m1, m2);

    let cc1 = estimate_cyclomatic_complexity(code, "rust");
    let cc2 = estimate_cyclomatic_complexity(code, "rust");
    assert_eq!(cc1.total_cc, cc2.total_cc);
    assert_eq!(cc1.max_cc, cc2.max_cc);
}

// ============================================================================
// 12. Serialization roundtrip (hash as content fingerprint)
// ============================================================================

#[test]
fn hash_roundtrip_file_to_bytes_to_file() {
    let tmp = tempfile::tempdir().unwrap();
    let content = b"roundtrip test content with special chars: \x01\x02\x03";

    // Write content, hash the file
    let path = tmp.path().join("original.bin");
    File::create(&path).unwrap().write_all(content).unwrap();
    let original_hash = hash_file(&path, 10000).unwrap();

    // Read back and hash bytes directly
    let bytes = read_head(&path, 10000).unwrap();
    let bytes_hash = hash_bytes(&bytes);
    assert_eq!(original_hash, bytes_hash);

    // Write to another file and hash again
    let path2 = tmp.path().join("copy.bin");
    File::create(&path2).unwrap().write_all(&bytes).unwrap();
    let copy_hash = hash_file(&path2, 10000).unwrap();
    assert_eq!(original_hash, copy_hash);
}

#[test]
fn entropy_roundtrip_consistency() {
    // Entropy of data read from file matches entropy of original bytes
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("entropy.bin");
    let content: Vec<u8> = (0..512).map(|i| (i % 64) as u8).collect();
    File::create(&path).unwrap().write_all(&content).unwrap();

    let file_bytes = read_head(&path, 10000).unwrap();
    let e_original = entropy_bits_per_byte(&content);
    let e_from_file = entropy_bits_per_byte(&file_bytes);
    assert_eq!(e_original, e_from_file);
}

#[test]
fn tag_count_roundtrip() {
    // count_tags on content read from file matches direct count
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("tags.txt");
    let code = "// TO\x44O: first\n// FI\x58ME: second\n// TO\x44O: third\n";
    File::create(&path)
        .unwrap()
        .write_all(code.as_bytes())
        .unwrap();

    let text = read_text_capped(&path, 10000).unwrap();
    let from_file = count_tags(&text, &["TODO", "FIXME"]);
    let direct = count_tags(code, &["TODO", "FIXME"]);
    assert_eq!(from_file, direct);
}

// ============================================================================
// Additional edge cases for completeness
// ============================================================================

#[test]
fn read_head_tail_max_bytes_zero() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("data.txt");
    File::create(&path).unwrap().write_all(b"content").unwrap();
    let bytes = read_head_tail(&path, 0).unwrap();
    assert!(bytes.is_empty());
}

#[test]
fn read_lines_zero_max_lines_reads_none() {
    // max_lines=0 triggers early return with empty Vec
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("lines.txt");
    let mut f = File::create(&path).unwrap();
    writeln!(f, "line 1").unwrap();
    writeln!(f, "line 2").unwrap();

    let lines = read_lines(&path, 0, 10_000).unwrap();
    assert_eq!(lines.len(), 0);
}

#[test]
fn nonexistent_file_errors() {
    let bad = std::path::Path::new("/tmp/tokmd_nonexistent_12345.bin");
    assert!(read_head(bad, 100).is_err());
    assert!(hash_file(bad, 100).is_err());
    assert!(read_text_capped(bad, 100).is_err());
    assert!(read_lines(bad, 10, 1000).is_err());
    assert!(read_head_tail(bad, 100).is_err());
}

#[test]
fn complexity_empty_code_all_defaults() {
    for lang in ["rust", "python", "javascript", "go", "unknown"] {
        let fm = analyze_functions("", lang);
        assert_eq!(fm.function_count, 0);
        assert_eq!(fm.max_function_length, 0);
        assert_eq!(fm.avg_function_length, 0.0);

        let cc = estimate_cyclomatic_complexity("", lang);
        assert_eq!(cc.function_count, 0);
        assert_eq!(cc.total_cc, 0);
        assert_eq!(cc.max_cc, 0);

        let cog = estimate_cognitive_complexity("", lang);
        assert_eq!(cog.function_count, 0);
        assert_eq!(cog.total, 0);

        let nest = analyze_nesting_depth("", lang);
        assert_eq!(nest.max_depth, 0);
        assert_eq!(nest.avg_depth, 0.0);
    }
}

#[test]
fn complexity_multiple_languages_detect_functions() {
    let rust_code = "fn a() {\n    let x = 1;\n}\n";
    let py_code = "def a():\n    x = 1\n";
    let js_code = "function a() {\n    let x = 1;\n}\n";
    let go_code = "func a() {\n    x := 1\n}\n";

    assert_eq!(analyze_functions(rust_code, "rust").function_count, 1);
    assert_eq!(analyze_functions(py_code, "python").function_count, 1);
    assert_eq!(analyze_functions(js_code, "javascript").function_count, 1);
    assert_eq!(analyze_functions(go_code, "go").function_count, 1);
}

#[test]
fn nesting_depth_tracks_max_depth_lines() {
    let code = "fn main() {\n    if true {\n        if false {\n            println!(\"deep\");\n        }\n    }\n}\n";
    let result = analyze_nesting_depth(code, "rust");
    assert!(result.max_depth >= 3);
    assert!(!result.max_depth_lines.is_empty());
}
