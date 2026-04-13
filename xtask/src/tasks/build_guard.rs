use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

const BYTES_PER_GIB: u64 = 1024 * 1024 * 1024;
const DEFAULT_MIN_FREE_GIB: u64 = 8;

pub struct ScopedTempDir {
    path: PathBuf,
}

impl ScopedTempDir {
    pub fn new(label: &str) -> Result<Self> {
        let path = std::env::temp_dir().join(format!(
            "tokmd-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir_all(&path)
            .with_context(|| format!("failed to create temp dir {}", path.display()))?;
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for ScopedTempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

pub fn ensure_min_free_space(path: &Path, label: &str) -> Result<()> {
    let min_free_bytes = configured_min_free_bytes();
    let Some(available_bytes) = available_space_bytes(path)? else {
        return Ok(());
    };

    if available_bytes >= min_free_bytes {
        return Ok(());
    }

    bail!(
        "{label}: only {} free on {} (need at least {}). Clean old target directories or set TOKMD_MIN_FREE_GB to override the threshold.",
        human_bytes(available_bytes),
        path.display(),
        human_bytes(min_free_bytes)
    );
}

fn configured_min_free_bytes() -> u64 {
    configured_min_free_gib(std::env::var("TOKMD_MIN_FREE_GB").ok().as_deref()) * BYTES_PER_GIB
}

fn configured_min_free_gib(value: Option<&str>) -> u64 {
    value
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_MIN_FREE_GIB)
}

fn available_space_bytes(path: &Path) -> Result<Option<u64>> {
    if !cfg!(unix) {
        return Ok(None);
    }

    let output = Command::new("df")
        .arg("-Pk")
        .arg(path)
        .output()
        .with_context(|| format!("failed to run `df -Pk {}`", path.display()))?;

    if !output.status.success() {
        bail!(
            "`df -Pk {}` failed with exit code {}",
            path.display(),
            output.status.code().unwrap_or(-1)
        );
    }

    Ok(parse_df_pk_available_bytes(&String::from_utf8_lossy(
        &output.stdout,
    )))
}

fn parse_df_pk_available_bytes(output: &str) -> Option<u64> {
    output
        .lines()
        .nth(1)?
        .split_whitespace()
        .nth(3)?
        .parse::<u64>()
        .ok()
        .map(|blocks| blocks.saturating_mul(1024))
}

fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KiB", "MiB", "GiB"];

    let mut value = bytes as f64;
    let mut unit_idx = 0usize;
    while value >= 1024.0 && unit_idx + 1 < UNITS.len() {
        value /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{bytes} {}", UNITS[unit_idx])
    } else {
        format!("{value:.1} {}", UNITS[unit_idx])
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BYTES_PER_GIB, configured_min_free_bytes, configured_min_free_gib,
        parse_df_pk_available_bytes,
    };

    #[test]
    fn parse_df_pk_available_bytes_extracts_available_column() {
        let output = "\
Filesystem 1024-blocks Used Available Capacity Mounted on
/dev/sda1 157286400 80111236 69175164 54% /\n";

        assert_eq!(
            parse_df_pk_available_bytes(output),
            Some(69_175_164u64 * 1024)
        );
    }

    #[test]
    fn configured_min_free_gib_defaults_when_env_is_missing() {
        assert_eq!(configured_min_free_gib(None), 8);
    }

    #[test]
    fn configured_min_free_gib_honors_env_override() {
        assert_eq!(configured_min_free_gib(Some("12")), 12);
        assert_eq!(configured_min_free_gib(Some("0")), 8);
        assert_eq!(configured_min_free_gib(Some("bogus")), 8);
    }

    #[test]
    fn configured_min_free_bytes_uses_current_env_or_default() {
        let expected = configured_min_free_gib(std::env::var("TOKMD_MIN_FREE_GB").ok().as_deref())
            * BYTES_PER_GIB;
        assert_eq!(configured_min_free_bytes(), expected);
    }
}
