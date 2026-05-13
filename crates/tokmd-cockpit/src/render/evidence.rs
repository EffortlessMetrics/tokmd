//! Evidence artifact and availability helpers for cockpit review packets.

use serde_json::{Value, json};

use crate::doc_artifacts_evidence::{
    DOC_ARTIFACTS_CHECK_SCHEMA, DOC_ARTIFACTS_PACKET_PATH, DocArtifactsEvidenceInput,
    source_of_truth_path,
};
use crate::proof_evidence::{ProofEvidenceInput, normalize_proof_evidence_inputs};
use crate::{CockpitReceipt, CommitMatch, GateMeta, GateStatus, ReviewItem};

pub(super) fn review_packet_evidence(
    receipt: &CockpitReceipt,
    proof_inputs: &[ProofEvidenceInput],
    doc_artifacts: Option<&DocArtifactsEvidenceInput>,
) -> Value {
    let gates: Vec<_> = review_packet_evidence_gate_specs(receipt)
        .into_iter()
        .map(|(id, meta)| evidence_gate(id, meta))
        .collect();

    let mut evidence = json!({
        "schema": "tokmd.review_packet_evidence.v1",
        "overall_status": receipt.evidence.overall_status,
        "base_ref": receipt.base_ref,
        "head_ref": receipt.head_ref,
        "gates": gates,
    });

    let proof = review_packet_proof_evidence(receipt, proof_inputs);
    if !proof.is_empty() {
        evidence["proof"] = Value::Array(proof);
    }
    if let Some(doc_artifacts) = review_packet_doc_artifacts_evidence(receipt, doc_artifacts) {
        evidence["doc_artifacts"] = doc_artifacts;
    }

    evidence
}

pub(super) fn review_packet_doc_artifacts_evidence(
    receipt: &CockpitReceipt,
    doc_artifacts: Option<&DocArtifactsEvidenceInput>,
) -> Option<Value> {
    match doc_artifacts {
        Some(input) => Some(json!({
            "source": normalize_path_for_output(&input.source_path),
            "source_schema": input.receipt.schema,
            "ok": input.receipt.ok,
            "availability": input.availability(),
            "checked": {
                "required_docs": input.receipt.checked.required_docs,
                "family_files": input.receipt.checked.family_files,
                "active_goals": input.receipt.checked.active_goals,
            },
            "errors": input.receipt.errors,
            "refs": [format!("{DOC_ARTIFACTS_PACKET_PATH}")],
        })),
        None if doc_artifacts_expected(receipt) => Some(json!({
            "source": null,
            "source_schema": DOC_ARTIFACTS_CHECK_SCHEMA,
            "ok": null,
            "availability": "missing",
            "checked": null,
            "errors": [],
            "refs": [],
        })),
        None => None,
    }
}

pub(super) fn doc_artifacts_expected(receipt: &CockpitReceipt) -> bool {
    receipt
        .review_plan
        .iter()
        .any(review_item_is_source_of_truth)
}

pub(super) fn review_item_is_source_of_truth(item: &ReviewItem) -> bool {
    source_of_truth_path(&item.path)
}

fn review_packet_proof_evidence(
    receipt: &CockpitReceipt,
    proof_inputs: &[ProofEvidenceInput],
) -> Vec<Value> {
    normalize_proof_evidence_inputs(
        proof_inputs,
        Some(&receipt.base_ref),
        Some(&receipt.head_ref),
    )
    .into_iter()
    .map(|item| {
        json!({
            "kind": item.kind.as_str(),
            "source": normalize_path_for_output(&item.source_path),
            "source_schema": item.source_schema,
            "profile": item.profile,
            "scope": item.scope,
            "command": item.command,
            "required": item.required,
            "advisory": item.advisory,
            "execution_status": item.execution_status.as_str(),
            "availability": item.availability.as_str(),
            "commit_match": item.commit_match,
            "refs": item.artifact_refs,
        })
    })
    .collect()
}

fn normalize_path_for_output(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub(super) fn review_packet_evidence_summary(receipt: &CockpitReceipt) -> Value {
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

pub(super) fn review_packet_evidence_capabilities(receipt: &CockpitReceipt) -> Value {
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

pub(super) fn review_packet_evidence_gate_specs(
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

pub(super) fn evidence_availability_optional(meta: Option<&GateMeta>) -> &'static str {
    match meta {
        Some(meta) => evidence_availability(meta),
        None => "unavailable",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EvidenceSource, ScopeCoverage};

    fn gate_meta(status: GateStatus, relevant: &[&str], tested: &[&str]) -> GateMeta {
        GateMeta {
            status,
            source: EvidenceSource::Cached,
            commit_match: CommitMatch::Unknown,
            scope: ScopeCoverage {
                relevant: relevant.iter().map(|path| (*path).to_string()).collect(),
                tested: tested.iter().map(|path| (*path).to_string()).collect(),
                ratio: 0.0,
                lines_relevant: None,
                lines_tested: None,
            },
            evidence_commit: None,
            evidence_generated_at_ms: None,
        }
    }

    #[test]
    fn absent_optional_gate_is_unavailable() {
        assert_eq!(evidence_availability_optional(None), "unavailable");
    }

    #[test]
    fn pending_relevant_gate_without_tested_scope_is_missing() {
        let meta = gate_meta(GateStatus::Pending, &["src/lib.rs"], &[]);

        assert_eq!(evidence_availability_optional(Some(&meta)), "missing");
    }

    #[test]
    fn skipped_gate_is_skipped_even_when_scope_is_untested() {
        let meta = gate_meta(GateStatus::Skipped, &["src/lib.rs"], &[]);

        assert_eq!(evidence_availability_optional(Some(&meta)), "skipped");
    }
}
