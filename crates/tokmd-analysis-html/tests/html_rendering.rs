//! Deeper HTML rendering tests: edge cases, large datasets, unicode,
//! number formatting boundaries, and structural validation.

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

// ── Unicode in paths and modules ────────────────────────────────────

#[test]
fn unicode_path_is_rendered_safely() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row("src/日本語/模块.rs", "日本語", "Rust", 100)];
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    assert!(html.contains("日本語"));
    assert!(html.contains("模块.rs"));
    assert!(html.starts_with("<!DOCTYPE html>"));
}

#[test]
fn emoji_in_module_name_is_rendered() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row("src/🚀/launch.rs", "🚀", "Rust", 50)];
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    assert!(html.contains("🚀"));
    assert!(html.contains("launch.rs"));
}

// ── Number formatting boundaries ────────────────────────────────────

#[test]
fn code_below_1000_rendered_as_plain_number() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row("small.rs", ".", "Rust", 999)];
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    assert!(html.contains("999"), "999 should appear as plain number");
}

#[test]
fn code_at_1000_rendered_with_k_suffix() {
    let mut receipt = minimal_receipt();
    let mut derived = derived_with_files(vec![make_file_row("med.rs", ".", "Rust", 1000)]);
    derived.totals.code = 1000;
    receipt.derived = Some(derived);

    let html = render(&receipt);

    assert!(
        html.contains("1.0K"),
        "1000 should render as 1.0K in metric cards"
    );
}

#[test]
fn code_at_million_rendered_with_m_suffix() {
    let mut receipt = minimal_receipt();
    let mut derived = derived_with_files(vec![make_file_row("huge.rs", ".", "Rust", 1_000_000)]);
    derived.totals.code = 1_000_000;
    receipt.derived = Some(derived);

    let html = render(&receipt);

    assert!(
        html.contains("1.0M"),
        "1000000 should render as 1.0M in metric cards"
    );
}

#[test]
fn zero_code_rendered_as_zero() {
    let mut receipt = minimal_receipt();
    let mut derived = derived_with_files(vec![]);
    derived.totals.code = 0;
    derived.totals.files = 0;
    derived.totals.lines = 0;
    derived.totals.tokens = 0;
    receipt.derived = Some(derived);

    let html = render(&receipt);

    // The "0" value should appear in metric cards
    assert!(html.contains(">0<"), "zero should render as plain 0");
}

// ── Large dataset ───────────────────────────────────────────────────

#[test]
fn large_dataset_200_files_capped_at_100_rows() {
    let mut receipt = minimal_receipt();
    let files: Vec<FileStatRow> = (0..200)
        .map(|i| {
            make_file_row(
                &format!("src/mod_{i}/file.rs"),
                &format!("mod_{i}"),
                "Rust",
                50 + i,
            )
        })
        .collect();
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    let row_count = html.matches("<tr><td").count();
    assert!(
        row_count <= 100,
        "table should cap at 100 rows, got {row_count}"
    );
}

#[test]
fn large_dataset_produces_valid_html() {
    let mut receipt = minimal_receipt();
    let files: Vec<FileStatRow> = (0..200)
        .map(|i| make_file_row(&format!("src/f{i}.rs"), "src", "Rust", 100 + i))
        .collect();
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("</html>"));
    assert!(html.contains("<body>"));
    assert!(html.contains("</body>"));
}

#[test]
fn large_dataset_json_contains_all_files() {
    let mut receipt = minimal_receipt();
    let files: Vec<FileStatRow> = (0..200)
        .map(|i| make_file_row(&format!("src/f{i}.rs"), "src", "Rust", 100 + i))
        .collect();
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    // The JSON data should contain all 200 files (no cap on JSON)
    for i in 0..200 {
        let path = format!("src/f{i}.rs");
        assert!(
            html.contains(&path),
            "JSON should contain {path} even if table is capped"
        );
    }
}

// ── Deep nesting paths ──────────────────────────────────────────────

#[test]
fn deeply_nested_path_rendered() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row(
        "a/b/c/d/e/f/g/h/i/j/deep.rs",
        "a/b/c/d/e/f/g/h/i/j",
        "Rust",
        10,
    )];
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    assert!(html.contains("a/b/c/d/e/f/g/h/i/j/deep.rs"));
}

// ── Empty top offenders with non-empty totals ───────────────────────

#[test]
fn derived_with_empty_largest_lines_produces_no_table_rows() {
    let mut receipt = minimal_receipt();
    let mut derived = derived_with_files(vec![]);
    // Manually set totals to non-zero while keeping top offenders empty
    derived.totals.files = 5;
    derived.totals.code = 500;
    derived.totals.lines = 600;
    derived.totals.tokens = 1500;
    receipt.derived = Some(derived);

    let html = render(&receipt);

    assert_eq!(
        html.matches("<tr><td").count(),
        0,
        "no rows when largest_lines is empty"
    );
    // But metric cards should still appear
    assert!(html.contains("metric-card"));
}

// ── Template structural invariants ──────────────────────────────────

#[test]
fn rendered_html_contains_style_section() {
    let receipt = minimal_receipt();
    let html = render(&receipt);

    assert!(
        html.contains("<style>") || html.contains("<style "),
        "HTML must contain a <style> section"
    );
}

#[test]
fn rendered_html_contains_script_section() {
    let receipt = minimal_receipt();
    let html = render(&receipt);

    assert!(
        html.contains("<script>") || html.contains("<script "),
        "HTML must contain a <script> section"
    );
}

#[test]
fn rendered_html_contains_report_data_variable() {
    let receipt = minimal_receipt();
    let html = render(&receipt);

    assert!(
        html.contains("const REPORT_DATA ="),
        "must contain REPORT_DATA JS variable"
    );
}

#[test]
fn rendered_html_has_meta_charset() {
    let receipt = minimal_receipt();
    let html = render(&receipt);

    assert!(
        html.contains("charset") || html.contains("UTF-8") || html.contains("utf-8"),
        "HTML should declare character encoding"
    );
}

// ── Report JSON safety for large datasets ───────────────────────────

#[test]
fn report_json_has_no_raw_angle_brackets_in_large_dataset() {
    let mut receipt = minimal_receipt();
    let files: Vec<FileStatRow> = (0..50)
        .map(|i| make_file_row(&format!("src/f{i}.rs"), "src", "Rust", 100))
        .collect();
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    // Find the JSON data section
    if let Some(start) = html.find("const REPORT_DATA =") {
        let json_start = start + "const REPORT_DATA =".len();
        if let Some(end) = html[json_start..].find(';') {
            let json_section = &html[json_start..json_start + end];
            assert!(
                !json_section.contains('<'),
                "JSON section must not contain raw <"
            );
            assert!(
                !json_section.contains('>'),
                "JSON section must not contain raw >"
            );
        }
    }
}

// ── Special characters in all fields ────────────────────────────────

#[test]
fn ampersand_in_module_is_escaped() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row("test.rs", "a&b", "Rust", 10)];
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    assert!(html.contains("a&amp;b"), "ampersand must be escaped");
    assert!(
        !html.contains("data-module=\"a&b\""),
        "raw & must not appear in attribute"
    );
}

#[test]
fn quotes_in_lang_are_escaped() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row("test.rs", "mod", "C\"++", 10)];
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    assert!(html.contains("C&quot;++"), "quote in lang must be escaped");
}

// ── Determinism for large datasets ──────────────────────────────────

#[test]
fn large_dataset_rendering_is_deterministic() {
    let mut receipt = minimal_receipt();
    let files: Vec<FileStatRow> = (0..100)
        .map(|i| make_file_row(&format!("src/f{i}.rs"), "src", "Rust", 50 + i))
        .collect();
    receipt.derived = Some(derived_with_files(files));

    let html_a = render(&receipt);
    let html_b = render(&receipt);

    // Strip timestamps for comparison
    let strip = |h: &str| h.replace(char::is_numeric, "N");
    assert_eq!(
        strip(&html_a),
        strip(&html_b),
        "large dataset render must be deterministic"
    );
}

// ── Minimal derived data edge case ──────────────────────────────────

#[test]
fn single_file_with_zero_code_lines() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row("empty.txt", ".", "Text", 0)];
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("empty.txt"));
}

#[test]
fn single_file_with_very_large_code_count() {
    let mut receipt = minimal_receipt();
    let files = vec![make_file_row("huge.rs", ".", "Rust", 10_000_000)];
    receipt.derived = Some(derived_with_files(files));

    let html = render(&receipt);

    assert!(
        html.contains("10.0M"),
        "10M code lines should show as 10.0M"
    );
}
