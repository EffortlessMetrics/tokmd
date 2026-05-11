//! Sensor envelope artifact writing and Markdown comment rendering.
//!
//! This module owns the command's 3-layer output topology while the parent
//! command owns git resolution, cockpit execution, and envelope assembly.

use std::io::Write;

use anyhow::{Context, Result};
use tokmd_envelope::{Artifact, GateResults, SensorReport};

use crate::cli;

use super::super::cockpit::CockpitReceipt;
use super::gates::map_gates;

pub(super) fn write_outputs(
    args: &cli::SensorArgs,
    mut report: SensorReport,
    cockpit_receipt: &CockpitReceipt,
) -> Result<()> {
    let output_path = &args.output;
    let artifact_dir = output_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    let extras_dir = artifact_dir.join("extras");
    let comment_path = artifact_dir.join("comment.md");

    if !artifact_dir.as_os_str().is_empty() {
        std::fs::create_dir_all(artifact_dir)?;
    }
    std::fs::create_dir_all(&extras_dir)?;

    let cockpit_sidecar_path = extras_dir.join("cockpit_receipt.json");
    let cockpit_json_str = serde_json::to_string_pretty(cockpit_receipt)?;
    std::fs::write(&cockpit_sidecar_path, cockpit_json_str.as_bytes())?;

    let gates = map_gates(&cockpit_receipt.evidence);
    let data = serde_json::json!({
        "gates": serde_json::to_value(gates)?,
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

    let comment_md = render_sensor_md(&report);
    std::fs::write(&comment_path, comment_md.as_bytes())?;

    let json_str = serde_json::to_string_pretty(&report)?;
    let mut file = std::fs::File::create(output_path)
        .with_context(|| format!("Failed to create output file: {}", output_path.display()))?;
    file.write_all(json_str.as_bytes())?;

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

#[cfg(test)]
mod tests {
    use tokmd_envelope::{
        Finding, FindingSeverity, GateItem, GateResults, SensorReport, ToolMeta, Verdict,
        findings as envelope_findings,
    };

    use super::*;

    #[test]
    fn render_sensor_md_includes_findings_and_gates() {
        let mut report = SensorReport::new(
            ToolMeta::tokmd("1.0.0", "sensor"),
            "2024-01-01T00:00:00Z".to_string(),
            Verdict::Warn,
            "Summary text".to_string(),
        );
        report.add_finding(
            Finding::new(
                envelope_findings::risk::CHECK_ID,
                envelope_findings::risk::HOTSPOT,
                FindingSeverity::Warn,
                "Hotspot",
                "High churn detected",
            )
            .with_fingerprint("tokmd"),
        );

        let gates = GateResults::new(
            Verdict::Warn,
            vec![GateItem::new("mutation", Verdict::Warn).with_source("computed")],
        );
        report = report.with_data(serde_json::json!({
            "gates": serde_json::to_value(gates).expect("should serialize"),
        }));

        let md = render_sensor_md(&report);
        assert!(md.contains("## Sensor Report: tokmd"));
        assert!(md.contains("### Findings"));
        assert!(md.contains("risk.hotspot"));
        assert!(md.contains("### Gates (warn)"));
        assert!(md.contains("mutation"));
    }
}
