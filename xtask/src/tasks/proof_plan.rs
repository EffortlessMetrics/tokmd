use crate::cli::{ProofArgs, ProofProfile};
use crate::proof::policy_ast::ProofPolicy;
use crate::tasks::affected::{
    AffectedReport, AffectedScope, affected_report, changed_files, load_checked_policy,
};
use anyhow::{Result, bail};
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

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
    required: bool,
    command: String,
}

pub fn run(args: ProofArgs) -> Result<()> {
    if !args.plan {
        bail!("proof execution is not implemented yet; pass --plan to print the proof plan");
    }

    let policy = load_checked_policy(&args.policy)?;
    let report = proof_plan_report(&policy, &args)?;
    if let Some(path) = &args.summary_md {
        write_markdown_summary(path, &report)?;
    }
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
    let commands = affected_commands(policy, &affected);

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

fn affected_commands(policy: &ProofPolicy, affected: &AffectedReport) -> Vec<ProofPlanCommand> {
    let mut commands = Vec::new();

    for scope in &affected.scopes {
        for command in &scope.proof {
            commands.push(command_for_scope(scope, "proof", command));
        }

        if let Some(command) = coverage_command(policy, scope) {
            commands.push(command);
        }

        commands.extend(mutation_commands(policy, scope));
    }

    dedupe_commands(commands)
}

fn coverage_command(policy: &ProofPolicy, scope: &AffectedScope) -> Option<ProofPlanCommand> {
    if !scope.coverage || !matches!(scope.kind, crate::proof::policy_ast::ScopeKind::Rust) {
        return None;
    }

    let packages = sorted(scope.packages.clone());
    if packages.is_empty() {
        return None;
    }

    let package_flags = packages
        .iter()
        .map(|package| format!("-p {package}"))
        .collect::<Vec<_>>()
        .join(" ");
    let tool = coverage_command_tool(policy);
    let output_path = format!("target/proof/coverage/{}.lcov", artifact_name(&scope.name));
    let command =
        format!("{tool} {package_flags} --all-features --lcov --output-path {output_path}");

    Some(advisory_command_for_scope(scope, "coverage", &command))
}

fn mutation_commands(policy: &ProofPolicy, scope: &AffectedScope) -> Vec<ProofPlanCommand> {
    if !scope.mutation || !matches!(scope.kind, crate::proof::policy_ast::ScopeKind::Rust) {
        return Vec::new();
    }

    let timeout = policy.defaults.mutation_timeout_seconds.unwrap_or(300);
    let mut commands = scope
        .matched_files
        .iter()
        .filter(|file| is_mutation_candidate(file))
        .map(|file| {
            advisory_command_for_scope(
                scope,
                "mutation",
                &format!("cargo mutants --file {file} --timeout {timeout}"),
            )
        })
        .collect::<Vec<_>>();

    if commands.is_empty() {
        commands.extend(package_mutation_command(scope, timeout));
    }

    commands
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
        required: true,
        command: command.to_string(),
    }
}

fn advisory_command_for_scope(
    scope: &AffectedScope,
    kind: &str,
    command: &str,
) -> ProofPlanCommand {
    ProofPlanCommand {
        scope: scope.name.clone(),
        kind: kind.to_string(),
        required: false,
        command: command.to_string(),
    }
}

fn command_for_scope(scope: &AffectedScope, kind: &str, command_text: &str) -> ProofPlanCommand {
    command(&scope.name, kind, command_text)
}

fn dedupe_commands(commands: Vec<ProofPlanCommand>) -> Vec<ProofPlanCommand> {
    let mut commands = commands
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    commands.sort_by(compare_commands);
    commands
}

fn write_markdown_summary(path: &Path, report: &ProofPlanReport) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, render_markdown_summary(report))?;
    Ok(())
}

fn render_markdown_summary(report: &ProofPlanReport) -> String {
    let mut out = String::new();

    out.push_str("## Proof Plan Summary\n\n");
    out.push_str("| Field | Value |\n");
    out.push_str("| --- | --- |\n");
    out.push_str(&format!("| Profile | `{}` |\n", escape_md(&report.profile)));
    out.push_str(&format!("| Base | `{}` |\n", escape_md(&report.base)));
    out.push_str(&format!("| Head | `{}` |\n", escape_md(&report.head)));
    out.push_str(&format!("| OK | `{}` |\n", report.ok));
    out.push_str(&format!(
        "| Changed files | {} |\n",
        report.changed_files.len()
    ));
    out.push_str(&format!(
        "| Unknown files | {} |\n",
        report.unknown_files.len()
    ));
    out.push_str(&format!("| Commands | {} |\n", report.commands.len()));
    out.push('\n');
    out.push_str(
        "Required commands are the current proof selection. Advisory commands are planned evidence candidates and are not CI gates yet.\n\n",
    );

    if report.commands.is_empty() {
        out.push_str("No proof commands planned.\n");
    } else {
        out.push_str("### Command Counts\n\n");
        out.push_str("| Kind | Required | Count |\n");
        out.push_str("| --- | --- | ---: |\n");
        for ((kind, required), count) in command_counts(report) {
            out.push_str(&format!(
                "| `{}` | `{}` | {} |\n",
                escape_md(&kind),
                required,
                count
            ));
        }

        out.push_str("\n### Commands\n\n");
        out.push_str("| Scope | Kind | Required | Command |\n");
        out.push_str("| --- | --- | --- | --- |\n");
        for command in &report.commands {
            out.push_str(&format!(
                "| `{}` | `{}` | `{}` | `{}` |\n",
                escape_md(&command.scope),
                escape_md(&command.kind),
                command.required,
                escape_md(&command.command)
            ));
        }
    }

    if !report.unknown_files.is_empty() {
        out.push_str("\n### Unknown Files\n\n");
        for file in &report.unknown_files {
            out.push_str(&format!("- `{}`\n", escape_md(file)));
        }
    }

    out
}

fn command_counts(report: &ProofPlanReport) -> BTreeMap<(String, bool), usize> {
    let mut counts = BTreeMap::new();
    for command in &report.commands {
        *counts
            .entry((command.kind.clone(), command.required))
            .or_insert(0) += 1;
    }
    counts
}

fn escape_md(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

fn compare_commands(left: &ProofPlanCommand, right: &ProofPlanCommand) -> Ordering {
    left.scope
        .cmp(&right.scope)
        .then_with(|| kind_rank(&left.kind).cmp(&kind_rank(&right.kind)))
        .then_with(|| left.kind.cmp(&right.kind))
        .then_with(|| left.command.cmp(&right.command))
}

fn kind_rank(kind: &str) -> u8 {
    match kind {
        "proof" => 0,
        "coverage" => 1,
        "mutation" => 2,
        "fuzz" => 3,
        _ => 4,
    }
}

fn coverage_command_tool(policy: &ProofPolicy) -> &str {
    match policy.tools.coverage.as_deref() {
        Some("cargo-llvm-cov") | None => "cargo llvm-cov",
        Some(tool) => tool,
    }
}

fn sorted(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn package_mutation_command(scope: &AffectedScope, timeout: u64) -> Option<ProofPlanCommand> {
    let packages = sorted(scope.packages.clone());
    if packages.is_empty() {
        return None;
    }

    let package_flags = packages
        .iter()
        .map(|package| format!("-p {package}"))
        .collect::<Vec<_>>()
        .join(" ");
    Some(advisory_command_for_scope(
        scope,
        "mutation",
        &format!("cargo mutants {package_flags} --timeout {timeout}"),
    ))
}

fn artifact_name(name: &str) -> String {
    name.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn is_mutation_candidate(path: &str) -> bool {
    path.ends_with(".rs")
        && !path.starts_with("fuzz/")
        && !path.contains("/tests/")
        && !path.contains("/benches/")
        && !path.contains("/examples/")
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
    use super::{
        ProofPlanCommand, ProofPlanReport, affected_commands, dedupe_commands,
        is_mutation_candidate, render_markdown_summary, static_profile_commands,
    };
    use crate::cli::ProofProfile;
    use crate::proof::policy::parse_policy_str;
    use crate::tasks::affected::{AffectedReport, AffectedScope};

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

    #[test]
    fn affected_plan_adds_scoped_coverage_and_mutation_commands() {
        let policy = parse_policy_str(
            r#"
schema = "tokmd.proof_policy.v1"

[defaults]
mutation_timeout_seconds = 123

[tools]
coverage = "cargo-llvm-cov"
"#,
        )
        .expect("policy should parse");
        let affected = AffectedReport {
            schema: "tokmd.affected.v1".to_string(),
            ok: true,
            base: "base".to_string(),
            head: "head".to_string(),
            changed_files: vec![
                "crates/tokmd-core/src/ffi.rs".to_string(),
                "crates/tokmd-core/tests/ffi.rs".to_string(),
            ],
            scopes: vec![AffectedScope {
                name: "tokmd_core_ffi".to_string(),
                kind: crate::proof::policy_ast::ScopeKind::Rust,
                reason: "matched crates/tokmd-core/src/ffi.rs".to_string(),
                matched_files: vec![
                    "crates/tokmd-core/src/ffi.rs".to_string(),
                    "crates/tokmd-core/tests/ffi.rs".to_string(),
                ],
                packages: vec!["tokmd-core".to_string()],
                proof: vec!["cargo test -p tokmd-core ffi".to_string()],
                mutation: true,
                coverage: true,
            }],
            unknown_files: Vec::new(),
        };

        let commands = affected_commands(&policy, &affected);

        assert_eq!(commands[0].kind, "proof");
        assert!(commands[0].required);
        assert!(
            commands
                .iter()
                .any(|cmd| cmd.command == "cargo test -p tokmd-core ffi")
        );
        assert!(commands.iter().any(|cmd| {
            cmd.kind == "coverage"
                && !cmd.required
                && cmd.command == "cargo llvm-cov -p tokmd-core --all-features --lcov --output-path target/proof/coverage/tokmd_core_ffi.lcov"
        }));
        assert!(commands.iter().any(|cmd| {
            cmd.kind == "mutation"
                && !cmd.required
                && cmd.command == "cargo mutants --file crates/tokmd-core/src/ffi.rs --timeout 123"
        }));
        assert!(
            !commands
                .iter()
                .any(|cmd| cmd.command.contains("crates/tokmd-core/tests/ffi.rs"))
        );
    }

    #[test]
    fn markdown_summary_marks_advisory_evidence_commands() {
        let report = ProofPlanReport {
            schema: "tokmd.proof_plan.v1".to_string(),
            ok: true,
            profile: "affected".to_string(),
            base: "origin/main".to_string(),
            head: "HEAD".to_string(),
            changed_files: vec!["crates/tokmd-core/src/ffi.rs".to_string()],
            commands: vec![
                ProofPlanCommand {
                    scope: "tokmd_core_ffi".to_string(),
                    kind: "proof".to_string(),
                    required: true,
                    command: "cargo test -p tokmd-core ffi".to_string(),
                },
                ProofPlanCommand {
                    scope: "tokmd_core_ffi".to_string(),
                    kind: "coverage".to_string(),
                    required: false,
                    command: "cargo llvm-cov -p tokmd-core --all-features --lcov".to_string(),
                },
                ProofPlanCommand {
                    scope: "tokmd_core_ffi".to_string(),
                    kind: "mutation".to_string(),
                    required: false,
                    command: "cargo mutants --file crates/tokmd-core/src/ffi.rs --timeout 300"
                        .to_string(),
                },
            ],
            unknown_files: Vec::new(),
        };

        let summary = render_markdown_summary(&report);

        assert!(summary.contains("Required commands are the current proof selection"));
        assert!(summary.contains("| `proof` | `true` | 1 |"));
        assert!(summary.contains("| `coverage` | `false` | 1 |"));
        assert!(summary.contains("| `mutation` | `false` | 1 |"));
        assert!(summary.contains("cargo mutants --file crates/tokmd-core/src/ffi.rs"));
    }

    #[test]
    fn affected_plan_uses_package_mutation_fallback_without_source_files() {
        let policy = parse_policy_str(
            r#"
schema = "tokmd.proof_policy.v1"

[defaults]
mutation_timeout_seconds = 77
"#,
        )
        .expect("policy should parse");
        let affected = AffectedReport {
            schema: "tokmd.affected.v1".to_string(),
            ok: true,
            base: "base".to_string(),
            head: "head".to_string(),
            changed_files: vec!["crates/tokmd-core/Cargo.toml".to_string()],
            scopes: vec![AffectedScope {
                name: "tokmd_core_manifest".to_string(),
                kind: crate::proof::policy_ast::ScopeKind::Rust,
                reason: "matched crates/tokmd-core/Cargo.toml".to_string(),
                matched_files: vec!["crates/tokmd-core/Cargo.toml".to_string()],
                packages: vec!["tokmd-core".to_string()],
                proof: vec!["cargo test -p tokmd-core".to_string()],
                mutation: true,
                coverage: false,
            }],
            unknown_files: Vec::new(),
        };

        let commands = affected_commands(&policy, &affected);

        assert!(commands.iter().any(|cmd| {
            cmd.kind == "mutation"
                && !cmd.required
                && cmd.command == "cargo mutants -p tokmd-core --timeout 77"
        }));
    }

    #[test]
    fn mutation_candidates_exclude_test_and_fixture_surfaces() {
        assert!(is_mutation_candidate("crates/tokmd-core/src/ffi.rs"));
        assert!(!is_mutation_candidate("crates/tokmd-core/tests/ffi.rs"));
        assert!(!is_mutation_candidate(
            "fuzz/fuzz_targets/fuzz_badge_svg.rs"
        ));
        assert!(!is_mutation_candidate("crates/tokmd/examples/demo.rs"));
    }
}
