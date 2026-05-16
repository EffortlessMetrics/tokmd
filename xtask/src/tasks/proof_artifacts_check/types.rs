use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub(super) enum VerificationMode {
    NoExecution,
    Execution,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct ProofArtifactsReport {
    pub(super) selected: usize,
    pub(super) executed: usize,
    pub(super) execution_status: String,
    pub(super) guard_reason: String,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct ProofRunArtifactsReport {
    pub(super) executed: usize,
    pub(super) guard_reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(super) struct ProofRunObservation {
    pub(super) schema: String,
    pub(super) status: String,
    pub(super) execution_status: String,
    pub(super) profile: String,
    pub(super) base: String,
    pub(super) head: String,
    pub(super) ok: bool,
    pub(super) execution_guard: ProofRunObservationGuard,
    pub(super) counts: ProofRunObservationCounts,
    pub(super) scopes: Vec<ProofRunObservationScope>,
    pub(super) changed_files: Vec<String>,
    pub(super) unknown_files: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(super) struct ProofRunObservationGuard {
    pub(super) enabled: bool,
    pub(super) ci: bool,
    pub(super) reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(super) struct ProofRunObservationCounts {
    pub(super) commands_total: usize,
    pub(super) required_planned: usize,
    pub(super) advisory_skipped: usize,
    pub(super) executed: usize,
    pub(super) passed: usize,
    pub(super) failed: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(super) struct ProofRunObservationScope {
    pub(super) name: String,
    pub(super) kind: String,
    pub(super) command: String,
    pub(super) status: String,
    pub(super) exit_code: Option<i64>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct ProofRunObservationCollection {
    pub(super) schema: String,
    pub(super) ok: bool,
    pub(super) counts: ProofRunObservationCollectionCounts,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) window: Option<ProofRunObservationWindow>,
    pub(super) profiles: Vec<ProofRunObservationProfileSummary>,
    pub(super) scopes: Vec<ProofRunObservationScopeSummary>,
    pub(super) guards: Vec<ProofRunObservationGuardSummary>,
    pub(super) sources: Vec<ProofRunObservationSourceSummary>,
}

#[derive(Debug, Default, Serialize, PartialEq, Eq)]
pub(super) struct ProofRunObservationCollectionCounts {
    pub(super) observations: usize,
    pub(super) commands_total: usize,
    pub(super) required_planned: usize,
    pub(super) advisory_skipped: usize,
    pub(super) executed: usize,
    pub(super) passed: usize,
    pub(super) failed: usize,
    pub(super) unknown_files: usize,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct ProofRunObservationWindow {
    pub(super) source: String,
    pub(super) expected_runs: usize,
    pub(super) observed_runs: usize,
    pub(super) missing_runs: usize,
    pub(super) unmatched_observations: usize,
    pub(super) missing: Vec<ProofExecutorSourceRun>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct ProofRunObservationProfileSummary {
    pub(super) profile: String,
    pub(super) observations: usize,
    pub(super) required_planned: usize,
    pub(super) executed: usize,
    pub(super) passed: usize,
    pub(super) failed: usize,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct ProofRunObservationScopeSummary {
    pub(super) name: String,
    pub(super) kind: String,
    pub(super) observations: usize,
    pub(super) executed: usize,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct ProofRunObservationGuardSummary {
    pub(super) reason: String,
    pub(super) observations: usize,
    pub(super) ci_observations: usize,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct ProofRunObservationSourceSummary {
    pub(super) path: String,
    pub(super) status: String,
    pub(super) execution_status: String,
    pub(super) profile: String,
    pub(super) base: String,
    pub(super) head: String,
    pub(super) guard_reason: String,
    pub(super) commands_total: usize,
    pub(super) required_planned: usize,
    pub(super) executed: usize,
    pub(super) passed: usize,
    pub(super) failed: usize,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ExecutionStateContext<'a> {
    pub(super) execution_status: &'a str,
    pub(super) guard_enabled: bool,
    pub(super) selected: usize,
    pub(super) executed: usize,
    pub(super) artifact_root: Option<&'a Path>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(super) struct ProofExecutionObservation {
    pub(super) schema: String,
    pub(super) status: String,
    pub(super) execution_status: String,
    pub(super) profile: String,
    pub(super) base: String,
    pub(super) head: String,
    pub(super) family: String,
    pub(super) required: bool,
    pub(super) ok: bool,
    pub(super) execution_guard: ProofExecutionObservationGuard,
    pub(super) counts: ProofExecutionObservationCounts,
    pub(super) scopes: Vec<ProofExecutionObservationScope>,
    pub(super) changed_files: Vec<String>,
    pub(super) unknown_files: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(super) struct ProofExecutionObservationGuard {
    pub(super) enabled: bool,
    pub(super) ci: bool,
    pub(super) reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(super) struct ProofExecutionObservationCounts {
    pub(super) selected: usize,
    pub(super) executed: usize,
    pub(super) passed: usize,
    pub(super) failed: usize,
    pub(super) artifacts: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(super) struct ProofExecutionObservationScope {
    pub(super) name: String,
    pub(super) kind: String,
    pub(super) command: String,
    pub(super) artifact_path: Option<String>,
    pub(super) status: String,
    pub(super) exit_code: Option<i64>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct ProofExecutionObservationCollection {
    pub(super) schema: String,
    pub(super) ok: bool,
    pub(super) counts: ProofExecutionObservationCollectionCounts,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) window: Option<ProofExecutionObservationWindow>,
    pub(super) families: Vec<ProofExecutionObservationFamilySummary>,
    pub(super) scopes: Vec<ProofExecutionObservationScopeSummary>,
    pub(super) sources: Vec<ProofExecutionObservationSourceSummary>,
}

#[derive(Debug, Default, Serialize, PartialEq, Eq)]
pub(super) struct ProofExecutionObservationCollectionCounts {
    pub(super) observations: usize,
    pub(super) selected: usize,
    pub(super) executed: usize,
    pub(super) passed: usize,
    pub(super) failed: usize,
    pub(super) artifacts: usize,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct ProofExecutionObservationWindow {
    pub(super) source: String,
    pub(super) expected_runs: usize,
    pub(super) observed_runs: usize,
    pub(super) missing_runs: usize,
    pub(super) unmatched_observations: usize,
    pub(super) missing: Vec<ProofExecutorSourceRun>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct ProofExecutionObservationFamilySummary {
    pub(super) family: String,
    pub(super) observations: usize,
    pub(super) selected: usize,
    pub(super) executed: usize,
    pub(super) passed: usize,
    pub(super) artifacts: usize,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct ProofExecutionObservationScopeSummary {
    pub(super) name: String,
    pub(super) kind: String,
    pub(super) family: String,
    pub(super) observations: usize,
    pub(super) executed: usize,
    pub(super) artifacts: usize,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct ProofExecutionObservationSourceSummary {
    pub(super) path: String,
    pub(super) status: String,
    pub(super) execution_status: String,
    pub(super) profile: String,
    pub(super) base: String,
    pub(super) head: String,
    pub(super) family: String,
    pub(super) guard_reason: String,
    pub(super) selected: usize,
    pub(super) executed: usize,
    pub(super) passed: usize,
    pub(super) artifacts: usize,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct ProofExecutorPromotionReadiness {
    pub(super) schema: String,
    pub(super) ok: bool,
    pub(super) thresholds: ProofExecutorPromotionReadinessThresholds,
    pub(super) actuals: ProofExecutorPromotionReadinessActuals,
    pub(super) collector_runs: Vec<ProofExecutorPromotionCollectorRun>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct ProofExecutorPromotionReadinessThresholds {
    pub(super) min_observations: usize,
    pub(super) min_executed: usize,
    pub(super) min_scopes: usize,
    pub(super) min_artifacts: usize,
    pub(super) min_passing_collector_runs: usize,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct ProofExecutorPromotionReadinessActuals {
    pub(super) observations: usize,
    pub(super) executed: usize,
    pub(super) scopes: usize,
    pub(super) artifacts: usize,
    pub(super) passing_collector_runs: usize,
}

#[derive(Debug, Deserialize)]
pub(super) struct GithubRun {
    #[serde(rename = "databaseId")]
    pub(super) database_id: u64,

    #[serde(default)]
    pub(super) event: Option<String>,

    #[serde(rename = "headBranch", default)]
    pub(super) head_branch: Option<String>,

    #[serde(rename = "headSha", default)]
    pub(super) head_sha: Option<String>,

    #[serde(rename = "createdAt", default)]
    pub(super) created_at: Option<String>,

    #[serde(default)]
    pub(super) url: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(super) struct ProofExecutorSourceRun {
    pub(super) database_id: u64,
    pub(super) event: Option<String>,
    pub(super) head_branch: Option<String>,
    pub(super) head_sha: Option<String>,
    pub(super) created_at: Option<String>,
    pub(super) url: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct ProofExecutorPromotionCollectorRun {
    pub(super) database_id: u64,
    pub(super) event: Option<String>,
    pub(super) head_branch: Option<String>,
    pub(super) head_sha: Option<String>,
    pub(super) created_at: Option<String>,
    pub(super) url: Option<String>,
}

#[derive(Debug)]
pub(super) struct SourcedProofRunObservation {
    pub(super) path: PathBuf,
    pub(super) observation: ProofRunObservation,
}

#[derive(Debug)]
pub(super) struct SourcedProofExecutionObservation {
    pub(super) path: PathBuf,
    pub(super) observation: ProofExecutionObservation,
}

#[derive(Default)]
pub(super) struct ProofRunProfileAccumulator {
    pub(super) observations: usize,
    pub(super) required_planned: usize,
    pub(super) executed: usize,
    pub(super) passed: usize,
    pub(super) failed: usize,
}

#[derive(Default)]
pub(super) struct ProofRunScopeAccumulator {
    pub(super) observations: usize,
    pub(super) executed: usize,
}

#[derive(Default)]
pub(super) struct ProofRunGuardAccumulator {
    pub(super) observations: usize,
    pub(super) ci_observations: usize,
}

#[derive(Default)]
pub(super) struct FamilyAccumulator {
    pub(super) observations: usize,
    pub(super) selected: usize,
    pub(super) executed: usize,
    pub(super) passed: usize,
    pub(super) artifacts: usize,
}

#[derive(Default)]
pub(super) struct ScopeAccumulator {
    pub(super) observations: usize,
    pub(super) executed: usize,
    pub(super) artifacts: usize,
}
