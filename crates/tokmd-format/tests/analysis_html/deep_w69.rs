//! Deep tests for tokmd-format analysis HTML rendering (W69).
//!
//! Covers: HTML output generation, escaping, structure validation,
//! metric cards, table rows, report JSON, determinism.

use tokmd_analysis_types::*;
use tokmd_format::analysis::html::render;
use tokmd_types::{ScanStatus, ToolInfo};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn minimal_receipt() -> AnalysisReceipt {
    AnalysisReceipt {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: ToolInfo {
            name: "tokmd".into(),
            version: "0.0.0".into(),
        },
        mode: "analysis".into(),
        status: ScanStatus::Complete,
        warnings: vec![],
        source: AnalysisSource {
            inputs: vec!["test".into()],
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
            format: "html".into(),
            window_tokens: None,
            git: None,
            max_files: None,
            max_bytes: None,
            max_commits: None,
            max_commit_files: None,
            max_file_bytes: None,
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
        effort: None,
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
            by_lang: vec![],
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
        histogram: vec![],
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
        tree: None,
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
        cocomo: None,
        todo: None,
        integrity: IntegrityReport {
            algo: "blake3".into(),
            hash: "abc123".into(),
            entries: 10,
        },
    }
}

// ===================================================================
// 1. HTML structure
// ===================================================================

#[test]
fn html_starts_with_doctype() {
    let receipt = minimal_receipt();
    let html = render(&receipt);
    assert!(html.starts_with("<!DOCTYPE html>"));
}

#[test]
fn html_contains_closing_tags() {
    let receipt = minimal_receipt();
    let html = render(&receipt);
    assert!(html.contains("</html>"));
    assert!(html.contains("</head>"));
    assert!(html.contains("</body>"));
}

#[test]
fn html_contains_timestamp() {
    let receipt = minimal_receipt();
    let html = render(&receipt);
    assert!(html.contains("UTC"));
}

// ===================================================================
// 2. Metric cards
// ===================================================================

#[test]
fn html_no_metric_cards_without_derived() {
    let receipt = minimal_receipt();
    let html = render(&receipt);
    assert!(!html.contains("class=\"metric-card\""));
}

#[test]
fn html_metric_cards_with_derived() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let html = render(&receipt);
    assert!(html.contains("class=\"metric-card\""));
    assert!(html.contains("Files"));
    assert!(html.contains("Code"));
    assert!(html.contains("Tokens"));
}

#[test]
fn html_context_fit_card_present() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let html = render(&receipt);
    assert!(html.contains("Context Fit"));
}

// ===================================================================
// 3. Table rows
// ===================================================================

#[test]
fn html_table_rows_present_with_derived() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let html = render(&receipt);
    assert!(html.contains("src/lib.rs"));
    assert!(html.contains("<tr>"));
}

#[test]
fn html_no_table_rows_without_derived() {
    let receipt = minimal_receipt();
    let html = render(&receipt);
    assert!(!html.contains("data-path="));
}

// ===================================================================
// 4. HTML escaping of special characters
// ===================================================================

#[test]
fn html_escapes_angle_brackets_in_paths() {
    let mut receipt = minimal_receipt();
    let mut derived = sample_derived();
    derived.top.largest_lines[0].path = "<script>alert('xss')</script>".into();
    receipt.derived = Some(derived);
    let html = render(&receipt);
    assert!(html.contains("&lt;script&gt;"));
    assert!(!html.contains("<script>alert"));
}

#[test]
fn html_escapes_ampersands() {
    let mut receipt = minimal_receipt();
    let mut derived = sample_derived();
    derived.top.largest_lines[0].module = "mod&test".into();
    receipt.derived = Some(derived);
    let html = render(&receipt);
    assert!(html.contains("mod&amp;test"));
}

#[test]
fn html_escapes_quotes_in_lang() {
    let mut receipt = minimal_receipt();
    let mut derived = sample_derived();
    derived.top.largest_lines[0].lang = "Ru\"st".into();
    receipt.derived = Some(derived);
    let html = render(&receipt);
    assert!(html.contains("Ru&quot;st"));
}

// ===================================================================
// 5. Report JSON (XSS prevention)
// ===================================================================

#[test]
fn html_report_json_no_raw_angle_brackets() {
    let mut receipt = minimal_receipt();
    let mut derived = sample_derived();
    derived.top.largest_lines[0].path = "</script><img onerror=alert(1)>".into();
    receipt.derived = Some(derived);
    let html = render(&receipt);
    let json_start = html.find("const REPORT_DATA =");
    assert!(json_start.is_some());
    let json_section = &html[json_start.unwrap()..];
    let json_end = json_section.find(';').unwrap();
    let json_fragment = &json_section[..json_end];
    assert!(!json_fragment.contains("</script>"));
}

// ===================================================================
// 6. Determinism
// ===================================================================

#[test]
fn html_rendering_is_deterministic_ignoring_timestamp() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let a = render(&receipt);
    let b = render(&receipt);
    // Timestamp varies, so strip it
    let strip_ts = |s: String| {
        s.lines()
            .filter(|l| !l.contains("UTC"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(strip_ts(a), strip_ts(b));
}

#[test]
fn html_empty_derived_produces_no_rows_deterministically() {
    let mut receipt = minimal_receipt();
    let mut derived = sample_derived();
    derived.top.largest_lines.clear();
    receipt.derived = Some(derived);
    let a = render(&receipt);
    let b = render(&receipt);
    let strip_ts = |s: String| {
        s.lines()
            .filter(|l| !l.contains("UTC"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(strip_ts(a), strip_ts(b));
}
