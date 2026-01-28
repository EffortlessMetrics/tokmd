//! # tokmd-walk
//!
//! File listing helpers for tokmd analysis.

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
    if let Some(mut files) = git_ls_files(root)? {
        if let Some(limit) = max_files {
            if files.len() > limit {
                files.truncate(limit);
            }
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
        if let Some(limit) = max_files {
            if files.len() >= limit {
                break;
            }
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
        if name.starts_with("license")
            || name.starts_with("copying")
            || name.starts_with("notice")
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
    let meta = std::fs::metadata(&path)
        .with_context(|| format!("Failed to stat {}", path.display()))?;
    Ok(meta.len())
}
