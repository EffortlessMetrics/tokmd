//! Golden snapshot tests for analysis format rendering.
//!
//! These tests complement the assertion-based tests in render_formats.rs by
//! capturing full rendered output as insta snapshots, making regressions
//! immediately visible as diffs.

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
            files: 5,
            code: 500,
            comments: 80,
            blanks: 40,
            lines: 620,
            bytes: 5000,
            tokens: 1200,
        },
        doc_density: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 80,
                denominator: 580,
                ratio: 0.1379,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        whitespace: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 40,
                denominator: 620,
                ratio: 0.0645,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        verbosity: RateReport {
            total: RateRow {
                key: "total".into(),
                numerator: 5000,
                denominator: 620,
                rate: 8.06,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        max_file: MaxFileReport {
            overall: FileStatRow {
                path: "src/main.rs".into(),
                module: "src".into(),
                lang: "Rust".into(),
                code: 200,
                comments: 30,
                blanks: 15,
                lines: 245,
                bytes: 2000,
                tokens: 500,
                doc_pct: Some(0.13),
                bytes_per_line: Some(8.16),
                depth: 1,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        lang_purity: LangPurityReport { rows: vec![] },
        nesting: NestingReport {
            max: 4,
            avg: 2.0,
            by_module: vec![],
        },
        test_density: TestDensityReport {
            test_lines: 100,
            prod_lines: 400,
            test_files: 2,
            prod_files: 3,
            ratio: 0.25,
        },
        boilerplate: BoilerplateReport {
            infra_lines: 50,
            logic_lines: 570,
            ratio: 0.0877,
            infra_langs: vec!["TOML".into()],
        },
        polyglot: PolyglotReport {
            lang_count: 2,
            entropy: 0.72,
            dominant_lang: "Rust".into(),
            dominant_lines: 450,
            dominant_pct: 0.9,
        },
        distribution: DistributionReport {
            count: 5,
            min: 25,
            max: 245,
            mean: 124.0,
            median: 100.0,
            p90: 245.0,
            p99: 245.0,
            gini: 0.32,
        },
        histogram: vec![
            HistogramBucket {
                label: "Small".into(),
                min: 0,
                max: Some(100),
                files: 3,
                pct: 0.6,
            },
            HistogramBucket {
                label: "Medium".into(),
                min: 101,
                max: Some(500),
                files: 2,
                pct: 0.4,
            },
        ],
        top: TopOffenders {
            largest_lines: vec![FileStatRow {
                path: "src/main.rs".into(),
                module: "src".into(),
                lang: "Rust".into(),
                code: 200,
                comments: 30,
                blanks: 15,
                lines: 245,
                bytes: 2000,
                tokens: 500,
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
            minutes: 31.0,
            lines_per_minute: 20,
            basis_lines: 620,
        },
        context_window: None,
        cocomo: None,
        todo: None,
        integrity: IntegrityReport {
            algo: "blake3".into(),
            hash: "a".repeat(64),
            entries: 5,
        },
    }
}

fn text(output: RenderedOutput) -> String {
    match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text output"),
    }
}

// ===========================================================================
// JSON snapshots
// ===========================================================================

#[test]
fn snapshot_json_minimal() {
    let receipt = minimal_receipt();
    let out = text(render(&receipt, AnalysisFormat::Json).unwrap());
    insta::assert_snapshot!("json_minimal", out);
}

#[test]
fn snapshot_json_with_derived() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let out = text(render(&receipt, AnalysisFormat::Json).unwrap());
    insta::assert_snapshot!("json_with_derived", out);
}

#[test]
fn snapshot_json_with_archetype() {
    let mut receipt = minimal_receipt();
    receipt.archetype = Some(Archetype {
        kind: "library".into(),
        evidence: vec!["Cargo.toml".into(), "src/lib.rs".into()],
    });
    let out = text(render(&receipt, AnalysisFormat::Json).unwrap());
    insta::assert_snapshot!("json_with_archetype", out);
}

#[test]
fn snapshot_json_with_eco_label() {
    let mut receipt = minimal_receipt();
    receipt.fun = Some(FunReport {
        eco_label: Some(EcoLabel {
            score: 95.0,
            label: "A".into(),
            bytes: 500_000,
            notes: "Size-based eco label (0.48 MB)".into(),
        }),
    });
    let out = text(render(&receipt, AnalysisFormat::Json).unwrap());
    insta::assert_snapshot!("json_with_eco_label", out);
}

// ===========================================================================
// Markdown snapshots
// ===========================================================================

#[test]
fn snapshot_md_minimal() {
    let receipt = minimal_receipt();
    let out = text(render(&receipt, AnalysisFormat::Md).unwrap());
    insta::assert_snapshot!("md_minimal", out);
}

#[test]
fn snapshot_md_with_derived() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let out = text(render(&receipt, AnalysisFormat::Md).unwrap());
    insta::assert_snapshot!("md_with_derived", out);
}

#[test]
fn snapshot_md_with_archetype_and_eco() {
    let mut receipt = minimal_receipt();
    receipt.archetype = Some(Archetype {
        kind: "monorepo".into(),
        evidence: vec!["Cargo.toml".into(), "package.json".into()],
    });
    receipt.fun = Some(FunReport {
        eco_label: Some(EcoLabel {
            score: 80.0,
            label: "B".into(),
            bytes: 5_000_000,
            notes: "Size-based eco label (4.77 MB)".into(),
        }),
    });
    let out = text(render(&receipt, AnalysisFormat::Md).unwrap());
    insta::assert_snapshot!("md_with_archetype_and_eco", out);
}

// ===========================================================================
// JSON-LD snapshot
// ===========================================================================

#[test]
fn snapshot_jsonld_with_derived() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let out = text(render(&receipt, AnalysisFormat::Jsonld).unwrap());
    insta::assert_snapshot!("jsonld_with_derived", out);
}

// ===========================================================================
// Mermaid snapshot
// ===========================================================================

#[test]
fn snapshot_mermaid_with_imports() {
    let mut receipt = minimal_receipt();
    receipt.imports = Some(ImportReport {
        granularity: "module".into(),
        edges: vec![
            ImportEdge {
                from: "mod-a".into(),
                to: "mod-b".into(),
                count: 2,
            },
            ImportEdge {
                from: "mod-b".into(),
                to: "mod-c".into(),
                count: 7,
            },
            ImportEdge {
                from: "mod-a".into(),
                to: "mod-c".into(),
                count: 1,
            },
        ],
    });
    let out = text(render(&receipt, AnalysisFormat::Mermaid).unwrap());
    insta::assert_snapshot!("mermaid_with_imports", out);
}

// ===========================================================================
// SVG snapshot
// ===========================================================================

#[test]
fn snapshot_svg_with_derived() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let out = text(render(&receipt, AnalysisFormat::Svg).unwrap());
    insta::assert_snapshot!("svg_with_derived", out);
}

// ===========================================================================
// Tree snapshot
// ===========================================================================

#[test]
fn snapshot_tree_minimal() {
    let receipt = minimal_receipt();
    let out = text(render(&receipt, AnalysisFormat::Tree).unwrap());
    insta::assert_snapshot!("tree_minimal", out);
}

// ===========================================================================
// HTML snapshot
// ===========================================================================

#[test]
fn snapshot_html_with_derived() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let out = text(render(&receipt, AnalysisFormat::Html).unwrap());
    insta::assert_snapshot!("html_with_derived", out);
}

// ===========================================================================
// XML snapshot
// ===========================================================================

#[test]
fn snapshot_xml_with_derived() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let out = text(render(&receipt, AnalysisFormat::Xml).unwrap());
    insta::assert_snapshot!("xml_with_derived", out);
}
