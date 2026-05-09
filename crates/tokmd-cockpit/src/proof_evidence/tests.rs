use std::path::PathBuf;

use tokmd_types::cockpit::CommitMatch;

use super::artifacts::{
    COVERAGE_RECEIPT_SCHEMA, PROOF_EXECUTOR_OBSERVATION_SCHEMA, PROOF_RUN_OBSERVATION_SCHEMA,
    PROOF_RUN_SUMMARY_SCHEMA, ProofEvidenceArtifact, parse_proof_evidence_json,
};
use super::model::{
    NormalizedProofEvidence, ProofEvidenceAvailability, ProofEvidenceKind, ProofExecutionStatus,
};
use super::normalize::normalize_proof_evidence;
use super::proof_evidence_kind;

fn parse_value(value: serde_json::Value) -> ProofEvidenceArtifact {
    parse_proof_evidence_json(&value.to_string()).expect("parse proof evidence")
}

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

fn proof_run_summary_artifact(head: &str) -> ProofEvidenceArtifact {
    parse_value(serde_json::json!({
        "schema": PROOF_RUN_SUMMARY_SCHEMA,
        "status": "passed",
        "execution_status": "executed",
        "execution_guard": {
            "required": true,
            "enabled": true,
            "ci": true,
            "allow_ci_required_execution": true,
            "allow_local_required_execution": false,
            "reason": "ci_required_execution_opted_in"
        },
        "profile": "fast",
        "base": "origin/main",
        "head": head,
        "ok": true,
        "changed_files": ["crates/tokmd-cockpit/src/lib.rs"],
        "counts": {
            "commands_total": 1,
            "required_planned": 1,
            "advisory_skipped": 0,
            "executed": 1,
            "passed": 1,
            "failed": 0
        },
        "entries": [
            {
                "scope": "tokmd_cockpit",
                "kind": "test",
                "command": "cargo test -p tokmd-cockpit",
                "required": true,
                "advisory": false,
                "artifact_path": null,
                "status": "passed",
                "skip_reason": "",
                "exit_code": 0
            }
        ],
        "unknown_files": []
    }))
}

fn proof_run_observation_artifact(head: &str) -> ProofEvidenceArtifact {
    parse_value(serde_json::json!({
        "schema": PROOF_RUN_OBSERVATION_SCHEMA,
        "status": "passed",
        "execution_status": "executed",
        "profile": "fast",
        "base": "origin/main",
        "head": head,
        "ok": true,
        "execution_guard": {
            "enabled": true,
            "ci": true,
            "reason": "required proof-run summary verified"
        },
        "counts": {
            "commands_total": 1,
            "required_planned": 1,
            "advisory_skipped": 0,
            "executed": 1,
            "passed": 1,
            "failed": 0
        },
        "scopes": [
            {
                "name": "tokmd_cockpit",
                "kind": "test",
                "command": "cargo test -p tokmd-cockpit",
                "status": "passed",
                "exit_code": 0
            }
        ],
        "changed_files": ["crates/tokmd-cockpit/src/lib.rs"],
        "unknown_files": []
    }))
}

fn proof_executor_observation_artifact(head: &str) -> ProofEvidenceArtifact {
    parse_value(serde_json::json!({
        "schema": PROOF_EXECUTOR_OBSERVATION_SCHEMA,
        "status": "dry_run",
        "execution_status": "dry_run",
        "profile": "affected",
        "base": "origin/main",
        "head": head,
        "family": "coverage",
        "required": false,
        "ok": true,
        "execution_guard": {
            "enabled": true,
            "ci": true,
            "reason": "advisory_executor_enabled"
        },
        "counts": {
            "selected": 1,
            "executed": 0,
            "passed": 0,
            "failed": 0,
            "artifacts": 1
        },
        "scopes": [
            {
                "name": "tokmd_cockpit",
                "kind": "coverage",
                "command": "cargo llvm-cov -p tokmd-cockpit",
                "artifact_path": "target/proof/coverage/tokmd-cockpit.lcov",
                "status": "dry_run",
                "exit_code": null
            }
        ],
        "changed_files": ["crates/tokmd-cockpit/src/render/review_packet.rs"],
        "unknown_files": []
    }))
}

fn coverage_receipt_artifact(sha: &str, ok: bool, non_empty: bool) -> ProofEvidenceArtifact {
    parse_value(serde_json::json!({
        "schema": COVERAGE_RECEIPT_SCHEMA,
        "schema_version": 1,
        "repo": "EffortlessMetrics/tokmd",
        "lane": "scoped",
        "flag": "tokmd_cockpit",
        "workflow": "Coverage",
        "sha": sha,
        "github": {
            "run_id": "12345",
            "run_attempt": "1",
            "event_name": "pull_request",
            "ref_name": "feature"
        },
        "artifacts": [
            {
                "path": "target/proof/coverage/tokmd-cockpit.lcov",
                "kind": "lcov",
                "bytes": if non_empty { 42 } else { 0 },
                "non_empty": non_empty
            }
        ],
        "status": {
            "ok": ok,
            "missing": [],
            "empty": if non_empty {
                Vec::<String>::new()
            } else {
                vec!["target/proof/coverage/tokmd-cockpit.lcov".to_string()]
            }
        }
    }))
}

#[test]
fn parses_proof_run_summary() {
    let artifact = proof_run_summary_artifact("abc123");

    let ProofEvidenceArtifact::ProofRunSummary(summary) = artifact else {
        panic!("expected proof-run summary");
    };
    assert_eq!(summary.schema, PROOF_RUN_SUMMARY_SCHEMA);
    assert_eq!(summary.profile, "fast");
    assert!(summary.execution_guard.required);
    assert_eq!(summary.entries[0].scope, "tokmd_cockpit");
}

#[test]
fn reports_proof_evidence_kind() {
    let raw = serde_json::json!({
        "schema": COVERAGE_RECEIPT_SCHEMA,
        "schema_version": 1,
        "repo": "EffortlessMetrics/tokmd",
        "lane": "scoped",
        "flag": "tokmd_cockpit",
        "workflow": "Coverage",
        "sha": "abc123",
        "github": {
            "run_id": "12345",
            "run_attempt": "1",
            "event_name": "pull_request",
            "ref_name": "feature"
        },
        "artifacts": [],
        "status": { "ok": true, "missing": [], "empty": [] }
    });

    assert_eq!(
        proof_evidence_kind(&raw.to_string()).expect("proof evidence kind"),
        ProofEvidenceKind::CoverageReceipt
    );
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

#[test]
fn parses_proof_run_observation() {
    let artifact = proof_run_observation_artifact("abc123");

    let ProofEvidenceArtifact::ProofRunObservation(observation) = artifact else {
        panic!("expected proof-run observation");
    };
    assert_eq!(observation.schema, PROOF_RUN_OBSERVATION_SCHEMA);
    assert_eq!(observation.profile, "fast");
    assert_eq!(observation.scopes[0].name, "tokmd_cockpit");
}

#[test]
fn parses_proof_executor_observation() {
    let artifact = proof_executor_observation_artifact("abc123");

    let ProofEvidenceArtifact::ProofExecutorObservation(observation) = artifact else {
        panic!("expected proof-executor observation");
    };
    assert_eq!(observation.schema, PROOF_EXECUTOR_OBSERVATION_SCHEMA);
    assert_eq!(observation.family, "coverage");
    assert!(!observation.required);
    assert_eq!(
        observation.scopes[0].artifact_path.as_deref(),
        Some("target/proof/coverage/tokmd-cockpit.lcov")
    );
}

#[test]
fn parses_coverage_receipt() {
    let artifact = coverage_receipt_artifact("abc123", true, true);

    let ProofEvidenceArtifact::CoverageReceipt(receipt) = artifact else {
        panic!("expected coverage receipt");
    };
    assert_eq!(receipt.schema, COVERAGE_RECEIPT_SCHEMA);
    assert_eq!(receipt.sha, "abc123");
    assert!(receipt.status.ok);
    assert_eq!(receipt.artifacts[0].kind, "lcov");
}

#[test]
fn rejects_unknown_schema() {
    let err = parse_proof_evidence_json(r#"{ "schema": "tokmd.unknown.v1" }"#)
        .expect_err("unknown schema should fail");
    assert!(
        err.to_string()
            .contains("unsupported proof evidence schema `tokmd.unknown.v1`")
    );
}
