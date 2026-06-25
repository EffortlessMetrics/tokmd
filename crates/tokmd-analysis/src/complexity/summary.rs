//! Aggregate summary calculations for complexity reports.

use tokmd_analysis_types::{ComplexityRisk, FileComplexity};

use super::math::round_f64;

pub(super) struct ComplexitySummary {
    pub(super) total_functions: usize,
    pub(super) avg_function_length: f64,
    pub(super) max_function_length: usize,
    pub(super) avg_cyclomatic: f64,
    pub(super) max_cyclomatic: usize,
    pub(super) avg_cognitive: Option<f64>,
    pub(super) max_cognitive: Option<usize>,
    pub(super) avg_nesting_depth: Option<f64>,
    pub(super) max_nesting_depth: Option<usize>,
    pub(super) high_risk_files: usize,
}

pub(super) fn summarize_file_complexities(
    files: &[FileComplexity],
    per_file_max_cyclomatic: &[usize],
) -> ComplexitySummary {
    debug_assert_eq!(files.len(), per_file_max_cyclomatic.len());
    let total_functions: usize = files.iter().map(|f| f.function_count).sum();
    let file_count = files.len();

    let avg_function_length = if total_functions == 0 {
        0.0
    } else {
        let total_max_len: usize = files.iter().map(|f| f.max_function_length).sum();
        round_f64(total_max_len as f64 / file_count as f64, 2)
    };

    let max_function_length = files
        .iter()
        .map(|f| f.max_function_length)
        .max()
        .unwrap_or(0);

    let avg_cyclomatic = if file_count == 0 {
        0.0
    } else {
        let total_cyclo: usize = files
            .iter()
            .filter(|f| f.function_count > 0)
            .map(|f| f.cyclomatic_complexity)
            .sum();
        if total_functions == 0 {
            0.0
        } else {
            round_f64(total_cyclo as f64 / total_functions as f64, 2)
        }
    };

    let max_cyclomatic = per_file_max_cyclomatic.iter().copied().max().unwrap_or(0);

    let cognitive_values: Vec<usize> = files
        .iter()
        .filter_map(|f| f.cognitive_complexity)
        .collect();
    let (avg_cognitive, max_cognitive) = summarize_optional_values(&cognitive_values);

    let nesting_values: Vec<usize> = files.iter().filter_map(|f| f.max_nesting).collect();
    let (avg_nesting_depth, max_nesting_depth) = summarize_optional_values(&nesting_values);

    let high_risk_files = files
        .iter()
        .filter(|f| {
            matches!(
                f.risk_level,
                ComplexityRisk::High | ComplexityRisk::Critical
            )
        })
        .count();

    ComplexitySummary {
        total_functions,
        avg_function_length,
        max_function_length,
        avg_cyclomatic,
        max_cyclomatic,
        avg_cognitive,
        max_cognitive,
        avg_nesting_depth,
        max_nesting_depth,
        high_risk_files,
    }
}

fn summarize_optional_values(values: &[usize]) -> (Option<f64>, Option<usize>) {
    if values.is_empty() {
        (None, None)
    } else {
        let total: usize = values.iter().sum();
        let max = values.iter().copied().max().unwrap_or(0);
        (
            Some(round_f64(total as f64 / values.len() as f64, 2)),
            Some(max),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(
        function_count: usize,
        max_function_length: usize,
        cyclomatic_complexity: usize,
        cognitive_complexity: Option<usize>,
        max_nesting: Option<usize>,
        risk_level: ComplexityRisk,
    ) -> FileComplexity {
        FileComplexity {
            path: "src/lib.rs".to_string(),
            module: "src".to_string(),
            function_count,
            max_function_length,
            cyclomatic_complexity,
            cognitive_complexity,
            max_nesting,
            risk_level,
            functions: None,
        }
    }

    #[test]
    fn empty_input_produces_zero_summary() {
        let summary = summarize_file_complexities(&[], &[]);

        assert_eq!(summary.total_functions, 0);
        assert_eq!(summary.avg_function_length, 0.0);
        assert_eq!(summary.max_function_length, 0);
        assert_eq!(summary.avg_cyclomatic, 0.0);
        assert_eq!(summary.max_cyclomatic, 0);
        assert_eq!(summary.avg_cognitive, None);
        assert_eq!(summary.max_cognitive, None);
        assert_eq!(summary.avg_nesting_depth, None);
        assert_eq!(summary.max_nesting_depth, None);
        assert_eq!(summary.high_risk_files, 0);
    }

    #[test]
    fn summarizes_file_complexity_aggregates() {
        let files = vec![
            file(2, 10, 3, Some(4), Some(2), ComplexityRisk::Low),
            file(4, 21, 8, Some(9), Some(5), ComplexityRisk::High),
            file(0, 0, 1, None, None, ComplexityRisk::Critical),
        ];

        let per_file_max = [2, 5, 0];
        let summary = summarize_file_complexities(&files, &per_file_max);

        assert_eq!(summary.total_functions, 6);
        assert_eq!(summary.avg_function_length, 10.33);
        assert_eq!(summary.max_function_length, 21);
        assert_eq!(summary.avg_cyclomatic, 1.83);
        assert_eq!(summary.max_cyclomatic, 5);
        assert_eq!(summary.avg_cognitive, Some(6.5));
        assert_eq!(summary.max_cognitive, Some(9));
        assert_eq!(summary.avg_nesting_depth, Some(3.5));
        assert_eq!(summary.max_nesting_depth, Some(5));
        assert_eq!(summary.high_risk_files, 2);
    }

    #[test]
    fn avg_cyclomatic_uses_function_count_not_file_count() {
        let files = vec![
            file(2, 10, 10, None, None, ComplexityRisk::Low),
            file(3, 12, 15, None, None, ComplexityRisk::Low),
        ];

        let per_file_max = [7, 8];
        let summary = summarize_file_complexities(&files, &per_file_max);

        assert_eq!(summary.avg_cyclomatic, 5.0);
        assert_eq!(summary.max_cyclomatic, 8);
        assert!(summary.avg_cyclomatic <= summary.max_cyclomatic as f64);
    }
}
