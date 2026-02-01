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

/// Cockpit receipt schema version.
const SCHEMA_VERSION: u32 = 3;

/// Handle the cockpit command.
pub(crate) fn handle(args: cli::CockpitArgs, _global: &cli::GlobalArgs) -> Result<()> {
    #[cfg(not(feature = "git"))]
    {
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

        let mut receipt = compute_cockpit(&repo_root, &args.base, &args.head)?;

        // Load baseline and compute trend if provided
        if let Some(baseline_path) = &args.baseline {
            receipt.trend = Some(load_and_compute_trend(baseline_path, &receipt)?);
        }

        let output = match args.format {
            cli::CockpitFormat::Json => render_json(&receipt)?,
            cli::CockpitFormat::Md => render_markdown(&receipt),
            cli::CockpitFormat::Sections => render_sections(&receipt),
        };

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
fn compute_cockpit(repo_root: &PathBuf, base: &str, head: &str) -> Result<CockpitReceipt> {
    let generated_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    // Get changed files with their stats
    let file_stats = get_file_stats(repo_root, base, head)?;
    let changed_files: Vec<String> = file_stats.iter().map(|f| f.path.clone()).collect();

    // Get change surface from git
    let change_surface = compute_change_surface(repo_root, base, head, &file_stats)?;

    // Compute composition with test ratio
    let composition = compute_composition(&changed_files);

    // Detect contract changes
    let contracts = detect_contracts(&changed_files);

    // Compute code health
    let code_health = compute_code_health(&file_stats, &contracts);

    // Compute risk based on various factors
    let risk = compute_risk(&file_stats, &contracts, &code_health);

    // Compute all gate evidence
    let evidence = compute_evidence(repo_root, base, head, &changed_files, &contracts)?;

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
) -> Result<Evidence> {
    let mutation = compute_mutation_gate(repo_root, base, head, changed_files)?;
    let diff_coverage = compute_diff_coverage_gate(repo_root)?;
    let contracts = compute_contract_gate(repo_root, changed_files, contracts_info)?;
    let supply_chain = compute_supply_chain_gate(repo_root, changed_files)?;
    let determinism = compute_determinism_gate(repo_root)?;
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
/// Looks for coverage.json or lcov.info artifact.
#[cfg(feature = "git")]
fn compute_diff_coverage_gate(_repo_root: &Path) -> Result<Option<DiffCoverageGate>> {
    // TODO: Look for coverage artifacts and parse them
    // For now, return None (gate not configured)
    Ok(None)
}

/// Compute contract diff gate (semver, CLI, schema).
#[cfg(feature = "git")]
fn compute_contract_gate(
    _repo_root: &Path,
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
        // TODO: Run cargo-semver-checks if available
        // For now, mark as pending
        semver = Some(SemverSubGate {
            status: GateStatus::Pending,
            breaking_changes: Vec::new(),
        });
    }

    // Check for CLI changes
    if contracts_info.cli_changed {
        // TODO: Diff --help output
        cli = Some(CliSubGate {
            status: GateStatus::Pending,
            diff_summary: None,
        });
    }

    // Check for schema changes
    if contracts_info.schema_changed {
        // TODO: Diff schema.json
        schema = Some(SchemaSubGate {
            status: GateStatus::Pending,
            diff_summary: None,
        });
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

/// Compute supply chain gate.
/// Checks if Cargo.lock changed and runs cargo-audit if available.
#[cfg(feature = "git")]
fn compute_supply_chain_gate(
    _repo_root: &Path,
    changed_files: &[String],
) -> Result<Option<SupplyChainGate>> {
    // Only compute if Cargo.lock changed
    let lock_changed = changed_files.iter().any(|f| f.ends_with("Cargo.lock"));
    if !lock_changed {
        return Ok(None);
    }

    // Check if cargo-audit is available
    let check = Command::new("cargo").arg("audit").arg("--version").output();

    let status = if check.is_ok() && check.unwrap().status.success() {
        // TODO: Actually run cargo audit and parse results
        GateStatus::Pending
    } else {
        GateStatus::Pending
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
        vulnerabilities: Vec::new(),
        denied: Vec::new(),
        advisory_db_version: None,
    }))
}

/// Compute determinism gate.
/// Compares expected hash (from baseline) with actual hash.
#[cfg(feature = "git")]
fn compute_determinism_gate(_repo_root: &Path) -> Result<Option<DeterminismGate>> {
    // TODO: Look for baseline hash and compare with current
    // For now, return None (no baseline available)
    Ok(None)
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

    // Sort high complexity files by cyclomatic complexity (descending)
    high_complexity_files.sort_by(|a, b| b.cyclomatic.cmp(&a.cyclomatic));

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
    #[allow(dead_code)]
    schema_version: u32,
    commit: String,
    #[allow(dead_code)]
    base_ref: String,
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

/// Parsed mutation outcome from cargo-mutants output.
#[derive(Debug, Clone, Deserialize)]
struct MutantsOutcome {
    outcomes: Vec<MutantOutcomeEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct MutantOutcomeEntry {
    scenario: MutantScenario,
    summary: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
#[allow(non_snake_case, dead_code)]
enum MutantScenario {
    BaselineString(String),
    Mutant { Mutant: MutantInfo },
}

#[derive(Debug, Clone, Deserialize)]
struct MutantInfo {
    file: String,
    name: String,
    span: MutantSpan,
}

#[derive(Debug, Clone, Deserialize)]
struct MutantSpan {
    start: MutantPosition,
}

#[derive(Debug, Clone, Deserialize)]
struct MutantPosition {
    line: usize,
}

/// Compute the mutation gate status.
#[cfg(feature = "git")]
fn compute_mutation_gate(
    repo_root: &PathBuf,
    _base: &str,
    _head: &str,
    changed_files: &[String],
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

        // Cache the results for future use
        cache_mutation_results(repo_root, head_commit, &gate)?;

        return Ok(Some(gate));
    }

    // Fall back to raw cargo-mutants output (mutants.out/outcomes.json)
    let outcomes_path = repo_root.join("mutants.out").join("outcomes.json");
    if !outcomes_path.exists() {
        return Ok(None);
    }

    // Parse outcomes.json
    let content = std::fs::read_to_string(&outcomes_path)
        .with_context(|| format!("Failed to read {}", outcomes_path.display()))?;

    let outcomes: MutantsOutcome = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {}", outcomes_path.display()))?;

    let gate = parse_mutation_outcomes(
        &outcomes,
        relevant_files,
        EvidenceSource::CiArtifact,
        head_commit,
    );

    // Cache the results for future use
    cache_mutation_results(repo_root, head_commit, &gate)?;

    Ok(Some(gate))
}

/// Try to load mutation results from local cache (.tokmd/cache/mutants-{commit}.json).
#[cfg(feature = "git")]
fn try_load_cached(
    repo_root: &Path,
    head_commit: &str,
    relevant_files: &[String],
) -> Result<Option<MutationGate>> {
    let cache_dir = repo_root.join(".tokmd").join("cache");
    let cache_file = cache_dir.join(format!(
        "mutants-{}.json",
        &head_commit[..12.min(head_commit.len())]
    ));

    if !cache_file.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&cache_file)
        .with_context(|| format!("Failed to read cache file {}", cache_file.display()))?;

    let mut gate: MutationGate = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse cache file {}", cache_file.display()))?;

    // Verify scope matches (or is superset of) relevant files
    let cached_scope: std::collections::HashSet<_> = gate.meta.scope.tested.iter().collect();
    let needed: Vec<_> = relevant_files
        .iter()
        .filter(|f| !cached_scope.contains(f))
        .collect();

    if !needed.is_empty() {
        // Cache doesn't cover all files we need
        return Ok(None);
    }

    gate.meta.source = EvidenceSource::Cached;
    gate.meta.commit_match = CommitMatch::Exact;
    Ok(Some(gate))
}

/// Cache mutation results for future use.
#[cfg(feature = "git")]
fn cache_mutation_results(repo_root: &Path, head_commit: &str, gate: &MutationGate) -> Result<()> {
    let cache_dir = repo_root.join(".tokmd").join("cache");
    std::fs::create_dir_all(&cache_dir)?;

    let cache_file = cache_dir.join(format!(
        "mutants-{}.json",
        &head_commit[..12.min(head_commit.len())]
    ));
    let content = serde_json::to_string_pretty(gate)?;
    std::fs::write(&cache_file, content)?;

    Ok(())
}

/// Run mutation testing on the given files.
#[cfg(feature = "git")]
fn run_mutations(repo_root: &PathBuf, relevant_files: &[String]) -> Result<MutationGate> {
    // Check if cargo-mutants is available
    let check = Command::new("cargo")
        .arg("mutants")
        .arg("--version")
        .output();

    let head_commit = get_head_commit(repo_root).ok();

    if check.is_err() || !check.unwrap().status.success() {
        // cargo-mutants not installed
        return Ok(MutationGate {
            meta: GateMeta {
                status: GateStatus::Pending,
                source: EvidenceSource::RanLocal,
                commit_match: CommitMatch::Unknown,
                scope: ScopeCoverage {
                    relevant: relevant_files.to_vec(),
                    tested: Vec::new(),
                    ratio: 0.0,
                    lines_relevant: None,
                    lines_tested: None,
                },
                evidence_commit: head_commit,
                evidence_generated_at_ms: None,
            },
            survivors: Vec::new(),
            killed: 0,
            timeout: 0,
            unviable: 0,
        });
    }

    let mut all_survivors = Vec::new();
    let mut total_killed = 0usize;
    let mut total_timeout = 0usize;
    let mut total_unviable = 0usize;
    let mut tested_files = Vec::new();

    for file in relevant_files.iter().take(20) {
        // Limit to 20 files like CI
        // Determine if file is in a crate subdirectory
        let (work_dir, file_arg) = if file.starts_with("crates/") {
            // Extract crate directory (e.g., "crates/tokmd-types")
            let parts: Vec<&str> = file.splitn(3, '/').collect();
            if parts.len() >= 3 {
                let crate_dir = repo_root.join(parts[0]).join(parts[1]);
                if crate_dir.join("Cargo.toml").exists() {
                    // Run from crate directory with relative path
                    let rel_path = parts[2..].join("/");
                    (crate_dir, rel_path)
                } else {
                    (repo_root.clone(), file.clone())
                }
            } else {
                (repo_root.clone(), file.clone())
            }
        } else {
            (repo_root.clone(), file.clone())
        };

        let output = Command::new("cargo")
            .arg("mutants")
            .arg("--file")
            .arg(&file_arg)
            .arg("--timeout")
            .arg("120")
            .arg("--json")
            .current_dir(&work_dir)
            .output();

        match output {
            Ok(out) if out.status.success() => {
                tested_files.push(file.clone());

                // Parse the outcomes.json that cargo-mutants creates
                let outcomes_path = work_dir.join("mutants.out").join("outcomes.json");
                if let Ok(content) = std::fs::read_to_string(&outcomes_path)
                    && let Ok(outcomes) = serde_json::from_str::<MutantsOutcome>(&content)
                {
                    for entry in &outcomes.outcomes {
                        let MutantScenario::Mutant { Mutant: info } = &entry.scenario else {
                            continue;
                        };
                        match entry.summary.as_str() {
                            "CaughtMutant" => total_killed += 1,
                            "Timeout" => total_timeout += 1,
                            "Unviable" => total_unviable += 1,
                            "MissedMutant" => {
                                all_survivors.push(MutationSurvivor {
                                    file: info.file.clone(),
                                    line: info.span.start.line,
                                    mutation: info.name.clone(),
                                });
                            }
                            _ => {}
                        }
                    }
                }
            }
            Ok(_) => {
                // Command failed but ran - file was attempted
                tested_files.push(file.clone());
            }
            Err(_) => {
                // Command failed to execute
                continue;
            }
        }
    }

    let status = if all_survivors.is_empty() {
        GateStatus::Pass
    } else {
        GateStatus::Fail
    };

    let scope_ratio = if relevant_files.is_empty() {
        1.0
    } else {
        tested_files.len() as f64 / relevant_files.len() as f64
    };

    let generated_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let gate = MutationGate {
        meta: GateMeta {
            status,
            source: EvidenceSource::RanLocal,
            commit_match: CommitMatch::Exact,
            scope: ScopeCoverage {
                relevant: relevant_files.to_vec(),
                tested: tested_files,
                ratio: scope_ratio.min(1.0),
                lines_relevant: None,
                lines_tested: None,
            },
            evidence_commit: head_commit.clone(),
            evidence_generated_at_ms: Some(generated_at_ms),
        },
        survivors: all_survivors,
        killed: total_killed,
        timeout: total_timeout,
        unviable: total_unviable,
    };

    // Cache the results
    if let Some(ref commit) = head_commit {
        let _ = cache_mutation_results(repo_root, commit, &gate);
    }

    Ok(gate)
}

/// Parse mutation outcomes into a MutationGate.
fn parse_mutation_outcomes(
    outcomes: &MutantsOutcome,
    relevant_files: &[String],
    source: EvidenceSource,
    head_commit: &str,
) -> MutationGate {
    let relevant_set: std::collections::HashSet<_> = relevant_files.iter().collect();

    let mut survivors = Vec::new();
    let mut killed = 0usize;
    let mut timeout = 0usize;
    let mut unviable = 0usize;
    let mut scope_set: std::collections::HashSet<String> = std::collections::HashSet::new();

    for entry in &outcomes.outcomes {
        let MutantScenario::Mutant { Mutant: info } = &entry.scenario else {
            continue;
        };

        // Normalize path for comparison
        let file_normalized = info.file.replace('\\', "/");

        // Only count if file is in our relevant set
        if !relevant_set
            .iter()
            .any(|f| file_normalized.ends_with(*f) || f.ends_with(&file_normalized))
        {
            continue;
        }

        scope_set.insert(file_normalized.clone());

        match entry.summary.as_str() {
            "CaughtMutant" => killed += 1,
            "Timeout" => timeout += 1,
            "Unviable" => unviable += 1,
            "MissedMutant" => {
                survivors.push(MutationSurvivor {
                    file: file_normalized,
                    line: info.span.start.line,
                    mutation: info.name.clone(),
                });
            }
            _ => {}
        }
    }

    let status = if survivors.is_empty() {
        GateStatus::Pass
    } else {
        GateStatus::Fail
    };

    let tested: Vec<String> = scope_set.into_iter().collect();
    let scope_ratio = if relevant_files.is_empty() {
        1.0
    } else {
        tested.len() as f64 / relevant_files.len() as f64
    };

    MutationGate {
        meta: GateMeta {
            status,
            source,
            commit_match: CommitMatch::Unknown,
            scope: ScopeCoverage {
                relevant: relevant_files.to_vec(),
                tested,
                ratio: scope_ratio.min(1.0),
                lines_relevant: None,
                lines_tested: None,
            },
            evidence_commit: Some(head_commit.to_string()),
            evidence_generated_at_ms: None,
        },
        survivors,
        killed,
        timeout,
        unviable,
    }
}

/// Compute evidence when git feature is disabled.
#[cfg(not(feature = "git"))]
fn compute_evidence_disabled() -> Evidence {
    Evidence {
        overall_status: GateStatus::Skipped,
        mutation: MutationGate {
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
        },
        diff_coverage: None,
        contracts: None,
        supply_chain: None,
        determinism: None,
        complexity: None,
    }
}

/// Per-file statistics from git diff.
#[derive(Debug, Clone)]
struct FileStats {
    path: String,
    insertions: usize,
    deletions: usize,
}

impl FileStats {
    fn total_lines(&self) -> usize {
        self.insertions + self.deletions
    }
}

/// Get per-file diff statistics between two commits.
///
/// Uses two-dot syntax (`base..head`) which shows the actual diff between commits.
/// Do NOT use three-dot syntax (`base...head`) here - that shows changes from the
/// merge-base ancestor, which inflates line counts when comparing tags/branches.
///
/// Two-dot vs three-dot:
/// - `A..B`  = commits reachable from B but not A (actual diff)
/// - `A...B` = commits reachable from either but not both (symmetric difference)
#[cfg(feature = "git")]
fn get_file_stats(repo_root: &PathBuf, base: &str, head: &str) -> Result<Vec<FileStats>> {
    // NOTE: Using two-dot syntax for accurate diff stats between commits.
    // Three-dot would show changes from merge-base, inflating counts.
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("diff")
        .arg("--numstat")
        .arg(format!("{}..{}", base, head))
        .output()
        .context("Failed to run git diff --numstat")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git diff --numstat failed: {}", stderr.trim());
    }

    let stats_str = String::from_utf8_lossy(&output.stdout);
    let mut stats = Vec::new();

    for line in stats_str.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            // Handle binary files (shown as -)
            let insertions = parts[0].parse().unwrap_or(0);
            let deletions = parts[1].parse().unwrap_or(0);
            let path = parts[2..].join(" ");

            stats.push(FileStats {
                path,
                insertions,
                deletions,
            });
        }
    }

    Ok(stats)
}

#[cfg(feature = "git")]
fn compute_change_surface(
    repo_root: &PathBuf,
    base: &str,
    head: &str,
    file_stats: &[FileStats],
) -> Result<ChangeSurface> {
    // Get commit count
    let commits = get_commit_count(repo_root, base, head)?;

    // Calculate totals from file stats
    let files_changed = file_stats.len();
    let insertions: usize = file_stats.iter().map(|f| f.insertions).sum();
    let deletions: usize = file_stats.iter().map(|f| f.deletions).sum();
    let net_lines = insertions as i64 - deletions as i64;

    // Churn velocity: average lines changed per commit
    let total_churn = insertions + deletions;
    let churn_velocity = if commits > 0 {
        round_pct(total_churn as f64 / commits as f64)
    } else {
        0.0
    };

    // Change concentration: what % of changes are in top 20% of files
    let change_concentration = compute_change_concentration(file_stats);

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

/// Compute what percentage of changes are concentrated in top 20% of files.
fn compute_change_concentration(file_stats: &[FileStats]) -> f64 {
    if file_stats.is_empty() {
        return 0.0;
    }

    let total_lines: usize = file_stats.iter().map(|f| f.total_lines()).sum();
    if total_lines == 0 {
        return 0.0;
    }

    // Sort by lines changed (descending)
    let mut sorted: Vec<usize> = file_stats.iter().map(|f| f.total_lines()).collect();
    sorted.sort_by(|a, b| b.cmp(a));

    // Get top 20% of files
    let top_count = (file_stats.len() as f64 * 0.2).ceil() as usize;
    let top_count = top_count.max(1);

    let top_lines: usize = sorted.iter().take(top_count).sum();
    round_pct(top_lines as f64 / total_lines as f64 * 100.0)
}

#[cfg(feature = "git")]
fn get_commit_count(repo_root: &PathBuf, base: &str, head: &str) -> Result<usize> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("rev-list")
        .arg("--count")
        .arg(format!("{}..{}", base, head))
        .output()
        .context("Failed to run git rev-list")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git rev-list failed: {}", stderr.trim());
    }

    let count_str = String::from_utf8_lossy(&output.stdout);
    count_str
        .trim()
        .parse::<usize>()
        .context("Failed to parse commit count")
}

/// File classification for composition analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum FileCategory {
    Code,
    Test,
    Docs,
    Config,
}

fn classify_file(path: &str) -> FileCategory {
    let path_lower = path.to_lowercase();

    // Test patterns
    if path_lower.contains("/tests/")
        || path_lower.contains("/test/")
        || path_lower.starts_with("tests/")
        || path_lower.starts_with("test/")
        || path_lower.contains("_test.")
        || path_lower.contains(".test.")
        || path_lower.contains("_spec.")
        || path_lower.ends_with("_test.rs")
        || path_lower.ends_with("_tests.rs")
    {
        return FileCategory::Test;
    }

    // Docs patterns
    if path_lower.ends_with(".md")
        || path_lower.starts_with("docs/")
        || path_lower.contains("/docs/")
        || path_lower.contains("readme")
    {
        return FileCategory::Docs;
    }

    // Config/CI patterns
    if path_lower.starts_with(".github/")
        || path_lower.ends_with(".toml")
        || path_lower.ends_with(".yml")
        || path_lower.ends_with(".yaml")
        || path_lower.ends_with(".json")
        || path_lower == "justfile"
        || path_lower == "makefile"
        || path_lower.ends_with(".lock")
    {
        return FileCategory::Config;
    }

    // Everything else is code
    FileCategory::Code
}

fn compute_composition(files: &[String]) -> Composition {
    if files.is_empty() {
        return Composition {
            code_pct: 0.0,
            test_pct: 0.0,
            docs_pct: 0.0,
            config_pct: 0.0,
            test_ratio: 0.0,
        };
    }

    let mut counts: BTreeMap<FileCategory, usize> = BTreeMap::new();
    for file in files {
        let cat = classify_file(file);
        *counts.entry(cat).or_insert(0) += 1;
    }

    let total = files.len() as f64;
    let code = *counts.get(&FileCategory::Code).unwrap_or(&0) as f64;
    let test = *counts.get(&FileCategory::Test).unwrap_or(&0) as f64;
    let docs = *counts.get(&FileCategory::Docs).unwrap_or(&0) as f64;
    let config = *counts.get(&FileCategory::Config).unwrap_or(&0) as f64;

    // Test-to-code ratio: how many test files per code file
    let test_ratio = if code > 0.0 {
        round_pct(test / code)
    } else {
        0.0
    };

    Composition {
        code_pct: round_pct(code / total * 100.0),
        test_pct: round_pct(test / total * 100.0),
        docs_pct: round_pct(docs / total * 100.0),
        config_pct: round_pct(config / total * 100.0),
        test_ratio,
    }
}

fn round_pct(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

/// Contract detection patterns.
fn detect_contracts(files: &[String]) -> Contracts {
    let mut api_changed = false;
    let mut cli_changed = false;
    let mut schema_changed = false;
    let mut breaking_indicators = 0;

    for file in files {
        // API changes: lib.rs files in crates
        if file.contains("crates/") && file.ends_with("/src/lib.rs") {
            api_changed = true;
            breaking_indicators += 1;
        }
        if file.ends_with("/mod.rs") {
            api_changed = true;
        }

        // CLI changes
        if file.contains("crates/tokmd/src/commands/") {
            cli_changed = true;
        }
        if file.contains("crates/tokmd-config/") {
            cli_changed = true;
            breaking_indicators += 1;
        }

        // Schema changes
        if file == "docs/schema.json" {
            schema_changed = true;
            breaking_indicators += 2; // Schema changes are high impact
        }
        if file.contains("crates/tokmd-types/") {
            schema_changed = true;
            breaking_indicators += 1;
        }
        if file.contains("crates/tokmd-analysis-types/") {
            schema_changed = true;
            breaking_indicators += 1;
        }
    }

    Contracts {
        api_changed,
        cli_changed,
        schema_changed,
        breaking_indicators,
    }
}

/// Compute code health metrics for DevEx analysis.
fn compute_code_health(file_stats: &[FileStats], contracts: &Contracts) -> CodeHealth {
    let mut warnings: Vec<HealthWarning> = Vec::new();

    // Count large files (>500 lines changed)
    let large_files_touched = file_stats.iter().filter(|f| f.total_lines() > 500).count();

    // Average file size
    let avg_file_size = if file_stats.is_empty() {
        0
    } else {
        file_stats.iter().map(|f| f.total_lines()).sum::<usize>() / file_stats.len()
    };

    // Add warnings for large files
    for file in file_stats.iter().filter(|f| f.total_lines() > 500) {
        warnings.push(HealthWarning {
            path: file.path.clone(),
            warning_type: WarningType::LargeFile,
            message: format!("Large change: {} lines modified", file.total_lines()),
        });
    }

    // Add warnings for high churn files (>200 lines with deletions)
    for file in file_stats
        .iter()
        .filter(|f| f.total_lines() > 200 && f.deletions > 100)
    {
        warnings.push(HealthWarning {
            path: file.path.clone(),
            warning_type: WarningType::HighChurn,
            message: format!("High churn: +{} -{} lines", file.insertions, file.deletions),
        });
    }

    // Compute complexity indicator
    let complexity_indicator = compute_complexity_indicator(file_stats, contracts);

    // Compute health score (0-100)
    let score = compute_health_score(
        file_stats,
        large_files_touched,
        &complexity_indicator,
        contracts,
    );

    // Compute grade
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

fn compute_complexity_indicator(
    file_stats: &[FileStats],
    contracts: &Contracts,
) -> ComplexityIndicator {
    let total_lines: usize = file_stats.iter().map(|f| f.total_lines()).sum();
    let file_count = file_stats.len();

    // Multiple factors contribute to complexity
    let mut complexity_score = 0;

    // Factor 1: Total lines changed
    if total_lines > 1000 {
        complexity_score += 2;
    } else if total_lines > 500 {
        complexity_score += 1;
    }

    // Factor 2: Number of files
    if file_count > 20 {
        complexity_score += 2;
    } else if file_count > 10 {
        complexity_score += 1;
    }

    // Factor 3: Breaking changes
    if contracts.breaking_indicators >= 3 {
        complexity_score += 2;
    } else if contracts.breaking_indicators >= 1 {
        complexity_score += 1;
    }

    // Factor 4: Schema changes are always complex
    if contracts.schema_changed {
        complexity_score += 1;
    }

    match complexity_score {
        0..=1 => ComplexityIndicator::Low,
        2..=3 => ComplexityIndicator::Medium,
        4..=5 => ComplexityIndicator::High,
        _ => ComplexityIndicator::Critical,
    }
}

fn compute_health_score(
    file_stats: &[FileStats],
    large_files: usize,
    complexity: &ComplexityIndicator,
    contracts: &Contracts,
) -> u32 {
    let mut score = 100i32;

    // Deduct for large files
    score -= (large_files * 5) as i32;

    // Deduct for complexity
    match complexity {
        ComplexityIndicator::Low => {}
        ComplexityIndicator::Medium => score -= 10,
        ComplexityIndicator::High => score -= 20,
        ComplexityIndicator::Critical => score -= 35,
    }

    // Deduct for breaking changes
    score -= (contracts.breaking_indicators * 3) as i32;

    // Deduct for very large total changes
    let total_lines: usize = file_stats.iter().map(|f| f.total_lines()).sum();
    if total_lines > 2000 {
        score -= 15;
    } else if total_lines > 1000 {
        score -= 10;
    } else if total_lines > 500 {
        score -= 5;
    }

    score.clamp(0, 100) as u32
}

/// Compute risk indicators for the PR.
fn compute_risk(file_stats: &[FileStats], contracts: &Contracts, health: &CodeHealth) -> Risk {
    let mut hotspots_touched = Vec::new();
    let mut bus_factor_warnings = Vec::new();

    // High-churn files are potential hotspots
    for file in file_stats.iter().filter(|f| f.total_lines() > 300) {
        hotspots_touched.push(file.path.clone());
    }

    // Core infrastructure files are bus factor risks
    for file in file_stats {
        if file.path.contains("/src/lib.rs")
            || file.path.contains("/src/main.rs")
            || file.path == "Cargo.toml"
        {
            bus_factor_warnings.push(format!("Core file modified: {}", file.path));
        }
    }

    // Compute risk score
    let mut risk_score = 0u32;

    // Factor in file count
    if file_stats.len() > 50 {
        risk_score += 30;
    } else if file_stats.len() > 20 {
        risk_score += 15;
    } else if file_stats.len() > 10 {
        risk_score += 5;
    }

    // Factor in breaking changes
    risk_score += (contracts.breaking_indicators * 10).min(40) as u32;

    // Factor in health score (inverse)
    risk_score += (100 - health.score) / 3;

    // Factor in hotspots
    risk_score += (hotspots_touched.len() * 5).min(20) as u32;

    let risk_score = risk_score.min(100);

    let level = match risk_score {
        0..=20 => RiskLevel::Low,
        21..=45 => RiskLevel::Medium,
        46..=70 => RiskLevel::High,
        _ => RiskLevel::Critical,
    };

    Risk {
        hotspots_touched,
        bus_factor_warnings,
        level,
        score: risk_score,
    }
}

fn generate_review_plan(file_stats: &[FileStats], contracts: &Contracts) -> Vec<ReviewItem> {
    let mut items: Vec<ReviewItem> = Vec::new();
    let mut priority = 1u32;

    // Helper to find file stats (currently unused but available for future use)
    let _find_stats = |path: &str| file_stats.iter().find(|f| f.path == path);

    // Helper to compute complexity (1-5) based on lines changed
    let compute_file_complexity = |stats: Option<&FileStats>| -> u8 {
        match stats.map(|s| s.total_lines()).unwrap_or(0) {
            0..=50 => 1,
            51..=150 => 2,
            151..=300 => 3,
            301..=500 => 4,
            _ => 5,
        }
    };

    // Priority 1: Schema changes (high impact)
    if contracts.schema_changed {
        for fs in file_stats {
            if fs.path == "docs/schema.json"
                || fs.path.contains("crates/tokmd-types/")
                || fs.path.contains("crates/tokmd-analysis-types/")
            {
                items.push(ReviewItem {
                    path: fs.path.clone(),
                    reason: "Schema change".to_string(),
                    priority,
                    complexity: Some(compute_file_complexity(Some(fs))),
                    lines_changed: Some(fs.total_lines()),
                });
            }
        }
        priority += 1;
    }

    // Priority 2: API changes
    if contracts.api_changed {
        for fs in file_stats {
            if ((fs.path.contains("crates/") && fs.path.ends_with("/src/lib.rs"))
                || fs.path.ends_with("/mod.rs"))
                && !items.iter().any(|i| i.path == fs.path)
            {
                items.push(ReviewItem {
                    path: fs.path.clone(),
                    reason: "API surface".to_string(),
                    priority,
                    complexity: Some(compute_file_complexity(Some(fs))),
                    lines_changed: Some(fs.total_lines()),
                });
            }
        }
        priority += 1;
    }

    // Priority 3: CLI changes
    if contracts.cli_changed {
        for fs in file_stats {
            if (fs.path.contains("crates/tokmd/src/commands/")
                || fs.path.contains("crates/tokmd-config/"))
                && !items.iter().any(|i| i.path == fs.path)
            {
                items.push(ReviewItem {
                    path: fs.path.clone(),
                    reason: "CLI interface".to_string(),
                    priority,
                    complexity: Some(compute_file_complexity(Some(fs))),
                    lines_changed: Some(fs.total_lines()),
                });
            }
        }
        priority += 1;
    }

    // Priority 4: Test files
    for fs in file_stats {
        if classify_file(&fs.path) == FileCategory::Test && !items.iter().any(|i| i.path == fs.path)
        {
            items.push(ReviewItem {
                path: fs.path.clone(),
                reason: "Test coverage".to_string(),
                priority,
                complexity: Some(compute_file_complexity(Some(fs))),
                lines_changed: Some(fs.total_lines()),
            });
        }
    }
    if items.iter().any(|i| i.reason == "Test coverage") {
        priority += 1;
    }

    // Priority 5: Remaining files (sorted by lines changed descending)
    let mut remaining: Vec<&FileStats> = file_stats
        .iter()
        .filter(|fs| !items.iter().any(|i| i.path == fs.path))
        .collect();
    remaining.sort_by_key(|f| Reverse(f.total_lines()));

    for fs in remaining {
        let cat = classify_file(&fs.path);
        let reason = match cat {
            FileCategory::Code => "Implementation".to_string(),
            FileCategory::Docs => "Documentation".to_string(),
            FileCategory::Config => "Configuration".to_string(),
            FileCategory::Test => "Test".to_string(),
        };
        items.push(ReviewItem {
            path: fs.path.clone(),
            reason,
            priority,
            complexity: Some(compute_file_complexity(Some(fs))),
            lines_changed: Some(fs.total_lines()),
        });
    }

    items
}

fn render_json(receipt: &CockpitReceipt) -> Result<String> {
    serde_json::to_string_pretty(receipt).context("Failed to serialize cockpit receipt")
}

fn render_markdown(receipt: &CockpitReceipt) -> String {
    let mut out = String::new();

    out.push_str("## Glass Cockpit\n\n");

    // Health summary badge-style
    out.push_str(&format!(
        "**Health:** {} ({}/100) | **Risk:** {:?} ({}/100)\n\n",
        receipt.code_health.grade,
        receipt.code_health.score,
        receipt.risk.level,
        receipt.risk.score
    ));

    // Change Surface
    out.push_str("### Change Surface\n\n");
    out.push_str("| Metric | Value |\n");
    out.push_str("|--------|-------|\n");
    out.push_str(&format!(
        "| Commits | {} |\n",
        receipt.change_surface.commits
    ));
    out.push_str(&format!(
        "| Files changed | {} |\n",
        receipt.change_surface.files_changed
    ));
    out.push_str(&format!(
        "| Lines | +{}/-{} (net: {}) |\n",
        receipt.change_surface.insertions,
        receipt.change_surface.deletions,
        receipt.change_surface.net_lines
    ));
    out.push_str(&format!(
        "| Churn velocity | {:.1} lines/commit |\n",
        receipt.change_surface.churn_velocity
    ));
    out.push_str(&format!(
        "| Change concentration | {:.1}% in top 20% files |\n",
        receipt.change_surface.change_concentration
    ));
    out.push('\n');

    // Composition
    out.push_str("### Composition\n\n");
    out.push_str("| Category | Percentage |\n");
    out.push_str("|----------|------------|\n");
    out.push_str(&format!(
        "| Code | {:.1}% |\n",
        receipt.composition.code_pct
    ));
    out.push_str(&format!(
        "| Tests | {:.1}% |\n",
        receipt.composition.test_pct
    ));
    out.push_str(&format!(
        "| Docs | {:.1}% |\n",
        receipt.composition.docs_pct
    ));
    out.push_str(&format!(
        "| Config | {:.1}% |\n",
        receipt.composition.config_pct
    ));
    out.push_str(&format!(
        "| **Test ratio** | {:.2} tests/code |\n",
        receipt.composition.test_ratio
    ));
    out.push('\n');

    // Code Health
    out.push_str("### Code Health\n\n");
    out.push_str("| Metric | Value |\n");
    out.push_str("|--------|-------|\n");
    out.push_str(&format!(
        "| Health score | {} ({}) |\n",
        receipt.code_health.score, receipt.code_health.grade
    ));
    out.push_str(&format!(
        "| Complexity | {:?} |\n",
        receipt.code_health.complexity_indicator
    ));
    out.push_str(&format!(
        "| Large files touched | {} |\n",
        receipt.code_health.large_files_touched
    ));
    out.push_str(&format!(
        "| Avg file size | {} lines |\n",
        receipt.code_health.avg_file_size
    ));
    out.push('\n');

    // Health warnings
    if !receipt.code_health.warnings.is_empty() {
        out.push_str("#### Warnings\n\n");
        for warning in &receipt.code_health.warnings {
            out.push_str(&format!(
                "- **{:?}**: `{}` - {}\n",
                warning.warning_type, warning.path, warning.message
            ));
        }
        out.push('\n');
    }

    // Contracts
    out.push_str("### Contracts\n\n");
    out.push_str("| Contract | Changed | Breaking |\n");
    out.push_str("|----------|:-------:|:--------:|\n");
    out.push_str(&format!(
        "| API | {} | {} |\n",
        if receipt.contracts.api_changed {
            "Yes"
        } else {
            "No"
        },
        if receipt.contracts.api_changed {
            "Possible"
        } else {
            "-"
        }
    ));
    out.push_str(&format!(
        "| CLI | {} | {} |\n",
        if receipt.contracts.cli_changed {
            "Yes"
        } else {
            "No"
        },
        if receipt.contracts.cli_changed {
            "Possible"
        } else {
            "-"
        }
    ));
    out.push_str(&format!(
        "| Schema | {} | {} |\n",
        if receipt.contracts.schema_changed {
            "Yes"
        } else {
            "No"
        },
        if receipt.contracts.schema_changed {
            "Likely"
        } else {
            "-"
        }
    ));
    out.push('\n');

    // Evidence section
    out.push_str("### Evidence\n\n");
    render_evidence_table(&mut out, &receipt.evidence);

    out.push_str("#### Mutation Testing\n\n");
    render_mutation_gate_markdown(&mut out, &receipt.evidence.mutation);

    // Complexity gate section
    if let Some(ref gate) = receipt.evidence.complexity {
        out.push_str("#### Complexity Analysis\n\n");
        render_complexity_gate_markdown(&mut out, gate);
    }

    // Risk assessment
    if !receipt.risk.hotspots_touched.is_empty() || !receipt.risk.bus_factor_warnings.is_empty() {
        out.push_str("### Risk Assessment\n\n");
        if !receipt.risk.hotspots_touched.is_empty() {
            out.push_str("**Hotspots touched:**\n");
            for hotspot in &receipt.risk.hotspots_touched {
                out.push_str(&format!("- `{}`\n", hotspot));
            }
            out.push('\n');
        }
        if !receipt.risk.bus_factor_warnings.is_empty() {
            out.push_str("**Bus factor warnings:**\n");
            for warning in &receipt.risk.bus_factor_warnings {
                out.push_str(&format!("- {}\n", warning));
            }
            out.push('\n');
        }
    }

    // Review Plan
    out.push_str("### Review Plan\n\n");
    out.push_str("| Priority | File | Reason | Complexity | Lines |\n");
    out.push_str("|:--------:|------|--------|:----------:|------:|\n");
    for item in &receipt.review_plan {
        let complexity_stars = match item.complexity.unwrap_or(1) {
            1 => "*",
            2 => "**",
            3 => "***",
            4 => "****",
            _ => "*****",
        };
        out.push_str(&format!(
            "| {} | `{}` | {} | {} | {} |\n",
            item.priority,
            item.path,
            item.reason,
            complexity_stars,
            item.lines_changed.unwrap_or(0)
        ));
    }
    out.push('\n');

    out
}

/// Format gate status as string.
fn format_gate_status(status: GateStatus) -> &'static str {
    match status {
        GateStatus::Pass => "PASS",
        GateStatus::Fail => "FAIL",
        GateStatus::Skipped => "SKIPPED",
        GateStatus::Pending => "PENDING",
    }
}

/// Format evidence source as string.
fn format_source(source: EvidenceSource) -> &'static str {
    match source {
        EvidenceSource::CiArtifact => "CI artifact",
        EvidenceSource::Cached => "cached",
        EvidenceSource::RanLocal => "local",
    }
}

/// Format commit match as string.
fn format_commit_match(cm: CommitMatch) -> &'static str {
    match cm {
        CommitMatch::Exact => "exact",
        CommitMatch::Partial => "partial",
        CommitMatch::Stale => "stale",
        CommitMatch::Unknown => "-",
    }
}

/// Format scope coverage as string (e.g., "5/5 (100%)").
fn format_scope(scope: &ScopeCoverage) -> String {
    let tested = scope.tested.len();
    let relevant = scope.relevant.len();
    let pct = (scope.ratio * 100.0).round() as usize;
    if relevant == 0 {
        "-".to_string()
    } else {
        format!("{}/{} ({}%)", tested, relevant, pct)
    }
}

/// Render evidence summary table to markdown.
fn render_evidence_table(out: &mut String, evidence: &Evidence) {
    out.push_str("| Gate | Status | Source | Scope | Commit |\n");
    out.push_str("|------|--------|--------|-------|--------|\n");

    // Mutation gate (always present)
    out.push_str(&format!(
        "| Mutation | {} | {} | {} | {} |\n",
        format_gate_status(evidence.mutation.meta.status),
        format_source(evidence.mutation.meta.source),
        format_scope(&evidence.mutation.meta.scope),
        format_commit_match(evidence.mutation.meta.commit_match)
    ));

    // Diff Coverage gate
    if let Some(ref gate) = evidence.diff_coverage {
        out.push_str(&format!(
            "| Diff Coverage | {} | {} | {} | {} |\n",
            format_gate_status(gate.meta.status),
            format_source(gate.meta.source),
            format_scope(&gate.meta.scope),
            format_commit_match(gate.meta.commit_match)
        ));
    } else {
        out.push_str("| Diff Coverage | - | - | - | - |\n");
    }

    // Contracts gate
    if let Some(ref gate) = evidence.contracts {
        out.push_str(&format!(
            "| Contracts | {} | {} | {} | {} |\n",
            format_gate_status(gate.meta.status),
            format_source(gate.meta.source),
            format_scope(&gate.meta.scope),
            format_commit_match(gate.meta.commit_match)
        ));
    } else {
        out.push_str("| Contracts | - | - | - | - |\n");
    }

    // Supply Chain gate
    if let Some(ref gate) = evidence.supply_chain {
        out.push_str(&format!(
            "| Supply Chain | {} | {} | {} | {} |\n",
            format_gate_status(gate.meta.status),
            format_source(gate.meta.source),
            format_scope(&gate.meta.scope),
            format_commit_match(gate.meta.commit_match)
        ));
    } else {
        out.push_str("| Supply Chain | - | - | - | - |\n");
    }

    // Determinism gate
    if let Some(ref gate) = evidence.determinism {
        out.push_str(&format!(
            "| Determinism | {} | {} | {} | {} |\n",
            format_gate_status(gate.meta.status),
            format_source(gate.meta.source),
            format_scope(&gate.meta.scope),
            format_commit_match(gate.meta.commit_match)
        ));
    } else {
        out.push_str("| Determinism | - | - | - | - |\n");
    }

    // Complexity gate
    if let Some(ref gate) = evidence.complexity {
        out.push_str(&format!(
            "| Complexity | {} | {} | {} | {} |\n",
            format_gate_status(gate.meta.status),
            format_source(gate.meta.source),
            format_scope(&gate.meta.scope),
            format_commit_match(gate.meta.commit_match)
        ));
    } else {
        out.push_str("| Complexity | - | - | - | - |\n");
    }

    out.push_str(&format!(
        "\n**Overall:** {}\n\n",
        format_gate_status(evidence.overall_status)
    ));
}

/// Render mutation gate status to markdown.
fn render_mutation_gate_markdown(out: &mut String, gate: &MutationGate) {
    let status_icon = match gate.meta.status {
        GateStatus::Pass => "PASS",
        GateStatus::Fail => "FAIL",
        GateStatus::Skipped => "SKIPPED",
        GateStatus::Pending => "PENDING",
    };

    let source_label = match gate.meta.source {
        EvidenceSource::Cached => "cached",
        EvidenceSource::RanLocal => "ran",
        EvidenceSource::CiArtifact => "CI artifact",
    };

    out.push_str(&format!(
        "**Status:** {} (source: {})\n\n",
        status_icon, source_label
    ));

    match gate.meta.status {
        GateStatus::Pass => {
            out.push_str(&format!(
                "0 survivors | {} killed | {} timeout | {} unviable\n\n",
                gate.killed, gate.timeout, gate.unviable
            ));
            if !gate.meta.scope.tested.is_empty() {
                out.push_str(&format!(
                    "**Scope:** {} file(s) tested\n\n",
                    gate.meta.scope.tested.len()
                ));
            }
        }
        GateStatus::Fail => {
            out.push_str(&format!(
                "{} survivors | {} killed | {} timeout | {} unviable\n\n",
                gate.survivors.len(),
                gate.killed,
                gate.timeout,
                gate.unviable
            ));
            out.push_str("**Survivors:**\n\n");
            for survivor in &gate.survivors {
                out.push_str(&format!(
                    "- `{}:{}` - {}\n",
                    survivor.file, survivor.line, survivor.mutation
                ));
            }
            out.push('\n');
        }
        GateStatus::Skipped => {
            out.push_str("No relevant Rust source files in diff.\n\n");
        }
        GateStatus::Pending => {
            out.push_str(
                "Mutation testing results not available. Install `cargo-mutants` to enable.\n\n",
            );
            if !gate.meta.scope.relevant.is_empty() {
                out.push_str(&format!(
                    "**Pending scope:** {} file(s)\n\n",
                    gate.meta.scope.relevant.len()
                ));
            }
        }
    }
}

/// Render complexity gate status to markdown.
fn render_complexity_gate_markdown(out: &mut String, gate: &ComplexityGate) {
    let status_icon = match gate.meta.status {
        GateStatus::Pass => "PASS",
        GateStatus::Fail => "FAIL",
        GateStatus::Skipped => "SKIPPED",
        GateStatus::Pending => "WARN", // Pending is used as warning for complexity
    };

    out.push_str(&format!(
        "**Status:** {} | Threshold: CC > {}\n\n",
        status_icon, COMPLEXITY_THRESHOLD
    ));

    out.push_str(&format!(
        "**Files analyzed:** {} | **Avg CC:** {:.1} | **Max CC:** {}\n\n",
        gate.files_analyzed, gate.avg_cyclomatic, gate.max_cyclomatic
    ));

    if !gate.high_complexity_files.is_empty() {
        out.push_str("**High Complexity Files:**\n\n");
        out.push_str("| File | CC | Functions | Max Length |\n");
        out.push_str("|------|---:|----------:|-----------:|\n");
        for file in &gate.high_complexity_files {
            out.push_str(&format!(
                "| `{}` | {} | {} | {} lines |\n",
                file.path, file.cyclomatic, file.function_count, file.max_function_length
            ));
        }
        out.push('\n');
    } else {
        out.push_str("No files exceed the complexity threshold.\n\n");
    }
}

fn render_sections(receipt: &CockpitReceipt) -> String {
    let mut out = String::new();

    // COCKPIT section (for AI-FILL:COCKPIT)
    out.push_str("<!-- SECTION:COCKPIT -->\n");
    out.push_str("| Metric | Value |\n");
    out.push_str("|--------|-------|\n");
    out.push_str(&format!(
        "| **Health** | {} ({}/100) |\n",
        receipt.code_health.grade, receipt.code_health.score
    ));
    out.push_str(&format!(
        "| **Risk** | {:?} ({}/100) |\n",
        receipt.risk.level, receipt.risk.score
    ));
    out.push_str("| **Change Surface** | |\n");
    out.push_str(&format!(
        "| Commits | {} |\n",
        receipt.change_surface.commits
    ));
    out.push_str(&format!(
        "| Files changed | {} |\n",
        receipt.change_surface.files_changed
    ));
    out.push_str(&format!(
        "| Lines (+/-) | +{}/-{} |\n",
        receipt.change_surface.insertions, receipt.change_surface.deletions
    ));
    out.push_str(&format!(
        "| Net lines | {} |\n",
        receipt.change_surface.net_lines
    ));
    out.push_str(&format!(
        "| Churn velocity | {:.1} lines/commit |\n",
        receipt.change_surface.churn_velocity
    ));
    out.push_str("| **Composition** | |\n");
    out.push_str(&format!(
        "| Code | {:.1}% |\n",
        receipt.composition.code_pct
    ));
    out.push_str(&format!(
        "| Tests | {:.1}% |\n",
        receipt.composition.test_pct
    ));
    out.push_str(&format!(
        "| Docs | {:.1}% |\n",
        receipt.composition.docs_pct
    ));
    out.push_str(&format!(
        "| Config | {:.1}% |\n",
        receipt.composition.config_pct
    ));
    out.push_str(&format!(
        "| Test ratio | {:.2} |\n",
        receipt.composition.test_ratio
    ));
    out.push_str("| **Code Health** | |\n");
    out.push_str(&format!(
        "| Complexity | {:?} |\n",
        receipt.code_health.complexity_indicator
    ));
    out.push_str(&format!(
        "| Large files | {} |\n",
        receipt.code_health.large_files_touched
    ));
    out.push_str("| **Contracts** | |\n");
    out.push_str(&format!(
        "| API changed | {} |\n",
        if receipt.contracts.api_changed {
            "Yes"
        } else {
            "No"
        }
    ));
    out.push_str(&format!(
        "| CLI changed | {} |\n",
        if receipt.contracts.cli_changed {
            "Yes"
        } else {
            "No"
        }
    ));
    out.push_str(&format!(
        "| Schema changed | {} |\n",
        if receipt.contracts.schema_changed {
            "Yes"
        } else {
            "No"
        }
    ));
    out.push_str("| **Evidence** | |\n");
    render_mutation_gate_sections(&mut out, &receipt.evidence.mutation);
    out.push_str("<!-- /SECTION:COCKPIT -->\n\n");

    // REVIEW_PLAN section (for AI-FILL:REVIEW_PLAN)
    out.push_str("<!-- SECTION:REVIEW_PLAN -->\n");
    out.push_str("| Priority | File | Reason | Complexity |\n");
    out.push_str("|----------|------|--------|:----------:|\n");
    for item in &receipt.review_plan {
        let complexity_stars = match item.complexity.unwrap_or(1) {
            1 => "*",
            2 => "**",
            3 => "***",
            4 => "****",
            _ => "*****",
        };
        out.push_str(&format!(
            "| {} | `{}` | {} | {} |\n",
            item.priority, item.path, item.reason, complexity_stars
        ));
    }
    out.push_str("<!-- /SECTION:REVIEW_PLAN -->\n\n");

    // COMPLEXITY section (for AI-FILL:COMPLEXITY)
    out.push_str("<!-- SECTION:COMPLEXITY -->\n");
    if let Some(ref gate) = receipt.evidence.complexity {
        render_complexity_gate_sections(&mut out, gate);
    } else {
        out.push_str("No complexity analysis available.\n");
    }
    out.push_str("<!-- /SECTION:COMPLEXITY -->\n\n");

    // RECEIPTS section (full JSON)
    out.push_str("<!-- SECTION:RECEIPTS -->\n");
    out.push_str("```json\n");
    if let Ok(json) = serde_json::to_string_pretty(receipt) {
        out.push_str(&json);
    }
    out.push_str("\n```\n");
    out.push_str("<!-- /SECTION:RECEIPTS -->\n");

    out
}

/// Render mutation gate status for sections format.
fn render_mutation_gate_sections(out: &mut String, gate: &MutationGate) {
    let status_str = match gate.meta.status {
        GateStatus::Pass => "Pass",
        GateStatus::Fail => "Fail",
        GateStatus::Skipped => "Skipped",
        GateStatus::Pending => "Pending",
    };

    out.push_str(&format!("| Mutation gate | {} |\n", status_str));

    match gate.meta.status {
        GateStatus::Pass => {
            out.push_str(&format!("| Mutations killed | {} |\n", gate.killed));
            out.push_str("| Survivors | 0 |\n");
        }
        GateStatus::Fail => {
            out.push_str(&format!("| Mutations killed | {} |\n", gate.killed));
            out.push_str(&format!("| Survivors | {} |\n", gate.survivors.len()));
        }
        GateStatus::Skipped | GateStatus::Pending => {
            // No additional rows for skipped/pending
        }
    }
}

/// Render complexity gate status for sections format.
fn render_complexity_gate_sections(out: &mut String, gate: &ComplexityGate) {
    let status_str = match gate.meta.status {
        GateStatus::Pass => "Pass",
        GateStatus::Fail => "Fail",
        GateStatus::Skipped => "Skipped",
        GateStatus::Pending => "Warn",
    };

    out.push_str(&format!(
        "| Complexity gate | {} (threshold: CC > {}) |\n",
        status_str, COMPLEXITY_THRESHOLD
    ));
    out.push_str(&format!("| Files analyzed | {} |\n", gate.files_analyzed));
    out.push_str(&format!(
        "| Avg cyclomatic | {:.1} |\n",
        gate.avg_cyclomatic
    ));
    out.push_str(&format!("| Max cyclomatic | {} |\n", gate.max_cyclomatic));
    out.push_str(&format!(
        "| High complexity files | {} |\n",
        gate.high_complexity_files.len()
    ));

    if !gate.high_complexity_files.is_empty() {
        out.push_str("\n**High Complexity Files:**\n\n");
        out.push_str("| File | CC | Functions | Max Length |\n");
        out.push_str("|------|---:|----------:|-----------:|\n");
        for file in &gate.high_complexity_files {
            out.push_str(&format!(
                "| `{}` | {} | {} | {} |\n",
                file.path, file.cyclomatic, file.function_count, file.max_function_length
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_file_code() {
        assert_eq!(classify_file("src/lib.rs"), FileCategory::Code);
        assert_eq!(classify_file("crates/foo/src/main.rs"), FileCategory::Code);
    }

    #[test]
    fn test_classify_file_test() {
        assert_eq!(classify_file("tests/integration.rs"), FileCategory::Test);
        assert_eq!(classify_file("src/foo_test.rs"), FileCategory::Test);
        assert_eq!(classify_file("crates/bar/tests/it.rs"), FileCategory::Test);
    }

    #[test]
    fn test_classify_file_docs() {
        assert_eq!(classify_file("README.md"), FileCategory::Docs);
        assert_eq!(classify_file("docs/guide.md"), FileCategory::Docs);
        assert_eq!(classify_file("CHANGELOG.md"), FileCategory::Docs);
    }

    #[test]
    fn test_classify_file_config() {
        assert_eq!(classify_file("Cargo.toml"), FileCategory::Config);
        assert_eq!(
            classify_file(".github/workflows/ci.yml"),
            FileCategory::Config
        );
        assert_eq!(classify_file("Justfile"), FileCategory::Config);
    }

    #[test]
    fn test_compute_composition() {
        let files = vec![
            "src/lib.rs".to_string(),
            "src/main.rs".to_string(),
            "tests/test.rs".to_string(),
            "README.md".to_string(),
            "Cargo.toml".to_string(),
        ];
        let comp = compute_composition(&files);
        assert_eq!(comp.code_pct, 40.0); // 2/5
        assert_eq!(comp.test_pct, 20.0); // 1/5
        assert_eq!(comp.docs_pct, 20.0); // 1/5
        assert_eq!(comp.config_pct, 20.0); // 1/5
        assert_eq!(comp.test_ratio, 0.5); // 1 test / 2 code
    }

    #[test]
    fn test_compute_change_concentration() {
        // 5 files with changes: 100, 50, 30, 15, 5 = 200 total
        // Top 20% (1 file) = 100, which is 50% of total
        let file_stats = vec![
            FileStats {
                path: "big.rs".to_string(),
                insertions: 80,
                deletions: 20,
            },
            FileStats {
                path: "medium.rs".to_string(),
                insertions: 40,
                deletions: 10,
            },
            FileStats {
                path: "small1.rs".to_string(),
                insertions: 25,
                deletions: 5,
            },
            FileStats {
                path: "small2.rs".to_string(),
                insertions: 10,
                deletions: 5,
            },
            FileStats {
                path: "tiny.rs".to_string(),
                insertions: 4,
                deletions: 1,
            },
        ];
        let concentration = compute_change_concentration(&file_stats);
        assert_eq!(concentration, 50.0); // 100/200 = 50%
    }

    #[test]
    fn test_compute_code_health_score() {
        let file_stats = vec![FileStats {
            path: "normal.rs".to_string(),
            insertions: 50,
            deletions: 10,
        }];
        let contracts = Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        };
        let health = compute_code_health(&file_stats, &contracts);
        assert!(health.score >= 80, "Simple change should have high health");
        assert_eq!(health.grade, "A");
    }

    #[test]
    fn test_risk_level_computation() {
        let file_stats = vec![FileStats {
            path: "small.rs".to_string(),
            insertions: 10,
            deletions: 5,
        }];
        let contracts = Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        };
        let health = compute_code_health(&file_stats, &contracts);
        let risk = compute_risk(&file_stats, &contracts, &health);
        assert_eq!(risk.level, RiskLevel::Low);
    }

    #[test]
    fn test_detect_contracts_api() {
        let files = vec!["crates/tokmd-types/src/lib.rs".to_string()];
        let contracts = detect_contracts(&files);
        assert!(contracts.api_changed);
        assert!(contracts.schema_changed);
    }

    #[test]
    fn test_detect_contracts_cli() {
        let files = vec!["crates/tokmd/src/commands/cockpit.rs".to_string()];
        let contracts = detect_contracts(&files);
        assert!(contracts.cli_changed);
    }

    #[test]
    fn test_detect_contracts_schema() {
        let files = vec!["docs/schema.json".to_string()];
        let contracts = detect_contracts(&files);
        assert!(contracts.schema_changed);
    }

    #[test]
    fn test_is_relevant_rust_source() {
        // Should include
        assert!(is_relevant_rust_source("src/lib.rs"));
        assert!(is_relevant_rust_source(
            "crates/tokmd/src/commands/cockpit.rs"
        ));
        assert!(is_relevant_rust_source("src/main.rs"));

        // Should exclude - test directories
        assert!(!is_relevant_rust_source("tests/integration.rs"));
        assert!(!is_relevant_rust_source("crates/foo/tests/test.rs"));

        // Should exclude - test files
        assert!(!is_relevant_rust_source("src/foo_test.rs"));
        assert!(!is_relevant_rust_source("src/bar_tests.rs"));

        // Should exclude - fuzz
        assert!(!is_relevant_rust_source("fuzz/target.rs"));
        assert!(!is_relevant_rust_source("crates/foo/fuzz/harness.rs"));

        // Should exclude - non-Rust
        assert!(!is_relevant_rust_source("src/lib.py"));
        assert!(!is_relevant_rust_source("Cargo.toml"));
    }

    #[test]
    fn test_mutation_gate_status_serialization() {
        let gate = MutationGate {
            meta: GateMeta {
                status: GateStatus::Pass,
                source: EvidenceSource::Cached,
                commit_match: CommitMatch::Exact,
                scope: ScopeCoverage {
                    relevant: vec!["src/lib.rs".to_string()],
                    tested: vec!["src/lib.rs".to_string()],
                    ratio: 1.0,
                    lines_relevant: None,
                    lines_tested: None,
                },
                evidence_commit: Some("abc123".to_string()),
                evidence_generated_at_ms: None,
            },
            survivors: Vec::new(),
            killed: 10,
            timeout: 2,
            unviable: 1,
        };

        let json = serde_json::to_string(&gate).unwrap();
        assert!(json.contains("\"status\":\"pass\""));
        assert!(json.contains("\"source\":\"cached\""));

        let deserialized: MutationGate = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.meta.status, GateStatus::Pass);
        assert_eq!(deserialized.meta.source, EvidenceSource::Cached);
    }

    #[test]
    fn test_mutation_gate_with_survivors() {
        let gate = MutationGate {
            meta: GateMeta {
                status: GateStatus::Fail,
                source: EvidenceSource::RanLocal,
                commit_match: CommitMatch::Exact,
                scope: ScopeCoverage {
                    relevant: vec!["src/lib.rs".to_string()],
                    tested: vec!["src/lib.rs".to_string()],
                    ratio: 1.0,
                    lines_relevant: None,
                    lines_tested: None,
                },
                evidence_commit: None,
                evidence_generated_at_ms: None,
            },
            survivors: vec![MutationSurvivor {
                file: "src/lib.rs".to_string(),
                line: 42,
                mutation: "replace foo -> bool with true".to_string(),
            }],
            killed: 5,
            timeout: 0,
            unviable: 0,
        };

        assert_eq!(gate.meta.status, GateStatus::Fail);
        assert_eq!(gate.survivors.len(), 1);
        assert_eq!(gate.survivors[0].line, 42);
    }

    #[test]
    fn test_overall_status_computation() {
        // Test all pass
        let mutation_pass = MutationGate {
            meta: GateMeta {
                status: GateStatus::Pass,
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
        };

        let overall = compute_overall_status(&mutation_pass, &None, &None, &None, &None, &None);
        assert_eq!(overall, GateStatus::Pass);

        // Test fail
        let mutation_fail = MutationGate {
            meta: GateMeta {
                status: GateStatus::Fail,
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
        };

        let overall = compute_overall_status(&mutation_fail, &None, &None, &None, &None, &None);
        assert_eq!(overall, GateStatus::Fail);
    }

    #[test]
    fn test_evidence_source_serialization() {
        // Test snake_case serialization
        let source = EvidenceSource::CiArtifact;
        let json = serde_json::to_string(&source).unwrap();
        assert_eq!(json, "\"ci_artifact\"");

        let source = EvidenceSource::RanLocal;
        let json = serde_json::to_string(&source).unwrap();
        assert_eq!(json, "\"ran_local\"");
    }

    #[test]
    fn test_analyze_rust_complexity_simple() {
        let code = r#"
fn simple() {
    println!("hello");
}
"#;
        let analysis = analyze_rust_complexity(code);
        assert_eq!(analysis.function_count, 1);
        assert_eq!(analysis.max_complexity, 1);
    }

    #[test]
    fn test_analyze_rust_complexity_with_branches() {
        let code = r#"
fn with_branches(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            x * 2
        } else {
            x + 1
        }
    } else {
        0
    }
}
"#;
        let analysis = analyze_rust_complexity(code);
        assert_eq!(analysis.function_count, 1);
        // 1 (base) + 2 (if statements) = 3
        assert!(analysis.max_complexity >= 3);
    }

    #[test]
    fn test_analyze_rust_complexity_with_match() {
        let code = r#"
fn with_match(x: Option<i32>) -> i32 {
    match x {
        Some(v) => v,
        None => 0,
    }
}
"#;
        let analysis = analyze_rust_complexity(code);
        assert_eq!(analysis.function_count, 1);
        // 1 (base) + 1 (match) + 2 (match arms) = 4
        assert!(analysis.max_complexity >= 3);
    }

    #[test]
    fn test_analyze_rust_complexity_with_loops() {
        let code = r#"
fn with_loops() {
    for i in 0..10 {
        while i > 5 {
            break;
        }
    }
}
"#;
        let analysis = analyze_rust_complexity(code);
        assert_eq!(analysis.function_count, 1);
        // 1 (base) + 1 (for) + 1 (while) = 3
        assert!(analysis.max_complexity >= 3);
    }

    #[test]
    fn test_analyze_rust_complexity_with_logical_ops() {
        let code = r#"
fn with_logical(a: bool, b: bool, c: bool) -> bool {
    if a && b || c {
        true
    } else {
        false
    }
}
"#;
        let analysis = analyze_rust_complexity(code);
        assert_eq!(analysis.function_count, 1);
        // 1 (base) + 1 (if) + 1 (&&) + 1 (||) = 4
        assert!(analysis.max_complexity >= 4);
    }

    #[test]
    fn test_analyze_rust_complexity_multiple_functions() {
        let code = r#"
fn first() {
    println!("first");
}

fn second(x: i32) -> i32 {
    if x > 0 { x } else { -x }
}

fn third() {
    for i in 0..5 {
        println!("{}", i);
    }
}
"#;
        let analysis = analyze_rust_complexity(code);
        assert_eq!(analysis.function_count, 3);
        assert!(analysis.total_complexity >= 3);
    }

    #[test]
    fn test_complexity_gate_status_pass() {
        let gate = ComplexityGate {
            meta: GateMeta {
                status: GateStatus::Pass,
                source: EvidenceSource::RanLocal,
                commit_match: CommitMatch::Exact,
                scope: ScopeCoverage {
                    relevant: vec!["src/lib.rs".to_string()],
                    tested: vec!["src/lib.rs".to_string()],
                    ratio: 1.0,
                    lines_relevant: None,
                    lines_tested: None,
                },
                evidence_commit: None,
                evidence_generated_at_ms: None,
            },
            files_analyzed: 1,
            high_complexity_files: Vec::new(),
            avg_cyclomatic: 5.0,
            max_cyclomatic: 10,
            threshold_exceeded: false,
        };

        assert_eq!(gate.meta.status, GateStatus::Pass);
        assert!(gate.high_complexity_files.is_empty());
    }

    #[test]
    fn test_complexity_gate_serialization() {
        let gate = ComplexityGate {
            meta: GateMeta {
                status: GateStatus::Pending,
                source: EvidenceSource::RanLocal,
                commit_match: CommitMatch::Exact,
                scope: ScopeCoverage {
                    relevant: vec!["src/lib.rs".to_string()],
                    tested: vec!["src/lib.rs".to_string()],
                    ratio: 1.0,
                    lines_relevant: None,
                    lines_tested: None,
                },
                evidence_commit: None,
                evidence_generated_at_ms: None,
            },
            files_analyzed: 2,
            high_complexity_files: vec![HighComplexityFile {
                path: "src/complex.rs".to_string(),
                cyclomatic: 20,
                function_count: 5,
                max_function_length: 100,
            }],
            avg_cyclomatic: 12.5,
            max_cyclomatic: 20,
            threshold_exceeded: true,
        };

        let json = serde_json::to_string(&gate).unwrap();
        assert!(json.contains("\"status\":\"pending\""));
        assert!(json.contains("\"files_analyzed\":2"));
        assert!(json.contains("\"avg_cyclomatic\":12.5"));
        assert!(json.contains("\"threshold_exceeded\":true"));

        let deserialized: ComplexityGate = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.files_analyzed, 2);
        assert_eq!(deserialized.high_complexity_files.len(), 1);
        assert_eq!(deserialized.high_complexity_files[0].cyclomatic, 20);
    }
}
