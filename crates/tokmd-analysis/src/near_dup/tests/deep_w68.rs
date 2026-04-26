//! Deep tests for analysis near-duplicate module (w68).

use crate::near_dup::{NearDupLimits, build_near_dup_report};
use std::fs;
use tempfile::tempdir;
use tokmd_analysis_types::NearDupScope;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

fn make_row(path: &str, lang: &str, module: &str, code: usize, bytes: usize) -> FileRow {
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

fn default_limits() -> NearDupLimits {
    NearDupLimits {
        max_bytes: None,
        max_file_bytes: Some(512_000),
    }
}

/// Generate reproducible source text with N distinct tokens.
fn gen_source(n: usize, seed: usize) -> String {
    (0..n)
        .map(|i| format!("token_{}_{}", seed, i))
        .collect::<Vec<_>>()
        .join(" ")
}

// ---------------------------------------------------------------------------
// Basic: identical files produce high similarity
// ---------------------------------------------------------------------------

#[test]
fn identical_files_detected() {
    let dir = tempdir().unwrap();
    let body = gen_source(100, 0);
    fs::write(dir.path().join("a.rs"), &body).unwrap();
    fs::write(dir.path().join("b.rs"), &body).unwrap();
    let export = make_export(vec![
        make_row("a.rs", "Rust", "src", 100, body.len()),
        make_row("b.rs", "Rust", "src", 100, body.len()),
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
    assert!(
        !report.pairs.is_empty(),
        "identical files should produce pairs"
    );
    assert!(report.pairs[0].similarity >= 0.99);
}

// ---------------------------------------------------------------------------
// Completely different files produce no pairs
// ---------------------------------------------------------------------------

#[test]
fn different_files_no_pairs() {
    let dir = tempdir().unwrap();
    let a = gen_source(100, 1);
    let b = gen_source(100, 9999);
    fs::write(dir.path().join("a.rs"), &a).unwrap();
    fs::write(dir.path().join("b.rs"), &b).unwrap();
    let export = make_export(vec![
        make_row("a.rs", "Rust", "src", 100, a.len()),
        make_row("b.rs", "Rust", "src", 100, b.len()),
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
    assert!(
        report.pairs.is_empty(),
        "completely different files should not be paired"
    );
}

// ---------------------------------------------------------------------------
// Empty export produces empty report
// ---------------------------------------------------------------------------

#[test]
fn empty_export_empty_report() {
    let dir = tempdir().unwrap();
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
    assert_eq!(report.files_analyzed, 0);
    assert!(report.pairs.is_empty());
    assert!(report.clusters.is_none());
}

// ---------------------------------------------------------------------------
// Single file: nothing to compare
// ---------------------------------------------------------------------------

#[test]
fn single_file_no_pairs() {
    let dir = tempdir().unwrap();
    let body = gen_source(100, 42);
    fs::write(dir.path().join("only.rs"), &body).unwrap();
    let export = make_export(vec![make_row("only.rs", "Rust", "src", 100, body.len())]);
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
}

// ---------------------------------------------------------------------------
// Pairs sorted by similarity desc
// ---------------------------------------------------------------------------

#[test]
fn pairs_sorted_by_similarity_desc() {
    let dir = tempdir().unwrap();
    let base = gen_source(100, 0);
    // a == b (identical), c shares ~50% with a
    let half: String = base
        .split_whitespace()
        .take(50)
        .collect::<Vec<_>>()
        .join(" ");
    let other_half = gen_source(50, 7777);
    let c_body = format!("{half} {other_half}");
    fs::write(dir.path().join("a.rs"), &base).unwrap();
    fs::write(dir.path().join("b.rs"), &base).unwrap();
    fs::write(dir.path().join("c.rs"), &c_body).unwrap();
    let export = make_export(vec![
        make_row("a.rs", "Rust", "src", 100, base.len()),
        make_row("b.rs", "Rust", "src", 100, base.len()),
        make_row("c.rs", "Rust", "src", 100, c_body.len()),
    ]);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.1,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();
    for w in report.pairs.windows(2) {
        assert!(w[0].similarity >= w[1].similarity);
    }
}

// ---------------------------------------------------------------------------
// Truncation via max_pairs
// ---------------------------------------------------------------------------

#[test]
fn max_pairs_truncates() {
    let dir = tempdir().unwrap();
    let body = gen_source(100, 0);
    // 3 identical files = 3 pairs
    for name in &["a.rs", "b.rs", "c.rs"] {
        fs::write(dir.path().join(name), &body).unwrap();
    }
    let export = make_export(vec![
        make_row("a.rs", "Rust", "src", 100, body.len()),
        make_row("b.rs", "Rust", "src", 100, body.len()),
        make_row("c.rs", "Rust", "src", 100, body.len()),
    ]);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        Some(1),
        &default_limits(),
        &[],
    )
    .unwrap();
    assert!(report.pairs.len() <= 1);
    assert!(report.truncated);
}

// ---------------------------------------------------------------------------
// Exclude patterns filter files
// ---------------------------------------------------------------------------

#[test]
fn exclude_patterns_filter_files() {
    let dir = tempdir().unwrap();
    let body = gen_source(100, 0);
    fs::write(dir.path().join("a.rs"), &body).unwrap();
    fs::write(dir.path().join("b.rs"), &body).unwrap();
    let export = make_export(vec![
        make_row("a.rs", "Rust", "src", 100, body.len()),
        make_row("b.rs", "Rust", "src", 100, body.len()),
    ]);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &default_limits(),
        &["a.*".to_string()],
    )
    .unwrap();
    // After excluding a.rs, only b.rs left => no pairs
    assert!(report.pairs.is_empty());
    assert_eq!(report.excluded_by_pattern, Some(1));
}

// ---------------------------------------------------------------------------
// max_files cap
// ---------------------------------------------------------------------------

#[test]
fn max_files_caps_analysis() {
    let dir = tempdir().unwrap();
    let body = gen_source(100, 0);
    for name in &["a.rs", "b.rs", "c.rs"] {
        fs::write(dir.path().join(name), &body).unwrap();
    }
    let export = make_export(vec![
        make_row("a.rs", "Rust", "src", 100, body.len()),
        make_row("b.rs", "Rust", "src", 100, body.len()),
        make_row("c.rs", "Rust", "src", 100, body.len()),
    ]);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        2, // only analyze 2 files
        None,
        &default_limits(),
        &[],
    )
    .unwrap();
    assert_eq!(report.files_analyzed, 2);
    assert_eq!(report.files_skipped, 1);
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn deterministic_across_runs() {
    let dir = tempdir().unwrap();
    let body = gen_source(100, 0);
    fs::write(dir.path().join("a.rs"), &body).unwrap();
    fs::write(dir.path().join("b.rs"), &body).unwrap();
    let export = make_export(vec![
        make_row("a.rs", "Rust", "src", 100, body.len()),
        make_row("b.rs", "Rust", "src", 100, body.len()),
    ]);
    let r1 = build_near_dup_report(
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
    let r2 = build_near_dup_report(
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
    assert_eq!(r1.pairs.len(), r2.pairs.len());
    for (a, b) in r1.pairs.iter().zip(&r2.pairs) {
        assert_eq!(a.left, b.left);
        assert_eq!(a.right, b.right);
        assert_eq!(a.similarity, b.similarity);
    }
}

// ---------------------------------------------------------------------------
// Scope: Module isolates comparisons
// ---------------------------------------------------------------------------

#[test]
fn module_scope_isolates() {
    let dir = tempdir().unwrap();
    let body = gen_source(100, 0);
    fs::write(dir.path().join("a.rs"), &body).unwrap();
    fs::write(dir.path().join("b.rs"), &body).unwrap();
    let export = make_export(vec![
        make_row("a.rs", "Rust", "mod_a", 100, body.len()),
        make_row("b.rs", "Rust", "mod_b", 100, body.len()),
    ]);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Module,
        0.5,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();
    // Different modules => no cross-module comparison
    assert!(report.pairs.is_empty());
}

// ---------------------------------------------------------------------------
// Scope: Lang isolates comparisons
// ---------------------------------------------------------------------------

#[test]
fn lang_scope_isolates() {
    let dir = tempdir().unwrap();
    let body = gen_source(100, 0);
    fs::write(dir.path().join("a.rs"), &body).unwrap();
    fs::write(dir.path().join("b.py"), &body).unwrap();
    let export = make_export(vec![
        make_row("a.rs", "Rust", "src", 100, body.len()),
        make_row("b.py", "Python", "src", 100, body.len()),
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
    assert!(report.pairs.is_empty());
}

// ---------------------------------------------------------------------------
// Clusters produced from pairs
// ---------------------------------------------------------------------------

#[test]
fn clusters_formed_from_identical_files() {
    let dir = tempdir().unwrap();
    let body = gen_source(100, 0);
    for name in &["a.rs", "b.rs", "c.rs"] {
        fs::write(dir.path().join(name), &body).unwrap();
    }
    let export = make_export(vec![
        make_row("a.rs", "Rust", "src", 100, body.len()),
        make_row("b.rs", "Rust", "src", 100, body.len()),
        make_row("c.rs", "Rust", "src", 100, body.len()),
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
    assert!(report.clusters.is_some());
    let clusters = report.clusters.unwrap();
    // All 3 identical files should form 1 cluster
    assert_eq!(clusters.len(), 1);
    assert_eq!(clusters[0].files.len(), 3);
}

// ---------------------------------------------------------------------------
// Child rows are excluded
// ---------------------------------------------------------------------------

#[test]
fn child_rows_excluded() {
    let dir = tempdir().unwrap();
    let body = gen_source(100, 0);
    fs::write(dir.path().join("a.rs"), &body).unwrap();
    fs::write(dir.path().join("b.rs"), &body).unwrap();
    let export = make_export(vec![
        make_row("a.rs", "Rust", "src", 100, body.len()),
        FileRow {
            path: "b.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Child,
            code: 100,
            comments: 0,
            blanks: 0,
            lines: 100,
            bytes: body.len(),
            tokens: 500,
        },
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
    // Only 1 parent file => no pairs possible
    assert!(report.pairs.is_empty());
}

// ---------------------------------------------------------------------------
// Files over max_file_bytes excluded
// ---------------------------------------------------------------------------

#[test]
fn large_files_excluded_by_byte_limit() {
    let dir = tempdir().unwrap();
    let body = gen_source(100, 0);
    fs::write(dir.path().join("a.rs"), &body).unwrap();
    fs::write(dir.path().join("b.rs"), &body).unwrap();
    let export = make_export(vec![
        make_row("a.rs", "Rust", "src", 100, body.len()),
        // Declare b.rs as very large in the export row
        make_row("b.rs", "Rust", "src", 100, 1_000_000),
    ]);
    let limits = NearDupLimits {
        max_bytes: None,
        max_file_bytes: Some(512_000),
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
    // b.rs is over the byte limit so only 1 file eligible => no pairs
    assert!(report.pairs.is_empty());
}

// ---------------------------------------------------------------------------
// Threshold boundary: similarity exactly at threshold
// ---------------------------------------------------------------------------

#[test]
fn threshold_at_one_requires_perfect_match() {
    let dir = tempdir().unwrap();
    let body = gen_source(100, 0);
    fs::write(dir.path().join("a.rs"), &body).unwrap();
    fs::write(dir.path().join("b.rs"), &body).unwrap();
    let export = make_export(vec![
        make_row("a.rs", "Rust", "src", 100, body.len()),
        make_row("b.rs", "Rust", "src", 100, body.len()),
    ]);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        1.0,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();
    // Identical files should have similarity == 1.0 and meet threshold
    assert!(!report.pairs.is_empty());
    assert!((report.pairs[0].similarity - 1.0).abs() < 1e-4);
}
