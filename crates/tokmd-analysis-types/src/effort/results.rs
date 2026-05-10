//! Effort result DTOs.
//!
//! These serde-stable contract types remain re-exported from the crate root.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffortResults {
    pub effort_pm_p50: f64,
    pub schedule_months_p50: f64,
    pub staff_p50: f64,
    pub effort_pm_low: f64,
    pub effort_pm_p80: f64,
    pub schedule_months_low: f64,
    pub schedule_months_p80: f64,
    pub staff_low: f64,
    pub staff_p80: f64,
}

#[cfg(test)]
mod tests {
    use super::EffortResults;

    #[test]
    fn effort_results_roundtrip_preserves_bounds() {
        let results = EffortResults {
            effort_pm_p50: 10.0,
            schedule_months_p50: 4.0,
            staff_p50: 2.5,
            effort_pm_low: 7.0,
            effort_pm_p80: 15.0,
            schedule_months_low: 3.0,
            schedule_months_p80: 6.0,
            staff_low: 2.0,
            staff_p80: 3.0,
        };

        let json = serde_json::to_string(&results).unwrap();
        let back: EffortResults = serde_json::from_str(&json).unwrap();

        assert_eq!(back.effort_pm_low, 7.0);
        assert_eq!(back.effort_pm_p50, 10.0);
        assert_eq!(back.effort_pm_p80, 15.0);
    }
}
