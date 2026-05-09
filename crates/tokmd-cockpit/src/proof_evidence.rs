//! Deserializable proof-control-plane evidence artifacts.
//!
//! Cockpit imports these artifacts into review packet evidence when callers
//! explicitly provide them. This module locks the accepted input shapes so
//! packet wiring can classify proof evidence without duplicating the `xtask`
//! JSON contracts.

#![allow(dead_code)]

use std::path::PathBuf;

use anyhow::Result;

mod artifacts;
mod inputs;
mod model;
mod normalize;
#[cfg(test)]
mod tests;

pub(crate) use artifacts::ProofEvidenceArtifact;
use artifacts::parse_proof_evidence_json;
pub(crate) use model::{NormalizedProofEvidence, ProofEvidenceAvailability, ProofExecutionStatus};
pub use model::{ProofEvidenceInput, ProofEvidenceKind};
pub(crate) use normalize::{normalize_proof_evidence, normalize_proof_evidence_inputs};

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
