//! Hotspot analysis tests for tokmd-git.
//!
//! Verifies that `collect_history()` data can identify frequently changed
//! files (hotspots) with deterministic ordering.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use tokmd_git::{collect_history, git_available, repo_root};

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
        "{}-{:?}-{}-hotspot-{}",
        std::process::id(),
        std::thread::current().id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0),
        suffix,
    );
    let dir = std::env::temp_dir().join(format!("tokmd-git-hotspot-{}", id));
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
        .args(["config", "user.email", "hot@test.com"])
        .output()
        .ok()?;
    git_in(&dir)
        .args(["config", "user.name", "Hotspot Tester"])
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

/// Build a hotspot report: file → change count, sorted descending by count then by name.
fn build_hotspot_report(commits: &[tokmd_git::GitCommit]) -> Vec<(String, usize)> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for c in commits {
        for f in &c.files {
            *counts.entry(f.clone()).or_default() += 1;
        }
    }
    let mut report: Vec<(String, usize)> = counts.into_iter().collect();
    report.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    report
}

// ============================================================================
// Tests
// ============================================================================

/// File changed most often appears first in hotspot report.
#[test]
fn hotspot_identifies_most_changed_file() {
    let repo = make_repo("most-changed").expect("repo");

    // hot.txt changed 5 times, cold.txt changed once
    for i in 0..5 {
        commit_file(
            &repo.path,
            "hot.txt",
            &format!("v{i}"),
            &format!("edit hot {i}"),
        );
    }
    commit_file(&repo.path, "cold.txt", "cold", "add cold");

    let root = repo_root(&repo.path).unwrap();
    let commits = collect_history(&root, None, None).unwrap();
    let report = build_hotspot_report(&commits);

    assert!(!report.is_empty(), "Hotspot report should not be empty");

    let (top_file, top_count) = &report[0];
    assert_eq!(top_file, "hot.txt", "hot.txt should be the top hotspot");
    assert_eq!(*top_count, 5, "hot.txt should have 5 changes");

    let cold_entry = report.iter().find(|(f, _)| f == "cold.txt");
    assert!(cold_entry.is_some(), "cold.txt should appear in report");
    assert_eq!(cold_entry.unwrap().1, 1, "cold.txt changed once");
}

/// Ordering is deterministic: descending by change count, then ascending by name.
#[test]
fn hotspot_ordering_is_deterministic() {
    let repo = make_repo("ordering").expect("repo");

    // Create files with known change counts
    // b.txt: 3 changes, a.txt: 3 changes, c.txt: 1 change
    for i in 0..3 {
        commit_file(
            &repo.path,
            "b.txt",
            &format!("b-v{i}"),
            &format!("edit b {i}"),
        );
    }
    for i in 0..3 {
        commit_file(
            &repo.path,
            "a.txt",
            &format!("a-v{i}"),
            &format!("edit a {i}"),
        );
    }
    commit_file(&repo.path, "c.txt", "c", "add c");

    let root = repo_root(&repo.path).unwrap();
    let commits = collect_history(&root, None, None).unwrap();

    // Run twice to verify determinism
    let report1 = build_hotspot_report(&commits);
    let report2 = build_hotspot_report(&commits);
    assert_eq!(report1, report2, "Hotspot report should be deterministic");

    // Filter out seed.txt for cleaner assertions
    let filtered: Vec<_> = report1.iter().filter(|(f, _)| f != "seed.txt").collect();

    // a.txt and b.txt both have 3 changes; a.txt should come first (alpha sort)
    assert_eq!(filtered[0].0, "a.txt");
    assert_eq!(filtered[0].1, 3);
    assert_eq!(filtered[1].0, "b.txt");
    assert_eq!(filtered[1].1, 3);
    assert_eq!(filtered[2].0, "c.txt");
    assert_eq!(filtered[2].1, 1);
}

/// Single-commit repo has all files with count 1.
#[test]
fn hotspot_single_commit_all_count_one() {
    let repo = make_repo("single-count").expect("repo");

    let root = repo_root(&repo.path).unwrap();
    let commits = collect_history(&root, None, None).unwrap();
    let report = build_hotspot_report(&commits);

    for (file, count) in &report {
        assert_eq!(
            *count, 1,
            "File {file} in single-commit repo should have count 1"
        );
    }
}

/// Hotspot report from empty history is empty.
#[test]
fn hotspot_empty_history_produces_empty_report() {
    let commits: Vec<tokmd_git::GitCommit> = Vec::new();
    let report = build_hotspot_report(&commits);
    assert!(report.is_empty(), "Empty commits should yield empty report");
}

/// Files in subdirectories are tracked with full relative paths.
#[test]
fn hotspot_tracks_nested_paths() {
    let repo = make_repo("nested-paths").expect("repo");

    commit_file(&repo.path, "src/lib.rs", "fn main() {}", "add lib");
    commit_file(
        &repo.path,
        "src/lib.rs",
        "fn main() { println!() }",
        "edit lib",
    );
    commit_file(&repo.path, "tests/test.rs", "#[test] fn t() {}", "add test");

    let root = repo_root(&repo.path).unwrap();
    let commits = collect_history(&root, None, None).unwrap();
    let report = build_hotspot_report(&commits);

    let lib_count = report
        .iter()
        .find(|(f, _)| f == "src/lib.rs")
        .map(|(_, c)| *c)
        .unwrap_or(0);
    let test_count = report
        .iter()
        .find(|(f, _)| f == "tests/test.rs")
        .map(|(_, c)| *c)
        .unwrap_or(0);

    assert_eq!(lib_count, 2, "src/lib.rs should have 2 changes");
    assert_eq!(test_count, 1, "tests/test.rs should have 1 change");
}

/// Multiple files changed in same commit each get their own count.
#[test]
fn hotspot_multi_file_commit_counts_each_file() {
    let repo = make_repo("multi-file-count").expect("repo");

    // Single commit touching 3 files
    std::fs::write(repo.path.join("m1.txt"), "m1").unwrap();
    std::fs::write(repo.path.join("m2.txt"), "m2").unwrap();
    std::fs::write(repo.path.join("m3.txt"), "m3").unwrap();
    git_in(&repo.path).args(["add", "."]).output().unwrap();
    git_in(&repo.path)
        .args(["commit", "-m", "add three files"])
        .output()
        .unwrap();

    let root = repo_root(&repo.path).unwrap();
    let commits = collect_history(&root, None, None).unwrap();
    let report = build_hotspot_report(&commits);

    for name in &["m1.txt", "m2.txt", "m3.txt"] {
        let entry = report.iter().find(|(f, _)| f == name);
        assert!(entry.is_some(), "{name} should appear in hotspot report");
        assert_eq!(
            entry.unwrap().1,
            1,
            "{name} should have count 1 from multi-file commit"
        );
    }
}
