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
pub mod findings;
mod fun;
mod git;
mod imports;
mod license;
mod source;
mod topics;
pub mod util;

use serde::{Deserialize, Serialize};
use tokmd_types::{ScanStatus, ToolInfo};

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
pub use fun::{EcoLabel, FunReport};
pub use git::{
    BusFactorRow, CodeAgeBucket, CodeAgeDistributionReport, CommitIntentCounts, CommitIntentKind,
    CommitIntentReport, CouplingRow, FreshnessReport, GitReport, HotspotRow, ModuleFreshnessRow,
    ModuleIntentRow,
};
pub use imports::{ImportEdge, ImportReport};
pub use license::{LicenseFinding, LicenseReport, LicenseSourceKind};
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

// =========================
// Ecosystem Envelope (v1) — re-exported from tokmd-envelope
// =========================

/// Schema identifier for ecosystem envelope format.
/// v1: Initial envelope specification for multi-sensor integration.
pub const ENVELOPE_SCHEMA: &str = tokmd_envelope::SENSOR_REPORT_SCHEMA;

// Re-export all envelope types with backwards-compatible aliases
pub use tokmd_envelope::Artifact;
pub use tokmd_envelope::Finding;
pub use tokmd_envelope::FindingLocation;
pub use tokmd_envelope::FindingSeverity;
pub use tokmd_envelope::GateItem;
pub use tokmd_envelope::GateResults as GatesEnvelope;
pub use tokmd_envelope::SensorReport as Envelope;
pub use tokmd_envelope::ToolMeta as EnvelopeTool;
pub use tokmd_envelope::Verdict;

// Also re-export the canonical names for new code
pub use tokmd_envelope::GateResults;
pub use tokmd_envelope::SensorReport;
pub use tokmd_envelope::ToolMeta;

#[cfg(test)]
mod tests {
    use super::*;

    // ── Schema version constant ───────────────────────────────────────
    #[test]
    fn analysis_schema_version_constant() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(ANALYSIS_SCHEMA_VERSION, 9);
        Ok(())
    }

    // ── Enum serde roundtrips ─────────────────────────────────────────
    #[test]
    fn trend_class_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        for variant in [TrendClass::Rising, TrendClass::Flat, TrendClass::Falling] {
            let json = serde_json::to_string(&variant)?;
            let back: TrendClass = serde_json::from_str(&json)?;
            assert_eq!(back, variant);
        }
        Ok(())
    }

    #[test]
    fn complexity_risk_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        for variant in [
            ComplexityRisk::Low,
            ComplexityRisk::Moderate,
            ComplexityRisk::High,
            ComplexityRisk::Critical,
        ] {
            let json = serde_json::to_string(&variant)?;
            let back: ComplexityRisk = serde_json::from_str(&json)?;
            assert_eq!(back, variant);
        }
        Ok(())
    }

    #[test]
    fn technical_debt_level_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        for variant in [
            TechnicalDebtLevel::Low,
            TechnicalDebtLevel::Moderate,
            TechnicalDebtLevel::High,
            TechnicalDebtLevel::Critical,
        ] {
            let json = serde_json::to_string(&variant)?;
            let back: TechnicalDebtLevel = serde_json::from_str(&json)?;
            assert_eq!(back, variant);
        }
        Ok(())
    }

    // ── Enum naming conventions ───────────────────────────────────────
    #[test]
    fn trend_class_uses_snake_case() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(serde_json::to_string(&TrendClass::Rising)?, "\"rising\"");
        Ok(())
    }

    #[test]
    fn effort_model_display_strings_are_stable() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(EffortModel::Cocomo81Basic.to_string(), "cocomo81-basic");
        assert_eq!(EffortModel::Cocomo2Early.to_string(), "cocomo2-early");
        assert_eq!(EffortModel::Ensemble.to_string(), "ensemble");
        Ok(())
    }

    #[test]
    fn effort_confidence_level_display_strings_are_stable() -> Result<(), Box<dyn std::error::Error>>
    {
        assert_eq!(EffortConfidenceLevel::Low.to_string(), "low");
        assert_eq!(EffortConfidenceLevel::Medium.to_string(), "medium");
        assert_eq!(EffortConfidenceLevel::High.to_string(), "high");
        Ok(())
    }

    #[test]
    fn effort_delta_classification_display_strings_are_stable()
    -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(EffortDeltaClassification::Low.to_string(), "low");
        assert_eq!(EffortDeltaClassification::Medium.to_string(), "medium");
        assert_eq!(EffortDeltaClassification::High.to_string(), "high");
        assert_eq!(EffortDeltaClassification::Critical.to_string(), "critical");
        Ok(())
    }

    #[test]
    fn complexity_risk_uses_snake_case() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            serde_json::to_string(&ComplexityRisk::Moderate)?,
            "\"moderate\""
        );
        Ok(())
    }

    // ── Struct serde roundtrips ───────────────────────────────────────
    #[test]
    fn eco_label_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let label = EcoLabel {
            score: 85.0,
            label: "A".into(),
            bytes: 1000,
            notes: "Good".into(),
        };
        let json = serde_json::to_string(&label)?;
        let back: EcoLabel = serde_json::from_str(&json)?;
        assert_eq!(back.label, "A");
        assert_eq!(back.bytes, 1000);
        Ok(())
    }

    #[test]
    fn topic_term_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let term = TopicTerm {
            term: "async".into(),
            score: 0.95,
            tf: 10,
            df: 3,
        };
        let json = serde_json::to_string(&term)?;
        let back: TopicTerm = serde_json::from_str(&json)?;
        assert_eq!(back.term, "async");
        assert_eq!(back.tf, 10);
        Ok(())
    }

    // ── ComplexityHistogram ───────────────────────────────────────────
    #[test]
    fn complexity_histogram_to_ascii_basic() -> Result<(), Box<dyn std::error::Error>> {
        let h = ComplexityHistogram {
            buckets: vec![0, 5, 10],
            counts: vec![10, 5, 2],
            total: 17,
        };
        let ascii = h.to_ascii(20);
        assert!(!ascii.is_empty());
        // Should have 3 lines (one per bucket)
        assert_eq!(ascii.lines().count(), 3);
        Ok(())
    }

    #[test]
    fn complexity_histogram_to_ascii_empty_counts() -> Result<(), Box<dyn std::error::Error>> {
        let h = ComplexityHistogram {
            buckets: vec![0, 5],
            counts: vec![0, 0],
            total: 0,
        };
        let ascii = h.to_ascii(20);
        assert!(!ascii.is_empty());
        Ok(())
    }
}
