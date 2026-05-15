use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path};

use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::Value;

use crate::cli::ProofObservationStatusArgs;

const DECISION_SCHEMA: &str = "tokmd.proof_observation_decision.v1";
const MODE: &str = "observation_only";

pub fn run(args: ProofObservationStatusArgs) -> Result<()> {
    let packet = build_packet(&args)?;
    write_packet(&args.json, &packet)?;

    println!(
        "proof observation status: wrote {} source artifact(s) to {}",
        packet.source_artifacts.len(),
        args.json.display()
    );
    Ok(())
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct ProofObservationDecisionPacket {
    schema: &'static str,
    ok: bool,
    mode: &'static str,
    source_artifacts: Vec<SourceArtifact>,
    policy_state: PolicyState,
    required_proof: RequiredProofSummary,
    advisory_proof: AdvisoryProofSummary,
    freshness: FreshnessSummary,
    thresholds: ThresholdSummary,
    criteria_met: Vec<DecisionCriterion>,
    criteria_missing: Vec<DecisionCriterion>,
    reproduce: Vec<String>,
    errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct SourceArtifact {
    kind: &'static str,
    path: String,
    schema: Option<String>,
}

#[derive(Debug, Clone)]
struct SourceDocument {
    kind: SourceKind,
    path: String,
    schema: Option<String>,
    value: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum SourceKind {
    Affected,
    ProofPolicy,
    ProofPlan,
    ProofEvidence,
    ProofRunSummary,
    ProofRunObservation,
    ProofRunObservationCollection,
    ExecutorSummary,
    ExecutorManifest,
    ExecutorObservation,
    ExecutorObservationCollection,
    PromotionReadiness,
    CoverageReceipt,
}

impl SourceKind {
    const fn label(self) -> &'static str {
        match self {
            Self::Affected => "affected",
            Self::ProofPolicy => "proof_policy",
            Self::ProofPlan => "proof_plan",
            Self::ProofEvidence => "proof_evidence",
            Self::ProofRunSummary => "proof_run_summary",
            Self::ProofRunObservation => "proof_run_observation",
            Self::ProofRunObservationCollection => "proof_run_observation_collection",
            Self::ExecutorSummary => "executor_summary",
            Self::ExecutorManifest => "executor_manifest",
            Self::ExecutorObservation => "executor_observation",
            Self::ExecutorObservationCollection => "executor_observation_collection",
            Self::PromotionReadiness => "promotion_readiness",
            Self::CoverageReceipt => "coverage_receipt",
        }
    }

    const fn expected_schema(self) -> &'static str {
        match self {
            Self::Affected => "tokmd.affected.v1",
            Self::ProofPolicy => "tokmd.proof_policy.v1",
            Self::ProofPlan => "tokmd.proof_plan.v1",
            Self::ProofEvidence => "tokmd.proof_evidence_plan.v1",
            Self::ProofRunSummary => "tokmd.proof_run_summary.v1",
            Self::ProofRunObservation => "tokmd.proof_run_observation.v1",
            Self::ProofRunObservationCollection => "tokmd.proof_run_observation_collection.v1",
            Self::ExecutorSummary => "tokmd.proof_executor_summary.v1",
            Self::ExecutorManifest => "tokmd.proof_executor_manifest.v1",
            Self::ExecutorObservation => "tokmd.proof_executor_observation.v1",
            Self::ExecutorObservationCollection => "tokmd.proof_executor_observation_collection.v1",
            Self::PromotionReadiness => "tokmd.proof_executor_promotion_readiness.v1",
            Self::CoverageReceipt => "tokmd.coverage_receipt.v1",
        }
    }
}

#[derive(Debug, Default, Serialize, PartialEq, Eq)]
struct PolicyState {
    proof_policy_present: bool,
    executor_pr_required: Option<bool>,
    executor_pr_codecov_upload: Option<bool>,
    promotion_required_gate: Option<bool>,
    promotion_default_codecov_upload: Option<bool>,
    proof_run_pr_required: Option<bool>,
}

#[derive(Debug, Default, Serialize, PartialEq, Eq)]
struct RequiredProofSummary {
    planned: u64,
    executed: u64,
    passed: u64,
    failed: u64,
    observations: u64,
}

#[derive(Debug, Default, Serialize, PartialEq, Eq)]
struct AdvisoryProofSummary {
    planned: u64,
    selected: u64,
    executed: u64,
    passed: u64,
    failed: u64,
    skipped: u64,
    artifacts: u64,
    observations: u64,
}

#[derive(Debug, Default, Serialize, PartialEq, Eq)]
struct FreshnessSummary {
    commit_match: String,
    base: Option<String>,
    head: Option<String>,
    sources: Vec<FreshnessSource>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct FreshnessSource {
    kind: &'static str,
    path: String,
    base: Option<String>,
    head: Option<String>,
}

#[derive(Debug, Default, Serialize, PartialEq, Eq)]
struct ThresholdSummary {
    min_observations: Option<u64>,
    min_executed: Option<u64>,
    min_scopes: Option<u64>,
    min_artifacts: Option<u64>,
    min_passing_collector_runs: Option<u64>,
    observations: Option<u64>,
    executed: Option<u64>,
    scopes: Option<u64>,
    artifacts: Option<u64>,
    passing_collector_runs: Option<u64>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct DecisionCriterion {
    id: &'static str,
    detail: String,
}

fn build_packet(args: &ProofObservationStatusArgs) -> Result<ProofObservationDecisionPacket> {
    let docs = load_sources(args)?;
    if docs.is_empty() {
        bail!("proof-observation-status requires at least one source artifact");
    }

    let source_artifacts = docs
        .iter()
        .map(|doc| SourceArtifact {
            kind: doc.kind.label(),
            path: doc.path.clone(),
            schema: doc.schema.clone(),
        })
        .collect();

    let policy_state = policy_state(&docs);
    let required_proof = required_proof_summary(&docs);
    let advisory_proof = advisory_proof_summary(&docs);
    let freshness = freshness_summary(&docs);
    let thresholds = threshold_summary(&docs);
    let (criteria_met, criteria_missing) = decision_criteria(
        &docs,
        &policy_state,
        &required_proof,
        &advisory_proof,
        &thresholds,
    );
    let reproduce = reproduce_commands(&docs);

    Ok(ProofObservationDecisionPacket {
        schema: DECISION_SCHEMA,
        ok: true,
        mode: MODE,
        source_artifacts,
        policy_state,
        required_proof,
        advisory_proof,
        freshness,
        thresholds,
        criteria_met,
        criteria_missing,
        reproduce,
        errors: Vec::new(),
    })
}

fn load_sources(args: &ProofObservationStatusArgs) -> Result<Vec<SourceDocument>> {
    let sources = [
        (SourceKind::Affected, args.affected.as_ref()),
        (SourceKind::ProofPolicy, args.proof_policy.as_ref()),
        (SourceKind::ProofPlan, args.proof_plan.as_ref()),
        (SourceKind::ProofEvidence, args.proof_evidence.as_ref()),
        (SourceKind::ProofRunSummary, args.proof_run_summary.as_ref()),
        (
            SourceKind::ProofRunObservation,
            args.proof_run_observation.as_ref(),
        ),
        (
            SourceKind::ProofRunObservationCollection,
            args.proof_run_observation_collection.as_ref(),
        ),
        (SourceKind::ExecutorSummary, args.executor_summary.as_ref()),
        (
            SourceKind::ExecutorManifest,
            args.executor_manifest.as_ref(),
        ),
        (
            SourceKind::ExecutorObservation,
            args.executor_observation.as_ref(),
        ),
        (
            SourceKind::ExecutorObservationCollection,
            args.executor_observation_collection.as_ref(),
        ),
        (
            SourceKind::PromotionReadiness,
            args.promotion_readiness.as_ref(),
        ),
        (SourceKind::CoverageReceipt, args.coverage_receipt.as_ref()),
    ];

    sources
        .into_iter()
        .filter_map(|(kind, path)| path.map(|path| (kind, path)))
        .map(|(kind, path)| load_source(kind, path))
        .collect()
}

fn load_source(kind: SourceKind, path: &Path) -> Result<SourceDocument> {
    let display_path = repo_relative_path(path)?;
    let raw = fs::read_to_string(path).with_context(|| format!("read {display_path}"))?;
    let value: Value =
        serde_json::from_str(&raw).with_context(|| format!("parse {display_path}"))?;
    let schema = value
        .get("schema")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    if schema.as_deref() != Some(kind.expected_schema()) {
        bail!(
            "{} artifact `{display_path}` must have schema `{}`, got `{}`",
            kind.label(),
            kind.expected_schema(),
            schema.as_deref().unwrap_or("<missing>")
        );
    }

    Ok(SourceDocument {
        kind,
        path: display_path,
        schema,
        value,
    })
}

fn repo_relative_path(path: &Path) -> Result<String> {
    if path.is_absolute() {
        bail!(
            "source artifact path must be repo-relative: {}",
            path.display()
        );
    }

    let mut normalized = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => normalized.push(part.to_string_lossy().into_owned()),
            Component::CurDir => {}
            Component::ParentDir => bail!(
                "source artifact path must not escape the repo: {}",
                path.display()
            ),
            Component::Prefix(_) | Component::RootDir => bail!(
                "source artifact path must be repo-relative: {}",
                path.display()
            ),
        }
    }

    if normalized.is_empty() {
        bail!("source artifact path must name a file");
    }
    Ok(normalized.join("/"))
}

fn write_packet(path: &Path, packet: &ProofObservationDecisionPacket) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let json =
        serde_json::to_string_pretty(packet).context("serialize proof observation status")?;
    fs::write(path, format!("{json}\n")).with_context(|| format!("write {}", path.display()))
}

fn policy_state(docs: &[SourceDocument]) -> PolicyState {
    let mut state = PolicyState::default();
    if let Some(policy) = find(docs, SourceKind::ProofPolicy) {
        state.proof_policy_present = true;
        state.executor_pr_required = bool_at(&policy.value, &["executor", "pr", "required"]);
        state.executor_pr_codecov_upload =
            bool_at(&policy.value, &["executor", "pr", "codecov_upload"]);
        state.promotion_required_gate =
            bool_at(&policy.value, &["executor", "promotion", "required_gate"]);
        state.promotion_default_codecov_upload = bool_at(
            &policy.value,
            &["executor", "promotion", "default_codecov_upload"],
        );
        state.proof_run_pr_required = bool_at(&policy.value, &["proof_run", "pr", "required"]);
    }
    state
}

fn required_proof_summary(docs: &[SourceDocument]) -> RequiredProofSummary {
    let mut summary = RequiredProofSummary::default();

    if let Some(plan) = find(docs, SourceKind::ProofPlan) {
        summary.planned = summary.planned.max(count_commands(&plan.value, true));
    }
    if let Some(evidence) = find(docs, SourceKind::ProofEvidence) {
        summary.planned = summary
            .planned
            .max(u64_at(&evidence.value, &["counts", "required_total"]).unwrap_or(0));
    }

    for kind in [
        SourceKind::ProofRunSummary,
        SourceKind::ProofRunObservation,
        SourceKind::ProofRunObservationCollection,
    ] {
        if let Some(doc) = find(docs, kind) {
            summary.planned = summary
                .planned
                .max(u64_at(&doc.value, &["counts", "required_planned"]).unwrap_or(0));
            summary.executed = summary
                .executed
                .max(u64_at(&doc.value, &["counts", "executed"]).unwrap_or(0));
            summary.passed = summary
                .passed
                .max(u64_at(&doc.value, &["counts", "passed"]).unwrap_or(0));
            summary.failed = summary
                .failed
                .max(u64_at(&doc.value, &["counts", "failed"]).unwrap_or(0));
        }
    }

    if contains(docs, SourceKind::ProofRunObservation) {
        summary.observations = summary.observations.max(1);
    }
    if let Some(collection) = find(docs, SourceKind::ProofRunObservationCollection) {
        summary.observations = summary
            .observations
            .max(u64_at(&collection.value, &["counts", "observations"]).unwrap_or(0));
    }

    summary
}

fn advisory_proof_summary(docs: &[SourceDocument]) -> AdvisoryProofSummary {
    let mut summary = AdvisoryProofSummary::default();

    if let Some(plan) = find(docs, SourceKind::ProofPlan) {
        summary.planned = summary.planned.max(count_commands(&plan.value, false));
    }
    if let Some(evidence) = find(docs, SourceKind::ProofEvidence) {
        summary.planned = summary
            .planned
            .max(u64_at(&evidence.value, &["counts", "advisory_total"]).unwrap_or(0));
    }

    if let Some(executor) = find(docs, SourceKind::ExecutorSummary) {
        summary.planned = summary
            .planned
            .max(u64_at(&executor.value, &["counts", "family_planned"]).unwrap_or(0));
        summary.selected = summary
            .selected
            .max(u64_at(&executor.value, &["counts", "selected"]).unwrap_or(0));
        summary.skipped = summary
            .skipped
            .max(u64_at(&executor.value, &["counts", "skipped"]).unwrap_or(0));
        summary.executed = summary
            .executed
            .max(u64_at(&executor.value, &["counts", "executed"]).unwrap_or(0));
        summary.passed = summary
            .passed
            .max(u64_at(&executor.value, &["counts", "passed"]).unwrap_or(0));
        summary.failed = summary
            .failed
            .max(u64_at(&executor.value, &["counts", "failed"]).unwrap_or(0));
    }

    for kind in [
        SourceKind::ExecutorObservation,
        SourceKind::ExecutorObservationCollection,
    ] {
        if let Some(doc) = find(docs, kind) {
            summary.selected = summary
                .selected
                .max(u64_at(&doc.value, &["counts", "selected"]).unwrap_or(0));
            summary.executed = summary
                .executed
                .max(u64_at(&doc.value, &["counts", "executed"]).unwrap_or(0));
            summary.passed = summary
                .passed
                .max(u64_at(&doc.value, &["counts", "passed"]).unwrap_or(0));
            summary.failed = summary
                .failed
                .max(u64_at(&doc.value, &["counts", "failed"]).unwrap_or(0));
            summary.artifacts = summary
                .artifacts
                .max(u64_at(&doc.value, &["counts", "artifacts"]).unwrap_or(0));
        }
    }

    if contains(docs, SourceKind::ExecutorObservation) {
        summary.observations = summary.observations.max(1);
    }
    if let Some(collection) = find(docs, SourceKind::ExecutorObservationCollection) {
        summary.observations = summary
            .observations
            .max(u64_at(&collection.value, &["counts", "observations"]).unwrap_or(0));
    }
    if let Some(readiness) = find(docs, SourceKind::PromotionReadiness) {
        summary.executed = summary
            .executed
            .max(u64_at(&readiness.value, &["actuals", "executed"]).unwrap_or(0));
        summary.artifacts = summary
            .artifacts
            .max(u64_at(&readiness.value, &["actuals", "artifacts"]).unwrap_or(0));
    }

    summary
}

fn freshness_summary(docs: &[SourceDocument]) -> FreshnessSummary {
    let sources: Vec<_> = docs
        .iter()
        .filter_map(|doc| {
            let base = string_at(&doc.value, &["base"]);
            let head = string_at(&doc.value, &["head"]);
            if base.is_none() && head.is_none() {
                return None;
            }
            Some(FreshnessSource {
                kind: doc.kind.label(),
                path: doc.path.clone(),
                base,
                head,
            })
        })
        .collect();

    let mut summary = FreshnessSummary {
        commit_match: "unknown".to_string(),
        base: None,
        head: None,
        sources,
    };

    let Some(first) = summary.sources.first() else {
        return summary;
    };
    summary.base = first.base.clone();
    summary.head = first.head.clone();

    if summary.base.is_none() || summary.head.is_none() {
        summary.commit_match = "partial".to_string();
        return summary;
    }

    let all_exact = summary
        .sources
        .iter()
        .all(|source| source.base == summary.base && source.head == summary.head);
    let any_partial = summary
        .sources
        .iter()
        .any(|source| source.base.is_none() || source.head.is_none());

    summary.commit_match = if all_exact {
        "exact".to_string()
    } else if any_partial {
        "partial".to_string()
    } else {
        "stale".to_string()
    };
    summary
}

fn threshold_summary(docs: &[SourceDocument]) -> ThresholdSummary {
    let mut summary = ThresholdSummary::default();

    if let Some(policy) = find(docs, SourceKind::ProofPolicy) {
        summary.min_observations = u64_at(
            &policy.value,
            &["executor", "promotion", "min_observations"],
        );
        summary.min_executed = u64_at(&policy.value, &["executor", "promotion", "min_executed"]);
        summary.min_scopes = u64_at(&policy.value, &["executor", "promotion", "min_scopes"]);
        summary.min_artifacts = u64_at(&policy.value, &["executor", "promotion", "min_artifacts"]);
        summary.min_passing_collector_runs = u64_at(
            &policy.value,
            &["executor", "promotion", "min_passing_collector_runs"],
        );
    }

    if let Some(readiness) = find(docs, SourceKind::PromotionReadiness) {
        summary.min_observations = summary
            .min_observations
            .or_else(|| u64_at(&readiness.value, &["thresholds", "min_observations"]));
        summary.min_executed = summary
            .min_executed
            .or_else(|| u64_at(&readiness.value, &["thresholds", "min_executed"]));
        summary.min_scopes = summary
            .min_scopes
            .or_else(|| u64_at(&readiness.value, &["thresholds", "min_scopes"]));
        summary.min_artifacts = summary
            .min_artifacts
            .or_else(|| u64_at(&readiness.value, &["thresholds", "min_artifacts"]));
        summary.min_passing_collector_runs = summary.min_passing_collector_runs.or_else(|| {
            u64_at(
                &readiness.value,
                &["thresholds", "min_passing_collector_runs"],
            )
        });
        summary.observations = u64_at(&readiness.value, &["actuals", "observations"]);
        summary.executed = u64_at(&readiness.value, &["actuals", "executed"]);
        summary.scopes = u64_at(&readiness.value, &["actuals", "scopes"]);
        summary.artifacts = u64_at(&readiness.value, &["actuals", "artifacts"]);
        summary.passing_collector_runs =
            u64_at(&readiness.value, &["actuals", "passing_collector_runs"]);
    }

    summary
}

fn decision_criteria(
    docs: &[SourceDocument],
    policy_state: &PolicyState,
    required: &RequiredProofSummary,
    advisory: &AdvisoryProofSummary,
    thresholds: &ThresholdSummary,
) -> (Vec<DecisionCriterion>, Vec<DecisionCriterion>) {
    let mut met = Vec::new();
    let mut missing = Vec::new();

    push_presence(
        &mut met,
        &mut missing,
        contains(docs, SourceKind::ProofPolicy),
        "proof_policy_present",
        "checked proof policy artifact was supplied",
        "checked proof policy artifact was not supplied",
    );
    push_presence(
        &mut met,
        &mut missing,
        contains(docs, SourceKind::Affected),
        "affected_present",
        "affected proof routing artifact was supplied",
        "affected proof routing artifact was not supplied",
    );

    if let Some(affected) = find(docs, SourceKind::Affected) {
        let unknown = affected
            .value
            .get("unknown_files")
            .and_then(Value::as_array)
            .map_or(0, Vec::len);
        push_presence(
            &mut met,
            &mut missing,
            unknown == 0,
            "affected_unknown_files",
            "affected proof routing reported zero unknown files",
            "affected proof routing reported unknown files",
        );
    }

    push_presence(
        &mut met,
        &mut missing,
        policy_state.promotion_required_gate == Some(false),
        "promotion_required_gate_off",
        "proof policy keeps executor promotion required_gate disabled",
        "proof policy did not prove executor promotion required_gate is disabled",
    );
    push_presence(
        &mut met,
        &mut missing,
        policy_state.promotion_default_codecov_upload == Some(false),
        "promotion_codecov_upload_off",
        "proof policy keeps default Codecov upload disabled",
        "proof policy did not prove default Codecov upload is disabled",
    );

    push_presence(
        &mut met,
        &mut missing,
        required.executed > 0 && required.failed == 0,
        "required_proof_observed",
        "required proof execution was observed with zero failures",
        "required proof execution was not observed as passing evidence",
    );
    push_presence(
        &mut met,
        &mut missing,
        advisory.executed > 0 && advisory.failed == 0,
        "advisory_proof_observed",
        "advisory executor proof was observed with zero failures",
        "advisory executor proof was not observed as passing evidence",
    );

    if contains(docs, SourceKind::PromotionReadiness) {
        let ready = thresholds
            .min_observations
            .zip(thresholds.observations)
            .is_some_and(|(min, actual)| actual >= min)
            && thresholds
                .min_executed
                .zip(thresholds.executed)
                .is_some_and(|(min, actual)| actual >= min)
            && thresholds
                .min_scopes
                .zip(thresholds.scopes)
                .is_some_and(|(min, actual)| actual >= min)
            && thresholds
                .min_artifacts
                .zip(thresholds.artifacts)
                .is_some_and(|(min, actual)| actual >= min)
            && thresholds
                .min_passing_collector_runs
                .zip(thresholds.passing_collector_runs)
                .is_some_and(|(min, actual)| actual >= min);
        push_presence(
            &mut met,
            &mut missing,
            ready,
            "promotion_thresholds_satisfied",
            "supplied promotion-readiness receipt satisfies policy thresholds",
            "supplied promotion-readiness receipt does not satisfy all policy thresholds",
        );
    } else {
        missing.push(DecisionCriterion {
            id: "promotion_readiness_missing",
            detail: "promotion-readiness receipt was not supplied".to_string(),
        });
    }

    (met, missing)
}

fn push_presence(
    met: &mut Vec<DecisionCriterion>,
    missing: &mut Vec<DecisionCriterion>,
    ok: bool,
    id: &'static str,
    met_detail: &'static str,
    missing_detail: &'static str,
) {
    let target = if ok { met } else { missing };
    target.push(DecisionCriterion {
        id,
        detail: if ok { met_detail } else { missing_detail }.to_string(),
    });
}

fn reproduce_commands(docs: &[SourceDocument]) -> Vec<String> {
    let mut commands = BTreeSet::new();

    for doc in docs {
        match doc.kind {
            SourceKind::Affected => {
                commands.insert(format!(
                    "cargo xtask affected --base origin/main --head HEAD --json-output {}",
                    doc.path
                ));
            }
            SourceKind::ProofPolicy => {
                commands.insert(format!(
                    "cargo xtask proof-policy --json-output {}",
                    doc.path
                ));
            }
            SourceKind::ProofPlan | SourceKind::ProofEvidence => {
                let plan = find(docs, SourceKind::ProofPlan)
                    .map(|doc| doc.path.as_str())
                    .unwrap_or("target/proof/proof-plan.json");
                let evidence = find(docs, SourceKind::ProofEvidence)
                    .map(|doc| doc.path.as_str())
                    .unwrap_or("target/proof/proof-evidence.json");
                commands.insert(format!(
                    "cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json {plan} --evidence-json {evidence}"
                ));
            }
            SourceKind::ProofRunSummary => {
                commands.insert(format!(
                    "cargo xtask proof --profile affected --base origin/main --head HEAD --run-required --allow-local-required-execution --proof-run-summary {}",
                    doc.path
                ));
            }
            SourceKind::ProofRunObservation => {
                commands.insert(format!(
                    "cargo xtask proof-run-observation --proof-run-summary target/proof-run/proof-run-summary.json --output {}",
                    doc.path
                ));
            }
            SourceKind::ProofRunObservationCollection => {
                commands.insert(format!(
                    "cargo xtask proof-run-observations-summary --observations-dir target/proof-run-observations/runs --output {}",
                    doc.path
                ));
            }
            SourceKind::ExecutorSummary | SourceKind::ExecutorManifest => {
                let summary = find(docs, SourceKind::ExecutorSummary)
                    .map(|doc| doc.path.as_str())
                    .unwrap_or("target/proof/executor-summary.json");
                let manifest = find(docs, SourceKind::ExecutorManifest)
                    .map(|doc| doc.path.as_str())
                    .unwrap_or("target/proof/executor-manifest.json");
                commands.insert(format!(
                    "cargo xtask proof --profile affected --base origin/main --head HEAD --plan --executor-summary {summary} --executor-manifest {manifest}"
                ));
            }
            SourceKind::ExecutorObservation => {
                commands.insert(format!(
                    "cargo xtask proof-execution-observation --executor-summary target/proof/executor-summary.json --executor-manifest target/proof/executor-manifest.json --output {}",
                    doc.path
                ));
            }
            SourceKind::ExecutorObservationCollection => {
                commands.insert(format!(
                    "cargo xtask proof-execution-observations-summary --observations-dir target/proof-observations/runs --output {}",
                    doc.path
                ));
            }
            SourceKind::PromotionReadiness => {
                commands.insert(format!(
                    "cargo xtask proof-execution-observations-summary --observations-dir target/proof-observations/runs --promotion-readiness {}",
                    doc.path
                ));
            }
            SourceKind::CoverageReceipt => {
                commands.insert(format!(
                    "cargo xtask coverage-receipt --output {}",
                    doc.path
                ));
            }
        }
    }

    commands.into_iter().collect()
}

fn find(docs: &[SourceDocument], kind: SourceKind) -> Option<&SourceDocument> {
    docs.iter().find(|doc| doc.kind == kind)
}

fn contains(docs: &[SourceDocument], kind: SourceKind) -> bool {
    find(docs, kind).is_some()
}

fn count_commands(value: &Value, required: bool) -> u64 {
    value
        .get("commands")
        .and_then(Value::as_array)
        .map(|commands| {
            commands
                .iter()
                .filter(|command| {
                    command.get("required").and_then(Value::as_bool) == Some(required)
                })
                .count() as u64
        })
        .unwrap_or(0)
}

fn bool_at(value: &Value, path: &[&str]) -> Option<bool> {
    value_at(value, path).and_then(Value::as_bool)
}

fn u64_at(value: &Value, path: &[&str]) -> Option<u64> {
    value_at(value, path).and_then(Value::as_u64)
}

fn string_at(value: &Value, path: &[&str]) -> Option<String> {
    value_at(value, path)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn value_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    Some(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::ProofObservationStatusArgs;
    use serde_json::json;
    use std::path::PathBuf;

    fn args(json_path: PathBuf) -> ProofObservationStatusArgs {
        ProofObservationStatusArgs {
            affected: None,
            proof_policy: None,
            proof_plan: None,
            proof_evidence: None,
            proof_run_summary: None,
            proof_run_observation: None,
            proof_run_observation_collection: None,
            executor_summary: None,
            executor_manifest: None,
            executor_observation: None,
            executor_observation_collection: None,
            promotion_readiness: None,
            coverage_receipt: None,
            json: json_path,
        }
    }

    fn write_json(path: &Path, value: Value) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
    }

    fn test_root(name: &str) -> PathBuf {
        let root = PathBuf::from("target")
            .join("test-proof-observation-status")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn aggregates_required_advisory_and_policy_counts() {
        let root = test_root("aggregate");

        write_json(
            &root.join("proof-policy.json"),
            json!({
                "schema": "tokmd.proof_policy.v1",
                "executor": {
                    "pr": {"required": false, "codecov_upload": false},
                    "promotion": {
                        "min_observations": 1,
                        "min_executed": 4,
                        "min_scopes": 4,
                        "min_artifacts": 4,
                        "min_passing_collector_runs": 1,
                        "required_gate": false,
                        "default_codecov_upload": false
                    }
                },
                "proof_run": {"pr": {"required": false}}
            }),
        );
        write_json(
            &root.join("affected.json"),
            json!({"schema": "tokmd.affected.v1", "unknown_files": []}),
        );
        write_json(
            &root.join("proof-plan.json"),
            json!({
                "schema": "tokmd.proof_plan.v1",
                "base": "origin/main",
                "head": "HEAD",
                "commands": [
                    {"required": true, "scope": "a", "kind": "test", "command": "cargo test"},
                    {"required": false, "scope": "a", "kind": "coverage", "command": "cargo llvm-cov"}
                ]
            }),
        );
        write_json(
            &root.join("proof-run-observation.json"),
            json!({
                "schema": "tokmd.proof_run_observation.v1",
                "base": "origin/main",
                "head": "HEAD",
                "counts": {
                    "required_planned": 1,
                    "executed": 1,
                    "passed": 1,
                    "failed": 0
                }
            }),
        );
        write_json(
            &root.join("proof-executor-observation.json"),
            json!({
                "schema": "tokmd.proof_executor_observation.v1",
                "base": "origin/main",
                "head": "HEAD",
                "counts": {
                    "selected": 1,
                    "executed": 1,
                    "passed": 1,
                    "failed": 0,
                    "artifacts": 1
                }
            }),
        );

        let mut test_args = args(root.join("status.json"));
        test_args.proof_policy = Some(root.join("proof-policy.json"));
        test_args.affected = Some(root.join("affected.json"));
        test_args.proof_plan = Some(root.join("proof-plan.json"));
        test_args.proof_run_observation = Some(root.join("proof-run-observation.json"));
        test_args.executor_observation = Some(root.join("proof-executor-observation.json"));

        let packet = build_packet(&test_args).unwrap();
        assert_eq!(packet.schema, DECISION_SCHEMA);
        assert_eq!(packet.policy_state.promotion_required_gate, Some(false));
        assert_eq!(packet.required_proof.planned, 1);
        assert_eq!(packet.required_proof.executed, 1);
        assert_eq!(packet.required_proof.passed, 1);
        assert_eq!(packet.advisory_proof.planned, 1);
        assert_eq!(packet.advisory_proof.executed, 1);
        assert_eq!(packet.advisory_proof.artifacts, 1);
        assert_eq!(packet.freshness.commit_match, "exact");
        assert!(
            packet
                .criteria_met
                .iter()
                .any(|item| item.id == "required_proof_observed")
        );
        assert!(
            packet
                .criteria_met
                .iter()
                .any(|item| item.id == "advisory_proof_observed")
        );
    }

    #[test]
    fn missing_optional_evidence_is_reported_as_missing_not_passing() {
        let docs = vec![SourceDocument {
            kind: SourceKind::ProofPolicy,
            path: "target/proof/proof-policy.json".to_string(),
            schema: Some("tokmd.proof_policy.v1".to_string()),
            value: json!({
                "schema": "tokmd.proof_policy.v1",
                "executor": {
                    "promotion": {
                        "required_gate": false,
                        "default_codecov_upload": false
                    }
                }
            }),
        }];

        let policy = policy_state(&docs);
        let required = required_proof_summary(&docs);
        let advisory = advisory_proof_summary(&docs);
        let thresholds = threshold_summary(&docs);
        let (met, missing) = decision_criteria(&docs, &policy, &required, &advisory, &thresholds);

        assert!(met.iter().any(|item| item.id == "proof_policy_present"));
        assert!(missing.iter().any(|item| item.id == "affected_present"));
        assert!(
            missing
                .iter()
                .any(|item| item.id == "required_proof_observed")
        );
        assert!(
            missing
                .iter()
                .any(|item| item.id == "advisory_proof_observed")
        );
    }

    #[test]
    fn rejects_absolute_and_escape_source_paths() {
        let absolute = if cfg!(windows) {
            PathBuf::from("C:/tmp/proof.json")
        } else {
            PathBuf::from("/tmp/proof.json")
        };
        let err = repo_relative_path(&absolute).unwrap_err().to_string();
        assert!(
            err.contains("source artifact path must be repo-relative"),
            "{err}"
        );

        let err = repo_relative_path(Path::new("../proof.json"))
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("source artifact path must not escape the repo"),
            "{err}"
        );
    }

    #[test]
    fn rejects_mismatched_artifact_schema() {
        let root = test_root("mismatched-schema");
        write_json(
            &root.join("proof-policy.json"),
            json!({"schema": "tokmd.proof_plan.v1"}),
        );

        let mut test_args = args(root.join("status.json"));
        test_args.proof_policy = Some(root.join("proof-policy.json"));

        let err = build_packet(&test_args).unwrap_err().to_string();
        assert!(
            err.contains("must have schema `tokmd.proof_policy.v1`"),
            "{err}"
        );
    }
}
