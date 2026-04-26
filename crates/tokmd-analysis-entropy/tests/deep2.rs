//! Additional deep tests for entropy detection.
//!
//! Covers serialization roundtrips, Shannon entropy calculation,
//! zero/max entropy cases, classification boundaries, and edge cases.

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::tempdir;
use tokmd_analysis_entropy::build_entropy_report;
use tokmd_analysis_types::AnalysisLimits;
use tokmd_analysis_types::{EntropyClass, EntropyFinding, EntropyReport};
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

fn write_pseudorandom(path: &Path, seed: u32, len: usize) {
    let mut data = Vec::with_capacity(len);
    let mut x = seed;
    for _ in 0..len {
        x = x.wrapping_mul(1664525).wrapping_add(1013904223);
        data.push((x >> 16) as u8);
    }
    fs::write(path, data).unwrap();
}

// ── 1. EntropyReport serialization roundtrip ────────────────────

#[test]
fn entropy_report_serialization_roundtrip() {
    let report = EntropyReport {
        suspects: vec![
            EntropyFinding {
                path: "secret.bin".to_string(),
                module: "(root)".to_string(),
                entropy_bits_per_byte: 7.9,
                sample_bytes: 1024,
                class: EntropyClass::High,
            },
            EntropyFinding {
                path: "empty.bin".to_string(),
                module: "(root)".to_string(),
                entropy_bits_per_byte: 0.0,
                sample_bytes: 512,
                class: EntropyClass::Low,
            },
        ],
    };

    let json = serde_json::to_string_pretty(&report).unwrap();
    let back: EntropyReport = serde_json::from_str(&json).unwrap();

    assert_eq!(back.suspects.len(), 2);
    assert_eq!(back.suspects[0].path, "secret.bin");
    assert_eq!(back.suspects[0].class, EntropyClass::High);
    assert_eq!(back.suspects[1].path, "empty.bin");
    assert_eq!(back.suspects[1].class, EntropyClass::Low);
}

// ── 2. EntropyFinding serialization roundtrip ───────────────────

#[test]
fn entropy_finding_serialization_roundtrip() {
    let finding = EntropyFinding {
        path: "data/key.pem".to_string(),
        module: "data".to_string(),
        entropy_bits_per_byte: 6.8,
        sample_bytes: 2048,
        class: EntropyClass::Suspicious,
    };

    let json = serde_json::to_string(&finding).unwrap();
    let back: EntropyFinding = serde_json::from_str(&json).unwrap();

    assert_eq!(back.path, "data/key.pem");
    assert_eq!(back.module, "data");
    assert!((back.entropy_bits_per_byte - 6.8).abs() < 0.01);
    assert_eq!(back.sample_bytes, 2048);
    assert_eq!(back.class, EntropyClass::Suspicious);
}

// ── 3. EntropyClass serialization roundtrip ─────────────────────

#[test]
fn entropy_class_serialization_roundtrip() {
    for class in [
        EntropyClass::Low,
        EntropyClass::Normal,
        EntropyClass::Suspicious,
        EntropyClass::High,
    ] {
        let json = serde_json::to_string(&class).unwrap();
        let back: EntropyClass = serde_json::from_str(&json).unwrap();
        assert_eq!(back, class);
    }
}

// ── 4. Zero entropy: single byte repeated ───────────────────────

#[test]
fn single_byte_repeated_has_zero_entropy() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("uniform.bin");
    write_repeated(&f, 0x42, 2048);

    let export = export_for_paths(&["uniform.bin"]);
    let files = vec![PathBuf::from("uniform.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert!(
        report.suspects[0].entropy_bits_per_byte < 0.01,
        "single repeated byte should have ~0 entropy, got {}",
        report.suspects[0].entropy_bits_per_byte
    );
    assert_eq!(report.suspects[0].class, EntropyClass::Low);
}

// ── 5. Max entropy: uniform byte distribution ───────────────────

#[test]
fn uniform_byte_distribution_has_max_entropy() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("max_entropy.bin");
    // Create data with all 256 byte values equally distributed
    let mut data = Vec::with_capacity(256 * 16);
    for _ in 0..16 {
        for b in 0u8..=255 {
            data.push(b);
        }
    }
    fs::write(&f, &data).unwrap();

    let export = export_for_paths(&["max_entropy.bin"]);
    let files = vec![PathBuf::from("max_entropy.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert!(
        report.suspects[0].entropy_bits_per_byte > 7.5,
        "uniform distribution should have entropy ~8.0, got {}",
        report.suspects[0].entropy_bits_per_byte
    );
    assert_eq!(report.suspects[0].class, EntropyClass::High);
}

// ── 6. Empty files list produces empty report ───────────────────

#[test]
fn empty_files_list_produces_empty_report() {
    let dir = tempdir().unwrap();
    let export = export_for_paths(&[]);
    let files: Vec<PathBuf> = vec![];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert!(report.suspects.is_empty());
}

// ── 7. Classification boundary: entropy > 7.5 → High ───────────

#[test]
fn classification_boundary_high() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("boundary_high.bin");
    write_pseudorandom(&f, 0xAAAA, 4096);

    let export = export_for_paths(&["boundary_high.bin"]);
    let files = vec![PathBuf::from("boundary_high.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert!(report.suspects[0].entropy_bits_per_byte > 7.5);
    assert_eq!(report.suspects[0].class, EntropyClass::High);
}

// ── 8. Classification boundary: entropy < 2.0 → Low ────────────

#[test]
fn classification_boundary_low() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("boundary_low.bin");
    // Two byte values only → entropy ≈ 1.0 bit/byte → Low
    let data: Vec<u8> = (0..2048)
        .map(|i| if i % 2 == 0 { 0x00 } else { 0x01 })
        .collect();
    fs::write(&f, &data).unwrap();

    let export = export_for_paths(&["boundary_low.bin"]);
    let files = vec![PathBuf::from("boundary_low.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert!(report.suspects[0].entropy_bits_per_byte < 2.0);
    assert_eq!(report.suspects[0].class, EntropyClass::Low);
}

// ── 9. Normal range (2.0-6.5) excluded from suspects ────────────

#[test]
fn normal_range_excluded_from_suspects() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("normal.txt");
    // English text typically has entropy ~3.5-4.5 bits/byte
    let text = "The quick brown fox jumps over the lazy dog. ".repeat(50);
    fs::write(&f, text.as_bytes()).unwrap();

    let export = export_for_paths(&["normal.txt"]);
    let files = vec![PathBuf::from("normal.txt")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    // Normal class is filtered out of suspects
    for suspect in &report.suspects {
        assert_ne!(suspect.class, EntropyClass::Normal);
    }
}

// ── 10. Suspects sorted by entropy descending ───────────────────

#[test]
fn suspects_sorted_by_entropy_descending() {
    let dir = tempdir().unwrap();
    // Create files with varying entropy levels
    write_repeated(&dir.path().join("low.bin"), 0x00, 1024);
    write_pseudorandom(&dir.path().join("high.bin"), 0x1111, 4096);
    // Two byte values → entropy ~1.0
    let two_bytes: Vec<u8> = (0..2048)
        .map(|i| if i % 2 == 0 { 0xAA } else { 0x55 })
        .collect();
    fs::write(dir.path().join("medium_low.bin"), &two_bytes).unwrap();

    let export = export_for_paths(&["low.bin", "high.bin", "medium_low.bin"]);
    let files = vec![
        PathBuf::from("low.bin"),
        PathBuf::from("high.bin"),
        PathBuf::from("medium_low.bin"),
    ];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    for window in report.suspects.windows(2) {
        assert!(
            window[0].entropy_bits_per_byte >= window[1].entropy_bits_per_byte,
            "suspects not sorted by entropy desc: {} < {}",
            window[0].entropy_bits_per_byte,
            window[1].entropy_bits_per_byte
        );
    }
}

// ── 11. max_file_bytes limit controls sample size ───────────────

#[test]
fn max_file_bytes_controls_sample_size() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("data.bin");
    write_pseudorandom(&f, 0x5678, 8192);

    let export = export_for_paths(&["data.bin"]);
    let files = vec![PathBuf::from("data.bin")];
    let limits = AnalysisLimits {
        max_file_bytes: Some(256),
        ..AnalysisLimits::default()
    };
    let report = build_entropy_report(dir.path(), &files, &export, &limits).unwrap();

    // With max_file_bytes=256, sample_bytes should be ≤256
    if !report.suspects.is_empty() {
        assert!(
            report.suspects[0].sample_bytes <= 256,
            "sample_bytes {} should be <= max_file_bytes 256",
            report.suspects[0].sample_bytes
        );
    }
}

// ── 12. max_bytes budget stops scanning mid-list ────────────────

#[test]
fn max_bytes_budget_stops_scanning_early() {
    let dir = tempdir().unwrap();
    for i in 0..10 {
        let f = dir.path().join(format!("f{i}.bin"));
        write_pseudorandom(&f, i + 100, 2048);
    }

    let names: Vec<String> = (0..10).map(|i| format!("f{i}.bin")).collect();
    let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let export = export_for_paths(&name_refs);
    let files: Vec<PathBuf> = names.iter().map(PathBuf::from).collect();

    // Budget of 4096 bytes → should scan ~2-4 files (each reads ~1024 bytes default)
    let limits = AnalysisLimits {
        max_bytes: Some(4096),
        ..AnalysisLimits::default()
    };
    let report = build_entropy_report(dir.path(), &files, &export, &limits).unwrap();

    assert!(
        report.suspects.len() < 10,
        "should not scan all 10 files with limited budget, got {}",
        report.suspects.len()
    );
}

// ── 13. Empty file produces no suspects ─────────────────────────

#[test]
fn empty_file_produces_no_suspects() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("empty.bin");
    fs::write(&f, b"").unwrap();

    let export = export_for_paths(&["empty.bin"]);
    let files = vec![PathBuf::from("empty.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert!(
        report.suspects.is_empty(),
        "empty file should not produce suspects"
    );
}

// ── 14. Suspicious class boundary (6.5 ≤ entropy ≤ 7.5) ────────

#[test]
fn suspicious_class_exists_in_boundary() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("suspicious.bin");
    // Use a restricted alphabet to target entropy ~6.5-7.5
    // 128 byte values → max entropy = log2(128) = 7.0 bits/byte
    let mut data = Vec::with_capacity(4096);
    let mut x = 0xDEADu32;
    for _ in 0..4096 {
        x = x.wrapping_mul(1664525).wrapping_add(1013904223);
        data.push((x >> 16) as u8 & 0x7F); // restrict to 0-127
    }
    fs::write(&f, &data).unwrap();

    let export = export_for_paths(&["suspicious.bin"]);
    let files = vec![PathBuf::from("suspicious.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    if !report.suspects.is_empty() {
        let s = &report.suspects[0];
        // Should be Suspicious (6.5-7.5) or High (>7.5) depending on exact distribution
        assert!(
            s.class == EntropyClass::Suspicious || s.class == EntropyClass::High,
            "128-value alphabet should produce Suspicious or High, got {:?} (entropy={})",
            s.class,
            s.entropy_bits_per_byte
        );
    }
}

// ── 15. Two byte values → entropy ≈ 1.0 (Low class) ────────────

#[test]
fn two_byte_values_low_entropy() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("two_vals.bin");
    let data: Vec<u8> = (0..2048)
        .map(|i| if i % 2 == 0 { 0x00 } else { 0xFF })
        .collect();
    fs::write(&f, &data).unwrap();

    let export = export_for_paths(&["two_vals.bin"]);
    let files = vec![PathBuf::from("two_vals.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert!(
        (report.suspects[0].entropy_bits_per_byte - 1.0).abs() < 0.1,
        "two equally distributed byte values should have entropy ~1.0, got {}",
        report.suspects[0].entropy_bits_per_byte
    );
    assert_eq!(report.suspects[0].class, EntropyClass::Low);
}

// ── 16. Four byte values → entropy ≈ 2.0 (Normal) ──────────────

#[test]
fn four_byte_values_normal_entropy() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("four_vals.bin");
    let data: Vec<u8> = (0..2048).map(|i| (i % 4) as u8).collect();
    fs::write(&f, &data).unwrap();

    let export = export_for_paths(&["four_vals.bin"]);
    let files = vec![PathBuf::from("four_vals.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    // 4 equally distributed values → entropy = log2(4) = 2.0 → Normal
    // Normal is excluded from suspects
    assert!(
        report.suspects.is_empty(),
        "four equally distributed byte values (entropy ~2.0) should be Normal"
    );
}

// ── 17. Sample bytes field is positive for non-empty files ──────

#[test]
fn sample_bytes_positive_for_non_empty_files() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("data.bin");
    write_pseudorandom(&f, 0x9999, 4096);

    let export = export_for_paths(&["data.bin"]);
    let files = vec![PathBuf::from("data.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    for suspect in &report.suspects {
        assert!(
            suspect.sample_bytes > 0,
            "sample_bytes should be positive for non-empty files"
        );
    }
}

// ── 18. Determinism: identical runs produce same output ─────────

#[test]
fn five_runs_produce_identical_results() {
    let dir = tempdir().unwrap();
    write_repeated(&dir.path().join("lo.bin"), 0x00, 1024);
    write_pseudorandom(&dir.path().join("hi.bin"), 0xBBBB, 4096);

    let export = export_for_paths(&["lo.bin", "hi.bin"]);
    let files = vec![PathBuf::from("lo.bin"), PathBuf::from("hi.bin")];

    let results: Vec<EntropyReport> = (0..5)
        .map(|_| {
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap()
        })
        .collect();

    for i in 1..5 {
        assert_eq!(results[0].suspects.len(), results[i].suspects.len());
        for (a, b) in results[0].suspects.iter().zip(results[i].suspects.iter()) {
            assert_eq!(a.path, b.path);
            assert_eq!(a.class, b.class);
            assert!((a.entropy_bits_per_byte - b.entropy_bits_per_byte).abs() < f32::EPSILON);
        }
    }
}

// ── 19. EntropyClass uses snake_case in JSON ────────────────────

#[test]
fn entropy_class_uses_snake_case_json() {
    assert_eq!(
        serde_json::to_string(&EntropyClass::Low).unwrap(),
        "\"low\""
    );
    assert_eq!(
        serde_json::to_string(&EntropyClass::Normal).unwrap(),
        "\"normal\""
    );
    assert_eq!(
        serde_json::to_string(&EntropyClass::Suspicious).unwrap(),
        "\"suspicious\""
    );
    assert_eq!(
        serde_json::to_string(&EntropyClass::High).unwrap(),
        "\"high\""
    );
}

// ── 20. Unknown module defaults to "(unknown)" ──────────────────

#[test]
fn unknown_module_defaults_correctly() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("orphan.bin");
    write_pseudorandom(&f, 0xCCCC, 2048);

    // Export has no matching row for "orphan.bin"
    let export = export_for_paths(&[]);
    let files = vec![PathBuf::from("orphan.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    if !report.suspects.is_empty() {
        assert_eq!(
            report.suspects[0].module, "(unknown)",
            "files not in export should get '(unknown)' module"
        );
    }
}

// ── 21. Multiple files with mixed entropy classes ───────────────

#[test]
fn mixed_entropy_classes_in_single_report() {
    let dir = tempdir().unwrap();
    write_repeated(&dir.path().join("low.bin"), 0x00, 1024);
    write_pseudorandom(&dir.path().join("high.bin"), 0xEEEE, 4096);

    let export = export_for_paths(&["low.bin", "high.bin"]);
    let files = vec![PathBuf::from("low.bin"), PathBuf::from("high.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    let classes: Vec<EntropyClass> = report.suspects.iter().map(|s| s.class).collect();
    assert!(classes.contains(&EntropyClass::Low));
    assert!(classes.contains(&EntropyClass::High));
}

// ── 22. Entropy bits per byte is in valid range [0, 8] ──────────

#[test]
fn entropy_bits_per_byte_in_valid_range() {
    let dir = tempdir().unwrap();
    for (i, byte) in [0x00u8, 0xFF, 0x42].iter().enumerate() {
        let f = dir.path().join(format!("f{i}.bin"));
        write_repeated(&f, *byte, 1024);
    }
    write_pseudorandom(&dir.path().join("random.bin"), 0x1234, 4096);

    let export = export_for_paths(&["f0.bin", "f1.bin", "f2.bin", "random.bin"]);
    let files = vec![
        PathBuf::from("f0.bin"),
        PathBuf::from("f1.bin"),
        PathBuf::from("f2.bin"),
        PathBuf::from("random.bin"),
    ];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    for suspect in &report.suspects {
        assert!(
            suspect.entropy_bits_per_byte >= 0.0 && suspect.entropy_bits_per_byte <= 8.0,
            "entropy {} out of [0, 8] for {}",
            suspect.entropy_bits_per_byte,
            suspect.path
        );
    }
}

// ── 23. Path preserved correctly in findings ────────────────────

#[test]
fn path_preserved_in_findings() {
    let dir = tempdir().unwrap();
    let sub = dir.path().join("nested").join("deep");
    fs::create_dir_all(&sub).unwrap();
    let f = sub.join("secret.key");
    write_pseudorandom(&f, 0xDDDD, 2048);

    let rows = vec![FileRow {
        path: "nested/deep/secret.key".to_string(),
        module: "nested/deep".to_string(),
        lang: "Text".to_string(),
        kind: FileKind::Parent,
        code: 1,
        comments: 0,
        blanks: 0,
        lines: 1,
        bytes: 10,
        tokens: 2,
    }];
    let export = ExportData {
        rows,
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };
    let files = vec![PathBuf::from("nested/deep/secret.key")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].path, "nested/deep/secret.key");
    assert_eq!(report.suspects[0].module, "nested/deep");
}

// ── 24. Large file with default limits samples correctly ────────

#[test]
fn large_file_samples_with_default_limits() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("large.bin");
    write_pseudorandom(&f, 0xFFFF, 1_000_000);

    let export = export_for_paths(&["large.bin"]);
    let files = vec![PathBuf::from("large.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    // Default sample should be much less than the full file
    assert!(
        (report.suspects[0].sample_bytes as usize) < 1_000_000,
        "should not read the entire file with default limits"
    );
}

// ── 25. EntropyReport with empty suspects serializes correctly ──

#[test]
fn empty_entropy_report_serialization() {
    let report = EntropyReport { suspects: vec![] };

    let json = serde_json::to_string(&report).unwrap();
    let back: EntropyReport = serde_json::from_str(&json).unwrap();

    assert!(back.suspects.is_empty());
    assert!(json.contains("suspects"));
}
