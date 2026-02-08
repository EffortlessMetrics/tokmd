//! Handler for the `tokmd sensor` command.
//!
//! Runs tokmd as a conforming sensor, producing a `SensorReport` envelope
//! backed by cockpit computation. Implements a 3-layer output topology:
//!
//! 1. **report.json** — Thin envelope with findings, gates, summary metrics
//! 2. **extras/cockpit_receipt.json** — Full cockpit receipt sidecar
//! 3. **comment.md** — Markdown summary for PR comments

use std::io::Write;

use anyhow::{Context, Result, bail};
use tokmd_config as cli;
use tokmd_envelope::findings;
use tokmd_envelope::{
    Artifact, Finding, FindingSeverity, GateItem, GateResults, SensorReport, ToolMeta, Verdict,
};

/// Maximum findings emitted per category to avoid spamming the bus.
#[cfg(feature = "git")]
const MAX_FINDINGS_PER_CATEGORY: usize = 10;

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

        // Emit findings from cockpit data (all with fingerprints)
        emit_risk_findings(&mut report, &cockpit_receipt.risk);
        emit_contract_findings(&mut report, &cockpit_receipt.contracts);
        emit_complexity_findings(&mut report, &cockpit_receipt.evidence);
        emit_gate_findings(&mut report, &cockpit_receipt.evidence);

        // --- 3-layer output topology ---
        let output_path = &args.output;
        let artifact_dir = output_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        let extras_dir = artifact_dir.join("extras");
        let comment_path = artifact_dir.join("comment.md");

        // Ensure directories exist
        if !artifact_dir.as_os_str().is_empty() {
            std::fs::create_dir_all(artifact_dir)?;
        }
        std::fs::create_dir_all(&extras_dir)?;

        // Write extras/cockpit_receipt.json (full sidecar)
        let cockpit_sidecar_path = extras_dir.join("cockpit_receipt.json");
        let cockpit_json_str = serde_json::to_string_pretty(&cockpit_receipt)?;
        std::fs::write(&cockpit_sidecar_path, cockpit_json_str.as_bytes())?;

        // Slim data: only gates + summary metrics (no embedded cockpit receipt)
        let gates = map_gates(&cockpit_receipt.evidence);
        let data = serde_json::json!({
            "gates": serde_json::to_value(&gates)?,
            "summary_metrics": {
                "files_changed": cockpit_receipt.change_surface.files_changed,
                "insertions": cockpit_receipt.change_surface.insertions,
                "deletions": cockpit_receipt.change_surface.deletions,
                "health_score": cockpit_receipt.code_health.score,
                "risk_level": cockpit_receipt.risk.level.to_string(),
                "risk_score": cockpit_receipt.risk.score,
            },
        });
        report = report.with_data(data);

        // Build enriched artifacts array with id/mime
        let path_str = |p: &std::path::Path| p.display().to_string().replace('\\', "/");
        report = report.with_artifacts(vec![
            Artifact::receipt(path_str(output_path))
                .with_id("receipt")
                .with_mime("application/json"),
            Artifact::new("evidence", path_str(&cockpit_sidecar_path))
                .with_id("cockpit")
                .with_mime("application/json"),
            Artifact::comment(path_str(&comment_path))
                .with_id("comment")
                .with_mime("text/markdown"),
        ]);

        // Render markdown AFTER data + artifacts are populated so gates section is included
        let comment_md = render_sensor_md(&report);
        std::fs::write(&comment_path, comment_md.as_bytes())?;

        // Write canonical JSON report to output path
        let json_str = serde_json::to_string_pretty(&report)?;
        let mut file = std::fs::File::create(output_path)
            .with_context(|| format!("Failed to create output file: {}", output_path.display()))?;
        file.write_all(json_str.as_bytes())?;

        // Print to stdout based on format flag
        match args.format {
            cli::SensorFormat::Json => {
                print!("{}", json_str);
            }
            cli::SensorFormat::Md => {
                print!("{}", comment_md);
            }
        }

        Ok(())
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
            .with_location(tokmd_envelope::FindingLocation::path(hotspot))
            .with_fingerprint("tokmd"),
        );
    }

    for path in &risk.bus_factor_warnings {
        report.add_finding(
            Finding::new(
                findings::risk::CHECK_ID,
                findings::risk::BUS_FACTOR,
                FindingSeverity::Warn,
                "Bus factor warning",
                format!("{} has single-author ownership", path),
            )
            .with_fingerprint("tokmd"),
        );
    }
}

/// Emit contract findings from cockpit data.
#[cfg(feature = "git")]
fn emit_contract_findings(report: &mut SensorReport, contracts: &super::cockpit::Contracts) {
    if contracts.schema_changed {
        report.add_finding(
            Finding::new(
                findings::contract::CHECK_ID,
                findings::contract::SCHEMA_CHANGED,
                FindingSeverity::Info,
                "Schema version changed",
                "Schema version files were modified in this PR",
            )
            .with_fingerprint("tokmd"),
        );
    }
    if contracts.api_changed {
        report.add_finding(
            Finding::new(
                findings::contract::CHECK_ID,
                findings::contract::API_CHANGED,
                FindingSeverity::Warn,
                "Public API changed",
                "Public API surface files were modified",
            )
            .with_fingerprint("tokmd"),
        );
    }
    if contracts.cli_changed {
        report.add_finding(
            Finding::new(
                findings::contract::CHECK_ID,
                findings::contract::CLI_CHANGED,
                FindingSeverity::Info,
                "CLI interface changed",
                "CLI definition files were modified",
            )
            .with_fingerprint("tokmd"),
        );
    }
}

/// Emit complexity findings from cockpit evidence.
///
/// Inspects the complexity gate and emits per-file findings for high cyclomatic
/// complexity. Capped at `MAX_FINDINGS_PER_CATEGORY` per category.
#[cfg(feature = "git")]
fn emit_complexity_findings(report: &mut SensorReport, evidence: &super::cockpit::Evidence) {
    let Some(ref cx) = evidence.complexity else {
        return;
    };

    // Emit COMPLEXITY_HIGH findings for high-complexity files
    for file in cx
        .high_complexity_files
        .iter()
        .take(MAX_FINDINGS_PER_CATEGORY)
    {
        report.add_finding(
            Finding::new(
                findings::risk::CHECK_ID,
                findings::risk::COMPLEXITY_HIGH,
                FindingSeverity::Warn,
                "High cyclomatic complexity",
                format!(
                    "{} has cyclomatic complexity {} ({} functions)",
                    file.path, file.cyclomatic, file.function_count
                ),
            )
            .with_location(tokmd_envelope::FindingLocation::path(&file.path))
            .with_evidence(serde_json::json!({
                "cyclomatic": file.cyclomatic,
                "function_count": file.function_count,
                "max_function_length": file.max_function_length,
            }))
            .with_fingerprint("tokmd"),
        );
    }
}

/// Emit gate failure findings from cockpit evidence.
///
/// Inspects evidence gates and emits findings for any that failed.
#[cfg(feature = "git")]
fn emit_gate_findings(report: &mut SensorReport, evidence: &super::cockpit::Evidence) {
    // Mutation gate failure
    if evidence.mutation.meta.status == super::cockpit::GateStatus::Fail {
        report.add_finding(
            Finding::new(
                findings::gate::CHECK_ID,
                findings::gate::MUTATION_FAILED,
                FindingSeverity::Error,
                "Mutation gate failed",
                format!(
                    "{} mutation(s) survived testing",
                    evidence.mutation.survivors.len()
                ),
            )
            .with_fingerprint("tokmd"),
        );
    }

    // Diff coverage gate failure
    if let Some(ref dc) = evidence.diff_coverage
        && dc.meta.status == super::cockpit::GateStatus::Fail
    {
        report.add_finding(
            Finding::new(
                findings::gate::CHECK_ID,
                findings::gate::COVERAGE_FAILED,
                FindingSeverity::Error,
                "Diff coverage gate failed",
                format!(
                    "Coverage {:.1}% below threshold ({} of {} lines covered)",
                    dc.coverage_pct * 100.0,
                    dc.lines_covered,
                    dc.lines_added
                ),
            )
            .with_fingerprint("tokmd"),
        );
    }

    // Complexity gate failure
    if let Some(ref cx) = evidence.complexity
        && cx.meta.status == super::cockpit::GateStatus::Fail
    {
        report.add_finding(
            Finding::new(
                findings::gate::CHECK_ID,
                findings::gate::COMPLEXITY_FAILED,
                FindingSeverity::Error,
                "Complexity gate failed",
                format!(
                    "Max cyclomatic {} exceeds threshold ({} files analyzed)",
                    cx.max_cyclomatic, cx.files_analyzed
                ),
            )
            .with_fingerprint("tokmd"),
        );
    }
}

fn render_sensor_md(report: &SensorReport) -> String {
    use std::fmt::Write;
    let mut s = String::new();
    let _ = writeln!(s, "## Sensor Report: {}", report.tool.name);
    let _ = writeln!(s);
    let _ = writeln!(s, "**Verdict**: {}", report.verdict);
    let _ = writeln!(s, "**Summary**: {}", report.summary);
    let _ = writeln!(s);

    if !report.findings.is_empty() {
        let _ = writeln!(s, "### Findings");
        let _ = writeln!(s);
        for f in &report.findings {
            let _ = writeln!(
                s,
                "- **[{}]** {}.{}: {} — {}",
                f.severity, f.check_id, f.code, f.title, f.message
            );
        }
        let _ = writeln!(s);
    }

    // Extract gates from data (gates are embedded inside data, not top-level)
    if let Some(ref data) = report.data
        && let Some(gates_val) = data.get("gates")
        && let Ok(gates) = serde_json::from_value::<GateResults>(gates_val.clone())
    {
        let _ = writeln!(s, "### Gates ({})", gates.status);
        let _ = writeln!(s);
        for g in &gates.items {
            let _ = writeln!(s, "- **{}**: {}", g.id, g.status);
        }
    }

    s
}

fn now_iso8601() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}
