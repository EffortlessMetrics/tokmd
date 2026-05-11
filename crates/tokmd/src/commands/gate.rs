//! Handler for the `tokmd gate` command.

use crate::cli;
use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::path::Path;
use tokmd_analysis as analysis;
use tokmd_analysis_types as analysis_types;
use tokmd_gate::{GateResult, RatchetGateResult, evaluate_policy, evaluate_ratchet_policy};

use crate::analysis_utils;
use crate::config::ResolvedConfig;
use crate::export_bundle;

#[path = "gate/policy.rs"]
mod policy;
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
    let receipt = load_or_compute_receipt(&args, global)?;

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

/// Load receipt from file or compute from path.
fn load_or_compute_receipt(
    args: &cli::CliGateArgs,
    global: &cli::GlobalArgs,
) -> Result<serde_json::Value> {
    let input = args.input.clone().unwrap_or_else(|| ".".into());

    // Check if input is a JSON file
    if input.extension().map(|e| e == "json").unwrap_or(false) && input.exists() {
        let content = std::fs::read_to_string(&input)
            .with_context(|| format!("Failed to read receipt from {}", input.display()))?;
        return serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON from {}", input.display()));
    }

    // Otherwise, compute analysis receipt
    // Default to Health preset when baseline is provided (it includes complexity metrics),
    // otherwise Receipt for lightweight checks
    let preset = args.preset.unwrap_or_else(|| {
        if args.baseline.is_some() {
            cli::AnalysisPreset::Health
        } else {
            cli::AnalysisPreset::Receipt
        }
    });
    compute_receipt(&input, preset, global)
}

/// Compute an analysis receipt from a path.
fn compute_receipt(
    input: &Path,
    preset: cli::AnalysisPreset,
    global: &cli::GlobalArgs,
) -> Result<serde_json::Value> {
    let inputs = vec![input.to_path_buf()];
    let bundle = export_bundle::load_export_from_inputs(&inputs, global)?;

    let source = analysis_types::AnalysisSource {
        inputs: inputs.iter().map(|p| p.display().to_string()).collect(),
        export_path: bundle.export_path.as_ref().map(|p| p.display().to_string()),
        base_receipt_path: bundle.export_path.as_ref().map(|p| p.display().to_string()),
        export_schema_version: bundle.meta.schema_version,
        export_generated_at_ms: bundle.meta.generated_at_ms,
        base_signature: None,
        module_roots: bundle.meta.module_roots.clone(),
        module_depth: bundle.meta.module_depth,
        children: analysis_utils::child_include_to_string(bundle.meta.children),
    };

    let args_meta = analysis_types::AnalysisArgsMeta {
        preset: analysis_utils::preset_to_string(preset),
        format: "json".to_string(),
        window_tokens: None,
        git: None,
        max_files: None,
        max_bytes: None,
        max_file_bytes: None,
        max_commits: None,
        max_commit_files: None,
        import_granularity: "module".to_string(),
    };

    let request = analysis::AnalysisRequest {
        preset: analysis_utils::map_preset(preset),
        args: args_meta,
        limits: analysis::AnalysisLimits::default(),
        window_tokens: None,
        git: None,
        import_granularity: analysis::ImportGranularity::Module,
        detail_functions: false,
        near_dup: false,
        near_dup_threshold: 0.80,
        near_dup_max_files: 2000,
        near_dup_scope: analysis::NearDupScope::Module,
        near_dup_max_pairs: None,
        near_dup_exclude: Vec::new(),
        effort: None,
    };

    let ctx = analysis::AnalysisContext {
        export: bundle.export,
        root: bundle.root,
        source,
    };

    let receipt = analysis::analyze(ctx, request)?;

    // Convert to JSON Value for policy evaluation
    serde_json::to_value(receipt).context("Failed to serialize receipt to JSON")
}
