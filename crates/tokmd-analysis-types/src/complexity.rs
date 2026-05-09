//! Complexity, Halstead, maintainability, and technical-debt receipt DTOs.
//!
//! These contract types remain re-exported from the crate root to preserve
//! existing `tokmd_analysis_types::...` names.

use serde::{Deserialize, Serialize};

/// Halstead software science metrics computed from operator/operand token counts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HalsteadMetrics {
    /// Number of distinct operators (n1).
    pub distinct_operators: usize,
    /// Number of distinct operands (n2).
    pub distinct_operands: usize,
    /// Total number of operators (N1).
    pub total_operators: usize,
    /// Total number of operands (N2).
    pub total_operands: usize,
    /// Program vocabulary: n1 + n2.
    pub vocabulary: usize,
    /// Program length: N1 + N2.
    pub length: usize,
    /// Volume: N * log2(n).
    pub volume: f64,
    /// Difficulty: (n1/2) * (N2/n2).
    pub difficulty: f64,
    /// Effort: D * V.
    pub effort: f64,
    /// Estimated programming time in seconds: E / 18.
    pub time_seconds: f64,
    /// Estimated number of bugs: V / 3000.
    pub estimated_bugs: f64,
}

/// Composite maintainability index based on the SEI formula.
///
/// MI = 171 - 5.2 * ln(V) - 0.23 * CC - 16.2 * ln(LOC)
///
/// When Halstead volume is unavailable, a simplified formula is used.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintainabilityIndex {
    /// Maintainability index score (0-171 scale, higher is better).
    pub score: f64,
    /// Average cyclomatic complexity used in calculation.
    pub avg_cyclomatic: f64,
    /// Average lines of code per file used in calculation.
    pub avg_loc: f64,
    /// Average Halstead volume (if Halstead metrics were computed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_halstead_volume: Option<f64>,
    /// Letter grade: "A" (>=85), "B" (65-84), "C" (<65).
    pub grade: String,
}

/// Complexity-to-size ratio heuristic for technical debt estimation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalDebtRatio {
    /// Complexity points per KLOC (higher means denser debt).
    pub ratio: f64,
    /// Aggregate complexity points used in the ratio.
    pub complexity_points: usize,
    /// KLOC basis used in the ratio denominator.
    pub code_kloc: f64,
    /// Bucketed interpretation of debt ratio.
    pub level: TechnicalDebtLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TechnicalDebtLevel {
    Low,
    Moderate,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityReport {
    pub total_functions: usize,
    pub avg_function_length: f64,
    pub max_function_length: usize,
    pub avg_cyclomatic: f64,
    pub max_cyclomatic: usize,
    /// Average cognitive complexity across files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_cognitive: Option<f64>,
    /// Maximum cognitive complexity found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_cognitive: Option<usize>,
    /// Average nesting depth across files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_nesting_depth: Option<f64>,
    /// Maximum nesting depth found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_nesting_depth: Option<usize>,
    pub high_risk_files: usize,
    /// Histogram of cyclomatic complexity distribution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub histogram: Option<ComplexityHistogram>,
    /// Halstead software science metrics (requires `halstead` feature).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub halstead: Option<HalsteadMetrics>,
    /// Composite maintainability index.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maintainability_index: Option<MaintainabilityIndex>,
    /// Complexity-to-size debt heuristic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technical_debt: Option<TechnicalDebtRatio>,
    pub files: Vec<FileComplexity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileComplexity {
    pub path: String,
    pub module: String,
    pub function_count: usize,
    pub max_function_length: usize,
    pub cyclomatic_complexity: usize,
    /// Cognitive complexity for this file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cognitive_complexity: Option<usize>,
    /// Maximum nesting depth in this file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_nesting: Option<usize>,
    pub risk_level: ComplexityRisk,
    /// Function-level complexity details (only when --detail-functions is used).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub functions: Option<Vec<FunctionComplexityDetail>>,
}

/// Function-level complexity details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionComplexityDetail {
    /// Function name.
    pub name: String,
    /// Start line (1-indexed).
    pub line_start: usize,
    /// End line (1-indexed).
    pub line_end: usize,
    /// Function length in lines.
    pub length: usize,
    /// Cyclomatic complexity.
    pub cyclomatic: usize,
    /// Cognitive complexity (if computed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cognitive: Option<usize>,
    /// Maximum nesting depth within the function.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_nesting: Option<usize>,
    /// Number of parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param_count: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplexityRisk {
    Low,
    Moderate,
    High,
    Critical,
}

/// Histogram of cyclomatic complexity distribution across files.
///
/// Used to visualize the distribution of complexity values in a codebase.
/// Default bucket boundaries are 0-4, 5-9, 10-14, 15-19, 20-24, 25-29, 30+.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityHistogram {
    /// Bucket boundaries (e.g., [0, 5, 10, 15, 20, 25, 30]).
    pub buckets: Vec<u32>,
    /// Count of files in each bucket.
    pub counts: Vec<u32>,
    /// Total files analyzed.
    pub total: u32,
}

impl ComplexityHistogram {
    /// Generate an ASCII bar chart visualization of the histogram.
    ///
    /// # Arguments
    /// * `width` - Maximum width of the bars in characters
    ///
    /// # Returns
    /// A multi-line string with labeled bars showing distribution
    pub fn to_ascii(&self, width: usize) -> String {
        use std::fmt::Write;
        let max_count = self.counts.iter().max().copied().unwrap_or(1).max(1);
        let mut output = String::with_capacity(self.counts.len() * (width + 20));
        for (i, count) in self.counts.iter().enumerate() {
            if i < self.buckets.len() - 1 {
                let _ = write!(
                    output,
                    "{:>2}-{:<2} |",
                    self.buckets[i],
                    self.buckets[i + 1] - 1
                );
            } else {
                let _ = write!(
                    output,
                    "{:>2}+  |",
                    self.buckets.get(i).copied().unwrap_or(30)
                );
            }

            let bar_len = (*count as f64 / max_count as f64 * width as f64) as usize;
            for _ in 0..bar_len {
                output.push('\u{2588}');
            }
            let _ = writeln!(output, " {}", count);
        }
        output
    }
}
