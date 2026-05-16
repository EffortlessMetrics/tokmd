//! Markdown work-order rendering for handoff bundles.

use std::path::Path;

use anyhow::Result;
use tokmd_types::{ArtifactEntry, ContextFileRow, InclusionPolicy};

use super::artifact::write_text_artifact;
use super::linked_evidence::{
    LinkedEvidenceSummary, render_linked_evidence_summary, summarize_linked_evidence,
};
use super::links::path_string;
use super::{HandoffLinkInputs, HandoffWorkOrderInputs};

pub(crate) fn write_work_order(
    out_dir: &Path,
    order: &HandoffWorkOrderInputs<'_>,
) -> Result<ArtifactEntry> {
    let linked_evidence = summarize_linked_evidence(order.links);
    write_text_artifact(
        out_dir,
        "work-order",
        "work-order.md",
        "Agent work order and consumption guide",
        &render_work_order(order, &linked_evidence),
    )
}

fn render_work_order(
    order: &HandoffWorkOrderInputs<'_>,
    linked_evidence: &LinkedEvidenceSummary,
) -> String {
    let mut out = String::new();
    push_work_order_header(&mut out);
    push_start_here_section(&mut out, order.links);
    push_bundle_summary_section(&mut out, order);
    push_linked_evidence_section(&mut out, order.links);
    render_linked_evidence_summary(&mut out, order.links, linked_evidence);
    push_included_files_section(&mut out, order.selected);
    push_agent_guardrails_section(&mut out);
    out
}

fn push_work_order_header(out: &mut String) {
    out.push_str("# Agent Work Order\n\n");
    out.push_str("This handoff is a deterministic source/context bundle for coding-agent work.\n");
    out.push_str("Treat linked review and proof receipts as external evidence handles; this file does not verify them.\n\n");
}

fn push_start_here_section(out: &mut String, links: &HandoffLinkInputs<'_>) {
    out.push_str("## Start Here\n\n");
    let mut steps = vec![
        "Read `manifest.json` for the authoritative artifact index, token budget, included files, and exclusions.",
        "Read `work-order.md` for the agent task map and guardrails.",
        "Read `code.txt` for the bounded source bundle.",
        "Use `map.jsonl` for full file inventory and path lookup.",
        "Use `intelligence.json` for repository shape, hotspots, complexity, and derived signals.",
    ];
    if links.review_packet_dir.is_some() || links.review_packet_check.is_some() {
        steps.push(
            "Use `review-links.json` for cockpit review packet and verifier receipt pointers.",
        );
    }
    if links.affected.is_some() || links.proof_plan.is_some() {
        steps.push("Use `proof-links.json` for affected-proof and proof-plan pointers.");
    }
    for (index, step) in steps.iter().enumerate() {
        out.push_str(&format!("{}. {}\n", index + 1, step));
    }
}

fn push_bundle_summary_section(out: &mut String, order: &HandoffWorkOrderInputs<'_>) {
    out.push_str("\n## Bundle Summary\n\n");
    out.push_str(&format!("- Inputs: {}\n", order.inputs.join(", ")));
    out.push_str(&format!("- Budget tokens: {}\n", order.budget_tokens));
    out.push_str(&format!("- Used tokens: {}\n", order.used_tokens));
    out.push_str(&format!("- Utilization: {:.2}%\n", order.utilization_pct));
    out.push_str(&format!("- Strategy: `{}`\n", order.strategy));
    out.push_str(&format!("- Rank metric: `{}`\n", order.rank_by));
    out.push_str(&format!(
        "- Intelligence preset: `{}`\n",
        order.intelligence_preset
    ));
    out.push_str(&format!("- Bundled files: {}\n", order.selected.len()));
    out.push_str(&format!("- Total scanned files: {}\n", order.total_files));
}

fn push_linked_evidence_section(out: &mut String, links: &HandoffLinkInputs<'_>) {
    out.push_str("\n## Linked Evidence\n\n");
    push_linked_path_line(out, "Review packet directory", links.review_packet_dir);
    push_linked_path_line(
        out,
        "Review packet verifier receipt",
        links.review_packet_check,
    );
    push_linked_path_line(out, "Affected proof report", links.affected);
    push_linked_path_line(out, "Proof plan report", links.proof_plan);
}

fn push_linked_path_line(out: &mut String, label: &str, path: Option<&Path>) {
    match path {
        Some(path) => out.push_str(&format!("- {}: `{}`\n", label, path_string(path))),
        None => out.push_str(&format!("- {}: not linked\n", label)),
    }
}

fn push_included_files_section(out: &mut String, selected: &[ContextFileRow]) {
    out.push_str("\n## Included Files\n\n");
    if selected.is_empty() {
        out.push_str("- No files were bundled.\n");
        return;
    }
    for file in selected.iter().take(20) {
        let effective_tokens = file.effective_tokens.unwrap_or(file.tokens);
        out.push_str(&format!(
            "- `{}`: {}, policy `{}`, {} effective tokens",
            file.path,
            file.lang,
            policy_label(file.policy),
            effective_tokens
        ));
        if !file.rank_reason.is_empty() {
            out.push_str(&format!(", reason: {}", file.rank_reason));
        }
        out.push('\n');
    }
    if selected.len() > 20 {
        out.push_str(&format!(
            "- ... {} more bundled file(s); see `manifest.json` for the full list.\n",
            selected.len() - 20
        ));
    }
}

fn push_agent_guardrails_section(out: &mut String) {
    out.push_str("\n## Agent Guardrails\n\n");
    out.push_str("- Treat missing, stale, degraded, skipped, or unavailable evidence as work to resolve, not as passing proof.\n");
    out.push_str("- Run reproduction commands from the linked review map before claiming a repair is proven.\n");
    out.push_str(
        "- Keep generated receipts with the work when they explain review or proof state.\n",
    );
    out.push_str("- Do not promote advisory proof, enable default Codecov upload, or turn this handoff into a merge verdict.\n");
}

fn policy_label(policy: InclusionPolicy) -> &'static str {
    match policy {
        InclusionPolicy::Full => "full",
        InclusionPolicy::HeadTail => "head_tail",
        InclusionPolicy::Summary => "summary",
        InclusionPolicy::Skip => "skip",
    }
}
