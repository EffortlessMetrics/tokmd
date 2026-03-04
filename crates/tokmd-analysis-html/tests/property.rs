//! Property-based tests for `tokmd_analysis_html::render`.
//!
//! These tests use `proptest` to verify invariants that must hold
//! for *any* valid input, not just hand-picked examples.

use proptest::prelude::*;
use tokmd_analysis_html::render;
use tokmd_analysis_types::*;

// ── Strategies ──────────────────────────────────────────────────────

/// Produce an arbitrary printable string that may include HTML-sensitive chars.
fn html_nasty_string() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z0-9 _/\\-.<>&\"'\\\\]{0,80}")
        .unwrap()
        .prop_union(
            prop::string::string_regex("<script>alert\\(1\\)</script>|<img src=x>|a&b|\"quoted\"|it's")
                .unwrap(),
        )
}

fn arb_file_row() -> impl Strategy<Value = FileStatRow> {
    (
        html_nasty_string(),
        html_nasty_string(),
        html_nasty_string(),
        0..100_000_usize,
    )
        .prop_map(|(path, module, lang, code)| FileStatRow {
            path,
            module,
            lang,
            code,
            comments: code / 5,
            blanks: code / 10,
            lines: code + code / 5 + code / 10,
            bytes: code * 50,
            tokens: code * 3,
            doc_pct: Some(0.15),
            bytes_per_line: Some(40.0),
            depth: 1,
        })
}

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
                .unwrap_or_else(|| FileStatRow {
                    path: "empty".into(),
                    module: ".".into(),
                    lang: "Text".into(),
                    code: 0,
                    comments: 0,
                    blanks: 0,
                    lines: 0,
                    bytes: 0,
                    tokens: 0,
                    doc_pct: None,
                    bytes_per_line: None,
                    depth: 0,
                }),
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

// ── Property: HTML always starts with DOCTYPE ───────────────────────

proptest! {
    #[test]
    fn render_always_starts_with_doctype(
        files in prop::collection::vec(arb_file_row(), 0..20)
    ) {
        let mut receipt = minimal_receipt();
        if !files.is_empty() {
            receipt.derived = Some(derived_with_files(files));
        }
        let html = render(&receipt);
        prop_assert!(html.starts_with("<!DOCTYPE html>"));
    }
}

// ── Property: HTML always has balanced structural tags ───────────────

proptest! {
    #[test]
    fn render_always_has_balanced_structure(
        files in prop::collection::vec(arb_file_row(), 0..10)
    ) {
        let mut receipt = minimal_receipt();
        if !files.is_empty() {
            receipt.derived = Some(derived_with_files(files));
        }
        let html = render(&receipt);
        prop_assert!(html.contains("<html"));
        prop_assert!(html.contains("</html>"));
        prop_assert!(html.contains("<head>"));
        prop_assert!(html.contains("</head>"));
        prop_assert!(html.contains("<body>"));
        prop_assert!(html.contains("</body>"));
    }
}

// ── Property: Table rows never exceed 100 ───────────────────────────

proptest! {
    #[test]
    fn render_table_rows_capped_at_100(
        count in 0..200_usize
    ) {
        let mut receipt = minimal_receipt();
        let files: Vec<FileStatRow> = (0..count)
            .map(|i| FileStatRow {
                path: format!("f{i}.rs"),
                module: ".".into(),
                lang: "Rust".into(),
                code: 10,
                comments: 2,
                blanks: 1,
                lines: 13,
                bytes: 500,
                tokens: 30,
                doc_pct: Some(0.15),
                bytes_per_line: Some(40.0),
                depth: 0,
            })
            .collect();
        receipt.derived = Some(derived_with_files(files));
        let html = render(&receipt);

        let row_count = html.matches("<tr><td").count();
        prop_assert!(row_count <= 100, "got {} rows for {} files", row_count, count);
    }
}

// ── Property: REPORT_DATA JSON section never contains raw < or > ────

proptest! {
    #[test]
    fn render_json_section_never_has_raw_angle_brackets(
        files in prop::collection::vec(arb_file_row(), 1..10)
    ) {
        let mut receipt = minimal_receipt();
        receipt.derived = Some(derived_with_files(files));
        let html = render(&receipt);

        // Extract the JSON section between "REPORT_DATA =" and the next ";"
        if let Some(start) = html.find("const REPORT_DATA =") {
            let json_start = start + "const REPORT_DATA =".len();
            if let Some(end) = html[json_start..].find(';') {
                let json_section = &html[json_start..json_start + end];
                prop_assert!(
                    !json_section.contains('<'),
                    "JSON section must not contain raw '<'"
                );
                prop_assert!(
                    !json_section.contains('>'),
                    "JSON section must not contain raw '>'"
                );
            }
        }
    }
}

// ── Property: table HTML attributes never contain unescaped chars ────

proptest! {
    #[test]
    fn render_table_cells_never_have_raw_angle_brackets_in_data_attrs(
        path in html_nasty_string(),
        module in html_nasty_string(),
        lang in html_nasty_string(),
    ) {
        let mut receipt = minimal_receipt();
        let files = vec![FileStatRow {
            path: path.clone(),
            module: module.clone(),
            lang: lang.clone(),
            code: 42,
            comments: 8,
            blanks: 4,
            lines: 54,
            bytes: 2100,
            tokens: 126,
            doc_pct: Some(0.15),
            bytes_per_line: Some(40.0),
            depth: 1,
        }];
        receipt.derived = Some(derived_with_files(files));
        let html = render(&receipt);

        // Find the table body and check that data-path values are escaped
        if let Some(tbody_start) = html.find("<tbody>") {
            if let Some(tbody_end) = html[tbody_start..].find("</tbody>") {
                let tbody = &html[tbody_start..tbody_start + tbody_end];
                // If the original strings contain '<' or '>', they must appear
                // as &lt; / &gt; in the table body HTML attributes
                if path.contains('<') || module.contains('<') || lang.contains('<') {
                    // The tbody should NOT contain the raw substring
                    // unless it's part of an HTML tag we generated (like <tr>, <td>)
                    // Check data-path attribute specifically
                    for attr_name in &["data-path=\"", "data-module=\"", "data-lang=\""] {
                        if let Some(pos) = tbody.find(attr_name) {
                            let attr_start = pos + attr_name.len();
                            if let Some(attr_end) = tbody[attr_start..].find('"') {
                                let attr_val = &tbody[attr_start..attr_start + attr_end];
                                prop_assert!(
                                    !attr_val.contains('<') && !attr_val.contains('>'),
                                    "attribute {} contains raw angle bracket: {}",
                                    attr_name, attr_val
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── Property: render never panics for edge-case numeric values ──────

proptest! {
    #[test]
    fn render_never_panics_for_any_totals(
        files_count in 0..5_usize,
        code in 0..10_000_000_usize,
        ratio in 0.0..1.0_f64,
    ) {
        let mut receipt = minimal_receipt();
        let files: Vec<FileStatRow> = (0..files_count)
            .map(|i| FileStatRow {
                path: format!("f{i}.rs"),
                module: ".".into(),
                lang: "Rust".into(),
                code,
                comments: code / 5,
                blanks: code / 10,
                lines: code + code / 5 + code / 10,
                bytes: code.saturating_mul(50),
                tokens: code.saturating_mul(3),
                doc_pct: Some(ratio),
                bytes_per_line: Some(40.0),
                depth: 0,
            })
            .collect();
        receipt.derived = Some(derived_with_files(files));

        // Must not panic
        let html = render(&receipt);
        prop_assert!(!html.is_empty());
    }
}

// ── Property: metric-card count is 5 or 6 when derived present ──────

proptest! {
    #[test]
    fn render_metric_card_count_is_5_or_6(
        has_context in prop::bool::ANY,
        code in 1..1000_usize,
    ) {
        let mut receipt = minimal_receipt();
        let files = vec![FileStatRow {
            path: "a.rs".into(),
            module: ".".into(),
            lang: "Rust".into(),
            code,
            comments: code / 5,
            blanks: code / 10,
            lines: code + code / 5 + code / 10,
            bytes: code * 50,
            tokens: code * 3,
            doc_pct: Some(0.15),
            bytes_per_line: Some(40.0),
            depth: 0,
        }];
        let mut derived = derived_with_files(files);
        if has_context {
            derived.context_window = Some(ContextWindowReport {
                window_tokens: 128_000,
                total_tokens: 300,
                pct: 0.002,
                fits: true,
            });
        }
        receipt.derived = Some(derived);

        let html = render(&receipt);
        let card_count = html.matches(r#"class="metric-card""#).count();

        if has_context {
            prop_assert_eq!(card_count, 6);
        } else {
            prop_assert_eq!(card_count, 5);
        }
    }
}
