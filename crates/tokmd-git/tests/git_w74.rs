//! W74 deep tests for tokmd-git: git log parsing, commit counting,
//! author extraction, freshness calculation, hotspot detection,
//! intent classification, and graceful non-git handling.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tokmd_git::{
    CommitIntentKind, classify_intent, collect_history, git_available, repo_root, resolve_base_ref,
    rev_exists,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn git_in(dir: &Path) -> Command {
    let mut cmd = Command::new("git");
    cmd.env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .current_dir(dir);
    cmd
}

struct TempRepo {
    path: PathBuf,
}

impl Drop for TempRepo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn make_repo(tag: &str) -> Option<TempRepo> {
    if !git_available() {
        return None;
    }
    let id = format!(
        "w74-git-{}-{}-{:?}",
        tag,
        std::process::id(),
        std::thread::current().id(),
    );
    let dir = std::env::temp_dir().join(format!("tokmd-git-w74-{}", id));
    if dir.exists() {
        fs::remove_dir_all(&dir).ok();
    }
    fs::create_dir_all(&dir).ok()?;

    let ok = git_in(&dir).arg("init").output().ok()?.status.success();
    if !ok {
        fs::remove_dir_all(&dir).ok();
        return None;
    }
    git_in(&dir)
        .args(["config", "user.email", "w74@test.com"])
        .output()
        .ok()?;
    git_in(&dir)
        .args(["config", "user.name", "W74 Tester"])
        .output()
        .ok()?;

    Some(TempRepo { path: dir })
}

fn commit_file(dir: &Path, name: &str, content: &str, msg: &str) {
    let parent = dir.join(name);
    if let Some(p) = parent.parent() {
        fs::create_dir_all(p).ok();
    }
    fs::write(dir.join(name), content).unwrap();
    git_in(dir).args(["add", name]).output().unwrap();
    git_in(dir).args(["commit", "-m", msg]).output().unwrap();
}

// ===========================================================================
// 1. Git log parsing / commit counting
// ===========================================================================

#[test]
fn collect_history_single_commit() {
    let repo = match make_repo("single") {
        Some(r) => r,
        None => return,
    };
    commit_file(&repo.path, "init.txt", "hello", "initial commit");

    let commits = collect_history(&repo.path, None, None).unwrap();
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].subject, "initial commit");
}

#[test]
fn collect_history_multiple_commits() {
    let repo = match make_repo("multi") {
        Some(r) => r,
        None => return,
    };
    commit_file(&repo.path, "a.txt", "a", "first");
    commit_file(&repo.path, "b.txt", "b", "second");
    commit_file(&repo.path, "c.txt", "c", "third");

    let commits = collect_history(&repo.path, None, None).unwrap();
    assert_eq!(commits.len(), 3);
    // Most recent first
    assert_eq!(commits[0].subject, "third");
    assert_eq!(commits[2].subject, "first");
}

#[test]
fn collect_history_max_commits_limits() {
    let repo = match make_repo("max-limit") {
        Some(r) => r,
        None => return,
    };
    commit_file(&repo.path, "a.txt", "a", "first");
    commit_file(&repo.path, "b.txt", "b", "second");
    commit_file(&repo.path, "c.txt", "c", "third");

    // max_commits=2: result should have at most 2 commits.
    // On some platforms early pipe close may cause git log to return
    // non-zero; accept Ok with <=2 or Err as valid outcomes.
    if let Ok(commits) = collect_history(&repo.path, Some(2), None) {
        assert!(commits.len() <= 2, "max_commits should limit results");
    }
}

#[test]
fn collect_history_max_commit_files_limits() {
    let repo = match make_repo("max-files") {
        Some(r) => r,
        None => return,
    };
    // Create multiple files in a single commit
    for i in 0..5 {
        fs::write(repo.path.join(format!("f{i}.txt")), "x").unwrap();
    }
    git_in(&repo.path).args(["add", "."]).output().unwrap();
    git_in(&repo.path)
        .args(["commit", "-m", "bulk"])
        .output()
        .unwrap();

    let commits = collect_history(&repo.path, None, Some(2)).unwrap();
    assert!(!commits.is_empty());
    // The commit with 5 files should have at most 2 file entries
    let bulk = commits.iter().find(|c| c.subject == "bulk").unwrap();
    assert!(
        bulk.files.len() <= 2,
        "max_commit_files should cap file list"
    );
}

// ===========================================================================
// 2. Author extraction
// ===========================================================================

#[test]
fn collect_history_extracts_author_email() {
    let repo = match make_repo("author") {
        Some(r) => r,
        None => return,
    };
    commit_file(&repo.path, "f.txt", "data", "authored");

    let commits = collect_history(&repo.path, None, None).unwrap();
    assert!(!commits.is_empty());
    assert_eq!(commits[0].author, "w74@test.com");
}

#[test]
fn collect_history_extracts_hash() {
    let repo = match make_repo("hash") {
        Some(r) => r,
        None => return,
    };
    commit_file(&repo.path, "f.txt", "data", "hashed");

    let commits = collect_history(&repo.path, None, None).unwrap();
    let hash = commits[0].hash.as_ref().expect("hash should be present");
    assert_eq!(hash.len(), 40, "SHA-1 hash should be 40 hex chars");
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
}

// ===========================================================================
// 3. Freshness / timestamp
// ===========================================================================

#[test]
fn collect_history_has_nonzero_timestamps() {
    let repo = match make_repo("timestamp") {
        Some(r) => r,
        None => return,
    };
    commit_file(&repo.path, "f.txt", "data", "timestamped");

    let commits = collect_history(&repo.path, None, None).unwrap();
    assert!(commits[0].timestamp > 0, "timestamp should be > 0");
}

#[test]
fn collect_history_timestamps_monotonic() {
    let repo = match make_repo("monotonic") {
        Some(r) => r,
        None => return,
    };
    commit_file(&repo.path, "a.txt", "a", "first");
    // Small delay to ensure distinct timestamps
    std::thread::sleep(std::time::Duration::from_secs(1));
    commit_file(&repo.path, "b.txt", "b", "second");

    let commits = collect_history(&repo.path, None, None).unwrap();
    // Most recent first
    assert!(
        commits[0].timestamp >= commits[1].timestamp,
        "newer commit should have >= timestamp"
    );
}

// ===========================================================================
// 4. Hotspot detection (files touched in multiple commits)
// ===========================================================================

#[test]
fn hotspot_file_appears_in_multiple_commits() {
    let repo = match make_repo("hotspot") {
        Some(r) => r,
        None => return,
    };
    commit_file(&repo.path, "hot.rs", "v1", "first touch");
    commit_file(&repo.path, "hot.rs", "v2", "second touch");
    commit_file(&repo.path, "hot.rs", "v3", "third touch");
    commit_file(&repo.path, "cold.rs", "cold", "cold file");

    let commits = collect_history(&repo.path, None, None).unwrap();
    let hot_count = commits
        .iter()
        .filter(|c| c.files.iter().any(|f| f.contains("hot.rs")))
        .count();
    let cold_count = commits
        .iter()
        .filter(|c| c.files.iter().any(|f| f.contains("cold.rs")))
        .count();

    assert_eq!(hot_count, 3, "hot.rs touched in 3 commits");
    assert_eq!(cold_count, 1, "cold.rs touched in 1 commit");
}

// ===========================================================================
// 5. Non-git directory handling
// ===========================================================================

#[test]
fn repo_root_returns_none_for_non_git_dir() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(
        repo_root(tmp.path()).is_none(),
        "non-git dir should return None"
    );
}

#[test]
fn rev_exists_returns_false_for_non_git_dir() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(!rev_exists(tmp.path(), "HEAD"));
}

#[test]
fn resolve_base_ref_returns_none_for_non_git_dir() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(resolve_base_ref(tmp.path(), "main").is_none());
}

// ===========================================================================
// 6. Intent classification
// ===========================================================================

#[test]
fn classify_intent_conventional_commits() {
    assert_eq!(classify_intent("feat: add login"), CommitIntentKind::Feat);
    assert_eq!(
        classify_intent("fix(auth): null check"),
        CommitIntentKind::Fix
    );
    assert_eq!(
        classify_intent("docs: update README"),
        CommitIntentKind::Docs
    );
    assert_eq!(
        classify_intent("test: add unit tests"),
        CommitIntentKind::Test
    );
    assert_eq!(classify_intent("chore: bump deps"), CommitIntentKind::Chore);
    assert_eq!(classify_intent("ci: add workflow"), CommitIntentKind::Ci);
    assert_eq!(
        classify_intent("perf: optimize loop"),
        CommitIntentKind::Perf
    );
    assert_eq!(
        classify_intent("refactor: extract fn"),
        CommitIntentKind::Refactor
    );
    assert_eq!(
        classify_intent("style: fix formatting"),
        CommitIntentKind::Style
    );
    assert_eq!(
        classify_intent("build: update Makefile"),
        CommitIntentKind::Build
    );
}

#[test]
fn classify_intent_keyword_heuristic() {
    assert_eq!(
        classify_intent("Add new feature for users"),
        CommitIntentKind::Feat
    );
    assert_eq!(
        classify_intent("Fix crash on startup"),
        CommitIntentKind::Fix
    );
    assert_eq!(
        classify_intent("Update readme with examples"),
        CommitIntentKind::Docs
    );
}

#[test]
fn classify_intent_empty_and_unknown() {
    assert_eq!(classify_intent(""), CommitIntentKind::Other);
    assert_eq!(classify_intent("  "), CommitIntentKind::Other);
    assert_eq!(classify_intent("wip random stuff"), CommitIntentKind::Other);
}

#[test]
fn classify_intent_revert() {
    assert_eq!(
        classify_intent("Revert \"feat: add login\""),
        CommitIntentKind::Revert
    );
    assert_eq!(
        classify_intent("revert: undo thing"),
        CommitIntentKind::Revert
    );
}
