//! Linked-evidence summary rendering for the handoff work order.

use std::fs;
use std::path::Path;

use serde_json::Value;

mod affected;
mod proof_plan;
mod proof_route;
mod review_map;
mod review_packet_check;

pub(in crate::commands::handoff) use affected::AffectedSummary;
pub(in crate::commands::handoff) use proof_plan::ProofPlanSummary;
pub(in crate::commands::handoff) use proof_route::ProofRouteSummary;
pub(in crate::commands::handoff) use review_map::ReviewMapSummary;
pub(in crate::commands::handoff) use review_packet_check::ReviewPacketCheckSummary;

#[derive(Default)]
pub(in crate::commands::handoff) struct LinkedEvidenceSummary {
    pub(in crate::commands::handoff) review_map: Option<ReviewMapSummary>,
    pub(in crate::commands::handoff) review_packet_check: Option<ReviewPacketCheckSummary>,
    pub(in crate::commands::handoff) affected: Option<AffectedSummary>,
    pub(in crate::commands::handoff) proof_plan: Option<ProofPlanSummary>,
    pub(in crate::commands::handoff) proof_route: Option<ProofRouteSummary>,
}

pub(super) fn summarize(links: &super::HandoffLinkInputs<'_>) -> LinkedEvidenceSummary {
    LinkedEvidenceSummary {
        review_map: links
            .review_packet_dir
            .and_then(|dir| read_json_value(&dir.join("review-map.json")))
            .and_then(|value| review_map::summarize(&value)),
        review_packet_check: links
            .review_packet_check
            .and_then(read_json_value)
            .map(|value| review_packet_check::summarize(&value)),
        affected: links
            .affected
            .and_then(read_json_value)
            .map(|value| affected::summarize(&value)),
        proof_plan: links
            .proof_plan
            .and_then(read_json_value)
            .map(|value| proof_plan::summarize(&value)),
        proof_route: links
            .proof_route
            .and_then(read_json_value)
            .map(|value| proof_route::summarize(&value)),
    }
}

pub(super) fn render(
    out: &mut String,
    links: &super::HandoffLinkInputs<'_>,
    summary: &LinkedEvidenceSummary,
) {
    if !has_any_link(links) {
        return;
    }

    out.push_str("\n## Linked Evidence Summary\n\n");
    out.push_str("These summaries are best-effort hints from linked receipts. They do not replace the linked verifier or proof artifacts.\n\n");

    if let Some(check) = &summary.review_packet_check {
        review_packet_check::render(out, check);
    } else if links.review_packet_check.is_some() {
        out.push_str("- Review packet verifier: linked but not readable\n");
    }

    if let Some(review_map) = &summary.review_map {
        review_map::render(out, review_map);
    } else if links.review_packet_dir.is_some() {
        out.push_str("- Review map: linked but not readable\n");
    }

    if let Some(affected) = &summary.affected {
        affected::render(out, affected);
    } else if links.affected.is_some() {
        out.push_str("- Affected proof: linked but not readable\n");
    }

    if let Some(proof_route) = &summary.proof_route {
        proof_route::render(out, proof_route);
    } else if links.proof_route.is_some() {
        out.push_str("- Proof route: linked but not readable\n");
    }

    if let Some(proof_plan) = &summary.proof_plan {
        proof_plan::render(out, proof_plan);
    } else if links.proof_plan.is_some() {
        out.push_str("- Proof plan: linked but not readable\n");
    }
}

fn has_any_link(links: &super::HandoffLinkInputs<'_>) -> bool {
    links.review_packet_dir.is_some()
        || links.review_packet_check.is_some()
        || links.affected.is_some()
        || links.proof_plan.is_some()
        || links.proof_route.is_some()
}

fn read_json_value(path: &Path) -> Option<Value> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}
