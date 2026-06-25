use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

const BYTES_PER_GIB: u64 = 1024 * 1024 * 1024;
const DEFAULT_MIN_FREE_GIB: u64 = 8;
const SECONDS_PER_HOUR: u64 = 60 * 60;
const DEFAULT_STALE_TARGET_AGE_HOURS: u64 = 3;

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

/// Remove stale prior-run scoped temp directories for `label` from the system
/// temp dir.
///
/// `ScopedTempDir::drop` only fires on a clean process exit. When a gate run is
/// SIGKILLed or the runner cancels the job, the multi-GiB disposable target
/// directory survives under `/tmp`. On long-lived self-hosted runners these
/// orphans accumulate and eventually push free space below the
/// `ensure_min_free_space` floor, producing false gate failures (see #309).
///
/// This is best-effort: any directory matching the `tokmd-{label}-` prefix and
/// older than the configured age is removed. Age gating keeps a concurrently
/// running job's freshly written target directory untouched, so cleanup never
/// races an active sibling run. Errors are swallowed because reclaiming space is
/// an optimization, not a correctness requirement.
pub fn cleanup_stale_scoped_dirs(label: &str) -> Vec<PathBuf> {
    cleanup_stale_scoped_dirs_in(
        &std::env::temp_dir(),
        label,
        configured_stale_age(),
        SystemTime::now(),
    )
}

fn cleanup_stale_scoped_dirs_in(
    base: &Path,
    label: &str,
    max_age: Duration,
    now: SystemTime,
) -> Vec<PathBuf> {
    let prefix = format!("tokmd-{label}-");
    let mut removed = Vec::new();

    let Ok(entries) = std::fs::read_dir(base) else {
        return removed;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !name.starts_with(&prefix) {
            continue;
        }
        if !entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
            continue;
        }
        if !is_older_than(&entry, max_age, now) {
            continue;
        }
        if std::fs::remove_dir_all(&path).is_ok() {
            removed.push(path);
        }
    }

    removed
}

fn is_older_than(entry: &std::fs::DirEntry, max_age: Duration, now: SystemTime) -> bool {
    entry
        .metadata()
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|modified| now.duration_since(modified).ok())
        .map(|age| age >= max_age)
        .unwrap_or(false)
}

fn configured_stale_age() -> Duration {
    let hours = configured_stale_age_hours(std::env::var("TOKMD_GATE_STALE_HOURS").ok().as_deref());
    Duration::from_secs(hours.saturating_mul(SECONDS_PER_HOUR))
}

fn configured_stale_age_hours(value: Option<&str>) -> u64 {
    value
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_STALE_TARGET_AGE_HOURS)
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
        BYTES_PER_GIB, DEFAULT_STALE_TARGET_AGE_HOURS, SECONDS_PER_HOUR,
        cleanup_stale_scoped_dirs_in, configured_min_free_bytes, configured_min_free_gib,
        configured_stale_age_hours, parse_df_pk_available_bytes,
    };
    use std::time::{Duration, SystemTime};

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

    #[test]
    fn configured_stale_age_hours_defaults_when_env_is_missing() {
        assert_eq!(
            configured_stale_age_hours(None),
            DEFAULT_STALE_TARGET_AGE_HOURS
        );
    }

    #[test]
    fn configured_stale_age_hours_honors_env_override() {
        assert_eq!(configured_stale_age_hours(Some("6")), 6);
        assert_eq!(
            configured_stale_age_hours(Some("0")),
            DEFAULT_STALE_TARGET_AGE_HOURS
        );
        assert_eq!(
            configured_stale_age_hours(Some("bogus")),
            DEFAULT_STALE_TARGET_AGE_HOURS
        );
    }

    #[test]
    fn cleanup_removes_only_old_matching_dirs() -> anyhow::Result<()> {
        let base = tempfile::tempdir()?;
        let base_path = base.path();

        let stale_one = base_path.join("tokmd-gate-target-111-222");
        let stale_two = base_path.join("tokmd-gate-target-333-444");
        let other_label = base_path.join("tokmd-other-label-1-2");
        let unrelated = base_path.join("some-other-dir");
        for dir in [&stale_one, &stale_two, &other_label, &unrelated] {
            std::fs::create_dir(dir)?;
        }

        // Treat "now" as far in the future so the just-created dirs read as old.
        let future = SystemTime::now() + Duration::from_secs(10 * SECONDS_PER_HOUR);
        let max_age = Duration::from_secs(DEFAULT_STALE_TARGET_AGE_HOURS * SECONDS_PER_HOUR);

        let mut removed = cleanup_stale_scoped_dirs_in(base_path, "gate-target", max_age, future);
        removed.sort();

        assert_eq!(removed, vec![stale_one.clone(), stale_two.clone()]);
        assert!(!stale_one.exists());
        assert!(!stale_two.exists());
        assert!(other_label.exists(), "non-matching label must be preserved");
        assert!(unrelated.exists(), "unrelated dir must be preserved");
        Ok(())
    }

    #[test]
    fn cleanup_keeps_recent_dirs() -> anyhow::Result<()> {
        let base = tempfile::tempdir()?;
        let base_path = base.path();

        let recent = base_path.join("tokmd-gate-target-555-666");
        std::fs::create_dir(&recent)?;

        let max_age = Duration::from_secs(DEFAULT_STALE_TARGET_AGE_HOURS * SECONDS_PER_HOUR);
        let removed =
            cleanup_stale_scoped_dirs_in(base_path, "gate-target", max_age, SystemTime::now());

        assert!(removed.is_empty(), "freshly created dir must be kept");
        assert!(recent.exists());
        Ok(())
    }

    #[test]
    fn cleanup_returns_empty_for_missing_base() -> anyhow::Result<()> {
        let base = tempfile::tempdir()?;
        let missing = base.path().join("does-not-exist");
        let removed = cleanup_stale_scoped_dirs_in(
            &missing,
            "gate-target",
            Duration::from_secs(SECONDS_PER_HOUR),
            SystemTime::now(),
        );
        assert!(removed.is_empty());
        Ok(())
    }
}
