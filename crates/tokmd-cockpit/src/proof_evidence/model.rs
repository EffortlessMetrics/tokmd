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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proof_evidence_kind_as_str_covers_all_variants() {
        assert_eq!(
            ProofEvidenceKind::ProofRunSummary.as_str(),
            "proof_run_summary"
        );
        assert_eq!(
            ProofEvidenceKind::ProofRunObservation.as_str(),
            "proof_run_observation"
        );
        assert_eq!(
            ProofEvidenceKind::ProofExecutorObservation.as_str(),
            "proof_executor_observation"
        );
        assert_eq!(
            ProofEvidenceKind::CoverageReceipt.as_str(),
            "coverage_receipt"
        );
    }

    #[test]
    fn proof_evidence_kind_packet_file_name_covers_all_variants() {
        assert_eq!(
            ProofEvidenceKind::ProofRunSummary.packet_file_name(),
            "proof-run-summary.json"
        );
        assert_eq!(
            ProofEvidenceKind::ProofRunObservation.packet_file_name(),
            "proof-run-observation.json"
        );
        assert_eq!(
            ProofEvidenceKind::ProofExecutorObservation.packet_file_name(),
            "proof-executor-observation.json"
        );
        assert_eq!(
            ProofEvidenceKind::CoverageReceipt.packet_file_name(),
            "coverage-receipt.json"
        );
    }

    #[test]
    fn proof_evidence_availability_as_str_covers_all_variants() {
        assert_eq!(ProofEvidenceAvailability::Available.as_str(), "available");
        assert_eq!(ProofEvidenceAvailability::Missing.as_str(), "missing");
        assert_eq!(ProofEvidenceAvailability::Skipped.as_str(), "skipped");
        assert_eq!(ProofEvidenceAvailability::Stale.as_str(), "stale");
        assert_eq!(ProofEvidenceAvailability::Degraded.as_str(), "degraded");
        assert_eq!(
            ProofEvidenceAvailability::Unavailable.as_str(),
            "unavailable"
        );
    }

    #[test]
    fn proof_execution_status_as_str_covers_all_variants() {
        assert_eq!(ProofExecutionStatus::Planned.as_str(), "planned");
        assert_eq!(
            ProofExecutionStatus::ExecutedPassed.as_str(),
            "executed_passed"
        );
        assert_eq!(
            ProofExecutionStatus::ExecutedFailed.as_str(),
            "executed_failed"
        );
        assert_eq!(ProofExecutionStatus::NotExecuted.as_str(), "not_executed");
        assert_eq!(ProofExecutionStatus::DryRun.as_str(), "dry_run");
    }
}
