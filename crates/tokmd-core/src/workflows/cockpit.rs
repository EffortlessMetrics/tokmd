//! Cockpit workflow facade.

use anyhow::{Context as _, Result};

use crate::error;
use crate::settings::CockpitSettings;

/// Cockpit workflow: compute PR metrics and evidence gates.
///
/// Runs the cockpit analysis pipeline using pure settings types.
///
/// # Arguments
///
/// * `settings` - Cockpit settings (base/head refs, range mode, baseline)
///
/// # Returns
///
/// A `CockpitReceipt` containing PR metrics, evidence gates, and review plan.
///
/// # Example
///
/// ```rust
/// use std::fs;
/// use std::process::{Command, ExitStatus};
/// use tokmd_core::{cockpit_workflow, settings::CockpitSettings};
///
/// fn expect_success(status: ExitStatus) {
///     assert!(status.success(), "git command failed: {status}");
/// }
///
/// // Setup a temporary git repository for the test
/// let temp = tempfile::tempdir().expect("tempdir");
/// let dir = temp.path();
/// expect_success(Command::new("git").arg("init").current_dir(dir).status().expect("git init"));
/// expect_success(Command::new("git").args(["config", "user.email", "test@example.com"]).current_dir(dir).status().expect("git config"));
/// expect_success(Command::new("git").args(["config", "user.name", "Test User"]).current_dir(dir).status().expect("git config"));
/// expect_success(Command::new("git").args(["config", "commit.gpgsign", "false"]).current_dir(dir).status().expect("git config"));
/// expect_success(Command::new("git").args(["config", "tag.gpgsign", "false"]).current_dir(dir).status().expect("git config"));
/// fs::write(dir.join("main.rs"), "fn main() {}").expect("write");
/// expect_success(Command::new("git").args(["add", "main.rs"]).current_dir(dir).status().expect("git add"));
/// expect_success(Command::new("git").args(["commit", "-m", "Initial commit"]).current_dir(dir).status().expect("git commit"));
/// fs::write(dir.join("main.rs"), "fn main() { println!(\"Hello\"); }").expect("write");
/// expect_success(Command::new("git").args(["commit", "-am", "Second commit"]).current_dir(dir).status().expect("git commit"));
///
/// // Run from within the temporary git repository
/// let original_dir = std::env::current_dir().expect("current dir");
/// std::env::set_current_dir(dir).expect("cd");
///
/// let settings = CockpitSettings {
///     base: "HEAD~1".to_string(),
///     head: "HEAD".to_string(),
///     range_mode: "2dot".to_string(),
///     ..Default::default()
/// };
///
/// let receipt = cockpit_workflow(&settings);
/// std::env::set_current_dir(original_dir).expect("cd back");
/// let receipt = receipt.expect("Cockpit scan failed");
/// assert!(!receipt.review_plan.is_empty());
/// ```
pub fn cockpit_workflow(
    settings: &CockpitSettings,
) -> Result<tokmd_types::cockpit::CockpitReceipt> {
    use tokmd_types::cockpit::CockpitReceipt;

    if !tokmd_git::git_available() {
        anyhow::bail!("git is not available on PATH");
    }

    let cwd = std::env::current_dir().context("Failed to resolve current directory")?;
    let repo_root =
        tokmd_git::repo_root(&cwd).ok_or_else(|| anyhow::anyhow!("not inside a git repository"))?;

    let range_mode = parse_cockpit_range_mode(&settings.range_mode)?;

    let resolved_base =
        tokmd_git::resolve_base_ref(&repo_root, &settings.base).ok_or_else(|| {
            anyhow::anyhow!(
                "base ref '{}' not found and no fallback resolved",
                settings.base
            )
        })?;

    let baseline_path = settings.baseline.as_deref();

    let mut receipt: CockpitReceipt = tokmd_cockpit::compute_cockpit(
        &repo_root,
        &resolved_base,
        &settings.head,
        range_mode,
        baseline_path.map(std::path::Path::new),
    )?;

    // Load baseline and compute trend if provided.
    if let Some(baseline_path) = baseline_path {
        receipt.trend = Some(tokmd_cockpit::load_and_compute_trend(
            std::path::Path::new(baseline_path),
            &receipt,
        )?);
    }

    Ok(receipt)
}

pub(crate) fn parse_cockpit_range_mode(value: &str) -> Result<tokmd_git::GitRangeMode> {
    let normalized = value.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "two-dot" | "2dot" => Ok(tokmd_git::GitRangeMode::TwoDot),
        "three-dot" | "3dot" => Ok(tokmd_git::GitRangeMode::ThreeDot),
        _ => Err(error::TokmdError::invalid_field(
            "range_mode",
            "'two-dot', '2dot', 'three-dot', or '3dot'",
        )
        .into()),
    }
}
