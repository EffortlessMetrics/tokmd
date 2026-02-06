//! Handler for the `tokmd sensor` command.
//!
//! Runs tokmd as a conforming sensor, producing a `SensorReport`.

use std::io::Write;

use anyhow::{Context, Result, bail};
use tokmd_config as cli;
use tokmd_envelope::{Artifact, Finding, FindingSeverity, SensorReport, ToolMeta, Verdict};
use tokmd_sensor::substrate_builder;
use tokmd_settings::ScanOptions;

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

        let cwd = std::env::current_dir().context("Failed to resolve current directory")?;
        let repo_root = tokmd_git::repo_root(&cwd)
            .ok_or_else(|| anyhow::anyhow!("not inside a git repository"))?;

        let scan_opts = ScanOptions::from(global);
        let repo_root_str = repo_root.display().to_string().replace('\\', "/");

        // Build diff range from git
        let diff_range = build_diff_range(&repo_root, &args.base, &args.head)?;

        // Build substrate
        let substrate = substrate_builder::build_substrate(
            &repo_root_str,
            &scan_opts,
            &["crates".to_string(), "packages".to_string()],
            2,
            Some(diff_range),
        )?;

        // Build the sensor report
        let generated_at = now_iso8601();
        let mut report = SensorReport::new(
            ToolMeta::tokmd(env!("CARGO_PKG_VERSION"), "sensor"),
            generated_at,
            Verdict::Pass,
            format!(
                "{} files, {} code lines, {} changed in {}..{}",
                substrate.files.len(),
                substrate.total_code_lines,
                substrate.diff_files().count(),
                args.base,
                args.head,
            ),
        );

        // Add findings for diff files
        let diff_file_count = substrate.diff_files().count();
        if diff_file_count > 0 {
            report.add_finding(Finding::new(
                "tokmd.sensor.diff_summary",
                FindingSeverity::Info,
                "Diff summary",
                format!(
                    "{} files changed in {}..{}",
                    diff_file_count, args.base, args.head
                ),
            ));
        }

        // Add data payload with substrate summary
        let data = serde_json::json!({
            "total_files": substrate.files.len(),
            "total_code_lines": substrate.total_code_lines,
            "total_tokens": substrate.total_tokens,
            "total_bytes": substrate.total_bytes,
            "languages": substrate.lang_summary.len(),
            "diff_files": diff_file_count,
        });
        report = report.with_data(data);

        // Add artifact reference
        if let Some(ref output_path) = args.output {
            let path_str = output_path.display().to_string().replace('\\', "/");
            report = report.with_artifacts(vec![Artifact::receipt(&path_str)]);
        }

        // Render output
        let output_str = match args.format {
            cli::SensorFormat::Json => serde_json::to_string_pretty(&report)?,
            cli::SensorFormat::Md => render_sensor_md(&report),
        };

        // Write output
        if let Some(output_path) = &args.output {
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut file = std::fs::File::create(output_path).with_context(|| {
                format!("Failed to create output file: {}", output_path.display())
            })?;
            file.write_all(output_str.as_bytes())?;
        } else {
            print!("{}", output_str);
        }

        Ok(())
    }
}

#[cfg(feature = "git")]
fn build_diff_range(
    repo_root: &std::path::Path,
    base: &str,
    head: &str,
) -> Result<tokmd_substrate::DiffRange> {
    use std::process::Command;

    // Get changed files
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("diff")
        .arg("--name-only")
        .arg(format!("{}..{}", base, head))
        .output()
        .context("Failed to run git diff --name-only")?;

    let changed_files: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.replace('\\', "/"))
        .collect();

    // Get commit count
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("rev-list")
        .arg("--count")
        .arg(format!("{}..{}", base, head))
        .output()
        .context("Failed to run git rev-list --count")?;

    let commit_count: usize = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .unwrap_or(0);

    // Get diff stats
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("diff")
        .arg("--shortstat")
        .arg(format!("{}..{}", base, head))
        .output()
        .context("Failed to run git diff --shortstat")?;

    let stat_line = String::from_utf8_lossy(&output.stdout);
    let (insertions, deletions) = parse_shortstat(&stat_line);

    Ok(tokmd_substrate::DiffRange {
        base: base.to_string(),
        head: head.to_string(),
        changed_files,
        commit_count,
        insertions,
        deletions,
    })
}

#[cfg(feature = "git")]
fn parse_shortstat(s: &str) -> (usize, usize) {
    let mut insertions = 0;
    let mut deletions = 0;
    for part in s.split(',') {
        let part = part.trim();
        if part.contains("insertion") {
            insertions = part
                .split_whitespace()
                .next()
                .and_then(|n| n.parse().ok())
                .unwrap_or(0);
        } else if part.contains("deletion") {
            deletions = part
                .split_whitespace()
                .next()
                .and_then(|n| n.parse().ok())
                .unwrap_or(0);
        }
    }
    (insertions, deletions)
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
            let _ = writeln!(s, "- **[{:?}]** {} — {}", f.severity, f.title, f.message);
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
