//! Contract tests for `tokmd-analysis-derived` enricher (w65).
//!
//! Covers: totals aggregation, doc density, whitespace ratio, verbosity,
//! distribution statistics, COCOMO estimation, context window, test density,
//! boilerplate detection, polyglot metrics, histogram, integrity, nesting,
//! reading time, and property-based invariants.

use tokmd_analysis_derived::derive_report;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ──────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn row(
    path: &str,
    module: &str,
    lang: &str,
    code: usize,
    comments: usize,
    blanks: usize,
    bytes: usize,
    tokens: usize,
) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code,
        comments,
        blanks,
        lines: code + comments + blanks,
        bytes,
        tokens,
    }
}

fn child_row(path: &str, lang: &str, code: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: "root".to_string(),
        lang: lang.to_string(),
        kind: FileKind::Child,
        code,
        comments: 0,
        blanks: 0,
        lines: code,
        bytes: code * 20,
        tokens: code * 5,
    }
}

fn export(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    }
}

// ── Totals aggregation ──────────────────────────────────────────

mod totals {
    use super::*;

    #[test]
    fn single_file_totals_match() {
        let r = derive_report(&export(vec![row("a.rs", "src", "Rust", 100, 20, 10, 3000, 500)]), None);
        assert_eq!(r.totals.files, 1);
        assert_eq!(r.totals.code, 100);
        assert_eq!(r.totals.comments, 20);
        assert_eq!(r.totals.blanks, 10);
        assert_eq!(r.totals.lines, 130);
        assert_eq!(r.totals.bytes, 3000);
        assert_eq!(r.totals.tokens, 500);
    }

    #[test]
    fn multi_file_totals_sum_correctly() {
        let data = export(vec![
            row("a.rs", "src", "Rust", 80, 10, 5, 2000, 400),
            row("b.py", "lib", "Python", 120, 30, 15, 4000, 600),
        ]);
        let r = derive_report(&data, None);
        assert_eq!(r.totals.files, 2);
        assert_eq!(r.totals.code, 200);
        assert_eq!(r.totals.comments, 40);
        assert_eq!(r.totals.blanks, 20);
        assert_eq!(r.totals.lines, 260);
    }

    #[test]
    fn child_rows_excluded_from_totals() {
        let data = export(vec![
            row("a.rs", "src", "Rust", 100, 10, 5, 2000, 400),
            child_row("a.rs/html", "HTML", 50),
        ]);
        let r = derive_report(&data, None);
        assert_eq!(r.totals.files, 1);
        assert_eq!(r.totals.code, 100);
    }

    #[test]
    fn empty_export_yields_zero_totals() {
        let r = derive_report(&export(vec![]), None);
        assert_eq!(r.totals.files, 0);
        assert_eq!(r.totals.code, 0);
        assert_eq!(r.totals.lines, 0);
        assert_eq!(r.totals.bytes, 0);
        assert_eq!(r.totals.tokens, 0);
    }
}

// ── Doc density ─────────────────────────────────────────────────

mod doc_density {
    use super::*;

    #[test]
    fn pure_code_has_zero_doc_density() {
        let r = derive_report(&export(vec![row("a.rs", "src", "Rust", 100, 0, 10, 2000, 500)]), None);
        assert_eq!(r.doc_density.total.ratio, 0.0);
    }

    #[test]
    fn equal_code_and_comments_gives_half() {
        let r = derive_report(&export(vec![row("a.rs", "src", "Rust", 50, 50, 0, 2000, 500)]), None);
        assert!((r.doc_density.total.ratio - 0.5).abs() < 1e-6);
    }

    #[test]
    fn doc_density_by_lang_present() {
        let data = export(vec![
            row("a.rs", "src", "Rust", 80, 20, 5, 2000, 400),
            row("b.py", "lib", "Python", 60, 40, 5, 2000, 400),
        ]);
        let r = derive_report(&data, None);
        assert!(r.doc_density.by_lang.len() >= 2);
    }

    #[test]
    fn doc_density_ratio_bounded_zero_to_one() {
        let data = export(vec![
            row("a.rs", "src", "Rust", 10, 90, 5, 2000, 400),
        ]);
        let r = derive_report(&data, None);
        assert!((0.0..=1.0).contains(&r.doc_density.total.ratio));
    }
}

// ── Whitespace ratio ────────────────────────────────────────────

mod whitespace {
    use super::*;

    #[test]
    fn no_blanks_yields_zero_whitespace() {
        let r = derive_report(&export(vec![row("a.rs", "src", "Rust", 100, 20, 0, 2000, 500)]), None);
        assert_eq!(r.whitespace.total.ratio, 0.0);
    }

    #[test]
    fn whitespace_ratio_increases_with_blanks() {
        let r = derive_report(
            &export(vec![row("a.rs", "src", "Rust", 50, 10, 40, 2000, 500)]),
            None,
        );
        assert!(r.whitespace.total.ratio > 0.0);
    }
}

// ── Verbosity (bytes per line) ──────────────────────────────────

mod verbosity {
    use super::*;

    #[test]
    fn verbosity_rate_correct() {
        let r = derive_report(
            &export(vec![row("a.rs", "src", "Rust", 100, 0, 0, 5000, 500)]),
            None,
        );
        assert!((r.verbosity.total.rate - 50.0).abs() < 1e-6);
    }

    #[test]
    fn zero_lines_yields_zero_verbosity() {
        let r = derive_report(&export(vec![]), None);
        assert_eq!(r.verbosity.total.rate, 0.0);
    }
}

// ── Distribution ────────────────────────────────────────────────

mod distribution {
    use super::*;

    #[test]
    fn single_file_distribution() {
        let r = derive_report(&export(vec![row("a.rs", "src", "Rust", 100, 20, 10, 2000, 500)]), None);
        assert_eq!(r.distribution.count, 1);
        assert_eq!(r.distribution.min, 130);
        assert_eq!(r.distribution.max, 130);
        assert!((r.distribution.mean - 130.0).abs() < 1e-6);
    }

    #[test]
    fn multi_file_distribution_stats() {
        let data = export(vec![
            row("a.rs", "src", "Rust", 10, 0, 0, 200, 50),
            row("b.rs", "src", "Rust", 90, 0, 0, 2000, 450),
        ]);
        let r = derive_report(&data, None);
        assert_eq!(r.distribution.count, 2);
        assert_eq!(r.distribution.min, 10);
        assert_eq!(r.distribution.max, 90);
        assert!((r.distribution.mean - 50.0).abs() < 1e-6);
    }

    #[test]
    fn empty_distribution_all_zero() {
        let r = derive_report(&export(vec![]), None);
        assert_eq!(r.distribution.count, 0);
        assert_eq!(r.distribution.min, 0);
        assert_eq!(r.distribution.max, 0);
        assert_eq!(r.distribution.gini, 0.0);
    }

    #[test]
    fn gini_zero_for_equal_files() {
        let data = export(vec![
            row("a.rs", "src", "Rust", 100, 0, 0, 2000, 500),
            row("b.rs", "src", "Rust", 100, 0, 0, 2000, 500),
            row("c.rs", "src", "Rust", 100, 0, 0, 2000, 500),
        ]);
        let r = derive_report(&data, None);
        assert!((r.distribution.gini).abs() < 1e-6);
    }

    #[test]
    fn gini_positive_for_unequal_files() {
        let data = export(vec![
            row("a.rs", "src", "Rust", 1, 0, 0, 20, 5),
            row("b.rs", "src", "Rust", 1000, 0, 0, 20000, 5000),
        ]);
        let r = derive_report(&data, None);
        assert!(r.distribution.gini > 0.0);
    }
}

// ── COCOMO ──────────────────────────────────────────────────────

mod cocomo {
    use super::*;

    #[test]
    fn cocomo_none_for_zero_code() {
        let r = derive_report(&export(vec![]), None);
        assert!(r.cocomo.is_none());
    }

    #[test]
    fn cocomo_present_for_nonzero_code() {
        let r = derive_report(
            &export(vec![row("a.rs", "src", "Rust", 1000, 100, 50, 20000, 5000)]),
            None,
        );
        let c = r.cocomo.as_ref().unwrap();
        assert_eq!(c.mode, "organic");
        assert!((c.kloc - 1.0).abs() < 1e-4);
        assert!(c.effort_pm > 0.0);
        assert!(c.duration_months > 0.0);
        assert!(c.staff > 0.0);
    }

    #[test]
    fn cocomo_effort_increases_with_code() {
        let small = derive_report(
            &export(vec![row("a.rs", "src", "Rust", 100, 0, 0, 2000, 500)]),
            None,
        );
        let large = derive_report(
            &export(vec![row("a.rs", "src", "Rust", 10000, 0, 0, 200000, 50000)]),
            None,
        );
        assert!(
            large.cocomo.as_ref().unwrap().effort_pm > small.cocomo.as_ref().unwrap().effort_pm
        );
    }

    #[test]
    fn cocomo_coefficients_are_organic() {
        let r = derive_report(
            &export(vec![row("a.rs", "src", "Rust", 500, 0, 0, 10000, 2500)]),
            None,
        );
        let c = r.cocomo.unwrap();
        assert!((c.a - 2.4).abs() < 1e-6);
        assert!((c.b - 1.05).abs() < 1e-6);
        assert!((c.c - 2.5).abs() < 1e-6);
        assert!((c.d - 0.38).abs() < 1e-6);
    }
}

// ── Context window ──────────────────────────────────────────────

mod context_window {
    use super::*;

    #[test]
    fn context_window_none_when_not_requested() {
        let r = derive_report(
            &export(vec![row("a.rs", "src", "Rust", 100, 0, 0, 2000, 500)]),
            None,
        );
        assert!(r.context_window.is_none());
    }

    #[test]
    fn context_window_fits_when_tokens_below_budget() {
        let r = derive_report(
            &export(vec![row("a.rs", "src", "Rust", 100, 0, 0, 2000, 500)]),
            Some(10000),
        );
        let cw = r.context_window.unwrap();
        assert!(cw.fits);
        assert_eq!(cw.total_tokens, 500);
        assert_eq!(cw.window_tokens, 10000);
    }

    #[test]
    fn context_window_does_not_fit_when_tokens_exceed_budget() {
        let r = derive_report(
            &export(vec![row("a.rs", "src", "Rust", 100, 0, 0, 2000, 5000)]),
            Some(1000),
        );
        let cw = r.context_window.unwrap();
        assert!(!cw.fits);
        assert!(cw.pct > 1.0);
    }

    #[test]
    fn context_window_zero_window_pct_is_zero() {
        let r = derive_report(
            &export(vec![row("a.rs", "src", "Rust", 100, 0, 0, 2000, 500)]),
            Some(0),
        );
        let cw = r.context_window.unwrap();
        assert_eq!(cw.pct, 0.0);
    }
}

// ── Test density ────────────────────────────────────────────────

mod test_density {
    use super::*;

    #[test]
    fn no_test_files_yields_zero_ratio() {
        let r = derive_report(
            &export(vec![row("src/main.rs", "src", "Rust", 200, 0, 0, 4000, 1000)]),
            None,
        );
        assert_eq!(r.test_density.test_files, 0);
        assert_eq!(r.test_density.ratio, 0.0);
    }

    #[test]
    fn test_files_counted_separately() {
        let data = export(vec![
            row("src/main.rs", "src", "Rust", 200, 0, 0, 4000, 1000),
            row("tests/test_main.rs", "tests", "Rust", 100, 0, 0, 2000, 500),
        ]);
        let r = derive_report(&data, None);
        assert_eq!(r.test_density.test_files, 1);
        assert_eq!(r.test_density.prod_files, 1);
        assert_eq!(r.test_density.test_lines, 100);
        assert_eq!(r.test_density.prod_lines, 200);
    }
}

// ── Boilerplate ─────────────────────────────────────────────────

mod boilerplate {
    use super::*;

    #[test]
    fn no_infra_langs_yields_zero_ratio() {
        let r = derive_report(
            &export(vec![row("a.rs", "src", "Rust", 100, 0, 0, 2000, 500)]),
            None,
        );
        assert_eq!(r.boilerplate.ratio, 0.0);
    }

    #[test]
    fn infra_langs_contribute_to_boilerplate() {
        let data = export(vec![
            row("a.rs", "src", "Rust", 100, 0, 0, 2000, 500),
            row("b.toml", "src", "TOML", 50, 5, 5, 1000, 250),
        ]);
        let r = derive_report(&data, None);
        assert!(r.boilerplate.ratio > 0.0);
    }
}

// ── Polyglot metrics ────────────────────────────────────────────

mod polyglot {
    use super::*;

    #[test]
    fn single_lang_zero_entropy() {
        let r = derive_report(
            &export(vec![row("a.rs", "src", "Rust", 100, 0, 0, 2000, 500)]),
            None,
        );
        assert_eq!(r.polyglot.lang_count, 1);
        assert_eq!(r.polyglot.entropy, 0.0);
        assert_eq!(r.polyglot.dominant_lang, "Rust");
    }

    #[test]
    fn multiple_langs_positive_entropy() {
        let data = export(vec![
            row("a.rs", "src", "Rust", 100, 0, 0, 2000, 500),
            row("b.py", "src", "Python", 100, 0, 0, 2000, 500),
        ]);
        let r = derive_report(&data, None);
        assert_eq!(r.polyglot.lang_count, 2);
        assert!(r.polyglot.entropy > 0.0);
    }

    #[test]
    fn dominant_lang_has_most_code() {
        let data = export(vec![
            row("a.rs", "src", "Rust", 200, 0, 0, 4000, 1000),
            row("b.py", "src", "Python", 50, 0, 0, 1000, 250),
        ]);
        let r = derive_report(&data, None);
        assert_eq!(r.polyglot.dominant_lang, "Rust");
        assert_eq!(r.polyglot.dominant_lines, 200);
    }
}

// ── Histogram ───────────────────────────────────────────────────

mod histogram {
    use super::*;

    #[test]
    fn histogram_has_five_buckets() {
        let r = derive_report(
            &export(vec![row("a.rs", "src", "Rust", 100, 0, 0, 2000, 500)]),
            None,
        );
        assert_eq!(r.histogram.len(), 5);
    }

    #[test]
    fn tiny_file_in_first_bucket() {
        let r = derive_report(
            &export(vec![row("a.rs", "src", "Rust", 10, 0, 0, 200, 50)]),
            None,
        );
        assert_eq!(r.histogram[0].files, 1);
        assert_eq!(r.histogram[0].label, "Tiny");
    }

    #[test]
    fn huge_file_in_last_bucket() {
        let r = derive_report(
            &export(vec![row("a.rs", "src", "Rust", 1500, 0, 0, 30000, 7500)]),
            None,
        );
        assert_eq!(r.histogram[4].files, 1);
        assert_eq!(r.histogram[4].label, "Huge");
    }

    #[test]
    fn histogram_percentages_sum_to_one() {
        let data = export(vec![
            row("a.rs", "src", "Rust", 10, 0, 0, 200, 50),
            row("b.rs", "src", "Rust", 150, 0, 0, 3000, 750),
            row("c.rs", "src", "Rust", 400, 0, 0, 8000, 2000),
        ]);
        let r = derive_report(&data, None);
        let total_pct: f64 = r.histogram.iter().map(|b| b.pct).sum();
        assert!((total_pct - 1.0).abs() < 0.01);
    }
}

// ── Integrity ───────────────────────────────────────────────────

mod integrity {
    use super::*;

    #[test]
    fn integrity_uses_blake3() {
        let r = derive_report(
            &export(vec![row("a.rs", "src", "Rust", 100, 0, 0, 2000, 500)]),
            None,
        );
        assert_eq!(r.integrity.algo, "blake3");
        assert!(!r.integrity.hash.is_empty());
    }

    #[test]
    fn integrity_entries_match_file_count() {
        let data = export(vec![
            row("a.rs", "src", "Rust", 100, 0, 0, 2000, 500),
            row("b.rs", "src", "Rust", 50, 0, 0, 1000, 250),
        ]);
        let r = derive_report(&data, None);
        assert_eq!(r.integrity.entries, 2);
    }

    #[test]
    fn integrity_deterministic() {
        let data = export(vec![
            row("a.rs", "src", "Rust", 100, 10, 5, 2000, 500),
            row("b.rs", "lib", "Python", 50, 5, 3, 1000, 250),
        ]);
        let r1 = derive_report(&data, None);
        let r2 = derive_report(&data, None);
        assert_eq!(r1.integrity.hash, r2.integrity.hash);
    }
}

// ── Nesting ─────────────────────────────────────────────────────

mod nesting {
    use super::*;

    #[test]
    fn flat_files_have_low_nesting() {
        let r = derive_report(
            &export(vec![row("main.rs", ".", "Rust", 100, 0, 0, 2000, 500)]),
            None,
        );
        // path_depth("main.rs") = 1 (single path segment)
        assert!(r.nesting.max <= 1);
    }

    #[test]
    fn deeply_nested_file_increases_nesting() {
        let r = derive_report(
            &export(vec![row("a/b/c/d/e.rs", "a/b", "Rust", 100, 0, 0, 2000, 500)]),
            None,
        );
        assert!(r.nesting.max > 0);
    }
}

// ── Reading time ────────────────────────────────────────────────

mod reading_time {
    use super::*;

    #[test]
    fn reading_time_based_on_code_lines() {
        let r = derive_report(
            &export(vec![row("a.rs", "src", "Rust", 200, 0, 0, 4000, 1000)]),
            None,
        );
        assert!((r.reading_time.minutes - 10.0).abs() < 1e-6);
        assert_eq!(r.reading_time.lines_per_minute, 20);
        assert_eq!(r.reading_time.basis_lines, 200);
    }

    #[test]
    fn zero_code_zero_reading_time() {
        let r = derive_report(&export(vec![]), None);
        assert_eq!(r.reading_time.minutes, 0.0);
        assert_eq!(r.reading_time.basis_lines, 0);
    }
}

// ── Property tests ──────────────────────────────────────────────

mod properties {
    use super::*;
    use proptest::prelude::*;

    fn arb_row() -> impl Strategy<Value = FileRow> {
        (1..5000usize, 0..2000usize, 0..500usize, 1..50000usize, 1..25000usize)
            .prop_map(|(code, comments, blanks, bytes, tokens)| {
                row("src/f.rs", "src", "Rust", code, comments, blanks, bytes, tokens)
            })
    }

    proptest! {
        #[test]
        fn doc_density_always_in_unit_interval(r in arb_row()) {
            let report = derive_report(&export(vec![r]), None);
            prop_assert!((0.0..=1.0).contains(&report.doc_density.total.ratio));
        }

        #[test]
        fn whitespace_ratio_always_non_negative(r in arb_row()) {
            let report = derive_report(&export(vec![r]), None);
            prop_assert!(report.whitespace.total.ratio >= 0.0);
        }

        #[test]
        fn distribution_min_le_max(r in arb_row()) {
            let report = derive_report(&export(vec![r]), None);
            prop_assert!(report.distribution.min <= report.distribution.max);
        }

        #[test]
        fn cocomo_effort_non_negative(code in 1..100000usize) {
            let r = row("a.rs", "src", "Rust", code, 0, 0, code * 20, code * 5);
            let report = derive_report(&export(vec![r]), None);
            if let Some(c) = report.cocomo {
                prop_assert!(c.effort_pm >= 0.0);
                prop_assert!(c.duration_months >= 0.0);
                prop_assert!(c.staff >= 0.0);
            }
        }

        #[test]
        fn totals_code_equals_sum_of_rows(
            c1 in 0..5000usize,
            c2 in 0..5000usize,
        ) {
            let data = export(vec![
                row("a.rs", "src", "Rust", c1, 0, 0, c1 * 20, c1 * 5),
                row("b.rs", "src", "Rust", c2, 0, 0, c2 * 20, c2 * 5),
            ]);
            let report = derive_report(&data, None);
            prop_assert_eq!(report.totals.code, c1 + c2);
        }
    }
}
