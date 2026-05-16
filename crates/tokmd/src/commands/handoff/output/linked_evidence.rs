//! Best-effort summaries of linked review and proof receipts.

use std::fs;

use serde_json::Value;

use super::HandoffLinkInputs;

#[derive(Default)]
pub(super) struct LinkedEvidenceSummary {
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

pub(super) fn summarize_linked_evidence(links: &HandoffLinkInputs<'_>) -> LinkedEvidenceSummary {
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

pub(super) fn render_linked_evidence_summary(
    out: &mut String,
    links: &HandoffLinkInputs<'_>,
    summary: &LinkedEvidenceSummary,
) {
    if !has_any_link(links) {
        return;
    }

    out.push_str("\n## Linked Evidence Summary\n\n");
    out.push_str("These summaries are best-effort hints from linked receipts. They do not replace the linked verifier or proof artifacts.\n\n");

    render_review_packet_check(out, links, summary);
    render_review_map(out, links, summary);
    render_affected(out, links, summary);
    render_proof_plan(out, links, summary);
}

fn render_review_packet_check(
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

fn render_review_map(
    out: &mut String,
    links: &HandoffLinkInputs<'_>,
    summary: &LinkedEvidenceSummary,
) {
    if let Some(review_map) = &summary.review_map {
        out.push_str(&format!("- Review map: {} item(s)", review_map.item_count));
        render_review_map_counts(out, review_map);
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

fn render_review_map_counts(out: &mut String, review_map: &ReviewMapSummary) {
    if review_map.available.is_none()
        && review_map.missing.is_none()
        && review_map.degraded.is_none()
        && review_map.stale.is_none()
        && review_map.skipped.is_none()
        && review_map.unavailable.is_none()
    {
        return;
    }

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

fn render_affected(
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

fn render_proof_plan(
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

fn read_json_value(path: &std::path::Path) -> Option<Value> {
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
    let scopes_array = value.get("scopes").and_then(Value::as_array);
    let scope_names = scopes_array
        .into_iter()
        .flat_map(|scopes| scopes.iter())
        .filter_map(|scope| scope.get("name").and_then(Value::as_str))
        .take(8)
        .map(str::to_string)
        .collect::<Vec<_>>();

    AffectedSummary {
        changed_files: array_len(value.get("changed_files")),
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
