//! Review-map artifact rendering for cockpit review packets.

use serde_json::{Value, json};

use crate::doc_artifacts_evidence::{DOC_ARTIFACTS_PACKET_PATH, DocArtifactsEvidenceInput};
use crate::proof_evidence::ProofEvidenceInput;
use crate::{CockpitReceipt, GateMeta, ReviewItem};

use super::evidence::{
    doc_artifacts_expected, evidence_availability_optional, evidence_counts,
    review_item_is_source_of_truth, review_packet_evidence_capabilities,
    review_packet_evidence_gate_specs, review_packet_evidence_summary,
};
use super::proof_summary::proof_evidence_summary;
use super::review_map_proof::{
    ReviewMapProofRef, review_map_item_proof, review_map_proof_refs, write_proof_block,
};

pub(super) fn review_packet_review_map(
    receipt: &CockpitReceipt,
    proof_inputs: &[ProofEvidenceInput],
    doc_artifacts: Option<&DocArtifactsEvidenceInput>,
) -> Value {
    let proof_refs = review_map_proof_refs(receipt, proof_inputs);
    let has_doc_artifacts_evidence = doc_artifacts.is_some() || doc_artifacts_expected(receipt);
    let evidence_refs =
        review_map_evidence_refs(!proof_refs.is_empty(), has_doc_artifacts_evidence);
    let items: Vec<_> = receipt
        .review_plan
        .iter()
        .enumerate()
        .map(|(idx, item)| review_map_item(idx, item, receipt, &proof_refs, doc_artifacts))
        .collect();

    json!({
        "schema": "tokmd.review_map.v1",
        "base_ref": receipt.base_ref,
        "head_ref": receipt.head_ref,
        "source": "cockpit.review_plan",
        "evidence": {
            "summary": review_packet_evidence_summary(receipt),
            "groups": review_packet_evidence_capabilities(receipt),
            "refs": evidence_refs,
        },
        "item_count": items.len(),
        "items": items,
    })
}

fn review_map_item(
    idx: usize,
    item: &ReviewItem,
    receipt: &CockpitReceipt,
    proof_refs: &[ReviewMapProofRef],
    doc_artifacts: Option<&DocArtifactsEvidenceInput>,
) -> Value {
    let evidence = review_map_item_evidence(item, receipt);
    let proof = review_map_item_proof(item, proof_refs);
    let doc_artifacts_refs = review_map_item_doc_artifacts_refs(item, doc_artifacts);

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
        "proof_refs": proof.refs,
        "doc_artifacts_refs": doc_artifacts_refs,
        "evidence": {
            "status": evidence.status(),
            "present": evidence.present,
            "missing": evidence.missing,
            "degraded": evidence.degraded,
            "stale": evidence.stale,
            "skipped": evidence.skipped,
            "unavailable": evidence.unavailable,
            "refs": ["evidence.json#/gates"],
        },
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

fn review_map_evidence_refs(
    has_proof: bool,
    has_doc_artifacts_evidence: bool,
) -> Vec<&'static str> {
    let mut refs = vec!["evidence.json#/gates"];
    if has_proof {
        refs.push("evidence.json#/proof");
    }
    if has_doc_artifacts_evidence {
        refs.push("evidence.json#/doc_artifacts");
    }
    refs
}

fn review_map_item_doc_artifacts_refs(
    item: &ReviewItem,
    doc_artifacts: Option<&DocArtifactsEvidenceInput>,
) -> Vec<&'static str> {
    if doc_artifacts.is_some() && review_item_is_source_of_truth(item) {
        vec!["evidence.json#/doc_artifacts", DOC_ARTIFACTS_PACKET_PATH]
    } else {
        Vec::new()
    }
}

#[derive(Default)]
struct ReviewMapItemEvidence {
    present: Vec<&'static str>,
    missing: Vec<&'static str>,
    degraded: Vec<&'static str>,
    stale: Vec<&'static str>,
    skipped: Vec<&'static str>,
    unavailable: Vec<&'static str>,
}

impl ReviewMapItemEvidence {
    fn status(&self) -> &'static str {
        if !self.missing.is_empty() {
            "missing"
        } else if !self.stale.is_empty() {
            "stale"
        } else if !self.degraded.is_empty() {
            "degraded"
        } else if !self.present.is_empty() {
            "available"
        } else if !self.skipped.is_empty() {
            "skipped"
        } else {
            "unavailable"
        }
    }
}

fn review_map_item_evidence(item: &ReviewItem, receipt: &CockpitReceipt) -> ReviewMapItemEvidence {
    let mut evidence = ReviewMapItemEvidence::default();

    for (id, meta) in review_packet_evidence_gate_specs(receipt) {
        if !evidence_gate_applies_to_item(meta, item) {
            continue;
        }

        match evidence_availability_optional(meta) {
            "available" => evidence.present.push(id),
            "missing" => evidence.missing.push(id),
            "degraded" => evidence.degraded.push(id),
            "stale" => evidence.stale.push(id),
            "skipped" => evidence.skipped.push(id),
            "unavailable" => evidence.unavailable.push(id),
            _ => {}
        }
    }

    evidence
}

fn evidence_gate_applies_to_item(meta: Option<&GateMeta>, item: &ReviewItem) -> bool {
    let Some(meta) = meta else {
        return false;
    };

    let is_global = meta.scope.relevant.is_empty() && meta.scope.tested.is_empty();
    is_global
        || meta.scope.relevant.iter().any(|path| path == &item.path)
        || meta.scope.tested.iter().any(|path| path == &item.path)
}

fn review_priority_label(priority: u32) -> &'static str {
    match priority {
        1 => "highest",
        2 => "medium",
        _ => "low",
    }
}

pub(super) fn render_review_map_md(
    receipt: &CockpitReceipt,
    proof_inputs: &[ProofEvidenceInput],
    doc_artifacts: Option<&DocArtifactsEvidenceInput>,
) -> String {
    use std::fmt::Write;

    let mut s = String::new();
    let proof_refs = review_map_proof_refs(receipt, proof_inputs);
    let _ = writeln!(s, "# Review Map");
    let _ = writeln!(s);
    let _ = writeln!(s, "Base: `{}`", receipt.base_ref);
    let _ = writeln!(s, "Head: `{}`", receipt.head_ref);
    let _ = writeln!(s);

    let evidence = evidence_counts(receipt);
    let _ = writeln!(
        s,
        "Evidence overview: {} available, {} degraded, {} stale, {} skipped, {} unavailable, {} missing.",
        evidence.available,
        evidence.degraded,
        evidence.stale,
        evidence.skipped,
        evidence.unavailable,
        evidence.missing,
    );
    let _ = writeln!(s);
    write_proof_overview(&mut s, receipt, proof_inputs);
    write_doc_artifacts_overview(&mut s, receipt, doc_artifacts);

    if receipt.review_plan.is_empty() {
        let _ = writeln!(s, "No prioritized files were identified.");
        return s;
    }

    let _ = writeln!(s, "## Review First");
    let _ = writeln!(s);

    for (idx, item) in receipt.review_plan.iter().enumerate() {
        let evidence = review_map_item_evidence(item, receipt);
        let proof = review_map_item_proof(item, &proof_refs);
        let _ = writeln!(
            s,
            "{}. `{}`
   Priority: {} ({})
   Why it matters: {}",
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
        let _ = writeln!(s, "   Evidence status: {}", evidence.status());
        write_evidence_list(&mut s, "Evidence present", &evidence.present);
        write_evidence_list(&mut s, "Evidence missing", &evidence.missing);
        write_evidence_list(&mut s, "Evidence degraded", &evidence.degraded);
        write_evidence_list(&mut s, "Evidence stale", &evidence.stale);
        write_evidence_list(&mut s, "Evidence skipped", &evidence.skipped);
        write_evidence_list(&mut s, "Evidence unavailable", &evidence.unavailable);
        write_doc_artifacts_block(&mut s, item, doc_artifacts);
        write_proof_block(&mut s, &proof);
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

fn write_doc_artifacts_overview(
    s: &mut String,
    receipt: &CockpitReceipt,
    doc_artifacts: Option<&DocArtifactsEvidenceInput>,
) {
    use std::fmt::Write;

    match doc_artifacts {
        Some(input) => {
            let _ = writeln!(
                s,
                "Doc artifacts: {} ({} required docs, {} family files, {} active goals).",
                if input.receipt.ok {
                    "verified"
                } else {
                    "degraded"
                },
                input.receipt.checked.required_docs,
                input.receipt.checked.family_files,
                input.receipt.checked.active_goals,
            );
            if !input.receipt.errors.is_empty() {
                let _ = writeln!(s, "- Errors: {}", input.receipt.errors.len());
            }
            let _ = writeln!(s);
        }
        None if doc_artifacts_expected(receipt) => {
            let _ = writeln!(s, "Doc artifacts: missing for source-of-truth changes.");
            let _ = writeln!(s);
        }
        None => {}
    }
}

fn write_doc_artifacts_block(
    s: &mut String,
    item: &ReviewItem,
    doc_artifacts: Option<&DocArtifactsEvidenceInput>,
) {
    use std::fmt::Write;

    if !review_item_is_source_of_truth(item) {
        return;
    }

    match doc_artifacts {
        Some(input) => {
            let _ = writeln!(
                s,
                "   Doc artifacts: {}",
                if input.receipt.ok {
                    "verified"
                } else {
                    "degraded"
                }
            );
            let _ = writeln!(s, "   - evidence.json#/doc_artifacts");
            let _ = writeln!(s, "   - {DOC_ARTIFACTS_PACKET_PATH}");
        }
        None => {
            let _ = writeln!(s, "   Doc artifacts: missing");
        }
    }
}

fn write_proof_overview(
    s: &mut String,
    receipt: &CockpitReceipt,
    proof_inputs: &[ProofEvidenceInput],
) {
    use std::fmt::Write;

    let counts = proof_evidence_summary(receipt, proof_inputs);
    if counts.total == 0 {
        return;
    }

    let _ = writeln!(s, "Proof evidence overview:");
    let _ = writeln!(
        s,
        "- Required proof: {} passed, {} failed, {} missing",
        counts.required_passed, counts.required_failed, counts.required_missing,
    );
    let _ = writeln!(
        s,
        "- Advisory proof: {} available, {} missing",
        counts.advisory_available, counts.advisory_missing,
    );
    let _ = writeln!(
        s,
        "- Freshness: {} exact, {} partial, {} stale, {} unknown",
        counts.exact, counts.partial, counts.stale, counts.unknown,
    );
    if counts.not_run > 0 {
        let _ = writeln!(s, "- Not run: {}", counts.not_run);
    }
    if counts.degraded > 0 || counts.skipped > 0 || counts.unavailable > 0 {
        let _ = writeln!(
            s,
            "- Other proof state: {} degraded, {} skipped, {} unavailable",
            counts.degraded, counts.skipped, counts.unavailable,
        );
    }
    let _ = writeln!(s);
}

fn write_evidence_list(s: &mut String, label: &str, gates: &[&str]) {
    use std::fmt::Write;

    if gates.is_empty() {
        return;
    }

    let _ = writeln!(s, "   {label}: {}", gates.join(", "));
}
