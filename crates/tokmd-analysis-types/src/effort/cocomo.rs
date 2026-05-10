//! Legacy COCOMO receipt DTOs.
//!
//! These types remain re-exported from the crate root to preserve the public
//! `tokmd_analysis_types::...` contract while keeping effort-family DTO
//! ownership local.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CocomoReport {
    pub mode: String,
    pub kloc: f64,
    pub effort_pm: f64,
    pub duration_months: f64,
    pub staff: f64,
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
}

#[cfg(test)]
mod tests {
    use super::CocomoReport;

    #[test]
    fn cocomo_report_roundtrip_preserves_model_coefficients() {
        let report = CocomoReport {
            mode: "organic".to_string(),
            kloc: 12.5,
            effort_pm: 34.0,
            duration_months: 8.0,
            staff: 4.25,
            a: 2.4,
            b: 1.05,
            c: 2.5,
            d: 0.38,
        };

        let json = serde_json::to_string(&report).unwrap();
        let back: CocomoReport = serde_json::from_str(&json).unwrap();

        assert_eq!(back.mode, "organic");
        assert_eq!(back.kloc, 12.5);
        assert_eq!(back.a, 2.4);
        assert_eq!(back.d, 0.38);
    }
}
