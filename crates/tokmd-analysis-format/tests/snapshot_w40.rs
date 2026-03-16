use std::collections::BTreeMap;

use tokmd_analysis_format::{RenderedOutput, render};
use tokmd_analysis_types::{
    ANALYSIS_SCHEMA_VERSION, AnalysisArgsMeta, AnalysisReceipt, AnalysisSource, Archetype,
    BoilerplateReport, DerivedReport, DerivedTotals, DistributionReport, FileStatRow,
    HistogramBucket, IntegrityReport, LangPurityReport, MaxFileReport, NestingReport,
    PolyglotReport, RateReport, RateRow, RatioReport, RatioRow, ReadingTimeReport,
    TestDensityReport, TopOffenders, TopicClouds, TopicTerm,
};
use tokmd_types::{AnalysisFormat, ScanStatus, ToolInfo};

fn fixed_tool() -> ToolInfo {
    ToolInfo {
        name: "tokmd".to_string(),
        version: "0.0.0-test".to_string(),
    }
}

fn minimal_source() -> AnalysisSource {
    AnalysisSource {
        inputs: vec![".".to_string()],
        export_path: None,
        base_receipt_path: None,
        export_schema_version: None,
        export_generated_at_ms: None,
        base_signature: None,
        module_roots: vec![],
        module_depth: 1,
        children: "collapse".to_string(),
    }
}

fn minimal_args() -> AnalysisArgsMeta {
    AnalysisArgsMeta {
        preset: "receipt".to_string(),
        format: "md".to_string(),
        window_tokens: None,
        git: None,
        max_files: None,
        max_bytes: None,
        max_commits: None,
        max_commit_files: None,
        max_file_bytes: None,
        import_granularity: "module".to_string(),
    }
}

fn minimal_receipt() -> AnalysisReceipt {
    AnalysisReceipt {
        effort: None,
        schema_version: ANALYSIS_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: fixed_tool(),
        mode: "analyze".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        source: minimal_source(),
        args: minimal_args(),
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

fn stub_file_stat() -> FileStatRow {
    FileStatRow {
        path: "src/lib.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        code: 500,
        comments: 80,
        blanks: 40,
        lines: 620,
        bytes: 18000,
        tokens: 1250,
        doc_pct: Some(0.16),
        bytes_per_line: Some(29.03),
        depth: 1,
    }
}

fn sample_derived() -> DerivedReport {
    DerivedReport {
        totals: DerivedTotals {
            files: 10,
            code: 2000,
            comments: 300,
            blanks: 200,
            lines: 2500,
            bytes: 75000,
            tokens: 5000,
        },
        doc_density: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 300,
                denominator: 2000,
                ratio: 0.15,
            },
            by_lang: vec![RatioRow {
                key: "Rust".into(),
                numerator: 300,
                denominator: 2000,
                ratio: 0.15,
            }],
            by_module: vec![],
        },
        whitespace: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 200,
                denominator: 2300,
                ratio: 0.087,
            },
            by_lang: vec![RatioRow {
                key: "Rust".into(),
                numerator: 200,
                denominator: 2300,
                ratio: 0.087,
            }],
            by_module: vec![],
        },
        verbosity: RateReport {
            total: RateRow {
                key: "total".into(),
                numerator: 75000,
                denominator: 2500,
                rate: 30.0,
            },
            by_lang: vec![RateRow {
                key: "Rust".into(),
                numerator: 75000,
                denominator: 2500,
                rate: 30.0,
            }],
            by_module: vec![],
        },
        max_file: MaxFileReport {
            overall: stub_file_stat(),
            by_lang: vec![],
            by_module: vec![],
        },
        lang_purity: LangPurityReport { rows: vec![] },
        nesting: NestingReport {
            max: 3,
            avg: 1.5,
            by_module: vec![],
        },
        test_density: TestDensityReport {
            test_lines: 400,
            prod_lines: 2100,
            test_files: 3,
            prod_files: 7,
            ratio: 0.19,
        },
        boilerplate: BoilerplateReport {
            infra_lines: 200,
            logic_lines: 1800,
            ratio: 0.10,
            infra_langs: vec!["TOML".into()],
        },
        polyglot: PolyglotReport {
            lang_count: 2,
            entropy: 0.45,
            dominant_lang: "Rust".into(),
            dominant_lines: 1800,
            dominant_pct: 0.90,
        },
        distribution: DistributionReport {
            count: 10,
            min: 20,
            max: 500,
            mean: 200.0,
            median: 180.0,
            p90: 450.0,
            p99: 500.0,
            gini: 0.35,
        },
        histogram: vec![
            HistogramBucket {
                label: "0–100".into(),
                min: 0,
                max: Some(100),
                files: 4,
                pct: 0.40,
            },
            HistogramBucket {
                label: "101–500".into(),
                min: 101,
                max: Some(500),
                files: 6,
                pct: 0.60,
            },
        ],
        top: TopOffenders {
            largest_lines: vec![stub_file_stat()],
            largest_tokens: vec![stub_file_stat()],
            largest_bytes: vec![stub_file_stat()],
            least_documented: vec![stub_file_stat()],
            most_dense: vec![stub_file_stat()],
        },
        tree: None,
        reading_time: ReadingTimeReport {
            minutes: 12.5,
            lines_per_minute: 200,
            basis_lines: 2500,
        },
        context_window: None,
        cocomo: None,
        todo: None,
        integrity: IntegrityReport {
            algo: "blake3".into(),
            hash: "abc123def456".into(),
            entries: 10,
        },
    }
}

// ── Markdown: minimal receipt ─────────────────────────────────────────

#[test]
fn snapshot_analysis_md_minimal() {
    let receipt = minimal_receipt();
    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = match output {
        RenderedOutput::Text(t) => t,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    insta::assert_snapshot!(text);
}

// ── JSON: all optional fields populated ───────────────────────────────

#[test]
fn snapshot_analysis_json_populated() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    receipt.archetype = Some(Archetype {
        kind: "cli-tool".into(),
        evidence: vec!["clap".into(), "main.rs".into()],
    });

    let output = render(&receipt, AnalysisFormat::Json).unwrap();
    let json_str = match output {
        RenderedOutput::Text(t) => t,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    let mut v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    // Normalize dynamic fields
    v["generated_at_ms"] = serde_json::json!(0);
    v["tool"]["version"] = serde_json::json!("0.0.0-test");
    insta::assert_json_snapshot!(v);
}

// ── Markdown: nested enricher data ────────────────────────────────────

#[test]
fn snapshot_analysis_md_with_enrichers() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    receipt.archetype = Some(Archetype {
        kind: "web-app".into(),
        evidence: vec!["package.json".into(), "index.html".into()],
    });
    receipt.topics = Some(TopicClouds {
        overall: vec![
            TopicTerm {
                term: "parsing".into(),
                score: 0.85,
                tf: 12,
                df: 3,
            },
            TopicTerm {
                term: "analysis".into(),
                score: 0.72,
                tf: 8,
                df: 2,
            },
        ],
        per_module: {
            let mut m = BTreeMap::new();
            m.insert(
                "src".to_string(),
                vec![TopicTerm {
                    term: "render".into(),
                    score: 0.60,
                    tf: 5,
                    df: 1,
                }],
            );
            m
        },
    });

    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = match output {
        RenderedOutput::Text(t) => t,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    insta::assert_snapshot!(text);
}

// ── Markdown: tree format ─────────────────────────────────────────────

#[test]
fn snapshot_analysis_tree_minimal() {
    let receipt = minimal_receipt();
    let output = render(&receipt, AnalysisFormat::Tree).unwrap();
    let text = match output {
        RenderedOutput::Text(t) => t,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    insta::assert_snapshot!(text);
}

// ── XML format ────────────────────────────────────────────────────────

#[test]
fn snapshot_analysis_xml_minimal() {
    let receipt = minimal_receipt();
    let output = render(&receipt, AnalysisFormat::Xml).unwrap();
    let text = match output {
        RenderedOutput::Text(t) => t,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    // Normalize dynamic fields in XML
    let normalized = text.replace(
        &format!("\"generated_at_ms\":{}", receipt.generated_at_ms),
        "\"generated_at_ms\":0",
    );
    insta::assert_snapshot!(normalized);
}
