//! Proof evidence normalization, execution status, and freshness classification.

use std::path::{Path, PathBuf};

use tokmd_types::cockpit::CommitMatch;

use super::artifacts::ProofEvidenceArtifact;
use super::model::{
    NormalizedProofEvidence, ProofEvidenceInput, ProofEvidenceKind, ProofExecutionStatus,
};
use super::status::{
    availability_for, availability_with_commit_match, coverage_availability,
    proof_run_entry_status, scope_status,
};

pub(crate) fn normalize_proof_evidence_inputs(
    inputs: &[ProofEvidenceInput],
    cockpit_base: Option<&str>,
    cockpit_head: Option<&str>,
) -> Vec<NormalizedProofEvidence> {
    inputs
        .iter()
        .flat_map(|input| {
            normalize_proof_evidence(
                &input.artifact,
                input.source_path.clone(),
                cockpit_base,
                cockpit_head,
            )
        })
        .collect()
}

pub(crate) fn normalize_proof_evidence(
    artifact: &ProofEvidenceArtifact,
    source_path: impl Into<PathBuf>,
    cockpit_base: Option<&str>,
    cockpit_head: Option<&str>,
) -> Vec<NormalizedProofEvidence> {
    let source_path = source_path.into();
    let source_ref = normalize_path_for_ref(&source_path);
    let commit_match = classify_commit_match(
        artifact_base(artifact),
        artifact.head(),
        cockpit_base,
        cockpit_head,
    );

    match artifact {
        ProofEvidenceArtifact::ProofRunSummary(summary) => summary
            .entries
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let execution_status = proof_run_entry_status(entry);
                let availability = availability_for(execution_status, commit_match);
                NormalizedProofEvidence {
                    source_path: source_path.clone(),
                    source_schema: summary.schema.clone(),
                    kind: ProofEvidenceKind::ProofRunSummary,
                    profile: Some(summary.profile.clone()),
                    scope: Some(entry.scope.clone()),
                    command: Some(entry.command.clone()),
                    required: entry.required,
                    advisory: !entry.required,
                    execution_status,
                    availability,
                    commit_match,
                    artifact_refs: vec![format!("{source_ref}#/entries/{idx}")],
                }
            })
            .collect(),
        ProofEvidenceArtifact::ProofRunObservation(observation) => observation
            .scopes
            .iter()
            .enumerate()
            .map(|(idx, scope)| {
                let execution_status = scope_status(&scope.status, scope.exit_code);
                let availability = availability_for(execution_status, commit_match);
                let required = observation.counts.required_planned > 0;
                NormalizedProofEvidence {
                    source_path: source_path.clone(),
                    source_schema: observation.schema.clone(),
                    kind: ProofEvidenceKind::ProofRunObservation,
                    profile: Some(observation.profile.clone()),
                    scope: Some(scope.name.clone()),
                    command: Some(scope.command.clone()),
                    required,
                    advisory: !required,
                    execution_status,
                    availability,
                    commit_match,
                    artifact_refs: vec![format!("{source_ref}#/scopes/{idx}")],
                }
            })
            .collect(),
        ProofEvidenceArtifact::ProofExecutorObservation(observation) => observation
            .scopes
            .iter()
            .enumerate()
            .map(|(idx, scope)| {
                let execution_status = scope_status(&scope.status, scope.exit_code);
                let availability = availability_for(execution_status, commit_match);
                NormalizedProofEvidence {
                    source_path: source_path.clone(),
                    source_schema: observation.schema.clone(),
                    kind: ProofEvidenceKind::ProofExecutorObservation,
                    profile: Some(observation.profile.clone()),
                    scope: Some(scope.name.clone()),
                    command: Some(scope.command.clone()),
                    required: observation.required,
                    advisory: !observation.required,
                    execution_status,
                    availability,
                    commit_match,
                    artifact_refs: vec![format!("{source_ref}#/scopes/{idx}")],
                }
            })
            .collect(),
        ProofEvidenceArtifact::CoverageReceipt(receipt) => {
            let execution_status = if receipt.status.ok {
                ProofExecutionStatus::ExecutedPassed
            } else {
                ProofExecutionStatus::ExecutedFailed
            };
            let base_availability = coverage_availability(receipt);
            let availability = availability_with_commit_match(base_availability, commit_match);
            let artifact_refs = receipt
                .artifacts
                .iter()
                .enumerate()
                .map(|(idx, _)| format!("{source_ref}#/artifacts/{idx}"))
                .collect();

            vec![NormalizedProofEvidence {
                source_path,
                source_schema: receipt.schema.clone(),
                kind: ProofEvidenceKind::CoverageReceipt,
                profile: None,
                scope: Some(receipt.flag.clone()),
                command: None,
                required: false,
                advisory: true,
                execution_status,
                availability,
                commit_match,
                artifact_refs,
            }]
        }
    }
}

fn artifact_base(artifact: &ProofEvidenceArtifact) -> Option<&str> {
    match artifact {
        ProofEvidenceArtifact::ProofRunSummary(artifact) => Some(&artifact.base),
        ProofEvidenceArtifact::ProofRunObservation(artifact) => Some(&artifact.base),
        ProofEvidenceArtifact::ProofExecutorObservation(artifact) => Some(&artifact.base),
        ProofEvidenceArtifact::CoverageReceipt(_) => None,
    }
}

fn classify_commit_match(
    artifact_base: Option<&str>,
    artifact_head: Option<&str>,
    cockpit_base: Option<&str>,
    cockpit_head: Option<&str>,
) -> CommitMatch {
    let artifact_head = non_empty(artifact_head);
    let cockpit_head = non_empty(cockpit_head);

    match (artifact_head, cockpit_head) {
        (Some(artifact_head), Some(cockpit_head)) if artifact_head == cockpit_head => {
            CommitMatch::Exact
        }
        (Some(_), Some(_)) => CommitMatch::Stale,
        _ if non_empty(artifact_base).is_some()
            || artifact_head.is_some()
            || non_empty(cockpit_base).is_some()
            || cockpit_head.is_some() =>
        {
            CommitMatch::Partial
        }
        _ => CommitMatch::Unknown,
    }
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then_some(trimmed)
    })
}

fn normalize_path_for_ref(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tokmd_types::cockpit::CommitMatch;

    use super::*;
    use crate::proof_evidence::fixtures::{
        coverage_receipt_artifact, proof_executor_observation_artifact,
        proof_run_observation_artifact, proof_run_summary_artifact,
    };
    use crate::proof_evidence::model::ProofEvidenceAvailability;

    fn single_evidence(
        artifact: &ProofEvidenceArtifact,
        source_path: &str,
        cockpit_head: Option<&str>,
    ) -> NormalizedProofEvidence {
        let mut evidence = normalize_proof_evidence(
            artifact,
            PathBuf::from(source_path),
            Some("origin/main"),
            cockpit_head,
        );
        assert_eq!(evidence.len(), 1);
        evidence.pop().expect("normalized evidence")
    }

    #[test]
    fn normalizes_proof_run_summary_as_required_exact_evidence() {
        let artifact = proof_run_summary_artifact("abc123");
        let evidence = single_evidence(&artifact, "proof-run-summary.json", Some("abc123"));

        assert_eq!(evidence.kind, ProofEvidenceKind::ProofRunSummary);
        assert_eq!(evidence.profile.as_deref(), Some("fast"));
        assert_eq!(evidence.scope.as_deref(), Some("tokmd_cockpit"));
        assert_eq!(
            evidence.command.as_deref(),
            Some("cargo test -p tokmd-cockpit")
        );
        assert!(evidence.required);
        assert!(!evidence.advisory);
        assert_eq!(
            evidence.execution_status,
            ProofExecutionStatus::ExecutedPassed
        );
        assert_eq!(evidence.availability, ProofEvidenceAvailability::Available);
        assert_eq!(evidence.commit_match, CommitMatch::Exact);
    }

    #[test]
    fn normalizes_proof_run_observation_scope_as_required_evidence() {
        let artifact = proof_run_observation_artifact("abc123");
        let evidence = single_evidence(&artifact, "proof-run-observation.json", Some("abc123"));

        assert_eq!(evidence.kind, ProofEvidenceKind::ProofRunObservation);
        assert_eq!(evidence.scope.as_deref(), Some("tokmd_cockpit"));
        assert!(evidence.required);
        assert_eq!(
            evidence.execution_status,
            ProofExecutionStatus::ExecutedPassed
        );
        assert_eq!(evidence.availability, ProofEvidenceAvailability::Available);
    }

    #[test]
    fn normalizes_executor_dry_run_as_advisory_skipped_evidence() {
        let artifact = proof_executor_observation_artifact("abc123");
        let evidence = single_evidence(
            &artifact,
            "proof/proof-executor-observation.json",
            Some("abc123"),
        );

        assert_eq!(evidence.kind, ProofEvidenceKind::ProofExecutorObservation);
        assert_eq!(evidence.profile.as_deref(), Some("affected"));
        assert_eq!(evidence.scope.as_deref(), Some("tokmd_cockpit"));
        assert!(!evidence.required);
        assert!(evidence.advisory);
        assert_eq!(evidence.execution_status, ProofExecutionStatus::DryRun);
        assert_eq!(evidence.availability, ProofEvidenceAvailability::Skipped);
        assert_eq!(
            evidence.artifact_refs,
            vec!["proof/proof-executor-observation.json#/scopes/0"]
        );
    }

    #[test]
    fn normalizes_coverage_receipt_as_advisory_artifact_evidence() {
        let artifact = coverage_receipt_artifact("abc123", true, true);
        let evidence = single_evidence(&artifact, "proof/coverage-receipt.json", Some("abc123"));

        assert_eq!(evidence.kind, ProofEvidenceKind::CoverageReceipt);
        assert_eq!(evidence.scope.as_deref(), Some("tokmd_cockpit"));
        assert!(!evidence.required);
        assert!(evidence.advisory);
        assert_eq!(
            evidence.execution_status,
            ProofExecutionStatus::ExecutedPassed
        );
        assert_eq!(evidence.availability, ProofEvidenceAvailability::Available);
        assert_eq!(
            evidence.artifact_refs,
            vec!["proof/coverage-receipt.json#/artifacts/0"]
        );
    }

    #[test]
    fn stale_commit_marks_otherwise_available_evidence_stale() {
        let artifact = coverage_receipt_artifact("old", true, true);
        let evidence = single_evidence(&artifact, "coverage-receipt.json", Some("new"));

        assert_eq!(evidence.commit_match, CommitMatch::Stale);
        assert_eq!(evidence.availability, ProofEvidenceAvailability::Stale);
    }

    #[test]
    fn unknown_commit_does_not_become_available_evidence() {
        let artifact = coverage_receipt_artifact("", true, true);
        let mut evidence = normalize_proof_evidence(
            &artifact,
            PathBuf::from("proof/coverage-receipt.json"),
            None,
            None,
        );
        assert_eq!(evidence.len(), 1);
        let evidence = evidence.pop().expect("normalized evidence");

        assert_eq!(evidence.commit_match, CommitMatch::Unknown);
        assert_eq!(evidence.availability, ProofEvidenceAvailability::Degraded);
    }
}
