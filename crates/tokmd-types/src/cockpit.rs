//! Cockpit receipt types for PR metrics and evidence gates.
//!
//! These types define the data model for the `tokmd cockpit` command output.
//! They are extracted here (Tier 0) so that lower-tier crates like `tokmd-cockpit`
//! and `tokmd-core` can reference them without depending on the CLI binary.

use serde::{Deserialize, Serialize};

mod evidence;

pub use evidence::{
    BreakingChange, CliSubGate, CommitMatch, ComplexityGate, ContractDiffGate, DeterminismGate,
    DiffCoverageGate, Evidence, EvidenceSource, GateMeta, GateStatus, HighComplexityFile,
    MutationGate, MutationSurvivor, SchemaSubGate, ScopeCoverage, SemverSubGate, SupplyChainGate,
    UncoveredHunk, Vulnerability,
};

/// Cockpit receipt schema version.
pub const COCKPIT_SCHEMA_VERSION: u32 = 3;

// =============================================================================
// Top-level receipt
// =============================================================================

/// Cockpit receipt containing all PR metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CockpitReceipt {
    pub schema_version: u32,
    pub mode: String,
    pub generated_at_ms: u64,
    pub base_ref: String,
    pub head_ref: String,
    pub change_surface: ChangeSurface,
    pub composition: Composition,
    pub code_health: CodeHealth,
    pub risk: Risk,
    pub contracts: Contracts,
    pub evidence: Evidence,
    pub review_plan: Vec<ReviewItem>,
    /// Trend comparison with baseline (if --baseline was provided).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trend: Option<TrendComparison>,
}

// =============================================================================
// Metric types
// =============================================================================

/// Change surface metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSurface {
    pub commits: usize,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub net_lines: i64,
    /// Churn velocity: average lines changed per commit.
    pub churn_velocity: f64,
    /// Change concentration: what % of changes are in top 20% of files.
    pub change_concentration: f64,
}

/// File composition breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Composition {
    pub code_pct: f64,
    pub test_pct: f64,
    pub docs_pct: f64,
    pub config_pct: f64,
    /// Test-to-code ratio (tests / code files).
    pub test_ratio: f64,
}

/// Code health indicators for DevEx.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeHealth {
    /// Overall health score (0-100).
    pub score: u32,
    /// Health grade (A-F).
    pub grade: String,
    /// Number of large files (>500 lines) being changed.
    pub large_files_touched: usize,
    /// Average file size in changed files.
    pub avg_file_size: usize,
    /// Complexity indicator based on file patterns.
    pub complexity_indicator: ComplexityIndicator,
    /// Files with potential issues.
    pub warnings: Vec<HealthWarning>,
}

/// Complexity indicator levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComplexityIndicator {
    Low,
    Medium,
    High,
    Critical,
}

/// Health warning for specific files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthWarning {
    pub path: String,
    pub warning_type: WarningType,
    pub message: String,
}

/// Types of health warnings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarningType {
    LargeFile,
    HighChurn,
    LowTestCoverage,
    ComplexChange,
    BusFactor,
}

/// Risk indicators.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub hotspots_touched: Vec<String>,
    pub bus_factor_warnings: Vec<String>,
    /// Overall risk level for this PR.
    pub level: RiskLevel,
    /// Risk score (0-100).
    pub score: u32,
}

/// Risk level classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "low"),
            RiskLevel::Medium => write!(f, "medium"),
            RiskLevel::High => write!(f, "high"),
            RiskLevel::Critical => write!(f, "critical"),
        }
    }
}

/// Contract change indicators.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contracts {
    pub api_changed: bool,
    pub cli_changed: bool,
    pub schema_changed: bool,
    /// Number of breaking change indicators.
    pub breaking_indicators: usize,
}

/// Review plan item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewItem {
    pub path: String,
    pub reason: String,
    pub priority: u32,
    /// Estimated review complexity (1-5).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complexity: Option<u8>,
    /// Lines changed in this file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines_changed: Option<usize>,
}

// =============================================================================
// Trend comparison types
// =============================================================================

/// Trend comparison between current state and baseline.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrendComparison {
    /// Whether a baseline was successfully loaded.
    pub baseline_available: bool,
    /// Path to the baseline file used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_path: Option<String>,
    /// Timestamp of baseline generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_generated_at_ms: Option<u64>,
    /// Health score trend.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<TrendMetric>,
    /// Risk score trend.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk: Option<TrendMetric>,
    /// Complexity trend indicator.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complexity: Option<TrendIndicator>,
}

/// A trend metric with current, previous, delta values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendMetric {
    /// Current value.
    pub current: f64,
    /// Previous (baseline) value.
    pub previous: f64,
    /// Absolute delta (current - previous).
    pub delta: f64,
    /// Percentage change.
    pub delta_pct: f64,
    /// Direction of change.
    pub direction: TrendDirection,
}

/// Complexity trend indicator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendIndicator {
    /// Overall trend direction.
    pub direction: TrendDirection,
    /// Human-readable summary.
    pub summary: String,
    /// Number of files that got more complex.
    pub files_increased: usize,
    /// Number of files that got less complex.
    pub files_decreased: usize,
    /// Average cyclomatic delta.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_cyclomatic_delta: Option<f64>,
    /// Average cognitive delta.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_cognitive_delta: Option<f64>,
}

/// Direction of a trend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cockpit_receipt_serde_roundtrip() {
        let receipt = CockpitReceipt {
            schema_version: COCKPIT_SCHEMA_VERSION,
            mode: "cockpit".to_string(),
            generated_at_ms: 1000,
            base_ref: "main".to_string(),
            head_ref: "HEAD".to_string(),
            change_surface: ChangeSurface {
                commits: 1,
                files_changed: 2,
                insertions: 10,
                deletions: 5,
                net_lines: 5,
                churn_velocity: 15.0,
                change_concentration: 0.8,
            },
            composition: Composition {
                code_pct: 70.0,
                test_pct: 20.0,
                docs_pct: 5.0,
                config_pct: 5.0,
                test_ratio: 0.29,
            },
            code_health: CodeHealth {
                score: 85,
                grade: "B".to_string(),
                large_files_touched: 0,
                avg_file_size: 100,
                complexity_indicator: ComplexityIndicator::Low,
                warnings: vec![],
            },
            risk: Risk {
                hotspots_touched: vec![],
                bus_factor_warnings: vec![],
                level: RiskLevel::Low,
                score: 10,
            },
            contracts: Contracts {
                api_changed: false,
                cli_changed: false,
                schema_changed: false,
                breaking_indicators: 0,
            },
            evidence: Evidence {
                overall_status: GateStatus::Pass,
                mutation: MutationGate {
                    meta: GateMeta {
                        status: GateStatus::Pass,
                        source: EvidenceSource::RanLocal,
                        commit_match: CommitMatch::Exact,
                        scope: ScopeCoverage {
                            relevant: vec![],
                            tested: vec![],
                            ratio: 1.0,
                            lines_relevant: None,
                            lines_tested: None,
                        },
                        evidence_commit: None,
                        evidence_generated_at_ms: None,
                    },
                    survivors: vec![],
                    killed: 0,
                    timeout: 0,
                    unviable: 0,
                },
                diff_coverage: None,
                contracts: None,
                supply_chain: None,
                determinism: None,
                complexity: None,
            },
            review_plan: vec![],
            trend: None,
        };

        let json = serde_json::to_string(&receipt).expect("serialize");
        let back: CockpitReceipt = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.schema_version, COCKPIT_SCHEMA_VERSION);
        assert_eq!(back.mode, "cockpit");
    }

    #[test]
    fn gate_status_serde() {
        let json = serde_json::to_string(&GateStatus::Pass).unwrap();
        assert_eq!(json, "\"pass\"");
        let back: GateStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back, GateStatus::Pass);
    }

    #[test]
    fn trend_direction_serde() {
        let json = serde_json::to_string(&TrendDirection::Improving).unwrap();
        assert_eq!(json, "\"improving\"");
        let back: TrendDirection = serde_json::from_str(&json).unwrap();
        assert_eq!(back, TrendDirection::Improving);
    }

    #[test]
    fn risk_level_display() {
        assert_eq!(RiskLevel::Low.to_string(), "low");
        assert_eq!(RiskLevel::Critical.to_string(), "critical");
    }
}
