//! Property-based tests for near-duplicate detection invariants.

use std::io::Write;

use proptest::prelude::*;
use tempfile::TempDir;
use tokmd_analysis_near_dup::{NearDupLimits, build_near_dup_report};
use tokmd_analysis_types::NearDupScope;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ──────────────────────────────────────────────────────

fn make_file_row(path: &str, code: usize, bytes: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: "root".to_string(),
        lang: "Rust".to_string(),
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

/// Generate source text with `n` tokens, using `seed` to vary content.
fn source_text(n: usize, seed: usize) -> String {
    (0..n)
        .map(|i| format!("tok_{}_{}", seed, i))
        .collect::<Vec<_>>()
        .join(" + ")
}

// ── Strategies ───────────────────────────────────────────────────

/// Valid threshold in [0.0, 1.0].
fn arb_threshold() -> impl Strategy<Value = f64> {
    (0u32..=100).prop_map(|v| v as f64 / 100.0)
}

/// Token count large enough for winnowing (K=25 minimum).
fn arb_token_count() -> impl Strategy<Value = usize> {
    30usize..200
}

/// Scope variants.
fn arb_scope() -> impl Strategy<Value = NearDupScope> {
    prop_oneof![
        Just(NearDupScope::Global),
        Just(NearDupScope::Module),
        Just(NearDupScope::Lang),
    ]
}

// ── Properties ───────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Report always succeeds (no panic/error) for valid inputs.
    #[test]
    fn report_never_errors(
        threshold in arb_threshold(),
        scope in arb_scope(),
        n_tokens in arb_token_count(),
    ) {
        let dir = TempDir::new().unwrap();
        let content = source_text(n_tokens, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);

        let rows = vec![
            make_file_row("a.rs", n_tokens, content.len()),
            make_file_row("b.rs", n_tokens, content.len()),
        ];

        let result = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            scope,
            threshold,
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        );

        prop_assert!(result.is_ok(), "report should never fail: {:?}", result.err());
    }

    /// Similarity values are always in [0.0, 1.0].
    #[test]
    fn similarity_in_unit_range(n_tokens in arb_token_count()) {
        let dir = TempDir::new().unwrap();
        let content = source_text(n_tokens, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);

        let rows = vec![
            make_file_row("a.rs", n_tokens, content.len()),
            make_file_row("b.rs", n_tokens, content.len()),
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
        ).unwrap();

        for pair in &report.pairs {
            prop_assert!(
                pair.similarity >= 0.0 && pair.similarity <= 1.0,
                "similarity out of range: {}",
                pair.similarity
            );
        }
    }

    /// Pairs are always sorted by similarity descending.
    #[test]
    fn pairs_sorted_descending(n_tokens in arb_token_count()) {
        let dir = TempDir::new().unwrap();
        // Create 3 files with same content to generate multiple pairs
        let content = source_text(n_tokens, 0);
        for name in &["a.rs", "b.rs", "c.rs"] {
            write_file(&dir, name, &content);
        }

        let rows: Vec<FileRow> = ["a.rs", "b.rs", "c.rs"]
            .iter()
            .map(|name| make_file_row(name, n_tokens, content.len()))
            .collect();

        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.0,
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        ).unwrap();

        for window in report.pairs.windows(2) {
            prop_assert!(
                window[0].similarity >= window[1].similarity,
                "pairs not sorted: {} vs {}",
                window[0].similarity,
                window[1].similarity
            );
        }
    }

    /// files_analyzed + files_skipped == eligible_files (when eligible_files is reported).
    #[test]
    fn analyzed_plus_skipped_equals_eligible(
        max_files in 1usize..10,
        file_count in 1usize..15,
        n_tokens in arb_token_count(),
    ) {
        let dir = TempDir::new().unwrap();
        let content = source_text(n_tokens, 0);
        let mut rows = Vec::new();
        for i in 0..file_count {
            let name = format!("f{}.rs", i);
            write_file(&dir, &name, &content);
            rows.push(make_file_row(&name, n_tokens, content.len()));
        }

        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.5,
            max_files,
            None,
            &NearDupLimits::default(),
            &[],
        ).unwrap();

        if let Some(eligible) = report.eligible_files {
            prop_assert_eq!(
                report.files_analyzed + report.files_skipped,
                eligible,
                "analyzed({}) + skipped({}) != eligible({})",
                report.files_analyzed,
                report.files_skipped,
                eligible
            );
        }
    }

    /// Higher thresholds never produce more pairs than lower thresholds (monotonicity).
    #[test]
    fn higher_threshold_fewer_or_equal_pairs(
        n_tokens in arb_token_count(),
        t_low in 0.0f64..0.5,
        t_delta in 0.0f64..0.5,
    ) {
        let t_high = (t_low + t_delta).min(1.0);
        let dir = TempDir::new().unwrap();

        // Shared content with minor variation
        let shared: Vec<String> = (0..n_tokens).map(|i| format!("shared_{}", i)).collect();
        let suffix_a: Vec<String> = (0..20).map(|i| format!("ua_{}", i)).collect();
        let suffix_b: Vec<String> = (0..20).map(|i| format!("ub_{}", i)).collect();
        let content_a = [shared.clone(), suffix_a].concat().join(" + ");
        let content_b = [shared, suffix_b].concat().join(" + ");

        write_file(&dir, "a.rs", &content_a);
        write_file(&dir, "b.rs", &content_b);

        let rows = vec![
            make_file_row("a.rs", n_tokens + 20, content_a.len()),
            make_file_row("b.rs", n_tokens + 20, content_b.len()),
        ];

        let report_low = build_near_dup_report(
            dir.path(),
            &make_export(rows.clone()),
            NearDupScope::Global,
            t_low,
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        ).unwrap();

        let report_high = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            t_high,
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        ).unwrap();

        prop_assert!(
            report_high.pairs.len() <= report_low.pairs.len(),
            "higher threshold {} yielded {} pairs > lower threshold {} with {} pairs",
            t_high,
            report_high.pairs.len(),
            t_low,
            report_low.pairs.len()
        );
    }

    /// Cluster file count is always >= 2 (a cluster needs at least a pair).
    #[test]
    fn clusters_have_at_least_two_files(n_tokens in arb_token_count()) {
        let dir = TempDir::new().unwrap();
        let content = source_text(n_tokens, 0);
        for name in &["a.rs", "b.rs", "c.rs"] {
            write_file(&dir, name, &content);
        }

        let rows: Vec<FileRow> = ["a.rs", "b.rs", "c.rs"]
            .iter()
            .map(|name| make_file_row(name, n_tokens, content.len()))
            .collect();

        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.0,
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        ).unwrap();

        if let Some(clusters) = &report.clusters {
            for cluster in clusters {
                prop_assert!(
                    cluster.files.len() >= 2,
                    "cluster has {} files, expected >= 2",
                    cluster.files.len()
                );
            }
        }
    }

    /// Cluster files are always sorted alphabetically.
    #[test]
    fn cluster_files_sorted(n_tokens in arb_token_count()) {
        let dir = TempDir::new().unwrap();
        let content = source_text(n_tokens, 0);
        for name in &["z.rs", "a.rs", "m.rs"] {
            write_file(&dir, name, &content);
        }

        let rows: Vec<FileRow> = ["z.rs", "a.rs", "m.rs"]
            .iter()
            .map(|name| make_file_row(name, n_tokens, content.len()))
            .collect();

        let report = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.0,
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        ).unwrap();

        if let Some(clusters) = &report.clusters {
            for cluster in clusters {
                let mut sorted = cluster.files.clone();
                sorted.sort();
                prop_assert_eq!(
                    &cluster.files,
                    &sorted,
                    "cluster files not sorted"
                );
            }
        }
    }

    /// Shared fingerprints never exceed min(left, right) fingerprints.
    #[test]
    fn shared_fingerprints_bounded(n_tokens in arb_token_count()) {
        let dir = TempDir::new().unwrap();
        let content = source_text(n_tokens, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);

        let rows = vec![
            make_file_row("a.rs", n_tokens, content.len()),
            make_file_row("b.rs", n_tokens, content.len()),
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
        ).unwrap();

        for pair in &report.pairs {
            let max_possible = pair.left_fingerprints.min(pair.right_fingerprints);
            prop_assert!(
                pair.shared_fingerprints <= max_possible,
                "shared {} > min(left={}, right={})",
                pair.shared_fingerprints,
                pair.left_fingerprints,
                pair.right_fingerprints
            );
        }
    }

    /// Idempotent: running the same report twice produces identical results.
    #[test]
    fn idempotent_report(n_tokens in arb_token_count()) {
        let dir = TempDir::new().unwrap();
        let content = source_text(n_tokens, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);

        let rows = vec![
            make_file_row("a.rs", n_tokens, content.len()),
            make_file_row("b.rs", n_tokens, content.len()),
        ];

        let report1 = build_near_dup_report(
            dir.path(),
            &make_export(rows.clone()),
            NearDupScope::Global,
            0.5,
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        ).unwrap();

        let report2 = build_near_dup_report(
            dir.path(),
            &make_export(rows),
            NearDupScope::Global,
            0.5,
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        ).unwrap();

        prop_assert_eq!(report1.pairs.len(), report2.pairs.len());
        for (p1, p2) in report1.pairs.iter().zip(report2.pairs.iter()) {
            prop_assert_eq!(&p1.left, &p2.left);
            prop_assert_eq!(&p1.right, &p2.right);
            prop_assert!((p1.similarity - p2.similarity).abs() < 1e-10);
        }
    }
}
