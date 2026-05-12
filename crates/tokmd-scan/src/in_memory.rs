//! In-memory scan input materialization.
//!
//! This module owns the browser/native contract for logical in-memory file
//! paths before they are materialized into a temporary tokei scan root.

use anyhow::Result;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use tokei::Languages;

use tokmd_settings::ScanOptions;

/// A single logical file supplied from memory rather than the host filesystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InMemoryFile {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
}

impl InMemoryFile {
    #[must_use]
    pub fn new(path: impl Into<PathBuf>, bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            path: path.into(),
            bytes: bytes.into(),
        }
    }
}

/// A scan result that keeps its backing temp root alive for downstream row modeling.
///
/// Keep this wrapper alive while any downstream code needs to read file metadata from
/// the scanned paths. `tokmd-model` uses the underlying paths to compute byte and token
/// counts after the scan phase.
///
/// When converting these scan results into `FileRow`s, pass [`Self::strip_prefix`] as the
/// `strip_prefix` argument so receipts keep the logical in-memory paths rather than the
/// temp backing root.
#[derive(Debug)]
pub struct MaterializedScan {
    languages: Languages,
    logical_paths: Vec<PathBuf>,
    root: tempfile::TempDir,
}

impl MaterializedScan {
    #[must_use]
    pub fn languages(&self) -> &Languages {
        &self.languages
    }

    #[must_use]
    pub fn logical_paths(&self) -> &[PathBuf] {
        &self.logical_paths
    }

    #[must_use]
    pub fn strip_prefix(&self) -> &Path {
        self.root.path()
    }
}

/// Normalize ordered in-memory inputs into deterministic logical paths.
///
/// This rejects empty, absolute, escaping, and case-only-colliding paths so
/// native and browser callers see the same contract.
pub fn normalize_in_memory_paths(inputs: &[InMemoryFile]) -> Result<Vec<PathBuf>> {
    normalize_logical_paths(inputs, true)
}

pub fn scan_in_memory(inputs: &[InMemoryFile], args: &ScanOptions) -> Result<MaterializedScan> {
    let root = tempfile::tempdir()?;
    let logical_paths = normalize_in_memory_paths(inputs)?;

    for (logical_path, input) in logical_paths.iter().zip(inputs) {
        let full_path = root.path().join(logical_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(full_path, &input.bytes)?;
    }

    let scan_root = vec![root.path().to_path_buf()];
    let languages = crate::scan(&scan_root, args)?;

    Ok(MaterializedScan {
        languages,
        logical_paths,
        root,
    })
}

fn normalize_logical_paths(
    inputs: &[InMemoryFile],
    case_insensitive: bool,
) -> Result<Vec<PathBuf>> {
    let mut seen = BTreeSet::new();
    let mut normalized = Vec::with_capacity(inputs.len());

    for input in inputs {
        let logical_path = normalize_logical_path(&input.path)?;
        if !seen.insert(logical_path_key(&logical_path, case_insensitive)) {
            anyhow::bail!("Duplicate in-memory path: {}", logical_path.display());
        }
        normalized.push(logical_path);
    }

    Ok(normalized)
}

fn logical_path_key(path: &Path, case_insensitive: bool) -> String {
    let rendered = path.to_string_lossy();
    if case_insensitive {
        rendered.to_lowercase()
    } else {
        rendered.into_owned()
    }
}

fn normalize_logical_path(path: &Path) -> Result<PathBuf> {
    if path.as_os_str().is_empty() {
        anyhow::bail!("In-memory path must not be empty");
    }

    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(segment) => normalized.push(segment),
            Component::CurDir => {}
            Component::ParentDir => {
                anyhow::bail!(
                    "In-memory path must not contain parent traversal: {}",
                    path.display()
                );
            }
            Component::RootDir | Component::Prefix(_) => {
                anyhow::bail!("In-memory path must be relative: {}", path.display());
            }
        }
    }

    if normalized.as_os_str().is_empty() {
        anyhow::bail!("In-memory path must resolve to a file: {}", path.display());
    }

    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_logical_path_strips_dot_segments() -> Result<()> {
        let normalized = normalize_logical_path(Path::new("./src/./lib.rs"))?;
        assert_eq!(normalized, PathBuf::from("src/lib.rs"));
        Ok(())
    }

    #[test]
    fn normalize_logical_path_rejects_absolute_paths() {
        let err = normalize_logical_path(Path::new("/src/lib.rs")).unwrap_err();
        assert!(err.to_string().contains("must be relative"));
    }

    #[test]
    fn normalize_logical_path_rejects_parent_traversal() {
        let err = normalize_logical_path(Path::new("../src/lib.rs")).unwrap_err();
        assert!(err.to_string().contains("parent traversal"));
    }

    #[test]
    fn normalize_logical_paths_rejects_duplicate_after_normalization() {
        let inputs = vec![
            InMemoryFile::new("./src/lib.rs", "fn main() {}\n"),
            InMemoryFile::new("src/lib.rs", "fn main() {}\n"),
        ];

        let err = normalize_logical_paths(&inputs, false).unwrap_err();
        assert!(err.to_string().contains("Duplicate in-memory path"));
    }

    #[test]
    fn normalize_logical_paths_rejects_case_only_collision_on_case_insensitive_fs() {
        let inputs = vec![
            InMemoryFile::new("src/lib.rs", "fn main() {}\n"),
            InMemoryFile::new("SRC/LIB.rs", "fn main() {}\n"),
        ];

        let err = normalize_logical_paths(&inputs, true).unwrap_err();
        assert!(err.to_string().contains("Duplicate in-memory path"));
    }
}
