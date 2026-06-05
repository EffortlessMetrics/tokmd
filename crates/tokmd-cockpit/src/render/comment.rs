//! PR comment rendering for cockpit receipts and review packets.

use crate::doc_artifacts_evidence::DocArtifactsEvidenceInput;
use crate::proof_evidence::ProofEvidenceInput;
use crate::{CockpitReceipt, GateStatus, RiskLevel};

use super::bun_ub_sensor::{BunUbSensorEvidence, bun_ub_sensor_refs, receipt_has_bun_ub_scope};
use super::evidence::{doc_artifacts_expected, evidence_counts};
use super::proof_summary::proof_evidence_summary;

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
    doc_artifacts: Option<&DocArtifactsEvidenceInput>,
    bun_ub_sensor: BunUbSensorEvidence,
) -> String {
    use std::fmt::Write;

    let mut s = render_comment_md(receipt);
    write_proof_evidence_summary(&mut s, receipt, proof_inputs);
    write_doc_artifacts_summary(&mut s, receipt, doc_artifacts);
    write_bun_ub_sensor_summary(&mut s, receipt, bun_ub_sensor);
    let _ = writeln!(s, "**Review packet artifacts**:");
    let _ = writeln!(s, "- [Evidence gates](evidence.json)");
    let _ = writeln!(s, "- [Review map](review-map.md)");
    let _ = writeln!(s, "- [Full cockpit receipt](cockpit.json)");
    let _ = writeln!(s);
    s
}

fn write_bun_ub_sensor_summary(
    s: &mut String,
    receipt: &CockpitReceipt,
    sensor: BunUbSensorEvidence,
) {
    use std::fmt::Write;

    if !receipt_has_bun_ub_scope(receipt) {
        return;
    }

    let _ = writeln!(s, "**Bun UB sensor artifacts**: {}.", sensor.status());
    for path in bun_ub_sensor_refs() {
        let _ = writeln!(s, "- {path}");
    }
    let missing = sensor.missing_paths();
    if !missing.is_empty() {
        let _ = writeln!(s, "- Missing: {}", missing.join(", "));
    }
    let _ = writeln!(s, "- Regeneration commands are listed in review-map.md.");
    let _ = writeln!(s);
}

fn write_doc_artifacts_summary(
    s: &mut String,
    receipt: &CockpitReceipt,
    doc_artifacts: Option<&DocArtifactsEvidenceInput>,
) {
    use std::fmt::Write;

    match doc_artifacts {
        Some(input) if input.receipt.ok => {
            let _ = writeln!(
                s,
                "**Doc artifacts**: verified ({} required docs, {} family files, {} active goals, {} spec-index artifacts, {} spec-index lanes).",
                input.receipt.checked.required_docs,
                input.receipt.checked.family_files,
                input.receipt.checked.active_goals,
                input.receipt.checked.spec_index_artifacts,
                input.receipt.checked.spec_index_lanes,
            );
            let _ = writeln!(s);
        }
        Some(input) => {
            let _ = writeln!(
                s,
                "**Doc artifacts**: degraded ({} error(s)).",
                input.receipt.errors.len()
            );
            let _ = writeln!(s);
        }
        None if doc_artifacts_expected(receipt) => {
            let _ = writeln!(s, "**Doc artifacts**: missing for source-of-truth changes.");
            let _ = writeln!(s);
        }
        None => {}
    }
}

fn write_proof_evidence_summary(
    s: &mut String,
    receipt: &CockpitReceipt,
    proof_inputs: &[ProofEvidenceInput],
) {
    use std::fmt::Write;

    let counts = proof_evidence_summary(receipt, proof_inputs);
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
    if counts.routing_available > 0
        || counts.routing_missing > 0
        || counts.routing_degraded > 0
        || counts.routing_stale > 0
        || counts.routing_skipped > 0
        || counts.routing_unavailable > 0
    {
        let _ = writeln!(
            s,
            "- Proof routing: {} available, {} missing, {} degraded, {} stale, {} skipped, {} unavailable",
            counts.routing_available,
            counts.routing_missing,
            counts.routing_degraded,
            counts.routing_stale,
            counts.routing_skipped,
            counts.routing_unavailable,
        );
    }
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
