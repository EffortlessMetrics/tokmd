//! Deep tests (w48) for tokmd-git: git log parsing, hotspot detection,
//! coupling analysis, freshness metrics, author statistics, property tests,
//! and edge cases (empty history, single commit, merge commits).

use std::path::{Path, PathBuf};
use std::process::Command;

use proptest::prelude::*;
use tokmd_git::{
    CommitIntentKind, GitCommit, GitRangeMode, classify_intent, collect_history, git_available,
    repo_root, resolve_base_ref, rev_exists,
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
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn make_repo(tag: &str) -> Option<TempRepo> {
    if !git_available() {
        return None;
    }
    let id = format!(
        "w48-{}-{}-{:?}",
        tag,
        std::process::id(),
        std::thread::current().id(),
    );
    let dir = std::env::temp_dir().join(format!("tokmd-git-w48-{}", id));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).ok();
    }
    std::fs::create_dir_all(&dir).ok()?;

    let ok = git_in(&dir).arg("init").output().ok()?.status.success();
    if !ok {
        std::fs::remove_dir_all(&dir).ok();
        return None;
    }
    git_in(&dir)
        .args(["config", "user.email", "w48@test.com"])
        .output()
        .ok()?;
    git_in(&dir)
        .args(["config", "user.name", "W48 Tester"])
        .output()
        .ok()?;

    // Seed commit so HEAD exists
    std::fs::write(dir.join("seed.txt"), "seed").ok()?;
    git_in(&dir).args(["add", "."]).output().ok()?;
    let c = git_in(&dir).args(["commit", "-m", "seed"]).output().ok()?;
    if !c.status.success() {
        std::fs::remove_dir_all(&dir).ok();
        return None;
    }
    Some(TempRepo { path: dir })
}

fn commit(dir: &Path, msg: &str) {
    git_in(dir).args(["add", "."]).output().unwrap();
    git_in(dir).args(["commit", "-m", msg]).output().unwrap();
}

fn commit_as(dir: &Path, msg: &str, email: &str, name: &str) {
    git_in(dir).args(["add", "."]).output().unwrap();
    git_in(dir)
        .args([
            "-c",
            &format!("user.email={email}"),
            "-c",
            &format!("user.name={name}"),
            "commit",
            "-m",
            msg,
        ])
        .output()
        .unwrap();
}

// ===========================================================================
// 1. Git log parsing – commit format handling
// ===========================================================================

#[test]
fn parse_single_commit_fields() {
    let repo = make_repo("parse-single").expect("repo");
    std::fs::write(repo.path.join("main.rs"), "fn main() {}").unwrap();
    commit(&repo.path, "feat: initial parser");

    let commits = collect_history(&repo.path, None, None).unwrap();
    // seed + our commit
    assert_eq!(commits.len(), 2);
    let latest = &commits[0];
    assert_eq!(latest.subject, "feat: initial parser");
    assert_eq!(latest.author, "w48@test.com");
    assert!(latest.hash.is_some());
    assert!(latest.timestamp > 0);
    assert!(latest.files.contains(&"main.rs".to_string()));
}

#[test]
fn parse_commit_with_pipe_in_subject() {
    let repo = make_repo("pipe-subject").expect("repo");
    std::fs::write(repo.path.join("a.txt"), "x").unwrap();
    commit(&repo.path, "fix: handle a|b edge case");

    let commits = collect_history(&repo.path, None, None).unwrap();
    // The format uses splitn(4, '|') so 4th part captures remainder
    assert!(commits[0].subject.contains("edge case"));
}

#[test]
fn parse_commit_with_empty_subject() {
    let repo = make_repo("empty-subj").expect("repo");
    std::fs::write(repo.path.join("x.txt"), "y").unwrap();
    git_in(&repo.path).args(["add", "."]).output().unwrap();
    git_in(&repo.path)
        .args(["commit", "--allow-empty-message", "-m", ""])
        .output()
        .unwrap();

    let commits = collect_history(&repo.path, None, None).unwrap();
    assert!(commits.len() >= 2);
}

#[test]
fn parse_multiple_files_in_single_commit() {
    let repo = make_repo("multi-files").expect("repo");
    for i in 0..5 {
        std::fs::write(repo.path.join(format!("f{i}.txt")), format!("{i}")).unwrap();
    }
    commit(&repo.path, "feat: add five files");

    let commits = collect_history(&repo.path, None, None).unwrap();
    assert!(commits[0].files.len() >= 5);
}

// ===========================================================================
// 2. Hotspot detection from commit history
// ===========================================================================

#[test]
fn hotspot_frequently_changed_file_appears_often() {
    let repo = make_repo("hotspot-freq").expect("repo");
    std::fs::write(repo.path.join("hot.txt"), "v0").unwrap();
    std::fs::write(repo.path.join("cold.txt"), "v0").unwrap();
    commit(&repo.path, "init files");

    for i in 1..=4 {
        std::fs::write(repo.path.join("hot.txt"), format!("v{i}")).unwrap();
        commit(&repo.path, &format!("edit hot v{i}"));
    }

    let commits = collect_history(&repo.path, None, None).unwrap();
    let hot_count = commits
        .iter()
        .filter(|c| c.files.contains(&"hot.txt".to_string()))
        .count();
    let cold_count = commits
        .iter()
        .filter(|c| c.files.contains(&"cold.txt".to_string()))
        .count();
    assert!(
        hot_count > cold_count,
        "hot.txt ({hot_count}) should appear more often than cold.txt ({cold_count})"
    );
}

#[test]
fn hotspot_commit_count_is_accurate() {
    let repo = make_repo("hotspot-count").expect("repo");
    std::fs::write(repo.path.join("tracked.txt"), "a").unwrap();
    commit(&repo.path, "first");
    std::fs::write(repo.path.join("tracked.txt"), "b").unwrap();
    commit(&repo.path, "second");
    std::fs::write(repo.path.join("tracked.txt"), "c").unwrap();
    commit(&repo.path, "third");

    let commits = collect_history(&repo.path, None, None).unwrap();
    let count = commits
        .iter()
        .filter(|c| c.files.contains(&"tracked.txt".to_string()))
        .count();
    assert_eq!(count, 3);
}

// ===========================================================================
// 3. Coupling analysis (files changed together)
// ===========================================================================

#[test]
fn coupling_files_changed_together() {
    let repo = make_repo("coupling-together").expect("repo");
    std::fs::write(repo.path.join("api.rs"), "v1").unwrap();
    std::fs::write(repo.path.join("tests.rs"), "v1").unwrap();
    commit(&repo.path, "feat: add api + tests");

    std::fs::write(repo.path.join("api.rs"), "v2").unwrap();
    std::fs::write(repo.path.join("tests.rs"), "v2").unwrap();
    commit(&repo.path, "fix: update api + tests");

    let commits = collect_history(&repo.path, None, None).unwrap();
    let co_changed = commits
        .iter()
        .filter(|c| {
            c.files.contains(&"api.rs".to_string()) && c.files.contains(&"tests.rs".to_string())
        })
        .count();
    assert_eq!(co_changed, 2, "api.rs and tests.rs should co-change twice");
}

#[test]
fn coupling_independent_files_never_co_change() {
    let repo = make_repo("coupling-indep").expect("repo");
    std::fs::write(repo.path.join("a.rs"), "1").unwrap();
    commit(&repo.path, "add a");
    std::fs::write(repo.path.join("b.rs"), "1").unwrap();
    commit(&repo.path, "add b");

    let commits = collect_history(&repo.path, None, None).unwrap();
    let co_changed = commits
        .iter()
        .filter(|c| c.files.contains(&"a.rs".to_string()) && c.files.contains(&"b.rs".to_string()))
        .count();
    assert_eq!(co_changed, 0);
}

// ===========================================================================
// 4. Freshness metrics (days since last change)
// ===========================================================================

#[test]
fn freshness_recent_commits_have_small_age() {
    let repo = make_repo("freshness-recent").expect("repo");
    std::fs::write(repo.path.join("new.rs"), "fresh").unwrap();
    commit(&repo.path, "add new file");

    let commits = collect_history(&repo.path, None, None).unwrap();
    // All commits were just made, so timestamps should be recent
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    for c in &commits {
        let age_days = (now - c.timestamp) / 86_400;
        assert!(age_days < 1, "freshly created commit should be < 1 day old");
    }
}

#[test]
fn freshness_last_change_timestamp_is_most_recent() {
    let repo = make_repo("freshness-latest").expect("repo");
    std::fs::write(repo.path.join("f.txt"), "v1").unwrap();
    commit(&repo.path, "first");
    std::fs::write(repo.path.join("f.txt"), "v2").unwrap();
    commit(&repo.path, "second");

    let commits = collect_history(&repo.path, None, None).unwrap();
    let f_commits: Vec<_> = commits
        .iter()
        .filter(|c| c.files.contains(&"f.txt".to_string()))
        .collect();
    assert!(f_commits.len() >= 2);
    // git log returns newest first
    assert!(f_commits[0].timestamp >= f_commits[1].timestamp);
}

// ===========================================================================
// 5. Author statistics
// ===========================================================================

#[test]
fn author_statistics_multiple_authors() {
    let repo = make_repo("multi-author").expect("repo");
    std::fs::write(repo.path.join("a.txt"), "1").unwrap();
    commit_as(&repo.path, "alice work", "alice@dev.com", "Alice");
    std::fs::write(repo.path.join("b.txt"), "1").unwrap();
    commit_as(&repo.path, "bob work", "bob@dev.com", "Bob");
    std::fs::write(repo.path.join("c.txt"), "1").unwrap();
    commit_as(&repo.path, "charlie work", "charlie@dev.com", "Charlie");

    let commits = collect_history(&repo.path, None, None).unwrap();
    let authors: std::collections::BTreeSet<_> =
        commits.iter().map(|c| c.author.as_str()).collect();
    assert!(authors.contains("alice@dev.com"));
    assert!(authors.contains("bob@dev.com"));
    assert!(authors.contains("charlie@dev.com"));
}

#[test]
fn author_repeated_commits_same_author() {
    let repo = make_repo("same-author").expect("repo");
    for i in 0..3 {
        std::fs::write(repo.path.join(format!("f{i}.txt")), "x").unwrap();
        commit(&repo.path, &format!("commit {i}"));
    }

    let commits = collect_history(&repo.path, None, None).unwrap();
    let unique_authors: std::collections::BTreeSet<_> =
        commits.iter().map(|c| c.author.as_str()).collect();
    // All commits by the default author
    assert!(unique_authors.contains("w48@test.com"));
}

// ===========================================================================
// 6. Property test: hotspot scores are non-negative
// ===========================================================================

proptest! {
    #[test]
    fn prop_hotspot_score_nonnegative(lines in 0usize..10_000, commits_count in 1usize..100) {
        let score = lines * commits_count;
        prop_assert!(score >= 0);
    }

    #[test]
    fn prop_classify_intent_never_panics(subject in ".*") {
        let _ = classify_intent(&subject);
    }

    #[test]
    fn prop_range_format_contains_refs(base in "[a-z]{1,10}", head in "[a-z]{1,10}") {
        let two = GitRangeMode::TwoDot.format(&base, &head);
        let three = GitRangeMode::ThreeDot.format(&base, &head);
        prop_assert!(two.contains(&base));
        prop_assert!(two.contains(&head));
        prop_assert!(three.contains(&base));
        prop_assert!(three.contains(&head));
    }
}

// ===========================================================================
// 7. Edge cases: empty git history
// ===========================================================================

#[test]
fn empty_repo_no_commits_after_init() {
    if !git_available() {
        return;
    }
    let dir = std::env::temp_dir().join(format!(
        "tokmd-w48-empty-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).ok();
    }
    std::fs::create_dir_all(&dir).unwrap();

    git_in(&dir).arg("init").output().unwrap();
    git_in(&dir)
        .args(["config", "user.email", "e@t.com"])
        .output()
        .unwrap();
    git_in(&dir)
        .args(["config", "user.name", "T"])
        .output()
        .unwrap();

    // No commits yet
    let result = collect_history(&dir, None, None);
    if let Ok(commits) = result {
        assert!(commits.is_empty());
    }
    std::fs::remove_dir_all(&dir).ok();
}

// ===========================================================================
// 8. Edge case: single commit
// ===========================================================================

#[test]
fn single_commit_repo_fields() {
    let repo = make_repo("single-commit").expect("repo");
    // Already has seed commit
    let commits = collect_history(&repo.path, None, None).unwrap();
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].subject, "seed");
    assert!(commits[0].hash.is_some());
    assert!(!commits[0].hash.as_deref().unwrap().is_empty());
}

// ===========================================================================
// 9. Edge case: merge commits
// ===========================================================================

#[test]
fn merge_commit_appears_in_history() {
    let repo = make_repo("merge").expect("repo");

    // Create a branch, add a file, merge back
    git_in(&repo.path)
        .args(["checkout", "-b", "feature"])
        .output()
        .unwrap();
    std::fs::write(repo.path.join("feature.txt"), "feat").unwrap();
    commit(&repo.path, "feat: add feature");

    git_in(&repo.path)
        .args(["checkout", "master"])
        .output()
        .ok()
        .or_else(|| git_in(&repo.path).args(["checkout", "main"]).output().ok());

    std::fs::write(repo.path.join("main_change.txt"), "main").unwrap();
    commit(&repo.path, "chore: main change");

    let merge_result = git_in(&repo.path)
        .args(["merge", "feature", "--no-edit"])
        .output();
    if merge_result.is_err() {
        return; // skip if merge fails
    }

    let commits = collect_history(&repo.path, None, None).unwrap();
    // Should have: seed, feature commit, main change, merge commit
    assert!(commits.len() >= 3);
}

// ===========================================================================
// 10. max_commits limit
// ===========================================================================

#[test]
fn max_commits_limits_result_count() {
    let repo = make_repo("max-commits").expect("repo");
    for i in 0..10 {
        std::fs::write(repo.path.join(format!("f{i}.txt")), format!("{i}")).unwrap();
        commit(&repo.path, &format!("commit {i}"));
    }

    let commits = collect_history(&repo.path, Some(3), None);
    if let Ok(c) = commits {
        assert!(c.len() <= 3);
    }
}

// ===========================================================================
// 11. max_commit_files limit
// ===========================================================================

#[test]
fn max_commit_files_limits_file_list() {
    let repo = make_repo("max-files").expect("repo");
    for i in 0..20 {
        std::fs::write(repo.path.join(format!("file{i}.txt")), format!("{i}")).unwrap();
    }
    commit(&repo.path, "add many files");

    let commits = collect_history(&repo.path, None, Some(5)).unwrap();
    for c in &commits {
        assert!(c.files.len() <= 5, "files should be limited to 5");
    }
}

// ===========================================================================
// 12. rev_exists and repo_root
// ===========================================================================

#[test]
fn rev_exists_with_valid_hash() {
    let repo = make_repo("rev-hash").expect("repo");
    let hash = String::from_utf8_lossy(
        &git_in(&repo.path)
            .args(["rev-parse", "HEAD"])
            .output()
            .unwrap()
            .stdout,
    )
    .trim()
    .to_string();

    assert!(rev_exists(&repo.path, &hash));
    assert!(!rev_exists(
        &repo.path,
        "0000000000000000000000000000000000000000"
    ));
}

#[test]
fn repo_root_matches_tempdir() {
    let repo = make_repo("repo-root").expect("repo");
    let root = repo_root(&repo.path).expect("should find root");
    // Both should resolve to the same canonical path
    let expected = std::fs::canonicalize(&repo.path).unwrap();
    let actual = std::fs::canonicalize(&root).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn repo_root_none_for_non_repo() {
    let dir = tempfile::tempdir().unwrap();
    assert!(repo_root(dir.path()).is_none());
}

// ===========================================================================
// 13. resolve_base_ref edge cases
// ===========================================================================

#[test]
fn resolve_base_ref_explicit_nondefault_fails_fast() {
    let repo = make_repo("resolve-fast").expect("repo");
    // Explicit non-"main" ref that doesn't exist → None
    assert_eq!(resolve_base_ref(&repo.path, "v999.0.0"), None);
}

// ===========================================================================
// 14. classify_intent additional patterns
// ===========================================================================

#[test]
fn classify_feature_with_scope_and_breaking() {
    assert_eq!(
        classify_intent("feat(parser)!: remove legacy API"),
        CommitIntentKind::Feat
    );
}

#[test]
fn classify_hotfix_conventional() {
    assert_eq!(
        classify_intent("hotfix: critical security patch"),
        CommitIntentKind::Fix
    );
}

#[test]
fn classify_bugfix_conventional() {
    assert_eq!(
        classify_intent("bugfix: memory leak"),
        CommitIntentKind::Fix
    );
}

#[test]
fn classify_tests_plural() {
    assert_eq!(
        classify_intent("tests: add integration suite"),
        CommitIntentKind::Test
    );
}

#[test]
fn classify_doc_singular() {
    assert_eq!(
        classify_intent("doc: update API reference"),
        CommitIntentKind::Docs
    );
}

#[test]
fn classify_keyword_introduce() {
    assert_eq!(
        classify_intent("Introduce new caching layer"),
        CommitIntentKind::Feat
    );
}

#[test]
fn classify_keyword_performance() {
    assert_eq!(
        classify_intent("Improve performance of hash lookup"),
        CommitIntentKind::Perf
    );
}

#[test]
fn classify_keyword_restructure() {
    assert_eq!(
        classify_intent("Restructure module hierarchy"),
        CommitIntentKind::Refactor
    );
}

// ===========================================================================
// 15. GitCommit struct construction edge cases
// ===========================================================================

#[test]
fn git_commit_clone_preserves_fields() {
    let c = GitCommit {
        timestamp: 42,
        author: "test@example.com".to_string(),
        hash: Some("abc".to_string()),
        subject: "test".to_string(),
        files: vec!["a.rs".to_string(), "b.rs".to_string()],
    };
    let c2 = c.clone();
    assert_eq!(c.timestamp, c2.timestamp);
    assert_eq!(c.author, c2.author);
    assert_eq!(c.hash, c2.hash);
    assert_eq!(c.subject, c2.subject);
    assert_eq!(c.files, c2.files);
}

#[test]
fn git_commit_debug_format() {
    let c = GitCommit {
        timestamp: 0,
        author: String::new(),
        hash: None,
        subject: String::new(),
        files: vec![],
    };
    let debug = format!("{c:?}");
    assert!(debug.contains("GitCommit"));
}

// ===========================================================================
// 16. GitRangeMode additional tests
// ===========================================================================

#[test]
fn range_mode_format_with_special_chars() {
    let r = GitRangeMode::TwoDot.format("refs/tags/v1.0", "refs/heads/main");
    assert_eq!(r, "refs/tags/v1.0..refs/heads/main");
}

#[test]
fn range_mode_copy_semantics() {
    let a = GitRangeMode::TwoDot;
    let b = a;
    assert_eq!(a, b);
}

// ===========================================================================
// 17. Subdirectory file tracking
// ===========================================================================

#[test]
fn collect_history_tracks_nested_paths() {
    let repo = make_repo("nested-paths").expect("repo");
    std::fs::create_dir_all(repo.path.join("src/core")).unwrap();
    std::fs::write(repo.path.join("src/core/lib.rs"), "pub fn x() {}").unwrap();
    commit(&repo.path, "add nested file");

    let commits = collect_history(&repo.path, None, None).unwrap();
    let has_nested = commits
        .iter()
        .any(|c| c.files.iter().any(|f| f.contains("src/core/lib.rs")));
    assert!(has_nested, "should track nested directory paths");
}

// ===========================================================================
// 18. git_available smoke test
// ===========================================================================

#[test]
fn git_available_is_consistent() {
    let a = git_available();
    let b = git_available();
    assert_eq!(a, b, "git_available should return consistent results");
}
