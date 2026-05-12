//! Git revision and base-reference resolution.

use std::path::Path;
use std::process::Stdio;

use crate::git_cmd;

/// Check whether a git revision resolves to a valid commit.
pub fn rev_exists(repo_root: &Path, rev: &str) -> bool {
    git_cmd()
        .arg("-C")
        .arg(repo_root)
        .args(["rev-parse", "--verify", "--quiet", "--end-of-options"])
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
        && env_base_ref_is_safe(&env_ref)
        && rev_exists(repo_root, &env_ref)
    {
        return Some(env_ref);
    }

    // GitHub Actions: origin/$GITHUB_BASE_REF
    if let Ok(gh_base) = std::env::var("GITHUB_BASE_REF")
        && env_base_ref_is_safe(&gh_base)
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

fn env_base_ref_is_safe(ref_name: &str) -> bool {
    !ref_name.is_empty()
        && !ref_name.starts_with('-')
        && !ref_name
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || c == '\\')
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    use crate::{git_available, git_cmd};

    fn test_git(dir: &Path) -> Command {
        let mut cmd = git_cmd();
        cmd.arg("-C").arg(dir);
        cmd
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
    fn rev_exists_treats_option_like_ref_as_missing() {
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

        assert!(!rev_exists(dir.path(), "--help"));
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

    #[test]
    fn env_base_ref_accepts_common_refs() {
        for ref_name in [
            "HEAD",
            "HEAD~1",
            "feature/foo",
            "release/v1.2.3",
            "dependabot/cargo/foo-1.2.3",
            "origin/main",
            "af6004c",
        ] {
            assert!(
                env_base_ref_is_safe(ref_name),
                "expected env base ref to be safe: {ref_name}"
            );
        }
    }

    #[test]
    fn env_base_ref_rejects_ambiguous_or_malformed_refs() {
        for ref_name in [
            "",
            "-bad",
            "--help",
            "feature foo",
            "main\nnext",
            "main\0next",
            r"feature\foo",
        ] {
            assert!(
                !env_base_ref_is_safe(ref_name),
                "expected env base ref to be rejected: {ref_name:?}"
            );
        }
    }
}
