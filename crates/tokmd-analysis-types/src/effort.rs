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
mod results;
mod size;

pub use confidence::{EffortConfidence, EffortConfidenceLevel};
pub use delta::{EffortDeltaClassification, EffortDeltaReport};
pub use driver::{EffortDriver, EffortDriverDirection};
pub use model::EffortModel;
pub use results::EffortResults;
pub use size::{EffortSizeBasis, EffortTagSizeRow};

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
