//! Top-level analysis receipt DTO.
//!
//! This module owns the serde-stable analysis receipt envelope. Public
//! consumers should keep using the crate-root `AnalysisReceipt` re-export.

use serde::{Deserialize, Serialize};
use tokmd_types::{ScanStatus, ToolInfo};

use crate::{
    AnalysisArgsMeta, AnalysisSource, ApiSurfaceReport, Archetype, AssetReport, ComplexityReport,
    CorporateFingerprint, DependencyReport, DerivedReport, DuplicateReport, EffortEstimateReport,
    EntropyReport, FunReport, GitReport, ImportReport, LicenseReport, PredictiveChurnReport,
    TopicClouds,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReceipt {
    pub schema_version: u32,
    pub generated_at_ms: u128,
    pub tool: ToolInfo,
    pub mode: String,
    pub status: ScanStatus,
    pub warnings: Vec<String>,
    pub source: AnalysisSource,
    pub args: AnalysisArgsMeta,
    pub archetype: Option<Archetype>,
    pub topics: Option<TopicClouds>,
    pub entropy: Option<EntropyReport>,
    pub predictive_churn: Option<PredictiveChurnReport>,
    pub corporate_fingerprint: Option<CorporateFingerprint>,
    pub license: Option<LicenseReport>,
    pub derived: Option<DerivedReport>,
    pub assets: Option<AssetReport>,
    pub deps: Option<DependencyReport>,
    pub git: Option<GitReport>,
    pub imports: Option<ImportReport>,
    pub dup: Option<DuplicateReport>,
    pub complexity: Option<ComplexityReport>,
    pub api_surface: Option<ApiSurfaceReport>,
    pub effort: Option<EffortEstimateReport>,
    pub fun: Option<FunReport>,
}
