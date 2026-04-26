//! Depth tests for tokmd-format analysis HTML rendering.

use tokmd_analysis_types::*;
use tokmd_format::analysis::html::render;

// ── helpers ────────────────────────────────────────────────────────

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

fn make_file_row(path: &str, lang: &str, code: usize) -> FileStatRow {
    FileStatRow {
        path: path.into(),
        module: "src".into(),
        lang: lang.into(),
        code,
        comments: code / 5,
        blanks: code / 10,
        lines: code + code / 5 + code / 10,
        bytes: code * 50,
        tokens: code * 2,
        doc_pct: Some(0.15),
        bytes_per_line: Some(38.0),
        depth: 1,
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
            overall: make_file_row("src/lib.rs", "Rust", 500),
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
            largest_lines: vec![make_file_row("src/lib.rs", "Rust", 500)],
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

fn receipt_with_derived() -> AnalysisReceipt {
    let mut r = minimal_receipt();
    r.derived = Some(sample_derived());
    r
}

// ── 1. Basic HTML structure ────────────────────────────────────────

#[test]
fn render_produces_valid_doctype() {
    let html = render(&minimal_receipt());
    assert!(html.starts_with("<!DOCTYPE html>"));
}

#[test]
fn render_contains_html_lang_attribute() {
    let html = render(&minimal_receipt());
    assert!(html.contains(r#"<html lang="en">"#));
}

#[test]
fn render_contains_meta_charset() {
    let html = render(&minimal_receipt());
    assert!(html.contains(r#"<meta charset="UTF-8">"#));
}

#[test]
fn render_contains_viewport_meta() {
    let html = render(&minimal_receipt());
    assert!(html.contains("viewport"));
}

#[test]
fn render_contains_title() {
    let html = render(&minimal_receipt());
    assert!(html.contains("<title>tokmd Analysis Report</title>"));
}

#[test]
fn render_contains_closing_tags() {
    let html = render(&minimal_receipt());
    assert!(html.contains("</html>"));
    assert!(html.contains("</head>"));
    assert!(html.contains("</body>"));
}

#[test]
fn render_contains_header_section() {
    let html = render(&minimal_receipt());
    assert!(html.contains("<header>"));
    assert!(html.contains("</header>"));
}

#[test]
fn render_contains_footer() {
    let html = render(&minimal_receipt());
    assert!(html.contains("<footer>"));
    assert!(html.contains("</footer>"));
}

// ── 2. Empty / no-derived rendering ───────────────────────────────

#[test]
fn render_empty_receipt_no_metric_cards() {
    let html = render(&minimal_receipt());
    assert!(!html.contains("class=\"metric-card\""));
}

#[test]
fn render_empty_receipt_no_table_rows() {
    let html = render(&minimal_receipt());
    // Table header exists, but no data rows
    assert!(html.contains("<thead>"));
    assert!(!html.contains("data-path="));
}

#[test]
fn render_empty_receipt_report_json_empty_files() {
    let html = render(&minimal_receipt());
    assert!(html.contains(r#"{"files":[]}"#));
}

// ── 3. Metrics cards with derived data ────────────────────────────

#[test]
fn render_with_derived_has_metric_cards() {
    let html = render(&receipt_with_derived());
    assert!(html.contains("class=\"metric-card\""));
}

#[test]
fn metric_card_shows_files_count() {
    let html = render(&receipt_with_derived());
    assert!(html.contains(">10<"));
    assert!(html.contains(">Files<"));
}

#[test]
fn metric_card_shows_lines_formatted() {
    let html = render(&receipt_with_derived());
    // 1300 lines → "1.3K"
    assert!(html.contains("1.3K"));
    assert!(html.contains(">Lines<"));
}

#[test]
fn metric_card_shows_code_formatted() {
    let html = render(&receipt_with_derived());
    // 1000 code → "1.0K"
    assert!(html.contains("1.0K"));
    assert!(html.contains(">Code<"));
}

#[test]
fn metric_card_shows_tokens_formatted() {
    let html = render(&receipt_with_derived());
    // 2500 tokens → "2.5K"
    assert!(html.contains("2.5K"));
    assert!(html.contains(">Tokens<"));
}

#[test]
fn metric_card_shows_doc_pct() {
    let html = render(&receipt_with_derived());
    assert!(html.contains("16.7%"));
    assert!(html.contains(">Doc%<"));
}

#[test]
fn metric_card_shows_context_fit() {
    let html = render(&receipt_with_derived());
    assert!(html.contains("2.5%"));
    assert!(html.contains(">Context Fit<"));
}

#[test]
fn metric_card_no_context_fit_when_absent() {
    let mut r = receipt_with_derived();
    r.derived.as_mut().unwrap().context_window = None;
    let html = render(&r);
    assert!(!html.contains("Context Fit"));
}

// ── 4. Table rows ─────────────────────────────────────────────────

#[test]
fn table_row_contains_path() {
    let html = render(&receipt_with_derived());
    assert!(html.contains("src/lib.rs"));
}

#[test]
fn table_row_contains_module() {
    let html = render(&receipt_with_derived());
    assert!(html.contains(r#"data-module="src""#));
}

#[test]
fn table_row_contains_lang_badge() {
    let html = render(&receipt_with_derived());
    assert!(html.contains("class=\"lang-badge\""));
    assert!(html.contains("Rust"));
}

#[test]
fn table_row_contains_data_attributes() {
    let html = render(&receipt_with_derived());
    assert!(html.contains("data-lines="));
    assert!(html.contains("data-code="));
    assert!(html.contains("data-tokens="));
    assert!(html.contains("data-bytes="));
}

#[test]
fn table_row_code_formatted() {
    let html = render(&receipt_with_derived());
    // code=500 → "500" (below 1000, raw number)
    assert!(html.contains(">500<"));
}

// ── 5. HTML escaping ──────────────────────────────────────────────

#[test]
fn escape_html_in_path_xss() {
    let mut r = receipt_with_derived();
    r.derived.as_mut().unwrap().top.largest_lines[0].path = "<script>alert('xss')</script>".into();
    let html = render(&r);
    assert!(html.contains("&lt;script&gt;"));
    assert!(!html.contains("<script>alert"));
}

#[test]
fn escape_html_ampersand_in_module() {
    let mut r = receipt_with_derived();
    r.derived.as_mut().unwrap().top.largest_lines[0].module = "a&b".into();
    let html = render(&r);
    assert!(html.contains("a&amp;b"));
}

#[test]
fn escape_html_quotes_in_lang() {
    let mut r = receipt_with_derived();
    r.derived.as_mut().unwrap().top.largest_lines[0].lang = r#"C"plus""#.into();
    let html = render(&r);
    assert!(html.contains("C&quot;plus&quot;"));
}

#[test]
fn escape_html_single_quote() {
    let mut r = receipt_with_derived();
    r.derived.as_mut().unwrap().top.largest_lines[0].path = "it's.rs".into();
    let html = render(&r);
    assert!(html.contains("it&#x27;s.rs"));
}

#[test]
fn escape_combined_special_chars() {
    let mut r = receipt_with_derived();
    r.derived.as_mut().unwrap().top.largest_lines[0].path = r#"<a href="x">&'"#.into();
    let html = render(&r);
    assert!(html.contains("&lt;a href=&quot;x&quot;&gt;&amp;&#x27;"));
}

// ── 6. Report JSON safety ─────────────────────────────────────────

#[test]
fn report_json_no_raw_angle_brackets() {
    let mut r = receipt_with_derived();
    r.derived.as_mut().unwrap().top.largest_lines[0].path =
        "</script><img onerror=alert(1)>".into();
    let html = render(&r);
    // The JSON section must not contain raw < or >
    let json_start = html.find("const REPORT_DATA =").unwrap();
    let json_section = &html[json_start..json_start + 500];
    assert!(!json_section.contains("</script>"));
}

#[test]
fn report_json_uses_unicode_escapes() {
    let mut r = receipt_with_derived();
    r.derived.as_mut().unwrap().top.largest_lines[0].path = "<>".into();
    let html = render(&r);
    assert!(html.contains("\\u003c"));
    assert!(html.contains("\\u003e"));
}

#[test]
fn report_json_preserves_valid_json_structure() {
    let r = receipt_with_derived();
    let html = render(&r);
    let json_start = html.find("const REPORT_DATA = ").unwrap() + "const REPORT_DATA = ".len();
    let json_end = html[json_start..].find(";\n").unwrap() + json_start;
    let json_str = &html[json_start..json_end];
    // Undo the angle-bracket escaping for parse validation
    let json_str = json_str.replace("\\u003c", "<").replace("\\u003e", ">");
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(parsed.get("files").unwrap().is_array());
}

// ── 7. Large dataset rendering ────────────────────────────────────

#[test]
fn table_rows_capped_at_100() {
    let mut r = receipt_with_derived();
    let d = r.derived.as_mut().unwrap();
    d.top.largest_lines = (0..150)
        .map(|i| make_file_row(&format!("src/file_{i}.rs"), "Rust", 100 + i))
        .collect();
    let html = render(&r);
    let count = html.matches("data-path=").count();
    assert_eq!(count, 100, "table rows should be capped at 100");
}

#[test]
fn large_dataset_report_json_has_all_files() {
    let mut r = receipt_with_derived();
    let d = r.derived.as_mut().unwrap();
    d.top.largest_lines = (0..200)
        .map(|i| make_file_row(&format!("src/file_{i}.rs"), "Rust", 100))
        .collect();
    let html = render(&r);
    // JSON section contains all files (no cap)
    let json_start = html.find("const REPORT_DATA = ").unwrap() + "const REPORT_DATA = ".len();
    let json_end = html[json_start..].find(";\n").unwrap() + json_start;
    let json_str = &html[json_start..json_end];
    let json_str = json_str.replace("\\u003c", "<").replace("\\u003e", ">");
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed["files"].as_array().unwrap().len(), 200);
}

// ── 8. Number formatting ──────────────────────────────────────────

#[test]
fn format_number_small_values() {
    let mut r = receipt_with_derived();
    let d = r.derived.as_mut().unwrap();
    d.totals.files = 3;
    let html = render(&r);
    assert!(html.contains(">3<"));
}

#[test]
fn format_number_thousands() {
    let mut r = receipt_with_derived();
    let d = r.derived.as_mut().unwrap();
    d.totals.code = 5500;
    let html = render(&r);
    assert!(html.contains("5.5K"));
}

#[test]
fn format_number_millions() {
    let mut r = receipt_with_derived();
    let d = r.derived.as_mut().unwrap();
    d.totals.lines = 3_200_000;
    let html = render(&r);
    assert!(html.contains("3.2M"));
}

#[test]
fn format_pct_renders_one_decimal() {
    let mut r = receipt_with_derived();
    let d = r.derived.as_mut().unwrap();
    d.doc_density.total.ratio = 0.3333;
    let html = render(&r);
    assert!(html.contains("33.3%"));
}

// ── 9. CSS classes ────────────────────────────────────────────────

#[test]
fn css_num_class_on_numeric_cells() {
    let html = render(&receipt_with_derived());
    assert!(html.contains(r#"class="num""#));
}

#[test]
fn css_path_class_on_path_cell() {
    let html = render(&receipt_with_derived());
    assert!(html.contains(r#"class="path""#));
}

#[test]
fn css_metric_card_class() {
    let html = render(&receipt_with_derived());
    assert!(html.contains(r#"class="metric-card""#));
}

#[test]
fn css_value_and_label_spans() {
    let html = render(&receipt_with_derived());
    assert!(html.contains(r#"class="value""#));
    assert!(html.contains(r#"class="label""#));
}

#[test]
fn css_search_box_present() {
    let html = render(&minimal_receipt());
    assert!(html.contains(r#"class="search-box""#));
}

#[test]
fn css_lang_badge_class() {
    let html = render(&receipt_with_derived());
    assert!(html.contains(r#"class="lang-badge""#));
}

// ── 10. Accessibility ─────────────────────────────────────────────

#[test]
fn table_has_thead_and_tbody() {
    let html = render(&minimal_receipt());
    assert!(html.contains("<thead>"));
    assert!(html.contains("</thead>"));
    assert!(html.contains("<tbody>"));
    assert!(html.contains("</tbody>"));
}

#[test]
fn table_header_cells_use_th() {
    let html = render(&minimal_receipt());
    assert!(html.contains("<th "));
    assert!(html.contains("</th>"));
}

#[test]
fn search_input_has_placeholder() {
    let html = render(&minimal_receipt());
    assert!(html.contains("placeholder="));
}

#[test]
fn data_sort_attributes_on_headers() {
    let html = render(&minimal_receipt());
    assert!(html.contains(r#"data-sort="path""#));
    assert!(html.contains(r#"data-sort="lines""#));
    assert!(html.contains(r#"data-sort="code""#));
    assert!(html.contains(r#"data-sort="tokens""#));
    assert!(html.contains(r#"data-sort="bytes""#));
}

// ── 11. Timestamp ─────────────────────────────────────────────────

#[test]
fn timestamp_is_embedded() {
    let html = render(&minimal_receipt());
    assert!(html.contains("Generated:"));
    assert!(html.contains("UTC"));
}

// ── 12. JavaScript embedded ───────────────────────────────────────

#[test]
fn javascript_block_present() {
    let html = render(&minimal_receipt());
    assert!(html.contains("<script>"));
    assert!(html.contains("const REPORT_DATA ="));
    assert!(html.contains("</script>"));
}

#[test]
fn treemap_container_present() {
    let html = render(&minimal_receipt());
    assert!(html.contains(r#"id="treemap""#));
}

// ── 13. Determinism ───────────────────────────────────────────────

#[test]
fn deterministic_same_input_same_output() {
    let r = receipt_with_derived();
    let a = render(&r);
    let b = render(&r);
    // Replace timestamps since they may differ by a second
    let strip = |s: String| {
        let start = s.find("Generated: ").unwrap();
        let end = s[start..].find("</div>").unwrap() + start;
        format!("{}{}", &s[..start], &s[end..])
    };
    assert_eq!(strip(a), strip(b));
}

#[test]
fn deterministic_no_derived_identical() {
    let r = minimal_receipt();
    let a = render(&r);
    let b = render(&r);
    let strip = |s: String| {
        let start = s.find("Generated: ").unwrap();
        let end = s[start..].find("</div>").unwrap() + start;
        format!("{}{}", &s[..start], &s[end..])
    };
    assert_eq!(strip(a), strip(b));
}

// ── 14. Snapshot tests ────────────────────────────────────────────

#[test]
fn snapshot_empty_receipt_structure() {
    let html = render(&minimal_receipt());
    // Redact the timestamp for stable snapshots
    let redacted = html
        .lines()
        .map(|l| {
            if l.contains("Generated:") {
                "        <div class=\"timestamp\">Generated: [REDACTED]</div>"
            } else {
                l
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    insta::assert_snapshot!("empty_receipt", redacted);
}

#[test]
fn snapshot_derived_receipt_metrics_section() {
    let r = receipt_with_derived();
    let html = render(&r);
    // Extract the metrics-grid div from the rendered HTML body
    let marker = r#"<div class="metrics-grid">"#;
    let start = html.find(marker).unwrap();
    // Find the closing </div> that matches the metrics-grid div
    let inner_start = start + marker.len();
    let end = html[inner_start..].find("</div>").unwrap() + inner_start + 6;
    let section = &html[start..end];
    insta::assert_snapshot!("derived_metrics_grid", section);
}

// ── 15. Property tests ────────────────────────────────────────────

mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn html_always_starts_with_doctype(
            files in 0usize..10,
            code in 1usize..10000,
        ) {
            let mut r = minimal_receipt();
            if files > 0 {
                let mut d = sample_derived();
                d.top.largest_lines = (0..files)
                    .map(|i| make_file_row(&format!("f{i}.rs"), "Rust", code))
                    .collect();
                d.totals.files = files;
                d.totals.code = code * files;
                r.derived = Some(d);
            }
            let html = render(&r);
            prop_assert!(html.starts_with("<!DOCTYPE html>"));
        }

        #[test]
        fn html_always_has_closing_tags(
            files in 0usize..5,
        ) {
            let mut r = minimal_receipt();
            if files > 0 {
                let mut d = sample_derived();
                d.top.largest_lines = (0..files)
                    .map(|i| make_file_row(&format!("f{i}.rs"), "Rust", 100))
                    .collect();
                r.derived = Some(d);
            }
            let html = render(&r);
            prop_assert!(html.contains("</html>"));
            prop_assert!(html.contains("</body>"));
        }

        #[test]
        fn json_section_never_contains_raw_script_close(
            path in "[a-zA-Z0-9/<>\"'& ]{1,50}",
        ) {
            let mut r = receipt_with_derived();
            r.derived.as_mut().unwrap().top.largest_lines[0].path = path;
            let html = render(&r);
            let json_start = html.find("const REPORT_DATA =").unwrap();
            let json_end = html[json_start..].find(";\n").unwrap() + json_start;
            let json_section = &html[json_start..json_end];
            prop_assert!(!json_section.contains("</script>"));
        }

        #[test]
        fn table_rows_never_contain_raw_angle_brackets_in_data(
            path in "[a-zA-Z0-9/<>\"'& ]{1,30}",
        ) {
            let mut r = receipt_with_derived();
            r.derived.as_mut().unwrap().top.largest_lines[0].path = path;
            let html = render(&r);
            // Extract tbody content
            let tbody_start = html.find("<tbody>").unwrap() + 7;
            let tbody_end = html[tbody_start..].find("</tbody>").unwrap() + tbody_start;
            let tbody = &html[tbody_start..tbody_end];
            // After stripping known HTML tags, no raw < or > should remain as data
            let stripped = tbody
                .replace("<tr>", "").replace("</tr>", "")
                .replace("<td", "").replace("</td>", "");
            // Every remaining < should be from &lt; escaping, not raw
            for (i, c) in stripped.char_indices() {
                if c == '<' {
                    // Must be part of an attribute close or known pattern
                    let before = &stripped[..i];
                    prop_assert!(
                        before.ends_with('=')
                            || before.ends_with('"')
                            || before.ends_with(' ')
                            || stripped[i..].starts_with("<span")
                            || stripped[i..].starts_with("</span"),
                        "unexpected raw < at position {i}"
                    );
                }
            }
        }

        #[test]
        fn format_number_never_panics(n in 0usize..usize::MAX / 2) {
            let mut r = minimal_receipt();
            let mut d = sample_derived();
            d.totals.code = n;
            r.derived = Some(d);
            let html = render(&r);
            prop_assert!(!html.is_empty());
        }
    }
}
