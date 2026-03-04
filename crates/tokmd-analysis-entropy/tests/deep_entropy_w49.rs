//! Wave-49 deep tests for entropy profiling.
//!
//! Covers classification boundaries, sorting, budget limits,
//! module mapping, serde roundtrips, and property-based tests.

use std::fs;
use std::path::PathBuf;

use tempfile::tempdir;
use tokmd_analysis_entropy::build_entropy_report;
use tokmd_analysis_types::{EntropyClass, EntropyReport};
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

fn export_with_modules(entries: &[(&str, &str)]) -> ExportData {
    let rows = entries
        .iter()
        .map(|(p, m)| FileRow {
            path: (*p).to_string(),
            module: (*m).to_string(),
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

fn write_repeated(path: &std::path::Path, byte: u8, len: usize) {
    fs::write(path, vec![byte; len]).unwrap();
}

fn write_pseudorandom(path: &std::path::Path, len: usize) {
    let mut data = Vec::with_capacity(len);
    let mut x = 0x12345678u32;
    for _ in 0..len {
        x = x.wrapping_mul(1664525).wrapping_add(1013904223);
        data.push((x & 0xFF) as u8);
    }
    fs::write(path, data).unwrap();
}

// ── 1. Empty file list → empty report ───────────────────────────

#[test]
fn empty_file_list_empty_report() {
    let dir = tempdir().unwrap();
    let export = export_for_paths(&[]);
    let report =
        build_entropy_report(dir.path(), &[], &export, &AnalysisLimits::default()).unwrap();
    assert!(report.suspects.is_empty());
}

// ── 2. Low entropy file detected ────────────────────────────────

#[test]
fn low_entropy_detected() {
    let dir = tempdir().unwrap();
    write_repeated(&dir.path().join("low.txt"), b'A', 1024);
    let export = export_for_paths(&["low.txt"]);
    let files = vec![PathBuf::from("low.txt")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();
    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].class, EntropyClass::Low);
    assert!(report.suspects[0].entropy_bits_per_byte < 2.0);
}

// ── 3. High entropy file detected ───────────────────────────────

#[test]
fn high_entropy_detected() {
    let dir = tempdir().unwrap();
    write_pseudorandom(&dir.path().join("high.bin"), 1024);
    let export = export_for_paths(&["high.bin"]);
    let files = vec![PathBuf::from("high.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();
    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].class, EntropyClass::High);
    assert!(report.suspects[0].entropy_bits_per_byte > 7.5);
}

// ── 4. Normal entropy excluded from suspects ────────────────────

#[test]
fn normal_entropy_excluded() {
    let dir = tempdir().unwrap();
    // Typical source code has entropy around 4-5 bits/byte
    let text = "fn main() {\n    println!(\"Hello, world!\");\n}\n".repeat(30);
    fs::write(dir.path().join("main.rs"), &text).unwrap();
    let export = export_for_paths(&["main.rs"]);
    let files = vec![PathBuf::from("main.rs")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();
    // Normal entropy files should NOT appear in suspects
    for s in &report.suspects {
        assert_ne!(s.class, EntropyClass::Normal);
    }
}

// ── 5. Sorting: entropy desc, path asc ──────────────────────────

#[test]
fn sorting_entropy_desc_path_asc() {
    let dir = tempdir().unwrap();
    // Two low-entropy files (same class) with different paths
    write_repeated(&dir.path().join("b_low.txt"), b'B', 1024);
    write_repeated(&dir.path().join("a_low.txt"), b'A', 1024);
    let export = export_for_paths(&["b_low.txt", "a_low.txt"]);
    let files = vec![PathBuf::from("b_low.txt"), PathBuf::from("a_low.txt")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();
    for pair in report.suspects.windows(2) {
        if (pair[0].entropy_bits_per_byte - pair[1].entropy_bits_per_byte).abs() < f32::EPSILON {
            assert!(
                pair[0].path <= pair[1].path,
                "tied entropy should sort by path asc: {} vs {}",
                pair[0].path,
                pair[1].path
            );
        } else {
            assert!(
                pair[0].entropy_bits_per_byte >= pair[1].entropy_bits_per_byte,
                "should sort by entropy desc"
            );
        }
    }
}

// ── 6. MAX_SUSPECTS cap at 50 ───────────────────────────────────

#[test]
fn max_suspects_capped_at_50() {
    let dir = tempdir().unwrap();
    let mut paths = Vec::new();
    let mut files = Vec::new();
    for i in 0..60 {
        let name = format!("low_{i:03}.txt");
        write_repeated(&dir.path().join(&name), b'X', 1024);
        paths.push(name.clone());
        files.push(PathBuf::from(name));
    }
    let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
    let export = export_for_paths(&path_refs);
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();
    assert!(
        report.suspects.len() <= 50,
        "suspects should be capped at 50, got {}",
        report.suspects.len()
    );
}

// ── 7. max_bytes budget stops scanning ──────────────────────────

#[test]
fn max_bytes_budget_stops_scanning() {
    let dir = tempdir().unwrap();
    // Create 10 files of 1024 bytes each, set budget to 3000
    for i in 0..10 {
        write_repeated(&dir.path().join(format!("f{i}.txt")), b'Z', 1024);
    }
    let paths: Vec<String> = (0..10).map(|i| format!("f{i}.txt")).collect();
    let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
    let export = export_for_paths(&path_refs);
    let files: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();

    let limits = AnalysisLimits {
        max_bytes: Some(3000),
        ..Default::default()
    };
    let report = build_entropy_report(dir.path(), &files, &export, &limits).unwrap();
    // With budget of 3000 and files of 1024 each, at most 3 files should be scanned
    assert!(
        report.suspects.len() <= 3,
        "budget should limit scanning: got {} suspects",
        report.suspects.len()
    );
}

// ── 8. Module mapping from export data ──────────────────────────

#[test]
fn module_mapping_from_export() {
    let dir = tempdir().unwrap();
    write_repeated(&dir.path().join("low.txt"), b'A', 1024);
    let export = export_with_modules(&[("low.txt", "my-module")]);
    let files = vec![PathBuf::from("low.txt")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();
    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].module, "my-module");
}

// ── 9. Unknown path maps to "(unknown)" module ──────────────────

#[test]
fn unknown_path_maps_to_unknown_module() {
    let dir = tempdir().unwrap();
    write_repeated(&dir.path().join("orphan.txt"), b'A', 1024);
    // Export has no matching row for "orphan.txt"
    let export = export_for_paths(&[]);
    let files = vec![PathBuf::from("orphan.txt")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();
    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].module, "(unknown)");
}

// ── 10. Serde roundtrip preserves all fields ────────────────────

#[test]
fn serde_roundtrip_preserves_all_fields() {
    let dir = tempdir().unwrap();
    write_repeated(&dir.path().join("low.txt"), b'A', 1024);
    write_pseudorandom(&dir.path().join("high.bin"), 1024);
    let export = export_for_paths(&["low.txt", "high.bin"]);
    let files = vec![PathBuf::from("low.txt"), PathBuf::from("high.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    let json = serde_json::to_string(&report).unwrap();
    let deser: EntropyReport = serde_json::from_str(&json).unwrap();

    assert_eq!(deser.suspects.len(), report.suspects.len());
    for (orig, rt) in report.suspects.iter().zip(deser.suspects.iter()) {
        assert_eq!(orig.path, rt.path);
        assert_eq!(orig.module, rt.module);
        assert_eq!(orig.class, rt.class);
        assert_eq!(orig.sample_bytes, rt.sample_bytes);
        assert!((orig.entropy_bits_per_byte - rt.entropy_bits_per_byte).abs() < f32::EPSILON);
    }
}

// ── 11. Empty file produces no suspects ─────────────────────────

#[test]
fn empty_file_no_suspects() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("empty.txt"), b"").unwrap();
    let export = export_for_paths(&["empty.txt"]);
    let files = vec![PathBuf::from("empty.txt")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();
    assert!(
        report.suspects.is_empty(),
        "empty file should produce no suspects"
    );
}

// ── Proptest ────────────────────────────────────────────────────

mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn entropy_class_serde_roundtrip(class_idx in 0u8..4) {
            let class = match class_idx {
                0 => EntropyClass::Low,
                1 => EntropyClass::Normal,
                2 => EntropyClass::Suspicious,
                _ => EntropyClass::High,
            };
            let json = serde_json::to_string(&class).unwrap();
            let rt: EntropyClass = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(class, rt);
        }
    }
}
