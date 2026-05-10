//! Maintainability index DTOs for complexity receipts.
//!
//! These serde-stable contract types remain re-exported from the crate root.

use serde::{Deserialize, Serialize};

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
