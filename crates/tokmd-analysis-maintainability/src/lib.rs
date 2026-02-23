//! Maintainability index scoring and Halstead integration helpers.

use tokmd_analysis_types::{ComplexityReport, HalsteadMetrics, MaintainabilityIndex};

/// Compute maintainability index using simplified or full SEI formula.
///
/// Simplified:
/// MI = 171 - 0.23 * CC - 16.2 * ln(LOC)
///
/// Full (when Halstead volume is available and positive):
/// MI = 171 - 5.2 * ln(V) - 0.23 * CC - 16.2 * ln(LOC)
pub fn compute_maintainability_index(
    avg_cyclomatic: f64,
    avg_loc: f64,
    halstead_volume: Option<f64>,
) -> Option<MaintainabilityIndex> {
    if avg_loc <= 0.0 {
        return None;
    }

    let avg_loc = round_f64(avg_loc, 2);
    let (raw_score, avg_halstead_volume) = match halstead_volume {
        Some(volume) if volume > 0.0 => (
            171.0 - 5.2 * volume.ln() - 0.23 * avg_cyclomatic - 16.2 * avg_loc.ln(),
            Some(volume),
        ),
        _ => (171.0 - 0.23 * avg_cyclomatic - 16.2 * avg_loc.ln(), None),
    };

    let score = round_f64(raw_score.max(0.0), 2);
    Some(MaintainabilityIndex {
        score,
        avg_cyclomatic,
        avg_loc,
        avg_halstead_volume,
        grade: grade_for_score(score).to_string(),
    })
}

/// Attach Halstead metrics and refresh maintainability index when possible.
///
/// The maintainability index is recomputed only when:
/// - `complexity.maintainability_index` is present, and
/// - `halstead.volume` is positive.
pub fn attach_halstead_metrics(complexity: &mut ComplexityReport, halstead: HalsteadMetrics) {
    if let Some(ref mut mi) = complexity.maintainability_index
        && halstead.volume > 0.0
        && let Some(updated) =
            compute_maintainability_index(mi.avg_cyclomatic, mi.avg_loc, Some(halstead.volume))
    {
        *mi = updated;
    }

    complexity.halstead = Some(halstead);
}

fn grade_for_score(score: f64) -> &'static str {
    if score >= 85.0 {
        "A"
    } else if score >= 65.0 {
        "B"
    } else {
        "C"
    }
}

fn round_f64(val: f64, decimals: u32) -> f64 {
    let factor = 10f64.powi(decimals as i32);
    (val * factor).round() / factor
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokmd_analysis_types::{ComplexityRisk, FileComplexity, TechnicalDebtRatio};

    #[test]
    fn compute_simplified_index() {
        let mi = compute_maintainability_index(10.0, 100.0, None).expect("mi");
        assert!((mi.score - 94.1).abs() < f64::EPSILON);
        assert_eq!(mi.grade, "A");
        assert_eq!(mi.avg_halstead_volume, None);
    }

    #[test]
    fn compute_full_index_with_halstead() {
        let mi = compute_maintainability_index(10.0, 100.0, Some(200.0)).expect("mi");
        assert!((mi.score - 66.54).abs() < f64::EPSILON);
        assert_eq!(mi.grade, "B");
        assert_eq!(mi.avg_halstead_volume, Some(200.0));
    }

    #[test]
    fn attach_halstead_recomputes_maintainability() {
        let mut complexity = sample_complexity();
        let before = complexity
            .maintainability_index
            .as_ref()
            .map(|mi| mi.score)
            .expect("maintainability");

        attach_halstead_metrics(
            &mut complexity,
            HalsteadMetrics {
                distinct_operators: 20,
                distinct_operands: 30,
                total_operators: 120,
                total_operands: 240,
                vocabulary: 50,
                length: 360,
                volume: 200.0,
                difficulty: 8.0,
                effort: 1600.0,
                time_seconds: 88.89,
                estimated_bugs: 0.0667,
            },
        );

        let mi = complexity
            .maintainability_index
            .as_ref()
            .expect("maintainability");
        assert!(mi.score < before);
        assert_eq!(mi.avg_halstead_volume, Some(200.0));
        assert_eq!(mi.grade, "B");
        assert_eq!(complexity.halstead.as_ref().map(|h| h.volume), Some(200.0));
    }

    #[test]
    fn attach_halstead_keeps_existing_index_when_volume_is_zero() {
        let mut complexity = sample_complexity();
        let before = complexity
            .maintainability_index
            .as_ref()
            .map(|mi| (mi.score, mi.avg_halstead_volume))
            .expect("maintainability");

        attach_halstead_metrics(
            &mut complexity,
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
            },
        );

        let after = complexity
            .maintainability_index
            .as_ref()
            .map(|mi| (mi.score, mi.avg_halstead_volume))
            .expect("maintainability");
        assert_eq!(before, after);
        assert_eq!(complexity.halstead.as_ref().map(|h| h.volume), Some(0.0));
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
                level: tokmd_analysis_types::TechnicalDebtLevel::Low,
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
}
