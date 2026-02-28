//! BDD-style scenario tests for fun/novelty output generation.
//!
//! Each test follows the pattern:
//! `given_<precondition>_when_<action>_then_<expected_result>`

use tokmd_analysis_fun::build_fun_report;
use tokmd_analysis_types::{
    BoilerplateReport, DerivedReport, DerivedTotals, DistributionReport, FileStatRow,
    IntegrityReport, LangPurityReport, MaxFileReport, NestingReport, PolyglotReport, RateReport,
    RateRow, RatioReport, RatioRow, ReadingTimeReport, TestDensityReport, TodoReport, TopOffenders,
};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn derived_with_bytes(bytes: usize) -> DerivedReport {
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

// ---------------------------------------------------------------------------
// Grade band: A  (≤ 1 MB)
// ---------------------------------------------------------------------------

#[test]
fn given_zero_bytes_when_report_built_then_grade_is_a() {
    let report = build_fun_report(&derived_with_bytes(0));
    let eco = report.eco_label.expect("eco_label must be present");
    assert_eq!(eco.label, "A");
    assert_eq!(eco.score, 95.0);
    assert_eq!(eco.bytes, 0);
}

#[test]
fn given_one_byte_when_report_built_then_grade_is_a() {
    let report = build_fun_report(&derived_with_bytes(1));
    let eco = report.eco_label.expect("eco_label must be present");
    assert_eq!(eco.label, "A");
    assert_eq!(eco.score, 95.0);
}

#[test]
fn given_exactly_one_mb_when_report_built_then_grade_is_a() {
    let bytes = 1024 * 1024; // 1 MB
    let report = build_fun_report(&derived_with_bytes(bytes));
    let eco = report.eco_label.unwrap();
    assert_eq!(eco.label, "A");
    assert_eq!(eco.score, 95.0);
    assert_eq!(eco.bytes, bytes as u64);
}

// ---------------------------------------------------------------------------
// Grade band: B  (> 1 MB, ≤ 10 MB)
// ---------------------------------------------------------------------------

#[test]
fn given_just_over_one_mb_when_report_built_then_grade_is_b() {
    let bytes = 1024 * 1024 + 1;
    let report = build_fun_report(&derived_with_bytes(bytes));
    let eco = report.eco_label.unwrap();
    assert_eq!(eco.label, "B");
    assert_eq!(eco.score, 80.0);
}

#[test]
fn given_exactly_ten_mb_when_report_built_then_grade_is_b() {
    let bytes = 10 * 1024 * 1024;
    let report = build_fun_report(&derived_with_bytes(bytes));
    let eco = report.eco_label.unwrap();
    assert_eq!(eco.label, "B");
    assert_eq!(eco.score, 80.0);
}

// ---------------------------------------------------------------------------
// Grade band: C  (> 10 MB, ≤ 50 MB)
// ---------------------------------------------------------------------------

#[test]
fn given_just_over_ten_mb_when_report_built_then_grade_is_c() {
    let bytes = 10 * 1024 * 1024 + 1;
    let report = build_fun_report(&derived_with_bytes(bytes));
    let eco = report.eco_label.unwrap();
    assert_eq!(eco.label, "C");
    assert_eq!(eco.score, 65.0);
}

#[test]
fn given_exactly_fifty_mb_when_report_built_then_grade_is_c() {
    let bytes = 50 * 1024 * 1024;
    let report = build_fun_report(&derived_with_bytes(bytes));
    let eco = report.eco_label.unwrap();
    assert_eq!(eco.label, "C");
    assert_eq!(eco.score, 65.0);
}

// ---------------------------------------------------------------------------
// Grade band: D  (> 50 MB, ≤ 200 MB)
// ---------------------------------------------------------------------------

#[test]
fn given_just_over_fifty_mb_when_report_built_then_grade_is_d() {
    let bytes = 50 * 1024 * 1024 + 1;
    let report = build_fun_report(&derived_with_bytes(bytes));
    let eco = report.eco_label.unwrap();
    assert_eq!(eco.label, "D");
    assert_eq!(eco.score, 45.0);
}

#[test]
fn given_exactly_two_hundred_mb_when_report_built_then_grade_is_d() {
    let bytes = 200 * 1024 * 1024;
    let report = build_fun_report(&derived_with_bytes(bytes));
    let eco = report.eco_label.unwrap();
    assert_eq!(eco.label, "D");
    assert_eq!(eco.score, 45.0);
}

// ---------------------------------------------------------------------------
// Grade band: E  (> 200 MB)
// ---------------------------------------------------------------------------

#[test]
fn given_just_over_two_hundred_mb_when_report_built_then_grade_is_e() {
    let bytes = 200 * 1024 * 1024 + 1;
    let report = build_fun_report(&derived_with_bytes(bytes));
    let eco = report.eco_label.unwrap();
    assert_eq!(eco.label, "E");
    assert_eq!(eco.score, 30.0);
}

#[test]
fn given_one_gb_when_report_built_then_grade_is_e() {
    let bytes = 1024 * 1024 * 1024;
    let report = build_fun_report(&derived_with_bytes(bytes));
    let eco = report.eco_label.unwrap();
    assert_eq!(eco.label, "E");
    assert_eq!(eco.score, 30.0);
}

// ---------------------------------------------------------------------------
// Output invariants
// ---------------------------------------------------------------------------

#[test]
fn given_any_input_when_report_built_then_eco_label_is_always_present() {
    for &bytes in &[0, 1, 1000, 1_000_000, 100_000_000, 1_000_000_000] {
        let report = build_fun_report(&derived_with_bytes(bytes));
        assert!(
            report.eco_label.is_some(),
            "eco_label must always be present for bytes={bytes}"
        );
    }
}

#[test]
fn given_any_input_when_report_built_then_bytes_field_matches_input() {
    for &bytes in &[0, 42, 999_999, 50_000_000] {
        let report = build_fun_report(&derived_with_bytes(bytes));
        let eco = report.eco_label.unwrap();
        assert_eq!(eco.bytes, bytes as u64, "bytes mismatch for input={bytes}");
    }
}

#[test]
fn given_any_input_when_report_built_then_notes_contain_mb_suffix() {
    let report = build_fun_report(&derived_with_bytes(5_000_000));
    let eco = report.eco_label.unwrap();
    assert!(
        eco.notes.contains("MB"),
        "notes should contain MB: {}",
        eco.notes
    );
}

#[test]
fn given_any_input_when_report_built_then_notes_are_human_readable() {
    let report = build_fun_report(&derived_with_bytes(5_000_000));
    let eco = report.eco_label.unwrap();
    assert!(eco.notes.starts_with("Size-based eco label"));
}

#[test]
fn given_any_input_when_report_built_then_label_is_single_uppercase_letter() {
    for &bytes in &[0, 1_000_000, 10_000_001, 60_000_000, 300_000_000] {
        let report = build_fun_report(&derived_with_bytes(bytes));
        let eco = report.eco_label.unwrap();
        assert!(
            eco.label.len() == 1 && eco.label.chars().next().unwrap().is_ascii_uppercase(),
            "label must be a single uppercase letter, got: {}",
            eco.label
        );
    }
}

#[test]
fn given_any_input_when_report_built_then_score_is_between_zero_and_one_hundred() {
    for &bytes in &[0, 500_000, 5_000_000, 30_000_000, 100_000_000, 500_000_000] {
        let report = build_fun_report(&derived_with_bytes(bytes));
        let eco = report.eco_label.unwrap();
        assert!(
            (0.0..=100.0).contains(&eco.score),
            "score out of range: {}",
            eco.score
        );
    }
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn given_same_input_when_report_built_twice_then_outputs_are_identical() {
    let derived = derived_with_bytes(7_500_000);
    let r1 = build_fun_report(&derived);
    let r2 = build_fun_report(&derived);

    let e1 = r1.eco_label.unwrap();
    let e2 = r2.eco_label.unwrap();

    assert_eq!(e1.label, e2.label);
    assert_eq!(e1.score, e2.score);
    assert_eq!(e1.bytes, e2.bytes);
    assert_eq!(e1.notes, e2.notes);
}

// ---------------------------------------------------------------------------
// Grade monotonicity: larger repos get worse grades
// ---------------------------------------------------------------------------

#[test]
fn given_increasing_sizes_when_reports_built_then_scores_decrease_monotonically() {
    let sizes: Vec<usize> = vec![
        0,
        1024 * 1024 + 1,
        10 * 1024 * 1024 + 1,
        50 * 1024 * 1024 + 1,
        200 * 1024 * 1024 + 1,
    ];

    let scores: Vec<f64> = sizes
        .iter()
        .map(|&b| {
            build_fun_report(&derived_with_bytes(b))
                .eco_label
                .unwrap()
                .score
        })
        .collect();

    for window in scores.windows(2) {
        assert!(window[0] >= window[1], "scores must decrease: {:?}", scores);
    }
}

// ---------------------------------------------------------------------------
// Notes formatting
// ---------------------------------------------------------------------------

#[test]
fn given_fractional_mb_when_report_built_then_notes_round_to_two_decimals() {
    // 1.5 MB = 1_572_864 bytes
    let bytes = 1_572_864;
    let report = build_fun_report(&derived_with_bytes(bytes));
    let eco = report.eco_label.unwrap();
    assert!(
        eco.notes.contains("1.5 MB"),
        "expected 1.5 MB in notes, got: {}",
        eco.notes
    );
}

#[test]
fn given_exact_zero_bytes_when_report_built_then_notes_show_zero_mb() {
    let report = build_fun_report(&derived_with_bytes(0));
    let eco = report.eco_label.unwrap();
    assert!(
        eco.notes.contains("0 MB"),
        "expected 0 MB in notes, got: {}",
        eco.notes
    );
}
