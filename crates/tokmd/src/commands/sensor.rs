//! Handler for the `tokmd sensor` command.
//!
//! Runs tokmd as a conforming sensor, producing a `SensorReport` envelope
//! backed by cockpit computation. Implements a 3-layer output topology:
//!
//! 1. **report.json** — Thin envelope with findings, gates, summary metrics
//! 2. **extras/cockpit_receipt.json** — Full cockpit receipt sidecar
//! 3. **comment.md** — Markdown summary for PR comments

use crate::cli;
#[cfg(feature = "git")]
use crate::progress::Progress;
#[cfg(feature = "git")]
use anyhow::Context;
use anyhow::{Result, bail};
#[cfg(feature = "git")]
use tokmd_envelope::{SensorReport, ToolMeta};

#[cfg(feature = "git")]
mod findings;
#[cfg(feature = "git")]
mod gates;
#[cfg(feature = "git")]
mod output;
#[cfg(feature = "git")]
use findings::{
    emit_complexity_findings, emit_contract_findings, emit_gate_findings, emit_risk_findings,
};
#[cfg(feature = "git")]
use gates::map_verdict;

pub(crate) fn handle(args: cli::SensorArgs, global: &cli::GlobalArgs) -> Result<()> {
    #[cfg(not(feature = "git"))]
    {
        let _ = (&args, global);
        bail!("The sensor command requires the 'git' feature. Rebuild with --features git");
    }

    #[cfg(feature = "git")]
    {
        if !tokmd_git::git_available() {
            bail!("git is not available on PATH");
        }

        // Sensor is a multi-step orchestrator (git diff + scan + render) like
        // cockpit. The spinner stays on stderr and machine-readable progress
        // events are emitted alongside it when TOKMD_PROGRESS_EVENTS is set;
        // stdout stays reserved for the sensor report.
        let progress = Progress::new(!global.no_progress);

        let cwd = std::env::current_dir().context("Failed to resolve current directory")?;
        let repo_root = tokmd_git::repo_root(&cwd)
            .ok_or_else(|| anyhow::anyhow!("not inside a git repository"))?;

        // Use two-dot range for sensor (same convention as cockpit)
        let range_mode = tokmd_git::GitRangeMode::TwoDot;

        progress.set_message("Resolving base reference...");
        let resolved_base =
            tokmd_git::resolve_base_ref(&repo_root, &args.base).ok_or_else(|| {
                anyhow::anyhow!(
                    "base ref '{}' not found and no fallback resolved. \
                 Use --base to specify a valid ref, or set TOKMD_GIT_BASE_REF",
                    args.base
                )
            })?;

        // Run cockpit computation (sensor mode has no baseline path)
        progress.set_message("Computing cockpit metrics (git diff + scan)...");
        let cockpit_receipt = super::cockpit::compute_cockpit(
            &repo_root,
            &resolved_base,
            &args.head,
            range_mode,
            None,
        )?;

        // Build the sensor report envelope
        progress.set_message("Building sensor report...");
        let generated_at = now_iso8601();
        let verdict = map_verdict(cockpit_receipt.evidence.overall_status);

        let mut report = SensorReport::new(
            ToolMeta::tokmd(env!("CARGO_PKG_VERSION"), "sensor"),
            generated_at,
            verdict,
            build_summary(&cockpit_receipt, &resolved_base, &args.head),
        );

        // Emit findings from cockpit data (all with fingerprints)
        emit_risk_findings(&mut report, &cockpit_receipt.risk);
        emit_contract_findings(&mut report, &cockpit_receipt.contracts);
        emit_complexity_findings(&mut report, &cockpit_receipt.evidence);
        emit_gate_findings(&mut report, &cockpit_receipt.evidence);

        // Clear the spinner before any stdout/file output so progress noise on
        // stderr never interleaves with the machine-readable report.
        progress.set_message("Writing sensor artifacts...");
        progress.finish_and_clear();

        output::write_outputs(&args, report, &cockpit_receipt)
    }
}

#[cfg(feature = "git")]
fn build_summary(receipt: &super::cockpit::CockpitReceipt, base: &str, head: &str) -> String {
    format!(
        "{} files changed, +{}/-{}, health {}/100, risk {} in {}..{}",
        receipt.change_surface.files_changed,
        receipt.change_surface.insertions,
        receipt.change_surface.deletions,
        receipt.code_health.score,
        receipt.risk.level,
        base,
        head,
    )
}

#[cfg(feature = "git")]
fn now_iso8601() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

#[cfg(test)]
#[cfg(feature = "git")]
mod tests {
    use super::*;

    use super::super::cockpit::{Risk, RiskLevel};

    #[cfg(feature = "git")]
    #[test]
    fn build_summary_formats_expected_fields() {
        let receipt = super::super::cockpit::CockpitReceipt {
            schema_version: 3,
            mode: "cockpit".to_string(),
            generated_at_ms: 0,
            base_ref: "main".to_string(),
            head_ref: "HEAD".to_string(),
            change_surface: super::super::cockpit::ChangeSurface {
                commits: 1,
                files_changed: 2,
                insertions: 10,
                deletions: 5,
                net_lines: 5,
                churn_velocity: 15.0,
                change_concentration: 0.4,
            },
            composition: super::super::cockpit::Composition {
                code_pct: 0.8,
                test_pct: 0.1,
                docs_pct: 0.05,
                config_pct: 0.05,
                test_ratio: 0.2,
            },
            code_health: super::super::cockpit::CodeHealth {
                score: 75,
                grade: "B".to_string(),
                large_files_touched: 0,
                avg_file_size: 10,
                complexity_indicator: super::super::cockpit::ComplexityIndicator::Low,
                warnings: vec![],
            },
            risk: Risk {
                hotspots_touched: vec![],
                bus_factor_warnings: vec![],
                level: RiskLevel::High,
                score: 80,
            },
            contracts: super::super::cockpit::Contracts {
                api_changed: false,
                cli_changed: false,
                schema_changed: false,
                breaking_indicators: 0,
            },
            evidence: gates::test_support::base_evidence(),
            review_plan: vec![],
            trend: None,
        };

        let summary = build_summary(&receipt, "main", "HEAD");
        assert!(summary.contains("2 files changed"));
        assert!(summary.contains("+10/-5"));
        assert!(summary.contains("health 75/100"));
        assert!(summary.contains("risk high"));
        assert!(summary.contains("main..HEAD"));
    }
}
