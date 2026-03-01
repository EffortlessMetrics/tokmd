//! Snapshot tests for HTML report output using `insta`.
//!
//! These tests capture the structure of rendered HTML output so that
//! unintentional changes to the HTML template or rendering logic are detected.
//! Timestamps are redacted for determinism.

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

/// Strip the dynamic timestamp so snapshots are deterministic.
fn redact_timestamp(html: &str) -> String {
    // Timestamp format: "YYYY-MM-DD HH:MM:SS UTC"
    // Find and replace all occurrences of the pattern
    let mut result = html.to_string();
    while let Some(pos) = result.find(" UTC") {
        // Walk back to find the start of the timestamp (19 chars: "YYYY-MM-DD HH:MM:SS")
        if pos >= 19 {
            let candidate = &result[pos - 19..pos + 4]; // "YYYY-MM-DD HH:MM:SS UTC"
            if candidate.len() == 23
                && candidate.as_bytes()[4] == b'-'
                && candidate.as_bytes()[7] == b'-'
                && candidate.as_bytes()[10] == b' '
            {
                result.replace_range(pos - 19..pos + 4, "[TIMESTAMP]");
                continue;
            }
        }
        // If we can't match, break to avoid infinite loop
        break;
    }
    result
}

// ── Snapshot: Empty receipt ──────────────────────────────────────────

#[test]
fn snapshot_empty_receipt_metrics_section() {
    let receipt = minimal_receipt();
    let html = render(&receipt);
    let html = redact_timestamp(&html);

    // Snapshot just the metrics cards region (empty)
    // Extract a focused slice around the metrics grid
    let metrics_start = html.find("metrics-grid").unwrap_or(0);
    let metrics_end = html[metrics_start..]
        .find("</div>")
        .map(|i| metrics_start + i + 6)
        .unwrap_or(metrics_start + 200);
    let section = &html[metrics_start..metrics_end.min(html.len())];

    insta::assert_snapshot!("empty_receipt_metrics", section);
}

#[test]
fn snapshot_empty_receipt_json() {
    let receipt = minimal_receipt();
    let html = render(&receipt);

    // Extract the REPORT_DATA JSON
    if let Some(start) = html.find("const REPORT_DATA =") {
        let json_start = start + "const REPORT_DATA =".len();
        if let Some(end) = html[json_start..].find(';') {
            let json_section = html[json_start..json_start + end].trim();
            insta::assert_snapshot!("empty_receipt_json", json_section);
        }
    }
}

// ── Snapshot: Single file receipt ───────────────────────────────────

#[test]
fn snapshot_single_file_table_rows() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row("src/main.rs", "src", "Rust", 250)];
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    // Extract all <tr> rows from the table
    let rows: Vec<&str> = html
        .split("<tr>")
        .skip(1) // skip content before first <tr>
        .filter(|s| s.contains("<td"))
        .collect();

    let rows_joined = rows.join("\n---\n");
    insta::assert_snapshot!("single_file_table_rows", rows_joined);
}

#[test]
fn snapshot_single_file_json_data() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row("src/main.rs", "src", "Rust", 250)];
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    if let Some(start) = html.find("const REPORT_DATA =") {
        let json_start = start + "const REPORT_DATA =".len();
        if let Some(end) = html[json_start..].find(';') {
            let json_str = html[json_start..json_start + end].trim();
            // Pretty-print the JSON for readable snapshot
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
                let pretty = serde_json::to_string_pretty(&val).unwrap();
                insta::assert_snapshot!("single_file_json_data", pretty);
            }
        }
    }
}

// ── Snapshot: Multi-language receipt ─────────────────────────────────

#[test]
fn snapshot_multi_language_table_rows() {
    let mut receipt = minimal_receipt();
    let files = vec![
        make_file_row("src/main.rs", "src", "Rust", 500),
        make_file_row("src/utils.py", "src", "Python", 300),
        make_file_row("web/app.ts", "web", "TypeScript", 200),
    ];
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    let rows: Vec<&str> = html
        .split("<tr>")
        .skip(1)
        .filter(|s| s.contains("<td"))
        .collect();

    let rows_joined = rows.join("\n---\n");
    insta::assert_snapshot!("multi_language_table_rows", rows_joined);
}

// ── Snapshot: XSS payload ───────────────────────────────────────────

#[test]
fn snapshot_xss_escaped_row() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row(
        "<script>alert('xss')</script>",
        "evil&mod",
        "Lang\"quoted",
        42,
    )];
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    let rows: Vec<&str> = html
        .split("<tr>")
        .skip(1)
        .filter(|s| s.contains("<td"))
        .collect();

    let rows_joined = rows.join("\n---\n");
    insta::assert_snapshot!("xss_escaped_row", rows_joined);
}

// ── Snapshot: Empty receipt full structure ───────────────────────────

#[test]
fn snapshot_empty_receipt_html_structure() {
    let receipt = minimal_receipt();
    let html = render(&receipt);
    let html = redact_timestamp(&html);

    // Snapshot the head section to detect template changes
    let head_start = html.find("<head>").unwrap();
    let head_end = html.find("</head>").unwrap() + "</head>".len();
    let head_section = &html[head_start..head_end.min(html.len())];

    // Just snapshot a stable structural marker: the title
    let title_start = head_section.find("<title>").unwrap();
    let title_end = head_section.find("</title>").unwrap() + "</title>".len();
    let title = &head_section[title_start..title_end];
    insta::assert_snapshot!("empty_receipt_title", title);
}

// ── Snapshot: Metric cards with derived data ────────────────────────

#[test]
fn snapshot_single_file_metrics_cards() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row("src/main.rs", "src", "Rust", 250)];
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);
    let html = redact_timestamp(&html);

    // Extract metric cards by finding the actual card elements
    let cards: Vec<&str> = html.split("class=\"metric-card\"").skip(1).collect();

    let card_labels: Vec<String> = cards
        .iter()
        .filter_map(|c| {
            let val_start = c.find("class=\"value\">")? + "class=\"value\">".len();
            let val_end = c[val_start..].find("</span>")? + val_start;
            let lbl_start = c.find("class=\"label\">")? + "class=\"label\">".len();
            let lbl_end = c[lbl_start..].find("</span>")? + lbl_start;
            Some(format!(
                "{}: {}",
                &c[lbl_start..lbl_end],
                &c[val_start..val_end]
            ))
        })
        .collect();

    let summary = card_labels.join("\n");
    insta::assert_snapshot!("single_file_metrics_cards", summary);
}

// ── Snapshot: Empty files derived data ──────────────────────────────

#[test]
fn snapshot_empty_files_json_data() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(derived_with_files(vec![]));

    let html = render(&receipt);

    if let Some(start) = html.find("const REPORT_DATA =") {
        let json_start = start + "const REPORT_DATA =".len();
        if let Some(end) = html[json_start..].find(';') {
            let json_str = html[json_start..json_start + end].trim();
            insta::assert_snapshot!("empty_files_json_data", json_str);
        }
    }
}
