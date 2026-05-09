//! Deserializable proof-control-plane evidence input shapes.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ProofRunSummaryInput {
    pub schema: String,
    pub status: String,
    pub execution_status: String,
    pub execution_guard: ProofRunExecutionGuardInput,
    pub profile: String,
    pub base: String,
    pub head: String,
    pub ok: bool,
    #[serde(default)]
    pub changed_files: Vec<String>,
    pub counts: ProofRunCountsInput,
    #[serde(default)]
    pub entries: Vec<ProofRunEntryInput>,
    #[serde(default)]
    pub unknown_files: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ProofRunExecutionGuardInput {
    pub required: bool,
    pub enabled: bool,
    pub ci: bool,
    pub allow_ci_required_execution: bool,
    pub allow_local_required_execution: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ProofRunCountsInput {
    pub commands_total: usize,
    pub required_planned: usize,
    pub advisory_skipped: usize,
    pub executed: usize,
    pub passed: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ProofRunEntryInput {
    pub scope: String,
    pub kind: String,
    pub required: bool,
    pub command: String,
    pub artifact_path: Option<String>,
    pub status: String,
    pub skip_reason: String,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ProofRunObservationInput {
    pub schema: String,
    pub status: String,
    pub execution_status: String,
    pub profile: String,
    pub base: String,
    pub head: String,
    pub ok: bool,
    pub execution_guard: ProofObservationGuardInput,
    pub counts: ProofRunObservationCountsInput,
    #[serde(default)]
    pub scopes: Vec<ProofObservationScopeInput>,
    #[serde(default)]
    pub changed_files: Vec<String>,
    #[serde(default)]
    pub unknown_files: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ProofObservationGuardInput {
    pub enabled: bool,
    pub ci: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ProofRunObservationCountsInput {
    pub commands_total: usize,
    pub required_planned: usize,
    pub advisory_skipped: usize,
    pub executed: usize,
    pub passed: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ProofObservationScopeInput {
    pub name: String,
    pub kind: String,
    pub command: String,
    pub status: String,
    pub exit_code: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ProofExecutorObservationInput {
    pub schema: String,
    pub status: String,
    pub execution_status: String,
    pub profile: String,
    pub base: String,
    pub head: String,
    pub family: String,
    pub required: bool,
    pub ok: bool,
    pub execution_guard: ProofObservationGuardInput,
    pub counts: ProofExecutorObservationCountsInput,
    #[serde(default)]
    pub scopes: Vec<ProofExecutorObservationScopeInput>,
    #[serde(default)]
    pub changed_files: Vec<String>,
    #[serde(default)]
    pub unknown_files: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ProofExecutorObservationCountsInput {
    pub selected: usize,
    pub executed: usize,
    pub passed: usize,
    pub failed: usize,
    pub artifacts: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ProofExecutorObservationScopeInput {
    pub name: String,
    pub kind: String,
    pub command: String,
    pub artifact_path: Option<String>,
    pub status: String,
    pub exit_code: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CoverageReceiptInput {
    pub schema: String,
    pub schema_version: u32,
    pub repo: String,
    pub lane: String,
    pub flag: String,
    pub workflow: String,
    pub sha: String,
    pub github: CoverageGithubInput,
    #[serde(default)]
    pub artifacts: Vec<CoverageArtifactInput>,
    pub status: CoverageStatusInput,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CoverageGithubInput {
    pub run_id: Option<String>,
    pub run_attempt: Option<String>,
    pub event_name: Option<String>,
    pub ref_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CoverageArtifactInput {
    pub path: String,
    pub kind: String,
    pub bytes: u64,
    pub non_empty: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CoverageStatusInput {
    pub ok: bool,
    #[serde(default)]
    pub missing: Vec<String>,
    #[serde(default)]
    pub empty: Vec<String>,
}
