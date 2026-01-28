//! # tokmd-git
//!
//! Streaming git log adapter for tokmd analysis.

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
                if let Some(limit) = max_commits
                    && commits.len() >= limit
                {
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
