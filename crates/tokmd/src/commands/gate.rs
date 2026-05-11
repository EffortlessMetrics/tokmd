//! Handler for the `tokmd gate` command.

use crate::cli;
use anyhow::{Result, bail};
use serde::Serialize;
use tokmd_gate::{GateResult, RatchetGateResult, evaluate_policy, evaluate_ratchet_policy};

use crate::config::ResolvedConfig;

#[path = "gate/policy.rs"]
mod policy;
#[path = "gate/receipt.rs"]
mod receipt;
#[path = "gate/render.rs"]
mod render;

/// Exit code for gate failure.
const EXIT_FAIL: i32 = 1;

/// Combined result of policy and ratchet evaluation.
#[derive(Debug, Clone, Serialize)]
struct CombinedGateResult {
    /// Overall pass/fail (policy errors + ratchet errors = 0).
    pub passed: bool,
    /// Policy evaluation result.
    pub policy: Option<GateResult>,
    /// Ratchet evaluation result.
    pub ratchet: Option<RatchetGateResult>,
    /// Total errors (policy + ratchet).
    pub total_errors: usize,
    /// Total warnings (policy + ratchet).
    pub total_warnings: usize,
}

/// Handle the gate command.
pub(crate) fn handle(
    args: cli::CliGateArgs,
    global: &cli::GlobalArgs,
    resolved: &ResolvedConfig,
) -> Result<()> {
    // Load or compute receipt (current state)
    let receipt = receipt::load_or_compute_receipt(&args, global)?;

    // Load policy from file, CLI args, or config (may be None if only ratchet is used)
    let policy = policy::load_policy(&args, resolved).ok();

    // Load baseline if provided
    let baseline = policy::load_baseline(&args, resolved)?;

    // Load ratchet config if baseline provided
    let ratchet_config = if baseline.is_some() {
        policy::load_ratchet_config(&args, resolved)?
    } else {
        None
    };

    // Ensure we have at least policy or ratchet rules
    if policy.is_none() && ratchet_config.is_none() {
        bail!(
            "No policy or ratchet rules specified.\n\
             \n\
             Use --policy <path> for policy rules, or\n\
             --baseline <path> with --ratchet-config <path> for ratchet rules, or\n\
             add rules to [gate] in tokmd.toml.\n\
             \n\
             Example tokmd.toml with policy rules:\n\
             \n\
             [[gate.rules]]\n\
             name = \"max_tokens\"\n\
             pointer = \"/derived/totals/tokens\"\n\
             op = \"lte\"\n\
             value = 500000\n\
             \n\
             Example tokmd.toml with ratchet rules:\n\
             \n\
             [gate]\n\
             baseline = \".tokmd/baseline.json\"\n\
             \n\
             [[gate.ratchet]]\n\
             pointer = \"/complexity/avg_cyclomatic\"\n\
             max_increase_pct = 10.0\n\
             description = \"Avg cyclomatic complexity\""
        );
    }

    // Evaluate policy rules (if present)
    let policy_result = policy.as_ref().map(|p| evaluate_policy(&receipt, p));

    // Evaluate ratchet rules (if baseline and ratchet config present)
    let ratchet_result = match (&baseline, &ratchet_config) {
        (Some(baseline_value), Some(ratchet)) => {
            Some(evaluate_ratchet_policy(ratchet, baseline_value, &receipt))
        }
        _ => None,
    };

    // Combine results
    let combined = combine_results(policy_result, ratchet_result);

    // Output results
    match args.format {
        cli::GateFormat::Text => render::print_text_result(&combined),
        cli::GateFormat::Json => render::print_json_result(&combined)?,
    }

    // Exit with appropriate code
    if !combined.passed {
        std::process::exit(EXIT_FAIL);
    }

    Ok(())
}

/// Combine policy and ratchet results into a single result.
fn combine_results(
    policy: Option<GateResult>,
    ratchet: Option<RatchetGateResult>,
) -> CombinedGateResult {
    let policy_errors = policy.as_ref().map(|p| p.errors).unwrap_or(0);
    let policy_warnings = policy.as_ref().map(|p| p.warnings).unwrap_or(0);
    let ratchet_errors = ratchet.as_ref().map(|r| r.errors).unwrap_or(0);
    let ratchet_warnings = ratchet.as_ref().map(|r| r.warnings).unwrap_or(0);

    let total_errors = policy_errors + ratchet_errors;
    let total_warnings = policy_warnings + ratchet_warnings;
    let passed = total_errors == 0;

    CombinedGateResult {
        passed,
        policy,
        ratchet,
        total_errors,
        total_warnings,
    }
}
