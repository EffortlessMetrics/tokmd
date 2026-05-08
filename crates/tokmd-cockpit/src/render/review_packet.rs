//! Cockpit review packet rendering.

use std::path::Path;

use anyhow::Result;

use crate::CockpitReceipt;

use super::evidence::review_packet_evidence;
use super::manifest::review_packet_manifest;
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
