//! # tokmd-analysis-fun
//!
//! Novelty/enrichment computations for analysis receipts.
//!
//! This crate intentionally contains *non-core* analysis enrichments so they can be
//! evolved independently from the main `tokmd-analysis` orchestration crate.
//! In practice this currently supports the eco-label generator used by `AnalysisPreset::Fun`.

use tokmd_analysis_types::{DerivedReport, EcoLabel, FunReport};

/// Build the fun/eco-label portion of an analysis receipt.
pub fn build_fun_report(derived: &DerivedReport) -> FunReport {
    let bytes = derived.totals.bytes as u64;
    let mb = bytes as f64 / (1024.0 * 1024.0);
    let (label, score) = fun_band(mb);

    FunReport {
        eco_label: Some(EcoLabel {
            score,
            label: label.to_string(),
            bytes,
            notes: format!("Size-based eco label ({} MB)", round_to_two(mb)),
        }),
    }
}

fn fun_band(mb: f64) -> (&'static str, f64) {
    if mb <= 1.0 {
        ("A", 95.0)
    } else if mb <= 10.0 {
        ("B", 80.0)
    } else if mb <= 50.0 {
        ("C", 65.0)
    } else if mb <= 200.0 {
        ("D", 45.0)
    } else {
        ("E", 30.0)
    }
}

fn round_to_two(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::{build_fun_report, fun_band};
    use tokmd_analysis_types::{
        BoilerplateReport, DerivedReport, DerivedTotals, DistributionReport, FileStatRow,
        IntegrityReport, LangPurityReport, MaxFileReport, NestingReport, PolyglotReport,
        RateReport, RateRow, RatioReport, RatioRow, ReadingTimeReport, TestDensityReport,
        TodoReport, TopOffenders,
    };

    fn tiny_derived(bytes: usize) -> DerivedReport {
        let zero_row = FileStatRow {
            path: "small.rs".to_string(),
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
                    key: "All".to_string(),
                    numerator: 0,
                    denominator: 1,
                    ratio: 0.0,
                },
                by_lang: vec![],
                by_module: vec![],
            },
            whitespace: RatioReport {
                total: RatioRow {
                    key: "All".to_string(),
                    numerator: 0,
                    denominator: 1,
                    ratio: 0.0,
                },
                by_lang: vec![],
                by_module: vec![],
            },
            verbosity: RateReport {
                total: RateRow {
                    key: "All".to_string(),
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

    #[test]
    fn fun_grade_boundaries_are_stable() {
        assert_eq!(fun_band(0.5), ("A", 95.0));
        assert_eq!(fun_band(10.0), ("B", 80.0));
        assert_eq!(fun_band(50.0), ("C", 65.0));
        assert_eq!(fun_band(200.0), ("D", 45.0));
        assert_eq!(fun_band(200.1), ("E", 30.0));
    }

    #[test]
    fn fun_report_contains_notes_and_bytes() {
        let bytes = 1024 * 1024;
        let report = build_fun_report(&tiny_derived(bytes));
        let eco = report.eco_label.expect("eco_label expected");

        assert_eq!(eco.label, "A");
        assert_eq!(eco.score, 95.0);
        assert_eq!(eco.bytes, bytes as u64);
        assert_eq!(eco.notes, "Size-based eco label (1 MB)");
    }
}
