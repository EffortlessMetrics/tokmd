//! Academic reference values and known MI computations.
//!
//! The maintainability index formula originates from Oman & Hagemeister (1992)
//! and was later adopted by the Software Engineering Institute (SEI).
//!
//! Simplified: MI = 171 − 0.23 × CC − 16.2 × ln(LOC)
//! Full:       MI = 171 − 5.2 × ln(V) − 0.23 × CC − 16.2 × ln(LOC)
//!
//! These tests verify known computation results against the formulas.

use tokmd_analysis_maintainability::compute_maintainability_index;

/// Helper: manually compute simplified MI for verification.
fn expected_simplified(cc: f64, loc: f64) -> f64 {
    let raw = 171.0 - 0.23 * cc - 16.2 * loc.ln();
    round2(raw.max(0.0))
}

/// Helper: manually compute full MI for verification.
fn expected_full(cc: f64, loc: f64, vol: f64) -> f64 {
    let raw = 171.0 - 5.2 * vol.ln() - 0.23 * cc - 16.2 * loc.ln();
    round2(raw.max(0.0))
}

fn round2(val: f64) -> f64 {
    (val * 100.0).round() / 100.0
}

// ---------------------------------------------------------------------------
// Reference: trivial identity cases
// ---------------------------------------------------------------------------

#[test]
fn identity_loc_1_cc_0_no_halstead() {
    // MI = 171 - 0 - 16.2*ln(1) = 171 - 0 = 171.0
    let mi = compute_maintainability_index(0.0, 1.0, None).unwrap();
    assert_eq!(mi.score, 171.0);
    assert_eq!(mi.grade, "A");
}

#[test]
fn identity_volume_1_equals_simplified() {
    // ln(1) = 0, so the Halstead term vanishes: full formula = simplified
    let simplified = compute_maintainability_index(10.0, 100.0, None).unwrap();
    let full = compute_maintainability_index(10.0, 100.0, Some(1.0)).unwrap();
    assert_eq!(simplified.score, full.score);
}

#[test]
fn identity_cc_0_removes_complexity_penalty() {
    // MI = 171 - 16.2*ln(LOC), the CC term is zero
    let mi = compute_maintainability_index(0.0, 50.0, None).unwrap();
    let expected = round2(171.0 - 16.2 * 50.0_f64.ln());
    assert!((mi.score - expected).abs() < f64::EPSILON);
}

// ---------------------------------------------------------------------------
// Reference: well-maintained small module
// ---------------------------------------------------------------------------

#[test]
fn reference_small_well_maintained_module() {
    // Typical small utility: CC=3, LOC=30, no Halstead
    // MI = 171 - 0.23*3 - 16.2*ln(30) ≈ 171 - 0.69 - 55.10 ≈ 115.21
    let mi = compute_maintainability_index(3.0, 30.0, None).unwrap();
    let expected = expected_simplified(3.0, 30.0);
    assert!((mi.score - expected).abs() < f64::EPSILON);
    assert_eq!(mi.grade, "A");
}

#[test]
fn reference_small_module_with_halstead() {
    // CC=3, LOC=30, V=80
    let mi = compute_maintainability_index(3.0, 30.0, Some(80.0)).unwrap();
    let expected = expected_full(3.0, 30.0, 80.0);
    assert!((mi.score - expected).abs() < f64::EPSILON);
    assert_eq!(mi.grade, "A");
}

// ---------------------------------------------------------------------------
// Reference: moderate complexity module
// ---------------------------------------------------------------------------

#[test]
fn reference_moderate_module_simplified() {
    // CC=15, LOC=300
    let mi = compute_maintainability_index(15.0, 300.0, None).unwrap();
    let expected = expected_simplified(15.0, 300.0);
    assert!((mi.score - expected).abs() < f64::EPSILON);
    assert_eq!(mi.grade, "B");
}

#[test]
fn reference_moderate_module_with_halstead() {
    // CC=15, LOC=300, V=1000
    let mi = compute_maintainability_index(15.0, 300.0, Some(1000.0)).unwrap();
    let expected = expected_full(15.0, 300.0, 1000.0);
    assert!((mi.score - expected).abs() < f64::EPSILON);
    // With Halstead, score drops into C territory
    assert_eq!(mi.grade, "C");
}

// ---------------------------------------------------------------------------
// Reference: complex legacy module
// ---------------------------------------------------------------------------

#[test]
fn reference_legacy_high_complexity() {
    // CC=50, LOC=2000, V=8000 — typical legacy monolith file
    let mi = compute_maintainability_index(50.0, 2000.0, Some(8000.0)).unwrap();
    let expected = expected_full(50.0, 2000.0, 8000.0);
    assert!((mi.score - expected).abs() < f64::EPSILON);
    assert_eq!(mi.grade, "C");
}

#[test]
fn reference_legacy_simplified_still_low() {
    // CC=80, LOC=5000 — even simplified formula yields low score
    let mi = compute_maintainability_index(80.0, 5000.0, None).unwrap();
    let expected = expected_simplified(80.0, 5000.0);
    assert!((mi.score - expected).abs() < f64::EPSILON);
    assert_eq!(mi.grade, "C");
}

// ---------------------------------------------------------------------------
// Reference: Euler's number (e) as LOC — ln(e) = 1
// ---------------------------------------------------------------------------

#[test]
fn reference_loc_is_e() {
    // LOC = e ≈ 2.71828, ln(e) = 1
    // MI = 171 - 0.23*CC - 16.2*1 = 171 - 0.23*CC - 16.2
    let e = std::f64::consts::E;
    let mi = compute_maintainability_index(10.0, e, None).unwrap();
    // avg_loc is rounded to 2 decimals: round(2.71828*100)/100 = 2.72
    // So MI = 171 - 2.3 - 16.2*ln(2.72)
    let loc_rounded = round2(e);
    let expected = expected_simplified(10.0, loc_rounded);
    assert!((mi.score - expected).abs() < f64::EPSILON);
}

#[test]
fn reference_volume_is_e() {
    // V = e, ln(e) = 1 → Halstead penalty = 5.2 * 1 = 5.2
    let e = std::f64::consts::E;
    let simplified = compute_maintainability_index(10.0, 100.0, None).unwrap();
    let full = compute_maintainability_index(10.0, 100.0, Some(e)).unwrap();
    let diff = simplified.score - full.score;
    // The difference should be approximately 5.2 (the ln(e) coefficient)
    assert!((diff - 5.2).abs() < 0.01);
}

// ---------------------------------------------------------------------------
// Reference: table of known values
// ---------------------------------------------------------------------------

#[test]
fn reference_table_simplified() {
    let cases: Vec<(f64, f64, f64, &str)> = vec![
        // (CC, LOC, expected_score, expected_grade)
        (1.0, 10.0, expected_simplified(1.0, 10.0), "A"),
        (5.0, 50.0, expected_simplified(5.0, 50.0), "A"),
        (10.0, 100.0, expected_simplified(10.0, 100.0), "A"),
        (20.0, 500.0, expected_simplified(20.0, 500.0), "B"),
        (50.0, 1000.0, expected_simplified(50.0, 1000.0), "C"),
        (100.0, 5000.0, expected_simplified(100.0, 5000.0), "C"),
    ];

    for (cc, loc, expected_score, expected_grade) in cases {
        let mi = compute_maintainability_index(cc, loc, None)
            .unwrap_or_else(|| panic!("None for CC={cc}, LOC={loc}"));
        assert!(
            (mi.score - expected_score).abs() < f64::EPSILON,
            "CC={cc}, LOC={loc}: expected {expected_score}, got {}",
            mi.score
        );
        assert_eq!(
            mi.grade, expected_grade,
            "CC={cc}, LOC={loc}: expected grade {expected_grade}, got {}",
            mi.grade
        );
    }
}

#[test]
fn reference_table_full() {
    let cases: Vec<(f64, f64, f64, f64, &str)> = vec![
        // (CC, LOC, Volume, expected_score, expected_grade)
        (1.0, 10.0, 20.0, expected_full(1.0, 10.0, 20.0), "A"),
        (5.0, 50.0, 100.0, expected_full(5.0, 50.0, 100.0), "B"),
        (10.0, 100.0, 200.0, expected_full(10.0, 100.0, 200.0), "B"),
        (20.0, 500.0, 2000.0, expected_full(20.0, 500.0, 2000.0), "C"),
        (
            50.0,
            2000.0,
            10000.0,
            expected_full(50.0, 2000.0, 10000.0),
            "C",
        ),
    ];

    for (cc, loc, vol, expected_score, expected_grade) in cases {
        let mi = compute_maintainability_index(cc, loc, Some(vol))
            .unwrap_or_else(|| panic!("None for CC={cc}, LOC={loc}, V={vol}"));
        assert!(
            (mi.score - expected_score).abs() < f64::EPSILON,
            "CC={cc}, LOC={loc}, V={vol}: expected {expected_score}, got {}",
            mi.score
        );
        assert_eq!(
            mi.grade, expected_grade,
            "CC={cc}, LOC={loc}, V={vol}: expected grade {expected_grade}, got {}",
            mi.grade
        );
    }
}

// ---------------------------------------------------------------------------
// Coefficient verification: isolated term contributions
// ---------------------------------------------------------------------------

#[test]
fn cc_coefficient_is_0_23() {
    // Increase CC by 1, LOC constant → score should drop by exactly 0.23
    let mi1 = compute_maintainability_index(10.0, 100.0, None).unwrap();
    let mi2 = compute_maintainability_index(11.0, 100.0, None).unwrap();
    let diff = mi1.score - mi2.score;
    assert!(
        (diff - 0.23).abs() < 0.01,
        "CC coefficient: expected ~0.23 drop, got {diff}"
    );
}

#[test]
fn halstead_coefficient_is_5_2() {
    // Compare volume=e^1 vs volume=e^2 with same CC/LOC
    // Difference in ln(V) is 1, so score difference should be ~5.2
    let e = std::f64::consts::E;
    let mi1 = compute_maintainability_index(10.0, 100.0, Some(e)).unwrap();
    let mi2 = compute_maintainability_index(10.0, 100.0, Some(e * e)).unwrap();
    let diff = mi1.score - mi2.score;
    assert!(
        (diff - 5.2).abs() < 0.01,
        "Halstead coefficient: expected ~5.2 drop, got {diff}"
    );
}

#[test]
fn loc_coefficient_is_16_2() {
    // Compare LOC=e vs LOC=e^2 with same CC
    // Difference in ln(LOC) is 1, so score difference should be ~16.2
    let e = std::f64::consts::E;
    let e_rounded = round2(e);
    let e2 = e * e;
    let e2_rounded = round2(e2);
    let mi1 = compute_maintainability_index(10.0, e, None).unwrap();
    let mi2 = compute_maintainability_index(10.0, e2, None).unwrap();
    let diff = mi1.score - mi2.score;
    // Account for LOC rounding: the coefficient applies to rounded LOC
    let expected_diff = 16.2 * (e2_rounded.ln() - e_rounded.ln());
    assert!(
        (diff - expected_diff).abs() < 0.1,
        "LOC coefficient: expected ~{expected_diff} drop, got {diff}"
    );
}
