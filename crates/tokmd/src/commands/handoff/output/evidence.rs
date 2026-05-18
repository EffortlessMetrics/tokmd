//! Best-effort summaries parsed from linked review and proof receipts.

use std::fs;
use std::path::Path;

use serde_json::Value;

use super::links::HandoffLinkInputs;

#[derive(Default)]
pub(super) struct LinkedEvidenceSummary {
    pub(super) review_map: Option<ReviewMapSummary>,
    pub(super) review_packet_check: Option<ReviewPacketCheckSummary>,
    pub(super) affected: Option<AffectedSummary>,
    pub(super) proof_plan: Option<ProofPlanSummary>,
}

pub(super) struct ReviewMapSummary {
    pub(super) item_count: usize,
    pub(super) first_items: Vec<ReviewMapItemSummary>,
    pub(super) available: Option<u64>,
    pub(super) missing: Option<u64>,
    pub(super) degraded: Option<u64>,
    pub(super) stale: Option<u64>,
    pub(super) skipped: Option<u64>,
    pub(super) unavailable: Option<u64>,
}

pub(super) struct ReviewMapItemSummary {
    pub(super) path: String,
    pub(super) reason: Option<String>,
}

pub(super) struct ReviewPacketCheckSummary {
    pub(super) ok: Option<bool>,
    pub(super) artifact_count: Option<u64>,
    pub(super) hashes_verified: Option<u64>,
}

pub(super) struct AffectedSummary {
    pub(super) changed_files: usize,
    pub(super) scopes: usize,
    pub(super) unknown_files: usize,
    pub(super) scope_names: Vec<String>,
}

pub(super) struct ProofPlanSummary {
    pub(super) commands: usize,
    pub(super) required: usize,
    pub(super) advisory: usize,
    pub(super) first_commands: Vec<String>,
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

fn count_field(value: Option<&Value>, field: &str) -> Option<u64> {
    value
        .and_then(|value| value.get(field))
        .and_then(Value::as_u64)
}

fn array_len(value: Option<&Value>) -> usize {
    value.and_then(Value::as_array).map_or(0, Vec::len)
}
