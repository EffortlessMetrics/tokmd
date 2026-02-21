//! Handler for the `tokmd gate` command.

use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::path::Path;
use tokmd_analysis as analysis;
use tokmd_analysis_types as analysis_types;
use tokmd_config as cli;
use tokmd_gate::{
    GateResult, PolicyConfig, PolicyRule, RatchetConfig, RatchetGateResult, RatchetRule, RuleLevel,
    RuleOperator, evaluate_policy, evaluate_ratchet_policy,
};

use crate::analysis_utils;
use crate::config::ResolvedConfig;
use crate::export_bundle;

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
    let policy = load_policy(&args, resolved).ok();

    // Load baseline if provided
    let baseline = load_baseline(&args, resolved)?;

    // Load ratchet config if baseline provided
    let ratchet_config = if baseline.is_some() {
        load_ratchet_config(&args, resolved)?
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
        cli::GateFormat::Text => print_text_result(&combined),
        cli::GateFormat::Json => print_json_result(&combined)?,
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

/// Load policy from file or config.
fn load_policy(args: &cli::CliGateArgs, resolved: &ResolvedConfig) -> Result<PolicyConfig> {
    // 1. CLI --policy flag takes precedence
    if let Some(policy_path) = &args.policy {
        return PolicyConfig::from_file(policy_path)
            .with_context(|| format!("Failed to load policy from {}", policy_path.display()));
    }

    // 2. Check tokmd.toml [gate] section for inline rules or policy path
    if let Some(toml) = resolved.toml {
        let gate_config = &toml.gate;

        // Check for policy path in config
        if let Some(policy_path) = &gate_config.policy {
            let path = std::path::PathBuf::from(policy_path);
            return PolicyConfig::from_file(&path)
                .with_context(|| format!("Failed to load policy from {}", path.display()));
        }

        // Check for inline rules
        if let Some(rules) = &gate_config.rules
            && !rules.is_empty()
        {
            let policy_rules: Vec<PolicyRule> = rules
                .iter()
                .map(convert_gate_rule)
                .collect::<Result<Vec<_>>>()?;

            return Ok(PolicyConfig {
                rules: policy_rules,
                fail_fast: gate_config.fail_fast.unwrap_or(false),
                allow_missing: false,
            });
        }
    }

    // No policy found
    bail!("No policy specified")
}

/// Load baseline receipt for ratchet comparison.
fn load_baseline(
    args: &cli::CliGateArgs,
    resolved: &ResolvedConfig,
) -> Result<Option<serde_json::Value>> {
    // 1. CLI --baseline flag takes precedence
    if let Some(baseline_path) = &args.baseline {
        let content = std::fs::read_to_string(baseline_path)
            .with_context(|| format!("Failed to read baseline from {}", baseline_path.display()))?;
        let value: serde_json::Value = serde_json::from_str(&content).with_context(|| {
            format!(
                "Failed to parse baseline JSON from {}",
                baseline_path.display()
            )
        })?;
        return Ok(Some(value));
    }

    // 2. Check tokmd.toml [gate.baseline]
    if let Some(toml) = resolved.toml
        && let Some(baseline_path) = &toml.gate.baseline
    {
        let path = std::path::PathBuf::from(baseline_path);
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read baseline from {}", path.display()))?;
        let value: serde_json::Value = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse baseline JSON from {}", path.display()))?;
        return Ok(Some(value));
    }

    // No baseline specified
    Ok(None)
}

/// Load ratchet config from file or TOML config.
fn load_ratchet_config(
    args: &cli::CliGateArgs,
    resolved: &ResolvedConfig,
) -> Result<Option<RatchetConfig>> {
    // 1. CLI --ratchet-config flag takes precedence
    if let Some(ratchet_path) = &args.ratchet_config {
        let config = RatchetConfig::from_file(ratchet_path).with_context(|| {
            format!(
                "Failed to load ratchet config from {}",
                ratchet_path.display()
            )
        })?;
        return Ok(Some(config));
    }

    // 2. Check tokmd.toml [[gate.ratchet]] for inline rules
    if let Some(toml) = resolved.toml {
        let gate_config = &toml.gate;

        if let Some(rules) = &gate_config.ratchet
            && !rules.is_empty()
        {
            let ratchet_rules: Vec<RatchetRule> = rules.iter().map(convert_ratchet_rule).collect();

            return Ok(Some(RatchetConfig {
                rules: ratchet_rules,
                fail_fast: gate_config.fail_fast.unwrap_or(false),
                allow_missing_baseline: gate_config.allow_missing_baseline.unwrap_or(false),
                allow_missing_current: gate_config.allow_missing_current.unwrap_or(false),
            }));
        }
    }

    // No ratchet config found
    Ok(None)
}

/// Convert a config RatchetRuleConfig to a gate RatchetRule.
fn convert_ratchet_rule(rule: &cli::RatchetRuleConfig) -> RatchetRule {
    RatchetRule {
        pointer: rule.pointer.clone(),
        max_increase_pct: rule.max_increase_pct,
        max_value: rule.max_value,
        level: parse_level(rule.level.as_deref()),
        description: rule.description.clone(),
    }
}

/// Convert a config GateRule to a gate PolicyRule.
fn convert_gate_rule(rule: &cli::GateRule) -> Result<PolicyRule> {
    let op = parse_operator(&rule.op)?;

    Ok(PolicyRule {
        name: rule.name.clone(),
        pointer: rule.pointer.clone(),
        op,
        value: rule.value.clone(),
        values: rule.values.clone(),
        negate: rule.negate,
        level: parse_level(rule.level.as_deref()),
        message: rule.message.clone(),
    })
}

/// Parse operator string to RuleOperator enum.
fn parse_operator(op: &str) -> Result<RuleOperator> {
    match op.to_lowercase().as_str() {
        "gt" | ">" => Ok(RuleOperator::Gt),
        "gte" | ">=" => Ok(RuleOperator::Gte),
        "lt" | "<" => Ok(RuleOperator::Lt),
        "lte" | "<=" => Ok(RuleOperator::Lte),
        "eq" | "==" | "=" => Ok(RuleOperator::Eq),
        "ne" | "!=" => Ok(RuleOperator::Ne),
        "in" => Ok(RuleOperator::In),
        "contains" => Ok(RuleOperator::Contains),
        "exists" => Ok(RuleOperator::Exists),
        _ => bail!(
            "Unknown operator: {}. Valid operators: gt, gte, lt, lte, eq, ne, in, contains, exists",
            op
        ),
    }
}

/// Parse level string to RuleLevel enum.
fn parse_level(level: Option<&str>) -> RuleLevel {
    match level.map(|s| s.to_lowercase()).as_deref() {
        Some("warn") | Some("warning") => RuleLevel::Warn,
        _ => RuleLevel::Error,
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
    };

    let ctx = analysis::AnalysisContext {
        export: bundle.export,
        root: bundle.root,
        source,
    };

    let receipt = analysis::analyze(ctx, request)?;

    // Convert to JSON Value for policy evaluation
    serde_json::to_value(&receipt).context("Failed to serialize receipt to JSON")
}

/// Print combined results in text format.
fn print_text_result(result: &CombinedGateResult) {
    let policy_count = result
        .policy
        .as_ref()
        .map(|p| p.rule_results.len())
        .unwrap_or(0);
    let ratchet_count = result
        .ratchet
        .as_ref()
        .map(|r| r.ratchet_results.len())
        .unwrap_or(0);
    let total_rules = policy_count + ratchet_count;

    if result.passed {
        println!("Gate PASSED ({} rules evaluated)", total_rules);
    } else {
        println!(
            "Gate FAILED: {} error(s), {} warning(s)",
            result.total_errors, result.total_warnings
        );
    }

    println!();

    // Print policy results
    if let Some(policy) = &result.policy
        && !policy.rule_results.is_empty()
    {
        println!("Policy Rules:");
        for rule_result in &policy.rule_results {
            let status = if rule_result.passed { "PASS" } else { "FAIL" };
            let level = match rule_result.level {
                RuleLevel::Error => "error",
                RuleLevel::Warn => "warn",
            };

            if rule_result.passed {
                println!("  [{}] {} ({})", status, rule_result.name, level);
            } else {
                println!("  [{}] {} ({})", status, rule_result.name, level);
                println!("        Expected: {}", rule_result.expected);
                if let Some(actual) = &rule_result.actual {
                    println!("        Actual: {}", actual);
                }
                if let Some(msg) = &rule_result.message {
                    println!("        Message: {}", msg);
                }
            }
        }
        println!();
    }

    // Print ratchet results
    if let Some(ratchet) = &result.ratchet
        && !ratchet.ratchet_results.is_empty()
    {
        println!("Ratchet Rules:");
        for ratchet_result in &ratchet.ratchet_results {
            let status = if ratchet_result.passed {
                "PASS"
            } else {
                "FAIL"
            };
            let level = match ratchet_result.rule.level {
                RuleLevel::Error => "error",
                RuleLevel::Warn => "warn",
            };

            let name = ratchet_result
                .rule
                .description
                .as_deref()
                .unwrap_or(&ratchet_result.rule.pointer);

            println!("  [{}] {} ({})", status, name, level);

            if let Some(baseline) = ratchet_result.baseline_value {
                if let Some(pct) = ratchet_result.change_pct {
                    println!(
                        "        Baseline: {:.2} -> Current: {:.2} ({:+.2}%)",
                        baseline, ratchet_result.current_value, pct
                    );
                } else {
                    println!(
                        "        Baseline: {:.2}, Current: {:.2}",
                        baseline, ratchet_result.current_value
                    );
                }
            } else {
                println!("        Current: {:.2}", ratchet_result.current_value);
            }

            if !ratchet_result.passed {
                println!("        Message: {}", ratchet_result.message);
            }
        }
    }
}

/// Print combined results in JSON format.
fn print_json_result(result: &CombinedGateResult) -> Result<()> {
    let json = serde_json::to_string_pretty(&result)?;
    println!("{}", json);
    Ok(())
}
