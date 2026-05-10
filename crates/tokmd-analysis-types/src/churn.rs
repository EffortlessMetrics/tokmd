//! Predictive churn receipt DTOs.
//!
//! These contract types remain re-exported from the crate root to preserve
//! existing `tokmd_analysis_types::...` names.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveChurnReport {
    pub per_module: BTreeMap<String, ChurnTrend>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChurnTrend {
    pub slope: f64,
    pub r2: f64,
    pub recent_change: i64,
    pub classification: TrendClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrendClass {
    Rising,
    Flat,
    Falling,
}

#[cfg(test)]
mod tests {
    use super::TrendClass;

    #[test]
    fn trend_class_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        for variant in [TrendClass::Rising, TrendClass::Flat, TrendClass::Falling] {
            let json = serde_json::to_string(&variant)?;
            let back: TrendClass = serde_json::from_str(&json)?;
            assert_eq!(back, variant);
        }
        Ok(())
    }

    #[test]
    fn trend_class_uses_snake_case() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(serde_json::to_string(&TrendClass::Rising)?, "\"rising\"");
        Ok(())
    }
}
