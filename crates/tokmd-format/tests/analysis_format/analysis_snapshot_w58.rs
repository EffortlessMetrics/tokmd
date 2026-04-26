//! Expanded insta snapshot tests for analysis format rendering (W58).
//!
//! Covers markdown, JSON, XML, SVG, mermaid, and tree renderings
//! with various preset and data configurations.

use tokmd_analysis_types::*;
use tokmd_format::analysis::{RenderedOutput, render};
use tokmd_types::{AnalysisFormat, ScanStatus, ToolInfo};

// ===========================================================================
// Fixtures
// ===========================================================================

fn base_receipt() -> AnalysisReceipt {
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

fn make_derived() -> DerivedReport {
    DerivedReport {
        totals: DerivedTotals {
            files: 20,
            code: 7000,
            comments: 800,
            blanks: 500,
            lines: 8300,
            bytes: 210_000,
            tokens: 70_000,
        },
        doc_density: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 800,
                denominator: 7800,
                ratio: 0.1026,
            },
            by_lang: vec![
                RatioRow {
                    key: "Rust".into(),
                    numerator: 600,
                    denominator: 5600,
                    ratio: 0.1071,
                },
                RatioRow {
                    key: "Python".into(),
                    numerator: 200,
                    denominator: 2200,
                    ratio: 0.0909,
                },
            ],
            by_module: vec![],
        },
        whitespace: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 500,
                denominator: 8300,
                ratio: 0.0602,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        verbosity: RateReport {
            total: RateRow {
                key: "total".into(),
                numerator: 210_000,
                denominator: 8300,
                rate: 25.3,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        max_file: MaxFileReport {
            overall: FileStatRow {
                path: "src/core/engine.rs".into(),
                module: "src/core".into(),
                lang: "Rust".into(),
                code: 900,
                comments: 120,
                blanks: 80,
                lines: 1100,
                bytes: 27_500,
                tokens: 9_000,
                doc_pct: Some(0.118),
                bytes_per_line: Some(25.0),
                depth: 2,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        lang_purity: LangPurityReport { rows: vec![] },
        nesting: NestingReport {
            max: 8,
            avg: 3.1,
            by_module: vec![],
        },
        test_density: TestDensityReport {
            test_lines: 1500,
            prod_lines: 6800,
            test_files: 10,
            prod_files: 10,
            ratio: 0.2206,
        },
        boilerplate: BoilerplateReport {
            infra_lines: 350,
            logic_lines: 7950,
            ratio: 0.044,
            infra_langs: vec!["TOML".into(), "YAML".into(), "JSON".into()],
        },
        polyglot: PolyglotReport {
            lang_count: 5,
            entropy: 1.2,
            dominant_lang: "Rust".into(),
            dominant_lines: 5600,
            dominant_pct: 0.675,
        },
        distribution: DistributionReport {
            count: 20,
            min: 10,
            max: 1100,
            mean: 415.0,
            median: 320.0,
            p90: 800.0,
            p99: 1050.0,
            gini: 0.42,
        },
        histogram: vec![
            HistogramBucket {
                label: "Tiny".into(),
                min: 0,
                max: Some(50),
                files: 3,
                pct: 0.15,
            },
            HistogramBucket {
                label: "Small".into(),
                min: 51,
                max: Some(200),
                files: 6,
                pct: 0.30,
            },
            HistogramBucket {
                label: "Medium".into(),
                min: 201,
                max: Some(500),
                files: 7,
                pct: 0.35,
            },
            HistogramBucket {
                label: "Large".into(),
                min: 501,
                max: None,
                files: 4,
                pct: 0.20,
            },
        ],
        top: TopOffenders {
            largest_lines: vec![FileStatRow {
                path: "src/core/engine.rs".into(),
                module: "src/core".into(),
                lang: "Rust".into(),
                code: 900,
                comments: 120,
                blanks: 80,
                lines: 1100,
                bytes: 27_500,
                tokens: 9_000,
                doc_pct: Some(0.118),
                bytes_per_line: Some(25.0),
                depth: 2,
            }],
            largest_tokens: vec![],
            largest_bytes: vec![],
            least_documented: vec![],
            most_dense: vec![],
        },
        tree: None,
        reading_time: ReadingTimeReport {
            minutes: 415.0,
            lines_per_minute: 20,
            basis_lines: 8300,
        },
        context_window: Some(ContextWindowReport {
            window_tokens: 128_000,
            total_tokens: 70_000,
            pct: 54.69,
            fits: true,
        }),
        cocomo: Some(CocomoReport {
            mode: "organic".into(),
            kloc: 7.0,
            effort_pm: 21.7,
            duration_months: 7.5,
            staff: 2.89,
            a: 2.4,
            b: 1.05,
            c: 2.5,
            d: 0.38,
        }),
        todo: Some(TodoReport {
            total: 12,
            density_per_kloc: 1.71,
            tags: vec![
                TodoTagRow {
                    tag: "TODO".into(),
                    count: 8,
                },
                TodoTagRow {
                    tag: "FIXME".into(),
                    count: 3,
                },
                TodoTagRow {
                    tag: "HACK".into(),
                    count: 1,
                },
            ],
        }),
        integrity: IntegrityReport {
            algo: "blake3".into(),
            hash: "a".repeat(64),
            entries: 20,
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
// Markdown – empty / minimal
// ===========================================================================

#[test]
fn w58_analysis_md_empty_receipt() {
    let r = base_receipt();
    insta::assert_snapshot!(
        "w58_analysis_md_empty_receipt",
        text(render(&r, AnalysisFormat::Md).unwrap())
    );
}

#[test]
fn w58_analysis_md_with_warnings() {
    let mut r = base_receipt();
    r.warnings = vec![
        "git feature disabled".into(),
        "content feature disabled".into(),
    ];
    insta::assert_snapshot!(
        "w58_analysis_md_with_warnings",
        text(render(&r, AnalysisFormat::Md).unwrap())
    );
}

// ===========================================================================
// Markdown – derived metrics
// ===========================================================================

#[test]
fn w58_analysis_md_derived_full() {
    let mut r = base_receipt();
    r.derived = Some(make_derived());
    insta::assert_snapshot!(
        "w58_analysis_md_derived_full",
        text(render(&r, AnalysisFormat::Md).unwrap())
    );
}

#[test]
fn w58_analysis_md_derived_no_todo() {
    let mut r = base_receipt();
    let mut d = make_derived();
    d.todo = None;
    r.derived = Some(d);
    insta::assert_snapshot!(
        "w58_analysis_md_derived_no_todo",
        text(render(&r, AnalysisFormat::Md).unwrap())
    );
}

#[test]
fn w58_analysis_md_derived_no_cocomo() {
    let mut r = base_receipt();
    let mut d = make_derived();
    d.cocomo = None;
    d.context_window = None;
    r.derived = Some(d);
    insta::assert_snapshot!(
        "w58_analysis_md_derived_no_cocomo",
        text(render(&r, AnalysisFormat::Md).unwrap())
    );
}

// ===========================================================================
// Markdown – preset sections
// ===========================================================================

#[test]
fn w58_analysis_md_health_preset() {
    let mut r = base_receipt();
    r.args.preset = "health".into();
    r.derived = Some(make_derived());
    r.complexity = Some(ComplexityReport {
        total_functions: 45,
        avg_function_length: 22.0,
        max_function_length: 120,
        avg_cyclomatic: 3.5,
        max_cyclomatic: 18,
        avg_cognitive: Some(5.2),
        max_cognitive: Some(25),
        avg_nesting_depth: Some(2.1),
        max_nesting_depth: Some(6),
        high_risk_files: 2,
        histogram: None,
        halstead: None,
        maintainability_index: None,
        technical_debt: None,
        files: vec![FileComplexity {
            path: "src/core/engine.rs".into(),
            module: "src/core".into(),
            function_count: 15,
            max_function_length: 120,
            cyclomatic_complexity: 18,
            cognitive_complexity: Some(25),
            max_nesting: Some(6),
            risk_level: ComplexityRisk::High,
            functions: None,
        }],
    });
    insta::assert_snapshot!(
        "w58_analysis_md_health_preset",
        text(render(&r, AnalysisFormat::Md).unwrap())
    );
}

#[test]
fn w58_analysis_md_risk_preset_with_entropy() {
    let mut r = base_receipt();
    r.args.preset = "risk".into();
    r.derived = Some(make_derived());
    r.entropy = Some(EntropyReport {
        suspects: vec![
            EntropyFinding {
                path: "data/keys.bin".into(),
                module: "data".into(),
                entropy_bits_per_byte: 7.85,
                sample_bytes: 4096,
                class: EntropyClass::High,
            },
            EntropyFinding {
                path: "config/token.enc".into(),
                module: "config".into(),
                entropy_bits_per_byte: 7.2,
                sample_bytes: 1024,
                class: EntropyClass::Suspicious,
            },
        ],
    });
    insta::assert_snapshot!(
        "w58_analysis_md_risk_preset_with_entropy",
        text(render(&r, AnalysisFormat::Md).unwrap())
    );
}

#[test]
fn w58_analysis_md_supply_preset() {
    let mut r = base_receipt();
    r.args.preset = "supply".into();
    r.derived = Some(make_derived());
    r.assets = Some(AssetReport {
        total_files: 15,
        total_bytes: 500_000,
        categories: vec![
            AssetCategoryRow {
                category: "images".into(),
                files: 8,
                bytes: 350_000,
                extensions: vec!["png".into(), "svg".into()],
            },
            AssetCategoryRow {
                category: "fonts".into(),
                files: 3,
                bytes: 120_000,
                extensions: vec!["woff2".into()],
            },
        ],
        top_files: vec![],
    });
    r.deps = Some(DependencyReport {
        total: 42,
        lockfiles: vec![LockfileReport {
            path: "Cargo.lock".into(),
            kind: "cargo".into(),
            dependencies: 42,
        }],
    });
    insta::assert_snapshot!(
        "w58_analysis_md_supply_preset",
        text(render(&r, AnalysisFormat::Md).unwrap())
    );
}

// ===========================================================================
// Markdown – complex / combined
// ===========================================================================

#[test]
fn w58_analysis_md_archetype_with_topics() {
    let mut r = base_receipt();
    r.archetype = Some(Archetype {
        kind: "monorepo".into(),
        evidence: vec![
            "Cargo.toml workspace".into(),
            "crates/".into(),
            "multiple Cargo.toml".into(),
        ],
    });
    r.topics = Some(TopicClouds {
        overall: vec![
            TopicTerm {
                term: "parsing".into(),
                score: 0.85,
                tf: 120,
                df: 8,
            },
            TopicTerm {
                term: "formatting".into(),
                score: 0.72,
                tf: 90,
                df: 6,
            },
        ],
        per_module: std::collections::BTreeMap::from([(
            "core".into(),
            vec![TopicTerm {
                term: "analysis".into(),
                score: 0.65,
                tf: 40,
                df: 3,
            }],
        )]),
    });
    insta::assert_snapshot!(
        "w58_analysis_md_archetype_with_topics",
        text(render(&r, AnalysisFormat::Md).unwrap())
    );
}

#[test]
fn w58_analysis_md_license_section() {
    let mut r = base_receipt();
    r.license = Some(LicenseReport {
        effective: Some("MIT OR Apache-2.0".into()),
        findings: vec![
            LicenseFinding {
                spdx: "MIT".into(),
                confidence: 0.95,
                source_path: "LICENSE-MIT".into(),
                source_kind: LicenseSourceKind::Text,
            },
            LicenseFinding {
                spdx: "Apache-2.0".into(),
                confidence: 0.92,
                source_path: "LICENSE-APACHE".into(),
                source_kind: LicenseSourceKind::Text,
            },
        ],
    });
    insta::assert_snapshot!(
        "w58_analysis_md_license_section",
        text(render(&r, AnalysisFormat::Md).unwrap())
    );
}

#[test]
fn w58_analysis_md_imports_section() {
    let mut r = base_receipt();
    r.imports = Some(ImportReport {
        granularity: "module".into(),
        edges: vec![
            ImportEdge {
                from: "core".into(),
                to: "types".into(),
                count: 25,
            },
            ImportEdge {
                from: "format".into(),
                to: "types".into(),
                count: 18,
            },
            ImportEdge {
                from: "cli".into(),
                to: "core".into(),
                count: 12,
            },
        ],
    });
    insta::assert_snapshot!(
        "w58_analysis_md_imports_section",
        text(render(&r, AnalysisFormat::Md).unwrap())
    );
}

// ===========================================================================
// JSON
// ===========================================================================

#[test]
fn w58_analysis_json_empty() {
    let r = base_receipt();
    let rendered = text(render(&r, AnalysisFormat::Json).unwrap());
    let v: serde_json::Value = serde_json::from_str(&rendered).unwrap();
    insta::assert_json_snapshot!("w58_analysis_json_empty", v);
}

#[test]
fn w58_analysis_json_with_derived() {
    let mut r = base_receipt();
    r.derived = Some(make_derived());
    let rendered = text(render(&r, AnalysisFormat::Json).unwrap());
    let v: serde_json::Value = serde_json::from_str(&rendered).unwrap();
    insta::assert_json_snapshot!("w58_analysis_json_with_derived", v);
}

// ===========================================================================
// XML
// ===========================================================================

#[test]
fn w58_analysis_xml_with_derived() {
    let mut r = base_receipt();
    r.derived = Some(make_derived());
    insta::assert_snapshot!(
        "w58_analysis_xml_with_derived",
        text(render(&r, AnalysisFormat::Xml).unwrap())
    );
}

// ===========================================================================
// SVG badge from analysis
// ===========================================================================

#[test]
fn w58_analysis_svg_with_context_window() {
    let mut r = base_receipt();
    r.derived = Some(make_derived());
    insta::assert_snapshot!(
        "w58_analysis_svg_with_context_window",
        text(render(&r, AnalysisFormat::Svg).unwrap())
    );
}
