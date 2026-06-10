//! Evidence packet manifest summary.

use serde_json::Value;

pub(in crate::commands::handoff) struct EvidencePacketSummary {
    pub(in crate::commands::handoff) status: Option<String>,
    pub(in crate::commands::handoff) preset: Option<String>,
    pub(in crate::commands::handoff) paths: Vec<String>,
    pub(in crate::commands::handoff) review_priority: Vec<EvidencePacketPriorityItemSummary>,
    pub(in crate::commands::handoff) warnings: Vec<String>,
    pub(in crate::commands::handoff) errors: Vec<String>,
    pub(in crate::commands::handoff) reproduce: Vec<String>,
}

pub(in crate::commands::handoff) struct EvidencePacketPriorityItemSummary {
    pub(in crate::commands::handoff) path: String,
    pub(in crate::commands::handoff) category: String,
    pub(in crate::commands::handoff) severity: String,
    pub(in crate::commands::handoff) score: u64,
    pub(in crate::commands::handoff) reason: String,
}

impl EvidencePacketSummary {
    pub(in crate::commands::handoff) fn status_label(&self) -> &str {
        self.status.as_deref().unwrap_or("unknown")
    }

    pub(in crate::commands::handoff) fn is_failed(&self) -> bool {
        self.status.as_deref() == Some("failed")
    }

    pub(in crate::commands::handoff) fn is_partial(&self) -> bool {
        self.status.as_deref() == Some("partial")
    }
}

pub(super) fn summarize(value: &Value) -> EvidencePacketSummary {
    EvidencePacketSummary {
        status: string_field(value, "status"),
        preset: string_field(value, "preset"),
        paths: string_array(value.get("paths")),
        review_priority: review_priority(value.get("review_priority")),
        warnings: string_array(value.get("warnings")),
        errors: string_array(value.get("errors")),
        reproduce: string_array(value.get("reproduce")),
    }
}

pub(super) fn render(out: &mut String, packet: &EvidencePacketSummary) {
    out.push_str(&format!(
        "- Evidence packet: status={} preset={} paths={} warning(s)={} error(s)={}\n",
        packet.status_label(),
        packet.preset.as_deref().unwrap_or("unknown"),
        packet.paths.len(),
        packet.warnings.len(),
        packet.errors.len()
    ));
    if !packet.review_priority.is_empty() {
        out.push_str("  - Review priority:\n");
        for item in &packet.review_priority {
            out.push_str(&format!(
                "    - `{}`: {}/{} score {} - {}\n",
                item.path, item.category, item.severity, item.score, item.reason
            ));
        }
    }
}

fn review_priority(value: Option<&Value>) -> Vec<EvidencePacketPriorityItemSummary> {
    let Some(items) = value.and_then(Value::as_array) else {
        return Vec::new();
    };
    items
        .iter()
        .take(5)
        .filter_map(|item| {
            Some(EvidencePacketPriorityItemSummary {
                path: string_field(item, "path")?,
                category: string_field(item, "category")?,
                severity: string_field(item, "severity")?,
                score: item.get("score").and_then(Value::as_u64).unwrap_or(0),
                reason: string_field(item, "reason")?,
            })
        })
        .collect()
}

fn string_field(value: &Value, field: &str) -> Option<String> {
    value.get(field).and_then(Value::as_str).map(str::to_string)
}

fn string_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}
