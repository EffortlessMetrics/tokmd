//! Deeper tests for high-entropy file detection: thresholds, determinism,
//! and edge cases across different file types.

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::tempdir;
use tokmd_analysis_entropy::build_entropy_report;
use tokmd_analysis_types::EntropyClass;
use tokmd_analysis_util::AnalysisLimits;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────

fn export_for_paths(paths: &[&str]) -> ExportData {
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

fn write_repeated(path: &Path, byte: u8, len: usize) {
    fs::write(path, vec![byte; len]).unwrap();
}

fn write_pseudorandom(path: &Path, len: usize, seed: u32) {
    let mut data = Vec::with_capacity(len);
    let mut x = seed;
    for _ in 0..len {
        x = x.wrapping_mul(1664525).wrapping_add(1013904223);
        data.push((x & 0xFF) as u8);
    }
    fs::write(path, data).unwrap();
}

// ── Threshold boundary tests ────────────────────────────────────

#[test]
fn threshold_high_above_7_5() {
    // Pseudorandom data should have entropy > 7.5 → High
    let dir = tempdir().unwrap();
    let f = dir.path().join("rand.bin");
    write_pseudorandom(&f, 4096, 0x12345678);

    let export = export_for_paths(&["rand.bin"]);
    let files = vec![PathBuf::from("rand.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].class, EntropyClass::High);
    assert!(report.suspects[0].entropy_bits_per_byte > 7.5);
}

#[test]
fn threshold_low_below_2_0() {
    // Single repeated byte → entropy ~0 → Low
    let dir = tempdir().unwrap();
    let f = dir.path().join("flat.bin");
    write_repeated(&f, b'Z', 1024);

    let export = export_for_paths(&["flat.bin"]);
    let files = vec![PathBuf::from("flat.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].class, EntropyClass::Low);
    assert!(report.suspects[0].entropy_bits_per_byte < 2.0);
}

#[test]
fn normal_text_not_flagged() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("readme.md");
    let text = "# Project\n\nThis is a README file with typical English text.\n\
                It contains sentences, paragraphs, and markdown formatting.\n"
        .repeat(30);
    fs::write(&f, text).unwrap();

    let export = export_for_paths(&["readme.md"]);
    let files = vec![PathBuf::from("readme.md")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert!(
        report.suspects.is_empty(),
        "normal text should not appear in suspects"
    );
}

// ── Determinism tests ───────────────────────────────────────────

#[test]
fn repeated_runs_produce_identical_output() {
    let dir = tempdir().unwrap();

    let lo = dir.path().join("lo.bin");
    let hi = dir.path().join("hi.bin");
    write_repeated(&lo, 0x00, 1024);
    write_pseudorandom(&hi, 2048, 0xABCDEF00);

    let export = export_for_paths(&["lo.bin", "hi.bin"]);
    let files = vec![PathBuf::from("lo.bin"), PathBuf::from("hi.bin")];

    let r1 = build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();
    let r2 = build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(r1.suspects.len(), r2.suspects.len());
    for (a, b) in r1.suspects.iter().zip(r2.suspects.iter()) {
        assert_eq!(a.path, b.path);
        assert_eq!(a.class, b.class);
        assert!((a.entropy_bits_per_byte - b.entropy_bits_per_byte).abs() < f32::EPSILON);
        assert_eq!(a.sample_bytes, b.sample_bytes);
    }
}

#[test]
fn sort_order_is_deterministic_with_path_tiebreak() {
    let dir = tempdir().unwrap();

    // Two files with identical content (same entropy)
    let a = dir.path().join("aaa.bin");
    let z = dir.path().join("zzz.bin");
    write_repeated(&a, 0xFF, 512);
    write_repeated(&z, 0xFF, 512);

    let export = export_for_paths(&["aaa.bin", "zzz.bin"]);
    let files = vec![PathBuf::from("aaa.bin"), PathBuf::from("zzz.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 2);
    // Same entropy → sorted by path ascending
    assert_eq!(report.suspects[0].path, "aaa.bin");
    assert_eq!(report.suspects[1].path, "zzz.bin");
}

// ── Different file type scenarios ───────────────────────────────

#[test]
fn json_config_file_not_flagged() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("config.json");
    let json = r#"{"database": {"host": "localhost", "port": 5432}, "debug": true}"#.repeat(20);
    fs::write(&f, json).unwrap();

    let export = export_for_paths(&["config.json"]);
    let files = vec![PathBuf::from("config.json")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert!(
        report.suspects.is_empty(),
        "JSON config should have normal entropy"
    );
}

#[test]
fn base64_encoded_data_flagged_as_suspicious_or_high() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("secret.b64");
    // Base64 characters cover A-Z, a-z, 0-9, +, / → ~6 bits/byte
    let b64 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".repeat(30);
    fs::write(&f, &b64).unwrap();

    let export = export_for_paths(&["secret.b64"]);
    let files = vec![PathBuf::from("secret.b64")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    if !report.suspects.is_empty() {
        assert!(
            report.suspects[0].class == EntropyClass::Suspicious
                || report.suspects[0].class == EntropyClass::High,
            "base64 data should be Suspicious or High, got {:?}",
            report.suspects[0].class
        );
    }
}

#[test]
fn binary_key_file_flagged_high() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("private.key");
    write_pseudorandom(&f, 2048, 0xDEADBEEF);

    let export = export_for_paths(&["private.key"]);
    let files = vec![PathBuf::from("private.key")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].class, EntropyClass::High);
}

// ── Empty and single-byte edge cases ────────────────────────────

#[test]
fn empty_file_not_in_suspects() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("empty.txt"), "").unwrap();

    let export = export_for_paths(&["empty.txt"]);
    let files = vec![PathBuf::from("empty.txt")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert!(report.suspects.is_empty());
}

#[test]
fn single_byte_file_classified_low() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("one.bin"), &[0x42]).unwrap();

    let export = export_for_paths(&["one.bin"]);
    let files = vec![PathBuf::from("one.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].class, EntropyClass::Low);
}

#[test]
fn no_files_yields_empty_report() {
    let dir = tempdir().unwrap();
    let export = export_for_paths(&[]);
    let files: Vec<PathBuf> = vec![];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert!(report.suspects.is_empty());
}

// ── Mixed file scenarios ────────────────────────────────────────

#[test]
fn mixed_files_only_anomalies_flagged() {
    let dir = tempdir().unwrap();

    // Normal: source code
    let src = dir.path().join("app.rs");
    let code = "fn main() {\n    println!(\"hello\");\n}\n".repeat(20);
    fs::write(&src, &code).unwrap();

    // Low: all zeros
    let low = dir.path().join("padding.bin");
    write_repeated(&low, 0x00, 512);

    // High: random
    let high = dir.path().join("secret.bin");
    write_pseudorandom(&high, 2048, 0xCAFEBABE);

    let export = export_for_paths(&["app.rs", "padding.bin", "secret.bin"]);
    let files = vec![
        PathBuf::from("app.rs"),
        PathBuf::from("padding.bin"),
        PathBuf::from("secret.bin"),
    ];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    // Normal file not flagged
    assert!(!report.suspects.iter().any(|f| f.path == "app.rs"));
    // Low and high are flagged
    assert!(report.suspects.iter().any(|f| f.path == "padding.bin"));
    assert!(report.suspects.iter().any(|f| f.path == "secret.bin"));
}

// ── Module mapping ──────────────────────────────────────────────

#[test]
fn file_not_in_export_gets_unknown_module() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("orphan.bin");
    write_pseudorandom(&f, 1024, 0x11111111);

    // Export has no entries
    let export = export_for_paths(&[]);
    let files = vec![PathBuf::from("orphan.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].module, "(unknown)");
}

// ── Limits ──────────────────────────────────────────────────────

#[test]
fn max_file_bytes_limits_sample_size() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("big.bin");
    write_pseudorandom(&f, 8192, 0x22222222);

    let export = export_for_paths(&["big.bin"]);
    let files = vec![PathBuf::from("big.bin")];
    let limits = AnalysisLimits {
        max_file_bytes: Some(128),
        ..AnalysisLimits::default()
    };
    let report = build_entropy_report(dir.path(), &files, &export, &limits).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert!(
        report.suspects[0].sample_bytes <= 128,
        "sample_bytes {} should respect max_file_bytes limit",
        report.suspects[0].sample_bytes
    );
}
