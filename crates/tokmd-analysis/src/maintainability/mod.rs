//! Maintainability index scoring and Halstead integration helpers.

mod index;

pub(crate) use index::compute_maintainability_index;

use tokmd_analysis_types::{ComplexityReport, HalsteadMetrics};

#[cfg(test)]
#[path = "tests.rs"]
mod moved_tests;

/// Attach Halstead metrics and refresh maintainability index when possible.
///
/// The maintainability index is recomputed only when:
/// - `complexity.maintainability_index` is present, and
/// - `halstead.volume` is positive.
pub(crate) fn attach_halstead_metrics(
    complexity: &mut ComplexityReport,
    halstead: HalsteadMetrics,
) {
    if let Some(ref mut mi) = complexity.maintainability_index
        && halstead.volume > 0.0
        && let Some(updated) =
            compute_maintainability_index(mi.avg_cyclomatic, mi.avg_loc, Some(halstead.volume))
    {
        *mi = updated;
    }

    complexity.halstead = Some(halstead);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokmd_analysis_types::{ComplexityRisk, FileComplexity, TechnicalDebtRatio};

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
