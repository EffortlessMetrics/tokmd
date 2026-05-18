//! Work-order rendering and linked-evidence summaries for handoff output.

use std::fs;
use std::path::Path;

use anyhow::Result;
use serde_json::Value;
use tokmd_types::{ArtifactEntry, ContextFileRow, InclusionPolicy};

use super::artifacts::write_text_artifact;
use super::links::{HandoffLinkInputs, path_string};

pub(in crate::commands::handoff) struct HandoffWorkOrderInputs<'a> {
    pub(in crate::commands::handoff) inputs: &'a [String],
    pub(in crate::commands::handoff) budget_tokens: usize,
    pub(in crate::commands::handoff) used_tokens: usize,
    pub(in crate::commands::handoff) utilization_pct: f64,
    pub(in crate::commands::handoff) strategy: &'a str,
    pub(in crate::commands::handoff) rank_by: &'a str,
    pub(in crate::commands::handoff) intelligence_preset: &'a str,
    pub(in crate::commands::handoff) total_files: usize,
    pub(in crate::commands::handoff) selected: &'a [ContextFileRow],
    pub(in crate::commands::handoff) links: &'a HandoffLinkInputs<'a>,
}

#[derive(Default)]
struct LinkedEvidenceSummary {
    review_map: Option<ReviewMapSummary>,
    review_packet_check: Option<ReviewPacketCheckSummary>,
    affected: Option<AffectedSummary>,
    proof_plan: Option<ProofPlanSummary>,
}

struct ReviewMapSummary {
    item_count: usize,
    first_items: Vec<ReviewMapItemSummary>,
    available: Option<u64>,
    missing: Option<u64>,
    degraded: Option<u64>,
    stale: Option<u64>,
    skipped: Option<u64>,
    unavailable: Option<u64>,
}

struct ReviewMapItemSummary {
    path: String,
    reason: Option<String>,
}

struct ReviewPacketCheckSummary {
    ok: Option<bool>,
    artifact_count: Option<u64>,
    hashes_verified: Option<u64>,
}

struct AffectedSummary {
    changed_files: usize,
    scopes: usize,
    unknown_files: usize,
    scope_names: Vec<String>,
}

struct ProofPlanSummary {
    commands: usize,
    required: usize,
    advisory: usize,
    first_commands: Vec<String>,
}

pub(super) fn write_work_order(
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

fn render_linked_evidence_summary(
    out: &mut String,
    links: &HandoffLinkInputs<'_>,
    summary: &LinkedEvidenceSummary,
) {
    if !has_any_link(links) {
        return;
    }

    out.push_str("\n## Linked Evidence Summary\n\n");
    out.push_str("These summaries are best-effort hints from linked receipts. They do not replace the linked verifier or proof artifacts.\n\n");

    render_review_packet_check_summary(out, links, summary);
    render_review_map_summary(out, links, summary);
    render_affected_summary(out, links, summary);
    render_proof_plan_summary(out, links, summary);
}

fn render_review_packet_check_summary(
    out: &mut String,
    links: &HandoffLinkInputs<'_>,
    summary: &LinkedEvidenceSummary,
) {
    if let Some(check) = &summary.review_packet_check {
        out.push_str("- Review packet verifier:");
        if let Some(ok) = check.ok {
            out.push_str(&format!(" ok={ok}"));
        } else {
            out.push_str(" ok=unknown");
        }
        if let Some(artifact_count) = check.artifact_count {
            out.push_str(&format!(", artifacts={artifact_count}"));
        }
        if let Some(hashes_verified) = check.hashes_verified {
            out.push_str(&format!(", hashes_verified={hashes_verified}"));
        }
        out.push('\n');
    } else if links.review_packet_check.is_some() {
        out.push_str("- Review packet verifier: linked but not readable\n");
    }
}

fn render_review_map_summary(
    out: &mut String,
    links: &HandoffLinkInputs<'_>,
    summary: &LinkedEvidenceSummary,
) {
    if let Some(review_map) = &summary.review_map {
        out.push_str(&format!("- Review map: {} item(s)", review_map.item_count));
        if review_map.available.is_some()
            || review_map.missing.is_some()
            || review_map.degraded.is_some()
            || review_map.stale.is_some()
            || review_map.skipped.is_some()
            || review_map.unavailable.is_some()
        {
            out.push_str(" (");
            push_count(out, "available", review_map.available);
            push_count(out, "missing", review_map.missing);
            push_count(out, "degraded", review_map.degraded);
            push_count(out, "stale", review_map.stale);
            push_count(out, "skipped", review_map.skipped);
            push_count(out, "unavailable", review_map.unavailable);
            trim_trailing_separator(out);
            out.push(')');
        }
        out.push('\n');
        if !review_map.first_items.is_empty() {
            out.push_str("  - Review first:\n");
            for item in &review_map.first_items {
                out.push_str(&format!("    - `{}`", item.path));
                if let Some(reason) = &item.reason {
                    out.push_str(&format!(": {reason}"));
                }
                out.push('\n');
            }
        }
    } else if links.review_packet_dir.is_some() {
        out.push_str("- Review map: linked but not readable\n");
    }
}

fn render_affected_summary(
    out: &mut String,
    links: &HandoffLinkInputs<'_>,
    summary: &LinkedEvidenceSummary,
) {
    if let Some(affected) = &summary.affected {
        out.push_str(&format!(
            "- Affected proof: {} changed file(s), {} scope(s), {} unknown file(s)\n",
            affected.changed_files, affected.scopes, affected.unknown_files
        ));
        if !affected.scope_names.is_empty() {
            out.push_str("  - Scopes: ");
            out.push_str(&affected.scope_names.join(", "));
            out.push('\n');
        }
    } else if links.affected.is_some() {
        out.push_str("- Affected proof: linked but not readable\n");
    }
}

fn render_proof_plan_summary(
    out: &mut String,
    links: &HandoffLinkInputs<'_>,
    summary: &LinkedEvidenceSummary,
) {
    if let Some(proof_plan) = &summary.proof_plan {
        out.push_str(&format!(
            "- Proof plan: {} command(s), {} required, {} advisory\n",
            proof_plan.commands, proof_plan.required, proof_plan.advisory
        ));
        if !proof_plan.first_commands.is_empty() {
            out.push_str("  - First commands:\n");
            for command in &proof_plan.first_commands {
                out.push_str(&format!("    - `{command}`\n"));
            }
            if proof_plan.commands > proof_plan.first_commands.len() {
                out.push_str(&format!(
                    "    - ... {} more command(s); open the proof plan for the full list.\n",
                    proof_plan.commands - proof_plan.first_commands.len()
                ));
            }
        }
        out.push_str("  - A proof plan is planned evidence, not execution proof.\n");
    } else if links.proof_plan.is_some() {
        out.push_str("- Proof plan: linked but not readable\n");
    }
}

fn summarize_linked_evidence(links: &HandoffLinkInputs<'_>) -> LinkedEvidenceSummary {
    LinkedEvidenceSummary {
        review_map: links
            .review_packet_dir
            .and_then(|dir| read_json_value(&dir.join("review-map.json")))
            .and_then(|value| summarize_review_map(&value)),
        review_packet_check: links
            .review_packet_check
            .and_then(read_json_value)
            .map(|value| summarize_review_packet_check(&value)),
        affected: links
            .affected
            .and_then(read_json_value)
            .map(|value| summarize_affected(&value)),
        proof_plan: links
            .proof_plan
            .and_then(read_json_value)
            .map(|value| summarize_proof_plan(&value)),
    }
}

fn read_json_value(path: &Path) -> Option<Value> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn summarize_review_map(value: &Value) -> Option<ReviewMapSummary> {
    let items = value.get("items")?.as_array()?;
    let item_count = value
        .get("item_count")
        .and_then(Value::as_u64)
        .map(|count| count as usize)
        .unwrap_or(items.len());
    let first_items = items
        .iter()
        .take(5)
        .filter_map(|item| {
            let path = item.get("path")?.as_str()?.to_string();
            let reason = item
                .get("reason")
                .and_then(Value::as_str)
                .map(str::to_string);
            Some(ReviewMapItemSummary { path, reason })
        })
        .collect();
    let evidence_summary = value.get("evidence").and_then(|e| e.get("summary"));

    Some(ReviewMapSummary {
        item_count,
        first_items,
        available: count_field(evidence_summary, "available"),
        missing: count_field(evidence_summary, "missing"),
        degraded: count_field(evidence_summary, "degraded"),
        stale: count_field(evidence_summary, "stale"),
        skipped: count_field(evidence_summary, "skipped"),
        unavailable: count_field(evidence_summary, "unavailable"),
    })
}

fn summarize_review_packet_check(value: &Value) -> ReviewPacketCheckSummary {
    ReviewPacketCheckSummary {
        ok: value.get("ok").and_then(Value::as_bool),
        artifact_count: value.get("artifact_count").and_then(Value::as_u64),
        hashes_verified: value.get("hashes_verified").and_then(Value::as_u64),
    }
}

fn summarize_affected(value: &Value) -> AffectedSummary {
    let changed_files = array_len(value.get("changed_files"));
    let scopes_array = value.get("scopes").and_then(Value::as_array);
    let scope_names = scopes_array
        .into_iter()
        .flat_map(|scopes| scopes.iter())
        .filter_map(|scope| scope.get("name").and_then(Value::as_str))
        .take(8)
        .map(str::to_string)
        .collect::<Vec<_>>();

    AffectedSummary {
        changed_files,
        scopes: array_len(value.get("scopes")),
        unknown_files: array_len(value.get("unknown_files")),
        scope_names,
    }
}

fn summarize_proof_plan(value: &Value) -> ProofPlanSummary {
    let Some(commands) = value.get("commands").and_then(Value::as_array) else {
        return ProofPlanSummary {
            commands: 0,
            required: 0,
            advisory: 0,
            first_commands: Vec::new(),
        };
    };
    let required = commands
        .iter()
        .filter(|command| command.get("required").and_then(Value::as_bool) == Some(true))
        .count();
    let advisory = commands.len().saturating_sub(required);
    let first_commands = commands
        .iter()
        .filter_map(|command| command.get("command").and_then(Value::as_str))
        .take(5)
        .map(str::to_string)
        .collect();

    ProofPlanSummary {
        commands: commands.len(),
        required,
        advisory,
        first_commands,
    }
}

fn has_any_link(links: &HandoffLinkInputs<'_>) -> bool {
    links.review_packet_dir.is_some()
        || links.review_packet_check.is_some()
        || links.affected.is_some()
        || links.proof_plan.is_some()
}

fn count_field(value: Option<&Value>, field: &str) -> Option<u64> {
    value
        .and_then(|value| value.get(field))
        .and_then(Value::as_u64)
}

fn array_len(value: Option<&Value>) -> usize {
    value.and_then(Value::as_array).map_or(0, Vec::len)
}

fn push_count(out: &mut String, label: &str, count: Option<u64>) {
    if let Some(count) = count {
        out.push_str(&format!("{label}={count}, "));
    }
}

fn trim_trailing_separator(out: &mut String) {
    if out.ends_with(", ") {
        out.truncate(out.len() - 2);
    }
}

fn policy_label(policy: InclusionPolicy) -> &'static str {
    match policy {
        InclusionPolicy::Full => "full",
        InclusionPolicy::HeadTail => "head_tail",
        InclusionPolicy::Summary => "summary",
        InclusionPolicy::Skip => "skip",
    }
}
