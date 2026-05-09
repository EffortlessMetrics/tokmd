//! Normalized proof evidence model shared by cockpit renderers.

use std::path::PathBuf;

use tokmd_types::cockpit::CommitMatch;

use super::artifacts::ProofEvidenceArtifact;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProofEvidenceKind {
    ProofRunSummary,
    ProofRunObservation,
    ProofExecutorObservation,
    CoverageReceipt,
}

impl ProofEvidenceKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::ProofRunSummary => "proof_run_summary",
            Self::ProofRunObservation => "proof_run_observation",
            Self::ProofExecutorObservation => "proof_executor_observation",
            Self::CoverageReceipt => "coverage_receipt",
        }
    }

    pub(crate) fn packet_file_name(self) -> &'static str {
        match self {
            Self::ProofRunSummary => "proof-run-summary.json",
            Self::ProofRunObservation => "proof-run-observation.json",
            Self::ProofExecutorObservation => "proof-executor-observation.json",
            Self::CoverageReceipt => "coverage-receipt.json",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProofEvidenceAvailability {
    Available,
    Missing,
    Skipped,
    Stale,
    Degraded,
    Unavailable,
}

impl ProofEvidenceAvailability {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Missing => "missing",
            Self::Skipped => "skipped",
            Self::Stale => "stale",
            Self::Degraded => "degraded",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProofExecutionStatus {
    Planned,
    ExecutedPassed,
    ExecutedFailed,
    NotExecuted,
    DryRun,
}

impl ProofExecutionStatus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::ExecutedPassed => "executed_passed",
            Self::ExecutedFailed => "executed_failed",
            Self::NotExecuted => "not_executed",
            Self::DryRun => "dry_run",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NormalizedProofEvidence {
    pub source_path: PathBuf,
    pub source_schema: String,
    pub kind: ProofEvidenceKind,
    pub profile: Option<String>,
    pub scope: Option<String>,
    pub command: Option<String>,
    pub required: bool,
    pub advisory: bool,
    pub execution_status: ProofExecutionStatus,
    pub availability: ProofEvidenceAvailability,
    pub commit_match: CommitMatch,
    pub artifact_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofEvidenceInput {
    pub source_path: PathBuf,
    pub artifact: ProofEvidenceArtifact,
}

impl ProofEvidenceInput {
    pub fn kind(&self) -> ProofEvidenceKind {
        self.artifact.kind()
    }
}
