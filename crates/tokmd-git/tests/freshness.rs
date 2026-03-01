//! Freshness analysis tests for tokmd-git.
//!
//! Verifies that `collect_history()` data supports computing file staleness
//! metrics: last-modified timestamps, age relative to HEAD, and handling
//! of files absent from git history.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use tokmd_git::{GitCommit, collect_history, git_available, repo_root};

// ============================================================================
// Helpers
// ============================================================================

fn git_in(dir: &Path) -> Command {
    let mut cmd = Command::new("git");
    cmd.env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .current_dir(dir);
    cmd
}

struct TempGitRepo {
    path: PathBuf,
}

impl Drop for TempGitRepo {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.path).ok();
    }
}

fn make_repo(suffix: &str) -> Option<TempGitRepo> {
    if !git_available() {
        return None;
    }
    let id = format!(
        "{}-{:?}-{}-fresh-{}",
        std::process::id(),
        std::thread::current().id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0),
        suffix,
    );
    let dir = std::env::temp_dir().join(format!("tokmd-git-fresh-{}", id));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).ok();
    }
    std::fs::create_dir_all(&dir).ok()?;

    let ok = |o: std::process::Output| o.status.success();
    if !ok(git_in(&dir).args(["init"]).output().ok()?) {
        std::fs::remove_dir_all(&dir).ok();
        return None;
    }
    git_in(&dir)
        .args(["config", "user.email", "fresh@test.com"])
        .output()
        .ok()?;
    git_in(&dir)
        .args(["config", "user.name", "Freshness Tester"])
        .output()
        .ok()?;

    std::fs::write(dir.join("seed.txt"), "seed").ok()?;
    git_in(&dir).args(["add", "."]).output().ok()?;
    if !ok(git_in(&dir).args(["commit", "-m", "seed"]).output().ok()?) {
        std::fs::remove_dir_all(&dir).ok();
        return None;
    }
    Some(TempGitRepo { path: dir })
}

fn commit_file(dir: &Path, name: &str, content: &str, message: &str) {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, content).unwrap();
    git_in(dir).args(["add", name]).output().unwrap();
    git_in(dir)
        .args(["commit", "-m", message])
        .output()
        .unwrap();
}

/// Build a freshness map: file → most recent timestamp it was modified.
fn build_freshness_map(commits: &[GitCommit]) -> BTreeMap<String, i64> {
    let mut freshness: BTreeMap<String, i64> = BTreeMap::new();
    for c in commits {
        for f in &c.files {
            let entry = freshness.entry(f.clone()).or_insert(0);
            if c.timestamp > *entry {
                *entry = c.timestamp;
            }
        }
    }
    freshness
}

// ============================================================================
// Tests
// ============================================================================

/// Recently committed file has a later timestamp than older file.
#[test]
fn recent_file_has_later_timestamp() {
    let repo = make_repo("recent-vs-old").expect("repo");

    commit_file(&repo.path, "old.txt", "old", "add old");

    // Sleep briefly to ensure timestamps differ
    std::thread::sleep(std::time::Duration::from_secs(1));

    commit_file(&repo.path, "new.txt", "new", "add new");

    let root = repo_root(&repo.path).unwrap();
    let commits = collect_history(&root, None, None).unwrap();
    let freshness = build_freshness_map(&commits);

    let old_ts = freshness.get("old.txt").copied().unwrap_or(0);
    let new_ts = freshness.get("new.txt").copied().unwrap_or(0);

    assert!(old_ts > 0, "old.txt should have a positive timestamp");
    assert!(new_ts > 0, "new.txt should have a positive timestamp");
    assert!(
        new_ts >= old_ts,
        "new.txt ({new_ts}) should have timestamp >= old.txt ({old_ts})"
    );
}

/// File modified multiple times has the latest timestamp.
#[test]
fn modified_file_gets_latest_timestamp() {
    let repo = make_repo("multi-modify").expect("repo");

    commit_file(&repo.path, "evolving.txt", "v1", "first version");
    let root = repo_root(&repo.path).unwrap();
    let commits_before = collect_history(&root, None, None).unwrap();
    let ts_before = build_freshness_map(&commits_before)
        .get("evolving.txt")
        .copied()
        .unwrap_or(0);

    std::thread::sleep(std::time::Duration::from_secs(1));

    commit_file(&repo.path, "evolving.txt", "v2", "second version");
    let commits_after = collect_history(&root, None, None).unwrap();
    let ts_after = build_freshness_map(&commits_after)
        .get("evolving.txt")
        .copied()
        .unwrap_or(0);

    assert!(
        ts_after >= ts_before,
        "Freshness should update: {ts_after} >= {ts_before}"
    );
}

/// Files not in git history do not appear in the freshness map.
#[test]
fn untracked_files_not_in_freshness() {
    let repo = make_repo("untracked").expect("repo");

    // Create a file but don't commit it
    std::fs::write(repo.path.join("untracked.txt"), "not committed").unwrap();

    let root = repo_root(&repo.path).unwrap();
    let commits = collect_history(&root, None, None).unwrap();
    let freshness = build_freshness_map(&commits);

    assert!(
        !freshness.contains_key("untracked.txt"),
        "Untracked file should not appear in freshness map"
    );
}

/// Deleted file still appears in freshness map (it was in a commit).
#[test]
fn deleted_file_retains_freshness() {
    let repo = make_repo("deleted").expect("repo");

    commit_file(&repo.path, "doomed.txt", "bye", "add doomed");

    git_in(&repo.path)
        .args(["rm", "doomed.txt"])
        .output()
        .unwrap();
    git_in(&repo.path)
        .args(["commit", "-m", "remove doomed"])
        .output()
        .unwrap();

    let root = repo_root(&repo.path).unwrap();
    let commits = collect_history(&root, None, None).unwrap();
    let freshness = build_freshness_map(&commits);

    assert!(
        freshness.contains_key("doomed.txt"),
        "Deleted file should still appear in freshness (it was in commit history)"
    );
    assert!(
        freshness["doomed.txt"] > 0,
        "Freshness timestamp should be positive"
    );
}

/// All freshness timestamps are positive.
#[test]
fn all_freshness_timestamps_positive() {
    let repo = make_repo("all-positive").expect("repo");

    commit_file(&repo.path, "a.txt", "a", "add a");
    commit_file(&repo.path, "b.txt", "b", "add b");

    let root = repo_root(&repo.path).unwrap();
    let commits = collect_history(&root, None, None).unwrap();
    let freshness = build_freshness_map(&commits);

    for (file, ts) in &freshness {
        assert!(
            *ts > 0,
            "File {file} should have positive timestamp, got {ts}"
        );
    }
}

/// Empty commit list produces empty freshness map.
#[test]
fn empty_commits_produce_empty_freshness() {
    let commits: Vec<GitCommit> = Vec::new();
    let freshness = build_freshness_map(&commits);
    assert!(
        freshness.is_empty(),
        "Empty commits should yield empty freshness"
    );
}

/// Freshness map is deterministic across multiple invocations.
#[test]
fn freshness_is_deterministic() {
    let repo = make_repo("deterministic").expect("repo");

    commit_file(&repo.path, "det1.txt", "d1", "add d1");
    commit_file(&repo.path, "det2.txt", "d2", "add d2");

    let root = repo_root(&repo.path).unwrap();
    let commits = collect_history(&root, None, None).unwrap();

    let map1 = build_freshness_map(&commits);
    let map2 = build_freshness_map(&commits);
    assert_eq!(map1, map2, "Freshness map should be deterministic");
}

/// Staleness: the age of each file relative to the most recent commit.
#[test]
fn staleness_computation_is_non_negative() {
    let repo = make_repo("staleness").expect("repo");

    commit_file(&repo.path, "stale.txt", "s", "add stale");
    std::thread::sleep(std::time::Duration::from_secs(1));
    commit_file(&repo.path, "fresh.txt", "f", "add fresh");

    let root = repo_root(&repo.path).unwrap();
    let commits = collect_history(&root, None, None).unwrap();
    let freshness = build_freshness_map(&commits);

    // Find the most recent timestamp across all files
    let max_ts = freshness.values().copied().max().unwrap_or(0);

    for (file, ts) in &freshness {
        let staleness = max_ts - ts;
        assert!(
            staleness >= 0,
            "Staleness of {file} should be non-negative, got {staleness}"
        );
    }

    // fresh.txt should have zero or minimal staleness
    let fresh_staleness = max_ts - freshness.get("fresh.txt").copied().unwrap_or(0);
    let stale_staleness = max_ts - freshness.get("stale.txt").copied().unwrap_or(0);
    assert!(
        fresh_staleness <= stale_staleness,
        "fresh.txt should be less stale than stale.txt: {fresh_staleness} <= {stale_staleness}"
    );
}
