#![allow(dead_code)]

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

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
}
