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

    let commits = parse_git_log(reader, max_commits, max_commit_files)?;

    let status = child.wait()?;
    if !status.success() {
        return Err(anyhow::anyhow!("git log failed"));
    }

    Ok(commits)
}

/// Parse git log output from a reader.
///
/// This function is exposed to allow benchmarking and testing without running actual git commands.
pub fn parse_git_log<R: BufRead>(
    mut reader: R,
    max_commits: Option<usize>,
    max_commit_files: Option<usize>,
) -> Result<Vec<GitCommit>> {
    let mut commits: Vec<GitCommit> = Vec::new();
    let mut current: Option<GitCommit> = None;
    let mut line = String::new();

    while reader.read_line(&mut line)? > 0 {
        {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                if let Some(commit) = current.take() {
                    commits.push(commit);
                    if max_commits.is_some_and(|limit| commits.len() >= limit) {
                        break;
                    }
                }
                line.clear();
                continue;
            }

            if let Some(commit) = current.as_mut() {
                if max_commit_files.is_none_or(|limit| commit.files.len() < limit) {
                    commit.files.push(trimmed.to_string());
                }
            } else {
                let mut parts = trimmed.splitn(2, '|');
                let ts = parts.next().unwrap_or("0").parse::<i64>().unwrap_or(0);
                let author = parts.next().unwrap_or("").to_string();
                current = Some(GitCommit {
                    timestamp: ts,
                    author,
                    files: Vec::new(),
                });
            }
        }
        line.clear();
    }

    if let Some(commit) = current.take() {
        commits.push(commit);
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

    #[test]
    fn test_parse_git_log() {
        let input = "1600000000|author@example.com\nfile1.rs\nfile2.rs\n\n1600000001|author2@example.com\nfile3.rs\n";
        let reader = std::io::Cursor::new(input);
        let commits = parse_git_log(reader, None, None).unwrap();

        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].timestamp, 1600000000);
        assert_eq!(commits[0].author, "author@example.com");
        assert_eq!(commits[0].files.len(), 2);
        assert_eq!(commits[0].files[0], "file1.rs");

        assert_eq!(commits[1].timestamp, 1600000001);
        assert_eq!(commits[1].author, "author2@example.com");
        assert_eq!(commits[1].files.len(), 1);
        assert_eq!(commits[1].files[0], "file3.rs");
    }
}
