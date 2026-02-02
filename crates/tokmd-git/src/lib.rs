//! # tokmd-git
//!
//! **Tier 2 (Utilities)**
//!
//! Streaming git log adapter for tokmd analysis. Collects commit history
//! without loading the entire history into memory.
//!
//! ## What belongs here
//! * Git history collection
//! * Commit parsing (timestamp, author, affected files)
//! * Streaming interface
//!
//! ## What does NOT belong here
//! * Analysis computation (use tokmd-analysis)
//! * Git history modification
//! * Complex git operations (use git2 crate directly if needed)

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct GitCommit {
    pub timestamp: i64,
    pub author: String,
    pub files: Vec<String>,
}

/// Git range syntax for comparing commits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GitRangeMode {
    /// Two-dot syntax: `A..B` - commits in B but not A.
    #[default]
    TwoDot,
    /// Three-dot syntax: `A...B` - symmetric difference from merge-base.
    ThreeDot,
}

impl GitRangeMode {
    /// Format the range string for git commands.
    pub fn format(&self, base: &str, head: &str) -> String {
        match self {
            GitRangeMode::TwoDot => format!("{}..{}", base, head),
            GitRangeMode::ThreeDot => format!("{}...{}", base, head),
        }
    }
}

pub fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn repo_root(path: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if root.is_empty() {
        None
    } else {
        Some(PathBuf::from(root))
    }
}

pub fn collect_history(
    repo_root: &Path,
    max_commits: Option<usize>,
    max_commit_files: Option<usize>,
) -> Result<Vec<GitCommit>> {
    let mut child = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("log")
        .arg("--name-only")
        .arg("--pretty=format:%ct|%ae")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to spawn git log")?;

    let stdout = child.stdout.take().context("Missing git log stdout")?;
    let reader = BufReader::new(stdout);

    let mut commits: Vec<GitCommit> = Vec::new();
    let mut current: Option<GitCommit> = None;

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            if let Some(commit) = current.take() {
                commits.push(commit);
                if max_commits.is_some_and(|limit| commits.len() >= limit) {
                    break;
                }
            }
            continue;
        }

        if current.is_none() {
            let mut parts = line.splitn(2, '|');
            let ts = parts.next().unwrap_or("0").parse::<i64>().unwrap_or(0);
            let author = parts.next().unwrap_or("").to_string();
            current = Some(GitCommit {
                timestamp: ts,
                author,
                files: Vec::new(),
            });
            continue;
        }

        if let Some(commit) = current.as_mut()
            && max_commit_files
                .map(|limit| commit.files.len() < limit)
                .unwrap_or(true)
        {
            commit.files.push(line.trim().to_string());
        }
    }

    if let Some(commit) = current.take() {
        commits.push(commit);
    }

    let status = child.wait()?;
    if !status.success() {
        return Err(anyhow::anyhow!("git log failed"));
    }

    Ok(commits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_range_two_dot_format() {
        assert_eq!(GitRangeMode::TwoDot.format("main", "HEAD"), "main..HEAD");
    }

    #[test]
    fn git_range_three_dot_format() {
        assert_eq!(GitRangeMode::ThreeDot.format("main", "HEAD"), "main...HEAD");
    }

    #[test]
    fn git_range_default_is_two_dot() {
        assert_eq!(GitRangeMode::default(), GitRangeMode::TwoDot);
    }
}
