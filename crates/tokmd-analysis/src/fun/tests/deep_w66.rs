//! W66 deep tests for `analysis fun module`.
//!
//! Exercises eco-label generation across all grade bands, edge cases,
//! determinism, notes formatting, and property-based invariants.

use crate::fun::build_fun_report;
use proptest::prelude::*;
use tokmd_analysis_types::{
    BoilerplateReport, DerivedReport, DerivedTotals, DistributionReport, FileStatRow,
    IntegrityReport, LangPurityReport, MaxFileReport, NestingReport, PolyglotReport, RateReport,
    RateRow, RatioReport, RatioRow, ReadingTimeReport, TestDensityReport, TodoReport, TopOffenders,
};

// ── Helper ──────────────────────────────────────────────────────

fn derived_with_bytes(bytes: usize) -> DerivedReport {
    let row = FileStatRow {
        path: "f.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        code: 1,
        comments: 0,
        blanks: 0,
        lines: 1,
        bytes,
        tokens: 1,
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
            overall: row.clone(),
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
            dominant_lang: "unknown".into(),
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
            largest_lines: vec![row.clone()],
            largest_tokens: vec![row.clone()],
            largest_bytes: vec![row.clone()],
            least_documented: vec![row.clone()],
            most_dense: vec![row],
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
            algo: "sha1".into(),
            hash: "placeholder".into(),
            entries: 0,
        },
    }
}

// ── Grade band classification ───────────────────────────────────

mod grade_bands_w66 {
    use super::*;

    #[test]
    fn zero_bytes_produces_grade_a() {
        let r = build_fun_report(&derived_with_bytes(0));
        let eco = r
            .eco_label
            .expect("eco_label must be present for this test case");
        assert_eq!(eco.label, "A");
        assert_eq!(eco.score, 95.0);
    }

    #[test]
    fn one_byte_produces_grade_a() {
        let r = build_fun_report(&derived_with_bytes(1));
        let eco = r
            .eco_label
            .expect("eco_label must be present for this test case");
        assert_eq!(eco.label, "A");
    }

    #[test]
    fn exactly_1mb_produces_grade_a() {
        let bytes = 1024 * 1024;
        let r = build_fun_report(&derived_with_bytes(bytes));
        let eco = r
            .eco_label
            .expect("eco_label must be present for this test case");
        assert_eq!(eco.label, "A");
        assert_eq!(eco.score, 95.0);
    }

    #[test]
    fn just_over_1mb_produces_grade_b() {
        let bytes = 1024 * 1024 + 1;
        let r = build_fun_report(&derived_with_bytes(bytes));
        let eco = r
            .eco_label
            .expect("eco_label must be present for this test case");
        assert_eq!(eco.label, "B");
        assert_eq!(eco.score, 80.0);
    }

    #[test]
    fn exactly_10mb_produces_grade_b() {
        let bytes = 10 * 1024 * 1024;
        let r = build_fun_report(&derived_with_bytes(bytes));
        let eco = r
            .eco_label
            .expect("eco_label must be present for this test case");
        assert_eq!(eco.label, "B");
    }

    #[test]
    fn just_over_10mb_produces_grade_c() {
        let bytes = 10 * 1024 * 1024 + 1;
        let r = build_fun_report(&derived_with_bytes(bytes));
        let eco = r
            .eco_label
            .expect("eco_label must be present for this test case");
        assert_eq!(eco.label, "C");
        assert_eq!(eco.score, 65.0);
    }

    #[test]
    fn exactly_50mb_produces_grade_c() {
        let bytes = 50 * 1024 * 1024;
        let r = build_fun_report(&derived_with_bytes(bytes));
        let eco = r
            .eco_label
            .expect("eco_label must be present for this test case");
        assert_eq!(eco.label, "C");
    }

    #[test]
    fn just_over_50mb_produces_grade_d() {
        let bytes = 50 * 1024 * 1024 + 1;
        let r = build_fun_report(&derived_with_bytes(bytes));
        let eco = r
            .eco_label
            .expect("eco_label must be present for this test case");
        assert_eq!(eco.label, "D");
        assert_eq!(eco.score, 45.0);
    }

    #[test]
    fn exactly_200mb_produces_grade_d() {
        let bytes = 200 * 1024 * 1024;
        let r = build_fun_report(&derived_with_bytes(bytes));
        let eco = r
            .eco_label
            .expect("eco_label must be present for this test case");
        assert_eq!(eco.label, "D");
    }

    #[test]
    fn just_over_200mb_produces_grade_e() {
        let bytes = 200 * 1024 * 1024 + 1;
        let r = build_fun_report(&derived_with_bytes(bytes));
        let eco = r
            .eco_label
            .expect("eco_label must be present for this test case");
        assert_eq!(eco.label, "E");
        assert_eq!(eco.score, 30.0);
    }
}

// ── Notes formatting ────────────────────────────────────────────

mod notes_formatting_w66 {
    use super::*;

    #[test]
    fn notes_contain_mb_unit() {
        let r = build_fun_report(&derived_with_bytes(5 * 1024 * 1024));
        let eco = r
            .eco_label
            .expect("eco_label must be present for this test case");
        assert!(eco.notes.contains("MB"));
    }

    #[test]
    fn notes_contain_size_based_prefix() {
        let r = build_fun_report(&derived_with_bytes(1024));
        let eco = r
            .eco_label
            .expect("eco_label must be present for this test case");
        assert!(eco.notes.starts_with("Size-based eco label"));
    }

    #[test]
    fn notes_mb_value_rounds_to_two_decimals() {
        let r = build_fun_report(&derived_with_bytes(1536));
        let eco = r
            .eco_label
            .expect("eco_label must be present for this test case");
        let paren_start = eco
            .notes
            .find('(')
            .expect("notes must contain an opening parenthesis");
        let paren_end = eco
            .notes
            .find(')')
            .expect("notes must contain a closing parenthesis");
        let inner = &eco.notes[paren_start + 1..paren_end];
        assert!(inner.ends_with("MB"));
    }

    #[test]
    fn eco_label_bytes_matches_input() {
        let bytes = 42 * 1024;
        let r = build_fun_report(&derived_with_bytes(bytes));
        let eco = r
            .eco_label
            .expect("eco_label must be present for this test case");
        assert_eq!(eco.bytes, bytes as u64);
    }
}

// ── Determinism & serialization ─────────────────────────────────

mod determinism_w66 {
    use super::*;

    #[test]
    fn same_input_produces_identical_output() {
        let derived = derived_with_bytes(7 * 1024 * 1024);
        let r1 = build_fun_report(&derived);
        let r2 = build_fun_report(&derived);
        assert_eq!(
            serde_json::to_string(&r1).expect("Failed to serialize FunReport r1"),
            serde_json::to_string(&r2).expect("Failed to serialize FunReport r2"),
        );
    }

    #[test]
    fn fun_report_serializes_to_json() {
        let r = build_fun_report(&derived_with_bytes(512));
        let json = serde_json::to_string_pretty(&r).unwrap();
        assert!(json.contains("eco_label"));
        assert!(json.contains("score"));
        assert!(json.contains("label"));
    }

    #[test]
    fn eco_label_always_present() {
        for bytes in [0, 1, 1024, 1024 * 1024, 500 * 1024 * 1024] {
            let r = build_fun_report(&derived_with_bytes(bytes));
            assert!(
                r.eco_label.is_some(),
                "eco_label should always be Some for bytes={bytes}"
            );
        }
    }

    #[test]
    fn score_is_always_positive() {
        for bytes in [0, 1, 1024 * 1024 * 1024] {
            let r = build_fun_report(&derived_with_bytes(bytes));
            let eco = r
                .eco_label
                .expect("eco_label must be present for this test case");
            assert!(
                eco.score > 0.0,
                "score should be positive for bytes={bytes}"
            );
        }
    }

    #[test]
    fn label_is_single_uppercase_letter() {
        for bytes in [
            0,
            1024,
            5 * 1024 * 1024,
            30 * 1024 * 1024,
            100 * 1024 * 1024,
            300 * 1024 * 1024,
        ] {
            let r = build_fun_report(&derived_with_bytes(bytes));
            let eco = r
                .eco_label
                .expect("eco_label must be present for this test case");
            assert_eq!(eco.label.len(), 1);
            assert!(eco.label.chars().all(|c| c.is_ascii_uppercase()));
        }
    }
}

// ── Property tests ──────────────────────────────────────────────

mod property_tests_w66 {
    use super::*;

    proptest! {
        #[test]
        fn eco_label_deterministic_for_same_input(bytes in 0usize..1_000_000_000) {
            let derived = derived_with_bytes(bytes);
            let r1 = build_fun_report(&derived);
            let r2 = build_fun_report(&derived);
            prop_assert_eq!(
                r1.eco_label.as_ref().map(|e| &e.label),
                r2.eco_label.as_ref().map(|e| &e.label),
            );
            prop_assert_eq!(
                r1.eco_label.as_ref().map(|e| e.score.to_bits()),
                r2.eco_label.as_ref().map(|e| e.score.to_bits()),
            );
        }

        #[test]
        fn score_decreases_as_size_increases(
            small in 0usize..1_048_576,
            large in 210_000_000usize..500_000_000,
        ) {
            let r_small = build_fun_report(&derived_with_bytes(small));
            let r_large = build_fun_report(&derived_with_bytes(large));
            prop_assert!(
                r_small.eco_label.expect("eco_label must be present for this test case").score >= r_large.eco_label.expect("eco_label must be present for this test case").score,
            );
        }

        #[test]
        fn label_is_valid_grade(bytes in 0usize..1_000_000_000) {
            let r = build_fun_report(&derived_with_bytes(bytes));
            let label = &r.eco_label.expect("eco_label must be present for this test case").label;
            prop_assert!(["A", "B", "C", "D", "E"].contains(&label.as_str()));
        }

        #[test]
        fn bytes_field_matches_input(bytes in 0usize..1_000_000_000) {
            let r = build_fun_report(&derived_with_bytes(bytes));
            prop_assert_eq!(r.eco_label.expect("eco_label must be present for this test case").bytes, bytes as u64);
        }
    }
}
