//! Determinism and threshold boundary tests for `tokmd-analysis-entropy`.
//!
//! Supplements existing BDD and unit tests with explicit determinism
//! verification and entropy classification threshold boundary tests.

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::tempdir;
use tokmd_analysis_entropy::build_entropy_report;
use tokmd_analysis_types::AnalysisLimits;
use tokmd_analysis_types::EntropyClass;
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

// ── Determinism ─────────────────────────────────────────────────

mod determinism {
    use super::*;

    #[test]
    fn given_same_input_when_scanned_twice_then_identical_results() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("data.bin");
        write_pseudorandom(&f, 0xDEAD, 2048);

        let export = export_for_paths(&["data.bin"]);
        let files = vec![PathBuf::from("data.bin")];

        let r1 =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();
        let r2 =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        assert_eq!(r1.suspects.len(), r2.suspects.len());
        for (s1, s2) in r1.suspects.iter().zip(r2.suspects.iter()) {
            assert_eq!(s1.path, s2.path);
            assert_eq!(s1.class, s2.class);
            assert!(
                (s1.entropy_bits_per_byte - s2.entropy_bits_per_byte).abs() < f32::EPSILON,
                "entropy should be identical across runs"
            );
            assert_eq!(s1.sample_bytes, s2.sample_bytes);
        }
    }

    #[test]
    fn given_multiple_files_when_scanned_twice_then_order_is_identical() {
        let dir = tempdir().unwrap();

        let lo = dir.path().join("low.bin");
        let hi = dir.path().join("high.bin");
        write_repeated(&lo, 0x00, 1024);
        write_pseudorandom(&hi, 0xBEEF, 4096);

        let export = export_for_paths(&["low.bin", "high.bin"]);
        let files = vec![PathBuf::from("low.bin"), PathBuf::from("high.bin")];

        let r1 =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();
        let r2 =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        let paths1: Vec<&str> = r1.suspects.iter().map(|s| s.path.as_str()).collect();
        let paths2: Vec<&str> = r2.suspects.iter().map(|s| s.path.as_str()).collect();
        assert_eq!(paths1, paths2, "suspect order must be deterministic");
    }

    #[test]
    fn given_files_in_different_order_when_scanned_then_sorted_output_is_same() {
        let dir = tempdir().unwrap();

        let a = dir.path().join("a.bin");
        let b = dir.path().join("b.bin");
        write_pseudorandom(&a, 0x1111, 2048);
        write_pseudorandom(&b, 0x2222, 2048);

        let export = export_for_paths(&["a.bin", "b.bin"]);
        let files_ab = vec![PathBuf::from("a.bin"), PathBuf::from("b.bin")];
        let files_ba = vec![PathBuf::from("b.bin"), PathBuf::from("a.bin")];

        let r_ab = build_entropy_report(dir.path(), &files_ab, &export, &AnalysisLimits::default())
            .unwrap();
        let r_ba = build_entropy_report(dir.path(), &files_ba, &export, &AnalysisLimits::default())
            .unwrap();

        // Output is sorted by entropy desc then path asc, so order should be same
        assert_eq!(r_ab.suspects.len(), r_ba.suspects.len());
        for (a, b) in r_ab.suspects.iter().zip(r_ba.suspects.iter()) {
            assert_eq!(a.path, b.path);
            assert_eq!(a.class, b.class);
        }
    }
}

// ── Threshold boundaries ────────────────────────────────────────

mod threshold_boundaries {
    use super::*;

    #[test]
    fn given_all_zero_bytes_then_entropy_near_zero_classified_low() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("zeros.bin");
        write_repeated(&f, 0x00, 2048);

        let export = export_for_paths(&["zeros.bin"]);
        let files = vec![PathBuf::from("zeros.bin")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        assert_eq!(report.suspects.len(), 1);
        assert_eq!(report.suspects[0].class, EntropyClass::Low);
        assert!(
            report.suspects[0].entropy_bits_per_byte < 2.0,
            "all-zero file should have entropy < 2.0, got {}",
            report.suspects[0].entropy_bits_per_byte
        );
    }

    #[test]
    fn given_pseudorandom_data_then_entropy_above_7_5_classified_high() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("rand.bin");
        write_pseudorandom(&f, 0xABCDEF, 8192);

        let export = export_for_paths(&["rand.bin"]);
        let files = vec![PathBuf::from("rand.bin")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        assert_eq!(report.suspects.len(), 1);
        assert_eq!(report.suspects[0].class, EntropyClass::High);
        assert!(
            report.suspects[0].entropy_bits_per_byte > 7.5,
            "pseudorandom should have entropy > 7.5, got {}",
            report.suspects[0].entropy_bits_per_byte
        );
    }

    #[test]
    fn given_normal_text_then_not_in_suspects_entropy_between_2_and_6_5() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("normal.txt");
        let text = "The quick brown fox jumps over the lazy dog. \
                     Hello world, this is a normal text file with moderate entropy.\n"
            .repeat(30);
        fs::write(&f, text).unwrap();

        let export = export_for_paths(&["normal.txt"]);
        let files = vec![PathBuf::from("normal.txt")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        // Normal text should have entropy in [2.0, 6.5) → classified Normal → not in suspects
        assert!(
            report.suspects.is_empty(),
            "normal text should not be a suspect"
        );
    }
}

// ── Multiple classifications in one report ──────────────────────

mod mixed_classifications {
    use super::*;

    #[test]
    fn given_low_and_high_entropy_files_then_both_classified_correctly() {
        let dir = tempdir().unwrap();

        let lo = dir.path().join("constant.bin");
        let hi = dir.path().join("random.bin");
        write_repeated(&lo, b'X', 1024);
        write_pseudorandom(&hi, 0x9999, 4096);

        let export = export_for_paths(&["constant.bin", "random.bin"]);
        let files = vec![PathBuf::from("constant.bin"), PathBuf::from("random.bin")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        assert_eq!(report.suspects.len(), 2);
        let low_finding = report
            .suspects
            .iter()
            .find(|s| s.path == "constant.bin")
            .unwrap();
        let high_finding = report
            .suspects
            .iter()
            .find(|s| s.path == "random.bin")
            .unwrap();
        assert_eq!(low_finding.class, EntropyClass::Low);
        assert_eq!(high_finding.class, EntropyClass::High);
    }

    #[test]
    fn given_normal_among_abnormal_then_normal_excluded_from_suspects() {
        let dir = tempdir().unwrap();

        let lo = dir.path().join("lo.bin");
        let hi = dir.path().join("hi.bin");
        let normal = dir.path().join("code.rs");
        write_repeated(&lo, 0x00, 1024);
        write_pseudorandom(&hi, 0x4444, 4096);
        let code = "fn main() { println!(\"hello\"); }\n".repeat(20);
        fs::write(&normal, code).unwrap();

        let export = export_for_paths(&["lo.bin", "hi.bin", "code.rs"]);
        let files = vec![
            PathBuf::from("lo.bin"),
            PathBuf::from("hi.bin"),
            PathBuf::from("code.rs"),
        ];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        // code.rs should not be a suspect
        assert!(
            report.suspects.iter().all(|s| s.path != "code.rs"),
            "normal source code should be excluded from suspects"
        );
        // lo.bin and hi.bin should both be suspects
        assert!(report.suspects.iter().any(|s| s.path == "lo.bin"));
        assert!(report.suspects.iter().any(|s| s.path == "hi.bin"));
    }
}

// ── Child row filtering ─────────────────────────────────────────

mod child_filtering {
    use super::*;

    #[test]
    fn given_child_row_for_high_entropy_file_then_module_is_unknown() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("secret.bin");
        write_pseudorandom(&f, 0xFACE, 2048);

        // Export has only a child row for this file
        let rows = vec![FileRow {
            path: "secret.bin".to_string(),
            module: "embedded".to_string(),
            lang: "Text".to_string(),
            kind: FileKind::Child,
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
        let files = vec![PathBuf::from("secret.bin")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        assert_eq!(report.suspects.len(), 1);
        assert_eq!(
            report.suspects[0].module, "(unknown)",
            "child row should not be used for module lookup"
        );
    }
}
