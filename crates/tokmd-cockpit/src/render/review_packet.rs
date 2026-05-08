//! Cockpit review packet rendering.

use std::path::Path;

use anyhow::Result;
use serde_json::{Value, json};

use crate::CockpitReceipt;

use super::evidence::{
    review_packet_evidence, review_packet_evidence_capabilities, review_packet_evidence_summary,
};
use super::review_map::{render_review_map_md, review_packet_review_map};
use super::{render_comment_md, render_json};

/// Write review packet artifacts to directory.
///
/// This is the doc-first packet contract from `docs/review-packet.md`. It is
/// intentionally separate from `write_artifacts` so existing cockpit
/// integrations keep their shipped `cockpit.json` / `report.json` /
/// `comment.md` artifact shape until they opt into packet emission.
pub fn write_review_packet(dir: &Path, receipt: &CockpitReceipt) -> Result<()> {
    std::fs::create_dir_all(dir)?;

    let cockpit_json = render_json(receipt)?;
    let evidence_json = serde_json::to_string_pretty(&review_packet_evidence(receipt))?;
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

fn render_review_packet_comment_md(receipt: &CockpitReceipt) -> String {
    use std::fmt::Write;

    let mut s = render_comment_md(receipt);
    let _ = writeln!(s, "**Review packet artifacts**:");
    let _ = writeln!(s, "- [Evidence gates](evidence.json)");
    let _ = writeln!(s, "- [Review map](review-map.md)");
    let _ = writeln!(s, "- [Full cockpit receipt](cockpit.json)");
    let _ = writeln!(s);
    s
}

fn review_packet_manifest(
    receipt: &CockpitReceipt,
    cockpit_json: &str,
    evidence_json: &str,
    review_map_json: &str,
    review_map_md: &str,
    comment_md: &str,
) -> Value {
    let evidence_summary = review_packet_evidence_summary(receipt);
    let evidence_capabilities = review_packet_evidence_capabilities(receipt);

    json!({
        "schema": "tokmd.review_packet_manifest.v1",
        "generated_by": {
            "name": "tokmd",
            "version": env!("CARGO_PKG_VERSION"),
            "mode": "cockpit",
            "arguments": ["cockpit", "--review-packet-dir"],
        },
        "generated_at_ms": receipt.generated_at_ms,
        "base_ref": receipt.base_ref,
        "head_ref": receipt.head_ref,
        "verdict": {
            "status": receipt.evidence.overall_status,
            "blocking": false,
            "reason": "cockpit review packets are advisory by default",
            "evidence": evidence_summary,
        },
        "capabilities": {
            "evidence": evidence_capabilities,
        },
        "artifacts": [
            review_packet_artifact(
                "cockpit",
                "cockpit.json",
                "tokmd.cockpit_receipt.v3",
                "application/json",
                cockpit_json,
            ),
            review_packet_artifact(
                "evidence",
                "evidence.json",
                "tokmd.review_packet_evidence.v1",
                "application/json",
                evidence_json,
            ),
            review_packet_artifact(
                "review-map",
                "review-map.json",
                "tokmd.review_map.v1",
                "application/json",
                review_map_json,
            ),
            review_packet_artifact(
                "review-map-md",
                "review-map.md",
                "markdown",
                "text/markdown",
                review_map_md,
            ),
            review_packet_artifact(
                "comment",
                "comment.md",
                "markdown",
                "text/markdown",
                comment_md,
            ),
        ],
    })
}

fn review_packet_artifact(
    id: &str,
    path: &str,
    schema: &str,
    media_type: &str,
    content: &str,
) -> Value {
    json!({
        "id": id,
        "path": path,
        "schema": schema,
        "media_type": media_type,
        "hash": {
            "algo": "blake3",
            "hash": blake3::hash(content.as_bytes()).to_hex().to_string(),
        },
    })
}
