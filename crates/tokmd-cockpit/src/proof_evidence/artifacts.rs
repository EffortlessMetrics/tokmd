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
