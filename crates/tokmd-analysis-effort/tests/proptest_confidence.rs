use proptest::prelude::*;
use tokmd_analysis_effort::confidence::build_confidence;
use tokmd_analysis_types::{
    ApiSurfaceReport, BoilerplateReport, ComplexityReport, DerivedReport, DerivedTotals,
    DistributionReport, DuplicateReport, EffortConfidenceLevel, EffortSizeBasis, FileStatRow,
    FreshnessReport, GitReport, IntegrityReport, LangPurityReport, MaxFileReport, NestingReport,
    PolyglotReport, RateReport, RateRow, RatioReport, RatioRow, ReadingTimeReport,
    TestDensityReport, TopOffenders,
};

fn gen_size_basis() -> impl Strategy<Value = EffortSizeBasis> {
    (
        0.0..=1.0f64,
        0.0..=1.0f64,
        prop::sample::select(vec![
            EffortConfidenceLevel::Low,
            EffortConfidenceLevel::Medium,
            EffortConfidenceLevel::High,
        ]),
        any::<bool>(),
    )
        .prop_map(|(gen_pct, vend_pct, conf, has_warnings)| {
            let mut warnings = Vec::new();
            if has_warnings {
                warnings.push("warning".to_string());
            }
            EffortSizeBasis {
                total_lines: 1000,
                authored_lines: 800,
                generated_lines: 100,
                vendored_lines: 100,
                kloc_total: 1.0,
                kloc_authored: 0.8,
                generated_pct: gen_pct,
                vendored_pct: vend_pct,
                warnings,
                classification_confidence: conf,
                by_tag: vec![],
            }
        })
}

fn gen_derived_report() -> impl Strategy<Value = DerivedReport> {
    (0..=10usize, 0..=1000usize, 0..=100usize).prop_map(|(lang_count, prod_lines, test_lines)| {
        DerivedReport {
            test_density: TestDensityReport {
                test_lines,
                prod_lines,
                test_files: 0,
                prod_files: 0,
                ratio: if prod_lines > 0 {
                    test_lines as f64 / prod_lines as f64
                } else {
                    0.0
                },
            },
            polyglot: PolyglotReport {
                lang_count,
                entropy: 0.0,
                dominant_lang: "Rust".to_string(),
                dominant_lines: 100,
                dominant_pct: 1.0,
            },
            totals: DerivedTotals {
                code: 0,
                tokens: 0,
                files: 0,
                lines: 0,
                bytes: 0,
                comments: 0,
                blanks: 0,
            },
            doc_density: RatioReport {
                total: RatioRow {
                    ratio: 0.0,
                    key: "".into(),
                    numerator: 0,
                    denominator: 0,
                },
                by_lang: vec![],
                by_module: vec![],
            },
            whitespace: RatioReport {
                total: RatioRow {
                    ratio: 0.0,
                    key: "".into(),
                    numerator: 0,
                    denominator: 0,
                },
                by_lang: vec![],
                by_module: vec![],
            },
            verbosity: RateReport {
                total: RateRow {
                    rate: 0.0,
                    key: "".into(),
                    numerator: 0,
                    denominator: 0,
                },
                by_lang: vec![],
                by_module: vec![],
            },
            max_file: MaxFileReport {
                overall: FileStatRow {
                    lines: 0,
                    path: "".into(),
                    module: "".into(),
                    lang: "".into(),
                    code: 0,
                    comments: 0,
                    blanks: 0,
                    bytes: 0,
                    tokens: 0,
                    doc_pct: None,
                    bytes_per_line: None,
                    depth: 0,
                },
                by_lang: vec![],
                by_module: vec![],
            },
            lang_purity: LangPurityReport { rows: vec![] },
            nesting: NestingReport {
                max: 0,
                avg: 0.0,
                by_module: vec![],
            },
            boilerplate: BoilerplateReport {
                infra_lines: 0,
                logic_lines: 0,
                ratio: 0.0,
                infra_langs: vec![],
            },
            distribution: DistributionReport {
                min: 0,
                max: 0,
                mean: 0.0,
                median: 0.0,
                p90: 0.0,
                p99: 0.0,
                count: 0,
                gini: 0.0,
            },
            histogram: vec![],
            top: TopOffenders {
                largest_lines: vec![],
                largest_tokens: vec![],
                largest_bytes: vec![],
                least_documented: vec![],
                most_dense: vec![],
            },
            tree: None,
            reading_time: ReadingTimeReport {
                minutes: 0.0,
                basis_lines: 0,
                lines_per_minute: 0,
            },
            context_window: None,
            cocomo: None,
            todo: None,
            integrity: IntegrityReport {
                algo: "".to_string(),
                hash: "".to_string(),
                entries: 0,
            },
        }
    })
}

fn gen_git_report() -> impl Strategy<Value = GitReport> {
    (0.0..=1.0f64, any::<bool>()).prop_map(|(stale_pct, has_hotspots)| {
        let hotspots = if has_hotspots {
            vec![tokmd_analysis_types::HotspotRow {
                path: "src/main.rs".to_string(),
                commits: 10,
                lines: 100,
                score: 50,
            }]
        } else {
            vec![]
        };
        GitReport {
            commits_scanned: 10,
            files_seen: 5,
            freshness: FreshnessReport {
                threshold_days: 30,
                stale_files: 2,
                total_files: 10,
                stale_pct,
                by_module: vec![],
            },
            hotspots,
            coupling: vec![],
            bus_factor: vec![],
            age_distribution: None,
            intent: None,
        }
    })
}

fn gen_complexity_report() -> impl Strategy<Value = ComplexityReport> {
    (0.0..=100.0f64, 0..=100usize).prop_map(|(avg, max)| ComplexityReport {
        avg_cyclomatic: avg,
        max_cyclomatic: max,
        total_functions: 10,
        avg_function_length: 10.0,
        max_function_length: 50,
        avg_cognitive: None,
        max_cognitive: None,
        avg_nesting_depth: None,
        max_nesting_depth: None,
        high_risk_files: 0,
        histogram: None,
        halstead: None,
        maintainability_index: None,
        technical_debt: None,
        files: vec![],
    })
}

proptest! {
    #[test]
    fn confidence_score_is_bounded_and_consistent(
        basis in gen_size_basis(),
        derived in gen_derived_report(),
        delta_present in any::<bool>(),
    ) {
        let (conf, score) = build_confidence(&basis, &derived, None, None, None, None, delta_present);

        // Invariant 1: Score is bounded
        prop_assert!(score >= 0.0 && score <= 1.0, "Score {} is out of bounds", score);

        // Invariant 2: Level matches score
        let expected_level = if score >= 0.72 {
            EffortConfidenceLevel::High
        } else if score >= 0.45 {
            EffortConfidenceLevel::Medium
        } else {
            EffortConfidenceLevel::Low
        };
        prop_assert_eq!(conf.level, expected_level);

        // Invariant 3: High confidence means no reasons
        if conf.level == EffortConfidenceLevel::High {
            prop_assert!(conf.reasons.is_empty(), "High confidence should have no reasons");
        }
    }

    #[test]
    fn adding_signals_monotonic_increase(
        basis in gen_size_basis(),
        derived in gen_derived_report(),
        git in gen_git_report(),
        complexity in gen_complexity_report(),
        api_surface_ratio in 0.0..=1.0f64,
        dup_bytes in 0..=1000u64,
    ) {
        let api = ApiSurfaceReport {
            public_ratio: api_surface_ratio,
            documented_ratio: 0.5,
            public_items: 10,
            internal_items: 0,
            total_items: 10,
            by_language: std::collections::BTreeMap::new(),
            by_module: vec![],
            top_exporters: vec![],
        };
        let dup = DuplicateReport {
            wasted_bytes: dup_bytes,
            groups: vec![],
            strategy: "strict".to_string(),
            density: None,
            near: None,
        };

        let (_, score_none) = build_confidence(&basis, &derived, None, None, None, None, false);
        let (_, score_all) = build_confidence(&basis, &derived, Some(&git), Some(&complexity), Some(&api), Some(&dup), true);

        // Invariant 4: Providing reports strictly increases or keeps the score equal (monotonicity)
        prop_assert!(score_all >= score_none, "Score decreased from {} to {} when adding signals", score_none, score_all);
    }
}
