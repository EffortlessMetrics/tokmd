use crate::cli::ProofPolicyArgs;
use crate::proof::policy::load_policy;
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
    violations: Vec<PolicyViolation>,
}

pub fn run(args: ProofPolicyArgs) -> Result<()> {
    let _check_requested = args.check || !args.json;
    let path = args.policy;
    let policy = load_policy(&path)?;
    let violations = validate_policy(&policy);
    let report = ProofPolicyReport {
        ok: violations.is_empty(),
        policy: display_path(&path),
        schema: policy.schema.clone(),
        scope_count: policy.scope.len(),
        allowlist_count: policy.allow.workspace_area.len(),
        fixture_blob_rule_count: policy.forbid.fixture_blob.len(),
        dependency_boundary_count: policy.dependency_boundary.len(),
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

fn print_human_report(report: &ProofPolicyReport) {
    if report.ok {
        println!(
            "Proof policy OK: {} (schema {}, {} scope(s), {} allowlist(s), {} fixture blob rule(s), {} dependency boundary rule(s))",
            report.policy,
            report.schema,
            report.scope_count,
            report.allowlist_count,
            report.fixture_blob_rule_count,
            report.dependency_boundary_count
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
