//! Cockpit review packet rendering.

use std::path::Path;

use anyhow::Result;

use crate::{CockpitReceipt, ProofEvidenceInput};

use super::comment::render_review_packet_comment_md;
use super::evidence::review_packet_evidence;
use super::manifest::review_packet_manifest;
use super::render_json;
use super::review_map::{render_review_map_md, review_packet_review_map};

/// Write review packet artifacts to directory.
///
/// This is the doc-first packet contract from `docs/review-packet.md`. It is
/// intentionally separate from `write_artifacts` so existing cockpit
/// integrations keep their shipped `cockpit.json` / `report.json` /
/// `comment.md` artifact shape until they opt into packet emission.
pub fn write_review_packet(dir: &Path, receipt: &CockpitReceipt) -> Result<()> {
    write_review_packet_with_proof_evidence(dir, receipt, &[])
}

/// Write review packet artifacts and include imported proof evidence in
/// `evidence.json`.
pub fn write_review_packet_with_proof_evidence(
    dir: &Path,
    receipt: &CockpitReceipt,
    proof_evidence: &[ProofEvidenceInput],
) -> Result<()> {
    std::fs::create_dir_all(dir)?;

    let cockpit_json = render_json(receipt)?;
    let evidence_json =
        serde_json::to_string_pretty(&review_packet_evidence(receipt, proof_evidence))?;
    let review_map_json = serde_json::to_string_pretty(&review_packet_review_map(receipt))?;
    let review_map_md = render_review_map_md(receipt);
    let comment_md = render_review_packet_comment_md(receipt);

    std::fs::write(dir.join("cockpit.json"), &cockpit_json)?;
    std::fs::write(dir.join("evidence.json"), &evidence_json)?;
    std::fs::write(dir.join("review-map.json"), &review_map_json)?;
    std::fs::write(dir.join("review-map.md"), &review_map_md)?;
    std::fs::write(dir.join("comment.md"), &comment_md)?;

    let manifest = review_packet_manifest(
        receipt,
        &cockpit_json,
        &evidence_json,
        &review_map_json,
        &review_map_md,
        &comment_md,
    );
    std::fs::write(
        dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;

    Ok(())
}
