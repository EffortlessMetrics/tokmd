//! Complexity risk enum DTOs for complexity receipts.
//!
//! These serde-stable contract types remain re-exported from the crate root.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplexityRisk {
    Low,
    Moderate,
    High,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::ComplexityRisk;

    #[test]
    fn complexity_risk_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        for variant in [
            ComplexityRisk::Low,
            ComplexityRisk::Moderate,
            ComplexityRisk::High,
            ComplexityRisk::Critical,
        ] {
            let json = serde_json::to_string(&variant)?;
            let back: ComplexityRisk = serde_json::from_str(&json)?;
            assert_eq!(back, variant);
        }
        Ok(())
    }

    #[test]
    fn complexity_risk_uses_snake_case() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            serde_json::to_string(&ComplexityRisk::Moderate)?,
            "\"moderate\""
        );
        Ok(())
    }
}
