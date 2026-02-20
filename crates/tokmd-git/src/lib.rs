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
pub use tokmd_types::CommitIntentKind;

/// Create a `Command` for git with process-environment isolation.
///
/// Strips `GIT_DIR` and `GIT_WORK_TREE` so that inherited environment
/// variables cannot override the explicit `-C` path used by all
/// functions in this crate.
fn git_cmd() -> Command {
    let mut cmd = Command::new("git");
    cmd.env_remove("GIT_DIR").env_remove("GIT_WORK_TREE");
    cmd
}

#[derive(Debug, Clone)]
pub struct GitCommit {
    pub timestamp: i64,
    pub author: String,
    pub hash: Option<String>,
    pub subject: String,
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
    git_cmd()
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn repo_root(path: &Path) -> Option<PathBuf> {
    let output = git_cmd()
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
    let mut child = git_cmd()
        .arg("-C")
        .arg(repo_root)
        .arg("log")
        .arg("--name-only")
        .arg("--pretty=format:%ct|%ae|%H|%s")
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
            let mut parts = line.splitn(4, '|');
            let ts = parts.next().unwrap_or("0").parse::<i64>().unwrap_or(0);
            let author = parts.next().unwrap_or("").to_string();
            let hash_str = parts.next().unwrap_or("").to_string();
            let subject = parts.next().unwrap_or("").to_string();
            let hash = if hash_str.is_empty() {
                None
            } else {
                Some(hash_str)
            };
            current = Some(GitCommit {
                timestamp: ts,
                author,
                hash,
                subject,
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

/// Get the set of added line numbers per file between two refs.
pub fn get_added_lines(
    repo_root: &Path,
    base: &str,
    head: &str,
    range_mode: GitRangeMode,
) -> Result<std::collections::BTreeMap<PathBuf, std::collections::BTreeSet<usize>>> {
    let range = range_mode.format(base, head);
    let output = git_cmd()
        .arg("-C")
        .arg(repo_root)
        .args(["diff", "--unified=0", &range])
        .output()
        .context("Failed to run git diff")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("git diff failed: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut result: std::collections::BTreeMap<PathBuf, std::collections::BTreeSet<usize>> =
        std::collections::BTreeMap::new();
    let mut current_file: Option<PathBuf> = None;

    for line in stdout.lines() {
        if let Some(file_path) = line.strip_prefix("+++ b/") {
            current_file = Some(PathBuf::from(file_path));
            continue;
        }

        if line.starts_with("@@") {
            let Some(file) = current_file.as_ref() else {
                continue;
            };

            // Hunk header: @@ -a,b +c,d @@
            // We care about +c,d
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
                continue;
            }

            let new_range = parts[2]; // +c,d
            let range_str = new_range.strip_prefix('+').unwrap_or(new_range);
            let range_parts: Vec<&str> = range_str.split(',').collect();

            let start: usize = range_parts[0].parse().unwrap_or(0);
            let count: usize = if range_parts.len() > 1 {
                range_parts[1].parse().unwrap_or(1)
            } else {
                1
            };

            if count > 0 && start > 0 {
                let set = result.entry(file.clone()).or_default();
                for i in 0..count {
                    set.insert(start + i);
                }
            }
        }
    }

    Ok(result)
}

/// Check whether a git revision resolves to a valid commit.
pub fn rev_exists(repo_root: &Path, rev: &str) -> bool {
    git_cmd()
        .arg("-C")
        .arg(repo_root)
        .args(["rev-parse", "--verify", "--quiet"])
        .arg(format!("{rev}^{{commit}}"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Resolve a base ref with a fallback chain for CI environments.
///
/// Fallback order:
/// 1. `requested` itself (fast path)
/// 2. `TOKMD_GIT_BASE_REF` env var
/// 3. `origin/{GITHUB_BASE_REF}` (GitHub Actions)
/// 4. `origin/HEAD` (remote default branch)
/// 5. `origin/main`, `main`, `origin/master`, `master`
///
/// Returns `None` if nothing resolves.
pub fn resolve_base_ref(repo_root: &Path, requested: &str) -> Option<String> {
    // Fast path: the requested ref exists
    if rev_exists(repo_root, requested) {
        return Some(requested.to_string());
    }

    // Only use fallback resolution for the CLI default (`main`).
    // Explicitly requested bases should fail fast if missing.
    if requested != "main" {
        return None;
    }

    // TOKMD_GIT_BASE_REF env override
    if let Ok(env_ref) = std::env::var("TOKMD_GIT_BASE_REF")
        && !env_ref.is_empty()
        && rev_exists(repo_root, &env_ref)
    {
        return Some(env_ref);
    }

    // GitHub Actions: origin/$GITHUB_BASE_REF
    if let Ok(gh_base) = std::env::var("GITHUB_BASE_REF")
        && !gh_base.is_empty()
    {
        let candidate = format!("origin/{gh_base}");
        if rev_exists(repo_root, &candidate) {
            return Some(candidate);
        }
    }

    // Remote default branch
    static FALLBACKS: &[&str] = &[
        "origin/HEAD",
        "origin/main",
        "main",
        "origin/master",
        "master",
    ];

    for candidate in FALLBACKS {
        if rev_exists(repo_root, candidate) {
            return Some((*candidate).to_string());
        }
    }

    None
}

// -----------------------
// Commit intent classification
// -----------------------

/// Classify a commit subject line into an intent kind.
///
/// Uses a two-stage pipeline:
/// 1. **Conventional Commits**: Parse `type(scope)!: description` prefix
/// 2. **Keyword heuristic**: Match known keywords in the subject
pub fn classify_intent(subject: &str) -> CommitIntentKind {
    let trimmed = subject.trim();
    if trimmed.is_empty() {
        return CommitIntentKind::Other;
    }

    // Check for revert pattern first
    if trimmed.starts_with("Revert \"") || trimmed.starts_with("revert:") {
        return CommitIntentKind::Revert;
    }

    // Try conventional commit parsing
    if let Some(kind) = parse_conventional_prefix(trimmed) {
        return kind;
    }

    // Fall back to keyword heuristic
    keyword_heuristic(trimmed)
}

/// Parse a conventional commit prefix like `feat(scope)!: description`.
fn parse_conventional_prefix(subject: &str) -> Option<CommitIntentKind> {
    let colon_pos = subject.find(':')?;
    let prefix = &subject[..colon_pos];

    // Strip optional (scope) and trailing !
    let prefix = if let Some(paren_pos) = prefix.find('(') {
        &prefix[..paren_pos]
    } else {
        prefix
    };
    let prefix = prefix.trim_end_matches('!');

    match prefix.to_ascii_lowercase().as_str() {
        "feat" | "feature" => Some(CommitIntentKind::Feat),
        "fix" | "bugfix" | "hotfix" => Some(CommitIntentKind::Fix),
        "refactor" => Some(CommitIntentKind::Refactor),
        "docs" | "doc" => Some(CommitIntentKind::Docs),
        "test" | "tests" => Some(CommitIntentKind::Test),
        "chore" => Some(CommitIntentKind::Chore),
        "ci" => Some(CommitIntentKind::Ci),
        "build" => Some(CommitIntentKind::Build),
        "perf" => Some(CommitIntentKind::Perf),
        "style" => Some(CommitIntentKind::Style),
        "revert" => Some(CommitIntentKind::Revert),
        _ => None,
    }
}

/// Keyword-based heuristic for commit intent classification.
fn keyword_heuristic(subject: &str) -> CommitIntentKind {
    let lower = subject.to_ascii_lowercase();

    // Ordered by priority: more specific matches first
    if contains_word(&lower, "revert") {
        CommitIntentKind::Revert
    } else if contains_word(&lower, "fix")
        || contains_word(&lower, "bug")
        || contains_word(&lower, "patch")
        || contains_word(&lower, "hotfix")
    {
        CommitIntentKind::Fix
    } else if contains_word(&lower, "feat")
        || contains_word(&lower, "feature")
        || lower.starts_with("add ")
        || lower.starts_with("implement ")
        || lower.starts_with("introduce ")
    {
        CommitIntentKind::Feat
    } else if contains_word(&lower, "refactor") || contains_word(&lower, "restructure") {
        CommitIntentKind::Refactor
    } else if contains_word(&lower, "doc") || contains_word(&lower, "readme") {
        CommitIntentKind::Docs
    } else if contains_word(&lower, "test") {
        CommitIntentKind::Test
    } else if contains_word(&lower, "perf")
        || contains_word(&lower, "performance")
        || contains_word(&lower, "optimize")
    {
        CommitIntentKind::Perf
    } else if contains_word(&lower, "style")
        || contains_word(&lower, "format")
        || contains_word(&lower, "lint")
    {
        CommitIntentKind::Style
    } else if contains_word(&lower, "ci") || contains_word(&lower, "pipeline") {
        CommitIntentKind::Ci
    } else if contains_word(&lower, "build") || contains_word(&lower, "deps") {
        CommitIntentKind::Build
    } else if contains_word(&lower, "chore") || contains_word(&lower, "cleanup") {
        CommitIntentKind::Chore
    } else {
        CommitIntentKind::Other
    }
}

/// Check if a word appears as a word boundary match in the subject.
fn contains_word(haystack: &str, word: &str) -> bool {
    for (idx, _) in haystack.match_indices(word) {
        let before_ok = idx == 0 || !haystack.as_bytes()[idx - 1].is_ascii_alphanumeric();
        let after_idx = idx + word.len();
        let after_ok =
            after_idx >= haystack.len() || !haystack.as_bytes()[after_idx].is_ascii_alphanumeric();
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_git(dir: &Path) -> Command {
        let mut cmd = git_cmd();
        cmd.arg("-C").arg(dir);
        cmd
    }

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
    fn rev_exists_finds_head_in_repo() {
        if !git_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();

        // Init repo and create a commit so HEAD resolves
        test_git(dir.path()).arg("init").output().unwrap();
        test_git(dir.path())
            .args(["config", "user.email", "test@test.com"])
            .output()
            .unwrap();
        test_git(dir.path())
            .args(["config", "user.name", "Test"])
            .output()
            .unwrap();
        std::fs::write(dir.path().join("f.txt"), "hello").unwrap();
        test_git(dir.path()).args(["add", "."]).output().unwrap();
        test_git(dir.path())
            .args(["commit", "-m", "init"])
            .output()
            .unwrap();

        assert!(rev_exists(dir.path(), "HEAD"));
        assert!(!rev_exists(dir.path(), "nonexistent-branch-abc123"));
    }

    #[test]
    fn resolve_base_ref_returns_requested_when_valid() {
        if !git_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();

        test_git(dir.path())
            .args(["init", "-b", "main"])
            .output()
            .unwrap();
        test_git(dir.path())
            .args(["config", "user.email", "test@test.com"])
            .output()
            .unwrap();
        test_git(dir.path())
            .args(["config", "user.name", "Test"])
            .output()
            .unwrap();
        std::fs::write(dir.path().join("f.txt"), "hello").unwrap();
        test_git(dir.path()).args(["add", "."]).output().unwrap();
        test_git(dir.path())
            .args(["commit", "-m", "init"])
            .output()
            .unwrap();

        assert_eq!(
            resolve_base_ref(dir.path(), "main"),
            Some("main".to_string())
        );
    }

    #[test]
    fn resolve_base_ref_returns_none_when_nothing_resolves() {
        if !git_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();

        // Init on "trunk" with no commits, no remotes
        test_git(dir.path())
            .args(["init", "-b", "trunk"])
            .output()
            .unwrap();

        // No commits exist, so even "trunk" won't resolve to a commit
        assert_eq!(resolve_base_ref(dir.path(), "nonexistent"), None);
    }
}
