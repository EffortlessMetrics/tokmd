//! Handler for the  command.
//!
//! Generates PR cockpit metrics for code review automation.

#[cfg(feature = "git")]
use anyhow::Context;
use anyhow::{Result, bail};
#[cfg(feature = "git")]
use std::io::Write;
#[cfg(feature = "git")]
use std::path::PathBuf;
use tokmd_config as cli;

#[cfg(feature = "git")]
mod impl_git;
#[cfg(feature = "git")]
pub(crate) use impl_git::*;

pub(crate) fn handle(args: cli::CockpitArgs, _global: &cli::GlobalArgs) -> Result<()> {
    #[cfg(not(feature = "git"))]
    {
        let _ = &args; // Silence unused warning
        bail!("The cockpit command requires the 'git' feature. Rebuild with --features git");
    }

    #[cfg(feature = "git")]
    {
        if !tokmd_git::git_available() {
            bail!("git is not available on PATH");
        }

        let cwd = std::env::current_dir().context("Failed to resolve current directory")?;
        let repo_root = tokmd_git::repo_root(&cwd)
            .ok_or_else(|| anyhow::anyhow!("not inside a git repository"))?;

        let range_mode = match args.diff_range {
            cli::DiffRangeMode::TwoDot => tokmd_git::GitRangeMode::TwoDot,
            cli::DiffRangeMode::ThreeDot => tokmd_git::GitRangeMode::ThreeDot,
        };

        let mut receipt = compute_cockpit(&repo_root, &args.base, &args.head, range_mode)?;

        // Load baseline and compute trend if provided
        if let Some(baseline_path) = &args.baseline {
            receipt.trend = Some(load_and_compute_trend(baseline_path, &receipt)?);
        }

        // In sensor mode, write envelope to artifacts_dir
        if args.sensor_mode {
            let artifacts_dir = args
                .artifacts_dir
                .as_ref()
                .cloned()
                .unwrap_or_else(|| PathBuf::from("artifacts/tokmd"));
            write_sensor_artifacts(&artifacts_dir, &receipt, &args.base, &args.head)?;

            // In sensor mode, always print JSON to stdout for piping
            let output = render_json(&receipt)?;
            print!("{}", output);
            return Ok(());
        }

        // Standard (non-sensor) mode
        let output = match args.format {
            cli::CockpitFormat::Json => render_json(&receipt)?,
            cli::CockpitFormat::Md => render_markdown(&receipt),
            cli::CockpitFormat::Sections => render_sections(&receipt),
        };

        if let Some(artifacts_dir) = &args.artifacts_dir {
            write_artifacts(artifacts_dir, &receipt)?;
        }

        if let Some(output_path) = &args.output {
            let mut file = std::fs::File::create(output_path).with_context(|| {
                format!("Failed to create output file: {}", output_path.display())
            })?;
            file.write_all(output.as_bytes())?;
        } else {
            print!("{}", output);
        }

        Ok(())
    }
}
