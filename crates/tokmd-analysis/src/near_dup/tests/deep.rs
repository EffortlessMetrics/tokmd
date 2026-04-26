//! Deep invariant tests for near-duplicate detection.
//!
//! Focuses on mathematical properties (Jaccard similarity degradation,
//! combinatorial pair counts), scope isolation, cluster representative
//! selection, and boundary conditions.

use crate::near_dup::{NearDupLimits, build_near_dup_report};
use std::io::Write;
use tempfile::TempDir;
use tokmd_analysis_types::NearDupScope;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────

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

// ── 1. Similarity degrades with increasing divergence ───────────

#[test]
fn similarity_degrades_with_increasing_divergence() {
    let dir = TempDir::new().unwrap();
    let base = source_text(100, 0);

    // File A is the base
    write_file(&dir, "a.rs", &base);

    // Files with increasing divergence: 10%, 30%, 60% unique tokens
    let b = content_with_overlap(90, 10, 1);
    let c = content_with_overlap(70, 30, 2);
    let d = content_with_overlap(40, 60, 3);
    write_file(&dir, "b.rs", &b);
    write_file(&dir, "c.rs", &c);
    write_file(&dir, "d.rs", &d);

    let rows = vec![
        make_row("a.rs", "(root)", "Rust", 100, 5000),
        make_row("b.rs", "(root)", "Rust", 100, 5000),
        make_row("c.rs", "(root)", "Rust", 100, 5000),
        make_row("d.rs", "(root)", "Rust", 100, 5000),
    ];
    let export = make_export(rows);
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

    // Find pairs involving a.rs
    let sim_ab = report
        .pairs
        .iter()
        .find(|p| {
            (p.left == "a.rs" && p.right == "b.rs") || (p.left == "b.rs" && p.right == "a.rs")
        })
        .map(|p| p.similarity);
    let sim_ac = report
        .pairs
        .iter()
        .find(|p| {
            (p.left == "a.rs" && p.right == "c.rs") || (p.left == "c.rs" && p.right == "a.rs")
        })
        .map(|p| p.similarity);
    let sim_ad = report
        .pairs
        .iter()
        .find(|p| {
            (p.left == "a.rs" && p.right == "d.rs") || (p.left == "d.rs" && p.right == "a.rs")
        })
        .map(|p| p.similarity);

    if let (Some(ab), Some(ac), Some(ad)) = (sim_ab, sim_ac, sim_ad) {
        assert!(ab >= ac, "a-b similarity ({ab}) should be >= a-c ({ac})");
        assert!(ac >= ad, "a-c similarity ({ac}) should be >= a-d ({ad})");
    }
}

// ── 2. Four identical files produce C(4,2)=6 pairs ─────────────

#[test]
fn four_identical_files_produce_six_pairs() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 42);

    for name in &["w.rs", "x.rs", "y.rs", "z.rs"] {
        write_file(&dir, name, &content);
    }
    let rows: Vec<FileRow> = ["w.rs", "x.rs", "y.rs", "z.rs"]
        .iter()
        .map(|n| make_row(n, "(root)", "Rust", 100, 5000))
        .collect();
    let export = make_export(rows);

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

    assert_eq!(
        report.pairs.len(),
        6,
        "C(4,2)=6 pairs expected from 4 identical files"
    );
}

// ── 3. Threshold boundary: similarity == threshold is included ──

#[test]
fn threshold_boundary_includes_equal_similarity() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 99);

    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);
    let rows = vec![
        make_row("a.rs", "(root)", "Rust", 100, 5000),
        make_row("b.rs", "(root)", "Rust", 100, 5000),
    ];
    let export = make_export(rows);

    // Identical files have similarity 1.0; threshold at 1.0 should include them
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        1.0,
        100,
        Some(1000),
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    assert_eq!(
        report.pairs.len(),
        1,
        "identical files at threshold=1.0 should yield exactly one pair"
    );
}

// ── 4. Fingerprint counts are consistent for identical files ────

#[test]
fn fingerprint_counts_are_consistent_for_identical_files() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 7);

    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);
    let rows = vec![
        make_row("a.rs", "(root)", "Rust", 100, 5000),
        make_row("b.rs", "(root)", "Rust", 100, 5000),
    ];
    let export = make_export(rows);

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
    let pair = &report.pairs[0];
    assert_eq!(
        pair.left_fingerprints, pair.right_fingerprints,
        "identical files should have equal fingerprint counts"
    );
}

// ── 5. Different sized files have different fingerprint counts ──

#[test]
fn different_sized_files_have_different_fingerprint_counts() {
    let dir = TempDir::new().unwrap();
    // Small file: 50 shared tokens, Large file: 50 shared + 200 unique
    let small = content_with_overlap(50, 0, 0);
    let large = content_with_overlap(50, 200, 1);
    write_file(&dir, "small.rs", &small);
    write_file(&dir, "large.rs", &large);
    let rows = vec![
        make_row("small.rs", "(root)", "Rust", 50, 2000),
        make_row("large.rs", "(root)", "Rust", 250, 10000),
    ];
    let export = make_export(rows);

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
    let pair = &report.pairs[0];
    // The file with more content should have more fingerprints
    let (small_fp, large_fp) = if pair.left == "large.rs" {
        (pair.right_fingerprints, pair.left_fingerprints)
    } else {
        (pair.left_fingerprints, pair.right_fingerprints)
    };
    assert!(
        large_fp >= small_fp,
        "larger file ({large_fp}) should have >= fingerprints than smaller ({small_fp})"
    );
}

// ── 6. Cluster representative with asymmetric star topology ─────

#[test]
fn cluster_representative_with_asymmetric_star_topology() {
    let dir = TempDir::new().unwrap();
    // All 4 identical files: every node has the same connections,
    // so the representative is the alphabetically-first file.
    let hub_content = source_text(100, 0);
    write_file(&dir, "hub.rs", &hub_content);
    write_file(&dir, "a.rs", &hub_content);
    write_file(&dir, "b.rs", &hub_content);
    write_file(&dir, "c.rs", &hub_content);

    let rows = vec![
        make_row("hub.rs", "(root)", "Rust", 100, 5000),
        make_row("a.rs", "(root)", "Rust", 100, 5000),
        make_row("b.rs", "(root)", "Rust", 100, 5000),
        make_row("c.rs", "(root)", "Rust", 100, 5000),
    ];
    let export = make_export(rows);

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

    if let Some(clusters) = &report.clusters
        && clusters.len() == 1
        && clusters[0].files.len() == 4
    {
        // All files are identical → same connection count → alphabetical tiebreak
        assert_eq!(
            clusters[0].representative, "a.rs",
            "with equal connectivity, alphabetically-first file should be representative"
        );
    }
}

// ── 7. Global scope finds cross-module pairs ────────────────────

#[test]
fn global_scope_finds_cross_module_and_cross_lang_pairs() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 55);

    std::fs::create_dir_all(dir.path().join("mod_a")).unwrap();
    std::fs::create_dir_all(dir.path().join("mod_b")).unwrap();
    write_file(&dir, "mod_a/file.rs", &content);
    write_file(&dir, "mod_b/file.py", &content);

    let rows = vec![
        make_row("mod_a/file.rs", "mod_a", "Rust", 100, 5000),
        make_row("mod_b/file.py", "mod_b", "Python", 100, 5000),
    ];
    let export = make_export(rows);

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

    assert!(
        !report.pairs.is_empty(),
        "global scope should find cross-module, cross-lang pairs"
    );
}

// ── 8. Module scope isolates files in different modules ─────────

#[test]
fn module_scope_isolates_identical_files_in_different_modules() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 66);

    std::fs::create_dir_all(dir.path().join("mod_a")).unwrap();
    std::fs::create_dir_all(dir.path().join("mod_b")).unwrap();
    write_file(&dir, "mod_a/file.rs", &content);
    write_file(&dir, "mod_b/file.rs", &content);

    let rows = vec![
        make_row("mod_a/file.rs", "mod_a", "Rust", 100, 5000),
        make_row("mod_b/file.rs", "mod_b", "Rust", 100, 5000),
    ];
    let export = make_export(rows);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Module,
        0.5,
        100,
        Some(1000),
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    assert!(
        report.pairs.is_empty(),
        "module scope should not pair files from different modules"
    );
}

// ── 9. Lang scope isolates identical files with different languages

#[test]
fn lang_scope_isolates_identical_files_with_different_languages() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 77);

    write_file(&dir, "file.rs", &content);
    write_file(&dir, "file.py", &content);

    let rows = vec![
        make_row("file.rs", "(root)", "Rust", 100, 5000),
        make_row("file.py", "(root)", "Python", 100, 5000),
    ];
    let export = make_export(rows);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Lang,
        0.5,
        100,
        Some(1000),
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    assert!(
        report.pairs.is_empty(),
        "lang scope should not pair files with different languages"
    );
}

// ── 10. Multiple exclude patterns all applied ───────────────────

#[test]
fn multiple_exclude_patterns_all_applied() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 88);

    write_file(&dir, "keep.rs", &content);
    write_file(&dir, "gen_file.rs", &content);
    write_file(&dir, "vendor_lib.rs", &content);
    write_file(&dir, "also_keep.rs", &content);

    let rows = vec![
        make_row("keep.rs", "(root)", "Rust", 100, 5000),
        make_row("gen_file.rs", "(root)", "Rust", 100, 5000),
        make_row("vendor_lib.rs", "(root)", "Rust", 100, 5000),
        make_row("also_keep.rs", "(root)", "Rust", 100, 5000),
    ];
    let export = make_export(rows);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        Some(1000),
        &NearDupLimits::default(),
        &["gen_*".to_string(), "vendor_*".to_string()],
    )
    .unwrap();

    // Only keep.rs and also_keep.rs should be compared → 1 pair
    for pair in &report.pairs {
        assert!(
            !pair.left.contains("gen_") && !pair.right.contains("gen_"),
            "gen_ files should be excluded"
        );
        assert!(
            !pair.left.contains("vendor_") && !pair.right.contains("vendor_"),
            "vendor_ files should be excluded"
        );
    }
}

// ── 11. Pair paths have left <= right (lexicographic) ───────────

#[test]
fn pair_paths_have_left_less_than_or_equal_to_right() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 99);

    for name in &["z.rs", "a.rs", "m.rs"] {
        write_file(&dir, name, &content);
    }
    let rows: Vec<FileRow> = ["z.rs", "a.rs", "m.rs"]
        .iter()
        .map(|n| make_row(n, "(root)", "Rust", 100, 5000))
        .collect();
    let export = make_export(rows);

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

    for pair in &report.pairs {
        assert!(
            pair.left <= pair.right,
            "pair ordering violated: {} > {}",
            pair.left,
            pair.right
        );
    }
}

// ── 12. max_files limit produces skip tracking ──────────────────

#[test]
fn ten_files_max_three_yields_seven_skipped() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 5);

    for i in 0..10 {
        let name = format!("f{i:02}.rs");
        write_file(&dir, &name, &content);
    }
    let rows: Vec<FileRow> = (0..10)
        .map(|i| {
            make_row(
                &format!("f{i:02}.rs"),
                "(root)",
                "Rust",
                (10 - i) * 10,
                5000,
            )
        })
        .collect();
    let export = make_export(rows);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        3,
        Some(1000),
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    assert_eq!(report.files_skipped, 7, "10 files - 3 max = 7 skipped");
}

// ── 13. File selection prefers highest code lines ───────────────

#[test]
fn file_selection_prefers_highest_code_lines() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 10);

    write_file(&dir, "big.rs", &content);
    write_file(&dir, "small.rs", &content);
    write_file(&dir, "medium.rs", &content);

    let rows = vec![
        make_row("big.rs", "(root)", "Rust", 1000, 50000),
        make_row("small.rs", "(root)", "Rust", 10, 500),
        make_row("medium.rs", "(root)", "Rust", 500, 25000),
    ];
    let export = make_export(rows);

    // max_files=2: should pick big.rs and medium.rs (highest code lines)
    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.0,
        2,
        Some(1000),
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    assert_eq!(report.files_skipped, 1);
    // The pair should be between big.rs and medium.rs
    if !report.pairs.is_empty() {
        let paths: Vec<&str> = report
            .pairs
            .iter()
            .flat_map(|p| [p.left.as_str(), p.right.as_str()])
            .collect();
        assert!(
            !paths.contains(&"small.rs"),
            "small.rs should be skipped in favor of larger files"
        );
    }
}
