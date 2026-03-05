//! W71 deep tests for `tokmd-analysis-fun`.
//!
//! Fills gaps left by earlier waves: JSON round-trip fidelity, monotonic
//! score ordering, rounding precision edge cases, snapshot coverage, and
//! structural invariants of the `FunReport` envelope.

use tokmd_analysis_fun::build_fun_report;
use tokmd_analysis_types::{
    BoilerplateReport, DerivedReport, DerivedTotals, DistributionReport, FileStatRow,
    IntegrityReport, LangPurityReport, MaxFileReport, NestingReport, PolyglotReport, RateReport,
    RateRow, RatioReport, RatioRow, ReadingTimeReport, TestDensityReport, TodoReport,
    TopOffenders,
};

// ── Helper ──────────────────────────────────────────────────────

fn derived_with_bytes(bytes: usize) -> DerivedReport {
    let row = FileStatRow {
        path: "f.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        code: 1,
        comments: 0,
        blanks: 0,
        lines: 1,
        bytes,
        tokens: 1,
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
            total: RatioRow { key: "All".into(), numerator: 0, denominator: 1, ratio: 0.0 },
            by_lang: vec![],
            by_module: vec![],
        },
        whitespace: RatioReport {
            total: RatioRow { key: "All".into(), numerator: 0, denominator: 1, ratio: 0.0 },
            by_lang: vec![],
            by_module: vec![],
        },
        verbosity: RateReport {
            total: RateRow { key: "All".into(), numerator: 0, denominator: 1, rate: 0.0 },
            by_lang: vec![],
            by_module: vec![],
        },
        max_file: MaxFileReport {
            overall: row.clone(),
            by_lang: vec![],
            by_module: vec![],
        },
        lang_purity: LangPurityReport { rows: vec![] },
        nesting: NestingReport { max: 0, avg: 0.0, by_module: vec![] },
        test_density: TestDensityReport {
            test_lines: 0, prod_lines: 0, test_files: 0, prod_files: 0, ratio: 0.0,
        },
        boilerplate: BoilerplateReport {
            infra_lines: 0, logic_lines: 0, ratio: 0.0, infra_langs: vec![],
        },
        polyglot: PolyglotReport {
            lang_count: 0, entropy: 0.0, dominant_lang: "unknown".into(),
            dominant_lines: 0, dominant_pct: 0.0,
        },
        distribution: DistributionReport {
            count: 1, min: 1, max: 1, mean: 0.0, median: 0.0,
            p90: 0.0, p99: 0.0, gini: 0.0,
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
        reading_time: ReadingTimeReport { minutes: 0.0, lines_per_minute: 0, basis_lines: 0 },
        context_window: None,
        cocomo: None,
        todo: Some(TodoReport { total: 0, density_per_kloc: 0.0, tags: vec![] }),
        integrity: IntegrityReport {
            algo: "sha1".into(), hash: "placeholder".into(), entries: 0,
        },
    }
}

// =========================================================================
// 1. Monotonic score ordering across all five bands
// =========================================================================

#[test]
fn scores_decrease_monotonically_across_bands() {
    // Pick one representative from each band: A, B, C, D, E
    let samples: [(usize, &str); 5] = [
        (512 * 1024, "A"),       // 0.5 MB
        (5 * 1024 * 1024, "B"),  // 5 MB
        (30 * 1024 * 1024, "C"), // 30 MB
        (100 * 1024 * 1024, "D"), // 100 MB
        (300 * 1024 * 1024, "E"), // 300 MB
    ];

    let mut prev_score = f64::MAX;
    for (bytes, expected_label) in &samples {
        let r = build_fun_report(&derived_with_bytes(*bytes));
        let eco = r.eco_label.unwrap();
        assert_eq!(&eco.label, *expected_label, "wrong label for {bytes} bytes");
        assert!(
            eco.score < prev_score,
            "score for {expected_label} ({}) should be less than previous ({prev_score})",
            eco.score,
        );
        prev_score = eco.score;
    }
}

// =========================================================================
// 2. All five grades have unique, distinct scores
// =========================================================================

#[test]
fn all_grades_have_distinct_scores() {
    let byte_samples = [0usize, 2 * 1024 * 1024, 20 * 1024 * 1024, 80 * 1024 * 1024, 300 * 1024 * 1024];
    let scores: Vec<u64> = byte_samples
        .iter()
        .map(|b| {
            let eco = build_fun_report(&derived_with_bytes(*b)).eco_label.unwrap();
            eco.score.to_bits()
        })
        .collect();

    // All scores should be unique (no two bands share a score)
    let mut unique = scores.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), scores.len(), "each band must have a unique score");
}

// =========================================================================
// 3. JSON round-trip fidelity
// =========================================================================

#[test]
fn json_round_trip_preserves_all_fields() {
    let r = build_fun_report(&derived_with_bytes(7 * 1024 * 1024));
    let json = serde_json::to_string(&r).unwrap();
    let deserialized: serde_json::Value = serde_json::from_str(&json).unwrap();

    let eco = deserialized["eco_label"].as_object().unwrap();
    assert!(eco.contains_key("score"), "round-trip must preserve 'score'");
    assert!(eco.contains_key("label"), "round-trip must preserve 'label'");
    assert!(eco.contains_key("bytes"), "round-trip must preserve 'bytes'");
    assert!(eco.contains_key("notes"), "round-trip must preserve 'notes'");
    assert_eq!(eco.len(), 4, "FunReport.eco_label should have exactly 4 fields");
}

#[test]
fn json_round_trip_values_match() {
    let bytes = 15 * 1024 * 1024; // 15 MB → band C
    let r = build_fun_report(&derived_with_bytes(bytes));
    let json = serde_json::to_string(&r).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(v["eco_label"]["label"].as_str().unwrap(), "C");
    assert_eq!(v["eco_label"]["score"].as_f64().unwrap(), 65.0);
    assert_eq!(v["eco_label"]["bytes"].as_u64().unwrap(), bytes as u64);
}

// =========================================================================
// 4. Score is always in the (0, 100] range
// =========================================================================

#[test]
fn score_within_zero_to_hundred() {
    for bytes in [0, 1, 100, 1024, 1024 * 1024, 50 * 1024 * 1024, 500 * 1024 * 1024, usize::MAX / 2] {
        let eco = build_fun_report(&derived_with_bytes(bytes)).eco_label.unwrap();
        assert!(eco.score > 0.0, "score must be positive for bytes={bytes}");
        assert!(eco.score <= 100.0, "score must be ≤ 100 for bytes={bytes}");
    }
}

// =========================================================================
// 5. Notes format is exactly "Size-based eco label (X.XX MB)"
// =========================================================================

#[test]
fn notes_format_matches_expected_pattern() {
    for bytes in [0, 512, 1024 * 1024, 99 * 1024 * 1024] {
        let eco = build_fun_report(&derived_with_bytes(bytes)).eco_label.unwrap();
        assert!(eco.notes.starts_with("Size-based eco label ("), "notes prefix wrong for {bytes}");
        assert!(eco.notes.ends_with(" MB)"), "notes suffix wrong for {bytes}");
    }
}

// =========================================================================
// 6. Rounding precision edge cases
// =========================================================================

#[test]
fn rounding_one_third_mb() {
    // 1/3 MB ≈ 349525.33... bytes → 0.33 MB after rounding
    let bytes = 349525;
    let eco = build_fun_report(&derived_with_bytes(bytes)).eco_label.unwrap();
    // The MB value in parentheses should be rounded to 2 decimal places
    let paren = eco.notes.find('(').unwrap() + 1;
    let end = eco.notes.find(" MB)").unwrap();
    let mb_str = &eco.notes[paren..end];
    let mb_val: f64 = mb_str.parse().expect("MB value should be a valid float");
    // Verify it has at most 2 decimal places
    let rounded = (mb_val * 100.0).round() / 100.0;
    assert!(
        (mb_val - rounded).abs() < f64::EPSILON,
        "MB value {mb_val} should have at most 2 decimal places",
    );
}

#[test]
fn rounding_exact_megabyte() {
    let bytes = 1024 * 1024; // exactly 1 MB
    let eco = build_fun_report(&derived_with_bytes(bytes)).eco_label.unwrap();
    assert!(eco.notes.contains("(1 MB)"), "exact 1 MB should show as '(1 MB)', got: {}", eco.notes);
}

// =========================================================================
// 7. Snapshot test of representative eco labels per band
// =========================================================================

#[test]
fn snapshot_all_bands() {
    let bands = [
        ("A", 100),
        ("B", 5 * 1024 * 1024),
        ("C", 25 * 1024 * 1024),
        ("D", 120 * 1024 * 1024),
        ("E", 500 * 1024 * 1024),
    ];

    for (expected_label, bytes) in &bands {
        let r = build_fun_report(&derived_with_bytes(*bytes));
        let eco = r.eco_label.as_ref().unwrap();
        assert_eq!(&eco.label, *expected_label);
        // Verify the JSON output is well-formed
        let json = serde_json::to_string_pretty(&r).unwrap();
        assert!(!json.is_empty());
        // Verify re-parse succeeds
        let _: serde_json::Value = serde_json::from_str(&json).unwrap();
    }
}

// =========================================================================
// 8. bytes field faithfully reflects input totals
// =========================================================================

#[test]
fn bytes_field_is_exact() {
    for bytes in [0usize, 1, 42, 999_999, 1_073_741_824] {
        let eco = build_fun_report(&derived_with_bytes(bytes)).eco_label.unwrap();
        assert_eq!(eco.bytes, bytes as u64, "bytes mismatch for input {bytes}");
    }
}

// =========================================================================
// 9. Determinism: byte-identical JSON across 100 iterations
// =========================================================================

#[test]
fn determinism_100_iterations() {
    let derived = derived_with_bytes(42 * 1024 * 1024);
    let baseline = serde_json::to_string(&build_fun_report(&derived)).unwrap();
    for i in 0..100 {
        let json = serde_json::to_string(&build_fun_report(&derived)).unwrap();
        assert_eq!(json, baseline, "non-deterministic output on iteration {i}");
    }
}

// =========================================================================
// 10. Very large byte values don't panic or overflow
// =========================================================================

#[test]
fn large_byte_value_no_panic() {
    // Near-max usize (but safe for the DerivedReport construction)
    let bytes = usize::MAX / 4;
    let r = build_fun_report(&derived_with_bytes(bytes));
    let eco = r.eco_label.unwrap();
    assert_eq!(eco.label, "E", "very large repos should be grade E");
    assert_eq!(eco.score, 30.0);
    assert_eq!(eco.bytes, bytes as u64);
}

// =========================================================================
// 11. FunReport top-level structure has exactly one field
// =========================================================================

#[test]
fn fun_report_top_level_has_one_field() {
    let r = build_fun_report(&derived_with_bytes(1024));
    let json: serde_json::Value = serde_json::from_str(&serde_json::to_string(&r).unwrap()).unwrap();
    let obj = json.as_object().unwrap();
    assert_eq!(obj.len(), 1, "FunReport should serialize to exactly one top-level key");
    assert!(obj.contains_key("eco_label"));
}

// =========================================================================
// 12. Label is always a single uppercase ASCII letter in A-E
// =========================================================================

#[test]
fn label_is_valid_uppercase_a_through_e() {
    let valid = ["A", "B", "C", "D", "E"];
    let samples = [0, 500_000, 3 * 1024 * 1024, 15 * 1024 * 1024, 75 * 1024 * 1024, 250 * 1024 * 1024];
    for bytes in samples {
        let eco = build_fun_report(&derived_with_bytes(bytes)).eco_label.unwrap();
        assert!(
            valid.contains(&eco.label.as_str()),
            "label '{}' is not in valid set for bytes={bytes}",
            eco.label,
        );
    }
}

// =========================================================================
// 13. Band boundary inclusivity: upper bound belongs to lower band
// =========================================================================

#[test]
fn band_boundaries_inclusive_upper() {
    // Each threshold belongs to the lower band (≤ check)
    let cases: [(usize, &str); 4] = [
        (1 * 1024 * 1024, "A"),    // 1 MB → still A
        (10 * 1024 * 1024, "B"),   // 10 MB → still B
        (50 * 1024 * 1024, "C"),   // 50 MB → still C
        (200 * 1024 * 1024, "D"),  // 200 MB → still D
    ];
    for (bytes, expected) in &cases {
        let eco = build_fun_report(&derived_with_bytes(*bytes)).eco_label.unwrap();
        assert_eq!(
            &eco.label, *expected,
            "boundary value {bytes} bytes should map to band {expected}",
        );
    }
}

// =========================================================================
// 14. eco_label is never None
// =========================================================================

#[test]
fn eco_label_never_none() {
    // Exhaustively test: all five bands plus edge values
    let samples = [
        0, 1, 100, 1024, 1024 * 1024, 1024 * 1024 + 1,
        10 * 1024 * 1024, 10 * 1024 * 1024 + 1,
        50 * 1024 * 1024, 50 * 1024 * 1024 + 1,
        200 * 1024 * 1024, 200 * 1024 * 1024 + 1,
        usize::MAX / 8,
    ];
    for bytes in samples {
        let r = build_fun_report(&derived_with_bytes(bytes));
        assert!(r.eco_label.is_some(), "eco_label must be Some for bytes={bytes}");
    }
}

// =========================================================================
// 15. Notes MB value matches bytes / (1024*1024) rounded to 2 decimals
// =========================================================================

#[test]
fn notes_mb_matches_computed_value() {
    for bytes in [0usize, 512, 768 * 1024, 3 * 1024 * 1024 + 512 * 1024] {
        let eco = build_fun_report(&derived_with_bytes(bytes)).eco_label.unwrap();
        let expected_mb = (bytes as f64 / (1024.0 * 1024.0) * 100.0).round() / 100.0;
        let expected_notes = format!("Size-based eco label ({expected_mb} MB)");
        assert_eq!(
            eco.notes, expected_notes,
            "notes mismatch for bytes={bytes}",
        );
    }
}
