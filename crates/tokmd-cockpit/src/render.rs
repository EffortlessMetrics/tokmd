//! Rendering functions for cockpit receipts.
//!
//! Provides JSON, Markdown, sections, comment, and review packet output formats.

use std::path::Path;

use anyhow::{Context, Result};
use tokmd_envelope::{SensorReport, ToolMeta, Verdict};

use crate::{CockpitReceipt, GateStatus, now_iso8601};

mod comment;
mod evidence;
mod manifest;
mod markdown;
mod review_map;
mod review_packet;

pub use comment::render_comment_md;
pub use markdown::render_markdown;
pub use review_packet::{write_review_packet, write_review_packet_with_proof_evidence};

/// Render receipt as JSON.
pub fn render_json(receipt: &CockpitReceipt) -> Result<String> {
    serde_json::to_string_pretty(receipt).context("Failed to serialize receipt to JSON")
}

/// Render receipt as sectioned output.
pub fn render_sections(receipt: &CockpitReceipt) -> String {
    use std::fmt::Write;
    let mut s = String::new();

    let _ = writeln!(s, "<!-- SECTION:COCKPIT -->");
    let _ = writeln!(s);
    let _ = writeln!(s, "## Glass Cockpit");
    let _ = writeln!(s);
    let _ = writeln!(s, "**Base**: {}", receipt.base_ref);
    let _ = writeln!(s, "**Head**: {}", receipt.head_ref);
    let _ = writeln!(s);
    let _ = writeln!(s, "**Change Surface**:");
    let _ = writeln!(s, "- Files: {}", receipt.change_surface.files_changed);
    let _ = writeln!(s, "- Insertions: {}", receipt.change_surface.insertions);
    let _ = writeln!(s, "- Deletions: {}", receipt.change_surface.deletions);
    let _ = writeln!(s);
    let _ = writeln!(s, "**Composition**:");
    let _ = writeln!(s, "- Code: {:.1}%", receipt.composition.code_pct * 100.0);
    let _ = writeln!(s, "- Test: {:.1}%", receipt.composition.test_pct * 100.0);
    let _ = writeln!(s, "- Docs: {:.1}%", receipt.composition.docs_pct * 100.0);
    let _ = writeln!(
        s,
        "- Config: {:.1}%",
        receipt.composition.config_pct * 100.0
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "**Contracts**:");
    let _ = writeln!(
        s,
        "- API: {}",
        if receipt.contracts.api_changed {
            "Yes"
        } else {
            "No"
        }
    );
    let _ = writeln!(
        s,
        "- CLI: {}",
        if receipt.contracts.cli_changed {
            "Yes"
        } else {
            "No"
        }
    );
    let _ = writeln!(
        s,
        "- Schema: {}",
        if receipt.contracts.schema_changed {
            "Yes"
        } else {
            "No"
        }
    );
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "**Health**: {}/100 ({})",
        receipt.code_health.score, receipt.code_health.grade
    );
    let _ = writeln!(
        s,
        "**Risk**: {} ({}/100)",
        receipt.risk.level, receipt.risk.score
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "<!-- SECTION:REVIEW_PLAN -->");
    let _ = writeln!(s);
    let _ = writeln!(s, "## Review Plan");
    let _ = writeln!(s);
    if receipt.review_plan.is_empty() {
        let _ = writeln!(s, "No review items.");
    } else {
        for item in &receipt.review_plan {
            let _ = writeln!(s, "- {} (priority: {})", item.path, item.priority);
        }
    }
    let _ = writeln!(s);
    let _ = writeln!(s, "<!-- SECTION:RECEIPTS -->");
    let _ = writeln!(s);
    let _ = writeln!(s, "## Receipts");
    let _ = writeln!(s);
    let _ = writeln!(s, "Full receipt data available in JSON format.");
    let _ = writeln!(s);

    s
}

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
#[cfg(feature = "git")]
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
