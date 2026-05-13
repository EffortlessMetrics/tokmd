use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::cli::ProofRunPrPolicyArgs;

pub fn run(args: ProofRunPrPolicyArgs) -> Result<()> {
    let policy = read_policy_json(&args.proof_policy_json)?;
    let pr = policy
        .get("proof_run")
        .and_then(|proof_run| proof_run.get("pr"))
        .and_then(Value::as_object)
        .context("proof policy JSON is missing proof_run.pr")?;

    require_bool(
        pr.get("default_enabled"),
        true,
        "proof_run.pr.default_enabled",
    )?;
    require_bool(pr.get("required"), false, "proof_run.pr.required")?;

    let profile = require_string(pr.get("profile"), "fast", "proof_run.pr.profile")?;
    let artifact_name = require_string(
        pr.get("artifact_name"),
        "fast-proof-run",
        "proof_run.pr.artifact_name",
    )?;

    let output = format!("profile={profile}\nartifact_name={artifact_name}\n");
    write_github_output(&args.github_output, &output)?;

    println!(
        "proof-run PR policy: wrote GitHub output to {}",
        args.github_output.display()
    );
    Ok(())
}

fn read_policy_json(path: &Path) -> Result<Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn write_github_output(path: &Path, output: &str) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("create directory {}", parent.display()))?;
    }
    fs::write(path, output).with_context(|| format!("write {}", path.display()))
}

fn require_bool(value: Option<&Value>, expected: bool, field: &str) -> Result<()> {
    match value.and_then(Value::as_bool) {
        Some(actual) if actual == expected => Ok(()),
        Some(actual) => bail!("{field} must be {expected}, got {actual}"),
        None => bail!("{field} is missing or not a boolean"),
    }
}

fn require_string<'a>(value: Option<&'a Value>, expected: &str, field: &str) -> Result<&'a str> {
    match value.and_then(Value::as_str) {
        Some(actual) if actual == expected => Ok(actual),
        Some(actual) => bail!("{field} must be {expected}, got {actual:?}"),
        None => bail!("{field} is missing or not a string"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn accepts_expected_fast_profile() {
        let value = json!("fast");
        let profile = require_string(Some(&value), "fast", "proof_run.pr.profile").unwrap();

        assert_eq!(profile, "fast");
    }

    #[test]
    fn rejects_wrong_profile() {
        let err = require_string(Some(&json!("deep")), "fast", "proof_run.pr.profile")
            .unwrap_err()
            .to_string();

        assert!(err.contains("proof_run.pr.profile must be fast"), "{err}");
    }

    #[test]
    fn rejects_wrong_required_flag() {
        let err = require_bool(Some(&json!(true)), false, "proof_run.pr.required")
            .unwrap_err()
            .to_string();

        assert!(err.contains("proof_run.pr.required must be false"), "{err}");
    }
}
