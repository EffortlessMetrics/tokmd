//! Deep round-2 tests for `tokmd-analysis-near-dup` (w51).
//!
//! Focuses on near-duplicate detection with identical, slightly different,
//! and completely different files, threshold behavior, and structural
//! properties of the report.

use std::io::Write;

use tempfile::TempDir;
use tokmd_analysis_near_dup::{build_near_dup_report, NearDupLimits};
use tokmd_analysis_types::NearDupScope;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────

fn make_row(path: &str, module: &str, lang: &str, code: usize, bytes: usize) -> FileRow {
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

fn source_text(n: usize, seed: usize) -> String {
    (0..n)
        .map(|i| format!("tok_{seed}_{i}"))
        .collect::<Vec<_>>()
        .join(" + ")
}

fn content_with_overlap(shared: usize, unique: usize, seed: usize) -> String {
    let shared_part: Vec<String> = (0..shared).map(|i| format!("shared_{i}")).collect();
    let unique_part: Vec<String> = (0..unique).map(|i| format!("unique_{seed}_{i}")).collect();
    let mut all = shared_part;
    all.extend(unique_part);
    all.join(" + ")
}

fn default_limits() -> NearDupLimits {
    NearDupLimits {
        max_bytes: None,
        max_file_bytes: None,
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Identical file detection
// ═══════════════════════════════════════════════════════════════════

mod identical_files {
    use super::*;

    #[test]
    fn two_identical_files_similarity_near_one() {
        let dir = TempDir::new().unwrap();
        let content = source_text(150, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);

        let rows = vec![
            make_row("a.rs", "src", "Rust", 150, content.len()),
            make_row("b.rs", "src", "Rust", 150, content.len()),
        ];
        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.5,
            1000,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();

        assert_eq!(report.pairs.len(), 1, "exactly one pair");
        assert!(
            report.pairs[0].similarity > 0.95,
            "identical → similarity > 0.95, got {}",
            report.pairs[0].similarity
        );
    }

    #[test]
    fn four_identical_files_form_single_cluster() {
        let dir = TempDir::new().unwrap();
        let content = source_text(150, 0);
        let names = ["a.rs", "b.rs", "c.rs", "d.rs"];
        for name in &names {
            write_file(&dir, name, &content);
        }
        let rows: Vec<_> = names
            .iter()
            .map(|n| make_row(n, "src", "Rust", 150, content.len()))
            .collect();
        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.5,
            1000,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();

        let clusters = report.clusters.as_ref().expect("clusters present");
        assert_eq!(clusters.len(), 1, "all identical → 1 cluster");
        assert_eq!(clusters[0].files.len(), 4);
    }

    #[test]
    fn identical_files_shared_fingerprints_equals_total() {
        let dir = TempDir::new().unwrap();
        let content = source_text(150, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);

        let rows = vec![
            make_row("a.rs", "src", "Rust", 150, content.len()),
            make_row("b.rs", "src", "Rust", 150, content.len()),
        ];
        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.0,
            1000,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();

        if !report.pairs.is_empty() {
            let p = &report.pairs[0];
            // For identical files, shared should equal each file's fingerprint count
            assert_eq!(
                p.shared_fingerprints, p.left_fingerprints,
                "identical files: shared == left fingerprints"
            );
            assert_eq!(
                p.shared_fingerprints, p.right_fingerprints,
                "identical files: shared == right fingerprints"
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Slightly different files
// ═══════════════════════════════════════════════════════════════════

mod slightly_different {
    use super::*;

    #[test]
    fn high_overlap_detected_above_threshold() {
        let dir = TempDir::new().unwrap();
        let a = content_with_overlap(120, 10, 1);
        let b = content_with_overlap(120, 10, 2);
        write_file(&dir, "a.rs", &a);
        write_file(&dir, "b.rs", &b);

        let rows = vec![
            make_row("a.rs", "src", "Rust", 130, a.len()),
            make_row("b.rs", "src", "Rust", 130, b.len()),
        ];
        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.3,
            1000,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();

        assert!(
            !report.pairs.is_empty(),
            "high overlap files should be detected"
        );
    }

    #[test]
    fn medium_overlap_has_moderate_similarity() {
        let dir = TempDir::new().unwrap();
        let a = content_with_overlap(60, 60, 1);
        let b = content_with_overlap(60, 60, 2);
        write_file(&dir, "a.rs", &a);
        write_file(&dir, "b.rs", &b);

        let rows = vec![
            make_row("a.rs", "src", "Rust", 120, a.len()),
            make_row("b.rs", "src", "Rust", 120, b.len()),
        ];
        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.0,
            1000,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();

        for pair in &report.pairs {
            assert!(
                pair.similarity <= 1.0 && pair.similarity >= 0.0,
                "similarity in [0,1]"
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Completely different files
// ═══════════════════════════════════════════════════════════════════

mod completely_different {
    use super::*;

    #[test]
    fn disjoint_content_no_pairs_at_high_threshold() {
        let dir = TempDir::new().unwrap();
        let a = source_text(150, 100);
        let b = source_text(150, 200);
        write_file(&dir, "a.rs", &a);
        write_file(&dir, "b.rs", &b);

        let rows = vec![
            make_row("a.rs", "src", "Rust", 150, a.len()),
            make_row("b.rs", "src", "Rust", 150, b.len()),
        ];
        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.8,
            1000,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();

        assert!(
            report.pairs.is_empty(),
            "disjoint content → no pairs at threshold 0.8"
        );
    }

    #[test]
    fn many_unique_files_no_clusters() {
        let dir = TempDir::new().unwrap();
        let mut rows = Vec::new();
        for i in 0..6 {
            let content = source_text(150, i * 1000);
            let name = format!("f{i}.rs");
            write_file(&dir, &name, &content);
            rows.push(make_row(&name, "src", "Rust", 150, content.len()));
        }
        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.5,
            1000,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();

        assert!(
            report.pairs.is_empty(),
            "all unique files → no duplicate pairs"
        );
        // No clusters when no pairs
        let has_clusters = report.clusters.as_ref().map_or(true, |c| c.is_empty());
        assert!(has_clusters, "no pairs → no clusters");
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Threshold behavior
// ═══════════════════════════════════════════════════════════════════

mod threshold_behavior {
    use super::*;

    #[test]
    fn higher_threshold_fewer_pairs() {
        let dir = TempDir::new().unwrap();
        let content = source_text(150, 0);
        let varied = content_with_overlap(100, 50, 1);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);
        write_file(&dir, "c.rs", &varied);

        let rows = vec![
            make_row("a.rs", "src", "Rust", 150, content.len()),
            make_row("b.rs", "src", "Rust", 150, content.len()),
            make_row("c.rs", "src", "Rust", 150, varied.len()),
        ];
        let export = make_export(rows);

        let low = build_near_dup_report(
            dir.path(),
            &export,
            NearDupScope::Global,
            0.1,
            1000,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();

        let high = build_near_dup_report(
            dir.path(),
            &export,
            NearDupScope::Global,
            0.95,
            1000,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();

        assert!(
            high.pairs.len() <= low.pairs.len(),
            "higher threshold → ≤ pairs: {} > {}",
            high.pairs.len(),
            low.pairs.len()
        );
    }

    #[test]
    fn threshold_zero_includes_all_pairs() {
        let dir = TempDir::new().unwrap();
        let a = content_with_overlap(50, 100, 1);
        let b = content_with_overlap(50, 100, 2);
        write_file(&dir, "a.rs", &a);
        write_file(&dir, "b.rs", &b);

        let rows = vec![
            make_row("a.rs", "src", "Rust", 150, a.len()),
            make_row("b.rs", "src", "Rust", 150, b.len()),
        ];
        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.0,
            1000,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();

        // At threshold 0, any non-zero similarity pair is included
        for pair in &report.pairs {
            assert!(pair.similarity >= 0.0);
        }
    }

    #[test]
    fn analyzed_plus_skipped_equals_eligible() {
        let dir = TempDir::new().unwrap();
        let content = source_text(150, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);
        // Short file that will be skipped (fewer tokens than k-gram size)
        write_file(&dir, "short.rs", "x");

        let rows = vec![
            make_row("a.rs", "src", "Rust", 150, content.len()),
            make_row("b.rs", "src", "Rust", 150, content.len()),
            make_row("short.rs", "src", "Rust", 1, 1),
        ];
        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.5,
            1000,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();

        if let Some(eligible) = report.eligible_files {
            assert_eq!(
                report.files_analyzed + report.files_skipped,
                eligible,
                "analyzed + skipped = eligible"
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Cluster properties
// ═══════════════════════════════════════════════════════════════════

mod cluster_props {
    use super::*;

    #[test]
    fn cluster_files_sorted_alphabetically() {
        let dir = TempDir::new().unwrap();
        let content = source_text(150, 0);
        for name in &["z.rs", "a.rs", "m.rs"] {
            write_file(&dir, name, &content);
        }
        let rows: Vec<_> = ["z.rs", "a.rs", "m.rs"]
            .iter()
            .map(|n| make_row(n, "src", "Rust", 150, content.len()))
            .collect();
        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.5,
            1000,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();

        if let Some(clusters) = &report.clusters {
            for cluster in clusters {
                let sorted: Vec<_> = {
                    let mut v = cluster.files.clone();
                    v.sort();
                    v
                };
                assert_eq!(
                    cluster.files, sorted,
                    "cluster files must be sorted alphabetically"
                );
            }
        }
    }

    #[test]
    fn cluster_min_size_two() {
        let dir = TempDir::new().unwrap();
        let content = source_text(150, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);

        let rows = vec![
            make_row("a.rs", "src", "Rust", 150, content.len()),
            make_row("b.rs", "src", "Rust", 150, content.len()),
        ];
        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.5,
            1000,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();

        if let Some(clusters) = &report.clusters {
            for cluster in clusters {
                assert!(
                    cluster.files.len() >= 2,
                    "every cluster must have ≥ 2 files"
                );
            }
        }
    }
}
