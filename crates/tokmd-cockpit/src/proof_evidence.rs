//! Deserializable proof-control-plane evidence artifacts.
//!
//! Cockpit imports these artifacts into review packet evidence when callers
//! explicitly provide them. This module locks the accepted input shapes so
//! packet wiring can classify proof evidence without duplicating the `xtask`
//! JSON contracts.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde_json::Value;
use tokmd_types::cockpit::CommitMatch;

mod inputs;

use inputs::{
    CoverageReceiptInput, ProofExecutorObservationInput, ProofRunEntryInput,
    ProofRunObservationInput, ProofRunSummaryInput,
};

const PROOF_RUN_SUMMARY_SCHEMA: &str = "tokmd.proof_run_summary.v1";
const PROOF_RUN_OBSERVATION_SCHEMA: &str = "tokmd.proof_run_observation.v1";
const PROOF_EXECUTOR_OBSERVATION_SCHEMA: &str = "tokmd.proof_executor_observation.v1";
const COVERAGE_RECEIPT_SCHEMA: &str = "tokmd.coverage_receipt.v1";

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
pub enum ProofEvidenceArtifact {
    ProofRunSummary(ProofRunSummaryInput),
    ProofRunObservation(ProofRunObservationInput),
    ProofExecutorObservation(ProofExecutorObservationInput),
    CoverageReceipt(CoverageReceiptInput),
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

impl ProofEvidenceArtifact {
    pub fn kind(&self) -> ProofEvidenceKind {
        match self {
            Self::ProofRunSummary(_) => ProofEvidenceKind::ProofRunSummary,
            Self::ProofRunObservation(_) => ProofEvidenceKind::ProofRunObservation,
            Self::ProofExecutorObservation(_) => ProofEvidenceKind::ProofExecutorObservation,
            Self::CoverageReceipt(_) => ProofEvidenceKind::CoverageReceipt,
        }
    }

    pub fn schema(&self) -> &str {
        match self {
            Self::ProofRunSummary(artifact) => &artifact.schema,
            Self::ProofRunObservation(artifact) => &artifact.schema,
            Self::ProofExecutorObservation(artifact) => &artifact.schema,
            Self::CoverageReceipt(artifact) => &artifact.schema,
        }
    }

    pub fn profile(&self) -> Option<&str> {
        match self {
            Self::ProofRunSummary(artifact) => Some(&artifact.profile),
            Self::ProofRunObservation(artifact) => Some(&artifact.profile),
            Self::ProofExecutorObservation(artifact) => Some(&artifact.profile),
            Self::CoverageReceipt(_) => None,
        }
    }

    pub fn head(&self) -> Option<&str> {
        match self {
            Self::ProofRunSummary(artifact) => Some(&artifact.head),
            Self::ProofRunObservation(artifact) => Some(&artifact.head),
            Self::ProofExecutorObservation(artifact) => Some(&artifact.head),
            Self::CoverageReceipt(artifact) => Some(&artifact.sha),
        }
    }

    pub(crate) fn changed_files(&self) -> &[String] {
        match self {
            Self::ProofRunSummary(artifact) => &artifact.changed_files,
            Self::ProofRunObservation(artifact) => &artifact.changed_files,
            Self::ProofExecutorObservation(artifact) => &artifact.changed_files,
            Self::CoverageReceipt(_) => &[],
        }
    }
}

pub fn proof_evidence_kind(raw: &str) -> Result<ProofEvidenceKind> {
    parse_proof_evidence_json(raw).map(|artifact| artifact.kind())
}

pub fn parse_proof_evidence_input(
    raw: &str,
    source_path: impl Into<PathBuf>,
) -> Result<ProofEvidenceInput> {
    let artifact = parse_proof_evidence_json(raw)?;

    Ok(ProofEvidenceInput {
        source_path: source_path.into(),
        artifact,
    })
}

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

fn proof_run_entry_status(entry: &ProofRunEntryInput) -> ProofExecutionStatus {
    if !entry.skip_reason.trim().is_empty() && entry.exit_code.is_none() {
        return ProofExecutionStatus::NotExecuted;
    }

    scope_status(&entry.status, entry.exit_code.map(i64::from))
}

fn scope_status(status: &str, exit_code: Option<i64>) -> ProofExecutionStatus {
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

fn availability_for(
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

fn coverage_availability(receipt: &CoverageReceiptInput) -> ProofEvidenceAvailability {
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

fn availability_with_commit_match(
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

fn normalize_path_for_ref(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub fn parse_proof_evidence_json(raw: &str) -> Result<ProofEvidenceArtifact> {
    let value: Value = serde_json::from_str(raw).context("parse proof evidence JSON")?;
    let schema = value
        .get("schema")
        .and_then(Value::as_str)
        .context("proof evidence artifact missing string schema")?;

    match schema {
        PROOF_RUN_SUMMARY_SCHEMA => Ok(ProofEvidenceArtifact::ProofRunSummary(
            serde_json::from_value(value).context("parse proof-run summary evidence")?,
        )),
        PROOF_RUN_OBSERVATION_SCHEMA => Ok(ProofEvidenceArtifact::ProofRunObservation(
            serde_json::from_value(value).context("parse proof-run observation evidence")?,
        )),
        PROOF_EXECUTOR_OBSERVATION_SCHEMA => Ok(ProofEvidenceArtifact::ProofExecutorObservation(
            serde_json::from_value(value).context("parse proof-executor observation evidence")?,
        )),
        COVERAGE_RECEIPT_SCHEMA => Ok(ProofEvidenceArtifact::CoverageReceipt(
            serde_json::from_value(value).context("parse coverage receipt evidence")?,
        )),
        _ => bail!("unsupported proof evidence schema `{schema}`"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_value(value: serde_json::Value) -> ProofEvidenceArtifact {
        parse_proof_evidence_json(&value.to_string()).expect("parse proof evidence")
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
                    "required": true,
                    "command": "cargo test -p tokmd-cockpit",
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
                    "bytes": 42,
                    "non_empty": non_empty
                }
            ],
            "status": {
                "ok": ok,
                "missing": if ok { Vec::<String>::new() } else { vec!["tokmd_cockpit".to_string()] },
                "empty": Vec::<String>::new()
            }
        }))
    }

    #[test]
    fn parses_proof_run_summary() {
        let artifact = parse_proof_evidence_json(
            r#"{
  "schema": "tokmd.proof_run_summary.v1",
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
  "head": "abc123",
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
      "required": true,
      "command": "cargo test -p tokmd-cockpit",
      "artifact_path": null,
      "status": "passed",
      "skip_reason": "",
      "exit_code": 0
    }
  ],
  "unknown_files": []
}"#,
        )
        .expect("parse proof-run summary");

        let ProofEvidenceArtifact::ProofRunSummary(summary) = artifact else {
            panic!("expected proof-run summary");
        };
        assert_eq!(summary.schema, PROOF_RUN_SUMMARY_SCHEMA);
        assert!(summary.execution_guard.required);
        assert_eq!(summary.profile, "fast");
        assert_eq!(summary.entries[0].scope, "tokmd_cockpit");
        assert_eq!(summary.entries[0].exit_code, Some(0));
    }

    #[test]
    fn reports_proof_evidence_kind() {
        let kind = proof_evidence_kind(
            r#"{
  "schema": "tokmd.coverage_receipt.v1",
  "schema_version": 1,
  "repo": "EffortlessMetrics/tokmd",
  "lane": "scoped",
  "flag": "tokmd_cockpit",
  "workflow": "Coverage",
  "sha": "abc123",
  "github": {},
  "artifacts": [],
  "status": { "ok": true, "missing": [], "empty": [] }
}"#,
        )
        .expect("parse coverage receipt kind");

        assert_eq!(kind, ProofEvidenceKind::CoverageReceipt);
    }

    #[test]
    fn normalizes_proof_run_summary_as_required_exact_evidence() {
        let artifact = proof_run_summary_artifact("head123");
        let evidence = normalize_proof_evidence(
            &artifact,
            "proof/proof-run-summary.json",
            Some("origin/main"),
            Some("head123"),
        );

        assert_eq!(evidence.len(), 1);
        let item = &evidence[0];
        assert_eq!(
            item.source_path,
            PathBuf::from("proof/proof-run-summary.json")
        );
        assert_eq!(item.source_schema, PROOF_RUN_SUMMARY_SCHEMA);
        assert_eq!(item.kind, ProofEvidenceKind::ProofRunSummary);
        assert_eq!(item.profile.as_deref(), Some("fast"));
        assert_eq!(item.scope.as_deref(), Some("tokmd_cockpit"));
        assert_eq!(item.command.as_deref(), Some("cargo test -p tokmd-cockpit"));
        assert!(item.required);
        assert!(!item.advisory);
        assert_eq!(item.execution_status, ProofExecutionStatus::ExecutedPassed);
        assert_eq!(item.availability, ProofEvidenceAvailability::Available);
        assert_eq!(item.commit_match, CommitMatch::Exact);
        assert_eq!(
            item.artifact_refs,
            ["proof/proof-run-summary.json#/entries/0"]
        );
    }

    #[test]
    fn normalizes_proof_run_observation_scope_as_required_evidence() {
        let artifact = proof_run_observation_artifact("head123");
        let evidence = normalize_proof_evidence(
            &artifact,
            "proof/proof-run-observation.json",
            Some("origin/main"),
            Some("head123"),
        );

        assert_eq!(evidence.len(), 1);
        let item = &evidence[0];
        assert_eq!(item.kind, ProofEvidenceKind::ProofRunObservation);
        assert_eq!(item.profile.as_deref(), Some("fast"));
        assert_eq!(item.scope.as_deref(), Some("tokmd_cockpit"));
        assert!(item.required);
        assert!(!item.advisory);
        assert_eq!(item.execution_status, ProofExecutionStatus::ExecutedPassed);
        assert_eq!(item.availability, ProofEvidenceAvailability::Available);
        assert_eq!(item.commit_match, CommitMatch::Exact);
        assert_eq!(
            item.artifact_refs,
            ["proof/proof-run-observation.json#/scopes/0"]
        );
    }

    #[test]
    fn normalizes_executor_dry_run_as_advisory_skipped_evidence() {
        let artifact = proof_executor_observation_artifact("head123");
        let evidence = normalize_proof_evidence(
            &artifact,
            "proof/proof-executor-observation.json",
            Some("origin/main"),
            Some("head123"),
        );

        assert_eq!(evidence.len(), 1);
        let item = &evidence[0];
        assert_eq!(item.kind, ProofEvidenceKind::ProofExecutorObservation);
        assert_eq!(item.profile.as_deref(), Some("affected"));
        assert_eq!(item.scope.as_deref(), Some("tokmd_cockpit"));
        assert!(!item.required);
        assert!(item.advisory);
        assert_eq!(item.execution_status, ProofExecutionStatus::DryRun);
        assert_eq!(item.availability, ProofEvidenceAvailability::Skipped);
        assert_eq!(item.commit_match, CommitMatch::Exact);
        assert_eq!(
            item.artifact_refs,
            ["proof/proof-executor-observation.json#/scopes/0"]
        );
    }

    #[test]
    fn normalizes_coverage_receipt_as_advisory_artifact_evidence() {
        let artifact = coverage_receipt_artifact("head123", true, true);
        let evidence = normalize_proof_evidence(
            &artifact,
            "proof/coverage-receipt.json",
            None,
            Some("head123"),
        );

        assert_eq!(evidence.len(), 1);
        let item = &evidence[0];
        assert_eq!(item.kind, ProofEvidenceKind::CoverageReceipt);
        assert_eq!(item.source_schema, COVERAGE_RECEIPT_SCHEMA);
        assert_eq!(item.profile, None);
        assert_eq!(item.scope.as_deref(), Some("tokmd_cockpit"));
        assert_eq!(item.command, None);
        assert!(!item.required);
        assert!(item.advisory);
        assert_eq!(item.execution_status, ProofExecutionStatus::ExecutedPassed);
        assert_eq!(item.availability, ProofEvidenceAvailability::Available);
        assert_eq!(item.commit_match, CommitMatch::Exact);
        assert_eq!(
            item.artifact_refs,
            ["proof/coverage-receipt.json#/artifacts/0"]
        );
    }

    #[test]
    fn stale_commit_marks_otherwise_available_evidence_stale() {
        let artifact = proof_run_summary_artifact("old-head");
        let evidence = normalize_proof_evidence(
            &artifact,
            "proof/proof-run-summary.json",
            Some("origin/main"),
            Some("new-head"),
        );

        assert_eq!(
            evidence[0].execution_status,
            ProofExecutionStatus::ExecutedPassed
        );
        assert_eq!(evidence[0].commit_match, CommitMatch::Stale);
        assert_eq!(evidence[0].availability, ProofEvidenceAvailability::Stale);
    }

    #[test]
    fn unknown_commit_does_not_become_available_evidence() {
        let artifact = coverage_receipt_artifact("", true, true);
        let evidence =
            normalize_proof_evidence(&artifact, "proof/coverage-receipt.json", None, None);

        assert_eq!(
            evidence[0].execution_status,
            ProofExecutionStatus::ExecutedPassed
        );
        assert_eq!(evidence[0].commit_match, CommitMatch::Unknown);
        assert_eq!(
            evidence[0].availability,
            ProofEvidenceAvailability::Degraded
        );
    }

    #[test]
    fn parses_proof_run_observation() {
        let artifact = parse_proof_evidence_json(
            r#"{
  "schema": "tokmd.proof_run_observation.v1",
  "status": "passed",
  "execution_status": "executed",
  "profile": "fast",
  "base": "origin/main",
  "head": "abc123",
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
}"#,
        )
        .expect("parse proof-run observation");

        let ProofEvidenceArtifact::ProofRunObservation(observation) = artifact else {
            panic!("expected proof-run observation");
        };
        assert_eq!(observation.schema, PROOF_RUN_OBSERVATION_SCHEMA);
        assert_eq!(observation.scopes[0].name, "tokmd_cockpit");
        assert_eq!(observation.scopes[0].status, "passed");
    }

    #[test]
    fn parses_proof_executor_observation() {
        let artifact = parse_proof_evidence_json(
            r#"{
  "schema": "tokmd.proof_executor_observation.v1",
  "status": "dry_run",
  "execution_status": "dry_run",
  "profile": "affected",
  "base": "origin/main",
  "head": "def456",
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
}"#,
        )
        .expect("parse proof-executor observation");

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
        let artifact = parse_proof_evidence_json(
            r#"{
  "schema": "tokmd.coverage_receipt.v1",
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
  "artifacts": [
    {
      "path": "target/proof/coverage/tokmd-cockpit.lcov",
      "kind": "lcov",
      "bytes": 42,
      "non_empty": true
    }
  ],
  "status": {
    "ok": true,
    "missing": [],
    "empty": []
  }
}"#,
        )
        .expect("parse coverage receipt");

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
}
