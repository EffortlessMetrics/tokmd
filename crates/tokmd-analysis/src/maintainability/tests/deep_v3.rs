//! Deep tests for analysis maintainability module: formula verification, edge cases.

use crate::maintainability::{attach_halstead_metrics, compute_maintainability_index};
use tokmd_analysis_types::{
    ComplexityReport, ComplexityRisk, FileComplexity, HalsteadMetrics, TechnicalDebtLevel,
    TechnicalDebtRatio,
};

// ── Helpers ─────────────────────────────────────────────────────────

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

// ── compute_maintainability_index: formula precision ────────────────

#[test]
fn simplified_formula_known_values() {
    // MI = 171 - 0.23 * 5 - 16.2 * ln(50)
    // ln(50) ≈ 3.912023005
    // MI = 171 - 1.15 - 63.375 ≈ 106.475 → rounded to 106.47 or 106.48
    let mi = compute_maintainability_index(5.0, 50.0, None).unwrap();
    let expected = 171.0 - 0.23 * 5.0 - 16.2 * 50.0_f64.ln();
    let expected_rounded = (expected * 100.0).round() / 100.0;
    assert!(
        (mi.score - expected_rounded).abs() < f64::EPSILON,
        "expected {expected_rounded}, got {}",
        mi.score
    );
}

#[test]
fn full_formula_known_values() {
    // MI = 171 - 5.2*ln(500) - 0.23*15 - 16.2*ln(200)
    let mi = compute_maintainability_index(15.0, 200.0, Some(500.0)).unwrap();
    let expected = 171.0 - 5.2 * 500.0_f64.ln() - 0.23 * 15.0 - 16.2 * 200.0_f64.ln();
    let expected_clamped = expected.max(0.0);
    let expected_rounded = (expected_clamped * 100.0).round() / 100.0;
    assert!(
        (mi.score - expected_rounded).abs() < f64::EPSILON,
        "expected {expected_rounded}, got {}",
        mi.score
    );
}

#[test]
fn full_formula_with_small_volume() {
    let mi = compute_maintainability_index(1.0, 1.0, Some(1.0)).unwrap();
    // MI = 171 - 5.2*ln(1) - 0.23*1 - 16.2*ln(1) = 171 - 0 - 0.23 - 0 = 170.77
    assert!(
        (mi.score - 170.77).abs() < f64::EPSILON,
        "expected 170.77, got {}",
        mi.score
    );
}

// ── Grade boundaries: exact thresholds ──────────────────────────────

#[test]
fn grade_a_at_exactly_85() {
    // Find CC that gives score ~85 with LOC=100
    // 171 - 0.23*CC - 16.2*ln(100) = 85
    // 0.23*CC = 171 - 74.60 - 85 = 11.40
    // CC = 49.565...
    // With CC=49.56, score should be just above 85
    let mi = compute_maintainability_index(49.56, 100.0, None).unwrap();
    // The exact score depends on rounding; verify grade assignment
    if mi.score >= 85.0 {
        assert_eq!(mi.grade, "A");
    } else {
        assert_eq!(mi.grade, "B");
    }
}

#[test]
fn grade_b_at_exactly_65() {
    // 171 - 0.23*CC - 16.2*ln(100) = 65
    // 0.23*CC = 171 - 74.60 - 65 = 31.40
    // CC = 136.52...
    let mi = compute_maintainability_index(136.52, 100.0, None).unwrap();
    if mi.score >= 65.0 {
        assert_eq!(mi.grade, "B");
    } else {
        assert_eq!(mi.grade, "C");
    }
}

#[test]
fn grade_c_for_very_high_complexity() {
    let mi = compute_maintainability_index(500.0, 1000.0, None).unwrap();
    assert_eq!(mi.grade, "C");
    assert!(mi.score < 65.0);
}

// ── Edge cases: LOC ─────────────────────────────────────────────────

#[test]
fn loc_exactly_zero_returns_none() {
    assert!(compute_maintainability_index(0.0, 0.0, None).is_none());
}

#[test]
fn loc_very_small_positive_returns_some() {
    let mi = compute_maintainability_index(0.0, 0.01, None).unwrap();
    // MI = 171 - 0 - 16.2*ln(0.01)
    // ln(0.01) ≈ -4.605
    // MI = 171 + 74.6 = 245.6 → clamped? No, max(0) still 245.6
    // But score is not clamped at 171 — only at 0. Score can exceed 171!
    assert!(mi.score > 171.0);
}

#[test]
fn loc_negative_returns_none() {
    assert!(compute_maintainability_index(0.0, -1.0, None).is_none());
    assert!(compute_maintainability_index(10.0, -0.001, None).is_none());
}

#[test]
fn loc_very_large_produces_low_score() {
    let mi = compute_maintainability_index(0.0, 1e12, None).unwrap();
    // ln(1e12) ≈ 27.63
    // MI = 171 - 16.2*27.63 ≈ 171 - 447.6 → clamped to 0
    assert_eq!(mi.score, 0.0);
    assert_eq!(mi.grade, "C");
}

// ── Edge cases: cyclomatic complexity ───────────────────────────────

#[test]
fn zero_cyclomatic_yields_higher_score() {
    let mi_zero = compute_maintainability_index(0.0, 100.0, None).unwrap();
    let mi_ten = compute_maintainability_index(10.0, 100.0, None).unwrap();
    assert!(mi_zero.score > mi_ten.score);
}

#[test]
fn very_large_cyclomatic_clamps_to_zero() {
    let mi = compute_maintainability_index(100_000.0, 100.0, None).unwrap();
    assert_eq!(mi.score, 0.0);
}

// ── Edge cases: Halstead volume ─────────────────────────────────────

#[test]
fn halstead_volume_zero_uses_simplified() {
    let with_zero = compute_maintainability_index(10.0, 100.0, Some(0.0)).unwrap();
    let simplified = compute_maintainability_index(10.0, 100.0, None).unwrap();
    assert_eq!(with_zero.score, simplified.score);
    assert_eq!(with_zero.avg_halstead_volume, None);
}

#[test]
fn halstead_volume_negative_uses_simplified() {
    let with_neg = compute_maintainability_index(10.0, 100.0, Some(-50.0)).unwrap();
    let simplified = compute_maintainability_index(10.0, 100.0, None).unwrap();
    assert_eq!(with_neg.score, simplified.score);
    assert_eq!(with_neg.avg_halstead_volume, None);
}

#[test]
fn halstead_volume_very_small_positive_still_used() {
    let mi = compute_maintainability_index(10.0, 100.0, Some(0.001)).unwrap();
    assert_eq!(mi.avg_halstead_volume, Some(0.001));
    // ln(0.001) ≈ -6.9, so 5.2 * -6.9 ≈ -35.9, which raises score
    let simplified = compute_maintainability_index(10.0, 100.0, None).unwrap();
    assert!(
        mi.score > simplified.score,
        "tiny volume (negative ln) should raise score above simplified"
    );
}

// ── Rounding behavior ───────────────────────────────────────────────

#[test]
fn score_rounded_to_2_decimals() {
    let mi = compute_maintainability_index(7.3, 42.7, None).unwrap();
    let scaled = mi.score * 100.0;
    let diff = (scaled - scaled.round()).abs();
    assert!(diff < 1e-8, "score not rounded to 2 decimals: {}", mi.score);
}

#[test]
fn avg_loc_rounded_to_2_decimals() {
    let mi = compute_maintainability_index(5.0, 33.333, None).unwrap();
    assert_eq!(mi.avg_loc, 33.33);
}

#[test]
fn avg_loc_rounding_halfway() {
    let mi = compute_maintainability_index(5.0, 99.995, None).unwrap();
    // 99.995 * 100 = 9999.5 → rounds to 10000 → 100.0
    assert_eq!(mi.avg_loc, 100.0);
}

// ── Output field verification ───────────────────────────────────────

#[test]
fn output_preserves_input_cyclomatic() {
    let mi = compute_maintainability_index(42.5, 100.0, None).unwrap();
    assert_eq!(mi.avg_cyclomatic, 42.5);
}

#[test]
fn output_preserves_halstead_volume_when_positive() {
    let mi = compute_maintainability_index(10.0, 100.0, Some(350.0)).unwrap();
    assert_eq!(mi.avg_halstead_volume, Some(350.0));
}

// ── attach_halstead_metrics: integration ────────────────────────────

#[test]
fn attach_halstead_stores_metrics_even_without_mi() {
    let mut report = sample_complexity_with_mi(10.0, 100.0);
    report.maintainability_index = None;
    attach_halstead_metrics(&mut report, make_halstead(500.0));
    assert!(report.halstead.is_some());
    assert!(report.maintainability_index.is_none());
}

#[test]
fn attach_halstead_with_positive_volume_recomputes_mi() {
    let mut report = sample_complexity_with_mi(10.0, 100.0);
    let before = report.maintainability_index.as_ref().unwrap().score;
    attach_halstead_metrics(&mut report, make_halstead(200.0));
    let after = report.maintainability_index.as_ref().unwrap().score;
    assert!(after < before, "MI should decrease with halstead volume");
    assert_eq!(
        report
            .maintainability_index
            .as_ref()
            .unwrap()
            .avg_halstead_volume,
        Some(200.0)
    );
}

#[test]
fn attach_halstead_zero_volume_preserves_mi() {
    let mut report = sample_complexity_with_mi(10.0, 100.0);
    let before = report.maintainability_index.as_ref().unwrap().score;
    attach_halstead_metrics(&mut report, make_halstead(0.0));
    let after = report.maintainability_index.as_ref().unwrap().score;
    assert_eq!(before, after);
}

#[test]
fn attach_halstead_negative_volume_preserves_mi() {
    let mut report = sample_complexity_with_mi(10.0, 100.0);
    let before = report.maintainability_index.as_ref().unwrap().score;
    attach_halstead_metrics(&mut report, make_halstead(-100.0));
    let after = report.maintainability_index.as_ref().unwrap().score;
    assert_eq!(before, after);
}

#[test]
fn attach_halstead_preserves_all_halstead_fields() {
    let mut report = sample_complexity_with_mi(10.0, 100.0);
    let h = HalsteadMetrics {
        distinct_operators: 25,
        distinct_operands: 40,
        total_operators: 150,
        total_operands: 300,
        vocabulary: 65,
        length: 450,
        volume: 800.0,
        difficulty: 12.5,
        effort: 10000.0,
        time_seconds: 555.56,
        estimated_bugs: 0.27,
    };
    attach_halstead_metrics(&mut report, h);
    let stored = report.halstead.as_ref().unwrap();
    assert_eq!(stored.distinct_operators, 25);
    assert_eq!(stored.distinct_operands, 40);
    assert_eq!(stored.total_operators, 150);
    assert_eq!(stored.total_operands, 300);
    assert_eq!(stored.vocabulary, 65);
    assert_eq!(stored.length, 450);
    assert_eq!(stored.volume, 800.0);
    assert_eq!(stored.difficulty, 12.5);
    assert_eq!(stored.effort, 10000.0);
    assert!((stored.time_seconds - 555.56).abs() < f64::EPSILON);
    assert!((stored.estimated_bugs - 0.27).abs() < f64::EPSILON);
}

#[test]
fn attach_halstead_twice_overwrites_previous() {
    let mut report = sample_complexity_with_mi(10.0, 100.0);
    attach_halstead_metrics(&mut report, make_halstead(200.0));
    let first_score = report.maintainability_index.as_ref().unwrap().score;

    attach_halstead_metrics(&mut report, make_halstead(500.0));
    let second_score = report.maintainability_index.as_ref().unwrap().score;

    assert!(
        second_score < first_score,
        "larger volume should lower MI further"
    );
    assert_eq!(report.halstead.as_ref().unwrap().volume, 500.0);
}

// ── Determinism ─────────────────────────────────────────────────────

#[test]
fn compute_is_deterministic() {
    let a = compute_maintainability_index(15.5, 250.0, Some(400.0)).unwrap();
    let b = compute_maintainability_index(15.5, 250.0, Some(400.0)).unwrap();
    assert_eq!(a.score, b.score);
    assert_eq!(a.grade, b.grade);
}
