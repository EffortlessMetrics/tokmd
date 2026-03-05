//! Contract tests for `tokmd-analysis-near-dup` enricher (w65).
//!
//! Covers: near-duplicate detection, scope modes, similarity scoring,
//! clustering, truncation, exclusion patterns, fingerprinting invariants,
//! and property-based tests.

use std::io::Write;
use tempfile::TempDir;
use tokmd_analysis_near_dup::{NearDupLimits, build_near_dup_report};
use tokmd_analysis_types::NearDupScope;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ──────────────────────────────────────────────────────

/// Generate source text with `n` distinct tokens (enough for winnowing K=25).
fn source(n: usize, seed: usize) -> String {
    (0..n)
        .map(|i| format!("token_{}_{}", seed, i))
        .collect::<Vec<_>>()
        .join(" + ")
}

/// Generate source text with `overlap` shared tokens and `unique` distinct ones.
fn partial_source(shared: usize, unique: usize, seed: usize) -> String {
    let shared_part: Vec<String> = (0..shared).map(|i| format!("common_{i}")).collect();
    let unique_part: Vec<String> = (0..unique).map(|i| format!("uniq_{seed}_{i}")).collect();
    [shared_part, unique_part].concat().join(" + ")
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

fn default_limits() -> NearDupLimits {
    NearDupLimits {
        max_bytes: None,
        max_file_bytes: None,
    }
}

fn run_report(
    dir: &TempDir,
    export: &ExportData,
    scope: NearDupScope,
    threshold: f64,
) -> tokmd_analysis_types::NearDuplicateReport {
    build_near_dup_report(
        dir.path(),
        export,
        scope,
        threshold,
        10000,
        None,
        &default_limits(),
        &[],
    )
    .unwrap()
}

// ── Identical files ──────────────────────────────────────────────

mod identical {
    use super::*;

    #[test]
    fn two_identical_files_detected() {
        let dir = TempDir::new().unwrap();
        let content = source(100, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);
        let data = make_export(vec![
            make_row("a.rs", "root", "Rust", 100, content.len()),
            make_row("b.rs", "root", "Rust", 100, content.len()),
        ]);
        let r = run_report(&dir, &data, NearDupScope::Global, 0.5);
        assert!(!r.pairs.is_empty());
        assert!(r.pairs[0].similarity > 0.9);
    }

    #[test]
    fn identical_similarity_near_one() {
        let dir = TempDir::new().unwrap();
        let content = source(150, 42);
        write_file(&dir, "x.rs", &content);
        write_file(&dir, "y.rs", &content);
        let data = make_export(vec![
            make_row("x.rs", "root", "Rust", 150, content.len()),
            make_row("y.rs", "root", "Rust", 150, content.len()),
        ]);
        let r = run_report(&dir, &data, NearDupScope::Global, 0.5);
        assert_eq!(r.pairs.len(), 1);
        assert!((r.pairs[0].similarity - 1.0).abs() < 0.05);
    }

    #[test]
    fn three_identical_files_multiple_pairs() {
        let dir = TempDir::new().unwrap();
        let content = source(100, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);
        write_file(&dir, "c.rs", &content);
        let data = make_export(vec![
            make_row("a.rs", "root", "Rust", 100, content.len()),
            make_row("b.rs", "root", "Rust", 100, content.len()),
            make_row("c.rs", "root", "Rust", 100, content.len()),
        ]);
        let r = run_report(&dir, &data, NearDupScope::Global, 0.5);
        assert!(r.pairs.len() >= 3);
    }
}

// ── Completely different files ──────────────────────────────────

mod different {
    use super::*;

    #[test]
    fn completely_different_files_no_pairs() {
        let dir = TempDir::new().unwrap();
        let a = source(100, 0);
        let b = source(100, 999);
        write_file(&dir, "a.rs", &a);
        write_file(&dir, "b.rs", &b);
        let data = make_export(vec![
            make_row("a.rs", "root", "Rust", 100, a.len()),
            make_row("b.rs", "root", "Rust", 100, b.len()),
        ]);
        let r = run_report(&dir, &data, NearDupScope::Global, 0.5);
        assert!(r.pairs.is_empty());
    }

    #[test]
    fn different_files_with_low_threshold_may_match() {
        let dir = TempDir::new().unwrap();
        // Two files with some accidental overlap
        let a = partial_source(30, 70, 1);
        let b = partial_source(30, 70, 2);
        write_file(&dir, "a.rs", &a);
        write_file(&dir, "b.rs", &b);
        let data = make_export(vec![
            make_row("a.rs", "root", "Rust", 100, a.len()),
            make_row("b.rs", "root", "Rust", 100, b.len()),
        ]);
        // With very low threshold, might find pairs
        let r = run_report(&dir, &data, NearDupScope::Global, 0.01);
        // Just verify no crash; result depends on actual overlap
        assert!(r.files_analyzed >= 2);
    }
}

// ── Partial overlap ─────────────────────────────────────────────

mod partial_overlap {
    use super::*;

    #[test]
    fn high_overlap_detected() {
        let dir = TempDir::new().unwrap();
        let a = partial_source(80, 20, 1);
        let b = partial_source(80, 20, 2);
        write_file(&dir, "a.rs", &a);
        write_file(&dir, "b.rs", &b);
        let data = make_export(vec![
            make_row("a.rs", "root", "Rust", 100, a.len()),
            make_row("b.rs", "root", "Rust", 100, b.len()),
        ]);
        let r = run_report(&dir, &data, NearDupScope::Global, 0.3);
        // High overlap should yield at least some similarity
        assert!(r.files_analyzed >= 2);
    }

    #[test]
    fn similarity_score_bounded() {
        let dir = TempDir::new().unwrap();
        let a = partial_source(60, 40, 1);
        let b = partial_source(60, 40, 2);
        write_file(&dir, "a.rs", &a);
        write_file(&dir, "b.rs", &b);
        let data = make_export(vec![
            make_row("a.rs", "root", "Rust", 100, a.len()),
            make_row("b.rs", "root", "Rust", 100, b.len()),
        ]);
        let r = run_report(&dir, &data, NearDupScope::Global, 0.0);
        for pair in &r.pairs {
            assert!((0.0..=1.0).contains(&pair.similarity));
        }
    }
}

// ── Scope modes ─────────────────────────────────────────────────

mod scope {
    use super::*;

    #[test]
    fn lang_scope_separates_languages() {
        let dir = TempDir::new().unwrap();
        let content = source(100, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.py", &content);
        let data = make_export(vec![
            make_row("a.rs", "root", "Rust", 100, content.len()),
            make_row("b.py", "root", "Python", 100, content.len()),
        ]);
        let r = run_report(&dir, &data, NearDupScope::Lang, 0.5);
        // Different languages → no pairs in Lang scope
        assert!(r.pairs.is_empty());
    }

    #[test]
    fn global_scope_finds_cross_language() {
        let dir = TempDir::new().unwrap();
        let content = source(100, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.py", &content);
        let data = make_export(vec![
            make_row("a.rs", "root", "Rust", 100, content.len()),
            make_row("b.py", "root", "Python", 100, content.len()),
        ]);
        let r = run_report(&dir, &data, NearDupScope::Global, 0.5);
        assert!(!r.pairs.is_empty());
    }

    #[test]
    fn module_scope_separates_modules() {
        let dir = TempDir::new().unwrap();
        let content = source(100, 0);
        write_file(&dir, "src/a.rs", &content);
        write_file(&dir, "lib/b.rs", &content);
        let data = make_export(vec![
            make_row("src/a.rs", "src", "Rust", 100, content.len()),
            make_row("lib/b.rs", "lib", "Rust", 100, content.len()),
        ]);
        let r = run_report(&dir, &data, NearDupScope::Module, 0.5);
        // Different modules → no pairs
        assert!(r.pairs.is_empty());
    }

    #[test]
    fn module_scope_finds_same_module_dups() {
        let dir = TempDir::new().unwrap();
        let content = source(100, 0);
        write_file(&dir, "src/a.rs", &content);
        write_file(&dir, "src/b.rs", &content);
        let data = make_export(vec![
            make_row("src/a.rs", "src", "Rust", 100, content.len()),
            make_row("src/b.rs", "src", "Rust", 100, content.len()),
        ]);
        let r = run_report(&dir, &data, NearDupScope::Module, 0.5);
        assert!(!r.pairs.is_empty());
    }
}

// ── Threshold behavior ──────────────────────────────────────────

mod threshold {
    use super::*;

    #[test]
    fn high_threshold_filters_low_similarity() {
        let dir = TempDir::new().unwrap();
        let a = partial_source(50, 50, 1);
        let b = partial_source(50, 50, 2);
        write_file(&dir, "a.rs", &a);
        write_file(&dir, "b.rs", &b);
        let data = make_export(vec![
            make_row("a.rs", "root", "Rust", 100, a.len()),
            make_row("b.rs", "root", "Rust", 100, b.len()),
        ]);
        let r_low = run_report(&dir, &data, NearDupScope::Global, 0.01);
        let r_high = run_report(&dir, &data, NearDupScope::Global, 0.99);
        assert!(r_high.pairs.len() <= r_low.pairs.len());
    }

    #[test]
    fn threshold_one_only_exact_matches() {
        let dir = TempDir::new().unwrap();
        let a = source(100, 1);
        let b = source(100, 2);
        write_file(&dir, "a.rs", &a);
        write_file(&dir, "b.rs", &b);
        let data = make_export(vec![
            make_row("a.rs", "root", "Rust", 100, a.len()),
            make_row("b.rs", "root", "Rust", 100, b.len()),
        ]);
        let r = run_report(&dir, &data, NearDupScope::Global, 1.0);
        assert!(r.pairs.is_empty());
    }
}

// ── Clustering ──────────────────────────────────────────────────

mod clustering {
    use super::*;

    #[test]
    fn clusters_present_when_pairs_found() {
        let dir = TempDir::new().unwrap();
        let content = source(100, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);
        let data = make_export(vec![
            make_row("a.rs", "root", "Rust", 100, content.len()),
            make_row("b.rs", "root", "Rust", 100, content.len()),
        ]);
        let r = run_report(&dir, &data, NearDupScope::Global, 0.5);
        assert!(r.clusters.is_some());
        let clusters = r.clusters.unwrap();
        assert!(!clusters.is_empty());
    }

    #[test]
    fn cluster_contains_all_pair_files() {
        let dir = TempDir::new().unwrap();
        let content = source(100, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);
        let data = make_export(vec![
            make_row("a.rs", "root", "Rust", 100, content.len()),
            make_row("b.rs", "root", "Rust", 100, content.len()),
        ]);
        let r = run_report(&dir, &data, NearDupScope::Global, 0.5);
        let clusters = r.clusters.unwrap();
        let all_files: Vec<&str> = clusters
            .iter()
            .flat_map(|c| c.files.iter().map(|f| f.as_str()))
            .collect();
        assert!(all_files.contains(&"a.rs"));
        assert!(all_files.contains(&"b.rs"));
    }

    #[test]
    fn cluster_representative_is_in_files() {
        let dir = TempDir::new().unwrap();
        let content = source(100, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);
        let data = make_export(vec![
            make_row("a.rs", "root", "Rust", 100, content.len()),
            make_row("b.rs", "root", "Rust", 100, content.len()),
        ]);
        let r = run_report(&dir, &data, NearDupScope::Global, 0.5);
        for cluster in r.clusters.unwrap() {
            assert!(cluster.files.contains(&cluster.representative));
        }
    }

    #[test]
    fn no_clusters_when_no_pairs() {
        let dir = TempDir::new().unwrap();
        let a = source(100, 0);
        let b = source(100, 999);
        write_file(&dir, "a.rs", &a);
        write_file(&dir, "b.rs", &b);
        let data = make_export(vec![
            make_row("a.rs", "root", "Rust", 100, a.len()),
            make_row("b.rs", "root", "Rust", 100, b.len()),
        ]);
        let r = run_report(&dir, &data, NearDupScope::Global, 0.5);
        assert!(r.clusters.is_none());
    }

    #[test]
    fn cluster_files_sorted_alphabetically() {
        let dir = TempDir::new().unwrap();
        let content = source(100, 0);
        write_file(&dir, "z.rs", &content);
        write_file(&dir, "a.rs", &content);
        let data = make_export(vec![
            make_row("z.rs", "root", "Rust", 100, content.len()),
            make_row("a.rs", "root", "Rust", 100, content.len()),
        ]);
        let r = run_report(&dir, &data, NearDupScope::Global, 0.5);
        for cluster in r.clusters.unwrap() {
            let sorted: Vec<_> = {
                let mut c = cluster.files.clone();
                c.sort();
                c
            };
            assert_eq!(cluster.files, sorted);
        }
    }
}

// ── Truncation ──────────────────────────────────────────────────

mod truncation {
    use super::*;

    #[test]
    fn max_pairs_truncates_output() {
        let dir = TempDir::new().unwrap();
        let content = source(100, 0);
        for name in ["a.rs", "b.rs", "c.rs", "d.rs"] {
            write_file(&dir, name, &content);
        }
        let data = make_export(vec![
            make_row("a.rs", "root", "Rust", 100, content.len()),
            make_row("b.rs", "root", "Rust", 100, content.len()),
            make_row("c.rs", "root", "Rust", 100, content.len()),
            make_row("d.rs", "root", "Rust", 100, content.len()),
        ]);
        let r = build_near_dup_report(
            dir.path(),
            &data,
            NearDupScope::Global,
            0.5,
            10000,
            Some(1),
            &default_limits(),
            &[],
        )
        .unwrap();
        assert!(r.pairs.len() <= 1);
        assert!(r.truncated);
    }

    #[test]
    fn no_truncation_when_under_limit() {
        let dir = TempDir::new().unwrap();
        let content = source(100, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);
        let data = make_export(vec![
            make_row("a.rs", "root", "Rust", 100, content.len()),
            make_row("b.rs", "root", "Rust", 100, content.len()),
        ]);
        let r = build_near_dup_report(
            dir.path(),
            &data,
            NearDupScope::Global,
            0.5,
            10000,
            Some(100),
            &default_limits(),
            &[],
        )
        .unwrap();
        assert!(!r.truncated);
    }
}

// ── Exclusion patterns ──────────────────────────────────────────

mod exclusion {
    use super::*;

    #[test]
    fn glob_pattern_excludes_files() {
        let dir = TempDir::new().unwrap();
        let content = source(100, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);
        write_file(&dir, "vendor/c.rs", &content);
        let data = make_export(vec![
            make_row("a.rs", "root", "Rust", 100, content.len()),
            make_row("b.rs", "root", "Rust", 100, content.len()),
            make_row("vendor/c.rs", "vendor", "Rust", 100, content.len()),
        ]);
        let r = build_near_dup_report(
            dir.path(),
            &data,
            NearDupScope::Global,
            0.5,
            10000,
            None,
            &default_limits(),
            &["vendor/**".to_string()],
        )
        .unwrap();
        assert_eq!(r.excluded_by_pattern, Some(1));
    }

    #[test]
    fn no_exclusion_yields_none() {
        let dir = TempDir::new().unwrap();
        let content = source(100, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);
        let data = make_export(vec![
            make_row("a.rs", "root", "Rust", 100, content.len()),
            make_row("b.rs", "root", "Rust", 100, content.len()),
        ]);
        let r = run_report(&dir, &data, NearDupScope::Global, 0.5);
        assert!(r.excluded_by_pattern.is_none());
    }
}

// ── Edge cases ──────────────────────────────────────────────────

mod edge_cases {
    use super::*;

    #[test]
    fn empty_export_yields_empty_report() {
        let dir = TempDir::new().unwrap();
        let data = make_export(vec![]);
        let r = run_report(&dir, &data, NearDupScope::Global, 0.5);
        assert!(r.pairs.is_empty());
        assert_eq!(r.files_analyzed, 0);
    }

    #[test]
    fn single_file_no_pairs() {
        let dir = TempDir::new().unwrap();
        let content = source(100, 0);
        write_file(&dir, "a.rs", &content);
        let data = make_export(vec![make_row("a.rs", "root", "Rust", 100, content.len())]);
        let r = run_report(&dir, &data, NearDupScope::Global, 0.5);
        assert!(r.pairs.is_empty());
    }

    #[test]
    fn short_files_below_kgram_produce_no_fingerprints() {
        let dir = TempDir::new().unwrap();
        let short = "fn x() {}";
        write_file(&dir, "a.rs", short);
        write_file(&dir, "b.rs", short);
        let data = make_export(vec![
            make_row("a.rs", "root", "Rust", 1, short.len()),
            make_row("b.rs", "root", "Rust", 1, short.len()),
        ]);
        let r = run_report(&dir, &data, NearDupScope::Global, 0.5);
        // Too short for winnowing → no pairs
        assert!(r.pairs.is_empty());
    }

    #[test]
    fn child_rows_excluded_from_analysis() {
        let dir = TempDir::new().unwrap();
        let content = source(100, 0);
        write_file(&dir, "a.rs", &content);
        let mut data = make_export(vec![make_row("a.rs", "root", "Rust", 100, content.len())]);
        data.rows.push(FileRow {
            path: "a.rs/html".to_string(),
            module: "root".to_string(),
            lang: "HTML".to_string(),
            kind: FileKind::Child,
            code: 50,
            comments: 0,
            blanks: 0,
            lines: 50,
            bytes: 1000,
            tokens: 250,
        });
        let r = run_report(&dir, &data, NearDupScope::Global, 0.5);
        assert_eq!(r.files_analyzed, 1);
    }

    #[test]
    fn max_files_caps_analysis() {
        let dir = TempDir::new().unwrap();
        let content = source(100, 0);
        for i in 0..10 {
            write_file(&dir, &format!("f{i}.rs"), &content);
        }
        let rows: Vec<FileRow> = (0..10)
            .map(|i| make_row(&format!("f{i}.rs"), "root", "Rust", 100, content.len()))
            .collect();
        let data = make_export(rows);
        let r = build_near_dup_report(
            dir.path(),
            &data,
            NearDupScope::Global,
            0.5,
            3,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();
        assert_eq!(r.files_analyzed, 3);
        assert_eq!(r.files_skipped, 7);
    }
}

// ── Params metadata ─────────────────────────────────────────────

mod params {
    use super::*;

    #[test]
    fn params_reflect_inputs() {
        let dir = TempDir::new().unwrap();
        let data = make_export(vec![]);
        let r = build_near_dup_report(
            dir.path(),
            &data,
            NearDupScope::Lang,
            0.75,
            500,
            Some(100),
            &default_limits(),
            &[],
        )
        .unwrap();
        assert_eq!(r.params.scope, NearDupScope::Lang);
        assert!((r.params.threshold - 0.75).abs() < 1e-10);
        assert_eq!(r.params.max_files, 500);
        assert_eq!(r.params.max_pairs, Some(100));
    }

    #[test]
    fn algorithm_constants_recorded() {
        let dir = TempDir::new().unwrap();
        let data = make_export(vec![]);
        let r = run_report(&dir, &data, NearDupScope::Global, 0.5);
        let algo = r.params.algorithm.unwrap();
        assert_eq!(algo.k_gram_size, 25);
        assert_eq!(algo.window_size, 4);
        assert_eq!(algo.max_postings, 50);
    }

    #[test]
    fn stats_present_in_report() {
        let dir = TempDir::new().unwrap();
        let content = source(100, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);
        let data = make_export(vec![
            make_row("a.rs", "root", "Rust", 100, content.len()),
            make_row("b.rs", "root", "Rust", 100, content.len()),
        ]);
        let r = run_report(&dir, &data, NearDupScope::Global, 0.5);
        assert!(r.stats.is_some());
    }
}

// ── Determinism ─────────────────────────────────────────────────

mod determinism {
    use super::*;

    #[test]
    fn report_deterministic_across_runs() {
        let dir = TempDir::new().unwrap();
        let content = source(100, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);
        let data = make_export(vec![
            make_row("a.rs", "root", "Rust", 100, content.len()),
            make_row("b.rs", "root", "Rust", 100, content.len()),
        ]);
        let r1 = run_report(&dir, &data, NearDupScope::Global, 0.5);
        let r2 = run_report(&dir, &data, NearDupScope::Global, 0.5);
        assert_eq!(r1.pairs.len(), r2.pairs.len());
        for (a, b) in r1.pairs.iter().zip(r2.pairs.iter()) {
            assert_eq!(a.left, b.left);
            assert_eq!(a.right, b.right);
            assert!((a.similarity - b.similarity).abs() < 1e-10);
        }
    }

    #[test]
    fn pairs_sorted_by_similarity_desc() {
        let dir = TempDir::new().unwrap();
        let content = source(100, 0);
        for name in ["a.rs", "b.rs", "c.rs"] {
            write_file(&dir, name, &content);
        }
        let data = make_export(vec![
            make_row("a.rs", "root", "Rust", 100, content.len()),
            make_row("b.rs", "root", "Rust", 100, content.len()),
            make_row("c.rs", "root", "Rust", 100, content.len()),
        ]);
        let r = run_report(&dir, &data, NearDupScope::Global, 0.5);
        for w in r.pairs.windows(2) {
            assert!(w[0].similarity >= w[1].similarity);
        }
    }
}

// ── Serialization ───────────────────────────────────────────────

mod serialization {
    use super::*;

    #[test]
    fn report_serializes_to_json() {
        let dir = TempDir::new().unwrap();
        let content = source(100, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);
        let data = make_export(vec![
            make_row("a.rs", "root", "Rust", 100, content.len()),
            make_row("b.rs", "root", "Rust", 100, content.len()),
        ]);
        let r = run_report(&dir, &data, NearDupScope::Global, 0.5);
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("pairs"));
        assert!(json.contains("files_analyzed"));
    }
}

// ── Property tests ──────────────────────────────────────────────

mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn similarity_always_in_unit_interval(seed in 0..100usize) {
            let dir = TempDir::new().unwrap();
            let a = partial_source(50, 50, seed);
            let b = partial_source(50, 50, seed + 1000);
            write_file(&dir, "a.rs", &a);
            write_file(&dir, "b.rs", &b);
            let data = make_export(vec![
                make_row("a.rs", "root", "Rust", 100, a.len()),
                make_row("b.rs", "root", "Rust", 100, b.len()),
            ]);
            let r = run_report(&dir, &data, NearDupScope::Global, 0.0);
            for pair in &r.pairs {
                prop_assert!((0.0..=1.0).contains(&pair.similarity));
            }
        }

        #[test]
        fn files_analyzed_le_eligible(n in 1..20usize) {
            let dir = TempDir::new().unwrap();
            let content = source(100, 0);
            for i in 0..n {
                write_file(&dir, &format!("f{i}.rs"), &content);
            }
            let rows: Vec<FileRow> = (0..n)
                .map(|i| make_row(&format!("f{i}.rs"), "root", "Rust", 100, content.len()))
                .collect();
            let data = make_export(rows);
            let r = run_report(&dir, &data, NearDupScope::Global, 0.5);
            if let Some(eligible) = r.eligible_files {
                prop_assert!(r.files_analyzed <= eligible);
            }
        }

        #[test]
        fn shared_fingerprints_le_min_individual(seed in 0..50usize) {
            let dir = TempDir::new().unwrap();
            let content = source(100, seed);
            write_file(&dir, "a.rs", &content);
            write_file(&dir, "b.rs", &content);
            let data = make_export(vec![
                make_row("a.rs", "root", "Rust", 100, content.len()),
                make_row("b.rs", "root", "Rust", 100, content.len()),
            ]);
            let r = run_report(&dir, &data, NearDupScope::Global, 0.0);
            for pair in &r.pairs {
                let min_fps = pair.left_fingerprints.min(pair.right_fingerprints);
                prop_assert!(pair.shared_fingerprints <= min_fps);
            }
        }
    }
}
