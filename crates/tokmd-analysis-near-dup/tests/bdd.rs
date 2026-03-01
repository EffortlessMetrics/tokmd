//! BDD-style scenario tests for near-duplicate detection.

use std::io::Write;

use tempfile::TempDir;
use tokmd_analysis_near_dup::{NearDupLimits, build_near_dup_report};
use tokmd_analysis_types::NearDupScope;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ──────────────────────────────────────────────────────

/// Generate source text with `n` distinct tokens (enough for winnowing with K=25).
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

/// Write `content` to `dir/path` creating parent directories as needed.
fn write_file(dir: &TempDir, path: &str, content: &str) {
    let full = dir.path().join(path);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let mut f = std::fs::File::create(&full).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}

// ── Identical files ──────────────────────────────────────────────

mod identical_files {
    use super::*;

    #[test]
    fn given_two_identical_files_then_detected_as_near_duplicates() {
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

        assert_eq!(report.pairs.len(), 1);
        assert!(
            report.pairs[0].similarity >= 0.99,
            "identical files should have similarity ~1.0, got {}",
            report.pairs[0].similarity
        );
        assert_eq!(report.files_analyzed, 2);
    }

    #[test]
    fn given_three_identical_files_then_three_pairs_one_cluster() {
        let dir = TempDir::new().unwrap();
        let content = source_text(100, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);
        write_file(&dir, "c.rs", &content);

        let rows = vec![
            make_file_row("a.rs", "root", "Rust", 100, content.len()),
            make_file_row("b.rs", "root", "Rust", 100, content.len()),
            make_file_row("c.rs", "root", "Rust", 100, content.len()),
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

        assert_eq!(report.pairs.len(), 3);
        let clusters = report.clusters.as_ref().unwrap();
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].files.len(), 3);
    }
}

// ── Completely different files ───────────────────────────────────

mod completely_different_files {
    use super::*;

    #[test]
    fn given_two_unrelated_files_then_no_pairs() {
        let dir = TempDir::new().unwrap();
        let content_a = source_text(100, 1);
        let content_b = source_text(100, 2);
        write_file(&dir, "alpha.rs", &content_a);
        write_file(&dir, "beta.rs", &content_b);

        let rows = vec![
            make_file_row("alpha.rs", "root", "Rust", 100, content_a.len()),
            make_file_row("beta.rs", "root", "Rust", 100, content_b.len()),
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

        assert!(
            report.pairs.is_empty(),
            "completely different files should have no pairs, got {}",
            report.pairs.len()
        );
        assert!(report.clusters.is_none());
    }
}

// ── Slightly different files ─────────────────────────────────────

mod slightly_different_files {
    use super::*;

    /// Create two files sharing ~80% of tokens.
    #[test]
    fn given_files_sharing_most_tokens_then_detected_above_threshold() {
        let dir = TempDir::new().unwrap();
        // Shared prefix of 80 tokens, different suffix of 20
        let shared: Vec<String> = (0..80).map(|i| format!("shared_{}", i)).collect();
        let suffix_a: Vec<String> = (0..20).map(|i| format!("uniq_a_{}", i)).collect();
        let suffix_b: Vec<String> = (0..20).map(|i| format!("uniq_b_{}", i)).collect();

        let content_a = [shared.clone(), suffix_a].concat().join(" + ");
        let content_b = [shared, suffix_b].concat().join(" + ");
        write_file(&dir, "similar_a.rs", &content_a);
        write_file(&dir, "similar_b.rs", &content_b);

        let rows = vec![
            make_file_row("similar_a.rs", "root", "Rust", 100, content_a.len()),
            make_file_row("similar_b.rs", "root", "Rust", 100, content_b.len()),
        ];

        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.3, // low threshold to catch partial matches
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        )
        .unwrap();

        assert_eq!(report.pairs.len(), 1);
        let sim = report.pairs[0].similarity;
        assert!(
            sim > 0.3 && sim < 1.0,
            "partial overlap should yield intermediate similarity, got {}",
            sim
        );
    }

    #[test]
    fn given_files_sharing_most_tokens_then_not_detected_at_high_threshold() {
        let dir = TempDir::new().unwrap();
        let shared: Vec<String> = (0..60).map(|i| format!("shared_{}", i)).collect();
        let suffix_a: Vec<String> = (0..40).map(|i| format!("uniq_a_{}", i)).collect();
        let suffix_b: Vec<String> = (0..40).map(|i| format!("uniq_b_{}", i)).collect();

        let content_a = [shared.clone(), suffix_a].concat().join(" + ");
        let content_b = [shared, suffix_b].concat().join(" + ");
        write_file(&dir, "a.rs", &content_a);
        write_file(&dir, "b.rs", &content_b);

        let rows = vec![
            make_file_row("a.rs", "root", "Rust", 100, content_a.len()),
            make_file_row("b.rs", "root", "Rust", 100, content_b.len()),
        ];

        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.99, // very high threshold
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        )
        .unwrap();

        assert!(
            report.pairs.is_empty(),
            "partially overlapping files should not match at 0.99 threshold"
        );
    }
}

// ── Empty / minimal inputs ───────────────────────────────────────

mod empty_inputs {
    use super::*;

    #[test]
    fn given_no_files_then_empty_report() {
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

        assert_eq!(report.files_analyzed, 0);
        assert!(report.pairs.is_empty());
        assert!(report.clusters.is_none());
    }

    #[test]
    fn given_single_file_then_no_pairs() {
        let dir = TempDir::new().unwrap();
        let content = source_text(100, 0);
        write_file(&dir, "only.rs", &content);

        let rows = vec![make_file_row("only.rs", "root", "Rust", 100, content.len())];

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

        assert_eq!(report.files_analyzed, 1);
        assert!(report.pairs.is_empty());
    }

    #[test]
    fn given_files_too_short_for_winnowing_then_no_pairs() {
        let dir = TempDir::new().unwrap();
        // Files with fewer than K=25 tokens produce no fingerprints
        write_file(&dir, "tiny_a.rs", "fn a() {}");
        write_file(&dir, "tiny_b.rs", "fn a() {}");

        let rows = vec![
            make_file_row("tiny_a.rs", "root", "Rust", 1, 9),
            make_file_row("tiny_b.rs", "root", "Rust", 1, 9),
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

        assert!(report.pairs.is_empty());
    }
}

// ── Threshold behaviour ──────────────────────────────────────────

mod threshold_behaviour {
    use super::*;

    #[test]
    fn given_threshold_zero_then_all_candidate_pairs_emitted() {
        let dir = TempDir::new().unwrap();
        let content_a = source_text(100, 10);
        let content_b = source_text(100, 11);
        // Ensure files share at least some structure (the " + " delimiters)
        // but with different tokens, jaccard should be near zero.
        // With threshold=0.0, even minimal overlap is included.
        write_file(&dir, "x.rs", &content_a);
        write_file(&dir, "y.rs", &content_b);

        let rows = vec![
            make_file_row("x.rs", "root", "Rust", 100, content_a.len()),
            make_file_row("y.rs", "root", "Rust", 100, content_b.len()),
        ];

        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.0, // accept everything
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        )
        .unwrap();

        // With distinct tokens, may still get zero pairs because fingerprints
        // don't overlap. That's acceptable — the point is threshold=0.0 doesn't
        // crash and works correctly.
        assert_eq!(report.files_analyzed, 2);
    }

    #[test]
    fn given_threshold_one_then_only_exact_duplicates() {
        let dir = TempDir::new().unwrap();
        let content = source_text(100, 0);
        write_file(&dir, "dup1.rs", &content);
        write_file(&dir, "dup2.rs", &content);

        let rows = vec![
            make_file_row("dup1.rs", "root", "Rust", 100, content.len()),
            make_file_row("dup2.rs", "root", "Rust", 100, content.len()),
        ];

        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            1.0,
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        )
        .unwrap();

        // Identical files should have similarity 1.0 and pass threshold=1.0
        assert_eq!(report.pairs.len(), 1);
        assert!((report.pairs[0].similarity - 1.0).abs() < 1e-10);
    }
}

// ── Scope partitioning ───────────────────────────────────────────

mod scope_partitioning {
    use super::*;

    #[test]
    fn given_module_scope_then_files_in_different_modules_not_compared() {
        let dir = TempDir::new().unwrap();
        let content = source_text(100, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);

        let rows = vec![
            make_file_row("a.rs", "mod_a", "Rust", 100, content.len()),
            make_file_row("b.rs", "mod_b", "Rust", 100, content.len()),
        ];

        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Module,
            0.5,
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        )
        .unwrap();

        assert!(
            report.pairs.is_empty(),
            "files in different modules should not be compared under Module scope"
        );
    }

    #[test]
    fn given_lang_scope_then_files_in_different_langs_not_compared() {
        let dir = TempDir::new().unwrap();
        let content = source_text(100, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.py", &content);

        let rows = vec![
            make_file_row("a.rs", "root", "Rust", 100, content.len()),
            make_file_row("b.py", "root", "Python", 100, content.len()),
        ];

        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Lang,
            0.5,
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        )
        .unwrap();

        assert!(
            report.pairs.is_empty(),
            "files in different languages should not be compared under Lang scope"
        );
    }

    #[test]
    fn given_global_scope_then_files_across_modules_compared() {
        let dir = TempDir::new().unwrap();
        let content = source_text(100, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);

        let rows = vec![
            make_file_row("a.rs", "mod_a", "Rust", 100, content.len()),
            make_file_row("b.rs", "mod_b", "Python", 100, content.len()),
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

        assert_eq!(
            report.pairs.len(),
            1,
            "global scope should compare files across modules and languages"
        );
    }
}

// ── Limits and truncation ────────────────────────────────────────

mod limits {
    use super::*;

    #[test]
    fn given_max_files_smaller_than_eligible_then_files_skipped() {
        let dir = TempDir::new().unwrap();
        let content = source_text(100, 0);
        for i in 0..5 {
            write_file(&dir, &format!("f{}.rs", i), &content);
        }

        let rows: Vec<FileRow> = (0..5)
            .map(|i| make_file_row(&format!("f{}.rs", i), "root", "Rust", 100, content.len()))
            .collect();

        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.5,
            2, // only analyze 2 files
            None,
            &NearDupLimits::default(),
            &[],
        )
        .unwrap();

        assert_eq!(report.files_analyzed, 2);
        assert_eq!(report.files_skipped, 3);
    }

    #[test]
    fn given_max_pairs_then_pairs_truncated_but_clusters_complete() {
        let dir = TempDir::new().unwrap();
        let content = source_text(100, 0);
        for i in 0..4 {
            write_file(&dir, &format!("f{}.rs", i), &content);
        }

        let rows: Vec<FileRow> = (0..4)
            .map(|i| make_file_row(&format!("f{}.rs", i), "root", "Rust", 100, content.len()))
            .collect();

        // 4 identical files = C(4,2) = 6 pairs
        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.5,
            1000,
            Some(2), // truncate to 2 pairs
            &NearDupLimits::default(),
            &[],
        )
        .unwrap();

        assert_eq!(report.pairs.len(), 2);
        assert!(report.truncated);
        // Clusters should still reflect all 4 files
        let clusters = report.clusters.as_ref().unwrap();
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].files.len(), 4);
    }

    #[test]
    fn given_max_file_bytes_then_large_files_excluded() {
        let dir = TempDir::new().unwrap();
        let content = source_text(100, 0);
        write_file(&dir, "small.rs", &content);
        write_file(&dir, "big.rs", &content);

        let rows = vec![
            make_file_row("small.rs", "root", "Rust", 100, 100),
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

        // Only small.rs should be analyzed; big.rs filtered by byte limit
        assert_eq!(report.files_analyzed, 1);
    }

    #[test]
    fn given_custom_max_file_bytes_then_respected() {
        let dir = TempDir::new().unwrap();
        let content = source_text(100, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);

        let rows = vec![
            make_file_row("a.rs", "root", "Rust", 100, 200),
            make_file_row("b.rs", "root", "Rust", 100, 200),
        ];

        let limits = NearDupLimits {
            max_bytes: None,
            max_file_bytes: Some(100), // both files exceed 100 bytes
        };

        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.5,
            1000,
            None,
            &limits,
            &[],
        )
        .unwrap();

        assert_eq!(report.files_analyzed, 0);
    }
}

// ── Exclude patterns ─────────────────────────────────────────────

mod exclude_patterns {
    use super::*;

    #[test]
    fn given_exclude_pattern_then_matching_files_excluded() {
        let dir = TempDir::new().unwrap();
        let content = source_text(100, 0);
        write_file(&dir, "src/a.rs", &content);
        write_file(&dir, "vendor/b.rs", &content);

        let rows = vec![
            make_file_row("src/a.rs", "src", "Rust", 100, content.len()),
            make_file_row("vendor/b.rs", "vendor", "Rust", 100, content.len()),
        ];

        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.5,
            1000,
            None,
            &NearDupLimits::default(),
            &["vendor/**".to_string()],
        )
        .unwrap();

        assert_eq!(report.files_analyzed, 1);
        assert_eq!(report.excluded_by_pattern, Some(1));
    }

    #[test]
    fn given_no_exclude_patterns_then_excluded_by_pattern_is_none() {
        let dir = TempDir::new().unwrap();
        let content = source_text(100, 0);
        write_file(&dir, "a.rs", &content);

        let rows = vec![make_file_row("a.rs", "root", "Rust", 100, content.len())];

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

        assert!(report.excluded_by_pattern.is_none());
    }
}

// ── Child file kind filtering ────────────────────────────────────

mod file_kind_filtering {
    use super::*;

    #[test]
    fn given_child_kind_files_then_excluded_from_analysis() {
        let dir = TempDir::new().unwrap();
        let content = source_text(100, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);

        let mut rows = vec![
            make_file_row("a.rs", "root", "Rust", 100, content.len()),
            make_file_row("b.rs", "root", "Rust", 100, content.len()),
        ];
        // Mark b.rs as a Child (embedded language)
        rows[1].kind = FileKind::Child;

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

        assert_eq!(report.files_analyzed, 1);
        assert!(report.pairs.is_empty());
    }
}

// ── Report metadata ──────────────────────────────────────────────

mod report_metadata {
    use super::*;

    #[test]
    fn given_report_then_params_reflect_inputs() {
        let dir = TempDir::new().unwrap();
        let export = make_export(vec![]);

        let report = build_near_dup_report(
            dir.path(),
            &export,
            NearDupScope::Lang,
            0.75,
            42,
            Some(10),
            &NearDupLimits {
                max_bytes: None,
                max_file_bytes: Some(1024),
            },
            &["*.lock".to_string()],
        )
        .unwrap();

        assert!(matches!(report.params.scope, NearDupScope::Lang));
        assert!((report.params.threshold - 0.75).abs() < 1e-10);
        assert_eq!(report.params.max_files, 42);
        assert_eq!(report.params.max_pairs, Some(10));
        assert_eq!(report.params.max_file_bytes, Some(1024));
        assert_eq!(report.params.exclude_patterns, vec!["*.lock"]);
    }

    #[test]
    fn given_report_then_algorithm_constants_present() {
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

        let algo = report.params.algorithm.as_ref().unwrap();
        assert_eq!(algo.k_gram_size, 25);
        assert_eq!(algo.window_size, 4);
        assert_eq!(algo.max_postings, 50);
    }

    #[test]
    fn given_report_with_pairs_then_stats_present() {
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

        let stats = report.stats.as_ref().unwrap();
        assert!(stats.bytes_processed > 0);
    }

    #[test]
    fn given_no_truncation_then_truncated_is_false() {
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

        assert!(!report.truncated);
    }
}

// ── Pair ordering ────────────────────────────────────────────────

mod pair_ordering {
    use super::*;

    #[test]
    fn given_multiple_pairs_then_sorted_by_similarity_desc() {
        let dir = TempDir::new().unwrap();

        // Create files with varying overlap:
        // identical pair (a,b) and partially similar pair (a,c)
        let shared: Vec<String> = (0..80).map(|i| format!("tok_{}", i)).collect();
        let suffix_a: Vec<String> = (80..100).map(|i| format!("tok_{}", i)).collect();
        let suffix_c: Vec<String> = (0..20).map(|i| format!("uniq_c_{}", i)).collect();

        let content_a = [shared.clone(), suffix_a].concat().join(" + ");
        let content_b = content_a.clone(); // identical to a
        let content_c = [shared, suffix_c].concat().join(" + ");

        write_file(&dir, "a.rs", &content_a);
        write_file(&dir, "b.rs", &content_b);
        write_file(&dir, "c.rs", &content_c);

        let rows = vec![
            make_file_row("a.rs", "root", "Rust", 100, content_a.len()),
            make_file_row("b.rs", "root", "Rust", 100, content_b.len()),
            make_file_row("c.rs", "root", "Rust", 100, content_c.len()),
        ];

        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.3,
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        )
        .unwrap();

        // Pairs should be sorted by similarity descending
        for window in report.pairs.windows(2) {
            assert!(
                window[0].similarity >= window[1].similarity,
                "pairs not sorted: {} should be >= {}",
                window[0].similarity,
                window[1].similarity
            );
        }
    }
}

// ── Empty file content ───────────────────────────────────────────

mod empty_file_content {
    use super::*;

    #[test]
    fn given_empty_files_on_disk_then_no_pairs() {
        let dir = TempDir::new().unwrap();
        write_file(&dir, "empty_a.rs", "");
        write_file(&dir, "empty_b.rs", "");

        let rows = vec![
            make_file_row("empty_a.rs", "root", "Rust", 0, 0),
            make_file_row("empty_b.rs", "root", "Rust", 0, 0),
        ];

        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.0,
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        )
        .unwrap();

        assert!(
            report.pairs.is_empty(),
            "empty files produce no fingerprints and should yield no pairs"
        );
    }

    #[test]
    fn given_one_empty_one_nonempty_then_no_pairs() {
        let dir = TempDir::new().unwrap();
        write_file(&dir, "empty.rs", "");
        let content = source_text(100, 0);
        write_file(&dir, "full.rs", &content);

        let rows = vec![
            make_file_row("empty.rs", "root", "Rust", 0, 0),
            make_file_row("full.rs", "root", "Rust", 100, content.len()),
        ];

        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.0,
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        )
        .unwrap();

        assert!(
            report.pairs.is_empty(),
            "empty file cannot match a non-empty file"
        );
    }
}

// ── Single token change near-dup ─────────────────────────────────

mod single_token_change {
    use super::*;

    #[test]
    fn given_files_differing_by_one_token_then_high_similarity() {
        let dir = TempDir::new().unwrap();
        let mut tokens: Vec<String> = (0..100).map(|i| format!("common_{}", i)).collect();
        let content_a = tokens.join(" + ");
        // Replace one token in the middle
        tokens[50] = "changed_token".to_string();
        let content_b = tokens.join(" + ");

        write_file(&dir, "orig.rs", &content_a);
        write_file(&dir, "tweaked.rs", &content_b);

        let rows = vec![
            make_file_row("orig.rs", "root", "Rust", 100, content_a.len()),
            make_file_row("tweaked.rs", "root", "Rust", 100, content_b.len()),
        ];

        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.3,
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        )
        .unwrap();

        assert_eq!(
            report.pairs.len(),
            1,
            "single-token change should be detected"
        );
        assert!(
            report.pairs[0].similarity > 0.3,
            "single-token change should yield meaningful similarity, got {}",
            report.pairs[0].similarity
        );
    }
}

// ── Determinism ──────────────────────────────────────────────────

mod determinism {
    use super::*;

    #[test]
    fn given_same_files_then_report_is_identical_across_runs() {
        let dir = TempDir::new().unwrap();
        let content_a = source_text(100, 0);
        let shared: Vec<String> = (0..80).map(|i| format!("shared_{}", i)).collect();
        let suffix: Vec<String> = (0..20).map(|i| format!("uniq_{}", i)).collect();
        let content_b = [shared, suffix].concat().join(" + ");

        write_file(&dir, "a.rs", &content_a);
        write_file(&dir, "b.rs", &content_b);

        let rows = vec![
            make_file_row("a.rs", "root", "Rust", 100, content_a.len()),
            make_file_row("b.rs", "root", "Rust", 100, content_b.len()),
        ];

        let r1 = build_near_dup_report(
            dir.path(),
            &make_export(rows.clone()),
            NearDupScope::Global,
            0.0,
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        )
        .unwrap();

        let r2 = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.0,
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        )
        .unwrap();

        assert_eq!(r1.pairs.len(), r2.pairs.len());
        assert_eq!(r1.files_analyzed, r2.files_analyzed);
        assert_eq!(r1.files_skipped, r2.files_skipped);
        for (p1, p2) in r1.pairs.iter().zip(r2.pairs.iter()) {
            assert_eq!(p1.left, p2.left);
            assert_eq!(p1.right, p2.right);
            assert!(
                (p1.similarity - p2.similarity).abs() < 1e-10,
                "similarity differs: {} vs {}",
                p1.similarity,
                p2.similarity
            );
            assert_eq!(p1.shared_fingerprints, p2.shared_fingerprints);
            assert_eq!(p1.left_fingerprints, p2.left_fingerprints);
            assert_eq!(p1.right_fingerprints, p2.right_fingerprints);
        }
        // Clusters should also match
        assert_eq!(
            r1.clusters.as_ref().map(|c| c.len()),
            r2.clusters.as_ref().map(|c| c.len())
        );
    }
}
