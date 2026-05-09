//! Proof-control-plane evidence artifact classification and JSON parsing.

use anyhow::{Context, Result, bail};
use serde_json::Value;

use super::inputs::{
    CoverageReceiptInput, ProofExecutorObservationInput, ProofRunObservationInput,
    ProofRunSummaryInput,
};
use super::model::ProofEvidenceKind;

pub(super) const PROOF_RUN_SUMMARY_SCHEMA: &str = "tokmd.proof_run_summary.v1";
pub(super) const PROOF_RUN_OBSERVATION_SCHEMA: &str = "tokmd.proof_run_observation.v1";
pub(super) const PROOF_EXECUTOR_OBSERVATION_SCHEMA: &str = "tokmd.proof_executor_observation.v1";
pub(super) const COVERAGE_RECEIPT_SCHEMA: &str = "tokmd.coverage_receipt.v1";

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

    pub(crate) fn changed_files(&self) -> &[String] {
        match self {
            Self::ProofRunSummary(artifact) => &artifact.changed_files,
            Self::ProofRunObservation(artifact) => &artifact.changed_files,
            Self::ProofExecutorObservation(artifact) => &artifact.changed_files,
            Self::CoverageReceipt(_) => &[],
        }
    }
}

pub(super) fn parse_proof_evidence_json(raw: &str) -> Result<ProofEvidenceArtifact> {
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
    use crate::proof_evidence::fixtures::{
        coverage_receipt_artifact, proof_executor_observation_artifact,
        proof_run_observation_artifact, proof_run_summary_artifact,
    };

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
}
