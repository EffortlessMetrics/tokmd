//! W75 security & identity tests for entropy profiling.
//!
//! Focuses on:
//! - Entropy calculation on known byte patterns
//! - High-entropy detection (pseudorandom data)
//! - Low-entropy detection (repeated bytes)
//! - Suspicious-range entropy detection
//! - Entropy thresholds and classification boundaries
//! - File classification and report structure

use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use tokmd_analysis_entropy::build_entropy_report;
use tokmd_analysis_types::AnalysisLimits;
use tokmd_analysis_types::EntropyClass;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────

fn export_for(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn parent_row(path: &str) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: "(root)".to_string(),
        lang: "Text".to_string(),
        kind: FileKind::Parent,
        code: 1,
        comments: 0,
        blanks: 0,
        lines: 1,
        bytes: 10,
        tokens: 2,
    }
}

fn write_repeated(path: &Path, byte: u8, len: usize) {
    fs::write(path, vec![byte; len]).unwrap();
}

fn write_pseudorandom(path: &Path, seed: u32, len: usize) {
    let mut data = Vec::with_capacity(len);
    let mut x = seed;
    for _ in 0..len {
        x = x.wrapping_mul(1664525).wrapping_add(1013904223);
        data.push((x >> 16) as u8);
    }
    fs::write(path, data).unwrap();
}

fn write_two_byte_pattern(path: &Path, a: u8, b: u8, len: usize) {
    let data: Vec<u8> = (0..len).map(|i| if i % 2 == 0 { a } else { b }).collect();
    fs::write(path, data).unwrap();
}

// ===========================================================================
// 1. All-zeros file classified as Low entropy
// ===========================================================================

#[test]
fn all_zeros_classified_as_low() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("zeros.bin");
    write_repeated(&f, 0x00, 2048);

    let export = export_for(vec![parent_row("zeros.bin")]);
    let files = vec![PathBuf::from("zeros.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].class, EntropyClass::Low);
    assert!(report.suspects[0].entropy_bits_per_byte < 0.01);
}

// ===========================================================================
// 2. All-0xFF file classified as Low entropy
// ===========================================================================

#[test]
fn all_ff_classified_as_low() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("ones.bin");
    write_repeated(&f, 0xFF, 2048);

    let export = export_for(vec![parent_row("ones.bin")]);
    let files = vec![PathBuf::from("ones.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].class, EntropyClass::Low);
}

// ===========================================================================
// 3. Pseudorandom data classified as High entropy
// ===========================================================================

#[test]
fn pseudorandom_data_classified_as_high() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("random.bin");
    write_pseudorandom(&f, 0xABCDEF12, 4096);

    let export = export_for(vec![parent_row("random.bin")]);
    let files = vec![PathBuf::from("random.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].class, EntropyClass::High);
    assert!(report.suspects[0].entropy_bits_per_byte > 7.5);
}

// ===========================================================================
// 4. Normal English text excluded from suspects
// ===========================================================================

#[test]
fn normal_english_text_not_suspicious() {
    let dir = tempdir().unwrap();
    let text = "The quick brown fox jumps over the lazy dog. ".repeat(50);
    fs::write(dir.path().join("readme.txt"), text).unwrap();

    let export = export_for(vec![parent_row("readme.txt")]);
    let files = vec![PathBuf::from("readme.txt")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert!(
        report.suspects.is_empty(),
        "normal text should not appear as suspect"
    );
}

// ===========================================================================
// 5. Two-byte alternating pattern has low entropy (< 2.0)
// ===========================================================================

#[test]
fn two_byte_alternating_pattern_low_entropy() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("pattern.bin");
    write_two_byte_pattern(&f, 0xAA, 0x55, 2048);

    let export = export_for(vec![parent_row("pattern.bin")]);
    let files = vec![PathBuf::from("pattern.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].class, EntropyClass::Low);
    // Two distinct byte values → entropy = 1.0 bit/byte
    assert!(report.suspects[0].entropy_bits_per_byte < 2.0);
}

// ===========================================================================
// 6. Empty file excluded from suspects
// ===========================================================================

#[test]
fn empty_file_not_in_suspects() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("empty.txt"), b"").unwrap();

    let export = export_for(vec![parent_row("empty.txt")]);
    let files = vec![PathBuf::from("empty.txt")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert!(report.suspects.is_empty());
}

// ===========================================================================
// 7. High-entropy file near threshold (7.5 boundary)
// ===========================================================================

#[test]
fn high_entropy_threshold_is_above_7_5() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("high.bin");
    write_pseudorandom(&f, 0x42424242, 8192);

    let export = export_for(vec![parent_row("high.bin")]);
    let files = vec![PathBuf::from("high.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    if report.suspects[0].class == EntropyClass::High {
        assert!(report.suspects[0].entropy_bits_per_byte > 7.5);
    }
}

// ===========================================================================
// 8. Low entropy threshold is below 2.0
// ===========================================================================

#[test]
fn low_entropy_threshold_is_below_2_0() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("low.bin");
    write_repeated(&f, b'Z', 4096);

    let export = export_for(vec![parent_row("low.bin")]);
    let files = vec![PathBuf::from("low.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].class, EntropyClass::Low);
    assert!(report.suspects[0].entropy_bits_per_byte < 2.0);
}

// ===========================================================================
// 9. Multiple files: mixed entropy classes sorted correctly
// ===========================================================================

#[test]
fn mixed_entropy_files_sorted_descending() {
    let dir = tempdir().unwrap();
    let low = dir.path().join("low.bin");
    let high = dir.path().join("high.bin");
    write_repeated(&low, 0x00, 1024);
    write_pseudorandom(&high, 0xDEAD, 4096);

    let export = export_for(vec![parent_row("low.bin"), parent_row("high.bin")]);
    let files = vec![PathBuf::from("low.bin"), PathBuf::from("high.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert!(report.suspects.len() >= 2);
    // First suspect should have higher entropy
    assert!(
        report.suspects[0].entropy_bits_per_byte >= report.suspects[1].entropy_bits_per_byte,
        "suspects should be sorted by entropy descending"
    );
}

// ===========================================================================
// 10. Entropy report is deterministic
// ===========================================================================

#[test]
fn entropy_report_deterministic_across_calls() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("data.bin");
    write_pseudorandom(&f, 0xCAFE, 2048);

    let export = export_for(vec![parent_row("data.bin")]);
    let files = vec![PathBuf::from("data.bin")];

    let r1 = build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();
    let r2 = build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(r1.suspects.len(), r2.suspects.len());
    for (a, b) in r1.suspects.iter().zip(r2.suspects.iter()) {
        assert_eq!(a.path, b.path);
        assert_eq!(a.class, b.class);
        assert!(
            (a.entropy_bits_per_byte - b.entropy_bits_per_byte).abs() < f32::EPSILON,
            "entropy should be identical across calls"
        );
    }
}

// ===========================================================================
// 11. Sample bytes field matches actual file size for small files
// ===========================================================================

#[test]
fn sample_bytes_matches_small_file_size() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("tiny.bin");
    write_repeated(&f, 0x00, 100);

    let export = export_for(vec![parent_row("tiny.bin")]);
    let files = vec![PathBuf::from("tiny.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].sample_bytes, 100);
}

// ===========================================================================
// 12. max_file_bytes limits sample size per file
// ===========================================================================

#[test]
fn max_file_bytes_limits_sample() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("big.bin");
    write_pseudorandom(&f, 0x1111, 10000);

    let export = export_for(vec![parent_row("big.bin")]);
    let files = vec![PathBuf::from("big.bin")];
    let limits = AnalysisLimits {
        max_file_bytes: Some(256),
        ..AnalysisLimits::default()
    };
    let report = build_entropy_report(dir.path(), &files, &export, &limits).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert!(
        report.suspects[0].sample_bytes <= 256,
        "sample_bytes {} should respect max_file_bytes=256",
        report.suspects[0].sample_bytes
    );
}

// ===========================================================================
// 13. Suspects capped at 50
// ===========================================================================

#[test]
fn suspects_truncated_at_50() {
    let dir = tempdir().unwrap();
    let mut rows = Vec::new();
    let mut files = Vec::new();
    for i in 0..60 {
        let name = format!("f{i:03}.bin");
        let f = dir.path().join(&name);
        write_pseudorandom(&f, i as u32 + 1, 2048);
        rows.push(parent_row(&name));
        files.push(PathBuf::from(name));
    }

    let export = export_for(rows);
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert!(
        report.suspects.len() <= 50,
        "suspects should be capped at 50, got {}",
        report.suspects.len()
    );
}

// ===========================================================================
// 14. Path in finding uses forward slashes
// ===========================================================================

#[test]
fn finding_path_uses_forward_slashes() {
    let dir = tempdir().unwrap();
    let sub = dir.path().join("secrets");
    fs::create_dir_all(&sub).unwrap();
    let f = sub.join("key.bin");
    write_pseudorandom(&f, 0xBEEF, 2048);

    let rel = PathBuf::from("secrets").join("key.bin");
    let export = export_for(vec![parent_row("secrets/key.bin")]);
    let files = vec![rel];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert!(
        !report.suspects[0].path.contains('\\'),
        "path should use forward slashes: {}",
        report.suspects[0].path
    );
}

// ===========================================================================
// 15. No files yields empty report
// ===========================================================================

#[test]
fn no_files_yields_empty_report() {
    let dir = tempdir().unwrap();
    let export = export_for(vec![]);
    let report =
        build_entropy_report(dir.path(), &[], &export, &AnalysisLimits::default()).unwrap();

    assert!(report.suspects.is_empty());
}
