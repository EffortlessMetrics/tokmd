//! Opt-in per-request instrumentation for content file opens.
//!
//! Analysis content enrichers open the same repository files independently
//! (see `docs/plans/file-io-cache-evidence.md`). This module provides a
//! zero-cost-when-idle way to count `(read_mode, max_bytes, path)` open pairs
//! during a single analyze pass so a maintainer can measure the duplicate-read
//! rate before deciding whether a read-through cache is worth implementing.
//!
//! The trace is thread-local and only records while a [`scope`] is active on
//! the current thread. Analysis runs single-threaded, so a scope installed
//! around `analyze_workflow` captures every content open. When no scope is
//! active, [`record`] is a single thread-local `Option` check and returns
//! immediately.
//!
//! Opens are recorded at the content read facade (`content::io`) before the
//! underlying `File::open`, so the counts include open *attempts* (a repeated
//! attempt on a missing file is still a repeated open in the duplicate-read
//! hypothesis).

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// The bounded read strategy used at a content open site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IoReadMode {
    /// `read_head` — leading bytes only.
    Head,
    /// `read_head_tail` — leading and trailing bytes (entropy profiling).
    HeadTail,
    /// `read_lines` — bounded line collection (import parsing).
    Lines,
    /// `read_text_capped` — capped UTF-8 text (license/text scans).
    TextCapped,
}

impl IoReadMode {
    /// Stable identifier used as a per-mode report key.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            IoReadMode::Head => "head",
            IoReadMode::HeadTail => "head_tail",
            IoReadMode::Lines => "lines",
            IoReadMode::TextCapped => "text_capped",
        }
    }
}

/// Per-read-mode open statistics within a trace scope.
#[derive(Debug, Clone, Copy, Default)]
pub struct IoModeStats {
    /// Total opens recorded for this mode (includes repeats).
    pub opens: u64,
    /// Distinct `(max_bytes, path)` keys observed for this mode.
    pub unique_keys: u64,
}

/// Aggregated content-open statistics for a single trace scope.
#[derive(Debug, Clone, Default)]
pub struct IoTraceReport {
    /// Total content opens recorded across all modes.
    pub total_opens: u64,
    /// Distinct paths opened (ignoring mode and byte limit).
    pub unique_paths: u64,
    /// Distinct `(mode, max_bytes, path)` open keys.
    pub unique_keys: u64,
    /// Opens beyond the first for any repeated key (`total_opens - unique_keys`).
    pub duplicate_key_opens: u64,
    /// Highest open count observed for a single key.
    pub max_opens_for_key: u64,
    /// Per-mode breakdown keyed by [`IoReadMode::as_str`].
    pub by_mode: BTreeMap<&'static str, IoModeStats>,
}

impl IoTraceReport {
    /// Opens per distinct path, or `0.0` when nothing was opened.
    ///
    /// This maps to the plan's "repeated `(path, limit)` opens vs. file count"
    /// duplicate-read signal.
    #[must_use]
    pub fn opens_per_path(&self) -> f64 {
        if self.unique_paths == 0 {
            0.0
        } else {
            self.total_opens as f64 / self.unique_paths as f64
        }
    }
}

#[derive(Default)]
struct IoTraceState {
    keys: BTreeMap<(IoReadMode, usize, String), u64>,
    paths: BTreeSet<String>,
    total_opens: u64,
}

impl IoTraceState {
    fn record(&mut self, mode: IoReadMode, max_bytes: usize, path: &Path) {
        let path = path.to_string_lossy();
        self.total_opens = self.total_opens.saturating_add(1);
        let entry = self
            .keys
            .entry((mode, max_bytes, path.clone().into_owned()))
            .or_insert(0);
        *entry = entry.saturating_add(1);
        self.paths.insert(path.into_owned());
    }

    fn into_report(self) -> IoTraceReport {
        // usize -> u64 is lossless on all supported (<= 64-bit) targets.
        let unique_keys = self.keys.len() as u64;
        let unique_paths = self.paths.len() as u64;
        let mut by_mode: BTreeMap<&'static str, IoModeStats> = BTreeMap::new();
        let mut max_opens_for_key = 0u64;
        for ((mode, _bytes, _path), count) in &self.keys {
            max_opens_for_key = max_opens_for_key.max(*count);
            let entry = by_mode.entry(mode.as_str()).or_default();
            entry.opens = entry.opens.saturating_add(*count);
            entry.unique_keys = entry.unique_keys.saturating_add(1);
        }
        IoTraceReport {
            total_opens: self.total_opens,
            unique_paths,
            unique_keys,
            duplicate_key_opens: self.total_opens.saturating_sub(unique_keys),
            max_opens_for_key,
            by_mode,
        }
    }
}

thread_local! {
    static ACTIVE: RefCell<Option<IoTraceState>> = const { RefCell::new(None) };
}

/// Record a content open. No-op unless a [`scope`] is active on this thread.
pub(crate) fn record(mode: IoReadMode, max_bytes: usize, path: &Path) {
    ACTIVE.with(|cell| {
        if let Ok(mut guard) = cell.try_borrow_mut()
            && let Some(state) = guard.as_mut()
        {
            state.record(mode, max_bytes, path);
        }
    });
}

/// Restores the previous scope's state (if any) when a scope returns, even on
/// an early return from the traced closure.
struct Installed {
    previous: Option<IoTraceState>,
}

impl Drop for Installed {
    fn drop(&mut self) {
        let previous = self.previous.take();
        ACTIVE.with(|cell| {
            if let Ok(mut guard) = cell.try_borrow_mut() {
                *guard = previous;
            }
        });
    }
}

/// Run `f` with content-open tracing active on the current thread and return
/// its result alongside the collected [`IoTraceReport`].
///
/// Nested scopes are supported: the enclosing scope's state is restored when
/// this scope returns, and opens recorded inside the inner scope are counted
/// only by the inner report.
pub fn scope<R>(f: impl FnOnce() -> R) -> (R, IoTraceReport) {
    let guard = Installed {
        previous: ACTIVE.with(|cell| {
            cell.try_borrow_mut()
                .ok()
                .and_then(|mut g| g.replace(IoTraceState::default()))
        }),
    };
    let result = f();
    let state = ACTIVE.with(|cell| cell.try_borrow_mut().ok().and_then(|mut g| g.take()));
    drop(guard);
    let report = state.map(IoTraceState::into_report).unwrap_or_default();
    (result, report)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn idle_record_is_noop_and_report_is_empty() {
        record(IoReadMode::Head, 10, Path::new("outside/scope.rs"));
        let ((), report) = scope(|| {});
        assert_eq!(report.total_opens, 0);
        assert_eq!(report.unique_paths, 0);
        assert_eq!(report.unique_keys, 0);
        assert_eq!(report.duplicate_key_opens, 0);
        assert_eq!(report.opens_per_path(), 0.0);
    }

    #[test]
    fn records_repeated_opens_of_same_key() {
        let ((), report) = scope(|| {
            record(IoReadMode::Head, 4096, Path::new("src/a.rs"));
            record(IoReadMode::Head, 4096, Path::new("src/a.rs"));
            record(IoReadMode::Head, 4096, Path::new("src/a.rs"));
        });
        assert_eq!(report.total_opens, 3);
        assert_eq!(report.unique_paths, 1);
        assert_eq!(report.unique_keys, 1);
        assert_eq!(report.duplicate_key_opens, 2);
        assert_eq!(report.max_opens_for_key, 3);
        assert_eq!(report.opens_per_path(), 3.0);
    }

    #[test]
    fn same_path_different_mode_and_limit_are_distinct_keys() {
        let ((), report) = scope(|| {
            record(IoReadMode::Head, 4096, Path::new("src/a.rs"));
            record(IoReadMode::HeadTail, 4096, Path::new("src/a.rs"));
            record(IoReadMode::Head, 1024, Path::new("src/a.rs"));
        });
        assert_eq!(report.total_opens, 3);
        assert_eq!(report.unique_paths, 1);
        assert_eq!(report.unique_keys, 3);
        assert_eq!(report.duplicate_key_opens, 0);
        assert_eq!(report.by_mode.get("head").map(|s| s.opens), Some(2));
        assert_eq!(report.by_mode.get("head").map(|s| s.unique_keys), Some(2));
        assert_eq!(report.by_mode.get("head_tail").map(|s| s.opens), Some(1));
    }

    #[test]
    fn nested_scopes_are_isolated() {
        let ((), outer) = scope(|| {
            record(IoReadMode::Lines, 2048, Path::new("src/outer.rs"));
            let ((), inner) = scope(|| {
                record(IoReadMode::Lines, 2048, Path::new("src/inner.rs"));
                record(IoReadMode::Lines, 2048, Path::new("src/inner.rs"));
            });
            assert_eq!(inner.total_opens, 2);
            assert_eq!(inner.unique_paths, 1);
            record(IoReadMode::Lines, 2048, Path::new("src/outer.rs"));
        });
        assert_eq!(outer.total_opens, 2);
        assert_eq!(outer.unique_paths, 1);
        assert_eq!(outer.unique_keys, 1);
        assert_eq!(outer.duplicate_key_opens, 1);
    }
}

#[cfg(all(test, feature = "content"))]
mod content_wiring_tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn facade_read_head_records_head_open_within_scope() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("sample.rs");
        let mut file = std::fs::File::create(&path).expect("create sample");
        file.write_all(b"fn main() {}\n").expect("write sample");

        let (bytes, report) = scope(|| crate::content::io::read_head(&path, 4096));
        assert_eq!(bytes.expect("read_head"), b"fn main() {}\n");
        assert_eq!(report.total_opens, 1);
        assert_eq!(report.unique_paths, 1);
        assert_eq!(report.by_mode.get("head").map(|stats| stats.opens), Some(1));
    }

    #[test]
    fn facade_read_text_capped_records_single_text_capped_open() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("LICENSE");
        let mut file = std::fs::File::create(&path).expect("create license");
        file.write_all(b"MIT License\n").expect("write license");

        let (text, report) = scope(|| crate::content::io::read_text_capped(&path, 4096));
        assert_eq!(text.expect("read_text_capped"), "MIT License\n");
        // text_capped delegates to the inner head reader, so it must count as a
        // single text_capped open, not a duplicate head+text_capped pair.
        assert_eq!(report.total_opens, 1);
        assert_eq!(
            report.by_mode.get("text_capped").map(|stats| stats.opens),
            Some(1)
        );
        assert!(!report.by_mode.contains_key("head"));
    }
}
