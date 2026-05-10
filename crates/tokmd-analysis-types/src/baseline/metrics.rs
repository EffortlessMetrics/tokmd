//! Aggregate baseline metric receipt DTOs.
//!
//! This submodule owns baseline-wide numeric metrics while preserving the
//! existing `BaselineMetrics` re-export from `tokmd_analysis_types`.

use serde::{Deserialize, Serialize};

/// Aggregate baseline metrics for the entire codebase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineMetrics {
    /// Total lines of code across all files.
    pub total_code_lines: u64,
    /// Total number of source files.
    pub total_files: u64,
    /// Average cyclomatic complexity across all functions.
    pub avg_cyclomatic: f64,
    /// Maximum cyclomatic complexity found in any function.
    pub max_cyclomatic: u32,
    /// Average cognitive complexity across all functions.
    pub avg_cognitive: f64,
    /// Maximum cognitive complexity found in any function.
    pub max_cognitive: u32,
    /// Average nesting depth across all functions.
    pub avg_nesting_depth: f64,
    /// Maximum nesting depth found in any function.
    pub max_nesting_depth: u32,
    /// Total number of functions analyzed.
    pub function_count: u64,
    /// Average function length in lines.
    pub avg_function_length: f64,
}

impl Default for BaselineMetrics {
    fn default() -> Self {
        Self {
            total_code_lines: 0,
            total_files: 0,
            avg_cyclomatic: 0.0,
            max_cyclomatic: 0,
            avg_cognitive: 0.0,
            max_cognitive: 0,
            avg_nesting_depth: 0.0,
            max_nesting_depth: 0,
            function_count: 0,
            avg_function_length: 0.0,
        }
    }
}
