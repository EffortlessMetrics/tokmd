//! JSON artifacts that point at external review and proof receipts.

use std::path::Path;

use anyhow::Result;
use serde_json::{Value, json};
use tokmd_types::ArtifactEntry;

use super::HandoffLinkInputs;
use super::artifact::write_json_artifact;

pub(crate) fn write_link_artifacts(
    out_dir: &Path,
    links: &HandoffLinkInputs<'_>,
) -> Result<Vec<ArtifactEntry>> {
    let mut artifacts = Vec::new();

    if links.review_packet_dir.is_some() || links.review_packet_check.is_some() {
        artifacts.push(write_json_artifact(
            out_dir,
            "review-links",
            "review-links.json",
            "Linked cockpit review packet artifacts",
            &review_links_json(links.review_packet_dir, links.review_packet_check),
        )?);
    }

    if links.affected.is_some() || links.proof_plan.is_some() {
        artifacts.push(write_json_artifact(
            out_dir,
            "proof-links",
            "proof-links.json",
            "Linked affected-proof and proof-plan artifacts",
            &proof_links_json(links.affected, links.proof_plan),
        )?);
    }

    Ok(artifacts)
}

pub(super) fn path_string(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

fn review_links_json(
    review_packet_dir: Option<&Path>,
    review_packet_check: Option<&Path>,
) -> Value {
    let packet_artifacts = review_packet_dir
        .map(|dir| {
            [
                ("comment", "comment.md"),
                ("review_map_md", "review-map.md"),
                ("review_map_json", "review-map.json"),
                ("evidence", "evidence.json"),
                ("manifest", "manifest.json"),
                ("cockpit", "cockpit.json"),
            ]
            .into_iter()
            .map(|(name, relative)| path_link(name, &dir.join(relative)))
            .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    json!({
        "schema": "tokmd.handoff_review_links.v1",
        "review_packet_dir": review_packet_dir.map(path_string),
        "review_packet_check": review_packet_check.map(|path| path_link("review_packet_check", path)),
        "artifacts": packet_artifacts,
        "semantics": {
            "kind": "external_links",
            "copied": false,
            "integrity_source": "cargo xtask review-packet-check"
        }
    })
}

fn proof_links_json(affected: Option<&Path>, proof_plan: Option<&Path>) -> Value {
    let mut artifacts = Vec::new();
    if let Some(path) = affected {
        artifacts.push(path_link("affected", path));
    }
    if let Some(path) = proof_plan {
        artifacts.push(path_link("proof_plan", path));
    }

    json!({
        "schema": "tokmd.handoff_proof_links.v1",
        "artifacts": artifacts,
        "semantics": {
            "kind": "external_links",
            "copied": false,
            "integrity_source": "linked proof artifacts"
        }
    })
}

fn path_link(name: &str, path: &Path) -> Value {
    let bytes = path
        .metadata()
        .ok()
        .filter(|metadata| metadata.is_file())
        .map(|metadata| metadata.len());

    json!({
        "name": name,
        "path": path_string(path),
        "exists": path.exists(),
        "bytes": bytes,
    })
}
