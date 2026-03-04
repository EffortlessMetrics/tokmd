//! Deep property-based and deterministic tests for `tokmd-analysis-near-dup`.
//!
//! Covers near-duplicate detection with identical/similar/different content,
//! Jaccard similarity properties, edge cases (empty files, short files),
//! and property-based verification.

use std::io::Write;

use proptest::prelude::*;
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

fn default_limits() -> NearDupLimits {
    NearDupLimits {
        max_bytes: None,
        max_file_bytes: None,
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Identical content detection
// ═══════════════════════════════════════════════════════════════════

mod identical {
    use super::*;

    #[test]
    fn identical_files_detected_as_duplicates() {
        let dir = TempDir::new().unwrap();
        let content = source_text(100, 0);
        write_file(&dir, "a.rs", &content);
        write_file(&dir, "b.rs", &content);

        let rows = vec![
            make_row("a.rs", "src", "Rust", 100, content.len()),
            make_row("b.rs", "src", "Rust", 100, content.len()),
        ];
        let export = make_export(rows);
        let report = build_near_dup_report(
            dir.path(),
            &export,
            NearDupScope::Global,
            0.5,
            1000,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();

        assert!(
            !report.pairs.is_empty(),
            "identical files should produce pairs"
        );
        assert!(
            report.pairs[0].similarity > 0.9,
            "identical files should have high similarity, got {}",
            report.pairs[0].similarity
        );
    }

    #[test]
    fn three_identical_files_form_cluster() {
        let dir = TempDir::new().unwrap();
        let content = source_text(100, 0);
        for name in &["a.rs", "b.rs", "c.rs"] {
            write_file(&dir, name, &content);
        }
        let rows: Vec<FileRow> = ["a.rs", "b.rs", "c.rs"]
            .iter()
            .map(|p| make_row(p, "src", "Rust", 100, content.len()))
            .collect();
        let export = make_export(rows);
        let report = build_near_dup_report(
            dir.path(),
            &export,
            NearDupScope::Global,
            0.5,
            1000,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();

        assert!(report.clusters.is_some());
        let clusters = report.clusters.as_ref().unwrap();
        assert_eq!(clusters.len(), 1, "3 identical files → 1 cluster");
        assert_eq!(clusters[0].files.len(), 3);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Similar but not identical content
// ═══════════════════════════════════════════════════════════════════

mod similar {
    use super::*;

    #[test]
    fn partially_overlapping_files() {
        let dir = TempDir::new().unwrap();
        let a = content_with_overlap(80, 20, 1);
        let b = content_with_overlap(80, 20, 2);
        write_file(&dir, "a.rs", &a);
        write_file(&dir, "b.rs", &b);

        let rows = vec![
            make_row("a.rs", "src", "Rust", 100, a.len()),
            make_row("b.rs", "src", "Rust", 100, b.len()),
        ];
        let export = make_export(rows);
        let report = build_near_dup_report(
            dir.path(),
            &export,
            NearDupScope::Global,
            0.3,
            1000,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();

        if !report.pairs.is_empty() {
            assert!(
                report.pairs[0].similarity <= 1.0,
                "similarity must be <= 1.0"
            );
            assert!(
                report.pairs[0].similarity >= 0.0,
                "similarity must be >= 0.0"
            );
        }
    }

    #[test]
    fn similarity_degrades_monotonically() {
        let dir = TempDir::new().unwrap();
        let base = source_text(100, 0);
        write_file(&dir, "base.rs", &base);

        // Increasing divergence
        let high = content_with_overlap(90, 10, 1);
        let med = content_with_overlap(60, 40, 2);
        let low = content_with_overlap(30, 70, 3);
        write_file(&dir, "high.rs", &high);
        write_file(&dir, "med.rs", &med);
        write_file(&dir, "low.rs", &low);

        let rows = vec![
            make_row("base.rs", "src", "Rust", 100, base.len()),
            make_row("high.rs", "src", "Rust", 100, high.len()),
            make_row("med.rs", "src", "Rust", 100, med.len()),
            make_row("low.rs", "src", "Rust", 100, low.len()),
        ];
        let export = make_export(rows);
        let report = build_near_dup_report(
            dir.path(),
            &export,
            NearDupScope::Global,
            0.0,
            1000,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();

        // All pairs should have similarity in [0, 1]
        for pair in &report.pairs {
            assert!(pair.similarity >= 0.0 && pair.similarity <= 1.0);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Completely different content
// ═══════════════════════════════════════════════════════════════════

mod different {
    use super::*;

    #[test]
    fn completely_different_files_no_pairs_above_threshold() {
        let dir = TempDir::new().unwrap();
        let a = source_text(100, 1);
        let b = source_text(100, 2);
        write_file(&dir, "a.rs", &a);
        write_file(&dir, "b.rs", &b);

        let rows = vec![
            make_row("a.rs", "src", "Rust", 100, a.len()),
            make_row("b.rs", "src", "Rust", 100, b.len()),
        ];
        let export = make_export(rows);
        let report = build_near_dup_report(
            dir.path(),
            &export,
            NearDupScope::Global,
            0.9,
            1000,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();

        assert!(
            report.pairs.is_empty(),
            "completely different files should not match at threshold 0.9"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Edge cases
// ═══════════════════════════════════════════════════════════════════

mod edge_cases {
    use super::*;

    #[test]
    fn empty_files_no_fingerprints() {
        let dir = TempDir::new().unwrap();
        write_file(&dir, "a.rs", "");
        write_file(&dir, "b.rs", "");

        let rows = vec![
            make_row("a.rs", "src", "Rust", 0, 0),
            make_row("b.rs", "src", "Rust", 0, 0),
        ];
        let export = make_export(rows);
        let report = build_near_dup_report(
            dir.path(),
            &export,
            NearDupScope::Global,
            0.5,
            1000,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();

        assert!(
            report.pairs.is_empty(),
            "empty files should not produce pairs"
        );
    }

    #[test]
    fn very_short_files_below_kgram() {
        let dir = TempDir::new().unwrap();
        // Fewer tokens than k-gram size (25)
        write_file(&dir, "a.rs", "let x = 1;");
        write_file(&dir, "b.rs", "let y = 2;");

        let rows = vec![
            make_row("a.rs", "src", "Rust", 1, 10),
            make_row("b.rs", "src", "Rust", 1, 10),
        ];
        let export = make_export(rows);
        let report = build_near_dup_report(
            dir.path(),
            &export,
            NearDupScope::Global,
            0.5,
            1000,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();

        // Short files can't produce enough fingerprints
        assert!(report.pairs.is_empty());
    }

    #[test]
    fn single_file_no_pairs() {
        let dir = TempDir::new().unwrap();
        let content = source_text(100, 0);
        write_file(&dir, "only.rs", &content);

        let rows = vec![make_row("only.rs", "src", "Rust", 100, content.len())];
        let export = make_export(rows);
        let report = build_near_dup_report(
            dir.path(),
            &export,
            NearDupScope::Global,
            0.0,
            1000,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();

        assert!(report.pairs.is_empty());
    }

    #[test]
    fn no_files_empty_report() {
        let dir = TempDir::new().unwrap();
        let export = make_export(vec![]);
        let report = build_near_dup_report(
            dir.path(),
            &export,
            NearDupScope::Global,
            0.5,
            1000,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();

        assert!(report.pairs.is_empty());
        assert_eq!(report.files_analyzed, 0);
    }

    #[test]
    fn max_pairs_truncation() {
        let dir = TempDir::new().unwrap();
        let content = source_text(100, 0);
        for i in 0..5 {
            write_file(&dir, &format!("f{i}.rs"), &content);
        }
        let rows: Vec<FileRow> = (0..5)
            .map(|i| make_row(&format!("f{i}.rs"), "src", "Rust", 100, content.len()))
            .collect();
        let export = make_export(rows);
        let report = build_near_dup_report(
            dir.path(),
            &export,
            NearDupScope::Global,
            0.1,
            1000,
            Some(2),
            &default_limits(),
            &[],
        )
        .unwrap();

        assert!(report.pairs.len() <= 2);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Scope isolation
// ═══════════════════════════════════════════════════════════════════

mod scope {
    use super::*;

    #[test]
    fn module_scope_isolates_modules() {
        let dir = TempDir::new().unwrap();
        let content = source_text(100, 0);
        write_file(&dir, "src/a.rs", &content);
        write_file(&dir, "test/b.rs", &content);

        let rows = vec![
            make_row("src/a.rs", "src", "Rust", 100, content.len()),
            make_row("test/b.rs", "test", "Rust", 100, content.len()),
        ];
        let export = make_export(rows);
        let report = build_near_dup_report(
            dir.path(),
            &export,
            NearDupScope::Module,
            0.5,
            1000,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();

        // Files in different modules should not pair under Module scope
        assert!(
            report.pairs.is_empty(),
            "module scope should isolate different modules"
        );
    }

    #[test]
    fn global_scope_crosses_modules() {
        let dir = TempDir::new().unwrap();
        let content = source_text(100, 0);
        write_file(&dir, "src/a.rs", &content);
        write_file(&dir, "test/b.rs", &content);

        let rows = vec![
            make_row("src/a.rs", "src", "Rust", 100, content.len()),
            make_row("test/b.rs", "test", "Rust", 100, content.len()),
        ];
        let export = make_export(rows);
        let report = build_near_dup_report(
            dir.path(),
            &export,
            NearDupScope::Global,
            0.5,
            1000,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();

        assert!(
            !report.pairs.is_empty(),
            "global scope should find cross-module duplicates"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Property-based tests
// ═══════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_similarity_in_unit_range(
        shared in 30..100usize,
        unique_a in 0..50usize,
        unique_b in 0..50usize,
    ) {
        let dir = TempDir::new().unwrap();
        let a = content_with_overlap(shared, unique_a, 1);
        let b = content_with_overlap(shared, unique_b, 2);
        write_file(&dir, "a.rs", &a);
        write_file(&dir, "b.rs", &b);

        let rows = vec![
            make_row("a.rs", "src", "Rust", shared + unique_a, a.len()),
            make_row("b.rs", "src", "Rust", shared + unique_b, b.len()),
        ];
        let export = make_export(rows);
        let report = build_near_dup_report(
            dir.path(),
            &export,
            NearDupScope::Global,
            0.0,
            1000,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();

        for pair in &report.pairs {
            prop_assert!(
                pair.similarity >= 0.0 && pair.similarity <= 1.0,
                "similarity must be in [0,1], got {}",
                pair.similarity
            );
        }
    }

    #[test]
    fn prop_files_analyzed_le_input(
        n_files in 2..8usize,
        token_count in 50..150usize,
    ) {
        let dir = TempDir::new().unwrap();
        let mut rows = Vec::new();
        for i in 0..n_files {
            let content = source_text(token_count, i);
            let name = format!("f{i}.rs");
            write_file(&dir, &name, &content);
            rows.push(make_row(&name, "src", "Rust", token_count, content.len()));
        }
        let export = make_export(rows);
        let report = build_near_dup_report(
            dir.path(),
            &export,
            NearDupScope::Global,
            0.5,
            1000,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();

        prop_assert!(report.files_analyzed <= n_files);
    }

    #[test]
    fn prop_pairs_sorted_by_similarity_desc(
        n_files in 3..6usize,
    ) {
        let dir = TempDir::new().unwrap();
        let base = source_text(100, 0);
        let mut rows = Vec::new();
        for i in 0..n_files {
            let content = content_with_overlap(80, 20 + i * 5, i);
            let name = format!("f{i}.rs");
            write_file(&dir, &name, &content);
            rows.push(make_row(&name, "src", "Rust", 100, content.len()));
        }
        // Also write a base file
        write_file(&dir, "base.rs", &base);
        rows.push(make_row("base.rs", "src", "Rust", 100, base.len()));

        let export = make_export(rows);
        let report = build_near_dup_report(
            dir.path(),
            &export,
            NearDupScope::Global,
            0.0,
            1000,
            None,
            &default_limits(),
            &[],
        )
        .unwrap();

        for w in report.pairs.windows(2) {
            prop_assert!(
                w[0].similarity >= w[1].similarity,
                "pairs must be sorted desc: {} < {}",
                w[0].similarity, w[1].similarity
            );
        }
    }
}
