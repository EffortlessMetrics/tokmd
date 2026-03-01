//! Extended tests for analysis format rendering across multiple formats and edge cases.
//!
//! Covers: JSON, JSON-LD, XML, SVG, Mermaid, Tree, Markdown with various
//! receipt field combinations (archetype, topics, entropy, license, git,
//! imports, complexity, assets, deps, api_surface, fun eco-label).

use std::collections::BTreeMap;

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
            by_lang: vec![
                RatioRow {
                    key: "Rust".into(),
                    numerator: 70,
                    denominator: 470,
                    ratio: 0.1489,
                },
                RatioRow {
                    key: "TOML".into(),
                    numerator: 10,
                    denominator: 110,
                    ratio: 0.0909,
                },
            ],
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
            by_lang: vec![RateRow {
                key: "Rust".into(),
                numerator: 4500,
                denominator: 550,
                rate: 8.18,
            }],
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

// ---------------------------------------------------------------------------
// JSON format tests
// ---------------------------------------------------------------------------

#[test]
fn json_roundtrip_minimal() {
    let receipt = minimal_receipt();
    let output = render(&receipt, AnalysisFormat::Json).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    let parsed: AnalysisReceipt = serde_json::from_str(&text).unwrap();
    assert_eq!(parsed.schema_version, receipt.schema_version);
    assert_eq!(parsed.mode, "analyze");
    assert!(parsed.derived.is_none());
}

#[test]
fn json_roundtrip_with_derived() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let output = render(&receipt, AnalysisFormat::Json).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    let parsed: AnalysisReceipt = serde_json::from_str(&text).unwrap();
    let d = parsed.derived.unwrap();
    assert_eq!(d.totals.files, 5);
    assert_eq!(d.totals.code, 500);
}

#[test]
fn json_includes_all_optional_sections() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    receipt.archetype = Some(Archetype {
        kind: "library".into(),
        evidence: vec!["Cargo.toml".into()],
    });
    receipt.imports = Some(ImportReport {
        granularity: "module".into(),
        edges: vec![ImportEdge {
            from: "src/a".into(),
            to: "src/b".into(),
            count: 3,
        }],
    });
    let output = render(&receipt, AnalysisFormat::Json).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("\"archetype\""));
    assert!(text.contains("\"imports\""));
    assert!(text.contains("\"library\""));
}

// ---------------------------------------------------------------------------
// JSON-LD format tests
// ---------------------------------------------------------------------------

#[test]
fn jsonld_has_schema_context() {
    let receipt = minimal_receipt();
    let output = render(&receipt, AnalysisFormat::Jsonld).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("\"@context\": \"https://schema.org\""));
    assert!(text.contains("\"@type\": \"SoftwareSourceCode\""));
}

#[test]
fn jsonld_reflects_totals() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let output = render(&receipt, AnalysisFormat::Jsonld).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("\"codeLines\": 500"));
    assert!(text.contains("\"lineCount\": 620"));
    assert!(text.contains("\"fileSize\": 5000"));
}

#[test]
fn jsonld_zero_totals_when_no_derived() {
    let receipt = minimal_receipt();
    let output = render(&receipt, AnalysisFormat::Jsonld).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("\"codeLines\": 0"));
}

// ---------------------------------------------------------------------------
// XML format tests
// ---------------------------------------------------------------------------

#[test]
fn xml_with_derived_has_all_attributes() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let output = render(&receipt, AnalysisFormat::Xml).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("files=\"5\""));
    assert!(text.contains("code=\"500\""));
    assert!(text.contains("comments=\"80\""));
    assert!(text.contains("blanks=\"40\""));
    assert!(text.contains("lines=\"620\""));
    assert!(text.contains("bytes=\"5000\""));
    assert!(text.contains("tokens=\"1200\""));
}

#[test]
fn xml_empty_without_derived() {
    let receipt = minimal_receipt();
    let output = render(&receipt, AnalysisFormat::Xml).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert_eq!(text, "<analysis></analysis>");
}

// ---------------------------------------------------------------------------
// SVG format tests
// ---------------------------------------------------------------------------

#[test]
fn svg_with_context_window_shows_pct() {
    let mut receipt = minimal_receipt();
    let mut d = sample_derived();
    d.context_window = Some(ContextWindowReport {
        window_tokens: 128_000,
        total_tokens: 1200,
        pct: 0.009375,
        fits: true,
    });
    receipt.derived = Some(d);
    let output = render(&receipt, AnalysisFormat::Svg).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("context"));
    assert!(text.contains("0.9%"));
}

#[test]
fn svg_without_context_window_shows_tokens() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let output = render(&receipt, AnalysisFormat::Svg).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("tokens"));
    assert!(text.contains("1200"));
}

// ---------------------------------------------------------------------------
// Mermaid format tests
// ---------------------------------------------------------------------------

#[test]
fn mermaid_with_multiple_edges() {
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
    let output = render(&receipt, AnalysisFormat::Mermaid).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.starts_with("graph TD\n"));
    assert!(text.contains("mod_a -->|2| mod_b"));
    assert!(text.contains("mod_b -->|7| mod_c"));
    assert!(text.contains("mod_a -->|1| mod_c"));
}

#[test]
fn mermaid_sanitizes_special_chars() {
    let mut receipt = minimal_receipt();
    receipt.imports = Some(ImportReport {
        granularity: "file".into(),
        edges: vec![ImportEdge {
            from: "src/foo.rs".into(),
            to: "src/bar-baz.rs".into(),
            count: 1,
        }],
    });
    let output = render(&receipt, AnalysisFormat::Mermaid).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    // Slashes and dashes become underscores
    assert!(text.contains("src_foo_rs"));
    assert!(text.contains("src_bar_baz_rs"));
}

// ---------------------------------------------------------------------------
// Tree format tests
// ---------------------------------------------------------------------------

#[test]
fn tree_with_tree_string() {
    let mut receipt = minimal_receipt();
    let mut d = sample_derived();
    d.tree = Some("root\n  src/\n    lib.rs".into());
    receipt.derived = Some(d);
    let output = render(&receipt, AnalysisFormat::Tree).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("root"));
    assert!(text.contains("lib.rs"));
}

#[test]
fn tree_unavailable_fallback() {
    let receipt = minimal_receipt();
    let output = render(&receipt, AnalysisFormat::Tree).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert_eq!(text, "(tree unavailable)");
}

// ---------------------------------------------------------------------------
// Markdown – optional sections rendering
// ---------------------------------------------------------------------------

#[test]
fn md_renders_archetype_section() {
    let mut receipt = minimal_receipt();
    receipt.archetype = Some(Archetype {
        kind: "monorepo".into(),
        evidence: vec!["Cargo.toml".into(), "package.json".into()],
    });
    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("## Archetype"));
    assert!(text.contains("monorepo"));
    assert!(text.contains("Cargo.toml`, `package.json"));
}

#[test]
fn md_renders_topics_section() {
    let mut receipt = minimal_receipt();
    receipt.topics = Some(TopicClouds {
        overall: vec![
            TopicTerm {
                term: "async".into(),
                score: 0.9,
                tf: 5,
                df: 3,
            },
            TopicTerm {
                term: "http".into(),
                score: 0.7,
                tf: 3,
                df: 2,
            },
        ],
        per_module: BTreeMap::new(),
    });
    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("## Topics"));
    assert!(text.contains("async, http"));
}

#[test]
fn md_renders_entropy_section_with_suspects() {
    let mut receipt = minimal_receipt();
    receipt.entropy = Some(EntropyReport {
        suspects: vec![EntropyFinding {
            path: "secrets.bin".into(),
            module: "data".into(),
            entropy_bits_per_byte: 7.8,
            sample_bytes: 1024,
            class: EntropyClass::High,
        }],
    });
    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("## Entropy profiling"));
    assert!(text.contains("secrets.bin"));
    assert!(text.contains("High"));
}

#[test]
fn md_renders_entropy_section_no_suspects() {
    let mut receipt = minimal_receipt();
    receipt.entropy = Some(EntropyReport { suspects: vec![] });
    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("No entropy outliers detected"));
}

#[test]
fn md_renders_license_section() {
    let mut receipt = minimal_receipt();
    receipt.license = Some(LicenseReport {
        effective: Some("MIT".into()),
        findings: vec![LicenseFinding {
            spdx: "MIT".into(),
            confidence: 0.95,
            source_path: "LICENSE".into(),
            source_kind: LicenseSourceKind::Text,
        }],
    });
    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("## License radar"));
    assert!(text.contains("Effective: `MIT`"));
    assert!(text.contains("0.95"));
}

#[test]
fn md_renders_corporate_fingerprint() {
    let mut receipt = minimal_receipt();
    receipt.corporate_fingerprint = Some(CorporateFingerprint {
        domains: vec![
            DomainStat {
                domain: "example.com".into(),
                commits: 100,
                pct: 0.8,
            },
            DomainStat {
                domain: "other.org".into(),
                commits: 25,
                pct: 0.2,
            },
        ],
    });
    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("## Corporate fingerprint"));
    assert!(text.contains("example.com"));
    assert!(text.contains("other.org"));
}

#[test]
fn md_renders_corporate_fingerprint_empty() {
    let mut receipt = minimal_receipt();
    receipt.corporate_fingerprint = Some(CorporateFingerprint { domains: vec![] });
    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("No commit domains detected"));
}

#[test]
fn md_renders_predictive_churn() {
    let mut receipt = minimal_receipt();
    let mut per_module = BTreeMap::new();
    per_module.insert(
        "src".into(),
        ChurnTrend {
            slope: 1.5,
            r2: 0.85,
            recent_change: 3,
            classification: TrendClass::Rising,
        },
    );
    receipt.predictive_churn = Some(PredictiveChurnReport { per_module });
    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("## Predictive churn"));
    assert!(text.contains("src"));
    assert!(text.contains("Rising"));
}

#[test]
fn md_renders_assets_section() {
    let mut receipt = minimal_receipt();
    receipt.assets = Some(AssetReport {
        total_files: 2,
        total_bytes: 5000,
        categories: vec![AssetCategoryRow {
            category: "images".into(),
            files: 2,
            bytes: 5000,
            extensions: vec!["png".into(), "jpg".into()],
        }],
        top_files: vec![AssetFileRow {
            path: "logo.png".into(),
            bytes: 3000,
            category: "images".into(),
            extension: "png".into(),
        }],
    });
    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("## Assets"));
    assert!(text.contains("Total files: `2`"));
    assert!(text.contains("images"));
    assert!(text.contains("logo.png"));
}

#[test]
fn md_renders_deps_section() {
    let mut receipt = minimal_receipt();
    receipt.deps = Some(DependencyReport {
        total: 42,
        lockfiles: vec![LockfileReport {
            path: "Cargo.lock".into(),
            kind: "cargo".into(),
            dependencies: 42,
        }],
    });
    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("## Dependencies"));
    assert!(text.contains("Total: `42`"));
    assert!(text.contains("Cargo.lock"));
}

#[test]
fn md_renders_complexity_section() {
    let mut receipt = minimal_receipt();
    receipt.complexity = Some(ComplexityReport {
        total_functions: 50,
        avg_function_length: 15.5,
        max_function_length: 120,
        avg_cyclomatic: 3.2,
        max_cyclomatic: 25,
        avg_cognitive: Some(4.5),
        max_cognitive: Some(30),
        avg_nesting_depth: Some(2.1),
        max_nesting_depth: Some(6),
        high_risk_files: 2,
        histogram: None,
        halstead: None,
        maintainability_index: None,
        technical_debt: None,
        files: vec![FileComplexity {
            path: "src/complex.rs".into(),
            module: "src".into(),
            function_count: 10,
            max_function_length: 120,
            cyclomatic_complexity: 25,
            cognitive_complexity: Some(30),
            max_nesting: Some(6),
            risk_level: ComplexityRisk::High,
            functions: None,
        }],
    });
    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("## Complexity"));
    assert!(text.contains("|Total functions|50|"));
    assert!(text.contains("|Avg cognitive|4.50|"));
    assert!(text.contains("|Max cognitive|30|"));
    assert!(text.contains("|Avg nesting depth|2.10|"));
    assert!(text.contains("|Max nesting depth|6|"));
    assert!(text.contains("src/complex.rs"));
}

#[test]
fn md_renders_complexity_without_optional_fields() {
    let mut receipt = minimal_receipt();
    receipt.complexity = Some(ComplexityReport {
        total_functions: 10,
        avg_function_length: 8.0,
        max_function_length: 30,
        avg_cyclomatic: 2.0,
        max_cyclomatic: 8,
        avg_cognitive: None,
        max_cognitive: None,
        avg_nesting_depth: None,
        max_nesting_depth: None,
        high_risk_files: 0,
        histogram: None,
        halstead: None,
        maintainability_index: None,
        technical_debt: None,
        files: vec![],
    });
    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("## Complexity"));
    // Optional fields should not appear
    assert!(!text.contains("Avg cognitive"));
    assert!(!text.contains("Max cognitive"));
    assert!(!text.contains("Avg nesting depth"));
    assert!(!text.contains("Max nesting depth"));
}

#[test]
fn md_renders_api_surface() {
    let mut receipt = minimal_receipt();
    let mut by_language = BTreeMap::new();
    by_language.insert(
        "Rust".into(),
        LangApiSurface {
            total_items: 100,
            public_items: 40,
            internal_items: 60,
            public_ratio: 0.4,
        },
    );
    receipt.api_surface = Some(ApiSurfaceReport {
        total_items: 100,
        public_items: 40,
        internal_items: 60,
        public_ratio: 0.4,
        documented_ratio: 0.75,
        by_language,
        by_module: vec![ModuleApiRow {
            module: "src".into(),
            total_items: 80,
            public_items: 30,
            public_ratio: 0.375,
        }],
        top_exporters: vec![ApiExportItem {
            path: "src/lib.rs".into(),
            lang: "Rust".into(),
            public_items: 20,
            total_items: 40,
        }],
    });
    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("## API surface"));
    assert!(text.contains("|Total items|100|"));
    assert!(text.contains("|Public ratio|40.0%|"));
    assert!(text.contains("### By language"));
    assert!(text.contains("|Rust|100|40|60|40.0%|"));
    assert!(text.contains("### By module"));
    assert!(text.contains("### Top exporters"));
}

#[test]
fn md_renders_git_section() {
    let mut receipt = minimal_receipt();
    receipt.git = Some(GitReport {
        commits_scanned: 100,
        files_seen: 50,
        hotspots: vec![HotspotRow {
            path: "src/hot.rs".into(),
            commits: 30,
            lines: 200,
            score: 6000,
        }],
        bus_factor: vec![BusFactorRow {
            module: "src".into(),
            authors: 3,
        }],
        freshness: FreshnessReport {
            threshold_days: 90,
            stale_files: 10,
            total_files: 50,
            stale_pct: 0.2,
            by_module: vec![],
        },
        coupling: vec![],
        age_distribution: None,
        intent: None,
    });
    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("## Git metrics"));
    assert!(text.contains("Commits scanned: `100`"));
    assert!(text.contains("### Hotspots"));
    assert!(text.contains("src/hot.rs"));
    assert!(text.contains("### Bus factor"));
    assert!(text.contains("### Freshness"));
}

#[test]
fn md_renders_imports_section() {
    let mut receipt = minimal_receipt();
    receipt.imports = Some(ImportReport {
        granularity: "file".into(),
        edges: vec![
            ImportEdge {
                from: "src/a.rs".into(),
                to: "src/b.rs".into(),
                count: 3,
            },
            ImportEdge {
                from: "src/b.rs".into(),
                to: "src/c.rs".into(),
                count: 1,
            },
        ],
    });
    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("## Imports"));
    assert!(text.contains("Granularity: `file`"));
    assert!(text.contains("|src/a.rs|src/b.rs|3|"));
}

#[test]
fn md_renders_dup_section() {
    let mut receipt = minimal_receipt();
    receipt.dup = Some(DuplicateReport {
        groups: vec![DuplicateGroup {
            hash: "abc123".into(),
            bytes: 500,
            files: vec!["a.rs".into(), "b.rs".into()],
        }],
        wasted_bytes: 500,
        strategy: "blake3".into(),
        density: None,
        near: None,
    });
    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("## Duplicates"));
    assert!(text.contains("Wasted bytes: `500`"));
    assert!(text.contains("abc123"));
}

#[test]
fn md_renders_eco_label() {
    let mut receipt = minimal_receipt();
    receipt.fun = Some(FunReport {
        eco_label: Some(EcoLabel {
            score: 85.0,
            label: "A".into(),
            bytes: 5000,
            notes: "Clean and green".into(),
        }),
    });
    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("## Eco label"));
    assert!(text.contains("Label: `A`"));
    assert!(text.contains("Score: `85.0`"));
}

// ---------------------------------------------------------------------------
// Markdown – derived report sub-sections
// ---------------------------------------------------------------------------

#[test]
fn md_derived_cocomo_present() {
    let mut receipt = minimal_receipt();
    let mut d = sample_derived();
    d.cocomo = Some(CocomoReport {
        mode: "organic".into(),
        kloc: 0.5,
        effort_pm: 1.08,
        duration_months: 2.0,
        staff: 0.54,
        a: 2.4,
        b: 1.05,
        c: 2.5,
        d: 0.38,
    });
    receipt.derived = Some(d);
    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("## COCOMO estimate"));
    assert!(text.contains("Mode: `organic`"));
    assert!(text.contains("KLOC: `0.5000`"));
}

#[test]
fn md_derived_context_window_present() {
    let mut receipt = minimal_receipt();
    let mut d = sample_derived();
    d.context_window = Some(ContextWindowReport {
        window_tokens: 100_000,
        total_tokens: 1200,
        pct: 0.012,
        fits: true,
    });
    receipt.derived = Some(d);
    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("## Context window"));
    assert!(text.contains("Fits: `true`"));
}

#[test]
fn md_derived_todo_present() {
    let mut receipt = minimal_receipt();
    let mut d = sample_derived();
    d.todo = Some(TodoReport {
        total: 12,
        density_per_kloc: 24.0,
        tags: vec![
            TodoTagRow {
                tag: "TODO".into(),
                count: 8,
            },
            TodoTagRow {
                tag: "FIXME".into(),
                count: 4,
            },
        ],
    });
    receipt.derived = Some(d);
    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("## TODOs"));
    assert!(text.contains("Total: `12`"));
    assert!(text.contains("|TODO|8|"));
    assert!(text.contains("|FIXME|4|"));
}

#[test]
fn md_derived_doc_density_by_lang() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("### Doc density by language"));
    assert!(text.contains("|Rust|14.9%|"));
    assert!(text.contains("|TOML|9.1%|"));
}

#[test]
fn md_derived_verbosity_by_lang() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("### Verbosity by language"));
    assert!(text.contains("|Rust|8.18|4500|550|"));
}

// ---------------------------------------------------------------------------
// Edge cases: empty inputs, warnings
// ---------------------------------------------------------------------------

#[test]
fn md_no_inputs_skips_section() {
    let mut receipt = minimal_receipt();
    receipt.source.inputs.clear();
    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(!text.contains("## Inputs"));
}

#[test]
fn md_multiple_inputs_listed() {
    let mut receipt = minimal_receipt();
    receipt.source.inputs = vec!["src/".into(), "lib/".into(), "tests/".into()];
    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.contains("- `src/`"));
    assert!(text.contains("- `lib/`"));
    assert!(text.contains("- `tests/`"));
}

#[test]
fn md_histogram_unbounded_max() {
    let mut receipt = minimal_receipt();
    let mut d = sample_derived();
    d.histogram = vec![HistogramBucket {
        label: "Huge".into(),
        min: 1001,
        max: None,
        files: 1,
        pct: 1.0,
    }];
    receipt.derived = Some(d);
    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    // Unbounded max renders as ∞
    assert!(text.contains("|Huge|1001|∞|1|100.0%|"));
}

// ---------------------------------------------------------------------------
// Format dispatch: all formats produce output without panicking
// ---------------------------------------------------------------------------

#[test]
fn all_text_formats_succeed_on_minimal_receipt() {
    let receipt = minimal_receipt();
    for fmt in [
        AnalysisFormat::Md,
        AnalysisFormat::Json,
        AnalysisFormat::Jsonld,
        AnalysisFormat::Xml,
        AnalysisFormat::Svg,
        AnalysisFormat::Mermaid,
        AnalysisFormat::Tree,
        AnalysisFormat::Html,
    ] {
        let result = render(&receipt, fmt);
        assert!(
            result.is_ok(),
            "format {:?} failed: {:?}",
            fmt,
            result.err()
        );
        match result.unwrap() {
            RenderedOutput::Text(s) => {
                assert!(!s.is_empty(), "format {:?} produced empty output", fmt)
            }
            RenderedOutput::Binary(_) => panic!("format {:?} should produce text", fmt),
        }
    }
}

#[test]
fn all_text_formats_succeed_on_full_receipt() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    receipt.archetype = Some(Archetype {
        kind: "cli-tool".into(),
        evidence: vec!["main.rs".into()],
    });
    receipt.imports = Some(ImportReport {
        granularity: "module".into(),
        edges: vec![ImportEdge {
            from: "a".into(),
            to: "b".into(),
            count: 1,
        }],
    });
    receipt.fun = Some(FunReport {
        eco_label: Some(EcoLabel {
            score: 50.0,
            label: "C".into(),
            bytes: 100,
            notes: "ok".into(),
        }),
    });

    for fmt in [
        AnalysisFormat::Md,
        AnalysisFormat::Json,
        AnalysisFormat::Jsonld,
        AnalysisFormat::Xml,
        AnalysisFormat::Svg,
        AnalysisFormat::Mermaid,
        AnalysisFormat::Tree,
        AnalysisFormat::Html,
    ] {
        let result = render(&receipt, fmt);
        assert!(result.is_ok(), "format {:?} failed on full receipt", fmt);
    }
}

// ---------------------------------------------------------------------------
// OBJ/MIDI without fun feature should error
// ---------------------------------------------------------------------------

#[cfg(not(feature = "fun"))]
#[test]
fn obj_format_requires_fun_feature() {
    let receipt = minimal_receipt();
    let result = render(&receipt, AnalysisFormat::Obj);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.to_string().contains("fun"));
}

#[cfg(not(feature = "fun"))]
#[test]
fn midi_format_requires_fun_feature() {
    let receipt = minimal_receipt();
    let result = render(&receipt, AnalysisFormat::Midi);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.to_string().contains("fun"));
}
