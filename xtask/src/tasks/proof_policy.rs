use crate::cli::ProofPolicyArgs;
use crate::proof::policy::load_policy;
use crate::proof::policy_ast::{CiExecution, ProofPolicy};
use crate::proof::validate::{PolicyViolation, validate_policy};
use anyhow::{Result, bail};
use serde::Serialize;
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
    violations: Vec<PolicyViolation>,
}

#[derive(Debug, Serialize)]
struct ExecutorPolicyReport {
    family: Option<String>,
    ci_execution: Option<String>,
    max_dry_run_commands: Option<usize>,
}

pub fn run(args: ProofPolicyArgs) -> Result<()> {
    let _check_requested = args.check || !args.json;
    let path = args.policy;
    let policy = load_policy(&path)?;
    let violations = validate_policy(&policy);
    let executor = ExecutorPolicyReport::from_policy(&policy);
    let report = ProofPolicyReport {
        ok: violations.is_empty(),
        policy: display_path(&path),
        schema: policy.schema.clone(),
        scope_count: policy.scope.len(),
        allowlist_count: policy.allow.workspace_area.len(),
        fixture_blob_rule_count: policy.forbid.fixture_blob.len(),
        dependency_boundary_count: policy.dependency_boundary.len(),
        executor,
        violations,
    };

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
        }
    }
}

fn print_human_report(report: &ProofPolicyReport) {
    if report.ok {
        println!(
            "Proof policy OK: {} (schema {}, {} scope(s), {} allowlist(s), {} fixture blob rule(s), {} dependency boundary rule(s), executor {})",
            report.policy,
            report.schema,
            report.scope_count,
            report.allowlist_count,
            report.fixture_blob_rule_count,
            report.dependency_boundary_count,
            executor_summary(&report.executor)
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

fn executor_summary(executor: &ExecutorPolicyReport) -> String {
    match (
        executor.family.as_deref(),
        executor.ci_execution.as_deref(),
        executor.max_dry_run_commands,
    ) {
        (Some(family), Some(ci_execution), Some(max_dry_run_commands)) => {
            format!("{family}/{ci_execution}/max-dry-run-{max_dry_run_commands}")
        }
        _ => "not-configured".to_string(),
    }
}

fn ci_execution_name(ci_execution: &CiExecution) -> &'static str {
    match ci_execution {
        CiExecution::ExplicitOptIn => "explicit_opt_in",
    }
}
