//! Deeper tests for near-duplicate detection: clustering behavior,
//! BTreeMap ordering (determinism), empty inputs, single files, and identical files.

use std::io::Write;

use tempfile::TempDir;
use tokmd_analysis_near_dup::{NearDupLimits, build_near_dup_report};
use tokmd_analysis_types::NearDupScope;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ──────────────────────────────────────────────────────

/// Generate source text with `n` distinct tokens (enough for winnowing K=25).
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

fn write_file(dir: &TempDir, path: &str, content: &str) {
    let full = dir.path().join(path);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let mut f = std::fs::File::create(&full).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}

// ── Empty input ─────────────────────────────────────────────────

#[test]
fn empty_export_yields_empty_report() {
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

    assert!(report.pairs.is_empty());
    assert!(report.clusters.is_none());
    assert_eq!(report.files_analyzed, 0);
    assert!(!report.truncated);
}

// ── Single file ─────────────────────────────────────────────────

#[test]
fn single_file_yields_no_pairs() {
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

    assert!(report.pairs.is_empty());
    assert!(report.clusters.is_none());
    assert_eq!(report.files_analyzed, 1);
}

// ── Identical files ─────────────────────────────────────────────

#[test]
fn identical_files_detected_with_high_similarity() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 42);
    write_file(&dir, "copy_a.rs", &content);
    write_file(&dir, "copy_b.rs", &content);

    let rows = vec![
        make_file_row("copy_a.rs", "root", "Rust", 100, content.len()),
        make_file_row("copy_b.rs", "root", "Rust", 100, content.len()),
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

    // Cluster should contain both files
    let clusters = report.clusters.as_ref().unwrap();
    assert_eq!(clusters.len(), 1);
    assert_eq!(clusters[0].files.len(), 2);
}

// ── Completely different files ───────────────────────────────────

#[test]
fn completely_different_files_no_pairs() {
    let dir = TempDir::new().unwrap();
    let content_a = source_text(100, 0);
    let content_b = source_text(100, 999);
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
        "completely different files should not be paired"
    );
    assert_eq!(report.files_analyzed, 2);
}

// ── BTreeMap ordering (determinism) ─────────────────────────────

#[test]
fn report_is_deterministic_across_runs() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 7);
    write_file(&dir, "x.rs", &content);
    write_file(&dir, "y.rs", &content);
    write_file(&dir, "z.rs", &content);

    let rows = vec![
        make_file_row("x.rs", "root", "Rust", 100, content.len()),
        make_file_row("y.rs", "root", "Rust", 100, content.len()),
        make_file_row("z.rs", "root", "Rust", 100, content.len()),
    ];
    let export = make_export(rows);

    let r1 = build_near_dup_report(
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

    let r2 = build_near_dup_report(
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

    assert_eq!(r1.pairs.len(), r2.pairs.len());
    for (a, b) in r1.pairs.iter().zip(r2.pairs.iter()) {
        assert_eq!(a.left, b.left);
        assert_eq!(a.right, b.right);
        assert!((a.similarity - b.similarity).abs() < f64::EPSILON);
        assert_eq!(a.shared_fingerprints, b.shared_fingerprints);
    }

    // Cluster order must also be deterministic
    let c1 = r1.clusters.as_ref().unwrap();
    let c2 = r2.clusters.as_ref().unwrap();
    assert_eq!(c1.len(), c2.len());
    for (a, b) in c1.iter().zip(c2.iter()) {
        assert_eq!(a.files, b.files);
        assert_eq!(a.representative, b.representative);
        assert_eq!(a.pair_count, b.pair_count);
    }
}

#[test]
fn pairs_sorted_by_similarity_desc_then_left_then_right() {
    let dir = TempDir::new().unwrap();
    // Create three similar files to generate multiple pairs
    let base = source_text(100, 0);
    // Slightly different variant
    let variant = format!("{} extra_token_for_variance", &base);
    write_file(&dir, "a.rs", &base);
    write_file(&dir, "b.rs", &base);
    write_file(&dir, "c.rs", &variant);

    let rows = vec![
        make_file_row("a.rs", "root", "Rust", 100, base.len()),
        make_file_row("b.rs", "root", "Rust", 100, base.len()),
        make_file_row("c.rs", "root", "Rust", 101, variant.len()),
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

    // Verify sort order: descending similarity, then left, then right
    for pair in report.pairs.windows(2) {
        let order_ok = pair[0].similarity > pair[1].similarity
            || (pair[0].similarity == pair[1].similarity && pair[0].left < pair[1].left)
            || (pair[0].similarity == pair[1].similarity
                && pair[0].left == pair[1].left
                && pair[0].right <= pair[1].right);
        assert!(
            order_ok,
            "pairs not in expected order: ({}, {}, {:.4}) then ({}, {}, {:.4})",
            pair[0].left,
            pair[0].right,
            pair[0].similarity,
            pair[1].left,
            pair[1].right,
            pair[1].similarity,
        );
    }
}

// ── Cluster file lists are sorted ───────────────────────────────

#[test]
fn cluster_file_lists_are_sorted_alphabetically() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 3);
    write_file(&dir, "zebra.rs", &content);
    write_file(&dir, "apple.rs", &content);
    write_file(&dir, "mango.rs", &content);

    let rows = vec![
        make_file_row("zebra.rs", "root", "Rust", 100, content.len()),
        make_file_row("apple.rs", "root", "Rust", 100, content.len()),
        make_file_row("mango.rs", "root", "Rust", 100, content.len()),
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

    let clusters = report.clusters.as_ref().unwrap();
    for cluster in clusters {
        let sorted: Vec<String> = {
            let mut v = cluster.files.clone();
            v.sort();
            v
        };
        assert_eq!(
            cluster.files, sorted,
            "cluster files should be sorted alphabetically"
        );
    }
}

// ── Scope partitioning ──────────────────────────────────────────

#[test]
fn module_scope_only_compares_within_module() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 0);
    write_file(&dir, "mod_a/x.rs", &content);
    write_file(&dir, "mod_a/y.rs", &content);
    write_file(&dir, "mod_b/z.rs", &content);

    let rows = vec![
        make_file_row("mod_a/x.rs", "mod_a", "Rust", 100, content.len()),
        make_file_row("mod_a/y.rs", "mod_a", "Rust", 100, content.len()),
        make_file_row("mod_b/z.rs", "mod_b", "Rust", 100, content.len()),
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

    // Only mod_a/x.rs and mod_a/y.rs should be paired (same module)
    // mod_b/z.rs is alone in its partition
    assert_eq!(report.pairs.len(), 1);
    assert!(
        (report.pairs[0].left.contains("mod_a") && report.pairs[0].right.contains("mod_a")),
        "pair should be within mod_a"
    );
}

#[test]
fn lang_scope_only_compares_within_language() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 0);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);
    write_file(&dir, "c.py", &content);

    let rows = vec![
        make_file_row("a.rs", "root", "Rust", 100, content.len()),
        make_file_row("b.rs", "root", "Rust", 100, content.len()),
        make_file_row("c.py", "root", "Python", 100, content.len()),
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

    // a.rs and b.rs paired (same lang), c.py alone in Python partition
    assert_eq!(report.pairs.len(), 1);
    assert_eq!(report.pairs[0].left, "a.rs");
    assert_eq!(report.pairs[0].right, "b.rs");
}

// ── Truncation ──────────────────────────────────────────────────

#[test]
fn max_pairs_truncates_output() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 0);
    // Create 4 identical files → 6 pairs
    for name in &["a.rs", "b.rs", "c.rs", "d.rs"] {
        write_file(&dir, name, &content);
    }

    let rows: Vec<FileRow> = ["a.rs", "b.rs", "c.rs", "d.rs"]
        .iter()
        .map(|p| make_file_row(p, "root", "Rust", 100, content.len()))
        .collect();

    let report = build_near_dup_report(
        dir.path(),
        &make_export(rows),
        NearDupScope::Global,
        0.5,
        1000,
        Some(2),
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    assert!(report.pairs.len() <= 2, "pairs should be truncated to 2");
    assert!(report.truncated, "truncated flag should be true");
    // Clusters should still reflect all files (built before truncation)
    let clusters = report.clusters.as_ref().unwrap();
    let total_files: usize = clusters.iter().map(|c| c.files.len()).sum();
    assert!(total_files >= 4, "clusters should include all 4 files");
}

// ── Exclude patterns ────────────────────────────────────────────

#[test]
fn exclude_patterns_filter_files() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 0);
    write_file(&dir, "src/main.rs", &content);
    write_file(&dir, "src/lib.rs", &content);
    write_file(&dir, "vendor/dep.rs", &content);

    let rows = vec![
        make_file_row("src/main.rs", "src", "Rust", 100, content.len()),
        make_file_row("src/lib.rs", "src", "Rust", 100, content.len()),
        make_file_row("vendor/dep.rs", "vendor", "Rust", 100, content.len()),
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

    assert_eq!(report.excluded_by_pattern, Some(1));
    // Only src/ files should be compared
    for pair in &report.pairs {
        assert!(
            !pair.left.starts_with("vendor") && !pair.right.starts_with("vendor"),
            "vendor files should be excluded"
        );
    }
}

// ── Short files (fewer tokens than K) yield no fingerprints ─────

#[test]
fn short_files_produce_no_pairs() {
    let dir = TempDir::new().unwrap();
    // Only 5 tokens, below K=25 threshold
    let short = "fn main() { hello }";
    write_file(&dir, "short_a.rs", short);
    write_file(&dir, "short_b.rs", short);

    let rows = vec![
        make_file_row("short_a.rs", "root", "Rust", 1, short.len()),
        make_file_row("short_b.rs", "root", "Rust", 1, short.len()),
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
        "files shorter than K tokens should produce no fingerprints and no pairs"
    );
}

// ── Max files limit ─────────────────────────────────────────────

#[test]
fn max_files_limits_scope() {
    let dir = TempDir::new().unwrap();
    let content = source_text(100, 0);
    for name in &["a.rs", "b.rs", "c.rs", "d.rs", "e.rs"] {
        write_file(&dir, name, &content);
    }

    let rows: Vec<FileRow> = ["a.rs", "b.rs", "c.rs", "d.rs", "e.rs"]
        .iter()
        .map(|p| make_file_row(p, "root", "Rust", 100, content.len()))
        .collect();

    let report = build_near_dup_report(
        dir.path(),
        &make_export(rows),
        NearDupScope::Global,
        0.5,
        2, // Only analyze top 2 files
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    assert_eq!(report.files_analyzed, 2);
    assert_eq!(report.files_skipped, 3);
}

// ── Inverted index uses BTreeMap for deterministic iteration ────

#[test]
fn btreemap_ordering_guarantees_deterministic_pairs() {
    // This test verifies that the HashMap→BTreeMap fix produces stable output
    // by running the same analysis multiple times and checking consistency.
    let dir = TempDir::new().unwrap();
    let base = source_text(100, 0);
    let variant1 = format!("{} extra_a", &base);
    let variant2 = format!("{} extra_b extra_c", &base);

    write_file(&dir, "orig.rs", &base);
    write_file(&dir, "var1.rs", &variant1);
    write_file(&dir, "var2.rs", &variant2);

    let rows = vec![
        make_file_row("orig.rs", "root", "Rust", 100, base.len()),
        make_file_row("var1.rs", "root", "Rust", 101, variant1.len()),
        make_file_row("var2.rs", "root", "Rust", 102, variant2.len()),
    ];
    let export = make_export(rows);

    // Run 5 times and collect results
    let mut all_results = Vec::new();
    for _ in 0..5 {
        let report = build_near_dup_report(
            dir.path(),
            &export,
            NearDupScope::Global,
            0.3,
            1000,
            None,
            &NearDupLimits::default(),
            &[],
        )
        .unwrap();
        all_results.push(report);
    }

    // All runs must produce identical pair lists
    let first = &all_results[0];
    for (i, result) in all_results.iter().enumerate().skip(1) {
        assert_eq!(
            first.pairs.len(),
            result.pairs.len(),
            "run {} has different pair count",
            i
        );
        for (j, (a, b)) in first.pairs.iter().zip(result.pairs.iter()).enumerate() {
            assert_eq!(a.left, b.left, "run {} pair {} left differs", i, j);
            assert_eq!(a.right, b.right, "run {} pair {} right differs", i, j);
            assert!(
                (a.similarity - b.similarity).abs() < f64::EPSILON,
                "run {} pair {} similarity differs: {} vs {}",
                i,
                j,
                a.similarity,
                b.similarity
            );
        }
    }
}
