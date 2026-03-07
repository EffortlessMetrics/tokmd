//! Effort estimation scaffolding for tokmd analysis receipts.

use std::collections::BTreeMap;

use tokmd_analysis_types::{
    DerivedReport, EffortAssumptions, EffortConfidence, EffortConfidenceLevel,
    EffortEstimateReport, EffortModel, EffortResults, EffortSizeBasis,
};

/// Build a scaffold effort estimation report from derived metrics.
///
/// This is a placeholder that returns minimal/zeroed values.
/// Real model sophistication comes in later PRs.
pub fn build_effort_report(derived: &DerivedReport) -> EffortEstimateReport {
    let total = derived.totals.code;
    let kloc = total as f64 / 1000.0;

    EffortEstimateReport {
        model: EffortModel::Cocomo81Basic,
        size_basis: EffortSizeBasis {
            total_lines: total,
            authored_lines: total,
            generated_lines: 0,
            vendored_lines: 0,
            kloc_total: kloc,
            kloc_authored: kloc,
            generated_pct: 0.0,
            vendored_pct: 0.0,
            classification_confidence: EffortConfidenceLevel::Low,
            warnings: vec!["scaffold estimate — no classification applied".into()],
            by_tag: Vec::new(),
        },
        results: EffortResults {
            effort_pm_low: 0.0,
            effort_pm_p50: 0.0,
            effort_pm_p80: 0.0,
            schedule_months_low: 0.0,
            schedule_months_p50: 0.0,
            schedule_months_p80: 0.0,
            staff_low: 0.0,
            staff_p50: 0.0,
            staff_p80: 0.0,
        },
        confidence: EffortConfidence {
            level: EffortConfidenceLevel::Low,
            reasons: vec!["scaffold only — no real model applied".into()],
            data_coverage_pct: None,
        },
        drivers: Vec::new(),
        assumptions: EffortAssumptions {
            notes: vec!["scaffold estimate — values are placeholders".into()],
            overrides: BTreeMap::new(),
        },
        delta: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokmd_analysis_types::*;

    #[test]
    fn scaffold_report_has_correct_model() {
        let derived = make_test_derived(1000);
        let report = build_effort_report(&derived);
        assert_eq!(report.model, EffortModel::Cocomo81Basic);
    }

    #[test]
    fn scaffold_report_size_basis_matches_input() {
        let derived = make_test_derived(5000);
        let report = build_effort_report(&derived);
        assert_eq!(report.size_basis.total_lines, 5000);
        assert_eq!(report.size_basis.authored_lines, 5000);
        assert_eq!(report.size_basis.generated_lines, 0);
        assert_eq!(report.size_basis.vendored_lines, 0);
        assert!((report.size_basis.kloc_total - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn scaffold_report_confidence_is_low() {
        let derived = make_test_derived(1000);
        let report = build_effort_report(&derived);
        assert_eq!(report.confidence.level, EffortConfidenceLevel::Low);
        assert!(!report.confidence.reasons.is_empty());
    }

    #[test]
    fn scaffold_report_results_are_zeroed() {
        let derived = make_test_derived(1000);
        let report = build_effort_report(&derived);
        assert_eq!(report.results.effort_pm_low, 0.0);
        assert_eq!(report.results.effort_pm_p50, 0.0);
        assert_eq!(report.results.effort_pm_p80, 0.0);
        assert_eq!(report.results.schedule_months_low, 0.0);
        assert_eq!(report.results.staff_low, 0.0);
    }

    #[test]
    fn scaffold_report_serde_roundtrip() {
        let derived = make_test_derived(2000);
        let report = build_effort_report(&derived);
        let json = serde_json::to_string(&report).unwrap();
        let back: EffortEstimateReport = serde_json::from_str(&json).unwrap();
        assert_eq!(back.model, EffortModel::Cocomo81Basic);
        assert_eq!(back.size_basis.total_lines, 2000);
    }

    #[test]
    fn effort_model_enum_serde() {
        for model in [
            EffortModel::Cocomo81Basic,
            EffortModel::Cocomo2Early,
            EffortModel::Ensemble,
        ] {
            let json = serde_json::to_string(&model).unwrap();
            let back: EffortModel = serde_json::from_str(&json).unwrap();
            assert_eq!(back, model);
        }
    }

    #[test]
    fn effort_confidence_level_serde() {
        for level in [
            EffortConfidenceLevel::Low,
            EffortConfidenceLevel::Medium,
            EffortConfidenceLevel::High,
        ] {
            let json = serde_json::to_string(&level).unwrap();
            let back: EffortConfidenceLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(back, level);
        }
    }

    #[test]
    fn effort_driver_direction_serde() {
        for dir in [
            EffortDriverDirection::Raises,
            EffortDriverDirection::Lowers,
            EffortDriverDirection::Neutral,
        ] {
            let json = serde_json::to_string(&dir).unwrap();
            let back: EffortDriverDirection = serde_json::from_str(&json).unwrap();
            assert_eq!(back, dir);
        }
    }

    fn make_test_derived(code_lines: usize) -> DerivedReport {
        let ratio_zero = RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 0,
                denominator: code_lines,
                ratio: 0.0,
            },
            by_lang: vec![],
            by_module: vec![],
        };
        let rate_zero = RateReport {
            total: RateRow {
                key: "total".into(),
                numerator: 0,
                denominator: code_lines,
                rate: 0.0,
            },
            by_lang: vec![],
            by_module: vec![],
        };
        DerivedReport {
            totals: DerivedTotals {
                files: 10,
                code: code_lines,
                comments: 100,
                blanks: 50,
                lines: code_lines + 150,
                bytes: code_lines * 40,
                tokens: code_lines * 3,
            },
            doc_density: ratio_zero.clone(),
            whitespace: ratio_zero,
            verbosity: rate_zero,
            max_file: MaxFileReport {
                overall: FileStatRow {
                    path: "src/main.rs".into(),
                    module: "src".into(),
                    lang: "Rust".into(),
                    code: code_lines,
                    comments: 0,
                    blanks: 0,
                    lines: code_lines,
                    bytes: code_lines * 40,
                    tokens: code_lines * 3,
                    doc_pct: None,
                    bytes_per_line: Some(40.0),
                    depth: 1,
                },
                by_lang: vec![],
                by_module: vec![],
            },
            lang_purity: LangPurityReport { rows: vec![] },
            nesting: NestingReport {
                max: 1,
                avg: 1.0,
                by_module: vec![],
            },
            test_density: TestDensityReport {
                test_lines: 0,
                prod_lines: code_lines,
                test_files: 0,
                prod_files: 10,
                ratio: 0.0,
            },
            boilerplate: BoilerplateReport {
                infra_lines: 0,
                logic_lines: code_lines,
                ratio: 0.0,
                infra_langs: vec![],
            },
            polyglot: PolyglotReport {
                lang_count: 1,
                entropy: 0.0,
                dominant_lang: "Rust".into(),
                dominant_lines: code_lines,
                dominant_pct: 1.0,
            },
            distribution: DistributionReport {
                count: 10,
                min: 10,
                max: code_lines,
                mean: code_lines as f64 / 10.0,
                median: code_lines as f64 / 10.0,
                p90: code_lines as f64,
                p99: code_lines as f64,
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
                minutes: 1.0,
                lines_per_minute: 200,
                basis_lines: code_lines,
            },
            context_window: None,
            cocomo: None,
            todo: None,
            integrity: IntegrityReport {
                algo: "blake3".into(),
                hash: "test".into(),
                entries: 10,
            },
        }
    }
}
