//! Handler for the `tokmd gate` command.

use anyhow::{Context, Result, bail};
use std::path::Path;
use tokmd_analysis as analysis;
use tokmd_analysis_types as analysis_types;
use tokmd_config as cli;
use tokmd_gate::{GateResult, PolicyConfig, PolicyRule, RuleLevel, RuleOperator, evaluate_policy};

use crate::analysis_utils;
use crate::config::ResolvedConfig;
use crate::export_bundle;

/// Exit code for gate failure.
const EXIT_FAIL: i32 = 1;

/// Handle the gate command.
pub(crate) fn handle(
    args: cli::CliGateArgs,
    global: &cli::GlobalArgs,
    resolved: &ResolvedConfig,
) -> Result<()> {
    // Load policy from file, CLI args, or config
    let policy = load_policy(&args, resolved)?;

    // Load or compute receipt
    let receipt = load_or_compute_receipt(&args, global)?;

    // Evaluate policy
    let result = evaluate_policy(&receipt, &policy);

    // Output results
    match args.format {
        cli::GateFormat::Text => print_text_result(&result),
        cli::GateFormat::Json => print_json_result(&result)?,
    }

    // Exit with appropriate code
    if !result.passed {
        std::process::exit(EXIT_FAIL);
    }

    Ok(())
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
    bail!(
        "No policy specified. Use --policy <path> or add rules to [gate] in tokmd.toml.\n\
         \n\
         Example tokmd.toml:\n\
         \n\
         [[gate.rules]]\n\
         name = \"max_tokens\"\n\
         pointer = \"/derived/totals/tokens\"\n\
         op = \"lte\"\n\
         value = 500000\n\
         level = \"error\"\n\
         message = \"Codebase exceeds token budget\"\n\
         \n\
         Or use a separate policy file with --policy <path>"
    )
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
    let preset = args.preset.unwrap_or(cli::AnalysisPreset::Receipt);
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

/// Print results in text format.
fn print_text_result(result: &GateResult) {
    if result.passed {
        println!(
            "Gate PASSED ({} rules evaluated)",
            result.rule_results.len()
        );
    } else {
        println!(
            "Gate FAILED: {} error(s), {} warning(s)",
            result.errors, result.warnings
        );
    }

    println!();

    for rule_result in &result.rule_results {
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
}

/// Print results in JSON format.
fn print_json_result(result: &GateResult) -> Result<()> {
    let json = serde_json::to_string_pretty(&result)?;
    println!("{}", json);
    Ok(())
}
