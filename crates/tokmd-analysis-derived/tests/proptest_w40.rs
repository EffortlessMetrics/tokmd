//! Property-based tests for tokmd-analysis-derived.
//!
//! Covers: density bounds, COCOMO monotonicity, distribution invariants,
//! comment ratio bounds, histogram exhaustiveness, and determinism.

use proptest::prelude::*;
use tokmd_analysis_derived::derive_report;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

fn make_row(code: usize, comments: usize, blanks: usize) -> FileRow {
    let lines = code + comments + blanks;
    FileRow {
        path: format!("src/file_{}.rs", code ^ comments),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Parent,
        code,
        comments,
        blanks,
        lines,
        bytes: lines * 40,
        tokens: lines * 4,
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

// =========================================================================
// Density: doc_density ratio is always in [0.0, 1.0]
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn doc_density_between_zero_and_one(
        code in 0usize..5000,
        comments in 0usize..5000,
        blanks in 0usize..1000,
    ) {
        let export = make_export(vec![make_row(code, comments, blanks)]);
        let report = derive_report(&export, None);
        let ratio = report.doc_density.total.ratio;
        prop_assert!((0.0..=1.0).contains(&ratio),
            "doc_density ratio {} out of [0,1] for code={} comments={}", ratio, code, comments);
    }
}

// =========================================================================
// Whitespace ratio: always non-negative, matches blanks/(code+comments)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn whitespace_ratio_non_negative(
        code in 0usize..5000,
        comments in 0usize..5000,
        blanks in 0usize..5000,
    ) {
        let export = make_export(vec![make_row(code, comments, blanks)]);
        let report = derive_report(&export, None);
        let ratio = report.whitespace.total.ratio;
        prop_assert!(ratio >= 0.0,
            "whitespace ratio {} must be non-negative", ratio);
        let denom = code + comments;
        if denom == 0 {
            prop_assert_eq!(ratio, 0.0, "ratio should be 0 when denom is 0");
        }
    }
}

// =========================================================================
// COCOMO: effort is monotonically increasing with code lines
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn cocomo_effort_monotonic(
        small_code in 100usize..1000,
        extra in 500usize..5000,
    ) {
        let large_code = small_code + extra;
        let small_export = make_export(vec![make_row(small_code, 10, 5)]);
        let large_export = make_export(vec![make_row(large_code, 10, 5)]);
        let small_report = derive_report(&small_export, None);
        let large_report = derive_report(&large_export, None);

        let small_effort = small_report.cocomo.as_ref().unwrap().effort_pm;
        let large_effort = large_report.cocomo.as_ref().unwrap().effort_pm;
        prop_assert!(large_effort >= small_effort,
            "COCOMO effort should be monotonic: {} (code={}) vs {} (code={})",
            small_effort, small_code, large_effort, large_code);
    }

    #[test]
    fn cocomo_zero_code_returns_none(
        comments in 0usize..500,
        blanks in 0usize..200,
    ) {
        let export = make_export(vec![make_row(0, comments, blanks)]);
        let report = derive_report(&export, None);
        prop_assert!(report.cocomo.is_none(),
            "COCOMO should be None for zero code lines");
    }
}

// =========================================================================
// Distribution: histogram bucket percentages sum to ~1.0
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn histogram_pct_sum_to_one(
        sizes in prop::collection::vec(1usize..2000, 1..20),
    ) {
        let rows: Vec<FileRow> = sizes.iter().enumerate().map(|(i, &s)| {
            FileRow {
                path: format!("src/file_{}.rs", i),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: s / 2,
                comments: s / 4,
                blanks: s - s / 2 - s / 4,
                lines: s,
                bytes: s * 40,
                tokens: s * 4,
            }
        }).collect();

        let export = make_export(rows);
        let report = derive_report(&export, None);
        let pct_sum: f64 = report.histogram.iter().map(|b| b.pct).sum();
        prop_assert!((pct_sum - 1.0).abs() < 0.01,
            "Histogram percentages sum {} should be ~1.0", pct_sum);
    }

    #[test]
    fn histogram_files_sum_to_count(
        sizes in prop::collection::vec(1usize..2000, 1..20),
    ) {
        let rows: Vec<FileRow> = sizes.iter().enumerate().map(|(i, &s)| {
            FileRow {
                path: format!("src/file_{}.rs", i),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: s / 2,
                comments: s / 4,
                blanks: s - s / 2 - s / 4,
                lines: s,
                bytes: s * 40,
                tokens: s * 4,
            }
        }).collect();

        let n = rows.len();
        let export = make_export(rows);
        let report = derive_report(&export, None);
        let file_sum: usize = report.histogram.iter().map(|b| b.files).sum();
        prop_assert_eq!(file_sum, n,
            "Histogram file counts should sum to total file count");
    }
}

// =========================================================================
// Distribution statistics: min <= median <= max, gini in [0, 1]
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn distribution_min_leq_median_leq_max(
        sizes in prop::collection::vec(1usize..5000, 1..30),
    ) {
        let rows: Vec<FileRow> = sizes.iter().enumerate().map(|(i, &s)| {
            FileRow {
                path: format!("src/file_{}.rs", i),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: s,
                comments: 0,
                blanks: 0,
                lines: s,
                bytes: s * 30,
                tokens: s * 3,
            }
        }).collect();

        let export = make_export(rows);
        let report = derive_report(&export, None);
        let dist = &report.distribution;
        prop_assert!(dist.min as f64 <= dist.median,
            "min {} > median {}", dist.min, dist.median);
        prop_assert!(dist.median <= dist.max as f64,
            "median {} > max {}", dist.median, dist.max);
    }

    #[test]
    fn distribution_gini_in_unit_range(
        sizes in prop::collection::vec(1usize..5000, 2..30),
    ) {
        let rows: Vec<FileRow> = sizes.iter().enumerate().map(|(i, &s)| {
            FileRow {
                path: format!("src/file_{}.rs", i),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: s,
                comments: 0,
                blanks: 0,
                lines: s,
                bytes: s * 30,
                tokens: s * 3,
            }
        }).collect();

        let export = make_export(rows);
        let report = derive_report(&export, None);
        let gini = report.distribution.gini;
        prop_assert!((0.0..=1.0).contains(&gini),
            "Gini {} out of [0,1]", gini);
    }
}

// =========================================================================
// Determinism: same input always produces same output
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn derive_report_is_deterministic(
        code in 10usize..2000,
        comments in 0usize..500,
        blanks in 0usize..200,
    ) {
        let export = make_export(vec![make_row(code, comments, blanks)]);
        let a = derive_report(&export, Some(128_000));
        let b = derive_report(&export, Some(128_000));

        let json_a = serde_json::to_string(&a).unwrap();
        let json_b = serde_json::to_string(&b).unwrap();
        prop_assert_eq!(json_a, json_b, "derive_report must be deterministic");
    }
}
