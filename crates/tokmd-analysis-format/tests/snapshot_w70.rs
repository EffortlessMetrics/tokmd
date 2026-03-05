//! Golden snapshot tests for analysis format rendering (W70).
//!
//! Captures Markdown, JSON, JSON-LD, XML, SVG, Mermaid, Tree, and HTML
//! output for analysis receipts with various enrichment combinations.

use tokmd_analysis_format::{RenderedOutput, render};
use tokmd_analysis_types::*;
use tokmd_types::{AnalysisFormat, ScanStatus, ToolInfo};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn minimal_receipt() -> AnalysisReceipt {
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
            files: 8,
            code: 1200,
            comments: 150,
            blanks: 80,
            lines: 1430,
            bytes: 12_000,
            tokens: 3_000,
        },
        doc_density: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 150,
                denominator: 1350,
                ratio: 0.1111,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        whitespace: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 80,
                denominator: 1430,
                ratio: 0.0559,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        verbosity: RateReport {
            total: RateRow {
                key: "total".into(),
                numerator: 12_000,
                denominator: 1430,
                rate: 8.39,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        max_file: MaxFileReport {
            overall: FileStatRow {
                path: "src/engine.rs".into(),
                module: "src".into(),
                lang: "Rust".into(),
                code: 400,
                comments: 60,
                blanks: 30,
                lines: 490,
                bytes: 4_000,
                tokens: 1_000,
                doc_pct: Some(0.13),
                bytes_per_line: Some(8.16),
                depth: 1,
            },
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
            prod_files: 5,
            ratio: 0.2,
        },
        boilerplate: BoilerplateReport {
            infra_lines: 80,
            logic_lines: 1350,
            ratio: 0.0592,
            infra_langs: vec!["TOML".into(), "YAML".into()],
        },
        polyglot: PolyglotReport {
            lang_count: 3,
            entropy: 0.85,
            dominant_lang: "Rust".into(),
            dominant_lines: 1100,
            dominant_pct: 0.77,
        },
        distribution: DistributionReport {
            count: 8,
            min: 30,
            max: 490,
            mean: 178.75,
            median: 150.0,
            p90: 490.0,
            p99: 490.0,
            gini: 0.35,
        },
        histogram: vec![
            HistogramBucket {
                label: "Small".into(),
                min: 0,
                max: Some(100),
                files: 4,
                pct: 0.5,
            },
            HistogramBucket {
                label: "Medium".into(),
                min: 101,
                max: Some(500),
                files: 4,
                pct: 0.5,
            },
        ],
        top: TopOffenders {
            largest_lines: vec![FileStatRow {
                path: "src/engine.rs".into(),
                module: "src".into(),
                lang: "Rust".into(),
                code: 400,
                comments: 60,
                blanks: 30,
                lines: 490,
                bytes: 4_000,
                tokens: 1_000,
                doc_pct: Some(0.13),
                bytes_per_line: Some(8.16),
                depth: 1,
            }],
            largest_tokens: vec![],
            largest_bytes: vec![],
            least_documented: vec![],
            most_dense: vec![],
        },
        tree: None,
        reading_time: ReadingTimeReport {
            minutes: 71.5,
            lines_per_minute: 20,
            basis_lines: 1430,
        },
        context_window: None,
        cocomo: None,
        todo: None,
        integrity: IntegrityReport {
            algo: "blake3".into(),
            hash: "b".repeat(64),
            entries: 8,
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
// JSON snapshots
// ===========================================================================

#[test]
fn w70_analysis_json_minimal() {
    let receipt = minimal_receipt();
    let out = text(render(&receipt, AnalysisFormat::Json).unwrap());
    insta::assert_snapshot!("w70_analysis_json_minimal", out);
}

#[test]
fn w70_analysis_json_with_derived() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let out = text(render(&receipt, AnalysisFormat::Json).unwrap());
    insta::assert_snapshot!("w70_analysis_json_with_derived", out);
}

#[test]
fn w70_analysis_json_archetype_and_eco() {
    let mut receipt = minimal_receipt();
    receipt.archetype = Some(Archetype {
        kind: "cli-tool".into(),
        evidence: vec!["src/main.rs".into(), "Cargo.toml".into()],
    });
    receipt.fun = Some(FunReport {
        eco_label: Some(EcoLabel {
            score: 88.0,
            label: "A".into(),
            bytes: 300_000,
            notes: "Size-based eco label (0.29 MB)".into(),
        }),
    });
    let out = text(render(&receipt, AnalysisFormat::Json).unwrap());
    insta::assert_snapshot!("w70_analysis_json_archetype_eco", out);
}

// ===========================================================================
// Markdown snapshots
// ===========================================================================

#[test]
fn w70_analysis_md_minimal() {
    let receipt = minimal_receipt();
    let out = text(render(&receipt, AnalysisFormat::Md).unwrap());
    insta::assert_snapshot!("w70_analysis_md_minimal", out);
}

#[test]
fn w70_analysis_md_with_derived() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let out = text(render(&receipt, AnalysisFormat::Md).unwrap());
    insta::assert_snapshot!("w70_analysis_md_with_derived", out);
}

#[test]
fn w70_analysis_md_with_warnings() {
    let mut receipt = minimal_receipt();
    receipt.warnings = vec!["scan timed out".into(), "some files skipped".into()];
    let out = text(render(&receipt, AnalysisFormat::Md).unwrap());
    insta::assert_snapshot!("w70_analysis_md_with_warnings", out);
}

#[test]
fn w70_analysis_md_archetype_topics() {
    let mut receipt = minimal_receipt();
    receipt.archetype = Some(Archetype {
        kind: "monorepo".into(),
        evidence: vec!["Cargo.toml".into(), "package.json".into()],
    });
    receipt.topics = Some(TopicClouds {
        per_module: std::collections::BTreeMap::new(),
        overall: vec![
            TopicTerm {
                term: "systems".into(),
                score: 0.85,
                tf: 12,
                df: 3,
            },
            TopicTerm {
                term: "cli".into(),
                score: 0.72,
                tf: 8,
                df: 2,
            },
        ],
    });
    let out = text(render(&receipt, AnalysisFormat::Md).unwrap());
    insta::assert_snapshot!("w70_analysis_md_archetype_topics", out);
}

// ===========================================================================
// JSON-LD snapshot
// ===========================================================================

#[test]
fn w70_analysis_jsonld_with_derived() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let out = text(render(&receipt, AnalysisFormat::Jsonld).unwrap());
    insta::assert_snapshot!("w70_analysis_jsonld_with_derived", out);
}

// ===========================================================================
// XML snapshots
// ===========================================================================

#[test]
fn w70_analysis_xml_minimal() {
    let receipt = minimal_receipt();
    let out = text(render(&receipt, AnalysisFormat::Xml).unwrap());
    insta::assert_snapshot!("w70_analysis_xml_minimal", out);
}

#[test]
fn w70_analysis_xml_with_derived() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let out = text(render(&receipt, AnalysisFormat::Xml).unwrap());
    insta::assert_snapshot!("w70_analysis_xml_with_derived", out);
}

// ===========================================================================
// SVG snapshots
// ===========================================================================

#[test]
fn w70_analysis_svg_minimal() {
    let receipt = minimal_receipt();
    let out = text(render(&receipt, AnalysisFormat::Svg).unwrap());
    insta::assert_snapshot!("w70_analysis_svg_minimal", out);
}

#[test]
fn w70_analysis_svg_with_context_window() {
    let mut receipt = minimal_receipt();
    let mut d = sample_derived();
    d.context_window = Some(ContextWindowReport {
        window_tokens: 128_000,
        total_tokens: 3_000,
        pct: 2.34,
        fits: true,
    });
    receipt.derived = Some(d);
    let out = text(render(&receipt, AnalysisFormat::Svg).unwrap());
    insta::assert_snapshot!("w70_analysis_svg_with_context_window", out);
}

// ===========================================================================
// Mermaid snapshot
// ===========================================================================

#[test]
fn w70_analysis_mermaid_with_imports() {
    let mut receipt = minimal_receipt();
    receipt.imports = Some(ImportReport {
        granularity: "module".into(),
        edges: vec![
            ImportEdge {
                from: "engine".into(),
                to: "api".into(),
                count: 5,
            },
            ImportEdge {
                from: "api".into(),
                to: "types".into(),
                count: 3,
            },
            ImportEdge {
                from: "engine".into(),
                to: "types".into(),
                count: 2,
            },
        ],
    });
    let out = text(render(&receipt, AnalysisFormat::Mermaid).unwrap());
    insta::assert_snapshot!("w70_analysis_mermaid_with_imports", out);
}

// ===========================================================================
// Tree snapshot
// ===========================================================================

#[test]
fn w70_analysis_tree_with_data() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let out = text(render(&receipt, AnalysisFormat::Tree).unwrap());
    insta::assert_snapshot!("w70_analysis_tree_with_data", out);
}

// ===========================================================================
// HTML snapshot
// ===========================================================================

#[test]
fn w70_analysis_html_with_derived() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let out = redact_timestamp(&text(render(&receipt, AnalysisFormat::Html).unwrap()));
    insta::assert_snapshot!("w70_analysis_html_with_derived", out);
}
