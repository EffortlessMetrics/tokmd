//! Insta snapshot tests for analysis format rendering.
//!
//! Pins the exact output of each text format so regressions are caught
//! at review time. Uses the same fixture helpers as render_formats.rs.

use tokmd_analysis_format::{RenderedOutput, render};
use tokmd_analysis_types::*;
use tokmd_types::{AnalysisFormat, ScanStatus, ToolInfo};

// ---------------------------------------------------------------------------
// Fixtures (mirrors render_formats.rs)
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
            format: "md".into(),
            window_tokens: None,
            git: None,
            max_files: None,
            max_bytes: None,
            max_file_bytes: None,
            max_commits: None,
            max_commit_files: None,
            import_granularity: "module".into(),
            effort_model: None,
            effort_layer: None,
            base_ref: None,
            head_ref: None,
            monte_carlo: None,
            mc_iterations: None,
            mc_seed: None,
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
        effort: None,
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
            by_lang: vec![RatioRow {
                key: "Rust".into(),
                numerator: 70,
                denominator: 470,
                ratio: 0.1489,
            }],
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
        RenderedOutput::Binary(_) => panic!("expected text"),
    }
}

// ---------------------------------------------------------------------------
// Markdown snapshots
// ---------------------------------------------------------------------------

#[test]
fn snapshot_analysis_md_minimal() {
    let receipt = minimal_receipt();
    let rendered = text(render(&receipt, AnalysisFormat::Md).unwrap());
    insta::assert_snapshot!("analysis_md_minimal", rendered);
}

#[test]
fn snapshot_analysis_md_with_derived() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let rendered = text(render(&receipt, AnalysisFormat::Md).unwrap());
    insta::assert_snapshot!("analysis_md_with_derived", rendered);
}

#[test]
fn snapshot_analysis_md_full() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    receipt.archetype = Some(Archetype {
        kind: "cli-tool".into(),
        evidence: vec!["main.rs".into(), "Cargo.toml".into()],
    });
    receipt.imports = Some(ImportReport {
        granularity: "module".into(),
        edges: vec![
            ImportEdge {
                from: "src/a".into(),
                to: "src/b".into(),
                count: 3,
            },
            ImportEdge {
                from: "src/b".into(),
                to: "src/c".into(),
                count: 1,
            },
        ],
    });
    receipt.entropy = Some(EntropyReport {
        suspects: vec![EntropyFinding {
            path: "secrets.bin".into(),
            module: "data".into(),
            entropy_bits_per_byte: 7.8,
            sample_bytes: 1024,
            class: EntropyClass::High,
        }],
    });
    receipt.license = Some(LicenseReport {
        effective: Some("MIT".into()),
        findings: vec![LicenseFinding {
            spdx: "MIT".into(),
            confidence: 0.95,
            source_path: "LICENSE".into(),
            source_kind: LicenseSourceKind::Text,
        }],
    });
    let rendered = text(render(&receipt, AnalysisFormat::Md).unwrap());
    insta::assert_snapshot!("analysis_md_full", rendered);
}

// ---------------------------------------------------------------------------
// JSON snapshots
// ---------------------------------------------------------------------------

#[test]
fn snapshot_analysis_json_minimal() {
    let receipt = minimal_receipt();
    let rendered = text(render(&receipt, AnalysisFormat::Json).unwrap());
    insta::assert_snapshot!("analysis_json_minimal", rendered);
}

#[test]
fn snapshot_analysis_json_with_derived() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let rendered = text(render(&receipt, AnalysisFormat::Json).unwrap());
    insta::assert_snapshot!("analysis_json_with_derived", rendered);
}

// ---------------------------------------------------------------------------
// JSON-LD snapshots
// ---------------------------------------------------------------------------

#[test]
fn snapshot_analysis_jsonld_minimal() {
    let receipt = minimal_receipt();
    let rendered = text(render(&receipt, AnalysisFormat::Jsonld).unwrap());
    insta::assert_snapshot!("analysis_jsonld_minimal", rendered);
}

#[test]
fn snapshot_analysis_jsonld_with_derived() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let rendered = text(render(&receipt, AnalysisFormat::Jsonld).unwrap());
    insta::assert_snapshot!("analysis_jsonld_with_derived", rendered);
}

// ---------------------------------------------------------------------------
// XML snapshots
// ---------------------------------------------------------------------------

#[test]
fn snapshot_analysis_xml_minimal() {
    let receipt = minimal_receipt();
    let rendered = text(render(&receipt, AnalysisFormat::Xml).unwrap());
    insta::assert_snapshot!("analysis_xml_minimal", rendered);
}

#[test]
fn snapshot_analysis_xml_with_derived() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let rendered = text(render(&receipt, AnalysisFormat::Xml).unwrap());
    insta::assert_snapshot!("analysis_xml_with_derived", rendered);
}

// ---------------------------------------------------------------------------
// SVG snapshots
// ---------------------------------------------------------------------------

#[test]
fn snapshot_analysis_svg_minimal() {
    let receipt = minimal_receipt();
    let rendered = text(render(&receipt, AnalysisFormat::Svg).unwrap());
    insta::assert_snapshot!("analysis_svg_minimal", rendered);
}

#[test]
fn snapshot_analysis_svg_with_derived() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let rendered = text(render(&receipt, AnalysisFormat::Svg).unwrap());
    insta::assert_snapshot!("analysis_svg_with_derived", rendered);
}

// ---------------------------------------------------------------------------
// Mermaid snapshots
// ---------------------------------------------------------------------------

#[test]
fn snapshot_analysis_mermaid_no_imports() {
    let receipt = minimal_receipt();
    let rendered = text(render(&receipt, AnalysisFormat::Mermaid).unwrap());
    insta::assert_snapshot!("analysis_mermaid_no_imports", rendered);
}

#[test]
fn snapshot_analysis_mermaid_with_imports() {
    let mut receipt = minimal_receipt();
    receipt.imports = Some(ImportReport {
        granularity: "module".into(),
        edges: vec![
            ImportEdge {
                from: "mod-a".into(),
                to: "mod-b".into(),
                count: 5,
            },
            ImportEdge {
                from: "mod-b".into(),
                to: "mod-c".into(),
                count: 2,
            },
        ],
    });
    let rendered = text(render(&receipt, AnalysisFormat::Mermaid).unwrap());
    insta::assert_snapshot!("analysis_mermaid_with_imports", rendered);
}

// ---------------------------------------------------------------------------
// Tree snapshots
// ---------------------------------------------------------------------------

#[test]
fn snapshot_analysis_tree_unavailable() {
    let receipt = minimal_receipt();
    let rendered = text(render(&receipt, AnalysisFormat::Tree).unwrap());
    insta::assert_snapshot!("analysis_tree_unavailable", rendered);
}

#[test]
fn snapshot_analysis_tree_with_data() {
    let mut receipt = minimal_receipt();
    let mut d = sample_derived();
    d.tree = Some("root\n  src/\n    lib.rs (500 lines)\n    main.rs (245 lines)\n  tests/\n    smoke.rs (65 lines)".into());
    receipt.derived = Some(d);
    let rendered = text(render(&receipt, AnalysisFormat::Tree).unwrap());
    insta::assert_snapshot!("analysis_tree_with_data", rendered);
}

// ---------------------------------------------------------------------------
// HTML snapshot
// ---------------------------------------------------------------------------

#[test]
fn snapshot_analysis_html_with_derived() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let rendered = text(render(&receipt, AnalysisFormat::Html).unwrap());
    // Normalize the embedded timestamp so the snapshot is deterministic.
    let rendered = regex_replace_timestamp(&rendered);
    insta::assert_snapshot!("analysis_html_with_derived", rendered);
}

/// Replace the HTML timestamp line with a fixed value for determinism.
fn regex_replace_timestamp(s: &str) -> String {
    s.lines()
        .map(|line| {
            if line.contains("class=\"timestamp\"") && line.contains("Generated:") {
                "        <div class=\"timestamp\">Generated: 1970-01-01 00:00:00 UTC</div>"
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
