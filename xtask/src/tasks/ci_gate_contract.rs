//! Single-tight CI gate contract checker for ub-review adoption (#226).
//!
//! Validates that a workflow file encodes the family gate shape documented in
//! `docs/specs/ub-review-ci-gate.md`: one required check, advisory route job,
//! deterministic core floor, and non-blocking ub-review in the same gate job.

use std::fs;

use anyhow::{Context, Result, bail};

use crate::cli::CiGateContractArgs;

const REQUIRED_MARKERS: &[&str] = &[
    "name: Tokmd Rust Result",
    "Route CI runner",
    "EM_RUNNER_READ_TOKEN",
    "gh api \"orgs/EffortlessMetrics/actions/runners",
    "runner_kind",
    "self-hosted",
    "em-ci",
    "trusted-pr",
    "fromJSON(needs.route.outputs.runner)",
    "dtolnay/rust-toolchain",
    "Swatinem/rust-cache@v2",
    "Fast precontext and launch core gate",
    "cargo xtask gate --check",
    "core_exit",
    "Assert core gate verdict",
    "UB Review (advisory)",
    "UB Review advisory status",
    "EffortlessMetrics/ub-review@",
    "mode: intelligent-ci",
    "posting: review",
    "fail-on-gate: false",
    "setup-rust: false",
    "provider-policy: primary-with-fallback",
    "minimax-model: MiniMax-M3",
    "opencode-model: deepseek-v4-flash",
    "pr-thread-context: target/ci-core/precontext.md",
    "continue-on-error: true",
    "github.event.pull_request.head.repo.fork == false",
];

const FORBIDDEN_MARKERS: &[&str] = &[
    "name: CI (Required)",
    "route-rust-small",
    "router_target=",
    "Rust Small on CPX42",
    "Rust Small on CX43",
    "Rust Small on CX53",
    "Rust Small Fallback on GitHub Hosted",
    "Rust Small Blocked (capacity/config)",
    "fallback_mode=full",
    "no-github-fallback",
    "name: Tokmd Rust Small Result",
    "em-ci-rust:1.95",
    "docker run --rm",
];

pub fn run(args: CiGateContractArgs) -> Result<()> {
    let text = fs::read_to_string(&args.workflow)
        .with_context(|| format!("failed to read {}", args.workflow.display()))?;

    let mut errors = Vec::new();

    for needle in REQUIRED_MARKERS {
        if !text.contains(needle) {
            errors.push(format!("missing required marker: {needle}"));
        }
    }

    if text.contains("repos/${") && text.contains("/actions/runners") {
        errors.push(
            "must not use repository runner discovery (org-level orgs/EffortlessMetrics only)"
                .to_string(),
        );
    }

    for forbidden in FORBIDDEN_MARKERS {
        if text.contains(forbidden) {
            errors.push(format!("forbidden retired marker present: {forbidden}"));
        }
    }

    if errors.is_empty() {
        println!("ci-gate-contract OK: {}", args.workflow.display());
        return Ok(());
    }

    for error in &errors {
        eprintln!("ci-gate-contract: {error}");
    }

    if args.check {
        bail!(
            "ci-gate-contract: {} error(s) in {}",
            errors.len(),
            args.workflow.display()
        );
    }

    println!(
        "ci-gate-contract: {} finding(s) in {} (advisory mode)",
        errors.len(),
        args.workflow.display()
    );
    Ok(())
}
