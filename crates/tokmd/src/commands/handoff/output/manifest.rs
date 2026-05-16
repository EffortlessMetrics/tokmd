//! Manifest writer for handoff bundles.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use tokmd_types::HandoffManifest;

pub(crate) fn write_manifest_json(out_dir: &Path, manifest: &HandoffManifest) -> Result<usize> {
    let manifest_path = out_dir.join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(manifest)?;
    fs::write(&manifest_path, &manifest_json)
        .with_context(|| format!("Failed to write {}", manifest_path.display()))?;
    Ok(manifest_json.len())
}
