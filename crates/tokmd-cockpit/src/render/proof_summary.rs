//! Compact proof evidence counts shared by packet renderers.

use crate::proof_evidence::{
    ProofEvidenceAvailability, ProofEvidenceInput, ProofExecutionStatus,
    normalize_proof_evidence_inputs,
};
use crate::{CockpitReceipt, CommitMatch};

#[derive(Default)]
pub(super) struct ProofEvidenceSummary {
    pub(super) total: usize,
    pub(super) required_passed: usize,
    pub(super) required_failed: usize,
    pub(super) required_missing: usize,
    pub(super) advisory_available: usize,
    pub(super) advisory_missing: usize,
    pub(super) exact: usize,
    pub(super) partial: usize,
    pub(super) stale: usize,
    pub(super) unknown: usize,
    pub(super) not_run: usize,
    pub(super) degraded: usize,
    pub(super) skipped: usize,
    pub(super) unavailable: usize,
}

pub(super) fn proof_evidence_summary(
    receipt: &CockpitReceipt,
    proof_inputs: &[ProofEvidenceInput],
) -> ProofEvidenceSummary {
    let mut counts = ProofEvidenceSummary::default();

    for item in normalize_proof_evidence_inputs(
        proof_inputs,
        Some(&receipt.base_ref),
        Some(&receipt.head_ref),
    ) {
        counts.total += 1;

        match item.commit_match {
            CommitMatch::Exact => counts.exact += 1,
            CommitMatch::Partial => counts.partial += 1,
            CommitMatch::Stale => counts.stale += 1,
            CommitMatch::Unknown => counts.unknown += 1,
        }

        match item.availability {
            ProofEvidenceAvailability::Degraded => counts.degraded += 1,
            ProofEvidenceAvailability::Skipped => counts.skipped += 1,
            ProofEvidenceAvailability::Unavailable => counts.unavailable += 1,
            _ => {}
        }

        if matches!(
            item.execution_status,
            ProofExecutionStatus::Planned | ProofExecutionStatus::NotExecuted
        ) {
            counts.not_run += 1;
        }

        if item.required {
            if item.execution_status == ProofExecutionStatus::ExecutedPassed
                && item.availability == ProofEvidenceAvailability::Available
            {
                counts.required_passed += 1;
            } else if item.execution_status == ProofExecutionStatus::ExecutedFailed {
                counts.required_failed += 1;
            } else if matches!(
                item.availability,
                ProofEvidenceAvailability::Missing | ProofEvidenceAvailability::Unavailable
            ) {
                counts.required_missing += 1;
            }
        }

        if item.advisory {
            if item.availability == ProofEvidenceAvailability::Available {
                counts.advisory_available += 1;
            } else if matches!(
                item.availability,
                ProofEvidenceAvailability::Missing | ProofEvidenceAvailability::Unavailable
            ) {
                counts.advisory_missing += 1;
            }
        }
    }

    counts
}
