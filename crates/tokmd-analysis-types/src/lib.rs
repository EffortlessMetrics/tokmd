//! # tokmd-analysis-types
//!
//! **Tier 0 (Analysis Contract)**
//!
//! Pure data structures for analysis receipts. No I/O or business logic.
//!
//! ## What belongs here
//! * Analysis-specific receipt types and findings
//! * Schema definitions for analysis outputs
//! * Type enums for classification results
//!
//! ## What does NOT belong here
//! * Analysis computation logic (use tokmd-analysis)
//! * Formatting logic (use tokmd-format::analysis)
//! * File I/O operations

mod api_surface;
mod archetype;
mod args;
mod assets;
mod baseline;
mod churn;
mod complexity;
mod corporate;
mod dependencies;
mod derived;
mod duplication;
mod effort;
mod entropy;
mod envelope;
pub mod findings;
mod fun;
mod git;
mod imports;
mod license;
mod receipt;
mod source;
mod topics;
pub mod util;

pub use api_surface::{ApiExportItem, ApiSurfaceReport, LangApiSurface, ModuleApiRow};
pub use archetype::Archetype;
pub use args::AnalysisArgsMeta;
pub use assets::{AssetCategoryRow, AssetFileRow, AssetReport};
pub use baseline::{
    BASELINE_VERSION, BaselineComplexitySection, BaselineMetrics, ComplexityBaseline,
    DeterminismBaseline, FileBaselineEntry,
};
pub use churn::{ChurnTrend, PredictiveChurnReport, TrendClass};
pub use complexity::{
    ComplexityHistogram, ComplexityReport, ComplexityRisk, FileComplexity,
    FunctionComplexityDetail, HalsteadMetrics, MaintainabilityIndex, TechnicalDebtLevel,
    TechnicalDebtRatio,
};
pub use corporate::{CorporateFingerprint, DomainStat};
pub use dependencies::{DependencyReport, LockfileReport};
pub use derived::{
    BoilerplateReport, ContextWindowReport, DerivedReport, DerivedTotals, DistributionReport,
    FileStatRow, HistogramBucket, IntegrityReport, LangPurityReport, LangPurityRow, MaxFileReport,
    MaxFileRow, NestingReport, NestingRow, PolyglotReport, RateReport, RateRow, RatioReport,
    RatioRow, ReadingTimeReport, TestDensityReport, TodoReport, TodoTagRow, TopOffenders,
};
pub use duplication::{
    DuplicateGroup, DuplicateReport, DuplicationDensityReport, ModuleDuplicationDensityRow,
    NearDupAlgorithm, NearDupCluster, NearDupPairRow, NearDupParams, NearDupScope, NearDupStats,
    NearDuplicateReport,
};
pub use effort::{
    CocomoReport, EffortAssumptions, EffortConfidence, EffortConfidenceLevel,
    EffortDeltaClassification, EffortDeltaReport, EffortDriver, EffortDriverDirection,
    EffortEstimateReport, EffortModel, EffortResults, EffortSizeBasis, EffortTagSizeRow,
};
pub use entropy::{EntropyClass, EntropyFinding, EntropyReport};
pub use envelope::{
    Artifact, ENVELOPE_SCHEMA, Envelope, EnvelopeTool, Finding, FindingLocation, FindingSeverity,
    GateItem, GateResults, GatesEnvelope, SensorReport, ToolMeta, Verdict,
};
pub use fun::{EcoLabel, FunReport};
pub use git::{
    BusFactorRow, CodeAgeBucket, CodeAgeDistributionReport, CommitIntentCounts, CommitIntentKind,
    CommitIntentReport, CouplingRow, FreshnessReport, GitReport, HotspotRow, ModuleFreshnessRow,
    ModuleIntentRow,
};
pub use imports::{ImportEdge, ImportReport};
pub use license::{LicenseFinding, LicenseReport, LicenseSourceKind};
pub use receipt::AnalysisReceipt;
pub use source::AnalysisSource;
pub use topics::{TopicClouds, TopicTerm};
pub use util::{
    AnalysisLimits, empty_file_row, is_infra_lang, is_test_path, normalize_path, normalize_root,
    now_ms, path_depth,
};

#[cfg(test)]
pub use tokmd_scan::{gini_coefficient, percentile, round_f64, safe_ratio};

/// Schema version for analysis receipts.
/// v7: Added coupling normalization (Jaccard/Lift), commit intent classification, near-duplicate detection.
/// v8: Near-dup clusters, selection metadata, max_pairs guardrail, runtime stats.
/// v9: Added effort estimation report.
pub const ANALYSIS_SCHEMA_VERSION: u32 = 9;

#[cfg(test)]
mod tests {
    use super::*;

    // ── Schema version constant ───────────────────────────────────────
    #[test]
    fn analysis_schema_version_constant() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(ANALYSIS_SCHEMA_VERSION, 9);
        Ok(())
    }
}
