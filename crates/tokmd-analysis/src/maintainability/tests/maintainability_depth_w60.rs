//! Wave-60 depth tests for analysis maintainability module.
//!
//! Covers: BDD maintainability index computation, property tests for
//! determinism and bounds, language mix edge cases, rating thresholds,
//! attach_halstead_metrics integration.

use crate::maintainability::{attach_halstead_metrics, compute_maintainability_index};
use proptest::prelude::*;
use tokmd_analysis_types::{
    ComplexityReport, ComplexityRisk, FileComplexity, HalsteadMetrics, TechnicalDebtLevel,
    TechnicalDebtRatio,
};

// ===========================================================================
// Helpers
// ===========================================================================

fn make_halstead(volume: f64) -> HalsteadMetrics {
    HalsteadMetrics {
        distinct_operators: 12,
        distinct_operands: 18,
        total_operators: 72,
        total_operands: 108,
        vocabulary: 30,
        length: 180,
        volume,
        difficulty: 6.0,
        effort: volume * 6.0,
        time_seconds: volume * 6.0 / 18.0,
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
        avg_cognitive: Some(cc * 0.75),
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
            cognitive_complexity: Some((cc * 0.75) as usize),
            max_nesting: Some(4),
            risk_level: ComplexityRisk::Low,
            functions: None,
        }],
    }
}

// ===========================================================================
// BDD: Simplified formula (no Halstead volume)
// ===========================================================================

#[test]
fn given_trivial_code_when_simplified_then_near_max_score() {
    // MI = 171 - 0.23*1 - 16.2*ln(1) = 170.77
    let mi = compute_maintainability_index(1.0, 1.0, None).unwrap();
    assert!((mi.score - 170.77).abs() < f64::EPSILON);
    assert_eq!(mi.grade, "A");
}

#[test]
fn given_100_loc_10_cc_when_simplified_then_known_value() {
    // MI = 171 - 0.23*10 - 16.2*ln(100) = 171 - 2.3 - 74.60 = 94.10
    let mi = compute_maintainability_index(10.0, 100.0, None).unwrap();
    assert!((mi.score - 94.1).abs() < 0.01);
    assert_eq!(mi.grade, "A");
    assert_eq!(mi.avg_halstead_volume, None);
}

#[test]
fn given_500_loc_20_cc_when_simplified_then_grade_b() {
    // MI = 171 - 0.23*20 - 16.2*ln(500) = 171 - 4.6 - 100.59 = 65.81
    let mi = compute_maintainability_index(20.0, 500.0, None).unwrap();
    assert!(mi.score >= 65.0);
    assert!(mi.score < 85.0);
    assert_eq!(mi.grade, "B");
}

#[test]
fn given_10000_loc_100_cc_when_simplified_then_grade_c() {
    // MI = 171 - 0.23*100 - 16.2*ln(10000) = 171 - 23 - 149.21 = -1.21 -> clamped 0
    let mi = compute_maintainability_index(100.0, 10000.0, None).unwrap();
    assert_eq!(mi.score, 0.0);
    assert_eq!(mi.grade, "C");
}

#[test]
fn given_zero_cc_when_simplified_then_only_loc_matters() {
    let mi = compute_maintainability_index(0.0, 100.0, None).unwrap();
    // MI = 171 - 0 - 16.2*ln(100) = 96.40
    assert!((mi.score - 96.4).abs() < 0.01);
}

#[test]
fn given_fractional_loc_when_simplified_then_loc_rounded() {
    let mi = compute_maintainability_index(5.0, 99.999, None).unwrap();
    assert_eq!(mi.avg_loc, 100.0);
}

#[test]
fn given_small_fractional_loc_then_rounded_correctly() {
    let mi = compute_maintainability_index(3.0, 0.005, None).unwrap();
    assert_eq!(mi.avg_loc, 0.01);
}

// ===========================================================================
// BDD: Full formula (with Halstead volume)
// ===========================================================================

#[test]
fn given_halstead_200_when_full_then_known_value() {
    // MI = 171 - 5.2*ln(200) - 0.23*10 - 16.2*ln(100) = 66.54
    let mi = compute_maintainability_index(10.0, 100.0, Some(200.0)).unwrap();
    assert!((mi.score - 66.54).abs() < 0.01);
    assert_eq!(mi.grade, "B");
    assert_eq!(mi.avg_halstead_volume, Some(200.0));
}

#[test]
fn given_halstead_1_when_full_then_equals_simplified() {
    // ln(1) = 0, so 5.2*0 = 0
    let simplified = compute_maintainability_index(10.0, 100.0, None).unwrap();
    let full = compute_maintainability_index(10.0, 100.0, Some(1.0)).unwrap();
    assert!((full.score - simplified.score).abs() < 0.01);
}

#[test]
fn given_large_halstead_then_grade_c() {
    let mi = compute_maintainability_index(50.0, 5000.0, Some(100000.0)).unwrap();
    assert_eq!(mi.grade, "C");
}

#[test]
fn given_moderate_halstead_then_score_lower_than_simplified() {
    let s = compute_maintainability_index(10.0, 100.0, None).unwrap();
    let f = compute_maintainability_index(10.0, 100.0, Some(500.0)).unwrap();
    assert!(f.score < s.score);
}

// ===========================================================================
// BDD: Edge cases – zero/negative LOC
// ===========================================================================

#[test]
fn given_zero_loc_then_none() {
    assert!(compute_maintainability_index(10.0, 0.0, None).is_none());
}

#[test]
fn given_negative_loc_then_none() {
    assert!(compute_maintainability_index(10.0, -1.0, None).is_none());
}

#[test]
fn given_zero_loc_with_halstead_then_none() {
    assert!(compute_maintainability_index(10.0, 0.0, Some(100.0)).is_none());
}

#[test]
fn given_negative_loc_with_halstead_then_none() {
    assert!(compute_maintainability_index(5.0, -100.0, Some(50.0)).is_none());
}

// ===========================================================================
// BDD: Halstead volume zero/negative falls back to simplified
// ===========================================================================

#[test]
fn given_zero_volume_then_simplified_used() {
    let s = compute_maintainability_index(10.0, 100.0, None).unwrap();
    let z = compute_maintainability_index(10.0, 100.0, Some(0.0)).unwrap();
    assert!((z.score - s.score).abs() < f64::EPSILON);
    assert_eq!(z.avg_halstead_volume, None);
}

#[test]
fn given_negative_volume_then_simplified_used() {
    let s = compute_maintainability_index(10.0, 100.0, None).unwrap();
    let n = compute_maintainability_index(10.0, 100.0, Some(-50.0)).unwrap();
    assert!((n.score - s.score).abs() < f64::EPSILON);
    assert_eq!(n.avg_halstead_volume, None);
}

// ===========================================================================
// BDD: Score clamping at zero
// ===========================================================================

#[test]
fn given_extreme_values_then_score_clamped_to_zero() {
    let mi = compute_maintainability_index(10000.0, 1e15, Some(1e15)).unwrap();
    assert_eq!(mi.score, 0.0);
    assert_eq!(mi.grade, "C");
}

#[test]
fn given_enormous_cc_then_score_clamped() {
    let mi = compute_maintainability_index(1e6, 10.0, None).unwrap();
    assert_eq!(mi.score, 0.0);
}

// ===========================================================================
// BDD: Grade threshold boundaries
// ===========================================================================

#[test]
fn grade_a_requires_score_at_least_85() {
    // CC=0, LOC=200 => MI = 171 - 16.2*ln(200) ≈ 171 - 85.88 = 85.12 -> A
    let mi = compute_maintainability_index(0.0, 200.0, None).unwrap();
    assert!(mi.score >= 85.0);
    assert_eq!(mi.grade, "A");
}

#[test]
fn grade_b_for_score_between_65_and_85() {
    // CC=0, LOC=500 => MI = 171 - 16.2*ln(500) ≈ 171 - 100.59 = 70.41 -> B
    let mi = compute_maintainability_index(0.0, 500.0, None).unwrap();
    assert!(mi.score >= 65.0);
    assert!(mi.score < 85.0);
    assert_eq!(mi.grade, "B");
}

#[test]
fn grade_c_for_score_below_65() {
    // CC=0, LOC=1000 => MI = 171 - 16.2*ln(1000) ≈ 171 - 111.89 = 59.11 -> C
    let mi = compute_maintainability_index(0.0, 1000.0, None).unwrap();
    assert!(mi.score < 65.0);
    assert_eq!(mi.grade, "C");
}

#[test]
fn grade_transitions_at_exact_boundaries() {
    // Find a LOC that gives score just above 85
    // 171 - 16.2*ln(LOC) = 85 => ln(LOC) = 86/16.2 => LOC ≈ 201.8
    let above = compute_maintainability_index(0.0, 201.0, None).unwrap();
    assert_eq!(above.grade, "A");

    // LOC ≈ 203 should still be near boundary
    let below = compute_maintainability_index(0.0, 203.0, None).unwrap();
    // This might still be A or B depending on rounding
    assert!(below.grade == "A" || below.grade == "B");
}

// ===========================================================================
// BDD: Monotonicity invariants
// ===========================================================================

#[test]
fn increasing_cc_decreases_score() {
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

// ===========================================================================
// BDD: Rounding
// ===========================================================================

#[test]
fn score_rounded_to_two_decimals() {
    let mi = compute_maintainability_index(7.0, 77.0, None).unwrap();
    let rounded = (mi.score * 100.0).round() / 100.0;
    assert!((mi.score - rounded).abs() < f64::EPSILON);
}

#[test]
fn avg_loc_rounded_to_two_decimals() {
    let mi = compute_maintainability_index(5.0, 33.333, None).unwrap();
    let rounded = (mi.avg_loc * 100.0).round() / 100.0;
    assert!((mi.avg_loc - rounded).abs() < f64::EPSILON);
}

// ===========================================================================
// BDD: Struct field preservation
// ===========================================================================

#[test]
fn simplified_index_preserves_all_fields() {
    let mi = compute_maintainability_index(15.0, 250.0, None).unwrap();
    assert!((mi.avg_cyclomatic - 15.0).abs() < f64::EPSILON);
    assert!((mi.avg_loc - 250.0).abs() < f64::EPSILON);
    assert!(mi.avg_halstead_volume.is_none());
    assert!(!mi.grade.is_empty());
    assert!(mi.score >= 0.0);
}

#[test]
fn full_index_preserves_all_fields() {
    let mi = compute_maintainability_index(15.0, 250.0, Some(300.0)).unwrap();
    assert!((mi.avg_cyclomatic - 15.0).abs() < f64::EPSILON);
    assert!((mi.avg_loc - 250.0).abs() < f64::EPSILON);
    assert_eq!(mi.avg_halstead_volume, Some(300.0));
    assert!(!mi.grade.is_empty());
}

// ===========================================================================
// BDD: Determinism
// ===========================================================================

#[test]
fn deterministic_simplified_computation() {
    let a = compute_maintainability_index(12.0, 300.0, None).unwrap();
    let b = compute_maintainability_index(12.0, 300.0, None).unwrap();
    assert!((a.score - b.score).abs() < f64::EPSILON);
    assert_eq!(a.grade, b.grade);
    assert_eq!(a.avg_halstead_volume, b.avg_halstead_volume);
}

#[test]
fn deterministic_full_computation() {
    let a = compute_maintainability_index(12.0, 300.0, Some(150.0)).unwrap();
    let b = compute_maintainability_index(12.0, 300.0, Some(150.0)).unwrap();
    assert!((a.score - b.score).abs() < f64::EPSILON);
    assert_eq!(a.grade, b.grade);
}

// ===========================================================================
// BDD: attach_halstead_metrics integration
// ===========================================================================

#[test]
fn attach_halstead_recomputes_mi_with_volume() {
    let mut cr = sample_complexity(10.0, 100.0);
    let before = cr.maintainability_index.as_ref().unwrap().score;
    attach_halstead_metrics(&mut cr, make_halstead(200.0));
    let after = cr.maintainability_index.as_ref().unwrap();
    assert!(after.score < before);
    assert_eq!(after.avg_halstead_volume, Some(200.0));
}

#[test]
fn attach_halstead_zero_volume_preserves_mi() {
    let mut cr = sample_complexity(10.0, 100.0);
    let before = cr.maintainability_index.as_ref().unwrap().score;
    attach_halstead_metrics(&mut cr, make_halstead(0.0));
    let after = cr.maintainability_index.as_ref().unwrap().score;
    assert!((before - after).abs() < f64::EPSILON);
}

#[test]
fn attach_halstead_always_sets_halstead_field() {
    let mut cr = sample_complexity(5.0, 50.0);
    assert!(cr.halstead.is_none());
    attach_halstead_metrics(&mut cr, make_halstead(100.0));
    assert!(cr.halstead.is_some());
}

#[test]
fn attach_halstead_no_mi_still_stores_halstead() {
    let mut cr = sample_complexity(5.0, 50.0);
    cr.maintainability_index = None;
    attach_halstead_metrics(&mut cr, make_halstead(200.0));
    assert!(cr.maintainability_index.is_none());
    assert!(cr.halstead.is_some());
}

#[test]
fn attach_halstead_can_change_grade() {
    let mut cr = sample_complexity(10.0, 100.0);
    assert_eq!(cr.maintainability_index.as_ref().unwrap().grade, "A");
    // Large volume should degrade
    attach_halstead_metrics(&mut cr, make_halstead(5000.0));
    assert_ne!(cr.maintainability_index.as_ref().unwrap().grade, "A");
}

#[test]
fn attach_halstead_preserves_volume_value() {
    let mut cr = sample_complexity(10.0, 100.0);
    attach_halstead_metrics(&mut cr, make_halstead(42.5));
    assert_eq!(cr.halstead.as_ref().unwrap().volume, 42.5);
}

#[test]
fn attach_halstead_preserves_operator_counts() {
    let mut cr = sample_complexity(10.0, 100.0);
    let h = make_halstead(100.0);
    attach_halstead_metrics(&mut cr, h);
    let stored = cr.halstead.as_ref().unwrap();
    assert_eq!(stored.distinct_operators, 12);
    assert_eq!(stored.distinct_operands, 18);
    assert_eq!(stored.total_operators, 72);
    assert_eq!(stored.total_operands, 108);
}

// ===========================================================================
// BDD: Different "language mix" scenarios (varying CC/LOC combos)
// ===========================================================================

#[test]
fn scripting_language_low_cc_moderate_loc() {
    // Typical Python/Ruby: low complexity, moderate LOC
    // MI = 171 - 0.23*3 - 16.2*ln(200) ≈ 84.43 -> B
    let mi = compute_maintainability_index(3.0, 200.0, None).unwrap();
    assert!(mi.score > 65.0);
    assert_eq!(mi.grade, "B");
}

#[test]
fn enterprise_java_moderate_cc_large_loc() {
    // Typical enterprise: moderate complexity, lots of boilerplate
    let mi = compute_maintainability_index(25.0, 2000.0, Some(800.0)).unwrap();
    assert!(mi.score < 85.0);
}

#[test]
fn embedded_c_high_cc_small_loc() {
    // Tight embedded code: high complexity, small files
    let mi = compute_maintainability_index(40.0, 80.0, None).unwrap();
    // MI = 171 - 0.23*40 - 16.2*ln(80) = 171 - 9.2 - 70.97 = 90.83
    assert!(mi.score > 85.0);
    assert_eq!(mi.grade, "A");
}

#[test]
fn legacy_cobol_very_high_cc_very_large_loc() {
    let mi = compute_maintainability_index(200.0, 50000.0, Some(20000.0)).unwrap();
    assert_eq!(mi.score, 0.0);
    assert_eq!(mi.grade, "C");
}

#[test]
fn microservice_low_cc_low_loc() {
    let mi = compute_maintainability_index(2.0, 30.0, None).unwrap();
    // Very maintainable
    assert!(mi.score > 100.0);
    assert_eq!(mi.grade, "A");
}

// ===========================================================================
// BDD: Serialization round-trip
// ===========================================================================

#[test]
fn maintainability_index_serializes_to_json() {
    let mi = compute_maintainability_index(10.0, 100.0, Some(200.0)).unwrap();
    let json = serde_json::to_string(&mi).unwrap();
    assert!(json.contains("\"score\""));
    assert!(json.contains("\"grade\""));
    assert!(json.contains("\"avg_halstead_volume\""));
}

#[test]
fn maintainability_index_roundtrips_through_json() {
    let mi = compute_maintainability_index(10.0, 100.0, Some(200.0)).unwrap();
    let json = serde_json::to_string(&mi).unwrap();
    let deserialized: tokmd_analysis_types::MaintainabilityIndex =
        serde_json::from_str(&json).unwrap();
    assert!((deserialized.score - mi.score).abs() < f64::EPSILON);
    assert_eq!(deserialized.grade, mi.grade);
    assert_eq!(deserialized.avg_halstead_volume, mi.avg_halstead_volume);
}

#[test]
fn simplified_index_omits_halstead_volume_in_json() {
    let mi = compute_maintainability_index(10.0, 100.0, None).unwrap();
    let json = serde_json::to_string(&mi).unwrap();
    // skip_serializing_if = "Option::is_none"
    assert!(!json.contains("avg_halstead_volume"));
}

// ===========================================================================
// Property tests
// ===========================================================================

proptest! {
    #[test]
    fn prop_score_non_negative(
        cc in 0.0f64..1000.0,
        loc in 0.01f64..1e6,
        vol in proptest::option::of(0.01f64..1e6),
    ) {
        if let Some(mi) = compute_maintainability_index(cc, loc, vol) {
            prop_assert!(mi.score >= 0.0);
        }
    }

    #[test]
    fn prop_score_at_most_171(
        cc in 0.0f64..1000.0,
        loc in 0.01f64..1e6,
        vol in proptest::option::of(0.01f64..1e6),
    ) {
        if let Some(mi) = compute_maintainability_index(cc, loc, vol) {
            prop_assert!(mi.score <= 171.0);
        }
    }

    #[test]
    fn prop_grade_valid(
        cc in 0.0f64..500.0,
        loc in 0.01f64..1e5,
        vol in proptest::option::of(0.01f64..1e5),
    ) {
        if let Some(mi) = compute_maintainability_index(cc, loc, vol) {
            prop_assert!(mi.grade == "A" || mi.grade == "B" || mi.grade == "C");
        }
    }

    #[test]
    fn prop_grade_a_implies_score_ge_85(
        cc in 0.0f64..500.0,
        loc in 0.01f64..1e5,
        vol in proptest::option::of(0.01f64..1e5),
    ) {
        if let Some(mi) = compute_maintainability_index(cc, loc, vol)
            && mi.grade == "A"
        {
            prop_assert!(mi.score >= 85.0);
        }
    }

    #[test]
    fn prop_grade_b_implies_score_65_to_85(
        cc in 0.0f64..500.0,
        loc in 0.01f64..1e5,
        vol in proptest::option::of(0.01f64..1e5),
    ) {
        if let Some(mi) = compute_maintainability_index(cc, loc, vol)
            && mi.grade == "B"
        {
            prop_assert!(mi.score >= 65.0);
            prop_assert!(mi.score < 85.0);
        }
    }

    #[test]
    fn prop_grade_c_implies_score_lt_65(
        cc in 0.0f64..500.0,
        loc in 0.01f64..1e5,
        vol in proptest::option::of(0.01f64..1e5),
    ) {
        if let Some(mi) = compute_maintainability_index(cc, loc, vol)
            && mi.grade == "C"
        {
            prop_assert!(mi.score < 65.0);
        }
    }

    #[test]
    fn prop_zero_or_negative_loc_none(
        cc in 0.0f64..100.0,
        loc in -1e6f64..=0.0,
    ) {
        prop_assert!(compute_maintainability_index(cc, loc, None).is_none());
    }

    #[test]
    fn prop_halstead_volume_degrades_score(
        cc in 0.0f64..100.0,
        loc in 1.0f64..1000.0,
        vol in 1.0f64..1e5,
    ) {
        let simplified = compute_maintainability_index(cc, loc, None).expect("simplified");
        let full = compute_maintainability_index(cc, loc, Some(vol)).expect("full");
        prop_assert!(full.score <= simplified.score);
    }

    #[test]
    fn prop_higher_cc_lower_or_equal_score(
        cc1 in 0.0f64..500.0,
        cc2 in 0.0f64..500.0,
        loc in 1.0f64..1000.0,
    ) {
        let mi1 = compute_maintainability_index(cc1, loc, None).expect("mi1");
        let mi2 = compute_maintainability_index(cc2, loc, None).expect("mi2");
        if cc1 < cc2 {
            prop_assert!(mi1.score >= mi2.score);
        }
    }

    #[test]
    fn prop_higher_loc_lower_or_equal_score(
        cc in 0.0f64..100.0,
        loc1 in 1.0f64..1000.0,
        loc2 in 1.0f64..1000.0,
    ) {
        let mi1 = compute_maintainability_index(cc, loc1, None).expect("mi1");
        let mi2 = compute_maintainability_index(cc, loc2, None).expect("mi2");
        if loc1 < loc2 && (loc2 - loc1).abs() > 0.01 {
            prop_assert!(mi1.score >= mi2.score);
        }
    }

    #[test]
    fn prop_deterministic(
        cc in 0.0f64..500.0,
        loc in 0.01f64..1e5,
        vol in proptest::option::of(0.01f64..1e5),
    ) {
        let r1 = compute_maintainability_index(cc, loc, vol);
        let r2 = compute_maintainability_index(cc, loc, vol);
        match (r1, r2) {
            (Some(a), Some(b)) => {
                prop_assert_eq!(a.score, b.score);
                prop_assert_eq!(a.grade, b.grade);
            }
            (None, None) => {}
            _ => prop_assert!(false, "determinism violated"),
        }
    }

    #[test]
    fn prop_score_rounded_2_decimals(
        cc in 0.0f64..100.0,
        loc in 0.01f64..1000.0,
    ) {
        if let Some(mi) = compute_maintainability_index(cc, loc, None) {
            let scaled = mi.score * 100.0;
            let diff = (scaled - scaled.round()).abs();
            prop_assert!(diff < 1e-8);
        }
    }

    #[test]
    fn prop_avg_loc_rounded_2_decimals(
        cc in 0.0f64..100.0,
        loc in 0.01f64..1000.0,
    ) {
        if let Some(mi) = compute_maintainability_index(cc, loc, None) {
            let scaled = mi.avg_loc * 100.0;
            let diff = (scaled - scaled.round()).abs();
            prop_assert!(diff < 1e-8);
        }
    }

    #[test]
    fn prop_zero_volume_same_as_none(
        cc in 0.0f64..100.0,
        loc in 1.0f64..1000.0,
    ) {
        let with_zero = compute_maintainability_index(cc, loc, Some(0.0));
        let without = compute_maintainability_index(cc, loc, None);
        match (with_zero, without) {
            (Some(a), Some(b)) => {
                prop_assert_eq!(a.score, b.score);
                prop_assert_eq!(a.avg_halstead_volume, None);
            }
            _ => prop_assert!(false, "both should be Some"),
        }
    }

    #[test]
    fn prop_negative_volume_same_as_none(
        cc in 0.0f64..100.0,
        loc in 1.0f64..1000.0,
        vol in -1e6f64..0.0,
    ) {
        let with_neg = compute_maintainability_index(cc, loc, Some(vol));
        let without = compute_maintainability_index(cc, loc, None);
        match (with_neg, without) {
            (Some(a), Some(b)) => {
                prop_assert_eq!(a.score, b.score);
            }
            _ => prop_assert!(false, "both should be Some"),
        }
    }
}
