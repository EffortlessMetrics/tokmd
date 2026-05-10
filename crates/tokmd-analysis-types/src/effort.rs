//! Effort estimation and legacy COCOMO receipt DTOs.
//!
//! These types remain re-exported from the crate root to preserve the public
//! `tokmd_analysis_types::...` contract while keeping the DTO family in an
//! owner module.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod confidence;
mod delta;
mod driver;
mod model;

pub use confidence::{EffortConfidence, EffortConfidenceLevel};
pub use delta::{EffortDeltaClassification, EffortDeltaReport};
pub use driver::{EffortDriver, EffortDriverDirection};
pub use model::EffortModel;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffortSizeBasis {
    pub total_lines: usize,
    pub authored_lines: usize,
    pub generated_lines: usize,
    pub vendored_lines: usize,
    pub kloc_total: f64,
    pub kloc_authored: f64,
    pub generated_pct: f64,
    pub vendored_pct: f64,
    pub classification_confidence: EffortConfidenceLevel,
    pub warnings: Vec<String>,
    pub by_tag: Vec<EffortTagSizeRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffortTagSizeRow {
    pub tag: String,
    pub lines: usize,
    pub authored_lines: usize,
    pub pct_of_total: f64,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffortAssumptions {
    pub notes: Vec<String>,
    pub overrides: BTreeMap<String, String>,
}

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
