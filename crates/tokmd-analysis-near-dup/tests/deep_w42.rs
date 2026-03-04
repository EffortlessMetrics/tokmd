//! Wave-42 deep tests for near-duplicate detection.
//!
//! Tests various similarity thresholds, exact duplicates, unique codebases,
//! edge cases (empty files, one-line files), scope isolation, and serde.

use std::io::Write;
use tempfile::TempDir;
use tokmd_analysis_near_dup::{NearDupLimits, build_near_dup_report};
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
        children: ChildIncludeMode::Separate,
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

fn long_source(n: usize, seed: usize) -> String {
    (0..n)
        .map(|i| format!("token_{seed}_{i}"))
        .collect::<Vec<_>>()
        .join(" + ")
}

fn limits() -> NearDupLimits {
    NearDupLimits {
        max_bytes: None,
        max_file_bytes: Some(1_000_000),
    }
}

// ── 1. Exact duplicate files produce similarity ~1.0 ────────────

#[test]
fn exact_duplicates_similarity_near_one() {
    let dir = TempDir::new().unwrap();
    let content = long_source(150, 0);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);

    let rows = vec![
        make_row("a.rs", "mod", "Rust", 150, content.len()),
        make_row("b.rs", "mod", "Rust", 150, content.len()),
    ];
    let export = make_export(rows);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &limits(),
        &[],
    )
    .unwrap();

    assert_eq!(report.pairs.len(), 1);
    assert!(
        report.pairs[0].similarity >= 0.99,
        "exact dups should be ~1.0, got {}",
        report.pairs[0].similarity
    );
}

// ── 2. Completely unique files produce no pairs ─────────────────

#[test]
fn unique_files_no_pairs() {
    let dir = TempDir::new().unwrap();
    write_file(&dir, "a.rs", &long_source(100, 1));
    write_file(&dir, "b.rs", &long_source(100, 2));
    write_file(&dir, "c.rs", &long_source(100, 3));

    let rows = vec![
        make_row("a.rs", "mod", "Rust", 100, 500),
        make_row("b.rs", "mod", "Rust", 100, 500),
        make_row("c.rs", "mod", "Rust", 100, 500),
    ];
    let export = make_export(rows);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &limits(),
        &[],
    )
    .unwrap();

    assert!(
        report.pairs.is_empty(),
        "unique files should produce no pairs, got {}",
        report.pairs.len()
    );
}

// ── 3. Empty files produce no fingerprints (no pairs) ───────────

#[test]
fn empty_files_no_pairs() {
    let dir = TempDir::new().unwrap();
    write_file(&dir, "empty1.rs", "");
    write_file(&dir, "empty2.rs", "");

    let rows = vec![
        make_row("empty1.rs", "mod", "Rust", 0, 0),
        make_row("empty2.rs", "mod", "Rust", 0, 0),
    ];
    let export = make_export(rows);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.1,
        100,
        None,
        &limits(),
        &[],
    )
    .unwrap();

    assert!(
        report.pairs.is_empty(),
        "empty files should produce no pairs"
    );
}

// ── 4. One-line files too short for k-gram ──────────────────────

#[test]
fn one_line_files_too_short_for_fingerprinting() {
    let dir = TempDir::new().unwrap();
    write_file(&dir, "short1.rs", "fn main() {}");
    write_file(&dir, "short2.rs", "fn main() {}");

    let rows = vec![
        make_row("short1.rs", "mod", "Rust", 1, 13),
        make_row("short2.rs", "mod", "Rust", 1, 13),
    ];
    let export = make_export(rows);
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.1,
        100,
        None,
        &limits(),
        &[],
    )
    .unwrap();

    // Files with fewer than K=25 tokens produce no fingerprints
    assert!(
        report.pairs.is_empty(),
        "short files should produce no pairs"
    );
}

// ── 5. High threshold filters out moderate duplicates ───────────

#[test]
fn high_threshold_filters_moderate_duplicates() {
    let dir = TempDir::new().unwrap();
    // Shared prefix with divergent suffixes
    let shared: String = (0..80)
        .map(|i| format!("shared_{i}"))
        .collect::<Vec<_>>()
        .join(" + ");
    let a = format!("{} + {}", shared, long_source(40, 10));
    let b = format!("{} + {}", shared, long_source(40, 20));
    write_file(&dir, "a.rs", &a);
    write_file(&dir, "b.rs", &b);

    let rows = vec![
        make_row("a.rs", "mod", "Rust", 120, a.len()),
        make_row("b.rs", "mod", "Rust", 120, b.len()),
    ];
    let export = make_export(rows);

    // With threshold 0.99 the partially-similar pair should be filtered
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.99,
        100,
        None,
        &limits(),
        &[],
    )
    .unwrap();

    assert!(
        report.pairs.is_empty(),
        "threshold 0.99 should filter moderate dups"
    );
}

// ── 6. Low threshold catches moderate duplicates ────────────────

#[test]
fn low_threshold_catches_moderate_duplicates() {
    let dir = TempDir::new().unwrap();
    let shared: String = (0..80)
        .map(|i| format!("common_{i}"))
        .collect::<Vec<_>>()
        .join(" + ");
    let a = format!("{} + {}", shared, long_source(30, 10));
    let b = format!("{} + {}", shared, long_source(30, 20));
    write_file(&dir, "a.rs", &a);
    write_file(&dir, "b.rs", &b);

    let rows = vec![
        make_row("a.rs", "mod", "Rust", 110, a.len()),
        make_row("b.rs", "mod", "Rust", 110, b.len()),
    ];
    let export = make_export(rows);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.3,
        100,
        None,
        &limits(),
        &[],
    )
    .unwrap();

    assert!(
        !report.pairs.is_empty(),
        "low threshold should detect moderate dups"
    );
}

// ── 7. Module scope isolates comparisons ────────────────────────

#[test]
fn module_scope_isolates_comparisons() {
    let dir = TempDir::new().unwrap();
    let content = long_source(100, 0);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);

    // Same content, different modules
    let rows = vec![
        make_row("a.rs", "mod_a", "Rust", 100, content.len()),
        make_row("b.rs", "mod_b", "Rust", 100, content.len()),
    ];
    let export = make_export(rows);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Module,
        0.5,
        100,
        None,
        &limits(),
        &[],
    )
    .unwrap();

    // Files in different modules should not be compared under Module scope
    assert!(
        report.pairs.is_empty(),
        "module scope should isolate comparisons"
    );
}

// ── 8. Lang scope groups by language ────────────────────────────

#[test]
fn lang_scope_groups_by_language() {
    let dir = TempDir::new().unwrap();
    let content = long_source(100, 0);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.py", &content);

    let rows = vec![
        make_row("a.rs", "mod", "Rust", 100, content.len()),
        make_row("b.py", "mod", "Python", 100, content.len()),
    ];
    let export = make_export(rows);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Lang,
        0.5,
        100,
        None,
        &limits(),
        &[],
    )
    .unwrap();

    // Different languages under Lang scope should not pair
    assert!(
        report.pairs.is_empty(),
        "lang scope should separate Rust and Python"
    );
}

// ── 9. max_pairs truncates output ───────────────────────────────

#[test]
fn max_pairs_truncates_output() {
    let dir = TempDir::new().unwrap();
    let content = long_source(100, 0);
    // Create 4 identical files → 6 pairs
    for i in 0..4 {
        write_file(&dir, &format!("f{i}.rs"), &content);
    }

    let rows: Vec<FileRow> = (0..4)
        .map(|i| make_row(&format!("f{i}.rs"), "mod", "Rust", 100, content.len()))
        .collect();
    let export = make_export(rows);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        Some(2),
        &limits(),
        &[],
    )
    .unwrap();

    assert!(report.pairs.len() <= 2, "max_pairs should truncate");
    assert!(report.truncated, "truncated flag should be set");
}

// ── 10. Child rows are excluded from analysis ───────────────────

#[test]
fn child_rows_excluded() {
    let dir = TempDir::new().unwrap();
    let content = long_source(100, 0);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);

    let rows = vec![
        FileRow {
            path: "a.rs".to_string(),
            module: "mod".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Child,
            code: 100,
            comments: 0,
            blanks: 0,
            lines: 100,
            bytes: content.len(),
            tokens: 500,
        },
        FileRow {
            path: "b.rs".to_string(),
            module: "mod".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Child,
            code: 100,
            comments: 0,
            blanks: 0,
            lines: 100,
            bytes: content.len(),
            tokens: 500,
        },
    ];
    let export = make_export(rows);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &limits(),
        &[],
    )
    .unwrap();

    assert_eq!(report.files_analyzed, 0, "child rows should be excluded");
}

// ── 11. Serde roundtrip for NearDuplicateReport ─────────────────

#[test]
fn near_dup_report_serde_roundtrip() {
    let dir = TempDir::new().unwrap();
    let content = long_source(100, 0);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);

    let rows = vec![
        make_row("a.rs", "mod", "Rust", 100, content.len()),
        make_row("b.rs", "mod", "Rust", 100, content.len()),
    ];
    let export = make_export(rows);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &limits(),
        &[],
    )
    .unwrap();

    let json = serde_json::to_string_pretty(&report).unwrap();
    let deser: tokmd_analysis_types::NearDuplicateReport = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.files_analyzed, report.files_analyzed);
    assert_eq!(deser.pairs.len(), report.pairs.len());
}

// ── 12. Pairs sorted by similarity descending ───────────────────

#[test]
fn pairs_sorted_by_similarity_desc() {
    let dir = TempDir::new().unwrap();
    let base: String = (0..100)
        .map(|i| format!("base_{i}"))
        .collect::<Vec<_>>()
        .join(" + ");

    // Create files with varying similarity to base
    write_file(&dir, "base.rs", &base);
    let near = format!("{} + {}", base, long_source(5, 1));
    write_file(&dir, "near.rs", &near);
    let far = format!(
        "{} + {}",
        (0..50)
            .map(|i| format!("base_{i}"))
            .collect::<Vec<_>>()
            .join(" + "),
        long_source(60, 2)
    );
    write_file(&dir, "far.rs", &far);

    let rows = vec![
        make_row("base.rs", "mod", "Rust", 100, base.len()),
        make_row("near.rs", "mod", "Rust", 105, near.len()),
        make_row("far.rs", "mod", "Rust", 110, far.len()),
    ];
    let export = make_export(rows);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.1,
        100,
        None,
        &limits(),
        &[],
    )
    .unwrap();

    for window in report.pairs.windows(2) {
        assert!(
            window[0].similarity >= window[1].similarity,
            "pairs should be sorted by similarity desc"
        );
    }
}

// ── 13. Exclude patterns filter files ───────────────────────────

#[test]
fn exclude_patterns_filter_files() {
    let dir = TempDir::new().unwrap();
    let content = long_source(100, 0);
    write_file(&dir, "src/a.rs", &content);
    write_file(&dir, "src/b.rs", &content);
    write_file(&dir, "vendor/c.rs", &content);

    let rows = vec![
        make_row("src/a.rs", "src", "Rust", 100, content.len()),
        make_row("src/b.rs", "src", "Rust", 100, content.len()),
        make_row("vendor/c.rs", "vendor", "Rust", 100, content.len()),
    ];
    let export = make_export(rows);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &limits(),
        &["vendor/**".to_string()],
    )
    .unwrap();

    assert_eq!(report.excluded_by_pattern, Some(1));
    // Only src files compared
    assert_eq!(report.files_analyzed, 2);
}

// ── 14. Global scope finds cross-module duplicates ──────────────

#[test]
fn global_scope_finds_cross_module_dups() {
    let dir = TempDir::new().unwrap();
    let content = long_source(100, 0);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);

    let rows = vec![
        make_row("a.rs", "mod_a", "Rust", 100, content.len()),
        make_row("b.rs", "mod_b", "Python", 100, content.len()),
    ];
    let export = make_export(rows);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &limits(),
        &[],
    )
    .unwrap();

    assert!(
        !report.pairs.is_empty(),
        "global scope should find cross-module dups"
    );
}
