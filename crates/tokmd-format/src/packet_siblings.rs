//! Derive packet preset sections from sibling bundle files.

use std::collections::BTreeMap;

use tokmd_types::{
    ManualCandidateRecord, PacketPresetInput, PacketSiblingInputs, ReviewCardRecord,
    TokmdPacketsManifest,
};

/// Resolve the effective preset input from manifest and optional sibling files.
pub fn resolve_preset_input(
    manifest: &TokmdPacketsManifest,
    siblings: &PacketSiblingInputs,
    preset: &str,
) -> (Option<PacketPresetInput>, Vec<String>) {
    let mut notes = Vec::new();

    if let Some(manifest_input) = manifest.preset_inputs.get(preset) {
        if !manifest_input.sections.is_empty() {
            return (Some(manifest_input.clone()), notes);
        }
        notes.push(format!(
            "manifest `preset_inputs[{preset}]` is present but has no `sections`; attempting sibling supplementation"
        ));
        if let Some(mut derived) = derive_from_siblings(siblings, preset, &mut notes) {
            derived
                .limitations
                .extend(manifest_input.limitations.iter().cloned());
            derived
                .missing_sections
                .extend(manifest_input.missing_sections.iter().cloned());
            return (Some(derived), notes);
        }
        return (Some(manifest_input.clone()), notes);
    }

    if let Some(derived) = derive_from_siblings(siblings, preset, &mut notes) {
        return (Some(derived), notes);
    }

    notes.push(format!(
        "no manifest `preset_inputs[{preset}]` and no derivable sibling sections"
    ));
    (None, notes)
}

fn derive_from_siblings(
    siblings: &PacketSiblingInputs,
    preset: &str,
    notes: &mut Vec<String>,
) -> Option<PacketPresetInput> {
    let candidate = siblings
        .manual_candidates
        .as_ref()
        .and_then(|file| file.candidates.first());

    let cards = siblings.cards.as_ref().map(|file| file.cards.as_slice());

    match preset {
        "bun-ub-handoff" => candidate.map(|c| derive_handoff(c, notes)),
        "bun-ub-pr-body" => candidate.map(|c| derive_pr_body(c, notes)),
        "bun-ub-ledger-note" => candidate.map(|c| derive_ledger_note(c, notes)),
        "bun-ub-review-map" => Some(derive_review_map(candidate, cards, notes)),
        "bun-ub-next-pick" => candidate.map(|c| derive_next_pick(c, notes)),
        _ => None,
    }
}

fn derive_handoff(candidate: &ManualCandidateRecord, notes: &mut Vec<String>) -> PacketPresetInput {
    notes.push(
        "sections derived from `manual-candidates.json` because manifest `preset_inputs` was absent or empty"
            .into(),
    );
    let mut sections = BTreeMap::new();
    if let Some(body) = candidate_identity(candidate) {
        sections.insert("candidate_identity".into(), body);
    }
    if let Some(body) = candidate.operation_family.as_deref() {
        sections.insert("stable_byte_family".into(), body.into());
    }
    if let Some(body) = candidate.safe_caller.as_deref() {
        sections.insert("safe_js_caller_route".into(), body.into());
    }
    if let Some(body) = candidate.location_text.as_deref() {
        sections.insert("rust_native_seam".into(), body.into());
    }
    if let Some(body) = candidate.proof_mode.as_deref() {
        sections.insert("proof_mode".into(), body.into());
    }
    if let Some(body) = candidate.invariant.as_deref() {
        sections.insert("invariant_at_risk".into(), body.into());
    }
    if let Some(body) = candidate.fix_boundary.as_deref() {
        sections.insert("suggested_fix_boundary".into(), body.into());
    }
    if let Some(body) = join_lines(&candidate.test_targets) {
        sections.insert("test_or_witness_target".into(), body);
    }
    if let Some(body) = join_lines(&candidate.do_not_touch) {
        sections.insert("do_not_touch".into(), body);
    }

    let missing_sections = missing_handoff_sections(&sections);
    PacketPresetInput {
        sections,
        limitations: vec![
            "sibling-derived handoff sections are partial; producer `preset_inputs` is authoritative when present"
                .into(),
        ],
        missing_sections,
    }
}

fn derive_pr_body(candidate: &ManualCandidateRecord, notes: &mut Vec<String>) -> PacketPresetInput {
    notes.push(
        "sections derived from `manual-candidates.json` because manifest `preset_inputs` was absent or empty"
            .into(),
    );
    let mut sections = BTreeMap::new();
    if let Some(body) = candidate.title.as_deref() {
        sections.insert("problem_statement".into(), body.into());
    }
    if let Some(body) = candidate.invariant.as_deref() {
        sections.insert("user_visible_or_invariant_risk".into(), body.into());
    }
    if let Some(body) = candidate
        .pr_aperture
        .as_deref()
        .or(candidate.fix_boundary.as_deref())
    {
        sections.insert("smallest_changed_surface".into(), body.into());
    }
    if let Some(body) = join_lines(&candidate.do_not_touch) {
        sections.insert("non_goals".into(), body);
    }
    sections.insert(
        "exact_claims_not_made".into(),
        "No UB proof; no runtime oracle executed by this renderer.".into(),
    );

    PacketPresetInput {
        sections,
        limitations: vec![
            "sibling-derived PR-body sections omit compatibility oracle and external evidence unless producer supplies `preset_inputs`"
                .into(),
        ],
        missing_sections: vec![],
    }
}

fn derive_ledger_note(
    candidate: &ManualCandidateRecord,
    notes: &mut Vec<String>,
) -> PacketPresetInput {
    notes.push(
        "sections derived from `manual-candidates.json` because manifest `preset_inputs` was absent or empty"
            .into(),
    );
    let mut sections = BTreeMap::new();
    if let Some(body) = candidate.id.as_deref() {
        sections.insert("seed_or_candidate_id".into(), body.into());
    }
    sections.insert(
        "ledger_state_transition".into(),
        "not available from sibling index alone".into(),
    );
    if let Some(body) = join_lines(&candidate.do_not_touch) {
        sections.insert("remaining_outside_aperture".into(), body);
    }

    PacketPresetInput {
        sections,
        limitations: vec![
            "ledger note requires producer transition metadata; sibling index does not justify state changes"
                .into(),
        ],
        missing_sections: vec![
            "old ledger state".into(),
            "new ledger state".into(),
            "evidence or PR receipt".into(),
        ],
    }
}

fn derive_review_map(
    candidate: Option<&ManualCandidateRecord>,
    cards: Option<&[ReviewCardRecord]>,
    notes: &mut Vec<String>,
) -> PacketPresetInput {
    notes.push(
        "sections derived from sibling bundle files because manifest `preset_inputs` was absent or empty"
            .into(),
    );
    let mut sections = BTreeMap::new();
    let mut limitations = Vec::new();

    if let Some(candidate) = candidate {
        if let Some(body) = candidate_identity(candidate) {
            sections.insert("manual_candidate_identity".into(), body);
        }
        if let Some(body) = candidate.location_text.as_deref() {
            sections.insert("changed_unsafe_native_seams".into(), body.into());
        }
    } else {
        limitations.push(
            "`manual-candidates.json` had no candidates for review-map supplementation".into(),
        );
    }

    if let Some(cards) = cards {
        if cards.is_empty() {
            limitations.push("`cards.json` was present but contained no ReviewCards".into());
        } else {
            let body = cards
                .iter()
                .filter_map(review_card_line)
                .collect::<Vec<_>>()
                .join("\n");
            sections.insert("reviewcard_ids_and_seams".into(), body);
        }
    } else {
        limitations.push("`cards.json` was not ingested; ReviewCard seams are unavailable".into());
    }

    sections.insert(
        "explicit_no_posting_boundary".into(),
        "tokmd render does not post comments and sibling ingestion does not authorize posting"
            .into(),
    );

    PacketPresetInput {
        sections,
        limitations,
        missing_sections: vec!["comment-plan.json selection summary".into()],
    }
}

fn derive_next_pick(
    candidate: &ManualCandidateRecord,
    notes: &mut Vec<String>,
) -> PacketPresetInput {
    notes.push(
        "sections derived from `manual-candidates.json` because manifest `preset_inputs` was absent or empty"
            .into(),
    );
    let mut sections = BTreeMap::new();
    if let Some(body) = candidate_identity(candidate) {
        sections.insert("ranked_next_candidate".into(), body);
    }
    if let Some(body) = candidate.proof_mode.as_deref() {
        sections.insert("proof_mode".into(), body.into());
    }
    if let Some(body) = candidate
        .pr_aperture
        .as_deref()
        .or(candidate.fix_boundary.as_deref())
    {
        sections.insert("smallest_first_pr".into(), body.into());
    }
    if let Some(body) = join_lines(&candidate.do_not_touch) {
        sections.insert("non_goals".into(), body);
    }

    PacketPresetInput {
        sections,
        limitations: vec![
            "next-pick ranking is not recomputed; first manual candidate is surfaced for formatting only"
                .into(),
        ],
        missing_sections: vec!["owner lane".into(), "dependencies or parked-followup unblock".into()],
    }
}

fn candidate_identity(candidate: &ManualCandidateRecord) -> Option<String> {
    match (candidate.id.as_deref(), candidate.title.as_deref()) {
        (Some(id), Some(title)) => Some(format!("{id} / {title}")),
        (Some(id), None) => Some(id.into()),
        (None, Some(title)) => Some(title.into()),
        (None, None) => None,
    }
}

fn review_card_line(card: &ReviewCardRecord) -> Option<String> {
    let id = card.id.as_deref()?;
    let location = card
        .location_text
        .as_deref()
        .or(card.unsafe_operation.as_deref())
        .unwrap_or("location unknown");
    Some(format!("ReviewCard {id} @ {location}"))
}

fn join_lines(values: &[String]) -> Option<String> {
    if values.is_empty() {
        None
    } else {
        Some(values.join("; "))
    }
}

fn missing_handoff_sections(sections: &BTreeMap<String, String>) -> Vec<String> {
    const REQUIRED: &[&str] = &[
        "candidate_identity",
        "stable_byte_family",
        "safe_js_caller_route",
        "rust_native_seam",
        "proof_mode",
        "suggested_fix_boundary",
        "test_or_witness_target",
        "do_not_touch",
        "ledger_state_and_next_action",
        "pr_aperture_and_stop_line",
    ];
    REQUIRED
        .iter()
        .filter(|name| !sections.contains_key(**name))
        .map(|name| (*name).to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokmd_types::{
        MANUAL_CANDIDATES_SCHEMA, ManualCandidatesFile, PacketSiblingInputs, TOKMD_PACKETS_SCHEMA,
        TokmdPacketsManifest,
    };

    fn sample_candidate() -> ManualCandidateRecord {
        ManualCandidateRecord {
            id: Some("seed-42".into()),
            title: Some("MarkdownObject UTF-8 boundary".into()),
            invariant: Some("UTF-8 bounds".into()),
            safe_caller: Some("TextDecoder.decode".into()),
            location_text: Some("src/runtime/api/MarkdownObject.rs:120".into()),
            proof_mode: Some("manual witness required".into()),
            fix_boundary: Some("narrow UTF-8 validation at JS seam only".into()),
            test_targets: vec!["utf8-boundary-fixture".into()],
            ..ManualCandidateRecord::default()
        }
    }

    #[test]
    fn derives_handoff_from_manual_candidates_when_manifest_empty() {
        let manifest = TokmdPacketsManifest {
            schema: TOKMD_PACKETS_SCHEMA.into(),
            producer: None,
            inputs_present: vec!["manual-candidates.json".into()],
            inputs_absent: vec![],
            non_claims: vec![],
            preset_inputs: BTreeMap::new(),
        };
        let siblings = PacketSiblingInputs {
            manual_candidates: Some(ManualCandidatesFile {
                schema_version: MANUAL_CANDIDATES_SCHEMA.into(),
                candidates: vec![sample_candidate()],
            }),
            ..PacketSiblingInputs::default()
        };
        let (input, notes) = resolve_preset_input(&manifest, &siblings, "bun-ub-handoff");
        let input = input.expect("derived input");
        assert!(input.sections.contains_key("candidate_identity"));
        assert!(
            notes
                .iter()
                .any(|note| note.contains("manual-candidates.json"))
        );
        assert!(!input.missing_sections.is_empty());
    }

    #[test]
    fn manifest_preset_inputs_take_precedence_over_siblings() {
        let mut sections = BTreeMap::new();
        sections.insert("candidate_identity".into(), "from manifest".into());
        let manifest = TokmdPacketsManifest {
            schema: TOKMD_PACKETS_SCHEMA.into(),
            producer: None,
            inputs_present: vec!["manual-candidates.json".into()],
            inputs_absent: vec![],
            non_claims: vec![],
            preset_inputs: BTreeMap::from([(
                "bun-ub-handoff".into(),
                PacketPresetInput {
                    sections,
                    limitations: vec![],
                    missing_sections: vec![],
                },
            )]),
        };
        let siblings = PacketSiblingInputs {
            manual_candidates: Some(ManualCandidatesFile {
                schema_version: MANUAL_CANDIDATES_SCHEMA.into(),
                candidates: vec![sample_candidate()],
            }),
            ..PacketSiblingInputs::default()
        };
        let (input, notes) = resolve_preset_input(&manifest, &siblings, "bun-ub-handoff");
        let input = input.expect("manifest input");
        assert_eq!(
            input.sections.get("candidate_identity").map(String::as_str),
            Some("from manifest")
        );
        assert!(notes.is_empty());
    }
}
