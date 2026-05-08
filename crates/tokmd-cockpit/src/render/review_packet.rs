//! Cockpit review packet rendering.

use std::path::Path;

use anyhow::Result;
use serde_json::{Value, json};

use crate::{CockpitReceipt, CommitMatch, GateMeta, GateStatus, ReviewItem};

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

fn review_packet_evidence(receipt: &CockpitReceipt) -> Value {
    let gates: Vec<_> = review_packet_evidence_gate_specs(receipt)
        .into_iter()
        .map(|(id, meta)| evidence_gate(id, meta))
        .collect();

    json!({
        "schema": "tokmd.review_packet_evidence.v1",
        "overall_status": receipt.evidence.overall_status,
        "base_ref": receipt.base_ref,
        "head_ref": receipt.head_ref,
        "gates": gates,
    })
}

fn review_packet_evidence_summary(receipt: &CockpitReceipt) -> Value {
    let counts = evidence_counts(receipt);

    json!({
        "details": "evidence.json#/gates",
        "total_gates": counts.total_gates(),
        "available": counts.available,
        "degraded": counts.degraded,
        "stale": counts.stale,
        "skipped": counts.skipped,
        "unavailable": counts.unavailable,
        "missing": counts.missing,
    })
}

#[derive(Default)]
pub(super) struct EvidenceAvailabilityCounts {
    pub(super) available: usize,
    pub(super) degraded: usize,
    pub(super) stale: usize,
    pub(super) skipped: usize,
    pub(super) unavailable: usize,
    pub(super) missing: usize,
}

impl EvidenceAvailabilityCounts {
    fn total_gates(&self) -> usize {
        self.available + self.degraded + self.stale + self.skipped + self.unavailable + self.missing
    }
}

pub(super) fn evidence_counts(receipt: &CockpitReceipt) -> EvidenceAvailabilityCounts {
    let mut counts = EvidenceAvailabilityCounts::default();

    for (_, meta) in review_packet_evidence_gate_specs(receipt) {
        match evidence_availability_optional(meta) {
            "available" => counts.available += 1,
            "degraded" => counts.degraded += 1,
            "stale" => counts.stale += 1,
            "skipped" => counts.skipped += 1,
            "unavailable" => counts.unavailable += 1,
            "missing" => counts.missing += 1,
            _ => {}
        }
    }

    counts
}

fn review_packet_evidence_capabilities(receipt: &CockpitReceipt) -> Value {
    let mut available = Vec::new();
    let mut degraded = Vec::new();
    let mut stale = Vec::new();
    let mut skipped = Vec::new();
    let mut unavailable = Vec::new();
    let mut missing = Vec::new();

    for (id, meta) in review_packet_evidence_gate_specs(receipt) {
        match evidence_availability_optional(meta) {
            "available" => available.push(id),
            "degraded" => degraded.push(id),
            "stale" => stale.push(id),
            "skipped" => skipped.push(id),
            "unavailable" => unavailable.push(id),
            "missing" => missing.push(id),
            _ => {}
        }
    }

    json!({
        "details": "evidence.json#/gates",
        "available": available,
        "degraded": degraded,
        "stale": stale,
        "skipped": skipped,
        "unavailable": unavailable,
        "missing": missing,
    })
}

fn review_packet_evidence_gate_specs(
    receipt: &CockpitReceipt,
) -> [(&'static str, Option<&GateMeta>); 6] {
    [
        ("mutation", Some(&receipt.evidence.mutation.meta)),
        (
            "diff_coverage",
            receipt
                .evidence
                .diff_coverage
                .as_ref()
                .map(|gate| &gate.meta),
        ),
        (
            "contracts",
            receipt.evidence.contracts.as_ref().map(|gate| &gate.meta),
        ),
        (
            "supply_chain",
            receipt
                .evidence
                .supply_chain
                .as_ref()
                .map(|gate| &gate.meta),
        ),
        (
            "determinism",
            receipt.evidence.determinism.as_ref().map(|gate| &gate.meta),
        ),
        (
            "complexity",
            receipt.evidence.complexity.as_ref().map(|gate| &gate.meta),
        ),
    ]
}

fn review_packet_review_map(receipt: &CockpitReceipt) -> Value {
    let items: Vec<_> = receipt
        .review_plan
        .iter()
        .enumerate()
        .map(|(idx, item)| review_map_item(idx, item, receipt))
        .collect();

    json!({
        "schema": "tokmd.review_map.v1",
        "base_ref": receipt.base_ref,
        "head_ref": receipt.head_ref,
        "source": "cockpit.review_plan",
        "item_count": items.len(),
        "items": items,
    })
}

fn review_map_item(idx: usize, item: &ReviewItem, receipt: &CockpitReceipt) -> Value {
    json!({
        "rank": idx + 1,
        "path": &item.path,
        "priority": item.priority,
        "priority_label": review_priority_label(item.priority),
        "reason": &item.reason,
        "complexity": item.complexity,
        "lines_changed": item.lines_changed,
        "evidence_refs": [
            format!("cockpit.json#/review_plan/{idx}"),
            "evidence.json#/gates",
        ],
        "reproduce": [
            format!(
                "tokmd cockpit --base {} --head {} --format json",
                receipt.base_ref, receipt.head_ref
            ),
            format!(
                "tokmd cockpit --base {} --head {} --review-packet-dir .tokmd/review",
                receipt.base_ref, receipt.head_ref
            ),
        ],
    })
}

fn review_priority_label(priority: u32) -> &'static str {
    match priority {
        1 => "highest",
        2 => "medium",
        _ => "low",
    }
}

fn render_review_map_md(receipt: &CockpitReceipt) -> String {
    use std::fmt::Write;

    let mut s = String::new();
    let _ = writeln!(s, "# Review Map");
    let _ = writeln!(s);
    let _ = writeln!(s, "Base: `{}`", receipt.base_ref);
    let _ = writeln!(s, "Head: `{}`", receipt.head_ref);
    let _ = writeln!(s);

    if receipt.review_plan.is_empty() {
        let _ = writeln!(s, "No prioritized files were identified.");
        return s;
    }

    for (idx, item) in receipt.review_plan.iter().enumerate() {
        let _ = writeln!(
            s,
            "{}. `{}`
   Priority: {} ({})
   Reason: {}",
            idx + 1,
            item.path,
            item.priority,
            review_priority_label(item.priority),
            item.reason
        );

        if let Some(lines_changed) = item.lines_changed {
            let _ = writeln!(s, "   Lines changed: {lines_changed}");
        }
        if let Some(complexity) = item.complexity {
            let _ = writeln!(s, "   Review complexity: {complexity}/5");
        }
        let _ = writeln!(s, "   Evidence references:");
        let _ = writeln!(s, "   - cockpit.json#/review_plan/{idx}");
        let _ = writeln!(s, "   - evidence.json#/gates");
        let _ = writeln!(s, "   Reproduce:");
        let _ = writeln!(
            s,
            "   - `tokmd cockpit --base {} --head {} --format json`",
            receipt.base_ref, receipt.head_ref
        );
        let _ = writeln!(
            s,
            "   - `tokmd cockpit --base {} --head {} --review-packet-dir .tokmd/review`",
            receipt.base_ref, receipt.head_ref
        );
        let _ = writeln!(s);
    }

    s
}

fn evidence_gate(id: &str, meta: Option<&GateMeta>) -> Value {
    match meta {
        Some(meta) => json!({
            "id": id,
            "status": meta.status,
            "availability": evidence_availability(meta),
            "source": meta.source,
            "commit_match": meta.commit_match,
            "scope": {
                "relevant": &meta.scope.relevant,
                "tested": &meta.scope.tested,
                "ratio": meta.scope.ratio,
                "lines_relevant": meta.scope.lines_relevant,
                "lines_tested": meta.scope.lines_tested,
            },
            "evidence_commit": &meta.evidence_commit,
            "evidence_generated_at_ms": meta.evidence_generated_at_ms,
        }),
        None => json!({
            "id": id,
            "status": "unavailable",
            "availability": "unavailable",
            "source": null,
            "commit_match": null,
            "scope": {
                "relevant": [],
                "tested": [],
                "ratio": 0.0,
                "lines_relevant": null,
                "lines_tested": null,
            },
            "evidence_commit": null,
            "evidence_generated_at_ms": null,
        }),
    }
}

fn evidence_availability(meta: &GateMeta) -> &'static str {
    if matches!(meta.status, GateStatus::Skipped) {
        return "skipped";
    }

    if matches!(meta.status, GateStatus::Pending)
        && !meta.scope.relevant.is_empty()
        && meta.scope.tested.is_empty()
    {
        return "missing";
    }

    match meta.commit_match {
        CommitMatch::Exact => "available",
        CommitMatch::Partial | CommitMatch::Unknown => "degraded",
        CommitMatch::Stale => "stale",
    }
}

fn evidence_availability_optional(meta: Option<&GateMeta>) -> &'static str {
    match meta {
        Some(meta) => evidence_availability(meta),
        None => "unavailable",
    }
}
