//! Effort estimate receipt DTOs.
//!
//! These types remain re-exported from the crate root to preserve the public
//! `tokmd_analysis_types::...` contract while keeping effort-family DTO
//! ownership local.

use serde::{Deserialize, Serialize};

use super::{
    EffortAssumptions, EffortConfidence, EffortDeltaReport, EffortDriver, EffortModel,
    EffortResults, EffortSizeBasis,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffortEstimateReport {
    pub model: EffortModel,
    pub size_basis: EffortSizeBasis,
    pub results: EffortResults,
    pub confidence: EffortConfidence,
    pub drivers: Vec<EffortDriver>,
    pub assumptions: EffortAssumptions,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<EffortDeltaReport>,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        EffortAssumptions, EffortConfidence, EffortEstimateReport, EffortModel, EffortResults,
        EffortSizeBasis,
    };
    use crate::EffortConfidenceLevel;

    #[test]
    fn effort_estimate_report_roundtrip_preserves_optional_delta_absence() {
        let report = EffortEstimateReport {
            model: EffortModel::Ensemble,
            size_basis: EffortSizeBasis {
                total_lines: 100,
                authored_lines: 90,
                generated_lines: 5,
                vendored_lines: 5,
                kloc_total: 0.1,
                kloc_authored: 0.09,
                generated_pct: 5.0,
                vendored_pct: 5.0,
                classification_confidence: EffortConfidenceLevel::High,
                warnings: Vec::new(),
                by_tag: Vec::new(),
            },
            results: EffortResults {
                effort_pm_p50: 4.2,
                schedule_months_p50: 2.1,
                staff_p50: 2.0,
                effort_pm_low: 3.0,
                effort_pm_p80: 5.0,
                schedule_months_low: 1.8,
                schedule_months_p80: 2.6,
                staff_low: 1.5,
                staff_p80: 2.4,
            },
            confidence: EffortConfidence {
                level: EffortConfidenceLevel::Medium,
                reasons: vec!["fixture".to_string()],
                data_coverage_pct: Some(75.0),
            },
            drivers: Vec::new(),
            assumptions: EffortAssumptions {
                notes: vec!["default blended estimate".to_string()],
                overrides: BTreeMap::new(),
            },
            delta: None,
        };

        let json = serde_json::to_string(&report).unwrap();
        assert!(!json.contains("delta"));

        let back: EffortEstimateReport = serde_json::from_str(&json).unwrap();
        assert!(back.delta.is_none());
        assert_eq!(back.model, EffortModel::Ensemble);
        assert_eq!(back.assumptions.notes, ["default blended estimate"]);
    }
}
