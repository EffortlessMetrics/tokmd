//! Edge-case tests for Shannon entropy computation.
//!
//! Exercises boundary conditions, large inputs, and file-based round-trips
//! that complement the BDD and property tests.

use proptest::prelude::*;
use std::fs::File;
use std::io::Write;
use tokmd_content::{entropy_bits_per_byte, read_head};

// ============================================================================
// File-based entropy edge cases
// ============================================================================

#[test]
fn entropy_of_empty_file_is_zero() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("empty.bin");
    File::create(&path).unwrap();

    let bytes = read_head(&path, 1_000_000).unwrap();
    let entropy = entropy_bits_per_byte(&bytes);
    assert_eq!(entropy, 0.0);
}

#[test]
fn entropy_of_single_repeated_byte_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("repeated.bin");
    let mut f = File::create(&path).unwrap();
    f.write_all(&vec![0x42; 10_000]).unwrap();

    let bytes = read_head(&path, 1_000_000).unwrap();
    let entropy = entropy_bits_per_byte(&bytes);
    assert!(entropy.abs() < 1e-6, "expected ~0, got {entropy}");
}

#[test]
fn entropy_of_pseudorandom_file_is_high() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("random.bin");
    let mut f = File::create(&path).unwrap();
    // Use a simple LCG to generate all 256 byte values many times
    let data: Vec<u8> = (0u16..8192).map(|i| (i % 256) as u8).collect();
    f.write_all(&data).unwrap();

    let bytes = read_head(&path, 1_000_000).unwrap();
    let entropy = entropy_bits_per_byte(&bytes);
    assert!(
        entropy > 7.9,
        "expected high entropy (~8 bits) for uniform distribution, got {entropy}"
    );
}

#[test]
fn entropy_of_large_file_does_not_panic() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("large.bin");
    let mut f = File::create(&path).unwrap();
    // 1 MB of repeating pattern
    let pattern: Vec<u8> = (0u8..=255).collect();
    for _ in 0..4096 {
        f.write_all(&pattern).unwrap();
    }

    let bytes = read_head(&path, 1_048_576).unwrap();
    let entropy = entropy_bits_per_byte(&bytes);
    assert!(
        entropy.is_finite(),
        "entropy must be finite for large input"
    );
    assert!(
        entropy > 7.9,
        "expected ~8 bits for uniform data, got {entropy}"
    );
}

#[test]
fn entropy_file_head_truncation_affects_result() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("mixed.bin");
    let mut f = File::create(&path).unwrap();
    // Head: low-entropy (all zeros)
    f.write_all(&vec![0u8; 1000]).unwrap();
    // Tail: high-entropy (all 256 values)
    let tail: Vec<u8> = (0..4000).map(|i| (i % 256) as u8).collect();
    f.write_all(&tail).unwrap();

    // Reading only the first 1000 bytes sees only zeros
    let head_bytes = read_head(&path, 1000).unwrap();
    let head_entropy = entropy_bits_per_byte(&head_bytes);
    assert!(
        head_entropy < 0.01,
        "head-only should be low entropy, got {head_entropy}"
    );

    // Reading the whole file should have higher entropy
    let all_bytes = read_head(&path, 1_000_000).unwrap();
    let full_entropy = entropy_bits_per_byte(&all_bytes);
    assert!(
        full_entropy > head_entropy,
        "full file entropy ({full_entropy}) should exceed head-only ({head_entropy})"
    );
}

#[test]
fn entropy_two_byte_alphabet_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("two_vals.bin");
    let mut f = File::create(&path).unwrap();
    let data: Vec<u8> = (0..10_000)
        .map(|i| if i % 2 == 0 { 0 } else { 255 })
        .collect();
    f.write_all(&data).unwrap();

    let bytes = read_head(&path, 1_000_000).unwrap();
    let entropy = entropy_bits_per_byte(&bytes);
    assert!(
        (entropy - 1.0).abs() < 0.01,
        "expected ~1 bit for two equally likely values, got {entropy}"
    );
}

#[test]
fn entropy_single_byte_file_is_zero() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("one_byte.bin");
    let mut f = File::create(&path).unwrap();
    f.write_all(&[0xDE]).unwrap();

    let bytes = read_head(&path, 1_000_000).unwrap();
    let entropy = entropy_bits_per_byte(&bytes);
    assert!(
        entropy.abs() < 1e-6,
        "single byte should have 0 entropy, got {entropy}"
    );
}

#[test]
fn entropy_source_code_file_moderate() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("code.rs");
    let mut f = File::create(&path).unwrap();
    writeln!(f, "fn main() {{").unwrap();
    writeln!(f, "    let x = 42;").unwrap();
    writeln!(f, "    println!(\"hello world\");").unwrap();
    writeln!(f, "    for i in 0..10 {{").unwrap();
    writeln!(f, "        if i > 5 {{").unwrap();
    writeln!(f, "            println!(\"{{i}}\");").unwrap();
    writeln!(f, "        }}").unwrap();
    writeln!(f, "    }}").unwrap();
    writeln!(f, "}}").unwrap();

    let bytes = read_head(&path, 1_000_000).unwrap();
    let entropy = entropy_bits_per_byte(&bytes);
    // Source code typically has moderate entropy (3-6 bits)
    assert!(
        entropy > 3.0 && entropy < 6.5,
        "source code expected 3-6.5 bits, got {entropy}"
    );
}

// ============================================================================
// Property: entropy is always between 0 and 8 (file-based variant)
// ============================================================================

proptest! {
    #[test]
    fn prop_entropy_bounded_via_file(data in prop::collection::vec(any::<u8>(), 0..4096)) {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("prop.bin");
        File::create(&path).unwrap().write_all(&data).unwrap();

        let bytes = read_head(&path, 1_000_000).unwrap();
        let entropy = entropy_bits_per_byte(&bytes);
        prop_assert!(entropy >= 0.0, "entropy must be >= 0: got {}", entropy);
        prop_assert!(entropy <= 8.0, "entropy must be <= 8: got {}", entropy);
        prop_assert!(entropy.is_finite(), "entropy must be finite");
    }

    #[test]
    fn prop_file_entropy_matches_direct(data in prop::collection::vec(any::<u8>(), 1..2048)) {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("match.bin");
        File::create(&path).unwrap().write_all(&data).unwrap();

        let file_bytes = read_head(&path, data.len()).unwrap();
        let file_entropy = entropy_bits_per_byte(&file_bytes);
        let direct_entropy = entropy_bits_per_byte(&data);
        prop_assert!(
            (file_entropy - direct_entropy).abs() < 1e-6,
            "file and direct entropy should match: {} vs {}",
            file_entropy,
            direct_entropy
        );
    }
}
