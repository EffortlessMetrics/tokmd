use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path};

use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::Value;

use crate::cli::{ProofWorkflowKind, ProofWorkflowStatusArgs, ProofWorkflowStatusCheckArgs};

const STATUS_SCHEMA: &str = "tokmd.proof_workflow_status.v1";
const STATUS_CHECK_SCHEMA: &str = "tokmd.proof_workflow_status_check.v1";
const MODE: &str = "workflow_status_only";
const FAST_PROOF_RUN_LABEL: &str = "fast_proof_run";
const FAST_PROOF_RUN_TITLE: &str = "Fast Proof Run";
const FAST_PROOF_RUN_ADVISORY_NOTE: &str = "Fast proof-run artifact generation is advisory and is not part of the required CI aggregate yet.";
const SCOPED_COVERAGE_EXECUTOR_LABEL: &str = "scoped_coverage_executor";
const SCOPED_COVERAGE_EXECUTOR_TITLE: &str = "Scoped Coverage Executor";
const SCOPED_COVERAGE_EXECUTOR_ADVISORY_NOTE: &str = "Scoped coverage executor is an explicitly non-required PR/manual experiment. It runs only planner-selected non-required coverage commands and does not replace required PR proof jobs.";
const FAST_PROOF_REQUIRED_STATUSES: [&str; 3] = [
    "proof_run_status",
    "proof_run_artifacts_status",
    "proof_run_observation_status",
];
const SCOPED_COVERAGE_REQUIRED_STATUSES: [&str; 5] = [
    "affected_status",
    "executor_status",
    "verifier_status",
    "observation_status",
    "collection_status",
];

pub fn run(args: ProofWorkflowStatusArgs) -> Result<()> {
    let packet = build_packet(&args)?;
    write_json(&args.json, &packet)?;

    if let Some(path) = &args.summary_md {
        write_text(path, &render_markdown(&packet))?;
    }

    if let Some(path) = &args.env_output {
        write_text(path, &render_env_output(&packet))?;
    }

    println!(
        "proof workflow status: wrote {} status(es), {} source artifact(s), recommended_exit_code={} to {}",
        packet.command_statuses.len(),
        packet.source_artifacts.len(),
        packet.recommended_exit_code,
        args.json.display()
    );

    Ok(())
}

pub fn run_check(args: ProofWorkflowStatusCheckArgs) -> Result<()> {
    let value = read_json_file(&args.status, "proof workflow status packet")?;
    let report = validate_status_packet(&value, &args.status)?;

    if let Some(path) = &args.json {
        write_json(path, &report)?;
    }

    println!(
        "Proof workflow status OK: {} source artifact(s), {} command status(es), recommended_exit_code={} in `{}`",
        report.source_artifacts,
        report.command_statuses,
        report.recommended_exit_code,
        args.status.display()
    );

    Ok(())
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct ProofWorkflowStatusPacket {
    schema: &'static str,
    ok: bool,
    mode: &'static str,
    workflow_kind: &'static str,
    required: bool,
    advisory: bool,
    policy_guardrails: PolicyGuardrails,
    source_artifacts: Vec<SourceArtifact>,
    command_statuses: Vec<CommandStatus>,
    recommended_exit_code: i32,
    summary: WorkflowSummary,
    errors: Vec<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct PolicyGuardrails {
    required_gate: bool,
    codecov_default_upload: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct SourceArtifact {
    role: &'static str,
    path: String,
    schema: &'static str,
    required: bool,
    available: bool,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct CommandStatus {
    name: String,
    exit_code: i32,
    blocking: bool,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct WorkflowSummary {
    title: &'static str,
    advisory_note: &'static str,
    commands: Vec<SummaryCommand>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct SummaryCommand {
    label: &'static str,
    status_name: String,
    exit_code: i32,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct ProofWorkflowStatusCheckReport {
    schema: &'static str,
    ok: bool,
    checked_artifacts: usize,
    status: VerifiedStatusArtifact,
    source_artifacts: usize,
    command_statuses: usize,
    recommended_exit_code: i32,
    errors: Vec<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct VerifiedStatusArtifact {
    path: String,
    schema: String,
    mode: String,
    workflow_kind: String,
}

#[derive(Debug, Clone, Copy)]
struct SourceRole {
    role: &'static str,
    expected_schema: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct WorkflowContract {
    label: &'static str,
    title: &'static str,
    advisory_note: &'static str,
    statuses: &'static [&'static str],
    sources: &'static [SourceRole],
}

const FAST_PROOF_SOURCE_ROLES: [SourceRole; 5] = [
    SourceRole {
        role: "proof_policy",
        expected_schema: "tokmd.proof_policy.v1",
    },
    SourceRole {
        role: "proof_plan",
        expected_schema: "tokmd.proof_plan.v1",
    },
    SourceRole {
        role: "proof_run_summary",
        expected_schema: "tokmd.proof_run_summary.v1",
    },
    SourceRole {
        role: "proof_run_artifacts_check",
        expected_schema: "tokmd.proof_run_artifacts_check.v1",
    },
    SourceRole {
        role: "proof_run_observation",
        expected_schema: "tokmd.proof_run_observation.v1",
    },
];

const SCOPED_COVERAGE_SOURCE_ROLES: [SourceRole; 8] = [
    SourceRole {
        role: "proof_policy",
        expected_schema: "tokmd.proof_policy.v1",
    },
    SourceRole {
        role: "affected",
        expected_schema: "tokmd.affected.v1",
    },
    SourceRole {
        role: "proof_plan",
        expected_schema: "tokmd.proof_plan.v1",
    },
    SourceRole {
        role: "executor_summary",
        expected_schema: "tokmd.proof_executor_summary.v1",
    },
    SourceRole {
        role: "executor_manifest",
        expected_schema: "tokmd.proof_executor_manifest.v1",
    },
    SourceRole {
        role: "proof_execution_artifacts_check",
        expected_schema: "tokmd.proof_execution_artifacts_check.v1",
    },
    SourceRole {
        role: "proof_executor_observation",
        expected_schema: "tokmd.proof_executor_observation.v1",
    },
    SourceRole {
        role: "proof_executor_observation_collection",
        expected_schema: "tokmd.proof_executor_observation_collection.v1",
    },
];

const FAST_PROOF_CONTRACT: WorkflowContract = WorkflowContract {
    label: FAST_PROOF_RUN_LABEL,
    title: FAST_PROOF_RUN_TITLE,
    advisory_note: FAST_PROOF_RUN_ADVISORY_NOTE,
    statuses: &FAST_PROOF_REQUIRED_STATUSES,
    sources: &FAST_PROOF_SOURCE_ROLES,
};

const SCOPED_COVERAGE_CONTRACT: WorkflowContract = WorkflowContract {
    label: SCOPED_COVERAGE_EXECUTOR_LABEL,
    title: SCOPED_COVERAGE_EXECUTOR_TITLE,
    advisory_note: SCOPED_COVERAGE_EXECUTOR_ADVISORY_NOTE,
    statuses: &SCOPED_COVERAGE_REQUIRED_STATUSES,
    sources: &SCOPED_COVERAGE_SOURCE_ROLES,
};

fn build_packet(args: &ProofWorkflowStatusArgs) -> Result<ProofWorkflowStatusPacket> {
    let contract = workflow_contract(args.workflow_kind);
    let source_artifacts = load_sources(args, contract.sources)?;
    let policy = read_json_file(&args.proof_policy, "proof policy")?;
    let guardrails = policy_guardrails(args.workflow_kind, &policy)?;
    if guardrails.required_gate {
        bail!(
            "{} policy must remain advisory; required gate must be false",
            contract.label
        );
    }
    if guardrails.codecov_default_upload {
        bail!("Codecov default upload must remain disabled for proof workflow status packets");
    }

    let statuses = parse_command_statuses(&args.statuses, contract.statuses, contract.label)?;
    let recommended_exit_code = recommended_exit_code(&statuses, contract.statuses);
    let summary = WorkflowSummary {
        title: contract.title,
        advisory_note: contract.advisory_note,
        commands: summary_commands(&statuses, contract.statuses),
    };

    Ok(ProofWorkflowStatusPacket {
        schema: STATUS_SCHEMA,
        ok: true,
        mode: MODE,
        workflow_kind: contract.label,
        required: false,
        advisory: true,
        policy_guardrails: guardrails,
        source_artifacts,
        command_statuses: command_statuses(&statuses, contract.statuses),
        recommended_exit_code,
        summary,
        errors: Vec::new(),
    })
}

fn workflow_contract(kind: ProofWorkflowKind) -> &'static WorkflowContract {
    match kind {
        ProofWorkflowKind::FastProofRun => &FAST_PROOF_CONTRACT,
        ProofWorkflowKind::ScopedCoverageExecutor => &SCOPED_COVERAGE_CONTRACT,
    }
}

fn workflow_contract_by_label(label: &str) -> Option<&'static WorkflowContract> {
    match label {
        FAST_PROOF_RUN_LABEL => Some(&FAST_PROOF_CONTRACT),
        SCOPED_COVERAGE_EXECUTOR_LABEL => Some(&SCOPED_COVERAGE_CONTRACT),
        _ => None,
    }
}

fn source_path<'a>(args: &'a ProofWorkflowStatusArgs, role: &str) -> &'a Path {
    match role {
        "proof_policy" => &args.proof_policy,
        "proof_plan" => &args.proof_plan,
        "proof_run_summary" => &args.proof_run_summary,
        "proof_run_artifacts_check" => &args.proof_run_artifacts_check,
        "proof_run_observation" => &args.proof_run_observation,
        "affected" => &args.affected,
        "executor_summary" => &args.executor_summary,
        "executor_manifest" => &args.executor_manifest,
        "proof_execution_artifacts_check" => &args.proof_execution_artifacts_check,
        "proof_executor_observation" => &args.proof_executor_observation,
        "proof_executor_observation_collection" => &args.proof_executor_observation_collection,
        _ => unreachable!("unknown proof workflow source role: {role}"),
    }
}

fn load_sources(
    args: &ProofWorkflowStatusArgs,
    roles: &'static [SourceRole],
) -> Result<Vec<SourceArtifact>> {
    roles
        .iter()
        .map(|role| load_source_artifact(*role, source_path(args, role.role)))
        .collect()
}

fn load_source_artifact(role: SourceRole, path: &Path) -> Result<SourceArtifact> {
    let display_path = repo_relative_path(path)?;
    let value = read_json_file(path, role.role)?;
    let schema = value
        .get("schema")
        .and_then(Value::as_str)
        .unwrap_or("<missing>");
    if schema != role.expected_schema {
        bail!(
            "{} artifact `{display_path}` must have schema `{}`, got `{schema}`",
            role.role,
            role.expected_schema
        );
    }

    Ok(SourceArtifact {
        role: role.role,
        path: display_path,
        schema: role.expected_schema,
        required: true,
        available: true,
    })
}

fn policy_guardrails(kind: ProofWorkflowKind, policy: &Value) -> Result<PolicyGuardrails> {
    let required_gate = match kind {
        ProofWorkflowKind::FastProofRun => bool_at(policy, &["proof_run", "pr", "required"])
            .context("proof policy JSON is missing proof_run.pr.required")?,
        ProofWorkflowKind::ScopedCoverageExecutor => {
            bool_at(policy, &["executor", "pr", "required"])
                .context("proof policy JSON is missing executor.pr.required")?
        }
    };
    let codecov_default_upload =
        bool_at(policy, &["executor", "promotion", "default_codecov_upload"])
            .or_else(|| bool_at(policy, &["executor", "pr", "codecov_upload"]))
            .unwrap_or(false);

    Ok(PolicyGuardrails {
        required_gate,
        codecov_default_upload,
    })
}

fn bool_at(value: &Value, path: &[&str]) -> Option<bool> {
    let mut cursor = value;
    for segment in path {
        cursor = cursor.get(*segment)?;
    }
    cursor.as_bool()
}

fn parse_command_statuses(
    raw: &[String],
    required_order: &[&'static str],
    workflow_label: &str,
) -> Result<BTreeMap<String, i32>> {
    let allowed: BTreeSet<&str> = required_order.iter().copied().collect();
    let mut statuses = BTreeMap::new();

    for item in raw {
        let Some((name, value)) = item.split_once('=') else {
            bail!("status `{item}` must use NAME=INTEGER form");
        };
        if name.is_empty() {
            bail!("status name must not be empty");
        }
        if !allowed.contains(name) {
            bail!("unsupported status `{name}` for {workflow_label}");
        }
        let exit_code = value
            .parse::<i32>()
            .with_context(|| format!("status `{name}` must be an integer exit code"))?;
        if exit_code < 0 {
            bail!("status `{name}` must not be negative");
        }
        if statuses.insert(name.to_owned(), exit_code).is_some() {
            bail!("duplicate status `{name}`");
        }
    }

    for required in required_order {
        if !statuses.contains_key(*required) {
            bail!("missing required status `{required}`");
        }
    }

    Ok(statuses)
}

fn recommended_exit_code(statuses: &BTreeMap<String, i32>, priority: &[&str]) -> i32 {
    priority
        .iter()
        .filter_map(|name| statuses.get(*name))
        .copied()
        .find(|code| *code != 0)
        .unwrap_or(0)
}

fn command_statuses(statuses: &BTreeMap<String, i32>, priority: &[&str]) -> Vec<CommandStatus> {
    priority
        .iter()
        .map(|name| CommandStatus {
            name: (*name).to_owned(),
            exit_code: statuses.get(*name).copied().unwrap_or_default(),
            blocking: true,
        })
        .collect()
}

fn summary_commands(statuses: &BTreeMap<String, i32>, priority: &[&str]) -> Vec<SummaryCommand> {
    priority
        .iter()
        .map(|name| SummaryCommand {
            label: status_label(name),
            status_name: (*name).to_owned(),
            exit_code: statuses.get(*name).copied().unwrap_or_default(),
        })
        .collect()
}

fn status_label(name: &str) -> &'static str {
    match name {
        "proof_run_status" => "proof run",
        "proof_run_artifacts_status" => "proof run artifacts",
        "proof_run_observation_status" => "proof run observation",
        "affected_status" => "affected",
        "executor_status" => "proof executor",
        "verifier_status" => "execution artifact verifier",
        "observation_status" => "execution observation",
        "collection_status" => "observation collection",
        _ => "unknown",
    }
}

fn render_markdown(packet: &ProofWorkflowStatusPacket) -> String {
    let mut markdown = String::new();
    markdown.push_str("## ");
    markdown.push_str(packet.summary.title);
    markdown.push_str("\n\n");
    markdown.push_str("| Command | Exit |\n");
    markdown.push_str("| --- | ---: |\n");
    for command in &packet.summary.commands {
        markdown.push_str("| ");
        markdown.push_str(command.label);
        markdown.push_str(" | ");
        markdown.push_str(&command.exit_code.to_string());
        markdown.push_str(" |\n");
    }
    markdown.push('\n');
    markdown.push_str(packet.summary.advisory_note);
    markdown.push_str("\n\n");
    markdown.push_str(&format!(
        "Recommended workflow exit code: {}\n\n",
        packet.recommended_exit_code
    ));
    markdown.push_str("Source artifacts:\n");
    for artifact in &packet.source_artifacts {
        markdown.push_str("- ");
        markdown.push_str(artifact.role);
        markdown.push_str(": ");
        markdown.push_str(&artifact.path);
        markdown.push('\n');
    }
    markdown
}

fn render_env_output(packet: &ProofWorkflowStatusPacket) -> String {
    format!(
        "ok={}\nrecommended_exit_code={}\nworkflow_kind={}\n",
        packet.ok, packet.recommended_exit_code, packet.workflow_kind
    )
}

fn validate_status_packet(value: &Value, path: &Path) -> Result<ProofWorkflowStatusCheckReport> {
    let mut errors = Vec::new();

    let schema = require_string_field(value, "schema", &mut errors).unwrap_or_default();
    if schema != STATUS_SCHEMA {
        errors.push(format!(
            "schema `{schema}` does not match `{STATUS_SCHEMA}`"
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

    let workflow_kind =
        require_string_field(value, "workflow_kind", &mut errors).unwrap_or_default();
    let contract = workflow_contract_by_label(&workflow_kind).unwrap_or_else(|| {
        errors.push(format!("workflow_kind `{workflow_kind}` is not supported"));
        &FAST_PROOF_CONTRACT
    });

    require_bool_value(value, "required", false, &mut errors);
    require_bool_value(value, "advisory", true, &mut errors);
    validate_policy_guardrails(value, &mut errors);
    let source_artifacts = validate_source_artifacts(value, contract.sources, &mut errors);
    let (command_statuses, parsed_statuses) =
        validate_command_statuses(value, contract.statuses, &mut errors);
    let recommended_exit_code = validate_recommended_exit_code(value, &mut errors);
    validate_summary(value, contract, &parsed_statuses, &mut errors);
    validate_embedded_errors(value, &mut errors);

    if !errors.is_empty() {
        bail!(
            "proof workflow status check failed:\n- {}",
            errors.join("\n- ")
        );
    }

    Ok(ProofWorkflowStatusCheckReport {
        schema: STATUS_CHECK_SCHEMA,
        ok: true,
        checked_artifacts: 1,
        status: VerifiedStatusArtifact {
            path: repo_relative_path(path)?,
            schema,
            mode,
            workflow_kind,
        },
        source_artifacts,
        command_statuses,
        recommended_exit_code,
        errors,
    })
}

fn validate_policy_guardrails(value: &Value, errors: &mut Vec<String>) {
    let Some(guardrails) = value.get("policy_guardrails").and_then(Value::as_object) else {
        errors.push("missing object field `policy_guardrails`".to_string());
        return;
    };

    match guardrails.get("required_gate").and_then(Value::as_bool) {
        Some(false) => {}
        Some(true) => errors.push("policy_guardrails.required_gate must remain false".to_string()),
        None => errors.push("missing bool field `policy_guardrails.required_gate`".to_string()),
    }
    match guardrails
        .get("codecov_default_upload")
        .and_then(Value::as_bool)
    {
        Some(false) => {}
        Some(true) => {
            errors.push("policy_guardrails.codecov_default_upload must remain false".to_string());
        }
        None => {
            errors
                .push("missing bool field `policy_guardrails.codecov_default_upload`".to_string());
        }
    }
}

fn validate_source_artifacts(
    value: &Value,
    expected_roles: &'static [SourceRole],
    errors: &mut Vec<String>,
) -> usize {
    let Some(artifacts) = value.get("source_artifacts").and_then(Value::as_array) else {
        errors.push("missing array field `source_artifacts`".to_string());
        return 0;
    };

    let expected: BTreeMap<&str, &str> = expected_roles
        .iter()
        .map(|role| (role.role, role.expected_schema))
        .collect();
    let mut seen = BTreeSet::new();

    for (index, artifact) in artifacts.iter().enumerate() {
        let Some(object) = artifact.as_object() else {
            errors.push(format!("source_artifacts[{index}] must be an object"));
            continue;
        };
        let role = object.get("role").and_then(Value::as_str).unwrap_or("");
        let path = object.get("path").and_then(Value::as_str).unwrap_or("");
        let schema = object.get("schema").and_then(Value::as_str).unwrap_or("");
        if role.is_empty() {
            errors.push(format!("source_artifacts[{index}].role must not be empty"));
        } else if !expected.contains_key(role) {
            errors.push(format!("unsupported source artifact role `{role}`"));
        } else if !seen.insert(role.to_owned()) {
            errors.push(format!("duplicate source artifact role `{role}`"));
        }
        if let Err(error) = validate_relative_path_str(path) {
            errors.push(format!("source artifact `{role}` path invalid: {error}"));
        }
        if let Some(expected_schema) = expected.get(role)
            && schema != *expected_schema
        {
            errors.push(format!(
                "source artifact `{role}` schema `{schema}` does not match `{expected_schema}`"
            ));
        }
        match object.get("required").and_then(Value::as_bool) {
            Some(true) => {}
            Some(false) => errors.push(format!("source artifact `{role}` must be required")),
            None => errors.push(format!("source artifact `{role}` missing required flag")),
        }
        match object.get("available").and_then(Value::as_bool) {
            Some(true) => {}
            Some(false) => errors.push(format!("source artifact `{role}` must be available")),
            None => errors.push(format!("source artifact `{role}` missing available flag")),
        }
    }

    for role in expected.keys() {
        if !seen.contains(*role) {
            errors.push(format!("missing source artifact role `{role}`"));
        }
    }

    artifacts.len()
}

fn validate_command_statuses(
    value: &Value,
    expected_statuses: &[&'static str],
    errors: &mut Vec<String>,
) -> (usize, BTreeMap<String, i32>) {
    let Some(statuses) = value.get("command_statuses").and_then(Value::as_array) else {
        errors.push("missing array field `command_statuses`".to_string());
        return (0, BTreeMap::new());
    };

    let mut parsed = BTreeMap::new();
    for (index, status) in statuses.iter().enumerate() {
        let Some(object) = status.as_object() else {
            errors.push(format!("command_statuses[{index}] must be an object"));
            continue;
        };
        let name = object.get("name").and_then(Value::as_str).unwrap_or("");
        let exit_code = object.get("exit_code").and_then(Value::as_i64);
        match exit_code {
            Some(code) if code >= 0 && i32::try_from(code).is_ok() => {
                if parsed.insert(name.to_owned(), code as i32).is_some() {
                    errors.push(format!("duplicate command status `{name}`"));
                }
            }
            Some(_) => errors.push(format!(
                "command status `{name}` exit_code must be a non-negative i32"
            )),
            None => errors.push(format!("command status `{name}` missing integer exit_code")),
        }
        match object.get("blocking").and_then(Value::as_bool) {
            Some(true) => {}
            Some(false) => errors.push(format!("command status `{name}` must be blocking")),
            None => errors.push(format!("command status `{name}` missing blocking flag")),
        }
    }

    for required in expected_statuses {
        if !parsed.contains_key(*required) {
            errors.push(format!("missing command status `{required}`"));
        }
    }

    let expected_exit = recommended_exit_code(&parsed, expected_statuses);
    match value.get("recommended_exit_code").and_then(Value::as_i64) {
        Some(actual) if actual == i64::from(expected_exit) => {}
        Some(actual) => errors.push(format!(
            "recommended_exit_code `{actual}` does not match priority result `{expected_exit}`"
        )),
        None => errors.push("missing integer field `recommended_exit_code`".to_string()),
    }

    (statuses.len(), parsed)
}

fn validate_recommended_exit_code(value: &Value, errors: &mut Vec<String>) -> i32 {
    match value.get("recommended_exit_code").and_then(Value::as_i64) {
        Some(code) if code >= 0 && i32::try_from(code).is_ok() => code as i32,
        Some(_) => {
            errors.push("recommended_exit_code must be a non-negative i32".to_string());
            0
        }
        None => {
            errors.push("missing integer field `recommended_exit_code`".to_string());
            0
        }
    }
}

fn validate_summary(
    value: &Value,
    contract: &WorkflowContract,
    statuses: &BTreeMap<String, i32>,
    errors: &mut Vec<String>,
) {
    let Some(summary) = value.get("summary").and_then(Value::as_object) else {
        errors.push("missing object field `summary`".to_string());
        return;
    };
    match summary.get("title").and_then(Value::as_str) {
        Some(title) if title == contract.title => {}
        Some(title) => errors.push(format!(
            "summary.title `{title}` does not match {}",
            contract.label
        )),
        None => errors.push("missing string field `summary.title`".to_string()),
    }
    match summary.get("advisory_note").and_then(Value::as_str) {
        Some(note) if note == contract.advisory_note => {}
        Some(_) => {
            errors.push("summary.advisory_note does not match advisory boundary".to_string())
        }
        None => errors.push("missing string field `summary.advisory_note`".to_string()),
    }
    let Some(commands) = summary.get("commands").and_then(Value::as_array) else {
        errors.push("missing array field `summary.commands`".to_string());
        return;
    };
    if commands.len() != contract.statuses.len() {
        errors.push("summary.commands length does not match command statuses".to_string());
    }
    for (index, expected) in contract.statuses.iter().enumerate() {
        let Some(command) = commands.get(index).and_then(Value::as_object) else {
            continue;
        };
        match command.get("status_name").and_then(Value::as_str) {
            Some(actual) if actual == *expected => {}
            Some(actual) => errors.push(format!(
                "summary.commands[{index}].status_name `{actual}` does not match `{expected}`"
            )),
            None => errors.push(format!(
                "summary.commands[{index}].status_name must be present"
            )),
        }
        match command.get("exit_code").and_then(Value::as_i64) {
            Some(actual) if Some(actual) == statuses.get(*expected).map(|code| i64::from(*code)) => {}
            Some(actual) => errors.push(format!(
                "summary.commands[{index}].exit_code `{actual}` does not match command status `{expected}`"
            )),
            None => errors.push(format!(
                "summary.commands[{index}].exit_code must be present"
            )),
        }
    }
}

fn validate_embedded_errors(value: &Value, errors: &mut Vec<String>) {
    match value.get("errors").and_then(Value::as_array) {
        Some(values) if values.is_empty() => {}
        Some(_) => errors.push("packet marked ok must not embed errors".to_string()),
        None => errors.push("missing array field `errors`".to_string()),
    }
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

fn require_bool_value(value: &Value, field: &str, expected: bool, errors: &mut Vec<String>) {
    match value.get(field).and_then(Value::as_bool) {
        Some(actual) if actual == expected => {}
        Some(actual) => errors.push(format!("field `{field}` must be {expected}, got {actual}")),
        None => errors.push(format!("missing bool field `{field}`")),
    }
}

fn repo_relative_path(path: &Path) -> Result<String> {
    if path.is_absolute() {
        bail!(
            "source artifact path must be repo-relative: {}",
            path.display()
        );
    }
    validate_relative_path_components(path.components(), &path.display().to_string())
}

fn validate_relative_path_str(path: &str) -> Result<()> {
    if path.is_empty() {
        bail!("source artifact path must name a file");
    }
    if path.contains('\\') {
        bail!("source artifact path must use forward slashes");
    }
    let path = Path::new(path);
    if path.is_absolute() {
        bail!("source artifact path must be repo-relative");
    }
    validate_relative_path_components(path.components(), &path.display().to_string()).map(|_| ())
}

fn validate_relative_path_components<'a>(
    components: impl Iterator<Item = Component<'a>>,
    display_path: &str,
) -> Result<String> {
    let mut normalized = Vec::new();
    for component in components {
        match component {
            Component::Normal(part) => normalized.push(part.to_string_lossy().into_owned()),
            Component::CurDir => {}
            Component::ParentDir => {
                bail!("source artifact path must not escape the repo: {display_path}");
            }
            Component::Prefix(_) | Component::RootDir => {
                bail!("source artifact path must be repo-relative: {display_path}");
            }
        }
    }

    if normalized.is_empty() {
        bail!("source artifact path must name a file");
    }

    Ok(normalized.join("/"))
}

fn read_json_file(path: &Path, label: &str) -> Result<Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {label}"))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {label}"))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(value).context("serialize proof workflow status")?;
    fs::write(path, format!("{json}\n")).with_context(|| format!("write {}", path.display()))
}

fn write_text(path: &Path, text: &str) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, text).with_context(|| format!("write {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_statuses_and_preserves_priority() {
        let statuses = parse_command_statuses(
            &[
                "proof_run_status=0".to_string(),
                "proof_run_artifacts_status=7".to_string(),
                "proof_run_observation_status=3".to_string(),
            ],
            &FAST_PROOF_REQUIRED_STATUSES,
            FAST_PROOF_RUN_LABEL,
        )
        .unwrap();

        assert_eq!(
            recommended_exit_code(&statuses, &FAST_PROOF_REQUIRED_STATUSES),
            7
        );
    }

    #[test]
    fn rejects_absolute_source_paths() {
        let path = if cfg!(windows) {
            Path::new("C:/tmp/proof-policy.json")
        } else {
            Path::new("/tmp/proof-policy.json")
        };

        let error = repo_relative_path(path).unwrap_err().to_string();

        assert!(error.contains("repo-relative"), "{error}");
    }

    #[test]
    fn scoped_statuses_preserve_executor_priority() {
        let statuses = parse_command_statuses(
            &[
                "affected_status=0".to_string(),
                "executor_status=0".to_string(),
                "verifier_status=9".to_string(),
                "observation_status=7".to_string(),
                "collection_status=5".to_string(),
            ],
            &SCOPED_COVERAGE_REQUIRED_STATUSES,
            SCOPED_COVERAGE_EXECUTOR_LABEL,
        )
        .unwrap();

        assert_eq!(
            recommended_exit_code(&statuses, &SCOPED_COVERAGE_REQUIRED_STATUSES),
            9
        );
    }
}
