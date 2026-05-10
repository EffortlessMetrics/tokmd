//! Ratchet-compatible baseline complexity section DTOs.
//!
//! This submodule owns the receipt-shaped baseline complexity section while
//! preserving the existing `BaselineComplexitySection` re-export from
//! `tokmd_analysis_types`.

use serde::{Deserialize, Serialize};

/// Complexity section mirroring analysis receipt structure for ratchet compatibility.
///
/// This provides the same field names as `ComplexityReport` so that JSON pointers
/// like `/complexity/avg_cyclomatic` work consistently across baselines and receipts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineComplexitySection {
    /// Total number of functions analyzed.
    pub total_functions: usize,
    /// Average function length in lines.
    pub avg_function_length: f64,
    /// Maximum function length found.
    pub max_function_length: usize,
    /// Average cyclomatic complexity across all files.
    pub avg_cyclomatic: f64,
    /// Maximum cyclomatic complexity found in any file.
    pub max_cyclomatic: usize,
    /// Average cognitive complexity across all files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_cognitive: Option<f64>,
    /// Maximum cognitive complexity found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_cognitive: Option<usize>,
    /// Average nesting depth across all files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_nesting_depth: Option<f64>,
    /// Maximum nesting depth found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_nesting_depth: Option<usize>,
    /// Number of high-risk files.
    pub high_risk_files: usize,
}
