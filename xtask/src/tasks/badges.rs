//! Generated public Shields endpoint badges.
//!
//! Public README badges are repo-scoped. Diff-scoped RIPR evidence belongs in
//! PR summaries and uploaded artifacts, not in committed endpoint JSON.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::cli::BadgesArgs;

const BADGE_ENDPOINT_DIR: &str = "badges";
const BADGE_ENDPOINT_TARGET_DIR: &str = "target/xtask/badges";

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct ShieldsEndpointBadge {
    #[serde(rename = "schemaVersion")]
    schema_version: u8,
    label: String,
    message: String,
    color: String,
}

pub fn run(args: BadgesArgs) -> Result<()> {
    let workspace_root = workspace_root_path()?;
    let target_dir = workspace_root.join(BADGE_ENDPOINT_TARGET_DIR);

    fs::create_dir_all(&target_dir).with_context(|| format!("create {}", target_dir.display()))?;

    let ripr = ripr_badge(&workspace_root)?;
    validate_shields_badge(&ripr, Some("ripr"))?;
    write_json_pretty(&target_dir.join("ripr.json"), &ripr)?;

    if args.check {
        let committed_dir = workspace_root.join(BADGE_ENDPOINT_DIR);
        compare_files(
            &committed_dir.join("ripr.json"),
            &target_dir.join("ripr.json"),
        )?;
        println!("badges: committed endpoints are current");
        return Ok(());
    }

    let committed_dir = workspace_root.join(BADGE_ENDPOINT_DIR);
    fs::create_dir_all(&committed_dir)
        .with_context(|| format!("create {}", committed_dir.display()))?;
    fs::copy(
        target_dir.join("ripr.json"),
        committed_dir.join("ripr.json"),
    )
    .with_context(|| "copy generated ripr endpoint into badges/".to_string())?;

    println!("badges: refreshed public endpoint JSON under badges/");
    Ok(())
}

fn ripr_badge(workspace_root: &Path) -> Result<ShieldsEndpointBadge> {
    let ripr_bin = std::env::var("RIPR_BIN").unwrap_or_else(|_| "ripr".to_string());

    let output = Command::new(&ripr_bin)
        .arg("check")
        .arg("--root")
        .arg(workspace_root)
        .arg("--format")
        .arg("repo-badge-shields")
        .current_dir(workspace_root)
        .output()
        .with_context(|| format!("run {ripr_bin} for repo-scoped ripr badge"))?;

    if !output.status.success() {
        bail!(
            "{ripr_bin} repo-badge-shields failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    serde_json::from_slice(&output.stdout)
        .with_context(|| format!("{ripr_bin} emitted invalid Shields endpoint JSON"))
}

pub(crate) fn validate_shields_badge(
    badge: &ShieldsEndpointBadge,
    expected_label: Option<&str>,
) -> Result<()> {
    if badge.schema_version != 1 {
        bail!("badge `{}` has unsupported schemaVersion", badge.label);
    }

    if let Some(expected_label) = expected_label
        && badge.label != expected_label
    {
        bail!(
            "badge label drifted: got `{}`, expected `{expected_label}`",
            badge.label
        );
    }

    if badge.message.trim().is_empty() {
        bail!("badge `{}` has empty message", badge.label);
    }

    if badge.color.trim().is_empty() {
        bail!("badge `{}` has empty color", badge.label);
    }

    Ok(())
}

fn write_json_pretty(path: &Path, badge: &ShieldsEndpointBadge) -> Result<()> {
    let body = serde_json::to_string_pretty(badge)?;
    fs::write(path, format!("{body}\n")).with_context(|| format!("write {}", path.display()))
}

fn compare_files(committed: &Path, generated: &Path) -> Result<()> {
    let committed_body = fs::read_to_string(committed)
        .with_context(|| format!("read committed badge endpoint {}", committed.display()))?;
    let generated_body = fs::read_to_string(generated)
        .with_context(|| format!("read generated badge endpoint {}", generated.display()))?;

    if committed_body != generated_body {
        bail!(
            "badge endpoint drift: {} differs from {}; run `cargo xtask badges`",
            committed.display(),
            generated.display()
        );
    }

    Ok(())
}

fn workspace_root_path() -> Result<PathBuf> {
    let metadata = cargo_metadata::MetadataCommand::new()
        .no_deps()
        .exec()
        .context("locate workspace root")?;
    Ok(metadata.workspace_root.into_std_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ripr_badge_shape_is_stable() {
        let badge = ShieldsEndpointBadge {
            schema_version: 1,
            label: "ripr".to_string(),
            message: "0".to_string(),
            color: "brightgreen".to_string(),
        };

        validate_shields_badge(&badge, Some("ripr")).unwrap();
    }

    #[test]
    fn badge_rejects_label_drift() {
        let badge = ShieldsEndpointBadge {
            schema_version: 1,
            label: "ripr".to_string(),
            message: "0".to_string(),
            color: "brightgreen".to_string(),
        };

        assert!(validate_shields_badge(&badge, Some("ripr+")).is_err());
    }
}
