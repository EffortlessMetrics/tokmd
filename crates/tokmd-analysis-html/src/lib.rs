//! # tokmd-analysis-html
//!
//! **Tier 3 (Formatting Adapter)**
//!
//! Single-responsibility HTML renderer for `AnalysisReceipt`.

use time::OffsetDateTime;
use time::macros::format_description;
use tokmd_analysis_types::AnalysisReceipt;

/// Render a self-contained HTML report for an analysis receipt.
pub fn render(receipt: &AnalysisReceipt) -> String {
    const TEMPLATE: &str = include_str!("templates/report.html");

    let timestamp = timestamp_utc();
    let metrics_cards = build_metrics_cards(receipt);
    let table_rows = build_table_rows(receipt);
    let report_json = build_report_json(receipt);

    TEMPLATE
        .replace("{{TIMESTAMP}}", &timestamp)
        .replace("{{METRICS_CARDS}}", &metrics_cards)
        .replace("{{TABLE_ROWS}}", &table_rows)
        .replace("{{REPORT_JSON}}", &report_json)
}

fn timestamp_utc() -> String {
    let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second] UTC");
    OffsetDateTime::now_utc()
        .format(&format)
        .unwrap_or_else(|_| "1970-01-01 00:00:00 UTC".to_string())
}

fn build_metrics_cards(receipt: &AnalysisReceipt) -> String {
    let mut cards = String::new();

    if let Some(derived) = &receipt.derived {
        let metrics = [
            ("Files", derived.totals.files.to_string()),
            ("Lines", format_number(derived.totals.lines)),
            ("Code", format_number(derived.totals.code)),
            ("Tokens", format_number(derived.totals.tokens)),
            ("Doc%", format_pct(derived.doc_density.total.ratio)),
        ];

        for (label, value) in metrics {
            cards.push_str(&format!(
                r#"<div class="metric-card"><span class="value">{}</span><span class="label">{}</span></div>"#,
                value, label
            ));
        }

        if let Some(ctx) = &derived.context_window {
            cards.push_str(&format!(
                r#"<div class="metric-card"><span class="value">{}</span><span class="label">Context Fit</span></div>"#,
                format_pct(ctx.pct)
            ));
        }
    }

    cards
}

fn build_table_rows(receipt: &AnalysisReceipt) -> String {
    let mut rows = String::new();

    if let Some(derived) = &receipt.derived {
        for row in derived.top.largest_lines.iter().take(100) {
            rows.push_str(&format!(
                r#"<tr><td class="path" data-path="{path}">{path}</td><td data-module="{module}">{module}</td><td data-lang="{lang}"><span class="lang-badge">{lang}</span></td><td class="num" data-lines="{lines}">{lines_fmt}</td><td class="num" data-code="{code}">{code_fmt}</td><td class="num" data-tokens="{tokens}">{tokens_fmt}</td><td class="num" data-bytes="{bytes}">{bytes_fmt}</td></tr>"#,
                path = escape_html(&row.path),
                module = escape_html(&row.module),
                lang = escape_html(&row.lang),
                lines = row.lines,
                lines_fmt = format_number(row.lines),
                code = row.code,
                code_fmt = format_number(row.code),
                tokens = row.tokens,
                tokens_fmt = format_number(row.tokens),
                bytes = row.bytes,
                bytes_fmt = format_number(row.bytes),
            ));
        }
    }

    rows
}

fn build_report_json(receipt: &AnalysisReceipt) -> String {
    let mut files = Vec::new();

    if let Some(derived) = &receipt.derived {
        for row in &derived.top.largest_lines {
            files.push(serde_json::json!({
                "path": row.path,
                "module": row.module,
                "lang": row.lang,
                "code": row.code,
                "lines": row.lines,
                "tokens": row.tokens,
            }));
        }
    }

    // Escape < and > to prevent </script> breakout XSS attacks.
    // JSON remains valid because \u003c and \u003e are valid JSON string escapes.
    serde_json::json!({ "files": files })
        .to_string()
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
}

fn format_number(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn format_pct(ratio: f64) -> String {
    format!("{:.1}%", ratio * 100.0)
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
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
                format: "html".to_string(),
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

    #[test]
    fn format_number_thresholds() {
        assert_eq!(format_number(500), "500");
        assert_eq!(format_number(1_000), "1.0K");
        assert_eq!(format_number(1_500), "1.5K");
        assert_eq!(format_number(1_000_000), "1.0M");
        assert_eq!(format_number(2_500_000), "2.5M");
    }

    #[test]
    fn escape_html_encodes_special_chars() {
        assert_eq!(escape_html("hello"), "hello");
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("a & b"), "a &amp; b");
        assert_eq!(escape_html("\"quoted\""), "&quot;quoted&quot;");
        assert_eq!(escape_html("it's"), "it&#x27;s");
        assert_eq!(
            escape_html("<a href=\"test\">&'"),
            "&lt;a href=&quot;test&quot;&gt;&amp;&#x27;"
        );
    }

    #[test]
    fn timestamp_has_expected_shape() {
        let ts = timestamp_utc();
        assert!(ts.contains("UTC"));
        assert!(ts.len() > 10);
    }

    #[test]
    fn metrics_cards_empty_without_derived() {
        let receipt = minimal_receipt();
        assert!(build_metrics_cards(&receipt).is_empty());
    }

    #[test]
    fn metrics_cards_include_context_fit_when_available() {
        let mut receipt = minimal_receipt();
        receipt.derived = Some(sample_derived());
        let cards = build_metrics_cards(&receipt);
        assert!(cards.contains("class=\"metric-card\""));
        assert!(cards.contains("Context Fit"));
    }

    #[test]
    fn table_rows_are_html_escaped() {
        let mut receipt = minimal_receipt();
        let mut derived = sample_derived();
        derived.top.largest_lines[0].path = "src/<script>.rs".to_string();
        derived.top.largest_lines[0].module = "mod&name".to_string();
        derived.top.largest_lines[0].lang = "Ru\"st".to_string();
        receipt.derived = Some(derived);

        let rows = build_table_rows(&receipt);
        assert!(rows.contains("src/&lt;script&gt;.rs"));
        assert!(rows.contains("mod&amp;name"));
        assert!(rows.contains("Ru&quot;st"));
    }

    #[test]
    fn report_json_escapes_angle_brackets() {
        let mut receipt = minimal_receipt();
        let mut derived = sample_derived();
        derived.top.largest_lines[0].path = "</script><script>alert(1)</script>".to_string();
        receipt.derived = Some(derived);

        let json = build_report_json(&receipt);
        assert!(
            json.contains("\\u003c/script\\u003e\\u003cscript\\u003ealert(1)\\u003c/script\\u003e")
        );
        assert!(!json.contains('<'));
        assert!(!json.contains('>'));
    }

    #[test]
    fn report_json_without_derived_is_empty_files_array() {
        let receipt = minimal_receipt();
        assert_eq!(build_report_json(&receipt), "{\"files\":[]}");
    }

    #[test]
    fn render_inlines_template_content() {
        let mut receipt = minimal_receipt();
        receipt.derived = Some(sample_derived());

        let html = render(&receipt);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("metric-card"));
        assert!(html.contains("src/lib.rs"));
        assert!(html.contains("const REPORT_DATA ="));
    }
}
