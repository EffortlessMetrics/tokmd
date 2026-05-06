use crate::cli::{ProofArgs, ProofProfile};
use crate::proof::policy_ast::ProofPolicy;
use crate::tasks::affected::{affected_report, changed_files, load_checked_policy};
use anyhow::{Result, bail};
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Debug, Serialize)]
struct ProofPlanReport {
    schema: String,
    ok: bool,
    profile: String,
    base: String,
    head: String,
    changed_files: Vec<String>,
    commands: Vec<ProofPlanCommand>,
    unknown_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
struct ProofPlanCommand {
    scope: String,
    kind: String,
    command: String,
}

pub fn run(args: ProofArgs) -> Result<()> {
    if !args.plan {
        bail!("proof execution is not implemented yet; pass --plan to print the proof plan");
    }

    let policy = load_checked_policy(&args.policy)?;
    let report = proof_plan_report(&policy, &args)?;
    println!("{}", serde_json::to_string_pretty(&report)?);

    if report.ok {
        Ok(())
    } else {
        bail!(
            "proof plan has {} unknown file(s) that need scope policy",
            report.unknown_files.len()
        )
    }
}

fn proof_plan_report(policy: &ProofPolicy, args: &ProofArgs) -> Result<ProofPlanReport> {
    match args.profile {
        ProofProfile::Affected => affected_plan_report(policy, args),
        profile => Ok(static_plan_report(profile, &args.base, &args.head)),
    }
}

fn affected_plan_report(policy: &ProofPolicy, args: &ProofArgs) -> Result<ProofPlanReport> {
    let changed_files = changed_files(&args.base, &args.head)?;
    let affected = affected_report(policy, &args.base, &args.head, changed_files)?;
    let mut commands = Vec::new();

    for scope in &affected.scopes {
        for command in &scope.proof {
            commands.push(ProofPlanCommand {
                scope: scope.name.clone(),
                kind: "proof".to_string(),
                command: command.clone(),
            });
        }
    }

    Ok(ProofPlanReport {
        schema: "tokmd.proof_plan.v1".to_string(),
        ok: affected.ok,
        profile: profile_name(args.profile).to_string(),
        base: affected.base,
        head: affected.head,
        changed_files: affected.changed_files,
        commands: dedupe_commands(commands),
        unknown_files: affected.unknown_files,
    })
}

fn static_plan_report(profile: ProofProfile, base: &str, head: &str) -> ProofPlanReport {
    ProofPlanReport {
        schema: "tokmd.proof_plan.v1".to_string(),
        ok: true,
        profile: profile_name(profile).to_string(),
        base: base.to_string(),
        head: head.to_string(),
        changed_files: Vec::new(),
        commands: static_profile_commands(profile),
        unknown_files: Vec::new(),
    }
}

fn static_profile_commands(profile: ProofProfile) -> Vec<ProofPlanCommand> {
    let commands = match profile {
        ProofProfile::Fast => vec![
            command("workspace", "format", "cargo fmt-check"),
            command("proof_policy", "policy", "cargo xtask proof-policy --check"),
            command(
                "fixture_blobs",
                "guardrail",
                "cargo xtask fixture-blobs-check",
            ),
            command("boundaries", "guardrail", "cargo xtask boundaries-check"),
        ],
        ProofProfile::Release => vec![
            command("docs", "docs", "cargo xtask docs --check"),
            command("version", "release", "cargo xtask version-consistency"),
            command(
                "publish_surface",
                "release",
                "cargo xtask publish-surface --json --verify-publish",
            ),
            command(
                "dependencies",
                "security",
                "cargo deny --all-features check",
            ),
        ],
        ProofProfile::Deep => vec![
            command("workspace", "test", "cargo test --workspace"),
            command("coverage", "coverage", "cargo llvm-cov --workspace --lcov"),
            command("mutation", "mutation", "cargo mutants --timeout 300"),
            command("fuzz", "fuzz", "cargo +nightly fuzz list"),
        ],
        ProofProfile::Affected => Vec::new(),
    };

    dedupe_commands(commands)
}

fn command(scope: &str, kind: &str, command: &str) -> ProofPlanCommand {
    ProofPlanCommand {
        scope: scope.to_string(),
        kind: kind.to_string(),
        command: command.to_string(),
    }
}

fn dedupe_commands(commands: Vec<ProofPlanCommand>) -> Vec<ProofPlanCommand> {
    commands
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn profile_name(profile: ProofProfile) -> &'static str {
    match profile {
        ProofProfile::Fast => "fast",
        ProofProfile::Affected => "affected",
        ProofProfile::Release => "release",
        ProofProfile::Deep => "deep",
    }
}

#[cfg(test)]
mod tests {
    use super::{dedupe_commands, static_profile_commands};
    use crate::cli::ProofProfile;

    #[test]
    fn static_profiles_have_deterministic_commands() {
        let fast = static_profile_commands(ProofProfile::Fast);

        assert!(!fast.is_empty());
        assert_eq!(fast, dedupe_commands(fast.clone()));
        assert!(fast.iter().any(|cmd| cmd.command == "cargo fmt-check"));
    }

    #[test]
    fn release_profile_includes_release_facing_checks() {
        let release = static_profile_commands(ProofProfile::Release);

        assert!(
            release
                .iter()
                .any(|cmd| cmd.command.contains("docs --check"))
        );
        assert!(
            release
                .iter()
                .any(|cmd| cmd.command.contains("version-consistency"))
        );
        assert!(
            release
                .iter()
                .any(|cmd| cmd.command.contains("publish-surface"))
        );
    }

    #[test]
    fn deep_profile_includes_heavy_evidence_commands() {
        let deep = static_profile_commands(ProofProfile::Deep);

        assert!(deep.iter().any(|cmd| cmd.kind == "coverage"));
        assert!(deep.iter().any(|cmd| cmd.kind == "mutation"));
        assert!(deep.iter().any(|cmd| cmd.kind == "fuzz"));
    }
}
