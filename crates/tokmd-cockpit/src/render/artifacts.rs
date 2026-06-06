//! Artifact writers for cockpit receipt outputs.

use std::path::Path;

use anyhow::Result;
use tokmd_envelope::{SensorReport, ToolMeta, Verdict};

use crate::{CockpitReceipt, GateStatus, now_iso8601};

use super::{render_comment_md, render_json};

/// Write artifacts to directory.
pub fn write_artifacts(dir: &Path, receipt: &CockpitReceipt) -> Result<()> {
    std::fs::create_dir_all(dir)?;

    // Write cockpit.json (full receipt)
    let json = render_json(receipt)?;
    std::fs::write(dir.join("cockpit.json"), json)?;

    // Write report.json (sensor report envelope)
    let verdict = match receipt.evidence.overall_status {
        GateStatus::Pass => Verdict::Pass,
        GateStatus::Fail => Verdict::Fail,
        GateStatus::Warn => Verdict::Warn,
        GateStatus::Skipped => Verdict::Skip,
        GateStatus::Pending => Verdict::Pending,
    };

    let report = SensorReport::new(
        ToolMeta::tokmd(env!("CARGO_PKG_VERSION"), "cockpit"),
        now_iso8601(),
        verdict,
        format!(
            "{} files changed, +{}/-{}, health {}/100, risk {} in {}..{}",
            receipt.change_surface.files_changed,
            receipt.change_surface.insertions,
            receipt.change_surface.deletions,
            receipt.code_health.score,
            receipt.risk.level,
            receipt.base_ref,
            receipt.head_ref
        ),
    );

    let report_json = serde_json::to_string_pretty(&report)?;
    std::fs::write(dir.join("report.json"), report_json)?;

    // Write comment.md (markdown summary)
    let comment_md = render_comment_md(receipt);
    std::fs::write(dir.join("comment.md"), comment_md)?;

    Ok(())
}

/// Write sensor artifacts.
pub fn write_sensor_artifacts(
    dir: &Path,
    receipt: &CockpitReceipt,
    base: &str,
    head: &str,
) -> Result<()> {
    std::fs::create_dir_all(dir)?;

    // Build sensor report
    let verdict = match receipt.evidence.overall_status {
        GateStatus::Pass => Verdict::Pass,
        GateStatus::Fail => Verdict::Fail,
        GateStatus::Warn => Verdict::Warn,
        GateStatus::Skipped => Verdict::Skip,
        GateStatus::Pending => Verdict::Pending,
    };

    let report = SensorReport::new(
        ToolMeta::tokmd(env!("CARGO_PKG_VERSION"), "cockpit"),
        now_iso8601(),
        verdict,
        format!("Cockpit run for {}..{}", base, head),
    );

    let json = serde_json::to_string_pretty(&report)?;
    std::fs::write(dir.join("report.json"), json)?;

    Ok(())
}
