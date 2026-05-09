//! # tokmd-cockpit
//!
//! **Tier 2 (Computation & Rendering)**
//!
//! Cockpit PR metrics computation and rendering for tokmd.
//! Provides functions to compute change surface, code health, risk,
//! composition, evidence gates, and review plans for pull requests.
//!
//! ## What belongs here
//! * Cockpit metric computation functions
//! * Evidence gate computation (mutation, diff coverage, complexity, etc.)
//! * Markdown/JSON/sections rendering
//! * Determinism hashing helpers
//!
//! ## What does NOT belong here
//! * CLI argument parsing (use `tokmd::cli`)
//! * Type definitions (use tokmd-types::cockpit)

#[cfg(feature = "git")]
mod change_surface;
mod composition;
mod contracts;
pub mod determinism;
mod display;
mod file_stat;
#[cfg(feature = "git")]
mod gates;
mod health;
mod proof_evidence;
pub mod render;
mod review_plan;
mod risk;
#[cfg(feature = "git")]
mod supply_chain;
mod trend;

#[cfg(feature = "git")]
use std::path::{Path, PathBuf};

use anyhow::Result;
#[cfg(feature = "git")]
use change_surface::compute_change_surface;
#[cfg(feature = "git")]
pub use change_surface::get_file_stats;
pub use composition::compute_composition;
pub use contracts::detect_contracts;
pub use display::{format_signed_f64, now_iso8601, round_pct, sparkline, trend_direction_label};
pub use file_stat::FileStat;
#[cfg(feature = "git")]
pub use gates::compute_determinism_gate;
#[cfg(feature = "git")]
use gates::compute_evidence;
pub use health::compute_code_health;
pub use proof_evidence::{ProofEvidenceInput, ProofEvidenceKind};
pub use review_plan::generate_review_plan;
pub use risk::compute_risk;
#[cfg(feature = "git")]
use risk::compute_risk_owned;
#[cfg(all(test, feature = "git"))]
use tokmd_analysis::source_complexity::analyze_rust_function_complexity;
pub use trend::{compute_complexity_trend, compute_metric_trend, load_and_compute_trend};
// Re-export types from tokmd_types::cockpit for convenience
pub use tokmd_types::cockpit::*;

/// Cyclomatic complexity threshold for high complexity.
pub const COMPLEXITY_THRESHOLD: u32 = 15;

/// Parse a proof-control-plane evidence artifact and return its artifact family.
///
/// This is intentionally validation-only for now: cockpit can accept explicit
/// proof evidence inputs without changing review packet output semantics.
pub fn proof_evidence_kind(raw: &str) -> Result<ProofEvidenceKind> {
    proof_evidence::proof_evidence_kind(raw)
}

/// Parse a proof-control-plane evidence artifact with its source path.
pub fn parse_proof_evidence_input(
    raw: &str,
    source_path: impl Into<std::path::PathBuf>,
) -> Result<ProofEvidenceInput> {
    proof_evidence::parse_proof_evidence_input(raw, source_path)
}

// =============================================================================
// Core cockpit computation
// =============================================================================

/// Compute the full cockpit receipt for a PR.
#[cfg(feature = "git")]
pub fn compute_cockpit(
    repo_root: &PathBuf,
    base: &str,
    head: &str,
    range_mode: tokmd_git::GitRangeMode,
    baseline_path: Option<&Path>,
) -> Result<CockpitReceipt> {
    let generated_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    // Get changed files with their stats
    let file_stats = get_file_stats(repo_root, base, head, range_mode)?;

    // Get change surface from git
    let change_surface = compute_change_surface(repo_root, base, head, &file_stats, range_mode)?;

    // Compute composition with test ratio
    let composition = compute_composition(&file_stats);

    // Detect contract changes
    let contracts = detect_contracts(&file_stats);

    // Compute code health
    let code_health = compute_code_health(&file_stats, &contracts);

    // Compute all gate evidence
    let evidence = compute_evidence(
        repo_root,
        base,
        head,
        &file_stats,
        &contracts,
        range_mode,
        baseline_path,
    )?;

    // Generate review plan with complexity scores
    let review_plan = generate_review_plan(&file_stats, &contracts);

    // Compute risk based on various factors
    let risk = compute_risk_owned(file_stats, &contracts, &code_health);

    Ok(CockpitReceipt {
        schema_version: COCKPIT_SCHEMA_VERSION,
        mode: "cockpit".to_string(),
        generated_at_ms,
        base_ref: base.to_string(),
        head_ref: head.to_string(),
        change_surface,
        composition,
        code_health,
        risk,
        contracts,
        evidence,
        review_plan,
        trend: None, // Populated by caller if --baseline is provided
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "git")]
    fn test_rust_complexity_ignores_decisions_in_strings_and_comments() {
        let analysis = analyze_rust_function_complexity(
            r###"
fn only_real_branch(flag: bool) {
    let _normal = "if while for loop match && || ? => { }";
    let _raw = r##"if while for loop match && || ? => { }"##;
    let _char = '?';
    /* if outer /* while nested */ match ignored => */
    if flag {
        println!("ok"); // else if ignored && ||
    }
}
"###,
        );

        assert_eq!(analysis.function_count, 1);
        assert_eq!(analysis.total_complexity, 2);
        assert_eq!(analysis.max_complexity, 2);
    }

    #[test]
    #[cfg(feature = "git")]
    fn test_rust_complexity_counts_code_before_trailing_comment() {
        let analysis = analyze_rust_function_complexity(
            r#"
fn branch_before_comment(flag: bool) {
    if flag { return; } // if ignored && ||
}
"#,
        );

        assert_eq!(analysis.function_count, 1);
        assert_eq!(analysis.total_complexity, 2);
        assert_eq!(analysis.max_complexity, 2);
    }

    #[test]
    #[cfg(feature = "git")]
    fn test_rust_complexity_counts_else_if_once() {
        let analysis = analyze_rust_function_complexity(
            r#"
fn branchy(x: i32) -> i32 {
    if x > 0 {
        1
    } else if x < 0 {
        -1
    } else if x == 0 {
        0
    } else {
        42
    }
}
"#,
        );

        assert_eq!(analysis.function_count, 1);
        assert_eq!(analysis.total_complexity, 4);
        assert_eq!(analysis.max_complexity, 4);
    }

    // ---- FileStat AsRef ----

    #[test]
    fn test_filestat_as_ref() {
        let stat = FileStat {
            path: "src/main.rs".to_string(),
            insertions: 10,
            deletions: 5,
        };
        let s: &str = stat.as_ref();
        assert_eq!(s, "src/main.rs");
    }
}
