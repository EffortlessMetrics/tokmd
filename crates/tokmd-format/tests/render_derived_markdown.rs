//! Focused coverage for the derived-section Markdown renderer.
//!
//! These tests exercise the `pub(super)` rendering helpers in
//! `crates/tokmd-format/src/analysis/markdown/derived.rs` indirectly via
//! the public `tokmd_format::analysis::render` entry point.
//!
//! Run with: `cargo test -p tokmd-format --test render_derived_markdown`

use tokmd_analysis_types::{
    ANALYSIS_SCHEMA_VERSION, AnalysisArgsMeta, AnalysisReceipt, AnalysisSource, BoilerplateReport,
    CocomoReport, ContextWindowReport, DerivedReport, DerivedTotals, DistributionReport,
    FileStatRow, HistogramBucket, IntegrityReport, LangPurityReport, MaxFileReport, NestingReport,
    PolyglotReport, RateReport, RateRow, RatioReport, RatioRow, ReadingTimeReport,
    TestDensityReport, TodoReport, TodoTagRow, TopOffenders,
};
use tokmd_format::analysis::{RenderedOutput, render};
use tokmd_types::{AnalysisFormat, ScanStatus, ToolInfo};

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn fixed_tool() -> ToolInfo {
    ToolInfo {
        name: "tokmd".to_string(),
        version: "0.0.0-test".to_string(),
    }
}

fn minimal_source() -> AnalysisSource {
    AnalysisSource {
        inputs: vec![".".to_string()],
        export_path: None,
        base_receipt_path: None,
        export_schema_version: None,
        export_generated_at_ms: None,
        base_signature: None,
        module_roots: vec![],
        module_depth: 1,
        children: "collapse".to_string(),
    }
}

fn minimal_args() -> AnalysisArgsMeta {
    AnalysisArgsMeta {
        preset: "receipt".to_string(),
        format: "md".to_string(),
        window_tokens: None,
        git: None,
        max_files: None,
        max_bytes: None,
        max_commits: None,
        max_commit_files: None,
        max_file_bytes: None,
        import_granularity: "module".to_string(),
    }
}

fn minimal_receipt() -> AnalysisReceipt {
    AnalysisReceipt {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: fixed_tool(),
        mode: "analyze".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        source: minimal_source(),
        args: minimal_args(),
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
        effort: None,
    }
}

fn sample_file_stat(path: &str, lang: &str) -> FileStatRow {
    FileStatRow {
        path: path.to_string(),
        module: "src".to_string(),
        lang: lang.to_string(),
        code: 250,
        comments: 50,
        blanks: 20,
        lines: 320,
        bytes: 9000,
        tokens: 640,
        doc_pct: Some(0.16),
        bytes_per_line: Some(28.13),
        depth: 1,
    }
}

/// Construct a `DerivedReport` with the minimum required fields populated.
///
/// Optional sub-reports (`todo`, `context_window`, `cocomo`) are left as
/// `None` so each test can populate only what it needs to exercise.
fn sample_derived() -> DerivedReport {
    DerivedReport {
        totals: DerivedTotals {
            files: 12,
            code: 2400,
            comments: 360,
            blanks: 240,
            lines: 3000,
            bytes: 90000,
            tokens: 6000,
        },
        doc_density: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 360,
                denominator: 2400,
                ratio: 0.15,
            },
            by_lang: vec![RatioRow {
                key: "Rust".into(),
                numerator: 360,
                denominator: 2400,
                ratio: 0.15,
            }],
            by_module: vec![],
        },
        whitespace: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 240,
                denominator: 2760,
                ratio: 0.087,
            },
            by_lang: vec![RatioRow {
                key: "Rust".into(),
                numerator: 240,
                denominator: 2760,
                ratio: 0.087,
            }],
            by_module: vec![],
        },
        verbosity: RateReport {
            total: RateRow {
                key: "total".into(),
                numerator: 90000,
                denominator: 3000,
                rate: 30.0,
            },
            by_lang: vec![RateRow {
                key: "Rust".into(),
                numerator: 90000,
                denominator: 3000,
                rate: 30.0,
            }],
            by_module: vec![],
        },
        max_file: MaxFileReport {
            overall: sample_file_stat("src/lib.rs", "Rust"),
            by_lang: vec![],
            by_module: vec![],
        },
        lang_purity: LangPurityReport { rows: vec![] },
        nesting: NestingReport {
            max: 4,
            avg: 1.75,
            by_module: vec![],
        },
        test_density: TestDensityReport {
            test_lines: 480,
            prod_lines: 2520,
            test_files: 4,
            prod_files: 8,
            ratio: 0.19,
        },
        boilerplate: BoilerplateReport {
            infra_lines: 240,
            logic_lines: 2160,
            ratio: 0.10,
            infra_langs: vec!["TOML".into()],
        },
        polyglot: PolyglotReport {
            lang_count: 2,
            entropy: 0.45,
            dominant_lang: "Rust".into(),
            dominant_lines: 2160,
            dominant_pct: 0.90,
        },
        distribution: DistributionReport {
            count: 12,
            min: 30,
            max: 600,
            mean: 250.0,
            median: 220.0,
            p90: 560.0,
            p99: 600.0,
            gini: 0.42,
        },
        histogram: vec![
            HistogramBucket {
                label: "0–100".into(),
                min: 0,
                max: Some(100),
                files: 5,
                pct: 0.42,
            },
            HistogramBucket {
                label: "101+".into(),
                min: 101,
                max: None,
                files: 7,
                pct: 0.58,
            },
        ],
        top: TopOffenders {
            largest_lines: vec![sample_file_stat("src/big.rs", "Rust")],
            largest_tokens: vec![sample_file_stat("src/tokens.rs", "Rust")],
            largest_bytes: vec![sample_file_stat("src/bytes.rs", "Rust")],
            least_documented: vec![sample_file_stat("src/undoc.rs", "Rust")],
            most_dense: vec![sample_file_stat("src/dense.rs", "Rust")],
        },
        tree: None,
        reading_time: ReadingTimeReport {
            minutes: 15.0,
            lines_per_minute: 200,
            basis_lines: 3000,
        },
        context_window: None,
        cocomo: None,
        todo: None,
        integrity: IntegrityReport {
            algo: "blake3".into(),
            hash: "deadbeefcafebabe".into(),
            entries: 12,
        },
    }
}

/// Render an `AnalysisReceipt` to a Markdown string via the public entry
/// point.
fn render_md(receipt: &AnalysisReceipt) -> String {
    match render(receipt, AnalysisFormat::Md).expect("Md rendering must succeed") {
        RenderedOutput::Text(t) => t,
        RenderedOutput::Binary(_) => panic!("Md rendering must return text"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

/// Baseline derived rendering: every always-present section is emitted with
/// expected headers and key values from `derived.totals`/`derived.ratios`.
#[test]
fn derived_baseline_emits_totals_ratios_and_tables() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let md = render_md(&receipt);

    // Totals section header and totals row.
    assert!(md.contains("## Totals"), "expected Totals header");
    // The totals row contains all seven counters from `DerivedTotals`.
    assert!(
        md.contains("|12|2400|360|240|3000|90000|6000|"),
        "expected totals row with all DerivedTotals fields, got:\n{md}"
    );

    // Ratios section with doc density, whitespace ratio, bytes/line.
    assert!(md.contains("## Ratios"), "expected Ratios header");
    assert!(md.contains("|Doc density|15.0%|"));
    assert!(md.contains("|Whitespace ratio|8.7%|"));
    assert!(md.contains("|Bytes per line|30.00|"));

    // Per-language tables.
    assert!(md.contains("### Doc density by language"));
    assert!(md.contains("|Lang|Doc%|Comments|Code|"));
    assert!(md.contains("### Whitespace ratio by language"));
    assert!(md.contains("|Lang|Blank%|Blanks|Code+Comments|"));
    assert!(md.contains("### Verbosity by language"));
    assert!(md.contains("|Lang|Bytes/Line|Bytes|Lines|"));

    // Distribution section + a recognisable distribution row.
    assert!(md.contains("## Distribution"));
    assert!(md.contains("|Count|Min|Max|Mean|Median|P90|P99|Gini|"));
    assert!(md.contains("|12|30|600|250.00|220.00|560.00|600.00|0.4200|"));

    // File size histogram, including the `∞` placeholder for `max: None`.
    assert!(md.contains("## File size histogram"));
    assert!(md.contains("|0–100|0|100|5|42.0%|"));
    assert!(md.contains("|101+|101|∞|7|58.0%|"));

    // Top offender file tables – verify both the section header set and the
    // shared file-row table header from `render_file_table`.
    assert!(md.contains("## Top offenders"));
    assert!(md.contains("### Largest files by lines"));
    assert!(md.contains("### Largest files by tokens"));
    assert!(md.contains("### Largest files by bytes"));
    assert!(md.contains("### Least documented (min LOC)"));
    assert!(md.contains("### Most dense (bytes/line)"));
    assert!(md.contains("|Path|Lang|Lines|Code|Bytes|Tokens|Doc%|B/Line|"));
    // Sample file rows from each top list.
    assert!(md.contains("|src/big.rs|Rust|"));
    assert!(md.contains("|src/tokens.rs|Rust|"));
    assert!(md.contains("|src/bytes.rs|Rust|"));
    assert!(md.contains("|src/undoc.rs|Rust|"));
    assert!(md.contains("|src/dense.rs|Rust|"));

    // Structure / nesting summary.
    assert!(md.contains("## Structure"));
    assert!(md.contains("Max depth: `4`"));
    assert!(md.contains("Avg depth: `1.75`"));

    // Test density section.
    assert!(md.contains("## Test density"));
    assert!(md.contains("Test lines: `480`"));
    assert!(md.contains("Prod lines: `2520`"));
    assert!(md.contains("Test ratio: `19.0%`"));

    // Boilerplate section.
    assert!(md.contains("## Boilerplate ratio"));
    assert!(md.contains("Infra lines: `240`"));
    assert!(md.contains("Logic lines: `2160`"));
    assert!(md.contains("Infra ratio: `10.0%`"));

    // Polyglot section.
    assert!(md.contains("## Polyglot"));
    assert!(md.contains("Languages: `2`"));
    assert!(md.contains("Dominant: `Rust` (90.0%)"));

    // Reading time.
    assert!(md.contains("## Reading time"));
    assert!(md.contains("Minutes: `15.00` (200 lines/min)"));

    // Integrity – always emitted at the tail of the derived block.
    assert!(md.contains("## Integrity"));
    assert!(md.contains("Hash: `deadbeefcafebabe` (`blake3`)"));
    assert!(md.contains("Entries: `12`"));
}

/// Optional `context_window` populates a dedicated section with utilization
/// and fit boolean fields.
#[test]
fn derived_renders_context_window_when_present() {
    let mut derived = sample_derived();
    derived.context_window = Some(ContextWindowReport {
        window_tokens: 8192,
        total_tokens: 6000,
        pct: 0.732,
        fits: true,
    });
    let mut receipt = minimal_receipt();
    receipt.derived = Some(derived);
    let md = render_md(&receipt);

    assert!(md.contains("## Context window"));
    assert!(md.contains("Window tokens: `8192`"));
    assert!(md.contains("Total tokens: `6000`"));
    assert!(md.contains("Utilization: `73.2%`"));
    assert!(md.contains("Fits: `true`"));
}

/// When `context_window` is absent the section header must not appear.
#[test]
fn derived_omits_context_window_when_absent() {
    let mut receipt = minimal_receipt();
    let mut derived = sample_derived();
    derived.context_window = None;
    receipt.derived = Some(derived);
    let md = render_md(&receipt);

    assert!(!md.contains("## Context window"));
    // Other always-present sections still render.
    assert!(md.contains("## Totals"));
    assert!(md.contains("## Integrity"));
}

/// The legacy `derived.cocomo` fallback section is rendered only when
/// `receipt.effort` is `None` but `derived.cocomo` is populated.
#[test]
fn derived_renders_legacy_cocomo_fallback_when_effort_absent() {
    let mut derived = sample_derived();
    derived.cocomo = Some(CocomoReport {
        mode: "organic".to_string(),
        kloc: 2.4,
        effort_pm: 6.5,
        duration_months: 4.0,
        staff: 1.625,
        a: 2.4,
        b: 1.05,
        c: 2.5,
        d: 0.38,
    });
    let mut receipt = minimal_receipt();
    receipt.derived = Some(derived);
    // No top-level `effort` – the legacy COCOMO path must be taken.
    receipt.effort = None;

    let md = render_md(&receipt);
    assert!(md.contains("## Effort estimate"));
    // Legacy renderer emits a "Size basis" block including KLOC.
    assert!(md.contains("### Size basis"));
    assert!(md.contains("KLOC: `2.4000`"));
    // Legacy headline numbers from the COCOMO report.
    assert!(md.contains("Effort: `6.50` person-months"));
    assert!(md.contains("Duration: `4.00` months"));
    assert!(md.contains("Staff: `1.62`"));
    // The legacy renderer documents COCOMO mode + coefficients.
    assert!(md.contains("Model: `COCOMO` (`organic` mode)"));
    assert!(md.contains("Coefficients: `a=2.40`, `b=1.05`, `c=2.50`, `d=0.38`"));
    // It also notes the absence of delta data.
    assert!(md.contains("Baseline comparison is not available for this receipt."));
}

/// When neither `derived.cocomo` nor `receipt.effort` is populated the
/// Effort estimate section is omitted entirely.
#[test]
fn derived_omits_effort_section_when_both_absent() {
    let mut derived = sample_derived();
    derived.cocomo = None;
    let mut receipt = minimal_receipt();
    receipt.derived = Some(derived);
    receipt.effort = None;

    let md = render_md(&receipt);
    assert!(!md.contains("## Effort estimate"));
    // Integrity must still follow, since it is rendered unconditionally.
    assert!(md.contains("## Integrity"));
}

/// Optional `todo` section is rendered with its summary fields and tag
/// breakdown table when present.
#[test]
fn derived_renders_todo_section_when_present() {
    let mut derived = sample_derived();
    derived.todo = Some(TodoReport {
        total: 7,
        density_per_kloc: 2.9,
        tags: vec![
            TodoTagRow {
                tag: "TODO".into(),
                count: 4,
            },
            TodoTagRow {
                tag: "FIXME".into(),
                count: 3,
            },
        ],
    });
    let mut receipt = minimal_receipt();
    receipt.derived = Some(derived);
    let md = render_md(&receipt);

    assert!(md.contains("## TODOs"));
    assert!(md.contains("Total: `7`"));
    assert!(md.contains("Density (per KLOC): `2.90`"));
    assert!(md.contains("|TODO|4|"));
    assert!(md.contains("|FIXME|3|"));
}

/// File-row tables emit `-` placeholders when a row's optional doc/bytes
/// metrics are `None`, exercising the `Option` branch in `render_file_table`.
#[test]
fn derived_file_table_renders_dash_for_missing_optional_metrics() {
    let mut derived = sample_derived();
    // Force every top-offender row in `largest_lines` to have unset
    // `doc_pct` and `bytes_per_line` so the dash branch is exercised.
    let row_no_doc = FileStatRow {
        path: "src/nodoc.rs".into(),
        module: "src".into(),
        lang: "Rust".into(),
        code: 80,
        comments: 0,
        blanks: 5,
        lines: 85,
        bytes: 2000,
        tokens: 200,
        doc_pct: None,
        bytes_per_line: None,
        depth: 1,
    };
    derived.top.largest_lines = vec![row_no_doc];
    let mut receipt = minimal_receipt();
    receipt.derived = Some(derived);
    let md = render_md(&receipt);

    // The dash placeholder appears in the optional Doc% / B/Line columns.
    assert!(
        md.contains("|src/nodoc.rs|Rust|85|80|2000|200|-|-|"),
        "expected dashes for missing optional metrics, got:\n{md}"
    );
}
