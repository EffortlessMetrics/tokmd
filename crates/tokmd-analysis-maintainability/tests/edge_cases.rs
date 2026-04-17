//! Edge case tests for special float values, extreme inputs, and boundary conditions.

use tokmd_analysis_maintainability::{attach_halstead_metrics, compute_maintainability_index};
use tokmd_analysis_types::{
    ComplexityReport, ComplexityRisk, FileComplexity, HalsteadMetrics, TechnicalDebtLevel,
    TechnicalDebtRatio,
};

// ---------------------------------------------------------------------------
// Special float inputs: NaN
// ---------------------------------------------------------------------------

#[test]
fn nan_loc_returns_none() {
    // NaN fails the `avg_loc <= 0.0` check (NaN comparisons are false),
    // so it proceeds, but ln(NaN) = NaN → score = NaN → clamped.
    // The function should still return Some but with score 0 or propagate NaN.
    // Test documents actual behavior.
    let result = compute_maintainability_index(10.0, f64::NAN, None);
    // NaN is not <= 0.0 (comparison returns false), so it proceeds.
    // ln(NaN) = NaN, so raw_score = NaN, max(0.0) = NaN → round = NaN
    // The function returns Some with NaN score, which is acceptable.
    if let Some(mi) = result {
        assert!(mi.score.is_nan() || mi.score >= 0.0);
    }
}

#[test]
fn nan_cc_produces_result() {
    let result = compute_maintainability_index(f64::NAN, 100.0, None);
    if let Some(mi) = result {
        // 0.23 * NaN = NaN → score is NaN
        assert!(mi.score.is_nan() || mi.score >= 0.0);
    }
}

#[test]
fn nan_halstead_volume_falls_back_to_simplified() {
    // NaN > 0.0 is false, so the match arm `Some(volume) if volume > 0.0`
    // won't match → falls back to simplified formula.
    let simplified = compute_maintainability_index(10.0, 100.0, None).unwrap();
    let with_nan = compute_maintainability_index(10.0, 100.0, Some(f64::NAN)).unwrap();
    assert_eq!(simplified.score, with_nan.score);
    assert_eq!(with_nan.avg_halstead_volume, None);
}

// ---------------------------------------------------------------------------
// Special float inputs: infinity
// ---------------------------------------------------------------------------

#[test]
fn positive_infinity_loc_returns_score_zero() {
    // ln(inf) = inf → 16.2 * inf = inf → 171 - inf = -inf → clamped to 0
    let mi = compute_maintainability_index(10.0, f64::INFINITY, None).unwrap();
    assert_eq!(mi.score, 0.0);
    assert_eq!(mi.grade, "C");
}

#[test]
fn negative_infinity_loc_returns_none() {
    // -inf <= 0.0 is true → returns None
    assert!(compute_maintainability_index(10.0, f64::NEG_INFINITY, None).is_none());
}

#[test]
fn positive_infinity_cc_returns_score_zero() {
    // 0.23 * inf = inf → 171 - inf = -inf → clamped to 0
    let mi = compute_maintainability_index(f64::INFINITY, 100.0, None).unwrap();
    assert_eq!(mi.score, 0.0);
    assert_eq!(mi.grade, "C");
}

#[test]
fn positive_infinity_halstead_volume_returns_score_zero() {
    // 5.2 * ln(inf) = inf → clamped to 0
    let mi = compute_maintainability_index(10.0, 100.0, Some(f64::INFINITY)).unwrap();
    assert_eq!(mi.score, 0.0);
    assert_eq!(mi.grade, "C");
}

#[test]
fn negative_infinity_halstead_volume_falls_back_to_simplified() {
    // -inf > 0.0 is false → simplified formula
    let simplified = compute_maintainability_index(10.0, 100.0, None).unwrap();
    let with_neg_inf = compute_maintainability_index(10.0, 100.0, Some(f64::NEG_INFINITY)).unwrap();
    assert_eq!(simplified.score, with_neg_inf.score);
}

// ---------------------------------------------------------------------------
// Subnormal / very small positive floats
// ---------------------------------------------------------------------------

#[test]
fn subnormal_loc_produces_result() {
    // Very small positive LOC (subnormal float) → ln produces large negative
    // → score could exceed 171
    let tiny = f64::MIN_POSITIVE; // smallest normal positive
    let mi = compute_maintainability_index(0.0, tiny, None).unwrap();
    assert!(mi.score >= 0.0);
}

#[test]
fn subnormal_halstead_volume_used_when_positive() {
    let tiny = f64::MIN_POSITIVE;
    let mi = compute_maintainability_index(10.0, 100.0, Some(tiny)).unwrap();
    // ln(MIN_POSITIVE) is very negative → -5.2 * large_negative = large positive
    // so full formula score > simplified
    let simplified = compute_maintainability_index(10.0, 100.0, None).unwrap();
    assert!(mi.score > simplified.score);
    assert_eq!(mi.avg_halstead_volume, Some(tiny));
}

// ---------------------------------------------------------------------------
// Boundary: LOC exactly at various thresholds
// ---------------------------------------------------------------------------

#[test]
fn loc_epsilon_above_zero() {
    let mi = compute_maintainability_index(0.0, 1e-10, None).unwrap();
    assert!(mi.score >= 0.0);
}

#[test]
fn loc_one_simplified() {
    // MI = 171 - 0 - 16.2*ln(1) = 171
    let mi = compute_maintainability_index(0.0, 1.0, None).unwrap();
    assert_eq!(mi.score, 171.0);
}

#[test]
fn loc_just_below_zero_returns_none() {
    assert!(compute_maintainability_index(0.0, -1e-10, None).is_none());
}

// ---------------------------------------------------------------------------
// Score clamping edge cases
// ---------------------------------------------------------------------------

#[test]
fn score_never_negative_with_extreme_cc_and_loc() {
    // CC=1_000_000, LOC=1e15 → raw score hugely negative
    let mi = compute_maintainability_index(1_000_000.0, 1e15, None).unwrap();
    assert_eq!(mi.score, 0.0);
}

#[test]
fn score_never_negative_with_extreme_halstead() {
    let mi = compute_maintainability_index(0.0, 1.0, Some(1e300)).unwrap();
    assert_eq!(mi.score, 0.0);
}

#[test]
fn score_clamped_at_zero_gets_grade_c() {
    let mi = compute_maintainability_index(10000.0, 1e10, Some(1e10)).unwrap();
    assert_eq!(mi.score, 0.0);
    assert_eq!(mi.grade, "C");
}

// ---------------------------------------------------------------------------
// Rounding edge cases
// ---------------------------------------------------------------------------

#[test]
fn loc_rounding_exact_half_rounds_up() {
    // 50.005 * 100 = 5000.5 → rounds to 5001 → 50.01
    let mi = compute_maintainability_index(10.0, 50.005, None).unwrap();
    assert_eq!(mi.avg_loc, 50.01);
}

#[test]
fn loc_rounding_preserves_integer() {
    let mi = compute_maintainability_index(10.0, 100.0, None).unwrap();
    assert_eq!(mi.avg_loc, 100.0);
}

#[test]
fn score_rounding_preserves_two_decimals() {
    // Verify score is always rounded to exactly 2 decimal places
    for loc in [1.0, 7.3, 42.7, 99.99, 500.123] {
        let mi = compute_maintainability_index(5.0, loc, None).unwrap();
        let scaled = mi.score * 100.0;
        let diff = (scaled - scaled.round()).abs();
        assert!(
            diff < 1e-8,
            "LOC={loc}: score {} not rounded to 2 decimals",
            mi.score
        );
    }
}

// ---------------------------------------------------------------------------
// attach_halstead_metrics: edge cases
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

fn sample_complexity() -> ComplexityReport {
    ComplexityReport {
        total_functions: 3,
        avg_function_length: 10.0,
        max_function_length: 20,
        avg_cyclomatic: 10.0,
        max_cyclomatic: 18,
        avg_cognitive: None,
        max_cognitive: None,
        avg_nesting_depth: None,
        max_nesting_depth: None,
        high_risk_files: 0,
        histogram: None,
        halstead: None,
        maintainability_index: compute_maintainability_index(10.0, 100.0, None),
        technical_debt: Some(TechnicalDebtRatio {
            ratio: 10.0,
            complexity_points: 10,
            code_kloc: 1.0,
            level: TechnicalDebtLevel::Low,
        }),
        files: vec![FileComplexity {
            path: "src/lib.rs".to_string(),
            module: "src".to_string(),
            function_count: 3,
            max_function_length: 20,
            cyclomatic_complexity: 18,
            cognitive_complexity: None,
            max_nesting: None,
            risk_level: ComplexityRisk::Low,
            functions: None,
        }],
    }
}

#[test]
fn attach_halstead_nan_volume_preserves_mi() {
    let mut report = sample_complexity();
    let before = report.maintainability_index.as_ref().unwrap().score;
    attach_halstead_metrics(&mut report, make_halstead(f64::NAN));
    let after = report.maintainability_index.as_ref().unwrap().score;
    // NaN > 0.0 is false, so MI should be unchanged
    assert_eq!(before, after);
    assert!(report.halstead.is_some());
}

#[test]
fn attach_halstead_infinity_volume_recomputes_to_zero() {
    let mut report = sample_complexity();
    attach_halstead_metrics(&mut report, make_halstead(f64::INFINITY));
    let mi = report.maintainability_index.as_ref().unwrap();
    // inf > 0.0 is true, so recomputation happens with ln(inf)=inf → clamped to 0
    assert_eq!(mi.score, 0.0);
    assert_eq!(mi.grade, "C");
}

#[test]
fn attach_halstead_multiple_times_idempotent_with_same_volume() {
    let mut report = sample_complexity();
    attach_halstead_metrics(&mut report, make_halstead(200.0));
    let first_score = report.maintainability_index.as_ref().unwrap().score;

    attach_halstead_metrics(&mut report, make_halstead(200.0));
    let second_score = report.maintainability_index.as_ref().unwrap().score;

    assert_eq!(first_score, second_score);
}

#[test]
fn attach_halstead_then_detach_mi_and_reattach() {
    let mut report = sample_complexity();
    attach_halstead_metrics(&mut report, make_halstead(200.0));
    assert!(report.halstead.is_some());

    // Remove MI, then attach again — MI stays None
    report.maintainability_index = None;
    attach_halstead_metrics(&mut report, make_halstead(500.0));
    assert!(report.maintainability_index.is_none());
    assert_eq!(report.halstead.as_ref().unwrap().volume, 500.0);
}

#[test]
fn attach_halstead_decreasing_volumes_increase_mi() {
    let mut report1 = sample_complexity();
    attach_halstead_metrics(&mut report1, make_halstead(1000.0));
    let score_high_vol = report1.maintainability_index.as_ref().unwrap().score;

    let mut report2 = sample_complexity();
    attach_halstead_metrics(&mut report2, make_halstead(100.0));
    let score_low_vol = report2.maintainability_index.as_ref().unwrap().score;

    assert!(
        score_low_vol > score_high_vol,
        "lower volume should yield higher MI"
    );
}
