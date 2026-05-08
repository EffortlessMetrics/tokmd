//! Manifest artifact construction for cockpit review packets.

use serde_json::{Value, json};

use crate::CockpitReceipt;

use super::evidence::{review_packet_evidence_capabilities, review_packet_evidence_summary};

pub(super) fn review_packet_manifest(
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
