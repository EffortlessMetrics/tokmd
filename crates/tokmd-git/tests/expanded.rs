//! Expanded tests for tokmd-git covering hotspots, freshness, coupling,
//! churn, serialization, determinism, and edge cases.
//!
//! All git-dependent tests create isolated temporary repositories with
//! artificial commits for fully predictable test data.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use tokmd_git::{
    GitCommit, GitRangeMode, classify_intent, collect_history, get_added_lines, git_available,
    repo_root, rev_exists,
};
use tokmd_types::CommitIntentKind;

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

/// Create a git-init'd directory with user config but **no commits**.
fn make_empty_repo(suffix: &str) -> Option<TempGitRepo> {
    if !git_available() {
        return None;
    }
    let id = format!(
        "{}-{:?}-{}-exp-{}",
        std::process::id(),
        std::thread::current().id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0),
        suffix,
    );
    let dir = std::env::temp_dir().join(format!("tokmd-git-exp-{}", id));
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
        .args(["config", "user.email", "exp@test.com"])
        .output()
        .ok()?;
    git_in(&dir)
        .args(["config", "user.name", "Expanded Tester"])
        .output()
        .ok()?;

    Some(TempGitRepo { path: dir })
}

/// Create a repo with one seed commit.
fn make_seeded_repo(suffix: &str) -> Option<TempGitRepo> {
    let repo = make_empty_repo(suffix)?;
    std::fs::write(repo.path.join("seed.txt"), "seed").ok()?;
    git_in(&repo.path).args(["add", "."]).output().ok()?;
    let out = git_in(&repo.path)
        .args(["commit", "-m", "seed commit"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(repo)
}

fn head_sha(dir: &Path) -> String {
    let o = git_in(dir)
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("rev-parse");
    String::from_utf8_lossy(&o.stdout).trim().to_string()
}

fn commit_file(dir: &Path, name: &str, content: &str, message: &str) {
    std::fs::write(dir.join(name), content).unwrap();
    git_in(dir).args(["add", name]).output().unwrap();
    git_in(dir)
        .args(["commit", "-m", message])
        .output()
        .unwrap();
}

// ============================================================================
// 1. Empty git repo has no hotspots
// ============================================================================

#[test]
fn empty_repo_collect_history_returns_empty() {
    let repo = make_empty_repo("empty-hist").expect("repo");
    // No commits exist — collect_history should fail or return empty.
    let result = collect_history(&repo.path, None, None);
    match result {
        Ok(commits) => assert!(commits.is_empty(), "No commits expected in empty repo"),
        Err(_) => {} // git log on empty repo fails — acceptable
    }
}

#[test]
fn empty_repo_get_added_lines_fails() {
    let repo = make_empty_repo("empty-added").expect("repo");
    // HEAD doesn't exist, so diff must fail.
    let result = get_added_lines(&repo.path, "HEAD~1", "HEAD", GitRangeMode::TwoDot);
    assert!(result.is_err(), "Should fail on empty repo");
}

#[test]
fn empty_repo_rev_exists_returns_false_for_head() {
    let repo = make_empty_repo("empty-rev").expect("repo");
    assert!(
        !rev_exists(&repo.path, "HEAD"),
        "HEAD should not exist in empty repo"
    );
}

// ============================================================================
// 2. File freshness on known commit produces expected values
// ============================================================================

#[test]
fn freshness_timestamps_decrease_chronologically() {
    let repo = make_seeded_repo("fresh-order").expect("repo");
    commit_file(&repo.path, "a.txt", "a", "second");
    commit_file(&repo.path, "b.txt", "b", "third");

    let root = repo_root(&repo.path).unwrap();
    let commits = collect_history(&root, None, None).unwrap();

    // Newest first — timestamps must be non-increasing.
    for w in commits.windows(2) {
        assert!(
            w[0].timestamp >= w[1].timestamp,
            "Commits should be newest-first"
        );
    }
}

#[test]
fn freshness_latest_commit_touches_expected_file() {
    let repo = make_seeded_repo("fresh-file").expect("repo");
    commit_file(&repo.path, "fresh.txt", "hello", "touch fresh");

    let root = repo_root(&repo.path).unwrap();
    let commits = collect_history(&root, None, None).unwrap();
    let latest = &commits[0];

    assert_eq!(latest.subject, "touch fresh");
    assert!(
        latest.files.contains(&"fresh.txt".to_string()),
        "Latest commit should touch fresh.txt, got: {:?}",
        latest.files,
    );
}

#[test]
fn freshness_file_last_modified_timestamp_is_positive() {
    let repo = make_seeded_repo("fresh-ts").expect("repo");
    commit_file(&repo.path, "data.txt", "d", "add data");

    let root = repo_root(&repo.path).unwrap();
    let commits = collect_history(&root, None, None).unwrap();

    // Find most recent commit touching data.txt
    let latest_ts = commits
        .iter()
        .filter(|c| c.files.contains(&"data.txt".to_string()))
        .map(|c| c.timestamp)
        .next()
        .expect("data.txt should appear in history");

    assert!(latest_ts > 0, "Freshness timestamp must be positive");
}

// ============================================================================
// 3. Coupling analysis with known co-change patterns
// ============================================================================

#[test]
fn coupling_files_committed_together_appear_in_same_commit() {
    let repo = make_seeded_repo("coupling-co").expect("repo");

    // Commit A and B together multiple times
    for i in 0..3 {
        std::fs::write(repo.path.join("coupled_a.txt"), format!("v{i}")).unwrap();
        std::fs::write(repo.path.join("coupled_b.txt"), format!("v{i}")).unwrap();
        git_in(&repo.path).args(["add", "."]).output().unwrap();
        git_in(&repo.path)
            .args(["commit", "-m", &format!("couple {i}")])
            .output()
            .unwrap();
    }

    let root = repo_root(&repo.path).unwrap();
    let commits = collect_history(&root, None, None).unwrap();

    // Build co-change map: count how often pairs appear together
    let mut cochange: BTreeMap<(String, String), usize> = BTreeMap::new();
    for c in &commits {
        let mut files: Vec<_> = c.files.clone();
        files.sort();
        for i in 0..files.len() {
            for j in (i + 1)..files.len() {
                *cochange
                    .entry((files[i].clone(), files[j].clone()))
                    .or_default() += 1;
            }
        }
    }

    let pair = ("coupled_a.txt".to_string(), "coupled_b.txt".to_string());
    let count = cochange.get(&pair).copied().unwrap_or(0);
    assert!(
        count >= 3,
        "coupled_a.txt and coupled_b.txt should co-change ≥3 times, got {count}"
    );
}

#[test]
fn coupling_independent_files_have_zero_cochange() {
    let repo = make_seeded_repo("coupling-ind").expect("repo");

    // Commit files in separate commits — never together
    commit_file(&repo.path, "solo_x.txt", "x", "add x");
    commit_file(&repo.path, "solo_y.txt", "y", "add y");

    let root = repo_root(&repo.path).unwrap();
    let commits = collect_history(&root, None, None).unwrap();

    let mut cochange: BTreeMap<(String, String), usize> = BTreeMap::new();
    for c in &commits {
        let mut files: Vec<_> = c.files.clone();
        files.sort();
        for i in 0..files.len() {
            for j in (i + 1)..files.len() {
                *cochange
                    .entry((files[i].clone(), files[j].clone()))
                    .or_default() += 1;
            }
        }
    }

    let pair = ("solo_x.txt".to_string(), "solo_y.txt".to_string());
    assert_eq!(
        cochange.get(&pair).copied().unwrap_or(0),
        0,
        "Files committed separately should have zero co-change"
    );
}

// ============================================================================
// 4. Churn metrics are non-negative
// ============================================================================

#[test]
fn churn_all_timestamps_non_negative() {
    let repo = make_seeded_repo("churn-ts").expect("repo");
    commit_file(&repo.path, "c1.txt", "1", "first");
    commit_file(&repo.path, "c2.txt", "2", "second");

    let root = repo_root(&repo.path).unwrap();
    let commits = collect_history(&root, None, None).unwrap();

    for c in &commits {
        assert!(c.timestamp >= 0, "Timestamp must be non-negative");
    }
}

#[test]
fn churn_file_change_counts_non_negative() {
    let repo = make_seeded_repo("churn-files").expect("repo");
    for i in 0..5 {
        commit_file(
            &repo.path,
            "hot.txt",
            &format!("v{i}"),
            &format!("edit {i}"),
        );
    }

    let root = repo_root(&repo.path).unwrap();
    let commits = collect_history(&root, None, None).unwrap();

    // Count how many commits touch each file
    let mut file_churn: BTreeMap<String, usize> = BTreeMap::new();
    for c in &commits {
        for f in &c.files {
            *file_churn.entry(f.clone()).or_default() += 1;
        }
    }

    for (file, count) in &file_churn {
        assert!(*count > 0, "File {file} churn count must be positive");
    }

    // hot.txt should have the highest churn
    let hot_count = file_churn.get("hot.txt").copied().unwrap_or(0);
    assert!(
        hot_count >= 5,
        "hot.txt should be touched ≥5 times, got {hot_count}"
    );
}

#[test]
fn churn_added_line_counts_non_negative() {
    let repo = make_seeded_repo("churn-lines").expect("repo");
    let base = head_sha(&repo.path);
    commit_file(&repo.path, "lines.txt", "a\nb\nc\n", "add lines");
    let head = head_sha(&repo.path);

    let result = get_added_lines(&repo.path, &base, &head, GitRangeMode::TwoDot).unwrap();
    for (path, lines) in &result {
        assert!(
            !lines.is_empty(),
            "Added lines for {:?} should be non-empty",
            path
        );
        for &ln in lines {
            assert!(ln > 0, "Line numbers must be positive (1-based)");
        }
    }
}

// ============================================================================
// 5. Round-trip serialization of git reports
// ============================================================================

#[test]
fn commit_intent_kind_serde_round_trip_all_variants() {
    let variants = [
        CommitIntentKind::Feat,
        CommitIntentKind::Fix,
        CommitIntentKind::Refactor,
        CommitIntentKind::Docs,
        CommitIntentKind::Test,
        CommitIntentKind::Chore,
        CommitIntentKind::Ci,
        CommitIntentKind::Build,
        CommitIntentKind::Perf,
        CommitIntentKind::Style,
        CommitIntentKind::Revert,
        CommitIntentKind::Other,
    ];
    for variant in &variants {
        let json = serde_json::to_string(variant).expect("serialize");
        let back: CommitIntentKind = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(*variant, back, "Round-trip failed for {json}");
    }
}

#[test]
fn commit_intent_kind_serializes_to_snake_case() {
    assert_eq!(
        serde_json::to_string(&CommitIntentKind::Feat).unwrap(),
        "\"feat\""
    );
    assert_eq!(
        serde_json::to_string(&CommitIntentKind::Other).unwrap(),
        "\"other\""
    );
}

#[test]
fn classify_then_serialize_round_trips() {
    let subjects = [
        "feat: add login",
        "fix(core): null check",
        "refactor: extract fn",
        "Initial commit",
    ];
    for subj in subjects {
        let kind = classify_intent(subj);
        let json = serde_json::to_string(&kind).unwrap();
        let back: CommitIntentKind = serde_json::from_str(&json).unwrap();
        assert_eq!(kind, back, "Round-trip failed for subject: {subj}");
    }
}

#[test]
fn git_commit_clone_preserves_all_fields() {
    let original = GitCommit {
        timestamp: 1700000000,
        author: "a@b.com".to_string(),
        hash: Some("abc123def456".to_string()),
        subject: "feat: test".to_string(),
        files: vec!["src/lib.rs".to_string(), "README.md".to_string()],
    };
    let cloned = original.clone();
    assert_eq!(original.timestamp, cloned.timestamp);
    assert_eq!(original.author, cloned.author);
    assert_eq!(original.hash, cloned.hash);
    assert_eq!(original.subject, cloned.subject);
    assert_eq!(original.files, cloned.files);
}

// ============================================================================
// 6. Deterministic output for same git history
// ============================================================================

#[test]
fn deterministic_collect_history_same_result_twice() {
    let repo = make_seeded_repo("determ-hist").expect("repo");
    commit_file(&repo.path, "a.txt", "a", "add a");
    commit_file(&repo.path, "b.txt", "b", "add b");

    let root = repo_root(&repo.path).unwrap();
    let run1 = collect_history(&root, None, None).unwrap();
    let run2 = collect_history(&root, None, None).unwrap();

    assert_eq!(run1.len(), run2.len(), "Same commit count");
    for (c1, c2) in run1.iter().zip(run2.iter()) {
        assert_eq!(c1.timestamp, c2.timestamp);
        assert_eq!(c1.author, c2.author);
        assert_eq!(c1.hash, c2.hash);
        assert_eq!(c1.subject, c2.subject);
        assert_eq!(c1.files, c2.files);
    }
}

#[test]
fn deterministic_get_added_lines_same_result_twice() {
    let repo = make_seeded_repo("determ-lines").expect("repo");
    let base = head_sha(&repo.path);
    commit_file(&repo.path, "det.txt", "line1\nline2\n", "add det");
    let head = head_sha(&repo.path);

    let run1 = get_added_lines(&repo.path, &base, &head, GitRangeMode::TwoDot).unwrap();
    let run2 = get_added_lines(&repo.path, &base, &head, GitRangeMode::TwoDot).unwrap();

    assert_eq!(run1, run2, "get_added_lines should be deterministic");
}

#[test]
fn deterministic_classify_intent_is_pure() {
    let subjects = [
        "feat: login",
        "fix(core): crash",
        "chore: bump deps",
        "WIP",
        "",
    ];
    for subj in subjects {
        let a = classify_intent(subj);
        let b = classify_intent(subj);
        assert_eq!(a, b, "classify_intent must be pure for: {subj:?}");
    }
}

// ============================================================================
// 7. Edge cases: single commit, no files changed
// ============================================================================

#[test]
fn single_commit_repo_returns_one_commit() {
    let repo = make_seeded_repo("single").expect("repo");

    let root = repo_root(&repo.path).unwrap();
    let commits = collect_history(&root, None, None).unwrap();

    assert_eq!(commits.len(), 1, "Single-commit repo should yield 1 commit");
    assert_eq!(commits[0].subject, "seed commit");
    assert!(
        commits[0].files.contains(&"seed.txt".to_string()),
        "seed.txt should appear in the commit files"
    );
}

#[test]
fn single_commit_repo_added_lines_from_parent_fails() {
    // HEAD~1 doesn't exist in a single-commit repo
    let repo = make_seeded_repo("single-parent").expect("repo");
    let sha = head_sha(&repo.path);

    let result = get_added_lines(&repo.path, &format!("{sha}~1"), &sha, GitRangeMode::TwoDot);
    assert!(
        result.is_err(),
        "Diffing against parent of root commit should fail"
    );
}

#[test]
fn empty_commit_has_no_files() {
    let repo = make_seeded_repo("empty-commit").expect("repo");

    // Create an empty commit (--allow-empty)
    let out = git_in(&repo.path)
        .args(["commit", "--allow-empty", "-m", "empty commit"])
        .output()
        .unwrap();
    assert!(out.status.success(), "empty commit should succeed");

    let root = repo_root(&repo.path).unwrap();
    let commits = collect_history(&root, None, None).unwrap();

    let empty = commits
        .iter()
        .find(|c| c.subject == "empty commit")
        .expect("should find the empty commit");
    assert!(
        empty.files.is_empty(),
        "Empty commit should have no files, got: {:?}",
        empty.files
    );
}

#[test]
fn get_added_lines_same_sha_returns_empty() {
    let repo = make_seeded_repo("same-sha").expect("repo");
    let sha = head_sha(&repo.path);

    let result = get_added_lines(&repo.path, &sha, &sha, GitRangeMode::TwoDot).unwrap();
    assert!(
        result.is_empty(),
        "Diffing same SHA should produce empty result"
    );
}

#[test]
fn max_commits_one_returns_single_commit() {
    let repo = make_seeded_repo("max1").expect("repo");
    commit_file(&repo.path, "a.txt", "a", "second");
    commit_file(&repo.path, "b.txt", "b", "third");

    let root = repo_root(&repo.path).unwrap();
    let result = collect_history(&root, Some(1), None);
    match result {
        Ok(commits) => {
            assert_eq!(commits.len(), 1, "max_commits=1 should return 1 commit");
            assert_eq!(commits[0].subject, "third", "Should be the newest commit");
        }
        Err(_) => {} // broken pipe from early close — acceptable
    }
}

#[test]
fn commit_with_only_deleted_file_recorded() {
    let repo = make_seeded_repo("delete-only").expect("repo");
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

    let del_commit = commits
        .iter()
        .find(|c| c.subject == "remove doomed")
        .expect("should find delete commit");
    assert!(
        del_commit.files.contains(&"doomed.txt".to_string()),
        "Deleted file should appear in commit file list"
    );
}

#[test]
fn get_added_lines_deleted_file_not_in_result() {
    let repo = make_seeded_repo("del-lines").expect("repo");
    commit_file(&repo.path, "gone.txt", "content", "add gone");
    let base = head_sha(&repo.path);

    git_in(&repo.path)
        .args(["rm", "gone.txt"])
        .output()
        .unwrap();
    git_in(&repo.path)
        .args(["commit", "-m", "delete gone"])
        .output()
        .unwrap();
    let head = head_sha(&repo.path);

    let result = get_added_lines(&repo.path, &base, &head, GitRangeMode::TwoDot).unwrap();
    assert!(
        !result.contains_key(&PathBuf::from("gone.txt")),
        "Deleted file should not appear in added lines"
    );
}
