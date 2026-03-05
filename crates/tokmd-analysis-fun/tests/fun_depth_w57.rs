//! W57 depth tests for `tokmd-analysis-fun`.
//!
//! Exercises eco-label generation across a wide range of repo sizes and
//! compositions, serde roundtrips for all report sub-types, deterministic
//! output ordering, notes formatting edge cases, and property-based
//! monotonicity/invariant checks.

use tokmd_analysis_fun::build_fun_report;
use tokmd_analysis_types::{
    BoilerplateReport, DerivedReport, DerivedTotals, DistributionReport, EcoLabel, FileStatRow,
    FunReport, IntegrityReport, LangPurityReport, MaxFileReport, NestingReport, PolyglotReport,
    RateReport, RateRow, RatioReport, RatioRow, ReadingTimeReport, TestDensityReport, TodoReport,
    TopOffenders,
};

// ---------------------------------------------------------------------------
// Helper: build a DerivedReport with configurable bytes and file count
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
// 1. Empty repo (0 bytes) produces grade A
// ===========================================================================
#[test]
fn empty_repo_gives_grade_a() {
    let eco = build_fun_report(&derived_bytes(0)).eco_label.unwrap();
    assert_eq!(eco.label, "A");
    assert_eq!(eco.score, 95.0);
    assert_eq!(eco.bytes, 0);
}

// ===========================================================================
// 2. Single-file tiny repo
// ===========================================================================
#[test]
fn single_file_tiny_repo() {
    let eco = build_fun_report(&derived_with(256, 1, 10))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "A");
    assert_eq!(eco.bytes, 256);
}

// ===========================================================================
// 3. Large multi-file repo (50 MB range)
// ===========================================================================
#[test]
fn large_multi_file_repo_grade_c() {
    let bytes = 30 * 1024 * 1024;
    let eco = build_fun_report(&derived_with(bytes, 5000, 200_000))
        .eco_label
        .unwrap();
    assert_eq!(eco.label, "C");
    assert_eq!(eco.score, 65.0);
}

// ===========================================================================
// 4. Massive repo (500 MB) produces grade E
// ===========================================================================
#[test]
fn massive_repo_grade_e() {
    let bytes = 500 * 1024 * 1024;
    let eco = build_fun_report(&derived_bytes(bytes)).eco_label.unwrap();
    assert_eq!(eco.label, "E");
    assert_eq!(eco.score, 30.0);
}

// ===========================================================================
// 5. Eco-label always present regardless of bytes
// ===========================================================================
#[test]
fn eco_label_always_some() {
    for bytes in [0, 1, 100, 1_000_000, 1_000_000_000] {
        let report = build_fun_report(&derived_bytes(bytes));
        assert!(
            report.eco_label.is_some(),
            "eco_label missing for {bytes} bytes"
        );
    }
}

// ===========================================================================
// 6. Score monotonically decreases as bytes increase across all bands
// ===========================================================================
#[test]
fn score_monotone_decreasing_across_bands() {
    let sizes = [
        0,
        512 * 1024,
        2 * 1024 * 1024,
        15 * 1024 * 1024,
        100 * 1024 * 1024,
        300 * 1024 * 1024,
    ];
    let scores: Vec<f64> = sizes
        .iter()
        .map(|&b| build_fun_report(&derived_bytes(b)).eco_label.unwrap().score)
        .collect();
    for w in scores.windows(2) {
        assert!(w[0] >= w[1], "score not monotone: {} -> {}", w[0], w[1]);
    }
}

// ===========================================================================
// 7. FunReport serde roundtrip preserves all fields
// ===========================================================================
#[test]
fn fun_report_serde_roundtrip_all_fields() {
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
// 8. EcoLabel serde roundtrip standalone
// ===========================================================================
#[test]
fn eco_label_serde_roundtrip_standalone() {
    let label = EcoLabel {
        score: 80.0,
        label: "B".to_string(),
        bytes: 5_000_000,
        notes: "Size-based eco label (4.77 MB)".to_string(),
    };
    let json = serde_json::to_string(&label).unwrap();
    let rt: EcoLabel = serde_json::from_str(&json).unwrap();
    assert_eq!(label.label, rt.label);
    assert_eq!(label.score, rt.score);
    assert_eq!(label.bytes, rt.bytes);
    assert_eq!(label.notes, rt.notes);
}

// ===========================================================================
// 9. FunReport with None eco_label round-trips
// ===========================================================================
#[test]
fn fun_report_none_eco_label_roundtrip() {
    let report = FunReport { eco_label: None };
    let json = serde_json::to_string(&report).unwrap();
    let rt: FunReport = serde_json::from_str(&json).unwrap();
    assert!(rt.eco_label.is_none());
}

// ===========================================================================
// 10. JSON pretty-print round-trips identically
// ===========================================================================
#[test]
fn json_pretty_print_roundtrip() {
    let report = build_fun_report(&derived_bytes(42 * 1024 * 1024));
    let pretty = serde_json::to_string_pretty(&report).unwrap();
    let rt: FunReport = serde_json::from_str(&pretty).unwrap();
    let repretty = serde_json::to_string_pretty(&rt).unwrap();
    assert_eq!(pretty, repretty);
}

// ===========================================================================
// 11. Notes contain MB substring for every band
// ===========================================================================
#[test]
fn notes_always_contain_mb() {
    let sizes = [0, 500, 1_000_000, 50_000_000, 300_000_000];
    for &b in &sizes {
        let eco = build_fun_report(&derived_bytes(b)).eco_label.unwrap();
        assert!(eco.notes.contains("MB"), "no MB in notes for {b} bytes");
    }
}

// ===========================================================================
// 12. Notes prefix is constant
// ===========================================================================
#[test]
fn notes_prefix_constant() {
    for &b in &[0usize, 1024, 1_000_000, 999_999_999] {
        let eco = build_fun_report(&derived_bytes(b)).eco_label.unwrap();
        assert!(
            eco.notes.starts_with("Size-based eco label ("),
            "prefix wrong for {b}: {}",
            eco.notes
        );
    }
}

// ===========================================================================
// 13. Notes suffix is constant
// ===========================================================================
#[test]
fn notes_suffix_constant() {
    for &b in &[0usize, 1024, 1_000_000, 999_999_999] {
        let eco = build_fun_report(&derived_bytes(b)).eco_label.unwrap();
        assert!(
            eco.notes.ends_with(" MB)"),
            "suffix wrong for {b}: {}",
            eco.notes
        );
    }
}

// ===========================================================================
// 14. Bytes field always matches input
// ===========================================================================
#[test]
fn bytes_field_echoes_input() {
    for &b in &[0usize, 1, 1024, 999_999, 1_000_000_000] {
        let eco = build_fun_report(&derived_bytes(b)).eco_label.unwrap();
        assert_eq!(eco.bytes, b as u64);
    }
}

// ===========================================================================
// 15. Deterministic: identical input always yields identical JSON
// ===========================================================================
#[test]
fn deterministic_json_output() {
    let d = derived_bytes(12_345_678);
    let j1 = serde_json::to_string(&build_fun_report(&d)).unwrap();
    let j2 = serde_json::to_string(&build_fun_report(&d)).unwrap();
    assert_eq!(j1, j2);
}

// ===========================================================================
// 16. All five bands produce distinct (label, score) pairs
// ===========================================================================
#[test]
fn five_bands_distinct() {
    let band_bytes = [
        0usize,            // A
        5 * 1024 * 1024,   // B
        25 * 1024 * 1024,  // C
        100 * 1024 * 1024, // D
        500 * 1024 * 1024, // E
    ];
    let pairs: Vec<(String, f64)> = band_bytes
        .iter()
        .map(|&b| {
            let e = build_fun_report(&derived_bytes(b)).eco_label.unwrap();
            (e.label, e.score)
        })
        .collect();
    // All distinct
    for i in 0..pairs.len() {
        for j in (i + 1)..pairs.len() {
            assert_ne!(pairs[i], pairs[j], "bands {i} and {j} collide");
        }
    }
}

// ===========================================================================
// 17. Grade A score is highest
// ===========================================================================
#[test]
fn grade_a_has_highest_score() {
    let a_score = build_fun_report(&derived_bytes(0)).eco_label.unwrap().score;
    for &b in &[5_000_000usize, 25_000_000, 100_000_000, 500_000_000] {
        let s = build_fun_report(&derived_bytes(b)).eco_label.unwrap().score;
        assert!(a_score > s, "A score {} not > {}", a_score, s);
    }
}

// ===========================================================================
// 18. Grade E score is lowest
// ===========================================================================
#[test]
fn grade_e_has_lowest_score() {
    let e_score = build_fun_report(&derived_bytes(500_000_000))
        .eco_label
        .unwrap()
        .score;
    for &b in &[0usize, 5_000_000, 25_000_000, 100_000_000] {
        let s = build_fun_report(&derived_bytes(b)).eco_label.unwrap().score;
        assert!(e_score < s, "E score {} not < {}", e_score, s);
    }
}

// ===========================================================================
// 19. Notes MB value matches computed rounding
// ===========================================================================
#[test]
fn notes_mb_value_matches_computation() {
    for &bytes in &[0usize, 1, 524_288, 1_048_576, 10_485_760, 104_857_600] {
        let eco = build_fun_report(&derived_bytes(bytes)).eco_label.unwrap();
        let mb = bytes as f64 / (1024.0 * 1024.0);
        let rounded = (mb * 100.0).round() / 100.0;
        let expected = format!("Size-based eco label ({rounded} MB)");
        assert_eq!(eco.notes, expected, "notes mismatch for {bytes} bytes");
    }
}

// ===========================================================================
// 20. Changing file count but same bytes produces same eco-label
// ===========================================================================
#[test]
fn file_count_does_not_affect_eco_label() {
    let bytes = 5 * 1024 * 1024;
    let e1 = build_fun_report(&derived_with(bytes, 1, 100))
        .eco_label
        .unwrap();
    let e2 = build_fun_report(&derived_with(bytes, 10_000, 500_000))
        .eco_label
        .unwrap();
    assert_eq!(e1.label, e2.label);
    assert_eq!(e1.score, e2.score);
    assert_eq!(e1.bytes, e2.bytes);
}

// ===========================================================================
// 21. Changing code count but same bytes produces same eco-label
// ===========================================================================
#[test]
fn code_count_does_not_affect_eco_label() {
    let bytes = 20 * 1024 * 1024;
    let e1 = build_fun_report(&derived_with(bytes, 100, 10))
        .eco_label
        .unwrap();
    let e2 = build_fun_report(&derived_with(bytes, 100, 1_000_000))
        .eco_label
        .unwrap();
    assert_eq!(e1.label, e2.label);
    assert_eq!(e1.score, e2.score);
}

// ===========================================================================
// 22. JSON contains expected keys
// ===========================================================================
#[test]
fn json_contains_expected_keys() {
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
// 23. JSON null eco_label when None
// ===========================================================================
#[test]
fn json_null_eco_label() {
    let report = FunReport { eco_label: None };
    let val: serde_json::Value = serde_json::to_value(&report).unwrap();
    assert!(val.get("eco_label").unwrap().is_null());
}

// ===========================================================================
// 24. Boundary: exactly at each MB threshold
// ===========================================================================
#[test]
fn boundary_exact_thresholds() {
    let cases: Vec<(usize, &str)> = vec![
        (1_048_576, "A"),   // exactly 1 MB
        (10_485_760, "B"),  // exactly 10 MB
        (52_428_800, "C"),  // exactly 50 MB
        (209_715_200, "D"), // exactly 200 MB
    ];
    for (bytes, expected) in cases {
        let eco = build_fun_report(&derived_bytes(bytes)).eco_label.unwrap();
        assert_eq!(eco.label, expected, "wrong label at {} bytes", bytes);
    }
}

// ===========================================================================
// 25. Boundary: one byte past each threshold transitions to next band
// ===========================================================================
#[test]
fn boundary_one_past_threshold() {
    let cases: Vec<(usize, &str)> = vec![
        (1_048_577, "B"),   // 1 MB + 1
        (10_485_761, "C"),  // 10 MB + 1
        (52_428_801, "D"),  // 50 MB + 1
        (209_715_201, "E"), // 200 MB + 1
    ];
    for (bytes, expected) in cases {
        let eco = build_fun_report(&derived_bytes(bytes)).eco_label.unwrap();
        assert_eq!(eco.label, expected, "wrong label at {} bytes", bytes);
    }
}

// ===========================================================================
// 26. Score values are exactly the documented constants
// ===========================================================================
#[test]
fn score_exact_values() {
    let map: Vec<(&str, f64)> = vec![
        ("A", 95.0),
        ("B", 80.0),
        ("C", 65.0),
        ("D", 45.0),
        ("E", 30.0),
    ];
    let byte_samples = [0, 5_000_000, 25_000_000, 100_000_000, 500_000_000];
    for (&bytes, (expected_label, expected_score)) in byte_samples.iter().zip(map.iter()) {
        let eco = build_fun_report(&derived_bytes(bytes)).eco_label.unwrap();
        assert_eq!(eco.label, *expected_label);
        assert!((eco.score - expected_score).abs() < f64::EPSILON);
    }
}

// ===========================================================================
// 27. Notes with very large repos show correct MB value
// ===========================================================================
#[test]
fn notes_very_large_repo() {
    let bytes = 1024 * 1024 * 1024; // 1 GB
    let eco = build_fun_report(&derived_bytes(bytes)).eco_label.unwrap();
    assert!(eco.notes.contains("1024 MB"), "got: {}", eco.notes);
}

// ===========================================================================
// Property-based tests
// ===========================================================================

mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Every generated report round-trips through JSON.
        #[test]
        fn serde_roundtrip_arbitrary(bytes in 0usize..=(1024 * 1024 * 1024)) {
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

        /// Label is always one character long.
        #[test]
        fn label_single_char(bytes in 0usize..=(1024 * 1024 * 1024)) {
            let eco = build_fun_report(&derived_bytes(bytes)).eco_label.unwrap();
            prop_assert_eq!(eco.label.len(), 1);
        }

        /// Score is finite and positive.
        #[test]
        fn score_finite_positive(bytes in 0usize..=(1024 * 1024 * 1024)) {
            let eco = build_fun_report(&derived_bytes(bytes)).eco_label.unwrap();
            prop_assert!(eco.score.is_finite());
            prop_assert!(eco.score > 0.0);
        }
    }
}
