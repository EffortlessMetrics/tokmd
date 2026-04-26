//! Deep property-based and deterministic tests for `analysis derived module`.
//!
//! Covers COCOMO calculations, density metrics, distribution invariants,
//! edge cases (zero-line files, single-language repos), and property-based
//! verification of derived metric bounds.

use crate::derived::derive_report;
use proptest::prelude::*;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────

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

fn simple_row(path: &str, lang: &str, code: usize) -> FileRow {
    row(path, "src", lang, code, 0, 0, code * 40, code * 8)
}

fn export(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    }
}

// ═══════════════════════════════════════════════════════════════════
// § COCOMO cost model
// ═══════════════════════════════════════════════════════════════════

mod cocomo {
    use super::*;

    #[test]
    fn cocomo_known_1000_lines() {
        // 1 KLOC → organic COCOMO: effort = 2.4 * 1.0^1.05
        let r = derive_report(&export(vec![simple_row("a.rs", "Rust", 1000)]), None);
        let c = r.cocomo.as_ref().expect("cocomo present");
        assert_eq!(c.mode, "organic");
        assert!((c.kloc - 1.0).abs() < 0.001);
        assert!(
            (c.effort_pm - 2.4).abs() < 0.1,
            "effort ≈ 2.4, got {}",
            c.effort_pm
        );
        assert!(c.duration_months > 0.0);
        assert!(c.staff > 0.0);
    }

    #[test]
    fn cocomo_known_10000_lines() {
        // 10 KLOC → effort = 2.4 * 10^1.05 ≈ 26.9
        let r = derive_report(&export(vec![simple_row("a.rs", "Rust", 10_000)]), None);
        let c = r.cocomo.as_ref().unwrap();
        assert!((c.kloc - 10.0).abs() < 0.001);
        let expected_effort = 2.4 * 10.0_f64.powf(1.05);
        assert!(
            (c.effort_pm - expected_effort).abs() < 0.5,
            "effort ≈ {expected_effort:.2}, got {}",
            c.effort_pm
        );
    }

    #[test]
    fn cocomo_none_for_zero_code() {
        let r = derive_report(
            &export(vec![row("a.rs", "src", "Rust", 0, 10, 5, 100, 0)]),
            None,
        );
        assert!(r.cocomo.is_none());
    }

    #[test]
    fn cocomo_staff_equals_effort_div_duration() {
        let r = derive_report(&export(vec![simple_row("a.rs", "Rust", 5000)]), None);
        let c = r.cocomo.as_ref().unwrap();
        let expected_staff = c.effort_pm / c.duration_months;
        assert!(
            (c.staff - expected_staff).abs() < 0.1,
            "staff = effort/duration: expected {expected_staff:.2}, got {}",
            c.staff
        );
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Density metrics
// ═══════════════════════════════════════════════════════════════════

mod density {
    use super::*;

    #[test]
    fn doc_density_50_percent() {
        let r = derive_report(
            &export(vec![row("a.rs", "src", "Rust", 50, 50, 0, 4000, 800)]),
            None,
        );
        assert!((r.doc_density.total.ratio - 0.5).abs() < 0.001);
    }

    #[test]
    fn whitespace_ratio_computed_correctly() {
        // blanks / (code + comments) = 10 / 100 = 0.1
        let r = derive_report(
            &export(vec![row("a.rs", "src", "Rust", 80, 20, 10, 4000, 800)]),
            None,
        );
        assert_eq!(r.whitespace.total.numerator, 10);
        assert_eq!(r.whitespace.total.denominator, 100);
        assert!((r.whitespace.total.ratio - 0.1).abs() < 0.001);
    }

    #[test]
    fn density_zero_for_code_only_file() {
        let r = derive_report(
            &export(vec![row("a.rs", "src", "Rust", 100, 0, 0, 4000, 800)]),
            None,
        );
        assert_eq!(r.doc_density.total.ratio, 0.0);
        assert_eq!(r.whitespace.total.ratio, 0.0);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Distribution metrics
// ═══════════════════════════════════════════════════════════════════

mod distribution {
    use super::*;

    #[test]
    fn single_file_distribution() {
        let r = derive_report(&export(vec![simple_row("a.rs", "Rust", 100)]), None);
        let d = &r.distribution;
        assert_eq!(d.count, 1);
        assert_eq!(d.min, 100);
        assert_eq!(d.max, 100);
        assert!((d.mean - 100.0).abs() < 0.01);
        assert_eq!(d.gini, 0.0);
    }

    #[test]
    fn uniform_files_gini_zero() {
        let rows: Vec<FileRow> = (0..5)
            .map(|i| simple_row(&format!("f{i}.rs"), "Rust", 100))
            .collect();
        let r = derive_report(&export(rows), None);
        assert!(r.distribution.gini < 0.01, "uniform → gini ≈ 0");
    }

    #[test]
    fn skewed_files_gini_high() {
        let mut rows = vec![simple_row("big.rs", "Rust", 10_000)];
        for i in 0..9 {
            rows.push(simple_row(&format!("s{i}.rs"), "Rust", 1));
        }
        let r = derive_report(&export(rows), None);
        assert!(
            r.distribution.gini > 0.5,
            "skewed → gini > 0.5, got {}",
            r.distribution.gini
        );
    }

    #[test]
    fn language_concentration_single_lang() {
        let r = derive_report(
            &export(vec![
                simple_row("a.rs", "Rust", 100),
                simple_row("b.rs", "Rust", 200),
            ]),
            None,
        );
        assert_eq!(r.polyglot.lang_count, 1);
        assert_eq!(r.polyglot.entropy, 0.0);
    }

    #[test]
    fn language_concentration_two_langs() {
        let r = derive_report(
            &export(vec![
                simple_row("a.rs", "Rust", 100),
                simple_row("b.py", "Python", 100),
            ]),
            None,
        );
        assert_eq!(r.polyglot.lang_count, 2);
        assert!(r.polyglot.entropy > 0.0);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Edge cases
// ═══════════════════════════════════════════════════════════════════

mod edge_cases {
    use super::*;

    #[test]
    fn zero_line_file() {
        let r = derive_report(
            &export(vec![row("a.rs", "src", "Rust", 0, 0, 0, 0, 0)]),
            None,
        );
        assert_eq!(r.totals.files, 1);
        assert_eq!(r.totals.code, 0);
        assert!(r.cocomo.is_none());
    }

    #[test]
    fn all_blanks_file() {
        let r = derive_report(
            &export(vec![row("a.rs", "src", "Rust", 0, 0, 100, 100, 0)]),
            None,
        );
        assert_eq!(r.totals.blanks, 100);
        assert_eq!(r.totals.code, 0);
    }

    #[test]
    fn empty_export_data() {
        let r = derive_report(&export(vec![]), None);
        assert_eq!(r.totals.files, 0);
        assert_eq!(r.totals.code, 0);
        assert!(r.cocomo.is_none());
        assert_eq!(r.distribution.count, 0);
    }

    #[test]
    fn context_window_fits_when_tokens_under_budget() {
        let r = derive_report(
            &export(vec![row("a.rs", "src", "Rust", 10, 0, 0, 100, 50)]),
            Some(1000),
        );
        let cw = r.context_window.as_ref().unwrap();
        assert!(cw.fits);
        assert_eq!(cw.total_tokens, 50);
        assert_eq!(cw.window_tokens, 1000);
    }

    #[test]
    fn context_window_does_not_fit() {
        let r = derive_report(
            &export(vec![row("a.rs", "src", "Rust", 10, 0, 0, 100, 2000)]),
            Some(100),
        );
        let cw = r.context_window.as_ref().unwrap();
        assert!(!cw.fits);
    }

    #[test]
    fn context_window_none_when_no_budget() {
        let r = derive_report(&export(vec![simple_row("a.rs", "Rust", 10)]), None);
        assert!(r.context_window.is_none());
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Property-based tests
// ═══════════════════════════════════════════════════════════════════

fn arb_file_row() -> impl Strategy<Value = FileRow> {
    (
        "[a-z]{1,4}(/[a-z]{1,4}){0,3}\\.rs",
        "(root|src|lib|tests)",
        "(Rust|Python|TypeScript|TOML|JSON|Markdown)",
        0..5000usize,
        0..1000usize,
        0..500usize,
        0..500_000usize,
        0..100_000usize,
    )
        .prop_map(
            |(path, module, lang, code, comments, blanks, bytes, tokens)| FileRow {
                path,
                module,
                lang,
                kind: FileKind::Parent,
                code,
                comments,
                blanks,
                lines: code + comments + blanks,
                bytes,
                tokens,
            },
        )
}

fn arb_rows() -> impl Strategy<Value = Vec<FileRow>> {
    prop::collection::vec(arb_file_row(), 1..=20)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(150))]

    #[test]
    fn prop_density_values_in_unit_range(rows in arb_rows()) {
        let r = derive_report(&export(rows), None);
        prop_assert!(
            r.doc_density.total.ratio >= 0.0 && r.doc_density.total.ratio <= 1.0,
            "doc_density ratio must be in [0,1], got {}",
            r.doc_density.total.ratio
        );
        prop_assert!(
            r.whitespace.total.ratio >= 0.0,
            "whitespace ratio must be non-negative, got {}",
            r.whitespace.total.ratio
        );
    }

    #[test]
    fn prop_cocomo_effort_non_negative(rows in arb_rows()) {
        let r = derive_report(&export(rows), None);
        if let Some(c) = &r.cocomo {
            prop_assert!(c.effort_pm >= 0.0, "effort must be non-negative");
            prop_assert!(c.duration_months >= 0.0, "duration must be non-negative");
            prop_assert!(c.staff >= 0.0, "staff must be non-negative");
            prop_assert!(c.kloc >= 0.0, "kloc must be non-negative");
        }
    }

    #[test]
    fn prop_distribution_min_le_max(rows in arb_rows()) {
        let r = derive_report(&export(rows), None);
        prop_assert!(
            r.distribution.min <= r.distribution.max,
            "min {} must be <= max {}",
            r.distribution.min, r.distribution.max
        );
    }

    #[test]
    fn prop_distribution_gini_in_unit_range(rows in arb_rows()) {
        let r = derive_report(&export(rows), None);
        prop_assert!(
            r.distribution.gini >= 0.0 && r.distribution.gini <= 1.0,
            "gini must be in [0,1], got {}",
            r.distribution.gini
        );
    }

    #[test]
    fn prop_totals_match_sum(rows in arb_rows()) {
        let expected_code: usize = rows.iter().map(|r| r.code).sum();
        let expected_comments: usize = rows.iter().map(|r| r.comments).sum();
        let expected_blanks: usize = rows.iter().map(|r| r.blanks).sum();
        let r = derive_report(&export(rows), None);
        prop_assert_eq!(r.totals.code, expected_code);
        prop_assert_eq!(r.totals.comments, expected_comments);
        prop_assert_eq!(r.totals.blanks, expected_blanks);
    }

    #[test]
    fn prop_histogram_sums_to_file_count(rows in arb_rows()) {
        let n = rows.len();
        let r = derive_report(&export(rows), None);
        let total: usize = r.histogram.iter().map(|b| b.files).sum();
        prop_assert_eq!(total, n, "histogram file counts must sum to total files");
    }

    #[test]
    fn prop_reading_time_non_negative(rows in arb_rows()) {
        let r = derive_report(&export(rows), None);
        prop_assert!(r.reading_time.minutes >= 0.0);
    }

    #[test]
    fn prop_polyglot_entropy_non_negative(rows in arb_rows()) {
        let r = derive_report(&export(rows), None);
        prop_assert!(r.polyglot.entropy >= 0.0);
    }

    #[test]
    fn prop_integrity_hash_64_hex(rows in arb_rows()) {
        let r = derive_report(&export(rows), None);
        prop_assert_eq!(r.integrity.hash.len(), 64);
        prop_assert!(r.integrity.hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn prop_top_offenders_bounded(rows in arb_rows()) {
        let r = derive_report(&export(rows), None);
        prop_assert!(r.top.largest_lines.len() <= 10);
        prop_assert!(r.top.largest_tokens.len() <= 10);
        prop_assert!(r.top.largest_bytes.len() <= 10);
    }
}
