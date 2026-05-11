//! Gate mapping from cockpit evidence into sensor envelopes.

use tokmd_envelope::{GateItem, GateResults, Verdict};

use super::super::cockpit;

/// Map cockpit GateStatus to envelope Verdict.
pub(super) fn map_verdict(status: cockpit::GateStatus) -> Verdict {
    match status {
        cockpit::GateStatus::Pass => Verdict::Pass,
        cockpit::GateStatus::Warn => Verdict::Warn,
        cockpit::GateStatus::Fail => Verdict::Fail,
        cockpit::GateStatus::Skipped => Verdict::Skip,
        cockpit::GateStatus::Pending => Verdict::Pending,
    }
}

/// Map cockpit Evidence to envelope GateResults.
pub(super) fn map_gates(evidence: &cockpit::Evidence) -> GateResults {
    let mut items = Vec::new();

    items.push(
        GateItem::new("mutation", map_verdict(evidence.mutation.meta.status))
            .with_source("computed"),
    );

    if let Some(ref dc) = evidence.diff_coverage {
        items.push(
            GateItem::new("diff_coverage", map_verdict(dc.meta.status))
                .with_threshold(0.8, dc.coverage_pct)
                .with_source("computed"),
        );
    }

    if let Some(ref c) = evidence.contracts {
        let mut gate =
            GateItem::new("contracts", map_verdict(c.meta.status)).with_source("computed");
        if c.failures > 0 {
            gate = gate.with_reason(format!("{} sub-gate(s) failed", c.failures));
        }
        items.push(gate);
    }

    if let Some(ref sc) = evidence.supply_chain {
        items.push(
            GateItem::new("supply_chain", map_verdict(sc.meta.status)).with_source("computed"),
        );
    }

    if let Some(ref det) = evidence.determinism {
        items.push(
            GateItem::new("determinism", map_verdict(det.meta.status)).with_source("computed"),
        );
    }

    if let Some(ref cx) = evidence.complexity {
        items
            .push(GateItem::new("complexity", map_verdict(cx.meta.status)).with_source("computed"));
    }

    GateResults::new(map_verdict(evidence.overall_status), items)
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::cockpit::{
        CommitMatch, Evidence, EvidenceSource, GateMeta, GateStatus, MutationGate,
        MutationSurvivor, ScopeCoverage,
    };

    pub(crate) fn sample_scope() -> ScopeCoverage {
        ScopeCoverage {
            relevant: vec![],
            tested: vec![],
            ratio: 1.0,
            lines_relevant: None,
            lines_tested: None,
        }
    }

    pub(crate) fn sample_meta(status: GateStatus) -> GateMeta {
        GateMeta {
            status,
            source: EvidenceSource::RanLocal,
            commit_match: CommitMatch::Exact,
            scope: sample_scope(),
            evidence_commit: None,
            evidence_generated_at_ms: None,
        }
    }

    fn sample_mutation_gate(status: GateStatus) -> MutationGate {
        MutationGate {
            meta: sample_meta(status),
            survivors: vec![MutationSurvivor {
                file: "src/lib.rs".to_string(),
                line: 10,
                mutation: "replace".to_string(),
            }],
            killed: 0,
            timeout: 0,
            unviable: 0,
        }
    }

    pub(crate) fn base_evidence() -> Evidence {
        Evidence {
            overall_status: GateStatus::Warn,
            mutation: sample_mutation_gate(GateStatus::Warn),
            diff_coverage: None,
            contracts: None,
            supply_chain: None,
            determinism: None,
            complexity: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::cockpit::{
        ComplexityGate, ContractDiffGate, DeterminismGate, DiffCoverageGate, GateStatus,
        SupplyChainGate, UncoveredHunk,
    };
    use super::*;

    #[test]
    fn map_verdict_covers_all_gate_statuses() {
        assert_eq!(map_verdict(GateStatus::Pass), Verdict::Pass);
        assert_eq!(map_verdict(GateStatus::Warn), Verdict::Warn);
        assert_eq!(map_verdict(GateStatus::Fail), Verdict::Fail);
        assert_eq!(map_verdict(GateStatus::Skipped), Verdict::Skip);
        assert_eq!(map_verdict(GateStatus::Pending), Verdict::Pending);
    }

    #[test]
    fn map_gates_includes_optional_items_and_reasons() {
        let mut evidence = test_support::base_evidence();
        evidence.diff_coverage = Some(DiffCoverageGate {
            meta: test_support::sample_meta(GateStatus::Fail),
            lines_added: 10,
            lines_covered: 5,
            coverage_pct: 0.5,
            uncovered_hunks: vec![UncoveredHunk {
                file: "src/lib.rs".to_string(),
                start_line: 1,
                end_line: 3,
            }],
        });
        evidence.contracts = Some(ContractDiffGate {
            meta: test_support::sample_meta(GateStatus::Warn),
            semver: None,
            cli: None,
            schema: None,
            failures: 2,
        });
        evidence.supply_chain = Some(SupplyChainGate {
            meta: test_support::sample_meta(GateStatus::Pass),
            vulnerabilities: vec![],
            denied: vec![],
            advisory_db_version: None,
        });
        evidence.determinism = Some(DeterminismGate {
            meta: test_support::sample_meta(GateStatus::Warn),
            expected_hash: Some("abc".to_string()),
            actual_hash: Some("def".to_string()),
            algo: "blake3".to_string(),
            differences: vec!["target/app".to_string()],
        });
        evidence.complexity = Some(ComplexityGate {
            meta: test_support::sample_meta(GateStatus::Fail),
            files_analyzed: 1,
            high_complexity_files: vec![],
            avg_cyclomatic: 4.0,
            max_cyclomatic: 12,
            threshold_exceeded: true,
        });

        let gates = map_gates(&evidence);
        let ids: std::collections::BTreeSet<_> =
            gates.items.iter().map(|g| g.id.as_str()).collect();
        for id in [
            "mutation",
            "diff_coverage",
            "contracts",
            "supply_chain",
            "determinism",
            "complexity",
        ] {
            assert!(ids.contains(id), "missing gate {id}");
        }

        let diff_gate = gates
            .items
            .iter()
            .find(|g| g.id == "diff_coverage")
            .expect("diff_coverage gate should exist in GateResults");
        assert_eq!(diff_gate.threshold, Some(0.8));
        assert_eq!(diff_gate.actual, Some(0.5));

        let contracts_gate = gates
            .items
            .iter()
            .find(|g| g.id == "contracts")
            .expect("contracts gate should exist in GateResults");
        assert_eq!(
            contracts_gate.reason.as_deref(),
            Some("2 sub-gate(s) failed")
        );
    }
}
