use crate::cli::{CiRunnerHealthArgs, CiRunnerHealthStatusArg};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

pub const RUNNER_HEALTH_SCHEMA: &str = "tokmd.ci_runner_health.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CiRunnerHealthReceipt {
    pub schema: String,
    pub runner_name: String,
    pub labels: Vec<String>,
    pub generated_at_ms: u128,
    pub status: CiRunnerHealthStatus,
    pub reason: String,
    pub disk_free_bytes: Option<u64>,
    pub scratch_free_bytes: Option<u64>,
    pub min_free_bytes: u64,
    pub rustc: ToolHealth,
    pub git: ToolHealth,
    pub docker: ToolHealth,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CiRunnerHealthStatus {
    Healthy,
    Degraded,
    Quarantined,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolHealth {
    pub available: bool,
    pub version: Option<String>,
}

pub fn run(args: CiRunnerHealthArgs) -> Result<()> {
    let receipt = runner_health_receipt(&args)?;
    let body = to_pretty_json(&receipt)?;

    if let Some(parent) = args.json.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("create runner health dir {}", parent.display()))?;
    }
    fs::write(&args.json, body)
        .with_context(|| format!("write runner health receipt {}", args.json.display()))?;

    println!(
        "ci-runner-health: status={} runner={} receipt={}",
        receipt.status.as_str(),
        receipt.runner_name,
        args.json.display()
    );
    Ok(())
}

pub fn runner_health_receipt(args: &CiRunnerHealthArgs) -> Result<CiRunnerHealthReceipt> {
    let rustc = tool_health("rustc", args.rustc_available, args.rustc_version.as_deref());
    let git = tool_health("git", args.git_available, args.git_version.as_deref());
    let docker = if args.check_docker || args.docker_available.is_some() {
        tool_health(
            "docker",
            args.docker_available,
            args.docker_version.as_deref(),
        )
    } else {
        ToolHealth {
            available: false,
            version: None,
        }
    };
    let status = resolve_status(args, &rustc, &git);
    let reason = args
        .reason
        .clone()
        .unwrap_or_else(|| default_reason(args, status, &rustc, &git));
    let mut labels = args.labels.clone();
    labels.sort();
    labels.dedup();

    let receipt = CiRunnerHealthReceipt {
        schema: RUNNER_HEALTH_SCHEMA.to_string(),
        runner_name: args
            .runner_name
            .clone()
            .or_else(|| env_non_empty("RUNNER_NAME"))
            .unwrap_or_else(|| "unknown".to_string()),
        labels,
        generated_at_ms: args.timestamp_ms.unwrap_or_else(now_ms),
        status,
        reason,
        disk_free_bytes: args.disk_free_bytes,
        scratch_free_bytes: args.scratch_free_bytes,
        min_free_bytes: args.min_free_bytes,
        rustc,
        git,
        docker,
        warnings: Vec::new(),
        errors: Vec::new(),
    };
    validate_runner_health_receipt(&receipt)?;
    Ok(receipt)
}

pub fn read_runner_health_receipt(path: &Path) -> Result<CiRunnerHealthReceipt> {
    let body = fs::read_to_string(path)
        .with_context(|| format!("read runner health receipt {}", path.display()))?;
    let receipt: CiRunnerHealthReceipt = serde_json::from_str(&body)
        .with_context(|| format!("parse runner health receipt {}", path.display()))?;
    validate_runner_health_receipt(&receipt)?;
    Ok(receipt)
}

pub fn to_pretty_json(receipt: &CiRunnerHealthReceipt) -> Result<String> {
    validate_runner_health_receipt(receipt)?;
    let body = serde_json::to_string_pretty(receipt)?;
    Ok(format!("{body}\n"))
}

pub fn validate_runner_health_receipt(receipt: &CiRunnerHealthReceipt) -> Result<()> {
    if receipt.schema != RUNNER_HEALTH_SCHEMA {
        bail!(
            "runner health schema mismatch: expected {}, got {}",
            RUNNER_HEALTH_SCHEMA,
            receipt.schema
        );
    }

    for (field, value) in receipt_strings(receipt) {
        if looks_like_absolute_path(value) {
            bail!("runner health field {field} contains an absolute path");
        }
        if looks_like_secret(value) {
            bail!("runner health field {field} contains a secret-looking value");
        }
    }
    Ok(())
}

impl CiRunnerHealthStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Quarantined => "quarantined",
        }
    }
}

fn resolve_status(
    args: &CiRunnerHealthArgs,
    rustc: &ToolHealth,
    git: &ToolHealth,
) -> CiRunnerHealthStatus {
    if let Some(status) = args.status {
        return match status {
            CiRunnerHealthStatusArg::Healthy => CiRunnerHealthStatus::Healthy,
            CiRunnerHealthStatusArg::Degraded => CiRunnerHealthStatus::Degraded,
            CiRunnerHealthStatusArg::Quarantined => CiRunnerHealthStatus::Quarantined,
        };
    }

    if args
        .disk_free_bytes
        .is_some_and(|free| free < args.min_free_bytes)
        || args
            .scratch_free_bytes
            .is_some_and(|free| free < args.min_free_bytes)
        || !rustc.available
        || !git.available
    {
        CiRunnerHealthStatus::Degraded
    } else {
        CiRunnerHealthStatus::Healthy
    }
}

fn default_reason(
    args: &CiRunnerHealthArgs,
    status: CiRunnerHealthStatus,
    rustc: &ToolHealth,
    git: &ToolHealth,
) -> String {
    if let Some(free) = args.disk_free_bytes
        && free < args.min_free_bytes
    {
        return "disk_free_below_guard".to_string();
    }
    if let Some(free) = args.scratch_free_bytes
        && free < args.min_free_bytes
    {
        return "scratch_free_below_guard".to_string();
    }
    if !rustc.available {
        return "rustc_unavailable".to_string();
    }
    if !git.available {
        return "git_unavailable".to_string();
    }
    match status {
        CiRunnerHealthStatus::Healthy => "healthy".to_string(),
        CiRunnerHealthStatus::Degraded => "degraded".to_string(),
        CiRunnerHealthStatus::Quarantined => "quarantined".to_string(),
    }
}

fn tool_health(program: &str, available: Option<bool>, version: Option<&str>) -> ToolHealth {
    if let Some(available) = available {
        return ToolHealth {
            available,
            version: version.map(ToOwned::to_owned),
        };
    }

    match Command::new(program).arg("--version").output() {
        Ok(output) if output.status.success() => ToolHealth {
            available: true,
            version: Some(first_line(&output.stdout)),
        },
        _ => ToolHealth {
            available: false,
            version: version.map(ToOwned::to_owned),
        },
    }
}

fn first_line(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .lines()
        .next()
        .unwrap_or("unknown")
        .trim()
        .to_string()
}

fn receipt_strings(receipt: &CiRunnerHealthReceipt) -> Vec<(&'static str, &str)> {
    let mut values = vec![
        ("schema", receipt.schema.as_str()),
        ("runner_name", receipt.runner_name.as_str()),
        ("reason", receipt.reason.as_str()),
    ];
    values.extend(
        receipt
            .labels
            .iter()
            .map(|label| ("labels", label.as_str())),
    );
    if let Some(version) = &receipt.rustc.version {
        values.push(("rustc.version", version.as_str()));
    }
    if let Some(version) = &receipt.git.version {
        values.push(("git.version", version.as_str()));
    }
    if let Some(version) = &receipt.docker.version {
        values.push(("docker.version", version.as_str()));
    }
    values.extend(
        receipt
            .warnings
            .iter()
            .map(|warning| ("warnings", warning.as_str())),
    );
    values.extend(
        receipt
            .errors
            .iter()
            .map(|error| ("errors", error.as_str())),
    );
    values
}

fn looks_like_absolute_path(value: &str) -> bool {
    value
        .split(|ch: char| ch.is_whitespace() || ch == '"' || ch == '\'' || ch == '`')
        .any(|part| {
            let normalized = part
                .trim_matches(|ch: char| matches!(ch, ',' | ';' | ')' | '(' | '[' | ']'))
                .replace('\\', "/");
            normalized.starts_with('/')
                || (normalized.len() >= 3
                    && normalized.as_bytes()[1] == b':'
                    && normalized.as_bytes()[2] == b'/'
                    && normalized.as_bytes()[0].is_ascii_alphabetic())
        })
}

fn looks_like_secret(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    lowered.contains("ghp_")
        || lowered.contains("github_pat_")
        || lowered.contains("x-access-token")
        || lowered.contains("authorization:")
        || lowered.contains("bearer ")
        || lowered.contains("token=")
        || lowered.contains("secret=")
}

fn env_non_empty(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.trim().is_empty())
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args() -> CiRunnerHealthArgs {
        CiRunnerHealthArgs {
            runner_name: Some("CPX42".to_string()),
            labels: vec![
                "self-hosted".to_string(),
                "linux".to_string(),
                "em-ci-small".to_string(),
            ],
            timestamp_ms: Some(1_700_000_000_000),
            disk_free_bytes: Some(16 * 1024 * 1024 * 1024),
            scratch_free_bytes: Some(16 * 1024 * 1024 * 1024),
            rustc_available: Some(true),
            rustc_version: Some("rustc 1.95.0".to_string()),
            git_available: Some(true),
            git_version: Some("git version 2.50.0".to_string()),
            ..CiRunnerHealthArgs::default()
        }
    }

    #[test]
    fn runner_health_receipt_is_deterministic() {
        let receipt = runner_health_receipt(&args()).expect("receipt");
        let json = to_pretty_json(&receipt).expect("json");

        assert_eq!(
            json,
            "{\n  \"schema\": \"tokmd.ci_runner_health.v1\",\n  \"runner_name\": \"CPX42\",\n  \"labels\": [\n    \"em-ci-small\",\n    \"linux\",\n    \"self-hosted\"\n  ],\n  \"generated_at_ms\": 1700000000000,\n  \"status\": \"healthy\",\n  \"reason\": \"healthy\",\n  \"disk_free_bytes\": 17179869184,\n  \"scratch_free_bytes\": 17179869184,\n  \"min_free_bytes\": 8589934592,\n  \"rustc\": {\n    \"available\": true,\n    \"version\": \"rustc 1.95.0\"\n  },\n  \"git\": {\n    \"available\": true,\n    \"version\": \"git version 2.50.0\"\n  },\n  \"docker\": {\n    \"available\": false,\n    \"version\": null\n  },\n  \"warnings\": [],\n  \"errors\": []\n}\n"
        );
    }

    #[test]
    fn low_scratch_degrades_health() {
        let receipt = runner_health_receipt(&CiRunnerHealthArgs {
            scratch_free_bytes: Some(1024),
            ..args()
        })
        .expect("receipt");

        assert_eq!(receipt.status, CiRunnerHealthStatus::Degraded);
        assert_eq!(receipt.reason, "scratch_free_below_guard");
    }

    #[test]
    fn explicit_quarantine_wins() {
        let receipt = runner_health_receipt(&CiRunnerHealthArgs {
            status: Some(CiRunnerHealthStatusArg::Quarantined),
            reason: Some("manual_quarantine".to_string()),
            ..args()
        })
        .expect("receipt");

        assert_eq!(receipt.status, CiRunnerHealthStatus::Quarantined);
        assert_eq!(receipt.reason, "manual_quarantine");
    }

    #[test]
    fn missing_git_degrades_health() {
        let receipt = runner_health_receipt(&CiRunnerHealthArgs {
            git_available: Some(false),
            git_version: None,
            ..args()
        })
        .expect("receipt");

        assert_eq!(receipt.status, CiRunnerHealthStatus::Degraded);
        assert_eq!(receipt.reason, "git_unavailable");
    }

    #[test]
    fn rejects_secret_like_values() {
        let mut receipt = runner_health_receipt(&args()).expect("receipt");
        receipt.warnings.push("token=ghp_example".to_string());

        let err = validate_runner_health_receipt(&receipt).expect_err("secret");

        assert!(
            err.to_string().contains("secret-looking"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn rejects_absolute_machine_paths() {
        let mut receipt = runner_health_receipt(&args()).expect("receipt");
        receipt
            .errors
            .push("read C:/ci-scratch/health.json".to_string());

        let err = validate_runner_health_receipt(&receipt).expect_err("absolute path");

        assert!(
            err.to_string().contains("absolute path"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn run_writes_receipt() {
        let dir = tempfile::tempdir().expect("tempdir");
        let json = dir.path().join("health.json");

        run(CiRunnerHealthArgs {
            json: json.clone(),
            ..args()
        })
        .expect("run health");

        let receipt = read_runner_health_receipt(&json).expect("read receipt");
        assert_eq!(receipt.status, CiRunnerHealthStatus::Healthy);
    }
}
