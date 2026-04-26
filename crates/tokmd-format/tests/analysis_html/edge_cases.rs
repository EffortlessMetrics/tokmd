//! Additional edge-case tests and snapshot tests for tokmd-format analysis HTML.
//!
//! Covers boundary conditions, unusual inputs, and structural invariants
//! not exercised by the existing test suites.

use tokmd_analysis_types::*;
use tokmd_format::analysis::html::render;

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

/// Strip the dynamic timestamp so snapshots are deterministic.
fn redact_timestamp(html: &str) -> String {
    let mut result = html.to_string();
    while let Some(pos) = result.find(" UTC") {
        if pos >= 19 {
            let candidate = &result[pos - 19..pos + 4];
            if candidate.len() == 23
                && candidate.as_bytes()[4] == b'-'
                && candidate.as_bytes()[7] == b'-'
                && candidate.as_bytes()[10] == b' '
            {
                result.replace_range(pos - 19..pos + 4, "[TIMESTAMP]");
                continue;
            }
        }
        break;
    }
    result
}

// ═══════════════════════════════════════════════════════════════════
// Edge case: derived is Some but with empty file list
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_derived_with_zero_files_when_rendered_then_no_data_rows() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(derived_with_files(vec![]));
    let html = render(&receipt);

    assert!(
        !html.contains("<tr><td"),
        "no data rows for empty file list"
    );
    assert!(
        html.contains("metric-card"),
        "metric cards should still render"
    );
}

#[test]
fn given_derived_with_zero_files_when_rendered_then_report_json_has_empty_files() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(derived_with_files(vec![]));
    let html = render(&receipt);

    assert!(html.contains(r#"{"files":[]}"#));
}

// ═══════════════════════════════════════════════════════════════════
// Edge case: very long path
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_very_long_path_when_rendered_then_does_not_panic() {
    let long_path = "a/".repeat(500) + "very_long_file.rs";
    let mut receipt = minimal_receipt();
    receipt.derived = Some(derived_with_files(vec![make_file_row(
        &long_path, "deep", "Rust", 10,
    )]));

    let html = render(&receipt);
    assert!(html.contains("very_long_file.rs"));
}

// ═══════════════════════════════════════════════════════════════════
// Edge case: path with control characters (tabs, newlines)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_path_with_tab_when_rendered_then_does_not_panic() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(derived_with_files(vec![make_file_row(
        "src/\tfile.rs",
        "src",
        "Rust",
        10,
    )]));

    let html = render(&receipt);
    assert!(html.contains("<!DOCTYPE html>"));
}

#[test]
fn given_path_with_newline_when_rendered_then_does_not_panic() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(derived_with_files(vec![make_file_row(
        "src/\nfile.rs",
        "src",
        "Rust",
        10,
    )]));

    let html = render(&receipt);
    assert!(html.contains("<!DOCTYPE html>"));
}

// ═══════════════════════════════════════════════════════════════════
// Edge case: all five HTML-special chars combined
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_all_html_special_chars_when_rendered_then_all_escaped() {
    let nasty = r#"<>&"'"#;
    let mut receipt = minimal_receipt();
    receipt.derived = Some(derived_with_files(vec![make_file_row(
        nasty, nasty, nasty, 5,
    )]));

    let html = render(&receipt);

    // The raw combination must NOT appear in table cells
    assert!(
        !html.contains(r#"<>&"'"#),
        "raw special chars must not appear in HTML"
    );
    // Escaped versions must be present somewhere in the table
    assert!(html.contains("&lt;"));
    assert!(html.contains("&gt;"));
    assert!(html.contains("&amp;"));
    assert!(html.contains("&quot;"));
    assert!(html.contains("&#x27;"));
}

// ═══════════════════════════════════════════════════════════════════
// Edge case: exact numeric boundary values in metric cards
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_999_tokens_when_rendered_then_no_suffix() {
    let mut receipt = minimal_receipt();
    let mut derived = derived_with_files(vec![make_file_row("a.rs", ".", "Rust", 1)]);
    derived.totals.tokens = 999;
    receipt.derived = Some(derived);
    let html = render(&receipt);

    // The tokens metric card should show "999"
    assert!(
        html.contains(">999<"),
        "999 tokens should not have K suffix"
    );
}

#[test]
fn given_1000_tokens_when_rendered_then_k_suffix() {
    let mut receipt = minimal_receipt();
    let mut derived = derived_with_files(vec![make_file_row("a.rs", ".", "Rust", 1)]);
    derived.totals.tokens = 1000;
    receipt.derived = Some(derived);
    let html = render(&receipt);

    assert!(html.contains("1.0K"));
}

#[test]
fn given_999999_lines_when_rendered_then_k_suffix() {
    let mut receipt = minimal_receipt();
    let mut derived = derived_with_files(vec![make_file_row("a.rs", ".", "Rust", 1)]);
    derived.totals.lines = 999_999;
    receipt.derived = Some(derived);
    let html = render(&receipt);

    assert!(html.contains("1000.0K") || html.contains("999.9K") || html.contains("K"));
}

#[test]
fn given_1000000_code_when_rendered_then_m_suffix() {
    let mut receipt = minimal_receipt();
    let mut derived = derived_with_files(vec![make_file_row("a.rs", ".", "Rust", 1)]);
    derived.totals.code = 1_000_000;
    receipt.derived = Some(derived);
    let html = render(&receipt);

    assert!(html.contains("1.0M"));
}

#[test]
fn given_zero_totals_when_rendered_then_shows_zero() {
    let mut receipt = minimal_receipt();
    let mut derived = derived_with_files(vec![]);
    derived.totals.files = 0;
    derived.totals.code = 0;
    derived.totals.lines = 0;
    derived.totals.tokens = 0;
    receipt.derived = Some(derived);
    let html = render(&receipt);

    // All zero values should render as "0"
    let zero_cards = html.matches(r#"<span class="value">0</span>"#).count();
    assert!(
        zero_cards >= 3,
        "should have at least 3 zero-value cards, got {}",
        zero_cards
    );
}

// ═══════════════════════════════════════════════════════════════════
// Edge case: ratio values at boundaries (0.0 and 1.0)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_zero_doc_density_when_rendered_then_shows_zero_pct() {
    let mut receipt = minimal_receipt();
    let mut derived = derived_with_files(vec![make_file_row("a.rs", ".", "Rust", 100)]);
    derived.doc_density.total.ratio = 0.0;
    receipt.derived = Some(derived);
    let html = render(&receipt);

    assert!(html.contains("0.0%"), "should show 0.0% for zero ratio");
}

#[test]
fn given_full_doc_density_when_rendered_then_shows_100_pct() {
    let mut receipt = minimal_receipt();
    let mut derived = derived_with_files(vec![make_file_row("a.rs", ".", "Rust", 100)]);
    derived.doc_density.total.ratio = 1.0;
    receipt.derived = Some(derived);
    let html = render(&receipt);

    assert!(html.contains("100.0%"));
}

// ═══════════════════════════════════════════════════════════════════
// Edge case: deeply nested module paths
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_deeply_nested_path_when_rendered_then_full_path_in_table() {
    let deep_path = "src/a/b/c/d/e/f/g/h/i/j/k/lib.rs";
    let mut receipt = minimal_receipt();
    receipt.derived = Some(derived_with_files(vec![make_file_row(
        deep_path,
        "src/a/b/c",
        "Rust",
        100,
    )]));
    let html = render(&receipt);

    assert!(
        html.contains(deep_path),
        "deeply nested path should appear in table"
    );
    assert!(html.contains(r#"data-module="src/a/b/c""#));
}

// ═══════════════════════════════════════════════════════════════════
// Edge case: duplicate file paths in input
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_duplicate_paths_when_rendered_then_all_rows_appear() {
    let mut receipt = minimal_receipt();
    let files = vec![
        make_file_row("src/lib.rs", "src", "Rust", 100),
        make_file_row("src/lib.rs", "src", "Rust", 200),
    ];
    receipt.derived = Some(derived_with_files(files));
    let html = render(&receipt);

    let count = html.matches(r#"data-path="src/lib.rs""#).count();
    assert_eq!(count, 2, "both rows should appear even with same path");
}

// ═══════════════════════════════════════════════════════════════════
// Edge case: file with zero code lines
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_file_with_zero_code_when_rendered_then_shows_zero_in_table() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(derived_with_files(vec![make_file_row(
        "empty.rs", ".", "Rust", 0,
    )]));
    let html = render(&receipt);

    assert!(
        html.contains(r#"data-code="0""#),
        "zero-code file should have data-code=\"0\""
    );
}

// ═══════════════════════════════════════════════════════════════════
// Snapshot: metrics section with context window
// ═══════════════════════════════════════════════════════════════════

#[test]
fn snapshot_metrics_with_context_window() {
    let mut receipt = minimal_receipt();
    let mut derived = derived_with_files(vec![make_file_row("src/main.rs", "src", "Rust", 500)]);
    derived.context_window = Some(ContextWindowReport {
        window_tokens: 128_000,
        total_tokens: 1500,
        pct: 0.0117,
        fits: true,
    });
    receipt.derived = Some(derived);
    let html = render(&receipt);
    let html = redact_timestamp(&html);

    let metrics_start = html.find("metrics-grid").unwrap_or(0);
    let search_region = &html[metrics_start..];
    // Find the end of the metrics-grid div (after all metric-card divs)
    let end_marker = search_region.find("</div>\n").unwrap_or(200) + 6;
    let section = &search_region[..end_marker.min(search_region.len())];

    insta::assert_snapshot!("metrics_with_context_window", section);
}

// ═══════════════════════════════════════════════════════════════════
// Snapshot: empty derived (Some but no files)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn snapshot_derived_empty_files_json() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(derived_with_files(vec![]));
    let html = render(&receipt);

    if let Some(start) = html.find("const REPORT_DATA =") {
        let json_start = start + "const REPORT_DATA =".len();
        if let Some(end) = html[json_start..].find(';') {
            let json_str = html[json_start..json_start + end].trim();
            insta::assert_snapshot!("derived_empty_files_json", json_str);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Snapshot: unicode and special-char paths in JSON
// ═══════════════════════════════════════════════════════════════════

#[test]
fn snapshot_unicode_paths_json() {
    let mut receipt = minimal_receipt();
    let files = vec![
        make_file_row("src/日本語/ファイル.rs", "src/日本語", "Rust", 100),
        make_file_row("src/émojis/🦀.rs", "src/émojis", "Rust", 50),
    ];
    receipt.derived = Some(derived_with_files(files));
    let html = render(&receipt);

    if let Some(start) = html.find("const REPORT_DATA =") {
        let json_start = start + "const REPORT_DATA =".len();
        if let Some(end) = html[json_start..].find(';') {
            let json_str = html[json_start..json_start + end].trim();
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
                let pretty = serde_json::to_string_pretty(&val).unwrap();
                insta::assert_snapshot!("unicode_paths_json", pretty);
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Structural: script tag is properly closed
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_any_receipt_when_rendered_then_script_tag_is_closed() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(derived_with_files(vec![make_file_row(
        "a.rs", ".", "Rust", 10,
    )]));
    let html = render(&receipt);

    let open_scripts = html.matches("<script>").count();
    let close_scripts = html.matches("</script>").count();
    assert_eq!(
        open_scripts, close_scripts,
        "script tags must be balanced: {open_scripts} open vs {close_scripts} close"
    );
}

// ═══════════════════════════════════════════════════════════════════
// Structural: footer always present
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_any_receipt_when_rendered_then_footer_present() {
    let receipt = minimal_receipt();
    let html = render(&receipt);

    assert!(html.contains("<footer>"));
    assert!(html.contains("</footer>"));
    assert!(html.contains("tokmd"), "footer should credit tokmd");
}

// ═══════════════════════════════════════════════════════════════════
// Structural: search box always present when table is rendered
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_files_when_rendered_then_search_box_present() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(derived_with_files(vec![make_file_row(
        "a.rs", ".", "Rust", 10,
    )]));
    let html = render(&receipt);

    assert!(html.contains("search-box"), "search box should be present");
    assert!(
        html.contains(r#"id="search""#),
        "search input should have id"
    );
}

// ═══════════════════════════════════════════════════════════════════
// Structural: CSS is embedded (self-contained HTML)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_any_receipt_when_rendered_then_css_is_inline() {
    let html = render(&minimal_receipt());
    assert!(
        html.contains("<style>"),
        "CSS should be inline in <style> tag"
    );
    assert!(html.contains("--bg-primary"), "CSS variables should exist");
}

// ═══════════════════════════════════════════════════════════════════
// Structural: JavaScript treemap code is embedded
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_any_receipt_when_rendered_then_js_is_inline() {
    let html = render(&minimal_receipt());
    assert!(html.contains("squarify"), "treemap JS should be embedded");
    assert!(
        html.contains("LANG_COLORS"),
        "language color map should be embedded"
    );
}

// ═══════════════════════════════════════════════════════════════════
// Edge case: exact boundary at 100 files (not over, not under)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_exactly_100_files_when_rendered_then_exactly_100_rows() {
    let mut receipt = minimal_receipt();
    let files: Vec<FileStatRow> = (0..100)
        .map(|i| make_file_row(&format!("src/file_{i}.rs"), "src", "Rust", 10 + i))
        .collect();
    receipt.derived = Some(derived_with_files(files));
    let html = render(&receipt);

    let row_count = html.matches("<tr><td").count();
    assert_eq!(row_count, 100, "exactly 100 files → exactly 100 rows");
}

#[test]
fn given_101_files_when_rendered_then_exactly_100_rows() {
    let mut receipt = minimal_receipt();
    let files: Vec<FileStatRow> = (0..101)
        .map(|i| make_file_row(&format!("src/file_{i}.rs"), "src", "Rust", 10 + i))
        .collect();
    receipt.derived = Some(derived_with_files(files));
    let html = render(&receipt);

    let row_count = html.matches("<tr><td").count();
    assert_eq!(row_count, 100, "101 files → still 100 rows (capped)");
}

// ═══════════════════════════════════════════════════════════════════
// Edge case: JSON report data is valid JSON
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_files_when_rendered_then_report_json_is_valid() {
    let mut receipt = minimal_receipt();
    let files = vec![
        make_file_row("src/lib.rs", "src", "Rust", 500),
        make_file_row("tests/test.rs", "tests", "Rust", 100),
    ];
    receipt.derived = Some(derived_with_files(files));
    let html = render(&receipt);

    if let Some(start) = html.find("const REPORT_DATA =") {
        let json_start = start + "const REPORT_DATA =".len();
        if let Some(end) = html[json_start..].find(';') {
            let json_str = html[json_start..json_start + end].trim();
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(json_str);
            assert!(
                parsed.is_ok(),
                "REPORT_DATA must be valid JSON: {:?}",
                parsed.err()
            );

            let val = parsed.unwrap();
            assert!(val.get("files").is_some(), "JSON must have 'files' key");
            assert!(val["files"].is_array(), "'files' must be an array");
            assert_eq!(
                val["files"].as_array().unwrap().len(),
                2,
                "should have 2 file entries"
            );
        }
    }
}

#[test]
fn given_empty_receipt_when_rendered_then_report_json_is_valid() {
    let receipt = minimal_receipt();
    let html = render(&receipt);

    if let Some(start) = html.find("const REPORT_DATA =") {
        let json_start = start + "const REPORT_DATA =".len();
        if let Some(end) = html[json_start..].find(';') {
            let json_str = html[json_start..json_start + end].trim();
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(json_str);
            assert!(parsed.is_ok(), "empty REPORT_DATA must still be valid JSON");
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Edge case: JSON file entries have expected fields
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_file_when_rendered_then_json_entries_have_required_fields() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row("src/lib.rs", "src", "Rust", 42)];
    receipt.derived = Some(derived_with_files(files));
    let html = render(&receipt);

    if let Some(start) = html.find("const REPORT_DATA =") {
        let json_start = start + "const REPORT_DATA =".len();
        if let Some(end) = html[json_start..].find(';') {
            let json_str = html[json_start..json_start + end].trim();
            let val: serde_json::Value = serde_json::from_str(json_str).unwrap();
            let entry = &val["files"][0];

            assert!(entry.get("path").is_some(), "must have path");
            assert!(entry.get("module").is_some(), "must have module");
            assert!(entry.get("lang").is_some(), "must have lang");
            assert!(entry.get("code").is_some(), "must have code");
            assert!(entry.get("lines").is_some(), "must have lines");
            assert!(entry.get("tokens").is_some(), "must have tokens");
        }
    }
}
