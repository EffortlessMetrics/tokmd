//! Comprehensive tests for tokmd-analysis-fun enricher – wave 54.
//!
//! Covers eco-label generation, grade boundaries, notes formatting,
//! JSON round-trip, determinism, and property-based invariants.

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
    derived_with(bytes, 1, 1, 0, 0)
}

fn derived_with(
    bytes: usize,
    files: usize,
    code: usize,
    comments: usize,
    blanks: usize,
) -> DerivedReport {
    let lines = code + comments + blanks;
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
            comments,
            blanks,
            lines,
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

// ===========================================================================
// Eco-label is always present
// ===========================================================================

#[test]
fn eco_label_present_for_zero_bytes() {
    let r = build_fun_report(&derived_with_bytes(0));
    assert!(r.eco_label.is_some());
}

#[test]
fn eco_label_present_for_one_byte() {
    let r = build_fun_report(&derived_with_bytes(1));
    assert!(r.eco_label.is_some());
}

#[test]
fn eco_label_present_for_large_repo() {
    let r = build_fun_report(&derived_with_bytes(1_000_000_000));
    assert!(r.eco_label.is_some());
}

// ===========================================================================
// Grade A boundary tests (≤ 1 MB)
// ===========================================================================

#[test]
fn grade_a_at_zero() {
    let eco = build_fun_report(&derived_with_bytes(0)).eco_label.unwrap();
    assert_eq!(eco.label, "A");
    assert_eq!(eco.score, 95.0);
    assert_eq!(eco.bytes, 0);
}

#[test]
fn grade_a_at_512kb() {
    let eco = build_fun_report(&derived_with_bytes(512 * 1024))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "A");
}

#[test]
fn grade_a_at_exactly_1mb() {
    let eco = build_fun_report(&derived_with_bytes(1024 * 1024))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "A");
    assert_eq!(eco.score, 95.0);
}

// ===========================================================================
// Grade B boundary tests (> 1 MB, ≤ 10 MB)
// ===========================================================================

#[test]
fn grade_b_just_over_1mb() {
    let eco = build_fun_report(&derived_with_bytes(1024 * 1024 + 1))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "B");
    assert_eq!(eco.score, 80.0);
}

#[test]
fn grade_b_at_5mb() {
    let eco = build_fun_report(&derived_with_bytes(5 * 1024 * 1024))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "B");
}

#[test]
fn grade_b_at_exactly_10mb() {
    let eco = build_fun_report(&derived_with_bytes(10 * 1024 * 1024))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "B");
}

// ===========================================================================
// Grade C boundary tests (> 10 MB, ≤ 50 MB)
// ===========================================================================

#[test]
fn grade_c_just_over_10mb() {
    let eco = build_fun_report(&derived_with_bytes(10 * 1024 * 1024 + 1))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "C");
    assert_eq!(eco.score, 65.0);
}

#[test]
fn grade_c_at_25mb() {
    let eco = build_fun_report(&derived_with_bytes(25 * 1024 * 1024))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "C");
}

#[test]
fn grade_c_at_exactly_50mb() {
    let eco = build_fun_report(&derived_with_bytes(50 * 1024 * 1024))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "C");
}

// ===========================================================================
// Grade D boundary tests (> 50 MB, ≤ 200 MB)
// ===========================================================================

#[test]
fn grade_d_just_over_50mb() {
    let eco = build_fun_report(&derived_with_bytes(50 * 1024 * 1024 + 1))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "D");
    assert_eq!(eco.score, 45.0);
}

#[test]
fn grade_d_at_100mb() {
    let eco = build_fun_report(&derived_with_bytes(100 * 1024 * 1024))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "D");
}

#[test]
fn grade_d_at_exactly_200mb() {
    let eco = build_fun_report(&derived_with_bytes(200 * 1024 * 1024))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "D");
}

// ===========================================================================
// Grade E tests (> 200 MB)
// ===========================================================================

#[test]
fn grade_e_just_over_200mb() {
    let eco = build_fun_report(&derived_with_bytes(200 * 1024 * 1024 + 1))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "E");
    assert_eq!(eco.score, 30.0);
}

#[test]
fn grade_e_at_500mb() {
    let eco = build_fun_report(&derived_with_bytes(500 * 1024 * 1024))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "E");
}

#[test]
fn grade_e_at_1gb() {
    let eco = build_fun_report(&derived_with_bytes(1024 * 1024 * 1024))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "E");
    assert_eq!(eco.score, 30.0);
}

// ===========================================================================
// Bytes field always matches input
// ===========================================================================

#[test]
fn bytes_field_matches_input_for_all_grades() {
    let sizes = [
        0,
        1,
        999,
        1024 * 1024,
        5 * 1024 * 1024,
        30 * 1024 * 1024,
        100 * 1024 * 1024,
        500 * 1024 * 1024,
    ];
    for &bytes in &sizes {
        let eco = build_fun_report(&derived_with_bytes(bytes))
            .eco_label
            .unwrap();
        assert_eq!(eco.bytes, bytes as u64, "mismatch for {bytes} bytes");
    }
}

// ===========================================================================
// Notes formatting
// ===========================================================================

#[test]
fn notes_start_with_prefix() {
    let eco = build_fun_report(&derived_with_bytes(1000))
        .eco_label
        .unwrap();
    assert!(eco.notes.starts_with("Size-based eco label ("));
}

#[test]
fn notes_end_with_mb_suffix() {
    let eco = build_fun_report(&derived_with_bytes(1000))
        .eco_label
        .unwrap();
    assert!(eco.notes.ends_with(" MB)"));
}

#[test]
fn notes_zero_bytes_shows_0_mb() {
    let eco = build_fun_report(&derived_with_bytes(0)).eco_label.unwrap();
    assert_eq!(eco.notes, "Size-based eco label (0 MB)");
}

#[test]
fn notes_1mb_shows_1_mb() {
    let eco = build_fun_report(&derived_with_bytes(1024 * 1024))
        .eco_label
        .unwrap();
    assert!(eco.notes.contains("1 MB"), "got: {}", eco.notes);
}

#[test]
fn notes_2_5mb_shows_correct_decimal() {
    // 2.5 MB = 2621440 bytes
    let eco = build_fun_report(&derived_with_bytes(2_621_440))
        .eco_label
        .unwrap();
    assert!(eco.notes.contains("2.5 MB"), "got: {}", eco.notes);
}

#[test]
fn notes_fractional_bytes_round_to_two_decimals() {
    // 1.23 MB ≈ 1_289_748 bytes (1.23 * 1024 * 1024)
    let bytes = 1_289_748;
    let mb = bytes as f64 / (1024.0 * 1024.0);
    let rounded = (mb * 100.0).round() / 100.0;
    let eco = build_fun_report(&derived_with_bytes(bytes))
        .eco_label
        .unwrap();
    assert!(
        eco.notes.contains(&format!("{rounded} MB")),
        "got: {}",
        eco.notes
    );
}

// ===========================================================================
// Different DerivedReport configs produce same eco for same bytes
// ===========================================================================

#[test]
fn eco_depends_only_on_bytes_not_code_lines() {
    let bytes = 5_000_000;
    let r1 = build_fun_report(&derived_with(bytes, 1, 100, 0, 0));
    let r2 = build_fun_report(&derived_with(bytes, 100, 10000, 5000, 2000));
    let e1 = r1.eco_label.unwrap();
    let e2 = r2.eco_label.unwrap();
    assert_eq!(e1.label, e2.label);
    assert_eq!(e1.score, e2.score);
    assert_eq!(e1.bytes, e2.bytes);
    assert_eq!(e1.notes, e2.notes);
}

#[test]
fn eco_depends_only_on_bytes_not_file_count() {
    let bytes = 30_000_000;
    let r1 = build_fun_report(&derived_with(bytes, 1, 1, 0, 0));
    let r2 = build_fun_report(&derived_with(bytes, 10000, 50000, 10000, 5000));
    assert_eq!(r1.eco_label.unwrap().label, r2.eco_label.unwrap().label,);
}

// ===========================================================================
// Determinism
// ===========================================================================

#[test]
fn determinism_100_iterations() {
    let d = derived_with_bytes(7_654_321);
    let reference = build_fun_report(&d);
    for _ in 0..100 {
        let r = build_fun_report(&d);
        let e_ref = reference.eco_label.as_ref().unwrap();
        let e = r.eco_label.unwrap();
        assert_eq!(e.label, e_ref.label);
        assert_eq!(e.score, e_ref.score);
        assert_eq!(e.bytes, e_ref.bytes);
        assert_eq!(e.notes, e_ref.notes);
    }
}

// ===========================================================================
// Score monotonicity
// ===========================================================================

#[test]
fn scores_monotonically_decrease_across_all_bands() {
    let sizes = [
        0,
        100,
        1024 * 1024 - 1,
        1024 * 1024,
        1024 * 1024 + 1,
        5 * 1024 * 1024,
        10 * 1024 * 1024,
        10 * 1024 * 1024 + 1,
        30 * 1024 * 1024,
        50 * 1024 * 1024,
        50 * 1024 * 1024 + 1,
        100 * 1024 * 1024,
        200 * 1024 * 1024,
        200 * 1024 * 1024 + 1,
        500 * 1024 * 1024,
        1024 * 1024 * 1024,
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
    for w in scores.windows(2) {
        assert!(
            w[0] >= w[1],
            "score {} should be >= {}, sizes were monotonically increasing",
            w[0],
            w[1]
        );
    }
}

#[test]
fn labels_never_improve_as_size_grows() {
    let label_rank = |l: &str| -> u8 {
        match l {
            "A" => 5,
            "B" => 4,
            "C" => 3,
            "D" => 2,
            "E" => 1,
            _ => 0,
        }
    };
    let sizes = [
        0,
        1024 * 1024 + 1,
        10 * 1024 * 1024 + 1,
        50 * 1024 * 1024 + 1,
        200 * 1024 * 1024 + 1,
    ];
    let ranks: Vec<u8> = sizes
        .iter()
        .map(|&b| {
            label_rank(
                &build_fun_report(&derived_with_bytes(b))
                    .eco_label
                    .unwrap()
                    .label,
            )
        })
        .collect();
    for w in ranks.windows(2) {
        assert!(w[0] >= w[1], "labels should not improve as size grows");
    }
}

// ===========================================================================
// Score values are from the known set
// ===========================================================================

#[test]
fn score_is_one_of_five_known_values() {
    let known = [95.0, 80.0, 65.0, 45.0, 30.0];
    for bytes in [
        0,
        100,
        1_000_000,
        5_000_000,
        30_000_000,
        100_000_000,
        500_000_000,
    ] {
        let eco = build_fun_report(&derived_with_bytes(bytes))
            .eco_label
            .unwrap();
        assert!(
            known.contains(&eco.score),
            "unknown score {} for {} bytes",
            eco.score,
            bytes,
        );
    }
}

// ===========================================================================
// Label-score bijection
// ===========================================================================

#[test]
fn each_label_maps_to_exactly_one_score() {
    let cases: Vec<(usize, &str, f64)> = vec![
        (0, "A", 95.0),
        (2 * 1024 * 1024, "B", 80.0),
        (20 * 1024 * 1024, "C", 65.0),
        (100 * 1024 * 1024, "D", 45.0),
        (300 * 1024 * 1024, "E", 30.0),
    ];
    for (bytes, expected_label, expected_score) in cases {
        let eco = build_fun_report(&derived_with_bytes(bytes))
            .eco_label
            .unwrap();
        assert_eq!(eco.label, expected_label);
        assert_eq!(eco.score, expected_score);
    }
}

// ===========================================================================
// JSON round-trip
// ===========================================================================

#[test]
fn json_round_trip_preserves_all_fields() {
    let report = build_fun_report(&derived_with_bytes(42_000_000));
    let json = serde_json::to_string(&report).unwrap();
    let rt: tokmd_analysis_types::FunReport = serde_json::from_str(&json).unwrap();
    let orig = report.eco_label.unwrap();
    let de = rt.eco_label.unwrap();
    assert_eq!(orig.label, de.label);
    assert_eq!(orig.score, de.score);
    assert_eq!(orig.bytes, de.bytes);
    assert_eq!(orig.notes, de.notes);
}

#[test]
fn json_contains_eco_label_key() {
    let report = build_fun_report(&derived_with_bytes(1000));
    let json = serde_json::to_string(&report).unwrap();
    assert!(json.contains("eco_label"));
}

#[test]
fn json_contains_all_eco_fields() {
    let report = build_fun_report(&derived_with_bytes(5_000_000));
    let json = serde_json::to_string_pretty(&report).unwrap();
    assert!(json.contains("\"score\""));
    assert!(json.contains("\"label\""));
    assert!(json.contains("\"bytes\""));
    assert!(json.contains("\"notes\""));
}

#[test]
fn json_pretty_round_trip_for_every_grade() {
    for bytes in [
        0,
        500_000,
        5 * 1024 * 1024,
        25 * 1024 * 1024,
        100 * 1024 * 1024,
        300 * 1024 * 1024,
    ] {
        let report = build_fun_report(&derived_with_bytes(bytes));
        let json = serde_json::to_string_pretty(&report).unwrap();
        let rt: tokmd_analysis_types::FunReport = serde_json::from_str(&json).unwrap();
        assert_eq!(
            report.eco_label.unwrap().label,
            rt.eco_label.unwrap().label,
            "round-trip failed for {bytes} bytes"
        );
    }
}

// ===========================================================================
// Edge cases: zero-code repos
// ===========================================================================

#[test]
fn zero_code_zero_files_zero_bytes() {
    let d = derived_with(0, 0, 0, 0, 0);
    let eco = build_fun_report(&d).eco_label.unwrap();
    assert_eq!(eco.label, "A");
    assert_eq!(eco.bytes, 0);
}

#[test]
fn many_files_zero_bytes_is_grade_a() {
    let d = derived_with(0, 1000, 50000, 10000, 5000);
    let eco = build_fun_report(&d).eco_label.unwrap();
    assert_eq!(eco.label, "A");
    assert_eq!(eco.bytes, 0);
}

#[test]
fn single_file_one_byte() {
    let d = derived_with(1, 1, 1, 0, 0);
    let eco = build_fun_report(&d).eco_label.unwrap();
    assert_eq!(eco.label, "A");
    assert_eq!(eco.bytes, 1);
}

// ===========================================================================
// Power-of-two byte sizes
// ===========================================================================

#[test]
fn power_of_two_byte_sizes_produce_valid_labels() {
    let known = ["A", "B", "C", "D", "E"];
    for exp in 0..=30 {
        let bytes = 1usize << exp;
        let eco = build_fun_report(&derived_with_bytes(bytes))
            .eco_label
            .unwrap();
        assert!(
            known.contains(&eco.label.as_str()),
            "unknown label '{}' for 2^{} = {} bytes",
            eco.label,
            exp,
            bytes,
        );
    }
}

// ===========================================================================
// Property tests
// ===========================================================================

mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn eco_label_always_some(bytes in 0usize..=(2 * 1024 * 1024 * 1024)) {
            let r = build_fun_report(&derived_with_bytes(bytes));
            prop_assert!(r.eco_label.is_some());
        }

        #[test]
        fn bytes_field_equals_input(bytes in 0usize..=(2 * 1024 * 1024 * 1024)) {
            let eco = build_fun_report(&derived_with_bytes(bytes)).eco_label.unwrap();
            prop_assert_eq!(eco.bytes, bytes as u64);
        }

        #[test]
        fn label_in_known_set(bytes in 0usize..=(2 * 1024 * 1024 * 1024)) {
            let eco = build_fun_report(&derived_with_bytes(bytes)).eco_label.unwrap();
            prop_assert!(["A", "B", "C", "D", "E"].contains(&eco.label.as_str()));
        }

        #[test]
        fn score_in_0_to_100(bytes in 0usize..=(2 * 1024 * 1024 * 1024)) {
            let eco = build_fun_report(&derived_with_bytes(bytes)).eco_label.unwrap();
            prop_assert!(eco.score >= 0.0 && eco.score <= 100.0);
        }

        #[test]
        fn notes_format_valid(bytes in 0usize..=(2 * 1024 * 1024 * 1024)) {
            let eco = build_fun_report(&derived_with_bytes(bytes)).eco_label.unwrap();
            prop_assert!(eco.notes.starts_with("Size-based eco label ("));
            prop_assert!(eco.notes.ends_with(" MB)"));
        }

        #[test]
        fn deterministic(bytes in 0usize..=(2 * 1024 * 1024 * 1024)) {
            let d = derived_with_bytes(bytes);
            let e1 = build_fun_report(&d).eco_label.unwrap();
            let e2 = build_fun_report(&d).eco_label.unwrap();
            prop_assert_eq!(e1.label, e2.label);
            prop_assert_eq!(e1.score, e2.score);
            prop_assert_eq!(e1.bytes, e2.bytes);
            prop_assert_eq!(e1.notes, e2.notes);
        }

        #[test]
        fn monotonic_scores(
            a in 0usize..=(1024 * 1024 * 1024),
            b in 0usize..=(1024 * 1024 * 1024),
        ) {
            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
            let s_lo = build_fun_report(&derived_with_bytes(lo)).eco_label.unwrap().score;
            let s_hi = build_fun_report(&derived_with_bytes(hi)).eco_label.unwrap().score;
            prop_assert!(s_lo >= s_hi);
        }

        #[test]
        fn label_score_consistent(bytes in 0usize..=(2 * 1024 * 1024 * 1024)) {
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

        #[test]
        fn json_round_trip(bytes in 0usize..=(512 * 1024 * 1024)) {
            let report = build_fun_report(&derived_with_bytes(bytes));
            let json = serde_json::to_string(&report).unwrap();
            let rt: tokmd_analysis_types::FunReport = serde_json::from_str(&json).unwrap();
            let orig = report.eco_label.unwrap();
            let de = rt.eco_label.unwrap();
            prop_assert_eq!(orig.label, de.label);
            prop_assert_eq!(orig.score, de.score);
            prop_assert_eq!(orig.bytes, de.bytes);
        }

        #[test]
        fn different_totals_same_bytes_same_eco(
            bytes in 0usize..=(512 * 1024 * 1024),
            files in 1usize..=1000,
            code in 0usize..=100000,
        ) {
            let r1 = build_fun_report(&derived_with(bytes, 1, 1, 0, 0));
            let r2 = build_fun_report(&derived_with(bytes, files, code, 0, 0));
            let e1 = r1.eco_label.unwrap();
            let e2 = r2.eco_label.unwrap();
            prop_assert_eq!(e1.label, e2.label);
            prop_assert_eq!(e1.score, e2.score);
            prop_assert_eq!(e1.bytes, e2.bytes);
        }
    }
}
