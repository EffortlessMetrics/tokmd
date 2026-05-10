//! Analysis argument metadata receipt DTOs.
//!
//! This module owns the serde-stable command-argument snapshot stored in
//! analysis receipts. Public consumers should keep using the crate-root
//! re-export.

use serde::{Deserialize, Serialize};

/// Command argument metadata recorded in an analysis receipt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisArgsMeta {
    pub preset: String,
    pub format: String,
    pub window_tokens: Option<usize>,
    pub git: Option<bool>,
    pub max_files: Option<usize>,
    pub max_bytes: Option<u64>,
    pub max_commits: Option<usize>,
    pub max_commit_files: Option<usize>,
    pub max_file_bytes: Option<u64>,
    pub import_granularity: String,
}
