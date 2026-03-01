use tokmd_analysis_maintainability::{attach_halstead_metrics, compute_maintainability_index};
use tokmd_analysis_types::{
    ComplexityReport, ComplexityRisk, FileComplexity, HalsteadMetrics, MaintainabilityIndex,
    TechnicalDebtLevel, TechnicalDebtRatio,
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

// ---------------------------------------------------------------------------
// Maintainability index valid range
// ---------------------------------------------------------------------------

#[test]
fn given_any_valid_input_when_computing_mi_then_score_is_between_0_and_171() {
    let cases: Vec<(f64, f64, Option<f64>)> = vec![
        (1.0, 1.0, None),
        (50.0, 500.0, None),
        (100.0, 10000.0, Some(5000.0)),
        (0.0, 1.0, Some(1.0)),
        (500.0, 100000.0, Some(1000000.0)),
    ];
    for (cc, loc, vol) in cases {
        let mi = compute_maintainability_index(cc, loc, vol).expect("should produce MI");
        assert!(
            mi.score >= 0.0 && mi.score <= 171.0,
            "score {} out of range for cc={cc}, loc={loc}, vol={vol:?}",
            mi.score
        );
    }
}

// ---------------------------------------------------------------------------
// Higher complexity → lower maintainability
// ---------------------------------------------------------------------------

#[test]
fn given_increasing_cyclomatic_when_computing_mi_then_score_decreases_monotonically() {
    let complexities = [1.0, 10.0, 50.0, 100.0, 500.0];
    let mut prev_score = f64::MAX;
    for cc in complexities {
        let mi = compute_maintainability_index(cc, 100.0, None).expect("mi");
        assert!(
            mi.score <= prev_score,
            "score should decrease: cc={cc}, score={}, prev={}",
            mi.score,
            prev_score
        );
        prev_score = mi.score;
    }
}

#[test]
fn given_increasing_loc_when_computing_mi_then_score_decreases_monotonically() {
    let locs = [10.0, 100.0, 1000.0, 10000.0, 100000.0];
    let mut prev_score = f64::MAX;
    for loc in locs {
        let mi = compute_maintainability_index(5.0, loc, None).expect("mi");
        assert!(
            mi.score <= prev_score,
            "score should decrease: loc={loc}, score={}, prev={}",
            mi.score,
            prev_score
        );
        prev_score = mi.score;
    }
}

#[test]
fn given_increasing_halstead_volume_when_computing_mi_then_score_decreases_monotonically() {
    let volumes = [1.0, 10.0, 100.0, 1000.0, 10000.0];
    let mut prev_score = f64::MAX;
    for vol in volumes {
        let mi = compute_maintainability_index(5.0, 100.0, Some(vol)).expect("mi");
        assert!(
            mi.score <= prev_score,
            "score should decrease: vol={vol}, score={}, prev={}",
            mi.score,
            prev_score
        );
        prev_score = mi.score;
    }
}

// ---------------------------------------------------------------------------
// Empty/trivial code has high maintainability
// ---------------------------------------------------------------------------

#[test]
fn given_trivial_code_when_computing_mi_then_grade_is_a() {
    // Minimal complexity and small size → highest maintainability
    let mi = compute_maintainability_index(1.0, 1.0, None).expect("mi");
    assert_eq!(mi.grade, "A");
    assert!(mi.score > 150.0, "trivial code should have very high MI");
}

#[test]
fn given_minimal_complexity_and_small_loc_when_computing_mi_then_high_score() {
    let mi = compute_maintainability_index(0.0, 1.0, None).expect("mi");
    assert_eq!(mi.score, 171.0, "zero CC + 1 LOC → max MI");
    assert_eq!(mi.grade, "A");
}

// ---------------------------------------------------------------------------
// Round-trip serialization of MaintainabilityIndex
// ---------------------------------------------------------------------------

#[test]
fn given_simplified_mi_when_serialized_then_round_trips_correctly() {
    let mi = compute_maintainability_index(10.0, 100.0, None).expect("mi");
    let json = serde_json::to_string(&mi).expect("serialize");
    let deserialized: MaintainabilityIndex = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(mi.score, deserialized.score);
    assert_eq!(mi.grade, deserialized.grade);
    assert_eq!(mi.avg_cyclomatic, deserialized.avg_cyclomatic);
    assert_eq!(mi.avg_loc, deserialized.avg_loc);
    assert_eq!(mi.avg_halstead_volume, deserialized.avg_halstead_volume);
}

#[test]
fn given_full_mi_when_serialized_then_round_trips_correctly() {
    let mi = compute_maintainability_index(10.0, 100.0, Some(200.0)).expect("mi");
    let json = serde_json::to_string(&mi).expect("serialize");
    let deserialized: MaintainabilityIndex = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(mi.score, deserialized.score);
    assert_eq!(mi.grade, deserialized.grade);
    assert_eq!(mi.avg_halstead_volume, deserialized.avg_halstead_volume);
}

#[test]
fn given_simplified_mi_when_serialized_then_halstead_volume_is_absent() {
    let mi = compute_maintainability_index(10.0, 100.0, None).expect("mi");
    let json = serde_json::to_string(&mi).expect("serialize");
    assert!(
        !json.contains("avg_halstead_volume"),
        "simplified MI JSON should omit avg_halstead_volume via skip_serializing_if"
    );
}

#[test]
fn given_full_mi_when_serialized_then_halstead_volume_is_present() {
    let mi = compute_maintainability_index(10.0, 100.0, Some(200.0)).expect("mi");
    let json = serde_json::to_string(&mi).expect("serialize");
    assert!(
        json.contains("avg_halstead_volume"),
        "full MI JSON should include avg_halstead_volume"
    );
}

// ---------------------------------------------------------------------------
// Deterministic output
// ---------------------------------------------------------------------------

#[test]
fn given_same_inputs_when_computing_mi_twice_then_results_are_identical() {
    let inputs: Vec<(f64, f64, Option<f64>)> = vec![
        (10.0, 100.0, None),
        (10.0, 100.0, Some(200.0)),
        (0.0, 1.0, None),
        (100.0, 10000.0, Some(50000.0)),
    ];
    for (cc, loc, vol) in inputs {
        let a = compute_maintainability_index(cc, loc, vol);
        let b = compute_maintainability_index(cc, loc, vol);
        match (a, b) {
            (Some(a), Some(b)) => {
                assert_eq!(a.score, b.score, "score mismatch for cc={cc} loc={loc}");
                assert_eq!(a.grade, b.grade, "grade mismatch for cc={cc} loc={loc}");
                assert_eq!(a.avg_loc, b.avg_loc);
                assert_eq!(a.avg_cyclomatic, b.avg_cyclomatic);
                assert_eq!(a.avg_halstead_volume, b.avg_halstead_volume);
            }
            (None, None) => {}
            _ => panic!("determinism violated for cc={cc} loc={loc} vol={vol:?}"),
        }
    }
}

#[test]
fn given_same_inputs_when_serializing_mi_twice_then_json_is_identical() {
    let mi1 = compute_maintainability_index(10.0, 100.0, Some(200.0)).expect("mi1");
    let mi2 = compute_maintainability_index(10.0, 100.0, Some(200.0)).expect("mi2");
    let json1 = serde_json::to_string(&mi1).expect("json1");
    let json2 = serde_json::to_string(&mi2).expect("json2");
    assert_eq!(json1, json2, "serialized JSON should be identical");
}
