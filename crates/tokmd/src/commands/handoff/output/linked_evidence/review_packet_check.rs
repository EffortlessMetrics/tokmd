//! Review packet verifier receipt summary.

use serde_json::Value;

pub(in crate::commands::handoff) struct ReviewPacketCheckSummary {
    pub(in crate::commands::handoff) ok: Option<bool>,
    pub(in crate::commands::handoff) artifact_count: Option<u64>,
    pub(in crate::commands::handoff) hashes_verified: Option<u64>,
}

pub(super) fn summarize(value: &Value) -> ReviewPacketCheckSummary {
    ReviewPacketCheckSummary {
        ok: value.get("ok").and_then(Value::as_bool),
        artifact_count: value.get("artifact_count").and_then(Value::as_u64),
        hashes_verified: value.get("hashes_verified").and_then(Value::as_u64),
    }
}

pub(super) fn render(out: &mut String, check: &ReviewPacketCheckSummary) {
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
}
