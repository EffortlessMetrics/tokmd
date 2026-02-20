//! Rendering functions for cockpit receipts.
//!
//! Provides JSON, Markdown, sections, and comment output formats.

use std::path::Path;

use anyhow::{Context, Result};
use tokmd_envelope::{SensorReport, ToolMeta, Verdict};

use crate::{
    CockpitReceipt, GateStatus, format_signed_f64, now_iso8601, sparkline, trend_direction_label,
};

/// Render receipt as JSON.
pub fn render_json(receipt: &CockpitReceipt) -> Result<String> {
    serde_json::to_string_pretty(receipt).context("Failed to serialize receipt to JSON")
}

/// Render receipt as Markdown summary.
pub fn render_markdown(receipt: &CockpitReceipt) -> String {
    use std::fmt::Write;
    let mut s = String::new();

    let _ = writeln!(s, "## Glass Cockpit");
    let _ = writeln!(s);

    // Summary table
    s.push_str("### Summary\n\n");
    s.push_str("|Metric|Current|\n");
    s.push_str("|---|---:|\n");
    let _ = writeln!(
        s,
        "|Files Changed|{}|",
        receipt.change_surface.files_changed
    );
    let _ = writeln!(s, "|Insertions|{}|", receipt.change_surface.insertions);
    let _ = writeln!(s, "|Deletions|{}|", receipt.change_surface.deletions);
    let _ = writeln!(s, "|Net Lines|{}|", receipt.change_surface.net_lines);
    let _ = writeln!(s, "|Code Health Score|{}/100|", receipt.code_health.score);
    let _ = writeln!(s, "|Risk Score|{}/100|", receipt.risk.score);
    let _ = writeln!(s, "|Test Ratio|{:.2}|", receipt.composition.test_ratio);
    s.push('\n');

    if let Some(trend) = receipt.trend.as_ref().filter(|t| t.baseline_available) {
        s.push_str("### Summary Comparison\n\n");
        s.push_str("|Metric|Baseline|Current|Delta|Change|\n");
        s.push_str("|---|---:|---:|---:|---|\n");

        if let Some(health) = &trend.health {
            let _ = writeln!(
                s,
                "|Health Score|{:.1}|{:.1}|{}|{}|",
                health.previous,
                health.current,
                format_signed_f64(health.delta),
                trend_direction_label(health.direction)
            );
        }
        if let Some(risk) = &trend.risk {
            let _ = writeln!(
                s,
                "|Risk Score|{:.1}|{:.1}|{}|{}|",
                risk.previous,
                risk.current,
                format_signed_f64(risk.delta),
                trend_direction_label(risk.direction)
            );
        }
        if let Some(complexity) = &trend.complexity {
            let cyclomatic_delta = complexity
                .avg_cyclomatic_delta
                .map(format_signed_f64)
                .unwrap_or_else(|| "n/a".to_string());
            let _ = writeln!(
                s,
                "|Avg Cyclomatic|n/a|n/a|{}|{}|",
                cyclomatic_delta,
                trend_direction_label(complexity.direction)
            );
        }

        if let Some(path) = trend.baseline_path.as_deref() {
            let _ = writeln!(s, "\nBaseline: `{}`", path);
        }
        s.push('\n');
    }

    // Change Surface section
    let _ = writeln!(s, "### Change Surface");
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "- **Files changed**: {}",
        receipt.change_surface.files_changed
    );
    let _ = writeln!(s, "- **Insertions**: {}", receipt.change_surface.insertions);
    let _ = writeln!(s, "- **Deletions**: {}", receipt.change_surface.deletions);
    let _ = writeln!(s, "- **Net lines**: {}", receipt.change_surface.net_lines);
    let _ = writeln!(
        s,
        "- **Churn velocity**: {:.1}",
        receipt.change_surface.churn_velocity
    );
    let _ = writeln!(s);

    // Composition section
    let _ = writeln!(s, "### Composition");
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "- **Code**: {:.1}%",
        receipt.composition.code_pct * 100.0
    );
    let _ = writeln!(
        s,
        "- **Test**: {:.1}%",
        receipt.composition.test_pct * 100.0
    );
    let _ = writeln!(
        s,
        "- **Docs**: {:.1}%",
        receipt.composition.docs_pct * 100.0
    );
    let _ = writeln!(
        s,
        "- **Config**: {:.1}%",
        receipt.composition.config_pct * 100.0
    );
    let _ = writeln!(s, "- **Test ratio**: {:.2}", receipt.composition.test_ratio);
    let _ = writeln!(s);

    // Contracts section
    let _ = writeln!(s, "### Contracts");
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "- **API changed**: {}",
        if receipt.contracts.api_changed {
            "Yes"
        } else {
            "No"
        }
    );
    let _ = writeln!(
        s,
        "- **CLI changed**: {}",
        if receipt.contracts.cli_changed {
            "Yes"
        } else {
            "No"
        }
    );
    let _ = writeln!(
        s,
        "- **Schema changed**: {}",
        if receipt.contracts.schema_changed {
            "Yes"
        } else {
            "No"
        }
    );
    let _ = writeln!(
        s,
        "- **Breaking indicators**: {}",
        receipt.contracts.breaking_indicators
    );
    let _ = writeln!(s);

    // Code Health section
    let _ = writeln!(s, "### Code Health");
    let _ = writeln!(s);
    let _ = writeln!(s, "- **Score**: {}/100", receipt.code_health.score);
    let _ = writeln!(s, "- **Grade**: {}", receipt.code_health.grade);
    let _ = writeln!(
        s,
        "- **Large files touched**: {}",
        receipt.code_health.large_files_touched
    );
    let _ = writeln!(
        s,
        "- **Average file size**: {}",
        receipt.code_health.avg_file_size
    );
    let _ = writeln!(
        s,
        "- **Complexity indicator**: {:?}",
        receipt.code_health.complexity_indicator
    );
    if !receipt.code_health.warnings.is_empty() {
        let _ = writeln!(s, "- **Warnings**:");
        for warning in &receipt.code_health.warnings {
            let _ = writeln!(s, "  - {}: {}", warning.path, warning.message);
        }
    }
    let _ = writeln!(s);

    // Risk section
    let _ = writeln!(s, "### Risk");
    let _ = writeln!(s);
    let _ = writeln!(s, "- **Level**: {}", receipt.risk.level);
    let _ = writeln!(s, "- **Score**: {}/100", receipt.risk.score);
    if !receipt.risk.hotspots_touched.is_empty() {
        let _ = writeln!(s, "- **Hotspots touched**:");
        for hotspot in &receipt.risk.hotspots_touched {
            let _ = writeln!(s, "  - {}", hotspot);
        }
    }
    if !receipt.risk.bus_factor_warnings.is_empty() {
        let _ = writeln!(s, "- **Bus factor warnings**:");
        for warning in &receipt.risk.bus_factor_warnings {
            let _ = writeln!(s, "  - {}", warning);
        }
    }
    let _ = writeln!(s);

    // Evidence Gates section
    let _ = writeln!(s, "### Evidence Gates");
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "- **Overall status**: {:?}",
        receipt.evidence.overall_status
    );
    let _ = writeln!(
        s,
        "- **Mutation**: {:?} (killed: {}, survivors: {})",
        receipt.evidence.mutation.meta.status,
        receipt.evidence.mutation.killed,
        receipt.evidence.mutation.survivors.len()
    );
    if let Some(ref dc) = receipt.evidence.diff_coverage {
        let _ = writeln!(
            s,
            "- **Diff coverage**: {:?} ({:.1}%)",
            dc.meta.status,
            dc.coverage_pct * 100.0
        );
    }
    if let Some(ref contracts) = receipt.evidence.contracts {
        let _ = writeln!(
            s,
            "- **Contracts**: {:?} (failures: {})",
            contracts.meta.status, contracts.failures
        );
    }
    if let Some(ref sc) = receipt.evidence.supply_chain {
        let _ = writeln!(
            s,
            "- **Supply chain**: {:?} (vulnerabilities: {})",
            sc.meta.status,
            sc.vulnerabilities.len()
        );
    }
    if let Some(ref det) = receipt.evidence.determinism {
        let _ = writeln!(
            s,
            "- **Determinism**: {:?} (differences: {})",
            det.meta.status,
            det.differences.len()
        );
    }
    if let Some(ref cx) = receipt.evidence.complexity {
        let _ = writeln!(
            s,
            "- **Complexity**: {:?} (avg cyclomatic: {:.1}, max: {})",
            cx.meta.status, cx.avg_cyclomatic, cx.max_cyclomatic
        );
    }
    let _ = writeln!(s);

    // Review Plan section
    let _ = writeln!(s, "### Review Plan");
    let _ = writeln!(s);
    if receipt.review_plan.is_empty() {
        let _ = writeln!(s, "No review items.");
    } else {
        for item in &receipt.review_plan {
            let _ = writeln!(s, "- **{}** (priority: {})", item.path, item.priority);
            let _ = writeln!(s, "  - Reason: {}", item.reason);
            if let Some(complexity) = item.complexity {
                let _ = writeln!(s, "  - Complexity: {}", complexity);
            }
            if let Some(lines) = item.lines_changed {
                let _ = writeln!(s, "  - Lines changed: {}", lines);
            }
        }
    }
    let _ = writeln!(s);

    // Trend section (if available)
    if let Some(ref trend) = receipt.trend {
        let _ = writeln!(s, "### Trend");
        let _ = writeln!(s);
        if trend.baseline_available {
            let _ = writeln!(
                s,
                "- **Baseline**: {}",
                trend.baseline_path.as_deref().unwrap_or("N/A")
            );
            if let Some(ref health) = trend.health {
                let _ = writeln!(
                    s,
                    "- **Health**: {:.1} -> {:.1} {} ({:.1}%, {:?})",
                    health.previous,
                    health.current,
                    sparkline(&[health.previous, health.current]),
                    health.delta_pct,
                    health.direction
                );
            }
            if let Some(ref risk) = trend.risk {
                let _ = writeln!(
                    s,
                    "- **Risk**: {:.1} -> {:.1} {} ({:.1}%, {:?})",
                    risk.previous,
                    risk.current,
                    sparkline(&[risk.previous, risk.current]),
                    risk.delta_pct,
                    risk.direction
                );
            }
            if let Some(ref complexity) = trend.complexity {
                let _ = writeln!(
                    s,
                    "- **Complexity**: {} ({:?})",
                    complexity.summary, complexity.direction
                );
            }
        } else {
            let _ = writeln!(s, "No baseline available for comparison.");
        }
        let _ = writeln!(s);
    }

    s
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

/// Render comment.md for PR comments.
pub fn render_comment_md(receipt: &CockpitReceipt) -> String {
    use std::fmt::Write;
    let mut s = String::new();

    // Summary bullet points
    let _ = writeln!(s, "## Glass Cockpit Summary");
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "- **{} files changed**, +{}/-{}",
        receipt.change_surface.files_changed,
        receipt.change_surface.insertions,
        receipt.change_surface.deletions
    );
    let _ = writeln!(
        s,
        "- **Health**: {}/100 ({})",
        receipt.code_health.score, receipt.code_health.grade
    );
    let _ = writeln!(
        s,
        "- **Risk**: {} ({}/100)",
        receipt.risk.level, receipt.risk.score
    );
    let _ = writeln!(s);

    // Contract changes
    if receipt.contracts.api_changed
        || receipt.contracts.cli_changed
        || receipt.contracts.schema_changed
    {
        let _ = writeln!(s, "**Contract changes**:");
        if receipt.contracts.api_changed {
            let _ = writeln!(s, "- API contract changed");
        }
        if receipt.contracts.cli_changed {
            let _ = writeln!(s, "- CLI contract changed");
        }
        if receipt.contracts.schema_changed {
            let _ = writeln!(s, "- Schema contract changed");
        }
        if receipt.contracts.breaking_indicators > 0 {
            let _ = writeln!(
                s,
                "- {} breaking indicator(s)",
                receipt.contracts.breaking_indicators
            );
        }
        let _ = writeln!(s);
    }

    // Evidence gates
    let _ = writeln!(
        s,
        "**Evidence gates**: {:?}",
        receipt.evidence.overall_status
    );
    if !receipt.evidence.mutation.survivors.is_empty() {
        let _ = writeln!(
            s,
            "- Mutation: {} survivors detected",
            receipt.evidence.mutation.survivors.len()
        );
    }
    if let Some(ref dc) = receipt.evidence.diff_coverage {
        let _ = writeln!(s, "- Diff coverage: {:.1}%", dc.coverage_pct * 100.0);
    }
    if let Some(ref contracts) = receipt.evidence.contracts
        && contracts.failures > 0
    {
        let _ = writeln!(s, "- Contracts: {} sub-gate(s) failed", contracts.failures);
    }
    if let Some(ref sc) = receipt.evidence.supply_chain
        && !sc.vulnerabilities.is_empty()
    {
        let _ = writeln!(
            s,
            "- Supply chain: {} vulnerability/vulnerabilities",
            sc.vulnerabilities.len()
        );
    }
    if let Some(ref cx) = receipt.evidence.complexity
        && cx.threshold_exceeded
    {
        let _ = writeln!(
            s,
            "- Complexity: threshold exceeded (max cyclomatic: {})",
            cx.max_cyclomatic
        );
    }
    let _ = writeln!(s);

    // Review plan (priority items only)
    let priority_items: Vec<_> = receipt
        .review_plan
        .iter()
        .filter(|item| item.priority <= 2)
        .collect();

    if !priority_items.is_empty() {
        let _ = writeln!(s, "**Priority review items**:");
        for item in priority_items {
            let _ = writeln!(s, "- {} ({})", item.path, item.reason);
        }
        let _ = writeln!(s);
    }

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
        _ => Verdict::Warn,
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
        _ => Verdict::Warn,
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
