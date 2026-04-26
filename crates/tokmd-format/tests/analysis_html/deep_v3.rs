//! Deep tests for tokmd-format analysis HTML: render edge cases, escaping, structure.

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
                denominator: total_code,
                ratio: 0.2,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        whitespace: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: total_code / 10,
                denominator: total_lines,
                ratio: 0.07,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        verbosity: RateReport {
            total: RateRow {
                key: "total".into(),
                numerator: total_bytes,
                denominator: total_lines,
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

// ── Render: HTML structure invariants ────────────────────────────────

#[test]
fn render_produces_valid_doctype() {
    let html = render(&minimal_receipt());
    assert!(html.starts_with("<!DOCTYPE html>"));
}

#[test]
fn render_contains_charset_meta() {
    let html = render(&minimal_receipt());
    assert!(
        html.contains("charset") || html.contains("UTF-8"),
        "HTML should declare character encoding"
    );
}

#[test]
fn render_has_balanced_html_tags() {
    let html = render(&minimal_receipt());
    assert!(html.contains("<html"));
    assert!(html.contains("</html>"));
    assert!(html.contains("<head>"));
    assert!(html.contains("</head>"));
    assert!(html.contains("<body>"));
    assert!(html.contains("</body>"));
}

#[test]
fn render_contains_script_with_report_data() {
    let html = render(&minimal_receipt());
    assert!(
        html.contains("REPORT_DATA"),
        "must embed report data for treemap"
    );
}

// ── Render: XSS prevention vectors ──────────────────────────────────

#[test]
fn render_escapes_double_quote_in_path() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row(
        "\" onmouseover=\"alert(1)\"",
        "evil",
        "JS",
        10,
    )];
    receipt.derived = Some(derived_with_files(files));
    let html = render(&receipt);

    // The double-quote chars should be escaped as &quot;
    assert!(
        html.contains("&quot;"),
        "double quotes must be escaped in HTML attributes"
    );
}

#[test]
fn render_escapes_ampersand_in_module() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row("test.rs", "a&b<c>d", "Rust", 10)];
    receipt.derived = Some(derived_with_files(files));
    let html = render(&receipt);

    assert!(html.contains("a&amp;b&lt;c&gt;d"));
    assert!(!html.contains("a&b<c>d"));
}

#[test]
fn render_escapes_single_quotes_in_lang() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row("test.rs", "src", "Ru'st", 10)];
    receipt.derived = Some(derived_with_files(files));
    let html = render(&receipt);

    assert!(html.contains("Ru&#x27;st"));
}

#[test]
fn render_json_escapes_all_angle_brackets() {
    let mut receipt = minimal_receipt();
    let files = vec![
        make_file_row("<script>", ".", "JS", 10),
        make_file_row("a>b", ".", "JS", 5),
    ];
    receipt.derived = Some(derived_with_files(files));
    let html = render(&receipt);

    // In the REPORT_DATA JSON section, no raw < or > should appear
    let json_start = html.find("REPORT_DATA").unwrap();
    let json_section = &html[json_start..];
    let json_end = json_section.find(';').unwrap_or(json_section.len());
    let json_data = &json_section[..json_end];

    // The JSON itself uses \u003c and \u003e
    assert!(
        !json_data.contains('<') || json_data.contains("\\u003c"),
        "JSON section must not contain raw <"
    );
}

// ── Render: special characters in paths ─────────────────────────────

#[test]
fn render_handles_unicode_paths() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row("src/日本語.rs", "src", "Rust", 10)];
    receipt.derived = Some(derived_with_files(files));
    let html = render(&receipt);

    assert!(
        html.contains("日本語"),
        "Unicode paths should appear in output"
    );
}

#[test]
fn render_handles_empty_string_path() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row("", "", "", 0)];
    receipt.derived = Some(derived_with_files(files));
    // Should not panic
    let html = render(&receipt);
    assert!(html.contains("<!DOCTYPE html>"));
}

#[test]
fn render_handles_path_with_spaces() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row("src/my file.rs", "src", "Rust", 50)];
    receipt.derived = Some(derived_with_files(files));
    let html = render(&receipt);

    assert!(html.contains("src/my file.rs"));
}

// ── Render: numeric formatting edge cases ───────────────────────────

#[test]
fn render_formats_zero_code_as_plain_number() {
    let mut receipt = minimal_receipt();
    let mut derived = derived_with_files(vec![make_file_row("a.rs", ".", "Rust", 1)]);
    derived.totals.code = 0;
    receipt.derived = Some(derived);
    let html = render(&receipt);

    // 0 should appear as "0", not "0.0K" or "0.0M"
    assert!(html.contains(r#"<span class="value">0</span>"#));
}

#[test]
fn render_formats_999_without_suffix() {
    let mut receipt = minimal_receipt();
    let mut derived = derived_with_files(vec![make_file_row("a.rs", ".", "Rust", 1)]);
    derived.totals.code = 999;
    receipt.derived = Some(derived);
    let html = render(&receipt);

    assert!(html.contains(">999<"), "999 should not have K suffix");
}

#[test]
fn render_formats_exactly_1000_with_k_suffix() {
    let mut receipt = minimal_receipt();
    let mut derived = derived_with_files(vec![make_file_row("a.rs", ".", "Rust", 1)]);
    derived.totals.code = 1000;
    receipt.derived = Some(derived);
    let html = render(&receipt);

    assert!(html.contains("1.0K"));
}

// ── Render: context window card presence ────────────────────────────

#[test]
fn render_shows_5_card_divs_without_context_window() {
    let mut receipt = minimal_receipt();
    let mut derived = derived_with_files(vec![make_file_row("a.rs", ".", "Rust", 100)]);
    derived.context_window = None;
    receipt.derived = Some(derived);
    let html = render(&receipt);

    let card_count = html.matches(r#"class="metric-card""#).count();
    assert_eq!(card_count, 5, "should have exactly 5 metric card divs");
}

#[test]
fn render_shows_6_card_divs_with_context_window() {
    let mut receipt = minimal_receipt();
    let mut derived = derived_with_files(vec![make_file_row("a.rs", ".", "Rust", 100)]);
    derived.context_window = Some(ContextWindowReport {
        window_tokens: 128_000,
        total_tokens: 300,
        pct: 0.0023,
        fits: true,
    });
    receipt.derived = Some(derived);
    let html = render(&receipt);

    let card_count = html.matches(r#"class="metric-card""#).count();
    assert_eq!(
        card_count, 6,
        "should have 6 metric card divs with context window"
    );
}

// ── Render: table row count and ordering ────────────────────────────

#[test]
fn render_exactly_n_rows_for_n_files_under_100() {
    let mut receipt = minimal_receipt();
    let files: Vec<FileStatRow> = (0..50)
        .map(|i| make_file_row(&format!("f{i}.rs"), ".", "Rust", 10))
        .collect();
    receipt.derived = Some(derived_with_files(files));
    let html = render(&receipt);

    let row_count = html.matches("<tr><td").count();
    assert_eq!(row_count, 50, "should render exactly 50 rows");
}

#[test]
fn render_caps_at_100_rows() {
    let mut receipt = minimal_receipt();
    let files: Vec<FileStatRow> = (0..200)
        .map(|i| make_file_row(&format!("f{i}.rs"), ".", "Rust", 10))
        .collect();
    receipt.derived = Some(derived_with_files(files));
    let html = render(&receipt);

    let row_count = html.matches("<tr><td").count();
    assert_eq!(row_count, 100, "should cap at 100 rows");
}

// ── Render: determinism ─────────────────────────────────────────────

#[test]
fn render_is_deterministic_for_same_input() {
    let mut receipt = minimal_receipt();
    let files = vec![
        make_file_row("a.rs", "src", "Rust", 100),
        make_file_row("b.py", "lib", "Python", 50),
    ];
    receipt.derived = Some(derived_with_files(files));

    let html1 = render(&receipt);
    let html2 = render(&receipt);

    // Timestamps differ, so strip them for comparison
    let strip_ts = |s: &str| {
        let start = s.find("20").unwrap_or(0);
        let end = s.find(" UTC").map(|i| i + 4).unwrap_or(start);
        format!("{}{}", &s[..start], &s[end..])
    };
    assert_eq!(strip_ts(&html1), strip_ts(&html2));
}

// ── Render: data-attributes contain raw numeric values ──────────────

#[test]
fn render_data_attributes_contain_raw_numbers() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row("a.rs", ".", "Rust", 2500)];
    receipt.derived = Some(derived_with_files(files));
    let html = render(&receipt);

    // data-code should contain the raw number, not formatted
    assert!(
        html.contains("data-code=\"2500\""),
        "data-code should contain raw number"
    );
}
