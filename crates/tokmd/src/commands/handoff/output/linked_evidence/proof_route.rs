//! Proof-pack route receipt summary.

use std::collections::BTreeSet;

use serde_json::Value;

pub(in crate::commands::handoff) struct ProofRouteSummary {
    pub(in crate::commands::handoff) schema_version: Option<u64>,
    pub(in crate::commands::handoff) changed_file_count: usize,
    pub(in crate::commands::handoff) routed_file_count: usize,
    pub(in crate::commands::handoff) unmatched_file_count: usize,
    pub(in crate::commands::handoff) skipped_lane_count: usize,
    pub(in crate::commands::handoff) skipped_blocking_lanes: usize,
    pub(in crate::commands::handoff) surfaces: Vec<String>,
    pub(in crate::commands::handoff) skipped_reason_counts: Vec<(String, u64)>,
    pub(in crate::commands::handoff) first_changed_files: Vec<RouteFileSummary>,
    pub(in crate::commands::handoff) first_unmatched_files: Vec<String>,
    pub(in crate::commands::handoff) first_skipped_lanes: Vec<SkippedLaneSummary>,
}

pub(in crate::commands::handoff) struct RouteFileSummary {
    pub(in crate::commands::handoff) path: String,
    pub(in crate::commands::handoff) surface: String,
    pub(in crate::commands::handoff) proof_packs: Vec<String>,
}

pub(in crate::commands::handoff) struct SkippedLaneSummary {
    pub(in crate::commands::handoff) lane: String,
    pub(in crate::commands::handoff) reason: String,
    pub(in crate::commands::handoff) matched_files: usize,
    pub(in crate::commands::handoff) blocking: bool,
    pub(in crate::commands::handoff) estimated_lem: Option<u64>,
    pub(in crate::commands::handoff) estimate_source: Option<String>,
}

pub(super) fn summarize(value: &Value) -> ProofRouteSummary {
    let changed_files = value
        .get("changed_files")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let skipped = value
        .get("skipped_by_policy")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let unmatched_files = value
        .get("unmatched_files")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let summary = value.get("summary");

    let first_changed_files = changed_files
        .iter()
        .filter_map(route_file_summary)
        .take(8)
        .collect::<Vec<_>>();
    let first_unmatched_files = unmatched_files
        .iter()
        .filter_map(Value::as_str)
        .take(8)
        .map(str::to_string)
        .collect::<Vec<_>>();
    let first_skipped_lanes = skipped
        .iter()
        .filter_map(skipped_lane_summary)
        .take(8)
        .collect::<Vec<_>>();
    let surfaces = changed_files
        .iter()
        .filter_map(|file| file.get("surface").and_then(Value::as_str))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .take(10)
        .map(str::to_string)
        .collect::<Vec<_>>();

    ProofRouteSummary {
        schema_version: value.get("schema_version").and_then(Value::as_u64),
        changed_file_count: summary_count(summary, "changed_file_count", changed_files.len()),
        routed_file_count: summary_count(summary, "routed_file_count", changed_files.len()),
        unmatched_file_count: summary_count(summary, "unmatched_file_count", unmatched_files.len()),
        skipped_lane_count: summary_count(summary, "skipped_lane_count", skipped.len()),
        skipped_blocking_lanes: skipped
            .iter()
            .filter(|row| row.get("blocking").and_then(Value::as_bool) == Some(true))
            .count(),
        surfaces,
        skipped_reason_counts: skipped_reason_counts(summary),
        first_changed_files,
        first_unmatched_files,
        first_skipped_lanes,
    }
}

pub(super) fn render(out: &mut String, proof_route: &ProofRouteSummary) {
    let schema = proof_route
        .schema_version
        .map(|version| format!("schema v{version}, "))
        .unwrap_or_default();
    out.push_str(&format!(
        "- Proof route: {schema}{} changed file(s), {} routed, {} unmatched, {} skipped lane(s)\n",
        proof_route.changed_file_count,
        proof_route.routed_file_count,
        proof_route.unmatched_file_count,
        proof_route.skipped_lane_count
    ));
    if !proof_route.surfaces.is_empty() {
        out.push_str("  - Surfaces: ");
        out.push_str(&proof_route.surfaces.join(", "));
        out.push('\n');
    }
    if !proof_route.skipped_reason_counts.is_empty() {
        let counts = proof_route
            .skipped_reason_counts
            .iter()
            .map(|(reason, count)| format!("{reason}={count}"))
            .collect::<Vec<_>>();
        out.push_str("  - Skipped reasons: ");
        out.push_str(&counts.join(", "));
        out.push('\n');
    }
    if !proof_route.first_skipped_lanes.is_empty() {
        out.push_str("  - First skipped lanes:\n");
        for lane in &proof_route.first_skipped_lanes {
            let estimate = match (lane.estimated_lem, lane.estimate_source.as_deref()) {
                (Some(lem), Some(source)) => format!(", {lem} LEM, {source}"),
                (Some(lem), None) => format!(", {lem} LEM"),
                (None, _) => String::new(),
            };
            let blocking = if lane.blocking { ", blocking" } else { "" };
            out.push_str(&format!(
                "    - `{}`: {}{}{}, {} matched file(s)\n",
                lane.lane, lane.reason, blocking, estimate, lane.matched_files
            ));
        }
        if proof_route.skipped_lane_count > proof_route.first_skipped_lanes.len() {
            out.push_str(&format!(
                "    - ... {} more skipped lane(s); open the proof route for the full list.\n",
                proof_route.skipped_lane_count - proof_route.first_skipped_lanes.len()
            ));
        }
    }
    out.push_str("  - A proof route is routing evidence, not execution proof.\n");
}

fn route_file_summary(value: &Value) -> Option<RouteFileSummary> {
    let path = value
        .get("changed_file")
        .or_else(|| value.get("path"))
        .and_then(Value::as_str)?
        .to_string();
    let surface = value
        .get("surface")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let proof_packs = string_array_field(value, "required_packs")
        .or_else(|| string_array_field(value, "proof_packs"))
        .unwrap_or_default()
        .into_iter()
        .take(8)
        .collect();

    Some(RouteFileSummary {
        path,
        surface,
        proof_packs,
    })
}

fn string_array_field(value: &Value, field: &str) -> Option<Vec<String>> {
    Some(
        value
            .get(field)?
            .as_array()?
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
    )
}

fn skipped_lane_summary(value: &Value) -> Option<SkippedLaneSummary> {
    Some(SkippedLaneSummary {
        lane: value.get("lane").and_then(Value::as_str)?.to_string(),
        reason: value
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        matched_files: value
            .get("matched_files")
            .and_then(Value::as_array)
            .map_or(0, Vec::len),
        blocking: value.get("blocking").and_then(Value::as_bool) == Some(true),
        estimated_lem: value.get("estimated_lem").and_then(Value::as_u64),
        estimate_source: value
            .get("estimate_source")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

fn summary_count(summary: Option<&Value>, field: &str, fallback: usize) -> usize {
    summary
        .and_then(|summary| summary.get(field))
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(fallback)
}

fn skipped_reason_counts(summary: Option<&Value>) -> Vec<(String, u64)> {
    summary
        .and_then(|summary| summary.get("skipped_reason_counts"))
        .and_then(Value::as_object)
        .map(|counts| {
            counts
                .iter()
                .filter_map(|(reason, count)| count.as_u64().map(|count| (reason.clone(), count)))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proof_route_summary_reads_v5_counts_routes_and_skipped_reasons() {
        let value = serde_json::json!({
            "schema": "tokmd.proof_pack_route.v1",
            "schema_version": 5,
            "changed_files": [
                {
                    "changed_file": "docs/handoff.md",
                    "path": "docs/handoff.md",
                    "surface": "handoff_review_packet",
                    "required_packs": ["handoff_review_packet"],
                    "proof_packs": ["handoff_review_packet"]
                }
            ],
            "unmatched_files": ["docs/unrouted.md"],
            "summary": {
                "changed_file_count": 2,
                "routed_file_count": 1,
                "unmatched_file_count": 1,
                "skipped_lane_count": 1,
                "skipped_reason_counts": {
                    "deep_lane_requires_label": 1
                }
            },
            "skipped_by_policy": [
                {
                    "lane": "proptest_smoke",
                    "reason": "deep_lane_requires_label",
                    "matched_files": ["docs/handoff.md"],
                    "blocking": true,
                    "estimated_lem": 8,
                    "estimate_source": "static"
                }
            ]
        });

        let summary = summarize(&value);

        assert_eq!(summary.schema_version, Some(5));
        assert_eq!(summary.changed_file_count, 2);
        assert_eq!(summary.routed_file_count, 1);
        assert_eq!(summary.unmatched_file_count, 1);
        assert_eq!(summary.skipped_lane_count, 1);
        assert_eq!(summary.skipped_blocking_lanes, 1);
        assert_eq!(summary.surfaces, vec!["handoff_review_packet"]);
        assert_eq!(
            summary.skipped_reason_counts,
            vec![("deep_lane_requires_label".to_string(), 1)]
        );
        assert_eq!(summary.first_changed_files[0].path, "docs/handoff.md");
        assert_eq!(
            summary.first_changed_files[0].proof_packs,
            vec!["handoff_review_packet"]
        );
        assert_eq!(summary.first_unmatched_files, vec!["docs/unrouted.md"]);
        assert_eq!(summary.first_skipped_lanes[0].lane, "proptest_smoke");
        assert_eq!(summary.first_skipped_lanes[0].estimated_lem, Some(8));
        assert_eq!(
            summary.first_skipped_lanes[0].estimate_source.as_deref(),
            Some("static")
        );
    }

    #[test]
    fn proof_route_summary_accepts_v4_path_and_proof_pack_names() {
        let value = serde_json::json!({
            "schema": "tokmd.proof_pack_route.v1",
            "schema_version": 4,
            "changed_files": [
                {
                    "path": "docs/handoff.md",
                    "surface": "handoff_review_packet",
                    "proof_packs": ["handoff_review_packet"]
                }
            ],
            "unmatched_files": [],
            "summary": {
                "changed_file_count": 1,
                "routed_file_count": 1,
                "unmatched_file_count": 0,
                "skipped_lane_count": 0,
                "skipped_reason_counts": {}
            },
            "skipped_by_policy": []
        });

        let summary = summarize(&value);

        assert_eq!(summary.schema_version, Some(4));
        assert_eq!(summary.first_changed_files[0].path, "docs/handoff.md");
        assert_eq!(
            summary.first_changed_files[0].proof_packs,
            vec!["handoff_review_packet"]
        );
    }
}
