//! Extended BDD-style tests for tokmd-git.
//!
//! These tests cover edge cases not addressed by the existing test suite:
//! classify_intent boundary cases, collect_history with repeated file
//! modifications, GitRangeMode edge cases, resolve_base_ref fallback
//! behaviour, and get_added_lines with various diff scenarios.

use std::path::{Path, PathBuf};
use std::process::Command;

use tokmd_git::{
    classify_intent, collect_history, git_available, repo_root, resolve_base_ref, rev_exists,
    GitCommit, GitRangeMode,
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

fn make_repo(suffix: &str) -> Option<TempGitRepo> {
    if !git_available() {
        return None;
    }
    let id = format!(
        "{}-{:?}-{}-ext-{}",
        std::process::id(),
        std::thread::current().id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0),
        suffix,
    );
    let dir = std::env::temp_dir().join(format!("tokmd-git-ext-{}", id));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).ok();
    }
    std::fs::create_dir_all(&dir).ok()?;

    let ok = |out: std::process::Output| out.status.success();

    if !ok(git_in(&dir).args(["init"]).output().ok()?) {
        std::fs::remove_dir_all(&dir).ok();
        return None;
    }
    git_in(&dir)
        .args(["config", "user.email", "ext@test.com"])
        .output()
        .ok()?;
    git_in(&dir)
        .args(["config", "user.name", "Ext Tester"])
        .output()
        .ok()?;

    std::fs::write(dir.join("seed.txt"), "seed").ok()?;
    git_in(&dir).args(["add", "."]).output().ok()?;
    if !ok(git_in(&dir)
        .args(["commit", "-m", "seed commit"])
        .output()
        .ok()?)
    {
        std::fs::remove_dir_all(&dir).ok();
        return None;
    }
    Some(TempGitRepo { path: dir })
}

fn head_sha(dir: &Path) -> String {
    let o = git_in(dir)
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("rev-parse");
    String::from_utf8_lossy(&o.stdout).trim().to_string()
}

// ============================================================================
// Scenario: classify_intent – edge cases
// ============================================================================

#[test]
fn test_given_only_colon_when_classified_then_other() {
    assert_eq!(classify_intent(":"), CommitIntentKind::Other);
}

#[test]
fn test_given_only_parens_when_classified_then_other() {
    assert_eq!(classify_intent("(scope): thing"), CommitIntentKind::Other);
}

#[test]
fn test_given_unknown_conventional_prefix_when_classified_then_keyword_fallback() {
    // "release:" is not a known conventional type, falls to keyword heuristic
    assert_eq!(classify_intent("release: v1.0.0"), CommitIntentKind::Other);
}

#[test]
fn test_given_feat_with_bang_scope_when_classified_then_feat() {
    assert_eq!(
        classify_intent("feat(core)!: breaking API change"),
        CommitIntentKind::Feat
    );
}

#[test]
fn test_given_uppercase_fix_conventional_when_classified_then_fix() {
    assert_eq!(
        classify_intent("FIX(auth): session timeout"),
        CommitIntentKind::Fix
    );
}

#[test]
fn test_given_revert_colon_prefix_when_classified_then_revert() {
    assert_eq!(
        classify_intent("revert: accidentally merged"),
        CommitIntentKind::Revert
    );
}

#[test]
fn test_given_github_revert_format_when_classified_then_revert() {
    assert_eq!(
        classify_intent("Revert \"fix: null pointer\""),
        CommitIntentKind::Revert
    );
}

// ============================================================================
// Scenario: classify_intent – keyword heuristic nuances
// ============================================================================

#[test]
fn test_given_word_revert_in_middle_when_classified_then_revert() {
    assert_eq!(
        classify_intent("Please revert this change"),
        CommitIntentKind::Revert
    );
}

#[test]
fn test_given_add_at_start_when_classified_then_feat() {
    assert_eq!(
        classify_intent("Add new endpoint for users"),
        CommitIntentKind::Feat
    );
}

#[test]
fn test_given_implement_at_start_when_classified_then_feat() {
    assert_eq!(
        classify_intent("Implement caching for DB queries"),
        CommitIntentKind::Feat
    );
}

#[test]
fn test_given_introduce_at_start_when_classified_then_feat() {
    assert_eq!(
        classify_intent("Introduce rate limiting"),
        CommitIntentKind::Feat
    );
}

#[test]
fn test_given_patch_word_when_classified_then_fix() {
    assert_eq!(
        classify_intent("patch security vulnerability"),
        CommitIntentKind::Fix
    );
}

#[test]
fn test_given_word_cleanup_when_classified_then_chore() {
    assert_eq!(
        classify_intent("Cleanup old migration files"),
        CommitIntentKind::Chore
    );
}

#[test]
fn test_given_pipeline_word_when_classified_then_ci() {
    assert_eq!(
        classify_intent("Update pipeline for new runners"),
        CommitIntentKind::Ci
    );
}

#[test]
fn test_given_deps_word_when_classified_then_build() {
    assert_eq!(
        classify_intent("Bump deps to latest versions"),
        CommitIntentKind::Build
    );
}

// ============================================================================
// Scenario: classify_intent – word boundaries prevent false positives
// ============================================================================

#[test]
fn test_given_suffix_word_when_classified_then_no_false_match() {
    // "suffix" contains "fix" but should not match
    assert_ne!(
        classify_intent("Add a suffix to the name"),
        CommitIntentKind::Fix
    );
}

#[test]
fn test_given_documentary_word_when_classified_then_no_doc_match() {
    // "documentary" starts with "doc" but 'umentary' is alphanumeric after → no word boundary
    assert_ne!(
        classify_intent("Watch a documentary tonight"),
        CommitIntentKind::Docs
    );
}

// ============================================================================
// Scenario: GitCommit struct fields
// ============================================================================

#[test]
fn test_given_git_commit_with_no_hash_when_accessed_then_none() {
    let commit = GitCommit {
        timestamp: 0,
        author: String::new(),
        hash: None,
        subject: String::new(),
        files: vec![],
    };
    assert!(commit.hash.is_none());
}

#[test]
fn test_given_git_commit_with_empty_files_when_accessed_then_empty() {
    let commit = GitCommit {
        timestamp: 100,
        author: "dev@example.com".to_string(),
        hash: Some("abc".to_string()),
        subject: "init".to_string(),
        files: vec![],
    };
    assert!(commit.files.is_empty());
}

// ============================================================================
// Scenario: GitRangeMode – equality and default
// ============================================================================

#[test]
fn test_given_range_modes_when_compared_then_eq_works() {
    assert_eq!(GitRangeMode::TwoDot, GitRangeMode::TwoDot);
    assert_eq!(GitRangeMode::ThreeDot, GitRangeMode::ThreeDot);
    assert_ne!(GitRangeMode::TwoDot, GitRangeMode::ThreeDot);
}

#[test]
fn test_given_default_range_mode_then_two_dot() {
    let mode: GitRangeMode = Default::default();
    assert_eq!(mode, GitRangeMode::TwoDot);
}

#[test]
fn test_given_range_format_with_empty_refs_then_just_dots() {
    assert_eq!(GitRangeMode::TwoDot.format("", ""), "..");
    assert_eq!(GitRangeMode::ThreeDot.format("", ""), "...");
}

// ============================================================================
// Scenario: repo_root in non-git temp directory
// ============================================================================

#[test]
fn test_given_non_git_dir_when_repo_root_called_then_not_the_dir_itself() {
    let tmp = tempfile::tempdir().unwrap();
    let result = repo_root(tmp.path());
    // The temp dir is not a git repo; if repo_root returns Some, it's a parent repo
    if let Some(root) = result {
        assert_ne!(
            root.canonicalize().ok(),
            tmp.path().canonicalize().ok(),
            "temp dir should not be identified as a repo root"
        );
    }
}

// ============================================================================
// Scenario: rev_exists with HEAD in fresh repo
// ============================================================================

#[test]
fn test_given_fresh_repo_when_rev_exists_head_then_true() {
    let repo = make_repo("rev-head").expect("repo");
    assert!(rev_exists(&repo.path, "HEAD"));
}

#[test]
fn test_given_fresh_repo_when_rev_exists_nonsense_then_false() {
    let repo = make_repo("rev-nonsense").expect("repo");
    assert!(!rev_exists(&repo.path, "this-branch-does-not-exist-at-all"));
}

#[test]
fn test_given_non_git_dir_when_rev_exists_then_false() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(!rev_exists(tmp.path(), "HEAD"));
}

// ============================================================================
// Scenario: resolve_base_ref fast path and non-main rejection
// ============================================================================

#[test]
fn test_given_explicit_non_main_ref_when_missing_then_returns_none() {
    let repo = make_repo("resolve-custom").expect("repo");
    // Requesting a non-"main" ref that doesn't exist should return None
    // without attempting fallback
    let result = resolve_base_ref(&repo.path, "custom-branch-xyz");
    assert_eq!(result, None);
}

#[test]
fn test_given_head_as_ref_when_resolve_base_ref_then_returns_head() {
    let repo = make_repo("resolve-head").expect("repo");
    let result = resolve_base_ref(&repo.path, "HEAD");
    assert_eq!(result, Some("HEAD".to_string()));
}

// ============================================================================
// Scenario: collect_history with same file modified multiple times
// ============================================================================

#[test]
fn test_given_repeated_edits_when_history_collected_then_all_commits_present() {
    let repo = make_repo("repeated-edits").expect("repo");
    let root = repo_root(&repo.path).unwrap();

    // Modify the same file 3 more times (on top of seed commit)
    for i in 1..=3 {
        std::fs::write(repo.path.join("seed.txt"), format!("edit {i}")).unwrap();
        git_in(&repo.path).args(["add", "."]).output().unwrap();
        git_in(&repo.path)
            .args(["commit", "-m", &format!("edit {i}")])
            .output()
            .unwrap();
    }

    let commits = collect_history(&root, None, None).unwrap();
    // 1 seed + 3 edits = 4 commits
    assert_eq!(commits.len(), 4);

    // Every commit should list seed.txt in its files
    for c in &commits {
        assert!(
            c.files.iter().any(|f| f.contains("seed.txt")),
            "commit '{}' should touch seed.txt, files: {:?}",
            c.subject,
            c.files
        );
    }
}

// ============================================================================
// Scenario: collect_history file list correctness
// ============================================================================

#[test]
fn test_given_commit_with_two_files_when_history_collected_then_both_listed() {
    let repo = make_repo("two-files").expect("repo");
    let root = repo_root(&repo.path).unwrap();

    std::fs::write(repo.path.join("alpha.txt"), "a").unwrap();
    std::fs::write(repo.path.join("beta.txt"), "b").unwrap();
    git_in(&repo.path).args(["add", "."]).output().unwrap();
    git_in(&repo.path)
        .args(["commit", "-m", "add two files"])
        .output()
        .unwrap();

    let commits = collect_history(&root, None, None).unwrap();
    let latest = &commits[0];
    assert_eq!(latest.subject, "add two files");
    assert!(latest.files.contains(&"alpha.txt".to_string()));
    assert!(latest.files.contains(&"beta.txt".to_string()));
}

// ============================================================================
// Scenario: collect_history max_commits=1
// ============================================================================

#[test]
fn test_given_multi_commit_repo_when_max_commits_one_then_one_returned() {
    let repo = make_repo("max-one").expect("repo");
    let root = repo_root(&repo.path).unwrap();

    // Add a second commit
    std::fs::write(repo.path.join("extra.txt"), "x").unwrap();
    git_in(&repo.path).args(["add", "."]).output().unwrap();
    git_in(&repo.path)
        .args(["commit", "-m", "second"])
        .output()
        .unwrap();

    let commits = collect_history(&root, Some(1), None).unwrap();
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].subject, "second");
}

// ============================================================================
// Scenario: get_added_lines with deletion only
// ============================================================================

#[test]
fn test_given_only_deletion_when_get_added_lines_then_empty_map() {
    let repo = make_repo("delete-only").expect("repo");
    let base = head_sha(&repo.path);

    // Delete the seed file
    git_in(&repo.path)
        .args(["rm", "seed.txt"])
        .output()
        .unwrap();
    git_in(&repo.path)
        .args(["commit", "-m", "delete seed"])
        .output()
        .unwrap();
    let head = head_sha(&repo.path);

    let result =
        tokmd_git::get_added_lines(&repo.path, &base, &head, GitRangeMode::TwoDot).unwrap();
    assert!(
        result.is_empty(),
        "deletion-only diff should have no added lines"
    );
}

// ============================================================================
// Scenario: get_added_lines with new multi-line file
// ============================================================================

#[test]
fn test_given_new_multiline_file_when_get_added_lines_then_all_lines() {
    let repo = make_repo("multiline-new").expect("repo");
    let base = head_sha(&repo.path);

    std::fs::write(
        repo.path.join("multi.txt"),
        "line1\nline2\nline3\nline4\nline5\n",
    )
    .unwrap();
    git_in(&repo.path)
        .args(["add", "multi.txt"])
        .output()
        .unwrap();
    git_in(&repo.path)
        .args(["commit", "-m", "add multi.txt"])
        .output()
        .unwrap();
    let head = head_sha(&repo.path);

    let result =
        tokmd_git::get_added_lines(&repo.path, &base, &head, GitRangeMode::TwoDot).unwrap();
    let key = PathBuf::from("multi.txt");
    assert!(result.contains_key(&key));
    let lines = &result[&key];
    // All 5 lines should be present
    for i in 1..=5 {
        assert!(lines.contains(&i), "should contain line {i}");
    }
    assert_eq!(lines.len(), 5);
}

// ============================================================================
// Scenario: get_added_lines same ref returns empty
// ============================================================================

#[test]
fn test_given_same_ref_when_get_added_lines_then_empty() {
    let repo = make_repo("same-ref").expect("repo");
    let sha = head_sha(&repo.path);

    let result =
        tokmd_git::get_added_lines(&repo.path, &sha, &sha, GitRangeMode::TwoDot).unwrap();
    assert!(result.is_empty());
}

// ============================================================================
// Scenario: classify_intent – priority ordering
// ============================================================================

#[test]
fn test_given_revert_keyword_beats_fix_keyword() {
    // "revert" has higher priority than "fix" in keyword heuristic
    assert_eq!(classify_intent("revert the fix"), CommitIntentKind::Revert);
}

#[test]
fn test_given_fix_keyword_beats_feat_keyword() {
    // "fix" has higher priority than "feat"
    assert_eq!(
        classify_intent("fix the feature flag"),
        CommitIntentKind::Fix
    );
}

// ============================================================================
// Scenario: GitRangeMode Debug + Clone
// ============================================================================

#[test]
fn test_given_range_mode_when_debug_printed_then_not_empty() {
    let debug_str = format!("{:?}", GitRangeMode::TwoDot);
    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("TwoDot"));
}

#[test]
fn test_given_range_mode_when_cloned_then_equal() {
    let original = GitRangeMode::ThreeDot;
    let cloned = original;
    assert_eq!(original, cloned);
}

// ============================================================================
// Scenario: collect_history author is consistent
// ============================================================================

#[test]
fn test_given_repo_with_known_author_when_history_collected_then_all_match() {
    let repo = make_repo("author-check").expect("repo");
    let root = repo_root(&repo.path).unwrap();

    let commits = collect_history(&root, None, None).unwrap();
    for c in &commits {
        assert_eq!(c.author, "ext@test.com");
    }
}

// ============================================================================
// Scenario: collect_history timestamps are positive
// ============================================================================

#[test]
fn test_given_any_repo_when_history_collected_then_timestamps_positive() {
    let repo = make_repo("ts-positive").expect("repo");
    let root = repo_root(&repo.path).unwrap();

    let commits = collect_history(&root, None, None).unwrap();
    for c in &commits {
        assert!(c.timestamp > 0, "timestamp should be positive");
    }
}
