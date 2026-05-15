use crate::cli::{ProofArgs, ProofExecutorMode, ProofProfile};
use crate::proof::policy_ast::{CiExecution, ProofPolicy};
use crate::tasks::affected::{
    AffectedReport, AffectedScope, affected_report, changed_files, load_checked_policy,
};
use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

const CI_ENV_VAR: &str = "CI";

#[derive(Debug, Serialize)]
struct ProofPlanReport {
    schema: String,
    ok: bool,
    profile: String,
    base: String,
    head: String,
    changed_files: Vec<String>,
    commands: Vec<ProofPlanCommand>,
    unknown_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
struct ProofPlanCommand {
    scope: String,
    kind: String,
    required: bool,
    command: String,
}

#[derive(Debug, Serialize)]
struct ProofEvidencePlan {
    schema: String,
    status: String,
    execution_status: String,
    profile: String,
    base: String,
    head: String,
    ok: bool,
    changed_files: Vec<String>,
    counts: ProofEvidenceCounts,
    entries: Vec<ProofEvidenceEntry>,
    unknown_files: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ProofEvidenceCounts {
    commands_total: usize,
    required_total: usize,
    advisory_total: usize,
    coverage: ProofEvidenceKindCounts,
    mutation: ProofEvidenceKindCounts,
}

#[derive(Debug, Serialize)]
struct ProofEvidenceKindCounts {
    planned: usize,
    executed: usize,
}

#[derive(Debug, Serialize)]
struct ProofRunSummary {
    schema: String,
    status: String,
    execution_status: String,
    execution_guard: ProofRunExecutionGuard,
    profile: String,
    base: String,
    head: String,
    ok: bool,
    changed_files: Vec<String>,
    counts: ProofRunCounts,
    entries: Vec<ProofRunEntry>,
    unknown_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ProofRunExecutionGuard {
    required: bool,
    enabled: bool,
    ci: bool,
    allow_ci_required_execution: bool,
    allow_local_required_execution: bool,
    reason: String,
}

#[derive(Debug, Serialize)]
struct ProofRunCounts {
    commands_total: usize,
    required_planned: usize,
    advisory_skipped: usize,
    executed: usize,
    passed: usize,
    failed: usize,
}

#[derive(Debug, Serialize)]
struct ProofRunEntry {
    scope: String,
    kind: String,
    required: bool,
    command: String,
    artifact_path: Option<String>,
    status: String,
    skip_reason: String,
    exit_code: Option<i32>,
}

#[derive(Debug, Serialize)]
struct ProofEvidenceEntry {
    scope: String,
    kind: String,
    status: String,
    required: bool,
    command: String,
    artifact_path: Option<String>,
}

#[derive(Debug, Serialize)]
struct ProofExecutorSummary {
    schema: String,
    mode: String,
    status: String,
    execution_status: String,
    execution_guard: ProofExecutorExecutionGuard,
    family: String,
    required: bool,
    profile: String,
    base: String,
    head: String,
    ok: bool,
    changed_files: Vec<String>,
    counts: ProofExecutorCounts,
    entries: Vec<ProofExecutorEntry>,
    unknown_files: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ProofExecutorManifest {
    schema: String,
    mode: String,
    status: String,
    execution_status: String,
    execution_guard: ProofExecutorExecutionGuard,
    family: String,
    required: bool,
    profile: String,
    base: String,
    head: String,
    ok: bool,
    changed_files: Vec<String>,
    selection: ProofExecutorManifestSelection,
    commands: Vec<ProofExecutorManifestCommand>,
    unknown_files: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ProofExecutorManifestSelection {
    source: String,
    max_dry_run_commands: usize,
    required_included: bool,
    selected: usize,
    executed: usize,
}

#[derive(Debug, Serialize)]
struct ProofExecutorCounts {
    commands_total: usize,
    family_planned: usize,
    selected: usize,
    skipped: usize,
    dry_run: usize,
    executed: usize,
    passed: usize,
    failed: usize,
    required_excluded: usize,
    selection_excluded: usize,
    non_family_excluded: usize,
}

#[derive(Debug, Serialize)]
struct ProofExecutorEntry {
    scope: String,
    kind: String,
    required: bool,
    command: String,
    artifact_path: Option<String>,
    status: String,
    skip_reason: String,
    exit_code: Option<i32>,
}

#[derive(Debug, Serialize)]
struct ProofExecutorManifestCommand {
    id: String,
    index: usize,
    scope: String,
    kind: String,
    required: bool,
    command: String,
    artifact_path: Option<String>,
    status: String,
    skip_reason: String,
    exit_code: Option<i32>,
}

#[derive(Debug, Clone)]
struct ProofExecutorConfig {
    family: String,
    ci_execution: CiExecution,
    max_dry_run_commands: usize,
}

#[derive(Debug, Clone, Serialize)]
struct ProofExecutorExecutionGuard {
    required: bool,
    enabled: bool,
    ci: bool,
    ci_execution: String,
    allow_ci_evidence_execution: bool,
    allow_local_evidence_execution: bool,
    reason: String,
}

pub fn run(args: ProofArgs) -> Result<()> {
    if args.plan && args.executor_mode == ProofExecutorMode::Execute {
        bail!("--executor-mode execute cannot be combined with --plan");
    }

    if args.plan && args.run_required {
        bail!("--run-required cannot be combined with --plan");
    }

    if args.run_required && args.executor_mode == ProofExecutorMode::Execute {
        bail!("--run-required cannot be combined with --executor-mode execute");
    }

    if args.run_required && (args.summary_md.is_some() || args.evidence_json.is_some()) {
        bail!(
            "--run-required cannot be combined with --summary-md or --evidence-json; run --plan separately to write plan artifacts"
        );
    }

    if args.run_required
        && (args.executor_summary.is_some()
            || args.executor_manifest.is_some()
            || args.executor_max_commands.is_some()
            || args.allow_ci_evidence_execution
            || args.allow_local_evidence_execution)
    {
        bail!("--run-required cannot be combined with advisory executor options");
    }

    if !args.plan && !args.run_required && args.executor_mode != ProofExecutorMode::Execute {
        bail!(
            "proof execution requires explicit opt-in; pass --plan to print the proof plan, --run-required to execute required commands, or --executor-mode execute to run selected advisory evidence commands"
        );
    }

    let policy = load_checked_policy(&args.policy)?;
    let report = proof_plan_report(&policy, &args)?;

    if let Some(path) = &args.plan_json {
        write_plan_json(path, &report)?;
    }

    if args.executor_mode == ProofExecutorMode::Execute {
        let executor_config = proof_executor_config(&policy, args.executor_max_commands)?;
        run_executor_mode(&args, &report, &executor_config)?;
    } else if args.run_required {
        run_required_mode(&args, &report)?;
    } else {
        let executor_config = proof_executor_config(&policy, args.executor_max_commands)?;
        write_plan_artifacts(&args, &report, &executor_config)?;
    }

    println!("{}", serde_json::to_string_pretty(&report)?);

    if report.ok {
        Ok(())
    } else {
        bail!(
            "proof plan has {} unknown file(s) that need scope policy",
            report.unknown_files.len()
        )
    }
}

fn write_plan_artifacts(
    args: &ProofArgs,
    report: &ProofPlanReport,
    executor_config: &ProofExecutorConfig,
) -> Result<()> {
    let guard = proof_executor_execution_guard(
        args.allow_ci_evidence_execution,
        args.allow_local_evidence_execution,
        executor_config,
    );
    let executor_requested = args.executor_summary.is_some() || args.executor_manifest.is_some();
    let executor_summary = executor_requested.then(|| {
        proof_executor_summary(report, args.executor_mode, guard.clone(), executor_config)
    });
    let executor_manifest = executor_summary
        .as_ref()
        .map(|summary| proof_executor_manifest_from_summary(summary, executor_config));

    if let Some(path) = &args.summary_md {
        write_markdown_summary(path, report, executor_summary.as_ref())?;
    }
    if let Some(path) = &args.evidence_json {
        write_evidence_json(path, report)?;
    }
    if let (Some(path), Some(summary)) = (&args.executor_summary, executor_summary.as_ref()) {
        write_executor_summary(path, summary)?;
    }
    if let (Some(path), Some(manifest)) = (&args.executor_manifest, executor_manifest.as_ref()) {
        write_executor_manifest(path, manifest)?;
    }

    Ok(())
}

fn run_executor_mode(
    args: &ProofArgs,
    report: &ProofPlanReport,
    executor_config: &ProofExecutorConfig,
) -> Result<()> {
    let Some(summary_path) = args.executor_summary.as_ref() else {
        bail!("--executor-summary is required with --executor-mode execute");
    };
    let Some(manifest_path) = args.executor_manifest.as_ref() else {
        bail!("--executor-manifest is required with --executor-mode execute");
    };
    if !report.ok {
        bail!(
            "proof executor refused to run with {} unknown file(s)",
            report.unknown_files.len()
        );
    }

    let guard = proof_executor_execution_guard(
        args.allow_ci_evidence_execution,
        args.allow_local_evidence_execution,
        executor_config,
    );
    if !guard.enabled {
        bail!(
            "proof executor execution guard is not enabled: {}",
            guard.reason
        );
    }

    let summary = proof_executor_execute_summary(report, guard, executor_config)?;
    let manifest = proof_executor_manifest_from_summary(&summary, executor_config);
    write_executor_summary(summary_path, &summary)?;
    write_executor_manifest(manifest_path, &manifest)?;

    if summary.counts.failed == 0 {
        Ok(())
    } else {
        bail!(
            "proof executor command execution failed for {} command(s)",
            summary.counts.failed
        )
    }
}

fn run_required_mode(args: &ProofArgs, report: &ProofPlanReport) -> Result<()> {
    if !report.ok {
        bail!(
            "required proof execution refused to run with {} unknown file(s)",
            report.unknown_files.len()
        );
    }

    let guard = proof_run_execution_guard(
        args.allow_ci_required_execution,
        args.allow_local_required_execution,
    );
    if !guard.enabled {
        bail!(
            "required proof execution guard is not enabled: {}",
            guard.reason
        );
    }

    let summary = proof_run_execute_summary(report, guard)?;
    write_proof_run_summary(&args.proof_run_summary, &summary)?;

    if summary.counts.failed == 0 {
        Ok(())
    } else {
        bail!(
            "required proof command execution failed for {} command(s)",
            summary.counts.failed
        )
    }
}

fn proof_plan_report(policy: &ProofPolicy, args: &ProofArgs) -> Result<ProofPlanReport> {
    match args.profile {
        ProofProfile::Affected => affected_plan_report(policy, args),
        profile => Ok(static_plan_report(profile, &args.base, &args.head)),
    }
}

fn affected_plan_report(policy: &ProofPolicy, args: &ProofArgs) -> Result<ProofPlanReport> {
    let changed_files = changed_files(&args.base, &args.head)?;
    let affected = affected_report(policy, &args.base, &args.head, changed_files)?;
    let commands = affected_commands(policy, &affected);

    Ok(ProofPlanReport {
        schema: "tokmd.proof_plan.v1".to_string(),
        ok: affected.ok,
        profile: profile_name(args.profile).to_string(),
        base: affected.base,
        head: affected.head,
        changed_files: affected.changed_files,
        commands: dedupe_commands(commands),
        unknown_files: affected.unknown_files,
    })
}

fn affected_commands(policy: &ProofPolicy, affected: &AffectedReport) -> Vec<ProofPlanCommand> {
    let mut commands = Vec::new();

    for scope in &affected.scopes {
        for command in &scope.proof {
            commands.push(command_for_scope(scope, "proof", command));
        }

        if let Some(command) = coverage_command(policy, scope) {
            commands.push(command);
        }

        commands.extend(mutation_commands(policy, scope));
    }

    dedupe_commands(commands)
}

fn coverage_command(policy: &ProofPolicy, scope: &AffectedScope) -> Option<ProofPlanCommand> {
    if !scope.coverage || !matches!(scope.kind, crate::proof::policy_ast::ScopeKind::Rust) {
        return None;
    }

    let packages = sorted(scope.packages.clone());
    if packages.is_empty() {
        return None;
    }

    let package_flags = packages
        .iter()
        .map(|package| format!("-p {package}"))
        .collect::<Vec<_>>()
        .join(" ");
    let tool = coverage_command_tool(policy);
    let output_path = format!("target/proof/coverage/{}.lcov", artifact_name(&scope.name));
    let command =
        format!("{tool} {package_flags} --all-features --lcov --output-path {output_path}");

    Some(advisory_command_for_scope(scope, "coverage", &command))
}

fn mutation_commands(policy: &ProofPolicy, scope: &AffectedScope) -> Vec<ProofPlanCommand> {
    if !scope.mutation || !matches!(scope.kind, crate::proof::policy_ast::ScopeKind::Rust) {
        return Vec::new();
    }

    let timeout = policy.defaults.mutation_timeout_seconds.unwrap_or(300);
    let mut commands = scope
        .matched_files
        .iter()
        .filter(|file| is_mutation_candidate(file))
        .map(|file| {
            advisory_command_for_scope(
                scope,
                "mutation",
                &format!("cargo mutants --file {file} --timeout {timeout}"),
            )
        })
        .collect::<Vec<_>>();

    if commands.is_empty() {
        commands.extend(package_mutation_command(scope, timeout));
    }

    commands
}

fn static_plan_report(profile: ProofProfile, base: &str, head: &str) -> ProofPlanReport {
    ProofPlanReport {
        schema: "tokmd.proof_plan.v1".to_string(),
        ok: true,
        profile: profile_name(profile).to_string(),
        base: base.to_string(),
        head: head.to_string(),
        changed_files: Vec::new(),
        commands: static_profile_commands(profile),
        unknown_files: Vec::new(),
    }
}

fn static_profile_commands(profile: ProofProfile) -> Vec<ProofPlanCommand> {
    let commands = match profile {
        ProofProfile::Fast => vec![
            command("workspace", "format", "cargo fmt-check"),
            command("proof_policy", "policy", "cargo xtask proof-policy --check"),
            command(
                "fixture_blobs",
                "guardrail",
                "cargo xtask fixture-blobs-check",
            ),
            command("boundaries", "guardrail", "cargo xtask boundaries-check"),
        ],
        ProofProfile::Release => vec![
            command("docs", "docs", "cargo xtask docs --check"),
            command("version", "release", "cargo xtask version-consistency"),
            command(
                "publish_surface",
                "release",
                "cargo xtask publish-surface --json --verify-publish",
            ),
            command(
                "dependencies",
                "security",
                "cargo deny --all-features check",
            ),
        ],
        ProofProfile::Deep => vec![
            command("workspace", "test", "cargo test --workspace"),
            command("coverage", "coverage", "cargo llvm-cov --workspace --lcov"),
            command("mutation", "mutation", "cargo mutants --timeout 300"),
            command("fuzz", "fuzz", "cargo +nightly fuzz list"),
        ],
        ProofProfile::Affected => Vec::new(),
    };

    dedupe_commands(commands)
}

fn command(scope: &str, kind: &str, command: &str) -> ProofPlanCommand {
    ProofPlanCommand {
        scope: scope.to_string(),
        kind: kind.to_string(),
        required: true,
        command: command.to_string(),
    }
}

fn advisory_command_for_scope(
    scope: &AffectedScope,
    kind: &str,
    command: &str,
) -> ProofPlanCommand {
    ProofPlanCommand {
        scope: scope.name.clone(),
        kind: kind.to_string(),
        required: false,
        command: command.to_string(),
    }
}

fn command_for_scope(scope: &AffectedScope, kind: &str, command_text: &str) -> ProofPlanCommand {
    command(&scope.name, kind, command_text)
}

fn dedupe_commands(commands: Vec<ProofPlanCommand>) -> Vec<ProofPlanCommand> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();

    for command in commands {
        let key = (
            command.scope.clone(),
            command.kind.clone(),
            command.required,
            command.command.clone(),
        );
        if seen.insert(key) {
            deduped.push(command);
        }
    }

    deduped
}

fn write_markdown_summary(
    path: &Path,
    report: &ProofPlanReport,
    executor_summary: Option<&ProofExecutorSummary>,
) -> Result<()> {
    ensure_parent_dir(path)?;
    fs::write(path, render_markdown_summary(report, executor_summary))?;
    Ok(())
}

fn write_plan_json(path: &Path, report: &ProofPlanReport) -> Result<()> {
    ensure_parent_dir(path)?;
    fs::write(path, serde_json::to_string_pretty(report)?)?;
    Ok(())
}

fn write_evidence_json(path: &Path, report: &ProofPlanReport) -> Result<()> {
    ensure_parent_dir(path)?;
    fs::write(
        path,
        serde_json::to_string_pretty(&proof_evidence_plan(report))?,
    )?;
    Ok(())
}

fn write_proof_run_summary(path: &Path, summary: &ProofRunSummary) -> Result<()> {
    ensure_parent_dir(path)?;
    fs::write(path, serde_json::to_string_pretty(summary)?)?;
    Ok(())
}

fn write_executor_summary(path: &Path, summary: &ProofExecutorSummary) -> Result<()> {
    ensure_parent_dir(path)?;
    fs::write(path, serde_json::to_string_pretty(summary)?)?;
    Ok(())
}

fn write_executor_manifest(path: &Path, manifest: &ProofExecutorManifest) -> Result<()> {
    ensure_parent_dir(path)?;
    fs::write(path, serde_json::to_string_pretty(manifest)?)?;
    Ok(())
}

fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn proof_evidence_plan(report: &ProofPlanReport) -> ProofEvidencePlan {
    let entries = report
        .commands
        .iter()
        .filter(|command| is_evidence_kind(&command.kind))
        .map(evidence_entry)
        .collect::<Vec<_>>();
    let coverage_planned = entries
        .iter()
        .filter(|entry| entry.kind == "coverage")
        .count();
    let mutation_planned = entries
        .iter()
        .filter(|entry| entry.kind == "mutation")
        .count();

    ProofEvidencePlan {
        schema: "tokmd.proof_evidence_plan.v1".to_string(),
        status: "planned".to_string(),
        execution_status: "not_executed".to_string(),
        profile: report.profile.clone(),
        base: report.base.clone(),
        head: report.head.clone(),
        ok: report.ok,
        changed_files: report.changed_files.clone(),
        counts: ProofEvidenceCounts {
            commands_total: report.commands.len(),
            required_total: report
                .commands
                .iter()
                .filter(|command| command.required)
                .count(),
            advisory_total: report
                .commands
                .iter()
                .filter(|command| !command.required)
                .count(),
            coverage: ProofEvidenceKindCounts {
                planned: coverage_planned,
                executed: 0,
            },
            mutation: ProofEvidenceKindCounts {
                planned: mutation_planned,
                executed: 0,
            },
        },
        entries,
        unknown_files: report.unknown_files.clone(),
    }
}

fn proof_run_execute_summary(
    report: &ProofPlanReport,
    execution_guard: ProofRunExecutionGuard,
) -> Result<ProofRunSummary> {
    let required_commands = report
        .commands
        .iter()
        .filter(|command| command.required)
        .collect::<Vec<_>>();
    let entries = required_commands
        .iter()
        .map(|command| execute_proof_run_entry(command))
        .collect::<Result<Vec<_>>>()?;
    let executed = entries.len();
    let passed = entries
        .iter()
        .filter(|entry| entry.status == "passed")
        .count();
    let failed = entries
        .iter()
        .filter(|entry| entry.status == "failed")
        .count();

    Ok(ProofRunSummary {
        schema: "tokmd.proof_run_summary.v1".to_string(),
        status: if failed == 0 { "passed" } else { "failed" }.to_string(),
        execution_status: "executed".to_string(),
        execution_guard,
        profile: report.profile.clone(),
        base: report.base.clone(),
        head: report.head.clone(),
        ok: report.ok,
        changed_files: report.changed_files.clone(),
        counts: ProofRunCounts {
            commands_total: report.commands.len(),
            required_planned: required_commands.len(),
            advisory_skipped: report.commands.len() - required_commands.len(),
            executed,
            passed,
            failed,
        },
        entries,
        unknown_files: report.unknown_files.clone(),
    })
}

fn evidence_entry(command: &ProofPlanCommand) -> ProofEvidenceEntry {
    ProofEvidenceEntry {
        scope: command.scope.clone(),
        kind: command.kind.clone(),
        status: "planned".to_string(),
        required: command.required,
        command: command.command.clone(),
        artifact_path: evidence_artifact_path(command),
    }
}

fn evidence_artifact_path(command: &ProofPlanCommand) -> Option<String> {
    if command.kind != "coverage" {
        return None;
    }

    command
        .command
        .split_once("--output-path ")
        .map(|(_, path)| path.split_whitespace().next().unwrap_or(path).to_string())
}

fn is_evidence_kind(kind: &str) -> bool {
    matches!(kind, "coverage" | "mutation")
}

fn proof_executor_summary(
    report: &ProofPlanReport,
    mode: ProofExecutorMode,
    execution_guard: ProofExecutorExecutionGuard,
    config: &ProofExecutorConfig,
) -> ProofExecutorSummary {
    let family = config.family.as_str();
    let family_commands = report
        .commands
        .iter()
        .filter(|command| command.kind == family)
        .collect::<Vec<_>>();
    let selectable_commands = family_commands
        .iter()
        .copied()
        .filter(|command| !command.required)
        .collect::<Vec<_>>();
    let selected_commands =
        selected_executor_commands(&selectable_commands, mode, config.max_dry_run_commands);
    let entries = selected_commands
        .iter()
        .map(|command| executor_entry(command, mode))
        .collect::<Vec<_>>();
    let required_excluded = family_commands
        .iter()
        .filter(|command| command.required)
        .count();
    let selected = entries.len();
    let skipped = match mode {
        ProofExecutorMode::Prototype => selected,
        ProofExecutorMode::DryRun | ProofExecutorMode::Execute => 0,
    };
    let dry_run = match mode {
        ProofExecutorMode::Prototype => 0,
        ProofExecutorMode::DryRun => selected,
        ProofExecutorMode::Execute => 0,
    };

    ProofExecutorSummary {
        schema: "tokmd.proof_executor_summary.v1".to_string(),
        mode: executor_mode_name(mode).to_string(),
        status: executor_status(mode).to_string(),
        execution_status: executor_execution_status(mode).to_string(),
        execution_guard,
        family: family.to_string(),
        required: false,
        profile: report.profile.clone(),
        base: report.base.clone(),
        head: report.head.clone(),
        ok: report.ok,
        changed_files: report.changed_files.clone(),
        counts: ProofExecutorCounts {
            commands_total: report.commands.len(),
            family_planned: family_commands.len(),
            selected,
            skipped,
            dry_run,
            executed: 0,
            passed: 0,
            failed: 0,
            required_excluded,
            selection_excluded: selectable_commands.len() - selected,
            non_family_excluded: report.commands.len() - family_commands.len(),
        },
        entries,
        unknown_files: report.unknown_files.clone(),
    }
}

#[cfg(test)]
fn proof_executor_manifest(
    report: &ProofPlanReport,
    mode: ProofExecutorMode,
    execution_guard: ProofExecutorExecutionGuard,
    config: &ProofExecutorConfig,
) -> ProofExecutorManifest {
    let summary = proof_executor_summary(report, mode, execution_guard, config);
    proof_executor_manifest_from_summary(&summary, config)
}

fn proof_executor_execute_summary(
    report: &ProofPlanReport,
    execution_guard: ProofExecutorExecutionGuard,
    config: &ProofExecutorConfig,
) -> Result<ProofExecutorSummary> {
    let family = config.family.as_str();
    let family_commands = report
        .commands
        .iter()
        .filter(|command| command.kind == family)
        .collect::<Vec<_>>();
    let selectable_commands = family_commands
        .iter()
        .copied()
        .filter(|command| !command.required)
        .collect::<Vec<_>>();
    let selected_commands = selected_executor_commands(
        &selectable_commands,
        ProofExecutorMode::Execute,
        config.max_dry_run_commands,
    );
    let entries = selected_commands
        .iter()
        .map(|command| execute_entry(command))
        .collect::<Result<Vec<_>>>()?;
    let selected = entries.len();
    let passed = entries
        .iter()
        .filter(|entry| entry.status == "passed")
        .count();
    let failed = entries
        .iter()
        .filter(|entry| entry.status == "failed")
        .count();
    let required_excluded = family_commands
        .iter()
        .filter(|command| command.required)
        .count();

    Ok(ProofExecutorSummary {
        schema: "tokmd.proof_executor_summary.v1".to_string(),
        mode: executor_mode_name(ProofExecutorMode::Execute).to_string(),
        status: if failed == 0 { "passed" } else { "failed" }.to_string(),
        execution_status: executor_execution_status(ProofExecutorMode::Execute).to_string(),
        execution_guard,
        family: family.to_string(),
        required: false,
        profile: report.profile.clone(),
        base: report.base.clone(),
        head: report.head.clone(),
        ok: report.ok,
        changed_files: report.changed_files.clone(),
        counts: ProofExecutorCounts {
            commands_total: report.commands.len(),
            family_planned: family_commands.len(),
            selected,
            skipped: 0,
            dry_run: 0,
            executed: selected,
            passed,
            failed,
            required_excluded,
            selection_excluded: selectable_commands.len() - selected,
            non_family_excluded: report.commands.len() - family_commands.len(),
        },
        entries,
        unknown_files: report.unknown_files.clone(),
    })
}

fn proof_executor_manifest_from_summary(
    summary: &ProofExecutorSummary,
    config: &ProofExecutorConfig,
) -> ProofExecutorManifest {
    let commands = summary
        .entries
        .iter()
        .enumerate()
        .map(|(index, entry)| executor_manifest_command(index, entry))
        .collect::<Vec<_>>();

    ProofExecutorManifest {
        schema: "tokmd.proof_executor_manifest.v1".to_string(),
        mode: summary.mode.clone(),
        status: summary.status.clone(),
        execution_status: summary.execution_status.clone(),
        execution_guard: summary.execution_guard.clone(),
        family: summary.family.clone(),
        required: summary.required,
        profile: summary.profile.clone(),
        base: summary.base.clone(),
        head: summary.head.clone(),
        ok: summary.ok,
        changed_files: summary.changed_files.clone(),
        selection: ProofExecutorManifestSelection {
            source: "proof_plan".to_string(),
            max_dry_run_commands: config.max_dry_run_commands,
            required_included: false,
            selected: summary.counts.selected,
            executed: summary.counts.executed,
        },
        commands,
        unknown_files: summary.unknown_files.clone(),
    }
}

fn proof_executor_config(
    policy: &ProofPolicy,
    max_commands_override: Option<usize>,
) -> Result<ProofExecutorConfig> {
    let max_dry_run_commands = match max_commands_override {
        Some(0) => bail!("--executor-max-commands must be greater than zero"),
        Some(max) => max,
        None => policy.executor.max_dry_run_commands.unwrap_or(1),
    };

    Ok(ProofExecutorConfig {
        family: policy
            .executor
            .family
            .clone()
            .unwrap_or_else(|| "coverage".to_string()),
        ci_execution: policy
            .executor
            .ci_execution
            .clone()
            .unwrap_or(CiExecution::ExplicitOptIn),
        max_dry_run_commands,
    })
}

fn proof_executor_execution_guard(
    allow_ci_evidence_execution: bool,
    allow_local_evidence_execution: bool,
    config: &ProofExecutorConfig,
) -> ProofExecutorExecutionGuard {
    proof_executor_execution_guard_for(
        ci_env_enabled(),
        allow_ci_evidence_execution,
        allow_local_evidence_execution,
        &config.ci_execution,
    )
}

fn proof_run_execution_guard(
    allow_ci_required_execution: bool,
    allow_local_required_execution: bool,
) -> ProofRunExecutionGuard {
    proof_run_execution_guard_for(
        ci_env_enabled(),
        allow_ci_required_execution,
        allow_local_required_execution,
    )
}

fn ci_env_enabled() -> bool {
    std::env::var(CI_ENV_VAR)
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn proof_run_execution_guard_for(
    ci: bool,
    allow_ci_required_execution: bool,
    allow_local_required_execution: bool,
) -> ProofRunExecutionGuard {
    let enabled = if ci {
        allow_ci_required_execution
    } else {
        allow_local_required_execution
    };
    let reason = match (
        ci,
        allow_ci_required_execution,
        allow_local_required_execution,
    ) {
        (true, true, _) => "ci_explicit_required_opt_in_enabled",
        (true, false, _) => "ci_requires_--allow-ci-required-execution",
        (false, _, true) => "local_explicit_required_opt_in_enabled",
        (false, true, false) => "not_ci_execution_context",
        (false, false, false) => "local_requires_--allow-local-required-execution",
    };

    ProofRunExecutionGuard {
        required: true,
        enabled,
        ci,
        allow_ci_required_execution,
        allow_local_required_execution,
        reason: reason.to_string(),
    }
}

fn proof_executor_execution_guard_for(
    ci: bool,
    allow_ci_evidence_execution: bool,
    allow_local_evidence_execution: bool,
    ci_execution: &CiExecution,
) -> ProofExecutorExecutionGuard {
    let (enabled, reason, ci_execution_name) = match ci_execution {
        CiExecution::ExplicitOptIn => {
            let enabled = if ci {
                allow_ci_evidence_execution
            } else {
                allow_local_evidence_execution
            };
            let reason = match (
                ci,
                allow_ci_evidence_execution,
                allow_local_evidence_execution,
            ) {
                (true, true, _) => "ci_explicit_opt_in_enabled",
                (true, false, _) => "ci_requires_--allow-ci-evidence-execution",
                (false, _, true) => "local_explicit_opt_in_enabled",
                (false, true, false) => "not_ci_execution_context",
                (false, false, false) => "local_requires_--allow-local-evidence-execution",
            };
            (enabled, reason, "explicit_opt_in")
        }
    };

    ProofExecutorExecutionGuard {
        required: true,
        enabled,
        ci,
        ci_execution: ci_execution_name.to_string(),
        allow_ci_evidence_execution,
        allow_local_evidence_execution,
        reason: reason.to_string(),
    }
}

fn selected_executor_commands<'a>(
    commands: &'a [&'a ProofPlanCommand],
    mode: ProofExecutorMode,
    max_dry_run_commands: usize,
) -> Vec<&'a ProofPlanCommand> {
    match mode {
        ProofExecutorMode::Prototype => commands.to_vec(),
        ProofExecutorMode::DryRun | ProofExecutorMode::Execute => commands
            .iter()
            .take(max_dry_run_commands)
            .copied()
            .collect(),
    }
}

fn executor_entry(command: &ProofPlanCommand, mode: ProofExecutorMode) -> ProofExecutorEntry {
    ProofExecutorEntry {
        scope: command.scope.clone(),
        kind: command.kind.clone(),
        required: command.required,
        command: command.command.clone(),
        artifact_path: evidence_artifact_path(command),
        status: executor_entry_status(mode).to_string(),
        skip_reason: executor_skip_reason(mode).to_string(),
        exit_code: None,
    }
}

fn execute_proof_run_entry(command: &ProofPlanCommand) -> Result<ProofRunEntry> {
    let executor_entry = execute_entry(command)?;

    Ok(ProofRunEntry {
        scope: executor_entry.scope,
        kind: executor_entry.kind,
        required: executor_entry.required,
        command: executor_entry.command,
        artifact_path: executor_entry.artifact_path,
        status: executor_entry.status,
        skip_reason: executor_entry.skip_reason,
        exit_code: executor_entry.exit_code,
    })
}

fn execute_entry(command: &ProofPlanCommand) -> Result<ProofExecutorEntry> {
    if let Some(path) = evidence_artifact_path(command) {
        ensure_parent_dir(Path::new(&path))?;
    }

    let (program, args) = split_command(&command.command)?;
    let output = Command::new(&program)
        .args(&args)
        .output()
        .with_context(|| format!("failed to run executor command `{}`", command.command))?;
    let status = output.status;
    let passed = status.success();
    if !passed {
        emit_failed_executor_output(command, &output.stdout, &output.stderr);
    }

    Ok(ProofExecutorEntry {
        scope: command.scope.clone(),
        kind: command.kind.clone(),
        required: command.required,
        command: command.command.clone(),
        artifact_path: evidence_artifact_path(command),
        status: if passed { "passed" } else { "failed" }.to_string(),
        skip_reason: if passed { "" } else { "command_failed" }.to_string(),
        exit_code: status.code(),
    })
}

fn emit_failed_executor_output(command: &ProofPlanCommand, stdout: &[u8], stderr: &[u8]) {
    eprintln!("executor command failed: {}", command.command);
    if !stdout.is_empty() {
        eprintln!("executor stdout:\n{}", String::from_utf8_lossy(stdout));
    }
    if !stderr.is_empty() {
        eprintln!("executor stderr:\n{}", String::from_utf8_lossy(stderr));
    }
}

fn split_command(command: &str) -> Result<(String, Vec<String>)> {
    if command
        .chars()
        .any(|ch| matches!(ch, '"' | '\'' | '|' | '&' | ';' | '<' | '>'))
    {
        bail!("executor command `{command}` uses shell syntax unsupported by the local executor");
    }

    let mut parts = command.split_whitespace();
    let Some(program) = parts.next() else {
        bail!("executor command must not be empty");
    };
    Ok((
        program.to_string(),
        parts.map(ToOwned::to_owned).collect::<Vec<_>>(),
    ))
}

fn executor_manifest_command(
    index: usize,
    entry: &ProofExecutorEntry,
) -> ProofExecutorManifestCommand {
    ProofExecutorManifestCommand {
        id: executor_command_id(index, entry),
        index: index + 1,
        scope: entry.scope.clone(),
        kind: entry.kind.clone(),
        required: entry.required,
        command: entry.command.clone(),
        artifact_path: entry.artifact_path.clone(),
        status: entry.status.clone(),
        skip_reason: entry.skip_reason.clone(),
        exit_code: entry.exit_code,
    }
}

fn executor_command_id(index: usize, entry: &ProofExecutorEntry) -> String {
    format!(
        "{:04}-{}-{}",
        index + 1,
        artifact_name(&entry.scope),
        artifact_name(&entry.kind)
    )
}

fn executor_mode_name(mode: ProofExecutorMode) -> &'static str {
    match mode {
        ProofExecutorMode::Prototype => "prototype",
        ProofExecutorMode::DryRun => "dry_run",
        ProofExecutorMode::Execute => "execute",
    }
}

fn executor_status(mode: ProofExecutorMode) -> &'static str {
    match mode {
        ProofExecutorMode::Prototype => "prototype",
        ProofExecutorMode::DryRun => "dry_run",
        ProofExecutorMode::Execute => "execute",
    }
}

fn executor_execution_status(mode: ProofExecutorMode) -> &'static str {
    match mode {
        ProofExecutorMode::Prototype => "not_executed",
        ProofExecutorMode::DryRun => "dry_run",
        ProofExecutorMode::Execute => "executed",
    }
}

fn executor_entry_status(mode: ProofExecutorMode) -> &'static str {
    match mode {
        ProofExecutorMode::Prototype => "skipped",
        ProofExecutorMode::DryRun => "dry_run",
        ProofExecutorMode::Execute => "selected",
    }
}

fn executor_skip_reason(mode: ProofExecutorMode) -> &'static str {
    match mode {
        ProofExecutorMode::Prototype => "tool_execution_not_enabled",
        ProofExecutorMode::DryRun => "dry_run_only",
        ProofExecutorMode::Execute => "awaiting_execution",
    }
}

fn render_markdown_summary(
    report: &ProofPlanReport,
    executor_summary: Option<&ProofExecutorSummary>,
) -> String {
    let mut out = String::new();

    out.push_str("## Proof Plan Summary\n\n");
    out.push_str("| Field | Value |\n");
    out.push_str("| --- | --- |\n");
    out.push_str(&format!("| Profile | `{}` |\n", escape_md(&report.profile)));
    out.push_str(&format!("| Base | `{}` |\n", escape_md(&report.base)));
    out.push_str(&format!("| Head | `{}` |\n", escape_md(&report.head)));
    out.push_str(&format!("| OK | `{}` |\n", report.ok));
    out.push_str(&format!(
        "| Changed files | {} |\n",
        report.changed_files.len()
    ));
    out.push_str(&format!(
        "| Unknown files | {} |\n",
        report.unknown_files.len()
    ));
    out.push_str(&format!("| Commands | {} |\n", report.commands.len()));
    out.push('\n');
    out.push_str(
        "Required commands are the current proof selection. Advisory commands are planned evidence candidates and are not CI gates yet.\n\n",
    );

    if report.commands.is_empty() {
        out.push_str("No proof commands planned.\n");
    } else {
        out.push_str("### Command Counts\n\n");
        out.push_str("| Kind | Required | Count |\n");
        out.push_str("| --- | --- | ---: |\n");
        for ((kind, required), count) in command_counts(report) {
            out.push_str(&format!(
                "| `{}` | `{}` | {} |\n",
                escape_md(&kind),
                required,
                count
            ));
        }

        out.push_str("\n### Commands\n\n");
        out.push_str("| Scope | Kind | Required | Command |\n");
        out.push_str("| --- | --- | --- | --- |\n");
        for command in &report.commands {
            out.push_str(&format!(
                "| `{}` | `{}` | `{}` | `{}` |\n",
                escape_md(&command.scope),
                escape_md(&command.kind),
                command.required,
                escape_md(&command.command)
            ));
        }
    }

    if !report.unknown_files.is_empty() {
        out.push_str("\n### Unknown Files\n\n");
        for file in &report.unknown_files {
            out.push_str(&format!("- `{}`\n", escape_md(file)));
        }
    }

    if let Some(summary) = executor_summary {
        out.push_str("\n### Executor Guard\n\n");
        out.push_str("| Field | Value |\n");
        out.push_str("| --- | --- |\n");
        out.push_str(&format!("| Mode | `{}` |\n", escape_md(&summary.mode)));
        out.push_str(&format!(
            "| Execution status | `{}` |\n",
            escape_md(&summary.execution_status)
        ));
        out.push_str(&format!(
            "| Guard required | `{}` |\n",
            summary.execution_guard.required
        ));
        out.push_str(&format!(
            "| Guard enabled | `{}` |\n",
            summary.execution_guard.enabled
        ));
        out.push_str(&format!("| CI | `{}` |\n", summary.execution_guard.ci));
        out.push_str(&format!(
            "| CI execution policy | `{}` |\n",
            escape_md(&summary.execution_guard.ci_execution)
        ));
        out.push_str(&format!(
            "| CI opt-in flag | `{}` |\n",
            summary.execution_guard.allow_ci_evidence_execution
        ));
        out.push_str(&format!(
            "| Local opt-in flag | `{}` |\n",
            summary.execution_guard.allow_local_evidence_execution
        ));
        out.push_str(&format!(
            "| Reason | `{}` |\n",
            escape_md(&summary.execution_guard.reason)
        ));
        out.push_str(&format!(
            "| Selected commands | {} |\n",
            summary.counts.selected
        ));
        out.push_str(&format!(
            "| Executed commands | {} |\n",
            summary.counts.executed
        ));
        out.push_str(&format!(
            "| Failed commands | {} |\n",
            summary.counts.failed
        ));
        out.push_str(
            "\nPlanner-selected evidence commands remain informational and do not replace required proof jobs until maintainers intentionally promote them.\n",
        );
    }

    out
}

fn command_counts(report: &ProofPlanReport) -> BTreeMap<(String, bool), usize> {
    let mut counts = BTreeMap::new();
    for command in &report.commands {
        *counts
            .entry((command.kind.clone(), command.required))
            .or_insert(0) += 1;
    }
    counts
}

fn escape_md(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

fn coverage_command_tool(policy: &ProofPolicy) -> &str {
    match policy.tools.coverage.as_deref() {
        Some("cargo-llvm-cov") | None => "cargo llvm-cov",
        Some(tool) => tool,
    }
}

fn sorted(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn package_mutation_command(scope: &AffectedScope, timeout: u64) -> Option<ProofPlanCommand> {
    let packages = sorted(scope.packages.clone());
    if packages.is_empty() {
        return None;
    }

    let package_flags = packages
        .iter()
        .map(|package| format!("-p {package}"))
        .collect::<Vec<_>>()
        .join(" ");
    Some(advisory_command_for_scope(
        scope,
        "mutation",
        &format!("cargo mutants {package_flags} --timeout {timeout}"),
    ))
}

fn artifact_name(name: &str) -> String {
    name.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn is_mutation_candidate(path: &str) -> bool {
    path.ends_with(".rs")
        && !path.starts_with("fuzz/")
        && !path.contains("/tests/")
        && !path.contains("/benches/")
        && !path.contains("/examples/")
}

fn profile_name(profile: ProofProfile) -> &'static str {
    match profile {
        ProofProfile::Fast => "fast",
        ProofProfile::Affected => "affected",
        ProofProfile::Release => "release",
        ProofProfile::Deep => "deep",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ProofExecutorConfig, ProofPlanCommand, ProofPlanReport, affected_commands, dedupe_commands,
        is_mutation_candidate, proof_evidence_plan, proof_executor_config,
        proof_executor_execute_summary, proof_executor_execution_guard_for,
        proof_executor_manifest, proof_executor_summary, proof_run_execute_summary,
        proof_run_execution_guard_for, render_markdown_summary, split_command,
        static_profile_commands,
    };
    use crate::cli::{ProofExecutorMode, ProofProfile};
    use crate::proof::policy::parse_policy_str;
    use crate::tasks::affected::{AffectedReport, AffectedScope};

    fn coverage_executor_config(max_dry_run_commands: usize) -> ProofExecutorConfig {
        ProofExecutorConfig {
            family: "coverage".to_string(),
            ci_execution: crate::proof::policy_ast::CiExecution::ExplicitOptIn,
            max_dry_run_commands,
        }
    }

    fn explicit_opt_in_guard_for(
        ci: bool,
        allow_ci_evidence_execution: bool,
    ) -> super::ProofExecutorExecutionGuard {
        proof_executor_execution_guard_for(
            ci,
            allow_ci_evidence_execution,
            false,
            &crate::proof::policy_ast::CiExecution::ExplicitOptIn,
        )
    }

    fn explicit_local_opt_in_guard() -> super::ProofExecutorExecutionGuard {
        proof_executor_execution_guard_for(
            false,
            false,
            true,
            &crate::proof::policy_ast::CiExecution::ExplicitOptIn,
        )
    }

    #[test]
    fn dedupe_preserves_first_policy_order() {
        let commands = vec![
            ProofPlanCommand {
                scope: "analysis_ast_shadow".to_string(),
                kind: "proof".to_string(),
                required: true,
                command: "cargo xtask ast-shadow-compare --manifest policy/ast-shadow-corpus.toml --out target/tokmd-ast-shadow".to_string(),
            },
            ProofPlanCommand {
                scope: "analysis_ast_shadow".to_string(),
                kind: "proof".to_string(),
                required: true,
                command: "cargo xtask ast-shadow-check --dir target/tokmd-ast-shadow".to_string(),
            },
            ProofPlanCommand {
                scope: "analysis_ast_shadow".to_string(),
                kind: "proof".to_string(),
                required: true,
                command: "cargo xtask ast-shadow-compare --manifest policy/ast-shadow-corpus.toml --out target/tokmd-ast-shadow".to_string(),
            },
        ];

        let deduped = dedupe_commands(commands);

        assert_eq!(deduped.len(), 2);
        assert!(deduped[0].command.contains("ast-shadow-compare"));
        assert!(deduped[1].command.contains("ast-shadow-check"));
    }

    #[test]
    fn static_profiles_have_deterministic_commands() {
        let fast = static_profile_commands(ProofProfile::Fast);

        assert!(!fast.is_empty());
        assert_eq!(fast, dedupe_commands(fast.clone()));
        assert!(fast.iter().any(|cmd| cmd.command == "cargo fmt-check"));
    }

    #[test]
    fn release_profile_includes_release_facing_checks() {
        let release = static_profile_commands(ProofProfile::Release);

        assert!(
            release
                .iter()
                .any(|cmd| cmd.command.contains("docs --check"))
        );
        assert!(
            release
                .iter()
                .any(|cmd| cmd.command.contains("version-consistency"))
        );
        assert!(
            release
                .iter()
                .any(|cmd| cmd.command.contains("publish-surface"))
        );
    }

    #[test]
    fn deep_profile_includes_heavy_evidence_commands() {
        let deep = static_profile_commands(ProofProfile::Deep);

        assert!(deep.iter().any(|cmd| cmd.kind == "coverage"));
        assert!(deep.iter().any(|cmd| cmd.kind == "mutation"));
        assert!(deep.iter().any(|cmd| cmd.kind == "fuzz"));
    }

    #[test]
    fn executor_config_uses_policy_limit_by_default() {
        let policy = parse_policy_str(
            r#"
schema = "tokmd.proof_policy.v1"

[executor]
family = "coverage"
max_dry_run_commands = 2
"#,
        )
        .expect("policy should parse");

        let config = proof_executor_config(&policy, None).expect("config should load");

        assert_eq!(config.max_dry_run_commands, 2);
    }

    #[test]
    fn executor_config_allows_positive_selection_override() {
        let policy = parse_policy_str(
            r#"
schema = "tokmd.proof_policy.v1"

[executor]
family = "coverage"
max_dry_run_commands = 1
"#,
        )
        .expect("policy should parse");

        let config = proof_executor_config(&policy, Some(3)).expect("config should load");

        assert_eq!(config.max_dry_run_commands, 3);
    }

    #[test]
    fn executor_config_rejects_zero_selection_override() {
        let policy = parse_policy_str(
            r#"
schema = "tokmd.proof_policy.v1"
"#,
        )
        .expect("policy should parse");

        let error = proof_executor_config(&policy, Some(0))
            .unwrap_err()
            .to_string();

        assert!(error.contains("--executor-max-commands"));
    }

    #[test]
    fn affected_plan_adds_scoped_coverage_and_mutation_commands() {
        let policy = parse_policy_str(
            r#"
schema = "tokmd.proof_policy.v1"

[defaults]
mutation_timeout_seconds = 123

[tools]
coverage = "cargo-llvm-cov"
"#,
        )
        .expect("policy should parse");
        let affected = AffectedReport {
            schema: "tokmd.affected.v1".to_string(),
            ok: true,
            base: "base".to_string(),
            head: "head".to_string(),
            changed_files: vec![
                "crates/tokmd-core/src/ffi.rs".to_string(),
                "crates/tokmd-core/tests/ffi.rs".to_string(),
            ],
            scopes: vec![AffectedScope {
                name: "tokmd_core_ffi".to_string(),
                kind: crate::proof::policy_ast::ScopeKind::Rust,
                reason: "matched crates/tokmd-core/src/ffi.rs".to_string(),
                matched_files: vec![
                    "crates/tokmd-core/src/ffi.rs".to_string(),
                    "crates/tokmd-core/tests/ffi.rs".to_string(),
                ],
                packages: vec!["tokmd-core".to_string()],
                proof: vec!["cargo test -p tokmd-core ffi".to_string()],
                mutation: true,
                coverage: true,
            }],
            unknown_files: Vec::new(),
        };

        let commands = affected_commands(&policy, &affected);

        assert_eq!(commands[0].kind, "proof");
        assert!(commands[0].required);
        assert!(
            commands
                .iter()
                .any(|cmd| cmd.command == "cargo test -p tokmd-core ffi")
        );
        assert!(commands.iter().any(|cmd| {
            cmd.kind == "coverage"
                && !cmd.required
                && cmd.command == "cargo llvm-cov -p tokmd-core --all-features --lcov --output-path target/proof/coverage/tokmd_core_ffi.lcov"
        }));
        assert!(commands.iter().any(|cmd| {
            cmd.kind == "mutation"
                && !cmd.required
                && cmd.command == "cargo mutants --file crates/tokmd-core/src/ffi.rs --timeout 123"
        }));
        assert!(
            !commands
                .iter()
                .any(|cmd| cmd.command.contains("crates/tokmd-core/tests/ffi.rs"))
        );
    }

    #[test]
    fn markdown_summary_marks_advisory_evidence_commands() {
        let report = ProofPlanReport {
            schema: "tokmd.proof_plan.v1".to_string(),
            ok: true,
            profile: "affected".to_string(),
            base: "origin/main".to_string(),
            head: "HEAD".to_string(),
            changed_files: vec!["crates/tokmd-core/src/ffi.rs".to_string()],
            commands: vec![
                ProofPlanCommand {
                    scope: "tokmd_core_ffi".to_string(),
                    kind: "proof".to_string(),
                    required: true,
                    command: "cargo test -p tokmd-core ffi".to_string(),
                },
                ProofPlanCommand {
                    scope: "tokmd_core_ffi".to_string(),
                    kind: "coverage".to_string(),
                    required: false,
                    command: "cargo llvm-cov -p tokmd-core --all-features --lcov".to_string(),
                },
                ProofPlanCommand {
                    scope: "tokmd_core_ffi".to_string(),
                    kind: "mutation".to_string(),
                    required: false,
                    command: "cargo mutants --file crates/tokmd-core/src/ffi.rs --timeout 300"
                        .to_string(),
                },
            ],
            unknown_files: Vec::new(),
        };

        let summary = render_markdown_summary(&report, None);

        assert!(summary.contains("Required commands are the current proof selection"));
        assert!(summary.contains("| `proof` | `true` | 1 |"));
        assert!(summary.contains("| `coverage` | `false` | 1 |"));
        assert!(summary.contains("| `mutation` | `false` | 1 |"));
        assert!(summary.contains("cargo mutants --file crates/tokmd-core/src/ffi.rs"));
    }

    #[test]
    fn markdown_summary_surfaces_executor_guard_when_available() {
        let report = ProofPlanReport {
            schema: "tokmd.proof_plan.v1".to_string(),
            ok: true,
            profile: "affected".to_string(),
            base: "origin/main".to_string(),
            head: "HEAD".to_string(),
            changed_files: vec!["crates/tokmd-core/src/ffi.rs".to_string()],
            commands: vec![ProofPlanCommand {
                scope: "tokmd_core_ffi".to_string(),
                kind: "coverage".to_string(),
                required: false,
                command: "cargo llvm-cov -p tokmd-core --all-features --lcov --output-path target/proof/coverage/tokmd_core_ffi.lcov".to_string(),
            }],
            unknown_files: Vec::new(),
        };
        let executor_summary = proof_executor_summary(
            &report,
            ProofExecutorMode::DryRun,
            explicit_opt_in_guard_for(true, false),
            &coverage_executor_config(1),
        );

        let summary = render_markdown_summary(&report, Some(&executor_summary));

        assert!(summary.contains("### Executor Guard"));
        assert!(summary.contains("| Mode | `dry_run` |"));
        assert!(summary.contains("| Guard enabled | `false` |"));
        assert!(summary.contains("| CI | `true` |"));
        assert!(summary.contains("| CI execution policy | `explicit_opt_in` |"));
        assert!(summary.contains("ci_requires_--allow-ci-evidence-execution"));
        assert!(summary.contains("| Selected commands | 1 |"));
        assert!(summary.contains("| Executed commands | 0 |"));
    }

    #[test]
    fn evidence_plan_marks_scoped_evidence_as_planned_not_executed() {
        let report = ProofPlanReport {
            schema: "tokmd.proof_plan.v1".to_string(),
            ok: true,
            profile: "affected".to_string(),
            base: "origin/main".to_string(),
            head: "HEAD".to_string(),
            changed_files: vec!["crates/tokmd-core/src/ffi.rs".to_string()],
            commands: vec![
                ProofPlanCommand {
                    scope: "tokmd_core_ffi".to_string(),
                    kind: "proof".to_string(),
                    required: true,
                    command: "cargo test -p tokmd-core ffi".to_string(),
                },
                ProofPlanCommand {
                    scope: "tokmd_core_ffi".to_string(),
                    kind: "coverage".to_string(),
                    required: false,
                    command: "cargo llvm-cov -p tokmd-core --all-features --lcov --output-path target/proof/coverage/tokmd_core_ffi.lcov".to_string(),
                },
                ProofPlanCommand {
                    scope: "tokmd_core_ffi".to_string(),
                    kind: "mutation".to_string(),
                    required: false,
                    command: "cargo mutants --file crates/tokmd-core/src/ffi.rs --timeout 300"
                        .to_string(),
                },
            ],
            unknown_files: Vec::new(),
        };

        let evidence = proof_evidence_plan(&report);

        assert_eq!(evidence.schema, "tokmd.proof_evidence_plan.v1");
        assert_eq!(evidence.status, "planned");
        assert_eq!(evidence.execution_status, "not_executed");
        assert_eq!(evidence.counts.commands_total, 3);
        assert_eq!(evidence.counts.required_total, 1);
        assert_eq!(evidence.counts.advisory_total, 2);
        assert_eq!(evidence.counts.coverage.planned, 1);
        assert_eq!(evidence.counts.coverage.executed, 0);
        assert_eq!(evidence.counts.mutation.planned, 1);
        assert_eq!(evidence.counts.mutation.executed, 0);
        assert_eq!(evidence.entries.len(), 2);
        assert_eq!(evidence.entries[0].kind, "coverage");
        assert_eq!(evidence.entries[0].status, "planned");
        assert_eq!(
            evidence.entries[0].artifact_path.as_deref(),
            Some("target/proof/coverage/tokmd_core_ffi.lcov")
        );
        assert_eq!(evidence.entries[1].kind, "mutation");
        assert_eq!(evidence.entries[1].artifact_path, None);
    }

    #[test]
    fn required_proof_run_executes_required_commands_only() {
        let report = ProofPlanReport {
            schema: "tokmd.proof_plan.v1".to_string(),
            ok: true,
            profile: "affected".to_string(),
            base: "origin/main".to_string(),
            head: "HEAD".to_string(),
            changed_files: vec!["crates/tokmd-core/src/ffi.rs".to_string()],
            commands: vec![
                ProofPlanCommand {
                    scope: "tokmd_core_ffi".to_string(),
                    kind: "proof".to_string(),
                    required: true,
                    command: "rustc --version".to_string(),
                },
                ProofPlanCommand {
                    scope: "tokmd_core_ffi".to_string(),
                    kind: "coverage".to_string(),
                    required: false,
                    command: "definitely-not-a-real-command-for-advisory-proof".to_string(),
                },
                ProofPlanCommand {
                    scope: "tokmd_core_ffi".to_string(),
                    kind: "mutation".to_string(),
                    required: false,
                    command: "definitely-not-a-real-command-for-mutation-proof".to_string(),
                },
            ],
            unknown_files: Vec::new(),
        };

        let summary =
            proof_run_execute_summary(&report, proof_run_execution_guard_for(false, false, true))
                .expect("required proof summary");

        assert_eq!(summary.schema, "tokmd.proof_run_summary.v1");
        assert_eq!(summary.status, "passed");
        assert_eq!(summary.execution_status, "executed");
        assert!(summary.execution_guard.enabled);
        assert_eq!(
            summary.execution_guard.reason,
            "local_explicit_required_opt_in_enabled"
        );
        assert_eq!(summary.counts.commands_total, 3);
        assert_eq!(summary.counts.required_planned, 1);
        assert_eq!(summary.counts.advisory_skipped, 2);
        assert_eq!(summary.counts.executed, 1);
        assert_eq!(summary.counts.passed, 1);
        assert_eq!(summary.counts.failed, 0);
        assert_eq!(summary.entries.len(), 1);
        assert_eq!(summary.entries[0].kind, "proof");
        assert!(summary.entries[0].required);
        assert_eq!(summary.entries[0].status, "passed");
    }

    #[test]
    fn executor_summary_selects_only_advisory_coverage_without_execution() {
        let report = ProofPlanReport {
            schema: "tokmd.proof_plan.v1".to_string(),
            ok: true,
            profile: "affected".to_string(),
            base: "origin/main".to_string(),
            head: "HEAD".to_string(),
            changed_files: vec!["crates/tokmd-core/src/ffi.rs".to_string()],
            commands: vec![
                ProofPlanCommand {
                    scope: "tokmd_core_ffi".to_string(),
                    kind: "proof".to_string(),
                    required: true,
                    command: "cargo test -p tokmd-core ffi".to_string(),
                },
                ProofPlanCommand {
                    scope: "tokmd_core_ffi".to_string(),
                    kind: "coverage".to_string(),
                    required: false,
                    command: "cargo llvm-cov -p tokmd-core --all-features --lcov --output-path target/proof/coverage/tokmd_core_ffi.lcov".to_string(),
                },
                ProofPlanCommand {
                    scope: "coverage".to_string(),
                    kind: "coverage".to_string(),
                    required: true,
                    command: "cargo llvm-cov --workspace --lcov".to_string(),
                },
                ProofPlanCommand {
                    scope: "tokmd_core_ffi".to_string(),
                    kind: "mutation".to_string(),
                    required: false,
                    command: "cargo mutants --file crates/tokmd-core/src/ffi.rs --timeout 300"
                        .to_string(),
                },
            ],
            unknown_files: Vec::new(),
        };

        let summary = proof_executor_summary(
            &report,
            ProofExecutorMode::Prototype,
            explicit_opt_in_guard_for(false, false),
            &coverage_executor_config(1),
        );

        assert_eq!(summary.schema, "tokmd.proof_executor_summary.v1");
        assert_eq!(summary.mode, "prototype");
        assert_eq!(summary.status, "prototype");
        assert_eq!(summary.execution_status, "not_executed");
        assert!(summary.execution_guard.required);
        assert!(!summary.execution_guard.enabled);
        assert!(!summary.execution_guard.ci);
        assert_eq!(summary.execution_guard.ci_execution, "explicit_opt_in");
        assert!(!summary.execution_guard.allow_ci_evidence_execution);
        assert!(!summary.execution_guard.allow_local_evidence_execution);
        assert_eq!(
            summary.execution_guard.reason,
            "local_requires_--allow-local-evidence-execution"
        );
        assert_eq!(summary.family, "coverage");
        assert!(!summary.required);
        assert_eq!(summary.counts.commands_total, 4);
        assert_eq!(summary.counts.family_planned, 2);
        assert_eq!(summary.counts.selected, 1);
        assert_eq!(summary.counts.skipped, 1);
        assert_eq!(summary.counts.dry_run, 0);
        assert_eq!(summary.counts.executed, 0);
        assert_eq!(summary.counts.required_excluded, 1);
        assert_eq!(summary.counts.selection_excluded, 0);
        assert_eq!(summary.counts.non_family_excluded, 2);
        assert_eq!(summary.entries.len(), 1);
        assert_eq!(summary.entries[0].kind, "coverage");
        assert!(!summary.entries[0].required);
        assert_eq!(summary.entries[0].status, "skipped");
        assert_eq!(summary.entries[0].skip_reason, "tool_execution_not_enabled");
        assert_eq!(
            summary.entries[0].artifact_path.as_deref(),
            Some("target/proof/coverage/tokmd_core_ffi.lcov")
        );
    }

    #[test]
    fn dry_run_executor_summary_selects_one_advisory_coverage_command() {
        let report = ProofPlanReport {
            schema: "tokmd.proof_plan.v1".to_string(),
            ok: true,
            profile: "affected".to_string(),
            base: "origin/main".to_string(),
            head: "HEAD".to_string(),
            changed_files: vec![
                "crates/tokmd-core/src/ffi.rs".to_string(),
                "crates/tokmd/src/main.rs".to_string(),
            ],
            commands: vec![
                ProofPlanCommand {
                    scope: "tokmd_core_ffi".to_string(),
                    kind: "coverage".to_string(),
                    required: false,
                    command: "cargo llvm-cov -p tokmd-core --all-features --lcov --output-path target/proof/coverage/tokmd_core_ffi.lcov".to_string(),
                },
                ProofPlanCommand {
                    scope: "tokmd_cli".to_string(),
                    kind: "coverage".to_string(),
                    required: false,
                    command: "cargo llvm-cov -p tokmd --all-features --lcov --output-path target/proof/coverage/tokmd_cli.lcov".to_string(),
                },
                ProofPlanCommand {
                    scope: "tokmd_core_ffi".to_string(),
                    kind: "mutation".to_string(),
                    required: false,
                    command: "cargo mutants --file crates/tokmd-core/src/ffi.rs --timeout 300"
                        .to_string(),
                },
            ],
            unknown_files: Vec::new(),
        };

        let summary = proof_executor_summary(
            &report,
            ProofExecutorMode::DryRun,
            explicit_opt_in_guard_for(true, false),
            &coverage_executor_config(1),
        );

        assert_eq!(summary.mode, "dry_run");
        assert_eq!(summary.status, "dry_run");
        assert_eq!(summary.execution_status, "dry_run");
        assert!(summary.execution_guard.required);
        assert!(!summary.execution_guard.enabled);
        assert!(summary.execution_guard.ci);
        assert_eq!(summary.execution_guard.ci_execution, "explicit_opt_in");
        assert!(!summary.execution_guard.allow_ci_evidence_execution);
        assert!(!summary.execution_guard.allow_local_evidence_execution);
        assert_eq!(
            summary.execution_guard.reason,
            "ci_requires_--allow-ci-evidence-execution"
        );
        assert_eq!(summary.counts.family_planned, 2);
        assert_eq!(summary.counts.selected, 1);
        assert_eq!(summary.counts.skipped, 0);
        assert_eq!(summary.counts.dry_run, 1);
        assert_eq!(summary.counts.executed, 0);
        assert_eq!(summary.counts.selection_excluded, 1);
        assert_eq!(summary.entries.len(), 1);
        assert_eq!(summary.entries[0].status, "dry_run");
        assert_eq!(summary.entries[0].skip_reason, "dry_run_only");
        assert_eq!(summary.entries[0].scope, "tokmd_core_ffi");
    }

    #[test]
    fn dry_run_executor_summary_respects_policy_selection_limit() {
        let report = ProofPlanReport {
            schema: "tokmd.proof_plan.v1".to_string(),
            ok: true,
            profile: "affected".to_string(),
            base: "origin/main".to_string(),
            head: "HEAD".to_string(),
            changed_files: vec!["crates/tokmd-core/src/ffi.rs".to_string()],
            commands: vec![
                ProofPlanCommand {
                    scope: "tokmd_core_ffi".to_string(),
                    kind: "coverage".to_string(),
                    required: false,
                    command: "cargo llvm-cov -p tokmd-core --all-features --lcov --output-path target/proof/coverage/tokmd_core_ffi.lcov".to_string(),
                },
                ProofPlanCommand {
                    scope: "tokmd_cli".to_string(),
                    kind: "coverage".to_string(),
                    required: false,
                    command: "cargo llvm-cov -p tokmd --all-features --lcov --output-path target/proof/coverage/tokmd_cli.lcov".to_string(),
                },
            ],
            unknown_files: Vec::new(),
        };

        let summary = proof_executor_summary(
            &report,
            ProofExecutorMode::DryRun,
            explicit_opt_in_guard_for(true, false),
            &coverage_executor_config(2),
        );

        assert_eq!(summary.counts.family_planned, 2);
        assert_eq!(summary.counts.selected, 2);
        assert_eq!(summary.counts.selection_excluded, 0);
        assert_eq!(summary.entries.len(), 2);
    }

    #[test]
    fn executor_manifest_records_selected_commands_with_stable_ids() {
        let report = ProofPlanReport {
            schema: "tokmd.proof_plan.v1".to_string(),
            ok: true,
            profile: "affected".to_string(),
            base: "origin/main".to_string(),
            head: "HEAD".to_string(),
            changed_files: vec!["crates/tokmd-core/src/ffi.rs".to_string()],
            commands: vec![
                ProofPlanCommand {
                    scope: "tokmd_core_ffi".to_string(),
                    kind: "coverage".to_string(),
                    required: false,
                    command: "cargo llvm-cov -p tokmd-core --all-features --lcov --output-path target/proof/coverage/tokmd_core_ffi.lcov".to_string(),
                },
                ProofPlanCommand {
                    scope: "tokmd_cli".to_string(),
                    kind: "coverage".to_string(),
                    required: false,
                    command: "cargo llvm-cov -p tokmd --all-features --lcov --output-path target/proof/coverage/tokmd_cli.lcov".to_string(),
                },
            ],
            unknown_files: Vec::new(),
        };

        let manifest = proof_executor_manifest(
            &report,
            ProofExecutorMode::DryRun,
            explicit_opt_in_guard_for(true, false),
            &coverage_executor_config(2),
        );

        assert_eq!(manifest.schema, "tokmd.proof_executor_manifest.v1");
        assert_eq!(manifest.family, "coverage");
        assert_eq!(manifest.selection.source, "proof_plan");
        assert_eq!(manifest.selection.max_dry_run_commands, 2);
        assert!(!manifest.selection.required_included);
        assert_eq!(manifest.selection.selected, 2);
        assert_eq!(manifest.selection.executed, 0);
        assert_eq!(manifest.commands.len(), 2);
        assert_eq!(manifest.commands[0].id, "0001-tokmd_core_ffi-coverage");
        assert_eq!(manifest.commands[0].index, 1);
        assert_eq!(manifest.commands[0].status, "dry_run");
        assert_eq!(manifest.commands[0].skip_reason, "dry_run_only");
        assert_eq!(
            manifest.commands[0].artifact_path.as_deref(),
            Some("target/proof/coverage/tokmd_core_ffi.lcov")
        );
    }

    #[test]
    fn execute_executor_summary_runs_selected_local_coverage_command() {
        let report = ProofPlanReport {
            schema: "tokmd.proof_plan.v1".to_string(),
            ok: true,
            profile: "affected".to_string(),
            base: "origin/main".to_string(),
            head: "HEAD".to_string(),
            changed_files: vec!["crates/tokmd-core/src/ffi.rs".to_string()],
            commands: vec![
                ProofPlanCommand {
                    scope: "tokmd_core_ffi".to_string(),
                    kind: "coverage".to_string(),
                    required: false,
                    command: "rustc --version".to_string(),
                },
                ProofPlanCommand {
                    scope: "tokmd_cli".to_string(),
                    kind: "coverage".to_string(),
                    required: false,
                    command: "rustc --version".to_string(),
                },
            ],
            unknown_files: Vec::new(),
        };

        let summary = proof_executor_execute_summary(
            &report,
            explicit_local_opt_in_guard(),
            &coverage_executor_config(1),
        )
        .expect("executor summary");

        assert_eq!(summary.mode, "execute");
        assert_eq!(summary.status, "passed");
        assert_eq!(summary.execution_status, "executed");
        assert!(summary.execution_guard.enabled);
        assert_eq!(summary.counts.family_planned, 2);
        assert_eq!(summary.counts.selected, 1);
        assert_eq!(summary.counts.executed, 1);
        assert_eq!(summary.counts.passed, 1);
        assert_eq!(summary.counts.failed, 0);
        assert_eq!(summary.counts.selection_excluded, 1);
        assert_eq!(summary.entries.len(), 1);
        assert_eq!(summary.entries[0].status, "passed");
        assert_eq!(summary.entries[0].skip_reason, "");
    }

    #[test]
    fn executor_command_splitter_rejects_shell_syntax() {
        let (program, args) =
            split_command("cargo llvm-cov -p tokmd").expect("simple command should split");
        assert_eq!(program, "cargo");
        assert_eq!(args, vec!["llvm-cov", "-p", "tokmd"]);

        let error = split_command("cargo llvm-cov | tee out").unwrap_err();
        assert!(error.to_string().contains("unsupported"));
    }

    #[test]
    fn executor_execution_guard_requires_ci_and_explicit_flag() {
        let local_default = explicit_opt_in_guard_for(false, false);
        assert!(local_default.required);
        assert!(!local_default.enabled);
        assert_eq!(local_default.ci_execution, "explicit_opt_in");
        assert_eq!(
            local_default.reason,
            "local_requires_--allow-local-evidence-execution"
        );

        let ci_without_flag = explicit_opt_in_guard_for(true, false);
        assert!(!ci_without_flag.enabled);
        assert_eq!(
            ci_without_flag.reason,
            "ci_requires_--allow-ci-evidence-execution"
        );

        let local_with_ci_flag = explicit_opt_in_guard_for(false, true);
        assert!(!local_with_ci_flag.enabled);
        assert_eq!(local_with_ci_flag.reason, "not_ci_execution_context");

        let local_with_local_flag = explicit_local_opt_in_guard();
        assert!(local_with_local_flag.enabled);
        assert_eq!(
            local_with_local_flag.reason,
            "local_explicit_opt_in_enabled"
        );
        assert!(local_with_local_flag.allow_local_evidence_execution);

        let ci_with_flag = explicit_opt_in_guard_for(true, true);
        assert!(ci_with_flag.enabled);
        assert_eq!(ci_with_flag.reason, "ci_explicit_opt_in_enabled");
    }

    #[test]
    fn affected_plan_uses_package_mutation_fallback_without_source_files() {
        let policy = parse_policy_str(
            r#"
schema = "tokmd.proof_policy.v1"

[defaults]
mutation_timeout_seconds = 77
"#,
        )
        .expect("policy should parse");
        let affected = AffectedReport {
            schema: "tokmd.affected.v1".to_string(),
            ok: true,
            base: "base".to_string(),
            head: "head".to_string(),
            changed_files: vec!["crates/tokmd-core/Cargo.toml".to_string()],
            scopes: vec![AffectedScope {
                name: "tokmd_core_manifest".to_string(),
                kind: crate::proof::policy_ast::ScopeKind::Rust,
                reason: "matched crates/tokmd-core/Cargo.toml".to_string(),
                matched_files: vec!["crates/tokmd-core/Cargo.toml".to_string()],
                packages: vec!["tokmd-core".to_string()],
                proof: vec!["cargo test -p tokmd-core".to_string()],
                mutation: true,
                coverage: false,
            }],
            unknown_files: Vec::new(),
        };

        let commands = affected_commands(&policy, &affected);

        assert!(commands.iter().any(|cmd| {
            cmd.kind == "mutation"
                && !cmd.required
                && cmd.command == "cargo mutants -p tokmd-core --timeout 77"
        }));
    }

    #[test]
    fn mutation_candidates_exclude_test_and_fixture_surfaces() {
        assert!(is_mutation_candidate("crates/tokmd-core/src/ffi.rs"));
        assert!(!is_mutation_candidate("crates/tokmd-core/tests/ffi.rs"));
        assert!(!is_mutation_candidate(
            "fuzz/fuzz_targets/fuzz_badge_svg.rs"
        ));
        assert!(!is_mutation_candidate("crates/tokmd/examples/demo.rs"));
    }
}
