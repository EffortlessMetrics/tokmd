//! Effort confidence DTOs.
//!
//! These serde-stable contract types remain re-exported from the crate root.

use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffortConfidence {
    pub level: EffortConfidenceLevel,
    pub reasons: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_coverage_pct: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffortConfidenceLevel {
    Low,
    Medium,
    High,
}

impl fmt::Display for EffortConfidenceLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => f.write_str("low"),
            Self::Medium => f.write_str("medium"),
            Self::High => f.write_str("high"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EffortConfidenceLevel;

    #[test]
    fn effort_confidence_level_display_strings_are_stable() {
        assert_eq!(EffortConfidenceLevel::Low.to_string(), "low");
        assert_eq!(EffortConfidenceLevel::Medium.to_string(), "medium");
        assert_eq!(EffortConfidenceLevel::High.to_string(), "high");
    }
}
