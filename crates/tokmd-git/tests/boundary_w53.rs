//! Git module boundary tests.
//!
//! Verifies that tokmd-git functions handle non-git directories, empty
//! histories, and edge cases gracefully without panicking.

use std::path::{Path, PathBuf};
use tempfile::TempDir;
use tokmd_git::{classify_intent, collect_history, git_available, repo_root, rev_exists};
use tokmd_types::CommitIntentKind;

// ── helpers ──────────────────────────────────────────────────────────

fn git_cmd(dir: &Path, args: &[&str]) -> bool {
    std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn make_git_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    git_cmd(dir.path(), &["init", "-b", "main"]);
    git_cmd(dir.path(), &["config", "user.email", "test@test.com"]);
    git_cmd(dir.path(), &["config", "user.name", "Test"]);
    dir
}

fn add_commit(dir: &Path, filename: &str, content: &str, msg: &str) {
    std::fs::write(dir.join(filename), content).unwrap();
    git_cmd(dir, &["add", filename]);
    git_cmd(dir, &["commit", "-m", msg]);
}

// ── git_available ────────────────────────────────────────────────────

#[test]
fn git_available_returns_bool_without_panic() {
    // Should not panic regardless of environment
    let _available = git_available();
}

// ── repo_root on non-git directories ─────────────────────────────────

#[test]
fn repo_root_returns_none_for_non_git_dir() {
    let dir = TempDir::new().unwrap();
    assert!(
        repo_root(dir.path()).is_none(),
        "non-git directory should return None"
    );
}

#[test]
fn repo_root_returns_none_for_nonexistent_path() {
    let bogus = PathBuf::from("__does_not_exist_w53__");
    assert!(repo_root(&bogus).is_none());
}

#[test]
fn repo_root_returns_some_for_valid_git_repo() {
    if !git_available() {
        return;
    }
    let dir = make_git_repo();
    assert!(repo_root(dir.path()).is_some());
}

// ── collect_history edge cases ───────────────────────────────────────

#[test]
fn collect_history_empty_repo_returns_empty_or_error() {
    if !git_available() {
        return;
    }
    let dir = make_git_repo();
    // No commits yet — git log should be empty or error
    if let Ok(commits) = collect_history(dir.path(), Some(100), Some(300)) {
        assert!(commits.is_empty(), "empty repo should have no commits");
    }
}

#[test]
fn collect_history_single_commit() {
    if !git_available() {
        return;
    }
    let dir = make_git_repo();
    add_commit(dir.path(), "hello.txt", "hello", "feat: initial");
    let commits = collect_history(dir.path(), Some(100), Some(300)).unwrap();
    assert_eq!(commits.len(), 1);
    assert!(commits[0].subject.contains("initial"));
}

#[test]
fn collect_history_returns_all_commits() {
    if !git_available() {
        return;
    }
    let dir = make_git_repo();
    for i in 0..3 {
        add_commit(
            dir.path(),
            &format!("file{i}.txt"),
            &format!("content {i}"),
            &format!("commit {i}"),
        );
    }
    let commits = collect_history(dir.path(), None, Some(300)).unwrap();
    assert_eq!(commits.len(), 3, "should return all 3 commits");
}

#[test]
fn collect_history_non_git_dir_returns_error() {
    let dir = TempDir::new().unwrap();
    let result = collect_history(dir.path(), Some(100), Some(300));
    assert!(result.is_err(), "non-git dir should error");
}

// ── rev_exists ───────────────────────────────────────────────────────

#[test]
fn rev_exists_returns_false_for_bogus_rev() {
    if !git_available() {
        return;
    }
    let dir = make_git_repo();
    add_commit(dir.path(), "a.txt", "a", "init");
    assert!(!rev_exists(dir.path(), "nonexistent_branch_w53"));
}

#[test]
fn rev_exists_returns_true_for_head() {
    if !git_available() {
        return;
    }
    let dir = make_git_repo();
    add_commit(dir.path(), "a.txt", "a", "init");
    assert!(rev_exists(dir.path(), "HEAD"));
}

#[test]
fn rev_exists_returns_false_for_non_git_dir() {
    let dir = TempDir::new().unwrap();
    assert!(!rev_exists(dir.path(), "HEAD"));
}

// ── classify_intent ──────────────────────────────────────────────────

#[test]
fn classify_intent_conventional_feat() {
    assert_eq!(classify_intent("feat: add button"), CommitIntentKind::Feat);
}

#[test]
fn classify_intent_conventional_fix() {
    assert_eq!(classify_intent("fix: null pointer"), CommitIntentKind::Fix);
}

#[test]
fn classify_intent_unknown_falls_back_to_other() {
    assert_eq!(classify_intent("random message"), CommitIntentKind::Other);
}

#[test]
fn classify_intent_empty_string() {
    // Must not panic on empty subject
    let kind = classify_intent("");
    assert_eq!(kind, CommitIntentKind::Other);
}

#[test]
fn classify_intent_docs() {
    assert_eq!(
        classify_intent("docs: update README"),
        CommitIntentKind::Docs
    );
}
