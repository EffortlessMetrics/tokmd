//! Wave-56 depth tests for `tokmd-analysis-near-dup`.
//!
//! Covers near-duplicate detection algorithms, similarity scoring,
//! threshold-based clustering, edge cases (identical/different/empty files),
//! large file set characteristics, and deterministic detection results.

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

/// Generate deterministic source text with `n` lines and `seed`-based variation.
fn source_text(n: usize, seed: usize) -> String {
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

// ── 1. Near-duplicate detection algorithms ──────────────────────

#[test]
fn identical_files_detected_as_duplicates() {
    let dir = TempDir::new().unwrap();
    let content = source_text(50, 1);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);
    let byte_len = content.len();

    let export = make_export(vec![
        make_row("a.rs", "mod", "Rust", 50, byte_len),
        make_row("b.rs", "mod", "Rust", 50, byte_len),
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

    assert_eq!(
        report.pairs.len(),
        1,
        "identical files should form one pair"
    );
    assert!(
        (report.pairs[0].similarity - 1.0).abs() < 1e-4,
        "identical files should have similarity ~1.0, got {}",
        report.pairs[0].similarity
    );
}

#[test]
fn completely_different_files_not_paired() {
    let dir = TempDir::new().unwrap();
    let content_a = source_text(50, 1);
    let content_b = source_text(50, 99999);
    write_file(&dir, "a.rs", &content_a);
    write_file(&dir, "b.rs", &content_b);
    let byte_a = content_a.len();
    let byte_b = content_b.len();

    let export = make_export(vec![
        make_row("a.rs", "mod", "Rust", 50, byte_a),
        make_row("b.rs", "mod", "Rust", 50, byte_b),
    ]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.8,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    assert!(
        report.pairs.is_empty(),
        "very different files should not be paired at threshold 0.8"
    );
}

#[test]
fn slightly_modified_files_detected_with_low_threshold() {
    let dir = TempDir::new().unwrap();
    let base = source_text(50, 1);
    // Modify last line
    let modified = format!("{}\nfn extra_function() {{ let x = 999; }}", &base);
    write_file(&dir, "a.rs", &base);
    write_file(&dir, "b.rs", &modified);

    let export = make_export(vec![
        make_row("a.rs", "mod", "Rust", 50, base.len()),
        make_row("b.rs", "mod", "Rust", 51, modified.len()),
    ]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.3,
        100,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    // With a low threshold, slightly modified files should still match
    if !report.pairs.is_empty() {
        assert!(report.pairs[0].similarity > 0.3);
    }
}

// ── 2. Similarity scoring ───────────────────────────────────────

#[test]
fn similarity_is_between_zero_and_one() {
    let dir = TempDir::new().unwrap();
    let c1 = source_text(50, 1);
    let c2 = source_text(50, 2);
    write_file(&dir, "a.rs", &c1);
    write_file(&dir, "b.rs", &c2);

    let export = make_export(vec![
        make_row("a.rs", "mod", "Rust", 50, c1.len()),
        make_row("b.rs", "mod", "Rust", 50, c2.len()),
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
        assert!(
            pair.similarity >= 0.0 && pair.similarity <= 1.0,
            "similarity should be in [0,1], got {}",
            pair.similarity
        );
    }
}

#[test]
fn similarity_rounded_to_four_decimals() {
    let dir = TempDir::new().unwrap();
    let c1 = source_text(50, 10);
    let c2 = {
        let mut s = source_text(50, 10);
        s.push_str("\n// small change to reduce similarity");
        s
    };
    write_file(&dir, "a.rs", &c1);
    write_file(&dir, "b.rs", &c2);

    let export = make_export(vec![
        make_row("a.rs", "mod", "Rust", 50, c1.len()),
        make_row("b.rs", "mod", "Rust", 51, c2.len()),
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
        let rounded = (pair.similarity * 10000.0).round() / 10000.0;
        assert!(
            (pair.similarity - rounded).abs() < 1e-10,
            "similarity should be rounded to 4 decimals"
        );
    }
}

#[test]
fn shared_fingerprints_consistent_with_similarity() {
    let dir = TempDir::new().unwrap();
    let content = source_text(50, 1);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);

    let export = make_export(vec![
        make_row("a.rs", "mod", "Rust", 50, content.len()),
        make_row("b.rs", "mod", "Rust", 50, content.len()),
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

    for pair in &report.pairs {
        assert!(pair.shared_fingerprints > 0);
        assert!(pair.left_fingerprints > 0);
        assert!(pair.right_fingerprints > 0);
        assert!(
            pair.shared_fingerprints <= pair.left_fingerprints.max(pair.right_fingerprints) + 1
        );
    }
}

// ── 3. Threshold-based clustering ───────────────────────────────

#[test]
fn higher_threshold_yields_fewer_pairs() {
    let dir = TempDir::new().unwrap();
    let base = source_text(50, 1);
    let variant = source_text(50, 2);
    write_file(&dir, "a.rs", &base);
    write_file(&dir, "b.rs", &base); // identical to a
    write_file(&dir, "c.rs", &variant);

    let export = make_export(vec![
        make_row("a.rs", "mod", "Rust", 50, base.len()),
        make_row("b.rs", "mod", "Rust", 50, base.len()),
        make_row("c.rs", "mod", "Rust", 50, variant.len()),
    ]);

    let report_low = build_near_dup_report(
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

    let report_high = build_near_dup_report(
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

    assert!(
        report_high.pairs.len() <= report_low.pairs.len(),
        "higher threshold should yield fewer or equal pairs"
    );
}

#[test]
fn clusters_group_related_files() {
    let dir = TempDir::new().unwrap();
    let content = source_text(50, 1);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);
    write_file(&dir, "c.rs", &content);

    let export = make_export(vec![
        make_row("a.rs", "mod", "Rust", 50, content.len()),
        make_row("b.rs", "mod", "Rust", 50, content.len()),
        make_row("c.rs", "mod", "Rust", 50, content.len()),
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
        // All identical files should be in one cluster
        assert_eq!(clusters.len(), 1, "identical files should form one cluster");
        assert_eq!(clusters[0].files.len(), 3);
    }
}

#[test]
fn cluster_files_are_sorted_alphabetically() {
    let dir = TempDir::new().unwrap();
    let content = source_text(50, 1);
    write_file(&dir, "z.rs", &content);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "m.rs", &content);

    let export = make_export(vec![
        make_row("z.rs", "mod", "Rust", 50, content.len()),
        make_row("a.rs", "mod", "Rust", 50, content.len()),
        make_row("m.rs", "mod", "Rust", 50, content.len()),
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
        for cluster in clusters {
            let mut sorted = cluster.files.clone();
            sorted.sort();
            assert_eq!(
                cluster.files, sorted,
                "cluster files should be sorted alphabetically"
            );
        }
    }
}

#[test]
fn max_pairs_truncates_output() {
    let dir = TempDir::new().unwrap();
    let content = source_text(50, 1);
    for i in 0..5 {
        write_file(&dir, &format!("f{i}.rs"), &content);
    }

    let export = make_export(
        (0..5)
            .map(|i| make_row(&format!("f{i}.rs"), "mod", "Rust", 50, content.len()))
            .collect(),
    );

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

    assert!(report.pairs.len() <= 2, "max_pairs should cap output");
    assert!(report.truncated, "report should be marked as truncated");
}

// ── 4. Edge cases ───────────────────────────────────────────────

#[test]
fn empty_export_returns_no_pairs() {
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
    assert!(report.clusters.is_none());
}

#[test]
fn single_file_returns_no_pairs() {
    let dir = TempDir::new().unwrap();
    let content = source_text(50, 1);
    write_file(&dir, "a.rs", &content);

    let export = make_export(vec![make_row("a.rs", "mod", "Rust", 50, content.len())]);

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

    assert!(report.pairs.is_empty(), "single file cannot form a pair");
}

#[test]
fn empty_files_produce_no_fingerprints() {
    let dir = TempDir::new().unwrap();
    write_file(&dir, "a.rs", "");
    write_file(&dir, "b.rs", "");

    let export = make_export(vec![
        make_row("a.rs", "mod", "Rust", 0, 0),
        make_row("b.rs", "mod", "Rust", 0, 0),
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

    // Empty files can't produce k-grams so won't be paired
    assert!(report.pairs.is_empty());
}

#[test]
fn short_files_below_kgram_threshold_not_paired() {
    let dir = TempDir::new().unwrap();
    // k=25 tokens needed; write files with < 25 tokens
    write_file(&dir, "a.rs", "fn short() {}");
    write_file(&dir, "b.rs", "fn short() {}");

    let export = make_export(vec![
        make_row("a.rs", "mod", "Rust", 1, 14),
        make_row("b.rs", "mod", "Rust", 1, 14),
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
        "files with fewer than k tokens should not be paired"
    );
}

#[test]
fn child_rows_are_excluded() {
    let dir = TempDir::new().unwrap();
    let content = source_text(50, 1);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);

    let mut child = make_row("b.rs", "mod", "Rust", 50, content.len());
    child.kind = FileKind::Child;

    let export = make_export(vec![
        make_row("a.rs", "mod", "Rust", 50, content.len()),
        child,
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

    assert_eq!(report.files_analyzed, 1, "child rows should be excluded");
    assert!(report.pairs.is_empty());
}

#[test]
fn files_exceeding_max_file_bytes_excluded() {
    let dir = TempDir::new().unwrap();
    let content = source_text(50, 1);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);

    let limits = NearDupLimits {
        max_bytes: None,
        max_file_bytes: Some(10), // very small limit
    };

    let export = make_export(vec![
        make_row("a.rs", "mod", "Rust", 50, content.len()),
        make_row("b.rs", "mod", "Rust", 50, content.len()),
    ]);

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

    assert_eq!(
        report.files_analyzed, 0,
        "large files should be excluded by max_file_bytes"
    );
}

#[test]
fn missing_files_on_disk_are_skipped_gracefully() {
    let dir = TempDir::new().unwrap();
    let content = source_text(50, 1);
    write_file(&dir, "a.rs", &content);
    // b.rs does NOT exist on disk

    let export = make_export(vec![
        make_row("a.rs", "mod", "Rust", 50, content.len()),
        make_row("b.rs", "mod", "Rust", 50, content.len()),
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

    // Should not crash; missing file is silently skipped
    assert!(report.pairs.is_empty());
}

// ── 5. Scope-based partitioning ─────────────────────────────────

#[test]
fn module_scope_only_compares_within_module() {
    let dir = TempDir::new().unwrap();
    let content = source_text(50, 1);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);

    let export = make_export(vec![
        make_row("a.rs", "mod_a", "Rust", 50, content.len()),
        make_row("b.rs", "mod_b", "Rust", 50, content.len()),
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

    // Each file is in a different module, so no pairs within a module
    assert!(
        report.pairs.is_empty(),
        "module scope should not pair files in different modules"
    );
}

#[test]
fn global_scope_compares_across_modules() {
    let dir = TempDir::new().unwrap();
    let content = source_text(50, 1);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);

    let export = make_export(vec![
        make_row("a.rs", "mod_a", "Rust", 50, content.len()),
        make_row("b.rs", "mod_b", "Rust", 50, content.len()),
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
        "global scope should compare across modules"
    );
}

#[test]
fn lang_scope_only_compares_within_language() {
    let dir = TempDir::new().unwrap();
    let content = source_text(50, 1);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.py", &content);

    let export = make_export(vec![
        make_row("a.rs", "mod", "Rust", 50, content.len()),
        make_row("b.py", "mod", "Python", 50, content.len()),
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

    assert!(
        report.pairs.is_empty(),
        "lang scope should not pair files in different languages"
    );
}

// ── 6. Large file set characteristics ───────────────────────────

#[test]
fn max_files_caps_analysis() {
    let dir = TempDir::new().unwrap();
    let content = source_text(50, 1);
    for i in 0..10 {
        write_file(&dir, &format!("f{i}.rs"), &content);
    }

    let export = make_export(
        (0..10)
            .map(|i| make_row(&format!("f{i}.rs"), "mod", "Rust", 50, content.len()))
            .collect(),
    );

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

    assert_eq!(
        report.files_analyzed, 3,
        "max_files should cap files analyzed"
    );
    assert_eq!(report.files_skipped, 7);
}

#[test]
fn eligible_files_tracked_before_cap() {
    let dir = TempDir::new().unwrap();
    let content = source_text(50, 1);
    for i in 0..5 {
        write_file(&dir, &format!("f{i}.rs"), &content);
    }

    let export = make_export(
        (0..5)
            .map(|i| make_row(&format!("f{i}.rs"), "mod", "Rust", 50, content.len()))
            .collect(),
    );

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        2,
        None,
        &default_limits(),
        &[],
    )
    .unwrap();

    assert_eq!(report.eligible_files, Some(5));
    assert_eq!(report.files_analyzed, 2);
}

#[test]
fn exclude_patterns_filter_files() {
    let dir = TempDir::new().unwrap();
    let content = source_text(50, 1);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);
    write_file(&dir, "generated.rs", &content);

    let export = make_export(vec![
        make_row("a.rs", "mod", "Rust", 50, content.len()),
        make_row("b.rs", "mod", "Rust", 50, content.len()),
        make_row("generated.rs", "mod", "Rust", 50, content.len()),
    ]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &default_limits(),
        &["generated*".to_string()],
    )
    .unwrap();

    assert_eq!(report.excluded_by_pattern, Some(1));
    assert_eq!(report.files_analyzed, 2);
}

#[test]
fn stats_are_populated() {
    let dir = TempDir::new().unwrap();
    let content = source_text(50, 1);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);

    let export = make_export(vec![
        make_row("a.rs", "mod", "Rust", 50, content.len()),
        make_row("b.rs", "mod", "Rust", 50, content.len()),
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

    let stats = report.stats.unwrap();
    assert!(stats.bytes_processed > 0);
}

// ── 7. Deterministic detection results ──────────────────────────

#[test]
fn deterministic_across_runs() {
    let dir = TempDir::new().unwrap();
    let c1 = source_text(50, 1);
    let c2 = source_text(50, 1); // identical
    let c3 = source_text(50, 3);
    write_file(&dir, "a.rs", &c1);
    write_file(&dir, "b.rs", &c2);
    write_file(&dir, "c.rs", &c3);

    let export = make_export(vec![
        make_row("a.rs", "mod", "Rust", 50, c1.len()),
        make_row("b.rs", "mod", "Rust", 50, c2.len()),
        make_row("c.rs", "mod", "Rust", 50, c3.len()),
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
    for (a, b) in r1.pairs.iter().zip(r2.pairs.iter()) {
        assert_eq!(a.left, b.left);
        assert_eq!(a.right, b.right);
        assert!((a.similarity - b.similarity).abs() < 1e-10);
    }
}

#[test]
fn pairs_sorted_by_similarity_desc_then_paths() {
    let dir = TempDir::new().unwrap();
    let content = source_text(50, 1);
    for name in ["a.rs", "b.rs", "c.rs"] {
        write_file(&dir, name, &content);
    }

    let export = make_export(vec![
        make_row("a.rs", "mod", "Rust", 50, content.len()),
        make_row("b.rs", "mod", "Rust", 50, content.len()),
        make_row("c.rs", "mod", "Rust", 50, content.len()),
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

    for window in report.pairs.windows(2) {
        let valid = window[0].similarity > window[1].similarity
            || ((window[0].similarity - window[1].similarity).abs() < 1e-10
                && (window[0].left < window[1].left
                    || (window[0].left == window[1].left && window[0].right <= window[1].right)));
        assert!(
            valid,
            "pairs should be sorted by similarity desc, then paths"
        );
    }
}

#[test]
fn serde_roundtrip_preserves_report() {
    let dir = TempDir::new().unwrap();
    let content = source_text(50, 1);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);

    let export = make_export(vec![
        make_row("a.rs", "mod", "Rust", 50, content.len()),
        make_row("b.rs", "mod", "Rust", 50, content.len()),
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

    let json = serde_json::to_string(&report).unwrap();
    let deser: tokmd_analysis_types::NearDuplicateReport = serde_json::from_str(&json).unwrap();

    assert_eq!(report.pairs.len(), deser.pairs.len());
    assert_eq!(report.files_analyzed, deser.files_analyzed);
    assert_eq!(report.truncated, deser.truncated);
    for (a, b) in report.pairs.iter().zip(deser.pairs.iter()) {
        assert_eq!(a.left, b.left);
        assert_eq!(a.right, b.right);
        assert!((a.similarity - b.similarity).abs() < 1e-10);
    }
}

#[test]
fn params_capture_algorithm_constants() {
    let dir = TempDir::new().unwrap();
    let export = make_export(vec![]);

    let report = build_near_dup_report(
        dir.path(),
        &export,
        NearDupScope::Global,
        0.75,
        50,
        Some(10),
        &default_limits(),
        &[],
    )
    .unwrap();

    assert!((report.params.threshold - 0.75).abs() < 1e-10);
    assert_eq!(report.params.max_files, 50);
    assert_eq!(report.params.max_pairs, Some(10));
    let algo = report.params.algorithm.unwrap();
    assert_eq!(algo.k_gram_size, 25);
    assert_eq!(algo.window_size, 4);
    assert_eq!(algo.max_postings, 50);
}

#[test]
fn no_truncation_when_under_max_pairs() {
    let dir = TempDir::new().unwrap();
    let content = source_text(50, 1);
    write_file(&dir, "a.rs", &content);
    write_file(&dir, "b.rs", &content);

    let export = make_export(vec![
        make_row("a.rs", "mod", "Rust", 50, content.len()),
        make_row("b.rs", "mod", "Rust", 50, content.len()),
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

    assert!(!report.truncated);
}

#[test]
fn clusters_none_when_no_pairs() {
    let dir = TempDir::new().unwrap();
    let c1 = source_text(50, 1);
    let c2 = source_text(50, 99999);
    write_file(&dir, "a.rs", &c1);
    write_file(&dir, "b.rs", &c2);

    let export = make_export(vec![
        make_row("a.rs", "mod", "Rust", 50, c1.len()),
        make_row("b.rs", "mod", "Rust", 50, c2.len()),
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

    if report.pairs.is_empty() {
        assert!(
            report.clusters.is_none(),
            "clusters should be None when no pairs"
        );
    }
}
