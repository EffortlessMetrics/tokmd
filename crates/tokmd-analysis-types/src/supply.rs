//! Asset and dependency receipt DTOs.
//!
//! These supply-chain-adjacent contract types remain re-exported from the
//! crate root to preserve existing `tokmd_analysis_types::...` names.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetReport {
    pub total_files: usize,
    pub total_bytes: u64,
    pub categories: Vec<AssetCategoryRow>,
    pub top_files: Vec<AssetFileRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetCategoryRow {
    pub category: String,
    pub files: usize,
    pub bytes: u64,
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetFileRow {
    pub path: String,
    pub bytes: u64,
    pub category: String,
    pub extension: String,
}

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
