//! Golden snapshot tests for analysis format rendering (W74).
//!
//! Comprehensive insta snapshot coverage for Markdown, JSON, XML, SVG,
//! Mermaid, Tree, JSON-LD, and HTML analysis output formats.

use std::collections::BTreeMap;

use tokmd_analysis_format::{RenderedOutput, render};
use tokmd_analysis_types::*;
use tokmd_types::{AnalysisFormat, ScanStatus, ToolInfo};

// ===========================================================================
// Helpers
// ===========================================================================

fn minimal_receipt() -> AnalysisReceipt {
    AnalysisReceipt {
        effort: None,
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
            format: "json".into(),
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

fn sample_derived() -> DerivedReport {
    DerivedReport {
        totals: DerivedTotals {
            files: 12,
            code: 2400,
            comments: 310,
            blanks: 160,
            lines: 2870,
            bytes: 72_000,
            tokens: 19_200,
        },
        doc_density: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 310,
                denominator: 2710,
                ratio: 0.1143,
            },
            by_lang: vec![
                RatioRow {
                    key: "Rust".into(),
                    numerator: 250,
                    denominator: 2100,
                    ratio: 0.1190,
                },
                RatioRow {
                    key: "TOML".into(),
                    numerator: 60,
                    denominator: 610,
                    ratio: 0.0983,
                },
            ],
            by_module: vec![],
        },
        whitespace: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 160,
                denominator: 2870,
                ratio: 0.0557,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        verbosity: RateReport {
            total: RateRow {
                key: "total".into(),
                numerator: 72_000,
                denominator: 2870,
                rate: 25.08,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        max_file: MaxFileReport {
            overall: FileStatRow {
                path: "src/core.rs".into(),
                module: "src".into(),
                lang: "Rust".into(),
                code: 520,
                comments: 75,
                blanks: 40,
                lines: 635,
                bytes: 15_600,
                tokens: 4_160,
                doc_pct: Some(0.126),
                bytes_per_line: Some(24.56),
                depth: 1,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        lang_purity: LangPurityReport { rows: vec![] },
        nesting: NestingReport {
            max: 6,
            avg: 2.8,
            by_module: vec![],
        },
        test_density: TestDensityReport {
            test_lines: 350,
            prod_lines: 2050,
            test_files: 4,
            prod_files: 8,
            ratio: 0.1707,
        },
        boilerplate: BoilerplateReport {
            infra_lines: 120,
            logic_lines: 2750,
            ratio: 0.0436,
            infra_langs: vec!["TOML".into(), "YAML".into()],
        },
        polyglot: PolyglotReport {
            lang_count: 3,
            entropy: 0.78,
            dominant_lang: "Rust".into(),
            dominant_lines: 2100,
            dominant_pct: 0.73,
        },
        distribution: DistributionReport {
            count: 12,
            min: 45,
            max: 635,
            mean: 239.16,
            median: 195.0,
            p90: 520.0,
            p99: 635.0,
            gini: 0.32,
        },
        histogram: vec![
            HistogramBucket {
                label: "Small".into(),
                min: 0,
                max: Some(100),
                files: 5,
                pct: 0.4167,
            },
            HistogramBucket {
                label: "Medium".into(),
                min: 101,
                max: Some(500),
                files: 5,
                pct: 0.4167,
            },
            HistogramBucket {
                label: "Large".into(),
                min: 501,
                max: None,
                files: 2,
                pct: 0.1667,
            },
        ],
        top: TopOffenders {
            largest_lines: vec![FileStatRow {
                path: "src/core.rs".into(),
                module: "src".into(),
                lang: "Rust".into(),
                code: 520,
                comments: 75,
                blanks: 40,
                lines: 635,
                bytes: 15_600,
                tokens: 4_160,
                doc_pct: Some(0.126),
                bytes_per_line: Some(24.56),
                depth: 1,
            }],
            largest_tokens: vec![],
            largest_bytes: vec![],
            least_documented: vec![],
            most_dense: vec![],
        },
        tree: None,
        reading_time: ReadingTimeReport {
            minutes: 143.5,
            lines_per_minute: 20,
            basis_lines: 2870,
        },
        context_window: Some(ContextWindowReport {
            window_tokens: 128_000,
            total_tokens: 19_200,
            pct: 15.0,
            fits: true,
        }),
        cocomo: Some(CocomoReport {
            mode: "organic".into(),
            kloc: 2.4,
            effort_pm: 6.87,
            duration_months: 4.56,
            staff: 1.5,
            a: 2.4,
            b: 1.05,
            c: 2.5,
            d: 0.38,
        }),
        todo: Some(TodoReport {
            total: 8,
            density_per_kloc: 3.33,
            tags: vec![
                TodoTagRow {
                    tag: "TODO".into(),
                    count: 5,
                },
                TodoTagRow {
                    tag: "FIXME".into(),
                    count: 3,
                },
            ],
        }),
        integrity: IntegrityReport {
            algo: "blake3".into(),
            hash: "a".repeat(64),
            entries: 12,
        },
    }
}

fn text(output: RenderedOutput) -> String {
    match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text output"),
    }
}

fn redact_timestamp(html: &str) -> String {
    let mut result = html.to_string();
    while let Some(pos) = result.find(" UTC") {
        if pos >= 19 {
            let candidate = &result[pos - 19..pos + 4];
            if candidate.len() == 23
                && candidate.as_bytes()[4] == b'-'
                && candidate.as_bytes()[7] == b'-'
                && candidate.as_bytes()[10] == b' '
            {
                result.replace_range(pos - 19..pos + 4, "[TIMESTAMP]");
                continue;
            }
        }
        break;
    }
    result
}

// ===========================================================================
// Markdown snapshots (5 tests)
// ===========================================================================

#[test]
fn w74_analysis_md_minimal() {
    let receipt = minimal_receipt();
    let out = text(render(&receipt, AnalysisFormat::Md).unwrap());
    insta::assert_snapshot!("w74_analysis_md_minimal", out);
}

#[test]
fn w74_analysis_md_with_derived() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let out = text(render(&receipt, AnalysisFormat::Md).unwrap());
    insta::assert_snapshot!("w74_analysis_md_with_derived", out);
}

#[test]
fn w74_analysis_md_with_warnings() {
    let mut receipt = minimal_receipt();
    receipt.warnings = vec![
        "truncated at max_files".into(),
        "git history unavailable".into(),
    ];
    let out = text(render(&receipt, AnalysisFormat::Md).unwrap());
    insta::assert_snapshot!("w74_analysis_md_with_warnings", out);
}

#[test]
fn w74_analysis_md_archetype_and_topics() {
    let mut receipt = minimal_receipt();
    receipt.archetype = Some(Archetype {
        kind: "web-app".into(),
        evidence: vec!["package.json".into(), "src/index.tsx".into()],
    });
    receipt.topics = Some(TopicClouds {
        per_module: BTreeMap::new(),
        overall: vec![
            TopicTerm {
                term: "frontend".into(),
                score: 0.91,
                tf: 15,
                df: 4,
            },
            TopicTerm {
                term: "rendering".into(),
                score: 0.68,
                tf: 9,
                df: 3,
            },
        ],
    });
    let out = text(render(&receipt, AnalysisFormat::Md).unwrap());
    insta::assert_snapshot!("w74_analysis_md_archetype_topics", out);
}

#[test]
fn w74_analysis_md_derived_no_cocomo() {
    let mut receipt = minimal_receipt();
    let mut d = sample_derived();
    d.cocomo = None;
    d.context_window = None;
    d.todo = None;
    receipt.derived = Some(d);
    let out = text(render(&receipt, AnalysisFormat::Md).unwrap());
    insta::assert_snapshot!("w74_analysis_md_derived_no_cocomo", out);
}

// ===========================================================================
// JSON snapshots (3 tests)
// ===========================================================================

#[test]
fn w74_analysis_json_minimal() {
    let receipt = minimal_receipt();
    let out = text(render(&receipt, AnalysisFormat::Json).unwrap());
    insta::assert_snapshot!("w74_analysis_json_minimal", out);
}

#[test]
fn w74_analysis_json_with_derived() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let out = text(render(&receipt, AnalysisFormat::Json).unwrap());
    insta::assert_snapshot!("w74_analysis_json_with_derived", out);
}

#[test]
fn w74_analysis_json_eco_label() {
    let mut receipt = minimal_receipt();
    receipt.fun = Some(FunReport {
        eco_label: Some(EcoLabel {
            score: 92.0,
            label: "A+".into(),
            bytes: 150_000,
            notes: "Size-based eco label (0.14 MB)".into(),
        }),
    });
    let out = text(render(&receipt, AnalysisFormat::Json).unwrap());
    insta::assert_snapshot!("w74_analysis_json_eco_label", out);
}

// ===========================================================================
// XML snapshot (1 test)
// ===========================================================================

#[test]
fn w74_analysis_xml_with_derived() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let out = text(render(&receipt, AnalysisFormat::Xml).unwrap());
    insta::assert_snapshot!("w74_analysis_xml_with_derived", out);
}

// ===========================================================================
// SVG snapshot (1 test)
// ===========================================================================

#[test]
fn w74_analysis_svg_with_context_window() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let out = text(render(&receipt, AnalysisFormat::Svg).unwrap());
    insta::assert_snapshot!("w74_analysis_svg_with_context_window", out);
}

// ===========================================================================
// Mermaid snapshot (1 test)
// ===========================================================================

#[test]
fn w74_analysis_mermaid_with_imports() {
    let mut receipt = minimal_receipt();
    receipt.imports = Some(ImportReport {
        granularity: "module".into(),
        edges: vec![
            ImportEdge {
                from: "core".into(),
                to: "types".into(),
                count: 8,
            },
            ImportEdge {
                from: "api".into(),
                to: "core".into(),
                count: 4,
            },
            ImportEdge {
                from: "api".into(),
                to: "types".into(),
                count: 2,
            },
        ],
    });
    let out = text(render(&receipt, AnalysisFormat::Mermaid).unwrap());
    insta::assert_snapshot!("w74_analysis_mermaid_with_imports", out);
}

// ===========================================================================
// Tree snapshot (1 test)
// ===========================================================================

#[test]
fn w74_analysis_tree_with_derived() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let out = text(render(&receipt, AnalysisFormat::Tree).unwrap());
    insta::assert_snapshot!("w74_analysis_tree_with_derived", out);
}

// ===========================================================================
// JSON-LD snapshot (1 test)
// ===========================================================================

#[test]
fn w74_analysis_jsonld_with_derived() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let out = text(render(&receipt, AnalysisFormat::Jsonld).unwrap());
    insta::assert_snapshot!("w74_analysis_jsonld_with_derived", out);
}

// ===========================================================================
// HTML snapshots (2 tests)
// ===========================================================================

#[test]
fn w74_analysis_html_with_derived() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let out = redact_timestamp(&text(render(&receipt, AnalysisFormat::Html).unwrap()));
    insta::assert_snapshot!("w74_analysis_html_with_derived", out);
}

#[test]
fn w74_analysis_html_minimal() {
    let receipt = minimal_receipt();
    let out = redact_timestamp(&text(render(&receipt, AnalysisFormat::Html).unwrap()));
    insta::assert_snapshot!("w74_analysis_html_minimal", out);
}

