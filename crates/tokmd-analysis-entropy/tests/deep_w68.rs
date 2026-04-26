//! W68 deep tests for `tokmd-analysis-entropy`.
//!
//! Exercises entropy classification thresholds, sorting, budget limits,
//! module mapping, file content patterns, and determinism.

use std::fs;
use std::path::PathBuf;

use tempfile::tempdir;
use tokmd_analysis_entropy::build_entropy_report;
use tokmd_analysis_types::AnalysisLimits;
use tokmd_analysis_types::{EntropyClass, EntropyReport};
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

fn write_all_byte_values(path: &std::path::Path, repeats: usize) {
    let mut data = Vec::with_capacity(256 * repeats);
    for _ in 0..repeats {
        for b in 0u8..=255 {
            data.push(b);
        }
    }
    fs::write(path, data).unwrap();
}

// ── Classification boundary tests ───────────────────────────────

mod classification_w68 {
    use super::*;

    #[test]
    fn single_byte_repeated_is_low_entropy() {
        let dir = tempdir().unwrap();
        write_repeated(&dir.path().join("zeros.bin"), 0x00, 1024);
        let export = export_for_paths(&["zeros.bin"]);
        let files = vec![PathBuf::from("zeros.bin")];
        let r =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();
        assert_eq!(r.suspects.len(), 1);
        assert_eq!(r.suspects[0].class, EntropyClass::Low);
        assert!(r.suspects[0].entropy_bits_per_byte < 0.01);
    }

    #[test]
    fn pseudorandom_is_high_entropy() {
        let dir = tempdir().unwrap();
        write_pseudorandom(&dir.path().join("rand.bin"), 2048);
        let export = export_for_paths(&["rand.bin"]);
        let files = vec![PathBuf::from("rand.bin")];
        let r =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();
        assert_eq!(r.suspects.len(), 1);
        assert_eq!(r.suspects[0].class, EntropyClass::High);
        assert!(r.suspects[0].entropy_bits_per_byte > 7.5);
    }

    #[test]
    fn uniform_byte_distribution_is_high() {
        let dir = tempdir().unwrap();
        write_all_byte_values(&dir.path().join("uniform.bin"), 8);
        let export = export_for_paths(&["uniform.bin"]);
        let files = vec![PathBuf::from("uniform.bin")];
        let r =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();
        assert_eq!(r.suspects.len(), 1);
        assert_eq!(r.suspects[0].class, EntropyClass::High);
    }

    #[test]
    fn typical_source_code_is_normal() {
        let dir = tempdir().unwrap();
        let code = "fn main() {\n    println!(\"Hello, world!\");\n    let x = 42;\n}\n".repeat(40);
        fs::write(dir.path().join("main.rs"), &code).unwrap();
        let export = export_for_paths(&["main.rs"]);
        let files = vec![PathBuf::from("main.rs")];
        let r =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();
        // Normal entropy files are excluded from suspects
        assert!(
            r.suspects.is_empty(),
            "source code should be normal entropy"
        );
    }

    #[test]
    fn two_distinct_bytes_low_entropy() {
        let dir = tempdir().unwrap();
        let mut data = Vec::with_capacity(1024);
        for i in 0..1024 {
            data.push(if i % 2 == 0 { b'A' } else { b'B' });
        }
        fs::write(dir.path().join("ab.txt"), &data).unwrap();
        let export = export_for_paths(&["ab.txt"]);
        let files = vec![PathBuf::from("ab.txt")];
        let r =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();
        // 2 equally-distributed bytes → ~1.0 bits/byte → Low class (<2.0)
        assert_eq!(r.suspects.len(), 1);
        assert_eq!(r.suspects[0].class, EntropyClass::Low);
    }
}

// ── Sorting and truncation ──────────────────────────────────────

mod sorting_w68 {
    use super::*;

    #[test]
    fn suspects_sorted_by_entropy_desc() {
        let dir = tempdir().unwrap();
        write_repeated(&dir.path().join("low.txt"), b'A', 1024);
        write_pseudorandom(&dir.path().join("high.bin"), 1024);
        let export = export_for_paths(&["low.txt", "high.bin"]);
        let files = vec![PathBuf::from("low.txt"), PathBuf::from("high.bin")];
        let r =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();
        assert_eq!(r.suspects.len(), 2);
        assert!(r.suspects[0].entropy_bits_per_byte >= r.suspects[1].entropy_bits_per_byte);
    }

    #[test]
    fn tied_entropy_sorted_by_path_asc() {
        let dir = tempdir().unwrap();
        write_repeated(&dir.path().join("b.txt"), b'Z', 1024);
        write_repeated(&dir.path().join("a.txt"), b'Z', 1024);
        let export = export_for_paths(&["b.txt", "a.txt"]);
        let files = vec![PathBuf::from("b.txt"), PathBuf::from("a.txt")];
        let r =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();
        assert_eq!(r.suspects.len(), 2);
        assert_eq!(r.suspects[0].path, "a.txt");
        assert_eq!(r.suspects[1].path, "b.txt");
    }

    #[test]
    fn max_suspects_capped_at_50() {
        let dir = tempdir().unwrap();
        let mut paths = Vec::new();
        let mut files = Vec::new();
        for i in 0..60 {
            let name = format!("low_{i:03}.txt");
            write_repeated(&dir.path().join(&name), b'Q', 1024);
            paths.push(name.clone());
            files.push(PathBuf::from(name));
        }
        let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
        let export = export_for_paths(&path_refs);
        let r =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();
        assert!(r.suspects.len() <= 50, "got {} suspects", r.suspects.len());
    }
}

// ── Budget and limits ───────────────────────────────────────────

mod budget_w68 {
    use super::*;

    #[test]
    fn max_bytes_limits_scanning() {
        let dir = tempdir().unwrap();
        for i in 0..10 {
            write_repeated(&dir.path().join(format!("f{i}.txt")), b'X', 1024);
        }
        let paths: Vec<String> = (0..10).map(|i| format!("f{i}.txt")).collect();
        let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
        let export = export_for_paths(&path_refs);
        let files: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
        let limits = AnalysisLimits {
            max_bytes: Some(2000),
            ..Default::default()
        };
        let r = build_entropy_report(dir.path(), &files, &export, &limits).unwrap();
        assert!(
            r.suspects.len() <= 2,
            "budget should limit: got {}",
            r.suspects.len()
        );
    }

    #[test]
    fn empty_file_produces_no_suspects() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("empty.txt"), b"").unwrap();
        let export = export_for_paths(&["empty.txt"]);
        let files = vec![PathBuf::from("empty.txt")];
        let r =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();
        assert!(r.suspects.is_empty());
    }

    #[test]
    fn empty_file_list_gives_empty_report() {
        let dir = tempdir().unwrap();
        let r = build_entropy_report(
            dir.path(),
            &[],
            &export_for_paths(&[]),
            &AnalysisLimits::default(),
        )
        .unwrap();
        assert!(r.suspects.is_empty());
    }
}

// ── Module mapping ──────────────────────────────────────────────

mod module_mapping_w68 {
    use super::*;

    #[test]
    fn module_from_export_data() {
        let dir = tempdir().unwrap();
        write_repeated(&dir.path().join("low.txt"), b'A', 1024);
        let export = export_with_modules(&[("low.txt", "my_module")]);
        let files = vec![PathBuf::from("low.txt")];
        let r =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();
        assert_eq!(r.suspects[0].module, "my_module");
    }

    #[test]
    fn unmapped_file_gets_unknown_module() {
        let dir = tempdir().unwrap();
        write_repeated(&dir.path().join("orphan.txt"), b'A', 1024);
        let export = export_for_paths(&[]);
        let files = vec![PathBuf::from("orphan.txt")];
        let r =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();
        assert_eq!(r.suspects[0].module, "(unknown)");
    }
}

// ── Determinism and serde ───────────────────────────────────────

mod determinism_w68 {
    use super::*;

    #[test]
    fn report_deterministic_across_runs() {
        let dir = tempdir().unwrap();
        write_repeated(&dir.path().join("low.txt"), b'A', 1024);
        write_pseudorandom(&dir.path().join("high.bin"), 1024);
        let export = export_for_paths(&["low.txt", "high.bin"]);
        let files = vec![PathBuf::from("low.txt"), PathBuf::from("high.bin")];
        let r1 =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();
        let r2 =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();
        assert_eq!(
            serde_json::to_string(&r1).unwrap(),
            serde_json::to_string(&r2).unwrap(),
        );
    }

    #[test]
    fn serde_roundtrip_preserves_all_fields() {
        let dir = tempdir().unwrap();
        write_repeated(&dir.path().join("low.txt"), b'A', 1024);
        let export = export_for_paths(&["low.txt"]);
        let files = vec![PathBuf::from("low.txt")];
        let r =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();
        let json = serde_json::to_string(&r).unwrap();
        let rt: EntropyReport = serde_json::from_str(&json).unwrap();
        assert_eq!(rt.suspects.len(), r.suspects.len());
        assert_eq!(rt.suspects[0].path, r.suspects[0].path);
        assert_eq!(rt.suspects[0].class, r.suspects[0].class);
        assert_eq!(rt.suspects[0].sample_bytes, r.suspects[0].sample_bytes);
    }

    #[test]
    fn sample_bytes_field_populated() {
        let dir = tempdir().unwrap();
        write_repeated(&dir.path().join("low.txt"), b'A', 512);
        let export = export_for_paths(&["low.txt"]);
        let files = vec![PathBuf::from("low.txt")];
        let r =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();
        assert_eq!(r.suspects.len(), 1);
        assert!(r.suspects[0].sample_bytes > 0);
    }
}
