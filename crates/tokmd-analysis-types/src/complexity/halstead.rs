//! Halstead software science DTOs for complexity receipts.
//!
//! These serde-stable contract types remain re-exported from the crate root.

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
