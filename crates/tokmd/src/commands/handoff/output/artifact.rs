//! Shared artifact file writers.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;
use tokmd_types::{ArtifactEntry, ArtifactHash};

use super::hash::hash_bytes;

pub(super) fn write_json_artifact(
    out_dir: &Path,
    name: &str,
    relative_path: &str,
    description: &str,
    value: &Value,
) -> Result<ArtifactEntry> {
    let path = out_dir.join(relative_path);
    let json = serde_json::to_string_pretty(value)?;
    fs::write(&path, &json).with_context(|| format!("Failed to write {}", path.display()))?;

    Ok(ArtifactEntry {
        name: name.to_string(),
        path: relative_path.to_string(),
        description: description.to_string(),
        bytes: json.len() as u64,
        hash: Some(ArtifactHash {
            algo: "blake3".to_string(),
            hash: hash_bytes(json.as_bytes()),
        }),
    })
}

pub(super) fn write_text_artifact(
    out_dir: &Path,
    name: &str,
    relative_path: &str,
    description: &str,
    content: &str,
) -> Result<ArtifactEntry> {
    let path = out_dir.join(relative_path);
    fs::write(&path, content).with_context(|| format!("Failed to write {}", path.display()))?;

    Ok(ArtifactEntry {
        name: name.to_string(),
        path: relative_path.to_string(),
        description: description.to_string(),
        bytes: content.len() as u64,
        hash: Some(ArtifactHash {
            algo: "blake3".to_string(),
            hash: hash_bytes(content.as_bytes()),
        }),
    })
}
