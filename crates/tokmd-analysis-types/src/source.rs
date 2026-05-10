//! Analysis source metadata receipt DTOs.
//!
//! This module owns the source snapshot embedded in analysis receipts. Public
//! consumers should keep using the crate-root `AnalysisSource` re-export.

use serde::{Deserialize, Serialize};

/// Source metadata recorded in an analysis receipt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSource {
    pub inputs: Vec<String>,
    pub export_path: Option<String>,
    pub base_receipt_path: Option<String>,
    pub export_schema_version: Option<u32>,
    pub export_generated_at_ms: Option<u128>,
    pub base_signature: Option<String>,
    pub module_roots: Vec<String>,
    pub module_depth: usize,
    pub children: String,
}
