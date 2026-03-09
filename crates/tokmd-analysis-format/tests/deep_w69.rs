//! Deep tests for tokmd-analysis-format rendering (W69).
//!
//! Covers: Markdown rendering, JSON rendering, section formatting,
//! deterministic output, empty/minimal data, XML, JSON-LD, SVG, Mermaid, Tree.

use tokmd_analysis_format::{RenderedOutput, render};
use tokmd_analysis_types::*;
use tokmd_types::{AnalysisFormat, ScanStatus, ToolInfo};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn minimal_receipt() -> AnalysisReceipt {
    AnalysisReceipt {
        effort: None,
        schema_version: ANALYSIS_SCHEMA_VERSION,
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
                key: "total".into(),
                numerator: 200,
                denominator: 1200,
                ratio: 0.1667,
            },
            by_lang: vec![RatioRow {
                key: "Rust".into(),
                numerator: 150,
                denominator: 900,
                ratio: 0.1667,
            }],
            by_module: vec![],
        },
        whitespace: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 100,
                denominator: 1300,
                ratio: 0.0769,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        verbosity: RateReport {
            total: RateRow {
                key: "total".into(),
                numerator: 50000,
                denominator: 1300,
                rate: 38.46,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        max_file: MaxFileReport {
            overall: FileStatRow {
                path: "src/lib.rs".into(),
                module: "src".into(),
                lang: "Rust".into(),
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
            infra_langs: vec!["TOML".into()],
        },
        polyglot: PolyglotReport {
            lang_count: 2,
            entropy: 0.5,
            dominant_lang: "Rust".into(),
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
            label: "Small".into(),
            min: 0,
            max: Some(100),
            files: 5,
            pct: 0.5,
        }],
        top: TopOffenders {
            largest_lines: vec![FileStatRow {
                path: "src/lib.rs".into(),
                module: "src".into(),
                lang: "Rust".into(),
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
        tree: Some("test-tree".into()),
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
            mode: "organic".into(),
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
                tag: "TODO".into(),
                count: 5,
            }],
        }),
        integrity: IntegrityReport {
            algo: "blake3".into(),
            hash: "abc123".into(),
            entries: 10,
        },
    }
}

fn extract_text(output: RenderedOutput) -> String {
    match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text, got binary"),
    }
}

// ===================================================================
// 1. Markdown rendering — minimal receipt
// ===================================================================

#[test]
fn md_minimal_receipt_starts_with_header() {
    let receipt = minimal_receipt();
    let out = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = extract_text(out);
    assert!(text.starts_with("# tokmd analysis"));
}

#[test]
fn md_minimal_receipt_shows_preset() {
    let receipt = minimal_receipt();
    let text = extract_text(render(&receipt, AnalysisFormat::Md).unwrap());
    assert!(text.contains("Preset: `receipt`"));
}

#[test]
fn md_minimal_receipt_shows_inputs() {
    let receipt = minimal_receipt();
    let text = extract_text(render(&receipt, AnalysisFormat::Md).unwrap());
    assert!(text.contains("- `test`"));
}

#[test]
fn md_minimal_receipt_no_derived_sections() {
    let receipt = minimal_receipt();
    let text = extract_text(render(&receipt, AnalysisFormat::Md).unwrap());
    assert!(!text.contains("## Totals"));
    assert!(!text.contains("## Distribution"));
}

// ===================================================================
// 2. Markdown rendering — derived sections
// ===================================================================

#[test]
fn md_derived_totals_table() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let text = extract_text(render(&receipt, AnalysisFormat::Md).unwrap());
    assert!(text.contains("## Totals"));
    assert!(text.contains("|10|1000|200|100|1300|50000|2500|"));
}

#[test]
fn md_derived_ratios_section() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let text = extract_text(render(&receipt, AnalysisFormat::Md).unwrap());
    assert!(text.contains("## Ratios"));
    assert!(text.contains("|Doc density|"));
    assert!(text.contains("|Whitespace ratio|"));
}

#[test]
fn md_derived_distribution_section() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let text = extract_text(render(&receipt, AnalysisFormat::Md).unwrap());
    assert!(text.contains("## Distribution"));
    assert!(text.contains("|10|50|650|"));
}

#[test]
fn md_derived_cocomo_section() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let text = extract_text(render(&receipt, AnalysisFormat::Md).unwrap());
    assert!(text.contains("## Effort estimate"));
    assert!(text.contains("### Size basis"));
    assert!(text.contains("### Headline"));
    assert!(text.contains("### Why"));
    assert!(text.contains("### Delta"));
    assert!(text.contains("Model: `COCOMO` (`organic` mode)"));
}

#[test]
fn md_derived_context_window_section() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let text = extract_text(render(&receipt, AnalysisFormat::Md).unwrap());
    assert!(text.contains("## Context window"));
    assert!(text.contains("Fits: `true`"));
}

#[test]
fn md_derived_todo_section() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let text = extract_text(render(&receipt, AnalysisFormat::Md).unwrap());
    assert!(text.contains("## TODOs"));
    assert!(text.contains("|TODO|5|"));
}

#[test]
fn md_derived_integrity_section() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let text = extract_text(render(&receipt, AnalysisFormat::Md).unwrap());
    assert!(text.contains("## Integrity"));
    assert!(text.contains("abc123"));
}

// ===================================================================
// 3. Markdown — optional sections
// ===================================================================

#[test]
fn md_archetype_section() {
    let mut receipt = minimal_receipt();
    receipt.archetype = Some(Archetype {
        kind: "cli-tool".into(),
        evidence: vec!["Cargo.toml".into(), "main.rs".into()],
    });
    let text = extract_text(render(&receipt, AnalysisFormat::Md).unwrap());
    assert!(text.contains("## Archetype"));
    assert!(text.contains("Kind: `cli-tool`"));
    assert!(text.contains("Cargo.toml`, `main.rs"));
}

#[test]
fn md_imports_section() {
    let mut receipt = minimal_receipt();
    receipt.imports = Some(ImportReport {
        granularity: "module".into(),
        edges: vec![ImportEdge {
            from: "src".into(),
            to: "serde".into(),
            count: 3,
        }],
    });
    let text = extract_text(render(&receipt, AnalysisFormat::Md).unwrap());
    assert!(text.contains("## Imports"));
    assert!(text.contains("|src|serde|3|"));
}

#[test]
fn md_entropy_no_suspects() {
    let mut receipt = minimal_receipt();
    receipt.entropy = Some(EntropyReport { suspects: vec![] });
    let text = extract_text(render(&receipt, AnalysisFormat::Md).unwrap());
    assert!(text.contains("No entropy outliers detected"));
}

// ===================================================================
// 4. JSON rendering
// ===================================================================

#[test]
fn json_minimal_receipt_is_valid() {
    let receipt = minimal_receipt();
    let text = extract_text(render(&receipt, AnalysisFormat::Json).unwrap());
    let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        parsed["schema_version"].as_u64().unwrap(),
        ANALYSIS_SCHEMA_VERSION as u64
    );
}

#[test]
fn json_round_trip_preserves_fields() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let text = extract_text(render(&receipt, AnalysisFormat::Json).unwrap());
    let parsed: AnalysisReceipt = serde_json::from_str(&text).unwrap();
    assert_eq!(parsed.derived.as_ref().unwrap().totals.code, 1000);
    assert_eq!(parsed.args.preset, "receipt");
}

#[test]
fn json_with_derived_includes_cocomo() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let text = extract_text(render(&receipt, AnalysisFormat::Json).unwrap());
    let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(parsed["derived"]["cocomo"]["mode"], "organic");
}

// ===================================================================
// 5. JSON-LD rendering
// ===================================================================

#[test]
fn jsonld_has_schema_org_context() {
    let receipt = minimal_receipt();
    let text = extract_text(render(&receipt, AnalysisFormat::Jsonld).unwrap());
    let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(parsed["@context"], "https://schema.org");
    assert_eq!(parsed["@type"], "SoftwareSourceCode");
}

#[test]
fn jsonld_uses_first_input_as_name() {
    let receipt = minimal_receipt();
    let text = extract_text(render(&receipt, AnalysisFormat::Jsonld).unwrap());
    let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(parsed["name"], "test");
}

// ===================================================================
// 6. XML rendering
// ===================================================================

#[test]
fn xml_minimal_receipt_wraps_analysis() {
    let receipt = minimal_receipt();
    let text = extract_text(render(&receipt, AnalysisFormat::Xml).unwrap());
    assert!(text.starts_with("<analysis>"));
    assert!(text.ends_with("</analysis>"));
}

#[test]
fn xml_with_derived_includes_totals() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let text = extract_text(render(&receipt, AnalysisFormat::Xml).unwrap());
    assert!(text.contains("files=\"10\""));
    assert!(text.contains("code=\"1000\""));
}

// ===================================================================
// 7. SVG rendering
// ===================================================================

#[test]
fn svg_minimal_receipt_is_valid_svg() {
    let receipt = minimal_receipt();
    let text = extract_text(render(&receipt, AnalysisFormat::Svg).unwrap());
    assert!(text.starts_with("<svg"));
    assert!(text.ends_with("</svg>"));
}

#[test]
fn svg_with_context_window_shows_pct() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let text = extract_text(render(&receipt, AnalysisFormat::Svg).unwrap());
    assert!(text.contains("context"));
    assert!(text.contains("2.5%"));
}

// ===================================================================
// 8. Mermaid rendering
// ===================================================================

#[test]
fn mermaid_starts_with_graph_td() {
    let receipt = minimal_receipt();
    let text = extract_text(render(&receipt, AnalysisFormat::Mermaid).unwrap());
    assert!(text.starts_with("graph TD\n"));
}

#[test]
fn mermaid_includes_import_edges() {
    let mut receipt = minimal_receipt();
    receipt.imports = Some(ImportReport {
        granularity: "module".into(),
        edges: vec![ImportEdge {
            from: "src/main".into(),
            to: "lib/utils".into(),
            count: 2,
        }],
    });
    let text = extract_text(render(&receipt, AnalysisFormat::Mermaid).unwrap());
    assert!(text.contains("-->|2|"));
}

// ===================================================================
// 9. Tree rendering
// ===================================================================

#[test]
fn tree_unavailable_without_derived() {
    let receipt = minimal_receipt();
    let text = extract_text(render(&receipt, AnalysisFormat::Tree).unwrap());
    assert_eq!(text, "(tree unavailable)");
}

#[test]
fn tree_returns_derived_tree() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let text = extract_text(render(&receipt, AnalysisFormat::Tree).unwrap());
    assert_eq!(text, "test-tree");
}

// ===================================================================
// 10. Determinism
// ===================================================================

#[test]
fn md_rendering_is_deterministic() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let a = extract_text(render(&receipt, AnalysisFormat::Md).unwrap());
    let b = extract_text(render(&receipt, AnalysisFormat::Md).unwrap());
    assert_eq!(a, b, "Markdown output must be deterministic");
}

#[test]
fn json_rendering_is_deterministic() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let a = extract_text(render(&receipt, AnalysisFormat::Json).unwrap());
    let b = extract_text(render(&receipt, AnalysisFormat::Json).unwrap());
    assert_eq!(a, b, "JSON output must be deterministic");
}

#[test]
fn xml_rendering_is_deterministic() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let a = extract_text(render(&receipt, AnalysisFormat::Xml).unwrap());
    let b = extract_text(render(&receipt, AnalysisFormat::Xml).unwrap());
    assert_eq!(a, b, "XML output must be deterministic");
}
