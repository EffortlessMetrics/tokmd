use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::cli::ProofExecutorPrPolicyArgs;

pub fn run(args: ProofExecutorPrPolicyArgs) -> Result<()> {
    let policy = read_policy_json(&args.proof_policy_json)?;
    let executor = policy
        .get("executor")
        .and_then(Value::as_object)
        .context("proof policy JSON is missing executor")?;
    let pr = executor
        .get("pr")
        .and_then(Value::as_object)
        .context("proof policy JSON is missing executor.pr")?;

    require_bool(
        pr.get("default_enabled"),
        true,
        "executor.pr.default_enabled",
    )?;
    require_bool(pr.get("required"), false, "executor.pr.required")?;
    require_bool(
        pr.get("codecov_upload"),
        false,
        "executor.pr.codecov_upload",
    )?;

    let max_commands = resolve_max_commands(
        &args.max_commands,
        pr.get("max_commands"),
        executor.get("max_dry_run_commands"),
    )?;

    let env = format!(
        "\
PROOF_EXECUTOR_MAX_COMMANDS={max_commands}
PROOF_EXECUTOR_MAX_COMMANDS_SOURCE={}
PROOF_EXECUTOR_PR_DEFAULT_ENABLED=true
PROOF_EXECUTOR_PR_REQUIRED=false
PROOF_EXECUTOR_PR_CODECOV_UPLOAD=false
",
        max_commands.source
    );

    write_env_output(&args.env_output, &env)?;

    println!(
        "proof executor PR policy: wrote executor PR env to {}",
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

fn require_bool(value: Option<&Value>, expected: bool, field: &str) -> Result<()> {
    match value.and_then(Value::as_bool) {
        Some(actual) if actual == expected => Ok(()),
        Some(actual) => bail!("{field} must be {expected}, got {actual}"),
        None => bail!("{field} is missing or not a boolean"),
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ResolvedMaxCommands {
    value: u64,
    source: &'static str,
}

impl std::fmt::Display for ResolvedMaxCommands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

fn resolve_max_commands(
    override_text: &str,
    pr_value: Option<&Value>,
    executor_default: Option<&Value>,
) -> Result<ResolvedMaxCommands> {
    let trimmed = override_text.trim();
    let (value, source) = if trimmed.is_empty() {
        (
            pr_value
                .or(executor_default)
                .and_then(Value::as_u64)
                .context("executor.pr.max_commands is missing or not numeric")?,
            "ci/proof.toml",
        )
    } else {
        (
            trimmed.parse::<u64>().with_context(|| {
                format!("PROOF_EXECUTOR_MAX_COMMANDS must be an integer, got {trimmed:?}")
            })?,
            "workflow_dispatch",
        )
    };

    if value == 0 {
        bail!("PROOF_EXECUTOR_MAX_COMMANDS must be >= 1, got 0");
    }

    Ok(ResolvedMaxCommands { value, source })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn max_commands_uses_pr_policy_when_override_is_blank() {
        let resolved = resolve_max_commands("", Some(&json!(2)), Some(&json!(1))).unwrap();

        assert_eq!(
            resolved,
            ResolvedMaxCommands {
                value: 2,
                source: "ci/proof.toml"
            }
        );
    }

    #[test]
    fn max_commands_falls_back_to_executor_default() {
        let resolved = resolve_max_commands("", None, Some(&json!(1))).unwrap();

        assert_eq!(resolved.value, 1);
        assert_eq!(resolved.source, "ci/proof.toml");
    }

    #[test]
    fn max_commands_uses_workflow_override_when_present() {
        let resolved = resolve_max_commands("7", Some(&json!(2)), Some(&json!(1))).unwrap();

        assert_eq!(resolved.value, 7);
        assert_eq!(resolved.source, "workflow_dispatch");
    }

    #[test]
    fn max_commands_rejects_zero() {
        let err = resolve_max_commands("0", Some(&json!(2)), Some(&json!(1)))
            .unwrap_err()
            .to_string();

        assert!(
            err.contains("PROOF_EXECUTOR_MAX_COMMANDS must be >= 1"),
            "{err}"
        );
    }

    #[test]
    fn rejects_wrong_policy_bool() {
        let err = require_bool(Some(&json!(true)), false, "executor.pr.required")
            .unwrap_err()
            .to_string();

        assert!(err.contains("executor.pr.required must be false"), "{err}");
    }
}
