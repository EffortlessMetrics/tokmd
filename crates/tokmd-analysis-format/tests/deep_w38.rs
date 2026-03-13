//! Deep tests (wave 38) for `tokmd-analysis-format`.
//!
//! Covers Markdown rendering, JSON rendering with schema version,
//! rendering with missing optional fields, rendering with all enrichers
//! populated, section ordering, empty receipt rendering, and more.

use std::collections::BTreeMap;

use tokmd_analysis_format::{RenderedOutput, render};
use tokmd_analysis_types::*;
use tokmd_types::{AnalysisFormat, ScanStatus, ToolInfo};

// ── Helpers ─────────────────────────────────────────────────────

fn minimal_receipt() -> AnalysisReceipt {
    AnalysisReceipt {
        effort: None,
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
            files: 3,
            code: 300,
            comments: 60,
            blanks: 30,
            lines: 390,
            bytes: 3000,
            tokens: 750,
        },
        doc_density: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 60,
                denominator: 360,
                ratio: 0.1667,
            },
            by_lang: vec![RatioRow {
                key: "Rust".into(),
                numerator: 60,
                denominator: 360,
                ratio: 0.1667,
            }],
            by_module: vec![],
        },
        whitespace: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 30,
                denominator: 360,
                ratio: 0.0833,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        verbosity: RateReport {
            total: RateRow {
                key: "total".into(),
                numerator: 3000,
                denominator: 390,
                rate: 7.69,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        max_file: MaxFileReport {
            overall: FileStatRow {
                path: "src/lib.rs".into(),
                module: "src".into(),
                lang: "Rust".into(),
                code: 150,
                comments: 30,
                blanks: 15,
                lines: 195,
                bytes: 1500,
                tokens: 375,
                doc_pct: Some(0.1667),
                bytes_per_line: Some(7.69),
                depth: 1,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        lang_purity: LangPurityReport { rows: vec![] },
        nesting: NestingReport {
            max: 3,
            avg: 1.67,
            by_module: vec![],
        },
        test_density: TestDensityReport {
            test_lines: 50,
            prod_lines: 250,
            test_files: 1,
            prod_files: 2,
            ratio: 0.1667,
        },
        boilerplate: BoilerplateReport {
            infra_lines: 0,
            logic_lines: 390,
            ratio: 0.0,
            infra_langs: vec![],
        },
        polyglot: PolyglotReport {
            lang_count: 1,
            entropy: 0.0,
            dominant_lang: "Rust".into(),
            dominant_lines: 300,
            dominant_pct: 1.0,
        },
        distribution: DistributionReport {
            count: 3,
            min: 65,
            max: 195,
            mean: 130.0,
            median: 130.0,
            p90: 195.0,
            p99: 195.0,
            gini: 0.25,
        },
        histogram: vec![HistogramBucket {
            label: "Small".into(),
            min: 0,
            max: Some(200),
            files: 3,
            pct: 1.0,
        }],
        top: TopOffenders {
            largest_lines: vec![],
            largest_tokens: vec![],
            largest_bytes: vec![],
            least_documented: vec![],
            most_dense: vec![],
        },
        tree: None,
        reading_time: ReadingTimeReport {
            minutes: 15.0,
            lines_per_minute: 20,
            basis_lines: 300,
        },
        context_window: None,
        cocomo: Some(CocomoReport {
            mode: "organic".into(),
            kloc: 0.3,
            effort_pm: 0.69,
            duration_months: 2.12,
            staff: 0.33,
            a: 2.4,
            b: 1.05,
            c: 2.5,
            d: 0.38,
        }),
        todo: None,
        integrity: IntegrityReport {
            algo: "blake3".into(),
            hash: "a".repeat(64),
            entries: 3,
        },
    }
}

fn text(output: RenderedOutput) -> String {
    match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    }
}

// ── Markdown rendering ──────────────────────────────────────────

mod md_rendering {
    use super::*;

    #[test]
    fn md_starts_with_header() {
        let r = minimal_receipt();
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(md.starts_with("# tokmd analysis"));
    }

    #[test]
    fn md_contains_preset() {
        let r = minimal_receipt();
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(md.contains("Preset: `receipt`"));
    }

    #[test]
    fn md_contains_inputs_section() {
        let r = minimal_receipt();
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(md.contains("## Inputs"));
        assert!(md.contains("- `.`"));
    }

    #[test]
    fn md_with_derived_contains_totals() {
        let mut r = minimal_receipt();
        r.derived = Some(sample_derived());
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(md.contains("## Totals"));
        assert!(md.contains("|Files|Code|Comments|Blanks|Lines|Bytes|Tokens|"));
    }

    #[test]
    fn md_with_derived_contains_ratios() {
        let mut r = minimal_receipt();
        r.derived = Some(sample_derived());
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(md.contains("## Ratios"));
        assert!(md.contains("|Doc density|"));
        assert!(md.contains("|Whitespace ratio|"));
        assert!(md.contains("|Bytes per line|"));
    }

    #[test]
    fn md_with_derived_contains_distribution() {
        let mut r = minimal_receipt();
        r.derived = Some(sample_derived());
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(md.contains("## Distribution"));
        assert!(md.contains("|Count|Min|Max|Mean|Median|P90|P99|Gini|"));
    }

    #[test]
    fn md_with_derived_contains_histogram() {
        let mut r = minimal_receipt();
        r.derived = Some(sample_derived());
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(md.contains("## File size histogram"));
    }

    #[test]
    fn md_with_cocomo() {
        let mut r = minimal_receipt();
        r.derived = Some(sample_derived());
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(md.contains("## Effort estimate"));
        assert!(md.contains("### Size basis"));
        assert!(md.contains("### Headline"));
        assert!(md.contains("### Why"));
        assert!(md.contains("### Delta"));
        assert!(md.contains("Model: `COCOMO` (`organic` mode)"));
    }

    #[test]
    fn md_without_cocomo_omits_section() {
        let mut r = minimal_receipt();
        let mut derived = sample_derived();
        derived.cocomo = None;
        r.derived = Some(derived);
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(!md.contains("## Effort estimate"));
    }

    #[test]
    fn md_with_reading_time() {
        let mut r = minimal_receipt();
        r.derived = Some(sample_derived());
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(md.contains("## Reading time"));
        assert!(md.contains("lines/min"));
    }

    #[test]
    fn md_with_test_density() {
        let mut r = minimal_receipt();
        r.derived = Some(sample_derived());
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(md.contains("## Test density"));
        assert!(md.contains("Test lines: `50`"));
        assert!(md.contains("Prod lines: `250`"));
    }

    #[test]
    fn md_with_boilerplate() {
        let mut r = minimal_receipt();
        r.derived = Some(sample_derived());
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(md.contains("## Boilerplate ratio"));
    }

    #[test]
    fn md_with_polyglot() {
        let mut r = minimal_receipt();
        r.derived = Some(sample_derived());
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(md.contains("## Polyglot"));
        assert!(md.contains("Dominant: `Rust`"));
    }

    #[test]
    fn md_with_integrity() {
        let mut r = minimal_receipt();
        r.derived = Some(sample_derived());
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(md.contains("## Integrity"));
        assert!(md.contains("`blake3`"));
    }
}

// ── JSON rendering ──────────────────────────────────────────────

mod json_rendering {
    use super::*;

    #[test]
    fn json_contains_schema_version() {
        let r = minimal_receipt();
        let json = text(render(&r, AnalysisFormat::Json).unwrap());
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(
            parsed["schema_version"].as_u64().unwrap(),
            ANALYSIS_SCHEMA_VERSION as u64
        );
    }

    #[test]
    fn json_roundtrips_minimal_receipt() {
        let r = minimal_receipt();
        let json = text(render(&r, AnalysisFormat::Json).unwrap());
        let parsed: AnalysisReceipt = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.mode, "analyze");
        assert!(parsed.derived.is_none());
        assert!(parsed.archetype.is_none());
    }

    #[test]
    fn json_roundtrips_with_derived() {
        let mut r = minimal_receipt();
        r.derived = Some(sample_derived());
        let json = text(render(&r, AnalysisFormat::Json).unwrap());
        let parsed: AnalysisReceipt = serde_json::from_str(&json).unwrap();
        let d = parsed.derived.unwrap();
        assert_eq!(d.totals.files, 3);
        assert_eq!(d.totals.code, 300);
    }

    #[test]
    fn json_is_pretty_printed() {
        let r = minimal_receipt();
        let json = text(render(&r, AnalysisFormat::Json).unwrap());
        assert!(json.contains('\n'));
        assert!(json.contains("  "));
    }

    #[test]
    fn json_null_fields_for_none_options() {
        let r = minimal_receipt();
        let json = text(render(&r, AnalysisFormat::Json).unwrap());
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["derived"].is_null());
        assert!(parsed["archetype"].is_null());
        assert!(parsed["git"].is_null());
    }
}

// ── Missing optional fields ─────────────────────────────────────

mod missing_fields {
    use super::*;

    #[test]
    fn md_no_archetype_section() {
        let r = minimal_receipt();
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(!md.contains("## Archetype"));
    }

    #[test]
    fn md_no_topics_section() {
        let r = minimal_receipt();
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(!md.contains("## Topics"));
    }

    #[test]
    fn md_no_entropy_section() {
        let r = minimal_receipt();
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(!md.contains("## Entropy profiling"));
    }

    #[test]
    fn md_no_git_section() {
        let r = minimal_receipt();
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(!md.contains("## Git metrics"));
    }

    #[test]
    fn md_no_imports_section() {
        let r = minimal_receipt();
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(!md.contains("## Imports"));
    }

    #[test]
    fn md_no_assets_section() {
        let r = minimal_receipt();
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(!md.contains("## Assets"));
    }

    #[test]
    fn md_no_deps_section() {
        let r = minimal_receipt();
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(!md.contains("## Dependencies"));
    }

    #[test]
    fn md_no_context_window_when_absent() {
        let mut r = minimal_receipt();
        let mut derived = sample_derived();
        derived.context_window = None;
        r.derived = Some(derived);
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(!md.contains("## Context window"));
    }
}

// ── All enrichers populated ─────────────────────────────────────

mod all_enrichers {
    use super::*;

    fn fully_populated_receipt() -> AnalysisReceipt {
        let mut r = minimal_receipt();
        r.archetype = Some(Archetype {
            kind: "cli-tool".into(),
            evidence: vec!["main.rs".into(), "clap".into()],
        });
        r.topics = Some(TopicClouds {
            overall: vec![TopicTerm {
                term: "parser".into(),
                score: 0.85,
                tf: 10,
                df: 3,
            }],
            per_module: BTreeMap::new(),
        });
        r.entropy = Some(EntropyReport { suspects: vec![] });
        r.license = Some(LicenseReport {
            effective: Some("MIT".into()),
            findings: vec![],
        });
        r.assets = Some(AssetReport {
            total_files: 2,
            total_bytes: 1024,
            categories: vec![AssetCategoryRow {
                category: "image".into(),
                files: 2,
                bytes: 1024,
                extensions: vec!["png".into()],
            }],
            top_files: vec![],
        });
        r.deps = Some(DependencyReport {
            total: 10,
            lockfiles: vec![LockfileReport {
                path: "Cargo.lock".into(),
                kind: "cargo".into(),
                dependencies: 10,
            }],
        });
        r.derived = Some(sample_derived());
        r
    }

    #[test]
    fn md_contains_archetype_section() {
        let r = fully_populated_receipt();
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(md.contains("## Archetype"));
        assert!(md.contains("Kind: `cli-tool`"));
        assert!(md.contains("Evidence: `main.rs`, `clap`"));
    }

    #[test]
    fn md_contains_topics_section() {
        let r = fully_populated_receipt();
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(md.contains("## Topics"));
        assert!(md.contains("parser"));
    }

    #[test]
    fn md_contains_entropy_section() {
        let r = fully_populated_receipt();
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(md.contains("## Entropy profiling"));
        assert!(md.contains("No entropy outliers detected."));
    }

    #[test]
    fn md_contains_license_section() {
        let r = fully_populated_receipt();
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(md.contains("## License radar"));
        assert!(md.contains("Effective: `MIT`"));
    }

    #[test]
    fn md_contains_assets_section() {
        let r = fully_populated_receipt();
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(md.contains("## Assets"));
        assert!(md.contains("Total files: `2`"));
        assert!(md.contains("Total bytes: `1024`"));
    }

    #[test]
    fn md_contains_deps_section() {
        let r = fully_populated_receipt();
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(md.contains("## Dependencies"));
        assert!(md.contains("Total: `10`"));
        assert!(md.contains("Cargo.lock"));
    }

    #[test]
    fn json_roundtrips_fully_populated() {
        let r = fully_populated_receipt();
        let json = text(render(&r, AnalysisFormat::Json).unwrap());
        let parsed: AnalysisReceipt = serde_json::from_str(&json).unwrap();
        assert!(parsed.archetype.is_some());
        assert!(parsed.topics.is_some());
        assert!(parsed.entropy.is_some());
        assert!(parsed.license.is_some());
        assert!(parsed.assets.is_some());
        assert!(parsed.deps.is_some());
        assert!(parsed.derived.is_some());
    }
}

// ── Section ordering ────────────────────────────────────────────

mod section_ordering {
    use super::*;

    #[test]
    fn md_inputs_before_archetype() {
        let mut r = minimal_receipt();
        r.archetype = Some(Archetype {
            kind: "library".into(),
            evidence: vec![],
        });
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        let inputs_pos = md.find("## Inputs").unwrap();
        let arch_pos = md.find("## Archetype").unwrap();
        assert!(inputs_pos < arch_pos);
    }

    #[test]
    fn md_totals_before_ratios() {
        let mut r = minimal_receipt();
        r.derived = Some(sample_derived());
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        let totals_pos = md.find("## Totals").unwrap();
        let ratios_pos = md.find("## Ratios").unwrap();
        assert!(totals_pos < ratios_pos);
    }

    #[test]
    fn md_distribution_before_histogram() {
        let mut r = minimal_receipt();
        r.derived = Some(sample_derived());
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        let dist_pos = md.find("## Distribution").unwrap();
        let hist_pos = md.find("## File size histogram").unwrap();
        assert!(dist_pos < hist_pos);
    }

    #[test]
    fn md_top_offenders_after_histogram() {
        let mut r = minimal_receipt();
        r.derived = Some(sample_derived());
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        let hist_pos = md.find("## File size histogram").unwrap();
        let top_pos = md.find("## Top offenders").unwrap();
        assert!(hist_pos < top_pos);
    }

    #[test]
    fn md_assets_after_derived_integrity() {
        let mut r = minimal_receipt();
        r.derived = Some(sample_derived());
        r.assets = Some(AssetReport {
            total_files: 0,
            total_bytes: 0,
            categories: vec![],
            top_files: vec![],
        });
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        let integrity_pos = md.find("## Integrity").unwrap();
        let assets_pos = md.find("## Assets").unwrap();
        assert!(integrity_pos < assets_pos);
    }
}

// ── Empty receipt rendering ─────────────────────────────────────

mod empty_receipt {
    use super::*;

    #[test]
    fn md_renders_without_error() {
        let r = minimal_receipt();
        let result = render(&r, AnalysisFormat::Md);
        assert!(result.is_ok());
    }

    #[test]
    fn json_renders_without_error() {
        let r = minimal_receipt();
        let result = render(&r, AnalysisFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn xml_renders_without_error() {
        let r = minimal_receipt();
        let result = render(&r, AnalysisFormat::Xml);
        assert!(result.is_ok());
    }

    #[test]
    fn svg_renders_without_error() {
        let r = minimal_receipt();
        let result = render(&r, AnalysisFormat::Svg);
        assert!(result.is_ok());
    }

    #[test]
    fn mermaid_renders_without_error() {
        let r = minimal_receipt();
        let result = render(&r, AnalysisFormat::Mermaid);
        assert!(result.is_ok());
    }

    #[test]
    fn tree_renders_without_error() {
        let r = minimal_receipt();
        let result = render(&r, AnalysisFormat::Tree);
        assert!(result.is_ok());
    }

    #[test]
    fn html_renders_without_error() {
        let r = minimal_receipt();
        let result = render(&r, AnalysisFormat::Html);
        assert!(result.is_ok());
    }

    #[test]
    fn jsonld_renders_without_error() {
        let r = minimal_receipt();
        let result = render(&r, AnalysisFormat::Jsonld);
        assert!(result.is_ok());
    }
}

// ── Context window in MD ────────────────────────────────────────

mod context_window {
    use super::*;

    #[test]
    fn md_shows_context_window_when_present() {
        let mut r = minimal_receipt();
        let mut derived = sample_derived();
        derived.context_window = Some(ContextWindowReport {
            window_tokens: 128000,
            total_tokens: 750,
            pct: 0.0059,
            fits: true,
        });
        r.derived = Some(derived);
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(md.contains("## Context window"));
        assert!(md.contains("Window tokens: `128000`"));
        assert!(md.contains("Fits: `true`"));
    }
}

// ── TODO section rendering ──────────────────────────────────────

mod todo_section {
    use super::*;

    #[test]
    fn md_shows_todos_when_present() {
        let mut r = minimal_receipt();
        let mut derived = sample_derived();
        derived.todo = Some(TodoReport {
            total: 5,
            density_per_kloc: 16.67,
            tags: vec![
                TodoTagRow {
                    tag: "TODO".into(),
                    count: 3,
                },
                TodoTagRow {
                    tag: "FIXME".into(),
                    count: 2,
                },
            ],
        });
        r.derived = Some(derived);
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(md.contains("## TODOs"));
        assert!(md.contains("Total: `5`"));
        assert!(md.contains("|TODO|3|"));
        assert!(md.contains("|FIXME|2|"));
    }

    #[test]
    fn md_omits_todos_when_absent() {
        let mut r = minimal_receipt();
        r.derived = Some(sample_derived());
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(!md.contains("## TODOs"));
    }
}

// ── Doc density by language rendering ───────────────────────────

mod doc_density_rendering {
    use super::*;

    #[test]
    fn md_doc_density_by_lang_table() {
        let mut r = minimal_receipt();
        r.derived = Some(sample_derived());
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(md.contains("### Doc density by language"));
        assert!(md.contains("|Lang|Doc%|Comments|Code|"));
        assert!(md.contains("|Rust|"));
    }

    #[test]
    fn md_whitespace_by_lang_table() {
        let mut r = minimal_receipt();
        let mut derived = sample_derived();
        derived.whitespace.by_lang = vec![RatioRow {
            key: "Go".into(),
            numerator: 10,
            denominator: 100,
            ratio: 0.1,
        }];
        r.derived = Some(derived);
        let md = text(render(&r, AnalysisFormat::Md).unwrap());
        assert!(md.contains("### Whitespace ratio by language"));
        assert!(md.contains("|Go|"));
    }
}
