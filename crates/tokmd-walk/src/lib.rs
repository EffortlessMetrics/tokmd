//! # tokmd-walk
//!
//! **Tier 2 (Utilities)**
//!
//! File listing and asset discovery utilities. Provides filesystem traversal
//! with gitignore support for analysis workflows.
//!
//! ## What belongs here
//! * Filesystem traversal respecting gitignore
//! * License candidate detection
//! * File size queries
//!
//! ## What does NOT belong here
//! * Content scanning (use tokmd-content)
//! * Git history analysis (use tokmd-git)
//! * File modification

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use ignore::WalkBuilder;

#[derive(Debug, Clone)]
pub struct LicenseCandidates {
    pub license_files: Vec<PathBuf>,
    pub metadata_files: Vec<PathBuf>,
}

pub fn list_files(root: &Path, max_files: Option<usize>) -> Result<Vec<PathBuf>> {
    // Early return for zero-file limit
    if max_files == Some(0) {
        return Ok(Vec::new());
    }

    if let Some(mut files) = git_ls_files(root)? {
        if let Some(limit) = max_files
            && files.len() > limit
        {
            files.truncate(limit);
        }
        return Ok(files);
    }

    let mut files: Vec<PathBuf> = Vec::new();
    let mut builder = WalkBuilder::new(root);
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
        let path = entry.path().to_path_buf();
        let rel = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
        files.push(rel);
        if let Some(limit) = max_files
            && files.len() >= limit
        {
            break;
        }
    }

    files.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
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

    license_files.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
    metadata_files.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));

    LicenseCandidates {
        license_files,
        metadata_files,
    }
}

fn git_ls_files(root: &Path) -> Result<Option<Vec<PathBuf>>> {
    let output = Command::new("git")
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

pub fn file_size(root: &Path, relative: &Path) -> Result<u64> {
    let path = root.join(relative);
    let meta =
        std::fs::metadata(&path).with_context(|| format!("Failed to stat {}", path.display()))?;
    Ok(meta.len())
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
}
