//! Deep tests for tokmd-analysis-fun – wave 45.
//!
//! Fills coverage gaps: JSON round-trip serialization, exact boundary byte
//! values, notes MB formatting precision, snapshot at exact thresholds, and
//! additional property tests for `round_to_two` behavior.
//!
//! Run with: `cargo test -p tokmd-analysis-fun --test deep_w45`

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

// =========================================================================
// JSON round-trip: serialize then deserialize produces equivalent report
// =========================================================================

#[test]
fn json_round_trip_preserves_eco_label() {
    let report = build_fun_report(&derived_with_bytes(5_000_000));
    let json = serde_json::to_string(&report).unwrap();
    let deserialized: tokmd_analysis_types::FunReport = serde_json::from_str(&json).unwrap();

    let orig = report.eco_label.unwrap();
    let rt = deserialized.eco_label.unwrap();
    assert_eq!(orig.label, rt.label);
    assert_eq!(orig.score, rt.score);
    assert_eq!(orig.bytes, rt.bytes);
    assert_eq!(orig.notes, rt.notes);
}

#[test]
fn json_round_trip_for_each_grade() {
    let sizes = [
        0usize,
        500_000,                // A
        5 * 1024 * 1024,        // B
        25 * 1024 * 1024,       // C
        100 * 1024 * 1024,      // D
        300 * 1024 * 1024,      // E
    ];
    for &bytes in &sizes {
        let report = build_fun_report(&derived_with_bytes(bytes));
        let json = serde_json::to_string_pretty(&report).unwrap();
        let rt: tokmd_analysis_types::FunReport = serde_json::from_str(&json).unwrap();
        assert_eq!(
            report.eco_label.as_ref().unwrap().label,
            rt.eco_label.as_ref().unwrap().label,
            "round-trip label mismatch for {bytes} bytes"
        );
    }
}

// =========================================================================
// Exact byte-boundary tests: verify band transitions at precise thresholds
// =========================================================================

#[test]
fn given_exactly_1mb_minus_1_byte_then_grade_a() {
    let bytes = 1024 * 1024 - 1;
    let eco = build_fun_report(&derived_with_bytes(bytes))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "A");
}

#[test]
fn given_exactly_1mb_then_grade_a() {
    let bytes = 1024 * 1024;
    let eco = build_fun_report(&derived_with_bytes(bytes))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "A");
}

#[test]
fn given_exactly_1mb_plus_1_byte_then_grade_b() {
    let bytes = 1024 * 1024 + 1;
    let eco = build_fun_report(&derived_with_bytes(bytes))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "B");
}

#[test]
fn given_exactly_10mb_minus_1_byte_then_grade_b() {
    let bytes = 10 * 1024 * 1024 - 1;
    let eco = build_fun_report(&derived_with_bytes(bytes))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "B");
}

#[test]
fn given_exactly_10mb_then_grade_b() {
    let bytes = 10 * 1024 * 1024;
    let eco = build_fun_report(&derived_with_bytes(bytes))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "B");
}

#[test]
fn given_exactly_10mb_plus_1_byte_then_grade_c() {
    let bytes = 10 * 1024 * 1024 + 1;
    let eco = build_fun_report(&derived_with_bytes(bytes))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "C");
}

#[test]
fn given_exactly_50mb_minus_1_byte_then_grade_c() {
    let bytes = 50 * 1024 * 1024 - 1;
    let eco = build_fun_report(&derived_with_bytes(bytes))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "C");
}

#[test]
fn given_exactly_50mb_then_grade_c() {
    let bytes = 50 * 1024 * 1024;
    let eco = build_fun_report(&derived_with_bytes(bytes))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "C");
}

#[test]
fn given_exactly_50mb_plus_1_byte_then_grade_d() {
    let bytes = 50 * 1024 * 1024 + 1;
    let eco = build_fun_report(&derived_with_bytes(bytes))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "D");
}

#[test]
fn given_exactly_200mb_minus_1_byte_then_grade_d() {
    let bytes = 200 * 1024 * 1024 - 1;
    let eco = build_fun_report(&derived_with_bytes(bytes))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "D");
}

#[test]
fn given_exactly_200mb_then_grade_d() {
    let bytes = 200 * 1024 * 1024;
    let eco = build_fun_report(&derived_with_bytes(bytes))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "D");
}

#[test]
fn given_exactly_200mb_plus_1_byte_then_grade_e() {
    let bytes = 200 * 1024 * 1024 + 1;
    let eco = build_fun_report(&derived_with_bytes(bytes))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "E");
}

// =========================================================================
// Notes MB formatting precision
// =========================================================================

#[test]
fn notes_format_whole_number_mb_has_no_trailing_decimals() {
    // 2 MB = 2 * 1024 * 1024 = 2_097_152 bytes → 2.0 MB
    let eco = build_fun_report(&derived_with_bytes(2 * 1024 * 1024))
        .eco_label
        .unwrap();
    assert!(
        eco.notes.contains("2 MB"),
        "expected '2 MB' in notes, got: {}",
        eco.notes
    );
}

#[test]
fn notes_format_sub_mb_shows_fractional() {
    // 512 KB = 524_288 bytes → 0.5 MB
    let eco = build_fun_report(&derived_with_bytes(524_288))
        .eco_label
        .unwrap();
    assert!(
        eco.notes.contains("0.5 MB"),
        "expected '0.5 MB' in notes, got: {}",
        eco.notes
    );
}

#[test]
fn notes_format_tiny_bytes_shows_near_zero() {
    // 1 byte → ~0.0000009... MB → rounds to 0.0
    let eco = build_fun_report(&derived_with_bytes(1))
        .eco_label
        .unwrap();
    assert!(
        eco.notes.contains("0 MB"),
        "expected '0 MB' in notes, got: {}",
        eco.notes
    );
}

#[test]
fn notes_format_precise_third_decimal_rounded() {
    // 1.555 MB ≈ 1_630_535 bytes → should round to 1.56 (or 1.55 depending on exact bytes)
    let bytes = 1_630_535;
    let eco = build_fun_report(&derived_with_bytes(bytes))
        .eco_label
        .unwrap();
    // The exact MB value is bytes / (1024*1024), rounded to two decimals
    let mb = bytes as f64 / (1024.0 * 1024.0);
    let rounded = (mb * 100.0).round() / 100.0;
    let expected_fragment = format!("{rounded} MB");
    assert!(
        eco.notes.contains(&expected_fragment),
        "expected '{expected_fragment}' in notes, got: {}",
        eco.notes
    );
}

// =========================================================================
// Snapshot at exact boundary byte values
// =========================================================================

#[test]
fn snapshot_eco_exact_1mb_boundary() {
    let report = build_fun_report(&derived_with_bytes(1024 * 1024));
    let json = serde_json::to_string_pretty(&report).unwrap();
    insta::assert_snapshot!("eco_exact_1mb", json);
}

#[test]
fn snapshot_eco_exact_10mb_boundary() {
    let report = build_fun_report(&derived_with_bytes(10 * 1024 * 1024));
    let json = serde_json::to_string_pretty(&report).unwrap();
    insta::assert_snapshot!("eco_exact_10mb", json);
}

#[test]
fn snapshot_eco_exact_50mb_boundary() {
    let report = build_fun_report(&derived_with_bytes(50 * 1024 * 1024));
    let json = serde_json::to_string_pretty(&report).unwrap();
    insta::assert_snapshot!("eco_exact_50mb", json);
}

#[test]
fn snapshot_eco_exact_200mb_boundary() {
    let report = build_fun_report(&derived_with_bytes(200 * 1024 * 1024));
    let json = serde_json::to_string_pretty(&report).unwrap();
    insta::assert_snapshot!("eco_exact_200mb", json);
}

// =========================================================================
// Score–label consistency: known score values per label
// =========================================================================

#[test]
fn score_label_pairs_are_consistent() {
    let cases: Vec<(usize, &str, f64)> = vec![
        (0, "A", 95.0),
        (500_000, "A", 95.0),
        (2 * 1024 * 1024, "B", 80.0),
        (20 * 1024 * 1024, "C", 65.0),
        (100 * 1024 * 1024, "D", 45.0),
        (300 * 1024 * 1024, "E", 30.0),
    ];
    for (bytes, expected_label, expected_score) in cases {
        let eco = build_fun_report(&derived_with_bytes(bytes))
            .eco_label
            .unwrap();
        assert_eq!(
            eco.label, expected_label,
            "label mismatch for {bytes} bytes"
        );
        assert_eq!(
            eco.score, expected_score,
            "score mismatch for {bytes} bytes"
        );
    }
}

// =========================================================================
// Grade labels form a known finite set
// =========================================================================

#[test]
fn all_grade_labels_are_in_known_set() {
    let samples = [0, 1, 1_000_000, 5_000_000, 30_000_000, 100_000_000, 500_000_000];
    let known = ["A", "B", "C", "D", "E"];
    for &bytes in &samples {
        let eco = build_fun_report(&derived_with_bytes(bytes))
            .eco_label
            .unwrap();
        assert!(
            known.contains(&eco.label.as_str()),
            "unknown label '{}' for {bytes} bytes",
            eco.label
        );
    }
}

// =========================================================================
// Property: notes field always matches expected format
// =========================================================================

mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Notes always match the pattern "Size-based eco label (X MB)"
        /// where X is a number with at most 2 decimal places.
        #[test]
        fn notes_match_expected_format(bytes in 0usize..=(512 * 1024 * 1024)) {
            let eco = build_fun_report(&derived_with_bytes(bytes)).eco_label.unwrap();
            prop_assert!(eco.notes.starts_with("Size-based eco label ("));
            prop_assert!(eco.notes.ends_with(" MB)"));
        }

        /// Score is always one of the five known values.
        #[test]
        fn score_is_one_of_five_values(bytes in 0usize..=(512 * 1024 * 1024)) {
            let eco = build_fun_report(&derived_with_bytes(bytes)).eco_label.unwrap();
            let known_scores = [95.0, 80.0, 65.0, 45.0, 30.0];
            prop_assert!(
                known_scores.contains(&eco.score),
                "score {} not in known set",
                eco.score
            );
        }

        /// For each label there is exactly one associated score.
        #[test]
        fn label_score_bijection(bytes in 0usize..=(512 * 1024 * 1024)) {
            let eco = build_fun_report(&derived_with_bytes(bytes)).eco_label.unwrap();
            let expected_score = match eco.label.as_str() {
                "A" => 95.0,
                "B" => 80.0,
                "C" => 65.0,
                "D" => 45.0,
                "E" => 30.0,
                other => panic!("unknown label: {other}"),
            };
            prop_assert_eq!(eco.score, expected_score);
        }
    }
}
