use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path};

use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::Value;

use crate::cli::{ProofObservationStatusArgs, ProofObservationStatusCheckArgs};

const DECISION_SCHEMA: &str = "tokmd.proof_observation_decision.v1";
const DECISION_CHECK_SCHEMA: &str = "tokmd.proof_observation_decision_check.v1";
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

pub fn run_check(args: ProofObservationStatusCheckArgs) -> Result<()> {
    let value = read_json_file(&args.decision, "proof observation decision packet")?;
    let report = validate_decision_packet(&value, &args.decision)?;
    if let Some(path) = &args.json {
        write_check_receipt(path, &report)?;
    }

    println!(
        "Proof observation decision OK: {} source artifact(s), {} criteria checked in `{}`",
        report.source_artifacts,
        report.criteria.total(),
        args.decision.display()
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
    fn from_label(label: &str) -> Option<Self> {
        Some(match label {
            "affected" => Self::Affected,
            "proof_policy" => Self::ProofPolicy,
            "proof_plan" => Self::ProofPlan,
            "proof_evidence" => Self::ProofEvidence,
            "proof_run_summary" => Self::ProofRunSummary,
            "proof_run_observation" => Self::ProofRunObservation,
            "proof_run_observation_collection" => Self::ProofRunObservationCollection,
            "executor_summary" => Self::ExecutorSummary,
            "executor_manifest" => Self::ExecutorManifest,
            "executor_observation" => Self::ExecutorObservation,
            "executor_observation_collection" => Self::ExecutorObservationCollection,
            "promotion_readiness" => Self::PromotionReadiness,
            "coverage_receipt" => Self::CoverageReceipt,
            _ => return None,
        })
    }

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

#[derive(Debug, Serialize, PartialEq, Eq)]
struct ProofObservationDecisionCheckReport {
    schema: &'static str,
    ok: bool,
    checked_artifacts: usize,
    decision: VerifiedDecisionArtifact,
    source_artifacts: usize,
    criteria: CriteriaCheckCounts,
    errors: Vec<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct VerifiedDecisionArtifact {
    path: String,
    schema: String,
    mode: String,
}

#[derive(Debug, Default, Serialize, PartialEq, Eq)]
struct CriteriaCheckCounts {
    met: usize,
    missing: usize,
}

impl CriteriaCheckCounts {
    const fn total(&self) -> usize {
        self.met + self.missing
    }
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

fn write_check_receipt(path: &Path, report: &ProofObservationDecisionCheckReport) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let json = serde_json::to_string_pretty(report)
        .context("serialize proof observation decision check receipt")?;
    fs::write(path, format!("{json}\n")).with_context(|| format!("write {}", path.display()))
}

fn read_json_file(path: &Path, label: &str) -> Result<Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {label}"))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {label}"))
}

fn validate_decision_packet(
    value: &Value,
    path: &Path,
) -> Result<ProofObservationDecisionCheckReport> {
    let mut errors = Vec::new();

    let schema = require_string_field(value, "schema", &mut errors).unwrap_or_default();
    if schema != DECISION_SCHEMA {
        errors.push(format!(
            "schema `{schema}` does not match `{DECISION_SCHEMA}`"
        ));
    }

    match value.get("ok").and_then(Value::as_bool) {
        Some(true) => {}
        Some(false) => errors.push("ok is false".to_string()),
        None => errors.push("missing bool field `ok`".to_string()),
    }

    let mode = require_string_field(value, "mode", &mut errors).unwrap_or_default();
    if mode != MODE {
        errors.push(format!("mode `{mode}` does not match `{MODE}`"));
    }

    let source_artifacts = validate_source_artifacts(value, &mut errors);
    validate_policy_guardrails(value, &mut errors);
    validate_count_group(
        value,
        "required_proof",
        &["planned", "executed", "passed", "failed", "observations"],
        &mut errors,
    );
    validate_count_group(
        value,
        "advisory_proof",
        &[
            "planned",
            "selected",
            "executed",
            "passed",
            "failed",
            "skipped",
            "artifacts",
            "observations",
        ],
        &mut errors,
    );
    validate_freshness(value, &mut errors);
    validate_thresholds(value, &mut errors);
    let criteria = validate_criteria(value, &mut errors);
    validate_reproduce(value, &mut errors);
    validate_embedded_errors(value, &mut errors);
    validate_no_environment_leakage(value, &mut errors);

    if !errors.is_empty() {
        bail!(
            "proof observation decision check failed:\n- {}",
            errors.join("\n- ")
        );
    }

    Ok(ProofObservationDecisionCheckReport {
        schema: DECISION_CHECK_SCHEMA,
        ok: true,
        checked_artifacts: 1,
        decision: VerifiedDecisionArtifact {
            path: repo_relative_path(path)?,
            schema,
            mode,
        },
        source_artifacts,
        criteria,
        errors,
    })
}

fn require_string_field(value: &Value, field: &str, errors: &mut Vec<String>) -> Option<String> {
    match value.get(field).and_then(Value::as_str) {
        Some(value) if !value.is_empty() => Some(value.to_owned()),
        Some(_) => {
            errors.push(format!("field `{field}` must not be empty"));
            None
        }
        None => {
            errors.push(format!("missing string field `{field}`"));
            None
        }
    }
}

fn validate_source_artifacts(value: &Value, errors: &mut Vec<String>) -> usize {
    let Some(artifacts) = value.get("source_artifacts").and_then(Value::as_array) else {
        errors.push("missing array field `source_artifacts`".to_string());
        return 0;
    };
    if artifacts.is_empty() {
        errors.push("source_artifacts must not be empty".to_string());
    }

    let mut seen = BTreeSet::new();
    for (index, artifact) in artifacts.iter().enumerate() {
        let Some(kind) = artifact.get("kind").and_then(Value::as_str) else {
            errors.push(format!(
                "source_artifacts[{index}] is missing string field `kind`"
            ));
            continue;
        };
        if !seen.insert(kind.to_string()) {
            errors.push(format!("source_artifacts contains duplicate kind `{kind}`"));
        }

        let Some(kind) = SourceKind::from_label(kind) else {
            errors.push(format!("source_artifacts[{index}] kind is unknown"));
            continue;
        };

        match artifact.get("schema") {
            Some(Value::String(schema)) if schema == kind.expected_schema() => {}
            Some(Value::String(schema)) => errors.push(format!(
                "source_artifacts[{index}] schema `{schema}` does not match `{}`",
                kind.expected_schema()
            )),
            Some(Value::Null) => errors.push(format!(
                "source_artifacts[{index}] schema must be `{}`",
                kind.expected_schema()
            )),
            _ => errors.push(format!(
                "source_artifacts[{index}] is missing string field `schema`"
            )),
        }

        match artifact.get("path").and_then(Value::as_str) {
            Some(path) => validate_repo_relative_string(
                path,
                &format!("source_artifacts[{index}].path"),
                errors,
            ),
            None => errors.push(format!(
                "source_artifacts[{index}] is missing string field `path`"
            )),
        }
    }

    artifacts.len()
}

fn validate_policy_guardrails(value: &Value, errors: &mut Vec<String>) {
    let Some(policy) = value.get("policy_state").and_then(Value::as_object) else {
        errors.push("missing object field `policy_state`".to_string());
        return;
    };

    for field in [
        "executor_pr_required",
        "executor_pr_codecov_upload",
        "promotion_required_gate",
        "promotion_default_codecov_upload",
        "proof_run_pr_required",
    ] {
        match policy.get(field) {
            Some(Value::Bool(false) | Value::Null) | None => {}
            Some(Value::Bool(true)) => errors.push(format!(
                "policy_state.{field} is true; decision packets must remain observation-only"
            )),
            Some(_) => errors.push(format!(
                "policy_state.{field} must be a bool or null when present"
            )),
        }
    }
}

fn validate_count_group(value: &Value, group: &str, fields: &[&str], errors: &mut Vec<String>) {
    let Some(object) = value.get(group).and_then(Value::as_object) else {
        errors.push(format!("missing object field `{group}`"));
        return;
    };

    for field in fields {
        match object.get(*field).and_then(Value::as_u64) {
            Some(_) => {}
            None => errors.push(format!("{group}.{field} must be an unsigned integer")),
        }
    }

    let executed = object.get("executed").and_then(Value::as_u64).unwrap_or(0);
    let passed = object.get("passed").and_then(Value::as_u64).unwrap_or(0);
    let failed = object.get("failed").and_then(Value::as_u64).unwrap_or(0);
    if passed.saturating_add(failed) > executed {
        errors.push(format!(
            "{group}.passed + {group}.failed cannot exceed {group}.executed"
        ));
    }
}

fn validate_freshness(value: &Value, errors: &mut Vec<String>) {
    let Some(freshness) = value.get("freshness").and_then(Value::as_object) else {
        errors.push("missing object field `freshness`".to_string());
        return;
    };

    match freshness.get("commit_match").and_then(Value::as_str) {
        Some("exact" | "partial" | "stale" | "unknown") => {}
        Some(other) => errors.push(format!("freshness.commit_match `{other}` is unknown")),
        None => errors.push("freshness.commit_match must be a string".to_string()),
    }

    let Some(sources) = freshness.get("sources").and_then(Value::as_array) else {
        errors.push("freshness.sources must be an array".to_string());
        return;
    };
    for (index, source) in sources.iter().enumerate() {
        match source.get("kind").and_then(Value::as_str) {
            Some(kind) if SourceKind::from_label(kind).is_some() => {}
            Some(kind) => errors.push(format!(
                "freshness.sources[{index}] kind `{kind}` is unknown"
            )),
            None => errors.push(format!(
                "freshness.sources[{index}] is missing string field `kind`"
            )),
        }
        match source.get("path").and_then(Value::as_str) {
            Some(path) => validate_repo_relative_string(
                path,
                &format!("freshness.sources[{index}].path"),
                errors,
            ),
            None => errors.push(format!(
                "freshness.sources[{index}] is missing string field `path`"
            )),
        }
    }
}

fn validate_thresholds(value: &Value, errors: &mut Vec<String>) {
    let Some(thresholds) = value.get("thresholds").and_then(Value::as_object) else {
        errors.push("missing object field `thresholds`".to_string());
        return;
    };
    for field in [
        "min_observations",
        "min_executed",
        "min_scopes",
        "min_artifacts",
        "min_passing_collector_runs",
        "observations",
        "executed",
        "scopes",
        "artifacts",
        "passing_collector_runs",
    ] {
        match thresholds.get(field) {
            Some(Value::Number(number)) if number.as_u64().is_some() => {}
            Some(Value::Null) | None => {}
            Some(_) => errors.push(format!(
                "thresholds.{field} must be an unsigned integer or null"
            )),
        }
    }
}

fn validate_criteria(value: &Value, errors: &mut Vec<String>) -> CriteriaCheckCounts {
    let mut ids = BTreeSet::new();
    CriteriaCheckCounts {
        met: validate_criteria_array(value, "criteria_met", &mut ids, errors),
        missing: validate_criteria_array(value, "criteria_missing", &mut ids, errors),
    }
}

fn validate_criteria_array(
    value: &Value,
    field: &str,
    ids: &mut BTreeSet<String>,
    errors: &mut Vec<String>,
) -> usize {
    let Some(criteria) = value.get(field).and_then(Value::as_array) else {
        errors.push(format!("missing array field `{field}`"));
        return 0;
    };
    for (index, criterion) in criteria.iter().enumerate() {
        let Some(id) = criterion.get("id").and_then(Value::as_str) else {
            errors.push(format!("{field}[{index}] is missing string field `id`"));
            continue;
        };
        if id.is_empty() {
            errors.push(format!("{field}[{index}].id must not be empty"));
        }
        if !ids.insert(id.to_owned()) {
            errors.push(format!("decision criterion `{id}` appears more than once"));
        }
        match criterion.get("detail").and_then(Value::as_str) {
            Some(detail) if !detail.is_empty() => {}
            Some(_) => errors.push(format!("{field}[{index}].detail must not be empty")),
            None => errors.push(format!("{field}[{index}] is missing string field `detail`")),
        }
    }
    criteria.len()
}

fn validate_reproduce(value: &Value, errors: &mut Vec<String>) {
    let Some(commands) = value.get("reproduce").and_then(Value::as_array) else {
        errors.push("missing array field `reproduce`".to_string());
        return;
    };
    if commands.is_empty() {
        errors.push("reproduce must include at least one command".to_string());
    }
    let mut seen = BTreeSet::new();
    for (index, command) in commands.iter().enumerate() {
        let Some(command) = command.as_str() else {
            errors.push(format!("reproduce[{index}] must be a string"));
            continue;
        };
        if command.is_empty() {
            errors.push(format!("reproduce[{index}] must not be empty"));
        }
        if !command.starts_with("cargo xtask ") {
            errors.push(format!("reproduce[{index}] must be a cargo xtask command"));
        }
        if !seen.insert(command.to_owned()) {
            errors.push(format!("duplicate reproduce command `{command}`"));
        }
        validate_command_has_no_absolute_path(command, &format!("reproduce[{index}]"), errors);
    }
}

fn validate_embedded_errors(value: &Value, errors: &mut Vec<String>) {
    let Some(packet_errors) = value.get("errors").and_then(Value::as_array) else {
        errors.push("missing array field `errors`".to_string());
        return;
    };
    if !packet_errors.is_empty() {
        errors.push("packet errors must be empty for a valid decision packet".to_string());
    }
}

fn validate_no_environment_leakage(value: &Value, errors: &mut Vec<String>) {
    fn walk(value: &Value, path: &str, errors: &mut Vec<String>) {
        match value {
            Value::String(text) => {
                if text.contains("\\Users\\") || text.contains("\\AppData\\") {
                    errors.push(format!("{path} contains local environment path text"));
                }
            }
            Value::Array(items) => {
                for (index, item) in items.iter().enumerate() {
                    walk(item, &format!("{path}[{index}]"), errors);
                }
            }
            Value::Object(object) => {
                for (key, item) in object {
                    walk(item, &format!("{path}.{key}"), errors);
                }
            }
            Value::Null | Value::Bool(_) | Value::Number(_) => {}
        }
    }

    walk(value, "$", errors);
}

fn validate_repo_relative_string(path: &str, label: &str, errors: &mut Vec<String>) {
    match repo_relative_path(Path::new(path)) {
        Ok(normalized) if normalized == path => {}
        Ok(normalized) => errors.push(format!(
            "{label} must use normalized repo-relative slashes: `{path}` should be `{normalized}`"
        )),
        Err(error) => errors.push(format!("{label} is not repo-relative: {error}")),
    }
}

fn validate_command_has_no_absolute_path(command: &str, label: &str, errors: &mut Vec<String>) {
    for token in command.split_whitespace() {
        let trimmed = token.trim_matches('`').trim_matches('"').trim_matches('\'');
        if Path::new(trimmed).is_absolute() || trimmed.contains(":\\") {
            errors.push(format!("{label} contains absolute path `{trimmed}`"));
        }
    }
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
    use crate::cli::{ProofObservationStatusArgs, ProofObservationStatusCheckArgs};
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

    #[test]
    fn check_validates_decision_packet_and_writes_receipt_deterministically() {
        let root = test_root("check-valid");
        write_json(
            &root.join("proof-policy.json"),
            json!({
                "schema": "tokmd.proof_policy.v1",
                "executor": {
                    "pr": {"required": false, "codecov_upload": false},
                    "promotion": {
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

        let decision = root.join("proof-observation-decision.json");
        let mut test_args = args(decision.clone());
        test_args.proof_policy = Some(root.join("proof-policy.json"));
        test_args.affected = Some(root.join("affected.json"));
        let packet = build_packet(&test_args).unwrap();
        write_packet(&decision, &packet).unwrap();

        let first_receipt = root.join("check-1.json");
        run_check(ProofObservationStatusCheckArgs {
            decision: decision.clone(),
            json: Some(first_receipt.clone()),
        })
        .unwrap();
        let first = fs::read_to_string(&first_receipt).unwrap();
        let receipt: Value = serde_json::from_str(&first).unwrap();
        assert_eq!(
            receipt.get("schema").and_then(Value::as_str),
            Some(DECISION_CHECK_SCHEMA)
        );
        assert_eq!(
            receipt
                .get("source_artifacts")
                .and_then(Value::as_u64)
                .unwrap(),
            2
        );

        let second_receipt = root.join("check-2.json");
        run_check(ProofObservationStatusCheckArgs {
            decision,
            json: Some(second_receipt.clone()),
        })
        .unwrap();
        let second = fs::read_to_string(second_receipt).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn check_rejects_promoted_policy_state() {
        let mut packet = json!({
            "schema": DECISION_SCHEMA,
            "ok": true,
            "mode": MODE,
            "source_artifacts": [
                {
                    "kind": "proof_policy",
                    "path": "target/proof/proof-policy.json",
                    "schema": "tokmd.proof_policy.v1"
                }
            ],
            "policy_state": {
                "proof_policy_present": true,
                "executor_pr_required": false,
                "executor_pr_codecov_upload": false,
                "promotion_required_gate": false,
                "promotion_default_codecov_upload": false,
                "proof_run_pr_required": false
            },
            "required_proof": {
                "planned": 0,
                "executed": 0,
                "passed": 0,
                "failed": 0,
                "observations": 0
            },
            "advisory_proof": {
                "planned": 0,
                "selected": 0,
                "executed": 0,
                "passed": 0,
                "failed": 0,
                "skipped": 0,
                "artifacts": 0,
                "observations": 0
            },
            "freshness": {
                "commit_match": "unknown",
                "base": null,
                "head": null,
                "sources": []
            },
            "thresholds": {
                "min_observations": null,
                "min_executed": null,
                "min_scopes": null,
                "min_artifacts": null,
                "min_passing_collector_runs": null,
                "observations": null,
                "executed": null,
                "scopes": null,
                "artifacts": null,
                "passing_collector_runs": null
            },
            "criteria_met": [],
            "criteria_missing": [
                {"id": "promotion_readiness_missing", "detail": "missing"}
            ],
            "reproduce": [
                "cargo xtask proof-policy --json-output target/proof/proof-policy.json"
            ],
            "errors": []
        });
        packet["policy_state"]["promotion_required_gate"] = json!(true);

        let err = validate_decision_packet(
            &packet,
            Path::new("target/proof-observations/proof-observation-decision.json"),
        )
        .unwrap_err()
        .to_string();
        assert!(
            err.contains("policy_state.promotion_required_gate is true"),
            "{err}"
        );
    }

    #[test]
    fn check_rejects_absolute_source_artifact_path() {
        let packet = json!({
            "schema": DECISION_SCHEMA,
            "ok": true,
            "mode": MODE,
            "source_artifacts": [
                {
                    "kind": "proof_policy",
                    "path": if cfg!(windows) { "C:/tmp/proof-policy.json" } else { "/tmp/proof-policy.json" },
                    "schema": "tokmd.proof_policy.v1"
                }
            ],
            "policy_state": {
                "proof_policy_present": true,
                "executor_pr_required": false,
                "executor_pr_codecov_upload": false,
                "promotion_required_gate": false,
                "promotion_default_codecov_upload": false,
                "proof_run_pr_required": false
            },
            "required_proof": {
                "planned": 0,
                "executed": 0,
                "passed": 0,
                "failed": 0,
                "observations": 0
            },
            "advisory_proof": {
                "planned": 0,
                "selected": 0,
                "executed": 0,
                "passed": 0,
                "failed": 0,
                "skipped": 0,
                "artifacts": 0,
                "observations": 0
            },
            "freshness": {
                "commit_match": "unknown",
                "base": null,
                "head": null,
                "sources": []
            },
            "thresholds": {
                "min_observations": null,
                "min_executed": null,
                "min_scopes": null,
                "min_artifacts": null,
                "min_passing_collector_runs": null,
                "observations": null,
                "executed": null,
                "scopes": null,
                "artifacts": null,
                "passing_collector_runs": null
            },
            "criteria_met": [],
            "criteria_missing": [
                {"id": "promotion_readiness_missing", "detail": "missing"}
            ],
            "reproduce": [
                "cargo xtask proof-policy --json-output target/proof/proof-policy.json"
            ],
            "errors": []
        });

        let err = validate_decision_packet(
            &packet,
            Path::new("target/proof-observations/proof-observation-decision.json"),
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("source_artifacts[0].path"), "{err}");
    }
}
