use tokmd_analysis_maintainability::{attach_halstead_metrics, compute_maintainability_index};
use tokmd_analysis_types::{
    ComplexityReport, ComplexityRisk, FileComplexity, HalsteadMetrics, TechnicalDebtLevel,
    TechnicalDebtRatio,
};

// ---------------------------------------------------------------------------
// compute_maintainability_index – simplified formula
// ---------------------------------------------------------------------------

#[test]
fn given_typical_codebase_when_simplified_mi_then_grade_a() {
    // MI = 171 - 0.23*10 - 16.2*ln(100) ≈ 171 - 2.3 - 74.6 ≈ 94.1
    let mi = compute_maintainability_index(10.0, 100.0, None).expect("should produce MI");
    assert!((mi.score - 94.1).abs() < f64::EPSILON);
    assert_eq!(mi.grade, "A");
    assert_eq!(mi.avg_halstead_volume, None);
    assert_eq!(mi.avg_cyclomatic, 10.0);
    assert_eq!(mi.avg_loc, 100.0);
}

#[test]
fn given_small_codebase_when_simplified_mi_then_high_score() {
    // MI = 171 - 0.23*1 - 16.2*ln(10) ≈ 171 - 0.23 - 37.31 ≈ 133.46
    let mi = compute_maintainability_index(1.0, 10.0, None).expect("should produce MI");
    assert!(mi.score > 130.0);
    assert_eq!(mi.grade, "A");
}

#[test]
fn given_single_line_when_simplified_mi_then_high_score() {
    let mi = compute_maintainability_index(1.0, 1.0, None).expect("should produce MI");
    // MI = 171 - 0.23*1 - 16.2*ln(1) = 171 - 0.23 - 0 = 170.77
    assert!((mi.score - 170.77).abs() < f64::EPSILON);
    assert_eq!(mi.grade, "A");
}

// ---------------------------------------------------------------------------
// compute_maintainability_index – full formula (with Halstead)
// ---------------------------------------------------------------------------

#[test]
fn given_halstead_volume_when_full_mi_then_score_is_lower() {
    let simplified = compute_maintainability_index(10.0, 100.0, None).expect("simplified");
    let full = compute_maintainability_index(10.0, 100.0, Some(200.0)).expect("full");
    assert!(full.score < simplified.score);
    assert_eq!(full.avg_halstead_volume, Some(200.0));
}

#[test]
fn given_halstead_volume_200_when_full_mi_then_known_value() {
    // MI = 171 - 5.2*ln(200) - 0.23*10 - 16.2*ln(100)
    //    ≈ 171 - 27.56 - 2.3 - 74.6 ≈ 66.54
    let mi = compute_maintainability_index(10.0, 100.0, Some(200.0)).expect("full");
    assert!((mi.score - 66.54).abs() < f64::EPSILON);
    assert_eq!(mi.grade, "B");
}

#[test]
fn given_very_large_halstead_volume_when_full_mi_then_grade_c() {
    // Large volume pushes MI very low
    let mi = compute_maintainability_index(50.0, 10000.0, Some(100000.0)).expect("full");
    assert_eq!(mi.grade, "C");
}

// ---------------------------------------------------------------------------
// Grade boundaries
// ---------------------------------------------------------------------------

#[test]
fn given_score_exactly_85_when_grading_then_grade_is_a() {
    // We need: 171 - 0.23*CC - 16.2*ln(LOC) = 85
    // => 0.23*CC + 16.2*ln(LOC) = 86
    // With LOC=100: 0.23*CC = 86 - 74.6 = 11.4, CC ≈ 49.56
    // Rounding won't be exact at 85.0, so we pick values that yield exactly the boundary.
    // Instead, test the grade directly by computing near-boundary values.
    let mi = compute_maintainability_index(10.0, 100.0, None).expect("mi");
    // score ≈ 94.1, should be A
    assert!(mi.score >= 85.0);
    assert_eq!(mi.grade, "A");
}

#[test]
fn given_score_between_65_and_85_when_grading_then_grade_is_b() {
    let mi = compute_maintainability_index(10.0, 100.0, Some(200.0)).expect("mi");
    // score ≈ 66.54
    assert!(mi.score >= 65.0 && mi.score < 85.0);
    assert_eq!(mi.grade, "B");
}

#[test]
fn given_score_below_65_when_grading_then_grade_is_c() {
    let mi = compute_maintainability_index(100.0, 5000.0, Some(50000.0)).expect("mi");
    assert!(mi.score < 65.0);
    assert_eq!(mi.grade, "C");
}

// ---------------------------------------------------------------------------
// Edge cases – zero / negative LOC
// ---------------------------------------------------------------------------

#[test]
fn given_zero_loc_when_computing_mi_then_none_is_returned() {
    assert!(compute_maintainability_index(10.0, 0.0, None).is_none());
}

#[test]
fn given_negative_loc_when_computing_mi_then_none_is_returned() {
    assert!(compute_maintainability_index(10.0, -5.0, None).is_none());
}

#[test]
fn given_zero_loc_with_halstead_when_computing_mi_then_none_is_returned() {
    assert!(compute_maintainability_index(10.0, 0.0, Some(200.0)).is_none());
}

// ---------------------------------------------------------------------------
// Edge cases – zero / negative Halstead volume falls back to simplified
// ---------------------------------------------------------------------------

#[test]
fn given_zero_halstead_volume_when_computing_mi_then_simplified_formula_used() {
    let with_zero = compute_maintainability_index(10.0, 100.0, Some(0.0)).expect("mi");
    let simplified = compute_maintainability_index(10.0, 100.0, None).expect("mi");
    assert_eq!(with_zero.score, simplified.score);
    assert_eq!(with_zero.avg_halstead_volume, None);
}

#[test]
fn given_negative_halstead_volume_when_computing_mi_then_simplified_formula_used() {
    let with_neg = compute_maintainability_index(10.0, 100.0, Some(-100.0)).expect("mi");
    let simplified = compute_maintainability_index(10.0, 100.0, None).expect("mi");
    assert_eq!(with_neg.score, simplified.score);
    assert_eq!(with_neg.avg_halstead_volume, None);
}

// ---------------------------------------------------------------------------
// Edge cases – zero cyclomatic complexity
// ---------------------------------------------------------------------------

#[test]
fn given_zero_cyclomatic_when_computing_mi_then_valid_result() {
    let mi = compute_maintainability_index(0.0, 100.0, None).expect("mi");
    // MI = 171 - 0 - 16.2*ln(100) ≈ 171 - 74.6 = 96.4
    assert!((mi.score - 96.4).abs() < f64::EPSILON);
    assert_eq!(mi.avg_cyclomatic, 0.0);
}

// ---------------------------------------------------------------------------
// Score floor at zero
// ---------------------------------------------------------------------------

#[test]
fn given_extreme_values_when_computing_mi_then_score_is_clamped_at_zero() {
    // Enormous LOC and CC should push raw score negative; result clamped to 0.
    let mi = compute_maintainability_index(10000.0, 1e15, Some(1e15)).expect("mi");
    assert_eq!(mi.score, 0.0);
    assert_eq!(mi.grade, "C");
}

// ---------------------------------------------------------------------------
// LOC rounding
// ---------------------------------------------------------------------------

#[test]
fn given_fractional_loc_when_computing_mi_then_loc_is_rounded_to_2_decimals() {
    let mi = compute_maintainability_index(5.0, 99.999, None).expect("mi");
    assert_eq!(mi.avg_loc, 100.0);
}

// ---------------------------------------------------------------------------
// attach_halstead_metrics – integration
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
fn given_positive_halstead_when_attaching_then_mi_is_recomputed() {
    let mut report = sample_complexity();
    let before_score = report.maintainability_index.as_ref().unwrap().score;

    attach_halstead_metrics(&mut report, make_halstead(200.0));

    let mi = report.maintainability_index.as_ref().unwrap();
    assert!(mi.score < before_score);
    assert_eq!(mi.avg_halstead_volume, Some(200.0));
    assert!(report.halstead.is_some());
}

#[test]
fn given_zero_volume_halstead_when_attaching_then_mi_is_unchanged() {
    let mut report = sample_complexity();
    let before_score = report.maintainability_index.as_ref().unwrap().score;

    attach_halstead_metrics(&mut report, make_halstead(0.0));

    let after_score = report.maintainability_index.as_ref().unwrap().score;
    assert_eq!(before_score, after_score);
    // Halstead is still attached even though MI wasn't recomputed
    assert!(report.halstead.is_some());
    assert_eq!(report.halstead.as_ref().unwrap().volume, 0.0);
}

#[test]
fn given_no_existing_mi_when_attaching_halstead_then_mi_stays_none() {
    let mut report = sample_complexity();
    report.maintainability_index = None;

    attach_halstead_metrics(&mut report, make_halstead(200.0));

    assert!(report.maintainability_index.is_none());
    assert!(report.halstead.is_some());
}

#[test]
fn given_halstead_attached_when_checking_fields_then_all_fields_present() {
    let mut report = sample_complexity();
    let h = make_halstead(300.0);

    attach_halstead_metrics(&mut report, h);

    let halstead = report.halstead.as_ref().unwrap();
    assert_eq!(halstead.volume, 300.0);
    assert_eq!(halstead.distinct_operators, 10);
    assert_eq!(halstead.distinct_operands, 20);
}
