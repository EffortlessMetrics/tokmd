//! Review-map artifact rendering for cockpit review packets.

use serde_json::{Value, json};

use crate::doc_artifacts_evidence::{DOC_ARTIFACTS_PACKET_PATH, DocArtifactsEvidenceInput};
use crate::proof_evidence::ProofEvidenceInput;
use crate::{CockpitReceipt, GateMeta, ReviewItem};

use super::evidence::{
    doc_artifacts_expected, evidence_availability_optional, evidence_counts,
    review_item_is_source_of_truth, review_packet_evidence_capabilities,
    review_packet_evidence_gate_specs, review_packet_evidence_summary,
};
use super::proof_summary::proof_evidence_summary;
use super::review_map_proof::{
    ReviewMapProofRef, review_map_item_proof, review_map_proof_refs, write_proof_block,
};

pub(super) fn review_packet_review_map(
    receipt: &CockpitReceipt,
    proof_inputs: &[ProofEvidenceInput],
    doc_artifacts: Option<&DocArtifactsEvidenceInput>,
    review_packet_dir: &str,
) -> Value {
    let proof_refs = review_map_proof_refs(receipt, proof_inputs);
    let has_doc_artifacts_evidence = doc_artifacts.is_some() || doc_artifacts_expected(receipt);
    let evidence_refs =
        review_map_evidence_refs(!proof_refs.is_empty(), has_doc_artifacts_evidence);
    let ordered_items = ordered_review_map_items(receipt);
    let context = ReviewMapRenderContext {
        receipt,
        proof_refs: &proof_refs,
        doc_artifacts,
        review_packet_dir,
    };
    let items: Vec<_> = ordered_items
        .iter()
        .enumerate()
        .map(|(rank, ordered)| {
            review_map_item(
                rank + 1,
                ordered.source_index,
                ordered.item,
                &ordered.evidence,
                &context,
            )
        })
        .collect();

    json!({
        "schema": "tokmd.review_map.v1",
        "base_ref": receipt.base_ref,
        "head_ref": receipt.head_ref,
        "source": "cockpit.review_plan",
        "evidence": {
            "summary": review_packet_evidence_summary(receipt),
            "groups": review_packet_evidence_capabilities(receipt),
            "refs": evidence_refs,
        },
        "item_count": items.len(),
        "items": items,
    })
}

struct ReviewMapRenderContext<'a> {
    receipt: &'a CockpitReceipt,
    proof_refs: &'a [ReviewMapProofRef],
    doc_artifacts: Option<&'a DocArtifactsEvidenceInput>,
    review_packet_dir: &'a str,
}

fn review_map_item(
    rank: usize,
    source_index: usize,
    item: &ReviewItem,
    evidence: &ReviewMapItemEvidence,
    context: &ReviewMapRenderContext<'_>,
) -> Value {
    let proof = review_map_item_proof(item, context.proof_refs);
    let doc_artifacts_refs = review_map_item_doc_artifacts_refs(item, context.doc_artifacts);
    let reproduce = review_map_item_reproduce(item, context.receipt, context.review_packet_dir);

    json!({
        "rank": rank,
        "source_index": source_index,
        "path": &item.path,
        "priority": item.priority,
        "priority_label": review_priority_label(item.priority),
        "reason": &item.reason,
        "complexity": item.complexity,
        "lines_changed": item.lines_changed,
        "evidence_refs": [
            format!("cockpit.json#/review_plan/{source_index}"),
            "evidence.json#/gates",
        ],
        "proof_refs": proof.refs,
        "doc_artifacts_refs": doc_artifacts_refs,
        "evidence": {
            "status": evidence.status(),
            "present": &evidence.present,
            "missing": &evidence.missing,
            "degraded": &evidence.degraded,
            "stale": &evidence.stale,
            "skipped": &evidence.skipped,
            "unavailable": &evidence.unavailable,
            "refs": ["evidence.json#/gates"],
        },
        "reproduce": reproduce,
    })
}

struct OrderedReviewMapItem<'a> {
    source_index: usize,
    item: &'a ReviewItem,
    evidence: ReviewMapItemEvidence,
}

fn ordered_review_map_items(receipt: &CockpitReceipt) -> Vec<OrderedReviewMapItem<'_>> {
    let mut items: Vec<_> = receipt
        .review_plan
        .iter()
        .enumerate()
        .map(|(source_index, item)| OrderedReviewMapItem {
            source_index,
            item,
            evidence: review_map_item_evidence(item, receipt),
        })
        .collect();

    items.sort_by(|a, b| {
        review_order_bucket(a.item, &a.evidence)
            .cmp(&review_order_bucket(b.item, &b.evidence))
            .then_with(|| a.item.priority.cmp(&b.item.priority))
            .then_with(|| {
                b.item
                    .complexity
                    .unwrap_or(0)
                    .cmp(&a.item.complexity.unwrap_or(0))
            })
            .then_with(|| {
                b.item
                    .lines_changed
                    .unwrap_or(0)
                    .cmp(&a.item.lines_changed.unwrap_or(0))
            })
            .then_with(|| a.item.path.cmp(&b.item.path))
            .then_with(|| a.source_index.cmp(&b.source_index))
    });

    items
}

fn review_order_bucket(item: &ReviewItem, evidence: &ReviewMapItemEvidence) -> u8 {
    if review_item_is_source_of_truth(item) {
        0
    } else if !evidence.missing.is_empty() {
        1
    } else if !evidence.stale.is_empty() {
        2
    } else if !evidence.degraded.is_empty() {
        3
    } else if item.complexity.unwrap_or(0) >= 4 {
        4
    } else if review_contract_path(&item.path) {
        5
    } else if item.priority <= 1 {
        6
    } else if item.priority == 2 {
        7
    } else if !evidence.present.is_empty() {
        8
    } else if !evidence.skipped.is_empty() {
        9
    } else {
        10
    }
}

fn review_order_reason(item: &ReviewItem, evidence: &ReviewMapItemEvidence) -> &'static str {
    if review_item_is_source_of_truth(item) {
        "Source-of-truth artifact changed; review the governing contract before ordinary files."
    } else if !evidence.missing.is_empty() {
        "Evidence is missing for this item; repair or acknowledge the missing proof before relying on it."
    } else if !evidence.stale.is_empty() {
        "Evidence is stale for this item; regenerate it before treating the packet as current."
    } else if !evidence.degraded.is_empty() {
        "Evidence is degraded for this item; inspect the degraded receipt before relying on it."
    } else if item.complexity.unwrap_or(0) >= 4 {
        "High review complexity; inspect before lower-complexity changes."
    } else if review_contract_path(&item.path) {
        "Contract or policy path changed; review before ordinary implementation changes."
    } else if item.priority <= 1 {
        "Highest cockpit priority from the source review plan."
    } else if item.priority == 2 {
        "Medium cockpit priority from the source review plan."
    } else if !evidence.present.is_empty() {
        "Evidence is available; use the attached references to review efficiently."
    } else if !evidence.skipped.is_empty() {
        "Evidence was skipped; confirm the skip reason is expected."
    } else {
        "Lower-priority source review item; review after higher-risk signals."
    }
}

fn review_contract_path(path: &str) -> bool {
    schema_review_path(path)
        || policy_review_path(path)
        || cli_review_path(path)
        || api_review_path(path)
}

fn schema_review_path(path: &str) -> bool {
    path == "docs/schema.json"
        || path == "docs/SCHEMA.md"
        || path.starts_with("docs/") && path.ends_with(".schema.json")
        || path.starts_with("crates/tokmd/schemas/")
}

fn policy_review_path(path: &str) -> bool {
    path == "ci/proof.toml"
        || path == "codecov.yml"
        || path.starts_with("policy/")
        || path.starts_with(".github/workflows/")
}

fn cli_review_path(path: &str) -> bool {
    path.starts_with("crates/tokmd/src/commands/")
        || path.starts_with("crates/tokmd/src/cli/")
        || path == "crates/tokmd/src/config.rs"
}

fn api_review_path(path: &str) -> bool {
    path.ends_with("lib.rs") || path.ends_with("mod.rs")
}

fn review_map_item_reproduce(
    item: &ReviewItem,
    receipt: &CockpitReceipt,
    review_packet_dir: &str,
) -> Vec<String> {
    let mut commands = vec![
        format!(
            "tokmd cockpit --base {} --head {} --format json",
            receipt.base_ref, receipt.head_ref
        ),
        format!(
            "tokmd cockpit --base {} --head {} --review-packet-dir {}",
            receipt.base_ref, receipt.head_ref, review_packet_dir
        ),
    ];

    if review_item_is_source_of_truth(item) {
        commands.push(
            "cargo xtask doc-artifacts --check --json target/docs/doc-artifacts-check.json"
                .to_string(),
        );
    }

    commands
}

fn review_map_evidence_refs(
    has_proof: bool,
    has_doc_artifacts_evidence: bool,
) -> Vec<&'static str> {
    let mut refs = vec!["evidence.json#/gates"];
    if has_proof {
        refs.push("evidence.json#/proof");
    }
    if has_doc_artifacts_evidence {
        refs.push("evidence.json#/doc_artifacts");
    }
    refs
}

fn review_map_item_doc_artifacts_refs(
    item: &ReviewItem,
    doc_artifacts: Option<&DocArtifactsEvidenceInput>,
) -> Vec<&'static str> {
    if doc_artifacts.is_some() && review_item_is_source_of_truth(item) {
        vec!["evidence.json#/doc_artifacts", DOC_ARTIFACTS_PACKET_PATH]
    } else {
        Vec::new()
    }
}

#[derive(Default)]
struct ReviewMapItemEvidence {
    present: Vec<&'static str>,
    missing: Vec<&'static str>,
    degraded: Vec<&'static str>,
    stale: Vec<&'static str>,
    skipped: Vec<&'static str>,
    unavailable: Vec<&'static str>,
}

impl ReviewMapItemEvidence {
    fn status(&self) -> &'static str {
        if !self.missing.is_empty() {
            "missing"
        } else if !self.stale.is_empty() {
            "stale"
        } else if !self.degraded.is_empty() {
            "degraded"
        } else if !self.present.is_empty() {
            "available"
        } else if !self.skipped.is_empty() {
            "skipped"
        } else {
            "unavailable"
        }
    }
}

fn review_map_item_evidence(item: &ReviewItem, receipt: &CockpitReceipt) -> ReviewMapItemEvidence {
    let mut evidence = ReviewMapItemEvidence::default();

    for (id, meta) in review_packet_evidence_gate_specs(receipt) {
        if !evidence_gate_applies_to_item(meta, item) {
            continue;
        }

        match evidence_availability_optional(meta) {
            "available" => evidence.present.push(id),
            "missing" => evidence.missing.push(id),
            "degraded" => evidence.degraded.push(id),
            "stale" => evidence.stale.push(id),
            "skipped" => evidence.skipped.push(id),
            "unavailable" => evidence.unavailable.push(id),
            _ => {}
        }
    }

    evidence
}

fn evidence_gate_applies_to_item(meta: Option<&GateMeta>, item: &ReviewItem) -> bool {
    let Some(meta) = meta else {
        return false;
    };

    let is_global = meta.scope.relevant.is_empty() && meta.scope.tested.is_empty();
    is_global
        || meta.scope.relevant.iter().any(|path| path == &item.path)
        || meta.scope.tested.iter().any(|path| path == &item.path)
}

fn review_priority_label(priority: u32) -> &'static str {
    match priority {
        1 => "highest",
        2 => "medium",
        _ => "low",
    }
}

pub(super) fn render_review_map_md(
    receipt: &CockpitReceipt,
    proof_inputs: &[ProofEvidenceInput],
    doc_artifacts: Option<&DocArtifactsEvidenceInput>,
    review_packet_dir: &str,
) -> String {
    use std::fmt::Write;

    let mut s = String::new();
    let proof_refs = review_map_proof_refs(receipt, proof_inputs);
    let _ = writeln!(s, "# Review Map");
    let _ = writeln!(s);
    let _ = writeln!(s, "Base: `{}`", receipt.base_ref);
    let _ = writeln!(s, "Head: `{}`", receipt.head_ref);
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "Open this as a review work order: inspect items in order, regenerate the listed evidence when it is missing, stale, or degraded, and do not treat this packet as a merge verdict."
    );
    let _ = writeln!(s);

    let evidence = evidence_counts(receipt);
    let _ = writeln!(
        s,
        "Evidence overview: {} available, {} degraded, {} stale, {} skipped, {} unavailable, {} missing.",
        evidence.available,
        evidence.degraded,
        evidence.stale,
        evidence.skipped,
        evidence.unavailable,
        evidence.missing,
    );
    let _ = writeln!(s);
    write_proof_overview(&mut s, receipt, proof_inputs);
    write_doc_artifacts_overview(&mut s, receipt, doc_artifacts);

    if receipt.review_plan.is_empty() {
        let _ = writeln!(s, "No prioritized files were identified.");
        return s;
    }

    let _ = writeln!(s, "## Review First");
    let _ = writeln!(s);

    for (rank, ordered) in ordered_review_map_items(receipt).iter().enumerate() {
        let item = ordered.item;
        let source_index = ordered.source_index;
        let evidence = &ordered.evidence;
        let proof = review_map_item_proof(item, &proof_refs);
        let _ = writeln!(
            s,
            "{}. `{}`
   Priority: {} ({})
   Review-first signal: {}
   Why it matters: {}",
            rank + 1,
            item.path,
            item.priority,
            review_priority_label(item.priority),
            review_order_reason(item, evidence),
            item.reason
        );

        if let Some(lines_changed) = item.lines_changed {
            let _ = writeln!(s, "   Lines changed: {lines_changed}");
        }
        if let Some(complexity) = item.complexity {
            let _ = writeln!(s, "   Review complexity: {complexity}/5");
        }
        let _ = writeln!(s, "   Evidence status: {}", evidence.status());
        write_evidence_list(&mut s, "Evidence present", &evidence.present);
        write_evidence_list(&mut s, "Evidence missing", &evidence.missing);
        write_evidence_list(&mut s, "Evidence degraded", &evidence.degraded);
        write_evidence_list(&mut s, "Evidence stale", &evidence.stale);
        write_evidence_list(&mut s, "Evidence skipped", &evidence.skipped);
        write_evidence_list(&mut s, "Evidence unavailable", &evidence.unavailable);
        write_doc_artifacts_block(&mut s, item, doc_artifacts);
        write_proof_block(&mut s, &proof);
        let _ = writeln!(s, "   Evidence references:");
        let _ = writeln!(s, "   - cockpit.json#/review_plan/{source_index}");
        let _ = writeln!(s, "   - evidence.json#/gates");
        let _ = writeln!(s, "   Reproduce:");
        let _ = writeln!(
            s,
            "   - `tokmd cockpit --base {} --head {} --format json`",
            receipt.base_ref, receipt.head_ref
        );
        let _ = writeln!(
            s,
            "   - `tokmd cockpit --base {} --head {} --review-packet-dir {}`",
            receipt.base_ref, receipt.head_ref, review_packet_dir
        );
        if review_item_is_source_of_truth(item) {
            let _ = writeln!(
                s,
                "   - `cargo xtask doc-artifacts --check --json target/docs/doc-artifacts-check.json`"
            );
        }
        let _ = writeln!(s);
    }

    s
}

fn write_doc_artifacts_overview(
    s: &mut String,
    receipt: &CockpitReceipt,
    doc_artifacts: Option<&DocArtifactsEvidenceInput>,
) {
    use std::fmt::Write;

    match doc_artifacts {
        Some(input) => {
            let _ = writeln!(
                s,
                "Doc artifacts: {} ({} required docs, {} family files, {} active goals).",
                if input.receipt.ok {
                    "verified"
                } else {
                    "degraded"
                },
                input.receipt.checked.required_docs,
                input.receipt.checked.family_files,
                input.receipt.checked.active_goals,
            );
            if !input.receipt.errors.is_empty() {
                let _ = writeln!(s, "- Errors: {}", input.receipt.errors.len());
            }
            let _ = writeln!(s);
        }
        None if doc_artifacts_expected(receipt) => {
            let _ = writeln!(s, "Doc artifacts: missing for source-of-truth changes.");
            let _ = writeln!(s);
        }
        None => {}
    }
}

fn write_doc_artifacts_block(
    s: &mut String,
    item: &ReviewItem,
    doc_artifacts: Option<&DocArtifactsEvidenceInput>,
) {
    use std::fmt::Write;

    if !review_item_is_source_of_truth(item) {
        return;
    }

    match doc_artifacts {
        Some(input) => {
            let _ = writeln!(
                s,
                "   Doc artifacts: {}",
                if input.receipt.ok {
                    "verified"
                } else {
                    "degraded"
                }
            );
            let _ = writeln!(s, "   - evidence.json#/doc_artifacts");
            let _ = writeln!(s, "   - {DOC_ARTIFACTS_PACKET_PATH}");
        }
        None => {
            let _ = writeln!(s, "   Doc artifacts: missing");
        }
    }
}

fn write_proof_overview(
    s: &mut String,
    receipt: &CockpitReceipt,
    proof_inputs: &[ProofEvidenceInput],
) {
    use std::fmt::Write;

    let counts = proof_evidence_summary(receipt, proof_inputs);
    if counts.total == 0 {
        return;
    }

    let _ = writeln!(s, "Proof evidence overview:");
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
        "- Freshness: {} exact, {} partial, {} stale, {} unknown",
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

fn write_evidence_list(s: &mut String, label: &str, gates: &[&str]) {
    use std::fmt::Write;

    if gates.is_empty() {
        return;
    }

    let _ = writeln!(s, "   {label}: {}", gates.join(", "));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(path: &str, priority: u32) -> ReviewItem {
        ReviewItem {
            path: path.to_string(),
            reason: "test".to_string(),
            priority,
            complexity: None,
            lines_changed: None,
        }
    }

    // ---------- review_priority_label ----------

    #[test]
    fn priority_label_one_is_highest() {
        assert_eq!(review_priority_label(1), "highest");
    }

    #[test]
    fn priority_label_two_is_medium() {
        assert_eq!(review_priority_label(2), "medium");
    }

    #[test]
    fn priority_label_zero_is_low() {
        assert_eq!(review_priority_label(0), "low");
    }

    #[test]
    fn priority_label_three_is_low() {
        assert_eq!(review_priority_label(3), "low");
    }

    #[test]
    fn priority_label_large_is_low() {
        assert_eq!(review_priority_label(99), "low");
    }

    // ---------- schema_review_path ----------

    #[test]
    fn schema_review_path_exact_docs_schema_json() {
        assert!(schema_review_path("docs/schema.json"));
    }

    #[test]
    fn schema_review_path_exact_docs_schema_md() {
        assert!(schema_review_path("docs/SCHEMA.md"));
    }

    #[test]
    fn schema_review_path_docs_dot_schema_json() {
        assert!(schema_review_path("docs/foo.schema.json"));
    }

    #[test]
    fn schema_review_path_crates_schemas_dir() {
        assert!(schema_review_path("crates/tokmd/schemas/x.json"));
    }

    #[test]
    fn schema_review_path_rejects_unrelated_paths() {
        assert!(!schema_review_path("src/lib.rs"));
        assert!(!schema_review_path("docs/guide.md"));
        assert!(!schema_review_path("schemas/x.json"));
    }

    // ---------- policy_review_path ----------

    #[test]
    fn policy_review_path_ci_proof_toml() {
        assert!(policy_review_path("ci/proof.toml"));
    }

    #[test]
    fn policy_review_path_codecov_yml() {
        assert!(policy_review_path("codecov.yml"));
    }

    #[test]
    fn policy_review_path_policy_prefix() {
        assert!(policy_review_path("policy/x.toml"));
    }

    #[test]
    fn policy_review_path_github_workflows() {
        assert!(policy_review_path(".github/workflows/x.yml"));
    }

    #[test]
    fn policy_review_path_rejects_unrelated_paths() {
        assert!(!policy_review_path("src/lib.rs"));
        assert!(!policy_review_path("ci/other.toml"));
        assert!(!policy_review_path(".github/dependabot.yml"));
    }

    // ---------- cli_review_path ----------

    #[test]
    fn cli_review_path_commands_dir() {
        assert!(cli_review_path("crates/tokmd/src/commands/run.rs"));
    }

    #[test]
    fn cli_review_path_cli_dir() {
        assert!(cli_review_path("crates/tokmd/src/cli/parser.rs"));
    }

    #[test]
    fn cli_review_path_config_rs() {
        assert!(cli_review_path("crates/tokmd/src/config.rs"));
    }

    #[test]
    fn cli_review_path_rejects_unrelated_paths() {
        assert!(!cli_review_path("crates/tokmd/src/lib.rs"));
        assert!(!cli_review_path("crates/tokmd-cockpit/src/cli/parser.rs"));
        assert!(!cli_review_path("src/config.rs"));
    }

    // ---------- api_review_path ----------

    #[test]
    fn api_review_path_lib_rs() {
        assert!(api_review_path("crates/foo/src/lib.rs"));
    }

    #[test]
    fn api_review_path_mod_rs() {
        assert!(api_review_path("crates/foo/src/mod.rs"));
    }

    #[test]
    fn api_review_path_rejects_unrelated_paths() {
        assert!(!api_review_path("crates/foo/src/main.rs"));
        assert!(!api_review_path("crates/foo/src/util.rs"));
        assert!(!api_review_path("README.md"));
    }

    // ---------- review_contract_path ----------

    #[test]
    fn review_contract_path_matches_schema_predicate() {
        assert!(review_contract_path("docs/schema.json"));
    }

    #[test]
    fn review_contract_path_matches_policy_predicate() {
        assert!(review_contract_path("policy/x.toml"));
    }

    #[test]
    fn review_contract_path_matches_cli_predicate() {
        assert!(review_contract_path("crates/tokmd/src/config.rs"));
    }

    #[test]
    fn review_contract_path_matches_api_predicate() {
        assert!(review_contract_path("crates/foo/src/lib.rs"));
    }

    #[test]
    fn review_contract_path_rejects_unrelated_path() {
        assert!(!review_contract_path("src/main.rs"));
    }

    // ---------- review_map_evidence_refs ----------

    #[test]
    fn evidence_refs_only_gates_when_no_extras() {
        let refs = review_map_evidence_refs(false, false);
        assert_eq!(refs, vec!["evidence.json#/gates"]);
    }

    #[test]
    fn evidence_refs_adds_proof_only() {
        let refs = review_map_evidence_refs(true, false);
        assert_eq!(refs, vec!["evidence.json#/gates", "evidence.json#/proof"]);
    }

    #[test]
    fn evidence_refs_adds_doc_artifacts_only() {
        let refs = review_map_evidence_refs(false, true);
        assert_eq!(
            refs,
            vec!["evidence.json#/gates", "evidence.json#/doc_artifacts"]
        );
    }

    #[test]
    fn evidence_refs_adds_both_proof_and_doc_artifacts() {
        let refs = review_map_evidence_refs(true, true);
        assert_eq!(
            refs,
            vec![
                "evidence.json#/gates",
                "evidence.json#/proof",
                "evidence.json#/doc_artifacts",
            ]
        );
    }

    // ---------- ReviewMapItemEvidence::status ----------

    #[test]
    fn evidence_status_defaults_to_unavailable() {
        let ev = ReviewMapItemEvidence::default();
        assert_eq!(ev.status(), "unavailable");
    }

    #[test]
    fn evidence_status_missing_takes_priority() {
        let mut ev = ReviewMapItemEvidence::default();
        ev.missing.push("g");
        ev.stale.push("g");
        ev.degraded.push("g");
        ev.present.push("g");
        ev.skipped.push("g");
        ev.unavailable.push("g");
        assert_eq!(ev.status(), "missing");
    }

    #[test]
    fn evidence_status_stale_when_no_missing() {
        let mut ev = ReviewMapItemEvidence::default();
        ev.stale.push("g");
        ev.degraded.push("g");
        ev.present.push("g");
        assert_eq!(ev.status(), "stale");
    }

    #[test]
    fn evidence_status_degraded_when_no_stale_or_missing() {
        let mut ev = ReviewMapItemEvidence::default();
        ev.degraded.push("g");
        ev.present.push("g");
        assert_eq!(ev.status(), "degraded");
    }

    #[test]
    fn evidence_status_available_when_only_present() {
        let mut ev = ReviewMapItemEvidence::default();
        ev.present.push("g");
        assert_eq!(ev.status(), "available");
    }

    #[test]
    fn evidence_status_skipped_when_only_skipped() {
        let mut ev = ReviewMapItemEvidence::default();
        ev.skipped.push("g");
        assert_eq!(ev.status(), "skipped");
    }

    #[test]
    fn evidence_status_unavailable_when_only_unavailable() {
        let mut ev = ReviewMapItemEvidence::default();
        ev.unavailable.push("g");
        assert_eq!(ev.status(), "unavailable");
    }

    // ---------- review_order_bucket / review_order_reason ----------

    #[test]
    fn order_bucket_priority_one_falls_in_bucket_six() {
        let it = item("src/main.rs", 1);
        let ev = ReviewMapItemEvidence::default();
        assert_eq!(review_order_bucket(&it, &ev), 6);
    }

    #[test]
    fn order_reason_priority_one_is_highest_priority_message() {
        let it = item("src/main.rs", 1);
        let ev = ReviewMapItemEvidence::default();
        assert_eq!(
            review_order_reason(&it, &ev),
            "Highest cockpit priority from the source review plan."
        );
    }

    #[test]
    fn order_bucket_priority_two_falls_in_bucket_seven() {
        let it = item("src/main.rs", 2);
        let ev = ReviewMapItemEvidence::default();
        assert_eq!(review_order_bucket(&it, &ev), 7);
    }

    #[test]
    fn order_reason_priority_two_is_medium_priority_message() {
        let it = item("src/main.rs", 2);
        let ev = ReviewMapItemEvidence::default();
        assert_eq!(
            review_order_reason(&it, &ev),
            "Medium cockpit priority from the source review plan."
        );
    }

    #[test]
    fn order_bucket_default_low_priority_is_ten() {
        let it = item("src/main.rs", 5);
        let ev = ReviewMapItemEvidence::default();
        assert_eq!(review_order_bucket(&it, &ev), 10);
    }

    #[test]
    fn order_reason_default_low_priority_message() {
        let it = item("src/main.rs", 5);
        let ev = ReviewMapItemEvidence::default();
        assert_eq!(
            review_order_reason(&it, &ev),
            "Lower-priority source review item; review after higher-risk signals."
        );
    }

    #[test]
    fn order_bucket_contract_path_falls_in_bucket_five() {
        // `ci/proof.toml` is a policy contract path, but not a source-of-truth path.
        let it = item("ci/proof.toml", 3);
        let ev = ReviewMapItemEvidence::default();
        assert_eq!(review_order_bucket(&it, &ev), 5);
    }

    #[test]
    fn order_reason_contract_path_message() {
        let it = item("ci/proof.toml", 3);
        let ev = ReviewMapItemEvidence::default();
        assert_eq!(
            review_order_reason(&it, &ev),
            "Contract or policy path changed; review before ordinary implementation changes."
        );
    }
}
