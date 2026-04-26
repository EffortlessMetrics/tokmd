//! W68 deep tests for derived metrics computation.
//!
//! Covers code density, comment ratio, COCOMO estimation, distribution metrics,
//! context window, and property-based invariants.

use crate::derived::derive_report;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn empty_export() -> ExportData {
    ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    }
}

fn single_file_export(code: usize, comments: usize, blanks: usize) -> ExportData {
    let lines = code + comments + blanks;
    ExportData {
        rows: vec![FileRow {
            path: "src/lib.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code,
            comments,
            blanks,
            lines,
            bytes: lines * 25,
            tokens: code * 8,
        }],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    }
}

fn multi_file_export() -> ExportData {
    ExportData {
        rows: vec![
            FileRow {
                path: "src/main.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 200,
                comments: 50,
                blanks: 30,
                lines: 280,
                bytes: 7_000,
                tokens: 1_600,
            },
            FileRow {
                path: "src/lib.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 100,
                comments: 20,
                blanks: 10,
                lines: 130,
                bytes: 3_250,
                tokens: 800,
            },
            FileRow {
                path: "src/util.py".to_string(),
                module: "src".to_string(),
                lang: "Python".to_string(),
                kind: FileKind::Parent,
                code: 50,
                comments: 5,
                blanks: 5,
                lines: 60,
                bytes: 1_500,
                tokens: 400,
            },
        ],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    }
}

// ---------------------------------------------------------------------------
// Empty input
// ---------------------------------------------------------------------------

#[test]
fn derive_empty_totals_all_zero() {
    let report = derive_report(&empty_export(), None);
    assert_eq!(report.totals.files, 0);
    assert_eq!(report.totals.code, 0);
    assert_eq!(report.totals.comments, 0);
    assert_eq!(report.totals.blanks, 0);
    assert_eq!(report.totals.lines, 0);
    assert_eq!(report.totals.bytes, 0);
    assert_eq!(report.totals.tokens, 0);
}

#[test]
fn derive_empty_cocomo_is_none() {
    let report = derive_report(&empty_export(), None);
    assert!(report.cocomo.is_none());
}

#[test]
fn derive_empty_distribution_zeros() {
    let report = derive_report(&empty_export(), None);
    assert_eq!(report.distribution.count, 0);
    assert_eq!(report.distribution.min, 0);
    assert_eq!(report.distribution.max, 0);
    assert_eq!(report.distribution.mean, 0.0);
    assert_eq!(report.distribution.gini, 0.0);
}

#[test]
fn derive_empty_doc_density_zero() {
    let report = derive_report(&empty_export(), None);
    assert_eq!(report.doc_density.total.ratio, 0.0);
}

// ---------------------------------------------------------------------------
// Code density (doc_density is comments / (code + comments))
// ---------------------------------------------------------------------------

#[test]
fn doc_density_single_file() {
    let export = single_file_export(100, 20, 10);
    let report = derive_report(&export, None);
    // 20 / (100 + 20) = 0.1667
    let ratio = report.doc_density.total.ratio;
    assert!((ratio - 0.1667).abs() < 0.001, "ratio={ratio}");
}

#[test]
fn doc_density_all_comments() {
    let export = single_file_export(0, 50, 0);
    let report = derive_report(&export, None);
    // 50 / (0 + 50) = 1.0
    assert_eq!(report.doc_density.total.ratio, 1.0);
}

#[test]
fn doc_density_no_comments() {
    let export = single_file_export(100, 0, 10);
    let report = derive_report(&export, None);
    assert_eq!(report.doc_density.total.ratio, 0.0);
}

#[test]
fn doc_density_zero_code_and_comments() {
    let export = single_file_export(0, 0, 10);
    let report = derive_report(&export, None);
    assert_eq!(report.doc_density.total.ratio, 0.0);
}

// ---------------------------------------------------------------------------
// Whitespace ratio (blanks / (code + comments))
// ---------------------------------------------------------------------------

#[test]
fn whitespace_ratio_single_file() {
    let export = single_file_export(100, 20, 30);
    let report = derive_report(&export, None);
    // 30 / (100 + 20) = 0.25
    assert_eq!(report.whitespace.total.ratio, 0.25);
}

// ---------------------------------------------------------------------------
// COCOMO estimation
// ---------------------------------------------------------------------------

#[test]
fn cocomo_present_for_nonzero_code() {
    let export = single_file_export(1000, 100, 50);
    let report = derive_report(&export, None);
    let cocomo = report.cocomo.expect("COCOMO should be present");
    assert_eq!(cocomo.mode, "organic");
    assert_eq!(cocomo.kloc, 1.0);
    assert!(cocomo.effort_pm > 0.0);
    assert!(cocomo.duration_months > 0.0);
    assert!(cocomo.staff > 0.0);
}

#[test]
fn cocomo_kloc_calculation() {
    let export = single_file_export(5000, 100, 50);
    let report = derive_report(&export, None);
    let cocomo = report.cocomo.unwrap();
    assert_eq!(cocomo.kloc, 5.0);
}

#[test]
fn cocomo_coefficients() {
    let export = single_file_export(1000, 0, 0);
    let report = derive_report(&export, None);
    let cocomo = report.cocomo.unwrap();
    assert_eq!(cocomo.a, 2.4);
    assert_eq!(cocomo.b, 1.05);
    assert_eq!(cocomo.c, 2.5);
    assert_eq!(cocomo.d, 0.38);
}

#[test]
fn cocomo_effort_scales_with_kloc() {
    let small = derive_report(&single_file_export(1000, 0, 0), None);
    let large = derive_report(&single_file_export(10000, 0, 0), None);
    assert!(
        large.cocomo.unwrap().effort_pm > small.cocomo.unwrap().effort_pm,
        "Larger codebase should require more effort"
    );
}

// ---------------------------------------------------------------------------
// Distribution metrics
// ---------------------------------------------------------------------------

#[test]
fn distribution_single_file() {
    let export = single_file_export(100, 20, 10);
    let report = derive_report(&export, None);
    assert_eq!(report.distribution.count, 1);
    assert_eq!(report.distribution.min, 130);
    assert_eq!(report.distribution.max, 130);
    assert_eq!(report.distribution.median, 130.0);
}

#[test]
fn distribution_multi_file() {
    let report = derive_report(&multi_file_export(), None);
    assert_eq!(report.distribution.count, 3);
    assert_eq!(report.distribution.min, 60);
    assert_eq!(report.distribution.max, 280);
    assert_eq!(report.distribution.median, 130.0);
}

#[test]
fn distribution_gini_uniform_is_low() {
    // Two files with same line count -> gini should be 0
    let export = ExportData {
        rows: vec![
            FileRow {
                path: "a.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 50,
                comments: 5,
                blanks: 5,
                lines: 60,
                bytes: 1500,
                tokens: 400,
            },
            FileRow {
                path: "b.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 50,
                comments: 5,
                blanks: 5,
                lines: 60,
                bytes: 1500,
                tokens: 400,
            },
        ],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    };
    let report = derive_report(&export, None);
    assert_eq!(report.distribution.gini, 0.0);
}

// ---------------------------------------------------------------------------
// Context window
// ---------------------------------------------------------------------------

#[test]
fn context_window_none_when_not_requested() {
    let export = single_file_export(100, 10, 5);
    let report = derive_report(&export, None);
    assert!(report.context_window.is_none());
}

#[test]
fn context_window_fits_when_under_budget() {
    let export = single_file_export(100, 10, 5);
    let report = derive_report(&export, Some(100_000));
    let cw = report.context_window.unwrap();
    assert!(cw.fits);
    assert_eq!(cw.window_tokens, 100_000);
}

#[test]
fn context_window_does_not_fit_when_over_budget() {
    let export = single_file_export(100, 10, 5);
    let report = derive_report(&export, Some(1));
    let cw = report.context_window.unwrap();
    assert!(!cw.fits);
}

// ---------------------------------------------------------------------------
// Integrity
// ---------------------------------------------------------------------------

#[test]
fn integrity_hash_is_blake3() {
    let export = single_file_export(100, 10, 5);
    let report = derive_report(&export, None);
    assert_eq!(report.integrity.algo, "blake3");
    assert!(!report.integrity.hash.is_empty());
}

#[test]
fn integrity_hash_deterministic() {
    let export = single_file_export(100, 10, 5);
    let a = derive_report(&export, None);
    let b = derive_report(&export, None);
    assert_eq!(a.integrity.hash, b.integrity.hash);
}

// ---------------------------------------------------------------------------
// Reading time
// ---------------------------------------------------------------------------

#[test]
fn reading_time_proportional_to_code() {
    let export = single_file_export(200, 0, 0);
    let report = derive_report(&export, None);
    // 200 lines / 20 lines per minute = 10.0 minutes
    assert_eq!(report.reading_time.minutes, 10.0);
    assert_eq!(report.reading_time.basis_lines, 200);
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn doc_density_between_zero_and_one(
            code in 0usize..10_000,
            comments in 0usize..10_000,
            blanks in 0usize..1_000,
        ) {
            let export = single_file_export(code, comments, blanks);
            let report = derive_report(&export, None);
            let ratio = report.doc_density.total.ratio;
            prop_assert!((0.0..=1.0).contains(&ratio), "ratio={ratio}");
        }

        #[test]
        fn whitespace_ratio_non_negative(
            code in 0usize..10_000,
            comments in 0usize..10_000,
            blanks in 0usize..1_000,
        ) {
            let export = single_file_export(code, comments, blanks);
            let report = derive_report(&export, None);
            prop_assert!(report.whitespace.total.ratio >= 0.0);
        }

        #[test]
        fn totals_match_input(
            code in 1usize..10_000,
            comments in 0usize..5_000,
            blanks in 0usize..1_000,
        ) {
            let export = single_file_export(code, comments, blanks);
            let report = derive_report(&export, None);
            prop_assert_eq!(report.totals.code, code);
            prop_assert_eq!(report.totals.comments, comments);
            prop_assert_eq!(report.totals.blanks, blanks);
        }

        #[test]
        fn cocomo_effort_non_negative(code in 1usize..100_000) {
            let export = single_file_export(code, 0, 0);
            let report = derive_report(&export, None);
            if let Some(cocomo) = report.cocomo {
                prop_assert!(cocomo.effort_pm >= 0.0);
                prop_assert!(cocomo.duration_months >= 0.0);
                prop_assert!(cocomo.staff >= 0.0);
            }
        }
    }
}
