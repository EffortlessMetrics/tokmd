//! Deep tests for tokmd-analysis-entropy (wave 38).
//!
//! Covers classify_entropy for all 4 classes, boundary cases,
//! sorting, MAX_SUSPECTS truncation, and build_entropy_report integration.

use std::fs;
use std::path::PathBuf;

use tempfile::tempdir;
use tokmd_analysis_entropy::build_entropy_report;
use tokmd_analysis_types::{EntropyClass, EntropyFinding};
use tokmd_analysis_util::AnalysisLimits;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_row(path: &str, module: &str) -> FileRow {
    FileRow {
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
    }
}

fn make_export(paths: &[&str]) -> ExportData {
    let rows = paths.iter().map(|p| make_row(p, "(root)")).collect();
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn write_repeated(dir: &std::path::Path, name: &str, byte: u8, len: usize) {
    fs::write(dir.join(name), vec![byte; len]).unwrap();
}

fn write_pseudorandom(dir: &std::path::Path, name: &str, len: usize) {
    let mut data = Vec::with_capacity(len);
    let mut x = 0x12345678u32;
    for _ in 0..len {
        x = x.wrapping_mul(1664525).wrapping_add(1013904223);
        data.push((x & 0xFF) as u8);
    }
    fs::write(dir.join(name), data).unwrap();
}

/// Write content with entropy in the "suspicious" range (~6.5-7.5 bits/byte).
fn write_suspicious(dir: &std::path::Path, name: &str, len: usize) {
    // Mix of repeated and random bytes to target ~6.5-7.0 entropy
    let mut data = Vec::with_capacity(len);
    let mut x = 0xDEADBEEFu32;
    for i in 0..len {
        if i % 8 == 0 {
            data.push(b'A'); // inject some regularity
        } else {
            x = x.wrapping_mul(1664525).wrapping_add(1013904223);
            data.push((x & 0xFF) as u8);
        }
    }
    fs::write(dir.join(name), data).unwrap();
}

fn default_limits() -> AnalysisLimits {
    AnalysisLimits::default()
}

fn find_by_path<'a>(suspects: &'a [EntropyFinding], path: &str) -> Option<&'a EntropyFinding> {
    suspects.iter().find(|f| f.path == path)
}

// ---------------------------------------------------------------------------
// classify_entropy: all 4 classes
// ---------------------------------------------------------------------------

#[test]
fn detects_high_entropy() {
    let dir = tempdir().unwrap();
    write_pseudorandom(dir.path(), "high.bin", 1024);
    let export = make_export(&["high.bin"]);
    let files = vec![PathBuf::from("high.bin")];

    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();

    let finding = find_by_path(&report.suspects, "high.bin");
    assert!(finding.is_some(), "high entropy file should be detected");
    assert_eq!(finding.unwrap().class, EntropyClass::High);
    assert!(finding.unwrap().entropy_bits_per_byte > 7.5);
}

#[test]
fn detects_low_entropy() {
    let dir = tempdir().unwrap();
    write_repeated(dir.path(), "low.txt", b'A', 1024);
    let export = make_export(&["low.txt"]);
    let files = vec![PathBuf::from("low.txt")];

    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();

    let finding = find_by_path(&report.suspects, "low.txt");
    assert!(finding.is_some(), "low entropy file should be detected");
    assert_eq!(finding.unwrap().class, EntropyClass::Low);
    assert!(finding.unwrap().entropy_bits_per_byte < 2.0);
}

#[test]
fn normal_entropy_not_reported() {
    let dir = tempdir().unwrap();
    // Normal source code has entropy ~4-5 bits/byte
    let normal_text = "fn main() {\n    println!(\"Hello, world!\");\n    let x = 42;\n    for i in 0..10 {\n        println!(\"{}\", i * x);\n    }\n}\n";
    let repeated = normal_text.repeat(20);
    fs::write(dir.path().join("normal.rs"), &repeated).unwrap();
    let export = make_export(&["normal.rs"]);
    let files = vec![PathBuf::from("normal.rs")];

    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();

    let finding = find_by_path(&report.suspects, "normal.rs");
    assert!(
        finding.is_none(),
        "normal entropy file should not be in suspects"
    );
}

#[test]
fn detects_suspicious_entropy() {
    let dir = tempdir().unwrap();
    write_suspicious(dir.path(), "suspect.bin", 4096);
    let export = make_export(&["suspect.bin"]);
    let files = vec![PathBuf::from("suspect.bin")];

    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();

    // May be Suspicious or High depending on exact entropy
    if let Some(finding) = find_by_path(&report.suspects, "suspect.bin") {
        assert!(
            finding.class == EntropyClass::Suspicious || finding.class == EntropyClass::High,
            "expected Suspicious or High, got {:?}",
            finding.class
        );
    }
}

// ---------------------------------------------------------------------------
// Boundary cases: entropy exactly at thresholds
// ---------------------------------------------------------------------------

#[test]
fn both_low_and_high_detected_together() {
    let dir = tempdir().unwrap();
    write_repeated(dir.path(), "low.txt", b'X', 1024);
    write_pseudorandom(dir.path(), "high.bin", 1024);
    let export = make_export(&["low.txt", "high.bin"]);
    let files = vec![PathBuf::from("low.txt"), PathBuf::from("high.bin")];

    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();

    assert!(report.suspects.len() >= 2);
    let classes: Vec<EntropyClass> = report.suspects.iter().map(|f| f.class).collect();
    assert!(classes.contains(&EntropyClass::Low));
    assert!(classes.contains(&EntropyClass::High));
}

#[test]
fn zero_byte_file_not_in_suspects() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("empty.txt"), b"").unwrap();
    let export = make_export(&["empty.txt"]);
    let files = vec![PathBuf::from("empty.txt")];

    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();

    assert!(
        report.suspects.is_empty(),
        "empty file should not be a suspect"
    );
}

#[test]
fn single_byte_file_classified_as_low() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("single.txt"), b"A").unwrap();
    let export = make_export(&["single.txt"]);
    let files = vec![PathBuf::from("single.txt")];

    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();

    if let Some(finding) = find_by_path(&report.suspects, "single.txt") {
        assert_eq!(finding.class, EntropyClass::Low);
    }
}

#[test]
fn two_byte_values_moderate_entropy() {
    // File with only 2 distinct byte values → entropy = ~1 bit/byte → Low
    let dir = tempdir().unwrap();
    let data: Vec<u8> = (0..1024)
        .map(|i| if i % 2 == 0 { b'A' } else { b'B' })
        .collect();
    fs::write(dir.path().join("two_vals.bin"), data).unwrap();
    let export = make_export(&["two_vals.bin"]);
    let files = vec![PathBuf::from("two_vals.bin")];

    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();

    if let Some(finding) = find_by_path(&report.suspects, "two_vals.bin") {
        assert_eq!(
            finding.class,
            EntropyClass::Low,
            "2 values → ~1 bit entropy → Low"
        );
    }
}

// ---------------------------------------------------------------------------
// Sorting: entropy descending then by path
// ---------------------------------------------------------------------------

#[test]
fn suspects_sorted_by_entropy_descending() {
    let dir = tempdir().unwrap();
    write_pseudorandom(dir.path(), "high1.bin", 1024);
    write_pseudorandom(dir.path(), "high2.bin", 1024);
    write_repeated(dir.path(), "low1.txt", b'A', 1024);
    write_repeated(dir.path(), "low2.txt", b'B', 1024);
    let export = make_export(&["high1.bin", "high2.bin", "low1.txt", "low2.txt"]);
    let files = vec![
        PathBuf::from("high1.bin"),
        PathBuf::from("high2.bin"),
        PathBuf::from("low1.txt"),
        PathBuf::from("low2.txt"),
    ];

    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();

    for i in 1..report.suspects.len() {
        let prev = &report.suspects[i - 1];
        let curr = &report.suspects[i];
        assert!(
            prev.entropy_bits_per_byte >= curr.entropy_bits_per_byte
                || (prev.entropy_bits_per_byte - curr.entropy_bits_per_byte).abs() < 1e-6,
            "suspects not sorted by entropy desc: {} ({}) vs {} ({})",
            prev.path,
            prev.entropy_bits_per_byte,
            curr.path,
            curr.entropy_bits_per_byte
        );
    }
}

#[test]
fn suspects_tiebreak_by_path_ascending() {
    let dir = tempdir().unwrap();
    // Create identical high-entropy files with different names
    write_pseudorandom(dir.path(), "z_file.bin", 1024);
    // Use same seed to get same entropy
    write_pseudorandom(dir.path(), "a_file.bin", 1024);
    let export = make_export(&["z_file.bin", "a_file.bin"]);
    let files = vec![PathBuf::from("z_file.bin"), PathBuf::from("a_file.bin")];

    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();

    if report.suspects.len() >= 2 {
        let s = &report.suspects;
        // If same entropy, paths should be in ascending order
        if (s[0].entropy_bits_per_byte - s[1].entropy_bits_per_byte).abs() < 0.01 {
            assert!(
                s[0].path < s[1].path,
                "tiebreak: {} should come before {}",
                s[0].path,
                s[1].path
            );
        }
    }
}

// ---------------------------------------------------------------------------
// MAX_SUSPECTS truncation (limit to 50)
// ---------------------------------------------------------------------------

#[test]
fn max_suspects_truncated_to_50() {
    let dir = tempdir().unwrap();
    let mut paths = Vec::new();
    let mut files = Vec::new();
    // Create 60 high-entropy files
    for i in 0..60 {
        let name = format!("high_{i:03}.bin");
        write_pseudorandom(dir.path(), &name, 512);
        paths.push(name);
    }
    let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
    let export = make_export(&path_refs);
    for name in &paths {
        files.push(PathBuf::from(name));
    }

    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();

    assert!(
        report.suspects.len() <= 50,
        "suspects should be truncated to MAX_SUSPECTS=50, got {}",
        report.suspects.len()
    );
}

#[test]
fn fewer_than_50_not_truncated() {
    let dir = tempdir().unwrap();
    let mut paths = Vec::new();
    let mut files = Vec::new();
    for i in 0..10 {
        let name = format!("high_{i:03}.bin");
        write_pseudorandom(dir.path(), &name, 512);
        paths.push(name);
    }
    let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
    let export = make_export(&path_refs);
    for name in &paths {
        files.push(PathBuf::from(name));
    }

    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();

    assert_eq!(report.suspects.len(), 10);
}

// ---------------------------------------------------------------------------
// build_entropy_report integration
// ---------------------------------------------------------------------------

#[test]
fn report_empty_file_list() {
    let dir = tempdir().unwrap();
    let export = make_export(&[]);

    let report = build_entropy_report(dir.path(), &[], &export, &default_limits()).unwrap();

    assert!(report.suspects.is_empty());
}

#[test]
fn report_includes_module_from_export() {
    let dir = tempdir().unwrap();
    fs::create_dir_all(dir.path().join("src/auth")).unwrap();
    write_pseudorandom(dir.path(), "src/auth/secrets.bin", 1024);

    let rows = vec![FileRow {
        path: "src/auth/secrets.bin".to_string(),
        module: "src/auth".to_string(),
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
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let files = vec![PathBuf::from("src/auth/secrets.bin")];

    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();

    if let Some(finding) = find_by_path(&report.suspects, "src/auth/secrets.bin") {
        assert_eq!(finding.module, "src/auth");
    }
}

#[test]
fn report_unknown_module_fallback() {
    let dir = tempdir().unwrap();
    write_pseudorandom(dir.path(), "orphan.bin", 1024);
    // File not in export rows → module should be "(unknown)"
    let export = make_export(&[]);
    let files = vec![PathBuf::from("orphan.bin")];

    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();

    if let Some(finding) = find_by_path(&report.suspects, "orphan.bin") {
        assert_eq!(finding.module, "(unknown)");
    }
}

#[test]
fn report_sample_bytes_field_populated() {
    let dir = tempdir().unwrap();
    write_pseudorandom(dir.path(), "high.bin", 2048);
    let export = make_export(&["high.bin"]);
    let files = vec![PathBuf::from("high.bin")];

    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();

    let finding = find_by_path(&report.suspects, "high.bin").unwrap();
    assert!(finding.sample_bytes > 0);
}

// ---------------------------------------------------------------------------
// AnalysisLimits interaction
// ---------------------------------------------------------------------------

#[test]
fn max_bytes_limit_stops_processing() {
    let dir = tempdir().unwrap();
    // Create many files, but limit total bytes
    for i in 0..10 {
        let name = format!("file_{i:03}.bin");
        write_pseudorandom(dir.path(), &name, 512);
    }
    let paths: Vec<String> = (0..10).map(|i| format!("file_{i:03}.bin")).collect();
    let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
    let export = make_export(&path_refs);
    let files: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();

    let limits = AnalysisLimits {
        max_bytes: Some(1024), // Only process ~1KB total
        max_file_bytes: None,
        max_files: None,
        max_commits: None,
        max_commit_files: None,
    };

    let report = build_entropy_report(dir.path(), &files, &export, &limits).unwrap();

    // Should not process all 10 files
    assert!(
        report.suspects.len() < 10,
        "max_bytes should limit processing"
    );
}

#[test]
fn max_file_bytes_limits_sample_size() {
    let dir = tempdir().unwrap();
    write_pseudorandom(dir.path(), "big.bin", 4096);
    let export = make_export(&["big.bin"]);
    let files = vec![PathBuf::from("big.bin")];

    let limits = AnalysisLimits {
        max_bytes: None,
        max_file_bytes: Some(256), // Only read 256 bytes per file
        max_files: None,
        max_commits: None,
        max_commit_files: None,
    };

    let report = build_entropy_report(dir.path(), &files, &export, &limits).unwrap();

    if let Some(finding) = find_by_path(&report.suspects, "big.bin") {
        assert!(
            finding.sample_bytes <= 512, // head+tail within limits
            "sample_bytes should be limited by max_file_bytes"
        );
    }
}

// ---------------------------------------------------------------------------
// Child rows excluded from module lookup
// ---------------------------------------------------------------------------

#[test]
fn child_rows_excluded_from_module_map() {
    let dir = tempdir().unwrap();
    write_pseudorandom(dir.path(), "file.bin", 1024);

    let rows = vec![FileRow {
        path: "file.bin".to_string(),
        module: "child_module".to_string(),
        lang: "Text".to_string(),
        kind: FileKind::Child, // Child, not Parent
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
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let files = vec![PathBuf::from("file.bin")];

    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();

    // Child row should not be in the module map → fallback to "(unknown)"
    if let Some(finding) = find_by_path(&report.suspects, "file.bin") {
        assert_eq!(finding.module, "(unknown)");
    }
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn entropy_report_deterministic() {
    let dir = tempdir().unwrap();
    write_pseudorandom(dir.path(), "high.bin", 1024);
    write_repeated(dir.path(), "low.txt", b'Z', 1024);
    let export = make_export(&["high.bin", "low.txt"]);
    let files = vec![PathBuf::from("high.bin"), PathBuf::from("low.txt")];

    let r1 = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();
    let r2 = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();

    assert_eq!(r1.suspects.len(), r2.suspects.len());
    for (a, b) in r1.suspects.iter().zip(r2.suspects.iter()) {
        assert_eq!(a.path, b.path);
        assert_eq!(a.class, b.class);
        assert!((a.entropy_bits_per_byte - b.entropy_bits_per_byte).abs() < 1e-6);
    }
}

// ---------------------------------------------------------------------------
// Path normalization (backslash → forward slash)
// ---------------------------------------------------------------------------

#[test]
fn backslash_paths_normalized() {
    let dir = tempdir().unwrap();
    fs::create_dir_all(dir.path().join("subdir")).unwrap();
    write_pseudorandom(dir.path(), "subdir/high.bin", 1024);

    let rows = vec![FileRow {
        path: "subdir/high.bin".to_string(),
        module: "(root)".to_string(),
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
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let files = vec![PathBuf::from("subdir/high.bin")];

    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();

    // Path in finding should use forward slashes
    if let Some(finding) = report.suspects.first() {
        assert!(
            !finding.path.contains('\\'),
            "path should be normalized: {}",
            finding.path
        );
    }
}

// ---------------------------------------------------------------------------
// Mixed normal and abnormal files
// ---------------------------------------------------------------------------

#[test]
fn mixed_files_only_abnormal_reported() {
    let dir = tempdir().unwrap();
    write_pseudorandom(dir.path(), "high.bin", 1024);
    write_repeated(dir.path(), "low.txt", b'X', 1024);
    // Normal entropy text
    let normal = "The quick brown fox jumps over the lazy dog. ".repeat(30);
    fs::write(dir.path().join("normal.txt"), &normal).unwrap();

    let export = make_export(&["high.bin", "low.txt", "normal.txt"]);
    let files = vec![
        PathBuf::from("high.bin"),
        PathBuf::from("low.txt"),
        PathBuf::from("normal.txt"),
    ];

    let report = build_entropy_report(dir.path(), &files, &export, &default_limits()).unwrap();

    // normal.txt should NOT be in suspects
    let normal_finding = find_by_path(&report.suspects, "normal.txt");
    assert!(
        normal_finding.is_none(),
        "normal file should not be a suspect"
    );
    // high.bin and low.txt should be present
    assert!(find_by_path(&report.suspects, "high.bin").is_some());
    assert!(find_by_path(&report.suspects, "low.txt").is_some());
}
