//! Snapshot and edge-case tests for fun/novelty output generation.

use tokmd_analysis_fun::build_fun_report;
use tokmd_analysis_types::{
    BoilerplateReport, DerivedReport, DerivedTotals, DistributionReport, FileStatRow,
    IntegrityReport, LangPurityReport, MaxFileReport, NestingReport, PolyglotReport, RateReport,
    RateRow, RatioReport, RatioRow, ReadingTimeReport, TestDensityReport, TodoReport, TopOffenders,
};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn derived_with(bytes: usize, code: usize, files: usize) -> DerivedReport {
    let zero_row = FileStatRow {
        path: "f.rs".to_string(),
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
            files,
            code,
            comments: 0,
            blanks: 0,
            lines: code,
            bytes,
            tokens: code,
        },
        doc_density: RatioReport {
            total: RatioRow {
                key: "All".into(),
                numerator: 0,
                denominator: 1,
                ratio: 0.0,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        whitespace: RatioReport {
            total: RatioRow {
                key: "All".into(),
                numerator: 0,
                denominator: 1,
                ratio: 0.0,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        verbosity: RateReport {
            total: RateRow {
                key: "All".into(),
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

fn derived_with_bytes(bytes: usize) -> DerivedReport {
    derived_with(bytes, 1, 1)
}

// ---------------------------------------------------------------------------
// Insta snapshot tests
// ---------------------------------------------------------------------------

#[test]
fn snapshot_eco_label_grade_a() {
    let report = build_fun_report(&derived_with_bytes(512 * 1024)); // 0.5 MB
    insta::assert_json_snapshot!("eco_label_grade_a", report);
}

#[test]
fn snapshot_eco_label_grade_b() {
    let report = build_fun_report(&derived_with_bytes(5 * 1024 * 1024)); // 5 MB
    insta::assert_json_snapshot!("eco_label_grade_b", report);
}

#[test]
fn snapshot_eco_label_grade_c() {
    let report = build_fun_report(&derived_with_bytes(25 * 1024 * 1024)); // 25 MB
    insta::assert_json_snapshot!("eco_label_grade_c", report);
}

#[test]
fn snapshot_eco_label_grade_d() {
    let report = build_fun_report(&derived_with_bytes(100 * 1024 * 1024)); // 100 MB
    insta::assert_json_snapshot!("eco_label_grade_d", report);
}

#[test]
fn snapshot_eco_label_grade_e() {
    let report = build_fun_report(&derived_with_bytes(500 * 1024 * 1024)); // 500 MB
    insta::assert_json_snapshot!("eco_label_grade_e", report);
}

#[test]
fn snapshot_eco_label_zero_bytes() {
    let report = build_fun_report(&derived_with_bytes(0));
    insta::assert_json_snapshot!("eco_label_zero_bytes", report);
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn given_zero_code_lines_when_report_built_then_eco_label_still_present() {
    let report = build_fun_report(&derived_with(1024, 0, 0));
    let eco = report
        .eco_label
        .expect("eco_label must be present even with zero code");
    assert_eq!(eco.label, "A");
    assert_eq!(eco.bytes, 1024);
}

#[test]
fn given_single_file_zero_bytes_when_report_built_then_grade_a_with_zero_bytes() {
    let report = build_fun_report(&derived_with(0, 1, 1));
    let eco = report.eco_label.unwrap();
    assert_eq!(eco.label, "A");
    assert_eq!(eco.score, 95.0);
    assert_eq!(eco.bytes, 0);
    assert!(eco.notes.contains("0 MB"));
}

#[test]
fn given_single_large_file_when_report_built_then_grade_reflects_total_bytes() {
    let report = build_fun_report(&derived_with(300 * 1024 * 1024, 100_000, 1));
    let eco = report.eco_label.unwrap();
    assert_eq!(eco.label, "E");
    assert_eq!(eco.score, 30.0);
}

// ---------------------------------------------------------------------------
// Deterministic JSON serialization
// ---------------------------------------------------------------------------

#[test]
fn given_same_input_when_serialized_to_json_twice_then_identical_output() {
    let derived = derived_with_bytes(7_500_000);
    let r1 = build_fun_report(&derived);
    let r2 = build_fun_report(&derived);

    let j1 = serde_json::to_string_pretty(&r1).unwrap();
    let j2 = serde_json::to_string_pretty(&r2).unwrap();
    assert_eq!(j1, j2, "JSON serialization must be deterministic");
}

#[test]
fn given_valid_input_when_serialized_then_non_empty_json() {
    let report = build_fun_report(&derived_with_bytes(1024));
    let json = serde_json::to_string(&report).unwrap();
    assert!(!json.is_empty());
    assert!(json.contains("eco_label"));
    assert!(json.contains("score"));
    assert!(json.contains("label"));
}

// ---------------------------------------------------------------------------
// Round-trip serialization
// ---------------------------------------------------------------------------

#[test]
fn fun_report_round_trip_serialization() {
    let report = build_fun_report(&derived_with_bytes(42_000_000));
    let json = serde_json::to_string(&report).unwrap();
    let deserialized: tokmd_analysis_types::FunReport = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&deserialized).unwrap();
    assert_eq!(json, json2, "round-trip serialization must be stable");
}
