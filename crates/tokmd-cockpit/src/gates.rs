use std::path::{Path, PathBuf};

use anyhow::Result;
use tokmd_types::cockpit::*;

mod complexity;
mod contracts;
mod determinism_gate;
mod diff_coverage;
mod mutation;
mod rust_source;
pub mod source_complexity;

use complexity::compute_complexity_gate;
use contracts::compute_contract_gate;
use diff_coverage::compute_diff_coverage_gate;
use mutation::compute_mutation_gate;

use crate::FileStat;
use crate::supply_chain::compute_supply_chain_gate;

pub use determinism_gate::compute_determinism_gate;

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
