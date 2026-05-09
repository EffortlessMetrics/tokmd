//! Fun and novelty analysis receipt DTOs.
//!
//! These contract types remain re-exported from the crate root to preserve
//! existing `tokmd_analysis_types::...` names.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunReport {
    pub eco_label: Option<EcoLabel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcoLabel {
    pub score: f64,
    pub label: String,
    pub bytes: u64,
    pub notes: String,
}
