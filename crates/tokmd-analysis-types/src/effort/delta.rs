//! Effort delta DTOs.
//!
//! These serde-stable contract types remain re-exported from the crate root.

use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffortDeltaReport {
    pub base: String,
    pub head: String,
    pub files_changed: usize,
    pub modules_changed: usize,
    pub langs_changed: usize,
    pub hotspot_files_touched: usize,
    pub coupled_neighbors_touched: usize,
    pub blast_radius: f64,
    pub classification: EffortDeltaClassification,
    pub effort_pm_low: f64,
    pub effort_pm_est: f64,
    pub effort_pm_high: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffortDeltaClassification {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for EffortDeltaClassification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => f.write_str("low"),
            Self::Medium => f.write_str("medium"),
            Self::High => f.write_str("high"),
            Self::Critical => f.write_str("critical"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EffortDeltaClassification;

    #[test]
    fn effort_delta_classification_display_strings_are_stable() {
        assert_eq!(EffortDeltaClassification::Low.to_string(), "low");
        assert_eq!(EffortDeltaClassification::Medium.to_string(), "medium");
        assert_eq!(EffortDeltaClassification::High.to_string(), "high");
        assert_eq!(EffortDeltaClassification::Critical.to_string(), "critical");
    }
}
