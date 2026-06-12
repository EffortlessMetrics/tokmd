use crate::{
    cli::{CiRouteArgs, CiRouteHealth, CiRouteMode},
    tasks::ci_runner_health::{
        CiRunnerHealthReceipt, CiRunnerHealthStatus, read_runner_health_receipt,
    },
};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

pub const ROUTE_RECEIPT_SCHEMA: &str = "tokmd.ci_route.v1";
pub const RUST_SMALL_LANE: &str = "rust-small";
pub const DEFAULT_GITHUB_HOSTED_LABEL: &str = "ubuntu-24.04";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CiRouteReceipt {
    pub schema: String,
    pub lane: String,
    pub target: CiRouteTarget,
    pub reason: CiRouteReason,
    pub trusted_event: bool,
    pub event_name: String,
    pub repo: String,
    pub head_sha: String,
    pub eligible_runners: u32,
    pub busy_runners: u32,
    pub healthy_runners: u32,
    pub fallback_allowed: bool,
    pub selected_runner_label: String,
    pub selected_runner: Option<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CiRouteTarget {
    SelfHosted,
    GithubHosted,
    None,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CiRouteReason {
    TrustedCapacityAvailable,
    ForkPullRequest,
    UntrustedEvent,
    RunnerApiUnavailable,
    RunnerTokenUnavailable,
    RunnerHealthStale,
    RunnerHealthDegraded,
    SelfHostedCapacityFull,
    LowDisk,
    LowScratch,
    RunnerQuarantined,
    RouteBudgetExhausted,
    ManualForceGithubHosted,
    ManualForceSelfHostedDenied,
    UnknownState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CiRouteContext {
    pub event_name: String,
    pub repo: String,
    pub head_sha: String,
    pub trusted_event: bool,
}

impl CiRouteContext {
    pub fn new(
        event_name: impl Into<String>,
        repo: impl Into<String>,
        head_sha: impl Into<String>,
        trusted_event: bool,
    ) -> Self {
        Self {
            event_name: event_name.into(),
            repo: repo.into(),
            head_sha: head_sha.into(),
            trusted_event,
        }
    }
}

impl CiRouteReceipt {
    pub fn github_hosted_fallback(context: CiRouteContext, reason: CiRouteReason) -> Self {
        Self {
            schema: ROUTE_RECEIPT_SCHEMA.to_string(),
            lane: RUST_SMALL_LANE.to_string(),
            target: CiRouteTarget::GithubHosted,
            reason,
            trusted_event: context.trusted_event,
            event_name: context.event_name,
            repo: context.repo,
            head_sha: context.head_sha,
            eligible_runners: 0,
            busy_runners: 0,
            healthy_runners: 0,
            fallback_allowed: true,
            selected_runner_label: DEFAULT_GITHUB_HOSTED_LABEL.to_string(),
            selected_runner: None,
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn self_hosted(
        context: CiRouteContext,
        selected_runner_label: impl Into<String>,
        selected_runner: impl Into<String>,
        eligible_runners: u32,
        busy_runners: u32,
        healthy_runners: u32,
    ) -> Self {
        Self {
            schema: ROUTE_RECEIPT_SCHEMA.to_string(),
            lane: RUST_SMALL_LANE.to_string(),
            target: CiRouteTarget::SelfHosted,
            reason: CiRouteReason::TrustedCapacityAvailable,
            trusted_event: context.trusted_event,
            event_name: context.event_name,
            repo: context.repo,
            head_sha: context.head_sha,
            eligible_runners,
            busy_runners,
            healthy_runners,
            fallback_allowed: true,
            selected_runner_label: selected_runner_label.into(),
            selected_runner: Some(selected_runner.into()),
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn to_pretty_json(&self) -> Result<String> {
        validate_route_receipt(self)?;
        let body = serde_json::to_string_pretty(self)?;
        Ok(format!("{body}\n"))
    }
}

pub fn run(args: CiRouteArgs) -> Result<()> {
    let receipt = decide_route(&args)?;
    let body = receipt.to_pretty_json()?;

    if let Some(parent) = args.json.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("create route receipt dir {}", parent.display()))?;
    }
    fs::write(&args.json, body)
        .with_context(|| format!("write route receipt {}", args.json.display()))?;

    if let Some(github_output) = &args.github_output {
        append_github_output(github_output, &receipt, &args.json)?;
    }

    println!(
        "ci-route {}: target={} reason={} receipt={}",
        receipt.lane,
        receipt.target.as_str(),
        receipt.reason.as_str(),
        args.json.display()
    );

    Ok(())
}

pub fn decide_route(args: &CiRouteArgs) -> Result<CiRouteReceipt> {
    if args.lane != RUST_SMALL_LANE {
        bail!(
            "unsupported CI route lane `{}`; only `{}` is supported",
            args.lane,
            RUST_SMALL_LANE
        );
    }

    let context = route_context(args);
    let inputs = resolve_route_inputs(args);
    let mut receipt = if args.mode == CiRouteMode::ForceSelfHosted && !context.trusted_event {
        CiRouteReceipt::github_hosted_fallback(context, CiRouteReason::ManualForceSelfHostedDenied)
    } else if !context.trusted_event {
        let reason = if args.fork_pr {
            CiRouteReason::ForkPullRequest
        } else {
            CiRouteReason::UntrustedEvent
        };
        CiRouteReceipt::github_hosted_fallback(context, reason)
    } else if args.mode == CiRouteMode::ForceGithubHosted {
        CiRouteReceipt::github_hosted_fallback(context, CiRouteReason::ManualForceGithubHosted)
    } else if !runner_token_available(args) {
        CiRouteReceipt::github_hosted_fallback(context, CiRouteReason::RunnerTokenUnavailable)
    } else if !runner_api_available(args) {
        CiRouteReceipt::github_hosted_fallback(context, CiRouteReason::RunnerApiUnavailable)
    } else if inputs.health == CiRouteHealth::Stale {
        CiRouteReceipt::github_hosted_fallback(context, CiRouteReason::RunnerHealthStale)
    } else if inputs.health == CiRouteHealth::Quarantined {
        CiRouteReceipt::github_hosted_fallback(context, CiRouteReason::RunnerQuarantined)
    } else if inputs.low_disk {
        CiRouteReceipt::github_hosted_fallback(context, CiRouteReason::LowDisk)
    } else if inputs.low_scratch {
        CiRouteReceipt::github_hosted_fallback(context, CiRouteReason::LowScratch)
    } else if inputs.health == CiRouteHealth::Degraded {
        CiRouteReceipt::github_hosted_fallback(context, CiRouteReason::RunnerHealthDegraded)
    } else if inputs.health != CiRouteHealth::Healthy {
        CiRouteReceipt::github_hosted_fallback(context, CiRouteReason::UnknownState)
    } else if inputs.eligible_runners == 0
        || inputs.healthy_runners == 0
        || inputs.busy_runners >= inputs.healthy_runners
    {
        CiRouteReceipt::github_hosted_fallback(context, CiRouteReason::SelfHostedCapacityFull)
    } else if let Some(selected_runner) = &inputs.selected_runner {
        CiRouteReceipt::self_hosted(
            context,
            inputs.selected_runner_label.clone(),
            selected_runner.clone(),
            inputs.eligible_runners,
            inputs.busy_runners,
            inputs.healthy_runners,
        )
    } else {
        CiRouteReceipt::github_hosted_fallback(context, CiRouteReason::RunnerApiUnavailable)
    };

    if receipt.target == CiRouteTarget::GithubHosted {
        receipt.eligible_runners = inputs.eligible_runners;
        receipt.busy_runners = inputs.busy_runners;
        receipt.healthy_runners = inputs.healthy_runners;
    }
    receipt.warnings.extend(inputs.warnings);
    receipt.errors.extend(inputs.errors);
    validate_route_receipt(&receipt)?;
    Ok(receipt)
}

pub fn validate_route_receipt(receipt: &CiRouteReceipt) -> Result<()> {
    if receipt.schema != ROUTE_RECEIPT_SCHEMA {
        bail!(
            "route receipt schema mismatch: expected {}, got {}",
            ROUTE_RECEIPT_SCHEMA,
            receipt.schema
        );
    }

    if receipt.lane != RUST_SMALL_LANE {
        bail!(
            "route receipt lane mismatch: expected {}, got {}",
            RUST_SMALL_LANE,
            receipt.lane
        );
    }

    if receipt.target == CiRouteTarget::SelfHosted && !receipt.trusted_event {
        bail!("route receipt selected self-hosted for an untrusted event");
    }

    if receipt.target == CiRouteTarget::SelfHosted && receipt.selected_runner.is_none() {
        bail!("route receipt selected self-hosted without selected_runner");
    }

    if receipt.target == CiRouteTarget::GithubHosted
        && receipt.selected_runner_label != DEFAULT_GITHUB_HOSTED_LABEL
    {
        bail!(
            "route receipt selected GitHub-hosted with unexpected label {}",
            receipt.selected_runner_label
        );
    }

    for (field, value) in receipt_strings(receipt) {
        if looks_like_absolute_path(value) {
            bail!("route receipt field {field} contains an absolute path");
        }
        if looks_like_secret(value) {
            bail!("route receipt field {field} contains a secret-looking value");
        }
    }

    Ok(())
}

impl CiRouteTarget {
    fn as_str(self) -> &'static str {
        match self {
            Self::SelfHosted => "self-hosted",
            Self::GithubHosted => "github-hosted",
            Self::None => "none",
        }
    }
}

impl CiRouteReason {
    fn as_str(self) -> &'static str {
        match self {
            Self::TrustedCapacityAvailable => "trusted_capacity_available",
            Self::ForkPullRequest => "fork_pull_request",
            Self::UntrustedEvent => "untrusted_event",
            Self::RunnerApiUnavailable => "runner_api_unavailable",
            Self::RunnerTokenUnavailable => "runner_token_unavailable",
            Self::RunnerHealthStale => "runner_health_stale",
            Self::RunnerHealthDegraded => "runner_health_degraded",
            Self::SelfHostedCapacityFull => "self_hosted_capacity_full",
            Self::LowDisk => "low_disk",
            Self::LowScratch => "low_scratch",
            Self::RunnerQuarantined => "runner_quarantined",
            Self::RouteBudgetExhausted => "route_budget_exhausted",
            Self::ManualForceGithubHosted => "manual_force_github_hosted",
            Self::ManualForceSelfHostedDenied => "manual_force_self_hosted_denied",
            Self::UnknownState => "unknown_state",
        }
    }
}

fn route_context(args: &CiRouteArgs) -> CiRouteContext {
    let event_name = args
        .event_name
        .clone()
        .or_else(|| env_non_empty("GITHUB_EVENT_NAME"))
        .unwrap_or_else(|| "unknown".to_string());
    let trusted_event = args
        .trusted_event
        .unwrap_or_else(|| infer_trusted_event(&event_name));
    CiRouteContext::new(
        event_name,
        args.repo
            .clone()
            .or_else(|| env_non_empty("GITHUB_REPOSITORY"))
            .unwrap_or_else(|| "unknown".to_string()),
        args.head_sha
            .clone()
            .or_else(|| env_non_empty("GITHUB_SHA"))
            .unwrap_or_else(|| "unknown".to_string()),
        trusted_event,
    )
}

fn infer_trusted_event(event_name: &str) -> bool {
    matches!(event_name, "workflow_dispatch" | "merge_group" | "push")
}

fn runner_token_available(args: &CiRouteArgs) -> bool {
    args.runner_token_available.unwrap_or_else(|| {
        env_non_empty("GITHUB_TOKEN").is_some()
            || env_non_empty("GH_TOKEN").is_some()
            || env_non_empty("GH_PAT").is_some()
    })
}

fn runner_api_available(args: &CiRouteArgs) -> bool {
    args.runner_api_available.unwrap_or(false)
}

#[derive(Debug, Clone)]
struct RouteInputs {
    eligible_runners: u32,
    busy_runners: u32,
    healthy_runners: u32,
    health: CiRouteHealth,
    low_disk: bool,
    low_scratch: bool,
    selected_runner_label: String,
    selected_runner: Option<String>,
    warnings: Vec<String>,
    errors: Vec<String>,
}

fn resolve_route_inputs(args: &CiRouteArgs) -> RouteInputs {
    let mut inputs = RouteInputs {
        eligible_runners: args.eligible_runners,
        busy_runners: args.busy_runners,
        healthy_runners: args.healthy_runners,
        health: args.health,
        low_disk: args.low_disk,
        low_scratch: args.low_scratch,
        selected_runner_label: args.selected_runner_label.clone(),
        selected_runner: args.selected_runner.clone(),
        warnings: Vec::new(),
        errors: Vec::new(),
    };

    let Some(health_json) = &args.health_json else {
        return inputs;
    };

    let receipt = match read_runner_health_receipt(health_json) {
        Ok(receipt) => receipt,
        Err(_) => {
            inputs.health = CiRouteHealth::Unknown;
            inputs
                .warnings
                .push("runner_health_receipt_unavailable".to_string());
            return inputs;
        }
    };

    apply_health_receipt(args, &mut inputs, &receipt);
    inputs
}

fn apply_health_receipt(
    args: &CiRouteArgs,
    inputs: &mut RouteInputs,
    receipt: &CiRunnerHealthReceipt,
) {
    if is_health_stale(
        receipt.generated_at_ms,
        args.now_ms.unwrap_or_else(now_ms),
        args.health_max_age_seconds,
    ) {
        inputs.health = CiRouteHealth::Stale;
        inputs
            .warnings
            .push("runner_health_receipt_stale".to_string());
    } else {
        inputs.health = match receipt.status {
            CiRunnerHealthStatus::Healthy => CiRouteHealth::Healthy,
            CiRunnerHealthStatus::Degraded => CiRouteHealth::Degraded,
            CiRunnerHealthStatus::Quarantined => CiRouteHealth::Quarantined,
        };
    }

    inputs.low_disk |= receipt
        .disk_free_bytes
        .is_some_and(|free| free < receipt.min_free_bytes);
    inputs.low_scratch |= receipt
        .scratch_free_bytes
        .is_some_and(|free| free < receipt.min_free_bytes);

    inputs.eligible_runners = inputs.eligible_runners.max(1);
    if receipt.status == CiRunnerHealthStatus::Healthy && inputs.health == CiRouteHealth::Healthy {
        inputs.healthy_runners = inputs.healthy_runners.max(1);
    }
    inputs.selected_runner = inputs
        .selected_runner
        .clone()
        .or_else(|| Some(receipt.runner_name.clone()));
}

fn is_health_stale(generated_at_ms: u128, now_ms: u128, max_age_seconds: u64) -> bool {
    let max_age_ms = u128::from(max_age_seconds) * 1_000;
    now_ms.saturating_sub(generated_at_ms) > max_age_ms
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

fn append_github_output(path: &Path, receipt: &CiRouteReceipt, json_path: &Path) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("create GitHub output dir {}", parent.display()))?;
    }

    let mut body = String::new();
    body.push_str(&format!("target={}\n", receipt.target.as_str()));
    body.push_str(&format!("reason={}\n", receipt.reason.as_str()));
    body.push_str(&format!(
        "selected_runner_label={}\n",
        receipt.selected_runner_label
    ));
    body.push_str(&format!("receipt_path={}\n", json_path.display()));
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("open GitHub output {}", path.display()))?
        .write_all(body.as_bytes())
        .with_context(|| format!("append GitHub output {}", path.display()))?;
    Ok(())
}

fn receipt_strings(receipt: &CiRouteReceipt) -> Vec<(&'static str, &str)> {
    let mut values = vec![
        ("schema", receipt.schema.as_str()),
        ("lane", receipt.lane.as_str()),
        ("event_name", receipt.event_name.as_str()),
        ("repo", receipt.repo.as_str()),
        ("head_sha", receipt.head_sha.as_str()),
        (
            "selected_runner_label",
            receipt.selected_runner_label.as_str(),
        ),
    ];

    if let Some(selected_runner) = &receipt.selected_runner {
        values.push(("selected_runner", selected_runner.as_str()));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::ci_runner_health::ToolHealth;
    use std::fs;

    fn trusted_context() -> CiRouteContext {
        CiRouteContext::new(
            "pull_request",
            "EffortlessMetrics/tokmd-swarm",
            "abc123",
            true,
        )
    }

    #[test]
    fn github_hosted_fallback_receipt_is_deterministic() {
        let receipt = CiRouteReceipt::github_hosted_fallback(
            trusted_context(),
            CiRouteReason::SelfHostedCapacityFull,
        );

        let json = receipt.to_pretty_json().expect("json");

        assert_eq!(
            json,
            "{\n  \"schema\": \"tokmd.ci_route.v1\",\n  \"lane\": \"rust-small\",\n  \"target\": \"github-hosted\",\n  \"reason\": \"self_hosted_capacity_full\",\n  \"trusted_event\": true,\n  \"event_name\": \"pull_request\",\n  \"repo\": \"EffortlessMetrics/tokmd-swarm\",\n  \"head_sha\": \"abc123\",\n  \"eligible_runners\": 0,\n  \"busy_runners\": 0,\n  \"healthy_runners\": 0,\n  \"fallback_allowed\": true,\n  \"selected_runner_label\": \"ubuntu-24.04\",\n  \"selected_runner\": null,\n  \"warnings\": [],\n  \"errors\": []\n}\n"
        );
    }

    #[test]
    fn unknown_state_falls_back_to_github_hosted() {
        let receipt =
            CiRouteReceipt::github_hosted_fallback(trusted_context(), CiRouteReason::UnknownState);

        assert_eq!(receipt.target, CiRouteTarget::GithubHosted);
        assert_eq!(receipt.reason, CiRouteReason::UnknownState);
        assert_eq!(receipt.selected_runner_label, DEFAULT_GITHUB_HOSTED_LABEL);
        assert!(validate_route_receipt(&receipt).is_ok());
    }

    #[test]
    fn reason_enum_uses_stable_snake_case() {
        let value = serde_json::to_value(CiRouteReason::RunnerApiUnavailable).expect("reason");

        assert_eq!(value, serde_json::json!("runner_api_unavailable"));
    }

    #[test]
    fn rejects_untrusted_self_hosted_route() {
        let context = CiRouteContext::new(
            "pull_request",
            "EffortlessMetrics/tokmd-swarm",
            "abc123",
            false,
        );
        let receipt = CiRouteReceipt::self_hosted(context, "em-ci-small", "CPX42", 1, 0, 1);

        let err = validate_route_receipt(&receipt).expect_err("untrusted self-hosted");

        assert!(
            err.to_string().contains("untrusted event"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn rejects_secret_like_values() {
        let mut receipt = CiRouteReceipt::github_hosted_fallback(
            trusted_context(),
            CiRouteReason::RunnerApiUnavailable,
        );
        receipt
            .warnings
            .push("authorization: bearer ghp_example".to_string());

        let err = validate_route_receipt(&receipt).expect_err("secret-looking value");

        assert!(
            err.to_string().contains("secret-looking"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn rejects_absolute_machine_paths() {
        let mut receipt = CiRouteReceipt::github_hosted_fallback(
            trusted_context(),
            CiRouteReason::RunnerHealthDegraded,
        );
        receipt
            .warnings
            .push("scratch check read C:/ci-scratch/state.json".to_string());

        let err = validate_route_receipt(&receipt).expect_err("absolute path");

        assert!(
            err.to_string().contains("absolute path"),
            "unexpected error: {err}"
        );
    }

    fn route_args() -> CiRouteArgs {
        CiRouteArgs {
            event_name: Some("pull_request".to_string()),
            repo: Some("EffortlessMetrics/tokmd-swarm".to_string()),
            head_sha: Some("abc123".to_string()),
            runner_api_available: Some(true),
            runner_token_available: Some(true),
            ..CiRouteArgs::default()
        }
    }

    fn health_receipt(
        status: CiRunnerHealthStatus,
        generated_at_ms: u128,
    ) -> CiRunnerHealthReceipt {
        CiRunnerHealthReceipt {
            schema: crate::tasks::ci_runner_health::RUNNER_HEALTH_SCHEMA.to_string(),
            runner_name: "CPX42".to_string(),
            labels: vec![
                "em-ci-small".to_string(),
                "linux".to_string(),
                "self-hosted".to_string(),
            ],
            generated_at_ms,
            status,
            reason: status.as_str().to_string(),
            disk_free_bytes: Some(16 * 1024 * 1024 * 1024),
            scratch_free_bytes: Some(16 * 1024 * 1024 * 1024),
            min_free_bytes: 8 * 1024 * 1024 * 1024,
            rustc: ToolHealth {
                available: true,
                version: Some("rustc 1.95.0".to_string()),
            },
            git: ToolHealth {
                available: true,
                version: Some("git version 2.50.0".to_string()),
            },
            docker: ToolHealth {
                available: false,
                version: None,
            },
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn write_health_receipt(
        dir: &tempfile::TempDir,
        receipt: &CiRunnerHealthReceipt,
    ) -> std::path::PathBuf {
        let path = dir.path().join("runner-health.json");
        fs::write(
            &path,
            serde_json::to_string_pretty(receipt).expect("health json"),
        )
        .expect("write health");
        path
    }

    #[test]
    fn same_repo_pr_can_select_self_hosted_when_capacity_is_healthy() {
        let receipt = decide_route(&CiRouteArgs {
            trusted_event: Some(true),
            eligible_runners: 2,
            busy_runners: 1,
            healthy_runners: 2,
            health: CiRouteHealth::Healthy,
            selected_runner: Some("CPX42".to_string()),
            ..route_args()
        })
        .expect("route");

        assert_eq!(receipt.target, CiRouteTarget::SelfHosted);
        assert_eq!(receipt.reason, CiRouteReason::TrustedCapacityAvailable);
        assert_eq!(receipt.selected_runner.as_deref(), Some("CPX42"));
    }

    #[test]
    fn fresh_healthy_health_receipt_can_select_self_hosted() {
        let dir = tempfile::tempdir().expect("tempdir");
        let health_json = write_health_receipt(
            &dir,
            &health_receipt(CiRunnerHealthStatus::Healthy, 1_700_000_000_000),
        );

        let receipt = decide_route(&CiRouteArgs {
            trusted_event: Some(true),
            health_json: Some(health_json),
            now_ms: Some(1_700_000_000_500),
            ..route_args()
        })
        .expect("route");

        assert_eq!(receipt.target, CiRouteTarget::SelfHosted);
        assert_eq!(receipt.reason, CiRouteReason::TrustedCapacityAvailable);
        assert_eq!(receipt.eligible_runners, 1);
        assert_eq!(receipt.healthy_runners, 1);
        assert_eq!(receipt.selected_runner.as_deref(), Some("CPX42"));
    }

    #[test]
    fn stale_health_receipt_routes_to_github_hosted() {
        let dir = tempfile::tempdir().expect("tempdir");
        let health_json = write_health_receipt(
            &dir,
            &health_receipt(CiRunnerHealthStatus::Healthy, 1_700_000_000_000),
        );

        let receipt = decide_route(&CiRouteArgs {
            trusted_event: Some(true),
            health_json: Some(health_json),
            now_ms: Some(1_700_000_901_001),
            ..route_args()
        })
        .expect("route");

        assert_eq!(receipt.target, CiRouteTarget::GithubHosted);
        assert_eq!(receipt.reason, CiRouteReason::RunnerHealthStale);
        assert!(
            receipt
                .warnings
                .contains(&"runner_health_receipt_stale".to_string())
        );
    }

    #[test]
    fn degraded_health_receipt_routes_to_github_hosted() {
        let dir = tempfile::tempdir().expect("tempdir");
        let health_json = write_health_receipt(
            &dir,
            &health_receipt(CiRunnerHealthStatus::Degraded, 1_700_000_000_000),
        );

        let receipt = decide_route(&CiRouteArgs {
            trusted_event: Some(true),
            health_json: Some(health_json),
            now_ms: Some(1_700_000_000_500),
            ..route_args()
        })
        .expect("route");

        assert_eq!(receipt.target, CiRouteTarget::GithubHosted);
        assert_eq!(receipt.reason, CiRouteReason::RunnerHealthDegraded);
        assert_eq!(receipt.eligible_runners, 1);
        assert_eq!(receipt.healthy_runners, 0);
    }

    #[test]
    fn low_scratch_health_receipt_uses_low_scratch_reason() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut health = health_receipt(CiRunnerHealthStatus::Degraded, 1_700_000_000_000);
        health.reason = "scratch_free_below_guard".to_string();
        health.scratch_free_bytes = Some(1024);
        let health_json = write_health_receipt(&dir, &health);

        let receipt = decide_route(&CiRouteArgs {
            trusted_event: Some(true),
            health_json: Some(health_json),
            now_ms: Some(1_700_000_000_500),
            ..route_args()
        })
        .expect("route");

        assert_eq!(receipt.target, CiRouteTarget::GithubHosted);
        assert_eq!(receipt.reason, CiRouteReason::LowScratch);
        assert_eq!(receipt.eligible_runners, 1);
        assert_eq!(receipt.healthy_runners, 0);
    }

    #[test]
    fn quarantined_health_receipt_routes_to_github_hosted() {
        let dir = tempfile::tempdir().expect("tempdir");
        let health_json = write_health_receipt(
            &dir,
            &health_receipt(CiRunnerHealthStatus::Quarantined, 1_700_000_000_000),
        );

        let receipt = decide_route(&CiRouteArgs {
            trusted_event: Some(true),
            health_json: Some(health_json),
            now_ms: Some(1_700_000_000_500),
            ..route_args()
        })
        .expect("route");

        assert_eq!(receipt.target, CiRouteTarget::GithubHosted);
        assert_eq!(receipt.reason, CiRouteReason::RunnerQuarantined);
    }

    #[test]
    fn unreadable_health_receipt_falls_back_to_unknown_state() {
        let dir = tempfile::tempdir().expect("tempdir");
        let health_json = dir.path().join("missing-health.json");

        let receipt = decide_route(&CiRouteArgs {
            trusted_event: Some(true),
            health_json: Some(health_json),
            ..route_args()
        })
        .expect("route");

        assert_eq!(receipt.target, CiRouteTarget::GithubHosted);
        assert_eq!(receipt.reason, CiRouteReason::UnknownState);
        assert!(
            receipt
                .warnings
                .contains(&"runner_health_receipt_unavailable".to_string())
        );
    }

    #[test]
    fn fork_pr_routes_to_github_hosted() {
        let receipt = decide_route(&CiRouteArgs {
            trusted_event: Some(false),
            fork_pr: true,
            eligible_runners: 2,
            healthy_runners: 2,
            health: CiRouteHealth::Healthy,
            selected_runner: Some("CPX42".to_string()),
            ..route_args()
        })
        .expect("route");

        assert_eq!(receipt.target, CiRouteTarget::GithubHosted);
        assert_eq!(receipt.reason, CiRouteReason::ForkPullRequest);
    }

    #[test]
    fn missing_token_routes_to_github_hosted() {
        let receipt = decide_route(&CiRouteArgs {
            trusted_event: Some(true),
            runner_token_available: Some(false),
            eligible_runners: 2,
            healthy_runners: 2,
            health: CiRouteHealth::Healthy,
            selected_runner: Some("CPX42".to_string()),
            ..route_args()
        })
        .expect("route");

        assert_eq!(receipt.target, CiRouteTarget::GithubHosted);
        assert_eq!(receipt.reason, CiRouteReason::RunnerTokenUnavailable);
    }

    #[test]
    fn api_unavailable_routes_to_github_hosted() {
        let receipt = decide_route(&CiRouteArgs {
            trusted_event: Some(true),
            runner_api_available: Some(false),
            eligible_runners: 2,
            healthy_runners: 2,
            health: CiRouteHealth::Healthy,
            selected_runner: Some("CPX42".to_string()),
            ..route_args()
        })
        .expect("route");

        assert_eq!(receipt.target, CiRouteTarget::GithubHosted);
        assert_eq!(receipt.reason, CiRouteReason::RunnerApiUnavailable);
    }

    #[test]
    fn force_github_hosted_routes_to_github_hosted() {
        let receipt = decide_route(&CiRouteArgs {
            trusted_event: Some(true),
            mode: CiRouteMode::ForceGithubHosted,
            eligible_runners: 2,
            healthy_runners: 2,
            health: CiRouteHealth::Healthy,
            selected_runner: Some("CPX42".to_string()),
            ..route_args()
        })
        .expect("route");

        assert_eq!(receipt.target, CiRouteTarget::GithubHosted);
        assert_eq!(receipt.reason, CiRouteReason::ManualForceGithubHosted);
    }

    #[test]
    fn force_self_hosted_is_denied_for_untrusted_events() {
        let receipt = decide_route(&CiRouteArgs {
            trusted_event: Some(false),
            mode: CiRouteMode::ForceSelfHosted,
            eligible_runners: 2,
            healthy_runners: 2,
            health: CiRouteHealth::Healthy,
            selected_runner: Some("CPX42".to_string()),
            ..route_args()
        })
        .expect("route");

        assert_eq!(receipt.target, CiRouteTarget::GithubHosted);
        assert_eq!(receipt.reason, CiRouteReason::ManualForceSelfHostedDenied);
    }

    #[test]
    fn run_writes_receipt_and_github_outputs() {
        let dir = tempfile::tempdir().expect("tempdir");
        let json = dir.path().join("route.json");
        let github_output = dir.path().join("github-output.txt");

        run(CiRouteArgs {
            json: json.clone(),
            github_output: Some(github_output.clone()),
            trusted_event: Some(false),
            fork_pr: true,
            ..route_args()
        })
        .expect("run route helper");

        let receipt: CiRouteReceipt =
            serde_json::from_str(&fs::read_to_string(json).expect("read receipt"))
                .expect("parse receipt");
        assert_eq!(receipt.reason, CiRouteReason::ForkPullRequest);
        assert!(
            fs::read_to_string(github_output)
                .expect("read outputs")
                .contains("target=github-hosted")
        );
    }

    #[test]
    fn unsupported_lane_fails() {
        let err = decide_route(&CiRouteArgs {
            lane: "release".to_string(),
            ..route_args()
        })
        .expect_err("unsupported lane");

        assert!(
            err.to_string().contains("unsupported CI route lane"),
            "unexpected error: {err}"
        );
    }
}
