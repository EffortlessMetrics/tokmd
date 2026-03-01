//! BDD-style scenario tests for HTML report generation.
//!
//! Each test follows Given / When / Then structure to verify
//! `tokmd_analysis_html::render` behaviour across edge cases.

use tokmd_analysis_html::render;
use tokmd_analysis_types::*;

// ── Helpers ──────────────────────────────────────────────────────────

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

// ── Scenario: Empty input (no derived data) ─────────────────────────

#[test]
fn given_empty_receipt_when_rendered_then_produces_valid_html_shell() {
    // Given a receipt with no derived data
    let receipt = minimal_receipt();

    // When we render it
    let html = render(&receipt);

    // Then it produces a valid HTML document shell
    assert!(
        html.starts_with("<!DOCTYPE html>"),
        "must start with DOCTYPE"
    );
    assert!(html.contains("<html"), "must contain <html> tag");
    assert!(html.contains("</html>"), "must close <html> tag");
    assert!(html.contains("<head>"), "must contain <head>");
    assert!(html.contains("</head>"), "must close <head>");
    assert!(html.contains("<body>"), "must contain <body>");
    assert!(html.contains("</body>"), "must close <body>");
}

#[test]
fn given_empty_receipt_when_rendered_then_table_body_is_empty() {
    let receipt = minimal_receipt();
    let html = render(&receipt);

    // No <tr> rows should be present in the table body (only <th> in header)
    // The table rows placeholder is replaced with an empty string
    assert!(
        !html.contains("<tr><td"),
        "no data rows when derived is None"
    );
}

#[test]
fn given_empty_receipt_when_rendered_then_report_json_has_empty_files() {
    let receipt = minimal_receipt();
    let html = render(&receipt);

    assert!(
        html.contains(r#"{"files":[]}"#),
        "REPORT_DATA should contain empty files array"
    );
}

// ── Scenario: Single language, single file ──────────────────────────

#[test]
fn given_single_file_when_rendered_then_contains_file_path_in_table() {
    // Given a receipt with one Rust file
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row("src/main.rs", "src", "Rust", 100)];
    receipt.derived = Some(derived_with_files(files));

    // When rendered
    let html = render(&receipt);

    // Then the file path appears in the table
    assert!(
        html.contains("src/main.rs"),
        "file path must appear in table"
    );
}

#[test]
fn given_single_file_when_rendered_then_metrics_cards_show_totals() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row("src/main.rs", "src", "Rust", 100)];
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    assert!(html.contains("metric-card"), "must include metric cards");
    assert!(html.contains("Files"), "must show Files label");
    assert!(html.contains("Code"), "must show Code label");
    assert!(html.contains("Tokens"), "must show Tokens label");
}

#[test]
fn given_single_file_when_rendered_then_lang_badge_appears() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row("src/main.rs", "src", "Rust", 100)];
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    assert!(html.contains("lang-badge"), "must include language badge");
    assert!(html.contains("Rust"), "must show Rust language");
}

// ── Scenario: Multiple languages ────────────────────────────────────

#[test]
fn given_multi_language_when_rendered_then_all_languages_appear() {
    let mut receipt = minimal_receipt();
    let files = vec![
        make_file_row("src/main.rs", "src", "Rust", 500),
        make_file_row("src/utils.py", "src", "Python", 300),
        make_file_row("web/app.ts", "web", "TypeScript", 200),
    ];
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    assert!(html.contains("Rust"), "must contain Rust");
    assert!(html.contains("Python"), "must contain Python");
    assert!(html.contains("TypeScript"), "must contain TypeScript");
}

#[test]
fn given_multi_language_when_rendered_then_all_file_paths_appear() {
    let mut receipt = minimal_receipt();
    let files = vec![
        make_file_row("src/main.rs", "src", "Rust", 500),
        make_file_row("src/utils.py", "src", "Python", 300),
        make_file_row("web/app.ts", "web", "TypeScript", 200),
    ];
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    assert!(html.contains("src/main.rs"));
    assert!(html.contains("src/utils.py"));
    assert!(html.contains("web/app.ts"));
}

#[test]
fn given_multi_language_when_rendered_then_modules_appear() {
    let mut receipt = minimal_receipt();
    let files = vec![
        make_file_row("src/main.rs", "src", "Rust", 500),
        make_file_row("web/app.ts", "web", "TypeScript", 200),
    ];
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    // Both modules should be in the table
    assert!(html.contains(r#"data-module="src""#));
    assert!(html.contains(r#"data-module="web""#));
}

// ── Scenario: Treemap data ──────────────────────────────────────────

#[test]
fn given_derived_data_when_rendered_then_treemap_section_exists() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row("src/main.rs", "src", "Rust", 100)];
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    assert!(html.contains("treemap"), "must contain treemap element");
}

#[test]
fn given_derived_data_when_rendered_then_report_json_has_file_data() {
    let mut receipt = minimal_receipt();
    let files = vec![
        make_file_row("src/main.rs", "src", "Rust", 500),
        make_file_row("lib/utils.py", "lib", "Python", 200),
    ];
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    // The embedded JSON must contain file entries for the treemap
    assert!(
        html.contains("REPORT_DATA"),
        "must contain REPORT_DATA variable"
    );
    assert!(html.contains("src/main.rs"), "JSON must contain file path");
    assert!(
        html.contains("lib/utils.py"),
        "JSON must contain second file path"
    );
}

// ── Scenario: HTML escaping / XSS prevention ────────────────────────

#[test]
fn given_malicious_path_when_rendered_then_html_is_escaped() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row(
        "<img src=x onerror=alert(1)>",
        "evil",
        "JavaScript",
        10,
    )];
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    // The raw XSS payload must NOT appear unescaped
    assert!(
        !html.contains("<img src=x onerror=alert(1)>"),
        "raw XSS payload must not appear in HTML"
    );
    // The escaped form must appear in the table
    assert!(html.contains("&lt;img src=x onerror=alert(1)&gt;"));
}

#[test]
fn given_script_injection_in_lang_when_rendered_then_escaped() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row(
        "test.js",
        ".",
        "<script>alert('xss')</script>",
        10,
    )];
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    assert!(!html.contains("<script>alert('xss')</script>"));
}

#[test]
fn given_script_in_json_when_rendered_then_angle_brackets_escaped() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row(
        "</script><script>alert(1)</script>",
        ".",
        "JS",
        10,
    )];
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    // In the JSON section, < and > must be escaped to prevent script breakout
    // The JSON uses \u003c and \u003e escapes
    assert!(
        html.contains("\\u003c/script\\u003e"),
        "JSON must escape </script> to prevent breakout"
    );
}

// ── Scenario: Large number formatting ───────────────────────────────

#[test]
fn given_large_code_count_when_rendered_then_formatted_with_suffix() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row("big.rs", ".", "Rust", 1_500_000)];
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    // 1.5M code lines should appear formatted
    assert!(html.contains("1.5M"), "large numbers should use M suffix");
}

#[test]
fn given_medium_code_count_when_rendered_then_formatted_with_k_suffix() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row("med.rs", ".", "Rust", 2_500)];
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    assert!(html.contains("2.5K"), "medium numbers should use K suffix");
}

// ── Scenario: Context window presence ───────────────────────────────

#[test]
fn given_context_window_when_rendered_then_context_fit_card_appears() {
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

    assert!(html.contains("Context Fit"), "must show Context Fit card");
}

#[test]
fn given_no_context_window_when_rendered_then_no_context_fit_card() {
    let mut receipt = minimal_receipt();
    let mut derived = derived_with_files(vec![make_file_row("a.rs", ".", "Rust", 100)]);
    derived.context_window = None;
    receipt.derived = Some(derived);

    let html = render(&receipt);

    assert!(
        !html.contains("Context Fit"),
        "must NOT show Context Fit when context_window is None"
    );
}

// ── Scenario: Timestamp ─────────────────────────────────────────────

#[test]
fn given_any_receipt_when_rendered_then_timestamp_is_present() {
    let receipt = minimal_receipt();
    let html = render(&receipt);

    assert!(
        html.contains("UTC"),
        "rendered HTML must contain UTC timestamp"
    );
}

// ── Scenario: Table structure ───────────────────────────────────────

#[test]
fn given_files_when_rendered_then_table_has_data_attributes() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row("src/app.rs", "src", "Rust", 250)];
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    assert!(
        html.contains("data-path="),
        "rows must have data-path attribute"
    );
    assert!(
        html.contains("data-module="),
        "rows must have data-module attribute"
    );
    assert!(
        html.contains("data-lang="),
        "rows must have data-lang attribute"
    );
    assert!(
        html.contains("data-lines="),
        "rows must have data-lines attribute"
    );
    assert!(
        html.contains("data-code="),
        "rows must have data-code attribute"
    );
    assert!(
        html.contains("data-tokens="),
        "rows must have data-tokens attribute"
    );
    assert!(
        html.contains("data-bytes="),
        "rows must have data-bytes attribute"
    );
}

// ── Scenario: 100-file cap ──────────────────────────────────────────

#[test]
fn given_over_100_files_when_rendered_then_table_has_at_most_100_rows() {
    let mut receipt = minimal_receipt();
    let files: Vec<FileStatRow> = (0..150)
        .map(|i| make_file_row(&format!("src/file_{i}.rs"), "src", "Rust", 10 + i))
        .collect();
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    let row_count = html.matches("<tr><td").count();
    assert!(
        row_count <= 100,
        "table should have at most 100 rows, got {row_count}"
    );
}

// ── Scenario: HTML output is well-formed ────────────────────────────

#[test]
fn given_receipt_with_derived_when_rendered_then_html_is_well_formed() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row("src/main.rs", "src", "Rust", 200)];
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<html"), "must contain opening <html> tag");
    assert!(html.contains("</html>"), "must contain closing </html> tag");
    assert!(html.contains("<head>"));
    assert!(html.contains("</head>"));
    assert!(html.contains("<body>"));
    assert!(html.contains("</body>"));
    assert!(html.contains("<title>"), "must have a <title> tag");
    assert!(html.contains("</title>"));
}

#[test]
fn given_empty_receipt_when_rendered_then_html_is_well_formed() {
    let receipt = minimal_receipt();
    let html = render(&receipt);

    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<html"));
    assert!(html.contains("</html>"));
    assert!(html.contains("<head>"));
    assert!(html.contains("</head>"));
    assert!(html.contains("<body>"));
    assert!(html.contains("</body>"));
}

// ── Scenario: Known metrics appear in HTML output ───────────────────

#[test]
fn given_derived_data_when_rendered_then_known_metric_labels_appear() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row("src/main.rs", "src", "Rust", 500)];
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    assert!(html.contains("Files"), "must show Files metric");
    assert!(html.contains("Lines"), "must show Lines metric");
    assert!(html.contains("Code"), "must show Code metric");
    assert!(html.contains("Tokens"), "must show Tokens metric");
    assert!(html.contains("Doc%"), "must show Doc% metric");
}

#[test]
fn given_derived_with_context_window_then_all_six_metric_cards_appear() {
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

    let card_count = html.matches("class=\"metric-card\"").count();
    assert_eq!(
        card_count, 6,
        "expect 6 metric cards (5 base + Context Fit)"
    );
}

#[test]
fn given_derived_without_context_window_then_five_metric_cards_appear() {
    let mut receipt = minimal_receipt();
    let mut derived = derived_with_files(vec![make_file_row("a.rs", ".", "Rust", 100)]);
    derived.context_window = None;
    receipt.derived = Some(derived);

    let html = render(&receipt);

    let card_count = html.matches("class=\"metric-card\"").count();
    assert_eq!(
        card_count, 5,
        "expect 5 metric cards when no context window"
    );
}

// ── Scenario: HTML output is deterministic ──────────────────────────

#[test]
fn given_same_receipt_when_rendered_twice_then_output_is_identical_modulo_timestamp() {
    let mut receipt = minimal_receipt();
    let files = vec![
        make_file_row("src/main.rs", "src", "Rust", 500),
        make_file_row("src/utils.py", "src", "Python", 200),
    ];
    receipt.derived = Some(derived_with_files(files));

    let html_a = render(&receipt);
    let html_b = render(&receipt);

    // Strip timestamps to compare determinism of the rest.
    let strip_ts = |html: &str| -> String {
        let mut result = html.to_string();
        while let Some(pos) = result.find(" UTC") {
            if pos >= 19 {
                let candidate = &result[pos - 19..pos + 4];
                if candidate.len() == 23
                    && candidate.as_bytes()[4] == b'-'
                    && candidate.as_bytes()[7] == b'-'
                    && candidate.as_bytes()[10] == b' '
                {
                    result.replace_range(pos - 19..pos + 4, "[TS]");
                    continue;
                }
            }
            break;
        }
        result
    };

    assert_eq!(
        strip_ts(&html_a),
        strip_ts(&html_b),
        "render must be deterministic"
    );
}

#[test]
fn given_empty_receipt_when_rendered_twice_then_output_is_identical_modulo_timestamp() {
    let receipt = minimal_receipt();

    let html_a = render(&receipt);
    let html_b = render(&receipt);

    let strip = |h: &str| h.replace(char::is_numeric, "X");
    assert_eq!(strip(&html_a), strip(&html_b));
}

// ── Scenario: Empty analysis produces valid HTML ────────────────────

#[test]
fn given_no_derived_data_then_no_metric_cards() {
    let receipt = minimal_receipt();
    let html = render(&receipt);
    assert!(
        !html.contains("class=\"metric-card\""),
        "no metric cards without derived data"
    );
}

#[test]
fn given_no_derived_data_then_no_table_data_rows() {
    let receipt = minimal_receipt();
    let html = render(&receipt);
    assert_eq!(
        html.matches("<tr><td").count(),
        0,
        "no table data rows without derived data"
    );
}

#[test]
fn given_derived_with_empty_files_when_rendered_then_valid_html() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(derived_with_files(vec![]));

    let html = render(&receipt);

    assert!(html.contains("<html"));
    assert!(html.contains("</html>"));
    assert!(
        html.contains("metric-card"),
        "still has metric cards for totals"
    );
    assert_eq!(
        html.matches("<tr><td").count(),
        0,
        "no table rows for empty files"
    );
}
