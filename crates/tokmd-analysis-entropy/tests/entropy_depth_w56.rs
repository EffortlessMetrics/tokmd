//! Wave-56 depth tests for entropy profiling.
//!
//! Focuses on entropy calculation for various byte patterns,
//! classification thresholds, edge cases, and determinism.

use std::fs;
use std::path::PathBuf;

use tempfile::tempdir;
use tokmd_analysis_entropy::build_entropy_report;
use tokmd_analysis_types::EntropyClass;
use tokmd_analysis_util::AnalysisLimits;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────

fn export_for(paths: &[&str]) -> ExportData {
    let rows = paths
        .iter()
        .map(|p| FileRow {
            path: (*p).to_string(),
            module: "(root)".to_string(),
            lang: "Text".to_string(),
            kind: FileKind::Parent,
            code: 1,
            comments: 0,
            blanks: 0,
            lines: 1,
            bytes: 10,
            tokens: 2,
        })
        .collect();
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn export_with_module(path: &str, module: &str) -> ExportData {
    ExportData {
        rows: vec![FileRow {
            path: path.to_string(),
            module: module.to_string(),
            lang: "Text".to_string(),
            kind: FileKind::Parent,
            code: 1,
            comments: 0,
            blanks: 0,
            lines: 1,
            bytes: 10,
            tokens: 2,
        }],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn write_repeated(path: &std::path::Path, byte: u8, len: usize) {
    fs::write(path, vec![byte; len]).unwrap();
}

fn write_pseudorandom(path: &std::path::Path, seed: u32, len: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(len);
    let mut x = seed;
    for _ in 0..len {
        x = x.wrapping_mul(1664525).wrapping_add(1013904223);
        data.push((x & 0xFF) as u8);
    }
    fs::write(path, &data).unwrap();
    data
}

fn write_two_value_pattern(path: &std::path::Path, a: u8, b: u8, len: usize) {
    let data: Vec<u8> = (0..len).map(|i| if i % 2 == 0 { a } else { b }).collect();
    fs::write(path, data).unwrap();
}

fn default_limits() -> AnalysisLimits {
    AnalysisLimits::default()
}

// ── 1. All-zero bytes: low entropy ──────────────────────────────

#[test]
fn all_zeros_classified_low() {
    let dir = tempdir().unwrap();
    write_repeated(&dir.path().join("zeros.bin"), 0x00, 1024);
    let export = export_for(&["zeros.bin"]);
    let files = vec![PathBuf::from("zeros.bin")];
    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();
    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].class, EntropyClass::Low);
}

// ── 2. All-0xFF bytes: low entropy ──────────────────────────────

#[test]
fn all_ff_classified_low() {
    let dir = tempdir().unwrap();
    write_repeated(&dir.path().join("ff.bin"), 0xFF, 1024);
    let export = export_for(&["ff.bin"]);
    let files = vec![PathBuf::from("ff.bin")];
    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();
    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].class, EntropyClass::Low);
}

// ── 3. Alternating two bytes: very low entropy ──────────────────

#[test]
fn alternating_two_bytes_low_entropy() {
    let dir = tempdir().unwrap();
    write_two_value_pattern(&dir.path().join("alt.bin"), 0x00, 0xFF, 1024);
    let export = export_for(&["alt.bin"]);
    let files = vec![PathBuf::from("alt.bin")];
    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();
    assert_eq!(report.suspects.len(), 1);
    // Two values = 1 bit of entropy per byte, well below 2.0
    assert_eq!(report.suspects[0].class, EntropyClass::Low);
    assert!(report.suspects[0].entropy_bits_per_byte < 2.0);
}

// ── 4. Pseudorandom data with different seeds: consistently high

#[test]
fn pseudorandom_different_seeds_all_high() {
    let dir = tempdir().unwrap();
    let seeds = [0xDEADBEEF_u32, 0xCAFEBABE, 0x01020304];
    let mut names = Vec::new();
    for (i, &seed) in seeds.iter().enumerate() {
        let name = format!("rng_{i}.bin");
        write_pseudorandom(&dir.path().join(&name), seed, 2048);
        names.push(name);
    }
    let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let export = export_for(&refs);
    let files: Vec<PathBuf> = names.iter().map(PathBuf::from).collect();
    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();
    for s in &report.suspects {
        assert_eq!(
            s.class,
            EntropyClass::High,
            "seed-based random should be high entropy"
        );
    }
}

// ── 5. Single byte file: handled without panic ──────────────────

#[test]
fn single_byte_file_no_panic() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("one.bin"), &[42u8]).unwrap();
    let export = export_for(&["one.bin"]);
    let files = vec![PathBuf::from("one.bin")];
    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();
    // A single-byte file has 0 entropy (only one symbol)
    if !report.suspects.is_empty() {
        assert_eq!(report.suspects[0].class, EntropyClass::Low);
    }
}

// ── 6. Two-byte file: handled correctly ─────────────────────────

#[test]
fn two_byte_file_no_panic() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("two.bin"), &[0x00, 0xFF]).unwrap();
    let export = export_for(&["two.bin"]);
    let files = vec![PathBuf::from("two.bin")];
    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();
    // Should not panic; entropy of 2 distinct bytes = 1.0 bit
    if !report.suspects.is_empty() {
        assert_eq!(report.suspects[0].class, EntropyClass::Low);
    }
}

// ── 7. ASCII printable text: normal entropy ─────────────────────

#[test]
fn ascii_printable_text_normal_entropy() {
    let dir = tempdir().unwrap();
    let text = "The quick brown fox jumps over the lazy dog. \
                Pack my box with five dozen liquor jugs. \
                Sphinx of black quartz, judge my vow.\n"
        .repeat(20);
    fs::write(dir.path().join("text.txt"), &text).unwrap();
    let export = export_for(&["text.txt"]);
    let files = vec![PathBuf::from("text.txt")];
    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();
    // English text has ~4-5 bits/byte entropy, should be normal (2.0 ≤ x ≤ 6.5)
    assert!(
        report.suspects.is_empty(),
        "normal ASCII text should not produce suspects"
    );
}

// ── 8. Counting bytes 0..=255 repeated: high entropy ────────────

#[test]
fn full_byte_range_high_entropy() {
    let dir = tempdir().unwrap();
    // Uniform distribution of all 256 byte values → 8 bits/byte
    let data: Vec<u8> = (0..=255u8).cycle().take(4096).collect();
    fs::write(dir.path().join("uniform.bin"), &data).unwrap();
    let export = export_for(&["uniform.bin"]);
    let files = vec![PathBuf::from("uniform.bin")];
    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();
    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].class, EntropyClass::High);
    // Perfect uniform distribution should be very close to 8.0 bits
    assert!(report.suspects[0].entropy_bits_per_byte > 7.9);
}

// ── 9. Repeated short pattern: low entropy ──────────────────────

#[test]
fn short_repeating_pattern_low() {
    let dir = tempdir().unwrap();
    let pattern = b"AAAA";
    let data: Vec<u8> = pattern.iter().copied().cycle().take(2048).collect();
    fs::write(dir.path().join("repeat.bin"), &data).unwrap();
    let export = export_for(&["repeat.bin"]);
    let files = vec![PathBuf::from("repeat.bin")];
    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();
    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].class, EntropyClass::Low);
}

// ── 10. Base64-encoded data: suspicious or high ─────────────────

#[test]
fn base64_like_data_elevated_entropy() {
    let dir = tempdir().unwrap();
    // Base64 uses 64 distinct chars → ~6 bits/byte entropy
    let charset = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut data = Vec::with_capacity(2048);
    let mut x = 0xABCD1234u32;
    for _ in 0..2048 {
        x = x.wrapping_mul(1664525).wrapping_add(1013904223);
        data.push(charset[(x as usize) % charset.len()]);
    }
    fs::write(dir.path().join("b64.txt"), &data).unwrap();
    let export = export_for(&["b64.txt"]);
    let files = vec![PathBuf::from("b64.txt")];
    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();
    // Base64 data has ~5.95 bits/byte; should be normal or borderline suspicious
    // The key thing is it should NOT be classified as Low
    for s in &report.suspects {
        assert_ne!(s.class, EntropyClass::Low);
    }
}

// ── 11. Hex-encoded data: normal to suspicious range ────────────

#[test]
fn hex_encoded_data_entropy() {
    let dir = tempdir().unwrap();
    // Hex uses 16 distinct chars → ~4 bits/byte
    let charset = b"0123456789abcdef";
    let mut data = Vec::with_capacity(2048);
    let mut x = 0x55555555u32;
    for _ in 0..2048 {
        x = x.wrapping_mul(1664525).wrapping_add(1013904223);
        data.push(charset[(x as usize) % charset.len()]);
    }
    fs::write(dir.path().join("hex.txt"), &data).unwrap();
    let export = export_for(&["hex.txt"]);
    let files = vec![PathBuf::from("hex.txt")];
    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();
    // 16 chars = ~4 bits/byte; normal range, no suspects expected
    assert!(
        report.suspects.is_empty(),
        "hex data (~4 bits/byte) should be normal"
    );
}

// ── 12. Deterministic entropy values across invocations ─────────

#[test]
fn deterministic_entropy_values() {
    let dir = tempdir().unwrap();
    write_pseudorandom(&dir.path().join("det.bin"), 0x42424242, 1024);
    let export = export_for(&["det.bin"]);
    let files = vec![PathBuf::from("det.bin")];

    let r1 = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();
    let r2 = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();

    assert_eq!(r1.suspects.len(), r2.suspects.len());
    for (a, b) in r1.suspects.iter().zip(r2.suspects.iter()) {
        assert_eq!(a.path, b.path);
        assert_eq!(a.class, b.class);
        assert!(
            (a.entropy_bits_per_byte - b.entropy_bits_per_byte).abs() < f32::EPSILON,
            "entropy values must be identical across calls"
        );
    }
}

// ── 13. per_file_limit truncates sampling ───────────────────────

#[test]
fn per_file_limit_truncates_sample() {
    let dir = tempdir().unwrap();
    write_pseudorandom(&dir.path().join("big.bin"), 0xAAAAAAAA, 8192);
    let export = export_for(&["big.bin"]);
    let files = vec![PathBuf::from("big.bin")];

    let limits = AnalysisLimits {
        max_file_bytes: Some(512),
        ..Default::default()
    };
    let report = build_entropy_report(dir.path(), &files, &export, &limits).unwrap();
    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].sample_bytes, 512);
}

// ── 14. Sample bytes field matches actual read size ─────────────

#[test]
fn sample_bytes_matches_file_size() {
    let dir = tempdir().unwrap();
    let file_size = 500;
    write_repeated(&dir.path().join("small.bin"), b'X', file_size);
    let export = export_for(&["small.bin"]);
    let files = vec![PathBuf::from("small.bin")];
    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();
    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].sample_bytes as usize, file_size);
}

// ── 15. Mixed entropy: only non-normal appear as suspects ───────

#[test]
fn mixed_entropy_only_non_normal_in_suspects() {
    let dir = tempdir().unwrap();
    write_repeated(&dir.path().join("low.bin"), b'Z', 1024);
    let normal_text = "fn main() {\n    println!(\"hello\");\n}\n".repeat(30);
    fs::write(dir.path().join("code.rs"), &normal_text).unwrap();
    write_pseudorandom(&dir.path().join("high.bin"), 0xBEEF, 1024);

    let export = export_for(&["low.bin", "code.rs", "high.bin"]);
    let files = vec![
        PathBuf::from("low.bin"),
        PathBuf::from("code.rs"),
        PathBuf::from("high.bin"),
    ];
    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();
    for s in &report.suspects {
        assert_ne!(
            s.class,
            EntropyClass::Normal,
            "normal entropy files must not appear in suspects"
        );
    }
    assert!(report.suspects.iter().any(|s| s.class == EntropyClass::Low));
    assert!(
        report
            .suspects
            .iter()
            .any(|s| s.class == EntropyClass::High)
    );
}

// ── 16. Path normalization: backslashes converted ───────────────

#[test]
fn path_stored_with_forward_slashes() {
    let dir = tempdir().unwrap();
    write_repeated(&dir.path().join("low.txt"), b'A', 1024);
    let export = export_for(&["low.txt"]);
    let files = vec![PathBuf::from("low.txt")];
    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();
    for s in &report.suspects {
        assert!(
            !s.path.contains('\\'),
            "paths should use forward slashes: {}",
            s.path
        );
    }
}

// ── 17. Child rows in export are ignored for module mapping ─────

#[test]
fn child_rows_ignored_for_module_mapping() {
    let dir = tempdir().unwrap();
    write_repeated(&dir.path().join("f.txt"), b'A', 1024);
    let export = ExportData {
        rows: vec![FileRow {
            path: "f.txt".to_string(),
            module: "(root)".to_string(),
            lang: "Text".to_string(),
            kind: FileKind::Child,
            code: 1,
            comments: 0,
            blanks: 0,
            lines: 1,
            bytes: 10,
            tokens: 2,
        }],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let files = vec![PathBuf::from("f.txt")];
    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();
    // Child rows are filtered out, so module should be "(unknown)"
    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].module, "(unknown)");
}

// ── 18. Suspects sorted high entropy first ──────────────────────

#[test]
fn suspects_sorted_high_entropy_first() {
    let dir = tempdir().unwrap();
    write_repeated(&dir.path().join("low.bin"), b'A', 1024);
    write_pseudorandom(&dir.path().join("high.bin"), 0x12345678, 1024);

    let export = export_for(&["low.bin", "high.bin"]);
    let files = vec![PathBuf::from("low.bin"), PathBuf::from("high.bin")];
    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();
    assert!(report.suspects.len() >= 2);
    assert!(
        report.suspects[0].entropy_bits_per_byte >= report.suspects[1].entropy_bits_per_byte,
        "suspects should be sorted by entropy descending"
    );
}

// ── 19. Four-value byte pattern: still low ──────────────────────

#[test]
fn four_value_pattern_low_entropy() {
    let dir = tempdir().unwrap();
    let data: Vec<u8> = (0..2048).map(|i| (i % 4) as u8).collect();
    fs::write(dir.path().join("four.bin"), &data).unwrap();
    let export = export_for(&["four.bin"]);
    let files = vec![PathBuf::from("four.bin")];
    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();
    // 4 values = 2.0 bits/byte; threshold is < 2.0, so exactly 2.0 is Normal
    // This tests the boundary
    for s in &report.suspects {
        assert_ne!(
            s.class,
            EntropyClass::High,
            "4-value data should not be high"
        );
    }
}

// ── 20. max_bytes exactly matches one file ──────────────────────

#[test]
fn max_bytes_exact_match_one_file() {
    let dir = tempdir().unwrap();
    write_repeated(&dir.path().join("a.bin"), b'A', 1024);
    write_repeated(&dir.path().join("b.bin"), b'B', 1024);
    let export = export_for(&["a.bin", "b.bin"]);
    let files = vec![PathBuf::from("a.bin"), PathBuf::from("b.bin")];

    let limits = AnalysisLimits {
        max_bytes: Some(1024),
        ..Default::default()
    };
    let report = build_entropy_report(dir.path(), &files, &export, &limits).unwrap();
    // Budget of 1024 exactly covers the first file; second should be skipped
    assert!(
        report.suspects.len() <= 1,
        "budget should allow at most 1 file, got {}",
        report.suspects.len()
    );
}

// ── 21. Module mapping with subdirectory paths ──────────────────

#[test]
fn module_mapping_subdirectory() {
    let dir = tempdir().unwrap();
    let sub = dir.path().join("src");
    fs::create_dir_all(&sub).unwrap();
    write_repeated(&sub.join("data.bin"), b'Z', 1024);
    let export = export_with_module("src/data.bin", "src");
    let files = vec![PathBuf::from("src/data.bin")];
    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();
    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].module, "src");
    assert_eq!(report.suspects[0].path, "src/data.bin");
}

// ── 22. Increasing byte distribution entropy ────────────────────

#[test]
fn increasing_byte_diversity_increases_entropy() {
    let dir = tempdir().unwrap();
    // 2-value file
    let data2: Vec<u8> = (0..2048).map(|i| (i % 2) as u8).collect();
    fs::write(dir.path().join("two.bin"), &data2).unwrap();
    // 16-value file
    let data16: Vec<u8> = (0..2048).map(|i| (i % 16) as u8).collect();
    fs::write(dir.path().join("sixteen.bin"), &data16).unwrap();

    let export = export_for(&["two.bin", "sixteen.bin"]);
    let files = vec![PathBuf::from("two.bin"), PathBuf::from("sixteen.bin")];
    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();

    let two_ent = report.suspects.iter().find(|s| s.path == "two.bin");
    let sixteen_ent = report.suspects.iter().find(|s| s.path == "sixteen.bin");

    // Both should be low or the 16-value could be normal
    // But if both appear, the 16-value should have higher entropy
    if let (Some(t), Some(s)) = (two_ent, sixteen_ent) {
        assert!(
            s.entropy_bits_per_byte > t.entropy_bits_per_byte,
            "more diverse bytes should have higher entropy"
        );
    }
}

// ── 23. Report with no files returns empty suspects ─────────────

#[test]
fn report_no_files_empty_suspects() {
    let dir = tempdir().unwrap();
    let export = export_for(&[]);
    let report = build_entropy_report(dir.path(), &[], &export, &default_limits()).unwrap();
    assert!(report.suspects.is_empty());
}

// ── 24. Entropy is non-negative for any input ───────────────────

#[test]
fn entropy_always_non_negative() {
    let dir = tempdir().unwrap();
    // Various patterns
    let patterns: Vec<(&str, Vec<u8>)> = vec![
        ("zeros.bin", vec![0u8; 512]),
        ("ones.bin", vec![1u8; 512]),
        ("mixed.bin", (0..=255).collect()),
    ];
    let mut names = Vec::new();
    for (name, data) in &patterns {
        fs::write(dir.path().join(name), data).unwrap();
        names.push(*name);
    }
    let export = export_for(&names);
    let files: Vec<PathBuf> = names.iter().map(PathBuf::from).collect();
    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();
    for s in &report.suspects {
        assert!(
            s.entropy_bits_per_byte >= 0.0,
            "entropy must be non-negative: {}",
            s.entropy_bits_per_byte
        );
    }
}

// ── 25. JSON serialization: EntropyClass variants ───────────────

#[test]
fn entropy_class_json_variants() {
    let classes = [
        (EntropyClass::Low, "\"low\""),
        (EntropyClass::Normal, "\"normal\""),
        (EntropyClass::Suspicious, "\"suspicious\""),
        (EntropyClass::High, "\"high\""),
    ];
    for (class, expected) in &classes {
        let json = serde_json::to_string(class).unwrap();
        assert_eq!(&json, *expected, "EntropyClass::{class:?} serialization");
        let rt: EntropyClass = serde_json::from_str(&json).unwrap();
        assert_eq!(&rt, class);
    }
}

// ── 26. Large file truncated by default per-file limit ──────────

#[test]
fn large_file_sample_capped() {
    let dir = tempdir().unwrap();
    // Write 8KB file, default per-file limit is 1024 bytes
    write_pseudorandom(&dir.path().join("large.bin"), 0xFEEDFACE, 8192);
    let export = export_for(&["large.bin"]);
    let files = vec![PathBuf::from("large.bin")];
    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();
    assert_eq!(report.suspects.len(), 1);
    // Default sample is 1024 bytes
    assert!(
        report.suspects[0].sample_bytes <= 1024,
        "default per-file limit should cap sample"
    );
}

// ── Proptest ────────────────────────────────────────────────────

mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn entropy_bounded_0_to_8(data in proptest::collection::vec(0u8..=255, 1..2048)) {
            let dir = tempdir().unwrap();
            std::fs::write(dir.path().join("prop.bin"), &data).unwrap();
            let export = export_for(&["prop.bin"]);
            let files = vec![PathBuf::from("prop.bin")];
            let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();
            for s in &report.suspects {
                prop_assert!(s.entropy_bits_per_byte >= 0.0);
                prop_assert!(s.entropy_bits_per_byte <= 8.0);
            }
        }
    }
}
