//! Deep tests for tokmd-analysis-format rendering.
//!
//! Covers: determinism, JSON round-trip fidelity, schema version presence,
//! Unicode handling, large data sets, partial/full section rendering,
//! edge cases for near-dup/coupling/intent/complexity/API surface.

use std::collections::BTreeMap;
use tokmd_analysis_format::{RenderedOutput, render};
use tokmd_analysis_types::*;
use tokmd_types::{AnalysisFormat, ScanStatus, ToolInfo};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn minimal_receipt() -> AnalysisReceipt {
    AnalysisReceipt {
        schema_version: 2,
        generated_at_ms: 0,
        tool: ToolInfo {
            name: "tokmd".to_string(),
            version: "0.0.0".to_string(),
        },
        mode: "analysis".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        source: AnalysisSource {
            inputs: vec!["test".to_string()],
            export_path: None,
            base_receipt_path: None,
            export_schema_version: None,
            export_generated_at_ms: None,
            base_signature: None,
            module_roots: vec![],
            module_depth: 1,
            children: "collapse".to_string(),
        },
        args: AnalysisArgsMeta {
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
            files: 10,
            code: 1000,
            comments: 200,
            blanks: 100,
            lines: 1300,
            bytes: 50000,
            tokens: 2500,
        },
        doc_density: RatioReport {
            total: RatioRow {
                key: "total".to_string(),
                numerator: 200,
                denominator: 1200,
                ratio: 0.1667,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        whitespace: RatioReport {
            total: RatioRow {
                key: "total".to_string(),
                numerator: 100,
                denominator: 1300,
                ratio: 0.0769,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        verbosity: RateReport {
            total: RateRow {
                key: "total".to_string(),
                numerator: 50000,
                denominator: 1300,
                rate: 38.46,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        max_file: MaxFileReport {
            overall: FileStatRow {
                path: "src/lib.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                code: 500,
                comments: 100,
                blanks: 50,
                lines: 650,
                bytes: 25000,
                tokens: 1250,
                doc_pct: Some(0.167),
                bytes_per_line: Some(38.46),
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
            test_lines: 200,
            prod_lines: 1000,
            test_files: 5,
            prod_files: 5,
            ratio: 0.2,
        },
        boilerplate: BoilerplateReport {
            infra_lines: 100,
            logic_lines: 1100,
            ratio: 0.083,
            infra_langs: vec!["TOML".to_string()],
        },
        polyglot: PolyglotReport {
            lang_count: 2,
            entropy: 0.5,
            dominant_lang: "Rust".to_string(),
            dominant_lines: 1000,
            dominant_pct: 0.833,
        },
        distribution: DistributionReport {
            count: 10,
            min: 50,
            max: 650,
            mean: 130.0,
            median: 100.0,
            p90: 400.0,
            p99: 650.0,
            gini: 0.3,
        },
        histogram: vec![HistogramBucket {
            label: "Small".to_string(),
            min: 0,
            max: Some(100),
            files: 5,
            pct: 0.5,
        }],
        top: TopOffenders {
            largest_lines: vec![FileStatRow {
                path: "src/lib.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                code: 500,
                comments: 100,
                blanks: 50,
                lines: 650,
                bytes: 25000,
                tokens: 1250,
                doc_pct: Some(0.167),
                bytes_per_line: Some(38.46),
                depth: 1,
            }],
            largest_tokens: vec![],
            largest_bytes: vec![],
            least_documented: vec![],
            most_dense: vec![],
        },
        tree: Some("test-tree".to_string()),
        reading_time: ReadingTimeReport {
            minutes: 65.0,
            lines_per_minute: 20,
            basis_lines: 1300,
        },
        context_window: Some(ContextWindowReport {
            window_tokens: 100000,
            total_tokens: 2500,
            pct: 0.025,
            fits: true,
        }),
        cocomo: Some(CocomoReport {
            mode: "organic".to_string(),
            kloc: 1.0,
            effort_pm: 2.4,
            duration_months: 2.5,
            staff: 1.0,
            a: 2.4,
            b: 1.05,
            c: 2.5,
            d: 0.38,
        }),
        todo: Some(TodoReport {
            total: 5,
            density_per_kloc: 5.0,
            tags: vec![TodoTagRow {
                tag: "TODO".to_string(),
                count: 5,
            }],
        }),
        integrity: IntegrityReport {
            algo: "blake3".to_string(),
            hash: "abc123".to_string(),
            entries: 10,
        },
    }
}

fn extract_text(output: RenderedOutput) -> String {
    match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected Text, got Binary"),
    }
}

/// Build a receipt with all optional sections populated.
fn fully_populated_receipt() -> AnalysisReceipt {
    let mut r = minimal_receipt();
    r.derived = Some(sample_derived());
    r.archetype = Some(Archetype {
        kind: "library".to_string(),
        evidence: vec!["Cargo.toml".to_string()],
    });
    r.topics = Some(TopicClouds {
        overall: vec![TopicTerm {
            term: "code".to_string(),
            score: 2.0,
            tf: 20,
            df: 5,
        }],
        per_module: BTreeMap::new(),
    });
    r.entropy = Some(EntropyReport {
        suspects: vec![EntropyFinding {
            path: "secret.bin".to_string(),
            module: "root".to_string(),
            entropy_bits_per_byte: 7.5,
            sample_bytes: 1024,
            class: EntropyClass::High,
        }],
    });
    r.license = Some(LicenseReport {
        effective: Some("MIT".to_string()),
        findings: vec![LicenseFinding {
            spdx: "MIT".to_string(),
            confidence: 0.95,
            source_path: "LICENSE".to_string(),
            source_kind: LicenseSourceKind::Text,
        }],
    });
    r.corporate_fingerprint = Some(CorporateFingerprint {
        domains: vec![DomainStat {
            domain: "example.com".to_string(),
            commits: 50,
            pct: 0.75,
        }],
    });
    r.assets = Some(AssetReport {
        total_files: 3,
        total_bytes: 1000,
        categories: vec![AssetCategoryRow {
            category: "images".to_string(),
            files: 2,
            bytes: 800,
            extensions: vec!["png".to_string()],
        }],
        top_files: vec![AssetFileRow {
            path: "logo.png".to_string(),
            bytes: 500,
            category: "images".to_string(),
            extension: "png".to_string(),
        }],
    });
    r.deps = Some(DependencyReport {
        total: 10,
        lockfiles: vec![LockfileReport {
            path: "Cargo.lock".to_string(),
            kind: "cargo".to_string(),
            dependencies: 10,
        }],
    });
    r.git = Some(GitReport {
        commits_scanned: 100,
        files_seen: 50,
        hotspots: vec![HotspotRow {
            path: "src/lib.rs".to_string(),
            commits: 25,
            lines: 500,
            score: 12500,
        }],
        bus_factor: vec![BusFactorRow {
            module: "src".to_string(),
            authors: 3,
        }],
        freshness: FreshnessReport {
            threshold_days: 90,
            stale_files: 5,
            total_files: 50,
            stale_pct: 0.1,
            by_module: vec![],
        },
        coupling: vec![],
        age_distribution: None,
        intent: None,
    });
    r.imports = Some(ImportReport {
        granularity: "module".to_string(),
        edges: vec![ImportEdge {
            from: "src/main".to_string(),
            to: "src/lib".to_string(),
            count: 3,
        }],
    });
    r.dup = Some(DuplicateReport {
        wasted_bytes: 5000,
        strategy: "content".to_string(),
        groups: vec![DuplicateGroup {
            hash: "deadbeef".to_string(),
            bytes: 500,
            files: vec!["a.rs".to_string(), "b.rs".to_string()],
        }],
        density: None,
        near: None,
    });
    r.complexity = Some(ComplexityReport {
        total_functions: 50,
        avg_function_length: 20.5,
        max_function_length: 100,
        avg_cyclomatic: 3.2,
        max_cyclomatic: 15,
        avg_cognitive: Some(4.1),
        max_cognitive: Some(20),
        avg_nesting_depth: Some(2.3),
        max_nesting_depth: Some(6),
        high_risk_files: 2,
        histogram: None,
        halstead: None,
        maintainability_index: None,
        technical_debt: None,
        files: vec![FileComplexity {
            path: "src/complex.rs".to_string(),
            module: "src".to_string(),
            function_count: 10,
            max_function_length: 100,
            cyclomatic_complexity: 15,
            cognitive_complexity: Some(20),
            max_nesting: Some(6),
            risk_level: ComplexityRisk::High,
            functions: None,
        }],
    });
    r.api_surface = Some(ApiSurfaceReport {
        total_items: 100,
        public_items: 40,
        internal_items: 60,
        public_ratio: 0.4,
        documented_ratio: 0.75,
        by_language: BTreeMap::new(),
        by_module: vec![],
        top_exporters: vec![],
    });
    r.fun = Some(FunReport {
        eco_label: Some(EcoLabel {
            label: "A+".to_string(),
            score: 95.0,
            bytes: 10000,
            notes: "Efficient".to_string(),
        }),
    });
    r
}

// ---------------------------------------------------------------------------
// 1. Deterministic output – same input always produces identical output
// ---------------------------------------------------------------------------

#[test]
fn md_deterministic_output() {
    let r = fully_populated_receipt();
    let a = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    let b = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert_eq!(a, b, "Markdown rendering must be deterministic");
}

#[test]
fn json_deterministic_output() {
    let r = fully_populated_receipt();
    let a = extract_text(render(&r, AnalysisFormat::Json).unwrap());
    let b = extract_text(render(&r, AnalysisFormat::Json).unwrap());
    assert_eq!(a, b, "JSON rendering must be deterministic");
}

#[test]
fn xml_deterministic_output() {
    let r = fully_populated_receipt();
    let a = extract_text(render(&r, AnalysisFormat::Xml).unwrap());
    let b = extract_text(render(&r, AnalysisFormat::Xml).unwrap());
    assert_eq!(a, b, "XML rendering must be deterministic");
}

// ---------------------------------------------------------------------------
// 2. Schema version in JSON output
// ---------------------------------------------------------------------------

#[test]
fn json_contains_schema_version() {
    let r = minimal_receipt();
    let json = extract_text(render(&r, AnalysisFormat::Json).unwrap());
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["schema_version"], 2);
}

#[test]
fn json_schema_version_survives_roundtrip() {
    let r = minimal_receipt();
    let json = extract_text(render(&r, AnalysisFormat::Json).unwrap());
    let rt: AnalysisReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(rt.schema_version, r.schema_version);
}

// ---------------------------------------------------------------------------
// 3. JSON round-trip fidelity for all optional sections
// ---------------------------------------------------------------------------

#[test]
fn json_roundtrip_full_receipt() {
    let r = fully_populated_receipt();
    let json = extract_text(render(&r, AnalysisFormat::Json).unwrap());
    let rt: AnalysisReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(rt.schema_version, r.schema_version);
    assert_eq!(rt.archetype.as_ref().unwrap().kind, "library");
    assert_eq!(
        rt.license.as_ref().unwrap().effective.as_deref(),
        Some("MIT")
    );
    assert_eq!(rt.deps.as_ref().unwrap().total, 10);
    assert_eq!(rt.complexity.as_ref().unwrap().total_functions, 50);
    assert_eq!(rt.api_surface.as_ref().unwrap().public_items, 40);
    assert_eq!(rt.git.as_ref().unwrap().commits_scanned, 100);
}

#[test]
fn json_roundtrip_preserves_derived_totals() {
    let mut r = minimal_receipt();
    r.derived = Some(sample_derived());
    let json = extract_text(render(&r, AnalysisFormat::Json).unwrap());
    let rt: AnalysisReceipt = serde_json::from_str(&json).unwrap();
    let d = rt.derived.unwrap();
    assert_eq!(d.totals.code, 1000);
    assert_eq!(d.totals.tokens, 2500);
    assert_eq!(d.totals.files, 10);
}

// ---------------------------------------------------------------------------
// 4. Empty analysis data
// ---------------------------------------------------------------------------

#[test]
fn md_minimal_receipt_has_header() {
    let r = minimal_receipt();
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert!(md.starts_with("# tokmd analysis\n"));
    assert!(md.contains("Preset: `receipt`"));
}

#[test]
fn md_minimal_receipt_no_optional_sections() {
    let r = minimal_receipt();
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    for section in [
        "## Archetype",
        "## Topics",
        "## Entropy",
        "## License",
        "## Corporate fingerprint",
        "## Predictive churn",
        "## Totals",
        "## Assets",
        "## Dependencies",
        "## Git metrics",
        "## Imports",
        "## Duplicates",
        "## Complexity",
        "## API surface",
        "## Eco label",
    ] {
        assert!(!md.contains(section), "should not contain {section}");
    }
}

#[test]
fn xml_minimal_receipt_empty_element() {
    let r = minimal_receipt();
    let xml = extract_text(render(&r, AnalysisFormat::Xml).unwrap());
    assert_eq!(xml, "<analysis></analysis>");
}

#[test]
fn jsonld_minimal_receipt_defaults_to_zero() {
    let r = minimal_receipt();
    let jsonld = extract_text(render(&r, AnalysisFormat::Jsonld).unwrap());
    let v: serde_json::Value = serde_json::from_str(&jsonld).unwrap();
    assert_eq!(v["codeLines"], 0);
    assert_eq!(v["lineCount"], 0);
    assert_eq!(v["fileSize"], 0);
}

// ---------------------------------------------------------------------------
// 5. All sections present in Markdown
// ---------------------------------------------------------------------------

#[test]
fn md_full_receipt_contains_all_sections() {
    let r = fully_populated_receipt();
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    let sections = [
        "## Inputs",
        "## Archetype",
        "## Topics",
        "## Entropy profiling",
        "## License radar",
        "## Corporate fingerprint",
        "## Totals",
        "## Ratios",
        "## Distribution",
        "## File size histogram",
        "## Top offenders",
        "## Structure",
        "## Test density",
        "## TODOs",
        "## Boilerplate ratio",
        "## Polyglot",
        "## Reading time",
        "## Context window",
        "## COCOMO estimate",
        "## Integrity",
        "## Assets",
        "## Dependencies",
        "## Git metrics",
        "## Imports",
        "## Duplicates",
        "## Complexity",
        "## API surface",
        "## Eco label",
    ];
    for section in sections {
        assert!(md.contains(section), "missing section: {section}");
    }
}

// ---------------------------------------------------------------------------
// 6. Unicode content handling
// ---------------------------------------------------------------------------

#[test]
fn md_unicode_in_inputs() {
    let mut r = minimal_receipt();
    r.source.inputs = vec![
        "日本語パス/ソース".to_string(),
        "données/résultat".to_string(),
    ];
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert!(md.contains("日本語パス/ソース"));
    assert!(md.contains("données/résultat"));
}

#[test]
fn md_unicode_in_archetype() {
    let mut r = minimal_receipt();
    r.archetype = Some(Archetype {
        kind: "bibliothèque".to_string(),
        evidence: vec!["fichier.toml".to_string()],
    });
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert!(md.contains("bibliothèque"));
}

#[test]
fn json_unicode_roundtrip() {
    let mut r = minimal_receipt();
    r.source.inputs = vec!["中文路径".to_string()];
    r.archetype = Some(Archetype {
        kind: "アプリ".to_string(),
        evidence: vec!["설정.toml".to_string()],
    });
    let json = extract_text(render(&r, AnalysisFormat::Json).unwrap());
    let rt: AnalysisReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(rt.source.inputs[0], "中文路径");
    assert_eq!(rt.archetype.unwrap().kind, "アプリ");
}

#[test]
fn mermaid_unicode_sanitization() {
    let mut r = minimal_receipt();
    r.imports = Some(ImportReport {
        granularity: "module".to_string(),
        edges: vec![ImportEdge {
            from: "src/ñoño".to_string(),
            to: "src/über".to_string(),
            count: 1,
        }],
    });
    let merm = extract_text(render(&r, AnalysisFormat::Mermaid).unwrap());
    // Non-ASCII should be replaced with underscore
    assert!(merm.contains("src__o_o"));
    assert!(merm.contains("src__ber"));
}

// ---------------------------------------------------------------------------
// 7. Large data sets
// ---------------------------------------------------------------------------

#[test]
fn md_large_entropy_truncates_to_ten() {
    let mut r = minimal_receipt();
    let suspects: Vec<EntropyFinding> = (0..50)
        .map(|i| EntropyFinding {
            path: format!("file_{i}.bin"),
            module: "root".to_string(),
            entropy_bits_per_byte: 7.0 + (i as f32) * 0.01,
            sample_bytes: 1024,
            class: EntropyClass::High,
        })
        .collect();
    r.entropy = Some(EntropyReport { suspects });
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    // Only first 10 should appear
    assert!(md.contains("file_0.bin"));
    assert!(md.contains("file_9.bin"));
    assert!(!md.contains("file_10.bin"));
}

#[test]
fn md_large_hotspots_truncates_to_ten() {
    let mut r = minimal_receipt();
    let hotspots: Vec<HotspotRow> = (0..25)
        .map(|i| HotspotRow {
            path: format!("src/file_{i}.rs"),
            commits: 100 - i,
            lines: 500,
            score: (100 - i) * 500,
        })
        .collect();
    r.git = Some(GitReport {
        commits_scanned: 500,
        files_seen: 25,
        hotspots,
        bus_factor: vec![],
        freshness: FreshnessReport {
            threshold_days: 90,
            stale_files: 0,
            total_files: 25,
            stale_pct: 0.0,
            by_module: vec![],
        },
        coupling: vec![],
        age_distribution: None,
        intent: None,
    });
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert!(md.contains("src/file_0.rs"));
    assert!(md.contains("src/file_9.rs"));
    assert!(!md.contains("src/file_10.rs"));
}

#[test]
fn md_large_import_edges_truncates_to_twenty() {
    let mut r = minimal_receipt();
    let edges: Vec<ImportEdge> = (0..50)
        .map(|i| ImportEdge {
            from: format!("mod_{i}"),
            to: format!("mod_{}", i + 1),
            count: 1,
        })
        .collect();
    r.imports = Some(ImportReport {
        granularity: "module".to_string(),
        edges,
    });
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert!(md.contains("|mod_0|mod_1|"));
    assert!(md.contains("|mod_19|mod_20|"));
    assert!(!md.contains("|mod_20|mod_21|"));
}

// ---------------------------------------------------------------------------
// 8. Partial data (some sections missing)
// ---------------------------------------------------------------------------

#[test]
fn md_derived_without_cocomo() {
    let mut r = minimal_receipt();
    let mut d = sample_derived();
    d.cocomo = None;
    r.derived = Some(d);
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert!(md.contains("## Totals"));
    assert!(!md.contains("## COCOMO estimate"));
}

#[test]
fn md_derived_without_context_window() {
    let mut r = minimal_receipt();
    let mut d = sample_derived();
    d.context_window = None;
    r.derived = Some(d);
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert!(md.contains("## Totals"));
    assert!(!md.contains("## Context window"));
}

#[test]
fn md_derived_without_todo() {
    let mut r = minimal_receipt();
    let mut d = sample_derived();
    d.todo = None;
    r.derived = Some(d);
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert!(md.contains("## Totals"));
    assert!(!md.contains("## TODOs"));
}

// ---------------------------------------------------------------------------
// 9. Complexity section rendering
// ---------------------------------------------------------------------------

#[test]
fn md_complexity_all_optional_fields() {
    let mut r = minimal_receipt();
    r.complexity = Some(ComplexityReport {
        total_functions: 100,
        avg_function_length: 15.5,
        max_function_length: 200,
        avg_cyclomatic: 5.0,
        max_cyclomatic: 25,
        avg_cognitive: Some(7.3),
        max_cognitive: Some(30),
        avg_nesting_depth: Some(3.1),
        max_nesting_depth: Some(8),
        high_risk_files: 5,
        histogram: None,
        halstead: None,
        maintainability_index: None,
        technical_debt: None,
        files: vec![],
    });
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert!(md.contains("|Total functions|100|"));
    assert!(md.contains("|Avg function length|15.5|"));
    assert!(md.contains("|Max cyclomatic|25|"));
    assert!(md.contains("|Avg cognitive|7.30|"));
    assert!(md.contains("|Max cognitive|30|"));
    assert!(md.contains("|Avg nesting depth|3.10|"));
    assert!(md.contains("|Max nesting depth|8|"));
    assert!(md.contains("|High risk files|5|"));
}

#[test]
fn md_complexity_without_optional_metrics() {
    let mut r = minimal_receipt();
    r.complexity = Some(ComplexityReport {
        total_functions: 10,
        avg_function_length: 8.0,
        max_function_length: 30,
        avg_cyclomatic: 2.0,
        max_cyclomatic: 6,
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
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert!(md.contains("|Total functions|10|"));
    assert!(!md.contains("Avg cognitive"));
    assert!(!md.contains("Max cognitive"));
    assert!(!md.contains("Avg nesting depth"));
    assert!(!md.contains("Max nesting depth"));
}

#[test]
fn md_complexity_file_table() {
    let mut r = minimal_receipt();
    r.complexity = Some(ComplexityReport {
        total_functions: 10,
        avg_function_length: 20.0,
        max_function_length: 50,
        avg_cyclomatic: 3.0,
        max_cyclomatic: 12,
        avg_cognitive: None,
        max_cognitive: None,
        avg_nesting_depth: None,
        max_nesting_depth: None,
        high_risk_files: 1,
        histogram: None,
        halstead: None,
        maintainability_index: None,
        technical_debt: None,
        files: vec![FileComplexity {
            path: "src/parser.rs".to_string(),
            module: "src".to_string(),
            function_count: 5,
            max_function_length: 50,
            cyclomatic_complexity: 12,
            cognitive_complexity: None,
            max_nesting: None,
            risk_level: ComplexityRisk::High,
            functions: None,
        }],
    });
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert!(md.contains("### Top complex files"));
    assert!(md.contains("|src/parser.rs|12|5|50|"));
}

// ---------------------------------------------------------------------------
// 10. API surface rendering
// ---------------------------------------------------------------------------

#[test]
fn md_api_surface_with_language_breakdown() {
    let mut r = minimal_receipt();
    let mut by_language = BTreeMap::new();
    by_language.insert(
        "Rust".to_string(),
        LangApiSurface {
            total_items: 80,
            public_items: 30,
            internal_items: 50,
            public_ratio: 0.375,
        },
    );
    r.api_surface = Some(ApiSurfaceReport {
        total_items: 80,
        public_items: 30,
        internal_items: 50,
        public_ratio: 0.375,
        documented_ratio: 0.8,
        by_language,
        by_module: vec![ModuleApiRow {
            module: "src".to_string(),
            total_items: 80,
            public_items: 30,
            public_ratio: 0.375,
        }],
        top_exporters: vec![ApiExportItem {
            path: "src/lib.rs".to_string(),
            lang: "Rust".to_string(),
            public_items: 15,
            total_items: 30,
        }],
    });
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert!(md.contains("## API surface"));
    assert!(md.contains("|Total items|80|"));
    assert!(md.contains("|Public ratio|37.5%|"));
    assert!(md.contains("|Documented ratio|80.0%|"));
    assert!(md.contains("### By language"));
    assert!(md.contains("|Rust|80|30|50|37.5%|"));
    assert!(md.contains("### By module"));
    assert!(md.contains("|src|80|30|37.5%|"));
    assert!(md.contains("### Top exporters"));
    assert!(md.contains("|src/lib.rs|Rust|15|30|"));
}

// ---------------------------------------------------------------------------
// 11. Coupling filtering in Markdown (count >= 2 required)
// ---------------------------------------------------------------------------

#[test]
fn md_coupling_filters_low_count() {
    let mut r = minimal_receipt();
    r.git = Some(GitReport {
        commits_scanned: 100,
        files_seen: 10,
        hotspots: vec![],
        bus_factor: vec![],
        freshness: FreshnessReport {
            threshold_days: 90,
            stale_files: 0,
            total_files: 10,
            stale_pct: 0.0,
            by_module: vec![],
        },
        coupling: vec![
            CouplingRow {
                left: "a.rs".to_string(),
                right: "b.rs".to_string(),
                count: 1, // Should be filtered out
                jaccard: Some(0.9),
                lift: Some(5.0),
                n_left: Some(5),
                n_right: Some(5),
            },
            CouplingRow {
                left: "c.rs".to_string(),
                right: "d.rs".to_string(),
                count: 3, // Should be included
                jaccard: Some(0.6),
                lift: Some(2.0),
                n_left: Some(10),
                n_right: Some(8),
            },
        ],
        age_distribution: None,
        intent: None,
    });
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert!(md.contains("### Coupling"));
    assert!(!md.contains("|a.rs|b.rs|"), "count=1 should be filtered");
    assert!(md.contains("|c.rs|d.rs|3|"), "count=3 should be included");
}

#[test]
fn md_coupling_all_filtered_hides_section() {
    let mut r = minimal_receipt();
    r.git = Some(GitReport {
        commits_scanned: 50,
        files_seen: 5,
        hotspots: vec![],
        bus_factor: vec![],
        freshness: FreshnessReport {
            threshold_days: 90,
            stale_files: 0,
            total_files: 5,
            stale_pct: 0.0,
            by_module: vec![],
        },
        coupling: vec![CouplingRow {
            left: "a.rs".to_string(),
            right: "b.rs".to_string(),
            count: 1,
            jaccard: None,
            lift: None,
            n_left: None,
            n_right: None,
        }],
        age_distribution: None,
        intent: None,
    });
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert!(!md.contains("### Coupling"));
}

// ---------------------------------------------------------------------------
// 12. Git commit intent rendering
// ---------------------------------------------------------------------------

#[test]
fn md_git_intent_section() {
    let mut r = minimal_receipt();
    r.git = Some(GitReport {
        commits_scanned: 200,
        files_seen: 80,
        hotspots: vec![],
        bus_factor: vec![],
        freshness: FreshnessReport {
            threshold_days: 90,
            stale_files: 0,
            total_files: 80,
            stale_pct: 0.0,
            by_module: vec![],
        },
        coupling: vec![],
        age_distribution: None,
        intent: Some(CommitIntentReport {
            overall: CommitIntentCounts {
                feat: 50,
                fix: 30,
                refactor: 20,
                docs: 10,
                test: 15,
                chore: 5,
                ci: 3,
                build: 2,
                perf: 1,
                style: 0,
                revert: 4,
                other: 10,
                total: 150,
            },
            by_module: vec![ModuleIntentRow {
                module: "src".to_string(),
                counts: CommitIntentCounts {
                    feat: 20,
                    fix: 15,
                    refactor: 10,
                    docs: 5,
                    test: 5,
                    chore: 2,
                    ci: 1,
                    build: 1,
                    perf: 0,
                    style: 0,
                    revert: 3,
                    other: 5,
                    total: 67,
                },
            }],
            unknown_pct: 0.067,
            corrective_ratio: Some(0.227),
        }),
    });
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert!(md.contains("### Commit intent"));
    assert!(md.contains("|feat|50|"));
    assert!(md.contains("|fix|30|"));
    assert!(md.contains("|**total**|150|"));
    assert!(md.contains("Unknown: `6.7%`"));
    assert!(md.contains("Corrective ratio"));
    assert!(md.contains("22.7%"));
    // Maintenance hotspots
    assert!(md.contains("#### Maintenance hotspots"));
    assert!(md.contains("|src|18|67|")); // fix(15) + revert(3) = 18
}

#[test]
fn md_git_intent_skips_zero_counts() {
    let mut r = minimal_receipt();
    r.git = Some(GitReport {
        commits_scanned: 10,
        files_seen: 5,
        hotspots: vec![],
        bus_factor: vec![],
        freshness: FreshnessReport {
            threshold_days: 90,
            stale_files: 0,
            total_files: 5,
            stale_pct: 0.0,
            by_module: vec![],
        },
        coupling: vec![],
        age_distribution: None,
        intent: Some(CommitIntentReport {
            overall: CommitIntentCounts {
                feat: 5,
                fix: 0,
                refactor: 0,
                docs: 0,
                test: 0,
                chore: 0,
                ci: 0,
                build: 0,
                perf: 0,
                style: 0,
                revert: 0,
                other: 0,
                total: 5,
            },
            by_module: vec![],
            unknown_pct: 0.0,
            corrective_ratio: None,
        }),
    });
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert!(md.contains("|feat|5|"));
    assert!(!md.contains("|fix|"));
    assert!(!md.contains("Corrective ratio"));
}

// ---------------------------------------------------------------------------
// 13. Near-duplicate rendering
// ---------------------------------------------------------------------------

#[test]
fn md_near_dup_with_pairs_and_clusters() {
    let mut r = minimal_receipt();
    r.dup = Some(DuplicateReport {
        wasted_bytes: 1000,
        strategy: "content".to_string(),
        groups: vec![],
        density: None,
        near: Some(NearDuplicateReport {
            params: NearDupParams {
                scope: NearDupScope::Module,
                threshold: 0.80,
                max_files: 100,
                max_pairs: None,
                max_file_bytes: None,
                selection_method: None,
                algorithm: None,
                exclude_patterns: vec![],
            },
            pairs: vec![NearDupPairRow {
                left: "src/a.rs".to_string(),
                right: "src/b.rs".to_string(),
                similarity: 0.92,
                shared_fingerprints: 45,
                left_fingerprints: 50,
                right_fingerprints: 48,
            }],
            files_analyzed: 20,
            files_skipped: 5,
            eligible_files: Some(25),
            clusters: Some(vec![NearDupCluster {
                files: vec!["src/a.rs".to_string(), "src/b.rs".to_string()],
                max_similarity: 0.92,
                representative: "src/a.rs".to_string(),
                pair_count: 1,
            }]),
            truncated: false,
            excluded_by_pattern: None,
            stats: Some(NearDupStats {
                fingerprinting_ms: 15,
                pairing_ms: 8,
                bytes_processed: 50000,
            }),
        }),
    });
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert!(md.contains("### Near duplicates"));
    assert!(md.contains("Files analyzed: `20`"));
    assert!(md.contains("Files skipped: `5`"));
    assert!(md.contains("Threshold: `0.80`"));
    assert!(md.contains("Scope: `Module`"));
    assert!(md.contains("Eligible files: `25`"));
    assert!(md.contains("#### Clusters"));
    assert!(md.contains("|1|2|92.0%|src/a.rs|1|"));
    assert!(md.contains("#### Pairs"));
    assert!(md.contains("|src/a.rs|src/b.rs|92.0%|45|"));
    assert!(md.contains("Near-dup stats: fingerprinting 15ms, pairing 8ms, 50000 bytes"));
}

#[test]
fn md_near_dup_truncated_shows_warning() {
    let mut r = minimal_receipt();
    r.dup = Some(DuplicateReport {
        wasted_bytes: 0,
        strategy: "content".to_string(),
        groups: vec![],
        density: None,
        near: Some(NearDuplicateReport {
            params: NearDupParams {
                scope: NearDupScope::Lang,
                threshold: 0.75,
                max_files: 50,
                max_pairs: Some(100),
                max_file_bytes: None,
                selection_method: None,
                algorithm: None,
                exclude_patterns: vec![],
            },
            pairs: vec![],
            files_analyzed: 10,
            files_skipped: 0,
            eligible_files: None,
            clusters: None,
            truncated: true,
            excluded_by_pattern: None,
            stats: None,
        }),
    });
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert!(md.contains("**Warning**: Pair list truncated"));
}

#[test]
fn md_near_dup_no_pairs_shows_message() {
    let mut r = minimal_receipt();
    r.dup = Some(DuplicateReport {
        wasted_bytes: 0,
        strategy: "content".to_string(),
        groups: vec![],
        density: None,
        near: Some(NearDuplicateReport {
            params: NearDupParams {
                scope: NearDupScope::default(),
                threshold: 0.80,
                max_files: 100,
                max_pairs: None,
                max_file_bytes: None,
                selection_method: None,
                algorithm: None,
                exclude_patterns: vec![],
            },
            pairs: vec![],
            files_analyzed: 10,
            files_skipped: 0,
            eligible_files: None,
            clusters: None,
            truncated: false,
            excluded_by_pattern: None,
            stats: None,
        }),
    });
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert!(md.contains("No near-duplicate pairs detected"));
}

// ---------------------------------------------------------------------------
// 14. SVG rendering details
// ---------------------------------------------------------------------------

#[test]
fn svg_dimensions_correct() {
    let r = minimal_receipt();
    let svg = extract_text(render(&r, AnalysisFormat::Svg).unwrap());
    assert!(svg.contains("width=\"240\""));
    assert!(svg.contains("height=\"32\""));
    // label_width=80, value_width=160
    assert!(svg.contains("width=\"80\""));
    assert!(svg.contains("width=\"160\""));
}

#[test]
fn svg_with_context_shows_percentage() {
    let mut r = minimal_receipt();
    r.derived = Some(sample_derived());
    let svg = extract_text(render(&r, AnalysisFormat::Svg).unwrap());
    assert!(svg.contains("context"));
    assert!(svg.contains("2.5%"));
}

// ---------------------------------------------------------------------------
// 15. JSON-LD structure
// ---------------------------------------------------------------------------

#[test]
fn jsonld_has_interaction_statistic() {
    let mut r = minimal_receipt();
    r.derived = Some(sample_derived());
    let jsonld = extract_text(render(&r, AnalysisFormat::Jsonld).unwrap());
    let v: serde_json::Value = serde_json::from_str(&jsonld).unwrap();
    assert_eq!(v["@context"], "https://schema.org");
    assert_eq!(v["@type"], "SoftwareSourceCode");
    assert_eq!(v["codeLines"], 1000);
    assert_eq!(v["commentCount"], 200);
    assert_eq!(v["interactionStatistic"]["@type"], "InteractionCounter");
    assert_eq!(
        v["interactionStatistic"]["userInteractionCount"],
        2500 // tokens
    );
}

// ---------------------------------------------------------------------------
// 16. Tree format
// ---------------------------------------------------------------------------

#[test]
fn tree_shows_content_when_present() {
    let mut r = minimal_receipt();
    let mut d = sample_derived();
    d.tree = Some("root\n├── src\n│   └── lib.rs\n└── Cargo.toml".to_string());
    r.derived = Some(d);
    let tree = extract_text(render(&r, AnalysisFormat::Tree).unwrap());
    assert!(tree.contains("root"));
    assert!(tree.contains("src"));
    assert!(tree.contains("lib.rs"));
}

// ---------------------------------------------------------------------------
// 17. Mermaid sanitization
// ---------------------------------------------------------------------------

#[test]
fn mermaid_sanitizes_special_characters() {
    let mut r = minimal_receipt();
    r.imports = Some(ImportReport {
        granularity: "file".to_string(),
        edges: vec![ImportEdge {
            from: "src/foo-bar.baz".to_string(),
            to: "src/hello world".to_string(),
            count: 2,
        }],
    });
    let merm = extract_text(render(&r, AnalysisFormat::Mermaid).unwrap());
    assert!(merm.contains("src_foo_bar_baz"));
    assert!(merm.contains("src_hello_world"));
    assert!(merm.contains("|2|"));
}

// ---------------------------------------------------------------------------
// 18. render() dispatch correctness
// ---------------------------------------------------------------------------

#[test]
fn render_dispatch_html_returns_text() {
    let r = minimal_receipt();
    let result = render(&r, AnalysisFormat::Html).unwrap();
    match result {
        RenderedOutput::Text(s) => {
            assert!(
                s.contains("<html") || s.contains("<!DOCTYPE html>"),
                "HTML output should contain html tag"
            );
        }
        RenderedOutput::Binary(_) => panic!("HTML should return Text"),
    }
}

#[cfg(not(feature = "fun"))]
#[test]
fn render_dispatch_obj_errors_without_fun() {
    let r = minimal_receipt();
    let result = render(&r, AnalysisFormat::Obj);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.to_string().contains("fun"));
}

#[cfg(not(feature = "fun"))]
#[test]
fn render_dispatch_midi_errors_without_fun() {
    let r = minimal_receipt();
    let result = render(&r, AnalysisFormat::Midi);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.to_string().contains("fun"));
}

// ---------------------------------------------------------------------------
// 19. Histogram unbounded max
// ---------------------------------------------------------------------------

#[test]
fn md_histogram_unbounded_max_shows_infinity() {
    let mut r = minimal_receipt();
    let mut d = sample_derived();
    d.histogram = vec![
        HistogramBucket {
            label: "Small".to_string(),
            min: 0,
            max: Some(100),
            files: 3,
            pct: 0.3,
        },
        HistogramBucket {
            label: "Huge".to_string(),
            min: 1000,
            max: None,
            files: 2,
            pct: 0.2,
        },
    ];
    r.derived = Some(d);
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert!(md.contains("|Small|0|100|3|30.0%|"));
    assert!(md.contains("|Huge|1000|∞|2|20.0%|"));
}

// ---------------------------------------------------------------------------
// 20. Coupling with None jaccard/lift
// ---------------------------------------------------------------------------

#[test]
fn md_coupling_none_jaccard_lift_shows_dash() {
    let mut r = minimal_receipt();
    r.git = Some(GitReport {
        commits_scanned: 50,
        files_seen: 10,
        hotspots: vec![],
        bus_factor: vec![],
        freshness: FreshnessReport {
            threshold_days: 90,
            stale_files: 0,
            total_files: 10,
            stale_pct: 0.0,
            by_module: vec![],
        },
        coupling: vec![CouplingRow {
            left: "x.rs".to_string(),
            right: "y.rs".to_string(),
            count: 5,
            jaccard: None,
            lift: None,
            n_left: None,
            n_right: None,
        }],
        age_distribution: None,
        intent: None,
    });
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert!(md.contains("|x.rs|y.rs|5|-|-|"));
}

// ---------------------------------------------------------------------------
// 21. Doc density / verbosity / whitespace by language
// ---------------------------------------------------------------------------

#[test]
fn md_derived_by_lang_tables() {
    let mut r = minimal_receipt();
    let mut d = sample_derived();
    d.doc_density.by_lang = vec![RatioRow {
        key: "Rust".to_string(),
        numerator: 200,
        denominator: 1200,
        ratio: 0.167,
    }];
    d.whitespace.by_lang = vec![RatioRow {
        key: "Rust".to_string(),
        numerator: 100,
        denominator: 1300,
        ratio: 0.077,
    }];
    d.verbosity.by_lang = vec![RateRow {
        key: "Rust".to_string(),
        numerator: 50000,
        denominator: 1300,
        rate: 38.46,
    }];
    r.derived = Some(d);
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert!(md.contains("### Doc density by language"));
    assert!(md.contains("|Rust|16.7%|200|1000|"));
    assert!(md.contains("### Whitespace ratio by language"));
    assert!(md.contains("|Rust|7.7%|100|1300|"));
    assert!(md.contains("### Verbosity by language"));
    assert!(md.contains("|Rust|38.46|50000|1300|"));
}

// ---------------------------------------------------------------------------
// 22. XML with derived has correct attributes
// ---------------------------------------------------------------------------

#[test]
fn xml_with_derived_has_all_attributes() {
    let mut r = minimal_receipt();
    r.derived = Some(sample_derived());
    let xml = extract_text(render(&r, AnalysisFormat::Xml).unwrap());
    assert!(xml.contains("files=\"10\""));
    assert!(xml.contains("code=\"1000\""));
    assert!(xml.contains("comments=\"200\""));
    assert!(xml.contains("blanks=\"100\""));
    assert!(xml.contains("lines=\"1300\""));
    assert!(xml.contains("bytes=\"50000\""));
    assert!(xml.contains("tokens=\"2500\""));
}

// ---------------------------------------------------------------------------
// 23. Mermaid with large edge set truncates to 200
// ---------------------------------------------------------------------------

#[test]
fn mermaid_truncates_edges_to_200() {
    let mut r = minimal_receipt();
    let edges: Vec<ImportEdge> = (0..300)
        .map(|i| ImportEdge {
            from: format!("m{i}"),
            to: format!("m{}", i + 1),
            count: 1,
        })
        .collect();
    r.imports = Some(ImportReport {
        granularity: "module".to_string(),
        edges,
    });
    let merm = extract_text(render(&r, AnalysisFormat::Mermaid).unwrap());
    assert!(merm.contains("m199"));
    assert!(!merm.contains("m200 -->|1| m201"));
}

// ---------------------------------------------------------------------------
// 24. All text formats succeed on minimal receipt
// ---------------------------------------------------------------------------

#[test]
fn all_text_formats_render_minimal_receipt() {
    let r = minimal_receipt();
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
        let result = render(&r, fmt);
        assert!(result.is_ok(), "format {:?} failed on minimal receipt", fmt);
    }
}

// ---------------------------------------------------------------------------
// 25. All text formats succeed on full receipt
// ---------------------------------------------------------------------------

#[test]
fn all_text_formats_render_full_receipt() {
    let r = fully_populated_receipt();
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
        let result = render(&r, fmt);
        assert!(result.is_ok(), "format {:?} failed on full receipt", fmt);
    }
}

// ---------------------------------------------------------------------------
// 26. JSON valid structure
// ---------------------------------------------------------------------------

#[test]
fn json_output_is_valid_json() {
    let r = fully_populated_receipt();
    let json = extract_text(render(&r, AnalysisFormat::Json).unwrap());
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json);
    assert!(parsed.is_ok(), "JSON output must be valid JSON");
}

#[test]
fn json_includes_all_optional_sections_when_present() {
    let r = fully_populated_receipt();
    let json = extract_text(render(&r, AnalysisFormat::Json).unwrap());
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(v["archetype"].is_object());
    assert!(v["topics"].is_object());
    assert!(v["entropy"].is_object());
    assert!(v["license"].is_object());
    assert!(v["corporate_fingerprint"].is_object());
    assert!(v["derived"].is_object());
    assert!(v["assets"].is_object());
    assert!(v["deps"].is_object());
    assert!(v["git"].is_object());
    assert!(v["imports"].is_object());
    assert!(v["dup"].is_object());
    assert!(v["complexity"].is_object());
    assert!(v["api_surface"].is_object());
    assert!(v["fun"].is_object());
}

// ---------------------------------------------------------------------------
// 27. JSON omits null sections when not present
// ---------------------------------------------------------------------------

#[test]
fn json_minimal_has_null_optional_sections() {
    let r = minimal_receipt();
    let json = extract_text(render(&r, AnalysisFormat::Json).unwrap());
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(v["archetype"].is_null());
    assert!(v["derived"].is_null());
    assert!(v["git"].is_null());
    assert!(v["complexity"].is_null());
}

// ---------------------------------------------------------------------------
// 28. Predictive churn sorting (descending by slope)
// ---------------------------------------------------------------------------

#[test]
fn md_churn_sorted_descending_by_slope() {
    let mut r = minimal_receipt();
    let mut per_module = BTreeMap::new();
    per_module.insert(
        "low_slope".to_string(),
        ChurnTrend {
            slope: 0.1,
            r2: 0.5,
            recent_change: 1,
            classification: TrendClass::Flat,
        },
    );
    per_module.insert(
        "high_slope".to_string(),
        ChurnTrend {
            slope: 0.9,
            r2: 0.9,
            recent_change: 10,
            classification: TrendClass::Rising,
        },
    );
    per_module.insert(
        "mid_slope".to_string(),
        ChurnTrend {
            slope: 0.5,
            r2: 0.7,
            recent_change: 5,
            classification: TrendClass::Rising,
        },
    );
    r.predictive_churn = Some(PredictiveChurnReport { per_module });
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());

    let high_pos = md.find("|high_slope|").unwrap();
    let mid_pos = md.find("|mid_slope|").unwrap();
    let low_pos = md.find("|low_slope|").unwrap();
    assert!(
        high_pos < mid_pos,
        "high_slope should come before mid_slope"
    );
    assert!(mid_pos < low_pos, "mid_slope should come before low_slope");
}

// ---------------------------------------------------------------------------
// 29. License findings table rendering
// ---------------------------------------------------------------------------

#[test]
fn md_license_multiple_findings() {
    let mut r = minimal_receipt();
    r.license = Some(LicenseReport {
        effective: Some("MIT OR Apache-2.0".to_string()),
        findings: vec![
            LicenseFinding {
                spdx: "MIT".to_string(),
                confidence: 0.95,
                source_path: "LICENSE-MIT".to_string(),
                source_kind: LicenseSourceKind::Text,
            },
            LicenseFinding {
                spdx: "Apache-2.0".to_string(),
                confidence: 0.90,
                source_path: "LICENSE-APACHE".to_string(),
                source_kind: LicenseSourceKind::Text,
            },
        ],
    });
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert!(md.contains("Effective: `MIT OR Apache-2.0`"));
    assert!(md.contains("|MIT|0.95|LICENSE-MIT|Text|"));
    assert!(md.contains("|Apache-2.0|0.90|LICENSE-APACHE|Text|"));
}

// ---------------------------------------------------------------------------
// 30. Code age distribution rendering
// ---------------------------------------------------------------------------

#[test]
fn md_code_age_with_unbounded_max() {
    let mut r = minimal_receipt();
    r.git = Some(GitReport {
        commits_scanned: 50,
        files_seen: 20,
        hotspots: vec![],
        bus_factor: vec![],
        freshness: FreshnessReport {
            threshold_days: 90,
            stale_files: 0,
            total_files: 20,
            stale_pct: 0.0,
            by_module: vec![],
        },
        coupling: vec![],
        age_distribution: Some(CodeAgeDistributionReport {
            buckets: vec![
                CodeAgeBucket {
                    label: "0-30d".to_string(),
                    min_days: 0,
                    max_days: Some(30),
                    files: 10,
                    pct: 0.5,
                },
                CodeAgeBucket {
                    label: "365d+".to_string(),
                    min_days: 365,
                    max_days: None,
                    files: 3,
                    pct: 0.15,
                },
            ],
            recent_refreshes: 15,
            prior_refreshes: 10,
            refresh_trend: TrendClass::Flat,
        }),
        intent: None,
    });
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert!(md.contains("### Code age"));
    assert!(md.contains("Refresh trend: `Flat`"));
    assert!(md.contains("|0-30d|0|30|10|50.0%|"));
    assert!(md.contains("|365d+|365|∞|3|15.0%|"));
}

// ---------------------------------------------------------------------------
// 31. Duplication density by module
// ---------------------------------------------------------------------------

#[test]
fn md_dup_density_module_table() {
    let mut r = minimal_receipt();
    r.dup = Some(DuplicateReport {
        wasted_bytes: 3000,
        strategy: "content".to_string(),
        groups: vec![],
        density: Some(DuplicationDensityReport {
            duplicate_groups: 2,
            duplicate_files: 4,
            duplicated_bytes: 6000,
            wasted_bytes: 3000,
            wasted_pct_of_codebase: 0.05,
            by_module: vec![
                ModuleDuplicationDensityRow {
                    module: "src".to_string(),
                    duplicate_files: 3,
                    wasted_files: 1,
                    duplicated_bytes: 4000,
                    wasted_bytes: 2000,
                    module_bytes: 40000,
                    density: 0.05,
                },
                ModuleDuplicationDensityRow {
                    module: "tests".to_string(),
                    duplicate_files: 1,
                    wasted_files: 0,
                    duplicated_bytes: 2000,
                    wasted_bytes: 1000,
                    module_bytes: 10000,
                    density: 0.1,
                },
            ],
        }),
        near: None,
    });
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert!(md.contains("### Duplication density"));
    assert!(md.contains("Duplicate groups: `2`"));
    assert!(md.contains("Waste vs codebase: `5.0%`"));
    assert!(md.contains("|src|3|1|4000|2000|40000|5.0%|"));
    assert!(md.contains("|tests|1|0|2000|1000|10000|10.0%|"));
}

// ---------------------------------------------------------------------------
// 32. Boilerplate and polyglot rendering values
// ---------------------------------------------------------------------------

#[test]
fn md_boilerplate_and_polyglot_values() {
    let mut r = minimal_receipt();
    r.derived = Some(sample_derived());
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert!(md.contains("Infra lines: `100`"));
    assert!(md.contains("Logic lines: `1100`"));
    assert!(md.contains("Infra ratio: `8.3%`"));
    assert!(md.contains("Languages: `2`"));
    assert!(md.contains("Dominant: `Rust` (83.3%)"));
    assert!(md.contains("Entropy: `0.5000`"));
}

// ---------------------------------------------------------------------------
// 33. Reading time and integrity rendering
// ---------------------------------------------------------------------------

#[test]
fn md_reading_time_and_integrity() {
    let mut r = minimal_receipt();
    r.derived = Some(sample_derived());
    let md = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert!(md.contains("Minutes: `65.00` (20 lines/min)"));
    assert!(md.contains("Hash: `abc123` (`blake3`)"));
    assert!(md.contains("Entries: `10`"));
}
