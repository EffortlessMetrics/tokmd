//! Evidence artifact and availability helpers for cockpit review packets.

use serde_json::{Value, json};

use crate::{CockpitReceipt, CommitMatch, GateMeta, GateStatus};

pub(super) fn review_packet_evidence(receipt: &CockpitReceipt) -> Value {
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
