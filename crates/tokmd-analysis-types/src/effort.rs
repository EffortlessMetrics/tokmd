//! Effort estimation and legacy COCOMO receipt DTOs.
//!
//! These types remain re-exported from the crate root to preserve the public
//! `tokmd_analysis_types::...` contract while keeping the DTO family in an
//! owner module.

use serde::{Deserialize, Serialize};
mod assumptions;
mod cocomo;
mod confidence;
mod delta;
mod driver;
mod model;
mod results;
mod size;

pub use assumptions::EffortAssumptions;
pub use cocomo::CocomoReport;
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
