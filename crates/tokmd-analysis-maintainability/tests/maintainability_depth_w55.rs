//! Deep coverage tests for tokmd-analysis-maintainability.

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
        distinct_operands: 15,
        total_operators: 60,
        total_operands: 90,
        vocabulary: 25,
        length: 150,
        volume,
        difficulty: 5.0,
        effort: volume * 5.0,
        time_seconds: volume * 5.0 / 18.0,
        estimated_bugs: volume / 3000.0,
    }
}

fn sample_complexity(cc: f64, loc: f64) -> ComplexityReport {
    ComplexityReport {
        total_functions: 5,
        avg_function_length: loc / 5.0,
        max_function_length: loc as usize,
        avg_cyclomatic: cc,
        max_cyclomatic: cc as usize + 5,
        avg_cognitive: Some(cc * 0.8),
        max_cognitive: Some(cc as usize),
        avg_nesting_depth: Some(2.0),
        max_nesting_depth: Some(4),
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
            cyclomatic_complexity: cc as usize,
            cognitive_complexity: Some((cc * 0.8) as usize),
            max_nesting: Some(4),
            risk_level: ComplexityRisk::Low,
            functions: None,
        }],
    }
}

// ---------------------------------------------------------------------------
// Grade classification
// ---------------------------------------------------------------------------

#[test]
fn grade_a_at_threshold() {
    let mi = compute_maintainability_index(1.0, 1.0, None).unwrap();
    // Very low complexity, very small code -> high score -> A
    assert_eq!(mi.grade, "A");
    assert!(mi.score >= 85.0);
}

#[test]
fn grade_b_range() {
    // MI = 171 - 0.23*20 - 16.2*ln(100) = 171 - 4.6 - 74.60 = 91.80 -> A
    // Need bigger LOC. MI = 171 - 0.23*20 - 16.2*ln(10000) = 171 - 4.6 - 149.21 = 17.19 -> C
    // MI = 171 - 0.23*20 - 16.2*ln(700) = 171 - 4.6 - 106.17 = 60.23 -> C still
    // MI = 171 - 0.23*5 - 16.2*ln(400) = 171 - 1.15 - 97.04 = 72.81 -> B
    let mi = compute_maintainability_index(5.0, 400.0, None).unwrap();
    assert_eq!(mi.grade, "B");
    assert!(mi.score >= 65.0);
    assert!(mi.score < 85.0);
}

#[test]
fn grade_c_for_high_complexity() {
    let mi = compute_maintainability_index(100.0, 10000.0, None).unwrap();
    assert_eq!(mi.grade, "C");
    assert!(mi.score < 65.0);
}

#[test]
fn grade_a_boundary_exact() {
    // 171 - 0.23 * CC - 16.2 * ln(LOC) = 85
    // Solve for CC = 0: LOC = exp((171 - 85) / 16.2) ≈ exp(5.3086) ≈ 201.8
    // So LOC = 200 should give A, LOC = 210 should be close
    let mi = compute_maintainability_index(0.0, 200.0, None).unwrap();
    assert_eq!(mi.grade, "A");
}

#[test]
fn grade_b_boundary() {
    // 171 - 0.23*0 - 16.2*ln(500) = 171 - 100.59 = 70.41 -> B
    let mi = compute_maintainability_index(0.0, 500.0, None).unwrap();
    assert!(mi.score < 85.0);
    assert!(mi.score >= 65.0);
    assert_eq!(mi.grade, "B");
}

// ---------------------------------------------------------------------------
// Simplified formula (no Halstead)
// ---------------------------------------------------------------------------

#[test]
fn simplified_formula_basic() {
    // MI = 171 - 0.23 * 10 - 16.2 * ln(100)
    //    = 171 - 2.3 - 16.2 * 4.60517...
    //    = 171 - 2.3 - 74.6038 ≈ 94.10
    let mi = compute_maintainability_index(10.0, 100.0, None).unwrap();
    assert!((mi.score - 94.1).abs() < 0.01);
    assert_eq!(mi.avg_halstead_volume, None);
}

#[test]
fn simplified_zero_complexity() {
    let mi = compute_maintainability_index(0.0, 100.0, None).unwrap();
    // MI = 171 - 0 - 16.2 * ln(100) = 171 - 74.6038 ≈ 96.40
    assert!((mi.score - 96.4).abs() < 0.01);
}

#[test]
fn simplified_one_loc() {
    let mi = compute_maintainability_index(0.0, 1.0, None).unwrap();
    // MI = 171 - 0 - 16.2 * ln(1) = 171 - 0 = 171
    assert!((mi.score - 171.0).abs() < f64::EPSILON);
    assert_eq!(mi.grade, "A");
}

#[test]
fn simplified_large_loc() {
    let mi = compute_maintainability_index(0.0, 1_000_000.0, None).unwrap();
    // MI = 171 - 16.2 * ln(1e6) = 171 - 16.2 * 13.8155 ≈ 171 - 223.81 < 0 -> clamped to 0
    assert!((mi.score - 0.0).abs() < f64::EPSILON);
    assert_eq!(mi.grade, "C");
}

#[test]
fn simplified_preserves_inputs() {
    let mi = compute_maintainability_index(15.0, 250.0, None).unwrap();
    assert!((mi.avg_cyclomatic - 15.0).abs() < f64::EPSILON);
    assert!((mi.avg_loc - 250.0).abs() < f64::EPSILON);
}

// ---------------------------------------------------------------------------
// Full formula (with Halstead volume)
// ---------------------------------------------------------------------------

#[test]
fn full_formula_basic() {
    // MI = 171 - 5.2 * ln(200) - 0.23 * 10 - 16.2 * ln(100)
    //    = 171 - 5.2 * 5.2983 - 2.3 - 74.6038
    //    = 171 - 27.5511 - 2.3 - 74.6038 ≈ 66.54
    let mi = compute_maintainability_index(10.0, 100.0, Some(200.0)).unwrap();
    assert!((mi.score - 66.54).abs() < 0.01);
    assert_eq!(mi.avg_halstead_volume, Some(200.0));
    assert_eq!(mi.grade, "B");
}

#[test]
fn full_formula_high_volume_degrades_score() {
    let simplified = compute_maintainability_index(10.0, 100.0, None).unwrap();
    let full = compute_maintainability_index(10.0, 100.0, Some(500.0)).unwrap();
    assert!(full.score < simplified.score);
}

#[test]
fn full_formula_small_volume_barely_degrades() {
    let simplified = compute_maintainability_index(10.0, 100.0, None).unwrap();
    let full = compute_maintainability_index(10.0, 100.0, Some(1.0)).unwrap();
    // ln(1) = 0, so 5.2 * 0 = 0 -> score should be same
    assert!((full.score - simplified.score).abs() < 0.01);
}

#[test]
fn full_formula_volume_one_equals_simplified() {
    let simplified = compute_maintainability_index(5.0, 50.0, None).unwrap();
    let full = compute_maintainability_index(5.0, 50.0, Some(1.0)).unwrap();
    // ln(1) = 0 -> same formula
    assert!((full.score - simplified.score).abs() < 0.01);
}

// ---------------------------------------------------------------------------
// Edge cases: zero/negative LOC
// ---------------------------------------------------------------------------

#[test]
fn zero_loc_returns_none() {
    assert!(compute_maintainability_index(10.0, 0.0, None).is_none());
}

#[test]
fn negative_loc_returns_none() {
    assert!(compute_maintainability_index(10.0, -5.0, None).is_none());
}

#[test]
fn negative_loc_with_halstead_returns_none() {
    assert!(compute_maintainability_index(10.0, -1.0, Some(100.0)).is_none());
}

// ---------------------------------------------------------------------------
// Edge cases: Halstead volume zero/negative falls back to simplified
// ---------------------------------------------------------------------------

#[test]
fn zero_halstead_volume_uses_simplified() {
    let simplified = compute_maintainability_index(10.0, 100.0, None).unwrap();
    let with_zero = compute_maintainability_index(10.0, 100.0, Some(0.0)).unwrap();
    assert!((with_zero.score - simplified.score).abs() < f64::EPSILON);
    assert_eq!(with_zero.avg_halstead_volume, None);
}

#[test]
fn negative_halstead_volume_uses_simplified() {
    let simplified = compute_maintainability_index(10.0, 100.0, None).unwrap();
    let with_neg = compute_maintainability_index(10.0, 100.0, Some(-10.0)).unwrap();
    assert!((with_neg.score - simplified.score).abs() < f64::EPSILON);
    assert_eq!(with_neg.avg_halstead_volume, None);
}

// ---------------------------------------------------------------------------
// Score clamping
// ---------------------------------------------------------------------------

#[test]
fn score_never_negative() {
    let mi = compute_maintainability_index(500.0, 1_000_000.0, Some(1_000_000.0)).unwrap();
    assert!(mi.score >= 0.0);
}

#[test]
fn score_clamped_to_zero_for_extreme_inputs() {
    let mi = compute_maintainability_index(1000.0, 10_000_000.0, None).unwrap();
    assert!((mi.score - 0.0).abs() < f64::EPSILON);
}

// ---------------------------------------------------------------------------
// MaintainabilityIndex struct fields
// ---------------------------------------------------------------------------

#[test]
fn index_has_correct_fields_simplified() {
    let mi = compute_maintainability_index(5.0, 50.0, None).unwrap();
    assert!((mi.avg_cyclomatic - 5.0).abs() < f64::EPSILON);
    assert!((mi.avg_loc - 50.0).abs() < f64::EPSILON);
    assert!(mi.avg_halstead_volume.is_none());
    assert!(!mi.grade.is_empty());
    assert!(mi.score >= 0.0);
}

#[test]
fn index_has_correct_fields_full() {
    let mi = compute_maintainability_index(5.0, 50.0, Some(100.0)).unwrap();
    assert!((mi.avg_cyclomatic - 5.0).abs() < f64::EPSILON);
    assert!((mi.avg_loc - 50.0).abs() < f64::EPSILON);
    assert_eq!(mi.avg_halstead_volume, Some(100.0));
    assert!(!mi.grade.is_empty());
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn deterministic_simplified() {
    let a = compute_maintainability_index(12.0, 300.0, None).unwrap();
    let b = compute_maintainability_index(12.0, 300.0, None).unwrap();
    assert!((a.score - b.score).abs() < f64::EPSILON);
    assert_eq!(a.grade, b.grade);
}

#[test]
fn deterministic_full() {
    let a = compute_maintainability_index(12.0, 300.0, Some(150.0)).unwrap();
    let b = compute_maintainability_index(12.0, 300.0, Some(150.0)).unwrap();
    assert!((a.score - b.score).abs() < f64::EPSILON);
    assert_eq!(a.grade, b.grade);
}

// ---------------------------------------------------------------------------
// attach_halstead_metrics
// ---------------------------------------------------------------------------

#[test]
fn attach_halstead_recomputes_with_volume() {
    let mut cr = sample_complexity(10.0, 100.0);
    let before = cr.maintainability_index.as_ref().unwrap().score;

    attach_halstead_metrics(&mut cr, make_halstead(200.0));

    let after = cr.maintainability_index.as_ref().unwrap();
    assert!(after.score < before);
    assert_eq!(after.avg_halstead_volume, Some(200.0));
    assert!(cr.halstead.is_some());
}

#[test]
fn attach_halstead_zero_volume_preserves_index() {
    let mut cr = sample_complexity(10.0, 100.0);
    let before = cr.maintainability_index.as_ref().unwrap().score;

    attach_halstead_metrics(&mut cr, make_halstead(0.0));

    let after = cr.maintainability_index.as_ref().unwrap().score;
    assert!((before - after).abs() < f64::EPSILON);
    assert!(cr.halstead.is_some());
    assert_eq!(cr.halstead.as_ref().unwrap().volume, 0.0);
}

#[test]
fn attach_halstead_always_sets_halstead_field() {
    let mut cr = sample_complexity(5.0, 50.0);
    assert!(cr.halstead.is_none());

    attach_halstead_metrics(&mut cr, make_halstead(100.0));

    assert!(cr.halstead.is_some());
}

#[test]
fn attach_halstead_no_mi_still_sets_halstead() {
    let mut cr = sample_complexity(5.0, 50.0);
    cr.maintainability_index = None;

    attach_halstead_metrics(&mut cr, make_halstead(100.0));

    assert!(cr.maintainability_index.is_none());
    assert!(cr.halstead.is_some());
}

#[test]
fn attach_halstead_updates_grade() {
    let mut cr = sample_complexity(10.0, 100.0);
    let before_grade = cr.maintainability_index.as_ref().unwrap().grade.clone();
    assert_eq!(before_grade, "A");

    // Very large volume should degrade to B or C
    attach_halstead_metrics(&mut cr, make_halstead(5000.0));

    let after_grade = &cr.maintainability_index.as_ref().unwrap().grade;
    assert_ne!(after_grade, "A");
}

// ---------------------------------------------------------------------------
// Monotonicity: increasing CC or LOC decreases score
// ---------------------------------------------------------------------------

#[test]
fn increasing_complexity_decreases_score() {
    let low = compute_maintainability_index(5.0, 100.0, None).unwrap();
    let high = compute_maintainability_index(50.0, 100.0, None).unwrap();
    assert!(high.score < low.score);
}

#[test]
fn increasing_loc_decreases_score() {
    let small = compute_maintainability_index(10.0, 50.0, None).unwrap();
    let large = compute_maintainability_index(10.0, 5000.0, None).unwrap();
    assert!(large.score < small.score);
}

#[test]
fn increasing_halstead_volume_decreases_score() {
    let low_v = compute_maintainability_index(10.0, 100.0, Some(50.0)).unwrap();
    let high_v = compute_maintainability_index(10.0, 100.0, Some(5000.0)).unwrap();
    assert!(high_v.score < low_v.score);
}

// ---------------------------------------------------------------------------
// Rounding
// ---------------------------------------------------------------------------

#[test]
fn score_is_rounded_to_two_decimals() {
    let mi = compute_maintainability_index(7.0, 77.0, None).unwrap();
    let rounded = (mi.score * 100.0).round() / 100.0;
    assert!((mi.score - rounded).abs() < f64::EPSILON);
}

#[test]
fn avg_loc_is_rounded_to_two_decimals() {
    let mi = compute_maintainability_index(5.0, 33.333, None).unwrap();
    let rounded = (mi.avg_loc * 100.0).round() / 100.0;
    assert!((mi.avg_loc - rounded).abs() < f64::EPSILON);
}
