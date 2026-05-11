//! Policy and ratchet input loading for the `tokmd gate` command.

use crate::cli;
use crate::config::ResolvedConfig;
use anyhow::{Context, Result, bail};
use tokmd_gate::{PolicyConfig, PolicyRule, RatchetConfig, RatchetRule, RuleLevel, RuleOperator};

/// Load policy from file or config.
pub(super) fn load_policy(
    args: &cli::CliGateArgs,
    resolved: &ResolvedConfig,
) -> Result<PolicyConfig> {
    // CLI --policy flag takes precedence.
    if let Some(policy_path) = &args.policy {
        return PolicyConfig::from_file(policy_path)
            .with_context(|| format!("Failed to load policy from {}", policy_path.display()));
    }

    if let Some(toml) = resolved.toml {
        let gate_config = &toml.gate;

        if let Some(policy_path) = &gate_config.policy {
            let path = std::path::PathBuf::from(policy_path);
            return PolicyConfig::from_file(&path)
                .with_context(|| format!("Failed to load policy from {}", path.display()));
        }

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

    bail!("No policy specified")
}

/// Load baseline receipt for ratchet comparison.
pub(super) fn load_baseline(
    args: &cli::CliGateArgs,
    resolved: &ResolvedConfig,
) -> Result<Option<serde_json::Value>> {
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

    Ok(None)
}

/// Load ratchet config from file or TOML config.
pub(super) fn load_ratchet_config(
    args: &cli::CliGateArgs,
    resolved: &ResolvedConfig,
) -> Result<Option<RatchetConfig>> {
    if let Some(ratchet_path) = &args.ratchet_config {
        let config = RatchetConfig::from_file(ratchet_path).with_context(|| {
            format!(
                "Failed to load ratchet config from {}",
                ratchet_path.display()
            )
        })?;
        return Ok(Some(config));
    }

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

    Ok(None)
}

fn convert_ratchet_rule(rule: &cli::RatchetRuleConfig) -> RatchetRule {
    RatchetRule {
        pointer: rule.pointer.clone(),
        max_increase_pct: rule.max_increase_pct,
        max_value: rule.max_value,
        level: parse_level(rule.level.as_deref()),
        description: rule.description.clone(),
    }
}

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

fn parse_level(level: Option<&str>) -> RuleLevel {
    match level.map(|s| s.to_lowercase()).as_deref() {
        Some("warn") | Some("warning") => RuleLevel::Warn,
        _ => RuleLevel::Error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gate_rule(op: &str, level: Option<&str>) -> cli::GateRule {
        cli::GateRule {
            name: "tokens".to_string(),
            pointer: "/derived/totals/tokens".to_string(),
            op: op.to_string(),
            value: Some(serde_json::json!(500000)),
            values: None,
            negate: false,
            level: level.map(str::to_string),
            message: Some("too many tokens".to_string()),
        }
    }

    #[test]
    fn convert_gate_rule_preserves_inline_config_contract() {
        let converted = convert_gate_rule(&gate_rule("<=", Some("warn"))).unwrap();

        assert_eq!(converted.name, "tokens");
        assert_eq!(converted.pointer, "/derived/totals/tokens");
        assert_eq!(converted.op, RuleOperator::Lte);
        assert_eq!(converted.value, Some(serde_json::json!(500000)));
        assert_eq!(converted.level, RuleLevel::Warn);
        assert_eq!(converted.message.as_deref(), Some("too many tokens"));
    }

    #[test]
    fn parse_operator_accepts_cli_aliases() {
        let cases = [
            (">", RuleOperator::Gt),
            (">=", RuleOperator::Gte),
            ("<", RuleOperator::Lt),
            ("<=", RuleOperator::Lte),
            ("==", RuleOperator::Eq),
            ("=", RuleOperator::Eq),
            ("!=", RuleOperator::Ne),
        ];

        for (input, expected) in cases {
            assert_eq!(parse_operator(input).unwrap(), expected);
        }
    }

    #[test]
    fn parse_level_defaults_unknown_to_error() {
        assert_eq!(parse_level(None), RuleLevel::Error);
        assert_eq!(parse_level(Some("warning")), RuleLevel::Warn);
        assert_eq!(parse_level(Some("audit")), RuleLevel::Error);
    }
}
