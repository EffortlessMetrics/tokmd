use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::cli::ProofObservationThresholdsArgs;

pub fn run(args: ProofObservationThresholdsArgs) -> Result<()> {
    let policy = read_policy_json(&args.proof_policy_json)?;
    let promotion = policy
        .get("executor")
        .and_then(|executor| executor.get("promotion"))
        .and_then(Value::as_object)
        .context("proof policy JSON is missing executor.promotion")?;

    let resolved = [
        resolve_threshold("RUN_LIMIT", &args.run_limit, promotion.get("run_limit"), 1)?,
        resolve_threshold(
            "MIN_OBSERVATIONS",
            &args.min_observations,
            promotion.get("min_observations"),
            0,
        )?,
        resolve_threshold(
            "MIN_EXECUTED",
            &args.min_executed,
            promotion.get("min_executed"),
            0,
        )?,
        resolve_threshold(
            "MIN_SCOPES",
            &args.min_scopes,
            promotion.get("min_scopes"),
            0,
        )?,
        resolve_threshold(
            "MIN_ARTIFACTS",
            &args.min_artifacts,
            promotion.get("min_artifacts"),
            0,
        )?,
        resolve_threshold(
            "MIN_PASSING_COLLECTOR_RUNS",
            &args.min_passing_collector_runs,
            promotion.get("min_passing_collector_runs"),
            0,
        )?,
    ];

    let mut env = String::new();
    for threshold in &resolved {
        env.push_str(&format!("{}={}\n", threshold.env_name, threshold.value));
        env.push_str(&format!(
            "{}_SOURCE={}\n",
            threshold.env_name, threshold.source
        ));
    }

    write_env_output(&args.env_output, &env)?;

    println!(
        "proof observation thresholds: wrote {} threshold(s) to {}",
        resolved.len(),
        args.env_output.display()
    );
    Ok(())
}

fn read_policy_json(path: &Path) -> Result<Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn write_env_output(path: &Path, env: &str) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("create directory {}", parent.display()))?;
    }
    fs::write(path, env).with_context(|| format!("write {}", path.display()))
}

#[derive(Debug)]
struct ResolvedThreshold {
    env_name: &'static str,
    value: u64,
    source: &'static str,
}

fn resolve_threshold(
    env_name: &'static str,
    override_text: &str,
    policy_value: Option<&Value>,
    minimum: u64,
) -> Result<ResolvedThreshold> {
    let trimmed = override_text.trim();
    let (value, source) = if trimmed.is_empty() {
        (
            policy_value
                .and_then(Value::as_u64)
                .with_context(|| format!("executor.promotion is missing numeric {env_name}"))?,
            "ci/proof.toml",
        )
    } else {
        (
            trimmed
                .parse::<u64>()
                .with_context(|| format!("{env_name} must be an integer, got {trimmed:?}"))?,
            "workflow_dispatch",
        )
    };

    if value < minimum {
        bail!("{env_name} must be >= {minimum}, got {value}");
    }

    Ok(ResolvedThreshold {
        env_name,
        value,
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn threshold_uses_policy_value_when_override_is_blank() {
        let resolved = resolve_threshold("RUN_LIMIT", "", Some(&json!(100)), 1).unwrap();
        assert_eq!(resolved.env_name, "RUN_LIMIT");
        assert_eq!(resolved.value, 100);
        assert_eq!(resolved.source, "ci/proof.toml");
    }

    #[test]
    fn threshold_uses_workflow_override_when_present() {
        let resolved = resolve_threshold("MIN_EXECUTED", "7", Some(&json!(4)), 0).unwrap();
        assert_eq!(resolved.value, 7);
        assert_eq!(resolved.source, "workflow_dispatch");
    }

    #[test]
    fn threshold_rejects_values_below_floor() {
        let err = resolve_threshold("RUN_LIMIT", "0", Some(&json!(100)), 1)
            .unwrap_err()
            .to_string();
        assert!(err.contains("RUN_LIMIT must be >= 1"), "{err}");
    }
}
