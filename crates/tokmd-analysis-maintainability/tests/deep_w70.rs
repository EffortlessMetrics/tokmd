//! Deep tests for tokmd-analysis-maintainability (w70 wave).
//!
//! ~20 tests covering maintainability-index computation, grading,
//! Halstead attach/recompute, edge cases, and determinism.

use tokmd_analysis_maintainability::{attach_halstead_metrics, compute_maintainability_index};
use tokmd_analysis_types::{
    ComplexityReport, ComplexityRisk, FileComplexity, HalsteadMetrics, TechnicalDebtLevel,
    TechnicalDebtRatio,
};

// -- Helpers --

fn halstead(volume: f64) -> HalsteadMetrics {
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

fn complexity_with_mi(cc: f64, loc: f64) -> ComplexityReport {
    ComplexityReport {
        total_functions: 3,
        avg_function_length: 10.0,
        max_function_length: 20,
        avg_cyclomatic: cc,
        max_cyclomatic: 18,
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

// -- Simplified formula --

#[test]
fn simplified_low_complexity_grades_a() {
    let mi = compute_maintainability_index(1.0, 10.0, None).unwrap();
    assert!(mi.score > 85.0);
    assert_eq!(mi.grade, "A");
    assert_eq!(mi.avg_halstead_volume, None);
}

#[test]
fn simplified_moderate_complexity_grades_b() {
    let _mi = compute_maintainability_index(50.0, 5000.0, None).unwrap();
    let mi = compute_maintainability_index(5.0, 500.0, None).unwrap();
    assert!(mi.score >= 65.0 && mi.score < 85.0, "score={}", mi.score);
    assert_eq!(mi.grade, "B");
}

#[test]
fn simplified_high_complexity_grades_c() {
    let mi = compute_maintainability_index(100.0, 10_000.0, None).unwrap();
    assert!(mi.score < 65.0);
    assert_eq!(mi.grade, "C");
}

#[test]
fn simplified_score_is_non_negative() {
    let mi = compute_maintainability_index(1000.0, 1_000_000.0, None).unwrap();
    assert!(mi.score >= 0.0);
}

#[test]
fn simplified_records_avg_cyclomatic_and_loc() {
    let mi = compute_maintainability_index(7.5, 250.0, None).unwrap();
    assert!((mi.avg_cyclomatic - 7.5).abs() < f64::EPSILON);
    assert!((mi.avg_loc - 250.0).abs() < f64::EPSILON);
}

// -- Full formula (with Halstead volume) --

#[test]
fn full_formula_lowers_score_vs_simplified() {
    let simplified = compute_maintainability_index(10.0, 100.0, None).unwrap();
    let full = compute_maintainability_index(10.0, 100.0, Some(200.0)).unwrap();
    assert!(full.score < simplified.score);
    assert_eq!(full.avg_halstead_volume, Some(200.0));
}

#[test]
fn full_formula_small_volume_minimal_reduction() {
    let simplified = compute_maintainability_index(5.0, 50.0, None).unwrap();
    let full = compute_maintainability_index(5.0, 50.0, Some(1.1)).unwrap();
    assert!((simplified.score - full.score).abs() < 1.0);
}

#[test]
fn full_formula_large_volume_significant_reduction() {
    let simplified = compute_maintainability_index(5.0, 50.0, None).unwrap();
    let full = compute_maintainability_index(5.0, 50.0, Some(50_000.0)).unwrap();
    assert!(simplified.score - full.score > 40.0);
}

// -- Edge cases --

#[test]
fn zero_loc_returns_none() {
    assert!(compute_maintainability_index(10.0, 0.0, None).is_none());
}

#[test]
fn negative_loc_returns_none() {
    assert!(compute_maintainability_index(10.0, -5.0, None).is_none());
}

#[test]
fn zero_halstead_volume_uses_simplified() {
    let simplified = compute_maintainability_index(10.0, 100.0, None).unwrap();
    let with_zero = compute_maintainability_index(10.0, 100.0, Some(0.0)).unwrap();
    assert!((simplified.score - with_zero.score).abs() < f64::EPSILON);
    assert_eq!(with_zero.avg_halstead_volume, None);
}

#[test]
fn negative_halstead_volume_uses_simplified() {
    let simplified = compute_maintainability_index(10.0, 100.0, None).unwrap();
    let with_neg = compute_maintainability_index(10.0, 100.0, Some(-10.0)).unwrap();
    assert!((simplified.score - with_neg.score).abs() < f64::EPSILON);
    assert_eq!(with_neg.avg_halstead_volume, None);
}

#[test]
fn zero_cyclomatic_is_valid() {
    let mi = compute_maintainability_index(0.0, 50.0, None).unwrap();
    assert!(mi.score > 100.0);
    assert_eq!(mi.grade, "A");
}

#[test]
fn loc_one_maximises_simplified_score() {
    let mi = compute_maintainability_index(0.0, 1.0, None).unwrap();
    assert!((mi.score - 171.0).abs() < f64::EPSILON);
}

// -- Grade boundaries --

#[test]
fn grade_boundary_exactly_85() {
    let a = compute_maintainability_index(1.0, 10.0, None).unwrap();
    assert!(a.score >= 85.0);
    assert_eq!(a.grade, "A");
}

#[test]
fn grade_boundary_clamped_zero_is_c() {
    let mi = compute_maintainability_index(500.0, 100_000.0, None).unwrap();
    assert!((mi.score - 0.0).abs() < f64::EPSILON);
    assert_eq!(mi.grade, "C");
}

// -- attach_halstead_metrics --

#[test]
fn attach_halstead_recomputes_with_positive_volume() {
    let mut cr = complexity_with_mi(10.0, 100.0);
    let before = cr.maintainability_index.as_ref().unwrap().score;
    attach_halstead_metrics(&mut cr, halstead(200.0));

    let mi = cr.maintainability_index.as_ref().unwrap();
    assert!(
        mi.score < before,
        "Score should decrease with Halstead penalty"
    );
    assert_eq!(mi.avg_halstead_volume, Some(200.0));
    assert!(cr.halstead.is_some());
}

#[test]
fn attach_halstead_zero_volume_preserves_mi() {
    let mut cr = complexity_with_mi(10.0, 100.0);
    let before = cr.maintainability_index.clone().unwrap();
    attach_halstead_metrics(&mut cr, halstead(0.0));

    let after = cr.maintainability_index.as_ref().unwrap();
    assert!((after.score - before.score).abs() < f64::EPSILON);
    assert_eq!(after.avg_halstead_volume, before.avg_halstead_volume);
    assert!(cr.halstead.is_some());
}

#[test]
fn attach_halstead_no_existing_mi_just_stores_halstead() {
    let mut cr = complexity_with_mi(10.0, 100.0);
    cr.maintainability_index = None;
    attach_halstead_metrics(&mut cr, halstead(200.0));

    assert!(cr.maintainability_index.is_none());
    assert!(cr.halstead.is_some());
    assert!((cr.halstead.as_ref().unwrap().volume - 200.0).abs() < f64::EPSILON);
}

// -- Determinism --

#[test]
fn deterministic_simplified_across_runs() {
    let a = compute_maintainability_index(15.0, 300.0, None);
    let b = compute_maintainability_index(15.0, 300.0, None);
    assert_eq!(a.unwrap().score, b.unwrap().score);
}

#[test]
fn deterministic_full_across_runs() {
    let a = compute_maintainability_index(15.0, 300.0, Some(500.0));
    let b = compute_maintainability_index(15.0, 300.0, Some(500.0));
    assert_eq!(a.unwrap().score, b.unwrap().score);
}

#[test]
fn deterministic_grade_for_same_score() {
    let a = compute_maintainability_index(5.0, 200.0, Some(100.0)).unwrap();
    let b = compute_maintainability_index(5.0, 200.0, Some(100.0)).unwrap();
    assert_eq!(a.grade, b.grade);
}
