//! Review-map artifact rendering for cockpit review packets.

use std::collections::BTreeSet;

use serde_json::{Value, json};

use crate::proof_evidence::{
    NormalizedProofEvidence, ProofEvidenceAvailability, ProofEvidenceInput, ProofExecutionStatus,
    normalize_proof_evidence,
};
use crate::{CockpitReceipt, CommitMatch, GateMeta, ReviewItem};

use super::evidence::{
    evidence_availability_optional, evidence_counts, review_packet_evidence_capabilities,
    review_packet_evidence_gate_specs, review_packet_evidence_summary,
};

pub(super) fn review_packet_review_map(
    receipt: &CockpitReceipt,
    proof_inputs: &[ProofEvidenceInput],
) -> Value {
    let proof_refs = review_map_proof_refs(receipt, proof_inputs);
    let evidence_refs = review_map_evidence_refs(!proof_refs.is_empty());
    let items: Vec<_> = receipt
        .review_plan
        .iter()
        .enumerate()
        .map(|(idx, item)| review_map_item(idx, item, receipt, &proof_refs))
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
) -> Value {
    let evidence = review_map_item_evidence(item, receipt);
    let proof = review_map_item_proof(item, proof_refs);

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

struct ReviewMapProofRef {
    changed_files: BTreeSet<String>,
    refs: Vec<String>,
    line: String,
}

fn review_map_proof_refs(
    receipt: &CockpitReceipt,
    proof_inputs: &[ProofEvidenceInput],
) -> Vec<ReviewMapProofRef> {
    let mut proof_refs = Vec::new();
    let mut proof_index = 0;

    for input in proof_inputs {
        let changed_files = input
            .artifact
            .changed_files()
            .iter()
            .map(|path| normalize_path_for_match(path))
            .collect::<BTreeSet<_>>();
        let normalized = normalize_proof_evidence(
            &input.artifact,
            input.source_path.clone(),
            Some(&receipt.base_ref),
            Some(&receipt.head_ref),
        );

        for proof in normalized {
            let mut refs = Vec::with_capacity(1 + proof.artifact_refs.len());
            refs.push(format!("evidence.json#/proof/{proof_index}"));
            refs.extend(proof.artifact_refs.iter().cloned());
            proof_refs.push(ReviewMapProofRef {
                changed_files: changed_files.clone(),
                refs,
                line: review_map_proof_line(&proof),
            });
            proof_index += 1;
        }
    }

    proof_refs
}

fn review_map_evidence_refs(has_proof: bool) -> Vec<&'static str> {
    let mut refs = vec!["evidence.json#/gates"];
    if has_proof {
        refs.push("evidence.json#/proof");
    }
    refs
}

struct ReviewMapItemProof {
    lines: Vec<String>,
    refs: Vec<String>,
}

fn review_map_item_proof(
    item: &ReviewItem,
    proof_refs: &[ReviewMapProofRef],
) -> ReviewMapItemProof {
    let item_path = normalize_path_for_match(&item.path);
    let mut refs = BTreeSet::new();
    let mut seen_lines = BTreeSet::new();
    let mut lines = Vec::new();

    for proof_ref in proof_refs {
        if !proof_ref.changed_files.contains(&item_path) {
            continue;
        }

        refs.extend(proof_ref.refs.iter().cloned());
        if seen_lines.insert(proof_ref.line.clone()) {
            lines.push(proof_ref.line.clone());
        }
    }

    ReviewMapItemProof {
        lines,
        refs: refs.into_iter().collect(),
    }
}

fn review_map_proof_line(proof: &NormalizedProofEvidence) -> String {
    let class = if proof.required {
        "Required"
    } else {
        "Advisory"
    };
    let scope = proof
        .scope
        .as_deref()
        .unwrap_or_else(|| proof.kind.as_str());
    let mut line = format!(
        "{}: {} {} ({}, freshness: {})",
        class,
        scope,
        execution_status_label(proof.execution_status),
        availability_label(proof.availability),
        commit_match_label(proof.commit_match),
    );

    if let Some(command) = proof
        .command
        .as_deref()
        .filter(|command| !command.is_empty())
    {
        line.push_str(" - ");
        line.push_str(command);
    }

    line
}

fn execution_status_label(status: ProofExecutionStatus) -> &'static str {
    match status {
        ProofExecutionStatus::Planned => "planned",
        ProofExecutionStatus::ExecutedPassed => "passed",
        ProofExecutionStatus::ExecutedFailed => "failed",
        ProofExecutionStatus::NotExecuted => "not run",
        ProofExecutionStatus::DryRun => "dry run",
    }
}

fn availability_label(availability: ProofEvidenceAvailability) -> &'static str {
    match availability {
        ProofEvidenceAvailability::Available => "available",
        ProofEvidenceAvailability::Missing => "missing",
        ProofEvidenceAvailability::Skipped => "skipped",
        ProofEvidenceAvailability::Stale => "stale",
        ProofEvidenceAvailability::Degraded => "degraded",
        ProofEvidenceAvailability::Unavailable => "unavailable",
    }
}

fn commit_match_label(commit_match: CommitMatch) -> &'static str {
    match commit_match {
        CommitMatch::Exact => "exact",
        CommitMatch::Partial => "partial",
        CommitMatch::Stale => "stale",
        CommitMatch::Unknown => "unknown",
    }
}

fn normalize_path_for_match(path: &str) -> String {
    path.replace('\\', "/")
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

fn write_evidence_list(s: &mut String, label: &str, gates: &[&str]) {
    use std::fmt::Write;

    if gates.is_empty() {
        return;
    }

    let _ = writeln!(s, "   {label}: {}", gates.join(", "));
}

fn write_proof_block(s: &mut String, proof: &ReviewMapItemProof) {
    use std::fmt::Write;

    if proof.lines.is_empty() {
        return;
    }

    let _ = writeln!(s, "   Proof:");
    for line in &proof.lines {
        let _ = writeln!(s, "   - {line}");
    }
    let _ = writeln!(s, "   Proof references:");
    for reference in &proof.refs {
        let _ = writeln!(s, "   - {reference}");
    }
}
