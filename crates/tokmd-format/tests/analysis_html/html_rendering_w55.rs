//! W55 depth tests for tokmd-format analysis HTML: rendering structure,
//! escaping safety, numeric formatting, data attributes, and determinism.

use tokmd_analysis_types::*;
use tokmd_format::analysis::html::render;

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
        tokens: code * 3,
        doc_pct: Some(0.15),
        bytes_per_line: Some(40.0),
        depth: path.matches('/').count(),
    }
}

fn derived_with_files(files: Vec<FileStatRow>) -> DerivedReport {
    let total_code: usize = files.iter().map(|f| f.code).sum();
    let total_lines: usize = files.iter().map(|f| f.lines).sum();
    let total_tokens: usize = files.iter().map(|f| f.tokens).sum();
    let total_bytes: usize = files.iter().map(|f| f.bytes).sum();

    DerivedReport {
        totals: DerivedTotals {
            files: files.len(),
            code: total_code,
            comments: total_code / 5,
            blanks: total_code / 10,
            lines: total_lines,
            bytes: total_bytes,
            tokens: total_tokens,
        },
        doc_density: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: total_code / 5,
                denominator: total_code.max(1),
                ratio: 0.2,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        whitespace: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: total_code / 10,
                denominator: total_lines.max(1),
                ratio: 0.07,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        verbosity: RateReport {
            total: RateRow {
                key: "total".into(),
                numerator: total_bytes,
                denominator: total_lines.max(1),
                rate: 40.0,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        max_file: MaxFileReport {
            overall: files
                .first()
                .cloned()
                .unwrap_or_else(|| make_file_row("empty", ".", "Text", 0)),
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
            prod_lines: total_code,
            test_files: 0,
            prod_files: files.len(),
            ratio: 0.0,
        },
        boilerplate: BoilerplateReport {
            infra_lines: 0,
            logic_lines: total_code,
            ratio: 0.0,
            infra_langs: vec![],
        },
        polyglot: PolyglotReport {
            lang_count: 1,
            entropy: 0.0,
            dominant_lang: "Rust".into(),
            dominant_lines: total_code,
            dominant_pct: 1.0,
        },
        distribution: DistributionReport {
            count: files.len(),
            min: files.iter().map(|f| f.lines).min().unwrap_or(0),
            max: files.iter().map(|f| f.lines).max().unwrap_or(0),
            mean: if files.is_empty() {
                0.0
            } else {
                total_lines as f64 / files.len() as f64
            },
            median: 0.0,
            p90: 0.0,
            p99: 0.0,
            gini: 0.3,
        },
        histogram: vec![],
        top: TopOffenders {
            largest_lines: files.clone(),
            largest_tokens: vec![],
            largest_bytes: vec![],
            least_documented: vec![],
            most_dense: vec![],
        },
        tree: None,
        reading_time: ReadingTimeReport {
            minutes: total_lines as f64 / 20.0,
            lines_per_minute: 20,
            basis_lines: total_lines,
        },
        context_window: None,
        cocomo: None,
        todo: None,
        integrity: IntegrityReport {
            algo: "blake3".into(),
            hash: "test".into(),
            entries: files.len(),
        },
    }
}

// ── HTML structure: document skeleton ────────────────────────────────

#[test]
fn render_minimal_contains_title_element() {
    let html = render(&minimal_receipt());
    assert!(html.contains("<title>"), "should contain <title>");
    assert!(html.contains("</title>"), "should close </title>");
}

#[test]
fn render_minimal_contains_style_block() {
    let html = render(&minimal_receipt());
    assert!(html.contains("<style>"), "should embed CSS");
    assert!(html.contains("</style>"), "should close style block");
}

#[test]
fn render_minimal_contains_viewport_meta() {
    let html = render(&minimal_receipt());
    assert!(
        html.contains("viewport"),
        "should include viewport meta tag for responsive layout"
    );
}

#[test]
fn render_contains_lang_attribute_on_html_tag() {
    let html = render(&minimal_receipt());
    assert!(
        html.contains(r#"<html lang="en">"#),
        "html tag should have lang attribute"
    );
}

// ── HTML structure: metric cards ────────────────────────────────────

#[test]
fn render_no_metric_cards_without_derived() {
    let html = render(&minimal_receipt());
    assert_eq!(
        html.matches(r#"class="metric-card""#).count(),
        0,
        "no metric cards when derived is None"
    );
}

#[test]
fn render_metric_cards_show_files_label() {
    let mut r = minimal_receipt();
    r.derived = Some(derived_with_files(vec![make_file_row(
        "a.rs", ".", "Rust", 50,
    )]));
    let html = render(&r);
    assert!(html.contains(">Files<"), "Files label should appear");
}

#[test]
fn render_metric_cards_show_lines_label() {
    let mut r = minimal_receipt();
    r.derived = Some(derived_with_files(vec![make_file_row(
        "a.rs", ".", "Rust", 50,
    )]));
    let html = render(&r);
    assert!(html.contains(">Lines<"), "Lines label should appear");
}

#[test]
fn render_metric_cards_show_doc_pct_label() {
    let mut r = minimal_receipt();
    r.derived = Some(derived_with_files(vec![make_file_row(
        "a.rs", ".", "Rust", 50,
    )]));
    let html = render(&r);
    assert!(html.contains(">Doc%<"), "Doc% label should appear");
}

// ── HTML escaping: comprehensive XSS vectors ────────────────────────

#[test]
fn render_escapes_svg_onload_in_path() {
    let mut r = minimal_receipt();
    let files = vec![make_file_row("<svg onload=alert(1)>", ".", "SVG", 10)];
    r.derived = Some(derived_with_files(files));
    let html = render(&r);
    assert!(!html.contains("<svg onload=alert(1)>"));
    assert!(html.contains("&lt;svg onload=alert(1)&gt;"));
}

#[test]
fn render_escapes_ampersand_sequences_in_path() {
    let mut r = minimal_receipt();
    let files = vec![make_file_row("src/a&amp;b.rs", "src", "Rust", 10)];
    r.derived = Some(derived_with_files(files));
    let html = render(&r);
    // The literal & in &amp; should be double-escaped
    assert!(html.contains("a&amp;amp;b.rs"));
}

#[test]
fn render_escapes_all_five_html_special_chars() {
    let mut r = minimal_receipt();
    let files = vec![make_file_row("a<b>c&d\"e'f", "mod<x>", "L&\"ang'", 10)];
    r.derived = Some(derived_with_files(files));
    let html = render(&r);
    // path
    assert!(html.contains("a&lt;b&gt;c&amp;d&quot;e&#x27;f"));
    // module
    assert!(html.contains("mod&lt;x&gt;"));
    // lang
    assert!(html.contains("L&amp;&quot;ang&#x27;"));
}

#[test]
fn render_json_section_never_contains_raw_angle_brackets() {
    let mut r = minimal_receipt();
    let files = vec![
        make_file_row("<script>evil</script>", ".", "JS", 10),
        make_file_row("a>b<c", ".", "Text", 5),
    ];
    r.derived = Some(derived_with_files(files));
    let html = render(&r);

    if let Some(start) = html.find("const REPORT_DATA =") {
        let json_start = start + "const REPORT_DATA =".len();
        if let Some(end) = html[json_start..].find(';') {
            let json_section = &html[json_start..json_start + end];
            assert!(!json_section.contains('<'), "JSON must not contain raw <");
            assert!(!json_section.contains('>'), "JSON must not contain raw >");
        }
    }
}

#[test]
fn render_json_uses_unicode_escapes_for_angle_brackets() {
    let mut r = minimal_receipt();
    let files = vec![make_file_row("<test>", ".", "JS", 10)];
    r.derived = Some(derived_with_files(files));
    let html = render(&r);
    assert!(
        html.contains("\\u003c") && html.contains("\\u003e"),
        "JSON should use \\u003c and \\u003e"
    );
}

// ── Numeric formatting ──────────────────────────────────────────────

#[test]
fn render_small_numbers_have_no_suffix() {
    let mut r = minimal_receipt();
    let mut d = derived_with_files(vec![make_file_row("a.rs", ".", "Rust", 1)]);
    d.totals.code = 42;
    r.derived = Some(d);
    let html = render(&r);
    assert!(
        html.contains(r#"<span class="value">42</span>"#),
        "42 should appear without suffix"
    );
}

#[test]
fn render_exactly_999_has_no_k_suffix() {
    let mut r = minimal_receipt();
    let mut d = derived_with_files(vec![make_file_row("a.rs", ".", "Rust", 1)]);
    d.totals.lines = 999;
    r.derived = Some(d);
    let html = render(&r);
    assert!(html.contains(">999<"), "999 lines should not have K suffix");
}

#[test]
fn render_1500_uses_k_suffix() {
    let mut r = minimal_receipt();
    let mut d = derived_with_files(vec![make_file_row("a.rs", ".", "Rust", 1)]);
    d.totals.tokens = 1500;
    r.derived = Some(d);
    let html = render(&r);
    assert!(html.contains("1.5K"), "1500 should render as 1.5K");
}

#[test]
fn render_millions_use_m_suffix() {
    let mut r = minimal_receipt();
    let mut d = derived_with_files(vec![make_file_row("a.rs", ".", "Rust", 1)]);
    d.totals.code = 3_700_000;
    r.derived = Some(d);
    let html = render(&r);
    assert!(html.contains("3.7M"), "3700000 should render as 3.7M");
}

#[test]
fn render_zero_renders_as_plain_zero() {
    let mut r = minimal_receipt();
    let mut d = derived_with_files(vec![make_file_row("a.rs", ".", "Rust", 1)]);
    d.totals.tokens = 0;
    r.derived = Some(d);
    let html = render(&r);
    // "0" as a value span
    assert!(html.contains(r#"<span class="value">0</span><span class="label">Tokens</span>"#));
}

// ── Table rows: structure and data attributes ───────────────────────

#[test]
fn render_table_row_contains_all_seven_data_attributes() {
    let mut r = minimal_receipt();
    let files = vec![make_file_row("src/app.rs", "src", "Rust", 300)];
    r.derived = Some(derived_with_files(files));
    let html = render(&r);

    for attr in &[
        "data-path=",
        "data-module=",
        "data-lang=",
        "data-lines=",
        "data-code=",
        "data-tokens=",
        "data-bytes=",
    ] {
        assert!(html.contains(attr), "missing attribute {attr}");
    }
}

#[test]
fn render_data_code_holds_raw_numeric_value() {
    let mut r = minimal_receipt();
    let files = vec![make_file_row("x.rs", ".", "Rust", 4200)];
    r.derived = Some(derived_with_files(files));
    let html = render(&r);
    assert!(
        html.contains("data-code=\"4200\""),
        "data-code should hold raw 4200"
    );
}

#[test]
fn render_data_tokens_holds_raw_numeric_value() {
    let mut r = minimal_receipt();
    // code=1000 → tokens=1000*3=3000
    let files = vec![make_file_row("x.rs", ".", "Rust", 1000)];
    r.derived = Some(derived_with_files(files));
    let html = render(&r);
    assert!(
        html.contains("data-tokens=\"3000\""),
        "data-tokens should hold raw 3000"
    );
}

#[test]
fn render_lang_badge_class_present_in_rows() {
    let mut r = minimal_receipt();
    let files = vec![make_file_row("a.py", ".", "Python", 50)];
    r.derived = Some(derived_with_files(files));
    let html = render(&r);
    assert!(
        html.contains("lang-badge"),
        "table rows should include lang-badge class"
    );
}

// ── Table rows: file cap at 100 ─────────────────────────────────────

#[test]
fn render_exactly_100_files_renders_100_rows() {
    let mut r = minimal_receipt();
    let files: Vec<FileStatRow> = (0..100)
        .map(|i| make_file_row(&format!("f{i}.rs"), ".", "Rust", 10))
        .collect();
    r.derived = Some(derived_with_files(files));
    let html = render(&r);
    assert_eq!(html.matches("<tr><td").count(), 100);
}

#[test]
fn render_101_files_renders_only_100_rows() {
    let mut r = minimal_receipt();
    let files: Vec<FileStatRow> = (0..101)
        .map(|i| make_file_row(&format!("f{i}.rs"), ".", "Rust", 10))
        .collect();
    r.derived = Some(derived_with_files(files));
    let html = render(&r);
    assert_eq!(html.matches("<tr><td").count(), 100);
}

#[test]
fn render_zero_files_with_derived_renders_no_rows() {
    let mut r = minimal_receipt();
    r.derived = Some(derived_with_files(vec![]));
    let html = render(&r);
    assert_eq!(html.matches("<tr><td").count(), 0);
}

// ── Context window card ─────────────────────────────────────────────

#[test]
fn render_context_fit_card_shows_percentage() {
    let mut r = minimal_receipt();
    let mut d = derived_with_files(vec![make_file_row("a.rs", ".", "Rust", 100)]);
    d.context_window = Some(ContextWindowReport {
        window_tokens: 100_000,
        total_tokens: 5000,
        pct: 0.05,
        fits: true,
    });
    r.derived = Some(d);
    let html = render(&r);
    assert!(
        html.contains("5.0%"),
        "context fit card should display 5.0%"
    );
}

#[test]
fn render_no_context_fit_card_when_absent() {
    let mut r = minimal_receipt();
    let mut d = derived_with_files(vec![make_file_row("a.rs", ".", "Rust", 100)]);
    d.context_window = None;
    r.derived = Some(d);
    let html = render(&r);
    assert!(
        !html.contains("Context Fit"),
        "no Context Fit card without context_window"
    );
}

// ── Special characters in paths ─────────────────────────────────────

#[test]
fn render_handles_path_with_dots_and_hyphens() {
    let mut r = minimal_receipt();
    let files = vec![make_file_row(
        "my-project/src/lib.v2.rs",
        "my-project/src",
        "Rust",
        50,
    )];
    r.derived = Some(derived_with_files(files));
    let html = render(&r);
    assert!(html.contains("my-project/src/lib.v2.rs"));
}

#[test]
fn render_handles_deeply_nested_path() {
    let mut r = minimal_receipt();
    let files = vec![make_file_row(
        "a/b/c/d/e/f/g/h/i/j.rs",
        "a/b/c/d/e/f/g/h/i",
        "Rust",
        10,
    )];
    r.derived = Some(derived_with_files(files));
    let html = render(&r);
    assert!(html.contains("a/b/c/d/e/f/g/h/i/j.rs"));
}

#[test]
fn render_handles_unicode_in_module_name() {
    let mut r = minimal_receipt();
    let files = vec![make_file_row("src/café.rs", "src/café", "Rust", 10)];
    r.derived = Some(derived_with_files(files));
    let html = render(&r);
    assert!(html.contains("café"), "Unicode module name should appear");
}

// ── REPORT_DATA JSON structure ──────────────────────────────────────

#[test]
fn render_report_json_contains_files_key() {
    let mut r = minimal_receipt();
    r.derived = Some(derived_with_files(vec![make_file_row(
        "a.rs", ".", "Rust", 10,
    )]));
    let html = render(&r);
    assert!(html.contains("\"files\""), "JSON should contain files key");
}

#[test]
fn render_report_json_empty_when_no_derived() {
    let html = render(&minimal_receipt());
    assert!(
        html.contains("{\"files\":[]}"),
        "JSON should be empty files array when no derived"
    );
}

#[test]
fn render_report_json_contains_file_fields() {
    let mut r = minimal_receipt();
    let files = vec![make_file_row("src/lib.rs", "src", "Rust", 100)];
    r.derived = Some(derived_with_files(files));
    let html = render(&r);

    // Extract JSON section
    let start = html.find("const REPORT_DATA =").unwrap() + "const REPORT_DATA =".len();
    let end = html[start..].find(';').unwrap();
    let json_str = html[start..start + end].trim();
    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();

    let file = &parsed["files"][0];
    assert_eq!(file["path"], "src/lib.rs");
    assert_eq!(file["module"], "src");
    assert_eq!(file["lang"], "Rust");
    assert_eq!(file["code"], 100);
}

// ── Determinism ─────────────────────────────────────────────────────

#[test]
fn render_deterministic_excluding_timestamp() {
    let mut r = minimal_receipt();
    r.derived = Some(derived_with_files(vec![
        make_file_row("a.rs", "src", "Rust", 100),
        make_file_row("b.py", "lib", "Python", 50),
    ]));

    let html1 = render(&r);
    let html2 = render(&r);

    // Strip timestamps for comparison
    let strip_ts = |s: &str| -> String {
        if let (Some(start), Some(end)) = (s.find("20"), s.find(" UTC")) {
            return format!("{}{}", &s[..start], &s[end + 4..]);
        }
        s.to_string()
    };
    assert_eq!(strip_ts(&html1), strip_ts(&html2));
}

#[test]
fn render_same_input_produces_same_row_count() {
    let mut r = minimal_receipt();
    let files: Vec<FileStatRow> = (0..25)
        .map(|i| make_file_row(&format!("f{i}.rs"), ".", "Rust", 10 + i))
        .collect();
    r.derived = Some(derived_with_files(files));

    let count1 = render(&r).matches("<tr><td").count();
    let count2 = render(&r).matches("<tr><td").count();
    assert_eq!(count1, count2);
    assert_eq!(count1, 25);
}

// ── CSS classes present in template ─────────────────────────────────

#[test]
fn render_template_contains_container_class() {
    let html = render(&minimal_receipt());
    assert!(
        html.contains("container"),
        "template should use container class"
    );
}

#[test]
fn render_template_contains_css_variables() {
    let html = render(&minimal_receipt());
    assert!(
        html.contains("--bg-primary"),
        "template should define CSS custom properties"
    );
}
