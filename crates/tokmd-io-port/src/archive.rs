//! Archive ingestion admission engine (EXPERIMENTAL, `feature = "archive"`).
//!
//! This module implements the **security-critical core** of the archive
//! ingestion sub-seam described in `docs/specs/repo-snapshot.md`: the
//! fail-closed admission policy (path-safety rejection) and the zip-bomb
//! resource limits. It treats every entry as hostile until it passes
//! normalization and the resource caps, and a single rejected entry fails the
//! whole snapshot build rather than silently dropping it.
//!
//! ## Deliberate dependency boundary
//!
//! This engine carries **zero decompression dependencies**. It operates over
//! [`RawArchiveEntry`] values — provider-agnostic descriptors of *already
//! decoded* entries (name, kind, compressed size, uncompressed bytes) that a
//! future codec adapter produces. The concrete container decoder (for example a
//! `snapshot_from_zip_bytes` that selects and pins an audited ZIP crate) is a
//! deferred follow-up: the spec lists the archive crate choice as an open
//! question, and adding a decompression dependency is a trust-surface decision
//! that belongs in its own dependency-maintenance PR with `cargo deny` proof.
//!
//! The split is intentional: the admission policy is the part that must be
//! correct and fully tested, and it is testable without a codec.
//!
//! EXPERIMENTAL / UNSTABLE: surface may change until a real consumer promotes
//! it. Do not treat it as a stable support promise.

use std::collections::BTreeSet;
use std::path::Path;

use crate::{MemFs, RepoSnapshot, SnapshotError};

/// Resource limits for archive ingestion, enforced fail-closed.
///
/// All limits have conservative defaults (see [`ArchiveLimits::default`]).
/// Callers may tighten them freely; relaxing a limit is a security-relevant
/// choice that the caller takes on explicitly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArchiveLimits {
    /// Maximum uncompressed size of any single admitted entry, in bytes.
    pub max_entry_size: u64,
    /// Maximum total uncompressed size across all admitted entries, in bytes.
    pub max_total_size: u64,
    /// Maximum number of admitted file entries.
    pub max_entries: usize,
    /// Maximum per-entry compression ratio (uncompressed / compressed).
    ///
    /// Guards against highly compressible bomb entries whose individual size is
    /// still under [`Self::max_entry_size`]. An entry declaring a non-empty
    /// payload with a zero compressed size is treated as an infinite ratio and
    /// rejected.
    pub max_ratio: u64,
}

impl Default for ArchiveLimits {
    /// Conservative defaults: 64 MiB per entry, 1 GiB total, 65,536 entries,
    /// and a 100x compression-ratio guard.
    fn default() -> Self {
        Self {
            max_entry_size: 64 * 1024 * 1024,
            max_total_size: 1024 * 1024 * 1024,
            max_entries: 65_536,
            max_ratio: 100,
        }
    }
}

/// The kind of an archive entry, as classified by the codec adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    /// A regular file entry (admissible, becomes a snapshot file).
    File,
    /// A directory entry (name is validated, but it contributes no file).
    Directory,
    /// A symlink, hardlink, device, or other non-regular entry (rejected).
    Other,
}

/// An untrusted, already-decoded archive entry.
///
/// A future codec adapter (ZIP/tar) produces one of these per container entry,
/// inflating the payload under the per-entry cap before handing it to the
/// admission engine. The engine assumes the byte payload was produced by a
/// bounded read and re-checks it against the limits regardless.
#[derive(Debug, Clone)]
pub struct RawArchiveEntry {
    /// The raw, untrusted entry name exactly as stored in the archive.
    pub name: String,
    /// The classified entry kind.
    pub kind: EntryKind,
    /// The compressed (stored) size of the entry, in bytes, as declared by the
    /// container. Used for the compression-ratio guard.
    pub compressed_size: u64,
    /// The uncompressed payload bytes (empty for directory entries).
    pub bytes: Vec<u8>,
}

impl RawArchiveEntry {
    /// Build a regular-file entry descriptor.
    pub fn file(name: impl Into<String>, compressed_size: u64, bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            name: name.into(),
            kind: EntryKind::File,
            compressed_size,
            bytes: bytes.into(),
        }
    }

    /// Build a directory entry descriptor (carries no payload).
    pub fn directory(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: EntryKind::Directory,
            compressed_size: 0,
            bytes: Vec::new(),
        }
    }
}

/// A fail-closed archive ingestion error. Identifies the first violated
/// path-safety or resource limit and the offending entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArchiveError {
    /// The entry name is absolute or carries a drive/UNC prefix.
    AbsolutePath {
        /// The rejected raw entry name.
        name: String,
    },
    /// The entry name contains a `..` parent-traversal component.
    Traversal {
        /// The rejected raw entry name.
        name: String,
    },
    /// The entry name is empty, contains a NUL, or normalizes to nothing.
    InvalidName {
        /// The rejected raw entry name.
        name: String,
        /// Why the name was rejected.
        reason: &'static str,
    },
    /// The entry is a symlink, hardlink, device, or other non-regular entry.
    NonRegularEntry {
        /// The rejected raw entry name.
        name: String,
    },
    /// Two entries normalize to the same path (case-insensitively).
    DuplicateEntry {
        /// The normalized path that collided.
        normalized: String,
    },
    /// A single entry exceeds the per-entry uncompressed cap.
    EntryTooLarge {
        /// The offending normalized path.
        name: String,
        /// The entry's uncompressed size.
        size: u64,
        /// The per-entry cap that was exceeded.
        limit: u64,
    },
    /// The cumulative uncompressed size exceeds the total cap.
    TotalTooLarge {
        /// The cumulative size at the point of breach.
        size: u64,
        /// The total cap that was exceeded.
        limit: u64,
    },
    /// The number of admitted file entries exceeds the count cap.
    TooManyEntries {
        /// The count at the point of breach.
        count: usize,
        /// The entry-count cap that was exceeded.
        limit: usize,
    },
    /// An entry's compression ratio exceeds the guard.
    RatioExceeded {
        /// The offending normalized path.
        name: String,
        /// The declared compressed size.
        compressed: u64,
        /// The actual uncompressed size.
        uncompressed: u64,
        /// The ratio guard that was exceeded.
        limit: u64,
    },
    /// Capturing an admitted entry into the snapshot failed unexpectedly.
    ///
    /// This should not occur in normal operation because admitted bytes are
    /// staged in-memory; it exists so the build path never panics.
    Capture {
        /// The normalized path that failed to capture.
        name: String,
    },
    /// The container could not be decoded (malformed archive, unsupported
    /// compression method, or a decompression error). Only produced by a codec
    /// adapter such as [`snapshot_from_zip_bytes`]; the dependency-free
    /// admission engine never returns this.
    MalformedArchive {
        /// A human-readable description of the decode failure.
        reason: String,
    },
}

impl std::fmt::Display for ArchiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArchiveError::AbsolutePath { name } => {
                write!(f, "rejected absolute or drive/UNC path: '{name}'")
            }
            ArchiveError::Traversal { name } => {
                write!(f, "rejected parent-traversal path: '{name}'")
            }
            ArchiveError::InvalidName { name, reason } => {
                write!(f, "rejected invalid entry name '{name}': {reason}")
            }
            ArchiveError::NonRegularEntry { name } => {
                write!(f, "rejected non-regular entry: '{name}'")
            }
            ArchiveError::DuplicateEntry { normalized } => {
                write!(f, "rejected duplicate entry: '{normalized}'")
            }
            ArchiveError::EntryTooLarge { name, size, limit } => {
                write!(
                    f,
                    "entry '{name}' uncompressed size {size} exceeds per-entry limit {limit}"
                )
            }
            ArchiveError::TotalTooLarge { size, limit } => {
                write!(f, "total uncompressed size {size} exceeds limit {limit}")
            }
            ArchiveError::TooManyEntries { count, limit } => {
                write!(f, "entry count {count} exceeds limit {limit}")
            }
            ArchiveError::RatioExceeded {
                name,
                compressed,
                uncompressed,
                limit,
            } => write!(
                f,
                "entry '{name}' compression ratio ({uncompressed}/{compressed}) exceeds limit {limit}"
            ),
            ArchiveError::Capture { name } => {
                write!(f, "failed to capture admitted entry: '{name}'")
            }
            ArchiveError::MalformedArchive { reason } => {
                write!(f, "malformed or undecodable archive: {reason}")
            }
        }
    }
}

impl std::error::Error for ArchiveError {}

/// Validate and normalize an untrusted entry name to the forward-slash rule.
///
/// Returns the normalized path on success. Rejects (fail-closed) absolute or
/// drive/UNC paths, `..` traversal, empty/NUL names, and names that normalize
/// to nothing.
fn validate_and_normalize_name(name: &str) -> Result<String, ArchiveError> {
    if name.is_empty() {
        return Err(ArchiveError::InvalidName {
            name: name.to_string(),
            reason: "empty name",
        });
    }
    if name.contains('\0') {
        return Err(ArchiveError::InvalidName {
            name: name.to_string(),
            reason: "NUL byte in name",
        });
    }

    // Absolute (leading separator) or UNC (leading `\\`).
    let mut chars = name.chars();
    let first = chars.next();
    if matches!(first, Some('/') | Some('\\')) {
        return Err(ArchiveError::AbsolutePath {
            name: name.to_string(),
        });
    }
    // Drive prefix such as `C:` — a single ASCII letter followed by a colon.
    if let (Some(c0), Some(c1)) = (first, chars.next())
        && c0.is_ascii_alphabetic()
        && c1 == ':'
    {
        return Err(ArchiveError::AbsolutePath {
            name: name.to_string(),
        });
    }

    // Treat both separators uniformly, then validate every component.
    let unified = name.replace('\\', "/");
    let mut components: Vec<&str> = Vec::new();
    for component in unified.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                return Err(ArchiveError::Traversal {
                    name: name.to_string(),
                });
            }
            other => components.push(other),
        }
    }

    let normalized = components.join("/");
    if normalized.is_empty() {
        return Err(ArchiveError::InvalidName {
            name: name.to_string(),
            reason: "name normalizes to empty path",
        });
    }
    Ok(normalized)
}

/// Build a [`RepoSnapshot`] from untrusted archive entries, enforcing all
/// path-safety and resource limits fail-closed.
///
/// The engine treats every entry as hostile: it normalizes and validates each
/// name, rejects non-regular entries, enforces the [`ArchiveLimits`], and fails
/// the entire build on the first violation. No partial snapshot is produced on
/// error.
///
/// Directory entries have their names validated but contribute no file and do
/// not count toward the entry cap.
///
/// # Errors
///
/// Returns the first [`ArchiveError`] encountered: an invalid/absolute/traversal
/// name, a non-regular entry, a duplicate (case-insensitive) path, or a breach
/// of the per-entry, total, count, or compression-ratio limit.
pub fn snapshot_from_entries(
    root: impl AsRef<Path>,
    entries: impl IntoIterator<Item = RawArchiveEntry>,
    limits: &ArchiveLimits,
) -> Result<RepoSnapshot, ArchiveError> {
    let mut staged = MemFs::new();
    let mut admitted_paths: Vec<String> = Vec::new();
    let mut seen_ci: BTreeSet<String> = BTreeSet::new();
    let mut file_count: usize = 0;
    let mut total_size: u64 = 0;

    for entry in entries {
        let normalized = validate_and_normalize_name(&entry.name)?;

        match entry.kind {
            EntryKind::Other => {
                return Err(ArchiveError::NonRegularEntry { name: entry.name });
            }
            EntryKind::Directory => {
                // Name validated above; directories imply no file payload.
                continue;
            }
            EntryKind::File => {}
        }

        // Reject ambiguous duplicates (exact or case-insensitive collision).
        let ci_key = normalized.to_lowercase();
        if !seen_ci.insert(ci_key) {
            return Err(ArchiveError::DuplicateEntry { normalized });
        }

        // Entry-count cap.
        file_count = file_count.saturating_add(1);
        if file_count > limits.max_entries {
            return Err(ArchiveError::TooManyEntries {
                count: file_count,
                limit: limits.max_entries,
            });
        }

        // Per-entry size cap.
        let uncompressed = u64::try_from(entry.bytes.len()).unwrap_or(u64::MAX);
        if uncompressed > limits.max_entry_size {
            return Err(ArchiveError::EntryTooLarge {
                name: normalized,
                size: uncompressed,
                limit: limits.max_entry_size,
            });
        }

        // Compression-ratio guard (catches small-compressed/large-inflated bombs).
        if entry.compressed_size == 0 {
            if uncompressed > 0 {
                return Err(ArchiveError::RatioExceeded {
                    name: normalized,
                    compressed: entry.compressed_size,
                    uncompressed,
                    limit: limits.max_ratio,
                });
            }
        } else if uncompressed > entry.compressed_size.saturating_mul(limits.max_ratio) {
            return Err(ArchiveError::RatioExceeded {
                name: normalized,
                compressed: entry.compressed_size,
                uncompressed,
                limit: limits.max_ratio,
            });
        }

        // Total size cap.
        total_size = total_size.saturating_add(uncompressed);
        if total_size > limits.max_total_size {
            return Err(ArchiveError::TotalTooLarge {
                size: total_size,
                limit: limits.max_total_size,
            });
        }

        staged.add_bytes(std::path::PathBuf::from(&normalized), entry.bytes);
        admitted_paths.push(normalized);
    }

    let mut builder = RepoSnapshot::builder(&staged, root);
    builder
        .add_paths(&admitted_paths)
        .map_err(|err| match err {
            SnapshotError::Read { path, .. } => ArchiveError::Capture { name: path },
        })?;
    Ok(builder.build())
}

/// Decode an in-memory ZIP archive into a [`RepoSnapshot`], enforcing all
/// path-safety and resource limits fail-closed (`feature = "archive-zip"`).
///
/// This is the codec adapter described in `docs/specs/repo-snapshot.md`: it
/// enumerates the ZIP central directory, classifies each entry, **bounded-
/// inflates** regular files so a hostile declared size cannot force unbounded
/// allocation, and delegates the authoritative admission policy to
/// [`snapshot_from_entries`]. The archive is treated as hostile input.
///
/// Decoding is buffered (the whole archive is provided as a byte slice) per the
/// spec's first-implementation choice. Compression support is deflate-only
/// (plus stored); other methods surface as
/// [`ArchiveError::MalformedArchive`].
///
/// Memory safety against zip bombs is enforced at two layers: each regular
/// entry is inflated through a reader capped at
/// [`ArchiveLimits::max_entry_size`] + 1 byte (so an oversized entry is
/// detected without materializing its full declared payload), and a running
/// total guard stops decoding before the staged byte buffer can exceed
/// [`ArchiveLimits::max_total_size`]. [`snapshot_from_entries`] then re-checks
/// every limit as the single authoritative validator.
///
/// # Errors
///
/// Returns [`ArchiveError::MalformedArchive`] if the bytes are not a decodable
/// ZIP (or an entry uses an unsupported compression method), or any other
/// [`ArchiveError`] surfaced by the admission policy (invalid/traversal/
/// absolute name, non-regular entry, duplicate, or a breached size/count/ratio
/// limit).
#[cfg(feature = "archive-zip")]
pub fn snapshot_from_zip_bytes(
    root: impl AsRef<Path>,
    bytes: &[u8],
    limits: &ArchiveLimits,
) -> Result<RepoSnapshot, ArchiveError> {
    use std::io::{Cursor, Read};

    let reader = Cursor::new(bytes);
    let mut archive =
        zip::ZipArchive::new(reader).map_err(|err| ArchiveError::MalformedArchive {
            reason: err.to_string(),
        })?;

    let mut entries: Vec<RawArchiveEntry> = Vec::new();
    // Running total guard: bound the staged byte buffer during decode so a
    // multi-entry bomb cannot accumulate past the total cap before the
    // admission engine runs. The engine remains authoritative.
    let mut running_total: u64 = 0;
    // Per-entry read ceiling: one byte over the cap is enough to detect an
    // oversized entry without materializing the full declared payload.
    let read_ceiling = limits.max_entry_size.saturating_add(1);

    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|err| ArchiveError::MalformedArchive {
                reason: err.to_string(),
            })?;

        // Raw, untrusted stored name; the admission engine validates it.
        let name = file.name().to_string();

        // Classify: symlink/device/other (via unix type bits) before directory,
        // so a symlink whose name lacks a trailing slash is still rejected.
        let kind = classify_zip_entry(file.unix_mode(), file.is_dir());

        match kind {
            EntryKind::File => {
                let compressed_size = file.compressed_size();
                let mut buf = Vec::new();
                file.by_ref()
                    .take(read_ceiling)
                    .read_to_end(&mut buf)
                    .map_err(|err| ArchiveError::MalformedArchive {
                        reason: err.to_string(),
                    })?;

                let uncompressed = u64::try_from(buf.len()).unwrap_or(u64::MAX);
                // Early total guard to bound peak memory; the admission engine
                // re-checks this authoritatively.
                running_total = running_total.saturating_add(uncompressed);
                if running_total > limits.max_total_size {
                    return Err(ArchiveError::TotalTooLarge {
                        size: running_total,
                        limit: limits.max_total_size,
                    });
                }

                entries.push(RawArchiveEntry::file(name, compressed_size, buf));
            }
            EntryKind::Directory => entries.push(RawArchiveEntry::directory(name)),
            EntryKind::Other => entries.push(RawArchiveEntry {
                name,
                kind: EntryKind::Other,
                compressed_size: 0,
                bytes: Vec::new(),
            }),
        }
    }

    snapshot_from_entries(root, entries, limits)
}

/// Classify a ZIP entry into an [`EntryKind`] from its optional unix mode and
/// the codec's directory flag.
///
/// Symlinks and other non-regular unix types (FIFO, block/char device, socket)
/// are classified as [`EntryKind::Other`] so the admission engine rejects them.
/// Entries with no unix mode (e.g. archives written on Windows) fall back to
/// the directory flag, then to a regular file.
#[cfg(feature = "archive-zip")]
fn classify_zip_entry(unix_mode: Option<u32>, is_dir: bool) -> EntryKind {
    const S_IFMT: u32 = 0o170000;
    const S_IFREG: u32 = 0o100000;
    const S_IFDIR: u32 = 0o040000;

    if let Some(mode) = unix_mode {
        match mode & S_IFMT {
            S_IFREG => return EntryKind::File,
            S_IFDIR => return EntryKind::Directory,
            0 => {} // No type bits recorded; fall back to the directory flag.
            _ => return EntryKind::Other, // symlink, device, fifo, socket, ...
        }
    }

    if is_dir {
        EntryKind::Directory
    } else {
        EntryKind::File
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VirtualFile;

    fn limits() -> ArchiveLimits {
        ArchiveLimits::default()
    }

    #[test]
    fn admits_benign_entries_deterministically() -> Result<(), ArchiveError> {
        let entries = vec![
            RawArchiveEntry::file("z/last.rs", 4, b"zzzz".to_vec()),
            RawArchiveEntry::file("a/first.rs", 4, b"aaaa".to_vec()),
            RawArchiveEntry::directory("a/"),
        ];
        let snap = snapshot_from_entries("repo", entries, &limits())?;
        let paths: Vec<&str> = snap.paths().collect();
        assert_eq!(paths, vec!["a/first.rs", "z/last.rs"]);
        assert_eq!(
            snap.get("a/first.rs").map(VirtualFile::bytes),
            Some(&b"aaaa"[..])
        );
        Ok(())
    }

    #[test]
    fn rejects_absolute_path() {
        let entries = vec![RawArchiveEntry::file("/etc/passwd", 1, b"x".to_vec())];
        let err = snapshot_from_entries("repo", entries, &limits()).unwrap_err();
        assert!(matches!(err, ArchiveError::AbsolutePath { .. }));
    }

    #[test]
    fn rejects_drive_prefix() {
        let entries = vec![RawArchiveEntry::file("C:\\windows\\evil", 1, b"x".to_vec())];
        let err = snapshot_from_entries("repo", entries, &limits()).unwrap_err();
        assert!(matches!(err, ArchiveError::AbsolutePath { .. }));
    }

    #[test]
    fn rejects_unc_prefix() {
        let entries = vec![RawArchiveEntry::file(
            "\\\\host\\share\\f",
            1,
            b"x".to_vec(),
        )];
        let err = snapshot_from_entries("repo", entries, &limits()).unwrap_err();
        assert!(matches!(err, ArchiveError::AbsolutePath { .. }));
    }

    #[test]
    fn rejects_parent_traversal() {
        for name in ["../escape.rs", "a/../../escape.rs", "a\\..\\..\\escape"] {
            let entries = vec![RawArchiveEntry::file(name, 1, b"x".to_vec())];
            let err = snapshot_from_entries("repo", entries, &limits()).unwrap_err();
            assert!(matches!(err, ArchiveError::Traversal { .. }), "name={name}");
        }
    }

    #[test]
    fn rejects_nul_in_name() {
        let entries = vec![RawArchiveEntry::file("a\0b.rs", 1, b"x".to_vec())];
        let err = snapshot_from_entries("repo", entries, &limits()).unwrap_err();
        assert!(matches!(err, ArchiveError::InvalidName { .. }));
    }

    #[test]
    fn rejects_empty_and_dot_only_names() {
        for name in ["", ".", "/", "./"] {
            let entries = vec![RawArchiveEntry::file(name, 1, b"x".to_vec())];
            let err = snapshot_from_entries("repo", entries, &limits()).unwrap_err();
            assert!(
                matches!(
                    err,
                    ArchiveError::InvalidName { .. } | ArchiveError::AbsolutePath { .. }
                ),
                "name={name:?}"
            );
        }
    }

    #[test]
    fn rejects_non_regular_entry() {
        let entries = vec![RawArchiveEntry {
            name: "link".to_string(),
            kind: EntryKind::Other,
            compressed_size: 0,
            bytes: Vec::new(),
        }];
        let err = snapshot_from_entries("repo", entries, &limits()).unwrap_err();
        assert!(matches!(err, ArchiveError::NonRegularEntry { .. }));
    }

    #[test]
    fn rejects_case_insensitive_duplicate() {
        let entries = vec![
            RawArchiveEntry::file("src/Lib.rs", 4, b"aaaa".to_vec()),
            RawArchiveEntry::file("src/lib.rs", 4, b"bbbb".to_vec()),
        ];
        let err = snapshot_from_entries("repo", entries, &limits()).unwrap_err();
        assert!(matches!(err, ArchiveError::DuplicateEntry { .. }));
    }

    #[test]
    fn rejects_post_normalization_duplicate() {
        let entries = vec![
            RawArchiveEntry::file("src/lib.rs", 4, b"aaaa".to_vec()),
            RawArchiveEntry::file("./src/lib.rs", 4, b"bbbb".to_vec()),
        ];
        let err = snapshot_from_entries("repo", entries, &limits()).unwrap_err();
        assert!(matches!(err, ArchiveError::DuplicateEntry { .. }));
    }

    #[test]
    fn enforces_per_entry_size_cap() {
        let small = ArchiveLimits {
            max_entry_size: 8,
            ..ArchiveLimits::default()
        };
        let entries = vec![RawArchiveEntry::file("big.bin", 9, vec![0u8; 9])];
        let err = snapshot_from_entries("repo", entries, &small).unwrap_err();
        assert!(matches!(err, ArchiveError::EntryTooLarge { .. }));
    }

    #[test]
    fn enforces_total_size_cap() {
        let small = ArchiveLimits {
            max_entry_size: 1024,
            max_total_size: 10,
            ..ArchiveLimits::default()
        };
        let entries = vec![
            RawArchiveEntry::file("a.bin", 6, vec![0u8; 6]),
            RawArchiveEntry::file("b.bin", 6, vec![0u8; 6]),
        ];
        let err = snapshot_from_entries("repo", entries, &small).unwrap_err();
        assert!(matches!(err, ArchiveError::TotalTooLarge { .. }));
    }

    #[test]
    fn enforces_entry_count_cap() {
        let small = ArchiveLimits {
            max_entries: 1,
            ..ArchiveLimits::default()
        };
        let entries = vec![
            RawArchiveEntry::file("a.rs", 1, b"a".to_vec()),
            RawArchiveEntry::file("b.rs", 1, b"b".to_vec()),
        ];
        let err = snapshot_from_entries("repo", entries, &small).unwrap_err();
        assert!(matches!(err, ArchiveError::TooManyEntries { .. }));
    }

    #[test]
    fn enforces_compression_ratio_guard() {
        let strict = ArchiveLimits {
            max_ratio: 10,
            ..ArchiveLimits::default()
        };
        // 1 compressed byte declared, 1000 inflated bytes -> ratio 1000 > 10.
        let entries = vec![RawArchiveEntry::file("bomb.bin", 1, vec![0u8; 1000])];
        let err = snapshot_from_entries("repo", entries, &strict).unwrap_err();
        assert!(matches!(err, ArchiveError::RatioExceeded { .. }));
    }

    #[test]
    fn zero_compressed_nonempty_is_rejected_as_ratio() {
        let entries = vec![RawArchiveEntry::file("x.bin", 0, vec![0u8; 4])];
        let err = snapshot_from_entries("repo", entries, &limits()).unwrap_err();
        assert!(matches!(err, ArchiveError::RatioExceeded { .. }));
    }

    #[test]
    fn empty_stored_file_is_admitted() -> Result<(), ArchiveError> {
        let entries = vec![RawArchiveEntry::file("empty.txt", 0, Vec::new())];
        let snap = snapshot_from_entries("repo", entries, &limits())?;
        assert_eq!(snap.len(), 1);
        assert!(
            snap.get("empty.txt")
                .map(VirtualFile::is_empty)
                .unwrap_or(false)
        );
        Ok(())
    }

    #[test]
    fn fails_closed_no_partial_snapshot() {
        // A valid entry followed by a hostile one: the whole build must fail.
        let entries = vec![
            RawArchiveEntry::file("ok.rs", 4, b"good".to_vec()),
            RawArchiveEntry::file("../evil.rs", 4, b"evil".to_vec()),
        ];
        let err = snapshot_from_entries("repo", entries, &limits()).unwrap_err();
        assert!(matches!(err, ArchiveError::Traversal { .. }));
    }

    #[test]
    fn root_is_normalized() -> Result<(), ArchiveError> {
        let entries = vec![RawArchiveEntry::file("a.rs", 1, b"a".to_vec())];
        let snap = snapshot_from_entries(".\\nested\\root", entries, &limits())?;
        assert_eq!(snap.root(), "nested/root");
        Ok(())
    }
}

#[cfg(all(test, feature = "archive-zip"))]
mod zip_tests {
    use super::*;
    use crate::VirtualFile;
    use std::io::{Cursor, Write};
    use zip::CompressionMethod;
    use zip::write::{SimpleFileOptions, ZipWriter};

    type ZipResult = zip::result::ZipResult<Vec<u8>>;

    fn stored() -> SimpleFileOptions {
        SimpleFileOptions::default().compression_method(CompressionMethod::Stored)
    }

    fn deflated() -> SimpleFileOptions {
        SimpleFileOptions::default().compression_method(CompressionMethod::Deflated)
    }

    /// Build an in-memory ZIP via the provided closure.
    fn build_zip(
        build: impl FnOnce(&mut ZipWriter<Cursor<Vec<u8>>>) -> zip::result::ZipResult<()>,
    ) -> ZipResult {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        build(&mut writer)?;
        let cursor = writer.finish()?;
        Ok(cursor.into_inner())
    }

    #[test]
    fn round_trips_stored_and_deflated() -> Result<(), Box<dyn std::error::Error>> {
        let bytes = build_zip(|w| {
            w.start_file("src/main.rs", stored())?;
            w.write_all(b"fn main() {}")?;
            w.start_file("a/deflated.txt", deflated())?;
            w.write_all(b"hello deflated world")?;
            Ok(())
        })?;

        let snap = snapshot_from_zip_bytes("repo", &bytes, &ArchiveLimits::default())?;
        let paths: Vec<&str> = snap.paths().collect();
        assert_eq!(paths, vec!["a/deflated.txt", "src/main.rs"]);
        assert_eq!(
            snap.get("src/main.rs").map(VirtualFile::bytes),
            Some(&b"fn main() {}"[..])
        );
        assert_eq!(
            snap.get("a/deflated.txt").map(VirtualFile::bytes),
            Some(&b"hello deflated world"[..])
        );
        Ok(())
    }

    #[test]
    fn directory_entries_contribute_no_file() -> Result<(), Box<dyn std::error::Error>> {
        let bytes = build_zip(|w| {
            w.add_directory("src/", stored())?;
            w.start_file("src/lib.rs", stored())?;
            w.write_all(b"pub fn x() {}")?;
            Ok(())
        })?;

        let snap = snapshot_from_zip_bytes("repo", &bytes, &ArchiveLimits::default())?;
        let paths: Vec<&str> = snap.paths().collect();
        assert_eq!(paths, vec!["src/lib.rs"]);
        Ok(())
    }

    #[test]
    fn empty_stored_file_is_admitted() -> Result<(), Box<dyn std::error::Error>> {
        let bytes = build_zip(|w| {
            w.start_file("empty.txt", stored())?;
            Ok(())
        })?;

        let snap = snapshot_from_zip_bytes("repo", &bytes, &ArchiveLimits::default())?;
        assert_eq!(snap.len(), 1);
        assert!(
            snap.get("empty.txt")
                .map(VirtualFile::is_empty)
                .unwrap_or(false)
        );
        Ok(())
    }

    #[test]
    fn rejects_symlink_entry() -> Result<(), Box<dyn std::error::Error>> {
        let bytes = build_zip(|w| {
            w.add_symlink("link", "src/main.rs", stored())?;
            Ok(())
        })?;

        let err = snapshot_from_zip_bytes("repo", &bytes, &ArchiveLimits::default()).unwrap_err();
        assert!(matches!(err, ArchiveError::NonRegularEntry { .. }));
        Ok(())
    }

    #[test]
    fn rejects_traversal_name() -> Result<(), Box<dyn std::error::Error>> {
        let bytes = build_zip(|w| {
            w.start_file("../evil.rs", stored())?;
            w.write_all(b"pwn")?;
            Ok(())
        })?;

        let err = snapshot_from_zip_bytes("repo", &bytes, &ArchiveLimits::default()).unwrap_err();
        assert!(matches!(err, ArchiveError::Traversal { .. }));
        Ok(())
    }

    #[test]
    fn enforces_per_entry_size_cap_via_bounded_read() -> Result<(), Box<dyn std::error::Error>> {
        let small = ArchiveLimits {
            max_entry_size: 8,
            ..ArchiveLimits::default()
        };
        let bytes = build_zip(|w| {
            w.start_file("big.bin", stored())?;
            w.write_all(&[0u8; 64])?;
            Ok(())
        })?;

        let err = snapshot_from_zip_bytes("repo", &bytes, &small).unwrap_err();
        assert!(matches!(err, ArchiveError::EntryTooLarge { .. }));
        Ok(())
    }

    #[test]
    fn enforces_compression_ratio_guard() -> Result<(), Box<dyn std::error::Error>> {
        let strict = ArchiveLimits {
            max_ratio: 10,
            ..ArchiveLimits::default()
        };
        // 100 KiB of zeros deflates to a tiny payload -> ratio far above 10x.
        let bytes = build_zip(|w| {
            w.start_file("bomb.bin", deflated())?;
            w.write_all(&[0u8; 100_000])?;
            Ok(())
        })?;

        let err = snapshot_from_zip_bytes("repo", &bytes, &strict).unwrap_err();
        assert!(matches!(err, ArchiveError::RatioExceeded { .. }));
        Ok(())
    }

    #[test]
    fn enforces_total_size_cap_during_decode() -> Result<(), Box<dyn std::error::Error>> {
        let small = ArchiveLimits {
            max_entry_size: 1024,
            max_total_size: 10,
            max_ratio: u64::MAX,
            ..ArchiveLimits::default()
        };
        let bytes = build_zip(|w| {
            w.start_file("a.bin", stored())?;
            w.write_all(&[0u8; 6])?;
            w.start_file("b.bin", stored())?;
            w.write_all(&[0u8; 6])?;
            Ok(())
        })?;

        let err = snapshot_from_zip_bytes("repo", &bytes, &small).unwrap_err();
        assert!(matches!(err, ArchiveError::TotalTooLarge { .. }));
        Ok(())
    }

    #[test]
    fn rejects_malformed_archive() {
        let garbage = b"this is definitely not a zip archive";
        let err = snapshot_from_zip_bytes("repo", garbage, &ArchiveLimits::default()).unwrap_err();
        assert!(matches!(err, ArchiveError::MalformedArchive { .. }));
    }

    #[test]
    fn fails_closed_on_first_hostile_entry() -> Result<(), Box<dyn std::error::Error>> {
        let bytes = build_zip(|w| {
            w.start_file("ok.rs", stored())?;
            w.write_all(b"good")?;
            w.start_file("nested/../../evil.rs", stored())?;
            w.write_all(b"evil")?;
            Ok(())
        })?;

        let err = snapshot_from_zip_bytes("repo", &bytes, &ArchiveLimits::default()).unwrap_err();
        assert!(matches!(err, ArchiveError::Traversal { .. }));
        Ok(())
    }

    #[test]
    fn classify_falls_back_without_unix_mode() {
        assert_eq!(classify_zip_entry(None, false), EntryKind::File);
        assert_eq!(classify_zip_entry(None, true), EntryKind::Directory);
        assert_eq!(classify_zip_entry(Some(0o100644), false), EntryKind::File);
        assert_eq!(
            classify_zip_entry(Some(0o040755), false),
            EntryKind::Directory
        );
        assert_eq!(classify_zip_entry(Some(0o120777), false), EntryKind::Other);
    }
}
