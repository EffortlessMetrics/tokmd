//! Cockpit review packet rendering.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

use crate::doc_artifacts_evidence::{DOC_ARTIFACTS_PACKET_PATH, DocArtifactsEvidenceInput};
use crate::proof_evidence::ProofEvidenceArtifact;
use crate::{CockpitReceipt, ProofEvidenceInput};

use super::comment::render_review_packet_comment_md;
use super::evidence::review_packet_evidence;
use super::manifest::{ReviewPacketArtifactContent, review_packet_manifest};
use super::render_json;
use super::review_map::{render_review_map_md, review_packet_review_map};

/// Write review packet artifacts to directory.
///
/// This is the doc-first packet contract from `docs/review-packet.md`. It is
/// intentionally separate from `write_artifacts` so existing cockpit
/// integrations keep their shipped `cockpit.json` / `report.json` /
/// `comment.md` artifact shape until they opt into packet emission.
pub fn write_review_packet(dir: &Path, receipt: &CockpitReceipt) -> Result<()> {
    write_review_packet_with_imported_evidence(dir, receipt, &[], None)
}

/// Write review packet artifacts and include imported proof evidence in
/// `evidence.json`.
pub fn write_review_packet_with_proof_evidence(
    dir: &Path,
    receipt: &CockpitReceipt,
    proof_evidence: &[ProofEvidenceInput],
) -> Result<()> {
    write_review_packet_with_imported_evidence(dir, receipt, proof_evidence, None)
}

/// Write review packet artifacts and include imported proof and
/// documentation-control evidence in `evidence.json`.
pub fn write_review_packet_with_imported_evidence(
    dir: &Path,
    receipt: &CockpitReceipt,
    proof_evidence: &[ProofEvidenceInput],
    doc_artifacts: Option<&DocArtifactsEvidenceInput>,
) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    let review_packet_dir = review_packet_dir_for_reproduction(dir);
    let proof_artifacts = packet_proof_artifacts(proof_evidence)?;
    let packet_proof_inputs: Vec<_> = proof_artifacts
        .iter()
        .map(|artifact| artifact.input.clone())
        .collect();
    let doc_artifacts = packet_doc_artifacts_input(doc_artifacts)?;

    let cockpit_json = render_json(receipt)?;
    let evidence_json = serde_json::to_string_pretty(&review_packet_evidence(
        receipt,
        &packet_proof_inputs,
        doc_artifacts.as_ref(),
    ))?;
    let review_map_json = serde_json::to_string_pretty(&review_packet_review_map(
        receipt,
        &packet_proof_inputs,
        doc_artifacts.as_ref(),
        &review_packet_dir,
    ))?;
    let review_map_md = render_review_map_md(
        receipt,
        &packet_proof_inputs,
        doc_artifacts.as_ref(),
        &review_packet_dir,
    );
    let comment_md =
        render_review_packet_comment_md(receipt, &packet_proof_inputs, doc_artifacts.as_ref());

    std::fs::write(dir.join("cockpit.json"), &cockpit_json)?;
    std::fs::write(dir.join("evidence.json"), &evidence_json)?;
    std::fs::write(dir.join("review-map.json"), &review_map_json)?;
    std::fs::write(dir.join("review-map.md"), &review_map_md)?;
    std::fs::write(dir.join("comment.md"), &comment_md)?;
    if !proof_artifacts.is_empty() {
        std::fs::create_dir_all(dir.join("proof"))?;
        for artifact in &proof_artifacts {
            std::fs::write(dir.join(&artifact.path), &artifact.content)?;
        }
    }
    let doc_artifacts_json = doc_artifacts.as_ref().map(doc_artifacts_json).transpose()?;
    if doc_artifacts.is_some() {
        if let Some(parent) = Path::new(DOC_ARTIFACTS_PACKET_PATH).parent() {
            std::fs::create_dir_all(dir.join(parent))?;
        }
        if let Some(content) = doc_artifacts_json.as_deref() {
            std::fs::write(dir.join(DOC_ARTIFACTS_PACKET_PATH), content)?;
        }
    }

    let mut extra_artifacts: Vec<_> = proof_artifacts
        .iter()
        .map(|artifact| ReviewPacketArtifactContent {
            id: &artifact.id,
            path: &artifact.path,
            schema: &artifact.schema,
            media_type: "application/json",
            content: &artifact.content,
        })
        .collect();
    if let Some(content) = doc_artifacts_json.as_deref() {
        extra_artifacts.push(ReviewPacketArtifactContent {
            id: "doc-artifacts-check",
            path: DOC_ARTIFACTS_PACKET_PATH,
            schema: "tokmd.doc_artifacts_check.v1",
            media_type: "application/json",
            content,
        });
    }
    let manifest = review_packet_manifest(
        receipt,
        &cockpit_json,
        &evidence_json,
        &review_map_json,
        &review_map_md,
        &comment_md,
        &extra_artifacts,
    );
    std::fs::write(
        dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;

    Ok(())
}

fn review_packet_dir_for_reproduction(dir: &Path) -> String {
    if dir.is_absolute() {
        return "<REVIEW_PACKET_DIR>".to_string();
    }
    let rendered = dir.to_string_lossy().replace('\\', "/");
    if rendered.is_empty() {
        ".tokmd/review".to_string()
    } else {
        rendered
    }
}

fn packet_doc_artifacts_input(
    doc_artifacts: Option<&DocArtifactsEvidenceInput>,
) -> Result<Option<DocArtifactsEvidenceInput>> {
    doc_artifacts
        .map(|input| {
            Ok(DocArtifactsEvidenceInput {
                source_path: PathBuf::from(DOC_ARTIFACTS_PACKET_PATH),
                receipt: input.receipt.clone(),
            })
        })
        .transpose()
}

struct PacketProofArtifact {
    id: String,
    path: String,
    schema: String,
    content: String,
    input: ProofEvidenceInput,
}

fn packet_proof_artifacts(
    proof_evidence: &[ProofEvidenceInput],
) -> Result<Vec<PacketProofArtifact>> {
    let mut seen_paths = BTreeSet::new();
    let mut artifacts = Vec::new();

    for input in proof_evidence {
        let kind = input.kind();
        let file_name = kind.packet_file_name();
        let path = format!("proof/{file_name}");
        if !seen_paths.insert(path.clone()) {
            bail!("duplicate proof evidence artifact for packet path `{path}`");
        }

        artifacts.push(PacketProofArtifact {
            id: proof_artifact_id(file_name),
            path: path.clone(),
            schema: input.artifact.schema().to_string(),
            content: proof_artifact_json(input)?,
            input: ProofEvidenceInput {
                source_path: PathBuf::from(path),
                artifact: input.artifact.clone(),
            },
        });
    }

    Ok(artifacts)
}

fn proof_artifact_id(file_name: &str) -> String {
    file_name.trim_end_matches(".json").to_string()
}

fn proof_artifact_json(input: &ProofEvidenceInput) -> Result<String> {
    match &input.artifact {
        ProofEvidenceArtifact::ProofRunSummary(artifact) => {
            Ok(serde_json::to_string_pretty(artifact)?)
        }
        ProofEvidenceArtifact::ProofRunObservation(artifact) => {
            Ok(serde_json::to_string_pretty(artifact)?)
        }
        ProofEvidenceArtifact::ProofExecutorObservation(artifact) => {
            Ok(serde_json::to_string_pretty(artifact)?)
        }
        ProofEvidenceArtifact::CoverageReceipt(artifact) => {
            Ok(serde_json::to_string_pretty(artifact)?)
        }
    }
}

fn doc_artifacts_json(input: &DocArtifactsEvidenceInput) -> Result<String> {
    Ok(serde_json::to_string_pretty(&input.receipt)?)
}
