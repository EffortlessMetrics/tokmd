//! PR comment rendering for cockpit receipts and review packets.

use crate::proof_evidence::{
    ProofEvidenceAvailability, ProofEvidenceInput, ProofExecutionStatus,
    normalize_proof_evidence_inputs,
};
use crate::{CockpitReceipt, CommitMatch, GateStatus, RiskLevel};

use super::evidence::evidence_counts;

/// Render comment.md for PR comments.
pub fn render_comment_md(receipt: &CockpitReceipt) -> String {
    use std::fmt::Write;
    let mut s = String::new();

    // Summary bullet points
    let _ = writeln!(s, "## Glass Cockpit Summary");
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "- **{} files changed**, +{}/-{}",
        receipt.change_surface.files_changed,
        receipt.change_surface.insertions,
        receipt.change_surface.deletions
    );
    let _ = writeln!(
        s,
        "- **Health**: {}/100 ({})",
        receipt.code_health.score, receipt.code_health.grade
    );
    let _ = writeln!(
        s,
        "- **Risk**: {} ({}/100)",
        receipt.risk.level, receipt.risk.score
    );
    let _ = writeln!(s);

    // Contract changes
    if receipt.contracts.api_changed
        || receipt.contracts.cli_changed
        || receipt.contracts.schema_changed
    {
        let _ = writeln!(s, "**Contract changes**:");
        if receipt.contracts.api_changed {
            let _ = writeln!(s, "- API contract changed");
        }
        if receipt.contracts.cli_changed {
            let _ = writeln!(s, "- CLI contract changed");
        }
        if receipt.contracts.schema_changed {
            let _ = writeln!(s, "- Schema contract changed");
        }
        if receipt.contracts.breaking_indicators > 0 {
            let _ = writeln!(
                s,
                "- {} breaking indicator(s)",
                receipt.contracts.breaking_indicators
            );
        }
        let _ = writeln!(s);
    }

    // Evidence gates
    let _ = writeln!(
        s,
        "**Evidence gates**: {:?}",
        receipt.evidence.overall_status
    );
    let availability = evidence_counts(receipt);
    let _ = writeln!(
        s,
        "- **Evidence availability**: {} available, {} degraded, {} stale, {} skipped, {} unavailable, {} missing",
        availability.available,
        availability.degraded,
        availability.stale,
        availability.skipped,
        availability.unavailable,
        availability.missing,
    );
    if !receipt.evidence.mutation.survivors.is_empty() {
        let _ = writeln!(
            s,
            "- Mutation: {} survivors detected",
            receipt.evidence.mutation.survivors.len()
        );
    }
    if let Some(ref dc) = receipt.evidence.diff_coverage {
        let _ = writeln!(s, "- Diff coverage: {:.1}%", dc.coverage_pct * 100.0);
    }
    if let Some(ref contracts) = receipt.evidence.contracts
        && contracts.failures > 0
    {
        let _ = writeln!(s, "- Contracts: {} sub-gate(s) failed", contracts.failures);
    }
    if let Some(ref sc) = receipt.evidence.supply_chain
        && !sc.vulnerabilities.is_empty()
    {
        let _ = writeln!(
            s,
            "- Supply chain: {} vulnerability/vulnerabilities",
            sc.vulnerabilities.len()
        );
    }
    if let Some(ref cx) = receipt.evidence.complexity
        && cx.threshold_exceeded
    {
        let _ = writeln!(
            s,
            "- Complexity: threshold exceeded (max cyclomatic: {})",
            cx.max_cyclomatic
        );
    }
    let _ = writeln!(s);

    // Suggested next steps for PR authors and reviewers.
    let _ = writeln!(s, "**Next steps**:");
    match receipt.evidence.overall_status {
        GateStatus::Fail => {
            let _ = writeln!(s, "- [ ] Address failing evidence gates before merge");
        }
        GateStatus::Warn => {
            let _ = writeln!(
                s,
                "- [ ] Review warning evidence gates and capture risk acceptance"
            );
        }
        GateStatus::Pass => {
            let _ = writeln!(s, "- [ ] Proceed with reviewer sign-off");
        }
        GateStatus::Skipped | GateStatus::Pending => {
            let _ = writeln!(
                s,
                "- [ ] Capture missing or pending evidence before relying on this packet"
            );
        }
    }
    if receipt.contracts.breaking_indicators > 0 {
        let _ = writeln!(s, "- [ ] Confirm breaking changes are documented");
    }
    if matches!(receipt.risk.level, RiskLevel::High | RiskLevel::Critical) {
        let _ = writeln!(s, "- [ ] Add a domain reviewer for high-risk files");
    }
    let _ = writeln!(s);

    // Review plan (priority items only)
    let priority_items: Vec<_> = receipt
        .review_plan
        .iter()
        .filter(|item| item.priority <= 2)
        .collect();

    if !priority_items.is_empty() {
        let _ = writeln!(s, "**Priority review items**:");
        for item in priority_items {
            let _ = writeln!(s, "- {} ({})", item.path, item.reason);
        }
        let _ = writeln!(s);
    }

    s
}

pub(super) fn render_review_packet_comment_md(
    receipt: &CockpitReceipt,
    proof_inputs: &[ProofEvidenceInput],
) -> String {
    use std::fmt::Write;

    let mut s = render_comment_md(receipt);
    write_proof_evidence_summary(&mut s, receipt, proof_inputs);
    let _ = writeln!(s, "**Review packet artifacts**:");
    let _ = writeln!(s, "- [Evidence gates](evidence.json)");
    let _ = writeln!(s, "- [Review map](review-map.md)");
    let _ = writeln!(s, "- [Full cockpit receipt](cockpit.json)");
    let _ = writeln!(s);
    s
}

#[derive(Default)]
struct ProofCommentCounts {
    total: usize,
    required_passed: usize,
    required_failed: usize,
    required_missing: usize,
    advisory_available: usize,
    advisory_missing: usize,
    exact: usize,
    partial: usize,
    stale: usize,
    unknown: usize,
    not_run: usize,
    degraded: usize,
    skipped: usize,
    unavailable: usize,
}

fn write_proof_evidence_summary(
    s: &mut String,
    receipt: &CockpitReceipt,
    proof_inputs: &[ProofEvidenceInput],
) {
    use std::fmt::Write;

    let counts = proof_comment_counts(receipt, proof_inputs);
    if counts.total == 0 {
        return;
    }

    let _ = writeln!(s, "**Proof evidence**:");
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
        "- Proof freshness: {} exact, {} partial, {} stale, {} unknown",
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

fn proof_comment_counts(
    receipt: &CockpitReceipt,
    proof_inputs: &[ProofEvidenceInput],
) -> ProofCommentCounts {
    let mut counts = ProofCommentCounts::default();

    for item in normalize_proof_evidence_inputs(
        proof_inputs,
        Some(&receipt.base_ref),
        Some(&receipt.head_ref),
    ) {
        counts.total += 1;

        match item.commit_match {
            CommitMatch::Exact => counts.exact += 1,
            CommitMatch::Partial => counts.partial += 1,
            CommitMatch::Stale => counts.stale += 1,
            CommitMatch::Unknown => counts.unknown += 1,
        }

        match item.availability {
            ProofEvidenceAvailability::Degraded => counts.degraded += 1,
            ProofEvidenceAvailability::Skipped => counts.skipped += 1,
            ProofEvidenceAvailability::Unavailable => counts.unavailable += 1,
            _ => {}
        }

        if matches!(
            item.execution_status,
            ProofExecutionStatus::Planned | ProofExecutionStatus::NotExecuted
        ) {
            counts.not_run += 1;
        }

        if item.required {
            if item.execution_status == ProofExecutionStatus::ExecutedPassed
                && item.availability == ProofEvidenceAvailability::Available
            {
                counts.required_passed += 1;
            } else if item.execution_status == ProofExecutionStatus::ExecutedFailed {
                counts.required_failed += 1;
            } else if matches!(
                item.availability,
                ProofEvidenceAvailability::Missing | ProofEvidenceAvailability::Unavailable
            ) {
                counts.required_missing += 1;
            }
        }

        if item.advisory {
            if item.availability == ProofEvidenceAvailability::Available {
                counts.advisory_available += 1;
            } else if matches!(
                item.availability,
                ProofEvidenceAvailability::Missing | ProofEvidenceAvailability::Unavailable
            ) {
                counts.advisory_missing += 1;
            }
        }
    }

    counts
}
