use crate::cli::BadgesArgs;
use anyhow::{Context, Result, bail};
use cargo_metadata::MetadataCommand;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

const BADGE_ENDPOINT_DIR: &str = "badges";
const BADGE_ENDPOINT_TARGET_DIR: &str = "target/xtask/badges";

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
struct ShieldsEndpointBadge {
    #[serde(rename = "schemaVersion")]
    schema_version: u8,
    label: String,
    message: String,
    color: String,
}

pub fn run(args: BadgesArgs) -> Result<()> {
    badges(args.check)
}

fn badges(check: bool) -> Result<()> {
    let workspace_root = workspace_root_path()?;
    let target_dir = workspace_root.join(BADGE_ENDPOINT_TARGET_DIR);

    std::fs::create_dir_all(&target_dir)
        .with_context(|| format!("failed to create {}", target_dir.display()))?;

    let ripr_plus = ripr_plus_badge(&workspace_root)?;
    validate_shields_badge(&ripr_plus, Some("ripr+"))?;
    write_json_pretty(&target_dir.join("ripr-plus.json"), &ripr_plus)?;

    if check {
        let committed_dir = workspace_root.join(BADGE_ENDPOINT_DIR);
        compare_files(
            &committed_dir.join("ripr-plus.json"),
            &target_dir.join("ripr-plus.json"),
        )?;

        println!("badges: committed endpoints are current");
        return Ok(());
    }

    let committed_dir = workspace_root.join(BADGE_ENDPOINT_DIR);
    std::fs::create_dir_all(&committed_dir)
        .with_context(|| format!("failed to create {}", committed_dir.display()))?;
    std::fs::copy(
        target_dir.join("ripr-plus.json"),
        committed_dir.join("ripr-plus.json"),
    )
    .with_context(|| "failed to refresh badges/ripr-plus.json")?;

    println!("badges: refreshed public endpoint JSON under badges/");
    Ok(())
}

fn ripr_plus_badge(workspace_root: &Path) -> Result<ShieldsEndpointBadge> {
    let ripr_bin = std::env::var("RIPR_BIN").unwrap_or_else(|_| "ripr".to_string());

    // Public README badge: repo-scoped, not PR/diff scoped.
    let output = Command::new(&ripr_bin)
        .arg("check")
        .arg("--root")
        .arg(workspace_root)
        .arg("--format")
        .arg("repo-badge-plus-shields")
        .current_dir(workspace_root)
        .output()
        .with_context(|| format!("failed to run {ripr_bin}; set RIPR_BIN to override"))?;

    if !output.status.success() {
        bail!(
            "{ripr_bin} repo-badge-plus-shields failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    serde_json::from_slice(&output.stdout)
        .with_context(|| format!("{ripr_bin} emitted invalid Shields endpoint JSON"))
}

fn validate_shields_badge(
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
    let json = serde_json::to_string_pretty(badge)?;
    std::fs::write(path, format!("{json}\n"))
        .with_context(|| format!("failed to write {}", path.display()))
}

fn compare_files(committed: &Path, generated: &Path) -> Result<()> {
    let committed_bytes = std::fs::read(committed)
        .with_context(|| format!("missing committed badge endpoint {}", committed.display()))?;
    let generated_bytes = std::fs::read(generated)
        .with_context(|| format!("missing generated badge endpoint {}", generated.display()))?;

    if committed_bytes != generated_bytes {
        bail!(
            "badge endpoint drift detected for {}; run `cargo xtask badges`",
            committed.display()
        );
    }

    Ok(())
}

fn workspace_root_path() -> Result<PathBuf> {
    let metadata = MetadataCommand::new()
        .no_deps()
        .exec()
        .context("failed to load cargo metadata")?;
    Ok(metadata.workspace_root.into_std_path_buf())
}

#[cfg(test)]
mod tests {
    use super::{ShieldsEndpointBadge, validate_shields_badge};

    #[test]
    fn ripr_plus_badge_shape_is_stable() {
        let badge = ShieldsEndpointBadge {
            schema_version: 1,
            label: "ripr+".to_string(),
            message: "0".to_string(),
            color: "brightgreen".to_string(),
        };

        validate_shields_badge(&badge, Some("ripr+")).unwrap();
    }

    #[test]
    fn badge_shape_rejects_empty_message() {
        let badge = ShieldsEndpointBadge {
            schema_version: 1,
            label: "ripr+".to_string(),
            message: " ".to_string(),
            color: "brightgreen".to_string(),
        };

        assert!(validate_shields_badge(&badge, Some("ripr+")).is_err());
    }
}
