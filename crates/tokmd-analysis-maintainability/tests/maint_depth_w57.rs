//! W57 depth tests for tokmd-analysis-maintainability.

use tokmd_analysis_maintainability::{attach_halstead_metrics, compute_maintainability_index};
use tokmd_analysis_types::{
    ComplexityReport, ComplexityRisk, FileComplexity, HalsteadMetrics, MaintainabilityIndex,
    TechnicalDebtLevel, TechnicalDebtRatio,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn zero_halstead() -> HalsteadMetrics {
    HalsteadMetrics {
        distinct_operators: 0,
        distinct_operands: 0,
        total_operators: 0,
        total_operands: 0,
        vocabulary: 0,
        length: 0,
        volume: 0.0,
        difficulty: 0.0,
        effort: 0.0,
        time_seconds: 0.0,
        estimated_bugs: 0.0,
    }
}

fn sample_halstead(volume: f64) -> HalsteadMetrics {
    HalsteadMetrics {
        distinct_operators: 20,
        distinct_operands: 30,
        total_operators: 120,
        total_operands: 240,
        vocabulary: 50,
        length: 360,
        volume,
        difficulty: 8.0,
        effort: 1600.0,
        time_seconds: 88.89,
        estimated_bugs: 0.0667,
    }
}

fn sample_complexity() -> ComplexityReport {
    ComplexityReport {
        total_functions: 3,
        avg_function_length: 10.0,
        max_function_length: 20,
        avg_cyclomatic: 10.0,
        max_cyclomatic: 18,
        avg_cognitive: Some(6.5),
        max_cognitive: Some(10),
        avg_nesting_depth: Some(2.0),
        max_nesting_depth: Some(4),
        high_risk_files: 1,
        histogram: None,
        halstead: None,
        maintainability_index: compute_maintainability_index(10.0, 100.0, None),
        technical_debt: Some(TechnicalDebtRatio {
            ratio: 20.0,
            complexity_points: 20,
            code_kloc: 1.0,
            level: TechnicalDebtLevel::Low,
        }),
        files: vec![FileComplexity {
            path: "src/lib.rs".to_string(),
            module: "src".to_string(),
            function_count: 3,
            max_function_length: 20,
            cyclomatic_complexity: 18,
            cognitive_complexity: Some(10),
            max_nesting: Some(4),
            risk_level: ComplexityRisk::Moderate,
            functions: None,
        }],
    }
}

// ===========================================================================
// 1. Basic maintainability index computation
// ===========================================================================

#[test]
fn simplified_formula_low_complexity() {
    // MI = 171 - 0.23*1 - 16.2*ln(10) ≈ 171 - 0.23 - 37.30 ≈ 133.47
    let mi = compute_maintainability_index(1.0, 10.0, None).unwrap();
    assert!(mi.score > 130.0 && mi.score < 140.0);
    assert_eq!(mi.grade, "A");
    assert_eq!(mi.avg_halstead_volume, None);
}

#[test]
fn simplified_formula_medium_complexity() {
    // MI = 171 - 0.23*10 - 16.2*ln(100) ≈ 171 - 2.3 - 74.6 ≈ 94.1
    let mi = compute_maintainability_index(10.0, 100.0, None).unwrap();
    assert!((mi.score - 94.1).abs() < 0.01);
    assert_eq!(mi.grade, "A");
}

#[test]
fn full_formula_with_halstead() {
    let mi = compute_maintainability_index(10.0, 100.0, Some(200.0)).unwrap();
    assert!((mi.score - 66.54).abs() < 0.01);
    assert_eq!(mi.grade, "B");
    assert_eq!(mi.avg_halstead_volume, Some(200.0));
}

// ===========================================================================
// 2. Various code profiles
// ===========================================================================

#[test]
fn high_complexity_yields_low_score() {
    let mi = compute_maintainability_index(50.0, 5000.0, Some(10000.0)).unwrap();
    assert!(mi.score < 65.0);
    assert_eq!(mi.grade, "C");
}

#[test]
fn very_simple_code_yields_grade_a() {
    let mi = compute_maintainability_index(1.0, 5.0, None).unwrap();
    assert!(mi.score >= 85.0);
    assert_eq!(mi.grade, "A");
}

#[test]
fn moderate_halstead_volume_yields_grade_b() {
    let mi = compute_maintainability_index(5.0, 50.0, Some(500.0)).unwrap();
    assert!(mi.score >= 65.0 && mi.score < 85.0, "score={}", mi.score);
    assert_eq!(mi.grade, "B");
}

// ===========================================================================
// 3. Edge cases
// ===========================================================================

#[test]
fn zero_loc_returns_none() {
    assert!(compute_maintainability_index(10.0, 0.0, None).is_none());
}

#[test]
fn negative_loc_returns_none() {
    assert!(compute_maintainability_index(10.0, -5.0, None).is_none());
}

#[test]
fn zero_cyclomatic_accepted() {
    let mi = compute_maintainability_index(0.0, 100.0, None).unwrap();
    // MI = 171 - 0 - 16.2*ln(100) ≈ 96.4
    assert!(mi.score > 90.0);
    assert_eq!(mi.grade, "A");
}

#[test]
fn very_large_loc() {
    let mi = compute_maintainability_index(1.0, 1_000_000.0, None).unwrap();
    // ln(1M) ≈ 13.8, MI ≈ 171 - 0.23 - 223.7 → clamped to 0
    assert!(mi.score >= 0.0, "score should be clamped to 0");
}

#[test]
fn score_clamped_to_zero_floor() {
    let mi = compute_maintainability_index(200.0, 100000.0, Some(999999.0)).unwrap();
    assert_eq!(mi.score, 0.0);
    assert_eq!(mi.grade, "C");
}

#[test]
fn halstead_volume_zero_uses_simplified() {
    let mi = compute_maintainability_index(10.0, 100.0, Some(0.0)).unwrap();
    let simplified = compute_maintainability_index(10.0, 100.0, None).unwrap();
    assert!((mi.score - simplified.score).abs() < 1e-10);
}

#[test]
fn halstead_volume_negative_uses_simplified() {
    let mi = compute_maintainability_index(10.0, 100.0, Some(-5.0)).unwrap();
    let simplified = compute_maintainability_index(10.0, 100.0, None).unwrap();
    assert!((mi.score - simplified.score).abs() < 1e-10);
}

#[test]
fn single_line_file() {
    let mi = compute_maintainability_index(1.0, 1.0, None).unwrap();
    // MI = 171 - 0.23 - 16.2*ln(1) = 171 - 0.23 - 0 = 170.77
    assert!((mi.score - 170.77).abs() < 0.01);
    assert_eq!(mi.grade, "A");
}

// ===========================================================================
// 4. Deterministic ordering / repeatability
// ===========================================================================

#[test]
fn computation_is_deterministic() {
    let a = compute_maintainability_index(10.0, 100.0, Some(200.0)).unwrap();
    let b = compute_maintainability_index(10.0, 100.0, Some(200.0)).unwrap();
    assert!((a.score - b.score).abs() < 1e-10);
    assert_eq!(a.grade, b.grade);
    assert_eq!(a.avg_halstead_volume, b.avg_halstead_volume);
}

#[test]
fn grade_boundary_at_85() {
    // Find a score that's right at 85
    // MI = 171 - 0.23*CC - 16.2*ln(LOC) = 85 → 0.23*CC + 16.2*ln(LOC) = 86
    // Use LOC=100 (ln=4.605): 0.23*CC + 74.6 = 86 → CC ≈ 49.6
    let mi = compute_maintainability_index(49.5, 100.0, None).unwrap();
    assert!(mi.score >= 85.0, "score={}", mi.score);
    assert_eq!(mi.grade, "A");
}

#[test]
fn grade_boundary_at_65() {
    // MI = 171 - 0.23*CC - 16.2*ln(LOC) = 65 → 0.23*CC + 16.2*ln(LOC) = 106
    // LOC=100: 0.23*CC + 74.6 = 106 → CC ≈ 136.5
    let mi = compute_maintainability_index(136.0, 100.0, None).unwrap();
    assert!(mi.score >= 65.0, "score={}", mi.score);
    assert_eq!(mi.grade, "B");

    let mi2 = compute_maintainability_index(140.0, 100.0, None).unwrap();
    assert!(mi2.score < 65.0, "score={}", mi2.score);
    assert_eq!(mi2.grade, "C");
}

// ===========================================================================
// 5. attach_halstead_metrics
// ===========================================================================

#[test]
fn attach_halstead_recomputes_mi() {
    let mut c = sample_complexity();
    let before = c.maintainability_index.as_ref().unwrap().score;
    attach_halstead_metrics(&mut c, sample_halstead(200.0));
    let after = c.maintainability_index.as_ref().unwrap().score;
    assert!(after < before, "Full formula should lower score");
    assert_eq!(
        c.maintainability_index
            .as_ref()
            .unwrap()
            .avg_halstead_volume,
        Some(200.0)
    );
}

#[test]
fn attach_halstead_zero_volume_preserves_mi() {
    let mut c = sample_complexity();
    let before = c.maintainability_index.as_ref().unwrap().score;
    attach_halstead_metrics(&mut c, zero_halstead());
    let after = c.maintainability_index.as_ref().unwrap().score;
    assert!((before - after).abs() < 1e-10);
}

#[test]
fn attach_halstead_always_sets_halstead_field() {
    let mut c = sample_complexity();
    assert!(c.halstead.is_none());
    attach_halstead_metrics(&mut c, sample_halstead(100.0));
    assert!(c.halstead.is_some());
}

#[test]
fn attach_halstead_with_no_existing_mi() {
    let mut c = sample_complexity();
    c.maintainability_index = None;
    attach_halstead_metrics(&mut c, sample_halstead(200.0));
    // MI stays None since there was nothing to recompute
    assert!(c.maintainability_index.is_none());
    // But halstead is still attached
    assert!(c.halstead.is_some());
}

// ===========================================================================
// 6. Serde roundtrips
// ===========================================================================

#[test]
fn maintainability_index_serde_roundtrip() {
    let mi = compute_maintainability_index(10.0, 100.0, Some(200.0)).unwrap();
    let json = serde_json::to_string(&mi).unwrap();
    let back: MaintainabilityIndex = serde_json::from_str(&json).unwrap();
    assert!((mi.score - back.score).abs() < 1e-10);
    assert_eq!(mi.grade, back.grade);
    assert_eq!(mi.avg_halstead_volume, back.avg_halstead_volume);
}

#[test]
fn maintainability_index_serde_no_halstead() {
    let mi = compute_maintainability_index(5.0, 50.0, None).unwrap();
    let json = serde_json::to_string(&mi).unwrap();
    let back: MaintainabilityIndex = serde_json::from_str(&json).unwrap();
    assert_eq!(mi.grade, back.grade);
    assert_eq!(back.avg_halstead_volume, None);
}

#[test]
fn complexity_report_serde_roundtrip() {
    let mut c = sample_complexity();
    attach_halstead_metrics(&mut c, sample_halstead(300.0));
    let json = serde_json::to_string_pretty(&c).unwrap();
    let back: ComplexityReport = serde_json::from_str(&json).unwrap();
    assert_eq!(c.total_functions, back.total_functions);
    assert!(back.halstead.is_some());
    assert!(back.maintainability_index.is_some());
}

// ===========================================================================
// 7. avg_loc rounding
// ===========================================================================

#[test]
fn avg_loc_is_rounded_to_two_decimals() {
    let mi = compute_maintainability_index(1.0, 3.333333, None).unwrap();
    // avg_loc should be rounded to 3.33
    assert!((mi.avg_loc - 3.33).abs() < 1e-10);
}
