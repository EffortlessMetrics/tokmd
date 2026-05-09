//! Import graph receipt DTOs.
//!
//! These contract types remain re-exported from the crate root to preserve
//! existing `tokmd_analysis_types::...` names.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportReport {
    pub granularity: String,
    pub edges: Vec<ImportEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportEdge {
    pub from: String,
    pub to: String,
    pub count: usize,
}
