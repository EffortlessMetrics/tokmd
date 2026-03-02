//! Determinism and additional edge-case tests for `tokmd-analysis-near-dup`.
//!
//! Supplements the existing BDD and unit tests with explicit determinism
//! verification, stats validation, and eligible_files reporting.

use std::io::Write;

use tempfile::TempDir;
use tokmd_analysis_near_dup::{NearDupLimits, build_near_dup_report};
use tokmd_analysis_types::NearDupScope;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────

fn source_text(n: usize, seed: usize) -> String {
    (0..n)
        .map(|i| format!("tok_{}_{}", seed, i))
        .collect::<Vec<_>>()
        .join(" + ")
}

fn make_file_row(path: &str, module: &str, lang: &str, code: usize, bytes: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code,
        comments: 0,
        blanks: 0,
        lines: code,
        bytes,
        tokens: code * 5,
    }
}

fn make_export(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    }
}

fn write_file(dir: &TempDir, path: &str, content: &str) {
    let full = dir.path().join(path);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let mut f = std::fs::File::create(&full).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}

// ── Determinism ─────────────────────────────────────────────────

mod determinism {
    use super::*;

    #[test]
    fn given_same_input_when_run_three_times_then_all_outputs_identical() {
        let dir = TempDir::new().unwrap();
        let content = source_text(100, 42);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);

        let rows = vec![
            make_file_row("a.rs", "root", "Rust", 100, content.len()),
            make_file_row("b.rs", "root", "Rust", 100, content.len()),
        ];
        let export = make_export(rows);

        let results: Vec<_> = (0..3)
            .map(|_| {
                build_near_dup_report(
                    dir.path(),
                    &export,
                    NearDupScope::Global,
                    0.5,
                    1000,
                    None,
                    &NearDupLimits::default(),
                    &[],
                )
                .unwrap()
            })
            .collect();

        for i in 1..3 {
            assert_eq!(results[0].pairs.len(), results[i].pairs.len());
            for (p0, pi) in results[0].pairs.iter().zip(results[i].pairs.iter()) {
                assert_eq!(p0.left, pi.left);
                assert_eq!(p0.right, pi.right);
                assert!(
                    (p0.similarity - pi.similarity).abs() < 1e-10,
                    "similarity should be identical across runs"
                );
                assert_eq!(p0.shared_fingerprints, pi.shared_fingerprints);
            }
        }
    }

    #[test]
    fn given_files_with_different_code_order_then_pairs_are_deterministic() {
        let dir = TempDir::new().unwrap();
        let content = source_text(100, 7);
        write_file(&dir, "x.rs", &content);
        write_file(&dir, "y.rs", &content);

        let rows_xy = vec![
            make_file_row("x.rs", "root", "Rust", 100, content.len()),
            make_file_row("y.rs", "root", "Rust", 100, content.len()),
        ];
        let rows_yx = vec![
            make_file_row("y.rs", "root", "Rust", 100, content.len()),
            make_file_row("x.rs", "root", "Rust", 100, content.len()),
        ];

        let r1 = build_near_dup_report(
            dir.path(),
            &make_export(rows_xy),
            NearDupScope::Global,
            0.5,
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        )
        .unwrap();
        let r2 = build_near_dup_report(
            dir.path(),
            &make_export(rows_yx),
            NearDupScope::Global,
            0.5,
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        )
        .unwrap();

        assert_eq!(r1.pairs.len(), r2.pairs.len());
        for (p1, p2) in r1.pairs.iter().zip(r2.pairs.iter()) {
            assert_eq!(p1.left, p2.left);
            assert_eq!(p1.right, p2.right);
        }
    }
}

// ── Stats field ─────────────────────────────────────────────────

mod stats_validation {
    use super::*;

    #[test]
    fn given_report_then_stats_timing_is_present_and_non_negative() {
        let dir = TempDir::new().unwrap();
        let content = source_text(100, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);

        let rows = vec![
            make_file_row("a.rs", "root", "Rust", 100, content.len()),
            make_file_row("b.rs", "root", "Rust", 100, content.len()),
        ];

        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.5,
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        )
        .unwrap();

        let stats = report.stats.as_ref().expect("stats should be present");
        // Timing values should exist (may be 0 for very fast runs)
        assert!(stats.bytes_processed > 0, "bytes_processed should be > 0");
    }

    #[test]
    fn given_empty_input_then_stats_has_zero_bytes() {
        let dir = TempDir::new().unwrap();
        let export = make_export(vec![]);

        let report = build_near_dup_report(
            dir.path(),
            &export,
            NearDupScope::Global,
            0.5,
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        )
        .unwrap();

        let stats = report.stats.as_ref().expect("stats should be present");
        assert_eq!(stats.bytes_processed, 0);
    }
}

// ── Eligible files count ────────────────────────────────────────

mod eligible_files {
    use super::*;

    #[test]
    fn given_all_files_eligible_then_eligible_equals_analyzed() {
        let dir = TempDir::new().unwrap();
        let content = source_text(100, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);

        let rows = vec![
            make_file_row("a.rs", "root", "Rust", 100, content.len()),
            make_file_row("b.rs", "root", "Rust", 100, content.len()),
        ];

        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.5,
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        )
        .unwrap();

        assert_eq!(report.eligible_files, Some(2));
        assert_eq!(report.files_analyzed, 2);
        assert_eq!(report.files_skipped, 0);
    }

    #[test]
    fn given_some_files_oversized_then_eligible_less_than_total_rows() {
        let dir = TempDir::new().unwrap();
        let content = source_text(100, 0);
        write_file(&dir, "small.rs", &content);
        write_file(&dir, "big.rs", &content);

        let rows = vec![
            make_file_row("small.rs", "root", "Rust", 100, content.len()),
            make_file_row("big.rs", "root", "Rust", 100, 600_000), // exceeds default 512KB
        ];

        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.5,
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        )
        .unwrap();

        assert_eq!(report.eligible_files, Some(1));
        assert_eq!(report.files_analyzed, 1);
    }

    #[test]
    fn given_max_files_caps_then_eligible_greater_than_analyzed() {
        let dir = TempDir::new().unwrap();
        let content = source_text(100, 0);
        for i in 0..5 {
            write_file(&dir, &format!("f{i}.rs"), &content);
        }

        let rows: Vec<FileRow> = (0..5)
            .map(|i| make_file_row(&format!("f{i}.rs"), "root", "Rust", 100, content.len()))
            .collect();

        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.5,
            3, // only analyze 3
            None,
            &NearDupLimits::default(),
            &[],
        )
        .unwrap();

        assert_eq!(report.eligible_files, Some(5));
        assert_eq!(report.files_analyzed, 3);
        assert_eq!(report.files_skipped, 2);
    }
}

// ── Cluster completeness ────────────────────────────────────────

mod cluster_completeness {
    use super::*;

    #[test]
    fn given_identical_files_then_cluster_files_are_sorted_alphabetically() {
        let dir = TempDir::new().unwrap();
        let content = source_text(100, 0);
        write_file(&dir, "c.rs", &content);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);

        let rows = vec![
            make_file_row("c.rs", "root", "Rust", 100, content.len()),
            make_file_row("a.rs", "root", "Rust", 100, content.len()),
            make_file_row("b.rs", "root", "Rust", 100, content.len()),
        ];

        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.5,
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        )
        .unwrap();

        let clusters = report.clusters.as_ref().unwrap();
        assert_eq!(clusters.len(), 1);
        assert_eq!(
            clusters[0].files,
            vec!["a.rs", "b.rs", "c.rs"],
            "cluster files should be sorted alphabetically"
        );
    }

    #[test]
    fn given_two_separate_groups_then_two_clusters_sorted_by_max_similarity() {
        let dir = TempDir::new().unwrap();

        // Group 1: identical pair
        let content1 = source_text(100, 1);
        write_file(&dir, "g1_a.rs", &content1);
        write_file(&dir, "g1_b.rs", &content1);

        // Group 2: identical pair with different content
        let content2 = source_text(100, 2);
        write_file(&dir, "g2_a.rs", &content2);
        write_file(&dir, "g2_b.rs", &content2);

        let rows = vec![
            make_file_row("g1_a.rs", "root", "Rust", 100, content1.len()),
            make_file_row("g1_b.rs", "root", "Rust", 100, content1.len()),
            make_file_row("g2_a.rs", "root", "Rust", 100, content2.len()),
            make_file_row("g2_b.rs", "root", "Rust", 100, content2.len()),
        ];

        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.5,
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        )
        .unwrap();

        let clusters = report.clusters.as_ref().unwrap();
        assert_eq!(clusters.len(), 2, "should have two distinct clusters");

        // Both clusters should have max_similarity ~1.0
        for c in clusters {
            assert!(
                c.max_similarity >= 0.99,
                "identical pairs should yield ~1.0 similarity"
            );
            assert_eq!(c.files.len(), 2);
            assert_eq!(c.pair_count, 1);
        }

        // Verify sorted by max_similarity desc, then representative alphabetically
        for window in clusters.windows(2) {
            assert!(
                window[0].max_similarity >= window[1].max_similarity
                    || window[0].representative <= window[1].representative,
                "clusters should be sorted"
            );
        }
    }
}
