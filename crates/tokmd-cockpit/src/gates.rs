use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use tokmd_types::cockpit::*;

mod complexity;
mod contracts;
mod diff_coverage;
mod mutation;

use complexity::compute_complexity_gate;
use contracts::compute_contract_gate;
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
