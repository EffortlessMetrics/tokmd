//! File walking and repository asset discovery helpers.
//!
//! This module provides deterministic filesystem traversal with gitignore
//! support for scan and analysis workflows.
//!
//! ## What belongs here
//! * Filesystem traversal respecting gitignore
//! * License candidate detection
//! * File size queries
//!
//! ## What does NOT belong here
//! * Content scanning (use tokmd-analysis content helpers)
//! * Git history analysis (use tokmd-git)
//! * File modification

use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use ignore::WalkBuilder;
use tokmd_io_port::MemFs;

use crate::path::{BoundedPath, PathViolation, ValidatedRoot, normalize_bounded_relative_path};

#[derive(Debug, Clone)]
pub struct LicenseCandidates {
    pub license_files: Vec<PathBuf>,
    pub metadata_files: Vec<PathBuf>,
}

fn git_cmd() -> Command {
    let mut cmd = Command::new("git");
    cmd.env_remove("GIT_DIR").env_remove("GIT_WORK_TREE");
    cmd
}

pub fn list_files(root: &Path, max_files: Option<usize>) -> Result<Vec<PathBuf>> {
    // Early return for zero-file limit
    if max_files == Some(0) {
        return Ok(Vec::new());
    }

    let root = ValidatedRoot::new(root)?;

    if let Some(mut files) = git_ls_files(root.input())? {
        files = files
            .into_iter()
            .map(|path| bound_git_relative_path(&root, &path))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        if let Some(limit) = max_files
            && files.len() > limit
        {
            files.truncate(limit);
        }
        return Ok(files);
    }

    let mut files: Vec<PathBuf> = Vec::new();
    let mut builder = WalkBuilder::new(root.input());
    builder.hidden(false);
    builder.git_ignore(true);
    builder.git_exclude(true);
    builder.git_global(true);
    builder.follow_links(false);

    for entry in builder.build() {
        let entry = entry?;
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        let rel = BoundedPath::existing_child(&root, entry.path())?
            .relative()
            .to_path_buf();
        files.push(rel);
        if let Some(limit) = max_files
            && files.len() >= limit
        {
            break;
        }
    }

    files.sort();
    Ok(files)
}

/// List files from an in-memory filesystem backend.
///
/// Returned paths are relative to `root` and sorted for deterministic output.
pub fn list_files_from_memfs(
    fs: &MemFs,
    root: &Path,
    max_files: Option<usize>,
) -> Result<Vec<PathBuf>> {
    if max_files == Some(0) {
        return Ok(Vec::new());
    }

    let normalized_root = normalize_memfs_root(root)?;
    let mut files: Vec<PathBuf> = fs
        .file_paths()
        .filter_map(|path| memfs_relative_path(path, &normalized_root))
        .collect();

    files.sort();

    if let Some(limit) = max_files
        && files.len() > limit
    {
        files.truncate(limit);
    }

    Ok(files)
}

pub fn license_candidates(files: &[PathBuf]) -> LicenseCandidates {
    let mut license_files = Vec::new();
    let mut metadata_files = Vec::new();

    for rel in files {
        let name = rel
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        if name == "cargo.toml" || name == "package.json" || name == "pyproject.toml" {
            metadata_files.push(rel.clone());
            continue;
        }
        if name.starts_with("license") || name.starts_with("copying") || name.starts_with("notice")
        {
            license_files.push(rel.clone());
        }
    }

    license_files.sort();
    metadata_files.sort();

    LicenseCandidates {
        license_files,
        metadata_files,
    }
}

fn git_ls_files(root: &Path) -> Result<Option<Vec<PathBuf>>> {
    let output = git_cmd()
        .arg("-C")
        .arg(root)
        .arg("ls-files")
        .arg("-z")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    let output = match output {
        Ok(out) => out,
        Err(_) => return Ok(None),
    };
    if !output.status.success() {
        return Ok(None);
    }

    let mut files = Vec::new();
    let bytes = output.stdout;
    for part in bytes.split(|b| *b == 0) {
        if part.is_empty() {
            continue;
        }
        let s = String::from_utf8_lossy(part).to_string();
        files.push(PathBuf::from(s));
    }

    if files.is_empty() {
        return Ok(None);
    }

    Ok(Some(files))
}

fn bound_git_relative_path(root: &ValidatedRoot, path: &Path) -> Result<PathBuf, PathViolation> {
    Ok(BoundedPath::existing_relative(root, path)?
        .relative()
        .to_path_buf())
}

pub fn file_size(root: &Path, relative: &Path) -> Result<u64> {
    let root = ValidatedRoot::new(root)?;
    let path = BoundedPath::existing_relative(&root, relative)?;
    let meta = std::fs::metadata(path.canonical())
        .with_context(|| format!("Failed to stat {}", path.canonical().display()))?;
    Ok(meta.len())
}

/// Query a file size from an in-memory filesystem backend.
pub fn file_size_from_memfs(fs: &MemFs, root: &Path, relative: &Path) -> Result<u64> {
    let normalized_root = normalize_memfs_root(root)?;
    let normalized_relative = normalize_bounded_relative_path(relative)?;
    let path = if normalized_root.as_os_str().is_empty() {
        normalized_relative
    } else {
        normalized_root.join(normalized_relative)
    };
    fs.file_size(&path)
        .with_context(|| format!("Failed to stat {}", path.display()))
}

fn normalize_memfs_root(path: &Path) -> Result<PathBuf> {
    let mut normalized = PathBuf::new();
    if path.as_os_str().is_empty() {
        return Ok(normalized);
    }
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => normalized.push(part),
            Component::ParentDir => {
                return Err(PathViolation::ParentTraversal(path.to_path_buf()).into());
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(PathViolation::Absolute(path.to_path_buf()).into());
            }
        }
    }
    Ok(normalized)
}

fn memfs_relative_path(path: &Path, root: &Path) -> Option<PathBuf> {
    if root.as_os_str().is_empty() {
        return Some(path.to_path_buf());
    }
    path.strip_prefix(root).ok().map(Path::to_path_buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // ---- license_candidates tests ----

    #[test]
    fn test_license_candidates_detects_license_files() {
        let files = vec![
            PathBuf::from("LICENSE"),
            PathBuf::from("LICENSE.md"),
            PathBuf::from("LICENSE-MIT"),
            PathBuf::from("COPYING"),
            PathBuf::from("NOTICE"),
            PathBuf::from("src/main.rs"),
        ];
        let result = license_candidates(&files);
        assert_eq!(result.license_files.len(), 5);
        assert!(result.metadata_files.is_empty());
    }

    #[test]
    fn test_license_candidates_detects_metadata_files() {
        let files = vec![
            PathBuf::from("Cargo.toml"),
            PathBuf::from("package.json"),
            PathBuf::from("pyproject.toml"),
            PathBuf::from("src/lib.rs"),
        ];
        let result = license_candidates(&files);
        assert!(result.license_files.is_empty());
        assert_eq!(result.metadata_files.len(), 3);
    }

    #[test]
    fn test_license_candidates_mixed() {
        let files = vec![
            PathBuf::from("LICENSE"),
            PathBuf::from("Cargo.toml"),
            PathBuf::from("src/main.rs"),
        ];
        let result = license_candidates(&files);
        assert_eq!(result.license_files.len(), 1);
        assert_eq!(result.metadata_files.len(), 1);
    }

    #[test]
    fn test_license_candidates_empty_input() {
        let result = license_candidates(&[]);
        assert!(result.license_files.is_empty());
        assert!(result.metadata_files.is_empty());
    }

    #[test]
    fn test_license_candidates_case_insensitive() {
        let files = vec![PathBuf::from("license"), PathBuf::from("License.txt")];
        let result = license_candidates(&files);
        assert_eq!(result.license_files.len(), 2);
    }

    #[test]
    fn test_license_candidates_sorted_output() {
        let files = vec![
            PathBuf::from("z/Cargo.toml"),
            PathBuf::from("a/Cargo.toml"),
            PathBuf::from("z/LICENSE"),
            PathBuf::from("a/LICENSE"),
        ];
        let result = license_candidates(&files);
        assert_eq!(result.license_files[0], PathBuf::from("a/LICENSE"));
        assert_eq!(result.license_files[1], PathBuf::from("z/LICENSE"));
        assert_eq!(result.metadata_files[0], PathBuf::from("a/Cargo.toml"));
        assert_eq!(result.metadata_files[1], PathBuf::from("z/Cargo.toml"));
    }

    // ---- file_size tests ----

    #[test]
    fn test_file_size_returns_correct_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let content = "hello world";
        fs::write(dir.path().join("test.txt"), content).unwrap();
        let size = file_size(dir.path(), Path::new("test.txt")).unwrap();
        assert_eq!(size, content.len() as u64);
    }

    #[test]
    fn test_file_size_missing_file_errors() {
        let dir = tempfile::tempdir().unwrap();
        let result = file_size(dir.path(), Path::new("nonexistent.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_file_size_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("empty.txt"), "").unwrap();
        let size = file_size(dir.path(), Path::new("empty.txt")).unwrap();
        assert_eq!(size, 0);
    }

    // ---- list_files tests ----

    #[test]
    fn test_list_files_max_zero_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "content").unwrap();
        let files = list_files(dir.path(), Some(0)).unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_list_files_respects_max_limit() {
        let dir = tempfile::tempdir().unwrap();
        // Create .git dir so git_ls_files returns Some
        fs::create_dir_all(dir.path().join(".git")).unwrap();
        for i in 0..10 {
            fs::write(dir.path().join(format!("file{i}.txt")), "x").unwrap();
        }
        let files = list_files(dir.path(), Some(3)).unwrap();
        assert!(files.len() <= 3);
    }

    #[test]
    fn test_list_files_deterministic_sort() {
        let dir = tempfile::tempdir().unwrap();
        // Create .git dir so git_ls_files returns Some
        fs::create_dir_all(dir.path().join(".git")).unwrap();
        fs::create_dir_all(dir.path().join("foo")).unwrap();
        fs::write(dir.path().join("foo/bar"), "content").unwrap();
        fs::write(dir.path().join("foo/bar.rs"), "content").unwrap();
        fs::write(dir.path().join("foo.rs"), "content").unwrap();

        let files = list_files(dir.path(), None).unwrap();
        // The resulting paths are relative to root
        // Expected sort: foo.rs, foo/bar, foo/bar.rs
        // rather than lossy string sort which puts foo/bar before foo.rs
        let expected = vec![
            PathBuf::from("foo/bar"),
            PathBuf::from("foo/bar.rs"),
            PathBuf::from("foo.rs"),
        ];
        // Only checking that our added test files are sorted identically
        // Note: git_ls_files relies on git, so we filter out .git
        let actual: Vec<PathBuf> = files
            .into_iter()
            .filter(|p| {
                let s = p.to_string_lossy();
                s.starts_with("foo")
            })
            .collect();
        // They should already be sorted correctly, but if they aren't, the test will fail
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_bound_git_relative_path_accepts_existing_relative_file() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/lib.rs"), "pub fn lib() {}\n").unwrap();
        let root = ValidatedRoot::new(dir.path()).unwrap();

        let bounded = bound_git_relative_path(&root, Path::new("./src/lib.rs")).unwrap();

        assert_eq!(bounded, PathBuf::from("src/lib.rs"));
    }

    #[test]
    fn test_bound_git_relative_path_rejects_parent_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let root = ValidatedRoot::new(dir.path()).unwrap();

        let err = bound_git_relative_path(&root, Path::new("../secret.txt")).unwrap_err();

        assert!(err.to_string().contains("parent traversal"));
    }

    #[test]
    fn test_bound_git_relative_path_rejects_absolute_path() {
        let dir = tempfile::tempdir().unwrap();
        let root = ValidatedRoot::new(dir.path()).unwrap();

        let err = bound_git_relative_path(&root, Path::new("/secret.txt")).unwrap_err();

        assert!(err.to_string().contains("must be relative"));
    }
}
