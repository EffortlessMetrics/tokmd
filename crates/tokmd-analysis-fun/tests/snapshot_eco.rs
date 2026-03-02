//! Snapshot tests for the eco-label generator across all grade bands.
//!
//! Each test constructs a `DerivedReport` with a specific byte size to
//! trigger each eco-label band (A–E) and pins the resulting JSON as an
//! insta snapshot.

use tokmd_analysis_fun::build_fun_report;
use tokmd_analysis_types::{
    BoilerplateReport, DerivedReport, DerivedTotals, DistributionReport, FileStatRow,
    IntegrityReport, LangPurityReport, MaxFileReport, NestingReport, PolyglotReport, RateReport,
    RateRow, RatioReport, RatioRow, ReadingTimeReport, TestDensityReport, TodoReport, TopOffenders,
};

fn derived_with_bytes(bytes: usize) -> DerivedReport {
    let zero_row = FileStatRow {
        path: "file.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        code: 0,
        comments: 0,
        blanks: 0,
        lines: 0,
        bytes,
        tokens: 0,
        doc_pct: Some(0.0),
        bytes_per_line: Some(0.0),
        depth: 0,
    };

    DerivedReport {
        totals: DerivedTotals {
            files: 1,
            code: 1,
            comments: 0,
            blanks: 0,
            lines: 1,
            bytes,
            tokens: 1,
        },
        doc_density: RatioReport {
            total: RatioRow {
                key: "All".to_string(),
                numerator: 0,
                denominator: 1,
                ratio: 0.0,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        whitespace: RatioReport {
            total: RatioRow {
                key: "All".to_string(),
                numerator: 0,
                denominator: 1,
                ratio: 0.0,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        verbosity: RateReport {
            total: RateRow {
                key: "All".to_string(),
                numerator: 0,
                denominator: 1,
                rate: 0.0,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        max_file: MaxFileReport {
            overall: zero_row.clone(),
            by_lang: vec![],
            by_module: vec![],
        },
        lang_purity: LangPurityReport { rows: vec![] },
        nesting: NestingReport {
            max: 0,
            avg: 0.0,
            by_module: vec![],
        },
        test_density: TestDensityReport {
            test_lines: 0,
            prod_lines: 0,
            test_files: 0,
            prod_files: 0,
            ratio: 0.0,
        },
        boilerplate: BoilerplateReport {
            infra_lines: 0,
            logic_lines: 0,
            ratio: 0.0,
            infra_langs: vec![],
        },
        polyglot: PolyglotReport {
            lang_count: 0,
            entropy: 0.0,
            dominant_lang: "unknown".to_string(),
            dominant_lines: 0,
            dominant_pct: 0.0,
        },
        distribution: DistributionReport {
            count: 1,
            min: 1,
            max: 1,
            mean: 0.0,
            median: 0.0,
            p90: 0.0,
            p99: 0.0,
            gini: 0.0,
        },
        histogram: Vec::new(),
        top: TopOffenders {
            largest_lines: vec![zero_row.clone()],
            largest_tokens: vec![zero_row.clone()],
            largest_bytes: vec![zero_row.clone()],
            least_documented: vec![zero_row.clone()],
            most_dense: vec![zero_row],
        },
        tree: None,
        reading_time: ReadingTimeReport {
            minutes: 0.0,
            lines_per_minute: 0,
            basis_lines: 0,
        },
        context_window: None,
        cocomo: None,
        todo: Some(TodoReport {
            total: 0,
            density_per_kloc: 0.0,
            tags: vec![],
        }),
        integrity: IntegrityReport {
            algo: "sha1".to_string(),
            hash: "placeholder".to_string(),
            entries: 0,
        },
    }
}

// Grade A: <= 1 MB
#[test]
fn snapshot_eco_grade_a() {
    let report = build_fun_report(&derived_with_bytes(500_000));
    let json = serde_json::to_string_pretty(&report).unwrap();
    insta::assert_snapshot!("eco_grade_a", json);
}

// Grade B: <= 10 MB
#[test]
fn snapshot_eco_grade_b() {
    let report = build_fun_report(&derived_with_bytes(5 * 1024 * 1024));
    let json = serde_json::to_string_pretty(&report).unwrap();
    insta::assert_snapshot!("eco_grade_b", json);
}

// Grade C: <= 50 MB
#[test]
fn snapshot_eco_grade_c() {
    let report = build_fun_report(&derived_with_bytes(25 * 1024 * 1024));
    let json = serde_json::to_string_pretty(&report).unwrap();
    insta::assert_snapshot!("eco_grade_c", json);
}

// Grade D: <= 200 MB
#[test]
fn snapshot_eco_grade_d() {
    let report = build_fun_report(&derived_with_bytes(100 * 1024 * 1024));
    let json = serde_json::to_string_pretty(&report).unwrap();
    insta::assert_snapshot!("eco_grade_d", json);
}

// Grade E: > 200 MB
#[test]
fn snapshot_eco_grade_e() {
    let report = build_fun_report(&derived_with_bytes(300 * 1024 * 1024));
    let json = serde_json::to_string_pretty(&report).unwrap();
    insta::assert_snapshot!("eco_grade_e", json);
}

// Edge case: zero bytes
#[test]
fn snapshot_eco_zero_bytes() {
    let report = build_fun_report(&derived_with_bytes(0));
    let json = serde_json::to_string_pretty(&report).unwrap();
    insta::assert_snapshot!("eco_zero_bytes", json);
}
