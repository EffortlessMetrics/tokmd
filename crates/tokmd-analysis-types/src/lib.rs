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
mod assets;
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
mod topics;
pub mod util;

use serde::{Deserialize, Serialize};
use tokmd_types::{ScanStatus, ToolInfo};

pub use api_surface::{ApiExportItem, ApiSurfaceReport, LangApiSurface, ModuleApiRow};
pub use archetype::Archetype;
pub use assets::{AssetCategoryRow, AssetFileRow, AssetReport};
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

// -------------------
// Baseline/Ratchet types
// -------------------

/// Schema version for baseline files.
/// v1: Initial baseline format with complexity and determinism tracking.
pub const BASELINE_VERSION: u32 = 1;

/// Complexity baseline for tracking trends over time.
///
/// Used by the ratchet system to enforce that complexity metrics
/// do not regress across commits. The baseline captures a snapshot
/// of complexity at a known-good state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityBaseline {
    /// Schema version for forward compatibility.
    pub baseline_version: u32,
    /// ISO 8601 timestamp when this baseline was generated.
    pub generated_at: String,
    /// Git commit SHA at which this baseline was captured, if available.
    pub commit: Option<String>,
    /// Aggregate complexity metrics.
    pub metrics: BaselineMetrics,
    /// Per-file baseline entries for granular tracking.
    pub files: Vec<FileBaselineEntry>,
    /// Complexity section mirroring analysis receipt structure for ratchet compatibility.
    ///
    /// This allows using the same JSON pointers (e.g., `/complexity/avg_cyclomatic`)
    /// when comparing baselines against current analysis receipts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complexity: Option<BaselineComplexitySection>,
    /// Determinism baseline for reproducibility verification.
    ///
    /// Present when the baseline was generated with `--determinism`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub determinism: Option<DeterminismBaseline>,
}

impl ComplexityBaseline {
    /// Creates a new empty baseline with default values.
    pub fn new() -> Self {
        Self {
            baseline_version: BASELINE_VERSION,
            generated_at: String::new(),
            commit: None,
            metrics: BaselineMetrics::default(),
            files: Vec::new(),
            complexity: None,
            determinism: None,
        }
    }

    /// Creates a baseline from an analysis receipt.
    ///
    /// Extracts complexity information from the receipt's complexity report
    /// and derived totals to build a baseline snapshot.
    pub fn from_analysis(receipt: &AnalysisReceipt) -> Self {
        let generated_at = chrono_timestamp_iso8601(receipt.generated_at_ms);

        let total_code_lines = receipt
            .derived
            .as_ref()
            .map(|d| d.totals.code as u64)
            .unwrap_or(0);
        let total_files = receipt
            .derived
            .as_ref()
            .map(|d| d.totals.files as u64)
            .unwrap_or(0);

        let (metrics, files, complexity) = if let Some(ref complexity_report) = receipt.complexity {
            let metrics = BaselineMetrics {
                total_code_lines,
                total_files,
                avg_cyclomatic: complexity_report.avg_cyclomatic,
                max_cyclomatic: complexity_report.max_cyclomatic as u32,
                avg_cognitive: complexity_report.avg_cognitive.unwrap_or(0.0),
                max_cognitive: complexity_report.max_cognitive.unwrap_or(0) as u32,
                avg_nesting_depth: complexity_report.avg_nesting_depth.unwrap_or(0.0),
                max_nesting_depth: complexity_report.max_nesting_depth.unwrap_or(0) as u32,
                function_count: complexity_report.total_functions as u64,
                avg_function_length: complexity_report.avg_function_length,
            };

            let files: Vec<FileBaselineEntry> = complexity_report
                .files
                .iter()
                .map(|f| FileBaselineEntry {
                    path: f.path.clone(),
                    code_lines: 0, // Not available in FileComplexity
                    cyclomatic: f.cyclomatic_complexity as u32,
                    cognitive: f.cognitive_complexity.unwrap_or(0) as u32,
                    max_nesting: f.max_nesting.unwrap_or(0) as u32,
                    function_count: f.function_count as u32,
                    content_hash: None,
                })
                .collect();

            // Build complexity section mirroring analysis receipt structure
            let complexity_section = BaselineComplexitySection {
                total_functions: complexity_report.total_functions,
                avg_function_length: complexity_report.avg_function_length,
                max_function_length: complexity_report.max_function_length,
                avg_cyclomatic: complexity_report.avg_cyclomatic,
                max_cyclomatic: complexity_report.max_cyclomatic,
                avg_cognitive: complexity_report.avg_cognitive,
                max_cognitive: complexity_report.max_cognitive,
                avg_nesting_depth: complexity_report.avg_nesting_depth,
                max_nesting_depth: complexity_report.max_nesting_depth,
                high_risk_files: complexity_report.high_risk_files,
            };

            (metrics, files, Some(complexity_section))
        } else {
            let fallback_metrics = BaselineMetrics {
                total_code_lines,
                total_files,
                ..Default::default()
            };
            (fallback_metrics, Vec::new(), None)
        };

        Self {
            baseline_version: BASELINE_VERSION,
            generated_at,
            commit: None,
            metrics,
            files,
            complexity,
            determinism: None,
        }
    }
}

impl Default for ComplexityBaseline {
    fn default() -> Self {
        Self::new()
    }
}

/// Complexity section mirroring analysis receipt structure for ratchet compatibility.
///
/// This provides the same field names as `ComplexityReport` so that JSON pointers
/// like `/complexity/avg_cyclomatic` work consistently across baselines and receipts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineComplexitySection {
    /// Total number of functions analyzed.
    pub total_functions: usize,
    /// Average function length in lines.
    pub avg_function_length: f64,
    /// Maximum function length found.
    pub max_function_length: usize,
    /// Average cyclomatic complexity across all files.
    pub avg_cyclomatic: f64,
    /// Maximum cyclomatic complexity found in any file.
    pub max_cyclomatic: usize,
    /// Average cognitive complexity across all files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_cognitive: Option<f64>,
    /// Maximum cognitive complexity found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_cognitive: Option<usize>,
    /// Average nesting depth across all files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_nesting_depth: Option<f64>,
    /// Maximum nesting depth found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_nesting_depth: Option<usize>,
    /// Number of high-risk files.
    pub high_risk_files: usize,
}

/// Aggregate baseline metrics for the entire codebase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineMetrics {
    /// Total lines of code across all files.
    pub total_code_lines: u64,
    /// Total number of source files.
    pub total_files: u64,
    /// Average cyclomatic complexity across all functions.
    pub avg_cyclomatic: f64,
    /// Maximum cyclomatic complexity found in any function.
    pub max_cyclomatic: u32,
    /// Average cognitive complexity across all functions.
    pub avg_cognitive: f64,
    /// Maximum cognitive complexity found in any function.
    pub max_cognitive: u32,
    /// Average nesting depth across all functions.
    pub avg_nesting_depth: f64,
    /// Maximum nesting depth found in any function.
    pub max_nesting_depth: u32,
    /// Total number of functions analyzed.
    pub function_count: u64,
    /// Average function length in lines.
    pub avg_function_length: f64,
}

impl Default for BaselineMetrics {
    fn default() -> Self {
        Self {
            total_code_lines: 0,
            total_files: 0,
            avg_cyclomatic: 0.0,
            max_cyclomatic: 0,
            avg_cognitive: 0.0,
            max_cognitive: 0,
            avg_nesting_depth: 0.0,
            max_nesting_depth: 0,
            function_count: 0,
            avg_function_length: 0.0,
        }
    }
}

/// Per-file baseline entry for granular complexity tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileBaselineEntry {
    /// Normalized file path (forward slashes).
    pub path: String,
    /// Lines of code in this file.
    pub code_lines: u64,
    /// Cyclomatic complexity for this file.
    pub cyclomatic: u32,
    /// Cognitive complexity for this file.
    pub cognitive: u32,
    /// Maximum nesting depth in this file.
    pub max_nesting: u32,
    /// Number of functions in this file.
    pub function_count: u32,
    /// BLAKE3 hash of file content for change detection.
    pub content_hash: Option<String>,
}

/// Build determinism baseline for reproducibility verification.
///
/// Tracks hashes of build artifacts and source inputs to detect
/// non-deterministic builds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterminismBaseline {
    /// Schema version for forward compatibility.
    pub baseline_version: u32,
    /// ISO 8601 timestamp when this baseline was generated.
    pub generated_at: String,
    /// Hash of the final build artifact.
    pub build_hash: String,
    /// Hash of all source files combined.
    pub source_hash: String,
    /// Hash of Cargo.lock if present (Rust projects).
    pub cargo_lock_hash: Option<String>,
}

/// Helper to convert milliseconds timestamp to RFC 3339 / ISO 8601 string.
fn chrono_timestamp_iso8601(ms: u128) -> String {
    // Convert milliseconds to seconds and remaining millis
    let total_secs = (ms / 1000) as i64;
    let millis = (ms % 1000) as u32;

    // Constants for date calculation
    const SECS_PER_MIN: i64 = 60;
    const SECS_PER_HOUR: i64 = 3600;
    const SECS_PER_DAY: i64 = 86400;

    // Days since Unix epoch (1970-01-01)
    let days = total_secs / SECS_PER_DAY;
    let day_secs = total_secs % SECS_PER_DAY;

    // Handle negative timestamps (before epoch)
    let (days, day_secs) = if day_secs < 0 {
        (days - 1, day_secs + SECS_PER_DAY)
    } else {
        (days, day_secs)
    };

    // Time of day
    let hour = day_secs / SECS_PER_HOUR;
    let min = (day_secs % SECS_PER_HOUR) / SECS_PER_MIN;
    let sec = day_secs % SECS_PER_MIN;

    // Convert days since epoch to year/month/day
    // Using algorithm from Howard Hinnant's date library
    let z = days + 719468; // shift to March 1, year 0
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32; // day of era [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // year of era
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of year
    let mp = (5 * doy + 2) / 153; // month pseudo
    let d = doy - (153 * mp + 2) / 5 + 1; // day
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // month
    let y = if m <= 2 { y + 1 } else { y }; // year

    // Format as RFC 3339: YYYY-MM-DDTHH:MM:SS.sssZ
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        y, m, d, hour, min, sec, millis
    )
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
    use chrono::{SecondsFormat, TimeZone, Utc};
    use proptest::prelude::*;

    // ── Schema version constant ───────────────────────────────────────
    #[test]
    fn analysis_schema_version_constant() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(ANALYSIS_SCHEMA_VERSION, 9);
        Ok(())
    }

    #[test]
    fn baseline_version_constant() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(BASELINE_VERSION, 1);
        Ok(())
    }

    // ── Default impls ─────────────────────────────────────────────────
    #[test]
    fn complexity_baseline_default() -> Result<(), Box<dyn std::error::Error>> {
        let b = ComplexityBaseline::default();
        assert_eq!(b.baseline_version, BASELINE_VERSION);
        assert!(b.generated_at.is_empty());
        assert!(b.commit.is_none());
        assert!(b.files.is_empty());
        assert!(b.complexity.is_none());
        assert!(b.determinism.is_none());
        Ok(())
    }

    #[test]
    fn complexity_baseline_new_equals_default() -> Result<(), Box<dyn std::error::Error>> {
        let a = ComplexityBaseline::new();
        let b = ComplexityBaseline::default();
        assert_eq!(a.baseline_version, b.baseline_version);
        assert_eq!(a.generated_at, b.generated_at);
        assert_eq!(a.files.len(), b.files.len());
        Ok(())
    }

    #[test]
    fn baseline_metrics_default_is_zeroed() -> Result<(), Box<dyn std::error::Error>> {
        let m = BaselineMetrics::default();
        assert_eq!(m.total_code_lines, 0);
        assert_eq!(m.total_files, 0);
        assert_eq!(m.avg_cyclomatic, 0.0);
        assert_eq!(m.max_cyclomatic, 0);
        assert_eq!(m.avg_cognitive, 0.0);
        assert_eq!(m.function_count, 0);
        Ok(())
    }

    // ── Enum serde roundtrips ─────────────────────────────────────────
    #[test]
    fn entropy_class_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        for variant in [
            EntropyClass::Low,
            EntropyClass::Normal,
            EntropyClass::Suspicious,
            EntropyClass::High,
        ] {
            let json = serde_json::to_string(&variant)?;
            let back: EntropyClass = serde_json::from_str(&json)?;
            assert_eq!(back, variant);
        }
        Ok(())
    }

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
    fn license_source_kind_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        for variant in [LicenseSourceKind::Metadata, LicenseSourceKind::Text] {
            let json = serde_json::to_string(&variant)?;
            let back: LicenseSourceKind = serde_json::from_str(&json)?;
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
    fn entropy_class_uses_snake_case() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            serde_json::to_string(&EntropyClass::Suspicious)?,
            "\"suspicious\""
        );
        Ok(())
    }

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

    #[test]
    fn complexity_baseline_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let b = ComplexityBaseline {
            baseline_version: BASELINE_VERSION,
            generated_at: "2025-01-01T00:00:00.000Z".into(),
            commit: Some("abc123".into()),
            metrics: BaselineMetrics::default(),
            files: vec![FileBaselineEntry {
                path: "src/lib.rs".into(),
                code_lines: 100,
                cyclomatic: 5,
                cognitive: 3,
                max_nesting: 2,
                function_count: 10,
                content_hash: Some("deadbeef".into()),
            }],
            complexity: None,
            determinism: None,
        };
        let json = serde_json::to_string(&b)?;
        let back: ComplexityBaseline = serde_json::from_str(&json)?;
        assert_eq!(back.baseline_version, BASELINE_VERSION);
        assert_eq!(back.commit.as_deref(), Some("abc123"));
        assert_eq!(back.files.len(), 1);
        assert_eq!(back.files[0].path, "src/lib.rs");
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

    // ── chrono_timestamp_iso8601 ──────────────────────────────────────
    #[test]
    fn timestamp_epoch() -> Result<(), Box<dyn std::error::Error>> {
        let result = chrono_timestamp_iso8601(0);
        assert_eq!(result, "1970-01-01T00:00:00.000Z");
        Ok(())
    }

    #[test]
    fn timestamp_with_millis() -> Result<(), Box<dyn std::error::Error>> {
        // 2025-01-01T00:00:00.500Z = 1735689600500 ms
        let result = chrono_timestamp_iso8601(1735689600500);
        assert!(result.ends_with(".500Z"));
        assert!(result.starts_with("2025-01-01"));
        Ok(())
    }

    proptest! {
        #[test]
        fn chrono_timestamp_matches_chrono(ms in 0u128..253_402_300_799_000u128) {
            let chrono_dt = Utc
                .timestamp_millis_opt(ms as i64)
                .single()
                .expect("timestamp within supported range");
            let expected = chrono_dt.to_rfc3339_opts(SecondsFormat::Millis, true);
            prop_assert_eq!(chrono_timestamp_iso8601(ms), expected);
        }

        #[test]
        fn chrono_timestamp_is_rfc3339(ms in 0u128..253_402_300_799_000u128) {
            let rendered = chrono_timestamp_iso8601(ms);
            prop_assert!(chrono::DateTime::parse_from_rfc3339(&rendered).is_ok());
        }
    }
}
