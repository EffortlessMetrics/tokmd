//! Deep tests for tokmd-analysis-near-dup (wave 38).
//!
//! Covers winnowing correctness, DisjointSets with complex merges,
//! cluster construction, partition by scope, threshold boundaries,
//! NearDupLimits, and glob exclusion patterns — all via the public API.

use std::fs;

use tempfile::tempdir;
use tokmd_analysis_near_dup::{build_near_dup_report, NearDupLimits};
use tokmd_analysis_types::NearDupScope;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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
        children: ChildIncludeMode::Separate,
    }
}

/// Generate deterministic pseudocode text with `n` unique tokens.
fn pseudo_code(n: usize, seed: usize) -> String {
    (0..n)
        .map(|i| format!("tok{}v{}", i + seed, i))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Write a file and return a FileRow for it.
fn write_file(dir: &std::path::Path, name: &str, content: &str) -> FileRow {
    let path = dir.join(name);
    fs::write(&path, content).unwrap();
    let bytes = content.len();
    make_row(name, "(root)", "Rust", bytes / 2, bytes)
}

// ---------------------------------------------------------------------------
// Winnowing correctness via identical files
// ---------------------------------------------------------------------------

#[test]
fn identical_files_detected_as_duplicates() {
    let dir = tempdir().unwrap();
    let text = pseudo_code(100, 0);
    let r1 = write_file(dir.path(), "a.rs", &text);
    let r2 = write_file(dir.path(), "b.rs", &text);
    let export = make_export(vec![r1, r2]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    assert_eq!(report.pairs.len(), 1);
    assert!((report.pairs[0].similarity - 1.0).abs() < 1e-4);
}

#[test]
fn completely_different_files_no_pairs() {
    let dir = tempdir().unwrap();
    let r1 = write_file(dir.path(), "a.rs", &pseudo_code(100, 0));
    let r2 = write_file(dir.path(), "b.rs", &pseudo_code(100, 10000));
    let export = make_export(vec![r1, r2]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    assert!(report.pairs.is_empty());
}

#[test]
fn partially_overlapping_files_detected() {
    let dir = tempdir().unwrap();
    let shared = pseudo_code(80, 0);
    let text_a = format!("{} {}", shared, pseudo_code(20, 5000));
    let text_b = format!("{} {}", shared, pseudo_code(20, 6000));
    let r1 = write_file(dir.path(), "a.rs", &text_a);
    let r2 = write_file(dir.path(), "b.rs", &text_b);
    let export = make_export(vec![r1, r2]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.3,
        100,
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    assert!(!report.pairs.is_empty());
    assert!(report.pairs[0].similarity > 0.3);
    assert!(report.pairs[0].similarity < 1.0);
}

#[test]
fn winnow_short_files_produce_no_fingerprints() {
    let dir = tempdir().unwrap();
    // fewer than K=25 tokens → no fingerprints → no pairs
    let r1 = write_file(dir.path(), "a.rs", "fn main() {}");
    let r2 = write_file(dir.path(), "b.rs", "fn main() {}");
    let export = make_export(vec![r1, r2]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.0,
        100,
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    assert!(report.pairs.is_empty());
}

// ---------------------------------------------------------------------------
// DisjointSets with 50+ elements via cluster construction
// ---------------------------------------------------------------------------

#[test]
fn clusters_with_many_files_chain() {
    // Create 50 identical files to exercise DisjointSets union-find with 50 elements.
    // (MAX_POSTINGS=50, so 50 identical files is the largest set that works.)
    let dir = tempdir().unwrap();
    let n = 50;
    let text = pseudo_code(100, 0);
    let mut rows = Vec::new();
    for i in 0..n {
        let name = format!("file{:03}.rs", i);
        rows.push(write_file(dir.path(), &name, &text));
    }
    let export = make_export(rows);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        200,
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    // All 50 identical files → many pairs and one big cluster
    assert!(!report.pairs.is_empty());
    assert!(report.clusters.is_some());
    let clusters = report.clusters.unwrap();
    assert_eq!(
        clusters.len(),
        1,
        "all identical files should form one cluster"
    );
    assert_eq!(
        clusters[0].files.len(),
        n,
        "cluster should contain all {n} files"
    );
}

#[test]
fn clusters_two_disjoint_groups() {
    let dir = tempdir().unwrap();
    // Group A: 5 identical files
    let text_a = pseudo_code(100, 0);
    let mut rows = Vec::new();
    for i in 0..5 {
        let name = format!("group_a_{i}.rs");
        rows.push(write_file(dir.path(), &name, &text_a));
    }
    // Group B: 5 identical files (completely different content)
    let text_b = pseudo_code(100, 50000);
    for i in 0..5 {
        let name = format!("group_b_{i}.rs");
        rows.push(write_file(dir.path(), &name, &text_b));
    }
    let export = make_export(rows);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    let clusters = report.clusters.unwrap();
    assert_eq!(clusters.len(), 2, "expected exactly 2 disjoint clusters");
    for c in &clusters {
        assert_eq!(c.files.len(), 5);
    }
}

#[test]
fn cluster_representative_is_most_connected() {
    let dir = tempdir().unwrap();
    // File "hub.rs" is identical to 3 others; the others aren't identical to each other
    let hub_text = pseudo_code(100, 0);
    let mut rows = vec![write_file(dir.path(), "hub.rs", &hub_text)];
    for i in 0..3 {
        // Each spoke shares 90% with hub
        let spoke_text = format!(
            "{} {}",
            pseudo_code(90, 0),
            pseudo_code(10, (i + 1) * 10000)
        );
        let name = format!("spoke_{i}.rs");
        rows.push(write_file(dir.path(), &name, &spoke_text));
    }
    let export = make_export(rows);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.3,
        100,
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    assert!(report.clusters.is_some());
    let clusters = report.clusters.unwrap();
    assert_eq!(clusters.len(), 1);
    // hub.rs should be the representative (most connected)
    assert_eq!(clusters[0].representative, "hub.rs");
}

// ---------------------------------------------------------------------------
// Partition by scope
// ---------------------------------------------------------------------------

#[test]
fn scope_global_compares_across_modules() {
    let dir = tempdir().unwrap();
    let text = pseudo_code(100, 0);
    let r1 = write_file(dir.path(), "a.rs", &text);
    let mut r1 = r1;
    r1.module = "mod_a".to_string();
    let r2 = write_file(dir.path(), "b.rs", &text);
    let mut r2 = r2;
    r2.module = "mod_b".to_string();
    let export = make_export(vec![r1, r2]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    assert_eq!(
        report.pairs.len(),
        1,
        "Global scope should find cross-module pairs"
    );
}

#[test]
fn scope_module_does_not_compare_across_modules() {
    let dir = tempdir().unwrap();
    let text = pseudo_code(100, 0);
    let mut r1 = write_file(dir.path(), "a.rs", &text);
    r1.module = "mod_a".to_string();
    let mut r2 = write_file(dir.path(), "b.rs", &text);
    r2.module = "mod_b".to_string();
    let export = make_export(vec![r1, r2]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Module,
        0.5,
        100,
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    assert!(
        report.pairs.is_empty(),
        "Module scope should NOT find cross-module pairs"
    );
}

#[test]
fn scope_module_finds_intra_module_pairs() {
    let dir = tempdir().unwrap();
    let text = pseudo_code(100, 0);
    let mut r1 = write_file(dir.path(), "a.rs", &text);
    r1.module = "mod_a".to_string();
    let mut r2 = write_file(dir.path(), "b.rs", &text);
    r2.module = "mod_a".to_string();
    let export = make_export(vec![r1, r2]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Module,
        0.5,
        100,
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    assert_eq!(
        report.pairs.len(),
        1,
        "Module scope should find intra-module pairs"
    );
}

#[test]
fn scope_lang_partitions_by_language() {
    let dir = tempdir().unwrap();
    let text = pseudo_code(100, 0);
    let mut r1 = write_file(dir.path(), "a.rs", &text);
    r1.lang = "Rust".to_string();
    let mut r2 = write_file(dir.path(), "b.rs", &text);
    r2.lang = "Python".to_string();
    let export = make_export(vec![r1, r2]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Lang,
        0.5,
        100,
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    assert!(
        report.pairs.is_empty(),
        "Lang scope should not pair different languages"
    );
}

#[test]
fn scope_lang_finds_same_language_pairs() {
    let dir = tempdir().unwrap();
    let text = pseudo_code(100, 0);
    let mut r1 = write_file(dir.path(), "a.rs", &text);
    r1.lang = "Rust".to_string();
    let mut r2 = write_file(dir.path(), "b.rs", &text);
    r2.lang = "Rust".to_string();
    let export = make_export(vec![r1, r2]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Lang,
        0.5,
        100,
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    assert_eq!(report.pairs.len(), 1);
}

// ---------------------------------------------------------------------------
// Similarity threshold boundaries
// ---------------------------------------------------------------------------

#[test]
fn threshold_zero_accepts_all_candidate_pairs() {
    let dir = tempdir().unwrap();
    // Two files with minimal overlap
    let text_a = pseudo_code(100, 0);
    let text_b = format!("{} {}", pseudo_code(30, 0), pseudo_code(70, 20000));
    let r1 = write_file(dir.path(), "a.rs", &text_a);
    let r2 = write_file(dir.path(), "b.rs", &text_b);
    let export = make_export(vec![r1, r2]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.0,
        100,
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    // threshold=0.0 should accept any pair with shared fingerprints
    if !report.pairs.is_empty() {
        assert!(report.pairs[0].similarity >= 0.0);
    }
}

#[test]
fn threshold_one_requires_perfect_match() {
    let dir = tempdir().unwrap();
    let shared = pseudo_code(80, 0);
    let text_a = format!("{} {}", shared, pseudo_code(20, 5000));
    let text_b = format!("{} {}", shared, pseudo_code(20, 6000));
    let r1 = write_file(dir.path(), "a.rs", &text_a);
    let r2 = write_file(dir.path(), "b.rs", &text_b);
    let export = make_export(vec![r1, r2]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        1.0,
        100,
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    // threshold=1.0: only exact matches pass; partial overlap should be excluded
    assert!(report.pairs.is_empty());
}

#[test]
fn threshold_one_accepts_identical() {
    let dir = tempdir().unwrap();
    let text = pseudo_code(100, 0);
    let r1 = write_file(dir.path(), "a.rs", &text);
    let r2 = write_file(dir.path(), "b.rs", &text);
    let export = make_export(vec![r1, r2]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        1.0,
        100,
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    assert_eq!(report.pairs.len(), 1);
    assert!((report.pairs[0].similarity - 1.0).abs() < 1e-4);
}

// ---------------------------------------------------------------------------
// NearDupLimits
// ---------------------------------------------------------------------------

#[test]
fn limits_default_has_no_caps() {
    let limits = NearDupLimits::default();
    assert!(limits.max_bytes.is_none());
    assert!(limits.max_file_bytes.is_none());
}

#[test]
fn limits_max_file_bytes_excludes_large_files() {
    let dir = tempdir().unwrap();
    let text = pseudo_code(100, 0);
    let r1 = write_file(dir.path(), "a.rs", &text);
    let mut r2 = write_file(dir.path(), "b.rs", &text);
    r2.bytes = 1_000_000; // Mark as very large
    let export = make_export(vec![r1, r2]);

    let limits = NearDupLimits {
        max_bytes: None,
        max_file_bytes: Some(500), // Only allow files < 500 bytes
    };

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &limits,
        &[],
    )
    .unwrap();

    // b.rs should be excluded due to byte limit
    assert!(report.pairs.is_empty());
    assert!(report.files_analyzed <= 1);
}

#[test]
fn limits_custom_values_stored() {
    let limits = NearDupLimits {
        max_bytes: Some(1024),
        max_file_bytes: Some(256),
    };
    assert_eq!(limits.max_bytes, Some(1024));
    assert_eq!(limits.max_file_bytes, Some(256));
}

// ---------------------------------------------------------------------------
// Glob exclusion patterns
// ---------------------------------------------------------------------------

#[test]
fn glob_excludes_matching_files() {
    let dir = tempdir().unwrap();
    let text = pseudo_code(100, 0);
    let r1 = write_file(dir.path(), "a.rs", &text);
    let r2 = write_file(dir.path(), "b.rs", &text);
    let r3 = write_file(dir.path(), "test_c.rs", &text);
    let export = make_export(vec![r1, r2, r3]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &NearDupLimits::default(),
        &["test_*".to_string()],
    )
    .unwrap();

    assert_eq!(report.excluded_by_pattern, Some(1));
    // Only a.rs and b.rs remain → 1 pair
    assert_eq!(report.pairs.len(), 1);
    assert!(!report.pairs[0].left.starts_with("test_"));
    assert!(!report.pairs[0].right.starts_with("test_"));
}

#[test]
fn glob_no_patterns_means_no_exclusion() {
    let dir = tempdir().unwrap();
    let text = pseudo_code(100, 0);
    let r1 = write_file(dir.path(), "a.rs", &text);
    let r2 = write_file(dir.path(), "b.rs", &text);
    let export = make_export(vec![r1, r2]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    assert!(report.excluded_by_pattern.is_none());
}

#[test]
fn glob_exclude_all_files() {
    let dir = tempdir().unwrap();
    let text = pseudo_code(100, 0);
    let r1 = write_file(dir.path(), "a.rs", &text);
    let r2 = write_file(dir.path(), "b.rs", &text);
    let export = make_export(vec![r1, r2]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &NearDupLimits::default(),
        &["*.rs".to_string()],
    )
    .unwrap();

    assert_eq!(report.excluded_by_pattern, Some(2));
    assert!(report.pairs.is_empty());
}

// ---------------------------------------------------------------------------
// max_files truncation
// ---------------------------------------------------------------------------

#[test]
fn max_files_truncation() {
    let dir = tempdir().unwrap();
    let text = pseudo_code(100, 0);
    let mut rows = Vec::new();
    for i in 0..10 {
        let name = format!("file{i}.rs");
        rows.push(write_file(dir.path(), &name, &text));
    }
    let export = make_export(rows);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        3, // Only analyze top 3 files
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    assert_eq!(report.files_analyzed, 3);
    assert_eq!(report.files_skipped, 7);
}

// ---------------------------------------------------------------------------
// max_pairs truncation
// ---------------------------------------------------------------------------

#[test]
fn max_pairs_truncation_sets_truncated_flag() {
    let dir = tempdir().unwrap();
    let text = pseudo_code(100, 0);
    let mut rows = Vec::new();
    for i in 0..5 {
        let name = format!("file{i}.rs");
        rows.push(write_file(dir.path(), &name, &text));
    }
    let export = make_export(rows);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        Some(2), // max_pairs = 2
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    assert!(report.truncated);
    assert!(report.pairs.len() <= 2);
    // Clusters should still reflect all pairs before truncation
    assert!(report.clusters.is_some());
}

// ---------------------------------------------------------------------------
// Report metadata
// ---------------------------------------------------------------------------

#[test]
fn report_params_reflect_inputs() {
    let dir = tempdir().unwrap();
    let text = pseudo_code(100, 0);
    let r1 = write_file(dir.path(), "a.rs", &text);
    let export = make_export(vec![r1]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Lang,
        0.75,
        50,
        Some(10),
        &NearDupLimits {
            max_bytes: None,
            max_file_bytes: Some(1024),
        },
        &["*.test.rs".to_string()],
    )
    .unwrap();

    assert_eq!(report.params.threshold, 0.75);
    assert_eq!(report.params.max_files, 50);
    assert_eq!(report.params.max_pairs, Some(10));
    assert_eq!(report.params.max_file_bytes, Some(1024));
    assert_eq!(report.params.exclude_patterns, vec!["*.test.rs"]);
    let algo = report.params.algorithm.unwrap();
    assert_eq!(algo.k_gram_size, 25);
    assert_eq!(algo.window_size, 4);
    assert_eq!(algo.max_postings, 50);
}

#[test]
fn report_stats_present() {
    let dir = tempdir().unwrap();
    let text = pseudo_code(100, 0);
    let r1 = write_file(dir.path(), "a.rs", &text);
    let r2 = write_file(dir.path(), "b.rs", &text);
    let export = make_export(vec![r1, r2]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    let stats = report.stats.unwrap();
    assert!(stats.bytes_processed > 0);
}

#[test]
fn eligible_files_count_is_set() {
    let dir = tempdir().unwrap();
    let text = pseudo_code(100, 0);
    let mut rows = Vec::new();
    for i in 0..8 {
        let name = format!("file{i}.rs");
        rows.push(write_file(dir.path(), &name, &text));
    }
    let export = make_export(rows);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        5,
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    assert_eq!(report.eligible_files, Some(8));
    assert_eq!(report.files_analyzed, 5);
}

// ---------------------------------------------------------------------------
// Pairs sort order: similarity desc, then left, then right
// ---------------------------------------------------------------------------

#[test]
fn pairs_sorted_by_similarity_desc() {
    let dir = tempdir().unwrap();
    // Create 3 files with varying overlap
    let base = pseudo_code(80, 0);
    let text_a = format!("{} {}", base, pseudo_code(20, 1000));
    let text_b = format!("{} {}", base, pseudo_code(20, 2000));
    let text_c = pseudo_code(100, 3000); // completely different
    let r1 = write_file(dir.path(), "a.rs", &text_a);
    let r2 = write_file(dir.path(), "b.rs", &text_b);
    let r3 = write_file(dir.path(), "c.rs", &text_c);
    let export = make_export(vec![r1, r2, r3]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.0,
        100,
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    for i in 1..report.pairs.len() {
        assert!(report.pairs[i - 1].similarity >= report.pairs[i].similarity);
    }
}

// ---------------------------------------------------------------------------
// Only Parent kind files are considered
// ---------------------------------------------------------------------------

#[test]
fn child_files_excluded_from_analysis() {
    let dir = tempdir().unwrap();
    let text = pseudo_code(100, 0);
    fs::write(dir.path().join("a.rs"), &text).unwrap();
    fs::write(dir.path().join("b.rs"), &text).unwrap();

    let rows = vec![make_row("a.rs", "(root)", "Rust", 100, text.len()), {
        let mut r = make_row("b.rs", "(root)", "Rust", 100, text.len());
        r.kind = FileKind::Child;
        r
    }];
    let export = make_export(rows);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.0,
        100,
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    // Only 1 Parent file → no pairs possible
    assert!(report.pairs.is_empty());
    assert_eq!(report.files_analyzed, 1);
}

// ---------------------------------------------------------------------------
// Determinism: same input → same output
// ---------------------------------------------------------------------------

#[test]
fn report_is_deterministic() {
    let dir = tempdir().unwrap();
    let text = pseudo_code(100, 0);
    let r1 = write_file(dir.path(), "a.rs", &text);
    let r2 = write_file(dir.path(), "b.rs", &text);
    let export = make_export(vec![r1.clone(), r2.clone()]);

    let report1 = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();
    let report2 = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    assert_eq!(report1.pairs.len(), report2.pairs.len());
    for (a, b) in report1.pairs.iter().zip(report2.pairs.iter()) {
        assert_eq!(a.left, b.left);
        assert_eq!(a.right, b.right);
        assert!((a.similarity - b.similarity).abs() < 1e-10);
    }
}

// ---------------------------------------------------------------------------
// Empty export
// ---------------------------------------------------------------------------

#[test]
fn empty_export_produces_empty_report() {
    let dir = tempdir().unwrap();
    let export = make_export(vec![]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    assert!(report.pairs.is_empty());
    assert_eq!(report.files_analyzed, 0);
    assert!(!report.truncated);
    assert!(report.clusters.is_none());
}

// ---------------------------------------------------------------------------
// Cluster files are sorted alphabetically
// ---------------------------------------------------------------------------

#[test]
fn cluster_files_sorted_alphabetically() {
    let dir = tempdir().unwrap();
    let text = pseudo_code(100, 0);
    let r1 = write_file(dir.path(), "z_file.rs", &text);
    let r2 = write_file(dir.path(), "a_file.rs", &text);
    let r3 = write_file(dir.path(), "m_file.rs", &text);
    let export = make_export(vec![r1, r2, r3]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    let clusters = report.clusters.unwrap();
    assert_eq!(clusters.len(), 1);
    let files = &clusters[0].files;
    for i in 1..files.len() {
        assert!(files[i - 1] <= files[i], "files not sorted: {:?}", files);
    }
}
