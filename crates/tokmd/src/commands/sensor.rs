//! Handler for the `tokmd sensor` command.
//!
//! Runs tokmd as a conforming sensor, producing a `SensorReport` envelope
//! backed by cockpit computation. Always writes a canonical JSON receipt
//! to the output path; with `--format md` also prints markdown to stdout.

use std::io::Write;

use anyhow::{Context, Result, bail};
use tokmd_config as cli;
use tokmd_envelope::findings;
use tokmd_envelope::{
    Artifact, Finding, FindingSeverity, GateItem, GateResults, SensorReport, ToolMeta, Verdict,
};

pub(crate) fn handle(args: cli::SensorArgs, global: &cli::GlobalArgs) -> Result<()> {
    #[cfg(not(feature = "git"))]
    {
        let _ = (&args, global);
        bail!("The sensor command requires the 'git' feature. Rebuild with --features git");
    }

    #[cfg(feature = "git")]
    {
        let _ = global; // scan opts not needed for cockpit path

        if !tokmd_git::git_available() {
            bail!("git is not available on PATH");
        }

        let cwd = std::env::current_dir().context("Failed to resolve current directory")?;
        let repo_root = tokmd_git::repo_root(&cwd)
            .ok_or_else(|| anyhow::anyhow!("not inside a git repository"))?;

        // Use two-dot range for sensor (same convention as cockpit)
        let range_mode = tokmd_git::GitRangeMode::TwoDot;

        // Run cockpit computation
        let cockpit_receipt =
            super::cockpit::compute_cockpit(&repo_root, &args.base, &args.head, range_mode)?;

        // Build the sensor report envelope
        let generated_at = now_iso8601();
        let verdict = map_verdict(cockpit_receipt.evidence.overall_status);

        let mut report = SensorReport::new(
            ToolMeta::tokmd(env!("CARGO_PKG_VERSION"), "sensor"),
            generated_at,
            verdict,
            build_summary(&cockpit_receipt, &args.base, &args.head),
        );

        // Map evidence gates → envelope gates
        report = report.with_gates(map_gates(&cockpit_receipt.evidence));

        // Emit findings from cockpit data
        emit_risk_findings(&mut report, &cockpit_receipt.risk);
        emit_contract_findings(&mut report, &cockpit_receipt.contracts);

        // Embed full cockpit receipt under data.cockpit_receipt
        let cockpit_json = serde_json::to_value(&cockpit_receipt)?;
        let data = serde_json::json!({
            "cockpit_receipt": cockpit_json,
        });
        report = report.with_data(data);

        // Add artifact reference for the output path
        let output_path = &args.output;
        let path_str = output_path.display().to_string().replace('\\', "/");
        report = report.with_artifacts(vec![Artifact::receipt(&path_str)]);

        // Always write canonical JSON receipt to output path
        let json_str = serde_json::to_string_pretty(&report)?;
        if let Some(parent) = output_path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::File::create(output_path)
            .with_context(|| format!("Failed to create output file: {}", output_path.display()))?;
        file.write_all(json_str.as_bytes())?;

        // When --format md, also print markdown to stdout
        match args.format {
            cli::SensorFormat::Json => {
                // JSON already written to file; also print to stdout
                print!("{}", json_str);
            }
            cli::SensorFormat::Md => {
                // JSON written to file; print markdown to stdout
                print!("{}", render_sensor_md(&report));
            }
        }

        Ok(())
    }
}

#[cfg(feature = "git")]
fn build_summary(receipt: &super::cockpit::CockpitReceipt, base: &str, head: &str) -> String {
    format!(
        "{} files changed, +{}/-{}, health {}/100, risk {:?} in {}..{}",
        receipt.change_surface.files_changed,
        receipt.change_surface.insertions,
        receipt.change_surface.deletions,
        receipt.code_health.score,
        receipt.risk.level,
        base,
        head,
    )
}

/// Map cockpit GateStatus → envelope Verdict.
#[cfg(feature = "git")]
fn map_verdict(status: super::cockpit::GateStatus) -> Verdict {
    match status {
        super::cockpit::GateStatus::Pass => Verdict::Pass,
        super::cockpit::GateStatus::Warn => Verdict::Warn,
        super::cockpit::GateStatus::Fail => Verdict::Fail,
        super::cockpit::GateStatus::Skipped => Verdict::Skip,
        super::cockpit::GateStatus::Pending => Verdict::Pending,
    }
}

/// Map cockpit Evidence → envelope GateResults.
#[cfg(feature = "git")]
fn map_gates(evidence: &super::cockpit::Evidence) -> GateResults {
    let mut items = Vec::new();

    // Mutation gate (always present)
    items.push(
        GateItem::new("mutation", map_verdict(evidence.mutation.meta.status))
            .with_source("computed"),
    );

    // Optional gates
    if let Some(ref dc) = evidence.diff_coverage {
        items.push(
            GateItem::new("diff_coverage", map_verdict(dc.meta.status))
                .with_threshold(0.8, dc.coverage_pct)
                .with_source("computed"),
        );
    }

    if let Some(ref c) = evidence.contracts {
        let mut gate =
            GateItem::new("contracts", map_verdict(c.meta.status)).with_source("computed");
        if c.failures > 0 {
            gate = gate.with_reason(format!("{} sub-gate(s) failed", c.failures));
        }
        items.push(gate);
    }

    if let Some(ref sc) = evidence.supply_chain {
        items.push(
            GateItem::new("supply_chain", map_verdict(sc.meta.status)).with_source("computed"),
        );
    }

    if let Some(ref det) = evidence.determinism {
        items.push(
            GateItem::new("determinism", map_verdict(det.meta.status)).with_source("computed"),
        );
    }

    if let Some(ref cx) = evidence.complexity {
        items
            .push(GateItem::new("complexity", map_verdict(cx.meta.status)).with_source("computed"));
    }

    GateResults::new(map_verdict(evidence.overall_status), items)
}

/// Emit risk findings from cockpit data.
#[cfg(feature = "git")]
fn emit_risk_findings(report: &mut SensorReport, risk: &super::cockpit::Risk) {
    for hotspot in &risk.hotspots_touched {
        report.add_finding(
            Finding::new(
                findings::risk::CHECK_ID,
                findings::risk::HOTSPOT,
                FindingSeverity::Warn,
                "Hotspot file touched",
                format!("{} is a high-churn file", hotspot),
            )
            .with_location(tokmd_envelope::FindingLocation::path(hotspot)),
        );
    }

    for path in &risk.bus_factor_warnings {
        report.add_finding(Finding::new(
            findings::risk::CHECK_ID,
            findings::risk::BUS_FACTOR,
            FindingSeverity::Warn,
            "Bus factor warning",
            format!("{} has single-author ownership", path),
        ));
    }
}

/// Emit contract findings from cockpit data.
#[cfg(feature = "git")]
fn emit_contract_findings(report: &mut SensorReport, contracts: &super::cockpit::Contracts) {
    if contracts.schema_changed {
        report.add_finding(Finding::new(
            findings::contract::CHECK_ID,
            findings::contract::SCHEMA_CHANGED,
            FindingSeverity::Info,
            "Schema version changed",
            "Schema version files were modified in this PR",
        ));
    }
    if contracts.api_changed {
        report.add_finding(Finding::new(
            findings::contract::CHECK_ID,
            findings::contract::API_CHANGED,
            FindingSeverity::Warn,
            "Public API changed",
            "Public API surface files were modified",
        ));
    }
    if contracts.cli_changed {
        report.add_finding(Finding::new(
            findings::contract::CHECK_ID,
            findings::contract::CLI_CHANGED,
            FindingSeverity::Info,
            "CLI interface changed",
            "CLI definition files were modified",
        ));
    }
}

fn render_sensor_md(report: &SensorReport) -> String {
    use std::fmt::Write;
    let mut s = String::new();
    let _ = writeln!(s, "## Sensor Report: {}", report.tool.name);
    let _ = writeln!(s);
    let _ = writeln!(s, "**Verdict**: {:?}", report.verdict);
    let _ = writeln!(s, "**Summary**: {}", report.summary);
    let _ = writeln!(s);

    if !report.findings.is_empty() {
        let _ = writeln!(s, "### Findings");
        let _ = writeln!(s);
        for f in &report.findings {
            let _ = writeln!(
                s,
                "- **[{:?}]** {}.{}: {} — {}",
                f.severity, f.check_id, f.code, f.title, f.message
            );
        }
        let _ = writeln!(s);
    }

    if let Some(ref gates) = report.gates {
        let _ = writeln!(s, "### Gates ({:?})", gates.status);
        let _ = writeln!(s);
        for g in &gates.items {
            let _ = writeln!(s, "- **{}**: {:?}", g.id, g.status);
        }
    }

    s
}

fn now_iso8601() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    // Simple UTC format - good enough for sensor reports
    let days = secs / 86400;
    let day_secs = secs % 86400;
    let hour = day_secs / 3600;
    let min = (day_secs % 3600) / 60;
    let sec = day_secs % 60;

    // Simple epoch day → date conversion (Hinnant algorithm)
    let z = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, m, d, hour, min, sec
    )
}
