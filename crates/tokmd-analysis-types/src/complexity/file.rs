//! File and function-level complexity DTOs.
//!
//! These serde-stable contract types remain re-exported from the crate root.

use serde::{Deserialize, Serialize};

use super::ComplexityRisk;

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
