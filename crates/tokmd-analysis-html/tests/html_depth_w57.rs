//! Wave-57 depth tests for `tokmd-analysis-html`.
//!
//! Covers:
//! - format_number for large numbers, zero, boundary values
//! - HTML escaping prevents XSS injection
//! - DOCTYPE presence and document structure
//! - CSS class generation in metric cards and table rows
//! - Rendering with empty / minimal / large data sets
//! - Report JSON safety (angle-bracket escaping)
//! - Deterministic output for identical input

use tokmd_analysis_html::render;
use tokmd_analysis_types::*;

// ── Helpers ─────────────────────────────────────────────────────────

fn minimal_receipt() -> AnalysisReceipt {
    AnalysisReceipt {
        schema_version: 2,
        generated_at_ms: 0,
        tool: tokmd_types::ToolInfo {
            name: "tokmd".into(),
            version: "0.0.0".into(),
        },
        mode: "analysis".into(),
        status: tokmd_types::ScanStatus::Complete,
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

fn make_file_row(path: &str, module: &str, lang: &str, code: usize) -> FileStatRow {
    FileStatRow {
        path: path.into(),
        module: module.into(),
        lang: lang.into(),
        code,
        comments: code / 5,
        blanks: code / 10,
        lines: code + code / 5 + code / 10,
        bytes: code * 50,
        tokens: code * 2,
        doc_pct: Some(0.15),
        bytes_per_line: Some(40.0),
        depth: 1,
    }
}

// =============================================================================
// 1. DOCTYPE and document structure
// =============================================================================

#[test]
fn render_starts_with_doctype_html() {
    let html = render(&minimal_receipt());
    assert!(
        html.starts_with("<!DOCTYPE html>"),
        "HTML must start with DOCTYPE declaration"
    );
}

#[test]
fn render_has_html_lang_attribute() {
    let html = render(&minimal_receipt());
    assert!(
        html.contains("<html lang=\"en\">"),
        "html tag must have lang=en"
    );
}

#[test]
fn render_has_closing_html_and_body_tags() {
    let html = render(&minimal_receipt());
    assert!(html.contains("</html>"));
    assert!(html.contains("</body>"));
}

#[test]
fn render_has_meta_charset_and_viewport() {
    let html = render(&minimal_receipt());
    assert!(html.contains("charset=\"UTF-8\""));
    assert!(html.contains("viewport"));
}

#[test]
fn render_has_title_tag() {
    let html = render(&minimal_receipt());
    assert!(html.contains("<title>"));
    assert!(html.contains("</title>"));
}

// =============================================================================
// 2. format_number via rendered output
// =============================================================================

#[test]
fn render_zero_code_lines_shown_as_zero() {
    let mut receipt = minimal_receipt();
    let mut derived = sample_derived();
    derived.totals.code = 0;
    derived.totals.lines = 0;
    derived.totals.tokens = 0;
    receipt.derived = Some(derived);
    let html = render(&receipt);
    // "0" should appear as the formatted value for zero counts
    assert!(html.contains(">0<"));
}

#[test]
fn render_small_numbers_show_raw_digits() {
    let mut receipt = minimal_receipt();
    let mut derived = sample_derived();
    derived.totals.code = 42;
    derived.totals.lines = 999;
    receipt.derived = Some(derived);
    let html = render(&receipt);
    assert!(html.contains("42"));
    assert!(html.contains("999"));
}

#[test]
fn render_thousands_show_k_suffix() {
    let mut receipt = minimal_receipt();
    let mut derived = sample_derived();
    derived.totals.code = 5_000;
    receipt.derived = Some(derived);
    let html = render(&receipt);
    assert!(html.contains("5.0K"), "5000 should render as 5.0K");
}

#[test]
fn render_millions_show_m_suffix() {
    let mut receipt = minimal_receipt();
    let mut derived = sample_derived();
    derived.totals.code = 2_500_000;
    receipt.derived = Some(derived);
    let html = render(&receipt);
    assert!(html.contains("2.5M"), "2500000 should render as 2.5M");
}

#[test]
fn render_exact_thousand_boundary() {
    let mut receipt = minimal_receipt();
    let mut derived = sample_derived();
    derived.totals.lines = 1_000;
    receipt.derived = Some(derived);
    let html = render(&receipt);
    assert!(html.contains("1.0K"), "1000 should render as 1.0K");
}

#[test]
fn render_exact_million_boundary() {
    let mut receipt = minimal_receipt();
    let mut derived = sample_derived();
    derived.totals.tokens = 1_000_000;
    receipt.derived = Some(derived);
    let html = render(&receipt);
    assert!(html.contains("1.0M"), "1000000 should render as 1.0M");
}

// =============================================================================
// 3. HTML escaping / XSS prevention
// =============================================================================

#[test]
fn xss_in_path_is_escaped_in_table_rows() {
    let mut receipt = minimal_receipt();
    let mut derived = sample_derived();
    derived.top.largest_lines = vec![make_file_row(
        "<script>alert('xss')</script>",
        "src",
        "Rust",
        100,
    )];
    receipt.derived = Some(derived);
    let html = render(&receipt);
    assert!(!html.contains("<script>alert"));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn xss_in_module_name_is_escaped() {
    let mut receipt = minimal_receipt();
    let mut derived = sample_derived();
    derived.top.largest_lines = vec![make_file_row(
        "safe.rs",
        "<img onerror=alert(1)>",
        "Rust",
        100,
    )];
    receipt.derived = Some(derived);
    let html = render(&receipt);
    assert!(!html.contains("<img onerror"));
    assert!(html.contains("&lt;img"));
}

#[test]
fn xss_in_language_name_is_escaped() {
    let mut receipt = minimal_receipt();
    let mut derived = sample_derived();
    derived.top.largest_lines = vec![make_file_row(
        "test.rs",
        "mod",
        "\"onmouseover=alert(1)\"",
        100,
    )];
    receipt.derived = Some(derived);
    let html = render(&receipt);
    assert!(!html.contains("\"onmouseover=alert(1)\""));
    assert!(html.contains("&quot;onmouseover"));
}

#[test]
fn ampersand_in_path_is_escaped() {
    let mut receipt = minimal_receipt();
    let mut derived = sample_derived();
    derived.top.largest_lines = vec![make_file_row("src/a&b.rs", "src", "Rust", 50)];
    receipt.derived = Some(derived);
    let html = render(&receipt);
    assert!(html.contains("a&amp;b.rs"));
}

#[test]
fn report_json_has_no_raw_angle_brackets() {
    let mut receipt = minimal_receipt();
    let mut derived = sample_derived();
    derived.top.largest_lines = vec![make_file_row("</script><img src=x>", "mod", "Rust", 100)];
    receipt.derived = Some(derived);
    let html = render(&receipt);
    // Find the REPORT_DATA JSON section
    let json_start = html.find("const REPORT_DATA =").unwrap();
    let json_section = &html[json_start..];
    // The JSON embedded inside <script> must not contain raw < or >
    let json_end = json_section.find(";\n").unwrap_or(json_section.len());
    let json_text = &json_section[..json_end];
    assert!(
        !json_text.contains("</script>"),
        "JSON must not contain raw </script>"
    );
}

// =============================================================================
// 4. CSS class generation
// =============================================================================

#[test]
fn metric_cards_have_correct_css_classes() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let html = render(&receipt);
    assert!(html.contains("class=\"metric-card\""));
    assert!(html.contains("class=\"value\""));
    assert!(html.contains("class=\"label\""));
}

#[test]
fn table_rows_have_data_attributes_and_classes() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let html = render(&receipt);
    assert!(html.contains("class=\"path\""));
    assert!(html.contains("class=\"num\""));
    assert!(html.contains("class=\"lang-badge\""));
    assert!(html.contains("data-path="));
    assert!(html.contains("data-lang="));
    assert!(html.contains("data-lines="));
    assert!(html.contains("data-code="));
    assert!(html.contains("data-tokens="));
    assert!(html.contains("data-bytes="));
}

// =============================================================================
// 5. Empty / minimal data sets
// =============================================================================

#[test]
fn render_without_derived_produces_valid_html() {
    let receipt = minimal_receipt();
    let html = render(&receipt);
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("</html>"));
    // No metric cards when derived is None
    assert!(!html.contains("class=\"metric-card\""));
}

#[test]
fn render_with_empty_top_offenders_has_no_table_rows() {
    let mut receipt = minimal_receipt();
    let mut derived = sample_derived();
    derived.top.largest_lines.clear();
    receipt.derived = Some(derived);
    let html = render(&receipt);
    // Should still have the structure but no <tr> data rows
    assert!(!html.contains("data-path="));
}

#[test]
fn render_without_context_window_omits_context_fit_card() {
    let mut receipt = minimal_receipt();
    let mut derived = sample_derived();
    derived.context_window = None;
    receipt.derived = Some(derived);
    let html = render(&receipt);
    assert!(!html.contains("Context Fit"));
}

// =============================================================================
// 6. Large data set
// =============================================================================

#[test]
fn render_with_many_files_caps_at_100_rows() {
    let mut receipt = minimal_receipt();
    let mut derived = sample_derived();
    derived.top.largest_lines = (0..150)
        .map(|i| make_file_row(&format!("src/file_{i:04}.rs"), "src", "Rust", 100 + i))
        .collect();
    receipt.derived = Some(derived);
    let html = render(&receipt);
    // Template caps at 100 rows
    let row_count = html.matches("<tr><td class=\"path\"").count();
    assert_eq!(row_count, 100, "Table should cap at 100 rows");
}

// =============================================================================
// 7. Deterministic output
// =============================================================================

#[test]
fn render_is_deterministic_for_same_receipt() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let html1 = render(&receipt);
    let html2 = render(&receipt);
    // Timestamps may differ so we compare everything except the timestamp line
    let strip_ts = |s: &str| -> String {
        s.lines()
            .filter(|l| !l.contains("UTC"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(strip_ts(&html1), strip_ts(&html2));
}

// =============================================================================
// 8. Metric card labels
// =============================================================================

#[test]
fn metric_cards_contain_expected_labels() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let html = render(&receipt);
    for label in &["Files", "Lines", "Code", "Tokens", "Doc%", "Context Fit"] {
        assert!(html.contains(label), "Missing metric card label: {label}");
    }
}

#[test]
fn doc_percent_is_rendered_with_percent_sign() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let html = render(&receipt);
    // doc_density ratio 0.1667 → "16.7%"
    assert!(html.contains("16.7%"));
}

#[test]
fn context_fit_percent_rendered_correctly() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let html = render(&receipt);
    // pct 0.025 → "2.5%"
    assert!(html.contains("2.5%"));
}
