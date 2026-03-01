//! Insta snapshot tests for analysis format rendering.
//!
//! Covers: empty receipt, density metrics, COCOMO estimates, JSON output,
//! health warnings, and risk indicators.

use tokmd_analysis_format::{RenderedOutput, render};
use tokmd_analysis_types::*;
use tokmd_types::{AnalysisFormat, ScanStatus, ToolInfo};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn base_receipt() -> AnalysisReceipt {
    AnalysisReceipt {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: ToolInfo {
            name: "tokmd".into(),
            version: "0.0.0-test".into(),
        },
        mode: "analyze".into(),
        status: ScanStatus::Complete,
        warnings: vec![],
        source: AnalysisSource {
            inputs: vec![".".into()],
            export_path: None,
            base_receipt_path: None,
            export_schema_version: None,
            export_generated_at_ms: None,
            base_signature: None,
            module_roots: vec![],
            module_depth: 1,
            children: "collapse".into(),
        },
        args: AnalysisArgsMeta {
            preset: "receipt".into(),
            format: "md".into(),
            window_tokens: None,
            git: None,
            max_files: None,
            max_bytes: None,
            max_file_bytes: None,
            max_commits: None,
            max_commit_files: None,
            import_granularity: "module".into(),
        },
        archetype: None,
        topics: None,
        entropy: None,
        predictive_churn: None,
        corporate_fingerprint: None,
        license: None,
        derived: None,
        assets: None,
        deps: None,
        git: None,
        imports: None,
        dup: None,
        complexity: None,
        api_surface: None,
        fun: None,
    }
}

fn sample_file_stat(path: &str, lang: &str, code: usize) -> FileStatRow {
    FileStatRow {
        path: path.into(),
        module: "src".into(),
        lang: lang.into(),
        code,
        comments: code / 5,
        blanks: code / 10,
        lines: code + code / 5 + code / 10,
        bytes: code * 10,
        tokens: code * 2,
        doc_pct: Some(0.15),
        bytes_per_line: Some(8.0),
        depth: 1,
    }
}

fn sample_derived_with_density() -> DerivedReport {
    DerivedReport {
        totals: DerivedTotals {
            files: 10,
            code: 1200,
            comments: 200,
            blanks: 100,
            lines: 1500,
            bytes: 12000,
            tokens: 3000,
        },
        doc_density: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 200,
                denominator: 1400,
                ratio: 0.1429,
            },
            by_lang: vec![
                RatioRow {
                    key: "Rust".into(),
                    numerator: 150,
                    denominator: 1050,
                    ratio: 0.1429,
                },
                RatioRow {
                    key: "Python".into(),
                    numerator: 50,
                    denominator: 350,
                    ratio: 0.1429,
                },
            ],
            by_module: vec![],
        },
        whitespace: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 100,
                denominator: 1500,
                ratio: 0.0667,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        verbosity: RateReport {
            total: RateRow {
                key: "total".into(),
                numerator: 12000,
                denominator: 1500,
                rate: 8.0,
            },
            by_lang: vec![RateRow {
                key: "Rust".into(),
                numerator: 9000,
                denominator: 1100,
                rate: 8.18,
            }],
            by_module: vec![],
        },
        max_file: MaxFileReport {
            overall: sample_file_stat("src/main.rs", "Rust", 300),
            by_lang: vec![],
            by_module: vec![],
        },
        lang_purity: LangPurityReport { rows: vec![] },
        nesting: NestingReport {
            max: 5,
            avg: 2.5,
            by_module: vec![],
        },
        test_density: TestDensityReport {
            test_lines: 200,
            prod_lines: 1000,
            test_files: 3,
            prod_files: 7,
            ratio: 0.2,
        },
        boilerplate: BoilerplateReport {
            infra_lines: 80,
            logic_lines: 1420,
            ratio: 0.0563,
            infra_langs: vec!["TOML".into()],
        },
        polyglot: PolyglotReport {
            lang_count: 2,
            entropy: 0.65,
            dominant_lang: "Rust".into(),
            dominant_lines: 900,
            dominant_pct: 0.75,
        },
        distribution: DistributionReport {
            count: 10,
            min: 20,
            max: 390,
            mean: 150.0,
            median: 120.0,
            p90: 300.0,
            p99: 390.0,
            gini: 0.35,
        },
        histogram: vec![
            HistogramBucket {
                label: "Small".into(),
                min: 0,
                max: Some(100),
                files: 4,
                pct: 0.4,
            },
            HistogramBucket {
                label: "Medium".into(),
                min: 101,
                max: Some(500),
                files: 6,
                pct: 0.6,
            },
        ],
        top: TopOffenders {
            largest_lines: vec![sample_file_stat("src/main.rs", "Rust", 300)],
            largest_tokens: vec![sample_file_stat("src/lib.rs", "Rust", 250)],
            largest_bytes: vec![],
            least_documented: vec![],
            most_dense: vec![],
        },
        tree: None,
        reading_time: ReadingTimeReport {
            minutes: 75.0,
            lines_per_minute: 20,
            basis_lines: 1500,
        },
        context_window: None,
        cocomo: None,
        todo: None,
        integrity: IntegrityReport {
            algo: "blake3".into(),
            hash: "a".repeat(64),
            entries: 10,
        },
    }
}

fn render_text(receipt: &AnalysisReceipt, format: AnalysisFormat) -> String {
    match render(receipt, format).expect("render failed") {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text output"),
    }
}

// ---------------------------------------------------------------------------
// 1. Empty analysis receipt → markdown output
// ---------------------------------------------------------------------------

#[test]
fn empty_receipt_md() {
    let receipt = base_receipt();
    let md = render_text(&receipt, AnalysisFormat::Md);
    insta::assert_snapshot!(md);
}

// ---------------------------------------------------------------------------
// 2. Analysis receipt with density metrics → markdown output
// ---------------------------------------------------------------------------

#[test]
fn density_metrics_md() {
    let mut receipt = base_receipt();
    receipt.derived = Some(sample_derived_with_density());
    let md = render_text(&receipt, AnalysisFormat::Md);
    insta::assert_snapshot!(md);
}

// ---------------------------------------------------------------------------
// 3. Analysis receipt with COCOMO estimates → markdown output
// ---------------------------------------------------------------------------

#[test]
fn cocomo_estimates_md() {
    let mut receipt = base_receipt();
    let mut derived = sample_derived_with_density();
    derived.cocomo = Some(CocomoReport {
        mode: "organic".into(),
        kloc: 1.2,
        effort_pm: 3.14,
        duration_months: 2.5,
        staff: 1.26,
        a: 2.4,
        b: 1.05,
        c: 2.5,
        d: 0.38,
    });
    receipt.derived = Some(derived);
    let md = render_text(&receipt, AnalysisFormat::Md);
    insta::assert_snapshot!(md);
}

// ---------------------------------------------------------------------------
// 4. Analysis receipt → JSON output
// ---------------------------------------------------------------------------

#[test]
fn receipt_json_output() {
    let mut receipt = base_receipt();
    receipt.derived = Some(sample_derived_with_density());
    receipt.archetype = Some(Archetype {
        kind: "cli-tool".into(),
        evidence: vec!["Cargo.toml".into(), "src/main.rs".into()],
    });
    let json = render_text(&receipt, AnalysisFormat::Json);
    insta::assert_snapshot!(json);
}

// ---------------------------------------------------------------------------
// 5. Analysis receipt with health warnings
// ---------------------------------------------------------------------------

#[test]
fn health_warnings_md() {
    let mut receipt = base_receipt();
    receipt.warnings = vec![
        "Large file detected: src/generated.rs (5000 lines)".into(),
        "Low test coverage: 5.2%".into(),
        "High boilerplate ratio: 42.0%".into(),
    ];
    let mut derived = sample_derived_with_density();
    derived.todo = Some(TodoReport {
        total: 15,
        density_per_kloc: 12.5,
        tags: vec![
            TodoTagRow {
                tag: "TODO".into(),
                count: 10,
            },
            TodoTagRow {
                tag: "FIXME".into(),
                count: 3,
            },
            TodoTagRow {
                tag: "HACK".into(),
                count: 2,
            },
        ],
    });
    receipt.derived = Some(derived);
    let md = render_text(&receipt, AnalysisFormat::Md);
    insta::assert_snapshot!(md);
}

// ---------------------------------------------------------------------------
// 6. Analysis receipt with risk indicators
// ---------------------------------------------------------------------------

#[test]
fn risk_indicators_md() {
    let mut receipt = base_receipt();
    receipt.entropy = Some(EntropyReport {
        suspects: vec![EntropyFinding {
            path: "secrets/api_key.env".into(),
            module: "secrets".into(),
            entropy_bits_per_byte: 7.2,
            sample_bytes: 256,
            class: EntropyClass::High,
        }],
    });
    receipt.complexity = Some(ComplexityReport {
        total_functions: 42,
        avg_function_length: 18.5,
        max_function_length: 120,
        avg_cyclomatic: 4.3,
        max_cyclomatic: 25,
        avg_cognitive: Some(6.1),
        max_cognitive: Some(32),
        avg_nesting_depth: Some(2.8),
        max_nesting_depth: Some(7),
        high_risk_files: 3,
        histogram: None,
        halstead: None,
        maintainability_index: None,
        technical_debt: None,
        files: vec![FileComplexity {
            path: "src/parser.rs".into(),
            module: "src".into(),
            function_count: 12,
            max_function_length: 120,
            cyclomatic_complexity: 25,
            cognitive_complexity: Some(32),
            max_nesting: Some(7),
            risk_level: ComplexityRisk::Critical,
            functions: None,
        }],
    });
    receipt.git = Some(GitReport {
        commits_scanned: 500,
        files_seen: 80,
        hotspots: vec![HotspotRow {
            path: "src/parser.rs".into(),
            commits: 45,
            lines: 800,
            score: 36000,
        }],
        bus_factor: vec![BusFactorRow {
            module: "src".into(),
            authors: 1,
        }],
        freshness: FreshnessReport {
            threshold_days: 90,
            stale_files: 12,
            total_files: 80,
            stale_pct: 0.15,
            by_module: vec![ModuleFreshnessRow {
                module: "src".into(),
                avg_days: 45.5,
                p90_days: 120.0,
                stale_pct: 0.1,
            }],
        },
        coupling: vec![],
        age_distribution: None,
        intent: None,
    });
    let md = render_text(&receipt, AnalysisFormat::Md);
    insta::assert_snapshot!(md);
}
