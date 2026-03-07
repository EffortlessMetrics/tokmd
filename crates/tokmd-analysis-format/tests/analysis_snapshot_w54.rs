//! Golden snapshot tests for analysis format rendering (W54).
//!
//! Pins the output of each text format renderer so regressions are caught.

use tokmd_analysis_format::{RenderedOutput, render};
use tokmd_analysis_types::*;
use tokmd_types::{AnalysisFormat, ScanStatus, ToolInfo};

// ===========================================================================
// Fixtures
// ===========================================================================

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
            module_roots: vec!["src".into()],
            module_depth: 2,
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

fn make_derived() -> DerivedReport {
    DerivedReport {
        totals: DerivedTotals {
            files: 12,
            code: 3500,
            comments: 400,
            blanks: 300,
            lines: 4200,
            bytes: 105_000,
            tokens: 35_000,
        },
        doc_density: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 400,
                denominator: 3900,
                ratio: 0.1026,
            },
            by_lang: vec![
                RatioRow {
                    key: "Rust".into(),
                    numerator: 350,
                    denominator: 3200,
                    ratio: 0.1094,
                },
                RatioRow {
                    key: "Python".into(),
                    numerator: 50,
                    denominator: 700,
                    ratio: 0.0714,
                },
            ],
            by_module: vec![],
        },
        whitespace: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 300,
                denominator: 4200,
                ratio: 0.0714,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        verbosity: RateReport {
            total: RateRow {
                key: "total".into(),
                numerator: 105_000,
                denominator: 4200,
                rate: 25.0,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        max_file: MaxFileReport {
            overall: FileStatRow {
                path: "src/engine.rs".into(),
                module: "src".into(),
                lang: "Rust".into(),
                code: 500,
                comments: 60,
                blanks: 40,
                lines: 600,
                bytes: 15_000,
                tokens: 5_000,
                doc_pct: Some(0.107),
                bytes_per_line: Some(25.0),
                depth: 1,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        lang_purity: LangPurityReport { rows: vec![] },
        nesting: NestingReport {
            max: 6,
            avg: 2.5,
            by_module: vec![],
        },
        test_density: TestDensityReport {
            test_lines: 800,
            prod_lines: 3400,
            test_files: 5,
            prod_files: 7,
            ratio: 0.2353,
        },
        boilerplate: BoilerplateReport {
            infra_lines: 200,
            logic_lines: 4000,
            ratio: 0.05,
            infra_langs: vec!["TOML".into(), "YAML".into()],
        },
        polyglot: PolyglotReport {
            lang_count: 3,
            entropy: 0.85,
            dominant_lang: "Rust".into(),
            dominant_lines: 3200,
            dominant_pct: 0.762,
        },
        distribution: DistributionReport {
            count: 12,
            min: 15,
            max: 600,
            mean: 350.0,
            median: 280.0,
            p90: 550.0,
            p99: 600.0,
            gini: 0.38,
        },
        histogram: vec![
            HistogramBucket {
                label: "Tiny".into(),
                min: 0,
                max: Some(50),
                files: 2,
                pct: 0.167,
            },
            HistogramBucket {
                label: "Small".into(),
                min: 51,
                max: Some(200),
                files: 4,
                pct: 0.333,
            },
            HistogramBucket {
                label: "Medium".into(),
                min: 201,
                max: Some(500),
                files: 5,
                pct: 0.417,
            },
            HistogramBucket {
                label: "Large".into(),
                min: 501,
                max: None,
                files: 1,
                pct: 0.083,
            },
        ],
        top: TopOffenders {
            largest_lines: vec![FileStatRow {
                path: "src/engine.rs".into(),
                module: "src".into(),
                lang: "Rust".into(),
                code: 500,
                comments: 60,
                blanks: 40,
                lines: 600,
                bytes: 15_000,
                tokens: 5_000,
                doc_pct: Some(0.107),
                bytes_per_line: Some(25.0),
                depth: 1,
            }],
            largest_tokens: vec![],
            largest_bytes: vec![],
            least_documented: vec![],
            most_dense: vec![],
        },
        tree: None,
        reading_time: ReadingTimeReport {
            minutes: 210.0,
            lines_per_minute: 20,
            basis_lines: 4200,
        },
        context_window: Some(ContextWindowReport {
            window_tokens: 128_000,
            total_tokens: 35_000,
            pct: 27.34,
            fits: true,
        }),
        cocomo: Some(CocomoReport {
            mode: "organic".into(),
            kloc: 3.5,
            effort_pm: 10.14,
            duration_months: 5.54,
            staff: 1.83,
            a: 2.4,
            b: 1.05,
            c: 2.5,
            d: 0.38,
        }),
        todo: Some(TodoReport {
            total: 7,
            density_per_kloc: 2.0,
            tags: vec![
                TodoTagRow {
                    tag: "TODO".into(),
                    count: 5,
                },
                TodoTagRow {
                    tag: "FIXME".into(),
                    count: 2,
                },
            ],
        }),
        integrity: IntegrityReport {
            algo: "blake3".into(),
            hash: "b".repeat(64),
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

// ===========================================================================
// Markdown
// ===========================================================================

#[test]
fn w54_analysis_md_minimal() {
    let r = base_receipt();
    insta::assert_snapshot!(
        "w54_analysis_md_minimal",
        text(render(&r, AnalysisFormat::Md).unwrap())
    );
}

#[test]
fn w54_analysis_md_with_derived() {
    let mut r = base_receipt();
    r.derived = Some(make_derived());
    insta::assert_snapshot!(
        "w54_analysis_md_with_derived",
        text(render(&r, AnalysisFormat::Md).unwrap())
    );
}

#[test]
fn w54_analysis_md_with_cocomo() {
    let mut r = base_receipt();
    let mut d = make_derived();
    d.cocomo = Some(CocomoReport {
        mode: "semi-detached".into(),
        kloc: 50.0,
        effort_pm: 239.0,
        duration_months: 18.0,
        staff: 13.28,
        a: 3.0,
        b: 1.12,
        c: 2.5,
        d: 0.35,
    });
    r.derived = Some(d);
    insta::assert_snapshot!(
        "w54_analysis_md_with_cocomo",
        text(render(&r, AnalysisFormat::Md).unwrap())
    );
}

#[test]
fn w54_analysis_md_with_archetype() {
    let mut r = base_receipt();
    r.archetype = Some(Archetype {
        kind: "web-api".into(),
        evidence: vec![
            "Cargo.toml".into(),
            "src/routes.rs".into(),
            "Dockerfile".into(),
        ],
    });
    insta::assert_snapshot!(
        "w54_analysis_md_with_archetype",
        text(render(&r, AnalysisFormat::Md).unwrap())
    );
}

#[test]
fn w54_analysis_md_distribution() {
    let mut r = base_receipt();
    r.derived = Some(make_derived());
    insta::assert_snapshot!(
        "w54_analysis_md_distribution",
        text(render(&r, AnalysisFormat::Md).unwrap())
    );
}

// ===========================================================================
// JSON
// ===========================================================================

#[test]
fn w54_analysis_json_minimal() {
    let r = base_receipt();
    let rendered = text(render(&r, AnalysisFormat::Json).unwrap());
    let v: serde_json::Value = serde_json::from_str(&rendered).unwrap();
    insta::assert_json_snapshot!("w54_analysis_json_minimal", v);
}

#[test]
fn w54_analysis_json_with_derived() {
    let mut r = base_receipt();
    r.derived = Some(make_derived());
    let rendered = text(render(&r, AnalysisFormat::Json).unwrap());
    let v: serde_json::Value = serde_json::from_str(&rendered).unwrap();
    insta::assert_json_snapshot!("w54_analysis_json_with_derived", v);
}

// ===========================================================================
// XML
// ===========================================================================

#[test]
fn w54_analysis_xml_minimal() {
    let r = base_receipt();
    insta::assert_snapshot!(
        "w54_analysis_xml_minimal",
        text(render(&r, AnalysisFormat::Xml).unwrap())
    );
}

// ===========================================================================
// SVG
// ===========================================================================

#[test]
fn w54_analysis_svg_minimal() {
    let r = base_receipt();
    insta::assert_snapshot!(
        "w54_analysis_svg_minimal",
        text(render(&r, AnalysisFormat::Svg).unwrap())
    );
}

#[test]
fn w54_analysis_svg_with_derived() {
    let mut r = base_receipt();
    r.derived = Some(make_derived());
    insta::assert_snapshot!(
        "w54_analysis_svg_with_derived",
        text(render(&r, AnalysisFormat::Svg).unwrap())
    );
}

// ===========================================================================
// Mermaid
// ===========================================================================

#[test]
fn w54_analysis_mermaid_with_imports() {
    let mut r = base_receipt();
    r.imports = Some(ImportReport {
        granularity: "module".into(),
        edges: vec![
            ImportEdge {
                from: "core".into(),
                to: "utils".into(),
                count: 10,
            },
            ImportEdge {
                from: "cli".into(),
                to: "core".into(),
                count: 5,
            },
        ],
    });
    insta::assert_snapshot!(
        "w54_analysis_mermaid_with_imports",
        text(render(&r, AnalysisFormat::Mermaid).unwrap())
    );
}

// ===========================================================================
// Tree
// ===========================================================================

#[test]
fn w54_analysis_tree_with_data() {
    let mut r = base_receipt();
    let mut d = make_derived();
    d.tree = Some("project/\n  src/\n    engine.rs (600 lines)\n    lib.rs (225 lines)\n  tests/\n    smoke.rs (73 lines)".into());
    r.derived = Some(d);
    insta::assert_snapshot!(
        "w54_analysis_tree_with_data",
        text(render(&r, AnalysisFormat::Tree).unwrap())
    );
}

// ===========================================================================
// Full analysis
// ===========================================================================

#[test]
fn w54_analysis_md_full() {
    let mut r = base_receipt();
    r.derived = Some(make_derived());
    r.archetype = Some(Archetype {
        kind: "cli-tool".into(),
        evidence: vec!["main.rs".into(), "Cargo.toml".into()],
    });
    r.entropy = Some(EntropyReport {
        suspects: vec![EntropyFinding {
            path: "data/secret.bin".into(),
            module: "data".into(),
            entropy_bits_per_byte: 7.9,
            sample_bytes: 2048,
            class: EntropyClass::High,
        }],
    });
    r.license = Some(LicenseReport {
        effective: Some("Apache-2.0".into()),
        findings: vec![LicenseFinding {
            spdx: "Apache-2.0".into(),
            confidence: 0.98,
            source_path: "LICENSE".into(),
            source_kind: LicenseSourceKind::Text,
        }],
    });
    insta::assert_snapshot!(
        "w54_analysis_md_full",
        text(render(&r, AnalysisFormat::Md).unwrap())
    );
}
