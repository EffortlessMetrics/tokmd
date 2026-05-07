use crate::cli::{
    ProofArtifactsCheckArgs, ProofExecutionObservationArgs, ProofExecutionObservationsSummaryArgs,
};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const SUMMARY_SCHEMA: &str = "tokmd.proof_executor_summary.v1";
const MANIFEST_SCHEMA: &str = "tokmd.proof_executor_manifest.v1";
const OBSERVATION_SCHEMA: &str = "tokmd.proof_executor_observation.v1";
const OBSERVATION_COLLECTION_SCHEMA: &str = "tokmd.proof_executor_observation_collection.v1";

const SHARED_FIELDS: &[&str] = &[
    "mode",
    "status",
    "execution_status",
    "execution_guard",
    "family",
    "required",
    "profile",
    "base",
    "head",
    "ok",
    "changed_files",
    "unknown_files",
];

const ENTRY_FIELDS: &[&str] = &[
    "scope",
    "kind",
    "required",
    "command",
    "artifact_path",
    "status",
    "skip_reason",
    "exit_code",
];

pub fn run(args: ProofArtifactsCheckArgs) -> Result<()> {
    let summary = read_json(&args.executor_summary, "executor summary")?;
    let manifest = read_json(&args.executor_manifest, "executor manifest")?;

    let report = validate_executor_artifacts(&summary, &manifest, VerificationMode::NoExecution)?;
    println!(
        "Proof artifacts OK: {} command(s), execution_status {}, guard {}",
        report.selected, report.execution_status, report.guard_reason
    );
    Ok(())
}

pub fn run_execution(args: ProofArtifactsCheckArgs) -> Result<()> {
    let summary = read_json(&args.executor_summary, "executor summary")?;
    let manifest = read_json(&args.executor_manifest, "executor manifest")?;

    let report = validate_executor_artifacts(&summary, &manifest, VerificationMode::Execution)?;
    println!(
        "Proof execution artifacts OK: {} executed command(s), guard {}",
        report.executed, report.guard_reason
    );
    Ok(())
}

pub fn run_observation(args: ProofExecutionObservationArgs) -> Result<()> {
    let summary = read_json(&args.executor_summary, "executor summary")?;
    let manifest = read_json(&args.executor_manifest, "executor manifest")?;

    let observation = proof_execution_observation(&summary, &manifest)?;
    write_observation(&args.output, &observation)?;
    println!(
        "Proof execution observation OK: {} executed command(s), wrote `{}`",
        observation.counts.executed,
        args.output.display()
    );
    Ok(())
}

pub fn run_observations_summary(args: ProofExecutionObservationsSummaryArgs) -> Result<()> {
    let observations = collect_observation_paths(&args)?;
    let collection = proof_execution_observation_collection(&observations)?;
    validate_observation_collection_thresholds(&collection, &args)?;
    let json = serde_json::to_string_pretty(&collection)?;

    if let Some(output) = &args.output {
        write_text(output, &json)?;
        println!(
            "Proof execution observation collection OK: {} observation(s), {} scope(s), wrote `{}`",
            collection.counts.observations,
            collection.scopes.len(),
            output.display()
        );
    } else {
        println!("{json}");
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VerificationMode {
    NoExecution,
    Execution,
}

#[derive(Debug, PartialEq, Eq)]
struct ProofArtifactsReport {
    selected: usize,
    executed: usize,
    execution_status: String,
    guard_reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct ProofExecutionObservation {
    schema: String,
    status: String,
    execution_status: String,
    profile: String,
    base: String,
    head: String,
    family: String,
    required: bool,
    ok: bool,
    execution_guard: ProofExecutionObservationGuard,
    counts: ProofExecutionObservationCounts,
    scopes: Vec<ProofExecutionObservationScope>,
    changed_files: Vec<String>,
    unknown_files: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct ProofExecutionObservationGuard {
    enabled: bool,
    ci: bool,
    reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct ProofExecutionObservationCounts {
    selected: usize,
    executed: usize,
    passed: usize,
    failed: usize,
    artifacts: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct ProofExecutionObservationScope {
    name: String,
    kind: String,
    command: String,
    artifact_path: Option<String>,
    status: String,
    exit_code: Option<i64>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct ProofExecutionObservationCollection {
    schema: String,
    ok: bool,
    counts: ProofExecutionObservationCollectionCounts,
    families: Vec<ProofExecutionObservationFamilySummary>,
    scopes: Vec<ProofExecutionObservationScopeSummary>,
    sources: Vec<ProofExecutionObservationSourceSummary>,
}

#[derive(Debug, Default, Serialize, PartialEq, Eq)]
struct ProofExecutionObservationCollectionCounts {
    observations: usize,
    selected: usize,
    executed: usize,
    passed: usize,
    failed: usize,
    artifacts: usize,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct ProofExecutionObservationFamilySummary {
    family: String,
    observations: usize,
    selected: usize,
    executed: usize,
    passed: usize,
    artifacts: usize,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct ProofExecutionObservationScopeSummary {
    name: String,
    kind: String,
    family: String,
    observations: usize,
    executed: usize,
    artifacts: usize,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct ProofExecutionObservationSourceSummary {
    path: String,
    status: String,
    execution_status: String,
    profile: String,
    base: String,
    head: String,
    family: String,
    guard_reason: String,
    selected: usize,
    executed: usize,
    passed: usize,
    artifacts: usize,
}

#[derive(Debug)]
struct SourcedProofExecutionObservation {
    path: PathBuf,
    observation: ProofExecutionObservation,
}

#[derive(Default)]
struct FamilyAccumulator {
    observations: usize,
    selected: usize,
    executed: usize,
    passed: usize,
    artifacts: usize,
}

#[derive(Default)]
struct ScopeAccumulator {
    observations: usize,
    executed: usize,
    artifacts: usize,
}

fn read_json(path: &Path, label: &str) -> Result<Value> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read {label} artifact `{}`", path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse {label} artifact `{}`", path.display()))
}

fn write_observation(path: &Path, observation: &ProofExecutionObservation) -> Result<()> {
    write_text(path, &serde_json::to_string_pretty(observation)?)
}

fn write_text(path: &Path, text: &str) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create `{}`", parent.display()))?;
    }
    fs::write(path, text).with_context(|| format!("failed to write `{}`", path.display()))
}

fn proof_execution_observation(
    summary: &Value,
    manifest: &Value,
) -> Result<ProofExecutionObservation> {
    let report = validate_executor_artifacts(summary, manifest, VerificationMode::Execution)?;
    let entries = expect_array(
        field(summary, "entries", "executor summary")?,
        "entries",
        "executor summary",
    )?;
    let mut scopes = entries
        .iter()
        .map(observation_scope)
        .collect::<Result<Vec<_>>>()?;
    scopes.sort_by(|left, right| {
        (&left.name, &left.kind, &left.command).cmp(&(&right.name, &right.kind, &right.command))
    });
    let artifacts = scopes
        .iter()
        .filter(|scope| scope.artifact_path.is_some())
        .count();

    Ok(ProofExecutionObservation {
        schema: OBSERVATION_SCHEMA.to_string(),
        status: expect_string(
            field(summary, "status", "executor summary")?,
            "status",
            "executor summary",
        )?,
        execution_status: report.execution_status,
        profile: expect_string(
            field(summary, "profile", "executor summary")?,
            "profile",
            "executor summary",
        )?,
        base: expect_string(
            field(summary, "base", "executor summary")?,
            "base",
            "executor summary",
        )?,
        head: expect_string(
            field(summary, "head", "executor summary")?,
            "head",
            "executor summary",
        )?,
        family: expect_string(
            field(summary, "family", "executor summary")?,
            "family",
            "executor summary",
        )?,
        required: expect_bool(
            field(summary, "required", "executor summary")?,
            "required",
            "executor summary",
        )?,
        ok: expect_bool(
            field(summary, "ok", "executor summary")?,
            "ok",
            "executor summary",
        )?,
        execution_guard: ProofExecutionObservationGuard {
            enabled: expect_bool(
                field(summary, "execution_guard.enabled", "executor summary")?,
                "execution_guard.enabled",
                "executor summary",
            )?,
            ci: expect_bool(
                field(summary, "execution_guard.ci", "executor summary")?,
                "execution_guard.ci",
                "executor summary",
            )?,
            reason: report.guard_reason,
        },
        counts: ProofExecutionObservationCounts {
            selected: report.selected,
            executed: report.executed,
            passed: expect_usize(
                field(summary, "counts.passed", "executor summary")?,
                "counts.passed",
                "executor summary",
            )?,
            failed: expect_usize(
                field(summary, "counts.failed", "executor summary")?,
                "counts.failed",
                "executor summary",
            )?,
            artifacts,
        },
        scopes,
        changed_files: expect_string_array(
            field(summary, "changed_files", "executor summary")?,
            "changed_files",
            "executor summary",
        )?,
        unknown_files: expect_string_array(
            field(summary, "unknown_files", "executor summary")?,
            "unknown_files",
            "executor summary",
        )?,
    })
}

fn proof_execution_observation_collection(
    paths: &[PathBuf],
) -> Result<ProofExecutionObservationCollection> {
    if paths.is_empty() {
        bail!("at least one --observation path is required");
    }

    let observations = paths
        .iter()
        .map(|path| read_sourced_observation(path))
        .collect::<Result<Vec<_>>>()?;

    Ok(summarize_observations(&observations))
}

fn validate_observation_collection_thresholds(
    collection: &ProofExecutionObservationCollection,
    args: &ProofExecutionObservationsSummaryArgs,
) -> Result<()> {
    validate_minimum(
        "--min-observations",
        "observation(s)",
        collection.counts.observations,
        args.min_observations,
    )?;
    validate_minimum(
        "--min-executed",
        "executed command(s)",
        collection.counts.executed,
        args.min_executed,
    )?;
    validate_minimum(
        "--min-scopes",
        "scope(s)",
        collection.scopes.len(),
        args.min_scopes,
    )?;
    validate_minimum(
        "--min-artifacts",
        "artifact(s)",
        collection.counts.artifacts,
        args.min_artifacts,
    )
}

fn validate_minimum(flag: &str, display_label: &str, actual: usize, required: usize) -> Result<()> {
    if actual < required {
        bail!(
            "proof executor observation collection has {actual} {display_label}, below {flag} {required}"
        );
    }

    Ok(())
}

fn collect_observation_paths(args: &ProofExecutionObservationsSummaryArgs) -> Result<Vec<PathBuf>> {
    let mut paths = BTreeSet::new();

    if args.observations.is_empty() && args.observation_dirs.is_empty() {
        paths.insert(PathBuf::from(
            "target/proof/proof-executor-observation.json",
        ));
    }

    paths.extend(args.observations.iter().cloned());
    for dir in &args.observation_dirs {
        collect_observation_paths_from_dir(dir, &mut paths)?;
    }

    if paths.is_empty() {
        bail!("no proof executor observation artifacts found");
    }

    Ok(paths.into_iter().collect())
}

fn collect_observation_paths_from_dir(dir: &Path, paths: &mut BTreeSet<PathBuf>) -> Result<()> {
    if !dir.is_dir() {
        bail!(
            "observation directory `{}` is not a directory",
            dir.display()
        );
    }

    for entry in WalkDir::new(dir) {
        let entry = entry
            .with_context(|| format!("failed to scan observation directory `{}`", dir.display()))?;
        if entry.file_type().is_file()
            && entry.file_name().to_string_lossy() == "proof-executor-observation.json"
        {
            paths.insert(entry.path().to_path_buf());
        }
    }

    Ok(())
}

fn read_sourced_observation(path: &Path) -> Result<SourcedProofExecutionObservation> {
    let value = read_json(path, "proof executor observation")?;
    let observation = read_observation_value(&value)
        .with_context(|| format!("invalid proof executor observation `{}`", path.display()))?;
    Ok(SourcedProofExecutionObservation {
        path: path.to_path_buf(),
        observation,
    })
}

fn read_observation_value(value: &Value) -> Result<ProofExecutionObservation> {
    let observation: ProofExecutionObservation = serde_json::from_value(value.clone())
        .context("proof executor observation shape is invalid")?;
    validate_observation(&observation)?;
    Ok(observation)
}

fn validate_observation(observation: &ProofExecutionObservation) -> Result<()> {
    if observation.schema != OBSERVATION_SCHEMA {
        bail!(
            "proof executor observation schema must be `{OBSERVATION_SCHEMA}`, got `{}`",
            observation.schema
        );
    }
    if observation.status != "passed" {
        bail!(
            "proof executor observation status must be `passed`, got `{}`",
            observation.status
        );
    }
    if observation.execution_status != "executed" {
        bail!(
            "proof executor observation execution_status must be `executed`, got `{}`",
            observation.execution_status
        );
    }
    if !observation.ok {
        bail!("proof executor observation must have ok=true");
    }
    if observation.required {
        bail!("proof executor observation collection only accepts non-required executor evidence");
    }
    if !observation.execution_guard.enabled {
        bail!("proof executor observation guard must be enabled");
    }
    if observation.counts.failed != 0 {
        bail!(
            "proof executor observation reports {} failed command(s)",
            observation.counts.failed
        );
    }
    if observation.counts.selected != observation.counts.executed {
        bail!(
            "proof executor observation selected/executed drift: {} selected != {} executed",
            observation.counts.selected,
            observation.counts.executed
        );
    }
    if observation.counts.executed != observation.counts.passed {
        bail!(
            "proof executor observation executed/passed drift: {} executed != {} passed",
            observation.counts.executed,
            observation.counts.passed
        );
    }
    if observation.scopes.len() != observation.counts.selected {
        bail!(
            "proof executor observation has {} scope row(s) for {} selected command(s)",
            observation.scopes.len(),
            observation.counts.selected
        );
    }
    let artifact_count = observation
        .scopes
        .iter()
        .filter(|scope| scope.artifact_path.is_some())
        .count();
    if artifact_count != observation.counts.artifacts {
        bail!(
            "proof executor observation artifact count drift: {} scope artifact(s) != {} counted artifact(s)",
            artifact_count,
            observation.counts.artifacts
        );
    }
    if !observation.unknown_files.is_empty() {
        bail!(
            "proof executor observation reports {} unknown file(s)",
            observation.unknown_files.len()
        );
    }
    for scope in &observation.scopes {
        if scope.status != "passed" {
            bail!(
                "proof executor observation scope `{}` status must be `passed`, got `{}`",
                scope.name,
                scope.status
            );
        }
        if scope.exit_code != Some(0) {
            bail!(
                "proof executor observation scope `{}` exit_code must be 0, got {:?}",
                scope.name,
                scope.exit_code
            );
        }
    }

    Ok(())
}

fn summarize_observations(
    observations: &[SourcedProofExecutionObservation],
) -> ProofExecutionObservationCollection {
    let mut counts = ProofExecutionObservationCollectionCounts {
        observations: observations.len(),
        ..ProofExecutionObservationCollectionCounts::default()
    };
    let mut families = BTreeMap::<String, FamilyAccumulator>::new();
    let mut scopes = BTreeMap::<(String, String, String), ScopeAccumulator>::new();
    let mut sources = Vec::new();

    for sourced in observations {
        let observation = &sourced.observation;
        counts.selected += observation.counts.selected;
        counts.executed += observation.counts.executed;
        counts.passed += observation.counts.passed;
        counts.failed += observation.counts.failed;
        counts.artifacts += observation.counts.artifacts;

        let family = families.entry(observation.family.clone()).or_default();
        family.observations += 1;
        family.selected += observation.counts.selected;
        family.executed += observation.counts.executed;
        family.passed += observation.counts.passed;
        family.artifacts += observation.counts.artifacts;

        for scope in &observation.scopes {
            let key = (
                scope.name.clone(),
                scope.kind.clone(),
                observation.family.clone(),
            );
            let entry = scopes.entry(key).or_default();
            entry.observations += 1;
            entry.executed += 1;
            if scope.artifact_path.is_some() {
                entry.artifacts += 1;
            }
        }

        sources.push(ProofExecutionObservationSourceSummary {
            path: normalize_path(&sourced.path),
            status: observation.status.clone(),
            execution_status: observation.execution_status.clone(),
            profile: observation.profile.clone(),
            base: observation.base.clone(),
            head: observation.head.clone(),
            family: observation.family.clone(),
            guard_reason: observation.execution_guard.reason.clone(),
            selected: observation.counts.selected,
            executed: observation.counts.executed,
            passed: observation.counts.passed,
            artifacts: observation.counts.artifacts,
        });
    }

    sources.sort_by(|left, right| left.path.cmp(&right.path));

    ProofExecutionObservationCollection {
        schema: OBSERVATION_COLLECTION_SCHEMA.to_string(),
        ok: true,
        counts,
        families: families
            .into_iter()
            .map(|(family, entry)| ProofExecutionObservationFamilySummary {
                family,
                observations: entry.observations,
                selected: entry.selected,
                executed: entry.executed,
                passed: entry.passed,
                artifacts: entry.artifacts,
            })
            .collect(),
        scopes: scopes
            .into_iter()
            .map(
                |((name, kind, family), entry)| ProofExecutionObservationScopeSummary {
                    name,
                    kind,
                    family,
                    observations: entry.observations,
                    executed: entry.executed,
                    artifacts: entry.artifacts,
                },
            )
            .collect(),
        sources,
    }
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn observation_scope(entry: &Value) -> Result<ProofExecutionObservationScope> {
    Ok(ProofExecutionObservationScope {
        name: expect_string(
            field(entry, "scope", "executor summary entry")?,
            "scope",
            "executor summary entry",
        )?,
        kind: expect_string(
            field(entry, "kind", "executor summary entry")?,
            "kind",
            "executor summary entry",
        )?,
        command: expect_string(
            field(entry, "command", "executor summary entry")?,
            "command",
            "executor summary entry",
        )?,
        artifact_path: expect_optional_string(
            field(entry, "artifact_path", "executor summary entry")?,
            "artifact_path",
            "executor summary entry",
        )?,
        status: expect_string(
            field(entry, "status", "executor summary entry")?,
            "status",
            "executor summary entry",
        )?,
        exit_code: expect_optional_i64(
            field(entry, "exit_code", "executor summary entry")?,
            "exit_code",
            "executor summary entry",
        )?,
    })
}

fn validate_executor_artifacts(
    summary: &Value,
    manifest: &Value,
    mode: VerificationMode,
) -> Result<ProofArtifactsReport> {
    expect_schema(summary, SUMMARY_SCHEMA, "executor summary")?;
    expect_schema(manifest, MANIFEST_SCHEMA, "executor manifest")?;

    for field in SHARED_FIELDS {
        expect_equal(summary, manifest, field)?;
    }

    let execution_status = expect_string(
        field(summary, "execution_status", "executor summary")?,
        "execution_status",
        "executor summary",
    )?;
    let guard_enabled = expect_bool(
        field(summary, "execution_guard.enabled", "executor summary")?,
        "execution_guard.enabled",
        "executor summary",
    )?;

    let summary_selected = expect_usize(
        field(summary, "counts.selected", "executor summary")?,
        "counts.selected",
        "executor summary",
    )?;
    let summary_executed = expect_usize(
        field(summary, "counts.executed", "executor summary")?,
        "counts.executed",
        "executor summary",
    )?;
    let manifest_selected = expect_usize(
        field(manifest, "selection.selected", "executor manifest")?,
        "selection.selected",
        "executor manifest",
    )?;
    let manifest_executed = expect_usize(
        field(manifest, "selection.executed", "executor manifest")?,
        "selection.executed",
        "executor manifest",
    )?;

    if summary_selected != manifest_selected {
        bail!(
            "executor artifact mismatch at selected count: summary {summary_selected} != manifest {manifest_selected}"
        );
    }
    if summary_executed != manifest_executed {
        bail!(
            "executor artifact mismatch at executed count: summary {summary_executed} != manifest {manifest_executed}"
        );
    }

    expect_string_value(
        field(manifest, "selection.source", "executor manifest")?,
        "proof_plan",
        "selection.source",
        "executor manifest",
    )?;
    expect_bool_value(
        field(manifest, "selection.required_included", "executor manifest")?,
        false,
        "selection.required_included",
        "executor manifest",
    )?;

    let entries = expect_array(
        field(summary, "entries", "executor summary")?,
        "entries",
        "executor summary",
    )?;
    let commands = expect_array(
        field(manifest, "commands", "executor manifest")?,
        "commands",
        "executor manifest",
    )?;
    if entries.len() != summary_selected {
        bail!(
            "executor summary entries length {} does not match selected count {summary_selected}",
            entries.len()
        );
    }
    if commands.len() != manifest_selected {
        bail!(
            "executor manifest commands length {} does not match selected count {manifest_selected}",
            commands.len()
        );
    }

    for (index, (entry, command)) in entries.iter().zip(commands.iter()).enumerate() {
        validate_command_entry(index, entry, command)?;
    }

    validate_execution_state(
        summary,
        entries,
        mode,
        &execution_status,
        guard_enabled,
        summary_selected,
        summary_executed,
    )?;

    let guard_reason = expect_string(
        field(summary, "execution_guard.reason", "executor summary")?,
        "execution_guard.reason",
        "executor summary",
    )?;

    Ok(ProofArtifactsReport {
        selected: summary_selected,
        executed: summary_executed,
        execution_status,
        guard_reason,
    })
}

fn validate_execution_state(
    summary: &Value,
    entries: &[Value],
    mode: VerificationMode,
    execution_status: &str,
    guard_enabled: bool,
    selected: usize,
    executed: usize,
) -> Result<()> {
    match mode {
        VerificationMode::NoExecution => {
            if execution_status == "executed" {
                bail!(
                    "executor artifacts report executed commands; use proof-execution-artifacts-check for executed artifacts"
                );
            }
            if executed != 0 {
                bail!(
                    "executor artifacts report {executed} executed command(s); no-execution verifier requires zero"
                );
            }
        }
        VerificationMode::Execution => {
            expect_string_value(
                field(summary, "mode", "executor summary")?,
                "execute",
                "mode",
                "executor summary",
            )?;
            if execution_status != "executed" {
                bail!(
                    "executor artifacts have execution_status `{execution_status}`; execution verifier requires `executed`"
                );
            }
            if !guard_enabled {
                bail!(
                    "executor artifacts have execution_guard.enabled=false; execution verifier requires explicit opt-in"
                );
            }

            let failed = expect_usize(
                field(summary, "counts.failed", "executor summary")?,
                "counts.failed",
                "executor summary",
            )?;
            if failed != 0 {
                bail!(
                    "executor artifacts report {failed} failed command(s); execution verifier requires zero"
                );
            }

            let skipped = expect_usize(
                field(summary, "counts.skipped", "executor summary")?,
                "counts.skipped",
                "executor summary",
            )?;
            let dry_run = expect_usize(
                field(summary, "counts.dry_run", "executor summary")?,
                "counts.dry_run",
                "executor summary",
            )?;
            let passed = expect_usize(
                field(summary, "counts.passed", "executor summary")?,
                "counts.passed",
                "executor summary",
            )?;
            if skipped != 0 || dry_run != 0 {
                bail!(
                    "executor artifacts report skipped={skipped} dry_run={dry_run}; execution verifier requires executed commands only"
                );
            }
            if executed != selected {
                bail!(
                    "executor artifacts report {executed} executed command(s) for {selected} selected command(s)"
                );
            }
            if passed != selected {
                bail!(
                    "executor artifacts report {passed} passed command(s) for {selected} selected command(s)"
                );
            }
            expect_string_value(
                field(summary, "status", "executor summary")?,
                "passed",
                "status",
                "executor summary",
            )?;

            for (index, entry) in entries.iter().enumerate() {
                validate_executed_entry(index, entry)?;
                validate_executed_artifact_path(index, entry)?;
            }
        }
    }
    Ok(())
}

fn validate_executed_entry(index: usize, entry: &Value) -> Result<()> {
    let expected_index = index + 1;
    expect_string_value(
        field(entry, "status", "executor summary entry")?,
        "passed",
        "status",
        "executor summary entry",
    )
    .with_context(|| format!("executor summary entry {expected_index} failed status check"))?;
    expect_string_value(
        field(entry, "skip_reason", "executor summary entry")?,
        "",
        "skip_reason",
        "executor summary entry",
    )
    .with_context(|| format!("executor summary entry {expected_index} failed skip check"))?;

    let exit_code = field(entry, "exit_code", "executor summary entry")?;
    if exit_code.as_i64() != Some(0) {
        bail!(
            "executor summary entry {expected_index} exit_code must be 0 for passed execution, got {}",
            render_json(exit_code)
        );
    }
    Ok(())
}

fn validate_executed_artifact_path(index: usize, entry: &Value) -> Result<()> {
    let expected_index = index + 1;
    let kind = expect_string(
        field(entry, "kind", "executor summary entry")?,
        "kind",
        "executor summary entry",
    )?;
    let artifact_path = field(entry, "artifact_path", "executor summary entry")?;
    if artifact_path.is_null() {
        return Ok(());
    }

    let artifact_path = expect_string(artifact_path, "artifact_path", "executor summary entry")?;
    if artifact_path.trim().is_empty() {
        bail!("executor summary entry {expected_index} artifact_path must not be empty");
    }

    let metadata = fs::metadata(&artifact_path).with_context(|| {
        format!("executor summary entry {expected_index} artifact `{artifact_path}` was not found")
    })?;
    if !metadata.is_file() {
        bail!("executor summary entry {expected_index} artifact `{artifact_path}` is not a file");
    }
    if metadata.len() == 0 {
        bail!("executor summary entry {expected_index} artifact `{artifact_path}` is empty");
    }

    if kind == "coverage" {
        validate_lcov_artifact(expected_index, &artifact_path)?;
    }

    Ok(())
}

fn validate_lcov_artifact(index: usize, artifact_path: &str) -> Result<()> {
    let raw = fs::read_to_string(artifact_path).with_context(|| {
        format!(
            "executor summary entry {index} LCOV artifact `{artifact_path}` is not readable text"
        )
    })?;

    if !raw.lines().any(|line| line.starts_with("SF:")) {
        bail!("executor summary entry {index} LCOV artifact `{artifact_path}` has no `SF:` record");
    }
    if !raw.lines().any(|line| line == "end_of_record") {
        bail!(
            "executor summary entry {index} LCOV artifact `{artifact_path}` has no `end_of_record`"
        );
    }

    Ok(())
}

fn validate_command_entry(index: usize, entry: &Value, command: &Value) -> Result<()> {
    let expected_index = index + 1;
    let manifest_index = expect_usize(
        field(command, "index", "executor manifest command")?,
        "index",
        "executor manifest command",
    )?;
    if manifest_index != expected_index {
        bail!(
            "executor manifest command index mismatch at position {expected_index}: got {manifest_index}"
        );
    }

    let id = expect_string(
        field(command, "id", "executor manifest command")?,
        "id",
        "executor manifest command",
    )?;
    let expected_prefix = format!("{expected_index:04}-");
    if !id.starts_with(&expected_prefix) {
        bail!("executor manifest command id `{id}` does not start with `{expected_prefix}`");
    }

    for field_name in ENTRY_FIELDS {
        let entry_value = field(entry, field_name, "executor summary entry")?;
        let command_value = field(command, field_name, "executor manifest command")?;
        if entry_value != command_value {
            bail!(
                "executor command mismatch at `{field_name}` for command {expected_index}: summary {} != manifest {}",
                render_json(entry_value),
                render_json(command_value)
            );
        }
    }
    Ok(())
}

fn expect_schema(value: &Value, expected: &str, label: &str) -> Result<()> {
    expect_string_value(field(value, "schema", label)?, expected, "schema", label)
}

fn expect_equal(summary: &Value, manifest: &Value, path: &str) -> Result<()> {
    let summary_value = field(summary, path, "executor summary")?;
    let manifest_value = field(manifest, path, "executor manifest")?;
    if summary_value != manifest_value {
        bail!(
            "executor artifact mismatch at `{path}`: summary {} != manifest {}",
            render_json(summary_value),
            render_json(manifest_value)
        );
    }
    Ok(())
}

fn field<'a>(value: &'a Value, path: &str, label: &str) -> Result<&'a Value> {
    let mut current = value;
    for segment in path.split('.') {
        current = current
            .get(segment)
            .with_context(|| format!("{label} artifact is missing `{path}`"))?;
    }
    Ok(current)
}

fn expect_array<'a>(value: &'a Value, path: &str, label: &str) -> Result<&'a Vec<Value>> {
    value
        .as_array()
        .with_context(|| format!("{label} `{path}` must be an array"))
}

fn expect_bool(value: &Value, path: &str, label: &str) -> Result<bool> {
    value
        .as_bool()
        .with_context(|| format!("{label} `{path}` must be a boolean"))
}

fn expect_bool_value(value: &Value, expected: bool, path: &str, label: &str) -> Result<()> {
    let actual = expect_bool(value, path, label)?;
    if actual != expected {
        bail!("{label} `{path}` must be {expected}, got {actual}");
    }
    Ok(())
}

fn expect_string(value: &Value, path: &str, label: &str) -> Result<String> {
    value
        .as_str()
        .map(ToOwned::to_owned)
        .with_context(|| format!("{label} `{path}` must be a string"))
}

fn expect_optional_string(value: &Value, path: &str, label: &str) -> Result<Option<String>> {
    if value.is_null() {
        Ok(None)
    } else {
        expect_string(value, path, label).map(Some)
    }
}

fn expect_string_value(value: &Value, expected: &str, path: &str, label: &str) -> Result<()> {
    let actual = expect_string(value, path, label)?;
    if actual != expected {
        bail!("{label} `{path}` must be `{expected}`, got `{actual}`");
    }
    Ok(())
}

fn expect_string_array(value: &Value, path: &str, label: &str) -> Result<Vec<String>> {
    let values = expect_array(value, path, label)?;
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            expect_string(value, &format!("{path}[{index}]"), label)
                .with_context(|| format!("{label} `{path}` entry {index} must be a string"))
        })
        .collect()
}

fn expect_optional_i64(value: &Value, path: &str, label: &str) -> Result<Option<i64>> {
    if value.is_null() {
        return Ok(None);
    }
    value
        .as_i64()
        .map(Some)
        .with_context(|| format!("{label} `{path}` must be an integer or null"))
}

fn expect_usize(value: &Value, path: &str, label: &str) -> Result<usize> {
    let number = value
        .as_u64()
        .with_context(|| format!("{label} `{path}` must be a non-negative integer"))?;
    usize::try_from(number).with_context(|| format!("{label} `{path}` is too large"))
}

fn render_json(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "<unrenderable>".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_ARTIFACT_COUNTER: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn accepts_matching_no_execution_artifacts() {
        let (summary, manifest) = matching_artifacts();

        let report =
            validate_executor_artifacts(&summary, &manifest, VerificationMode::NoExecution)
                .unwrap();

        assert_eq!(
            report,
            ProofArtifactsReport {
                selected: 1,
                executed: 0,
                execution_status: "dry_run".to_string(),
                guard_reason: "local_requires_--allow-local-evidence-execution".to_string(),
            }
        );
    }

    #[test]
    fn rejects_selected_count_drift() {
        let (summary, mut manifest) = matching_artifacts();
        manifest["selection"]["selected"] = json!(2);

        let error = validate_executor_artifacts(&summary, &manifest, VerificationMode::NoExecution)
            .unwrap_err()
            .to_string();

        assert!(error.contains("selected count"));
    }

    #[test]
    fn rejects_command_payload_drift() {
        let (summary, mut manifest) = matching_artifacts();
        manifest["commands"][0]["command"] = json!("cargo llvm-cov -p tokmd-gate");

        let error = validate_executor_artifacts(&summary, &manifest, VerificationMode::NoExecution)
            .unwrap_err()
            .to_string();

        assert!(error.contains("executor command mismatch"));
    }

    #[test]
    fn accepts_enabled_execution_guard_when_no_commands_executed() {
        let (mut summary, mut manifest) = matching_artifacts();
        summary["execution_guard"]["enabled"] = json!(true);
        manifest["execution_guard"]["enabled"] = json!(true);
        summary["execution_guard"]["ci"] = json!(true);
        manifest["execution_guard"]["ci"] = json!(true);
        summary["execution_guard"]["allow_ci_evidence_execution"] = json!(true);
        manifest["execution_guard"]["allow_ci_evidence_execution"] = json!(true);
        summary["execution_guard"]["reason"] = json!("ci_explicit_opt_in_enabled");
        manifest["execution_guard"]["reason"] = json!("ci_explicit_opt_in_enabled");

        let report =
            validate_executor_artifacts(&summary, &manifest, VerificationMode::NoExecution)
                .unwrap();

        assert_eq!(
            report,
            ProofArtifactsReport {
                selected: 1,
                executed: 0,
                execution_status: "dry_run".to_string(),
                guard_reason: "ci_explicit_opt_in_enabled".to_string(),
            }
        );
    }

    #[test]
    fn rejects_executed_artifacts_even_when_guard_enabled() {
        let (mut summary, mut manifest) = matching_artifacts();
        summary["execution_status"] = json!("executed");
        manifest["execution_status"] = json!("executed");
        summary["execution_guard"]["enabled"] = json!(true);
        manifest["execution_guard"]["enabled"] = json!(true);
        summary["counts"]["executed"] = json!(1);
        manifest["selection"]["executed"] = json!(1);

        let error = validate_executor_artifacts(&summary, &manifest, VerificationMode::NoExecution)
            .unwrap_err()
            .to_string();

        assert!(error.contains("executed commands"));
    }

    #[test]
    fn accepts_matching_executed_artifacts() {
        let (summary, manifest) = executed_artifacts();

        let report =
            validate_executor_artifacts(&summary, &manifest, VerificationMode::Execution).unwrap();

        assert_eq!(
            report,
            ProofArtifactsReport {
                selected: 1,
                executed: 1,
                execution_status: "executed".to_string(),
                guard_reason: "local_explicit_opt_in_enabled".to_string(),
            }
        );
    }

    #[test]
    fn builds_compact_observation_for_executed_artifacts() {
        let (summary, manifest) = executed_artifacts();

        let observation = proof_execution_observation(&summary, &manifest).unwrap();

        assert_eq!(observation.schema, OBSERVATION_SCHEMA);
        assert_eq!(observation.status, "passed");
        assert_eq!(observation.execution_status, "executed");
        assert_eq!(observation.family, "coverage");
        assert_eq!(
            observation.execution_guard.reason,
            "local_explicit_opt_in_enabled"
        );
        assert_eq!(
            observation.counts,
            ProofExecutionObservationCounts {
                selected: 1,
                executed: 1,
                passed: 1,
                failed: 0,
                artifacts: 1,
            }
        );
        assert_eq!(observation.changed_files, ["crates/tokmd-core/src/ffi.rs"]);
        assert!(observation.unknown_files.is_empty());
        assert_eq!(observation.scopes.len(), 1);
        assert_eq!(observation.scopes[0].name, "tokmd_core_ffi");
        assert_eq!(observation.scopes[0].kind, "coverage");
        assert_eq!(observation.scopes[0].status, "passed");
        assert_eq!(observation.scopes[0].exit_code, Some(0));
        assert!(
            observation.scopes[0]
                .artifact_path
                .as_ref()
                .is_some_and(|path| path.contains("tokmd-proof-artifact"))
        );
    }

    #[test]
    fn summarizes_successful_observations_by_family_and_scope() {
        let (summary, manifest) = executed_artifacts();
        let first = proof_execution_observation(&summary, &manifest).unwrap();
        let mut second = first.clone();
        second.scopes[0].name = "analysis_derived".to_string();
        second.changed_files = vec!["crates/tokmd-analysis/src/derived/mod.rs".to_string()];

        let collection = summarize_observations(&[
            sourced("target/proof/run-b/proof-executor-observation.json", second),
            sourced("target/proof/run-a/proof-executor-observation.json", first),
        ]);

        assert_eq!(collection.schema, OBSERVATION_COLLECTION_SCHEMA);
        assert!(collection.ok);
        assert_eq!(
            collection.counts,
            ProofExecutionObservationCollectionCounts {
                observations: 2,
                selected: 2,
                executed: 2,
                passed: 2,
                failed: 0,
                artifacts: 2,
            }
        );
        assert_eq!(collection.families.len(), 1);
        assert_eq!(collection.families[0].family, "coverage");
        assert_eq!(collection.families[0].observations, 2);
        assert_eq!(
            collection
                .scopes
                .iter()
                .map(|scope| scope.name.as_str())
                .collect::<Vec<_>>(),
            ["analysis_derived", "tokmd_core_ffi"]
        );
        assert_eq!(
            collection
                .sources
                .iter()
                .map(|source| source.path.as_str())
                .collect::<Vec<_>>(),
            [
                "target/proof/run-a/proof-executor-observation.json",
                "target/proof/run-b/proof-executor-observation.json",
            ]
        );
    }

    #[test]
    fn validates_observation_collection_thresholds() {
        let (summary, manifest) = executed_artifacts();
        let first = proof_execution_observation(&summary, &manifest).unwrap();
        let mut second = first.clone();
        second.scopes[0].name = "analysis_derived".to_string();

        let collection = summarize_observations(&[
            sourced("target/proof/run-a/proof-executor-observation.json", first),
            sourced("target/proof/run-b/proof-executor-observation.json", second),
        ]);
        let args = summary_args_with_thresholds(2, 2, 2, 2);

        validate_observation_collection_thresholds(&collection, &args).unwrap();
    }

    #[test]
    fn rejects_observation_collection_below_thresholds() {
        let (summary, manifest) = executed_artifacts();
        let observation = proof_execution_observation(&summary, &manifest).unwrap();
        let collection = summarize_observations(&[sourced(
            "target/proof/run-a/proof-executor-observation.json",
            observation,
        )]);
        let args = summary_args_with_thresholds(2, 1, 1, 1);

        let error = validate_observation_collection_thresholds(&collection, &args)
            .unwrap_err()
            .to_string();

        assert!(error.contains("--min-observations 2"));
    }

    #[test]
    fn rejects_failed_observation_for_collection() {
        let (summary, manifest) = executed_artifacts();
        let mut observation = proof_execution_observation(&summary, &manifest).unwrap();
        observation.status = "failed".to_string();

        let value = serde_json::to_value(observation).unwrap();
        let error = read_observation_value(&value).unwrap_err().to_string();

        assert!(error.contains("status must be `passed`"));
    }

    #[test]
    fn rejects_observation_count_drift_for_collection() {
        let (summary, manifest) = executed_artifacts();
        let mut observation = proof_execution_observation(&summary, &manifest).unwrap();
        observation.counts.executed = 0;

        let value = serde_json::to_value(observation).unwrap();
        let error = read_observation_value(&value).unwrap_err().to_string();

        assert!(error.contains("selected/executed drift"));
    }

    #[test]
    fn rejects_execution_artifacts_without_enabled_guard() {
        let (mut summary, mut manifest) = executed_artifacts();
        summary["execution_guard"]["enabled"] = json!(false);
        manifest["execution_guard"]["enabled"] = json!(false);

        let error = validate_executor_artifacts(&summary, &manifest, VerificationMode::Execution)
            .unwrap_err()
            .to_string();

        assert!(error.contains("execution_guard.enabled=false"));
    }

    #[test]
    fn rejects_execution_artifacts_with_failed_commands() {
        let (mut summary, mut manifest) = executed_artifacts();
        summary["status"] = json!("failed");
        manifest["status"] = json!("failed");
        summary["counts"]["passed"] = json!(0);
        summary["counts"]["failed"] = json!(1);
        summary["entries"][0]["status"] = json!("failed");
        manifest["commands"][0]["status"] = json!("failed");
        summary["entries"][0]["skip_reason"] = json!("command_failed");
        manifest["commands"][0]["skip_reason"] = json!("command_failed");
        summary["entries"][0]["exit_code"] = json!(1);
        manifest["commands"][0]["exit_code"] = json!(1);

        let error = validate_executor_artifacts(&summary, &manifest, VerificationMode::Execution)
            .unwrap_err()
            .to_string();

        assert!(error.contains("failed command"));
    }

    #[test]
    fn rejects_execution_artifacts_with_missing_output_file() {
        let (mut summary, mut manifest) = executed_artifacts();
        let missing = std::env::temp_dir()
            .join(format!(
                "tokmd-missing-proof-artifact-{}.lcov",
                std::process::id()
            ))
            .to_string_lossy()
            .to_string();
        let _ = fs::remove_file(&missing);
        summary["entries"][0]["artifact_path"] = json!(missing);
        manifest["commands"][0]["artifact_path"] = json!(missing);

        let error = validate_executor_artifacts(&summary, &manifest, VerificationMode::Execution)
            .unwrap_err()
            .to_string();

        assert!(error.contains("was not found"));
    }

    #[test]
    fn rejects_execution_artifacts_with_malformed_lcov_output() {
        let (mut summary, mut manifest) = executed_artifacts();
        let malformed = write_test_artifact(
            "tokmd-malformed-proof-artifact",
            "this is not an LCOV payload\n",
        );
        summary["entries"][0]["artifact_path"] = json!(malformed);
        manifest["commands"][0]["artifact_path"] = json!(malformed);

        let error = validate_executor_artifacts(&summary, &manifest, VerificationMode::Execution)
            .unwrap_err()
            .to_string();

        assert!(error.contains("SF:"));
    }

    #[test]
    fn rejects_dry_run_artifacts_with_execution_verifier() {
        let (summary, manifest) = matching_artifacts();

        let error = validate_executor_artifacts(&summary, &manifest, VerificationMode::Execution)
            .unwrap_err()
            .to_string();

        assert!(error.contains("`mode` must be `execute`"));
    }

    #[test]
    fn rejects_dry_run_artifacts_for_observation() {
        let (summary, manifest) = matching_artifacts();

        let error = proof_execution_observation(&summary, &manifest)
            .unwrap_err()
            .to_string();

        assert!(error.contains("`mode` must be `execute`"));
    }

    fn executed_artifacts() -> (Value, Value) {
        let (mut summary, mut manifest) = matching_artifacts();
        let artifact_path = write_test_lcov_artifact();
        summary["mode"] = json!("execute");
        manifest["mode"] = json!("execute");
        summary["status"] = json!("passed");
        manifest["status"] = json!("passed");
        summary["execution_status"] = json!("executed");
        manifest["execution_status"] = json!("executed");
        summary["execution_guard"]["enabled"] = json!(true);
        manifest["execution_guard"]["enabled"] = json!(true);
        summary["execution_guard"]["allow_local_evidence_execution"] = json!(true);
        manifest["execution_guard"]["allow_local_evidence_execution"] = json!(true);
        summary["execution_guard"]["reason"] = json!("local_explicit_opt_in_enabled");
        manifest["execution_guard"]["reason"] = json!("local_explicit_opt_in_enabled");
        summary["counts"]["dry_run"] = json!(0);
        summary["counts"]["executed"] = json!(1);
        summary["counts"]["passed"] = json!(1);
        summary["counts"]["failed"] = json!(0);
        manifest["selection"]["executed"] = json!(1);
        summary["entries"][0]["status"] = json!("passed");
        manifest["commands"][0]["status"] = json!("passed");
        summary["entries"][0]["skip_reason"] = json!("");
        manifest["commands"][0]["skip_reason"] = json!("");
        summary["entries"][0]["exit_code"] = json!(0);
        manifest["commands"][0]["exit_code"] = json!(0);
        summary["entries"][0]["artifact_path"] = json!(artifact_path);
        manifest["commands"][0]["artifact_path"] = json!(artifact_path);
        (summary, manifest)
    }

    fn write_test_lcov_artifact() -> String {
        write_test_artifact(
            "tokmd-proof-artifact",
            "TN:\nSF:crates/tokmd-core/src/ffi.rs\nend_of_record\n",
        )
    }

    fn write_test_artifact(name: &str, content: &str) -> String {
        let index = TEST_ARTIFACT_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("{name}-{}-{index}.lcov", std::process::id()));
        fs::write(&path, content).expect("test artifact should be writable");
        path.to_string_lossy().to_string()
    }

    fn sourced(
        path: &str,
        observation: ProofExecutionObservation,
    ) -> SourcedProofExecutionObservation {
        SourcedProofExecutionObservation {
            path: PathBuf::from(path),
            observation,
        }
    }

    fn summary_args_with_thresholds(
        min_observations: usize,
        min_executed: usize,
        min_scopes: usize,
        min_artifacts: usize,
    ) -> ProofExecutionObservationsSummaryArgs {
        ProofExecutionObservationsSummaryArgs {
            observations: Vec::new(),
            observation_dirs: Vec::new(),
            min_observations,
            min_executed,
            min_scopes,
            min_artifacts,
            output: None,
        }
    }

    fn matching_artifacts() -> (Value, Value) {
        let guard = json!({
            "required": true,
            "enabled": false,
            "ci": false,
            "ci_execution": "explicit_opt_in",
            "allow_ci_evidence_execution": false,
            "reason": "local_requires_--allow-local-evidence-execution",
            "allow_local_evidence_execution": false
        });
        let entry = json!({
            "scope": "tokmd_core_ffi",
            "kind": "coverage",
            "required": false,
            "command": "cargo llvm-cov -p tokmd-core --lcov --output-path target/proof/coverage/tokmd_core_ffi.lcov",
            "artifact_path": "target/proof/coverage/tokmd_core_ffi.lcov",
            "status": "dry_run",
            "skip_reason": "dry_run_only",
            "exit_code": null
        });
        let summary = json!({
            "schema": SUMMARY_SCHEMA,
            "mode": "dry_run",
            "status": "dry_run",
            "execution_status": "dry_run",
            "execution_guard": guard.clone(),
            "family": "coverage",
            "required": false,
            "profile": "affected",
            "base": "origin/main",
            "head": "HEAD",
            "ok": true,
            "changed_files": ["crates/tokmd-core/src/ffi.rs"],
            "counts": {
                "commands_total": 2,
                "family_planned": 1,
                "selected": 1,
                "skipped": 0,
                "dry_run": 1,
                "executed": 0,
                "required_excluded": 0,
                "selection_excluded": 0,
                "non_family_excluded": 1
            },
            "entries": [entry.clone()],
            "unknown_files": []
        });
        let manifest = json!({
            "schema": MANIFEST_SCHEMA,
            "mode": "dry_run",
            "status": "dry_run",
            "execution_status": "dry_run",
            "execution_guard": guard,
            "family": "coverage",
            "required": false,
            "profile": "affected",
            "base": "origin/main",
            "head": "HEAD",
            "ok": true,
            "changed_files": ["crates/tokmd-core/src/ffi.rs"],
            "selection": {
                "source": "proof_plan",
                "max_dry_run_commands": 1,
                "required_included": false,
                "selected": 1,
                "executed": 0
            },
            "commands": [{
                "id": "0001-tokmd_core_ffi-coverage",
                "index": 1,
                "scope": "tokmd_core_ffi",
                "kind": "coverage",
                "required": false,
                "command": "cargo llvm-cov -p tokmd-core --lcov --output-path target/proof/coverage/tokmd_core_ffi.lcov",
                "artifact_path": "target/proof/coverage/tokmd_core_ffi.lcov",
                "status": "dry_run",
                "skip_reason": "dry_run_only",
                "exit_code": null
            }],
            "unknown_files": []
        });
        (summary, manifest)
    }
}
