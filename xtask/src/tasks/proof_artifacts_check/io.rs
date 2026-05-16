use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn read_json(path: &Path, label: &str) -> Result<Value> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read {label} artifact `{}`", path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse {label} artifact `{}`", path.display()))
}

pub(super) fn artifact_root_for(summary_path: &Path) -> PathBuf {
    summary_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

pub(super) fn write_observation<T: Serialize>(path: &Path, observation: &T) -> Result<()> {
    write_text(path, &serde_json::to_string_pretty(observation)?)
}

pub(super) fn write_proof_run_observation<T: Serialize>(
    path: &Path,
    observation: &T,
) -> Result<()> {
    write_text(path, &serde_json::to_string_pretty(observation)?)
}

pub(super) fn write_text(path: &Path, text: &str) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create `{}`", parent.display()))?;
    }
    fs::write(path, text).with_context(|| format!("failed to write `{}`", path.display()))
}
