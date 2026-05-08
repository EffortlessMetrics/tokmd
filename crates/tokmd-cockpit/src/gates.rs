use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use tokmd_types::cockpit::*;

mod complexity;
mod diff_coverage;
mod mutation;

use complexity::compute_complexity_gate;
use diff_coverage::compute_diff_coverage_gate;
use mutation::compute_mutation_gate;

use crate::FileStat;
use crate::determinism;
use crate::supply_chain::compute_supply_chain_gate;

// =============================================================================
// Evidence computation
// =============================================================================

/// Compute evidence section with all gates.
#[cfg(feature = "git")]
pub(crate) fn compute_evidence(
    repo_root: &PathBuf,
    base: &str,
    head: &str,
    changed_files: &[FileStat],
    contracts_info: &Contracts,
    range_mode: tokmd_git::GitRangeMode,
    baseline_path: Option<&Path>,
) -> Result<Evidence> {
    let mutation = compute_mutation_gate(repo_root, base, head, changed_files, range_mode)?;
    let diff_coverage = compute_diff_coverage_gate(repo_root, base, head, range_mode)?;
    let contracts = compute_contract_gate(repo_root, base, head, changed_files, contracts_info)?;
    let supply_chain = compute_supply_chain_gate(repo_root, changed_files)?;
    let determinism = compute_determinism_gate(repo_root, baseline_path)?;
    let complexity = compute_complexity_gate(repo_root, changed_files)?;

    // Compute overall status: any Fail -> Fail, all Pass -> Pass, otherwise Pending/Skipped
    let overall_status = compute_overall_status(
        &mutation,
        &diff_coverage,
        &contracts,
        &supply_chain,
        &determinism,
        &complexity,
    );

    Ok(Evidence {
        overall_status,
        mutation,
        diff_coverage,
        contracts,
        supply_chain,
        determinism,
        complexity,
    })
}

/// Compute overall status from all gates.
#[cfg(feature = "git")]
fn compute_overall_status(
    mutation: &MutationGate,
    diff_coverage: &Option<DiffCoverageGate>,
    contracts: &Option<ContractDiffGate>,
    supply_chain: &Option<SupplyChainGate>,
    determinism: &Option<DeterminismGate>,
    complexity: &Option<ComplexityGate>,
) -> GateStatus {
    let statuses: Vec<GateStatus> = [
        Some(mutation.meta.status),
        diff_coverage.as_ref().map(|g| g.meta.status),
        contracts.as_ref().map(|g| g.meta.status),
        supply_chain.as_ref().map(|g| g.meta.status),
        determinism.as_ref().map(|g| g.meta.status),
        complexity.as_ref().map(|g| g.meta.status),
    ]
    .into_iter()
    .flatten()
    .collect();

    if statuses.is_empty() {
        return GateStatus::Skipped;
    }

    // Any Fail -> overall Fail
    if statuses.contains(&GateStatus::Fail) {
        return GateStatus::Fail;
    }

    // All Pass -> overall Pass
    if statuses.iter().all(|s| *s == GateStatus::Pass) {
        return GateStatus::Pass;
    }

    // Any Pending (and no Fail) -> overall Pending
    if statuses.contains(&GateStatus::Pending) {
        return GateStatus::Pending;
    }

    // Any Warn (and no Fail/Pending) -> overall Warn
    if statuses.contains(&GateStatus::Warn) {
        return GateStatus::Warn;
    }

    // All Skipped -> Skipped; mix of Pass and Skipped -> Pass
    if statuses.iter().all(|s| *s == GateStatus::Skipped) {
        GateStatus::Skipped
    } else {
        GateStatus::Pass
    }
}

// =============================================================================
// Contract gate
// =============================================================================

/// Compute contract diff gate (semver, CLI, schema).
#[cfg(feature = "git")]
fn compute_contract_gate(
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

// =============================================================================
// Determinism gate
// =============================================================================

/// Compute determinism gate.
/// Compares expected source hash (from baseline) with a fresh hash of the repo.
#[cfg(feature = "git")]
pub fn compute_determinism_gate(
    repo_root: &Path,
    baseline_path: Option<&Path>,
) -> Result<Option<DeterminismGate>> {
    use tokmd_analysis_types::ComplexityBaseline;

    fn short16(s: &str) -> &str {
        s.get(..16).unwrap_or(s)
    }

    // Resolve baseline: explicit path or default location
    let resolved_path = match baseline_path {
        Some(p) => p.to_path_buf(),
        None => repo_root.join(".tokmd/baseline.json"),
    };

    // If no baseline file exists, skip the gate
    if !resolved_path.exists() {
        return Ok(None);
    }

    // Parse baseline
    let content = std::fs::read_to_string(&resolved_path)
        .with_context(|| format!("failed to read baseline at {}", resolved_path.display()))?;
    let json: serde_json::Value = serde_json::from_str(&content).with_context(|| {
        format!(
            "failed to parse baseline JSON at {}",
            resolved_path.display()
        )
    })?;
    let baseline: ComplexityBaseline = match serde_json::from_value(json.clone()) {
        Ok(parsed) => parsed,
        Err(_) => {
            // Allow cockpit receipts for trend comparison; determinism data is unavailable there.
            let mode = json
                .get("mode")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if mode == "cockpit" {
                return Ok(None);
            }
            bail!(
                "baseline JSON at {} is not a ComplexityBaseline (and not a cockpit receipt)",
                resolved_path.display()
            );
        }
    };

    // If baseline has no determinism section, skip the gate
    let det = match &baseline.determinism {
        Some(d) => d,
        None => return Ok(None),
    };

    // Recompute current source hash by walking the repo, excluding the baseline file itself
    let baseline_rel = resolved_path
        .strip_prefix(repo_root)
        .ok()
        .map(|p| p.to_string_lossy().replace('\\', "/"));
    let exclude: Vec<&str> = baseline_rel.as_deref().into_iter().collect();
    let actual_hash = determinism::hash_files_from_walk(repo_root, &exclude)?;
    let expected_hash = &det.source_hash;

    let mut differences = Vec::new();

    if actual_hash != *expected_hash {
        differences.push(format!(
            "source hash mismatch: expected {}, got {}",
            short16(expected_hash),
            short16(&actual_hash),
        ));
    }

    // Check Cargo.lock hash if baseline had one
    if let Some(expected_lock) = &det.cargo_lock_hash {
        let actual_lock = determinism::hash_cargo_lock(repo_root)?;
        match actual_lock {
            Some(ref actual) if actual != expected_lock => {
                differences.push(format!(
                    "Cargo.lock hash mismatch: expected {}, got {}",
                    short16(expected_lock),
                    short16(actual),
                ));
            }
            None => {
                differences.push("Cargo.lock missing (was present in baseline)".to_string());
            }
            _ => {}
        }
    }

    let status = if differences.is_empty() {
        GateStatus::Pass
    } else {
        GateStatus::Warn
    };

    Ok(Some(DeterminismGate {
        meta: GateMeta {
            status,
            source: EvidenceSource::RanLocal,
            commit_match: CommitMatch::Unknown,
            scope: ScopeCoverage {
                relevant: vec!["source files".to_string()],
                tested: vec!["source files".to_string()],
                ratio: 1.0,
                lines_relevant: None,
                lines_tested: None,
            },
            evidence_commit: None,
            evidence_generated_at_ms: None,
        },
        expected_hash: Some(expected_hash.clone()),
        actual_hash: Some(actual_hash),
        algo: "blake3".to_string(),
        differences,
    }))
}

/// Check if a file is a relevant Rust source file for mutation testing.
/// Excludes test files, fuzz targets, etc.
#[cfg(feature = "git")]
pub(super) fn is_relevant_rust_source(path: &str) -> bool {
    let path_lower = path.to_lowercase();

    // Must be a .rs file
    if !path_lower.ends_with(".rs") {
        return false;
    }

    // Exclude test directories
    if path_lower.contains("/tests/") || path_lower.starts_with("tests/") {
        return false;
    }

    // Exclude test files
    if path_lower.ends_with("_test.rs") || path_lower.ends_with("_tests.rs") {
        return false;
    }

    // Exclude fuzz targets
    if path_lower.contains("/fuzz/") || path_lower.starts_with("fuzz/") {
        return false;
    }

    true
}
