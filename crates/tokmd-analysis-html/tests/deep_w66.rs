//! W66 deep tests for `tokmd-analysis-html`.
//!
//! Exercises HTML rendering correctness, escaping edge cases,
//! empty/minimal input handling, and determinism.

use tokmd_analysis_html::render;
use tokmd_analysis_types::*;

// ── Helpers ─────────────────────────────────────────────────────

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

// ── HTML structure ──────────────────────────────────────────────

mod html_structure_w66 {
    use super::*;

    #[test]
    fn render_without_derived_produces_valid_html() {
        let html = render(&minimal_receipt());
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("</html>"));
    }

    #[test]
    fn render_with_empty_files_list() {
        let mut receipt = minimal_receipt();
        receipt.derived = Some(derived_with_files(vec![]));
        let html = render(&receipt);
        assert!(html.contains("<!DOCTYPE html>"));
        assert_eq!(html.matches("<tr><td").count(), 0);
    }

    #[test]
    fn render_includes_style_tag() {
        let html = render(&minimal_receipt());
        assert!(html.contains("<style>") || html.contains("<style "));
    }

    #[test]
    fn render_includes_script_tag() {
        let html = render(&minimal_receipt());
        assert!(html.contains("<script>") || html.contains("<script "));
    }

    #[test]
    fn render_json_section_is_valid() {
        let mut receipt = minimal_receipt();
        receipt.derived = Some(derived_with_files(vec![make_file_row(
            "a.rs", "src", "Rust", 100,
        )]));
        let html = render(&receipt);
        let marker = "const REPORT_DATA = ";
        if let Some(start) = html.find(marker) {
            let rest = &html[start + marker.len()..];
            if let Some(end) = rest.find(";\n") {
                let json_str = &rest[..end];
                let parsed: Result<serde_json::Value, _> = serde_json::from_str(json_str);
                assert!(parsed.is_ok(), "REPORT_DATA should be valid JSON");
            }
        }
    }
}

// ── Escaping edge cases ─────────────────────────────────────────

mod escaping_w66 {
    use super::*;

    #[test]
    fn script_tag_in_path_escaped() {
        let mut receipt = minimal_receipt();
        let files = vec![make_file_row("<script>alert(1)</script>", "src", "JS", 10)];
        receipt.derived = Some(derived_with_files(files));
        let html = render(&receipt);
        assert!(!html.contains("<script>alert(1)"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn ampersand_in_all_fields_escaped() {
        let mut receipt = minimal_receipt();
        let files = vec![make_file_row("a&b.rs", "c&d", "R&S", 10)];
        receipt.derived = Some(derived_with_files(files));
        let html = render(&receipt);
        assert!(html.contains("a&amp;b.rs"));
        assert!(html.contains("c&amp;d"));
        assert!(html.contains("R&amp;S"));
    }

    #[test]
    fn double_quote_in_path_escaped() {
        let mut receipt = minimal_receipt();
        let files = vec![make_file_row("file\"name.rs", "src", "Rust", 10)];
        receipt.derived = Some(derived_with_files(files));
        let html = render(&receipt);
        assert!(html.contains("file&quot;name.rs"));
    }

    #[test]
    fn single_quote_in_lang_escaped() {
        let mut receipt = minimal_receipt();
        let files = vec![make_file_row("a.rs", "src", "C'lang", 10)];
        receipt.derived = Some(derived_with_files(files));
        let html = render(&receipt);
        assert!(html.contains("C&#x27;lang"));
    }

    #[test]
    fn json_section_no_raw_angle_brackets() {
        let mut receipt = minimal_receipt();
        let files = vec![make_file_row("</script>", "src", "JS", 5)];
        receipt.derived = Some(derived_with_files(files));
        let html = render(&receipt);
        let marker = "REPORT_DATA";
        let json_start = html.find(marker).unwrap();
        let json_section = &html[json_start..];
        let json_end = json_section.find(';').unwrap_or(json_section.len());
        let json_data = &json_section[..json_end];
        assert!(!json_data.contains("</script>"));
    }
}

// ── Empty/minimal input ─────────────────────────────────────────

mod empty_input_w66 {
    use super::*;

    #[test]
    fn no_metric_cards_without_derived() {
        let html = render(&minimal_receipt());
        assert_eq!(html.matches(r#"class="metric-card"><span"#).count(), 0);
    }

    #[test]
    fn no_table_rows_without_derived() {
        let html = render(&minimal_receipt());
        assert_eq!(html.matches("<tr><td").count(), 0);
    }

    #[test]
    fn empty_json_files_without_derived() {
        let html = render(&minimal_receipt());
        assert!(html.contains("{\"files\":[]}"));
    }

    #[test]
    fn single_file_produces_one_row() {
        let mut receipt = minimal_receipt();
        let files = vec![make_file_row("main.rs", ".", "Rust", 50)];
        receipt.derived = Some(derived_with_files(files));
        let html = render(&receipt);
        assert_eq!(html.matches("<tr><td").count(), 1);
    }

    #[test]
    fn zero_code_renders_without_panic() {
        let mut receipt = minimal_receipt();
        let files = vec![make_file_row("empty.rs", ".", "Rust", 0)];
        receipt.derived = Some(derived_with_files(files));
        let html = render(&receipt);
        assert!(html.contains("<!DOCTYPE html>"));
    }
}

// ── Numeric formatting ──────────────────────────────────────────

mod formatting_w66 {
    use super::*;

    #[test]
    fn small_numbers_no_suffix() {
        let mut receipt = minimal_receipt();
        let mut derived = derived_with_files(vec![make_file_row("a.rs", ".", "Rust", 1)]);
        derived.totals.lines = 42;
        receipt.derived = Some(derived);
        let html = render(&receipt);
        assert!(html.contains(">42<"));
    }

    #[test]
    fn thousands_use_k_suffix() {
        let mut receipt = minimal_receipt();
        let mut derived = derived_with_files(vec![make_file_row("a.rs", ".", "Rust", 1)]);
        derived.totals.lines = 5500;
        receipt.derived = Some(derived);
        let html = render(&receipt);
        assert!(html.contains("5.5K"));
    }

    #[test]
    fn millions_use_m_suffix() {
        let mut receipt = minimal_receipt();
        let mut derived = derived_with_files(vec![make_file_row("a.rs", ".", "Rust", 1)]);
        derived.totals.lines = 2_500_000;
        receipt.derived = Some(derived);
        let html = render(&receipt);
        assert!(html.contains("2.5M"));
    }

    #[test]
    fn doc_pct_formatted_as_percentage() {
        let mut receipt = minimal_receipt();
        receipt.derived = Some(derived_with_files(vec![make_file_row(
            "a.rs", ".", "Rust", 100,
        )]));
        let html = render(&receipt);
        assert!(html.contains("%"));
    }
}

// ── Determinism ─────────────────────────────────────────────────

mod determinism_w66 {
    use super::*;

    #[test]
    fn render_deterministic_excluding_timestamp() {
        let mut receipt = minimal_receipt();
        let files = vec![
            make_file_row("a.rs", "src", "Rust", 100),
            make_file_row("b.py", "lib", "Python", 50),
        ];
        receipt.derived = Some(derived_with_files(files));

        let html1 = render(&receipt);
        let html2 = render(&receipt);

        let strip_ts = |s: &str| -> String {
            if let Some(start) = s.find(" UTC") {
                let region_start = s[..start].rfind("20").unwrap_or(start);
                format!("{}{}", &s[..region_start], &s[start + 4..])
            } else {
                s.to_string()
            }
        };
        assert_eq!(strip_ts(&html1), strip_ts(&html2));
    }

    #[test]
    fn render_same_receipt_twice_no_panic() {
        let mut receipt = minimal_receipt();
        receipt.derived = Some(derived_with_files(vec![make_file_row(
            "a.rs", ".", "Rust", 10,
        )]));
        let _ = render(&receipt);
        let _ = render(&receipt);
    }
}
