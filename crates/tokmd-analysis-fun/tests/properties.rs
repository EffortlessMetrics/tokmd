//! Property-based tests for fun/novelty output generation.

use proptest::prelude::*;
use tokmd_analysis_fun::build_fun_report;
use tokmd_analysis_types::{
    BoilerplateReport, DerivedReport, DerivedTotals, DistributionReport, FileStatRow,
    IntegrityReport, LangPurityReport, MaxFileReport, NestingReport, PolyglotReport, RateReport,
    RateRow, RatioReport, RatioRow, ReadingTimeReport, TestDensityReport, TodoReport, TopOffenders,
};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn derived_with_bytes(bytes: usize) -> DerivedReport {
    let zero_row = FileStatRow {
        path: "f.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        code: 0,
        comments: 0,
        blanks: 0,
        lines: 0,
        bytes,
        tokens: 0,
        doc_pct: Some(0.0),
        bytes_per_line: Some(0.0),
        depth: 0,
    };

    DerivedReport {
        totals: DerivedTotals {
            files: 1,
            code: 1,
            comments: 0,
            blanks: 0,
            lines: 1,
            bytes,
            tokens: 1,
        },
        doc_density: RatioReport {
            total: RatioRow {
                key: "All".into(),
                numerator: 0,
                denominator: 1,
                ratio: 0.0,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        whitespace: RatioReport {
            total: RatioRow {
                key: "All".into(),
                numerator: 0,
                denominator: 1,
                ratio: 0.0,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        verbosity: RateReport {
            total: RateRow {
                key: "All".into(),
                numerator: 0,
                denominator: 1,
                rate: 0.0,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        max_file: MaxFileReport {
            overall: zero_row.clone(),
            by_lang: vec![],
            by_module: vec![],
        },
        lang_purity: LangPurityReport { rows: vec![] },
        nesting: NestingReport {
            max: 0,
            avg: 0.0,
            by_module: vec![],
        },
        test_density: TestDensityReport {
            test_lines: 0,
            prod_lines: 0,
            test_files: 0,
            prod_files: 0,
            ratio: 0.0,
        },
        boilerplate: BoilerplateReport {
            infra_lines: 0,
            logic_lines: 0,
            ratio: 0.0,
            infra_langs: vec![],
        },
        polyglot: PolyglotReport {
            lang_count: 0,
            entropy: 0.0,
            dominant_lang: "unknown".to_string(),
            dominant_lines: 0,
            dominant_pct: 0.0,
        },
        distribution: DistributionReport {
            count: 1,
            min: 1,
            max: 1,
            mean: 0.0,
            median: 0.0,
            p90: 0.0,
            p99: 0.0,
            gini: 0.0,
        },
        histogram: Vec::new(),
        top: TopOffenders {
            largest_lines: vec![zero_row.clone()],
            largest_tokens: vec![zero_row.clone()],
            largest_bytes: vec![zero_row.clone()],
            least_documented: vec![zero_row.clone()],
            most_dense: vec![zero_row],
        },
        tree: None,
        reading_time: ReadingTimeReport {
            minutes: 0.0,
            lines_per_minute: 0,
            basis_lines: 0,
        },
        context_window: None,
        cocomo: None,
        todo: Some(TodoReport {
            total: 0,
            density_per_kloc: 0.0,
            tags: vec![],
        }),
        integrity: IntegrityReport {
            algo: "sha1".to_string(),
            hash: "placeholder".to_string(),
            entries: 0,
        },
    }
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    /// eco_label is always present regardless of byte count.
    #[test]
    fn eco_label_always_present(bytes in 0usize..=(2 * 1024 * 1024 * 1024)) {
        let report = build_fun_report(&derived_with_bytes(bytes));
        prop_assert!(report.eco_label.is_some());
    }

    /// bytes field in the eco_label always matches the input.
    #[test]
    fn bytes_field_matches_input(bytes in 0usize..=(2 * 1024 * 1024 * 1024)) {
        let report = build_fun_report(&derived_with_bytes(bytes));
        let eco = report.eco_label.unwrap();
        prop_assert_eq!(eco.bytes, bytes as u64);
    }

    /// Score is always in [0, 100].
    #[test]
    fn score_in_valid_range(bytes in 0usize..=(2 * 1024 * 1024 * 1024)) {
        let eco = build_fun_report(&derived_with_bytes(bytes)).eco_label.unwrap();
        prop_assert!(eco.score >= 0.0 && eco.score <= 100.0);
    }

    /// Label is always a single uppercase ASCII letter.
    #[test]
    fn label_is_single_uppercase_letter(bytes in 0usize..=(2 * 1024 * 1024 * 1024)) {
        let eco = build_fun_report(&derived_with_bytes(bytes)).eco_label.unwrap();
        prop_assert_eq!(eco.label.len(), 1);
        let ch = eco.label.chars().next().unwrap();
        prop_assert!(ch.is_ascii_uppercase(), "label was: {}", eco.label);
    }

    /// Label is one of the five known grades.
    #[test]
    fn label_is_known_grade(bytes in 0usize..=(2 * 1024 * 1024 * 1024)) {
        let eco = build_fun_report(&derived_with_bytes(bytes)).eco_label.unwrap();
        prop_assert!(
            ["A", "B", "C", "D", "E"].contains(&eco.label.as_str()),
            "unexpected label: {}",
            eco.label
        );
    }

    /// Notes always contain "MB" suffix.
    #[test]
    fn notes_contain_mb_suffix(bytes in 0usize..=(2 * 1024 * 1024 * 1024)) {
        let eco = build_fun_report(&derived_with_bytes(bytes)).eco_label.unwrap();
        prop_assert!(eco.notes.contains("MB"), "notes missing MB: {}", eco.notes);
    }

    /// Output is deterministic: same input always gives same output.
    #[test]
    fn deterministic_output(bytes in 0usize..=(2 * 1024 * 1024 * 1024)) {
        let d = derived_with_bytes(bytes);
        let e1 = build_fun_report(&d).eco_label.unwrap();
        let e2 = build_fun_report(&d).eco_label.unwrap();
        prop_assert_eq!(&e1.label, &e2.label);
        prop_assert_eq!(e1.score, e2.score);
        prop_assert_eq!(e1.bytes, e2.bytes);
        prop_assert_eq!(&e1.notes, &e2.notes);
    }

    /// Monotonicity: if bytes_a <= bytes_b then score_a >= score_b.
    #[test]
    fn scores_decrease_as_size_grows(
        a in 0usize..=(1024 * 1024 * 1024),
        b in 0usize..=(1024 * 1024 * 1024),
    ) {
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        let s_lo = build_fun_report(&derived_with_bytes(lo)).eco_label.unwrap().score;
        let s_hi = build_fun_report(&derived_with_bytes(hi)).eco_label.unwrap().score;
        prop_assert!(
            s_lo >= s_hi,
            "score for {} bytes ({}) should be >= score for {} bytes ({})",
            lo, s_lo, hi, s_hi
        );
    }

    /// Notes start with a fixed human-readable prefix.
    #[test]
    fn notes_have_fixed_prefix(bytes in 0usize..=(2 * 1024 * 1024 * 1024)) {
        let eco = build_fun_report(&derived_with_bytes(bytes)).eco_label.unwrap();
        prop_assert!(
            eco.notes.starts_with("Size-based eco label"),
            "unexpected prefix: {}",
            eco.notes
        );
    }
}
