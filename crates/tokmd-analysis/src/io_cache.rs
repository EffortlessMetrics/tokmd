//! Opt-in per-request read-through cache **prototype** for content file opens.
//!
//! This is the measurement instrument for the file-I/O cache decision described
//! in `docs/plans/file-io-cache-evidence.md`. The plan's PR D trace
//! ([`crate::io_trace`]) confirmed that a bounded `analyze` pass re-opens the
//! same `(read_mode, max_bytes, path)` key more than once (health preset:
//! ~1.54× on self-scan), but a trace counts open *attempts*, not the time
//! attributable to duplicate opens. This module lets a maintainer run a
//! health-preset before/after A/B and measure whether serving duplicate reads
//! from a request-scoped cache actually reduces `analyze total_ms` before any
//! production cache is proposed (the plan's PR E).
//!
//! Like [`crate::io_trace`], the cache is **thread-local and scope-gated**: it
//! is only active while a [`scope`] is installed on the current thread, and it
//! is dropped when the scope returns. When no scope is active,
//! [`get_or_read_bytes`] performs a single thread-local `Option` check and then
//! calls the reader directly, so the default `analyze` path is byte-for-byte
//! unchanged. This is a prototype instrument, not a product cache: it is only
//! ever installed by `cargo xtask perf-smoke --cache-io`.
//!
//! ## Scope of caching
//!
//! Only the byte-returning read modes ([`IoReadMode::Head`] and
//! [`IoReadMode::HeadTail`]) are cached, because those are the modes that fire
//! under the measured `health`/`security` presets and both return `Vec<u8>`.
//! `read_lines` and `read_text_capped` are intentionally not cached in this
//! prototype (the health preset never exercises them, and `read_text_capped`
//! already delegates to the inner head reader). A served value is an exact
//! clone of the first read for that key, so cached and uncached runs produce
//! identical bytes.

use std::cell::RefCell;
use std::collections::BTreeMap;

use anyhow::Result;
use std::path::Path;

use crate::io_trace::IoReadMode;

/// Aggregated read-cache statistics for a single [`scope`].
#[derive(Debug, Clone, Copy, Default)]
pub struct IoCacheReport {
    /// Cache lookups performed (one per cached-mode facade read).
    pub lookups: u64,
    /// Lookups served from the cache without opening the file.
    pub hits: u64,
    /// Lookups that read the file and populated the cache.
    pub misses: u64,
    /// Distinct `(mode, max_bytes, path)` entries retained at scope end.
    pub entries: u64,
    /// Total bytes served from the cache on hits (sum of served value lengths).
    pub bytes_served: u64,
}

impl IoCacheReport {
    /// Fraction of lookups served from the cache, or `0.0` when idle.
    #[must_use]
    pub fn hit_rate(&self) -> f64 {
        if self.lookups == 0 {
            0.0
        } else {
            self.hits as f64 / self.lookups as f64
        }
    }
}

#[derive(Default)]
struct IoCacheState {
    entries: BTreeMap<(IoReadMode, usize, String), Vec<u8>>,
    lookups: u64,
    hits: u64,
    bytes_served: u64,
}

impl IoCacheState {
    fn into_report(self) -> IoCacheReport {
        let entries = self.entries.len() as u64;
        IoCacheReport {
            lookups: self.lookups,
            hits: self.hits,
            misses: self.lookups.saturating_sub(self.hits),
            entries,
            bytes_served: self.bytes_served,
        }
    }
}

thread_local! {
    static ACTIVE: RefCell<Option<IoCacheState>> = const { RefCell::new(None) };
}

fn scope_active() -> bool {
    ACTIVE.with(|cell| cell.try_borrow().map(|g| g.is_some()).unwrap_or(false))
}

/// Look up a cached value, recording a lookup and (on hit) a served-bytes count.
///
/// The borrow is released before returning so the caller's reader never runs
/// while the cache is borrowed.
fn cache_get(key: &(IoReadMode, usize, String)) -> Option<Vec<u8>> {
    ACTIVE.with(|cell| {
        let mut guard = cell.try_borrow_mut().ok()?;
        let state = guard.as_mut()?;
        state.lookups = state.lookups.saturating_add(1);
        match state.entries.get(key) {
            Some(bytes) => {
                state.hits = state.hits.saturating_add(1);
                state.bytes_served = state.bytes_served.saturating_add(bytes.len() as u64);
                Some(bytes.clone())
            }
            None => None,
        }
    })
}

fn cache_insert(key: (IoReadMode, usize, String), bytes: &[u8]) {
    ACTIVE.with(|cell| {
        if let Ok(mut guard) = cell.try_borrow_mut()
            && let Some(state) = guard.as_mut()
        {
            state.entries.entry(key).or_insert_with(|| bytes.to_vec());
        }
    });
}

/// Serve a byte read from the request-scoped cache when a [`scope`] is active,
/// otherwise call `read` directly.
///
/// On a cache miss the file is read via `read`, the result is cloned into the
/// cache, and the bytes are returned. On a hit the cached bytes are cloned and
/// returned without invoking `read`, so the file is not re-opened. The returned
/// bytes are always identical to a fresh read of the same key.
pub(crate) fn get_or_read_bytes(
    mode: IoReadMode,
    max_bytes: usize,
    path: &Path,
    read: impl FnOnce() -> Result<Vec<u8>>,
) -> Result<Vec<u8>> {
    if !scope_active() {
        return read();
    }
    let key = (mode, max_bytes, path.to_string_lossy().into_owned());
    if let Some(bytes) = cache_get(&key) {
        return Ok(bytes);
    }
    let bytes = read()?;
    cache_insert(key, &bytes);
    Ok(bytes)
}

/// Restores the previous scope's state (if any) when a scope returns, even on
/// an early return from the cached closure.
struct Installed {
    previous: Option<IoCacheState>,
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

/// Run `f` with the request-scoped read cache active on the current thread and
/// return its result alongside the collected [`IoCacheReport`].
///
/// Nested scopes are isolated: the enclosing scope's cache is restored when
/// this scope returns, and reads inside the inner scope populate only the inner
/// cache.
pub fn scope<R>(f: impl FnOnce() -> R) -> (R, IoCacheReport) {
    let guard = Installed {
        previous: ACTIVE.with(|cell| {
            cell.try_borrow_mut()
                .ok()
                .and_then(|mut g| g.replace(IoCacheState::default()))
        }),
    };
    let result = f();
    let state = ACTIVE.with(|cell| cell.try_borrow_mut().ok().and_then(|mut g| g.take()));
    drop(guard);
    let report = state.map(IoCacheState::into_report).unwrap_or_default();
    (result, report)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_calls(counter: &std::cell::Cell<u32>, payload: &'static [u8]) -> Result<Vec<u8>> {
        counter.set(counter.get() + 1);
        Ok(payload.to_vec())
    }

    #[test]
    fn idle_passes_through_and_reports_empty() -> Result<()> {
        let counter = std::cell::Cell::new(0);
        // No scope active: reader is always called, cache is a no-op.
        let bytes = get_or_read_bytes(IoReadMode::Head, 16, Path::new("a.rs"), || {
            read_calls(&counter, b"hello")
        })?;
        assert_eq!(bytes, b"hello");
        assert_eq!(counter.get(), 1);

        let ((), report) = scope(|| {});
        assert_eq!(report.lookups, 0);
        assert_eq!(report.hits, 0);
        assert_eq!(report.misses, 0);
        assert_eq!(report.entries, 0);
        assert_eq!(report.hit_rate(), 0.0);
        Ok(())
    }

    #[test]
    fn second_read_of_same_key_is_served_from_cache() -> Result<()> {
        let counter = std::cell::Cell::new(0);
        let (bytes_pair, report) = scope(|| -> Result<(Vec<u8>, Vec<u8>)> {
            let first = get_or_read_bytes(IoReadMode::Head, 16, Path::new("a.rs"), || {
                read_calls(&counter, b"content")
            })?;
            let second = get_or_read_bytes(IoReadMode::Head, 16, Path::new("a.rs"), || {
                read_calls(&counter, b"content")
            })?;
            Ok((first, second))
        });
        let (first, second) = bytes_pair?;
        // The reader ran exactly once; the second lookup was a cache hit.
        assert_eq!(counter.get(), 1);
        // Cache parity: served bytes equal the originally-read bytes.
        assert_eq!(first, b"content");
        assert_eq!(second, b"content");
        assert_eq!(report.lookups, 2);
        assert_eq!(report.hits, 1);
        assert_eq!(report.misses, 1);
        assert_eq!(report.entries, 1);
        assert_eq!(report.bytes_served, b"content".len() as u64);
        assert_eq!(report.hit_rate(), 0.5);
        Ok(())
    }

    #[test]
    fn different_mode_or_limit_are_distinct_keys() {
        let counter = std::cell::Cell::new(0);
        let ((), report) = scope(|| {
            let _ = get_or_read_bytes(IoReadMode::Head, 16, Path::new("a.rs"), || {
                read_calls(&counter, b"x")
            });
            let _ = get_or_read_bytes(IoReadMode::HeadTail, 16, Path::new("a.rs"), || {
                read_calls(&counter, b"x")
            });
            let _ = get_or_read_bytes(IoReadMode::Head, 32, Path::new("a.rs"), || {
                read_calls(&counter, b"x")
            });
        });
        // Three distinct keys => three reads, no hits.
        assert_eq!(counter.get(), 3);
        assert_eq!(report.lookups, 3);
        assert_eq!(report.hits, 0);
        assert_eq!(report.entries, 3);
    }

    #[test]
    fn read_error_is_not_cached() {
        let ((), report) = scope(|| {
            let first: Result<Vec<u8>> =
                get_or_read_bytes(IoReadMode::Head, 16, Path::new("missing.rs"), || {
                    Err(anyhow::anyhow!("open failed"))
                });
            assert!(first.is_err());
        });
        // A failed read populates no entry, so a later read can retry.
        assert_eq!(report.lookups, 1);
        assert_eq!(report.hits, 0);
        assert_eq!(report.entries, 0);
    }

    #[test]
    fn nested_scopes_are_isolated() {
        let counter = std::cell::Cell::new(0);
        let ((), outer) = scope(|| {
            let _ = get_or_read_bytes(IoReadMode::Head, 8, Path::new("outer.rs"), || {
                read_calls(&counter, b"o")
            });
            let ((), inner) = scope(|| {
                // Inner scope starts empty: this is a miss even though the key
                // matches an outer entry.
                let _ = get_or_read_bytes(IoReadMode::Head, 8, Path::new("outer.rs"), || {
                    read_calls(&counter, b"o")
                });
            });
            assert_eq!(inner.lookups, 1);
            assert_eq!(inner.hits, 0);
            assert_eq!(inner.entries, 1);
            // Back in the outer scope, the original entry is still cached.
            let _ = get_or_read_bytes(IoReadMode::Head, 8, Path::new("outer.rs"), || {
                read_calls(&counter, b"o")
            });
        });
        assert_eq!(outer.lookups, 2);
        assert_eq!(outer.hits, 1);
        assert_eq!(outer.entries, 1);
    }
}

#[cfg(all(test, feature = "content"))]
mod content_wiring_tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn facade_read_head_is_served_from_cache_on_repeat() -> Result<()> {
        let tmp = tempfile::tempdir()?;
        let path = tmp.path().join("sample.rs");
        let mut file = std::fs::File::create(&path)?;
        file.write_all(b"fn main() {}\n")?;

        let (bytes_pair, report) = scope(|| -> Result<(Vec<u8>, Vec<u8>)> {
            let first = crate::content::io::read_head(&path, 4096)?;
            let second = crate::content::io::read_head(&path, 4096)?;
            Ok((first, second))
        });
        let (first, second) = bytes_pair?;

        // Byte parity between the read-through and the cached copy.
        assert_eq!(first, b"fn main() {}\n");
        assert_eq!(second, first);
        assert_eq!(report.lookups, 2);
        assert_eq!(report.hits, 1);
        assert_eq!(report.misses, 1);
        assert_eq!(report.entries, 1);
        Ok(())
    }

    #[test]
    fn facade_read_head_matches_uncached_bytes() -> Result<()> {
        let tmp = tempfile::tempdir()?;
        let path = tmp.path().join("parity.rs");
        let mut file = std::fs::File::create(&path)?;
        file.write_all(b"// parity check\npub fn f() {}\n")?;

        let uncached = crate::content::io::read_head(&path, 4096)?;
        let (cached, _report) = scope(|| crate::content::io::read_head(&path, 4096));
        assert_eq!(cached?, uncached);
        Ok(())
    }
}
