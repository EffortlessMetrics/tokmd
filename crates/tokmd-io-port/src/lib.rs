//! # tokmd-io-port
//!
//! **Tier 0 (Contract)**
//!
//! I/O port traits for host-abstracted file access.
//! Enables WASM targets by replacing real fs with in-memory backends.
//!
//! ## What belongs here
//! * The `ReadFs` trait and its implementations
//! * `HostFs` – delegates to `std::fs`
//! * `MemFs` – in-memory store for tests and WASM
//!
//! ## What does NOT belong here
//! * Directory traversal / walking (use tokmd-scan::walk)
//! * Content scanning (use tokmd-scan)

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[cfg(feature = "archive")]
pub mod archive;

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// Read-only filesystem port.
pub trait ReadFs {
    type Error: std::error::Error;

    fn read_to_string(&self, path: &Path) -> Result<String, Self::Error>;
    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, Self::Error>;
    fn exists(&self, path: &Path) -> bool;
    fn is_dir(&self, path: &Path) -> bool;
    fn is_file(&self, path: &Path) -> bool;
}

// ---------------------------------------------------------------------------
// HostFs – default std::fs implementation
// ---------------------------------------------------------------------------

/// Default host filesystem implementation.
#[derive(Debug, Clone, Copy)]
pub struct HostFs;

impl ReadFs for HostFs {
    type Error = std::io::Error;

    fn read_to_string(&self, path: &Path) -> Result<String, Self::Error> {
        std::fs::read_to_string(path)
    }

    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, Self::Error> {
        std::fs::read(path)
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }
}

// ---------------------------------------------------------------------------
// MemFs – in-memory filesystem for testing and WASM
// ---------------------------------------------------------------------------

/// In-memory filesystem for testing and WASM.
///
/// Files are stored as byte vectors keyed by path. Directories are inferred
/// from the set of stored file paths – any path that is a proper prefix of a
/// stored file is considered a directory.
#[derive(Debug, Clone, Default)]
pub struct MemFs {
    files: BTreeMap<PathBuf, Vec<u8>>,
}

/// Error type returned by [`MemFs`] operations.
#[derive(Debug)]
pub struct MemFsError {
    kind: MemFsErrorKind,
    path: PathBuf,
}

#[derive(Debug)]
enum MemFsErrorKind {
    NotFound,
    InvalidUtf8,
}

impl std::fmt::Display for MemFsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            MemFsErrorKind::NotFound => write!(f, "not found: {}", self.path.display()),
            MemFsErrorKind::InvalidUtf8 => {
                write!(f, "invalid UTF-8 in: {}", self.path.display())
            }
        }
    }
}

impl std::error::Error for MemFsError {}

impl MemFs {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a UTF-8 file.
    pub fn add_file(&mut self, path: impl Into<PathBuf>, contents: impl Into<String>) {
        self.files.insert(path.into(), contents.into().into_bytes());
    }

    /// Insert a binary file.
    pub fn add_bytes(&mut self, path: impl Into<PathBuf>, bytes: impl Into<Vec<u8>>) {
        self.files.insert(path.into(), bytes.into());
    }

    /// Iterate deterministic file paths stored in the virtual filesystem.
    pub fn file_paths(&self) -> impl Iterator<Item = &Path> {
        self.files.keys().map(PathBuf::as_path)
    }

    /// Return the size of a stored file in bytes.
    pub fn file_size(&self, path: &Path) -> Result<u64, MemFsError> {
        self.files
            .get(path)
            .map(|bytes| bytes.len() as u64)
            .ok_or_else(|| self.not_found(path))
    }

    fn not_found(&self, path: &Path) -> MemFsError {
        MemFsError {
            kind: MemFsErrorKind::NotFound,
            path: path.to_path_buf(),
        }
    }
}

impl ReadFs for MemFs {
    type Error = MemFsError;

    fn read_to_string(&self, path: &Path) -> Result<String, Self::Error> {
        let bytes = self.files.get(path).ok_or_else(|| self.not_found(path))?;
        String::from_utf8(bytes.clone()).map_err(|_| MemFsError {
            kind: MemFsErrorKind::InvalidUtf8,
            path: path.to_path_buf(),
        })
    }

    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, Self::Error> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| self.not_found(path))
    }

    fn exists(&self, path: &Path) -> bool {
        self.is_file(path) || self.is_dir(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        // A path is a directory if any stored file has it as a proper prefix.
        self.files.keys().any(|k| k.starts_with(path) && k != path)
    }

    fn is_file(&self, path: &Path) -> bool {
        self.files.contains_key(path)
    }
}

// ---------------------------------------------------------------------------
// RepoSnapshot / VirtualFile (experimental portability seam)
// ---------------------------------------------------------------------------
//
// These types implement the minimal slice of the repo-snapshot portability
// seam described in `docs/specs/repo-snapshot.md`. They capture a
// provider-agnostic, deterministic view of in-scope files read through any
// [`ReadFs`] backend (`HostFs`, `MemFs`, or a future host-supplied provider),
// so scan/aggregation logic can later operate on the snapshot instead of
// touching `std::fs` directly.
//
// EXPERIMENTAL / UNSTABLE: this is the first landing of the seam. The spec
// keeps it intentionally narrow (eager byte capture, no directory walking, no
// archive ingestion, no serialized format). The surface may change until a
// real consumer promotes it; do not treat it as a stable support promise.

/// Normalize a path to the forward-slash rule shared with `tokmd-model`.
///
/// `tokmd-model::normalize_path` is the canonical rule, but it lives in a
/// higher tier; this tier-0 crate mirrors the core of that rule (backslash to
/// forward slash, strip a leading `./`, trim leading `/`) without taking a
/// dependency on `tokmd-model`.
fn normalize_snapshot_path(path: &Path) -> String {
    let raw = path.to_string_lossy();
    let owned;
    let mut slice: &str = if raw.contains('\\') {
        owned = raw.replace('\\', "/");
        &owned
    } else {
        &raw
    };

    if let Some(stripped) = slice.strip_prefix("./") {
        slice = stripped;
    }
    slice = slice.trim_start_matches('/');
    if let Some(stripped) = slice.strip_prefix("./") {
        slice = stripped;
    }
    slice.trim_start_matches('/').to_string()
}

/// A single provider-agnostic file entry: a normalized forward-slash path plus
/// its eagerly captured bytes.
///
/// EXPERIMENTAL: part of the repo-snapshot seam (see module note).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualFile {
    path: String,
    bytes: Vec<u8>,
}

impl VirtualFile {
    /// The normalized forward-slash path of this entry.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The captured file contents.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// The byte length of the captured contents.
    pub fn len(&self) -> u64 {
        self.bytes.len() as u64
    }

    /// Whether the captured contents are empty.
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

/// Error raised while building a [`RepoSnapshot`].
///
/// EXPERIMENTAL: part of the repo-snapshot seam (see module note).
#[derive(Debug)]
pub enum SnapshotError<E> {
    /// Reading an in-scope path through the provider failed.
    Read {
        /// The normalized path that failed to read.
        path: String,
        /// The provider-specific read error.
        source: E,
    },
}

impl<E: std::fmt::Display> std::fmt::Display for SnapshotError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SnapshotError::Read { path, source } => {
                write!(f, "failed to read snapshot entry '{path}': {source}")
            }
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for SnapshotError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SnapshotError::Read { source, .. } => Some(source),
        }
    }
}

/// A deterministic, captured set of [`VirtualFile`] entries rooted at a logical
/// repository root, built from any [`ReadFs`] provider.
///
/// Entries are keyed by their normalized path in a sorted map, so enumeration
/// order is stable and independent of insertion order.
///
/// EXPERIMENTAL: part of the repo-snapshot seam (see module note).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RepoSnapshot {
    root: String,
    files: BTreeMap<String, VirtualFile>,
}

impl RepoSnapshot {
    /// Start building a snapshot rooted at `root`, reading entries through `fs`.
    pub fn builder<F: ReadFs>(fs: &F, root: impl AsRef<Path>) -> RepoSnapshotBuilder<'_, F> {
        RepoSnapshotBuilder {
            fs,
            root: normalize_snapshot_path(root.as_ref()),
            files: BTreeMap::new(),
        }
    }

    /// The normalized logical repository root.
    pub fn root(&self) -> &str {
        &self.root
    }

    /// The number of captured entries.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Whether the snapshot captured no entries.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Whether a normalized path is present in the snapshot.
    pub fn contains(&self, path: impl AsRef<Path>) -> bool {
        self.files
            .contains_key(&normalize_snapshot_path(path.as_ref()))
    }

    /// Look up a captured entry by path.
    pub fn get(&self, path: impl AsRef<Path>) -> Option<&VirtualFile> {
        self.files.get(&normalize_snapshot_path(path.as_ref()))
    }

    /// Iterate captured entries in deterministic (sorted-path) order.
    pub fn files(&self) -> impl Iterator<Item = &VirtualFile> {
        self.files.values()
    }

    /// Iterate captured paths in deterministic (sorted) order.
    pub fn paths(&self) -> impl Iterator<Item = &str> {
        self.files.keys().map(String::as_str)
    }
}

/// Builder that reads in-scope paths through a [`ReadFs`] provider and captures
/// them into a [`RepoSnapshot`].
///
/// EXPERIMENTAL: part of the repo-snapshot seam (see module note).
pub struct RepoSnapshotBuilder<'fs, F: ReadFs> {
    fs: &'fs F,
    root: String,
    files: BTreeMap<String, VirtualFile>,
}

impl<F: ReadFs> RepoSnapshotBuilder<'_, F> {
    /// Read one in-scope path through the provider and capture it.
    ///
    /// The path is normalized to the forward-slash rule before capture, so the
    /// same logical file produces the same key regardless of the host
    /// separator. Re-adding an already-captured path overwrites it.
    pub fn add_path(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<&mut Self, SnapshotError<F::Error>> {
        let path = path.as_ref();
        let normalized = normalize_snapshot_path(path);
        let bytes = self
            .fs
            .read_bytes(path)
            .map_err(|source| SnapshotError::Read {
                path: normalized.clone(),
                source,
            })?;
        self.files.insert(
            normalized.clone(),
            VirtualFile {
                path: normalized,
                bytes,
            },
        );
        Ok(self)
    }

    /// Read every path in `paths`, failing closed on the first read error.
    pub fn add_paths<P: AsRef<Path>>(
        &mut self,
        paths: impl IntoIterator<Item = P>,
    ) -> Result<&mut Self, SnapshotError<F::Error>> {
        for path in paths {
            self.add_path(path)?;
        }
        Ok(self)
    }

    /// Finish building, yielding a deterministic snapshot.
    pub fn build(self) -> RepoSnapshot {
        RepoSnapshot {
            root: self.root,
            files: self.files,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- HostFs tests ----

    #[test]
    fn host_fs_read_to_string() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("hello.txt");
        std::fs::write(&file, "hello world").unwrap();

        let fs = HostFs;
        assert_eq!(fs.read_to_string(&file).unwrap(), "hello world");
    }

    #[test]
    fn host_fs_read_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("data.bin");
        std::fs::write(&file, b"\x00\x01\x02").unwrap();

        let fs = HostFs;
        assert_eq!(fs.read_bytes(&file).unwrap(), vec![0, 1, 2]);
    }

    #[test]
    fn host_fs_exists() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("exists.txt");
        std::fs::write(&file, "").unwrap();

        let fs = HostFs;
        assert!(fs.exists(&file));
        assert!(!fs.exists(&dir.path().join("nope.txt")));
    }

    #[test]
    fn host_fs_is_dir() {
        let dir = tempfile::tempdir().unwrap();
        let fs = HostFs;
        assert!(fs.is_dir(dir.path()));
        assert!(!fs.is_dir(&dir.path().join("nope")));
    }

    #[test]
    fn host_fs_is_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("f.txt");
        std::fs::write(&file, "x").unwrap();

        let fs = HostFs;
        assert!(fs.is_file(&file));
        assert!(!fs.is_file(dir.path()));
    }

    #[test]
    fn host_fs_read_missing_file_errors() {
        let fs = HostFs;
        let result = fs.read_to_string(Path::new("/definitely/not/here.txt"));
        assert!(result.is_err());
    }

    // ---- MemFs tests ----

    #[test]
    fn mem_fs_read_to_string() {
        let mut fs = MemFs::new();
        fs.add_file(PathBuf::from("a.txt"), "contents");
        assert_eq!(fs.read_to_string(Path::new("a.txt")).unwrap(), "contents");
    }

    #[test]
    fn mem_fs_read_bytes() {
        let mut fs = MemFs::new();
        fs.add_bytes(PathBuf::from("b.bin"), vec![0xDE, 0xAD]);
        assert_eq!(fs.read_bytes(Path::new("b.bin")).unwrap(), vec![0xDE, 0xAD]);
    }

    #[test]
    fn mem_fs_not_found() {
        let fs = MemFs::new();
        let err = fs.read_to_string(Path::new("missing.txt")).unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn mem_fs_invalid_utf8() {
        let mut fs = MemFs::new();
        fs.add_bytes(PathBuf::from("bad.txt"), vec![0xFF, 0xFE]);
        let err = fs.read_to_string(Path::new("bad.txt")).unwrap_err();
        assert!(err.to_string().contains("invalid UTF-8"));
    }

    #[test]
    fn mem_fs_exists() {
        let mut fs = MemFs::new();
        fs.add_file(PathBuf::from("src/lib.rs"), "fn main() {}");
        assert!(fs.exists(Path::new("src/lib.rs")));
        assert!(fs.exists(Path::new("src"))); // directory
        assert!(!fs.exists(Path::new("nope")));
    }

    #[test]
    fn mem_fs_is_dir() {
        let mut fs = MemFs::new();
        fs.add_file(PathBuf::from("src/lib.rs"), "");
        assert!(fs.is_dir(Path::new("src")));
        assert!(!fs.is_dir(Path::new("src/lib.rs"))); // file, not dir
        assert!(!fs.is_dir(Path::new("other")));
    }

    #[test]
    fn mem_fs_is_file() {
        let mut fs = MemFs::new();
        fs.add_file(PathBuf::from("src/lib.rs"), "");
        assert!(fs.is_file(Path::new("src/lib.rs")));
        assert!(!fs.is_file(Path::new("src")));
    }

    #[test]
    fn mem_fs_default_is_empty() {
        let fs = MemFs::default();
        assert!(!fs.exists(Path::new("anything")));
    }

    #[test]
    fn mem_fs_file_paths_are_sorted() {
        let mut fs = MemFs::new();
        fs.add_file(PathBuf::from("z/file.txt"), "z");
        fs.add_file(PathBuf::from("a/file.txt"), "a");
        fs.add_file(PathBuf::from("m/file.txt"), "m");

        let paths: Vec<_> = fs
            .file_paths()
            .map(|path| path.to_string_lossy().into_owned())
            .collect();

        assert_eq!(paths, vec!["a/file.txt", "m/file.txt", "z/file.txt"]);
    }

    #[test]
    fn mem_fs_file_size_reads_inserted_length() {
        let mut fs = MemFs::new();
        fs.add_bytes(PathBuf::from("blob.bin"), vec![1, 2, 3, 4, 5]);

        assert_eq!(fs.file_size(Path::new("blob.bin")).unwrap(), 5);
    }

    #[test]
    fn mem_fs_file_size_missing_errors() {
        let fs = MemFs::new();
        let err = fs.file_size(Path::new("missing.bin")).unwrap_err();
        assert!(err.to_string().contains("not found"));
    }
}
