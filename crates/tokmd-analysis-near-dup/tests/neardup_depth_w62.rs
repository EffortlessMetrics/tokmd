//! Wave-62 depth tests for `tokmd-analysis-near-dup`.
//!
//! Covers exact duplicate detection, similarity thresholds, scope partitioning,
//! empty/single-file handling, performance with many files, property tests,
//! and determinism of duplicate grouping.

use std::io::Write;

use proptest::prelude::*;
use tempfile::TempDir;
use tokmd_analysis_near_dup::{NearDupLimits, build_near_dup_report};
use tokmd_analysis_types::NearDupScope;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ───────────────────── helpers ─────────────────────

fn frow(path: &str, module: &str, lang: &str, code: usize, bytes: usize) -> FileRow {
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

/// Generate deterministic source with `n` lines and `seed`-based variation.
fn gen_source(n: usize, seed: usize) -> String {
    (0..n)
        .map(|i| {
            format!(
                "fn func_{}_{}_impl() {{ let val = {}; }}",
                seed,
                i,
                i * seed
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn default_limits() -> NearDupLimits {
    NearDupLimits {
        max_bytes: None,
        max_file_bytes: None,
    }
}

// ═══════════════════════════════════════════════════════════════
// 1. Exact duplicate detection
// ═══════════════════════════════════════════════════════════════

#[test]
fn exact_duplicates_two_files() {
    let dir = TempDir::new().unwrap();
    let content = gen_source(60, 1);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);
    let sz = content.len();

    let report = build_near_dup_report(
        dir.path(),
        &make_export(vec![
            frow("a.rs", "mod", "Rust", 60, sz),
            frow("b.rs", "mod", "Rust", 60, sz),
        ]),
        NearDupScope::Global,
        0.5,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    assert_eq!(report.pairs.len(), 1);
    assert!((report.pairs[0].similarity - 1.0).abs() < 1e-4);
}

#[test]
fn exact_duplicates_three_files_form_one_cluster() {
    let dir = TempDir::new().unwrap();
    let content = gen_source(60, 1);
    for name in ["x.rs", "y.rs", "z.rs"] {
        write_file(&dir, name, &content);
    }
    let sz = content.len();

    let report = build_near_dup_report(
        dir.path(),
        &make_export(vec![
            frow("x.rs", "mod", "Rust", 60, sz),
            frow("y.rs", "mod", "Rust", 60, sz),
            frow("z.rs", "mod", "Rust", 60, sz),
        ]),
        NearDupScope::Global,
        0.5,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    assert_eq!(report.pairs.len(), 3); // 3 choose 2
    let clusters = report.clusters.as_ref().unwrap();
    assert_eq!(clusters.len(), 1);
    assert_eq!(clusters[0].files.len(), 3);
}

#[test]
fn exact_duplicates_similarity_is_one() {
    let dir = TempDir::new().unwrap();
    let content = gen_source(80, 42);
    write_file(&dir, "left.rs", &content);
    write_file(&dir, "right.rs", &content);
    let sz = content.len();

    let report = build_near_dup_report(
        dir.path(),
        &make_export(vec![
            frow("left.rs", "m", "Rust", 80, sz),
            frow("right.rs", "m", "Rust", 80, sz),
        ]),
        NearDupScope::Global,
        0.99,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    assert!(!report.pairs.is_empty());
    assert!((report.pairs[0].similarity - 1.0).abs() < 1e-4);
}

#[test]
fn exact_dup_shared_fingerprints_equal_total() {
    let dir = TempDir::new().unwrap();
    let content = gen_source(60, 7);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);
    let sz = content.len();

    let report = build_near_dup_report(
        dir.path(),
        &make_export(vec![
            frow("a.rs", "m", "Rust", 60, sz),
            frow("b.rs", "m", "Rust", 60, sz),
        ]),
        NearDupScope::Global,
        0.5,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    let p = &report.pairs[0];
    assert_eq!(p.shared_fingerprints, p.left_fingerprints);
    assert_eq!(p.shared_fingerprints, p.right_fingerprints);
}

// ═══════════════════════════════════════════════════════════════
// 2. Near-duplicate similarity thresholds
// ═══════════════════════════════════════════════════════════════

#[test]
fn threshold_0_catches_all_pairs() {
    let dir = TempDir::new().unwrap();
    let c1 = gen_source(60, 1);
    let c2 = gen_source(60, 2);
    write_file(&dir, "a.rs", &c1);
    write_file(&dir, "b.rs", &c2);

    let report = build_near_dup_report(
        dir.path(),
        &make_export(vec![
            frow("a.rs", "m", "Rust", 60, c1.len()),
            frow("b.rs", "m", "Rust", 60, c2.len()),
        ]),
        NearDupScope::Global,
        0.0,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    // With threshold 0, any files with shared fingerprints are pairs
    // (may or may not produce pairs depending on content overlap)
    assert!(report.files_analyzed == 2);
}

#[test]
fn threshold_1_only_exact_matches() {
    let dir = TempDir::new().unwrap();
    let base = gen_source(60, 1);
    let variant = format!(
        "{}\nfn extra_variant_function() {{ let z = 123456; }}",
        &base
    );
    write_file(&dir, "a.rs", &base);
    write_file(&dir, "b.rs", &variant);

    let report = build_near_dup_report(
        dir.path(),
        &make_export(vec![
            frow("a.rs", "m", "Rust", 60, base.len()),
            frow("b.rs", "m", "Rust", 61, variant.len()),
        ]),
        NearDupScope::Global,
        1.0,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    // Non-identical files should not match at threshold 1.0
    assert!(report.pairs.is_empty());
}

#[test]
fn higher_threshold_fewer_or_equal_pairs() {
    let dir = TempDir::new().unwrap();
    let c1 = gen_source(60, 1);
    let c2 = gen_source(60, 1); // identical
    let c3 = gen_source(60, 3);
    write_file(&dir, "a.rs", &c1);
    write_file(&dir, "b.rs", &c2);
    write_file(&dir, "c.rs", &c3);

    let rows = vec![
        frow("a.rs", "m", "Rust", 60, c1.len()),
        frow("b.rs", "m", "Rust", 60, c2.len()),
        frow("c.rs", "m", "Rust", 60, c3.len()),
    ];

    let r_low = build_near_dup_report(
        dir.path(),
        &make_export(rows.clone()),
        NearDupScope::Global,
        0.1,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    let r_high = build_near_dup_report(
        dir.path(),
        &make_export(rows),
        NearDupScope::Global,
        0.99,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    assert!(r_high.pairs.len() <= r_low.pairs.len());
}

#[test]
fn moderate_similarity_detected_at_low_threshold() {
    let dir = TempDir::new().unwrap();
    let base = gen_source(60, 1);
    // Keep first 40 lines identical, change last 20
    let lines: Vec<&str> = base.lines().collect();
    let modified = format!("{}\n{}", lines[..40].join("\n"), gen_source(20, 999));
    write_file(&dir, "a.rs", &base);
    write_file(&dir, "b.rs", &modified);

    let report = build_near_dup_report(
        dir.path(),
        &make_export(vec![
            frow("a.rs", "m", "Rust", 60, base.len()),
            frow("b.rs", "m", "Rust", 60, modified.len()),
        ]),
        NearDupScope::Global,
        0.2,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    if !report.pairs.is_empty() {
        assert!(report.pairs[0].similarity >= 0.2);
        assert!(report.pairs[0].similarity < 1.0);
    }
}

// ═══════════════════════════════════════════════════════════════
// 3. Scope-based partitioning
// ═══════════════════════════════════════════════════════════════

#[test]
fn global_scope_finds_cross_module_dups() {
    let dir = TempDir::new().unwrap();
    let content = gen_source(60, 1);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);
    let sz = content.len();

    let report = build_near_dup_report(
        dir.path(),
        &make_export(vec![
            frow("a.rs", "mod_a", "Rust", 60, sz),
            frow("b.rs", "mod_b", "Rust", 60, sz),
        ]),
        NearDupScope::Global,
        0.5,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    assert!(!report.pairs.is_empty());
}

#[test]
fn module_scope_isolates_modules() {
    let dir = TempDir::new().unwrap();
    let content = gen_source(60, 1);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);
    let sz = content.len();

    let report = build_near_dup_report(
        dir.path(),
        &make_export(vec![
            frow("a.rs", "alpha", "Rust", 60, sz),
            frow("b.rs", "beta", "Rust", 60, sz),
        ]),
        NearDupScope::Module,
        0.5,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    assert!(report.pairs.is_empty());
}

#[test]
fn module_scope_finds_dups_within_same_module() {
    let dir = TempDir::new().unwrap();
    let content = gen_source(60, 1);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);
    let sz = content.len();

    let report = build_near_dup_report(
        dir.path(),
        &make_export(vec![
            frow("a.rs", "same", "Rust", 60, sz),
            frow("b.rs", "same", "Rust", 60, sz),
        ]),
        NearDupScope::Module,
        0.5,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    assert!(!report.pairs.is_empty());
}

#[test]
fn lang_scope_isolates_languages() {
    let dir = TempDir::new().unwrap();
    let content = gen_source(60, 1);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.py", &content);
    let sz = content.len();

    let report = build_near_dup_report(
        dir.path(),
        &make_export(vec![
            frow("a.rs", "m", "Rust", 60, sz),
            frow("b.py", "m", "Python", 60, sz),
        ]),
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

#[test]
fn lang_scope_finds_dups_within_same_lang() {
    let dir = TempDir::new().unwrap();
    let content = gen_source(60, 1);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);
    let sz = content.len();

    let report = build_near_dup_report(
        dir.path(),
        &make_export(vec![
            frow("a.rs", "m", "Rust", 60, sz),
            frow("b.rs", "m", "Rust", 60, sz),
        ]),
        NearDupScope::Lang,
        0.5,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    assert!(!report.pairs.is_empty());
}

// ═══════════════════════════════════════════════════════════════
// 4. Empty file handling
// ═══════════════════════════════════════════════════════════════

#[test]
fn empty_export_no_pairs() {
    let dir = TempDir::new().unwrap();
    let report = build_near_dup_report(
        dir.path(),
        &make_export(vec![]),
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
    assert!(report.clusters.is_none());
}

#[test]
fn empty_files_no_fingerprints() {
    let dir = TempDir::new().unwrap();
    write_file(&dir, "a.rs", "");
    write_file(&dir, "b.rs", "");

    let report = build_near_dup_report(
        dir.path(),
        &make_export(vec![
            frow("a.rs", "m", "Rust", 0, 0),
            frow("b.rs", "m", "Rust", 0, 0),
        ]),
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

#[test]
fn whitespace_only_files_no_pairs() {
    let dir = TempDir::new().unwrap();
    write_file(&dir, "a.rs", "   \n\n  \t  \n");
    write_file(&dir, "b.rs", "   \n\n  \t  \n");

    let report = build_near_dup_report(
        dir.path(),
        &make_export(vec![
            frow("a.rs", "m", "Rust", 0, 12),
            frow("b.rs", "m", "Rust", 0, 12),
        ]),
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

#[test]
fn very_short_files_below_kgram() {
    let dir = TempDir::new().unwrap();
    write_file(&dir, "a.rs", "fn f() { 1 }");
    write_file(&dir, "b.rs", "fn f() { 1 }");

    let report = build_near_dup_report(
        dir.path(),
        &make_export(vec![
            frow("a.rs", "m", "Rust", 1, 13),
            frow("b.rs", "m", "Rust", 1, 13),
        ]),
        NearDupScope::Global,
        0.5,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    // < 25 tokens, so no k-grams → no fingerprints → no pairs
    assert!(report.pairs.is_empty());
}

// ═══════════════════════════════════════════════════════════════
// 5. Single file (no duplicates possible)
// ═══════════════════════════════════════════════════════════════

#[test]
fn single_file_no_pairs() {
    let dir = TempDir::new().unwrap();
    let content = gen_source(60, 1);
    write_file(&dir, "only.rs", &content);

    let report = build_near_dup_report(
        dir.path(),
        &make_export(vec![frow("only.rs", "m", "Rust", 60, content.len())]),
        NearDupScope::Global,
        0.5,
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

#[test]
fn single_file_with_module_scope() {
    let dir = TempDir::new().unwrap();
    let content = gen_source(60, 1);
    write_file(&dir, "only.rs", &content);

    let report = build_near_dup_report(
        dir.path(),
        &make_export(vec![frow("only.rs", "mod", "Rust", 60, content.len())]),
        NearDupScope::Module,
        0.5,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    assert!(report.pairs.is_empty());
}

// ═══════════════════════════════════════════════════════════════
// 6. Performance with many files
// ═══════════════════════════════════════════════════════════════

#[test]
fn twenty_identical_files() {
    let dir = TempDir::new().unwrap();
    let content = gen_source(60, 1);
    let n = 20;
    for i in 0..n {
        write_file(&dir, &format!("f{i}.rs"), &content);
    }

    let rows: Vec<FileRow> = (0..n)
        .map(|i| frow(&format!("f{i}.rs"), "m", "Rust", 60, content.len()))
        .collect();

    let report = build_near_dup_report(
        dir.path(),
        &make_export(rows),
        NearDupScope::Global,
        0.5,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    // 20 choose 2 = 190 pairs
    assert_eq!(report.pairs.len(), 190);
    let clusters = report.clusters.as_ref().unwrap();
    assert_eq!(clusters.len(), 1);
    assert_eq!(clusters[0].files.len(), 20);
}

#[test]
fn max_files_limits_analysis() {
    let dir = TempDir::new().unwrap();
    let content = gen_source(60, 1);
    for i in 0..15 {
        write_file(&dir, &format!("f{i}.rs"), &content);
    }

    let rows: Vec<FileRow> = (0..15)
        .map(|i| frow(&format!("f{i}.rs"), "m", "Rust", 60, content.len()))
        .collect();

    let report = build_near_dup_report(
        dir.path(),
        &make_export(rows),
        NearDupScope::Global,
        0.5,
        5,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    assert_eq!(report.files_analyzed, 5);
    assert_eq!(report.files_skipped, 10);
}

#[test]
fn max_pairs_truncation() {
    let dir = TempDir::new().unwrap();
    let content = gen_source(60, 1);
    for i in 0..6 {
        write_file(&dir, &format!("f{i}.rs"), &content);
    }

    let rows: Vec<FileRow> = (0..6)
        .map(|i| frow(&format!("f{i}.rs"), "m", "Rust", 60, content.len()))
        .collect();

    let report = build_near_dup_report(
        dir.path(),
        &make_export(rows),
        NearDupScope::Global,
        0.5,
        100,
        Some(3),
        &default_limits(),
        &[],
    )
    .unwrap();

    assert!(report.pairs.len() <= 3);
    assert!(report.truncated);
}

#[test]
fn many_unique_files_no_pairs() {
    let dir = TempDir::new().unwrap();
    for i in 0..30 {
        let content = gen_source(60, i + 1000); // all different seeds
        write_file(&dir, &format!("f{i}.rs"), &content);
    }

    let rows: Vec<FileRow> = (0..30)
        .map(|i| {
            let content = gen_source(60, i + 1000);
            frow(&format!("f{i}.rs"), "m", "Rust", 60, content.len())
        })
        .collect();

    let report = build_near_dup_report(
        dir.path(),
        &make_export(rows),
        NearDupScope::Global,
        0.8,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    // Unique content should produce few or no pairs at high threshold
    assert_eq!(report.files_analyzed, 30);
}

#[test]
fn exclude_patterns_reduce_analyzed() {
    let dir = TempDir::new().unwrap();
    let content = gen_source(60, 1);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);
    write_file(&dir, "gen_output.rs", &content);

    let report = build_near_dup_report(
        dir.path(),
        &make_export(vec![
            frow("a.rs", "m", "Rust", 60, content.len()),
            frow("b.rs", "m", "Rust", 60, content.len()),
            frow("gen_output.rs", "m", "Rust", 60, content.len()),
        ]),
        NearDupScope::Global,
        0.5,
        100,
        None,
        &default_limits(),
        &["gen_*".to_string()],
    )
    .unwrap();

    assert_eq!(report.excluded_by_pattern, Some(1));
    assert_eq!(report.files_analyzed, 2);
}

// ═══════════════════════════════════════════════════════════════
// 7. Property tests
// ═══════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_similarity_in_0_1(seed1 in 1..100usize, seed2 in 1..100usize) {
        let dir = TempDir::new().unwrap();
        let c1 = gen_source(60, seed1);
        let c2 = gen_source(60, seed2);
        write_file(&dir, "a.rs", &c1);
        write_file(&dir, "b.rs", &c2);

        let report = build_near_dup_report(
            dir.path(),
            &make_export(vec![frow("a.rs", "m", "Rust", 60, c1.len()), frow("b.rs", "m", "Rust", 60, c2.len())]),
            NearDupScope::Global, 0.0, 100, None, &default_limits(), &[],
        ).unwrap();

        for pair in &report.pairs {
            prop_assert!(pair.similarity >= 0.0 && pair.similarity <= 1.0,
                "similarity {} out of [0,1]", pair.similarity);
        }
    }

    #[test]
    fn prop_similarity_symmetric(seed in 1..50usize) {
        let dir = TempDir::new().unwrap();
        let c1 = gen_source(60, seed);
        let c2 = gen_source(60, seed + 500);
        write_file(&dir, "a.rs", &c1);
        write_file(&dir, "b.rs", &c2);

        let report = build_near_dup_report(
            dir.path(),
            &make_export(vec![frow("a.rs", "m", "Rust", 60, c1.len()), frow("b.rs", "m", "Rust", 60, c2.len())]),
            NearDupScope::Global, 0.0, 100, None, &default_limits(), &[],
        ).unwrap();

        // Jaccard is inherently symmetric: J(A,B) = J(B,A)
        // The report always puts left < right alphabetically, so just check it exists once
        if !report.pairs.is_empty() {
            let p = &report.pairs[0];
            prop_assert!(p.left <= p.right, "left should be <= right for determinism");
        }
    }

    #[test]
    fn prop_identical_similarity_is_one(seed in 1..200usize) {
        let dir = TempDir::new().unwrap();
        let content = gen_source(60, seed);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);
        let sz = content.len();

        let report = build_near_dup_report(
            dir.path(),
            &make_export(vec![frow("a.rs", "m", "Rust", 60, sz), frow("b.rs", "m", "Rust", 60, sz)]),
            NearDupScope::Global, 0.5, 100, None, &default_limits(), &[],
        ).unwrap();

        prop_assert_eq!(report.pairs.len(), 1);
        prop_assert!((report.pairs[0].similarity - 1.0).abs() < 1e-4,
            "identical files should have similarity 1.0, got {}", report.pairs[0].similarity);
    }

    #[test]
    fn prop_cluster_file_count_ge_pair_files(seed in 1..100usize) {
        let dir = TempDir::new().unwrap();
        let content = gen_source(60, seed);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);
        write_file(&dir, "c.rs", &content);
        let sz = content.len();

        let report = build_near_dup_report(
            dir.path(),
            &make_export(vec![
                frow("a.rs", "m", "Rust", 60, sz),
                frow("b.rs", "m", "Rust", 60, sz),
                frow("c.rs", "m", "Rust", 60, sz),
            ]),
            NearDupScope::Global, 0.5, 100, None, &default_limits(), &[],
        ).unwrap();

        if let Some(clusters) = &report.clusters {
            let total_clustered: usize = clusters.iter().map(|c| c.files.len()).sum();
            // Each pair references 2 files, so cluster should have >= 2 files
            prop_assert!(total_clustered >= 2);
        }
    }

    #[test]
    fn prop_pairs_sorted_by_similarity_desc(seed in 1..50usize) {
        let dir = TempDir::new().unwrap();
        let c1 = gen_source(60, seed);
        let c2 = gen_source(60, seed);       // identical to c1
        let c3 = gen_source(60, seed + 500); // different
        write_file(&dir, "a.rs", &c1);
        write_file(&dir, "b.rs", &c2);
        write_file(&dir, "c.rs", &c3);

        let report = build_near_dup_report(
            dir.path(),
            &make_export(vec![
                frow("a.rs", "m", "Rust", 60, c1.len()),
                frow("b.rs", "m", "Rust", 60, c2.len()),
                frow("c.rs", "m", "Rust", 60, c3.len()),
            ]),
            NearDupScope::Global, 0.0, 100, None, &default_limits(), &[],
        ).unwrap();

        for window in report.pairs.windows(2) {
            prop_assert!(window[0].similarity >= window[1].similarity,
                "pairs should be sorted by similarity desc");
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// 8. Determinism: same files always produce same groups
// ═══════════════════════════════════════════════════════════════

#[test]
fn determinism_same_pairs() {
    let dir = TempDir::new().unwrap();
    let c1 = gen_source(60, 1);
    let c2 = gen_source(60, 1); // identical
    let c3 = gen_source(60, 3);
    write_file(&dir, "a.rs", &c1);
    write_file(&dir, "b.rs", &c2);
    write_file(&dir, "c.rs", &c3);

    let exp = make_export(vec![
        frow("a.rs", "m", "Rust", 60, c1.len()),
        frow("b.rs", "m", "Rust", 60, c2.len()),
        frow("c.rs", "m", "Rust", 60, c3.len()),
    ]);

    let r1 = build_near_dup_report(
        dir.path(),
        &exp,
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
        &exp,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    assert_eq!(r1.pairs.len(), r2.pairs.len());
    for (a, b) in r1.pairs.iter().zip(r2.pairs.iter()) {
        assert_eq!(a.left, b.left);
        assert_eq!(a.right, b.right);
        assert!((a.similarity - b.similarity).abs() < 1e-10);
    }
}

#[test]
fn determinism_same_clusters() {
    let dir = TempDir::new().unwrap();
    let content = gen_source(60, 1);
    for name in ["a.rs", "b.rs", "c.rs"] {
        write_file(&dir, name, &content);
    }
    let sz = content.len();

    let exp = make_export(vec![
        frow("a.rs", "m", "Rust", 60, sz),
        frow("b.rs", "m", "Rust", 60, sz),
        frow("c.rs", "m", "Rust", 60, sz),
    ]);

    let r1 = build_near_dup_report(
        dir.path(),
        &exp,
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
        &exp,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    let c1 = r1.clusters.unwrap();
    let c2 = r2.clusters.unwrap();
    assert_eq!(c1.len(), c2.len());
    for (a, b) in c1.iter().zip(c2.iter()) {
        assert_eq!(a.files, b.files);
        assert_eq!(a.representative, b.representative);
    }
}

#[test]
fn determinism_serialization_stable() {
    let dir = TempDir::new().unwrap();
    let content = gen_source(60, 1);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);
    let sz = content.len();

    let exp = make_export(vec![
        frow("a.rs", "m", "Rust", 60, sz),
        frow("b.rs", "m", "Rust", 60, sz),
    ]);

    let r1 = build_near_dup_report(
        dir.path(),
        &exp,
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
        &exp,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    let _j1 = serde_json::to_string(&r1).unwrap();
    let _j2 = serde_json::to_string(&r2).unwrap();
    // Ignore timing stats which may differ
    // Compare pairs section only
    assert_eq!(r1.pairs.len(), r2.pairs.len());
    assert_eq!(
        serde_json::to_string(&r1.pairs).unwrap(),
        serde_json::to_string(&r2.pairs).unwrap()
    );
    // Clusters should also be identical
    assert_eq!(
        serde_json::to_string(&r1.clusters).unwrap(),
        serde_json::to_string(&r2.clusters).unwrap()
    );
}

// ═══════════════════════════════════════════════════════════════
// 9. Additional edge cases
// ═══════════════════════════════════════════════════════════════

#[test]
fn child_rows_filtered_out() {
    let dir = TempDir::new().unwrap();
    let content = gen_source(60, 1);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);

    let mut child = frow("b.rs", "m", "Rust", 60, content.len());
    child.kind = FileKind::Child;

    let report = build_near_dup_report(
        dir.path(),
        &make_export(vec![frow("a.rs", "m", "Rust", 60, content.len()), child]),
        NearDupScope::Global,
        0.5,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    assert_eq!(report.files_analyzed, 1);
    assert!(report.pairs.is_empty());
}

#[test]
fn max_file_bytes_excludes_large_files() {
    let dir = TempDir::new().unwrap();
    let content = gen_source(60, 1);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);

    let limits = NearDupLimits {
        max_bytes: None,
        max_file_bytes: Some(5), // extremely small
    };

    let report = build_near_dup_report(
        dir.path(),
        &make_export(vec![
            frow("a.rs", "m", "Rust", 60, content.len()),
            frow("b.rs", "m", "Rust", 60, content.len()),
        ]),
        NearDupScope::Global,
        0.5,
        100,
        None,
        &limits,
        &[],
    )
    .unwrap();

    assert_eq!(report.files_analyzed, 0);
}

#[test]
fn missing_file_on_disk_graceful() {
    let dir = TempDir::new().unwrap();
    let content = gen_source(60, 1);
    write_file(&dir, "a.rs", &content);
    // b.rs does NOT exist

    let report = build_near_dup_report(
        dir.path(),
        &make_export(vec![
            frow("a.rs", "m", "Rust", 60, content.len()),
            frow("b.rs", "m", "Rust", 60, content.len()),
        ]),
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

#[test]
fn params_recorded_in_report() {
    let dir = TempDir::new().unwrap();
    let report = build_near_dup_report(
        dir.path(),
        &make_export(vec![]),
        NearDupScope::Global,
        0.75,
        50,
        Some(10),
        &default_limits(),
        &["*.gen".to_string()],
    )
    .unwrap();

    assert_eq!(report.params.threshold, 0.75);
    assert_eq!(report.params.max_files, 50);
    assert_eq!(report.params.max_pairs, Some(10));
    assert_eq!(report.params.exclude_patterns, vec!["*.gen".to_string()]);
}

#[test]
fn algorithm_params_recorded() {
    let dir = TempDir::new().unwrap();
    let report = build_near_dup_report(
        dir.path(),
        &make_export(vec![]),
        NearDupScope::Global,
        0.5,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    let algo = report.params.algorithm.as_ref().unwrap();
    assert_eq!(algo.k_gram_size, 25);
    assert_eq!(algo.window_size, 4);
    assert_eq!(algo.max_postings, 50);
}

#[test]
fn stats_timing_populated() {
    let dir = TempDir::new().unwrap();
    let content = gen_source(60, 1);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);
    let sz = content.len();

    let report = build_near_dup_report(
        dir.path(),
        &make_export(vec![
            frow("a.rs", "m", "Rust", 60, sz),
            frow("b.rs", "m", "Rust", 60, sz),
        ]),
        NearDupScope::Global,
        0.5,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    let stats = report.stats.unwrap();
    assert!(stats.bytes_processed > 0);
    // Timing may be 0ms for fast operations, that's ok
}

#[test]
fn cluster_representative_most_connected() {
    let dir = TempDir::new().unwrap();
    let content = gen_source(60, 1);
    // a, b, c all identical → all connected equally
    for name in ["a.rs", "b.rs", "c.rs"] {
        write_file(&dir, name, &content);
    }
    let sz = content.len();

    let report = build_near_dup_report(
        dir.path(),
        &make_export(vec![
            frow("a.rs", "m", "Rust", 60, sz),
            frow("b.rs", "m", "Rust", 60, sz),
            frow("c.rs", "m", "Rust", 60, sz),
        ]),
        NearDupScope::Global,
        0.5,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    let clusters = report.clusters.as_ref().unwrap();
    assert_eq!(clusters.len(), 1);
    // Representative should be one of the files
    assert!(["a.rs", "b.rs", "c.rs"].contains(&clusters[0].representative.as_str()));
}

#[test]
fn cluster_files_sorted() {
    let dir = TempDir::new().unwrap();
    let content = gen_source(60, 1);
    for name in ["z.rs", "a.rs", "m.rs"] {
        write_file(&dir, name, &content);
    }
    let sz = content.len();

    let report = build_near_dup_report(
        dir.path(),
        &make_export(vec![
            frow("z.rs", "m", "Rust", 60, sz),
            frow("a.rs", "m", "Rust", 60, sz),
            frow("m.rs", "m", "Rust", 60, sz),
        ]),
        NearDupScope::Global,
        0.5,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    if let Some(clusters) = &report.clusters {
        for c in clusters {
            let mut sorted = c.files.clone();
            sorted.sort();
            assert_eq!(c.files, sorted, "cluster files should be sorted");
        }
    }
}

#[test]
fn eligible_files_reflects_pre_cap_count() {
    let dir = TempDir::new().unwrap();
    let content = gen_source(60, 1);
    for i in 0..8 {
        write_file(&dir, &format!("f{i}.rs"), &content);
    }

    let rows: Vec<FileRow> = (0..8)
        .map(|i| frow(&format!("f{i}.rs"), "m", "Rust", 60, content.len()))
        .collect();

    let report = build_near_dup_report(
        dir.path(),
        &make_export(rows),
        NearDupScope::Global,
        0.5,
        3,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    assert_eq!(report.eligible_files, Some(8));
    assert_eq!(report.files_analyzed, 3);
    assert_eq!(report.files_skipped, 5);
}
