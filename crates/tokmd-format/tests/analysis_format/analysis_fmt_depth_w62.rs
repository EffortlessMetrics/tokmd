//! Depth tests for tokmd-format analysis (W62).
//!
//! Covers Markdown, JSON, TSV-style, empty results, section ordering,
//! truncation behaviour, determinism, property tests, and insta snapshots.

use proptest::prelude::*;
use tokmd_analysis_types::{
    ANALYSIS_SCHEMA_VERSION, AnalysisArgsMeta, AnalysisReceipt, AnalysisSource, BoilerplateReport,
    CocomoReport, ContextWindowReport, DerivedReport, DerivedTotals, DistributionReport,
    FileStatRow, HistogramBucket, IntegrityReport, LangPurityReport, LangPurityRow, MaxFileReport,
    NestingReport, PolyglotReport, RateReport, RateRow, RatioReport, RatioRow, ReadingTimeReport,
    TestDensityReport, TodoReport, TodoTagRow, TopOffenders,
};
use tokmd_types::{AnalysisFormat, ScanStatus, ToolInfo};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn tool() -> ToolInfo {
    ToolInfo {
        name: "tokmd".into(),
        version: "0.0.0-test".into(),
    }
}

fn source(inputs: Vec<&str>) -> AnalysisSource {
    AnalysisSource {
        inputs: inputs.into_iter().map(String::from).collect(),
        export_path: None,
        base_receipt_path: None,
        export_schema_version: None,
        export_generated_at_ms: None,
        base_signature: None,
        module_roots: vec![],
        module_depth: 1,
        children: "collapse".into(),
    }
}

fn args(preset: &str, format: &str) -> AnalysisArgsMeta {
    AnalysisArgsMeta {
        preset: preset.into(),
        format: format.into(),
        window_tokens: None,
        git: None,
        max_files: None,
        max_bytes: None,
        max_commits: None,
        max_commit_files: None,
        max_file_bytes: None,
        import_granularity: "module".into(),
    }
}

fn empty_receipt() -> AnalysisReceipt {
    AnalysisReceipt {
        effort: None,
        schema_version: ANALYSIS_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tool(),
        mode: "analysis".into(),
        status: ScanStatus::Complete,
        warnings: vec![],
        source: source(vec![]),
        args: args("receipt", "md"),
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
            code: 500,
            comments: 50,
            blanks: 100,
            lines: 650,
            bytes: 20000,
            tokens: 3000,
        },
        doc_density: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 50,
                denominator: 550,
                ratio: 50.0 / 550.0,
            },
            by_lang: vec![RatioRow {
                key: "Rust".into(),
                numerator: 30,
                denominator: 300,
                ratio: 0.10,
            }],
            by_module: vec![],
        },
        whitespace: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 100,
                denominator: 550,
                ratio: 100.0 / 550.0,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        verbosity: RateReport {
            total: RateRow {
                key: "total".into(),
                numerator: 20000,
                denominator: 650,
                rate: 20000.0 / 650.0,
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
                comments: 20,
                blanks: 30,
                lines: 250,
                bytes: 8000,
                tokens: 1200,
                doc_pct: Some(0.10),
                bytes_per_line: Some(32.0),
                depth: 1,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        lang_purity: LangPurityReport {
            rows: vec![LangPurityRow {
                module: "src".into(),
                lang_count: 1,
                dominant_lang: "Rust".into(),
                dominant_lines: 500,
                dominant_pct: 1.0,
            }],
        },
        nesting: NestingReport {
            max: 3,
            avg: 1.5,
            by_module: vec![],
        },
        test_density: TestDensityReport {
            test_lines: 100,
            prod_lines: 400,
            test_files: 2,
            prod_files: 8,
            ratio: 0.25,
        },
        boilerplate: BoilerplateReport {
            infra_lines: 50,
            logic_lines: 450,
            ratio: 50.0 / 500.0,
            infra_langs: vec!["TOML".into()],
        },
        polyglot: PolyglotReport {
            lang_count: 2,
            entropy: 0.5,
            dominant_lang: "Rust".into(),
            dominant_lines: 400,
            dominant_pct: 0.80,
        },
        distribution: DistributionReport {
            count: 10,
            min: 10,
            max: 200,
            mean: 65.0,
            median: 50.0,
            p90: 180.0,
            p99: 200.0,
            gini: 0.35,
        },
        histogram: vec![
            HistogramBucket {
                label: "tiny".into(),
                min: 0,
                max: Some(50),
                files: 4,
                pct: 0.40,
            },
            HistogramBucket {
                label: "large".into(),
                min: 51,
                max: None,
                files: 6,
                pct: 0.60,
            },
        ],
        top: TopOffenders {
            largest_lines: vec![FileStatRow {
                path: "src/main.rs".into(),
                module: "src".into(),
                lang: "Rust".into(),
                code: 200,
                comments: 20,
                blanks: 30,
                lines: 250,
                bytes: 8000,
                tokens: 1200,
                doc_pct: Some(0.10),
                bytes_per_line: Some(32.0),
                depth: 1,
            }],
            largest_tokens: vec![],
            largest_bytes: vec![],
            least_documented: vec![],
            most_dense: vec![],
        },
        tree: None,
        reading_time: ReadingTimeReport {
            minutes: 3.25,
            lines_per_minute: 200,
            basis_lines: 650,
        },
        context_window: Some(ContextWindowReport {
            window_tokens: 128000,
            total_tokens: 3000,
            pct: 3000.0 / 128000.0,
            fits: true,
        }),
        cocomo: Some(CocomoReport {
            mode: "organic".into(),
            kloc: 0.5,
            effort_pm: 1.2,
            duration_months: 2.5,
            staff: 0.48,
            a: 2.4,
            b: 1.05,
            c: 2.5,
            d: 0.38,
        }),
        todo: Some(TodoReport {
            total: 5,
            density_per_kloc: 10.0,
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
        }),
        integrity: IntegrityReport {
            algo: "blake3".into(),
            hash: "abcdef1234567890".into(),
            entries: 10,
        },
    }
}

fn receipt_with_derived() -> AnalysisReceipt {
    let mut r = empty_receipt();
    r.derived = Some(sample_derived());
    r
}

fn render_text(receipt: &AnalysisReceipt, format: AnalysisFormat) -> String {
    match tokmd_format::analysis::render(receipt, format).unwrap() {
        tokmd_format::analysis::RenderedOutput::Text(t) => t,
        tokmd_format::analysis::RenderedOutput::Binary(_) => panic!("expected text"),
    }
}

// =========================================================================
// 1. Markdown rendering
// =========================================================================

#[test]
fn md_empty_receipt_has_header() {
    let md = render_text(&empty_receipt(), AnalysisFormat::Md);
    assert!(md.starts_with("# tokmd analysis"));
}

#[test]
fn md_empty_receipt_shows_preset() {
    let md = render_text(&empty_receipt(), AnalysisFormat::Md);
    assert!(md.contains("Preset: `receipt`"));
}

#[test]
fn md_with_inputs_shows_inputs_section() {
    let mut r = empty_receipt();
    r.source.inputs = vec!["src".into(), "lib".into()];
    let md = render_text(&r, AnalysisFormat::Md);
    assert!(md.contains("## Inputs"));
    assert!(md.contains("- `src`"));
    assert!(md.contains("- `lib`"));
}

#[test]
fn md_no_inputs_omits_inputs_section() {
    let md = render_text(&empty_receipt(), AnalysisFormat::Md);
    assert!(!md.contains("## Inputs"));
}

#[test]
fn md_derived_totals_table() {
    let md = render_text(&receipt_with_derived(), AnalysisFormat::Md);
    assert!(md.contains("## Totals"));
    assert!(md.contains("|10|500|50|100|650|20000|3000|"));
}

#[test]
fn md_ratios_section() {
    let md = render_text(&receipt_with_derived(), AnalysisFormat::Md);
    assert!(md.contains("## Ratios"));
    assert!(md.contains("|Doc density|"));
    assert!(md.contains("|Whitespace ratio|"));
    assert!(md.contains("|Bytes per line|"));
}

#[test]
fn md_distribution_section() {
    let md = render_text(&receipt_with_derived(), AnalysisFormat::Md);
    assert!(md.contains("## Distribution"));
    assert!(md.contains("|10|10|200|"));
}

#[test]
fn md_histogram_section() {
    let md = render_text(&receipt_with_derived(), AnalysisFormat::Md);
    assert!(md.contains("## File size histogram"));
    assert!(md.contains("|tiny|0|50|4|"));
    // Unbounded max renders as ∞
    assert!(md.contains("∞"));
}

#[test]
fn md_top_offenders_section() {
    let md = render_text(&receipt_with_derived(), AnalysisFormat::Md);
    assert!(md.contains("## Top offenders"));
    assert!(md.contains("### Largest files by lines"));
    assert!(md.contains("src/main.rs"));
}

#[test]
fn md_structure_section() {
    let md = render_text(&receipt_with_derived(), AnalysisFormat::Md);
    assert!(md.contains("## Structure"));
    assert!(md.contains("Max depth: `3`"));
    assert!(md.contains("Avg depth: `1.50`"));
}

#[test]
fn md_test_density_section() {
    let md = render_text(&receipt_with_derived(), AnalysisFormat::Md);
    assert!(md.contains("## Test density"));
    assert!(md.contains("Test lines: `100`"));
    assert!(md.contains("Prod lines: `400`"));
}

#[test]
fn md_todo_section() {
    let md = render_text(&receipt_with_derived(), AnalysisFormat::Md);
    assert!(md.contains("## TODOs"));
    assert!(md.contains("Total: `5`"));
    assert!(md.contains("|TODO|3|"));
    assert!(md.contains("|FIXME|2|"));
}

#[test]
fn md_boilerplate_section() {
    let md = render_text(&receipt_with_derived(), AnalysisFormat::Md);
    assert!(md.contains("## Boilerplate ratio"));
    assert!(md.contains("Infra lines: `50`"));
}

#[test]
fn md_polyglot_section() {
    let md = render_text(&receipt_with_derived(), AnalysisFormat::Md);
    assert!(md.contains("## Polyglot"));
    assert!(md.contains("Languages: `2`"));
    assert!(md.contains("Dominant: `Rust`"));
}

#[test]
fn md_reading_time_section() {
    let md = render_text(&receipt_with_derived(), AnalysisFormat::Md);
    assert!(md.contains("## Reading time"));
    assert!(md.contains("Minutes: `3.25`"));
}

#[test]
fn md_context_window_section() {
    let md = render_text(&receipt_with_derived(), AnalysisFormat::Md);
    assert!(md.contains("## Context window"));
    assert!(md.contains("Window tokens: `128000`"));
    assert!(md.contains("Fits: `true`"));
}

#[test]
fn md_cocomo_section() {
    let md = render_text(&receipt_with_derived(), AnalysisFormat::Md);
    assert!(md.contains("## Effort estimate"));
    assert!(md.contains("### Size basis"));
    assert!(md.contains("### Headline"));
    assert!(md.contains("### Why"));
    assert!(md.contains("### Delta"));
    assert!(md.contains("Model: `COCOMO` (`organic` mode)"));
    assert!(md.contains("KLOC: `0.5000`"));
}

#[test]
fn md_integrity_section() {
    let md = render_text(&receipt_with_derived(), AnalysisFormat::Md);
    assert!(md.contains("## Integrity"));
    assert!(md.contains("abcdef1234567890"));
    assert!(md.contains("`blake3`"));
}

#[test]
fn md_no_context_window_when_absent() {
    let mut r = receipt_with_derived();
    r.derived.as_mut().unwrap().context_window = None;
    let md = render_text(&r, AnalysisFormat::Md);
    assert!(!md.contains("## Context window"));
}

#[test]
fn md_no_cocomo_when_absent() {
    let mut r = receipt_with_derived();
    r.derived.as_mut().unwrap().cocomo = None;
    let md = render_text(&r, AnalysisFormat::Md);
    assert!(!md.contains("## Effort estimate"));
}

#[test]
fn md_no_todo_when_absent() {
    let mut r = receipt_with_derived();
    r.derived.as_mut().unwrap().todo = None;
    let md = render_text(&r, AnalysisFormat::Md);
    assert!(!md.contains("## TODOs"));
}

#[test]
fn md_doc_density_by_lang_table() {
    let md = render_text(&receipt_with_derived(), AnalysisFormat::Md);
    assert!(md.contains("### Doc density by language"));
    assert!(md.contains("|Rust|"));
}

// =========================================================================
// 2. JSON rendering
// =========================================================================

#[test]
fn json_empty_receipt_valid() {
    let json = render_text(&empty_receipt(), AnalysisFormat::Json);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["schema_version"], ANALYSIS_SCHEMA_VERSION);
}

#[test]
fn json_derived_totals_present() {
    let json = render_text(&receipt_with_derived(), AnalysisFormat::Json);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["derived"]["totals"]["code"], 500);
    assert_eq!(v["derived"]["totals"]["files"], 10);
}

#[test]
fn json_cocomo_present() {
    let json = render_text(&receipt_with_derived(), AnalysisFormat::Json);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["derived"]["cocomo"]["mode"], "organic");
}

#[test]
fn json_null_when_absent() {
    let json = render_text(&empty_receipt(), AnalysisFormat::Json);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(v["derived"].is_null());
    assert!(v["git"].is_null());
    assert!(v["assets"].is_null());
}

#[test]
fn json_roundtrip_preserves_status() {
    let json = render_text(&receipt_with_derived(), AnalysisFormat::Json);
    let roundtrip: AnalysisReceipt = serde_json::from_str(&json).unwrap();
    assert!(matches!(roundtrip.status, ScanStatus::Complete));
}

#[test]
fn json_roundtrip_preserves_warnings() {
    let mut r = receipt_with_derived();
    r.warnings = vec!["warn1".into(), "warn2".into()];
    r.status = ScanStatus::Partial;
    let json = render_text(&r, AnalysisFormat::Json);
    let roundtrip: AnalysisReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(roundtrip.warnings, vec!["warn1", "warn2"]);
    assert!(matches!(roundtrip.status, ScanStatus::Partial));
}

#[test]
fn json_integrity_hash_present() {
    let json = render_text(&receipt_with_derived(), AnalysisFormat::Json);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["derived"]["integrity"]["hash"], "abcdef1234567890");
}

// =========================================================================
// 3. XML rendering
// =========================================================================

#[test]
fn xml_empty_has_analysis_tag() {
    let xml = render_text(&empty_receipt(), AnalysisFormat::Xml);
    assert!(xml.contains("<analysis>"));
    assert!(xml.contains("</analysis>"));
}

#[test]
fn xml_totals_present() {
    let xml = render_text(&receipt_with_derived(), AnalysisFormat::Xml);
    assert!(xml.contains("code=\"500\""));
    assert!(xml.contains("files=\"10\""));
}

#[test]
fn xml_no_totals_when_derived_absent() {
    let xml = render_text(&empty_receipt(), AnalysisFormat::Xml);
    assert!(!xml.contains("code="));
}

// =========================================================================
// 4. SVG rendering
// =========================================================================

#[test]
fn svg_is_valid_svg() {
    let svg = render_text(&receipt_with_derived(), AnalysisFormat::Svg);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
}

#[test]
fn svg_shows_context_when_available() {
    let svg = render_text(&receipt_with_derived(), AnalysisFormat::Svg);
    // Context window is present so label should be "context"
    assert!(svg.contains("context"));
}

#[test]
fn svg_shows_tokens_when_no_context() {
    let mut r = receipt_with_derived();
    r.derived.as_mut().unwrap().context_window = None;
    let svg = render_text(&r, AnalysisFormat::Svg);
    assert!(svg.contains("tokens"));
}

// =========================================================================
// 5. Mermaid rendering
// =========================================================================

#[test]
fn mermaid_has_graph_header() {
    let out = render_text(&empty_receipt(), AnalysisFormat::Mermaid);
    assert!(out.starts_with("graph TD"));
}

#[test]
fn mermaid_empty_imports_minimal() {
    let out = render_text(&empty_receipt(), AnalysisFormat::Mermaid);
    // Just the header line with no edges
    assert_eq!(out.lines().count(), 1);
}

// =========================================================================
// 6. JSON-LD rendering
// =========================================================================

#[test]
fn jsonld_has_context() {
    let ld = render_text(&empty_receipt(), AnalysisFormat::Jsonld);
    let v: serde_json::Value = serde_json::from_str(&ld).unwrap();
    assert_eq!(v["@context"], "https://schema.org");
    assert_eq!(v["@type"], "SoftwareSourceCode");
}

#[test]
fn jsonld_code_lines_from_derived() {
    let ld = render_text(&receipt_with_derived(), AnalysisFormat::Jsonld);
    let v: serde_json::Value = serde_json::from_str(&ld).unwrap();
    assert_eq!(v["codeLines"], 500);
}

// =========================================================================
// 7. Tree rendering
// =========================================================================

#[test]
fn tree_unavailable_when_no_derived() {
    let out = render_text(&empty_receipt(), AnalysisFormat::Tree);
    assert!(out.contains("(tree unavailable)"));
}

#[test]
fn tree_unavailable_when_tree_is_none() {
    let r = receipt_with_derived();
    // tree is None by default in our sample
    let out = render_text(&r, AnalysisFormat::Tree);
    assert!(out.contains("(tree unavailable)"));
}

#[test]
fn tree_renders_when_set() {
    let mut r = receipt_with_derived();
    r.derived.as_mut().unwrap().tree = Some("root\n  src/\n    main.rs".into());
    let out = render_text(&r, AnalysisFormat::Tree);
    assert!(out.contains("root"));
    assert!(out.contains("main.rs"));
}

// =========================================================================
// 8. Empty results rendering
// =========================================================================

#[test]
fn empty_receipt_md_has_no_tables() {
    let md = render_text(&empty_receipt(), AnalysisFormat::Md);
    // No derived means no totals table
    assert!(!md.contains("## Totals"));
    assert!(!md.contains("## Distribution"));
}

#[test]
fn empty_receipt_json_is_well_formed() {
    let json = render_text(&empty_receipt(), AnalysisFormat::Json);
    let _v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
}

#[test]
fn empty_receipt_xml_is_minimal() {
    let xml = render_text(&empty_receipt(), AnalysisFormat::Xml);
    assert_eq!(xml, "<analysis></analysis>");
}

// =========================================================================
// 9. Section ordering in Markdown
// =========================================================================

#[test]
fn md_section_order_header_before_totals() {
    let md = render_text(&receipt_with_derived(), AnalysisFormat::Md);
    let header_pos = md.find("# tokmd analysis").unwrap();
    let totals_pos = md.find("## Totals").unwrap();
    assert!(header_pos < totals_pos);
}

#[test]
fn md_section_order_totals_before_distribution() {
    let md = render_text(&receipt_with_derived(), AnalysisFormat::Md);
    let totals_pos = md.find("## Totals").unwrap();
    let dist_pos = md.find("## Distribution").unwrap();
    assert!(totals_pos < dist_pos);
}

#[test]
fn md_section_order_distribution_before_histogram() {
    let md = render_text(&receipt_with_derived(), AnalysisFormat::Md);
    let dist_pos = md.find("## Distribution").unwrap();
    let hist_pos = md.find("## File size histogram").unwrap();
    assert!(dist_pos < hist_pos);
}

#[test]
fn md_section_order_integrity_after_effort() {
    let md = render_text(&receipt_with_derived(), AnalysisFormat::Md);
    let effort_pos = md.find("## Effort estimate").unwrap();
    let int_pos = md.find("## Integrity").unwrap();
    assert!(effort_pos < int_pos);
}

#[test]
fn md_section_order_polyglot_before_reading_time() {
    let md = render_text(&receipt_with_derived(), AnalysisFormat::Md);
    let poly_pos = md.find("## Polyglot").unwrap();
    let read_pos = md.find("## Reading time").unwrap();
    assert!(poly_pos < read_pos);
}

// =========================================================================
// 10. Truncation / limits
// =========================================================================

#[test]
fn md_histogram_unbounded_max_renders_infinity() {
    let md = render_text(&receipt_with_derived(), AnalysisFormat::Md);
    assert!(md.contains("∞"));
}

#[test]
fn md_doc_density_by_lang_limited_to_10() {
    let mut r = receipt_with_derived();
    let d = r.derived.as_mut().unwrap();
    d.doc_density.by_lang = (0..20)
        .map(|i| RatioRow {
            key: format!("Lang{i}"),
            numerator: i,
            denominator: 100,
            ratio: i as f64 / 100.0,
        })
        .collect();
    let md = render_text(&r, AnalysisFormat::Md);
    // take(10) means only first 10 appear
    assert!(md.contains("Lang0"));
    assert!(md.contains("Lang9"));
    assert!(!md.contains("Lang10"));
}

// =========================================================================
// 11. Determinism
// =========================================================================

#[test]
fn md_deterministic_across_calls() {
    let r = receipt_with_derived();
    let a = render_text(&r, AnalysisFormat::Md);
    let b = render_text(&r, AnalysisFormat::Md);
    assert_eq!(a, b, "Markdown output must be deterministic");
}

#[test]
fn json_deterministic_across_calls() {
    let r = receipt_with_derived();
    let a = render_text(&r, AnalysisFormat::Json);
    let b = render_text(&r, AnalysisFormat::Json);
    assert_eq!(a, b, "JSON output must be deterministic");
}

#[test]
fn xml_deterministic_across_calls() {
    let r = receipt_with_derived();
    let a = render_text(&r, AnalysisFormat::Xml);
    let b = render_text(&r, AnalysisFormat::Xml);
    assert_eq!(a, b, "XML output must be deterministic");
}

#[test]
fn svg_deterministic_across_calls() {
    let r = receipt_with_derived();
    let a = render_text(&r, AnalysisFormat::Svg);
    let b = render_text(&r, AnalysisFormat::Svg);
    assert_eq!(a, b, "SVG output must be deterministic");
}

#[test]
fn mermaid_deterministic_across_calls() {
    let r = receipt_with_derived();
    let a = render_text(&r, AnalysisFormat::Mermaid);
    let b = render_text(&r, AnalysisFormat::Mermaid);
    assert_eq!(a, b, "Mermaid output must be deterministic");
}

// =========================================================================
// 12. Property tests
// =========================================================================

proptest! {
    #[test]
    fn prop_md_always_starts_with_header(preset in "receipt|health|risk|deep") {
        let mut r = empty_receipt();
        r.args.preset = preset;
        let md = render_text(&r, AnalysisFormat::Md);
        prop_assert!(md.starts_with("# tokmd analysis"));
    }

    #[test]
    fn prop_json_always_valid(code in 0usize..100_000, files in 1usize..1000) {
        let mut r = receipt_with_derived();
        let d = r.derived.as_mut().unwrap();
        d.totals.code = code;
        d.totals.files = files;
        let json = render_text(&r, AnalysisFormat::Json);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(v["derived"]["totals"]["code"].as_u64().unwrap(), code as u64);
    }

    #[test]
    fn prop_xml_always_well_formed(code in 0usize..100_000) {
        let mut r = receipt_with_derived();
        r.derived.as_mut().unwrap().totals.code = code;
        let xml = render_text(&r, AnalysisFormat::Xml);
        prop_assert!(xml.starts_with("<analysis>"));
        prop_assert!(xml.ends_with("</analysis>"));
    }

    #[test]
    fn prop_md_preset_appears(preset in "[a-z]{3,10}") {
        let mut r = empty_receipt();
        r.args.preset = preset.clone();
        let md = render_text(&r, AnalysisFormat::Md);
        let expected = format!("Preset: `{}`", preset);
        prop_assert!(md.contains(&expected));
    }

    #[test]
    fn prop_json_roundtrip(code in 0usize..50_000, comments in 0usize..10_000) {
        let mut r = receipt_with_derived();
        let d = r.derived.as_mut().unwrap();
        d.totals.code = code;
        d.totals.comments = comments;
        let json = render_text(&r, AnalysisFormat::Json);
        let rt: AnalysisReceipt = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(rt.derived.as_ref().unwrap().totals.code, code);
        prop_assert_eq!(rt.derived.as_ref().unwrap().totals.comments, comments);
    }
}

// =========================================================================
// 13. Snapshot tests
// =========================================================================

#[test]
fn snapshot_empty_receipt_md() {
    let md = render_text(&empty_receipt(), AnalysisFormat::Md);
    insta::assert_snapshot!("empty_receipt_md_w62", md);
}

#[test]
fn snapshot_derived_receipt_md() {
    let mut r = receipt_with_derived();
    // Fix timestamp for deterministic snapshot
    r.generated_at_ms = 1700000000000;
    let md = render_text(&r, AnalysisFormat::Md);
    insta::assert_snapshot!("derived_receipt_md_w62", md);
}

#[test]
fn snapshot_empty_receipt_xml() {
    let xml = render_text(&empty_receipt(), AnalysisFormat::Xml);
    insta::assert_snapshot!("empty_receipt_xml_w62", xml);
}

#[test]
fn snapshot_derived_receipt_xml() {
    let xml = render_text(&receipt_with_derived(), AnalysisFormat::Xml);
    insta::assert_snapshot!("derived_receipt_xml_w62", xml);
}

#[test]
fn snapshot_derived_receipt_jsonld() {
    let ld = render_text(&receipt_with_derived(), AnalysisFormat::Jsonld);
    insta::assert_snapshot!("derived_receipt_jsonld_w62", ld);
}

// =========================================================================
// 14. HTML rendering
// =========================================================================

#[test]
fn html_empty_receipt_produces_output() {
    let html = render_text(&empty_receipt(), AnalysisFormat::Html);
    assert!(!html.is_empty());
}

#[test]
fn html_contains_totals() {
    let html = render_text(&receipt_with_derived(), AnalysisFormat::Html);
    // HTML should reference code count somewhere
    assert!(html.contains("500") || html.contains("code"));
}

// =========================================================================
// 15. Warnings & status
// =========================================================================

#[test]
fn json_warnings_reflected_in_output() {
    let mut r = empty_receipt();
    r.warnings = vec!["something went wrong".into()];
    r.status = ScanStatus::Partial;
    let json = render_text(&r, AnalysisFormat::Json);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["warnings"][0], "something went wrong");
    assert_eq!(v["status"], "partial");
}

#[test]
fn json_complete_status_rendered() {
    let json = render_text(&empty_receipt(), AnalysisFormat::Json);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["status"], "complete");
}
