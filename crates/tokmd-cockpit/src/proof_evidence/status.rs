//! Proof evidence execution status and availability classification.

use tokmd_types::cockpit::CommitMatch;

use super::inputs::{CoverageReceiptInput, ProofRunEntryInput};
use super::model::{ProofEvidenceAvailability, ProofExecutionStatus};

pub(super) fn proof_run_entry_status(entry: &ProofRunEntryInput) -> ProofExecutionStatus {
    if !entry.skip_reason.trim().is_empty() && entry.exit_code.is_none() {
        return ProofExecutionStatus::NotExecuted;
    }

    scope_status(&entry.status, entry.exit_code.map(i64::from))
}

pub(super) fn scope_status(status: &str, exit_code: Option<i64>) -> ProofExecutionStatus {
    match status.trim().to_ascii_lowercase().as_str() {
        "passed" | "pass" | "success" => ProofExecutionStatus::ExecutedPassed,
        "failed" | "fail" | "error" => ProofExecutionStatus::ExecutedFailed,
        "planned" => ProofExecutionStatus::Planned,
        "dry_run" | "dry-run" => ProofExecutionStatus::DryRun,
        "skipped" | "not_executed" | "not-executed" => ProofExecutionStatus::NotExecuted,
        _ => match exit_code {
            Some(0) => ProofExecutionStatus::ExecutedPassed,
            Some(_) => ProofExecutionStatus::ExecutedFailed,
            None => ProofExecutionStatus::NotExecuted,
        },
    }
}

pub(super) fn availability_for(
    execution_status: ProofExecutionStatus,
    commit_match: CommitMatch,
) -> ProofEvidenceAvailability {
    let base = match execution_status {
        ProofExecutionStatus::ExecutedPassed | ProofExecutionStatus::ExecutedFailed => {
            ProofEvidenceAvailability::Available
        }
        ProofExecutionStatus::Planned | ProofExecutionStatus::NotExecuted => {
            ProofEvidenceAvailability::Missing
        }
        ProofExecutionStatus::DryRun => ProofEvidenceAvailability::Skipped,
    };

    availability_with_commit_match(base, commit_match)
}

pub(super) fn coverage_availability(receipt: &CoverageReceiptInput) -> ProofEvidenceAvailability {
    if receipt.status.ok && receipt.artifacts.iter().any(|artifact| artifact.non_empty) {
        ProofEvidenceAvailability::Available
    } else if !receipt.status.missing.is_empty() {
        ProofEvidenceAvailability::Missing
    } else if !receipt.status.empty.is_empty()
        || receipt.artifacts.iter().all(|artifact| !artifact.non_empty)
    {
        ProofEvidenceAvailability::Degraded
    } else {
        ProofEvidenceAvailability::Unavailable
    }
}

pub(super) fn availability_with_commit_match(
    availability: ProofEvidenceAvailability,
    commit_match: CommitMatch,
) -> ProofEvidenceAvailability {
    match commit_match {
        CommitMatch::Exact => availability,
        CommitMatch::Stale => ProofEvidenceAvailability::Stale,
        CommitMatch::Partial | CommitMatch::Unknown
            if availability == ProofEvidenceAvailability::Available =>
        {
            ProofEvidenceAvailability::Degraded
        }
        CommitMatch::Partial | CommitMatch::Unknown => availability,
    }
}

#[cfg(test)]
mod tests {
    use super::super::inputs::{
        CoverageArtifactInput, CoverageGithubInput, CoverageReceiptInput, CoverageStatusInput,
    };
    use super::*;

    #[test]
    fn maps_scope_status_strings_and_exit_codes() {
        assert_eq!(
            scope_status("passed", None),
            ProofExecutionStatus::ExecutedPassed
        );
        assert_eq!(
            scope_status("failed", None),
            ProofExecutionStatus::ExecutedFailed
        );
        assert_eq!(scope_status("dry-run", None), ProofExecutionStatus::DryRun);
        assert_eq!(
            scope_status("unknown", Some(0)),
            ProofExecutionStatus::ExecutedPassed
        );
        assert_eq!(
            scope_status("unknown", Some(1)),
            ProofExecutionStatus::ExecutedFailed
        );
        assert_eq!(
            scope_status("unknown", None),
            ProofExecutionStatus::NotExecuted
        );
    }

    #[test]
    fn degrades_available_evidence_without_exact_commit_match() {
        assert_eq!(
            availability_with_commit_match(
                ProofEvidenceAvailability::Available,
                CommitMatch::Partial
            ),
            ProofEvidenceAvailability::Degraded
        );
        assert_eq!(
            availability_with_commit_match(
                ProofEvidenceAvailability::Available,
                CommitMatch::Unknown
            ),
            ProofEvidenceAvailability::Degraded
        );
        assert_eq!(
            availability_with_commit_match(
                ProofEvidenceAvailability::Available,
                CommitMatch::Stale
            ),
            ProofEvidenceAvailability::Stale
        );
    }

    #[test]
    fn classifies_coverage_receipt_availability() {
        let mut receipt = coverage_receipt(true, true);
        assert_eq!(
            coverage_availability(&receipt),
            ProofEvidenceAvailability::Available
        );

        receipt = coverage_receipt(false, false);
        receipt.status.missing.push("lcov.info".to_string());
        assert_eq!(
            coverage_availability(&receipt),
            ProofEvidenceAvailability::Missing
        );

        let receipt = coverage_receipt(true, false);
        assert_eq!(
            coverage_availability(&receipt),
            ProofEvidenceAvailability::Degraded
        );

        let receipt = CoverageReceiptInput {
            artifacts: Vec::new(),
            ..coverage_receipt(false, true)
        };
        assert_eq!(
            coverage_availability(&receipt),
            ProofEvidenceAvailability::Degraded
        );
    }

    fn coverage_receipt(ok: bool, non_empty: bool) -> CoverageReceiptInput {
        CoverageReceiptInput {
            schema: "tokmd.coverage_receipt.v1".to_string(),
            schema_version: 1,
            repo: "EffortlessMetrics/tokmd".to_string(),
            lane: "scoped".to_string(),
            flag: "tokmd_cockpit".to_string(),
            workflow: "Coverage".to_string(),
            sha: "abc123".to_string(),
            github: CoverageGithubInput {
                run_id: None,
                run_attempt: None,
                event_name: None,
                ref_name: None,
            },
            artifacts: vec![CoverageArtifactInput {
                path: "target/proof/coverage/tokmd-cockpit.lcov".to_string(),
                kind: "lcov".to_string(),
                bytes: if non_empty { 42 } else { 0 },
                non_empty,
            }],
            status: CoverageStatusInput {
                ok,
                missing: Vec::new(),
                empty: if non_empty {
                    Vec::new()
                } else {
                    vec!["target/proof/coverage/tokmd-cockpit.lcov".to_string()]
                },
            },
        }
    }
}
