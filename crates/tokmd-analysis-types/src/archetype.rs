//! Project archetype analysis receipt DTOs.
//!
//! These contract types remain re-exported from the crate root to preserve
//! existing `tokmd_analysis_types::...` names.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Archetype {
    pub kind: String,
    pub evidence: Vec<String>,
}
