//! Review map (cockpit review packet) summary.

use serde_json::Value;

pub(in crate::commands::handoff) struct ReviewMapSummary {
    pub(in crate::commands::handoff) item_count: usize,
    pub(in crate::commands::handoff) first_items: Vec<ReviewMapItemSummary>,
    pub(in crate::commands::handoff) available: Option<u64>,
    pub(in crate::commands::handoff) missing: Option<u64>,
    pub(in crate::commands::handoff) degraded: Option<u64>,
    pub(in crate::commands::handoff) stale: Option<u64>,
    pub(in crate::commands::handoff) skipped: Option<u64>,
    pub(in crate::commands::handoff) unavailable: Option<u64>,
}

pub(in crate::commands::handoff) struct ReviewMapItemSummary {
    pub(in crate::commands::handoff) path: String,
    pub(in crate::commands::handoff) reason: Option<String>,
}

pub(super) fn summarize(value: &Value) -> Option<ReviewMapSummary> {
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

pub(super) fn render(out: &mut String, review_map: &ReviewMapSummary) {
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
}

fn count_field(value: Option<&Value>, field: &str) -> Option<u64> {
    value
        .and_then(|value| value.get(field))
        .and_then(Value::as_u64)
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
