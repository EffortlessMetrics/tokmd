//! Technical debt ratio DTOs for complexity receipts.
//!
//! These serde-stable contract types remain re-exported from the crate root.

use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::TechnicalDebtLevel;

    #[test]
    fn technical_debt_level_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        for variant in [
            TechnicalDebtLevel::Low,
            TechnicalDebtLevel::Moderate,
            TechnicalDebtLevel::High,
            TechnicalDebtLevel::Critical,
        ] {
            let json = serde_json::to_string(&variant)?;
            let back: TechnicalDebtLevel = serde_json::from_str(&json)?;
            assert_eq!(back, variant);
        }
        Ok(())
    }
}
