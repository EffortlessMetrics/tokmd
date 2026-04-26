//! Mathematical invariant tests for the maintainability index formula.
//!
//! These tests verify structural properties of the MI formula that must hold
//! regardless of specific input values.

use crate::maintainability::compute_maintainability_index;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Additivity / linearity of the CC term
// ---------------------------------------------------------------------------

#[test]
fn cc_contribution_is_linear() {
    // Doubling CC should double its contribution (0.23 * CC)
    // Score(CC=10) - Score(CC=20) ≈ Score(CC=20) - Score(CC=30)
    let s10 = compute_maintainability_index(10.0, 100.0, None)
        .unwrap()
        .score;
    let s20 = compute_maintainability_index(20.0, 100.0, None)
        .unwrap()
        .score;
    let s30 = compute_maintainability_index(30.0, 100.0, None)
        .unwrap()
        .score;

    let d1 = s10 - s20;
    let d2 = s20 - s30;
    assert!(
        (d1 - d2).abs() < 0.01,
        "CC contribution should be linear: d1={d1}, d2={d2}"
    );
}

#[test]
fn cc_unit_step_equals_coefficient() {
    // Each unit increase in CC reduces score by exactly 0.23
    let s1 = compute_maintainability_index(50.0, 200.0, None)
        .unwrap()
        .score;
    let s2 = compute_maintainability_index(51.0, 200.0, None)
        .unwrap()
        .score;
    let diff = s1 - s2;
    assert!(
        (diff - 0.23).abs() < 0.01,
        "expected 0.23 drop per CC unit, got {diff}"
    );
}

// ---------------------------------------------------------------------------
// Logarithmic relationship of LOC term
// ---------------------------------------------------------------------------

#[test]
fn loc_doubling_has_constant_score_drop() {
    // ln(2*LOC) - ln(LOC) = ln(2) ≈ 0.693
    // So score drop for doubling LOC is 16.2 * ln(2) ≈ 11.23
    let expected_drop = 16.2 * 2.0_f64.ln();

    let s100 = compute_maintainability_index(10.0, 100.0, None)
        .unwrap()
        .score;
    let s200 = compute_maintainability_index(10.0, 200.0, None)
        .unwrap()
        .score;
    let diff = s100 - s200;
    assert!(
        (diff - expected_drop).abs() < 0.1,
        "expected ~{expected_drop} drop for LOC doubling, got {diff}"
    );
}

#[test]
fn loc_10x_increase_has_constant_score_drop() {
    // ln(10*LOC) - ln(LOC) = ln(10) ≈ 2.302
    // Score drop = 16.2 * ln(10) ≈ 37.30
    let expected_drop = 16.2 * 10.0_f64.ln();

    let s10 = compute_maintainability_index(5.0, 10.0, None)
        .unwrap()
        .score;
    let s100 = compute_maintainability_index(5.0, 100.0, None)
        .unwrap()
        .score;
    let diff = s10 - s100;
    assert!(
        (diff - expected_drop).abs() < 0.5,
        "expected ~{expected_drop} drop for 10x LOC, got {diff}"
    );
}

// ---------------------------------------------------------------------------
// Logarithmic relationship of Halstead volume term
// ---------------------------------------------------------------------------

#[test]
fn volume_doubling_has_constant_score_drop() {
    // ln(2*V) - ln(V) = ln(2) ≈ 0.693
    // Score drop = 5.2 * ln(2) ≈ 3.60
    let expected_drop = 5.2 * 2.0_f64.ln();

    let s1 = compute_maintainability_index(10.0, 100.0, Some(100.0))
        .unwrap()
        .score;
    let s2 = compute_maintainability_index(10.0, 100.0, Some(200.0))
        .unwrap()
        .score;
    let diff = s1 - s2;
    assert!(
        (diff - expected_drop).abs() < 0.1,
        "expected ~{expected_drop} drop for volume doubling, got {diff}"
    );
}

// ---------------------------------------------------------------------------
// Superposition: each term contributes independently
// ---------------------------------------------------------------------------

#[test]
fn terms_are_additive() {
    // Score(CC=a, LOC=L) - Score(CC=b, LOC=L)
    // should equal Score(CC=a, LOC=M) - Score(CC=b, LOC=M) for any L, M
    let s_a_l = compute_maintainability_index(10.0, 100.0, None)
        .unwrap()
        .score;
    let s_b_l = compute_maintainability_index(20.0, 100.0, None)
        .unwrap()
        .score;
    let s_a_m = compute_maintainability_index(10.0, 500.0, None)
        .unwrap()
        .score;
    let s_b_m = compute_maintainability_index(20.0, 500.0, None)
        .unwrap()
        .score;

    let diff_l = s_a_l - s_b_l;
    let diff_m = s_a_m - s_b_m;
    assert!(
        (diff_l - diff_m).abs() < 0.01,
        "CC contribution should be independent of LOC: diff_l={diff_l}, diff_m={diff_m}"
    );
}

#[test]
fn halstead_and_cc_terms_independent() {
    // Changing Halstead volume should produce the same delta regardless of CC
    let s1_cc10 = compute_maintainability_index(10.0, 100.0, Some(100.0))
        .unwrap()
        .score;
    let s2_cc10 = compute_maintainability_index(10.0, 100.0, Some(200.0))
        .unwrap()
        .score;
    let s1_cc30 = compute_maintainability_index(30.0, 100.0, Some(100.0))
        .unwrap()
        .score;
    let s2_cc30 = compute_maintainability_index(30.0, 100.0, Some(200.0))
        .unwrap()
        .score;

    let delta_cc10 = s1_cc10 - s2_cc10;
    let delta_cc30 = s1_cc30 - s2_cc30;
    assert!(
        (delta_cc10 - delta_cc30).abs() < 0.01,
        "Halstead delta should be independent of CC"
    );
}

// ---------------------------------------------------------------------------
// Strict monotonicity
// ---------------------------------------------------------------------------

#[test]
fn strictly_monotone_decreasing_in_cc() {
    let scores: Vec<f64> = (0..20)
        .map(|i| {
            let cc = i as f64 * 10.0;
            compute_maintainability_index(cc, 100.0, None)
                .unwrap()
                .score
        })
        .collect();

    for window in scores.windows(2) {
        assert!(
            window[0] >= window[1],
            "MI should decrease as CC increases: {} < {}",
            window[0],
            window[1]
        );
    }
}

#[test]
fn strictly_monotone_decreasing_in_loc() {
    let locs = [1.0, 5.0, 10.0, 50.0, 100.0, 500.0, 1000.0, 5000.0];
    let scores: Vec<f64> = locs
        .iter()
        .map(|&loc| {
            compute_maintainability_index(10.0, loc, None)
                .unwrap()
                .score
        })
        .collect();

    for window in scores.windows(2) {
        assert!(
            window[0] >= window[1],
            "MI should decrease as LOC increases: {} < {}",
            window[0],
            window[1]
        );
    }
}

#[test]
fn strictly_monotone_decreasing_in_volume() {
    let vols = [1.0, 10.0, 100.0, 500.0, 1000.0, 5000.0, 10000.0];
    let scores: Vec<f64> = vols
        .iter()
        .map(|&vol| {
            compute_maintainability_index(10.0, 100.0, Some(vol))
                .unwrap()
                .score
        })
        .collect();

    for window in scores.windows(2) {
        assert!(
            window[0] >= window[1],
            "MI should decrease as volume increases: {} < {}",
            window[0],
            window[1]
        );
    }
}

// ---------------------------------------------------------------------------
// Grade monotonicity: if score decreases, grade never improves
// ---------------------------------------------------------------------------

#[test]
fn grade_never_improves_with_increasing_complexity() {
    let grade_ord = |g: &str| -> u8 {
        match g {
            "A" => 2,
            "B" => 1,
            "C" => 0,
            _ => panic!("unexpected grade"),
        }
    };

    let mut prev_grade_ord = u8::MAX;
    for cc in (0..200).step_by(5) {
        let mi = compute_maintainability_index(cc as f64, 100.0, None).unwrap();
        let g = grade_ord(&mi.grade);
        assert!(
            g <= prev_grade_ord,
            "grade should never improve as CC increases"
        );
        prev_grade_ord = g;
    }
}

// ---------------------------------------------------------------------------
// Property tests: additional mathematical invariants
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn full_formula_score_leq_simplified(
        cc in 0.0f64..500.0,
        loc in 1.0f64..1e4,
        vol in 1.0f64..1e6,
    ) {
        let simplified = compute_maintainability_index(cc, loc, None).unwrap();
        let full = compute_maintainability_index(cc, loc, Some(vol)).unwrap();
        // Since ln(vol) > 0 for vol > 1, full formula adds a negative term
        prop_assert!(full.score <= simplified.score,
            "full ({}) should be <= simplified ({})", full.score, simplified.score);
    }

    #[test]
    fn cc_monotonicity_property(
        cc1 in 0.0f64..500.0,
        cc2 in 0.0f64..500.0,
        loc in 1.0f64..1e4,
    ) {
        let mi1 = compute_maintainability_index(cc1, loc, None).unwrap();
        let mi2 = compute_maintainability_index(cc2, loc, None).unwrap();
        if cc1 < cc2 {
            prop_assert!(mi1.score >= mi2.score);
        } else if cc1 > cc2 {
            prop_assert!(mi1.score <= mi2.score);
        } else {
            prop_assert_eq!(mi1.score, mi2.score);
        }
    }

    #[test]
    fn volume_monotonicity_property(
        cc in 0.0f64..100.0,
        loc in 1.0f64..1e4,
        vol1 in 1.0f64..1e5,
        vol2 in 1.0f64..1e5,
    ) {
        let mi1 = compute_maintainability_index(cc, loc, Some(vol1)).unwrap();
        let mi2 = compute_maintainability_index(cc, loc, Some(vol2)).unwrap();
        if vol1 < vol2 {
            prop_assert!(mi1.score >= mi2.score,
                "higher volume should yield lower score: V1={vol1} (score={}) vs V2={vol2} (score={})",
                mi1.score, mi2.score);
        }
    }

    #[test]
    fn grade_consistent_with_score(
        cc in 0.0f64..500.0,
        loc in 1.0f64..1e5,
        vol in proptest::option::of(1.0f64..1e5),
    ) {
        if let Some(mi) = compute_maintainability_index(cc, loc, vol) {
            match mi.grade.as_str() {
                "A" => prop_assert!(mi.score >= 85.0),
                "B" => prop_assert!(mi.score >= 65.0 && mi.score < 85.0),
                "C" => prop_assert!(mi.score < 65.0),
                g => prop_assert!(false, "unexpected grade: {g}"),
            }
        }
    }

    #[test]
    fn cc_delta_is_constant_0_23(
        cc in 0.0f64..499.0,
        loc in 1.0f64..1e4,
    ) {
        let mi1 = compute_maintainability_index(cc, loc, None).unwrap();
        let mi2 = compute_maintainability_index(cc + 1.0, loc, None).unwrap();
        // Both scores are clamped to 0, so delta may be less when at floor
        if mi1.score > 0.0 && mi2.score > 0.0 {
            let delta = mi1.score - mi2.score;
            prop_assert!((delta - 0.23).abs() < 0.01,
                "CC unit delta should be ~0.23, got {delta}");
        }
    }

    #[test]
    fn symmetry_of_none_and_zero_volume(
        cc in 0.0f64..100.0,
        loc in 1.0f64..1e4,
    ) {
        let none_vol = compute_maintainability_index(cc, loc, None).unwrap();
        let zero_vol = compute_maintainability_index(cc, loc, Some(0.0)).unwrap();
        prop_assert_eq!(none_vol.score, zero_vol.score,
            "None and Some(0.0) should yield same score");
    }

    #[test]
    fn volume_1_equals_simplified(
        cc in 0.0f64..100.0,
        loc in 1.0f64..1e4,
    ) {
        // ln(1) = 0, so full formula with V=1 should equal simplified
        let simplified = compute_maintainability_index(cc, loc, None).unwrap();
        let full = compute_maintainability_index(cc, loc, Some(1.0)).unwrap();
        prop_assert_eq!(simplified.score, full.score,
            "V=1 should give same score as simplified");
    }
}
