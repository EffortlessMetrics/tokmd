//! W61 depth tests for `tokmd-analysis-fun`.
//!
//! Exercises eco-label edge cases, boundary transitions, notes formatting,
//! FunReport/EcoLabel serde contracts, determinism, property-based invariants,
//! and structural guarantees across all grade bands.

use proptest::prelude::*;
use tokmd_analysis_fun::build_fun_report;
use tokmd_analysis_types::{
    BoilerplateReport, DerivedReport, DerivedTotals, DistributionReport, EcoLabel, FileStatRow,
    FunReport, IntegrityReport, LangPurityReport, MaxFileReport, NestingReport, PolyglotReport,
    RateReport, RateRow, RatioReport, RatioRow, ReadingTimeReport, TestDensityReport, TodoReport,
    TopOffenders,
};

// ---------------------------------------------------------------------------
// Helper: build a DerivedReport with configurable parameters
// ---------------------------------------------------------------------------

fn derived_with(bytes: usize, files: usize, code: usize) -> DerivedReport {
    let row = FileStatRow {
        path: "main.rs".to_string(),
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
            overall: row.clone(),
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
            lang_count: 1,
            entropy: 0.0,
            dominant_lang: "Rust".to_string(),
            dominant_lines: code,
            dominant_pct: 100.0,
        },
        distribution: DistributionReport {
            count: files,
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
            largest_lines: vec![row.clone()],
            largest_tokens: vec![row.clone()],
            largest_bytes: vec![row.clone()],
            least_documented: vec![row.clone()],
            most_dense: vec![row],
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

fn derived_bytes(bytes: usize) -> DerivedReport {
    derived_with(bytes, 1, 1)
}

// ===========================================================================
// 1. Zero bytes → grade A with 95.0 score
// ===========================================================================
#[test]
fn zero_bytes_grade_a() {
    let eco = build_fun_report(&derived_bytes(0)).eco_label.unwrap();
    assert_eq!(eco.label, "A");
    assert_eq!(eco.score, 95.0);
    assert_eq!(eco.bytes, 0);
}

// ===========================================================================
// 2. One byte → still grade A
// ===========================================================================
#[test]
fn one_byte_grade_a() {
    let eco = build_fun_report(&derived_bytes(1)).eco_label.unwrap();
    assert_eq!(eco.label, "A");
}

// ===========================================================================
// 3. Boundary: exactly 1 MB → grade A
// ===========================================================================
#[test]
fn exactly_1mb_grade_a() {
    let eco = build_fun_report(&derived_bytes(1_048_576))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "A");
    assert_eq!(eco.score, 95.0);
}

// ===========================================================================
// 4. Boundary: 1 MB + 1 → grade B
// ===========================================================================
#[test]
fn one_byte_past_1mb_grade_b() {
    let eco = build_fun_report(&derived_bytes(1_048_577))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "B");
    assert_eq!(eco.score, 80.0);
}

// ===========================================================================
// 5. Boundary: exactly 10 MB → grade B
// ===========================================================================
#[test]
fn exactly_10mb_grade_b() {
    let eco = build_fun_report(&derived_bytes(10_485_760))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "B");
    assert_eq!(eco.score, 80.0);
}

// ===========================================================================
// 6. Boundary: 10 MB + 1 → grade C
// ===========================================================================
#[test]
fn one_byte_past_10mb_grade_c() {
    let eco = build_fun_report(&derived_bytes(10_485_761))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "C");
    assert_eq!(eco.score, 65.0);
}

// ===========================================================================
// 7. Boundary: exactly 50 MB → grade C
// ===========================================================================
#[test]
fn exactly_50mb_grade_c() {
    let eco = build_fun_report(&derived_bytes(52_428_800))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "C");
    assert_eq!(eco.score, 65.0);
}

// ===========================================================================
// 8. Boundary: 50 MB + 1 → grade D
// ===========================================================================
#[test]
fn one_byte_past_50mb_grade_d() {
    let eco = build_fun_report(&derived_bytes(52_428_801))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "D");
    assert_eq!(eco.score, 45.0);
}

// ===========================================================================
// 9. Boundary: exactly 200 MB → grade D
// ===========================================================================
#[test]
fn exactly_200mb_grade_d() {
    let eco = build_fun_report(&derived_bytes(209_715_200))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "D");
    assert_eq!(eco.score, 45.0);
}

// ===========================================================================
// 10. Boundary: 200 MB + 1 → grade E
// ===========================================================================
#[test]
fn one_byte_past_200mb_grade_e() {
    let eco = build_fun_report(&derived_bytes(209_715_201))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "E");
    assert_eq!(eco.score, 30.0);
}

// ===========================================================================
// 11. Very large: 2 GB → grade E
// ===========================================================================
#[test]
fn two_gb_grade_e() {
    let bytes = 2 * 1024 * 1024 * 1024;
    let eco = build_fun_report(&derived_bytes(bytes)).eco_label.unwrap();
    assert_eq!(eco.label, "E");
    assert_eq!(eco.score, 30.0);
}

// ===========================================================================
// 12. Eco label bytes field always matches input bytes
// ===========================================================================
#[test]
fn bytes_field_echoes_input() {
    for bytes in [0, 1, 999, 1_048_576, 52_428_800, 209_715_200, 500_000_000] {
        let eco = build_fun_report(&derived_bytes(bytes)).eco_label.unwrap();
        assert_eq!(eco.bytes, bytes as u64, "mismatch for {bytes}");
    }
}

// ===========================================================================
// 13. Notes prefix is always "Size-based eco label ("
// ===========================================================================
#[test]
fn notes_prefix_stable() {
    for bytes in [0, 1024, 10_000_000, 999_999_999] {
        let eco = build_fun_report(&derived_bytes(bytes)).eco_label.unwrap();
        assert!(
            eco.notes.starts_with("Size-based eco label ("),
            "bad prefix for {bytes}: {}",
            eco.notes
        );
    }
}

// ===========================================================================
// 14. Notes suffix is always " MB)"
// ===========================================================================
#[test]
fn notes_suffix_stable() {
    for bytes in [0, 1024, 10_000_000, 999_999_999] {
        let eco = build_fun_report(&derived_bytes(bytes)).eco_label.unwrap();
        assert!(
            eco.notes.ends_with(" MB)"),
            "bad suffix for {bytes}: {}",
            eco.notes
        );
    }
}

// ===========================================================================
// 15. Notes MB value matches manual computation
// ===========================================================================
#[test]
fn notes_mb_value_correct() {
    for bytes in [0, 524_288, 1_048_576, 10_485_760, 104_857_600] {
        let eco = build_fun_report(&derived_bytes(bytes)).eco_label.unwrap();
        let mb = bytes as f64 / (1024.0 * 1024.0);
        let rounded = (mb * 100.0).round() / 100.0;
        let expected = format!("Size-based eco label ({rounded} MB)");
        assert_eq!(eco.notes, expected, "mismatch for {bytes}");
    }
}

// ===========================================================================
// 16. Notes for 0 bytes shows "0 MB"
// ===========================================================================
#[test]
fn notes_zero_bytes_shows_zero_mb() {
    let eco = build_fun_report(&derived_bytes(0)).eco_label.unwrap();
    assert!(eco.notes.contains("0 MB"), "got: {}", eco.notes);
}

// ===========================================================================
// 17. Notes for 1.5 MB shows "1.5 MB"
// ===========================================================================
#[test]
fn notes_fractional_mb() {
    let eco = build_fun_report(&derived_bytes(1_572_864))
        .eco_label
        .unwrap();
    assert!(eco.notes.contains("1.5 MB"), "got: {}", eco.notes);
}

// ===========================================================================
// 18. Score monotonically decreases across all bands
// ===========================================================================
#[test]
fn score_monotone_across_bands() {
    let sizes = [
        0,
        1_048_577,   // B
        10_485_761,  // C
        52_428_801,  // D
        209_715_201, // E
    ];
    let scores: Vec<f64> = sizes
        .iter()
        .map(|&b| build_fun_report(&derived_bytes(b)).eco_label.unwrap().score)
        .collect();
    for w in scores.windows(2) {
        assert!(w[0] > w[1], "not strictly decreasing: {} -> {}", w[0], w[1]);
    }
}

// ===========================================================================
// 19. All five bands produce distinct labels
// ===========================================================================
#[test]
fn five_bands_distinct_labels() {
    let band_bytes = [0, 5_000_000, 25_000_000, 100_000_000, 500_000_000];
    let labels: Vec<String> = band_bytes
        .iter()
        .map(|&b| build_fun_report(&derived_bytes(b)).eco_label.unwrap().label)
        .collect();
    assert_eq!(labels, vec!["A", "B", "C", "D", "E"]);
}

// ===========================================================================
// 20. All five bands produce distinct scores
// ===========================================================================
#[test]
fn five_bands_distinct_scores() {
    let band_bytes = [0, 5_000_000, 25_000_000, 100_000_000, 500_000_000];
    let scores: Vec<f64> = band_bytes
        .iter()
        .map(|&b| build_fun_report(&derived_bytes(b)).eco_label.unwrap().score)
        .collect();
    for i in 0..scores.len() {
        for j in (i + 1)..scores.len() {
            assert_ne!(scores[i], scores[j], "scores {i} and {j} collide");
        }
    }
}

// ===========================================================================
// 21. Changing files/code but same bytes → same eco label
// ===========================================================================
#[test]
fn files_code_dont_affect_label() {
    let bytes = 20 * 1024 * 1024;
    let e1 = build_fun_report(&derived_with(bytes, 1, 10))
        .eco_label
        .unwrap();
    let e2 = build_fun_report(&derived_with(bytes, 10_000, 500_000))
        .eco_label
        .unwrap();
    assert_eq!(e1.label, e2.label);
    assert_eq!(e1.score, e2.score);
    assert_eq!(e1.bytes, e2.bytes);
    assert_eq!(e1.notes, e2.notes);
}

// ===========================================================================
// 22. FunReport serde roundtrip (with eco_label)
// ===========================================================================
#[test]
fn fun_report_serde_roundtrip() {
    let report = build_fun_report(&derived_bytes(7 * 1024 * 1024));
    let json = serde_json::to_string(&report).unwrap();
    let rt: FunReport = serde_json::from_str(&json).unwrap();
    let orig = report.eco_label.unwrap();
    let back = rt.eco_label.unwrap();
    assert_eq!(orig.label, back.label);
    assert_eq!(orig.score, back.score);
    assert_eq!(orig.bytes, back.bytes);
    assert_eq!(orig.notes, back.notes);
}

// ===========================================================================
// 23. FunReport serde roundtrip (None eco_label)
// ===========================================================================
#[test]
fn fun_report_none_roundtrip() {
    let report = FunReport { eco_label: None };
    let json = serde_json::to_string(&report).unwrap();
    let rt: FunReport = serde_json::from_str(&json).unwrap();
    assert!(rt.eco_label.is_none());
}

// ===========================================================================
// 24. EcoLabel standalone serde roundtrip
// ===========================================================================
#[test]
fn eco_label_standalone_roundtrip() {
    let label = EcoLabel {
        score: 65.0,
        label: "C".to_string(),
        bytes: 30_000_000,
        notes: "Size-based eco label (28.61 MB)".to_string(),
    };
    let json = serde_json::to_string(&label).unwrap();
    let rt: EcoLabel = serde_json::from_str(&json).unwrap();
    assert_eq!(rt.label, "C");
    assert_eq!(rt.score, 65.0);
    assert_eq!(rt.bytes, 30_000_000);
}

// ===========================================================================
// 25. JSON pretty-print roundtrips identically
// ===========================================================================
#[test]
fn json_pretty_roundtrip_stable() {
    let report = build_fun_report(&derived_bytes(42 * 1024 * 1024));
    let pretty = serde_json::to_string_pretty(&report).unwrap();
    let rt: FunReport = serde_json::from_str(&pretty).unwrap();
    let repretty = serde_json::to_string_pretty(&rt).unwrap();
    assert_eq!(pretty, repretty);
}

// ===========================================================================
// 26. JSON contains expected keys
// ===========================================================================
#[test]
fn json_has_expected_keys() {
    let report = build_fun_report(&derived_bytes(1024));
    let val: serde_json::Value = serde_json::to_value(&report).unwrap();
    assert!(val.get("eco_label").is_some());
    let eco = val.get("eco_label").unwrap();
    assert!(eco.get("score").is_some());
    assert!(eco.get("label").is_some());
    assert!(eco.get("bytes").is_some());
    assert!(eco.get("notes").is_some());
}

// ===========================================================================
// 27. JSON null for None eco_label
// ===========================================================================
#[test]
fn json_null_eco_label() {
    let val: serde_json::Value = serde_json::to_value(FunReport { eco_label: None }).unwrap();
    assert!(val["eco_label"].is_null());
}

// ===========================================================================
// 28. Deterministic: same input twice → identical JSON
// ===========================================================================
#[test]
fn deterministic_json() {
    let d = derived_bytes(12_345_678);
    let j1 = serde_json::to_string(&build_fun_report(&d)).unwrap();
    let j2 = serde_json::to_string(&build_fun_report(&d)).unwrap();
    assert_eq!(j1, j2);
}

// ===========================================================================
// 29. Label is always a single uppercase ASCII letter
// ===========================================================================
#[test]
fn label_single_uppercase() {
    for bytes in [0, 1_000_000, 10_000_001, 60_000_000, 300_000_000] {
        let eco = build_fun_report(&derived_bytes(bytes)).eco_label.unwrap();
        assert_eq!(eco.label.len(), 1);
        assert!(eco.label.chars().next().unwrap().is_ascii_uppercase());
    }
}

// ===========================================================================
// 30. Label is always one of A–E
// ===========================================================================
#[test]
fn label_in_known_set() {
    for bytes in [0, 500_000, 5_000_000, 30_000_000, 100_000_000, 500_000_000] {
        let eco = build_fun_report(&derived_bytes(bytes)).eco_label.unwrap();
        assert!(
            ["A", "B", "C", "D", "E"].contains(&eco.label.as_str()),
            "unexpected: {}",
            eco.label
        );
    }
}

// ===========================================================================
// 31. Score always in [0, 100] range
// ===========================================================================
#[test]
fn score_in_range() {
    for bytes in [0, 1, 1_000_000, 100_000_000, 1_000_000_000] {
        let eco = build_fun_report(&derived_bytes(bytes)).eco_label.unwrap();
        assert!(
            (0.0..=100.0).contains(&eco.score),
            "out of range: {}",
            eco.score
        );
    }
}

// ===========================================================================
// 32. eco_label always present (never None from build_fun_report)
// ===========================================================================
#[test]
fn eco_label_always_some() {
    for bytes in [0, 1, 100, 1_000_000, 1_000_000_000] {
        assert!(build_fun_report(&derived_bytes(bytes)).eco_label.is_some());
    }
}

// ===========================================================================
// 33. Notes for 1 GB shows "1024 MB"
// ===========================================================================
#[test]
fn notes_1gb() {
    let eco = build_fun_report(&derived_bytes(1024 * 1024 * 1024))
        .eco_label
        .unwrap();
    assert!(eco.notes.contains("1024 MB"), "got: {}", eco.notes);
}

// ===========================================================================
// 34. Mid-band B: 5 MB
// ===========================================================================
#[test]
fn mid_band_b() {
    let eco = build_fun_report(&derived_bytes(5 * 1024 * 1024))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "B");
    assert_eq!(eco.score, 80.0);
}

// ===========================================================================
// 35. Mid-band C: 30 MB
// ===========================================================================
#[test]
fn mid_band_c() {
    let eco = build_fun_report(&derived_bytes(30 * 1024 * 1024))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "C");
    assert_eq!(eco.score, 65.0);
}

// ===========================================================================
// 36. Mid-band D: 100 MB
// ===========================================================================
#[test]
fn mid_band_d() {
    let eco = build_fun_report(&derived_bytes(100 * 1024 * 1024))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "D");
    assert_eq!(eco.score, 45.0);
}

// ===========================================================================
// 37. Score exact values for all bands
// ===========================================================================
#[test]
fn score_exact_values_all_bands() {
    let cases: Vec<(usize, f64)> = vec![
        (0, 95.0),
        (5_000_000, 80.0),
        (25_000_000, 65.0),
        (100_000_000, 45.0),
        (500_000_000, 30.0),
    ];
    for (bytes, expected_score) in cases {
        let eco = build_fun_report(&derived_bytes(bytes)).eco_label.unwrap();
        assert!(
            (eco.score - expected_score).abs() < 1e-10,
            "score for {} bytes: expected {}, got {}",
            bytes,
            expected_score,
            eco.score
        );
    }
}

// ===========================================================================
// 38. Grade A has highest score of all grades
// ===========================================================================
#[test]
fn grade_a_highest_score() {
    let a_score = build_fun_report(&derived_bytes(0)).eco_label.unwrap().score;
    for bytes in [5_000_000, 25_000_000, 100_000_000, 500_000_000] {
        let s = build_fun_report(&derived_bytes(bytes))
            .eco_label
            .unwrap()
            .score;
        assert!(a_score > s);
    }
}

// ===========================================================================
// Property-based tests
// ===========================================================================

mod properties {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        /// eco_label is always present.
        #[test]
        fn eco_label_always_present(bytes in 0usize..=(2 * 1024 * 1024 * 1024)) {
            prop_assert!(build_fun_report(&derived_bytes(bytes)).eco_label.is_some());
        }

        /// bytes field always matches input.
        #[test]
        fn bytes_matches_input(bytes in 0usize..=(2 * 1024 * 1024 * 1024)) {
            let eco = build_fun_report(&derived_bytes(bytes)).eco_label.unwrap();
            prop_assert_eq!(eco.bytes, bytes as u64);
        }

        /// Score in [0, 100].
        #[test]
        fn score_in_valid_range(bytes in 0usize..=(2 * 1024 * 1024 * 1024)) {
            let eco = build_fun_report(&derived_bytes(bytes)).eco_label.unwrap();
            prop_assert!(eco.score >= 0.0 && eco.score <= 100.0);
        }

        /// Label is one of A–E.
        #[test]
        fn label_known_grade(bytes in 0usize..=(2 * 1024 * 1024 * 1024)) {
            let eco = build_fun_report(&derived_bytes(bytes)).eco_label.unwrap();
            prop_assert!(["A", "B", "C", "D", "E"].contains(&eco.label.as_str()));
        }

        /// Notes always contain "MB".
        #[test]
        fn notes_contain_mb(bytes in 0usize..=(2 * 1024 * 1024 * 1024)) {
            let eco = build_fun_report(&derived_bytes(bytes)).eco_label.unwrap();
            prop_assert!(eco.notes.contains("MB"));
        }

        /// Deterministic: same input → same output.
        #[test]
        fn deterministic(bytes in 0usize..=(2 * 1024 * 1024 * 1024)) {
            let d = derived_bytes(bytes);
            let e1 = build_fun_report(&d).eco_label.unwrap();
            let e2 = build_fun_report(&d).eco_label.unwrap();
            prop_assert_eq!(&e1.label, &e2.label);
            prop_assert_eq!(e1.score, e2.score);
            prop_assert_eq!(e1.bytes, e2.bytes);
            prop_assert_eq!(&e1.notes, &e2.notes);
        }

        /// Monotonicity: smaller bytes → higher or equal score.
        #[test]
        fn monotone_score(
            a in 0usize..=(1024 * 1024 * 1024),
            b in 0usize..=(1024 * 1024 * 1024),
        ) {
            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
            let s_lo = build_fun_report(&derived_bytes(lo)).eco_label.unwrap().score;
            let s_hi = build_fun_report(&derived_bytes(hi)).eco_label.unwrap().score;
            prop_assert!(s_lo >= s_hi);
        }

        /// Serde roundtrip preserves all fields.
        #[test]
        fn serde_roundtrip(bytes in 0usize..=(1024 * 1024 * 1024)) {
            let report = build_fun_report(&derived_bytes(bytes));
            let json = serde_json::to_string(&report).unwrap();
            let rt: FunReport = serde_json::from_str(&json).unwrap();
            let o = report.eco_label.unwrap();
            let r = rt.eco_label.unwrap();
            prop_assert_eq!(&o.label, &r.label);
            prop_assert_eq!(o.score, r.score);
            prop_assert_eq!(o.bytes, r.bytes);
            prop_assert_eq!(&o.notes, &r.notes);
        }

        /// Notes always start with fixed prefix.
        #[test]
        fn notes_prefix(bytes in 0usize..=(2 * 1024 * 1024 * 1024)) {
            let eco = build_fun_report(&derived_bytes(bytes)).eco_label.unwrap();
            prop_assert!(eco.notes.starts_with("Size-based eco label ("));
        }

        /// Notes always end with " MB)".
        #[test]
        fn notes_suffix(bytes in 0usize..=(2 * 1024 * 1024 * 1024)) {
            let eco = build_fun_report(&derived_bytes(bytes)).eco_label.unwrap();
            prop_assert!(eco.notes.ends_with(" MB)"));
        }
    }
}
