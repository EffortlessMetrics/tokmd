//! Complexity, Halstead, maintainability, and technical-debt receipt DTOs.
//!
//! These contract types remain re-exported from the crate root to preserve
//! existing `tokmd_analysis_types::...` names.

use serde::{Deserialize, Serialize};

mod file;
mod halstead;
mod histogram;
mod maintainability;
mod risk;
mod technical_debt;

pub use file::{FileComplexity, FunctionComplexityDetail};
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
