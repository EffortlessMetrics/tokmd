//! Complexity, Halstead, maintainability, and technical-debt receipt DTOs.
//!
//! These contract types remain re-exported from the crate root to preserve
//! existing `tokmd_analysis_types::...` names.

use serde::{Deserialize, Serialize};

mod halstead;
mod histogram;
mod maintainability;
mod risk;
mod technical_debt;

pub use halstead::HalsteadMetrics;
pub use histogram::ComplexityHistogram;
pub use maintainability::MaintainabilityIndex;
pub use risk::ComplexityRisk;
pub use technical_debt::{TechnicalDebtLevel, TechnicalDebtRatio};

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
