//! # tokmd-analysis-format
//!
//! **Tier 3 (Formatting)**
//!
//! Rendering for analysis receipts. Supports multiple output formats including
//! Markdown, JSON, JSON-LD, XML, SVG, Mermaid, and optional fun outputs.
//!
//! ## Effort rendering
//!
//! Effort sections are rendered in two tiers:
//!
//! 1. `receipt.effort` the preferred path for the newer effort-estimation
//!    receipt surface. This can render size basis, confidence, drivers,
//!    assumptions, and optional delta data.
//! 2. `derived.cocomo` a legacy fallback used when the richer `effort`
//!    section is absent but classic derived COCOMO data is present.
//!
//! The formatter intentionally renders whatever the receipt contains without
//! inferring missing estimate data. If the upstream effort builder is still
//! scaffold-only, the formatter preserves that truth rather than making the
//! estimate look more complete than it is.
//!
//! ## What belongs here
//! * Analysis receipt rendering to various formats
//! * Format-specific transformations
//! * Fun output integration (OBJ, MIDI when enabled)
//!
//! ## What does NOT belong here
//! * Analysis computation (use tokmd-analysis)
//! * CLI argument parsing
//! * Base receipt formatting (use tokmd-format)
//!
//! ## Architecture
//!
//! This crate now delegates Markdown formatting to `tokmd-analysis-format-md`
//! microcrate, following the Single Responsibility Principle (SRP).

use anyhow::Result;
use std::fmt::Write;
use tokmd_analysis_types::AnalysisReceipt;
use tokmd_types::AnalysisFormat;

// Re-export the Markdown formatter from the microcrate
pub use tokmd_analysis_format_md::render_md;

pub enum RenderedOutput {
    Text(String),
    Binary(Vec<u8>),
}

pub fn render(receipt: &AnalysisReceipt, format: AnalysisFormat) -> Result<RenderedOutput> {
    match format {
        AnalysisFormat::Md => Ok(RenderedOutput::Text(render_md(receipt))),
        AnalysisFormat::Json => Ok(RenderedOutput::Text(serde_json::to_string_pretty(receipt)?)),
        AnalysisFormat::Jsonld => Ok(RenderedOutput::Text(render_jsonld(receipt))),
        AnalysisFormat::Xml => Ok(RenderedOutput::Text(render_xml(receipt))),
        AnalysisFormat::Svg => Ok(RenderedOutput::Text(render_svg(receipt))),
        AnalysisFormat::Mermaid => Ok(RenderedOutput::Text(render_mermaid(receipt))),
        AnalysisFormat::Obj => Ok(RenderedOutput::Text(render_obj(receipt)?)),
        AnalysisFormat::Midi => Ok(RenderedOutput::Binary(render_midi(receipt)?)),
        AnalysisFormat::Tree => Ok(RenderedOutput::Text(render_tree(receipt))),
        AnalysisFormat::Html => Ok(RenderedOutput::Text(render_html(receipt))),
    }
}

fn render_jsonld(receipt: &AnalysisReceipt) -> String {
    let name = receipt
        .source
        .inputs
        .first()
        .cloned()
        .unwrap_or_else(|| "tokmd".to_string());
    let totals = receipt.derived.as_ref().map(|d| &d.totals);
    let payload = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "SoftwareSourceCode",
        "name": name,
        "codeLines": totals.map(|t| t.code).unwrap_or(0),
        "commentCount": totals.map(|t| t.comments).unwrap_or(0),
        "lineCount": totals.map(|t| t.lines).unwrap_or(0),
        "fileSize": totals.map(|t| t.bytes).unwrap_or(0),
        "interactionStatistic": {
            "@type": "InteractionCounter",
            "interactionType": "http://schema.org/ReadAction",
            "userInteractionCount": totals.map(|t| t.tokens).unwrap_or(0)
        }
    });
    serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
}

fn render_xml(receipt: &AnalysisReceipt) -> String {
    let totals = receipt.derived.as_ref().map(|d| &d.totals);
    let mut out = String::new();
    out.push_str("<analysis>");
    if let Some(totals) = totals {
        let _ = write!(
            out,
            "<totals files=\"{}\" code=\"{}\" comments=\"{}\" blanks=\"{}\" lines=\"{}\" bytes=\"{}\" tokens=\"{}\"/>",
            totals.files,
            totals.code,
            totals.comments,
            totals.blanks,
            totals.lines,
            totals.bytes,
            totals.tokens
        );
    }
    out.push_str("</analysis>");
    out
}

fn render_svg(receipt: &AnalysisReceipt) -> String {
    let (label, value) = if let Some(derived) = &receipt.derived {
        if let Some(ctx) = &derived.context_window {
            ("context".to_string(), format!("{:.1}%", ctx.pct * 100.0))
        } else {
            ("tokens".to_string(), derived.totals.tokens.to_string())
        }
    } else {
        ("tokens".to_string(), "0".to_string())
    };

    let width = 240;
    let height = 32;
    let label_width = 80;
    let value_width = width - label_width;
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" role=\"img\"><rect width=\"{label_width}\" height=\"{height}\" fill=\"#555\"/><rect x=\"{label_width}\" width=\"{value_width}\" height=\"{height}\" fill=\"#4c9aff\"/><text x=\"{lx}\" y=\"{ty}\" fill=\"#fff\" font-family=\"Verdana\" font-size=\"12\" text-anchor=\"middle\">{label}</text><text x=\"{vx}\" y=\"{ty}\" fill=\"#fff\" font-family=\"Verdana\" font-size=\"12\" text-anchor=\"middle\">{value}</text></svg>",
        width = width,
        height = height,
        label_width = label_width,
        value_width = value_width,
        lx = label_width / 2,
        vx = label_width + value_width / 2,
        ty = 20,
        label = label,
        value = value
    )
}

fn render_mermaid(receipt: &AnalysisReceipt) -> String {
    let mut out = String::from("graph TD\n");
    if let Some(imports) = &receipt.imports {
        for edge in imports.edges.iter().take(200) {
            let from = sanitize_mermaid(&edge.from);
            let to = sanitize_mermaid(&edge.to);
            let _ = writeln!(out, "  {} -->|{}| {}", from, edge.count, to);
        }
    }
    out
}

fn render_tree(receipt: &AnalysisReceipt) -> String {
    receipt
        .derived
        .as_ref()
        .and_then(|d| d.tree.clone())
        .unwrap_or_else(|| "(tree unavailable)".to_string())
}

// --- fun enabled impls ---
#[cfg(feature = "fun")]
fn render_obj_fun(receipt: &AnalysisReceipt) -> Result<String> {
    if let Some(derived) = &receipt.derived {
        let buildings: Vec<tokmd_fun::ObjBuilding> = derived
            .top
            .largest_lines
            .iter()
            .enumerate()
            .map(|(idx, row)| {
                let x = (idx % 5) as f32 * 2.0;
                let y = (idx / 5) as f32 * 2.0;
                let h = (row.lines as f32 / 10.0).max(0.5);
                tokmd_fun::ObjBuilding {
                    name: row.path.clone(),
                    x,
                    y,
                    w: 1.5,
                    d: 1.5,
                    h,
                }
            })
            .collect();
        return Ok(tokmd_fun::render_obj(&buildings));
    }
    Ok("# tokmd code city\n".to_string())
}

#[cfg(feature = "fun")]
fn render_midi_fun(receipt: &AnalysisReceipt) -> Result<Vec<u8>> {
    let mut notes = Vec::new();
    if let Some(derived) = &receipt.derived {
        for (idx, row) in derived.top.largest_lines.iter().enumerate() {
            let key = 60u8 + (row.depth as u8 % 12);
            let velocity = (40 + (row.lines.min(127) as u8 / 2)).min(120);
            let start = (idx as u32) * 240;
            notes.push(tokmd_fun::MidiNote {
                key,
                velocity,
                start,
                duration: 180,
                channel: 0,
            });
        }
    }
    tokmd_fun::render_midi(&notes, 120)
}

// --- fun disabled impls (errors) ---
#[cfg(not(feature = "fun"))]
fn render_obj_disabled(_receipt: &AnalysisReceipt) -> Result<String> {
    anyhow::bail!(
        "OBJ format requires the `fun` feature: tokmd-analysis-format = {{ version = \"1.3\", features = [\"fun\"] }}"
    )
}

#[cfg(not(feature = "fun"))]
fn render_midi_disabled(_receipt: &AnalysisReceipt) -> Result<Vec<u8>> {
    anyhow::bail!(
        "MIDI format requires the `fun` feature: tokmd-analysis-format = {{ version = \"1.3\", features = [\"fun\"] }}"
    )
}

// --- stable API names used by the rest of the code ---
fn render_obj(receipt: &AnalysisReceipt) -> Result<String> {
    #[cfg(feature = "fun")]
    {
        render_obj_fun(receipt)
    }
    #[cfg(not(feature = "fun"))]
    {
        render_obj_disabled(receipt)
    }
}

fn render_midi(receipt: &AnalysisReceipt) -> Result<Vec<u8>> {
    #[cfg(feature = "fun")]
    {
        render_midi_fun(receipt)
    }
    #[cfg(not(feature = "fun"))]
    {
        render_midi_disabled(receipt)
    }
}

fn sanitize_mermaid(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

fn render_html(receipt: &AnalysisReceipt) -> String {
    tokmd_analysis_html::render(receipt)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use tokmd_analysis_types::*;

    fn minimal_receipt() -> AnalysisReceipt {
        AnalysisReceipt {
            schema_version: 2,
            generated_at_ms: 0,
            tool: tokmd_types::ToolInfo {
                name: "tokmd".to_string(),
                version: "0.0.0".to_string(),
            },
            mode: "analysis".to_string(),
            status: tokmd_types::ScanStatus::Complete,
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

    // Test sanitize_mermaid
    #[test]
    fn test_sanitize_mermaid() {
        assert_eq!(sanitize_mermaid("hello"), "hello");
        assert_eq!(sanitize_mermaid("hello-world"), "hello_world");
        assert_eq!(sanitize_mermaid("src/lib.rs"), "src_lib_rs");
        assert_eq!(sanitize_mermaid("test123"), "test123");
        assert_eq!(sanitize_mermaid("a b c"), "a_b_c");
    }

    // Test render_xml
    #[test]
    fn test_render_xml() {
        let mut receipt = minimal_receipt();
        receipt.derived = Some(sample_derived());
        let result = render_xml(&receipt);
        assert!(result.starts_with("<analysis>"));
        assert!(result.ends_with("</analysis>"));
        assert!(result.contains("files=\"10\""));
        assert!(result.contains("code=\"1000\""));
    }

    // Test render_xml without derived
    #[test]
    fn test_render_xml_no_derived() {
        let receipt = minimal_receipt();
        let result = render_xml(&receipt);
        assert_eq!(result, "<analysis></analysis>");
    }

    // Test render_jsonld
    #[test]
    fn test_render_jsonld() {
        let mut receipt = minimal_receipt();
        receipt.derived = Some(sample_derived());
        let result = render_jsonld(&receipt);
        assert!(result.contains("\"@context\": \"https://schema.org\""));
        assert!(result.contains("\"@type\": \"SoftwareSourceCode\""));
        assert!(result.contains("\"name\": \"test\""));
        assert!(result.contains("\"codeLines\": 1000"));
    }

    // Test render_jsonld without inputs
    #[test]
    fn test_render_jsonld_empty_inputs() {
        let mut receipt = minimal_receipt();
        receipt.source.inputs.clear();
        let result = render_jsonld(&receipt);
        assert!(result.contains("\"name\": \"tokmd\""));
    }

    // Test render_svg
    #[test]
    fn test_render_svg() {
        let mut receipt = minimal_receipt();
        receipt.derived = Some(sample_derived());
        let result = render_svg(&receipt);
        assert!(result.contains("<svg"));
        assert!(result.contains("</svg>"));
        assert!(result.contains("context")); // has context_window
        assert!(result.contains("2.5%")); // pct value
    }

    // Test render_svg without context_window
    #[test]
    fn test_render_svg_no_context() {
        let mut receipt = minimal_receipt();
        let mut derived = sample_derived();
        derived.context_window = None;
        receipt.derived = Some(derived);
        let result = render_svg(&receipt);
        assert!(result.contains("tokens"));
        assert!(result.contains("2500")); // total tokens
    }

    // Test render_svg without derived
    #[test]
    fn test_render_svg_no_derived() {
        let receipt = minimal_receipt();
        let result = render_svg(&receipt);
        assert!(result.contains("tokens"));
        assert!(result.contains(">0<")); // default 0 value
    }

    // Test render_mermaid
    #[test]
    fn test_render_mermaid() {
        let mut receipt = minimal_receipt();
        receipt.imports = Some(ImportReport {
            granularity: "module".to_string(),
            edges: vec![ImportEdge {
                from: "src/main".to_string(),
                to: "src/lib".to_string(),
                count: 5,
            }],
        });
        let result = render_mermaid(&receipt);
        assert!(result.starts_with("graph TD\n"));
        assert!(result.contains("src_main -->|5| src_lib"));
    }

    // Test render_mermaid no imports
    #[test]
    fn test_render_mermaid_no_imports() {
        let receipt = minimal_receipt();
        let result = render_mermaid(&receipt);
        assert_eq!(result, "graph TD\n");
    }

    // Test render_tree
    #[test]
    fn test_render_tree() {
        let mut receipt = minimal_receipt();
        receipt.derived = Some(sample_derived());
        let result = render_tree(&receipt);
        assert_eq!(result, "test-tree");
    }

    // Test render_tree without derived
    #[test]
    fn test_render_tree_no_derived() {
        let receipt = minimal_receipt();
        let result = render_tree(&receipt);
        assert_eq!(result, "(tree unavailable)");
    }

    // Test render_tree with no tree in derived
    #[test]
    fn test_render_tree_none() {
        let mut receipt = minimal_receipt();
        let mut derived = sample_derived();
        derived.tree = None;
        receipt.derived = Some(derived);
        let result = render_tree(&receipt);
        assert_eq!(result, "(tree unavailable)");
    }

    // Test render_obj (non-fun feature) returns error
    #[cfg(not(feature = "fun"))]
    #[test]
    fn test_render_obj_no_fun() {
        let receipt = minimal_receipt();
        let result = render_obj(&receipt);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("fun"));
    }

    // Test render_midi (non-fun feature) returns error
    #[cfg(not(feature = "fun"))]
    #[test]
    fn test_render_midi_no_fun() {
        let receipt = minimal_receipt();
        let result = render_midi(&receipt);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("fun"));
    }

    // Test render_md basic structure (delegated to microcrate)
    #[test]
    fn test_render_md_basic() {
        let receipt = minimal_receipt();
        let result = render_md(&receipt);
        assert!(result.starts_with("# tokmd analysis\n"));
        assert!(result.contains("Preset: `receipt`"));
    }

    // Test render function dispatch
    #[test]
    fn test_render_dispatch_md() {
        let receipt = minimal_receipt();
        let result = render(&receipt, AnalysisFormat::Md).unwrap();
        match result {
            RenderedOutput::Text(s) => assert!(s.starts_with("# tokmd analysis")),
            RenderedOutput::Binary(_) => panic!("expected text"),
        }
    }

    #[test]
    fn test_render_dispatch_json() {
        let receipt = minimal_receipt();
        let result = render(&receipt, AnalysisFormat::Json).unwrap();
        match result {
            RenderedOutput::Text(s) => assert!(s.contains("\"schema_version\": 2")),
            RenderedOutput::Binary(_) => panic!("expected text"),
        }
    }

    #[test]
    fn test_render_dispatch_xml() {
        let receipt = minimal_receipt();
        let result = render(&receipt, AnalysisFormat::Xml).unwrap();
        match result {
            RenderedOutput::Text(s) => assert!(s.contains("<analysis>")),
            RenderedOutput::Binary(_) => panic!("expected text"),
        }
    }

    #[test]
    fn test_render_dispatch_tree() {
        let receipt = minimal_receipt();
        let result = render(&receipt, AnalysisFormat::Tree).unwrap();
        match result {
            RenderedOutput::Text(s) => assert!(s.contains("(tree unavailable)")),
            RenderedOutput::Binary(_) => panic!("expected text"),
        }
    }

    #[test]
    fn test_render_dispatch_svg() {
        let receipt = minimal_receipt();
        let result = render(&receipt, AnalysisFormat::Svg).unwrap();
        match result {
            RenderedOutput::Text(s) => assert!(s.contains("<svg")),
            RenderedOutput::Binary(_) => panic!("expected text"),
        }
    }

    #[test]
    fn test_render_dispatch_mermaid() {
        let receipt = minimal_receipt();
        let result = render(&receipt, AnalysisFormat::Mermaid).unwrap();
        match result {
            RenderedOutput::Text(s) => assert!(s.starts_with("graph TD")),
            RenderedOutput::Binary(_) => panic!("expected text"),
        }
    }

    #[test]
    fn test_render_dispatch_jsonld() {
        let receipt = minimal_receipt();
        let result = render(&receipt, AnalysisFormat::Jsonld).unwrap();
        match result {
            RenderedOutput::Text(s) => assert!(s.contains("@context")),
            RenderedOutput::Binary(_) => panic!("expected text"),
        }
    }

    // Test render_html
    #[test]
    fn test_render_html() {
        let mut receipt = minimal_receipt();
        receipt.derived = Some(sample_derived());
        let result = render_html(&receipt);
        // Should delegate to tokmd-analysis-html
        assert!(!result.is_empty());
    }
}
