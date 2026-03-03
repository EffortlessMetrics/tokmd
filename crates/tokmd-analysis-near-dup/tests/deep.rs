//! Deep invariant tests for near-duplicate detection.
//!
//! Tests Jaccard similarity semantics, scope partitioning, clustering,
//! threshold boundaries, determinism, and edge cases.

use std::io::Write;
use tempfile::TempDir;
use tokmd_analysis_near_dup::{NearDupLimits, build_near_dup_report};
use tokmd_analysis_types::NearDupScope;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────

fn make_tokens(n: usize, prefix: &str) -> String {
    (0..n)
        .map(|i| format!("{prefix}_{i}"))
        .collect::<Vec<_>>()
        .join(" + ")
}

fn overlapping_content(shared: usize, unique: usize, seed: usize) -> String {
    let shared: Vec<String> = (0..shared).map(|i| format!("common_{i}")).collect();
    let unique: Vec<String> = (0..unique).map(|i| format!("uniq_{seed}_{i}")).collect();
    [shared, unique].concat().join(" + ")
}

fn file_row(path: &str, module: &str, lang: &str, code: usize, bytes: usize) -> FileRow {
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

fn write_to(dir: &TempDir, rel: &str, content: &str) {
    let path = dir.path().join(rel);
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p).unwrap();
    }
    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}

fn mk_export(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

// ── 1. Empty export produces empty report ───────────────────────

#[test]
fn empty_export_yields_no_pairs() {
    let dir = TempDir::new().unwrap();
    let export = mk_export(vec![]);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        Some(1000),
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();
    assert!(report.pairs.is_empty());
    assert!(
        report.clusters.as_ref().map_or(true, |c| c.is_empty()),
        "clusters should be empty or None"
    );
    assert_eq!(report.files_analyzed, 0);
}

// ── 2. Single file produces no pairs ────────────────────────────

#[test]
fn single_file_no_pairs() {
    let dir = TempDir::new().unwrap();
    let content = make_tokens(120, "only");
    write_to(&dir, "solo.rs", &content);
    let export = mk_export(vec![file_row("solo.rs", "(root)", "Rust", 50, 3000)]);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.0,
        100,
        Some(1000),
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();
    assert!(report.pairs.is_empty());
    assert_eq!(report.files_analyzed, 1);
}

// ── 3. Identical files produce similarity ~1.0 ──────────────────

#[test]
fn identical_files_similarity_near_one() {
    let dir = TempDir::new().unwrap();
    let content = make_tokens(150, "ident");
    write_to(&dir, "x.rs", &content);
    write_to(&dir, "y.rs", &content);
    let export = mk_export(vec![
        file_row("x.rs", "(root)", "Rust", 80, 5000),
        file_row("y.rs", "(root)", "Rust", 80, 5000),
    ]);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.0,
        100,
        Some(1000),
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();
    assert_eq!(report.pairs.len(), 1);
    assert!(
        report.pairs[0].similarity > 0.95,
        "identical files should have similarity near 1.0, got {}",
        report.pairs[0].similarity
    );
}

// ── 4. Similarity monotonically decreases with divergence ───────

#[test]
fn similarity_decreases_with_divergence() {
    let dir = TempDir::new().unwrap();
    let base = make_tokens(120, "base");
    write_to(&dir, "a.rs", &base);

    let high_overlap = overlapping_content(100, 20, 1);
    let med_overlap = overlapping_content(60, 60, 2);
    let low_overlap = overlapping_content(30, 90, 3);
    write_to(&dir, "b.rs", &high_overlap);
    write_to(&dir, "c.rs", &med_overlap);
    write_to(&dir, "d.rs", &low_overlap);

    let rows = vec![
        file_row("a.rs", "(root)", "Rust", 100, 5000),
        file_row("b.rs", "(root)", "Rust", 100, 5000),
        file_row("c.rs", "(root)", "Rust", 100, 5000),
        file_row("d.rs", "(root)", "Rust", 100, 5000),
    ];
    let export = mk_export(rows);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.0,
        100,
        Some(1000),
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    let find_sim = |l: &str, r: &str| -> Option<f64> {
        report
            .pairs
            .iter()
            .find(|p| (p.left == l && p.right == r) || (p.left == r && p.right == l))
            .map(|p| p.similarity)
    };

    if let (Some(bc), Some(bd)) = (find_sim("b.rs", "c.rs"), find_sim("b.rs", "d.rs")) {
        assert!(bc >= bd, "b-c ({bc}) should be >= b-d ({bd})");
    }
}

// ── 5. Three identical files produce C(3,2)=3 pairs ────────────

#[test]
fn three_identical_files_produce_three_pairs() {
    let dir = TempDir::new().unwrap();
    let content = make_tokens(120, "trio");
    for name in &["p.rs", "q.rs", "r.rs"] {
        write_to(&dir, name, &content);
    }
    let rows: Vec<FileRow> = ["p.rs", "q.rs", "r.rs"]
        .iter()
        .map(|n| file_row(n, "(root)", "Rust", 60, 4000))
        .collect();
    let export = mk_export(rows);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        Some(1000),
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();
    assert_eq!(report.pairs.len(), 3, "C(3,2)=3 pairs expected");
}

// ── 6. Threshold filters low-similarity pairs ───────────────────

#[test]
fn high_threshold_filters_dissimilar_pairs() {
    let dir = TempDir::new().unwrap();
    let a = make_tokens(120, "alpha");
    let b = make_tokens(120, "beta");
    write_to(&dir, "a.rs", &a);
    write_to(&dir, "b.rs", &b);
    let export = mk_export(vec![
        file_row("a.rs", "(root)", "Rust", 60, 4000),
        file_row("b.rs", "(root)", "Rust", 60, 4000),
    ]);

    let report_low = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.0,
        100,
        Some(1000),
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();
    let report_high = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.99,
        100,
        Some(1000),
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    assert!(
        report_high.pairs.len() <= report_low.pairs.len(),
        "higher threshold should yield <= pairs"
    );
}

// ── 7. Pairs always have left <= right lexicographically ────────

#[test]
fn pairs_are_lexicographically_ordered() {
    let dir = TempDir::new().unwrap();
    let content = make_tokens(120, "order");
    for name in &["z.rs", "a.rs", "m.rs"] {
        write_to(&dir, name, &content);
    }
    let rows: Vec<FileRow> = ["z.rs", "a.rs", "m.rs"]
        .iter()
        .map(|n| file_row(n, "(root)", "Rust", 60, 4000))
        .collect();
    let export = mk_export(rows);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.0,
        100,
        Some(1000),
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();
    for pair in &report.pairs {
        assert!(pair.left <= pair.right, "{} > {}", pair.left, pair.right);
    }
}

// ── 8. Lang scope isolates different languages ──────────────────

#[test]
fn lang_scope_isolates_different_languages() {
    let dir = TempDir::new().unwrap();
    let content = make_tokens(120, "lang");
    write_to(&dir, "code.rs", &content);
    write_to(&dir, "code.py", &content);
    let export = mk_export(vec![
        file_row("code.rs", "(root)", "Rust", 60, 4000),
        file_row("code.py", "(root)", "Python", 60, 4000),
    ]);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Lang,
        0.0,
        100,
        Some(1000),
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();
    assert!(
        report.pairs.is_empty(),
        "lang scope should not pair Rust with Python"
    );
}

// ── 9. Module scope isolates different modules ──────────────────

#[test]
fn module_scope_isolates_different_modules() {
    let dir = TempDir::new().unwrap();
    let content = make_tokens(120, "modsep");
    write_to(&dir, "a/f.rs", &content);
    write_to(&dir, "b/f.rs", &content);
    let export = mk_export(vec![
        file_row("a/f.rs", "a", "Rust", 60, 4000),
        file_row("b/f.rs", "b", "Rust", 60, 4000),
    ]);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Module,
        0.0,
        100,
        Some(1000),
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();
    assert!(
        report.pairs.is_empty(),
        "module scope should not pair files across modules"
    );
}

// ── 10. Exclude patterns filter files ───────────────────────────

#[test]
fn exclude_patterns_remove_matching_files() {
    let dir = TempDir::new().unwrap();
    let content = make_tokens(120, "excl");
    write_to(&dir, "keep.rs", &content);
    write_to(&dir, "gen_code.rs", &content);
    write_to(&dir, "also_keep.rs", &content);
    let export = mk_export(vec![
        file_row("keep.rs", "(root)", "Rust", 60, 4000),
        file_row("gen_code.rs", "(root)", "Rust", 60, 4000),
        file_row("also_keep.rs", "(root)", "Rust", 60, 4000),
    ]);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.0,
        100,
        Some(1000),
        &NearDupLimits::default(),
        &["gen_*".to_string()],
    )
    .unwrap();
    for pair in &report.pairs {
        assert!(!pair.left.starts_with("gen_") && !pair.right.starts_with("gen_"));
    }
}

// ── 11. max_files truncation tracks skipped count ───────────────

#[test]
fn max_files_tracks_skipped() {
    let dir = TempDir::new().unwrap();
    let content = make_tokens(120, "trunc");
    for i in 0..8 {
        write_to(&dir, &format!("f{i}.rs"), &content);
    }
    let rows: Vec<FileRow> = (0..8)
        .map(|i| file_row(&format!("f{i}.rs"), "(root)", "Rust", (8 - i) * 20, 4000))
        .collect();
    let export = mk_export(rows);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.0,
        3,
        Some(1000),
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();
    assert_eq!(report.files_skipped, 5, "8 files - 3 max = 5 skipped");
}

// ── 12. Deterministic output across multiple runs ───────────────

#[test]
fn deterministic_across_runs() {
    let dir = TempDir::new().unwrap();
    let content = make_tokens(120, "det");
    for name in &["a.rs", "b.rs", "c.rs"] {
        write_to(&dir, name, &content);
    }
    let rows: Vec<FileRow> = ["a.rs", "b.rs", "c.rs"]
        .iter()
        .map(|n| file_row(n, "(root)", "Rust", 60, 4000))
        .collect();
    let export = mk_export(rows);

    let results: Vec<_> = (0..3)
        .map(|_| {
            build_near_dup_report(
                dir.path(),
                &export,
                NearDupScope::Global,
                0.0,
                100,
                Some(1000),
                &NearDupLimits::default(),
                &[],
            )
            .unwrap()
        })
        .collect();

    for i in 1..3 {
        assert_eq!(results[0].pairs.len(), results[i].pairs.len());
        for (a, b) in results[0].pairs.iter().zip(results[i].pairs.iter()) {
            assert_eq!(a.left, b.left);
            assert_eq!(a.right, b.right);
            assert!((a.similarity - b.similarity).abs() < f64::EPSILON);
        }
    }
}

// ── 13. max_file_bytes limit excludes oversized files ───────────

#[test]
fn max_file_bytes_excludes_large_files() {
    let dir = TempDir::new().unwrap();
    let content = make_tokens(120, "size");
    write_to(&dir, "small.rs", &content);
    write_to(&dir, "big.rs", &content);
    let export = mk_export(vec![
        file_row("small.rs", "(root)", "Rust", 60, 100),
        file_row("big.rs", "(root)", "Rust", 60, 999_999),
    ]);
    let limits = NearDupLimits {
        max_bytes: None,
        max_file_bytes: Some(1000),
    };
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.0,
        100,
        Some(1000),
        &limits,
        &[],
    )
    .unwrap();
    assert!(report.pairs.is_empty(), "oversized file should be excluded");
}
