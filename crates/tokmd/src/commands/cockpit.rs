//! Handler for the `tokmd cockpit` command.
//!
//! This is a thin CLI handler that delegates to `tokmd-cockpit` for computation
//! and rendering. Types are re-exported from `tokmd_types::cockpit`.

use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use tokmd_config as cli;

// Re-export all cockpit types for backwards compatibility with sensor.rs
pub use tokmd_types::cockpit::*;

// Re-export computation functions used by sensor.rs
#[cfg(feature = "git")]
pub use tokmd_cockpit::compute_cockpit;

/// Handle the cockpit command.
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

        let resolved_base =
            tokmd_git::resolve_base_ref(&repo_root, &args.base).ok_or_else(|| {
                anyhow::anyhow!(
                    "base ref '{}' not found and no fallback resolved. \
                 Use --base to specify a valid ref, or set TOKMD_GIT_BASE_REF",
                    args.base
                )
            })?;

        let mut receipt = tokmd_cockpit::compute_cockpit(
            &repo_root,
            &resolved_base,
            &args.head,
            range_mode,
            args.baseline.as_deref(),
        )?;

        // Load baseline and compute trend if provided
        if let Some(baseline_path) = &args.baseline {
            receipt.trend = Some(tokmd_cockpit::load_and_compute_trend(
                baseline_path,
                &receipt,
            )?);
        }

        // In sensor mode, write envelope to artifacts_dir
        if args.sensor_mode {
            let artifacts_dir = args
                .artifacts_dir
                .as_ref()
                .cloned()
                .unwrap_or_else(|| PathBuf::from("artifacts/tokmd"));
            tokmd_cockpit::render::write_sensor_artifacts(
                &artifacts_dir,
                &receipt,
                &resolved_base,
                &args.head,
            )?;

            // In sensor mode, always print JSON to stdout for piping
            let output = tokmd_cockpit::render::render_json(&receipt)?;
            print!("{}", output);
            return Ok(());
        }

        // Standard (non-sensor) mode
        let output = match args.format {
            cli::CockpitFormat::Json => tokmd_cockpit::render::render_json(&receipt)?,
            cli::CockpitFormat::Md => tokmd_cockpit::render::render_markdown(&receipt),
            cli::CockpitFormat::Sections => tokmd_cockpit::render::render_sections(&receipt),
        };

        if let Some(artifacts_dir) = &args.artifacts_dir {
            tokmd_cockpit::render::write_artifacts(artifacts_dir, &receipt)?;
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

#[cfg(test)]
mod tests {
    #[cfg(feature = "git")]
    use tokmd_cockpit::compute_determinism_gate;
    use tokmd_cockpit::{TrendDirection, format_signed_f64, sparkline, trend_direction_label};
    #[cfg(feature = "git")]
    use anyhow::Result;
    #[cfg(feature = "git")]
    use std::fs;
    #[cfg(feature = "git")]
    use tempfile::tempdir;

    #[test]
    fn sparkline_rises() {
        let s = sparkline(&[10.0, 20.0, 30.0]);
        assert_eq!(s.chars().count(), 3);
        assert!(s.ends_with('\u{2588}'));
    }

    #[test]
    fn sparkline_flat() {
        let s = sparkline(&[5.0, 5.0, 5.0]);
        assert_eq!(s, "\u{2584}\u{2584}\u{2584}");
    }

    #[test]
    fn sparkline_empty() {
        assert!(sparkline(&[]).is_empty());
    }

    #[test]
    fn signed_float_formatting() {
        assert_eq!(format_signed_f64(1.25), "+1.25");
        assert_eq!(format_signed_f64(0.0), "0.00");
        assert_eq!(format_signed_f64(-1.25), "-1.25");
    }

    #[test]
    fn trend_direction_labels_are_stable() {
        assert_eq!(
            trend_direction_label(TrendDirection::Improving),
            "improving"
        );
        assert_eq!(trend_direction_label(TrendDirection::Stable), "stable");
        assert_eq!(
            trend_direction_label(TrendDirection::Degrading),
            "degrading"
        );
    }

    #[cfg(feature = "git")]
    #[test]
    fn determinism_gate_errors_on_invalid_baseline_json() -> Result<()> {
        let tmp = tempdir()?;
        let baseline = tmp.path().join("baseline.json");
        fs::write(&baseline, "{")?;

        let err = match compute_determinism_gate(tmp.path(), Some(&baseline)) {
            Ok(_) => {
                return Err(anyhow::anyhow!(
                    "invalid JSON should not silently skip determinism gate"
                ));
            }
            Err(err) => err,
        };
        assert!(err.to_string().contains("failed to parse baseline JSON at"));
        Ok(())
    }

    #[cfg(feature = "git")]
    #[test]
    fn determinism_gate_skips_cockpit_receipt_baseline() -> Result<()> {
        let tmp = tempdir()?;
        let baseline = tmp.path().join("baseline.json");
        fs::write(&baseline, r#"{"mode":"cockpit"}"#)?;

        let gate = compute_determinism_gate(tmp.path(), Some(&baseline))?;
        assert!(gate.is_none());
        Ok(())
    }

    #[cfg(feature = "git")]
    #[test]
    fn determinism_gate_errors_on_non_baseline_json_shape() -> Result<()> {
        let tmp = tempdir()?;
        let baseline = tmp.path().join("baseline.json");
        fs::write(&baseline, r#"{"mode":"lang"}"#)?;

        let err = match compute_determinism_gate(tmp.path(), Some(&baseline)) {
            Ok(_) => {
                return Err(anyhow::anyhow!(
                    "non-baseline JSON should be a configuration error"
                ));
            }
            Err(err) => err,
        };
        assert!(
            err.to_string()
                .contains("is not a ComplexityBaseline (and not a cockpit receipt)")
        );
        Ok(())
    }
}
