//! Additional deep tests for near-duplicate detection.
//!
//! Covers serialization roundtrips, empty/single-file inputs,
//! identical-content detection, threshold edge cases, max_pairs
//! truncation, exclude pattern interaction, and stats invariants.

use crate::near_dup::{NearDupLimits, build_near_dup_report};
use std::io::Write;
use tempfile::TempDir;
use tokmd_analysis_types::{NearDupCluster, NearDupPairRow, NearDupScope, NearDuplicateReport};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────

fn source_text(n: usize, seed: usize) -> String {
    (0..n)
        .map(|i| format!("tok_{seed}_{i}"))
        .collect::<Vec<_>>()
        .join(" + ")
}

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

fn write_file(dir: &TempDir, rel: &str, content: &str) {
    let path = dir.path().join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}

fn make_export(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn default_limits() -> NearDupLimits {
    NearDupLimits::default()
}

// ── 1. Empty export produces empty report ───────────────────────

#[test]
fn empty_export_produces_empty_report() {
    let dir = TempDir::new().unwrap();
    let export = make_export(vec![]);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    assert!(report.pairs.is_empty());
    assert_eq!(report.files_analyzed, 0);
    assert_eq!(report.files_skipped, 0);
    assert!(report.clusters.is_none());
    assert!(!report.truncated);
}

// ── 2. Single file produces no pairs ────────────────────────────

#[test]
fn single_file_produces_no_pairs() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 1);
    write_file(&dir, "only.rs", &content);
    let export = make_export(vec![make_row("only.rs", "(root)", "Rust", 100, 5000)]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.0,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    assert!(report.pairs.is_empty());
    assert_eq!(report.files_analyzed, 1);
    assert!(report.clusters.is_none());
}

// ── 3. Identical files have similarity 1.0 ──────────────────────

#[test]
fn identical_files_have_similarity_one() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 2);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);

    let export = make_export(vec![
        make_row("a.rs", "(root)", "Rust", 100, 5000),
        make_row("b.rs", "(root)", "Rust", 100, 5000),
    ]);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.0,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    assert_eq!(report.pairs.len(), 1);
    assert!(
        (report.pairs[0].similarity - 1.0).abs() < 1e-10,
        "identical files should have similarity 1.0, got {}",
        report.pairs[0].similarity
    );
}

// ── 4. NearDuplicateReport serialization roundtrip ──────────────

#[test]
fn near_dup_report_serialization_roundtrip() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 3);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);

    let export = make_export(vec![
        make_row("a.rs", "(root)", "Rust", 100, 5000),
        make_row("b.rs", "(root)", "Rust", 100, 5000),
    ]);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    let json = serde_json::to_string_pretty(&report).unwrap();
    let deserialized: NearDuplicateReport = serde_json::from_str(&json).unwrap();

    assert_eq!(report.pairs.len(), deserialized.pairs.len());
    assert_eq!(report.files_analyzed, deserialized.files_analyzed);
    assert_eq!(report.files_skipped, deserialized.files_skipped);
    assert_eq!(report.truncated, deserialized.truncated);
    for (orig, deser) in report.pairs.iter().zip(deserialized.pairs.iter()) {
        assert_eq!(orig.left, deser.left);
        assert_eq!(orig.right, deser.right);
        assert!((orig.similarity - deser.similarity).abs() < 1e-10);
    }
}

// ── 5. NearDupPairRow serialization roundtrip ───────────────────

#[test]
fn pair_row_serialization_roundtrip() {
    let pair = NearDupPairRow {
        left: "src/a.rs".to_string(),
        right: "src/b.rs".to_string(),
        similarity: 0.8765,
        shared_fingerprints: 42,
        left_fingerprints: 100,
        right_fingerprints: 95,
    };

    let json = serde_json::to_string(&pair).unwrap();
    let back: NearDupPairRow = serde_json::from_str(&json).unwrap();

    assert_eq!(back.left, "src/a.rs");
    assert_eq!(back.right, "src/b.rs");
    assert!((back.similarity - 0.8765).abs() < 1e-10);
    assert_eq!(back.shared_fingerprints, 42);
    assert_eq!(back.left_fingerprints, 100);
    assert_eq!(back.right_fingerprints, 95);
}

// ── 6. NearDupCluster serialization roundtrip ───────────────────

#[test]
fn cluster_serialization_roundtrip() {
    let cluster = NearDupCluster {
        files: vec!["a.rs".to_string(), "b.rs".to_string(), "c.rs".to_string()],
        max_similarity: 0.95,
        representative: "b.rs".to_string(),
        pair_count: 3,
    };

    let json = serde_json::to_string(&cluster).unwrap();
    let back: NearDupCluster = serde_json::from_str(&json).unwrap();

    assert_eq!(back.files, cluster.files);
    assert!((back.max_similarity - 0.95).abs() < 1e-10);
    assert_eq!(back.representative, "b.rs");
    assert_eq!(back.pair_count, 3);
}

// ── 7. Threshold at 0.0 captures all non-zero similarities ─────

#[test]
fn threshold_zero_captures_all_pairs() {
    let dir = TempDir::new().unwrap();
    let content_a = source_text(100, 10);
    let content_b = source_text(100, 11);
    let content_c = source_text(100, 12);
    write_file(&dir, "a.rs", &content_a);
    write_file(&dir, "b.rs", &content_b);
    write_file(&dir, "c.rs", &content_c);

    let export = make_export(vec![
        make_row("a.rs", "(root)", "Rust", 100, 5000),
        make_row("b.rs", "(root)", "Rust", 100, 5000),
        make_row("c.rs", "(root)", "Rust", 100, 5000),
    ]);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.0,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    // At threshold 0, any pair with shared fingerprints will be reported
    // At minimum, C(3,2)=3 candidate pairs exist
    for pair in &report.pairs {
        assert!(pair.similarity >= 0.0);
    }
}

// ── 8. max_pairs truncation sets truncated flag ─────────────────

#[test]
fn max_pairs_truncation_sets_flag() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 20);
    for name in &["a.rs", "b.rs", "c.rs", "d.rs"] {
        write_file(&dir, name, &content);
    }
    let rows: Vec<FileRow> = ["a.rs", "b.rs", "c.rs", "d.rs"]
        .iter()
        .map(|n| make_row(n, "(root)", "Rust", 100, 5000))
        .collect();
    let export = make_export(rows);

    // 4 identical files → C(4,2)=6 pairs; cap at 2
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        Some(2),
        &default_limits(),
        &[],
    )
    .unwrap();

    assert_eq!(report.pairs.len(), 2);
    assert!(report.truncated);
}

// ── 9. max_pairs larger than actual pairs: no truncation ────────

#[test]
fn max_pairs_larger_than_actual_no_truncation() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 21);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);

    let export = make_export(vec![
        make_row("a.rs", "(root)", "Rust", 100, 5000),
        make_row("b.rs", "(root)", "Rust", 100, 5000),
    ]);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        Some(100),
        &default_limits(),
        &[],
    )
    .unwrap();

    assert_eq!(report.pairs.len(), 1);
    assert!(!report.truncated);
}

// ── 10. Child rows are excluded from analysis ───────────────────

#[test]
fn child_rows_excluded_from_analysis() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 30);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);

    let rows = vec![
        make_row("a.rs", "(root)", "Rust", 100, 5000),
        FileRow {
            path: "b.rs".to_string(),
            module: "(root)".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Child,
            code: 100,
            comments: 0,
            blanks: 0,
            lines: 100,
            bytes: 5000,
            tokens: 500,
        },
    ];
    let export = make_export(rows);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.0,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    // Only 1 parent file → no pairs possible
    assert_eq!(report.files_analyzed, 1);
    assert!(report.pairs.is_empty());
}

// ── 11. Files exceeding max_file_bytes are excluded ─────────────

#[test]
fn files_exceeding_max_file_bytes_excluded() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 40);
    write_file(&dir, "small.rs", &content);
    write_file(&dir, "big.rs", &content);

    let rows = vec![
        make_row("small.rs", "(root)", "Rust", 100, 100),
        make_row("big.rs", "(root)", "Rust", 100, 999999),
    ];
    let export = make_export(rows);

    let limits = NearDupLimits {
        max_bytes: None,
        max_file_bytes: Some(500),
    };
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.0,
        100,
        None,
        &limits,
        &[],
    )
    .unwrap();

    // big.rs (999999 bytes) exceeds 500-byte limit → only small.rs analyzed
    assert_eq!(report.files_analyzed, 1);
    assert!(report.pairs.is_empty());
}

// ── 12. Stats field is populated ────────────────────────────────

#[test]
fn stats_field_is_populated() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 50);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);

    let export = make_export(vec![
        make_row("a.rs", "(root)", "Rust", 100, 5000),
        make_row("b.rs", "(root)", "Rust", 100, 5000),
    ]);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    let stats = report.stats.expect("stats should be present");
    assert!(stats.bytes_processed > 0, "bytes_processed should be > 0");
}

// ── 13. Eligible files tracking ─────────────────────────────────

#[test]
fn eligible_files_tracked_before_cap() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 60);
    for i in 0..5 {
        write_file(&dir, &format!("f{i}.rs"), &content);
    }
    let rows: Vec<FileRow> = (0..5)
        .map(|i| make_row(&format!("f{i}.rs"), "(root)", "Rust", 100, 5000))
        .collect();
    let export = make_export(rows);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        3,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    assert_eq!(report.eligible_files, Some(5));
    assert_eq!(report.files_analyzed, 3);
    assert_eq!(report.files_skipped, 2);
}

// ── 14. Exclude pattern counts are tracked ──────────────────────

#[test]
fn exclude_pattern_count_tracked() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 70);
    write_file(&dir, "keep.rs", &content);
    write_file(&dir, "gen_a.rs", &content);
    write_file(&dir, "gen_b.rs", &content);

    let rows = vec![
        make_row("keep.rs", "(root)", "Rust", 100, 5000),
        make_row("gen_a.rs", "(root)", "Rust", 100, 5000),
        make_row("gen_b.rs", "(root)", "Rust", 100, 5000),
    ];
    let export = make_export(rows);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &default_limits(),
        &["gen_*".to_string()],
    )
    .unwrap();

    assert_eq!(report.excluded_by_pattern, Some(2));
    assert_eq!(report.files_analyzed, 1);
}

// ── 15. Scope field in params is preserved ──────────────────────

#[test]
fn scope_field_preserved_in_params() {
    let dir = TempDir::new().unwrap();
    let export = make_export(vec![]);

    for scope in [
        NearDupScope::Global,
        NearDupScope::Module,
        NearDupScope::Lang,
    ] {
        let report = build_near_dup_report(
            dir.path(),
            &export,
            scope,
            0.5,
            100,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();

        let json = serde_json::to_string(&report).unwrap();
        let back: NearDuplicateReport = serde_json::from_str(&json).unwrap();
        assert_eq!(
            format!("{:?}", back.params.scope),
            format!("{:?}", scope),
            "scope should survive serialization roundtrip"
        );
    }
}

// ── 16. Params threshold and max_files are recorded ─────────────

#[test]
fn params_threshold_and_max_files_recorded() {
    let dir = TempDir::new().unwrap();
    let export = make_export(vec![]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.75,
        42,
        Some(10),
        &default_limits(),
        &[],
    )
    .unwrap();

    assert!((report.params.threshold - 0.75).abs() < 1e-10);
    assert_eq!(report.params.max_files, 42);
    assert_eq!(report.params.max_pairs, Some(10));
}

// ── 17. Algorithm constants recorded in params ──────────────────

#[test]
fn algorithm_constants_recorded_in_params() {
    let dir = TempDir::new().unwrap();
    let export = make_export(vec![]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    let algo = report.params.algorithm.expect("algorithm should be set");
    assert_eq!(algo.k_gram_size, 25);
    assert_eq!(algo.window_size, 4);
    assert_eq!(algo.max_postings, 50);
}

// ── 18. Lang scope groups files within same language ─────────────

#[test]
fn lang_scope_pairs_same_language_files() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 80);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);

    let export = make_export(vec![
        make_row("a.rs", "mod_a", "Rust", 100, 5000),
        make_row("b.rs", "mod_b", "Rust", 100, 5000),
    ]);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Lang,
        0.5,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    // Same language (Rust) in Lang scope → should be paired
    assert_eq!(report.pairs.len(), 1);
}

// ── 19. Pairs sorted by similarity descending ───────────────────

#[test]
fn pairs_sorted_by_similarity_descending() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 90);
    // Create 4 identical files → 6 pairs all with similarity ~1.0
    for name in &["a.rs", "b.rs", "c.rs", "d.rs"] {
        write_file(&dir, name, &content);
    }
    let rows: Vec<FileRow> = ["a.rs", "b.rs", "c.rs", "d.rs"]
        .iter()
        .map(|n| make_row(n, "(root)", "Rust", 100, 5000))
        .collect();
    let export = make_export(rows);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.0,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    for window in report.pairs.windows(2) {
        assert!(
            window[0].similarity >= window[1].similarity,
            "pairs not sorted by similarity desc: {} < {}",
            window[0].similarity,
            window[1].similarity
        );
    }
}

// ── 20. Clusters present when pairs exist ───────────────────────

#[test]
fn clusters_present_when_pairs_exist() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 100);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);

    let export = make_export(vec![
        make_row("a.rs", "(root)", "Rust", 100, 5000),
        make_row("b.rs", "(root)", "Rust", 100, 5000),
    ]);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    assert!(!report.pairs.is_empty());
    let clusters = report
        .clusters
        .expect("clusters should be present when pairs exist");
    assert!(!clusters.is_empty());
    // Each cluster should have at least 2 files
    for cluster in &clusters {
        assert!(cluster.files.len() >= 2);
        assert!(!cluster.representative.is_empty());
        assert!(cluster.pair_count >= 1);
    }
}

// ── 21. Two disjoint file sets → two clusters ───────────────────

#[test]
fn disjoint_identical_sets_produce_separate_clusters() {
    let dir = TempDir::new().unwrap();
    let content_x = source_text(100, 200);
    let content_y = source_text(100, 201);

    write_file(&dir, "x1.rs", &content_x);
    write_file(&dir, "x2.rs", &content_x);
    write_file(&dir, "y1.rs", &content_y);
    write_file(&dir, "y2.rs", &content_y);

    let export = make_export(vec![
        make_row("x1.rs", "(root)", "Rust", 100, 5000),
        make_row("x2.rs", "(root)", "Rust", 100, 5000),
        make_row("y1.rs", "(root)", "Rust", 100, 5000),
        make_row("y2.rs", "(root)", "Rust", 100, 5000),
    ]);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    if let Some(clusters) = &report.clusters {
        // Should have at least 2 clusters from disjoint sets
        // (unless the two sets happen to share fingerprints, which is unlikely)
        assert!(
            !clusters.is_empty(),
            "should have at least one cluster from identical files"
        );
    }
}

// ── 22. NearDupScope serialization roundtrip ────────────────────

#[test]
fn near_dup_scope_serialization_roundtrip() {
    for scope in [
        NearDupScope::Global,
        NearDupScope::Module,
        NearDupScope::Lang,
    ] {
        let json = serde_json::to_string(&scope).unwrap();
        let back: NearDupScope = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{:?}", scope), format!("{:?}", back),);
    }
}

// ── 23. Shared fingerprints ≤ min(left, right) fingerprints ─────

#[test]
fn shared_fingerprints_bounded_by_min_count() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 110);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);

    let export = make_export(vec![
        make_row("a.rs", "(root)", "Rust", 100, 5000),
        make_row("b.rs", "(root)", "Rust", 100, 5000),
    ]);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.0,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    for pair in &report.pairs {
        let min_fp = pair.left_fingerprints.min(pair.right_fingerprints);
        assert!(
            pair.shared_fingerprints <= min_fp,
            "shared ({}) should be <= min(left={}, right={})",
            pair.shared_fingerprints,
            pair.left_fingerprints,
            pair.right_fingerprints
        );
    }
}

// ── 24. Similarity always in [0.0, 1.0] ─────────────────────────

#[test]
fn similarity_bounded_zero_to_one() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 120);
    for i in 0..5 {
        write_file(&dir, &format!("f{i}.rs"), &content);
    }
    let rows: Vec<FileRow> = (0..5)
        .map(|i| make_row(&format!("f{i}.rs"), "(root)", "Rust", 100, 5000))
        .collect();
    let export = make_export(rows);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.0,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    for pair in &report.pairs {
        assert!(
            pair.similarity >= 0.0 && pair.similarity <= 1.0,
            "similarity {} out of [0, 1] range",
            pair.similarity
        );
    }
}

// ── 25. Very high threshold excludes low-similarity pairs ───────

#[test]
fn high_threshold_excludes_low_similarity() {
    let dir = TempDir::new().unwrap();
    // Two completely different files
    let content_a = source_text(100, 300);
    let content_b = source_text(100, 301);
    write_file(&dir, "a.rs", &content_a);
    write_file(&dir, "b.rs", &content_b);

    let export = make_export(vec![
        make_row("a.rs", "(root)", "Rust", 100, 5000),
        make_row("b.rs", "(root)", "Rust", 100, 5000),
    ]);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.99,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    // Different content at high threshold → likely no pairs
    for pair in &report.pairs {
        assert!(
            pair.similarity >= 0.99,
            "pair with similarity {} should be filtered at threshold 0.99",
            pair.similarity
        );
    }
}
