//! Property-based tests for entropy detection invariants.

use std::fs;
use std::path::PathBuf;

use proptest::prelude::*;
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

// ── Strategies ──────────────────────────────────────────────────

/// Arbitrary byte content of varying length.
fn arb_file_content() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 1..=2048)
}

/// Content with a single repeated byte (always low entropy).
fn low_entropy_content() -> impl Strategy<Value = Vec<u8>> {
    (any::<u8>(), 16..=1024usize).prop_map(|(byte, len)| vec![byte; len])
}

// ── Properties ──────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn suspects_never_contain_normal_class(data in arb_file_content()) {
        let dir = tempdir().unwrap();
        let f = dir.path().join("test.bin");
        fs::write(&f, &data).unwrap();

        let export = export_for_paths(&["test.bin"]);
        let files = vec![PathBuf::from("test.bin")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        for finding in &report.suspects {
            prop_assert_ne!(
                finding.class,
                EntropyClass::Normal,
                "Normal-class findings should never appear in suspects"
            );
        }
    }

    #[test]
    fn entropy_values_in_valid_range(data in arb_file_content()) {
        let dir = tempdir().unwrap();
        let f = dir.path().join("test.bin");
        fs::write(&f, &data).unwrap();

        let export = export_for_paths(&["test.bin"]);
        let files = vec![PathBuf::from("test.bin")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        for finding in &report.suspects {
            prop_assert!(
                finding.entropy_bits_per_byte >= 0.0 && finding.entropy_bits_per_byte <= 8.0,
                "entropy should be in [0, 8], got {}",
                finding.entropy_bits_per_byte
            );
        }
    }

    #[test]
    fn sample_bytes_always_positive(data in arb_file_content()) {
        let dir = tempdir().unwrap();
        let f = dir.path().join("test.bin");
        fs::write(&f, &data).unwrap();

        let export = export_for_paths(&["test.bin"]);
        let files = vec![PathBuf::from("test.bin")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        for finding in &report.suspects {
            prop_assert!(
                finding.sample_bytes > 0,
                "sample_bytes should be positive"
            );
        }
    }

    #[test]
    fn suspects_sorted_by_entropy_descending(data1 in arb_file_content(), data2 in arb_file_content()) {
        let dir = tempdir().unwrap();
        let f1 = dir.path().join("a.bin");
        let f2 = dir.path().join("b.bin");
        fs::write(&f1, &data1).unwrap();
        fs::write(&f2, &data2).unwrap();

        let export = export_for_paths(&["a.bin", "b.bin"]);
        let files = vec![PathBuf::from("a.bin"), PathBuf::from("b.bin")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        for window in report.suspects.windows(2) {
            prop_assert!(
                window[0].entropy_bits_per_byte >= window[1].entropy_bits_per_byte,
                "suspects should be sorted descending: {} >= {}",
                window[0].entropy_bits_per_byte,
                window[1].entropy_bits_per_byte
            );
        }
    }

    #[test]
    fn single_byte_value_always_low(content in low_entropy_content()) {
        let dir = tempdir().unwrap();
        let f = dir.path().join("mono.bin");
        fs::write(&f, &content).unwrap();

        let export = export_for_paths(&["mono.bin"]);
        let files = vec![PathBuf::from("mono.bin")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        prop_assert_eq!(report.suspects.len(), 1, "single repeated byte → 1 suspect");
        prop_assert_eq!(
            report.suspects[0].class,
            EntropyClass::Low,
            "single repeated byte should always be Low"
        );
    }

    #[test]
    fn suspect_count_bounded(data in prop::collection::vec(arb_file_content(), 1..=5)) {
        let dir = tempdir().unwrap();
        let mut paths = Vec::new();
        let mut path_strs = Vec::new();

        for (i, content) in data.iter().enumerate() {
            let name = format!("f{i}.bin");
            let f = dir.path().join(&name);
            fs::write(&f, content).unwrap();
            paths.push(PathBuf::from(&name));
            path_strs.push(name);
        }

        let str_refs: Vec<&str> = path_strs.iter().map(|s| s.as_str()).collect();
        let export = export_for_paths(&str_refs);
        let report =
            build_entropy_report(dir.path(), &paths, &export, &AnalysisLimits::default()).unwrap();

        prop_assert!(
            report.suspects.len() <= 50,
            "suspects should never exceed MAX_SUSPECTS (50)"
        );
    }

    #[test]
    fn paths_never_contain_backslashes(data in arb_file_content()) {
        let dir = tempdir().unwrap();
        let f = dir.path().join("test.bin");
        fs::write(&f, &data).unwrap();

        let export = export_for_paths(&["test.bin"]);
        let files = vec![PathBuf::from("test.bin")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        for finding in &report.suspects {
            prop_assert!(
                !finding.path.contains('\\'),
                "output paths should use forward slashes: {}",
                finding.path
            );
        }
    }
}
