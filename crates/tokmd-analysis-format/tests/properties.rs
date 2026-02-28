//! Property-based tests for analysis format rendering determinism.
//!
//! Verifies that:
//! 1. Rendering the same receipt twice produces identical output (determinism).
//! 2. JSON round-trips preserve structure (JSON fidelity).
//! 3. Markdown output always starts with the expected header.
//! 4. XML output is well-bracketed.
//! 5. SVG output is valid XML-ish structure.

use proptest::prelude::*;

use tokmd_analysis_format::{RenderedOutput, render};
use tokmd_analysis_types::*;
use tokmd_types::{AnalysisFormat, ScanStatus, ToolInfo};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_file_stat_row() -> impl Strategy<Value = FileStatRow> {
    (
        "[a-z/]{1,30}",           // path
        "[a-z]{1,10}",            // module
        prop_oneof!["Rust", "Python", "TOML", "JavaScript"], // lang
        1usize..10_000,           // code
        0usize..5_000,            // comments
        0usize..2_000,            // blanks
        1usize..20_000,           // lines
        1usize..500_000,          // bytes
        1usize..50_000,           // tokens
        proptest::option::of(0.0f64..1.0), // doc_pct
        proptest::option::of(1.0f64..100.0), // bytes_per_line
        0usize..20,               // depth
    )
        .prop_map(
            |(path, module, lang, code, comments, blanks, lines, bytes, tokens, doc_pct, bpl, depth)| {
                FileStatRow {
                    path,
                    module,
                    lang,
                    code,
                    comments,
                    blanks,
                    lines,
                    bytes,
                    tokens,
                    doc_pct,
                    bytes_per_line: bpl,
                    depth,
                }
            },
        )
}

fn arb_derived() -> impl Strategy<Value = DerivedReport> {
    (
        1usize..100,    // files
        1usize..100_000, // code
        0usize..50_000,  // comments
        0usize..20_000,  // blanks
        prop::collection::vec(arb_file_stat_row(), 0..5),
    )
        .prop_map(|(files, code, comments, blanks, largest)| {
            let lines = code + comments + blanks;
            let bytes = lines * 8;
            let tokens = lines * 2;
            DerivedReport {
                totals: DerivedTotals {
                    files,
                    code,
                    comments,
                    blanks,
                    lines,
                    bytes,
                    tokens,
                },
                doc_density: RatioReport {
                    total: RatioRow {
                        key: "total".into(),
                        numerator: comments,
                        denominator: code + comments,
                        ratio: if code + comments > 0 {
                            comments as f64 / (code + comments) as f64
                        } else {
                            0.0
                        },
                    },
                    by_lang: vec![],
                    by_module: vec![],
                },
                whitespace: RatioReport {
                    total: RatioRow {
                        key: "total".into(),
                        numerator: blanks,
                        denominator: lines,
                        ratio: if lines > 0 {
                            blanks as f64 / lines as f64
                        } else {
                            0.0
                        },
                    },
                    by_lang: vec![],
                    by_module: vec![],
                },
                verbosity: RateReport {
                    total: RateRow {
                        key: "total".into(),
                        numerator: bytes,
                        denominator: lines,
                        rate: if lines > 0 {
                            bytes as f64 / lines as f64
                        } else {
                            0.0
                        },
                    },
                    by_lang: vec![],
                    by_module: vec![],
                },
                max_file: MaxFileReport {
                    overall: FileStatRow {
                        path: "src/lib.rs".into(),
                        module: "src".into(),
                        lang: "Rust".into(),
                        code,
                        comments,
                        blanks,
                        lines,
                        bytes,
                        tokens,
                        doc_pct: Some(0.1),
                        bytes_per_line: Some(8.0),
                        depth: 1,
                    },
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
                    test_lines: 0,
                    prod_lines: code,
                    test_files: 0,
                    prod_files: files,
                    ratio: 0.0,
                },
                boilerplate: BoilerplateReport {
                    infra_lines: 0,
                    logic_lines: code,
                    ratio: 0.0,
                    infra_langs: vec![],
                },
                polyglot: PolyglotReport {
                    lang_count: 1,
                    entropy: 0.0,
                    dominant_lang: "Rust".into(),
                    dominant_lines: code,
                    dominant_pct: 1.0,
                },
                distribution: DistributionReport {
                    count: files,
                    min: 10,
                    max: lines,
                    mean: lines as f64 / files.max(1) as f64,
                    median: lines as f64 / files.max(1) as f64,
                    p90: lines as f64,
                    p99: lines as f64,
                    gini: 0.0,
                },
                histogram: vec![HistogramBucket {
                    label: "all".into(),
                    min: 0,
                    max: Some(lines),
                    files,
                    pct: 1.0,
                }],
                top: TopOffenders {
                    largest_lines: largest,
                    largest_tokens: vec![],
                    largest_bytes: vec![],
                    least_documented: vec![],
                    most_dense: vec![],
                },
                tree: None,
                reading_time: ReadingTimeReport {
                    minutes: lines as f64 / 20.0,
                    lines_per_minute: 20,
                    basis_lines: lines,
                },
                context_window: None,
                cocomo: None,
                todo: None,
                integrity: IntegrityReport {
                    algo: "blake3".into(),
                    hash: "0".repeat(64),
                    entries: files,
                },
            }
        })
}

fn arb_receipt() -> impl Strategy<Value = AnalysisReceipt> {
    (proptest::option::of(arb_derived()), "[a-z]{3,10}").prop_map(|(derived, preset)| {
        AnalysisReceipt {
            schema_version: ANALYSIS_SCHEMA_VERSION,
            generated_at_ms: 0,
            tool: ToolInfo {
                name: "tokmd".into(),
                version: "test".into(),
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
                preset,
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
            derived,
            assets: None,
            deps: None,
            git: None,
            imports: None,
            dup: None,
            complexity: None,
            api_surface: None,
            fun: None,
        }
    })
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Rendering the same receipt twice with the same format must produce identical output.
    #[test]
    fn md_render_is_deterministic(receipt in arb_receipt()) {
        let a = render(&receipt, AnalysisFormat::Md).unwrap();
        let b = render(&receipt, AnalysisFormat::Md).unwrap();
        let (a_text, b_text) = match (a, b) {
            (RenderedOutput::Text(a), RenderedOutput::Text(b)) => (a, b),
            _ => panic!("expected text output"),
        };
        prop_assert_eq!(a_text, b_text);
    }

    #[test]
    fn json_render_is_deterministic(receipt in arb_receipt()) {
        let a = render(&receipt, AnalysisFormat::Json).unwrap();
        let b = render(&receipt, AnalysisFormat::Json).unwrap();
        let (a_text, b_text) = match (a, b) {
            (RenderedOutput::Text(a), RenderedOutput::Text(b)) => (a, b),
            _ => panic!("expected text output"),
        };
        prop_assert_eq!(a_text, b_text);
    }

    #[test]
    fn xml_render_is_deterministic(receipt in arb_receipt()) {
        let a = render(&receipt, AnalysisFormat::Xml).unwrap();
        let b = render(&receipt, AnalysisFormat::Xml).unwrap();
        let (a_text, b_text) = match (a, b) {
            (RenderedOutput::Text(a), RenderedOutput::Text(b)) => (a, b),
            _ => panic!("expected text output"),
        };
        prop_assert_eq!(a_text, b_text);
    }

    #[test]
    fn svg_render_is_deterministic(receipt in arb_receipt()) {
        let a = render(&receipt, AnalysisFormat::Svg).unwrap();
        let b = render(&receipt, AnalysisFormat::Svg).unwrap();
        let (a_text, b_text) = match (a, b) {
            (RenderedOutput::Text(a), RenderedOutput::Text(b)) => (a, b),
            _ => panic!("expected text output"),
        };
        prop_assert_eq!(a_text, b_text);
    }

    /// JSON output round-trips back to a valid AnalysisReceipt.
    #[test]
    fn json_round_trips(receipt in arb_receipt()) {
        let rendered = render(&receipt, AnalysisFormat::Json).unwrap();
        let text = match rendered {
            RenderedOutput::Text(s) => s,
            _ => panic!("expected text"),
        };
        let parsed: Result<AnalysisReceipt, _> = serde_json::from_str(&text);
        prop_assert!(parsed.is_ok(), "JSON failed to parse: {:?}", parsed.err());
        let parsed = parsed.unwrap();
        prop_assert_eq!(parsed.schema_version, receipt.schema_version);
        prop_assert_eq!(parsed.mode, receipt.mode);
    }

    /// Markdown output always starts with the expected header.
    #[test]
    fn md_always_starts_with_header(receipt in arb_receipt()) {
        let rendered = render(&receipt, AnalysisFormat::Md).unwrap();
        let text = match rendered {
            RenderedOutput::Text(s) => s,
            _ => panic!("expected text"),
        };
        prop_assert!(text.starts_with("# tokmd analysis\n"));
    }

    /// Markdown output contains the preset name from args.
    #[test]
    fn md_contains_preset(receipt in arb_receipt()) {
        let preset = receipt.args.preset.clone();
        let rendered = render(&receipt, AnalysisFormat::Md).unwrap();
        let text = match rendered {
            RenderedOutput::Text(s) => s,
            _ => panic!("expected text"),
        };
        let expected = format!("Preset: `{}`", preset);
        prop_assert!(text.contains(&expected));
    }

    /// XML output is always well-bracketed.
    #[test]
    fn xml_is_well_bracketed(receipt in arb_receipt()) {
        let rendered = render(&receipt, AnalysisFormat::Xml).unwrap();
        let text = match rendered {
            RenderedOutput::Text(s) => s,
            _ => panic!("expected text"),
        };
        prop_assert!(text.starts_with("<analysis>"));
        prop_assert!(text.ends_with("</analysis>"));
    }

    /// SVG output has valid svg open/close tags.
    #[test]
    fn svg_has_valid_structure(receipt in arb_receipt()) {
        let rendered = render(&receipt, AnalysisFormat::Svg).unwrap();
        let text = match rendered {
            RenderedOutput::Text(s) => s,
            _ => panic!("expected text"),
        };
        prop_assert!(text.starts_with("<svg "));
        prop_assert!(text.ends_with("</svg>"));
    }

    /// JSON-LD output always has schema.org context.
    #[test]
    fn jsonld_has_context(receipt in arb_receipt()) {
        let rendered = render(&receipt, AnalysisFormat::Jsonld).unwrap();
        let text = match rendered {
            RenderedOutput::Text(s) => s,
            _ => panic!("expected text"),
        };
        prop_assert!(text.contains("schema.org"));
        prop_assert!(text.contains("SoftwareSourceCode"));
    }

    /// Mermaid output always starts with graph TD header.
    #[test]
    fn mermaid_always_starts_with_graph_td(receipt in arb_receipt()) {
        let rendered = render(&receipt, AnalysisFormat::Mermaid).unwrap();
        let text = match rendered {
            RenderedOutput::Text(s) => s,
            _ => panic!("expected text"),
        };
        prop_assert!(text.starts_with("graph TD\n"));
    }

    /// When derived is present, MD contains the Totals section.
    #[test]
    fn md_with_derived_has_totals(derived in arb_derived()) {
        let receipt = AnalysisReceipt {
            schema_version: ANALYSIS_SCHEMA_VERSION,
            generated_at_ms: 0,
            tool: ToolInfo { name: "tokmd".into(), version: "test".into() },
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
            derived: Some(derived),
            assets: None,
            deps: None,
            git: None,
            imports: None,
            dup: None,
            complexity: None,
            api_surface: None,
            fun: None,
        };
        let rendered = render(&receipt, AnalysisFormat::Md).unwrap();
        let text = match rendered {
            RenderedOutput::Text(s) => s,
            _ => panic!("expected text"),
        };
        prop_assert!(text.contains("## Totals"));
        prop_assert!(text.contains("## Ratios"));
        prop_assert!(text.contains("## Integrity"));
    }
}
