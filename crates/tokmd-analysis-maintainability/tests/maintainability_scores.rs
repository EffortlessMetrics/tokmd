//! Deeper tests for maintainability index calculations: known patterns,
//! edge cases, and score behavior.

use tokmd_analysis_maintainability::{attach_halstead_metrics, compute_maintainability_index};
use tokmd_analysis_types::{
    ComplexityReport, ComplexityRisk, FileComplexity, HalsteadMetrics, TechnicalDebtLevel,
    TechnicalDebtRatio,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_halstead(volume: f64) -> HalsteadMetrics {
    HalsteadMetrics {
        distinct_operators: 10,
        distinct_operands: 20,
        total_operators: 60,
        total_operands: 120,
        vocabulary: 30,
        length: 180,
        volume,
        difficulty: 5.0,
        effort: 500.0,
        time_seconds: 27.78,
        estimated_bugs: 0.05,
    }
}

fn sample_complexity_with_mi(cc: f64, loc: f64) -> ComplexityReport {
    ComplexityReport {
        total_functions: 5,
        avg_function_length: loc / 5.0,
        max_function_length: loc as usize,
        avg_cyclomatic: cc,
        max_cyclomatic: cc as usize + 5,
        avg_cognitive: None,
        max_cognitive: None,
        avg_nesting_depth: None,
        max_nesting_depth: None,
        high_risk_files: 0,
        histogram: None,
        halstead: None,
        maintainability_index: compute_maintainability_index(cc, loc, None),
        technical_debt: Some(TechnicalDebtRatio {
            ratio: 10.0,
            complexity_points: 10,
            code_kloc: 1.0,
            level: TechnicalDebtLevel::Low,
        }),
        files: vec![FileComplexity {
            path: "src/lib.rs".to_string(),
            module: "src".to_string(),
            function_count: 5,
            max_function_length: loc as usize,
            cyclomatic_complexity: cc as usize + 5,
            cognitive_complexity: None,
            max_nesting: None,
            risk_level: ComplexityRisk::Low,
            functions: None,
        }],
    }
}

// ===========================================================================
// High maintainability: well-factored, small, low complexity
// ===========================================================================

#[test]
fn high_maintainability_small_functions_low_complexity() {
    // Small codebase with minimal complexity → grade A
    let mi = compute_maintainability_index(2.0, 20.0, None).expect("mi");
    assert_eq!(mi.grade, "A");
    assert!(mi.score > 100.0, "well-factored code should score > 100");
}

#[test]
fn high_maintainability_with_small_halstead_volume() {
    // Small Halstead volume barely impacts the score
    let simplified = compute_maintainability_index(2.0, 20.0, None).expect("simplified");
    let with_small_vol = compute_maintainability_index(2.0, 20.0, Some(10.0)).expect("full");

    assert_eq!(with_small_vol.grade, "A");
    // Full formula reduces score, but small volume keeps it high
    assert!(with_small_vol.score > 100.0);
    assert!(with_small_vol.score < simplified.score);
}

// ===========================================================================
// Medium maintainability: moderate complexity and size
// ===========================================================================

#[test]
fn medium_maintainability_moderate_complexity() {
    // CC=10, LOC=100, V=200 → known value ≈ 66.54, grade B
    let mi = compute_maintainability_index(10.0, 100.0, Some(200.0)).expect("mi");
    assert_eq!(mi.grade, "B");
    assert!(mi.score >= 65.0 && mi.score < 85.0);
}

#[test]
fn medium_maintainability_higher_cc_no_halstead() {
    // CC=30, LOC=500 → simplified MI = 171 - 6.9 - 100.67 ≈ 63.43 → grade C
    // CC=20, LOC=200 → simplified MI = 171 - 4.6 - 85.79 ≈ 80.61 → grade B
    let mi = compute_maintainability_index(20.0, 200.0, None).expect("mi");
    assert_eq!(mi.grade, "B");
}

// ===========================================================================
// Low maintainability: high complexity, large codebase
// ===========================================================================

#[test]
fn low_maintainability_high_complexity_large_codebase() {
    let mi = compute_maintainability_index(80.0, 5000.0, Some(50000.0)).expect("mi");
    assert_eq!(mi.grade, "C");
    assert!(mi.score < 65.0);
}

#[test]
fn low_maintainability_extreme_loc() {
    let mi = compute_maintainability_index(5.0, 1_000_000.0, None).expect("mi");
    // MI = 171 - 1.15 - 16.2 * ln(1000000) ≈ 171 - 1.15 - 223.7 → clamped to 0
    assert_eq!(mi.score, 0.0);
    assert_eq!(mi.grade, "C");
}

// ===========================================================================
// Edge case: LOC = 0 or negative → None
// ===========================================================================

#[test]
fn zero_loc_returns_none() {
    assert!(compute_maintainability_index(10.0, 0.0, None).is_none());
    assert!(compute_maintainability_index(10.0, 0.0, Some(100.0)).is_none());
}

#[test]
fn negative_loc_returns_none() {
    assert!(compute_maintainability_index(10.0, -1.0, None).is_none());
    assert!(compute_maintainability_index(0.0, -100.0, Some(50.0)).is_none());
}

// ===========================================================================
// Edge case: very large values → score clamped at 0
// ===========================================================================

#[test]
fn extreme_values_clamp_score_to_zero() {
    let mi = compute_maintainability_index(100_000.0, 1e12, Some(1e12)).expect("mi");
    assert_eq!(mi.score, 0.0);
    assert_eq!(mi.grade, "C");
}

// ===========================================================================
// Edge case: CC = 0, LOC = 1 → maximum possible MI (171.0)
// ===========================================================================

#[test]
fn minimal_input_yields_maximum_mi() {
    let mi = compute_maintainability_index(0.0, 1.0, None).expect("mi");
    // MI = 171 - 0 - 16.2 * ln(1) = 171 - 0 - 0 = 171
    assert_eq!(mi.score, 171.0);
    assert_eq!(mi.grade, "A");
}

// ===========================================================================
// Halstead volume = 0 or negative → falls back to simplified formula
// ===========================================================================

#[test]
fn zero_halstead_volume_uses_simplified() {
    let simplified = compute_maintainability_index(10.0, 100.0, None).expect("simplified");
    let zero_vol = compute_maintainability_index(10.0, 100.0, Some(0.0)).expect("zero");

    assert_eq!(simplified.score, zero_vol.score);
    assert_eq!(zero_vol.avg_halstead_volume, None);
}

#[test]
fn negative_halstead_volume_uses_simplified() {
    let simplified = compute_maintainability_index(10.0, 100.0, None).expect("simplified");
    let neg_vol = compute_maintainability_index(10.0, 100.0, Some(-50.0)).expect("negative");

    assert_eq!(simplified.score, neg_vol.score);
    assert_eq!(neg_vol.avg_halstead_volume, None);
}

// ===========================================================================
// Grade boundaries: A ≥ 85, B ≥ 65, C < 65
// ===========================================================================

#[test]
fn grade_a_boundary() {
    // Find inputs that produce score just above 85
    // MI = 171 - 0.23*CC - 16.2*ln(LOC)
    // With LOC=100: MI = 171 - 0.23*CC - 74.6 = 96.4 - 0.23*CC
    // For MI=85: CC = (96.4-85)/0.23 ≈ 49.6 → CC=49 gives A, CC=50 might give B
    let mi_a = compute_maintainability_index(49.0, 100.0, None).expect("mi");
    assert_eq!(mi_a.grade, "A");
    assert!(mi_a.score >= 85.0);
}

#[test]
fn grade_b_boundary() {
    // With LOC=100: MI = 96.4 - 0.23*CC
    // For MI=65: CC = (96.4-65)/0.23 ≈ 136.5 → CC=136 gives B
    let mi_b = compute_maintainability_index(136.0, 100.0, None).expect("mi");
    assert_eq!(mi_b.grade, "B");
    assert!(mi_b.score >= 65.0 && mi_b.score < 85.0);
}

#[test]
fn grade_c_boundary() {
    // CC=137 with LOC=100 → MI = 96.4 - 31.51 = 64.89 → grade C
    let mi_c = compute_maintainability_index(137.0, 100.0, None).expect("mi");
    assert_eq!(mi_c.grade, "C");
    assert!(mi_c.score < 65.0);
}

// ===========================================================================
// Monotonicity: increasing CC → decreasing score
// ===========================================================================

#[test]
fn score_decreases_with_increasing_cc() {
    let scores: Vec<f64> = [1.0, 5.0, 20.0, 50.0, 100.0, 200.0]
        .iter()
        .map(|&cc| {
            compute_maintainability_index(cc, 100.0, None)
                .unwrap()
                .score
        })
        .collect();

    for window in scores.windows(2) {
        assert!(
            window[0] >= window[1],
            "score should decrease: {} >= {}",
            window[0],
            window[1]
        );
    }
}

// ===========================================================================
// Monotonicity: increasing LOC → decreasing score
// ===========================================================================

#[test]
fn score_decreases_with_increasing_loc() {
    let scores: Vec<f64> = [1.0, 10.0, 100.0, 1000.0, 10000.0]
        .iter()
        .map(|&loc| compute_maintainability_index(5.0, loc, None).unwrap().score)
        .collect();

    for window in scores.windows(2) {
        assert!(
            window[0] >= window[1],
            "score should decrease: {} >= {}",
            window[0],
            window[1]
        );
    }
}

// ===========================================================================
// Monotonicity: increasing Halstead volume → decreasing score
// ===========================================================================

#[test]
fn score_decreases_with_increasing_halstead_volume() {
    let scores: Vec<f64> = [1.0, 10.0, 100.0, 1000.0, 10000.0]
        .iter()
        .map(|&vol| {
            compute_maintainability_index(5.0, 100.0, Some(vol))
                .unwrap()
                .score
        })
        .collect();

    for window in scores.windows(2) {
        assert!(
            window[0] >= window[1],
            "score should decrease: {} >= {}",
            window[0],
            window[1]
        );
    }
}

// ===========================================================================
// Full formula always produces lower score than simplified (same CC, LOC)
// ===========================================================================

#[test]
fn full_formula_always_lower_than_simplified() {
    let test_cases = vec![
        (1.0, 10.0, 50.0),
        (10.0, 100.0, 200.0),
        (5.0, 500.0, 1000.0),
        (20.0, 50.0, 100.0),
    ];
    for (cc, loc, vol) in test_cases {
        let simplified = compute_maintainability_index(cc, loc, None).unwrap();
        let full = compute_maintainability_index(cc, loc, Some(vol)).unwrap();
        assert!(
            full.score <= simplified.score,
            "full ({}) should be <= simplified ({}) for cc={cc}, loc={loc}, vol={vol}",
            full.score,
            simplified.score
        );
    }
}

// ===========================================================================
// LOC rounding: fractional LOC values are rounded to 2 decimals
// ===========================================================================

#[test]
fn fractional_loc_rounded_to_two_decimals() {
    let mi = compute_maintainability_index(5.0, 99.999, None).expect("mi");
    assert_eq!(mi.avg_loc, 100.0);

    let mi2 = compute_maintainability_index(5.0, 33.335, None).expect("mi");
    assert_eq!(mi2.avg_loc, 33.34);
}

// ===========================================================================
// attach_halstead_metrics: recomputes MI with Halstead volume
// ===========================================================================

#[test]
fn attach_halstead_recomputes_mi_to_lower_score() {
    let mut report = sample_complexity_with_mi(10.0, 100.0);
    let before = report.maintainability_index.as_ref().unwrap().score;

    attach_halstead_metrics(&mut report, make_halstead(500.0));

    let after = report.maintainability_index.as_ref().unwrap().score;
    assert!(after < before, "MI should decrease after adding Halstead");
    assert_eq!(
        report
            .maintainability_index
            .as_ref()
            .unwrap()
            .avg_halstead_volume,
        Some(500.0)
    );
}

// ===========================================================================
// attach_halstead_metrics: zero volume → MI unchanged, halstead still stored
// ===========================================================================

#[test]
fn attach_halstead_zero_volume_preserves_mi() {
    let mut report = sample_complexity_with_mi(10.0, 100.0);
    let before = report.maintainability_index.as_ref().unwrap().score;

    attach_halstead_metrics(&mut report, make_halstead(0.0));

    let after = report.maintainability_index.as_ref().unwrap().score;
    assert_eq!(before, after);
    assert!(report.halstead.is_some());
}

// ===========================================================================
// attach_halstead_metrics: no prior MI → MI stays None
// ===========================================================================

#[test]
fn attach_halstead_without_prior_mi_stays_none() {
    let mut report = sample_complexity_with_mi(10.0, 100.0);
    report.maintainability_index = None;

    attach_halstead_metrics(&mut report, make_halstead(500.0));

    assert!(report.maintainability_index.is_none());
    assert!(report.halstead.is_some());
}

// ===========================================================================
// Determinism: same inputs → identical MI
// ===========================================================================

#[test]
fn compute_mi_is_deterministic() {
    let cases = vec![
        (0.0, 1.0, None),
        (10.0, 100.0, None),
        (10.0, 100.0, Some(200.0)),
        (50.0, 5000.0, Some(10000.0)),
    ];
    for (cc, loc, vol) in cases {
        let a = compute_maintainability_index(cc, loc, vol);
        let b = compute_maintainability_index(cc, loc, vol);
        match (a, b) {
            (Some(a), Some(b)) => {
                assert_eq!(a.score, b.score);
                assert_eq!(a.grade, b.grade);
                assert_eq!(a.avg_loc, b.avg_loc);
                assert_eq!(a.avg_cyclomatic, b.avg_cyclomatic);
                assert_eq!(a.avg_halstead_volume, b.avg_halstead_volume);
            }
            (None, None) => {}
            _ => panic!("non-deterministic for cc={cc}, loc={loc}, vol={vol:?}"),
        }
    }
}

// ===========================================================================
// Known computed values: verify exact scores
// ===========================================================================

#[test]
fn known_value_simplified_cc10_loc100() {
    // MI = 171 - 0.23*10 - 16.2*ln(100) = 171 - 2.3 - 74.6 = 94.1
    let mi = compute_maintainability_index(10.0, 100.0, None).unwrap();
    assert!((mi.score - 94.1).abs() < f64::EPSILON);
}

#[test]
fn known_value_full_cc10_loc100_vol200() {
    // MI = 171 - 5.2*ln(200) - 0.23*10 - 16.2*ln(100)
    //    = 171 - 27.56 - 2.3 - 74.6 = 66.54
    let mi = compute_maintainability_index(10.0, 100.0, Some(200.0)).unwrap();
    assert!((mi.score - 66.54).abs() < f64::EPSILON);
}

#[test]
fn known_value_simplified_cc1_loc1() {
    // MI = 171 - 0.23*1 - 16.2*ln(1) = 171 - 0.23 - 0 = 170.77
    let mi = compute_maintainability_index(1.0, 1.0, None).unwrap();
    assert!((mi.score - 170.77).abs() < f64::EPSILON);
}

// ===========================================================================
// Score valid range: always 0 ≤ score ≤ 171
// ===========================================================================

#[test]
fn score_always_in_valid_range() {
    let cases = vec![
        (0.0, 1.0, None),
        (1.0, 1.0, Some(1.0)),
        (500.0, 100_000.0, Some(1_000_000.0)),
        (1000.0, 1.0, None),
    ];
    for (cc, loc, vol) in cases {
        if let Some(mi) = compute_maintainability_index(cc, loc, vol) {
            assert!(
                mi.score >= 0.0 && mi.score <= 171.0,
                "score {} out of [0, 171] for cc={cc}, loc={loc}, vol={vol:?}",
                mi.score
            );
        }
    }
}

// ===========================================================================
// Halstead volume = 1.0 → minimal impact (ln(1) = 0)
// ===========================================================================

#[test]
fn halstead_volume_one_has_zero_ln_contribution() {
    // ln(1) = 0, so 5.2 * ln(1) = 0 → full formula equals simplified
    let simplified = compute_maintainability_index(10.0, 100.0, None).unwrap();
    let vol_one = compute_maintainability_index(10.0, 100.0, Some(1.0)).unwrap();

    assert_eq!(simplified.score, vol_one.score);
    // But avg_halstead_volume is recorded
    assert_eq!(vol_one.avg_halstead_volume, Some(1.0));
}
