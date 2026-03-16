//! Deep analysis-format tests (wave 48).
//!
//! Covers:
//! - Analysis output rendering (Markdown, JSON, XML, SVG, Mermaid, Tree, HTML)
//! - All renderers handle empty analysis results
//! - Determinism: same input → same rendered output
//! - Schema version in rendered JSON

use tokmd_analysis_format::{RenderedOutput, render};
use tokmd_analysis_types::*;
use tokmd_types::{AnalysisFormat, ScanStatus, ToolInfo};

// ─── Helpers ────────────────────────────────────────────────────────────────

fn minimal_receipt() -> AnalysisReceipt {
    AnalysisReceipt {
        effort: None,
        schema_version: ANALYSIS_SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: ToolInfo {
            name: "tokmd".to_string(),
            version: "0.0.0-test".to_string(),
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
            format: "json".to_string(),
            window_tokens: None,
            git: None,
            max_files: None,
            max_bytes: None,
            max_file_bytes: None,
            max_commits: None,
            max_commit_files: None,
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

fn receipt_with_derived() -> AnalysisReceipt {
    let mut r = minimal_receipt();
    r.derived = Some(DerivedReport {
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
                key: "total".to_string(),
                numerator: 60,
                denominator: 360,
                ratio: 0.1667,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        whitespace: RatioReport {
            total: RatioRow {
                key: "total".to_string(),
                numerator: 30,
                denominator: 390,
                ratio: 0.0769,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        verbosity: RateReport {
            total: RateRow {
                key: "total".to_string(),
                numerator: 3000,
                denominator: 390,
                rate: 7.69,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        max_file: MaxFileReport {
            overall: FileStatRow {
                path: "src/lib.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                code: 200,
                comments: 40,
                blanks: 20,
                lines: 260,
                bytes: 2000,
                tokens: 500,
                doc_pct: Some(0.167),
                bytes_per_line: Some(7.69),
                depth: 1,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        lang_purity: LangPurityReport { rows: vec![] },
        nesting: NestingReport {
            max: 2,
            avg: 1.0,
            by_module: vec![],
        },
        test_density: TestDensityReport {
            test_lines: 100,
            prod_lines: 200,
            test_files: 1,
            prod_files: 2,
            ratio: 0.5,
        },
        boilerplate: BoilerplateReport {
            infra_lines: 30,
            logic_lines: 330,
            ratio: 0.083,
            infra_langs: vec!["TOML".to_string()],
        },
        polyglot: PolyglotReport {
            lang_count: 2,
            entropy: 0.3,
            dominant_lang: "Rust".to_string(),
            dominant_lines: 300,
            dominant_pct: 0.91,
        },
        distribution: DistributionReport {
            count: 3,
            min: 30,
            max: 260,
            mean: 130.0,
            median: 100.0,
            p90: 260.0,
            p99: 260.0,
            gini: 0.25,
        },
        histogram: vec![],
        top: TopOffenders {
            largest_lines: vec![],
            largest_tokens: vec![],
            largest_bytes: vec![],
            least_documented: vec![],
            most_dense: vec![],
        },
        tree: None,
        reading_time: ReadingTimeReport {
            minutes: 19.5,
            lines_per_minute: 20,
            basis_lines: 390,
        },
        context_window: None,
        cocomo: None,
        integrity: IntegrityReport {
            hash: "abc123".to_string(),
            algo: "blake3".to_string(),
            entries: 3,
        },
        todo: None,
    });
    r
}

fn extract_text(output: RenderedOutput) -> String {
    match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("Expected text output"),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. Rendering format coverage — all text formats produce non-empty output
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn render_md_produces_header() {
    let text = extract_text(render(&minimal_receipt(), AnalysisFormat::Md).unwrap());
    assert!(text.starts_with("# tokmd analysis"));
}

#[test]
fn render_json_is_valid_json() {
    let text = extract_text(render(&minimal_receipt(), AnalysisFormat::Json).unwrap());
    let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert!(parsed.is_object());
}

#[test]
fn render_xml_produces_xml_header() {
    let text = extract_text(render(&minimal_receipt(), AnalysisFormat::Xml).unwrap());
    assert!(
        text.contains('<'),
        "XML output should contain angle brackets"
    );
}

#[test]
fn render_svg_produces_svg_content() {
    let text = extract_text(render(&minimal_receipt(), AnalysisFormat::Svg).unwrap());
    assert!(
        text.contains("svg") || text.contains("SVG") || text.contains('<'),
        "SVG output should contain svg-related content"
    );
}

#[test]
fn render_mermaid_produces_output() {
    let text = extract_text(render(&minimal_receipt(), AnalysisFormat::Mermaid).unwrap());
    assert!(!text.is_empty(), "Mermaid output should not be empty");
}

#[test]
fn render_tree_produces_output() {
    let text = extract_text(render(&minimal_receipt(), AnalysisFormat::Tree).unwrap());
    assert!(!text.is_empty(), "Tree output should not be empty");
}

#[test]
fn render_html_produces_output() {
    let text = extract_text(render(&minimal_receipt(), AnalysisFormat::Html).unwrap());
    assert!(!text.is_empty(), "HTML output should not be empty");
}

#[test]
fn render_jsonld_produces_valid_json() {
    let text = extract_text(render(&minimal_receipt(), AnalysisFormat::Jsonld).unwrap());
    let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert!(parsed.is_object());
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Empty analysis results handling
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn empty_receipt_renders_md_without_panic() {
    let result = render(&minimal_receipt(), AnalysisFormat::Md);
    assert!(result.is_ok());
}

#[test]
fn empty_receipt_renders_json_without_panic() {
    let result = render(&minimal_receipt(), AnalysisFormat::Json);
    assert!(result.is_ok());
}

#[test]
fn empty_receipt_renders_all_text_formats() {
    let text_formats = [
        AnalysisFormat::Md,
        AnalysisFormat::Json,
        AnalysisFormat::Jsonld,
        AnalysisFormat::Xml,
        AnalysisFormat::Svg,
        AnalysisFormat::Mermaid,
        AnalysisFormat::Tree,
        AnalysisFormat::Html,
    ];
    for fmt in &text_formats {
        let result = render(&minimal_receipt(), *fmt);
        assert!(result.is_ok(), "Empty receipt should render for {:?}", fmt,);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Determinism: same input → same rendered output
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn md_rendering_is_deterministic() {
    let r = receipt_with_derived();
    let a = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    let b = extract_text(render(&r, AnalysisFormat::Md).unwrap());
    assert_eq!(a, b, "Markdown rendering should be deterministic");
}

#[test]
fn json_rendering_is_deterministic() {
    let r = receipt_with_derived();
    let a = extract_text(render(&r, AnalysisFormat::Json).unwrap());
    let b = extract_text(render(&r, AnalysisFormat::Json).unwrap());
    assert_eq!(a, b, "JSON rendering should be deterministic");
}

#[test]
fn xml_rendering_is_deterministic() {
    let r = receipt_with_derived();
    let a = extract_text(render(&r, AnalysisFormat::Xml).unwrap());
    let b = extract_text(render(&r, AnalysisFormat::Xml).unwrap());
    assert_eq!(a, b, "XML rendering should be deterministic");
}

#[test]
fn tree_rendering_is_deterministic() {
    let r = receipt_with_derived();
    let a = extract_text(render(&r, AnalysisFormat::Tree).unwrap());
    let b = extract_text(render(&r, AnalysisFormat::Tree).unwrap());
    assert_eq!(a, b, "Tree rendering should be deterministic");
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Schema version in rendered JSON
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn json_contains_schema_version() {
    let text = extract_text(render(&minimal_receipt(), AnalysisFormat::Json).unwrap());
    let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        parsed["schema_version"].as_u64(),
        Some(ANALYSIS_SCHEMA_VERSION as u64)
    );
}

#[test]
fn json_contains_mode_field() {
    let text = extract_text(render(&minimal_receipt(), AnalysisFormat::Json).unwrap());
    let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(parsed["mode"].as_str(), Some("analysis"));
}

#[test]
fn json_contains_tool_info() {
    let text = extract_text(render(&minimal_receipt(), AnalysisFormat::Json).unwrap());
    let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(parsed["tool"]["name"].as_str(), Some("tokmd"));
}

#[test]
fn json_roundtrip_through_render_preserves_data() {
    let original = receipt_with_derived();
    let json_text = extract_text(render(&original, AnalysisFormat::Json).unwrap());
    let restored: AnalysisReceipt = serde_json::from_str(&json_text).unwrap();
    assert_eq!(restored.schema_version, original.schema_version);
    assert_eq!(restored.args.preset, original.args.preset);
    assert_eq!(
        restored.derived.as_ref().unwrap().totals.code,
        original.derived.as_ref().unwrap().totals.code
    );
}

#[test]
fn md_with_derived_contains_totals_section() {
    let text = extract_text(render(&receipt_with_derived(), AnalysisFormat::Md).unwrap());
    assert!(
        text.contains("## Totals"),
        "MD should contain Totals section"
    );
    assert!(text.contains("300"), "MD should contain code count");
}

#[test]
fn md_with_derived_contains_integrity_section() {
    let text = extract_text(render(&receipt_with_derived(), AnalysisFormat::Md).unwrap());
    assert!(
        text.contains("## Integrity"),
        "MD should contain Integrity section"
    );
    assert!(text.contains("abc123"), "MD should contain hash");
}
