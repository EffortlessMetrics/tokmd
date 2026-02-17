//! Handler for the `tokmd cockpit` command.
//!
//! Generates PR cockpit metrics for code review automation.

use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use tokmd_config as cli;
use tokmd_envelope::{SensorReport, ToolMeta, Verdict};

/// Cockpit receipt schema version.
const SCHEMA_VERSION: u32 = 3;

/// Handle the cockpit command.
pub(crate) fn handle(args: cli::CockpitArgs, _global: &cli::GlobalArgs) -> Result<()> {
    #[cfg(not(feature = "git"))]
    {
        let _ = &args; // Silence unused warning
        bail!("The cockpit command requires the 'git' feature. Rebuild with --features git");
    }

    #[cfg(feature = "git")]
    {
        if !tokmd_git::git_available() {
            bail!("git is not available on PATH");
        }

        let cwd = std::env::current_dir().context("Failed to resolve current directory")?;
        let repo_root = tokmd_git::repo_root(&cwd)
            .ok_or_else(|| anyhow::anyhow!("not inside a git repository"))?;

        let range_mode = match args.diff_range {
            cli::DiffRangeMode::TwoDot => tokmd_git::GitRangeMode::TwoDot,
            cli::DiffRangeMode::ThreeDot => tokmd_git::GitRangeMode::ThreeDot,
        };

        let resolved_base =
            tokmd_git::resolve_base_ref(&repo_root, &args.base).ok_or_else(|| {
                anyhow::anyhow!(
                    "base ref '{}' not found and no fallback resolved. \
                 Use --base to specify a valid ref, or set TOKMD_GIT_BASE_REF",
                    args.base
                )
            })?;

        let mut receipt = compute_cockpit(
            &repo_root,
            &resolved_base,
            &args.head,
            range_mode,
            args.baseline.as_deref(),
        )?;

        // Load baseline and compute trend if provided
        if let Some(baseline_path) = &args.baseline {
            receipt.trend = Some(load_and_compute_trend(baseline_path, &receipt)?);
        }

        // In sensor mode, write envelope to artifacts_dir
        if args.sensor_mode {
            let artifacts_dir = args
                .artifacts_dir
                .as_ref()
                .cloned()
                .unwrap_or_else(|| PathBuf::from("artifacts/tokmd"));
            write_sensor_artifacts(&artifacts_dir, &receipt, &resolved_base, &args.head)?;

            // In sensor mode, always print JSON to stdout for piping
            let output = render_json(&receipt)?;
            print!("{}", output);
            return Ok(());
        }

        // Standard (non-sensor) mode
        let output = match args.format {
            cli::CockpitFormat::Json => render_json(&receipt)?,
            cli::CockpitFormat::Md => render_markdown(&receipt),
            cli::CockpitFormat::Sections => render_sections(&receipt),
        };

        if let Some(artifacts_dir) = &args.artifacts_dir {
            write_artifacts(artifacts_dir, &receipt)?;
        }

        if let Some(output_path) = &args.output {
            let mut file = std::fs::File::create(output_path).with_context(|| {
                format!("Failed to create output file: {}", output_path.display())
            })?;
            file.write_all(output.as_bytes())?;
        } else {
            print!("{}", output);
        }

        Ok(())
    }
}

/// Load baseline receipt and compute trend comparison.
fn load_and_compute_trend(
    baseline_path: &std::path::Path,
    current: &CockpitReceipt,
) -> Result<TrendComparison> {
    // Try to load baseline
    let content = match std::fs::read_to_string(baseline_path) {
        Ok(c) => c,
        Err(_) => {
            return Ok(TrendComparison {
                baseline_available: false,
                baseline_path: Some(baseline_path.to_string_lossy().to_string()),
                ..Default::default()
            });
        }
    };

    let baseline: CockpitReceipt = match serde_json::from_str(&content) {
        Ok(b) => b,
        Err(_) => {
            return Ok(TrendComparison {
                baseline_available: false,
                baseline_path: Some(baseline_path.to_string_lossy().to_string()),
                ..Default::default()
            });
        }
    };

    // Compute health trend
    let health = compute_metric_trend(
        current.code_health.score as f64,
        baseline.code_health.score as f64,
        true, // Higher is better for health
    );

    // Compute risk trend
    let risk = compute_metric_trend(
        current.risk.score as f64,
        baseline.risk.score as f64,
        false, // Lower is better for risk
    );

    // Compute complexity trend indicator
    let complexity = compute_complexity_trend(current, &baseline);

    Ok(TrendComparison {
        baseline_available: true,
        baseline_path: Some(baseline_path.to_string_lossy().to_string()),
        baseline_generated_at_ms: Some(baseline.generated_at_ms),
        health: Some(health),
        risk: Some(risk),
        complexity: Some(complexity),
    })
}

/// Compute trend metric with direction.
fn compute_metric_trend(current: f64, previous: f64, higher_is_better: bool) -> TrendMetric {
    let delta = current - previous;
    let delta_pct = if previous != 0.0 {
        (delta / previous) * 100.0
    } else if current != 0.0 {
        100.0
    } else {
        0.0
    };

    // Determine direction based on whether improvement means higher or lower
    let direction = if delta.abs() < 1.0 {
        TrendDirection::Stable
    } else if higher_is_better {
        if delta > 0.0 {
            TrendDirection::Improving
        } else {
            TrendDirection::Degrading
        }
    } else {
        // Lower is better (e.g., risk)
        if delta < 0.0 {
            TrendDirection::Improving
        } else {
            TrendDirection::Degrading
        }
    };

    TrendMetric {
        current,
        previous,
        delta,
        delta_pct: round_pct(delta_pct),
        direction,
    }
}

/// Compute complexity trend indicator.
fn compute_complexity_trend(current: &CockpitReceipt, baseline: &CockpitReceipt) -> TrendIndicator {
    // Compare complexity gate results if available
    let current_complexity = current
        .evidence
        .complexity
        .as_ref()
        .map(|c| c.avg_cyclomatic)
        .unwrap_or(0.0);
    let baseline_complexity = baseline
        .evidence
        .complexity
        .as_ref()
        .map(|c| c.avg_cyclomatic)
        .unwrap_or(0.0);

    let delta = current_complexity - baseline_complexity;

    let direction = if delta.abs() < 0.5 {
        TrendDirection::Stable
    } else if delta < 0.0 {
        TrendDirection::Improving
    } else {
        TrendDirection::Degrading
    };

    let summary = match direction {
        TrendDirection::Improving => "Complexity decreased".to_string(),
        TrendDirection::Stable => "Complexity stable".to_string(),
        TrendDirection::Degrading => "Complexity increased".to_string(),
    };

    TrendIndicator {
        direction,
        summary,
        files_increased: 0, // Would require per-file comparison
        files_decreased: 0,
        avg_cyclomatic_delta: Some(round_pct(delta)),
        avg_cognitive_delta: None,
    }
}

/// Cockpit receipt containing all PR metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CockpitReceipt {
    pub schema_version: u32,
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
    Pass,    // Gate passed
    Warn,    // Gate has warnings but not critical
    Fail,    // Gate failed
    Skipped, // No relevant files changed
    Pending, // Results not available and couldn't run
}

/// Source of evidence/gate results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSource {
    CiArtifact, // Downloaded from CI workflow artifact
    Cached,     // Found in local cache (.tokmd/cache/)
    RanLocal,   // Executed locally during this run
}

/// Commit match quality for evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommitMatch {
    Exact,   // Evidence commit SHA matches HEAD exactly
    Partial, // Evidence covers merge base or subset
    Stale,   // Evidence from different commit
    Unknown, // Could not determine
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
/// Analyzes cyclomatic complexity of changed files.
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

/// A mutation that survived testing (escaped detection).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationSurvivor {
    pub file: String,
    pub line: usize,
    pub mutation: String,
}

/// Change surface metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSurface {
    pub commits: usize,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub net_lines: i64,
    /// Churn velocity: average lines changed per commit
    pub churn_velocity: f64,
    /// Change concentration: what % of changes are in top 20% of files
    pub change_concentration: f64,
}

/// File composition breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Composition {
    pub code_pct: f64,
    pub test_pct: f64,
    pub docs_pct: f64,
    pub config_pct: f64,
    /// Test-to-code ratio (tests / code files)
    pub test_ratio: f64,
}

/// Code health indicators for DevEx.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeHealth {
    /// Overall health score (0-100)
    pub score: u32,
    /// Health grade (A-F)
    pub grade: String,
    /// Number of large files (>500 lines) being changed
    pub large_files_touched: usize,
    /// Average file size in changed files
    pub avg_file_size: usize,
    /// Complexity indicator based on file patterns
    pub complexity_indicator: ComplexityIndicator,
    /// Files with potential issues
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
    /// Overall risk level for this PR
    pub level: RiskLevel,
    /// Risk score (0-100)
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
    /// Number of breaking change indicators
    pub breaking_indicators: usize,
}

/// Review plan item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewItem {
    pub path: String,
    pub reason: String,
    pub priority: u32,
    /// Estimated review complexity (1-5)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complexity: Option<u8>,
    /// Lines changed in this file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines_changed: Option<usize>,
}

// =============================================================================
// Trend Comparison Types
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
    /// Improving (lower risk, lower complexity, higher health).
    Improving,
    /// Stable (within tolerance).
    Stable,
    /// Degrading (higher risk, higher complexity, lower health).
    Degrading,
}

#[cfg(feature = "git")]
pub(crate) fn compute_cockpit(
    repo_root: &PathBuf,
    base: &str,
    head: &str,
    range_mode: tokmd_git::GitRangeMode,
    baseline_path: Option<&Path>,
) -> Result<CockpitReceipt> {
    let generated_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    // Get changed files with their stats
    let file_stats = get_file_stats(repo_root, base, head, range_mode)?;
    let changed_files: Vec<String> = file_stats.iter().map(|f| f.path.clone()).collect();

    // Get change surface from git
    let change_surface = compute_change_surface(repo_root, base, head, &file_stats, range_mode)?;

    // Compute composition with test ratio
    let composition = compute_composition(&changed_files);

    // Detect contract changes
    let contracts = detect_contracts(&changed_files);

    // Compute code health
    let code_health = compute_code_health(&file_stats, &contracts);

    // Compute risk based on various factors
    let risk = compute_risk(&file_stats, &contracts, &code_health);

    // Compute all gate evidence
    let evidence = compute_evidence(
        repo_root,
        base,
        head,
        &changed_files,
        &contracts,
        range_mode,
        baseline_path,
    )?;

    // Generate review plan with complexity scores
    let review_plan = generate_review_plan(&file_stats, &contracts);

    Ok(CockpitReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms,
        base_ref: base.to_string(),
        head_ref: head.to_string(),
        change_surface,
        composition,
        code_health,
        risk,
        contracts,
        evidence,
        review_plan,
        trend: None, // Populated by handle() if --baseline is provided
    })
}

/// Compute evidence section with all gates.
#[cfg(feature = "git")]
fn compute_evidence(
    repo_root: &PathBuf,
    base: &str,
    head: &str,
    changed_files: &[String],
    contracts_info: &Contracts,
    range_mode: tokmd_git::GitRangeMode,
    baseline_path: Option<&Path>,
) -> Result<Evidence> {
    let mutation = compute_mutation_gate(repo_root, base, head, changed_files, range_mode)?;
    let diff_coverage = compute_diff_coverage_gate(repo_root, base, head, range_mode)?;
    let contracts = compute_contract_gate(repo_root, base, head, changed_files, contracts_info)?;
    let supply_chain = compute_supply_chain_gate(repo_root, changed_files)?;
    let determinism = compute_determinism_gate(repo_root, baseline_path)?;
    let complexity = compute_complexity_gate(repo_root, changed_files)?;

    // Compute overall status: any Fail → Fail, all Pass → Pass, otherwise Pending/Skipped
    let overall_status = compute_overall_status(
        &mutation,
        &diff_coverage,
        &contracts,
        &supply_chain,
        &determinism,
        &complexity,
    );

    Ok(Evidence {
        overall_status,
        mutation,
        diff_coverage,
        contracts,
        supply_chain,
        determinism,
        complexity,
    })
}

/// Compute overall status from all gates.
fn compute_overall_status(
    mutation: &MutationGate,
    diff_coverage: &Option<DiffCoverageGate>,
    contracts: &Option<ContractDiffGate>,
    supply_chain: &Option<SupplyChainGate>,
    determinism: &Option<DeterminismGate>,
    complexity: &Option<ComplexityGate>,
) -> GateStatus {
    let statuses: Vec<GateStatus> = [
        Some(mutation.meta.status),
        diff_coverage.as_ref().map(|g| g.meta.status),
        contracts.as_ref().map(|g| g.meta.status),
        supply_chain.as_ref().map(|g| g.meta.status),
        determinism.as_ref().map(|g| g.meta.status),
        complexity.as_ref().map(|g| g.meta.status),
    ]
    .into_iter()
    .flatten()
    .collect();

    if statuses.is_empty() {
        return GateStatus::Skipped;
    }

    // Any Fail → overall Fail
    if statuses.contains(&GateStatus::Fail) {
        return GateStatus::Fail;
    }

    // All Pass → overall Pass
    if statuses.iter().all(|s| *s == GateStatus::Pass) {
        return GateStatus::Pass;
    }

    // Any Pending (and no Fail) → overall Pending
    if statuses.contains(&GateStatus::Pending) {
        return GateStatus::Pending;
    }

    // Otherwise (mix of Pass and Skipped) → Pass
    GateStatus::Pass
}

/// Compute diff coverage gate.
/// Looks for coverage artifacts (lcov.info, coverage.json, cobertura.xml) and parses them.
#[cfg(feature = "git")]
fn compute_diff_coverage_gate(
    repo_root: &Path,
    base: &str,
    head: &str,
    range_mode: tokmd_git::GitRangeMode,
) -> Result<Option<DiffCoverageGate>> {
    // 1. Get added lines from git
    let added_lines = match tokmd_git::get_added_lines(repo_root, base, head, range_mode) {
        Ok(lines) => lines,
        Err(_) => return Ok(None),
    };

    if added_lines.is_empty() {
        return Ok(None);
    }

    // 2. Search for coverage artifacts in common locations
    let search_paths = [
        "coverage/lcov.info",
        "target/coverage/lcov.info",
        "lcov.info",
        "coverage/cobertura.xml",
        "target/coverage/cobertura.xml",
        "cobertura.xml",
        "coverage/coverage.json",
        "target/coverage/coverage.json",
        "coverage.json",
    ];

    let mut lcov_path: Option<PathBuf> = None;
    for candidate in &search_paths {
        let path = repo_root.join(candidate);
        if path.exists() {
            lcov_path = Some(path);
            break;
        }
    }

    let lcov_path = match lcov_path {
        Some(p) => p,
        None => return Ok(None), // No coverage artifact found
    };

    // Only parse lcov.info format for now (most common in Rust via cargo-llvm-cov)
    let path_str = lcov_path.to_string_lossy();
    if !path_str.ends_with("lcov.info") {
        // We found a coverage file but can't parse non-lcov yet
        return Ok(None);
    }

    let content = match std::fs::read_to_string(&lcov_path) {
        Ok(c) => c,
        Err(_) => return Ok(None),
    };

    // 3. Parse LCOV into a lookup map: file -> line -> hit_count
    let mut lcov_data: BTreeMap<String, BTreeMap<usize, usize>> = BTreeMap::new();
    let mut current_file: Option<String> = None;

    for line in content.lines() {
        if let Some(sf) = line.strip_prefix("SF:") {
            // Normalize path to repo-relative
            let path = sf.replace('\\', "/");
            // If it's absolute, try to make it relative to repo root
            let normalized = if let Ok(abs) = Path::new(&path).canonicalize() {
                if let Ok(rel) = abs.strip_prefix(repo_root.canonicalize().unwrap_or_default()) {
                    rel.to_string_lossy().replace('\\', "/")
                } else {
                    path
                }
            } else {
                path
            };
            current_file = Some(normalized);
            lcov_data.entry(current_file.clone().unwrap()).or_default();
        } else if let Some(da) = line.strip_prefix("DA:") {
            if let Some(ref file) = current_file {
                let parts: Vec<&str> = da.splitn(2, ',').collect();
                if parts.len() == 2
                    && let (Ok(line_no), Ok(count)) =
                        (parts[0].parse::<usize>(), parts[1].parse::<usize>())
                {
                    lcov_data.get_mut(file).unwrap().insert(line_no, count);
                }
            }
        } else if line == "end_of_record" {
            current_file = None;
        }
    }

    // 4. Intersect added lines with LCOV hits
    let mut total_added = 0usize;
    let mut total_covered = 0usize;
    let mut uncovered_hunks: Vec<UncoveredHunk> = Vec::new();
    let mut tested_files: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for (file_path, lines) in added_lines {
        let file_path_str = file_path.to_string_lossy().replace('\\', "/");
        total_added += lines.len();

        let mut uncovered_in_file = Vec::new();

        if let Some(file_lcov) = lcov_data.get(&file_path_str) {
            tested_files.insert(file_path_str.clone());
            for line in lines {
                match file_lcov.get(&line) {
                    Some(&count) if count > 0 => {
                        total_covered += 1;
                    }
                    _ => {
                        uncovered_in_file.push(line);
                    }
                }
            }
        } else {
            // File not in LCOV - treat all added lines as uncovered
            uncovered_in_file.extend(lines);
        }

        flush_uncovered_hunks(&file_path_str, &uncovered_in_file, &mut uncovered_hunks);
    }

    if total_added == 0 {
        return Ok(None);
    }

    let coverage_pct = round_pct(total_covered as f64 / total_added as f64 * 100.0);
    let status = if coverage_pct >= 80.0 {
        GateStatus::Pass
    } else if coverage_pct >= 50.0 {
        GateStatus::Pending // Warn-level
    } else {
        GateStatus::Fail
    };

    // Limit uncovered hunks to avoid huge output
    uncovered_hunks.truncate(20);

    Ok(Some(DiffCoverageGate {
        meta: GateMeta {
            status,
            source: EvidenceSource::CiArtifact,
            commit_match: CommitMatch::Unknown,
            scope: ScopeCoverage {
                relevant: lcov_data.keys().cloned().collect(),
                tested: tested_files.into_iter().collect(),
                ratio: coverage_pct / 100.0,
                lines_relevant: Some(total_added),
                lines_tested: Some(total_covered),
            },
            evidence_commit: None,
            evidence_generated_at_ms: None,
        },
        lines_added: total_added,
        lines_covered: total_covered,
        coverage_pct,
        uncovered_hunks,
    }))
}

/// Flush consecutive uncovered lines into hunk ranges.
fn flush_uncovered_hunks(file: &str, uncovered: &[usize], hunks: &mut Vec<UncoveredHunk>) {
    if uncovered.is_empty() || file.is_empty() {
        return;
    }
    let mut sorted = uncovered.to_vec();
    sorted.sort_unstable();
    let mut start = sorted[0];
    let mut end = sorted[0];
    for &line in &sorted[1..] {
        if line == end + 1 {
            end = line;
        } else {
            hunks.push(UncoveredHunk {
                file: file.to_string(),
                start_line: start,
                end_line: end,
            });
            start = line;
            end = line;
        }
    }
    hunks.push(UncoveredHunk {
        file: file.to_string(),
        start_line: start,
        end_line: end,
    });
}

/// Compute contract diff gate (semver, CLI, schema).
#[cfg(feature = "git")]
fn compute_contract_gate(
    repo_root: &Path,
    base: &str,
    head: &str,
    changed_files: &[String],
    contracts_info: &Contracts,
) -> Result<Option<ContractDiffGate>> {
    // Only compute if any contract-relevant files changed
    if !contracts_info.api_changed && !contracts_info.cli_changed && !contracts_info.schema_changed
    {
        return Ok(None);
    }

    let mut failures = 0;
    let mut semver = None;
    let mut cli = None;
    let mut schema = None;

    // Check for semver changes (API files)
    if contracts_info.api_changed {
        semver = Some(run_semver_check(repo_root));
    }

    // Check for CLI changes
    if contracts_info.cli_changed {
        // Gather CLI-related files that changed
        let cli_files: Vec<&str> = changed_files
            .iter()
            .filter(|f| {
                f.contains("crates/tokmd/src/commands/") || f.contains("crates/tokmd-config/")
            })
            .map(|s| s.as_str())
            .collect();

        let diff_summary = if cli_files.is_empty() {
            None
        } else {
            let command_files = cli_files
                .iter()
                .filter(|f| f.contains("crates/tokmd/src/commands/"))
                .count();
            let config_files = cli_files
                .iter()
                .filter(|f| f.contains("crates/tokmd-config/"))
                .count();

            let mut parts = Vec::new();
            if command_files > 0 {
                parts.push(format!(
                    "{} command file{}",
                    command_files,
                    if command_files == 1 { "" } else { "s" }
                ));
            }
            if config_files > 0 {
                parts.push(format!(
                    "{} config file{}",
                    config_files,
                    if config_files == 1 { "" } else { "s" }
                ));
            }
            Some(parts.join(", "))
        };

        cli = Some(CliSubGate {
            status: GateStatus::Pass,
            diff_summary,
        });
    }

    // Check for schema changes
    if contracts_info.schema_changed {
        schema = Some(run_schema_diff(repo_root, base, head));
    }

    // Count failures from sub-gates
    if let Some(ref sg) = semver
        && sg.status == GateStatus::Fail
    {
        failures += 1;
    }
    if let Some(ref cg) = cli
        && cg.status == GateStatus::Fail
    {
        failures += 1;
    }
    if let Some(ref scg) = schema
        && scg.status == GateStatus::Fail
    {
        failures += 1;
    }

    // Determine overall status
    let status = if failures > 0 {
        GateStatus::Fail
    } else {
        // Check if any are pending
        let any_pending = [
            semver.as_ref().map(|g| g.status),
            cli.as_ref().map(|g| g.status),
            schema.as_ref().map(|g| g.status),
        ]
        .into_iter()
        .flatten()
        .any(|s| s == GateStatus::Pending);

        if any_pending {
            GateStatus::Pending
        } else {
            GateStatus::Pass
        }
    };

    // Collect relevant files for scope
    let relevant: Vec<String> = changed_files
        .iter()
        .filter(|f| {
            f.ends_with("/src/lib.rs")
                || f.ends_with("/mod.rs")
                || f.contains("crates/tokmd/src/commands/")
                || f.contains("crates/tokmd-config/")
                || *f == "docs/schema.json"
        })
        .cloned()
        .collect();

    Ok(Some(ContractDiffGate {
        meta: GateMeta {
            status,
            source: EvidenceSource::RanLocal,
            commit_match: CommitMatch::Unknown,
            scope: ScopeCoverage {
                relevant: relevant.clone(),
                tested: relevant,
                ratio: 1.0,
                lines_relevant: None,
                lines_tested: None,
            },
            evidence_commit: None,
            evidence_generated_at_ms: None,
        },
        semver,
        cli,
        schema,
        failures,
    }))
}

/// Run cargo-semver-checks if available.
/// Returns a SemverSubGate with the result.
fn run_semver_check(repo_root: &Path) -> SemverSubGate {
    // Check if cargo-semver-checks is available
    let available = Command::new("cargo")
        .args(["semver-checks", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !available {
        return SemverSubGate {
            status: GateStatus::Pending,
            breaking_changes: Vec::new(),
        };
    }

    // Run cargo semver-checks
    let output = match Command::new("cargo")
        .args(["semver-checks", "check-release"])
        .current_dir(repo_root)
        .output()
    {
        Ok(o) => o,
        Err(_) => {
            return SemverSubGate {
                status: GateStatus::Pending,
                breaking_changes: Vec::new(),
            };
        }
    };

    if output.status.success() {
        // Exit 0 = no breaking changes
        return SemverSubGate {
            status: GateStatus::Pass,
            breaking_changes: Vec::new(),
        };
    }

    // Non-zero exit = breaking changes found
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{}{}", stdout, stderr);

    // Parse breaking changes from output lines
    // cargo-semver-checks output format: "--- failure[kind]: message ---" or similar
    let mut breaking_changes: Vec<BreakingChange> = Vec::new();
    for line in combined.lines() {
        let trimmed = line.trim();
        if trimmed.contains("BREAKING") || trimmed.starts_with("---") {
            breaking_changes.push(BreakingChange {
                kind: "semver".to_string(),
                path: String::new(),
                message: trimmed.to_string(),
            });
        }
    }

    // If we couldn't parse specific changes but the tool failed, add a generic entry
    if breaking_changes.is_empty() {
        breaking_changes.push(BreakingChange {
            kind: "semver".to_string(),
            path: String::new(),
            message: "cargo-semver-checks reported breaking changes".to_string(),
        });
    }

    // Limit output
    breaking_changes.truncate(20);

    SemverSubGate {
        status: GateStatus::Fail,
        breaking_changes,
    }
}

/// Run git diff on docs/schema.json to detect schema changes.
/// Returns a SchemaSubGate with the result.
fn run_schema_diff(repo_root: &Path, base: &str, head: &str) -> SchemaSubGate {
    // Use two-dot syntax for comparing refs directly (per project convention)
    let range = format!("{}..{}", base, head);
    let output = match Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["diff", &range, "--", "docs/schema.json"])
        .output()
    {
        Ok(o) => o,
        Err(_) => {
            return SchemaSubGate {
                status: GateStatus::Pending,
                diff_summary: None,
            };
        }
    };

    if !output.status.success() {
        return SchemaSubGate {
            status: GateStatus::Pending,
            diff_summary: None,
        };
    }

    let diff = String::from_utf8_lossy(&output.stdout);
    if diff.trim().is_empty() {
        // No diff means schema.json didn't change between these refs
        return SchemaSubGate {
            status: GateStatus::Pass,
            diff_summary: None,
        };
    }

    // Analyze the diff for breaking vs additive changes
    let mut additions = 0usize;
    let mut removals = 0usize;
    let mut has_type_change = false;

    for line in diff.lines() {
        if line.starts_with('+') && !line.starts_with("+++") {
            additions += 1;
        } else if line.starts_with('-') && !line.starts_with("---") {
            removals += 1;
            // Check for type changes (field type modifications)
            let trimmed = line.trim_start_matches('-').trim();
            if trimmed.contains("\"type\"") {
                has_type_change = true;
            }
        }
    }

    let (status, summary) = if removals == 0 {
        // Only additions = safe additive change
        (
            GateStatus::Pass,
            Some(format!(
                "schema.json: {} line{} added (additive only)",
                additions,
                if additions == 1 { "" } else { "s" }
            )),
        )
    } else if has_type_change || removals > additions {
        // Type changes or net removals = likely breaking
        (
            GateStatus::Fail,
            Some(format!(
                "schema.json: {} addition{}, {} removal{} (potential breaking change)",
                additions,
                if additions == 1 { "" } else { "s" },
                removals,
                if removals == 1 { "" } else { "s" }
            )),
        )
    } else {
        // Removals but mostly additions = warn
        (
            GateStatus::Pass,
            Some(format!(
                "schema.json: {} addition{}, {} removal{}",
                additions,
                if additions == 1 { "" } else { "s" },
                removals,
                if removals == 1 { "" } else { "s" }
            )),
        )
    };

    SchemaSubGate {
        status,
        diff_summary: summary,
    }
}

/// Compute supply chain gate.
/// Checks if Cargo.lock changed and runs cargo-audit if available.
#[cfg(feature = "git")]
fn compute_supply_chain_gate(
    repo_root: &Path,
    changed_files: &[String],
) -> Result<Option<SupplyChainGate>> {
    // Only compute if Cargo.lock changed
    let lock_changed = changed_files.iter().any(|f| f.ends_with("Cargo.lock"));
    if !lock_changed {
        return Ok(None);
    }

    // Check if cargo-audit is available
    let check = Command::new("cargo").arg("audit").arg("--version").output();

    let audit_available = check.as_ref().map(|o| o.status.success()).unwrap_or(false);

    if !audit_available {
        // cargo-audit not available, return Pending status
        return Ok(Some(SupplyChainGate {
            meta: GateMeta {
                status: GateStatus::Pending,
                source: EvidenceSource::RanLocal,
                commit_match: CommitMatch::Unknown,
                scope: ScopeCoverage {
                    relevant: vec!["Cargo.lock".to_string()],
                    tested: Vec::new(),
                    ratio: 0.0,
                    lines_relevant: None,
                    lines_tested: None,
                },
                evidence_commit: None,
                evidence_generated_at_ms: None,
            },
            vulnerabilities: Vec::new(),
            denied: Vec::new(),
            advisory_db_version: None,
        }));
    }

    // Run cargo audit with JSON output
    let audit_output = Command::new("cargo")
        .args(["audit", "--json"])
        .current_dir(repo_root)
        .output();

    let output = match audit_output {
        Ok(o) => o,
        Err(_) => {
            // Failed to run cargo-audit, return Pending
            return Ok(Some(SupplyChainGate {
                meta: GateMeta {
                    status: GateStatus::Pending,
                    source: EvidenceSource::RanLocal,
                    commit_match: CommitMatch::Unknown,
                    scope: ScopeCoverage {
                        relevant: vec!["Cargo.lock".to_string()],
                        tested: Vec::new(),
                        ratio: 0.0,
                        lines_relevant: None,
                        lines_tested: None,
                    },
                    evidence_commit: None,
                    evidence_generated_at_ms: None,
                },
                vulnerabilities: Vec::new(),
                denied: Vec::new(),
                advisory_db_version: None,
            }));
        }
    };

    // Parse JSON output
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Intermediate structs for parsing cargo-audit JSON output
    #[derive(Deserialize)]
    struct AuditOutput {
        database: Option<AuditDatabase>,
        vulnerabilities: Option<AuditVulnerabilities>,
    }

    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct AuditDatabase {
        #[serde(rename = "advisory-count")]
        advisory_count: Option<u32>,
        version: Option<String>,
    }

    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct AuditVulnerabilities {
        found: Option<bool>,
        count: Option<u32>,
        list: Option<Vec<AuditVulnEntry>>,
    }

    #[derive(Deserialize)]
    struct AuditVulnEntry {
        advisory: Option<AuditAdvisory>,
        package: Option<AuditPackage>,
    }

    #[derive(Deserialize)]
    struct AuditAdvisory {
        id: Option<String>,
        severity: Option<String>,
        title: Option<String>,
    }

    #[derive(Deserialize)]
    struct AuditPackage {
        name: Option<String>,
    }

    let parsed: Result<AuditOutput, _> = serde_json::from_str(&stdout);

    let (vulnerabilities, advisory_db_version, status) = match parsed {
        Ok(audit) => {
            let db_version = audit.database.and_then(|db| db.version);

            let vulns: Vec<Vulnerability> = audit
                .vulnerabilities
                .and_then(|v| v.list)
                .unwrap_or_default()
                .into_iter()
                .filter_map(|entry| {
                    let advisory = entry.advisory?;
                    Some(Vulnerability {
                        id: advisory.id.unwrap_or_default(),
                        package: entry.package.and_then(|p| p.name).unwrap_or_default(),
                        severity: advisory
                            .severity
                            .clone()
                            .unwrap_or_else(|| "unknown".to_string()),
                        title: advisory.title.unwrap_or_default(),
                    })
                })
                .collect();

            // Determine status based on vulnerability severities
            let has_critical_or_high = vulns.iter().any(|v| {
                let sev = v.severity.to_lowercase();
                sev == "critical" || sev == "high"
            });
            let has_medium = vulns.iter().any(|v| v.severity.to_lowercase() == "medium");

            let status = if has_critical_or_high {
                GateStatus::Fail
            } else if has_medium {
                GateStatus::Warn
            } else {
                GateStatus::Pass
            };

            (vulns, db_version, status)
        }
        Err(_) => {
            // Failed to parse JSON, return Pending
            (Vec::new(), None, GateStatus::Pending)
        }
    };

    Ok(Some(SupplyChainGate {
        meta: GateMeta {
            status,
            source: EvidenceSource::RanLocal,
            commit_match: CommitMatch::Unknown,
            scope: ScopeCoverage {
                relevant: vec!["Cargo.lock".to_string()],
                tested: vec!["Cargo.lock".to_string()],
                ratio: 1.0,
                lines_relevant: None,
                lines_tested: None,
            },
            evidence_commit: None,
            evidence_generated_at_ms: None,
        },
        vulnerabilities,
        denied: Vec::new(),
        advisory_db_version,
    }))
}

/// Compute determinism gate.
/// Compares expected source hash (from baseline) with a fresh hash of the repo.
#[cfg(feature = "git")]
fn compute_determinism_gate(
    repo_root: &Path,
    baseline_path: Option<&Path>,
) -> Result<Option<DeterminismGate>> {
    use tokmd_analysis_types::ComplexityBaseline;

    fn short16(s: &str) -> &str {
        s.get(..16).unwrap_or(s)
    }

    // Resolve baseline: explicit path or default location
    let resolved_path = match baseline_path {
        Some(p) => p.to_path_buf(),
        None => repo_root.join(".tokmd/baseline.json"),
    };

    // If no baseline file exists, skip the gate
    if !resolved_path.exists() {
        return Ok(None);
    }

    // Parse baseline
    let content = std::fs::read_to_string(&resolved_path)
        .with_context(|| format!("failed to read baseline at {}", resolved_path.display()))?;
    let baseline: ComplexityBaseline = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse baseline at {}", resolved_path.display()))?;

    // If baseline has no determinism section, skip the gate
    let det = match &baseline.determinism {
        Some(d) => d,
        None => return Ok(None),
    };

    // Recompute current source hash by walking the repo, excluding the baseline file itself
    let baseline_rel = resolved_path
        .strip_prefix(repo_root)
        .ok()
        .map(|p| p.to_string_lossy().replace('\\', "/"));
    let exclude: Vec<&str> = baseline_rel.as_deref().into_iter().collect();
    let actual_hash = crate::determinism::hash_files_from_walk(repo_root, &exclude)?;
    let expected_hash = &det.source_hash;

    let mut differences = Vec::new();

    if actual_hash != *expected_hash {
        differences.push(format!(
            "source hash mismatch: expected {}, got {}",
            short16(expected_hash),
            short16(&actual_hash),
        ));
    }

    // Check Cargo.lock hash if baseline had one
    if let Some(expected_lock) = &det.cargo_lock_hash {
        let actual_lock = crate::determinism::hash_cargo_lock(repo_root)?;
        match actual_lock {
            Some(ref actual) if actual != expected_lock => {
                differences.push(format!(
                    "Cargo.lock hash mismatch: expected {}, got {}",
                    short16(expected_lock),
                    short16(actual),
                ));
            }
            None => {
                differences.push("Cargo.lock missing (was present in baseline)".to_string());
            }
            _ => {}
        }
    }

    let status = if differences.is_empty() {
        GateStatus::Pass
    } else {
        GateStatus::Warn
    };

    Ok(Some(DeterminismGate {
        meta: GateMeta {
            status,
            source: EvidenceSource::RanLocal,
            commit_match: CommitMatch::Unknown,
            scope: ScopeCoverage {
                relevant: vec!["source files".to_string()],
                tested: vec!["source files".to_string()],
                ratio: 1.0,
                lines_relevant: None,
                lines_tested: None,
            },
            evidence_commit: None,
            evidence_generated_at_ms: None,
        },
        expected_hash: Some(expected_hash.clone()),
        actual_hash: Some(actual_hash),
        algo: "blake3".to_string(),
        differences,
    }))
}

/// Cyclomatic complexity threshold for high complexity.
const COMPLEXITY_THRESHOLD: u32 = 15;

/// Compute complexity gate.
/// Analyzes cyclomatic complexity of changed Rust source files.
#[cfg(feature = "git")]
fn compute_complexity_gate(
    repo_root: &Path,
    changed_files: &[String],
) -> Result<Option<ComplexityGate>> {
    // Filter to relevant Rust source files
    let relevant_files: Vec<String> = changed_files
        .iter()
        .filter(|f| is_relevant_rust_source(f))
        .cloned()
        .collect();

    // If no relevant files, skip
    if relevant_files.is_empty() {
        return Ok(None);
    }

    let mut high_complexity_files = Vec::new();
    let mut total_complexity: u64 = 0;
    let mut max_cyclomatic: u32 = 0;
    let mut files_analyzed: usize = 0;

    for file_path in &relevant_files {
        let full_path = repo_root.join(file_path);
        if !full_path.exists() {
            continue;
        }

        if let Ok(content) = std::fs::read_to_string(&full_path) {
            let analysis = analyze_rust_complexity(&content);
            files_analyzed += 1;
            total_complexity += analysis.total_complexity as u64;
            max_cyclomatic = max_cyclomatic.max(analysis.max_complexity);

            if analysis.max_complexity > COMPLEXITY_THRESHOLD {
                high_complexity_files.push(HighComplexityFile {
                    path: file_path.clone(),
                    cyclomatic: analysis.max_complexity,
                    function_count: analysis.function_count,
                    max_function_length: analysis.max_function_length,
                });
            }
        }
    }

    // Sort high complexity files by cyclomatic complexity (descending), then path for determinism
    high_complexity_files.sort_by(|a, b| {
        b.cyclomatic
            .cmp(&a.cyclomatic)
            .then_with(|| a.path.cmp(&b.path))
    });

    let avg_cyclomatic = if files_analyzed > 0 {
        round_pct(total_complexity as f64 / files_analyzed as f64)
    } else {
        0.0
    };

    // Determine gate status:
    // - Pass: no high complexity files
    // - Warn (represented as Pending): 1-3 high complexity files
    // - Fail: >3 high complexity files
    let high_count = high_complexity_files.len();
    let (status, threshold_exceeded) = match high_count {
        0 => (GateStatus::Pass, false),
        1..=3 => (GateStatus::Pending, true), // Warn
        _ => (GateStatus::Fail, true),
    };

    Ok(Some(ComplexityGate {
        meta: GateMeta {
            status,
            source: EvidenceSource::RanLocal,
            commit_match: CommitMatch::Exact,
            scope: ScopeCoverage {
                relevant: relevant_files.clone(),
                tested: relevant_files,
                ratio: 1.0,
                lines_relevant: None,
                lines_tested: None,
            },
            evidence_commit: None,
            evidence_generated_at_ms: Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            ),
        },
        files_analyzed,
        high_complexity_files,
        avg_cyclomatic,
        max_cyclomatic,
        threshold_exceeded,
    }))
}

/// Results from analyzing a Rust file's complexity.
struct ComplexityAnalysis {
    /// Total cyclomatic complexity across all functions.
    total_complexity: u32,
    /// Maximum complexity of any single function.
    max_complexity: u32,
    /// Number of functions found.
    function_count: usize,
    /// Maximum function length in lines.
    max_function_length: usize,
}

/// Analyze the cyclomatic complexity of Rust source code.
/// Uses a simple heuristic approach counting decision points.
fn analyze_rust_complexity(content: &str) -> ComplexityAnalysis {
    let mut total_complexity: u32 = 0;
    let mut max_complexity: u32 = 0;
    let mut function_count: usize = 0;
    let mut max_function_length: usize = 0;

    let mut in_function = false;
    let mut brace_depth: i32 = 0;
    let mut function_brace_depth: i32 = 0; // Depth when function started
    let mut function_start_line: usize = 0;
    let mut current_complexity: u32 = 1; // Start at 1 for the function itself
    let mut in_string = false;
    let mut in_char = false;
    let mut in_block_comment = false;

    let lines: Vec<&str> = content.lines().collect();

    for (line_idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Check for function start BEFORE processing braces
        // (so we can track the starting brace depth correctly)
        let is_fn_start = !in_function
            && !in_block_comment
            && (trimmed.starts_with("fn ")
                || trimmed.starts_with("pub fn ")
                || trimmed.starts_with("pub(crate) fn ")
                || trimmed.starts_with("pub(super) fn ")
                || trimmed.starts_with("async fn ")
                || trimmed.starts_with("pub async fn ")
                || trimmed.starts_with("const fn ")
                || trimmed.starts_with("pub const fn ")
                || trimmed.starts_with("unsafe fn ")
                || trimmed.starts_with("pub unsafe fn "));

        if is_fn_start {
            in_function = true;
            function_start_line = line_idx;
            function_brace_depth = brace_depth;
            current_complexity = 1;
        }

        let mut in_line_comment = false;

        // Simple state machine for parsing
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            let next = chars.get(i + 1).copied();

            // Handle block comments
            if in_block_comment {
                if c == '*' && next == Some('/') {
                    in_block_comment = false;
                    i += 2;
                    continue;
                }
                i += 1;
                continue;
            }

            // Handle line comments
            if c == '/' && next == Some('/') {
                in_line_comment = true;
                break;
            }

            // Handle block comment start
            if c == '/' && next == Some('*') {
                in_block_comment = true;
                i += 2;
                continue;
            }

            // Handle strings
            if !in_char && c == '"' && (i == 0 || chars[i - 1] != '\\') {
                in_string = !in_string;
                i += 1;
                continue;
            }

            // Handle chars
            if !in_string && c == '\'' && (i == 0 || chars[i - 1] != '\\') {
                in_char = !in_char;
                i += 1;
                continue;
            }

            // Skip if in string or char
            if in_string || in_char {
                i += 1;
                continue;
            }

            // Track brace depth
            if c == '{' {
                brace_depth += 1;
            } else if c == '}' {
                brace_depth -= 1;
                if in_function && brace_depth == function_brace_depth {
                    // End of function
                    let function_length = line_idx - function_start_line + 1;
                    max_function_length = max_function_length.max(function_length);
                    total_complexity += current_complexity;
                    max_complexity = max_complexity.max(current_complexity);
                    function_count += 1;
                    in_function = false;
                    current_complexity = 1;
                }
            }

            i += 1;
        }

        // Skip complexity counting if in comment
        if in_line_comment || in_block_comment {
            continue;
        }

        // Count decision points for complexity (only inside functions)
        if in_function {
            // Count control flow keywords
            let keywords = [
                "if ", "else if ", "while ", "for ", "loop ", "match ", "&&", "||", "?",
            ];
            for kw in &keywords {
                // Count occurrences of each keyword
                let mut search_line = trimmed;
                while let Some(pos) = search_line.find(kw) {
                    current_complexity += 1;
                    search_line = &search_line[pos + kw.len()..];
                }
            }

            // Count match arms (each => in a match adds complexity)
            if trimmed.contains("=>") && !trimmed.starts_with("//") {
                // Count number of => in the line
                let arrow_count = trimmed.matches("=>").count();
                current_complexity += arrow_count as u32;
            }
        }
    }

    // Handle case where file ends without closing brace
    if in_function {
        function_count += 1;
        total_complexity += current_complexity;
        max_complexity = max_complexity.max(current_complexity);
    }

    ComplexityAnalysis {
        total_complexity,
        max_complexity,
        function_count,
        max_function_length,
    }
}

/// Check if a file is a relevant Rust source file for mutation testing.
/// Excludes test files, fuzz targets, etc.
fn is_relevant_rust_source(path: &str) -> bool {
    let path_lower = path.to_lowercase();

    // Must be a .rs file
    if !path_lower.ends_with(".rs") {
        return false;
    }

    // Exclude test directories
    if path_lower.contains("/tests/") || path_lower.starts_with("tests/") {
        return false;
    }

    // Exclude test files
    if path_lower.ends_with("_test.rs") || path_lower.ends_with("_tests.rs") {
        return false;
    }

    // Exclude fuzz targets
    if path_lower.contains("/fuzz/") || path_lower.starts_with("fuzz/") {
        return false;
    }

    true
}

/// Get the current HEAD commit hash.
#[cfg(feature = "git")]
fn get_head_commit(repo_root: &PathBuf) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .context("Failed to run git rev-parse HEAD")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git rev-parse HEAD failed: {}", stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// CI workflow summary format (mutants-summary.json).
#[derive(Debug, Clone, Deserialize)]
struct CiMutantsSummary {
    commit: String,
    status: String,
    scope: Vec<String>,
    survivors: Vec<CiSurvivor>,
    killed: usize,
    timeout: usize,
    unviable: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct CiSurvivor {
    file: String,
    line: usize,
    mutation: String,
}

/// Compute the mutation gate status.
#[cfg(feature = "git")]
fn compute_mutation_gate(
    repo_root: &PathBuf,
    _base: &str,
    _head: &str,
    changed_files: &[String],
    _range_mode: tokmd_git::GitRangeMode,
) -> Result<MutationGate> {
    // Filter to relevant Rust source files
    let relevant_files: Vec<String> = changed_files
        .iter()
        .filter(|f| is_relevant_rust_source(f))
        .cloned()
        .collect();

    // If no relevant files, skip
    if relevant_files.is_empty() {
        return Ok(MutationGate {
            meta: GateMeta {
                status: GateStatus::Skipped,
                source: EvidenceSource::RanLocal,
                commit_match: CommitMatch::Unknown,
                scope: ScopeCoverage {
                    relevant: Vec::new(),
                    tested: Vec::new(),
                    ratio: 1.0,
                    lines_relevant: None,
                    lines_tested: None,
                },
                evidence_commit: None,
                evidence_generated_at_ms: None,
            },
            survivors: Vec::new(),
            killed: 0,
            timeout: 0,
            unviable: 0,
        });
    }

    let head_commit = get_head_commit(repo_root)?;

    // Try to find cached results
    if let Some(gate) = try_load_ci_artifact(repo_root, &head_commit, &relevant_files)? {
        return Ok(gate);
    }

    if let Some(gate) = try_load_cached(repo_root, &head_commit, &relevant_files)? {
        return Ok(gate);
    }

    // Try to run mutations
    run_mutations(repo_root, &relevant_files)
}

/// Try to load mutation results from CI artifact.
/// Checks for mutants-summary.json (our format) first, then falls back to mutants.out/outcomes.json.
#[cfg(feature = "git")]
fn try_load_ci_artifact(
    repo_root: &Path,
    head_commit: &str,
    relevant_files: &[String],
) -> Result<Option<MutationGate>> {
    // First, check for our summary format (mutants-summary.json)
    let summary_path = repo_root.join("mutants-summary.json");
    if summary_path.exists()
        && let Ok(content) = std::fs::read_to_string(&summary_path)
        && let Ok(summary) = serde_json::from_str::<CiMutantsSummary>(&content)
    {
        // Determine commit match quality
        let commit_match = if summary.commit.starts_with(head_commit)
            || head_commit.starts_with(&summary.commit)
        {
            CommitMatch::Exact
        } else {
            CommitMatch::Stale
        };

        // Skip stale artifacts
        if commit_match == CommitMatch::Stale {
            return Ok(None);
        }

        let status = match summary.status.as_str() {
            "pass" => GateStatus::Pass,
            "fail" => GateStatus::Fail,
            "skipped" => GateStatus::Skipped,
            _ => GateStatus::Pending,
        };

        let survivors: Vec<MutationSurvivor> = summary
            .survivors
            .into_iter()
            .map(|s| MutationSurvivor {
                file: s.file,
                line: s.line,
                mutation: s.mutation,
            })
            .collect();

        let tested = summary.scope.clone();
        let scope_ratio = if relevant_files.is_empty() {
            1.0
        } else {
            tested.len() as f64 / relevant_files.len() as f64
        };

        let gate = MutationGate {
            meta: GateMeta {
                status,
                source: EvidenceSource::CiArtifact,
                commit_match,
                scope: ScopeCoverage {
                    relevant: relevant_files.to_vec(),
                    tested,
                    ratio: scope_ratio.min(1.0),
                    lines_relevant: None,
                    lines_tested: None,
                },
                evidence_commit: Some(summary.commit),
                evidence_generated_at_ms: None,
            },
            survivors,
            killed: summary.killed,
            timeout: summary.timeout,
            unviable: summary.unviable,
        };

        Ok(Some(gate))
    } else {
        Ok(None)
    }
}

/// Try to load cached mutation results.
#[cfg(feature = "git")]
fn try_load_cached(
    repo_root: &Path,
    head_commit: &str,
    relevant_files: &[String],
) -> Result<Option<MutationGate>> {
    let cache_dir = repo_root.join(".tokmd/cache/mutants");
    if !cache_dir.exists() {
        return Ok(None);
    }

    let cache_file = cache_dir.join(format!("{}.json", head_commit));
    if !cache_file.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&cache_file)?;
    let gate: MutationGate = serde_json::from_str(&content)?;

    // Verify scope hasn't changed significantly
    let tested = &gate.meta.scope.tested;
    let missing_files: Vec<_> = relevant_files
        .iter()
        .filter(|f| !tested.contains(f))
        .collect();

    if !missing_files.is_empty() {
        // Cache is partial
        return Ok(None);
    }

    Ok(Some(gate))
}

/// Run mutations locally.
#[cfg(feature = "git")]
fn run_mutations(_repo_root: &Path, relevant_files: &[String]) -> Result<MutationGate> {
    // This is expensive, so we only do it if explicitly asked or no other choice
    // For now, return Pending
    Ok(MutationGate {
        meta: GateMeta {
            status: GateStatus::Pending,
            source: EvidenceSource::RanLocal,
            commit_match: CommitMatch::Exact,
            scope: ScopeCoverage {
                relevant: relevant_files.to_vec(),
                tested: Vec::new(),
                ratio: 0.0,
                lines_relevant: None,
                lines_tested: None,
            },
            evidence_commit: None,
            evidence_generated_at_ms: None,
        },
        survivors: Vec::new(),
        killed: 0,
        timeout: 0,
        unviable: 0,
    })
}

/// Get file stats for changed files.
#[cfg(feature = "git")]
fn get_file_stats(
    repo_root: &Path,
    base: &str,
    head: &str,
    range_mode: tokmd_git::GitRangeMode,
) -> Result<Vec<FileStat>> {
    let range = range_mode.format(base, head);
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["diff", "--numstat", &range])
        .output()
        .context("Failed to run git diff --numstat")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git diff --numstat failed: {}", stderr.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut stats = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 3 {
            let insertions = parts[0].parse().unwrap_or(0);
            let deletions = parts[1].parse().unwrap_or(0);
            let path = parts[2].to_string();
            stats.push(FileStat {
                path,
                insertions,
                deletions,
            });
        }
    }

    Ok(stats)
}

#[derive(Debug, Clone)]
struct FileStat {
    path: String,
    insertions: usize,
    deletions: usize,
}

/// Compute change surface metrics.
#[cfg(feature = "git")]
fn compute_change_surface(
    repo_root: &Path,
    base: &str,
    head: &str,
    file_stats: &[FileStat],
    range_mode: tokmd_git::GitRangeMode,
) -> Result<ChangeSurface> {
    let range = range_mode.format(base, head);
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["rev-list", "--count", &range])
        .output()
        .context("Failed to run git rev-list --count")?;

    let commits = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .unwrap_or(0);

    let files_changed = file_stats.len();
    let insertions = file_stats.iter().map(|s| s.insertions).sum();
    let deletions = file_stats.iter().map(|s| s.deletions).sum();
    let net_lines = (insertions as i64) - (deletions as i64);

    let churn_velocity = if commits > 0 {
        (insertions + deletions) as f64 / commits as f64
    } else {
        0.0
    };

    // Simple change concentration: what % of changes are in top 20% of files
    let mut changes: Vec<usize> = file_stats
        .iter()
        .map(|s| s.insertions + s.deletions)
        .collect();
    changes.sort_by_key(|&c| Reverse(c));

    let top_count = (files_changed as f64 * 0.2).ceil() as usize;
    let total_changes: usize = changes.iter().sum();
    let top_changes: usize = changes.iter().take(top_count).sum();

    let change_concentration = if total_changes > 0 {
        top_changes as f64 / total_changes as f64
    } else {
        0.0
    };

    Ok(ChangeSurface {
        commits,
        files_changed,
        insertions,
        deletions,
        net_lines,
        churn_velocity,
        change_concentration,
    })
}

/// Compute composition metrics.
fn compute_composition(files: &[String]) -> Composition {
    let mut code = 0;
    let mut test = 0;
    let mut docs = 0;
    let mut config = 0;

    for file in files {
        let path = file.to_lowercase();
        if path.ends_with(".rs")
            || path.ends_with(".js")
            || path.ends_with(".ts")
            || path.ends_with(".py")
        {
            if path.contains("test") || path.contains("_spec") {
                test += 1;
            } else {
                code += 1;
            }
        } else if path.ends_with(".md") || path.contains("/docs/") {
            docs += 1;
        } else if path.ends_with(".toml")
            || path.ends_with(".json")
            || path.ends_with(".yml")
            || path.ends_with(".yaml")
        {
            config += 1;
        }
    }

    let total = (code + test + docs + config) as f64;
    let (code_pct, test_pct, docs_pct, config_pct) = if total > 0.0 {
        (
            code as f64 / total,
            test as f64 / total,
            docs as f64 / total,
            config as f64 / total,
        )
    } else {
        (0.0, 0.0, 0.0, 0.0)
    };

    let test_ratio = if code > 0 {
        test as f64 / code as f64
    } else if test > 0 {
        1.0
    } else {
        0.0
    };

    Composition {
        code_pct,
        test_pct,
        docs_pct,
        config_pct,
        test_ratio,
    }
}

/// Detect contract changes.
fn detect_contracts(files: &[String]) -> Contracts {
    let mut api_changed = false;
    let mut cli_changed = false;
    let mut schema_changed = false;
    let mut breaking_indicators = 0;

    for file in files {
        if file.ends_with("lib.rs") || file.ends_with("mod.rs") {
            api_changed = true;
        }
        if file.contains("crates/tokmd/src/commands/") || file.contains("crates/tokmd-config/") {
            cli_changed = true;
        }
        if file == "docs/schema.json" || file == "docs/SCHEMA.md" {
            schema_changed = true;
        }
    }

    if api_changed {
        breaking_indicators += 1;
    }
    if schema_changed {
        breaking_indicators += 1;
    }

    Contracts {
        api_changed,
        cli_changed,
        schema_changed,
        breaking_indicators,
    }
}

/// Compute code health metrics.
fn compute_code_health(file_stats: &[FileStat], contracts: &Contracts) -> CodeHealth {
    let mut large_files_touched = 0;
    let mut total_lines = 0;

    for stat in file_stats {
        let lines = stat.insertions + stat.deletions;
        if lines > 500 {
            large_files_touched += 1;
        }
        total_lines += lines;
    }

    let avg_file_size = if !file_stats.is_empty() {
        total_lines / file_stats.len()
    } else {
        0
    };

    let complexity_indicator = if large_files_touched > 5 {
        ComplexityIndicator::Critical
    } else if large_files_touched > 2 {
        ComplexityIndicator::High
    } else if large_files_touched > 0 {
        ComplexityIndicator::Medium
    } else {
        ComplexityIndicator::Low
    };

    let mut warnings = Vec::new();
    for stat in file_stats {
        if stat.insertions + stat.deletions > 500 {
            warnings.push(HealthWarning {
                path: stat.path.clone(),
                warning_type: WarningType::LargeFile,
                message: "Large file touched".to_string(),
            });
        }
    }

    let mut score: u32 = 100;
    score = score.saturating_sub((large_files_touched * 10) as u32);
    if contracts.breaking_indicators > 0 {
        score = score.saturating_sub(20);
    }

    let grade = match score {
        90..=100 => "A",
        80..=89 => "B",
        70..=79 => "C",
        60..=69 => "D",
        _ => "F",
    }
    .to_string();

    CodeHealth {
        score,
        grade,
        large_files_touched,
        avg_file_size,
        complexity_indicator,
        warnings,
    }
}

/// Compute risk metrics.
fn compute_risk(file_stats: &[FileStat], _contracts: &Contracts, health: &CodeHealth) -> Risk {
    let mut hotspots_touched = Vec::new();
    let bus_factor_warnings = Vec::new();

    for stat in file_stats {
        if stat.insertions + stat.deletions > 300 {
            hotspots_touched.push(stat.path.clone());
        }
    }

    let score = (hotspots_touched.len() * 15 + (100 - health.score) as usize).min(100) as u32;

    let level = match score {
        0..=20 => RiskLevel::Low,
        21..=50 => RiskLevel::Medium,
        51..=80 => RiskLevel::High,
        _ => RiskLevel::Critical,
    };

    Risk {
        hotspots_touched,
        bus_factor_warnings,
        level,
        score,
    }
}

/// Generate review plan.
fn generate_review_plan(file_stats: &[FileStat], _contracts: &Contracts) -> Vec<ReviewItem> {
    let mut items = Vec::new();

    for stat in file_stats {
        let lines = stat.insertions + stat.deletions;
        let priority = if lines > 200 {
            1
        } else if lines > 50 {
            2
        } else {
            3
        };
        let complexity = if lines > 300 {
            5
        } else if lines > 100 {
            3
        } else {
            1
        };

        items.push(ReviewItem {
            path: stat.path.clone(),
            reason: format!("{} lines changed", lines),
            priority,
            complexity: Some(complexity),
            lines_changed: Some(lines),
        });
    }

    items.sort_by_key(|i| i.priority);
    items
}

/// Render receipt as JSON.
fn render_json(receipt: &CockpitReceipt) -> Result<String> {
    serde_json::to_string_pretty(receipt).context("Failed to serialize receipt to JSON")
}

/// Render receipt as Markdown summary.
fn render_markdown(receipt: &CockpitReceipt) -> String {
    use std::fmt::Write;
    let mut s = String::new();

    let _ = writeln!(s, "## Glass Cockpit");
    let _ = writeln!(s);

    // Summary comparison table
    s.push_str("### Summary\n\n");
    s.push_str("|Metric|Value|\n");
    s.push_str("|---|---:|\n");
    let _ = writeln!(
        s,
        "|Files Changed|{}|",
        receipt.change_surface.files_changed
    );
    let _ = writeln!(s, "|Insertions|{}|", receipt.change_surface.insertions);
    let _ = writeln!(s, "|Deletions|{}|", receipt.change_surface.deletions);
    let _ = writeln!(s, "|Net Lines|{}|", receipt.change_surface.net_lines);
    let _ = writeln!(s, "|Code Health Score|{}/100|", receipt.code_health.score);
    let _ = writeln!(s, "|Risk Score|{}/100|", receipt.risk.score);
    let _ = writeln!(s, "|Test Ratio|{:.2}|", receipt.composition.test_ratio);
    s.push('\n');

    // Change Surface section
    let _ = writeln!(s, "### Change Surface");
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "- **Files changed**: {}",
        receipt.change_surface.files_changed
    );
    let _ = writeln!(s, "- **Insertions**: {}", receipt.change_surface.insertions);
    let _ = writeln!(s, "- **Deletions**: {}", receipt.change_surface.deletions);
    let _ = writeln!(s, "- **Net lines**: {}", receipt.change_surface.net_lines);
    let _ = writeln!(
        s,
        "- **Churn velocity**: {:.1}",
        receipt.change_surface.churn_velocity
    );
    let _ = writeln!(s);

    // Composition section
    let _ = writeln!(s, "### Composition");
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "- **Code**: {:.1}%",
        receipt.composition.code_pct * 100.0
    );
    let _ = writeln!(
        s,
        "- **Test**: {:.1}%",
        receipt.composition.test_pct * 100.0
    );
    let _ = writeln!(
        s,
        "- **Docs**: {:.1}%",
        receipt.composition.docs_pct * 100.0
    );
    let _ = writeln!(
        s,
        "- **Config**: {:.1}%",
        receipt.composition.config_pct * 100.0
    );
    let _ = writeln!(s, "- **Test ratio**: {:.2}", receipt.composition.test_ratio);
    let _ = writeln!(s);

    // Contracts section
    let _ = writeln!(s, "### Contracts");
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "- **API changed**: {}",
        if receipt.contracts.api_changed {
            "Yes"
        } else {
            "No"
        }
    );
    let _ = writeln!(
        s,
        "- **CLI changed**: {}",
        if receipt.contracts.cli_changed {
            "Yes"
        } else {
            "No"
        }
    );
    let _ = writeln!(
        s,
        "- **Schema changed**: {}",
        if receipt.contracts.schema_changed {
            "Yes"
        } else {
            "No"
        }
    );
    let _ = writeln!(
        s,
        "- **Breaking indicators**: {}",
        receipt.contracts.breaking_indicators
    );
    let _ = writeln!(s);

    // Code Health section
    let _ = writeln!(s, "### Code Health");
    let _ = writeln!(s);
    let _ = writeln!(s, "- **Score**: {}/100", receipt.code_health.score);
    let _ = writeln!(s, "- **Grade**: {}", receipt.code_health.grade);
    let _ = writeln!(
        s,
        "- **Large files touched**: {}",
        receipt.code_health.large_files_touched
    );
    let _ = writeln!(
        s,
        "- **Average file size**: {}",
        receipt.code_health.avg_file_size
    );
    let _ = writeln!(
        s,
        "- **Complexity indicator**: {:?}",
        receipt.code_health.complexity_indicator
    );
    if !receipt.code_health.warnings.is_empty() {
        let _ = writeln!(s, "- **Warnings**:");
        for warning in &receipt.code_health.warnings {
            let _ = writeln!(s, "  - {}: {}", warning.path, warning.message);
        }
    }
    let _ = writeln!(s);

    // Risk section
    let _ = writeln!(s, "### Risk");
    let _ = writeln!(s);
    let _ = writeln!(s, "- **Level**: {}", receipt.risk.level);
    let _ = writeln!(s, "- **Score**: {}/100", receipt.risk.score);
    if !receipt.risk.hotspots_touched.is_empty() {
        let _ = writeln!(s, "- **Hotspots touched**:");
        for hotspot in &receipt.risk.hotspots_touched {
            let _ = writeln!(s, "  - {}", hotspot);
        }
    }
    if !receipt.risk.bus_factor_warnings.is_empty() {
        let _ = writeln!(s, "- **Bus factor warnings**:");
        for warning in &receipt.risk.bus_factor_warnings {
            let _ = writeln!(s, "  - {}", warning);
        }
    }
    let _ = writeln!(s);

    // Evidence Gates section
    let _ = writeln!(s, "### Evidence Gates");
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "- **Overall status**: {:?}",
        receipt.evidence.overall_status
    );
    let _ = writeln!(
        s,
        "- **Mutation**: {:?} (killed: {}, survivors: {})",
        receipt.evidence.mutation.meta.status,
        receipt.evidence.mutation.killed,
        receipt.evidence.mutation.survivors.len()
    );
    if let Some(ref dc) = receipt.evidence.diff_coverage {
        let _ = writeln!(
            s,
            "- **Diff coverage**: {:?} ({:.1}%)",
            dc.meta.status,
            dc.coverage_pct * 100.0
        );
    }
    if let Some(ref contracts) = receipt.evidence.contracts {
        let _ = writeln!(
            s,
            "- **Contracts**: {:?} (failures: {})",
            contracts.meta.status, contracts.failures
        );
    }
    if let Some(ref sc) = receipt.evidence.supply_chain {
        let _ = writeln!(
            s,
            "- **Supply chain**: {:?} (vulnerabilities: {})",
            sc.meta.status,
            sc.vulnerabilities.len()
        );
    }
    if let Some(ref det) = receipt.evidence.determinism {
        let _ = writeln!(
            s,
            "- **Determinism**: {:?} (differences: {})",
            det.meta.status,
            det.differences.len()
        );
    }
    if let Some(ref cx) = receipt.evidence.complexity {
        let _ = writeln!(
            s,
            "- **Complexity**: {:?} (avg cyclomatic: {:.1}, max: {})",
            cx.meta.status, cx.avg_cyclomatic, cx.max_cyclomatic
        );
    }
    let _ = writeln!(s);

    // Review Plan section
    let _ = writeln!(s, "### Review Plan");
    let _ = writeln!(s);
    if receipt.review_plan.is_empty() {
        let _ = writeln!(s, "No review items.");
    } else {
        for item in &receipt.review_plan {
            let _ = writeln!(s, "- **{}** (priority: {})", item.path, item.priority);
            let _ = writeln!(s, "  - Reason: {}", item.reason);
            if let Some(complexity) = item.complexity {
                let _ = writeln!(s, "  - Complexity: {}", complexity);
            }
            if let Some(lines) = item.lines_changed {
                let _ = writeln!(s, "  - Lines changed: {}", lines);
            }
        }
    }
    let _ = writeln!(s);

    // Trend section (if available)
    if let Some(ref trend) = receipt.trend {
        let _ = writeln!(s, "### Trend");
        let _ = writeln!(s);
        if trend.baseline_available {
            let _ = writeln!(
                s,
                "- **Baseline**: {}",
                trend.baseline_path.as_deref().unwrap_or("N/A")
            );
            if let Some(ref health) = trend.health {
                let _ = writeln!(
                    s,
                    "- **Health**: {:.1} → {:.1} {} ({:.1}%, {:?})",
                    health.previous,
                    health.current,
                    sparkline(&[health.previous, health.current]),
                    health.delta_pct,
                    health.direction
                );
            }
            if let Some(ref risk) = trend.risk {
                let _ = writeln!(
                    s,
                    "- **Risk**: {:.1} → {:.1} {} ({:.1}%, {:?})",
                    risk.previous,
                    risk.current,
                    sparkline(&[risk.previous, risk.current]),
                    risk.delta_pct,
                    risk.direction
                );
            }
            if let Some(ref complexity) = trend.complexity {
                let _ = writeln!(
                    s,
                    "- **Complexity**: {} ({:?})",
                    complexity.summary, complexity.direction
                );
            }
        } else {
            let _ = writeln!(s, "No baseline available for comparison.");
        }
        let _ = writeln!(s);
    }

    s
}

/// Render receipt as sectioned output.
fn render_sections(receipt: &CockpitReceipt) -> String {
    use std::fmt::Write;
    let mut s = String::new();

    let _ = writeln!(s, "<!-- SECTION:COCKPIT -->");
    let _ = writeln!(s);
    let _ = writeln!(s, "## Glass Cockpit");
    let _ = writeln!(s);
    let _ = writeln!(s, "**Base**: {}", receipt.base_ref);
    let _ = writeln!(s, "**Head**: {}", receipt.head_ref);
    let _ = writeln!(s);
    let _ = writeln!(s, "**Change Surface**:");
    let _ = writeln!(s, "- Files: {}", receipt.change_surface.files_changed);
    let _ = writeln!(s, "- Insertions: {}", receipt.change_surface.insertions);
    let _ = writeln!(s, "- Deletions: {}", receipt.change_surface.deletions);
    let _ = writeln!(s);
    let _ = writeln!(s, "**Composition**:");
    let _ = writeln!(s, "- Code: {:.1}%", receipt.composition.code_pct * 100.0);
    let _ = writeln!(s, "- Test: {:.1}%", receipt.composition.test_pct * 100.0);
    let _ = writeln!(s, "- Docs: {:.1}%", receipt.composition.docs_pct * 100.0);
    let _ = writeln!(
        s,
        "- Config: {:.1}%",
        receipt.composition.config_pct * 100.0
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "**Contracts**:");
    let _ = writeln!(
        s,
        "- API: {}",
        if receipt.contracts.api_changed {
            "Yes"
        } else {
            "No"
        }
    );
    let _ = writeln!(
        s,
        "- CLI: {}",
        if receipt.contracts.cli_changed {
            "Yes"
        } else {
            "No"
        }
    );
    let _ = writeln!(
        s,
        "- Schema: {}",
        if receipt.contracts.schema_changed {
            "Yes"
        } else {
            "No"
        }
    );
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "**Health**: {}/100 ({})",
        receipt.code_health.score, receipt.code_health.grade
    );
    let _ = writeln!(
        s,
        "**Risk**: {} ({}/100)",
        receipt.risk.level, receipt.risk.score
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "<!-- SECTION:REVIEW_PLAN -->");
    let _ = writeln!(s);
    let _ = writeln!(s, "## Review Plan");
    let _ = writeln!(s);
    if receipt.review_plan.is_empty() {
        let _ = writeln!(s, "No review items.");
    } else {
        for item in &receipt.review_plan {
            let _ = writeln!(s, "- {} (priority: {})", item.path, item.priority);
        }
    }
    let _ = writeln!(s);
    let _ = writeln!(s, "<!-- SECTION:RECEIPTS -->");
    let _ = writeln!(s);
    let _ = writeln!(s, "## Receipts");
    let _ = writeln!(s);
    let _ = writeln!(s, "Full receipt data available in JSON format.");
    let _ = writeln!(s);

    s
}

/// Render comment.md for PR comments.
fn render_comment_md(receipt: &CockpitReceipt) -> String {
    use std::fmt::Write;
    let mut s = String::new();

    // Summary bullet points
    let _ = writeln!(s, "## Glass Cockpit Summary");
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "- **{} files changed**, +{}/-{}",
        receipt.change_surface.files_changed,
        receipt.change_surface.insertions,
        receipt.change_surface.deletions
    );
    let _ = writeln!(
        s,
        "- **Health**: {}/100 ({})",
        receipt.code_health.score, receipt.code_health.grade
    );
    let _ = writeln!(
        s,
        "- **Risk**: {} ({}/100)",
        receipt.risk.level, receipt.risk.score
    );
    let _ = writeln!(s);

    // Contract changes
    if receipt.contracts.api_changed
        || receipt.contracts.cli_changed
        || receipt.contracts.schema_changed
    {
        let _ = writeln!(s, "**Contract changes**:");
        if receipt.contracts.api_changed {
            let _ = writeln!(s, "- API contract changed");
        }
        if receipt.contracts.cli_changed {
            let _ = writeln!(s, "- CLI contract changed");
        }
        if receipt.contracts.schema_changed {
            let _ = writeln!(s, "- Schema contract changed");
        }
        if receipt.contracts.breaking_indicators > 0 {
            let _ = writeln!(
                s,
                "- {} breaking indicator(s)",
                receipt.contracts.breaking_indicators
            );
        }
        let _ = writeln!(s);
    }

    // Evidence gates
    let _ = writeln!(
        s,
        "**Evidence gates**: {:?}",
        receipt.evidence.overall_status
    );
    if !receipt.evidence.mutation.survivors.is_empty() {
        let _ = writeln!(
            s,
            "- Mutation: {} survivors detected",
            receipt.evidence.mutation.survivors.len()
        );
    }
    if let Some(ref dc) = receipt.evidence.diff_coverage {
        let _ = writeln!(s, "- Diff coverage: {:.1}%", dc.coverage_pct * 100.0);
    }
    if let Some(ref contracts) = receipt.evidence.contracts
        && contracts.failures > 0
    {
        let _ = writeln!(s, "- Contracts: {} sub-gate(s) failed", contracts.failures);
    }
    if let Some(ref sc) = receipt.evidence.supply_chain
        && !sc.vulnerabilities.is_empty()
    {
        let _ = writeln!(
            s,
            "- Supply chain: {} vulnerability/vulnerabilities",
            sc.vulnerabilities.len()
        );
    }
    if let Some(ref cx) = receipt.evidence.complexity
        && cx.threshold_exceeded
    {
        let _ = writeln!(
            s,
            "- Complexity: threshold exceeded (max cyclomatic: {})",
            cx.max_cyclomatic
        );
    }
    let _ = writeln!(s);

    // Review plan (priority items only)
    let priority_items: Vec<_> = receipt
        .review_plan
        .iter()
        .filter(|item| item.priority <= 2)
        .collect();

    if !priority_items.is_empty() {
        let _ = writeln!(s, "**Priority review items**:");
        for item in priority_items {
            let _ = writeln!(s, "- {} ({})", item.path, item.reason);
        }
        let _ = writeln!(s);
    }

    s
}

/// Write artifacts to directory.
fn write_artifacts(dir: &Path, receipt: &CockpitReceipt) -> Result<()> {
    std::fs::create_dir_all(dir)?;

    // Write cockpit.json (full receipt)
    let json = render_json(receipt)?;
    std::fs::write(dir.join("cockpit.json"), json)?;

    // Write report.json (sensor report envelope)
    let verdict = match receipt.evidence.overall_status {
        GateStatus::Pass => Verdict::Pass,
        GateStatus::Fail => Verdict::Fail,
        _ => Verdict::Warn,
    };

    let report = SensorReport::new(
        ToolMeta::tokmd(env!("CARGO_PKG_VERSION"), "cockpit"),
        now_iso8601(),
        verdict,
        format!(
            "{} files changed, +{}/-{}, health {}/100, risk {} in {}..{}",
            receipt.change_surface.files_changed,
            receipt.change_surface.insertions,
            receipt.change_surface.deletions,
            receipt.code_health.score,
            receipt.risk.level,
            receipt.base_ref,
            receipt.head_ref
        ),
    );

    let report_json = serde_json::to_string_pretty(&report)?;
    std::fs::write(dir.join("report.json"), report_json)?;

    // Write comment.md (markdown summary)
    let comment_md = render_comment_md(receipt);
    std::fs::write(dir.join("comment.md"), comment_md)?;

    Ok(())
}

/// Write sensor artifacts.
#[cfg(feature = "git")]
fn write_sensor_artifacts(
    dir: &Path,
    receipt: &CockpitReceipt,
    base: &str,
    head: &str,
) -> Result<()> {
    std::fs::create_dir_all(dir)?;

    // Build sensor report
    let verdict = match receipt.evidence.overall_status {
        GateStatus::Pass => Verdict::Pass,
        GateStatus::Fail => Verdict::Fail,
        _ => Verdict::Warn,
    };

    let report = SensorReport::new(
        ToolMeta::tokmd(env!("CARGO_PKG_VERSION"), "cockpit"),
        now_iso8601(),
        verdict,
        format!("Cockpit run for {}..{}", base, head),
    );

    let json = serde_json::to_string_pretty(&report)?;
    std::fs::write(dir.join("report.json"), json)?;

    Ok(())
}

fn sparkline(values: &[f64]) -> String {
    if values.is_empty() {
        return String::new();
    }

    const BARS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let min = values
        .iter()
        .copied()
        .fold(f64::INFINITY, |acc, v| acc.min(v));
    let max = values
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, |acc, v| acc.max(v));

    if !min.is_finite() || !max.is_finite() {
        return String::new();
    }

    if (max - min).abs() < f64::EPSILON {
        return std::iter::repeat(BARS[3]).take(values.len()).collect();
    }

    let span = max - min;
    values
        .iter()
        .map(|v| {
            let norm = ((v - min) / span).clamp(0.0, 1.0);
            let idx = (norm * (BARS.len() as f64 - 1.0)).round() as usize;
            BARS[idx]
        })
        .collect()
}

fn now_iso8601() -> String {
    "2024-01-01T00:00:00Z".to_string()
}

fn round_pct(val: f64) -> f64 {
    (val * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::sparkline;

    #[test]
    fn sparkline_rises() {
        let s = sparkline(&[10.0, 20.0, 30.0]);
        assert_eq!(s.chars().count(), 3);
        assert!(s.ends_with('█'));
    }

    #[test]
    fn sparkline_flat() {
        let s = sparkline(&[5.0, 5.0, 5.0]);
        assert_eq!(s, "▄▄▄");
    }

    #[test]
    fn sparkline_empty() {
        assert!(sparkline(&[]).is_empty());
    }
}
