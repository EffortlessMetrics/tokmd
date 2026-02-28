//! Property-based tests for derived metric invariants.

use proptest::prelude::*;
use tokmd_analysis_derived::derive_report;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────

fn export(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    }
}

// ── Strategies ──────────────────────────────────────────────────

fn arb_file_row() -> impl Strategy<Value = FileRow> {
    (
        "[a-z]{1,4}(/[a-z]{1,4}){0,3}\\.rs",
        "(root|src|lib|tests)",
        "(Rust|Python|TypeScript|TOML|JSON|Markdown)",
        0..5000usize,   // code
        0..1000usize,   // comments
        0..500usize,    // blanks
        0..500000usize, // bytes
        0..100000usize, // tokens
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

fn arb_file_rows() -> impl Strategy<Value = Vec<FileRow>> {
    prop::collection::vec(arb_file_row(), 1..=20)
}

fn arb_window_tokens() -> impl Strategy<Value = Option<usize>> {
    prop_oneof![
        Just(None),
        (1..=500_000usize).prop_map(Some),
    ]
}

// ── Properties ──────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn totals_equal_sum_of_rows(rows in arb_file_rows()) {
        let expected_code: usize = rows.iter().map(|r| r.code).sum();
        let expected_comments: usize = rows.iter().map(|r| r.comments).sum();
        let expected_blanks: usize = rows.iter().map(|r| r.blanks).sum();
        let expected_lines: usize = rows.iter().map(|r| r.lines).sum();
        let expected_bytes: usize = rows.iter().map(|r| r.bytes).sum();
        let expected_tokens: usize = rows.iter().map(|r| r.tokens).sum();

        let report = derive_report(&export(rows.clone()), None);

        prop_assert_eq!(report.totals.files, rows.len());
        prop_assert_eq!(report.totals.code, expected_code);
        prop_assert_eq!(report.totals.comments, expected_comments);
        prop_assert_eq!(report.totals.blanks, expected_blanks);
        prop_assert_eq!(report.totals.lines, expected_lines);
        prop_assert_eq!(report.totals.bytes, expected_bytes);
        prop_assert_eq!(report.totals.tokens, expected_tokens);
    }

    #[test]
    fn cocomo_is_none_iff_zero_code(rows in arb_file_rows()) {
        let total_code: usize = rows.iter().map(|r| r.code).sum();
        let report = derive_report(&export(rows), None);

        if total_code == 0 {
            prop_assert!(report.cocomo.is_none(), "COCOMO should be None when code is 0");
        } else {
            prop_assert!(report.cocomo.is_some(), "COCOMO should be Some when code > 0");
        }
    }

    #[test]
    fn cocomo_effort_and_duration_positive(rows in arb_file_rows()) {
        let report = derive_report(&export(rows), None);
        if let Some(cocomo) = &report.cocomo {
            prop_assert!(cocomo.kloc >= 0.0, "kloc must be non-negative");
            prop_assert!(cocomo.effort_pm > 0.0, "effort must be positive for non-zero code");
            prop_assert!(cocomo.duration_months > 0.0, "duration must be positive");
            prop_assert!(cocomo.staff > 0.0, "staff must be positive");
        }
    }

    #[test]
    fn context_window_fits_iff_tokens_le_window(rows in arb_file_rows(), window in arb_window_tokens()) {
        let report = derive_report(&export(rows), window);

        if let Some(cw) = &report.context_window {
            if cw.window_tokens == 0 {
                prop_assert_eq!(cw.pct, 0.0);
            } else if cw.total_tokens <= cw.window_tokens {
                prop_assert!(cw.fits, "should fit when total_tokens <= window_tokens");
            } else {
                prop_assert!(!cw.fits, "should not fit when total_tokens > window_tokens");
            }
        } else {
            prop_assert!(window.is_none(), "context_window is None iff no window passed");
        }
    }

    #[test]
    fn context_window_pct_non_negative(rows in arb_file_rows(), window in arb_window_tokens()) {
        let report = derive_report(&export(rows), window);
        if let Some(cw) = &report.context_window {
            prop_assert!(cw.pct >= 0.0, "pct must be non-negative, got {}", cw.pct);
        }
    }

    #[test]
    fn distribution_min_le_max(rows in arb_file_rows()) {
        let report = derive_report(&export(rows), None);
        let d = &report.distribution;
        prop_assert!(d.min <= d.max, "min ({}) must be <= max ({})", d.min, d.max);
    }

    #[test]
    fn distribution_count_equals_file_count(rows in arb_file_rows()) {
        let report = derive_report(&export(rows.clone()), None);
        prop_assert_eq!(report.distribution.count, rows.len());
    }

    #[test]
    fn distribution_mean_between_min_and_max(rows in arb_file_rows()) {
        let report = derive_report(&export(rows), None);
        let d = &report.distribution;
        prop_assert!(
            d.mean >= d.min as f64 && d.mean <= d.max as f64,
            "mean ({}) should be in [{}, {}]",
            d.mean, d.min, d.max
        );
    }

    #[test]
    fn distribution_gini_in_unit_range(rows in arb_file_rows()) {
        let report = derive_report(&export(rows), None);
        prop_assert!(
            report.distribution.gini >= 0.0 && report.distribution.gini <= 1.0,
            "gini should be in [0, 1], got {}",
            report.distribution.gini
        );
    }

    #[test]
    fn histogram_file_counts_sum_to_total(rows in arb_file_rows()) {
        let report = derive_report(&export(rows.clone()), None);
        let total: usize = report.histogram.iter().map(|b| b.files).sum();
        prop_assert_eq!(total, rows.len(), "histogram file counts must sum to total files");
    }

    #[test]
    fn histogram_pcts_sum_to_approximately_one(rows in arb_file_rows()) {
        let report = derive_report(&export(rows), None);
        let total_pct: f64 = report.histogram.iter().map(|b| b.pct).sum();
        prop_assert!(
            (total_pct - 1.0).abs() < 0.02,
            "histogram pcts should sum to ~1.0, got {}",
            total_pct
        );
    }

    #[test]
    fn reading_time_proportional_to_code(rows in arb_file_rows()) {
        let total_code: usize = rows.iter().map(|r| r.code).sum();
        let report = derive_report(&export(rows), None);
        prop_assert_eq!(report.reading_time.basis_lines, total_code);
        prop_assert_eq!(report.reading_time.lines_per_minute, 20);
    }

    #[test]
    fn doc_density_ratio_non_negative(rows in arb_file_rows()) {
        let report = derive_report(&export(rows), None);
        prop_assert!(
            report.doc_density.total.ratio >= 0.0,
            "doc_density ratio must be non-negative"
        );
    }

    #[test]
    fn polyglot_lang_count_matches_distinct_langs(rows in arb_file_rows()) {
        let distinct: std::collections::BTreeSet<String> =
            rows.iter().map(|r| r.lang.clone()).collect();
        let report = derive_report(&export(rows), None);
        prop_assert_eq!(
            report.polyglot.lang_count, distinct.len(),
            "polyglot lang_count should match distinct languages"
        );
    }

    #[test]
    fn polyglot_entropy_non_negative(rows in arb_file_rows()) {
        let report = derive_report(&export(rows), None);
        prop_assert!(
            report.polyglot.entropy >= 0.0,
            "entropy must be non-negative, got {}",
            report.polyglot.entropy
        );
    }

    #[test]
    fn integrity_hash_is_64_hex_chars(rows in arb_file_rows()) {
        let report = derive_report(&export(rows), None);
        prop_assert_eq!(report.integrity.hash.len(), 64);
        prop_assert!(
            report.integrity.hash.chars().all(|c| c.is_ascii_hexdigit()),
            "hash should be hex"
        );
    }

    #[test]
    fn integrity_entries_matches_file_count(rows in arb_file_rows()) {
        let report = derive_report(&export(rows.clone()), None);
        prop_assert_eq!(report.integrity.entries, rows.len());
    }

    #[test]
    fn top_offenders_bounded_by_ten(rows in arb_file_rows()) {
        let report = derive_report(&export(rows), None);
        prop_assert!(report.top.largest_lines.len() <= 10);
        prop_assert!(report.top.largest_tokens.len() <= 10);
        prop_assert!(report.top.largest_bytes.len() <= 10);
        prop_assert!(report.top.least_documented.len() <= 10);
        prop_assert!(report.top.most_dense.len() <= 10);
    }

    #[test]
    fn nesting_max_is_at_least_avg(rows in arb_file_rows()) {
        let report = derive_report(&export(rows), None);
        prop_assert!(
            report.nesting.max as f64 >= report.nesting.avg,
            "max ({}) must be >= avg ({})",
            report.nesting.max, report.nesting.avg
        );
    }
}
