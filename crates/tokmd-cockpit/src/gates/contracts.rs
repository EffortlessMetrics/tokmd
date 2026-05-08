use std::path::Path;
use std::process::Command;

use anyhow::Result;
use tokmd_types::cockpit::*;

use crate::FileStat;

/// Compute contract diff gate (semver, CLI, schema).
#[cfg(feature = "git")]
pub(super) fn compute_contract_gate(
    repo_root: &Path,
    base: &str,
    head: &str,
    changed_files: &[FileStat],
    contracts_info: &Contracts,
) -> Result<Option<ContractDiffGate>> {
    // Only compute if any contract-relevant files changed
    if !contracts_info.api_changed && !contracts_info.cli_changed && !contracts_info.schema_changed
    {
        return Ok(None);
    }

    let mut failures = 0;
    let mut semver = None;
    let mut cli = None;
    let mut schema = None;

    // Check for semver changes (API files)
    if contracts_info.api_changed {
        semver = Some(run_semver_check(repo_root));
    }

    // Check for CLI changes
    if contracts_info.cli_changed {
        // Gather CLI-related files that changed
        let cli_files: Vec<&str> = changed_files
            .iter()
            .filter(|f| {
                f.path.contains("crates/tokmd/src/commands/")
                    || f.path.contains("crates/tokmd/src/cli/")
                    || f.path == "crates/tokmd/src/config.rs"
            })
            .map(|s| s.path.as_str())
            .collect();

        let diff_summary = if cli_files.is_empty() {
            None
        } else {
            let command_files = cli_files
                .iter()
                .filter(|f| f.contains("crates/tokmd/src/commands/"))
                .count();
            let config_files = cli_files
                .iter()
                .filter(|f| {
                    f.contains("crates/tokmd/src/cli/") || **f == "crates/tokmd/src/config.rs"
                })
                .count();

            let mut parts = Vec::new();
            if command_files > 0 {
                parts.push(format!(
                    "{} command file{}",
                    command_files,
                    if command_files == 1 { "" } else { "s" }
                ));
            }
            if config_files > 0 {
                parts.push(format!(
                    "{} config file{}",
                    config_files,
                    if config_files == 1 { "" } else { "s" }
                ));
            }
            Some(parts.join(", "))
        };

        cli = Some(CliSubGate {
            status: GateStatus::Pass,
            diff_summary,
        });
    }

    // Check for schema changes
    if contracts_info.schema_changed {
        schema = Some(run_schema_diff(repo_root, base, head));
    }

    // Count failures from sub-gates
    if let Some(ref sg) = semver
        && sg.status == GateStatus::Fail
    {
        failures += 1;
    }
    if let Some(ref cg) = cli
        && cg.status == GateStatus::Fail
    {
        failures += 1;
    }
    if let Some(ref scg) = schema
        && scg.status == GateStatus::Fail
    {
        failures += 1;
    }

    // Determine overall status
    let status = if failures > 0 {
        GateStatus::Fail
    } else {
        // Check if any are pending
        let any_pending = [
            semver.as_ref().map(|g| g.status),
            cli.as_ref().map(|g| g.status),
            schema.as_ref().map(|g| g.status),
        ]
        .into_iter()
        .flatten()
        .any(|s| s == GateStatus::Pending);

        if any_pending {
            GateStatus::Pending
        } else {
            GateStatus::Pass
        }
    };

    // Collect relevant files for scope
    let relevant: Vec<String> = changed_files
        .iter()
        .filter(|f| {
            f.path.ends_with("/src/lib.rs")
                || f.path.ends_with("/mod.rs")
                || f.path.contains("crates/tokmd/src/commands/")
                || f.path.contains("crates/tokmd/src/cli/")
                || f.path == "crates/tokmd/src/config.rs"
                || f.path == "docs/schema.json"
        })
        .map(|f| f.path.clone())
        .collect();

    Ok(Some(ContractDiffGate {
        meta: GateMeta {
            status,
            source: EvidenceSource::RanLocal,
            commit_match: CommitMatch::Unknown,
            scope: ScopeCoverage {
                relevant: relevant.clone(),
                tested: relevant,
                ratio: 1.0,
                lines_relevant: None,
                lines_tested: None,
            },
            evidence_commit: None,
            evidence_generated_at_ms: None,
        },
        semver,
        cli,
        schema,
        failures,
    }))
}

/// Run cargo-semver-checks if available.
/// Returns a SemverSubGate with the result.
#[cfg(feature = "git")]
fn run_semver_check(repo_root: &Path) -> SemverSubGate {
    // Check if cargo-semver-checks is available
    let available = Command::new("cargo")
        .args(["semver-checks", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !available {
        return SemverSubGate {
            status: GateStatus::Pending,
            breaking_changes: Vec::new(),
        };
    }

    // Run cargo semver-checks
    let output = match Command::new("cargo")
        .args(["semver-checks", "check-release"])
        .current_dir(repo_root)
        .output()
    {
        Ok(o) => o,
        Err(_) => {
            return SemverSubGate {
                status: GateStatus::Pending,
                breaking_changes: Vec::new(),
            };
        }
    };

    if output.status.success() {
        // Exit 0 = no breaking changes
        return SemverSubGate {
            status: GateStatus::Pass,
            breaking_changes: Vec::new(),
        };
    }

    // Non-zero exit = breaking changes found
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{}{}", stdout, stderr);

    // Parse breaking changes from output lines
    // cargo-semver-checks output format: "--- failure[kind]: message ---" or similar
    let mut breaking_changes: Vec<BreakingChange> = Vec::new();
    for line in combined.lines() {
        let trimmed = line.trim();
        if trimmed.contains("BREAKING") || trimmed.starts_with("---") {
            breaking_changes.push(BreakingChange {
                kind: "semver".to_string(),
                path: String::new(),
                message: trimmed.to_string(),
            });
        }
    }

    // If we couldn't parse specific changes but the tool failed, add a generic entry
    if breaking_changes.is_empty() {
        breaking_changes.push(BreakingChange {
            kind: "semver".to_string(),
            path: String::new(),
            message: "cargo-semver-checks reported breaking changes".to_string(),
        });
    }

    // Limit output
    breaking_changes.truncate(20);

    SemverSubGate {
        status: GateStatus::Fail,
        breaking_changes,
    }
}

/// Run git diff on docs/schema.json to detect schema changes.
/// Returns a SchemaSubGate with the result.
#[cfg(feature = "git")]
fn run_schema_diff(repo_root: &Path, base: &str, head: &str) -> SchemaSubGate {
    // Use two-dot syntax for comparing refs directly (per project convention)
    let range = format!("{}..{}", base, head);
    let output = match tokmd_git::git_cmd()
        .arg("-C")
        .arg(repo_root)
        .args(["diff", &range, "--", "docs/schema.json"])
        .output()
    {
        Ok(o) => o,
        Err(_) => {
            return SchemaSubGate {
                status: GateStatus::Pending,
                diff_summary: None,
            };
        }
    };

    if !output.status.success() {
        return SchemaSubGate {
            status: GateStatus::Pending,
            diff_summary: None,
        };
    }

    let diff = String::from_utf8_lossy(&output.stdout);
    if diff.trim().is_empty() {
        // No diff means schema.json didn't change between these refs
        return SchemaSubGate {
            status: GateStatus::Pass,
            diff_summary: None,
        };
    }

    // Analyze the diff for breaking vs additive changes
    let mut additions = 0usize;
    let mut removals = 0usize;
    let mut has_type_change = false;

    for line in diff.lines() {
        if line.starts_with('+') && !line.starts_with("+++") {
            additions += 1;
        } else if line.starts_with('-') && !line.starts_with("---") {
            removals += 1;
            // Check for type changes (field type modifications)
            let trimmed = line.trim_start_matches('-').trim();
            if trimmed.contains("\"type\"") {
                has_type_change = true;
            }
        }
    }

    let (status, summary) = if removals == 0 {
        // Only additions = safe additive change
        (
            GateStatus::Pass,
            Some(format!(
                "schema.json: {} line{} added (additive only)",
                additions,
                if additions == 1 { "" } else { "s" }
            )),
        )
    } else if has_type_change || removals > additions {
        // Type changes or net removals = likely breaking
        (
            GateStatus::Fail,
            Some(format!(
                "schema.json: {} addition{}, {} removal{} (potential breaking change)",
                additions,
                if additions == 1 { "" } else { "s" },
                removals,
                if removals == 1 { "" } else { "s" }
            )),
        )
    } else {
        // Removals but mostly additions = warn
        (
            GateStatus::Pass,
            Some(format!(
                "schema.json: {} addition{}, {} removal{}",
                additions,
                if additions == 1 { "" } else { "s" },
                removals,
                if removals == 1 { "" } else { "s" }
            )),
        )
    };

    SchemaSubGate {
        status,
        diff_summary: summary,
    }
}
