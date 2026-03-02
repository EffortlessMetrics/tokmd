//! Integration tests for tokmd-analysis-near-dup.

use std::io::Write;
use tempfile::TempDir;
use tokmd_analysis_near_dup::{NearDupLimits, build_near_dup_report};
use tokmd_analysis_types::NearDupScope;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

/// Generate a Rust-like source body with `n` unique functions, producing enough
/// tokens to be fingerprinted (>= K=25 tokens).
fn rust_body(n: usize) -> String {
    (0..n)
        .map(|i| {
            format!(
                "fn func_{i}(arg: u32) -> u32 {{ let result = arg + {i}; println!(\"value is {{}}\", result); result }}\n"
            )
        })
        .collect()
}

/// Create a `FileRow` with sensible defaults for testing.
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

/// Build an ExportData from a list of FileRows.
fn export_from(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    }
}

/// Write a file into the temp directory and return its content length.
fn write_file(dir: &TempDir, rel_path: &str, content: &str) -> usize {
    let full = dir.path().join(rel_path);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let mut f = std::fs::File::create(&full).unwrap();
    f.write_all(content.as_bytes()).unwrap();
    content.len()
}

// ---------------------------------------------------------------------------
// 1. Identical files produce similarity 1.0
// ---------------------------------------------------------------------------

#[test]
fn identical_files_are_detected() {
    let dir = TempDir::new().unwrap();
    let body = rust_body(10);
    let len_a = write_file(&dir, "a.rs", &body);
    let len_b = write_file(&dir, "b.rs", &body);

    let export = export_from(vec![
        make_row("a.rs", "root", "Rust", 10, len_a),
        make_row("b.rs", "root", "Rust", 10, len_b),
    ]);

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
    assert!(
        (report.pairs[0].similarity - 1.0).abs() < 1e-6,
        "identical files should have similarity ~1.0, got {}",
        report.pairs[0].similarity
    );
}

// ---------------------------------------------------------------------------
// 2. Slightly different files still pair above threshold
// ---------------------------------------------------------------------------

#[test]
fn slightly_different_files_pair_above_threshold() {
    let dir = TempDir::new().unwrap();
    let base = rust_body(20);
    // Append a small unique suffix to one file
    let modified = format!("{base}\nfn extra_unique_function(x: u32) -> u32 {{ x + 999 }}\n");

    let len_a = write_file(&dir, "a.rs", &base);
    let len_b = write_file(&dir, "b.rs", &modified);

    let export = export_from(vec![
        make_row("a.rs", "root", "Rust", 20, len_a),
        make_row("b.rs", "root", "Rust", 21, len_b),
    ]);

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
    assert!(
        report.pairs[0].similarity > 0.5,
        "slightly different files should still be above threshold"
    );
    assert!(
        report.pairs[0].similarity < 1.0,
        "slightly different files should be below 1.0"
    );
}

// ---------------------------------------------------------------------------
// 3. Dissimilar files produce no pairs
// ---------------------------------------------------------------------------

#[test]
fn dissimilar_files_produce_no_pairs() {
    let dir = TempDir::new().unwrap();
    // Two completely different bodies with distinct tokens
    let body_a: String = (0..30)
        .map(|i| format!("fn alpha_{i}(x: i64) -> i64 {{ let val = x * {i}; val }}\n"))
        .collect();
    let body_b: String = (0..30)
        .map(|i| {
            format!(
                "fn bravo_{i}(y: String) -> String {{ let msg = format!(\"hello {{}}\", y); msg }}\n"
            )
        })
        .collect();

    let len_a = write_file(&dir, "alpha.rs", &body_a);
    let len_b = write_file(&dir, "bravo.rs", &body_b);

    let export = export_from(vec![
        make_row("alpha.rs", "root", "Rust", 30, len_a),
        make_row("bravo.rs", "root", "Rust", 30, len_b),
    ]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.8,
        100,
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    assert!(
        report.pairs.is_empty(),
        "dissimilar files should not pair at threshold 0.8"
    );
}

// ---------------------------------------------------------------------------
// 4. Clustering groups connected pairs
// ---------------------------------------------------------------------------

#[test]
fn clustering_groups_connected_files() {
    let dir = TempDir::new().unwrap();
    let base = rust_body(20);
    let variant1 = format!("{base}\nfn variant1(x: u32) -> u32 {{ x + 1 }}\n");
    let variant2 = format!("{base}\nfn variant2(x: u32) -> u32 {{ x + 2 }}\n");

    let len_a = write_file(&dir, "a.rs", &base);
    let len_b = write_file(&dir, "b.rs", &variant1);
    let len_c = write_file(&dir, "c.rs", &variant2);

    let export = export_from(vec![
        make_row("a.rs", "root", "Rust", 20, len_a),
        make_row("b.rs", "root", "Rust", 21, len_b),
        make_row("c.rs", "root", "Rust", 21, len_c),
    ]);

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

    assert!(!report.pairs.is_empty(), "should detect near-dup pairs");
    let clusters = report.clusters.as_ref().expect("clusters should be Some");
    // All three files should be in a single connected-component cluster
    assert_eq!(clusters.len(), 1, "all files should form one cluster");
    assert_eq!(clusters[0].files.len(), 3);
}

// ---------------------------------------------------------------------------
// 5. Deterministic output: BTreeMap ordering
// ---------------------------------------------------------------------------

#[test]
fn output_is_deterministic() {
    let dir = TempDir::new().unwrap();
    let body = rust_body(15);
    let variant = format!("{body}\nfn unique_extra() -> bool {{ true }}\n");

    let len_a = write_file(&dir, "a.rs", &body);
    let len_b = write_file(&dir, "b.rs", &variant);

    let export = export_from(vec![
        make_row("a.rs", "root", "Rust", 15, len_a),
        make_row("b.rs", "root", "Rust", 16, len_b),
    ]);

    let r1 = build_near_dup_report(
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
    let r2 = build_near_dup_report(
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

    assert_eq!(r1.pairs.len(), r2.pairs.len());
    for (p1, p2) in r1.pairs.iter().zip(r2.pairs.iter()) {
        assert_eq!(p1.left, p2.left);
        assert_eq!(p1.right, p2.right);
        assert!((p1.similarity - p2.similarity).abs() < 1e-10);
    }
}

// ---------------------------------------------------------------------------
// 6. Child file rows are excluded (only Parent rows scanned)
// ---------------------------------------------------------------------------

#[test]
fn child_file_rows_are_excluded() {
    let dir = TempDir::new().unwrap();
    let body = rust_body(10);
    let len_a = write_file(&dir, "a.rs", &body);
    let len_b = write_file(&dir, "b.rs", &body);

    let mut row_b = make_row("b.rs", "root", "Rust", 10, len_b);
    row_b.kind = FileKind::Child;

    let export = export_from(vec![make_row("a.rs", "root", "Rust", 10, len_a), row_b]);

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

    // Only 1 parent file eligible => no pairs
    assert!(report.pairs.is_empty());
    assert_eq!(report.files_analyzed, 1);
}

// ---------------------------------------------------------------------------
// 7. max_file_bytes limit filters large files
// ---------------------------------------------------------------------------

#[test]
fn max_file_bytes_filters_large_files() {
    let dir = TempDir::new().unwrap();
    let body = rust_body(10);
    let len = write_file(&dir, "a.rs", &body);
    write_file(&dir, "b.rs", &body);

    let export = export_from(vec![
        make_row("a.rs", "root", "Rust", 10, len),
        make_row("b.rs", "root", "Rust", 10, len),
    ]);

    // Set max_file_bytes smaller than the files
    let limits = NearDupLimits {
        max_bytes: None,
        max_file_bytes: Some(10),
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

    assert_eq!(report.files_analyzed, 0);
    assert!(report.pairs.is_empty());
}

// ---------------------------------------------------------------------------
// 8. max_files caps the number of files analyzed
// ---------------------------------------------------------------------------

#[test]
fn max_files_caps_file_count() {
    let dir = TempDir::new().unwrap();
    let body = rust_body(10);
    let len_a = write_file(&dir, "a.rs", &body);
    let len_b = write_file(&dir, "b.rs", &body);
    let len_c = write_file(&dir, "c.rs", &body);

    let export = export_from(vec![
        make_row("a.rs", "root", "Rust", 10, len_a),
        make_row("b.rs", "root", "Rust", 10, len_b),
        make_row("c.rs", "root", "Rust", 10, len_c),
    ]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        2, // only allow 2 files
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    assert_eq!(report.files_analyzed, 2);
    assert_eq!(report.files_skipped, 1);
}

// ---------------------------------------------------------------------------
// 9. max_pairs truncates output and sets truncated flag
// ---------------------------------------------------------------------------

#[test]
fn max_pairs_truncates_and_sets_flag() {
    let dir = TempDir::new().unwrap();
    let base = rust_body(20);
    let v1 = format!("{base}\nfn v1() -> u32 {{ 1 }}\n");
    let v2 = format!("{base}\nfn v2() -> u32 {{ 2 }}\n");
    let v3 = format!("{base}\nfn v3() -> u32 {{ 3 }}\n");

    let len_base = write_file(&dir, "base.rs", &base);
    let len_v1 = write_file(&dir, "v1.rs", &v1);
    let len_v2 = write_file(&dir, "v2.rs", &v2);
    let len_v3 = write_file(&dir, "v3.rs", &v3);

    let export = export_from(vec![
        make_row("base.rs", "root", "Rust", 20, len_base),
        make_row("v1.rs", "root", "Rust", 21, len_v1),
        make_row("v2.rs", "root", "Rust", 21, len_v2),
        make_row("v3.rs", "root", "Rust", 21, len_v3),
    ]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        Some(1), // truncate to 1 pair
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    assert_eq!(report.pairs.len(), 1);
    assert!(report.truncated);
    // Clusters are built from ALL pairs before truncation
    let clusters = report.clusters.as_ref().expect("should have clusters");
    assert!(!clusters.is_empty());
}

// ---------------------------------------------------------------------------
// 10. Scope::Module partitions by module
// ---------------------------------------------------------------------------

#[test]
fn module_scope_partitions_files() {
    let dir = TempDir::new().unwrap();
    let body = rust_body(10);

    let len_a = write_file(&dir, "mod_a/a.rs", &body);
    let len_b = write_file(&dir, "mod_a/b.rs", &body);
    let len_c = write_file(&dir, "mod_b/c.rs", &body);

    let export = export_from(vec![
        make_row("mod_a/a.rs", "mod_a", "Rust", 10, len_a),
        make_row("mod_a/b.rs", "mod_a", "Rust", 10, len_b),
        make_row("mod_b/c.rs", "mod_b", "Rust", 10, len_c),
    ]);

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

    // Only mod_a has 2 files that can pair; mod_b has 1 file (no pairs)
    assert_eq!(report.pairs.len(), 1);
    assert_eq!(report.pairs[0].left, "mod_a/a.rs");
    assert_eq!(report.pairs[0].right, "mod_a/b.rs");
}

// ---------------------------------------------------------------------------
// 11. Scope::Lang partitions by language
// ---------------------------------------------------------------------------

#[test]
fn lang_scope_partitions_by_language() {
    let dir = TempDir::new().unwrap();
    let body = rust_body(10);

    let len_a = write_file(&dir, "a.rs", &body);
    let len_b = write_file(&dir, "b.rs", &body);
    let len_c = write_file(&dir, "c.py", &body);

    let export = export_from(vec![
        make_row("a.rs", "root", "Rust", 10, len_a),
        make_row("b.rs", "root", "Rust", 10, len_b),
        make_row("c.py", "root", "Python", 10, len_c),
    ]);

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

    // Only the two "Rust" files pair; the "Python" file is alone in its partition
    assert_eq!(report.pairs.len(), 1);
    assert_eq!(report.pairs[0].left, "a.rs");
    assert_eq!(report.pairs[0].right, "b.rs");
}

// ---------------------------------------------------------------------------
// 12. Exclude patterns filter files by glob
// ---------------------------------------------------------------------------

#[test]
fn exclude_patterns_filter_files() {
    let dir = TempDir::new().unwrap();
    let body = rust_body(10);
    let len_a = write_file(&dir, "a.rs", &body);
    let len_b = write_file(&dir, "b.rs", &body);
    let len_c = write_file(&dir, "test_c.rs", &body);

    let export = export_from(vec![
        make_row("a.rs", "root", "Rust", 10, len_a),
        make_row("b.rs", "root", "Rust", 10, len_b),
        make_row("test_c.rs", "root", "Rust", 10, len_c),
    ]);

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
    assert_eq!(report.files_analyzed, 2);
}

// ---------------------------------------------------------------------------
// 13. Empty export data produces empty report
// ---------------------------------------------------------------------------

#[test]
fn empty_export_produces_empty_report() {
    let dir = TempDir::new().unwrap();
    let export = export_from(vec![]);

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

    assert_eq!(report.files_analyzed, 0);
    assert!(report.pairs.is_empty());
    assert!(report.clusters.is_none());
    assert!(!report.truncated);
}

// ---------------------------------------------------------------------------
// 14. Single file produces no pairs
// ---------------------------------------------------------------------------

#[test]
fn single_file_produces_no_pairs() {
    let dir = TempDir::new().unwrap();
    let body = rust_body(10);
    let len = write_file(&dir, "only.rs", &body);

    let export = export_from(vec![make_row("only.rs", "root", "Rust", 10, len)]);

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
    assert!(report.clusters.is_none());
    assert_eq!(report.files_analyzed, 1);
}

// ---------------------------------------------------------------------------
// 15. Pairs are sorted by similarity descending, then left, then right
// ---------------------------------------------------------------------------

#[test]
fn pairs_sorted_by_similarity_desc_then_path() {
    let dir = TempDir::new().unwrap();
    let base = rust_body(20);
    // Create files with varying degrees of divergence
    let close_variant = format!("{base}\nfn close() -> bool {{ true }}\n");
    let far_variant = {
        let half_base: String = (0..10)
            .map(|i| {
                format!(
                    "fn func_{i}(arg: u32) -> u32 {{ let result = arg + {i}; println!(\"value is {{}}\", result); result }}\n"
                )
            })
            .collect();
        let extra: String = (100..120)
            .map(|i| format!("fn different_{i}(z: f64) -> f64 {{ z * {i} as f64 }}\n"))
            .collect();
        format!("{half_base}{extra}")
    };

    let len_a = write_file(&dir, "a.rs", &base);
    let len_b = write_file(&dir, "b.rs", &close_variant);
    let len_c = write_file(&dir, "c.rs", &far_variant);

    let export = export_from(vec![
        make_row("a.rs", "root", "Rust", 20, len_a),
        make_row("b.rs", "root", "Rust", 21, len_b),
        make_row("c.rs", "root", "Rust", 30, len_c),
    ]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.1, // low threshold to capture all pairs
        100,
        None,
        &NearDupLimits::default(),
        &[],
    )
    .unwrap();

    // Verify pairs are sorted by similarity descending
    for window in report.pairs.windows(2) {
        assert!(
            window[0].similarity >= window[1].similarity,
            "pairs should be sorted by similarity desc: {} >= {}",
            window[0].similarity,
            window[1].similarity
        );
    }
}

// ---------------------------------------------------------------------------
// 16. Report params reflect input configuration
// ---------------------------------------------------------------------------

#[test]
fn report_params_reflect_configuration() {
    let dir = TempDir::new().unwrap();
    let export = export_from(vec![]);

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
        &["*.test.rs".to_string()],
    )
    .unwrap();

    assert!(matches!(report.params.scope, NearDupScope::Lang));
    assert!((report.params.threshold - 0.75).abs() < 1e-10);
    assert_eq!(report.params.max_files, 42);
    assert_eq!(report.params.max_pairs, Some(10));
    assert_eq!(report.params.max_file_bytes, Some(1024));
    assert_eq!(report.params.exclude_patterns, vec!["*.test.rs"]);
    let algo = report.params.algorithm.as_ref().unwrap();
    assert_eq!(algo.k_gram_size, 25);
    assert_eq!(algo.window_size, 4);
    assert_eq!(algo.max_postings, 50);
}

// ---------------------------------------------------------------------------
// 17. Files too short to fingerprint produce no pairs
// ---------------------------------------------------------------------------

#[test]
fn short_files_produce_no_pairs() {
    let dir = TempDir::new().unwrap();
    // Content with fewer than K=25 tokens will have no fingerprints
    let short = "fn main() { println!(\"hi\"); }";
    let len_a = write_file(&dir, "a.rs", short);
    let len_b = write_file(&dir, "b.rs", short);

    let export = export_from(vec![
        make_row("a.rs", "root", "Rust", 1, len_a),
        make_row("b.rs", "root", "Rust", 1, len_b),
    ]);

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

    // Files too short to produce k-grams -> no fingerprints -> no pairs
    assert!(report.pairs.is_empty());
}
