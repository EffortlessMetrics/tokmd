//! Cockpit review packet rendering.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::doc_artifacts_evidence::{DOC_ARTIFACTS_PACKET_PATH, DocArtifactsEvidenceInput};
use crate::proof_evidence::ProofEvidenceArtifact;
use crate::{CockpitReceipt, ProofEvidenceInput};

use super::bun_ub_sensor::BunUbSensorEvidence;
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
    write_review_packet_with_imported_evidence_and_bun_ub_sensor(
        dir,
        receipt,
        proof_evidence,
        doc_artifacts,
        BunUbSensorEvidence::missing(),
    )
}

/// Write review packet artifacts and include imported proof/documentation
/// evidence plus Bun UB sensor artifact availability in the review map.
pub fn write_review_packet_with_imported_evidence_and_bun_ub_sensor(
    dir: &Path,
    receipt: &CockpitReceipt,
    proof_evidence: &[ProofEvidenceInput],
    doc_artifacts: Option<&DocArtifactsEvidenceInput>,
    bun_ub_sensor: BunUbSensorEvidence,
) -> Result<()> {
    let packet_root = prepare_packet_root(dir)?;
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
        bun_ub_sensor,
    ))?;
    let review_map_md = render_review_map_md(
        receipt,
        &packet_proof_inputs,
        doc_artifacts.as_ref(),
        &review_packet_dir,
        bun_ub_sensor,
    );
    let comment_md = render_review_packet_comment_md(
        receipt,
        &packet_proof_inputs,
        doc_artifacts.as_ref(),
        bun_ub_sensor,
    );

    write_packet_artifact(&packet_root, "cockpit.json", &cockpit_json)?;
    write_packet_artifact(&packet_root, "evidence.json", &evidence_json)?;
    write_packet_artifact(&packet_root, "review-map.json", &review_map_json)?;
    write_packet_artifact(&packet_root, "review-map.md", &review_map_md)?;
    write_packet_artifact(&packet_root, "comment.md", &comment_md)?;
    if !proof_artifacts.is_empty() {
        for artifact in &proof_artifacts {
            write_packet_artifact(&packet_root, &artifact.path, &artifact.content)?;
        }
    }
    let doc_artifacts_json = doc_artifacts.as_ref().map(doc_artifacts_json).transpose()?;
    if doc_artifacts.is_some()
        && let Some(content) = doc_artifacts_json.as_deref()
    {
        write_packet_artifact(&packet_root, DOC_ARTIFACTS_PACKET_PATH, content)?;
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
    write_packet_artifact(
        &packet_root,
        "manifest.json",
        &serde_json::to_string_pretty(&manifest)?,
    )?;

    Ok(())
}

/// Resolve the packet directory to a canonical root and refuse to write
/// through a symlinked packet root.
///
/// A pull request can commit `.tokmd/review` as a symlink (Git records it as
/// mode `120000` and restores it on checkout), and `.tokmd/` is not
/// gitignored in this repository. Writing through such a root would emit
/// every packet artifact outside the directory the caller asked for.
fn prepare_packet_root(dir: &Path) -> Result<PathBuf> {
    if dir.exists() {
        let metadata = std::fs::symlink_metadata(dir)
            .with_context(|| format!("failed to stat review packet directory {}", dir.display()))?;
        if metadata.file_type().is_symlink() {
            bail!(
                "review packet directory must not be a symlink: {}",
                dir.display()
            );
        }
    } else {
        std::fs::create_dir_all(dir).with_context(|| {
            format!("failed to create review packet directory {}", dir.display())
        })?;
    }

    let root = dir.canonicalize().with_context(|| {
        format!(
            "failed to resolve review packet directory {}",
            dir.display()
        )
    })?;
    if !root.is_dir() {
        bail!(
            "review packet directory is not a directory: {}",
            dir.display()
        );
    }
    Ok(root)
}

/// Write one packet artifact, refusing any target that resolves outside the
/// packet root.
///
/// The relative path is checked component-wise and the resolved parent
/// directory is re-checked before the write, so a symlinked `proof` (or any
/// other intermediate directory) cannot redirect the write out of the packet.
fn write_packet_artifact(root: &Path, relative: &str, content: &str) -> Result<()> {
    let relative_path = Path::new(relative);
    for component in relative_path.components() {
        match component {
            std::path::Component::Normal(_) => {}
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir
            | std::path::Component::RootDir
            | std::path::Component::Prefix(_) => {
                bail!("review packet artifact path is not packet-local: {relative}");
            }
        }
    }

    let target = root.join(relative_path);
    let parent = target
        .parent()
        .ok_or_else(|| anyhow::anyhow!("review packet artifact has no parent: {relative}"))?;
    std::fs::create_dir_all(parent)
        .with_context(|| format!("failed to create {}", parent.display()))?;
    let resolved_parent = parent
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", parent.display()))?;
    if !resolved_parent.starts_with(root) {
        bail!("review packet artifact path resolves outside the packet directory: {relative}");
    }

    std::fs::write(&target, content)
        .with_context(|| format!("failed to write {}", target.display()))?;
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
        ProofEvidenceArtifact::ProofPackRoute(artifact) => {
            Ok(serde_json::to_string_pretty(artifact)?)
        }
    }
}

fn doc_artifacts_json(input: &DocArtifactsEvidenceInput) -> Result<String> {
    Ok(serde_json::to_string_pretty(&input.receipt)?)
}

#[cfg(test)]
mod tests {
    use super::{prepare_packet_root, write_packet_artifact};
    use std::path::Path;

    #[test]
    fn writes_artifact_inside_the_packet_root() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let root = prepare_packet_root(dir.path())?;
        write_packet_artifact(&root, "cockpit.json", "{}")?;
        write_packet_artifact(&root, "proof/proof-run-summary.json", "{}")?;
        assert!(root.join("cockpit.json").is_file());
        assert!(root.join("proof/proof-run-summary.json").is_file());
        Ok(())
    }

    #[test]
    fn rejects_artifact_path_escaping_the_packet_root() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let root = prepare_packet_root(dir.path())?;
        let error = write_packet_artifact(&root, "../escaped.json", "{}")
            .expect_err("parent traversal must be rejected");
        assert!(error.to_string().contains("not packet-local"), "{error}");
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_packet_root() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let outside = tempfile::tempdir()?;
        let link = dir.path().join("review");
        std::os::unix::fs::symlink(outside.path(), &link)?;
        let error =
            prepare_packet_root(&link).expect_err("a symlinked packet root must be rejected");
        assert!(
            error.to_string().contains("must not be a symlink"),
            "{error}"
        );
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_intermediate_directory() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let outside = tempfile::tempdir()?;
        let root = prepare_packet_root(dir.path())?;
        std::os::unix::fs::symlink(outside.path(), root.join("proof"))?;
        let error = write_packet_artifact(&root, "proof/proof-run-summary.json", "{}")
            .expect_err("a symlinked intermediate directory must be rejected");
        assert!(error.to_string().contains("resolves outside"), "{error}");
        assert!(
            !outside.path().join("proof-run-summary.json").exists(),
            "artifact must not be written outside the packet"
        );
        Ok(())
    }

    #[test]
    fn creates_missing_packet_directory() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let nested = dir.path().join("a").join("b").join("review");
        let root = prepare_packet_root(&nested)?;
        assert!(root.is_dir());
        assert!(Path::new(&root).is_dir());
        Ok(())
    }
}
