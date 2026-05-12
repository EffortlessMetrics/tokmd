//! Cockpit evidence gate receipt DTOs.
//!
//! These types describe the gate evidence embedded in cockpit receipts. They
//! stay serde-stable because review packets, hosted comments, and downstream
//! evidence consumers read these fields directly.

use serde::{Deserialize, Serialize};

/// Evidence section containing hard gates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    /// Aggregate status of all gates.
    pub overall_status: GateStatus,
    /// Mutation testing gate (always present).
    pub mutation: MutationGate,
    /// Diff coverage gate (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_coverage: Option<DiffCoverageGate>,
    /// Contract diff gate (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contracts: Option<ContractDiffGate>,
    /// Supply chain gate (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supply_chain: Option<SupplyChainGate>,
    /// Determinism gate (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub determinism: Option<DeterminismGate>,
    /// Complexity gate (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complexity: Option<ComplexityGate>,
}

/// Status of a gate check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GateStatus {
    Pass,
    Warn,
    Fail,
    Skipped,
    Pending,
}

/// Source of evidence/gate results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSource {
    CiArtifact,
    Cached,
    RanLocal,
}

/// Commit match quality for evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommitMatch {
    Exact,
    Partial,
    Stale,
    Unknown,
}

/// Common metadata for all gates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateMeta {
    pub status: GateStatus,
    pub source: EvidenceSource,
    pub commit_match: CommitMatch,
    pub scope: ScopeCoverage,
    /// SHA this evidence was generated for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_commit: Option<String>,
    /// Timestamp when evidence was generated (ms since epoch).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_generated_at_ms: Option<u64>,
}

/// Scope coverage for a gate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeCoverage {
    /// Files in scope for the gate.
    pub relevant: Vec<String>,
    /// Files actually tested.
    pub tested: Vec<String>,
    /// Coverage ratio (tested/relevant, 0.0-1.0).
    pub ratio: f64,
    /// Lines in scope (optional, for line-level gates).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines_relevant: Option<usize>,
    /// Lines actually tested (optional, for line-level gates).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines_tested: Option<usize>,
}

/// Mutation testing gate results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationGate {
    #[serde(flatten)]
    pub meta: GateMeta,
    pub survivors: Vec<MutationSurvivor>,
    pub killed: usize,
    pub timeout: usize,
    pub unviable: usize,
}

/// A mutation that survived testing (escaped detection).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationSurvivor {
    pub file: String,
    pub line: usize,
    pub mutation: String,
}

/// Diff coverage gate results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffCoverageGate {
    #[serde(flatten)]
    pub meta: GateMeta,
    pub lines_added: usize,
    pub lines_covered: usize,
    pub coverage_pct: f64,
    pub uncovered_hunks: Vec<UncoveredHunk>,
}

/// Uncovered hunk in diff coverage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncoveredHunk {
    pub file: String,
    pub start_line: usize,
    pub end_line: usize,
}

/// Contract diff gate results (compound gate).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractDiffGate {
    #[serde(flatten)]
    pub meta: GateMeta,
    /// Semver sub-gate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semver: Option<SemverSubGate>,
    /// CLI sub-gate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cli: Option<CliSubGate>,
    /// Schema sub-gate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<SchemaSubGate>,
    /// Count of failed sub-gates.
    pub failures: usize,
}

/// Semver sub-gate for contract diff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemverSubGate {
    pub status: GateStatus,
    pub breaking_changes: Vec<BreakingChange>,
}

/// Breaking change detected by semver check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChange {
    pub kind: String,
    pub path: String,
    pub message: String,
}

/// CLI sub-gate for contract diff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliSubGate {
    pub status: GateStatus,
    pub diff_summary: Option<String>,
}

/// Schema sub-gate for contract diff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaSubGate {
    pub status: GateStatus,
    pub diff_summary: Option<String>,
}

/// Supply chain gate results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyChainGate {
    #[serde(flatten)]
    pub meta: GateMeta,
    pub vulnerabilities: Vec<Vulnerability>,
    pub denied: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advisory_db_version: Option<String>,
}

/// Vulnerability from cargo-audit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub package: String,
    pub severity: String,
    pub title: String,
}

/// Determinism gate results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterminismGate {
    #[serde(flatten)]
    pub meta: GateMeta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_hash: Option<String>,
    pub algo: String,
    pub differences: Vec<String>,
}

/// Complexity gate results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityGate {
    #[serde(flatten)]
    pub meta: GateMeta,
    /// Number of files analyzed for complexity.
    pub files_analyzed: usize,
    /// Files with high complexity (CC > threshold).
    pub high_complexity_files: Vec<HighComplexityFile>,
    /// Average cyclomatic complexity across all analyzed files.
    pub avg_cyclomatic: f64,
    /// Maximum cyclomatic complexity found.
    pub max_cyclomatic: u32,
    /// Whether the threshold was exceeded.
    pub threshold_exceeded: bool,
}

/// A file with high cyclomatic complexity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighComplexityFile {
    /// Path to the file.
    pub path: String,
    /// Cyclomatic complexity score.
    pub cyclomatic: u32,
    /// Number of functions in the file.
    pub function_count: usize,
    /// Maximum function length in lines.
    pub max_function_length: usize,
}
