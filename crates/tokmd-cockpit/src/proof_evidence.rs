//! Deserializable proof-control-plane evidence artifacts.
//!
//! Cockpit does not import these artifacts into review packets yet. This module
//! locks the accepted input shapes so future packet wiring can classify proof
//! evidence without duplicating the `xtask` JSON contracts.

#![allow(dead_code)]

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use serde_json::Value;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofEvidenceArtifact {
    ProofRunSummary(ProofRunSummaryInput),
    ProofRunObservation(ProofRunObservationInput),
    ProofExecutorObservation(ProofExecutorObservationInput),
    CoverageReceipt(CoverageReceiptInput),
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
}

pub fn proof_evidence_kind(raw: &str) -> Result<ProofEvidenceKind> {
    parse_proof_evidence_json(raw).map(|artifact| artifact.kind())
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

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ProofRunExecutionGuardInput {
    pub required: bool,
    pub enabled: bool,
    pub ci: bool,
    pub allow_ci_required_execution: bool,
    pub allow_local_required_execution: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ProofRunCountsInput {
    pub commands_total: usize,
    pub required_planned: usize,
    pub advisory_skipped: usize,
    pub executed: usize,
    pub passed: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ProofObservationGuardInput {
    pub enabled: bool,
    pub ci: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ProofRunObservationCountsInput {
    pub commands_total: usize,
    pub required_planned: usize,
    pub advisory_skipped: usize,
    pub executed: usize,
    pub passed: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ProofObservationScopeInput {
    pub name: String,
    pub kind: String,
    pub command: String,
    pub status: String,
    pub exit_code: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ProofExecutorObservationCountsInput {
    pub selected: usize,
    pub executed: usize,
    pub passed: usize,
    pub failed: usize,
    pub artifacts: usize,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ProofExecutorObservationScopeInput {
    pub name: String,
    pub kind: String,
    pub command: String,
    pub artifact_path: Option<String>,
    pub status: String,
    pub exit_code: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct CoverageGithubInput {
    pub run_id: Option<String>,
    pub run_attempt: Option<String>,
    pub event_name: Option<String>,
    pub ref_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct CoverageArtifactInput {
    pub path: String,
    pub kind: String,
    pub bytes: u64,
    pub non_empty: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct CoverageStatusInput {
    pub ok: bool,
    #[serde(default)]
    pub missing: Vec<String>,
    #[serde(default)]
    pub empty: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
