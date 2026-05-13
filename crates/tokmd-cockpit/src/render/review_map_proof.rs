//! Proof evidence matching and proof-reference rendering for review maps.

use std::collections::BTreeSet;

use crate::proof_evidence::{
    NormalizedProofEvidence, ProofEvidenceAvailability, ProofEvidenceInput, ProofExecutionStatus,
    normalize_proof_evidence,
};
use crate::{CockpitReceipt, CommitMatch, ReviewItem};

pub(super) struct ReviewMapProofRef {
    changed_files: BTreeSet<String>,
    refs: Vec<String>,
    line: String,
}

pub(super) fn review_map_proof_refs(
    receipt: &CockpitReceipt,
    proof_inputs: &[ProofEvidenceInput],
) -> Vec<ReviewMapProofRef> {
    let mut proof_refs = Vec::new();
    let mut proof_index = 0;

    for input in proof_inputs {
        let normalized = normalize_proof_evidence(
            &input.artifact,
            input.source_path.clone(),
            Some(&receipt.base_ref),
            Some(&receipt.head_ref),
        );
        let changed_files = unambiguous_changed_files(input.artifact.changed_files(), &normalized);

        for proof in normalized {
            let mut refs = Vec::with_capacity(1 + proof.artifact_refs.len());
            refs.push(format!("evidence.json#/proof/{proof_index}"));
            refs.extend(proof.artifact_refs.iter().cloned());
            proof_refs.push(ReviewMapProofRef {
                changed_files: changed_files.clone(),
                refs,
                line: review_map_proof_line(&proof),
            });
            proof_index += 1;
        }
    }

    proof_refs
}

fn unambiguous_changed_files(
    changed_files: &[String],
    normalized: &[NormalizedProofEvidence],
) -> BTreeSet<String> {
    let scopes = normalized
        .iter()
        .filter_map(|proof| proof.scope.as_deref())
        .collect::<BTreeSet<_>>();

    if normalized.len() > 1 && scopes.len() > 1 {
        return BTreeSet::new();
    }

    changed_files
        .iter()
        .map(|path| normalize_path_for_match(path))
        .collect()
}

pub(super) struct ReviewMapItemProof {
    pub(super) lines: Vec<String>,
    pub(super) refs: Vec<String>,
}

pub(super) fn review_map_item_proof(
    item: &ReviewItem,
    proof_refs: &[ReviewMapProofRef],
) -> ReviewMapItemProof {
    let item_path = normalize_path_for_match(&item.path);
    let mut refs = BTreeSet::new();
    let mut seen_lines = BTreeSet::new();
    let mut lines = Vec::new();

    for proof_ref in proof_refs {
        if !proof_ref.changed_files.contains(&item_path) {
            continue;
        }

        refs.extend(proof_ref.refs.iter().cloned());
        if seen_lines.insert(proof_ref.line.clone()) {
            lines.push(proof_ref.line.clone());
        }
    }

    ReviewMapItemProof {
        lines,
        refs: refs.into_iter().collect(),
    }
}

fn review_map_proof_line(proof: &NormalizedProofEvidence) -> String {
    let class = if proof.required {
        "Required"
    } else {
        "Advisory"
    };
    let scope = proof
        .scope
        .as_deref()
        .unwrap_or_else(|| proof.kind.as_str());
    let mut line = format!(
        "{}: {} {} ({}, freshness: {})",
        class,
        scope,
        execution_status_label(proof.execution_status),
        availability_label(proof.availability),
        commit_match_label(proof.commit_match),
    );

    if let Some(command) = proof
        .command
        .as_deref()
        .filter(|command| !command.is_empty())
    {
        line.push_str(" - ");
        line.push_str(command);
    }

    line
}

fn execution_status_label(status: ProofExecutionStatus) -> &'static str {
    match status {
        ProofExecutionStatus::Planned => "planned",
        ProofExecutionStatus::ExecutedPassed => "passed",
        ProofExecutionStatus::ExecutedFailed => "failed",
        ProofExecutionStatus::NotExecuted => "not run",
        ProofExecutionStatus::DryRun => "dry run",
    }
}

fn availability_label(availability: ProofEvidenceAvailability) -> &'static str {
    match availability {
        ProofEvidenceAvailability::Available => "available",
        ProofEvidenceAvailability::Missing => "missing",
        ProofEvidenceAvailability::Skipped => "skipped",
        ProofEvidenceAvailability::Stale => "stale",
        ProofEvidenceAvailability::Degraded => "degraded",
        ProofEvidenceAvailability::Unavailable => "unavailable",
    }
}

fn commit_match_label(commit_match: CommitMatch) -> &'static str {
    match commit_match {
        CommitMatch::Exact => "exact",
        CommitMatch::Partial => "partial",
        CommitMatch::Stale => "stale",
        CommitMatch::Unknown => "unknown",
    }
}

fn normalize_path_for_match(path: &str) -> String {
    path.replace('\\', "/")
}

pub(super) fn write_proof_block(s: &mut String, proof: &ReviewMapItemProof) {
    use std::fmt::Write;

    if proof.lines.is_empty() {
        return;
    }

    let _ = writeln!(s, "   Proof:");
    for line in &proof.lines {
        let _ = writeln!(s, "   - {line}");
    }
    let _ = writeln!(s, "   Proof references:");
    for reference in &proof.refs {
        let _ = writeln!(s, "   - {reference}");
    }
}
