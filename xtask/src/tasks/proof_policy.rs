use crate::cli::ProofPolicyArgs;
use crate::proof::policy::load_policy;
use crate::proof::policy_ast::{CiExecution, ExecutorPromotionWindow, ProofPolicy};
use crate::proof::validate::{PolicyViolation, validate_policy};
use anyhow::{Result, bail};
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize)]
struct ProofPolicyReport {
    ok: bool,
    policy: String,
    schema: String,
    scope_count: usize,
    allowlist_count: usize,
    fixture_blob_rule_count: usize,
    dependency_boundary_count: usize,
    executor: ExecutorPolicyReport,
    proof_run: ProofRunPolicyReport,
    violations: Vec<PolicyViolation>,
}

#[derive(Debug, Serialize)]
struct ExecutorPolicyReport {
    family: Option<String>,
    ci_execution: Option<String>,
    max_dry_run_commands: Option<usize>,
    pr: Option<ExecutorPrReport>,
    promotion: Option<ExecutorPromotionReport>,
}

#[derive(Debug, Serialize)]
struct ExecutorPrReport {
    default_enabled: Option<bool>,
    required: Option<bool>,
    max_commands: Option<usize>,
    codecov_upload: Option<bool>,
}

#[derive(Debug, Serialize)]
struct ExecutorPromotionReport {
    window: Option<String>,
    run_limit: Option<usize>,
    min_observations: Option<usize>,
    min_executed: Option<usize>,
    min_scopes: Option<usize>,
    min_artifacts: Option<usize>,
    min_passing_collector_runs: Option<usize>,
    required_gate: Option<bool>,
    default_codecov_upload: Option<bool>,
}

#[derive(Debug, Serialize)]
struct ProofRunPolicyReport {
    pr: Option<ProofRunPrReport>,
}

#[derive(Debug, Serialize)]
struct ProofRunPrReport {
    default_enabled: Option<bool>,
    profile: Option<String>,
    required: Option<bool>,
    artifact_name: Option<String>,
}

pub fn run(args: ProofPolicyArgs) -> Result<()> {
    let _check_requested = args.check || !args.json;
    let path = args.policy;
    let policy = load_policy(&path)?;
    let violations = validate_policy(&policy);
    let executor = ExecutorPolicyReport::from_policy(&policy);
    let proof_run = ProofRunPolicyReport::from_policy(&policy);
    let report = ProofPolicyReport {
        ok: violations.is_empty(),
        policy: display_path(&path),
        schema: policy.schema.clone(),
        scope_count: policy.scope.len(),
        allowlist_count: policy.allow.workspace_area.len(),
        fixture_blob_rule_count: policy.forbid.fixture_blob.len(),
        dependency_boundary_count: policy.dependency_boundary.len(),
        executor,
        proof_run,
        violations,
    };

    if let Some(path) = &args.json_output {
        write_json_output(path, &report)?;
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_human_report(&report);
    }

    if report.ok {
        Ok(())
    } else {
        bail!(
            "proof policy validation failed with {} violation(s)",
            report.violations.len()
        )
    }
}

fn write_json_output(path: &Path, report: &ProofPolicyReport) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(report)?)?;
    Ok(())
}

impl ProofRunPolicyReport {
    fn from_policy(policy: &ProofPolicy) -> Self {
        Self {
            pr: policy
                .proof_run
                .pr
                .as_ref()
                .map(ProofRunPrReport::from_policy),
        }
    }
}

impl ProofRunPrReport {
    fn from_policy(pr: &crate::proof::policy_ast::ProofRunPr) -> Self {
        Self {
            default_enabled: pr.default_enabled,
            profile: pr.profile.clone(),
            required: pr.required,
            artifact_name: pr.artifact_name.clone(),
        }
    }
}

impl ExecutorPolicyReport {
    fn from_policy(policy: &ProofPolicy) -> Self {
        Self {
            family: policy.executor.family.clone(),
            ci_execution: policy
                .executor
                .ci_execution
                .as_ref()
                .map(ci_execution_name)
                .map(str::to_string),
            max_dry_run_commands: policy.executor.max_dry_run_commands,
            pr: policy
                .executor
                .pr
                .as_ref()
                .map(ExecutorPrReport::from_policy),
            promotion: policy
                .executor
                .promotion
                .as_ref()
                .map(ExecutorPromotionReport::from_policy),
        }
    }
}

impl ExecutorPrReport {
    fn from_policy(pr: &crate::proof::policy_ast::ExecutorPr) -> Self {
        Self {
            default_enabled: pr.default_enabled,
            required: pr.required,
            max_commands: pr.max_commands,
            codecov_upload: pr.codecov_upload,
        }
    }
}

impl ExecutorPromotionReport {
    fn from_policy(promotion: &crate::proof::policy_ast::ExecutorPromotion) -> Self {
        Self {
            window: promotion
                .window
                .as_ref()
                .map(promotion_window_name)
                .map(str::to_string),
            run_limit: promotion.run_limit,
            min_observations: promotion.min_observations,
            min_executed: promotion.min_executed,
            min_scopes: promotion.min_scopes,
            min_artifacts: promotion.min_artifacts,
            min_passing_collector_runs: promotion.min_passing_collector_runs,
            required_gate: promotion.required_gate,
            default_codecov_upload: promotion.default_codecov_upload,
        }
    }
}

fn print_human_report(report: &ProofPolicyReport) {
    if report.ok {
        println!(
            "Proof policy OK: {} (schema {}, {} scope(s), {} allowlist(s), {} fixture blob rule(s), {} dependency boundary rule(s), executor {}, proof-run {})",
            report.policy,
            report.schema,
            report.scope_count,
            report.allowlist_count,
            report.fixture_blob_rule_count,
            report.dependency_boundary_count,
            executor_summary(&report.executor),
            proof_run_summary(&report.proof_run)
        );
        return;
    }

    eprintln!("Proof policy violations in {}:", report.policy);
    for violation in &report.violations {
        eprintln!("  - {}: {}", violation.path, violation.message);
    }
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn proof_run_summary(proof_run: &ProofRunPolicyReport) -> String {
    match proof_run.pr.as_ref() {
        Some(pr) => format!(
            "pr-default-{}/pr-profile-{}/pr-required-{}/pr-artifact-{}",
            display_optional_bool(pr.default_enabled),
            display_optional_string(pr.profile.as_deref()),
            display_optional_bool(pr.required),
            display_optional_string(pr.artifact_name.as_deref())
        ),
        None => "not-configured".to_string(),
    }
}

fn executor_summary(executor: &ExecutorPolicyReport) -> String {
    let base = match (
        executor.family.as_deref(),
        executor.ci_execution.as_deref(),
        executor.max_dry_run_commands,
    ) {
        (Some(family), Some(ci_execution), Some(max_dry_run_commands)) => {
            format!("{family}/{ci_execution}/max-dry-run-{max_dry_run_commands}")
        }
        _ => "not-configured".to_string(),
    };

    let base = match executor.pr.as_ref() {
        Some(pr) => format!(
            "{base}/pr-default-{}/pr-required-{}/pr-max-commands-{}/pr-codecov-upload-{}",
            display_optional_bool(pr.default_enabled),
            display_optional_bool(pr.required),
            display_optional_usize(pr.max_commands),
            display_optional_bool(pr.codecov_upload)
        ),
        None => base,
    };

    match executor.promotion.as_ref() {
        Some(promotion) => format!(
            "{base}/promotion-window-{}/run-limit-{}/min-observations-{}/min-executed-{}/min-scopes-{}/min-artifacts-{}/min-passing-collector-runs-{}/required-gate-{}/codecov-upload-{}",
            display_optional_string(promotion.window.as_deref()),
            display_optional_usize(promotion.run_limit),
            display_optional_usize(promotion.min_observations),
            display_optional_usize(promotion.min_executed),
            display_optional_usize(promotion.min_scopes),
            display_optional_usize(promotion.min_artifacts),
            display_optional_usize(promotion.min_passing_collector_runs),
            display_optional_bool(promotion.required_gate),
            display_optional_bool(promotion.default_codecov_upload)
        ),
        None => base,
    }
}

fn display_optional_string(value: Option<&str>) -> &str {
    value.unwrap_or("unset")
}

fn display_optional_usize(value: Option<usize>) -> String {
    value
        .map(|number| number.to_string())
        .unwrap_or_else(|| "unset".to_string())
}

fn display_optional_bool(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "on",
        Some(false) => "off",
        None => "unset",
    }
}

fn ci_execution_name(ci_execution: &CiExecution) -> &'static str {
    match ci_execution {
        CiExecution::ExplicitOptIn => "explicit_opt_in",
    }
}

fn promotion_window_name(window: &ExecutorPromotionWindow) -> &'static str {
    match window {
        ExecutorPromotionWindow::LastSuccessfulRuns => "last_successful_runs",
    }
}
