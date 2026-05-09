//! Dependency receipt DTOs.
//!
//! These supply-chain-adjacent contract types remain re-exported from the
//! crate root to preserve existing `tokmd_analysis_types::...` names.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyReport {
    pub total: usize,
    pub lockfiles: Vec<LockfileReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockfileReport {
    pub path: String,
    pub kind: String,
    pub dependencies: usize,
}
