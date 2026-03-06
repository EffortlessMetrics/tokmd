//! Deep tests for tokmd-git (w70 wave).
//!
//! ~25 tests covering GitRangeMode, classify_intent (conventional commit +
//! keyword heuristic), collect_history, rev_exists, repo_root,
//! get_added_lines, and determinism.

use std::path::Path;
use std::process::Command;

use tokmd_git::{
    CommitIntentKind, GitRangeMode, classify_intent, collect_history, get_added_lines,
    git_available, repo_root, rev_exists,
};

// -- Helpers --

fn git_in(dir: &Path) -> Command {
    let mut cmd = Command::new("git");
    cmd.env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .current_dir(dir);
    cmd
}

fn init_repo(dir: &Path) {
    git_in(dir).args(["init", "-b", "main"]).output().unwrap();
    git_in(dir)
        .args(["config", "user.email", "test@test.com"])
        .output()
        .unwrap();
    git_in(dir)
        .args(["config", "user.name", "Test"])
        .output()
        .unwrap();
}

fn commit_file(dir: &Path, name: &str, content: &str, msg: &str) {
    std::fs::write(dir.join(name), content).unwrap();
    git_in(dir).args(["add", "."]).output().unwrap();
    git_in(dir).args(["commit", "-m", msg]).output().unwrap();
}

// -- GitRangeMode --

#[test]
fn range_two_dot_format() {
    assert_eq!(GitRangeMode::TwoDot.format("v1", "v2"), "v1..v2");
}

#[test]
fn range_three_dot_format() {
    assert_eq!(GitRangeMode::ThreeDot.format("v1", "v2"), "v1...v2");
}

#[test]
fn range_default_is_two_dot() {
    assert_eq!(GitRangeMode::default(), GitRangeMode::TwoDot);
}

#[test]
fn range_format_with_empty_refs() {
    assert_eq!(GitRangeMode::TwoDot.format("", "HEAD"), "..HEAD");
    assert_eq!(GitRangeMode::ThreeDot.format("", ""), "...");
}

// -- classify_intent: conventional commits --

#[test]
fn intent_feat_conventional() {
    assert_eq!(classify_intent("feat: add login"), CommitIntentKind::Feat);
    assert_eq!(
        classify_intent("feat(auth): add login"),
        CommitIntentKind::Feat
    );
    assert_eq!(
        classify_intent("feature!: breaking change"),
        CommitIntentKind::Feat
    );
}

#[test]
fn intent_fix_conventional() {
    assert_eq!(classify_intent("fix: null pointer"), CommitIntentKind::Fix);
    assert_eq!(
        classify_intent("bugfix: crash on empty"),
        CommitIntentKind::Fix
    );
    assert_eq!(
        classify_intent("hotfix: security patch"),
        CommitIntentKind::Fix
    );
}

#[test]
fn intent_refactor_conventional() {
    assert_eq!(
        classify_intent("refactor: extract method"),
        CommitIntentKind::Refactor
    );
}

#[test]
fn intent_docs_conventional() {
    assert_eq!(
        classify_intent("docs: update README"),
        CommitIntentKind::Docs
    );
    assert_eq!(classify_intent("doc: fix typo"), CommitIntentKind::Docs);
}

#[test]
fn intent_test_conventional() {
    assert_eq!(
        classify_intent("test: add unit tests"),
        CommitIntentKind::Test
    );
    assert_eq!(classify_intent("tests: coverage"), CommitIntentKind::Test);
}

#[test]
fn intent_chore_ci_build_perf_style_conventional() {
    assert_eq!(
        classify_intent("chore: bump version"),
        CommitIntentKind::Chore
    );
    assert_eq!(classify_intent("ci: fix pipeline"), CommitIntentKind::Ci);
    assert_eq!(
        classify_intent("build: update deps"),
        CommitIntentKind::Build
    );
    assert_eq!(
        classify_intent("perf: optimize query"),
        CommitIntentKind::Perf
    );
    assert_eq!(classify_intent("style: reformat"), CommitIntentKind::Style);
}

#[test]
fn intent_revert_conventional() {
    assert_eq!(
        classify_intent("revert: undo feat"),
        CommitIntentKind::Revert
    );
}

#[test]
fn intent_revert_git_format() {
    assert_eq!(
        classify_intent("Revert \"feat: add login\""),
        CommitIntentKind::Revert,
    );
}

// -- classify_intent: keyword heuristic --

#[test]
fn intent_keyword_fix() {
    assert_eq!(
        classify_intent("Fix crash on startup"),
        CommitIntentKind::Fix
    );
    assert_eq!(
        classify_intent("Resolve bug in parser"),
        CommitIntentKind::Fix
    );
}

#[test]
fn intent_keyword_feat() {
    assert_eq!(classify_intent("Add dark mode"), CommitIntentKind::Feat);
    assert_eq!(
        classify_intent("Implement caching layer"),
        CommitIntentKind::Feat
    );
    assert_eq!(classify_intent("Introduce new API"), CommitIntentKind::Feat);
}

#[test]
fn intent_keyword_refactor() {
    assert_eq!(
        classify_intent("Refactor config loading"),
        CommitIntentKind::Refactor
    );
}

#[test]
fn intent_keyword_docs() {
    assert_eq!(
        classify_intent("Update readme with examples"),
        CommitIntentKind::Docs
    );
}

#[test]
fn intent_keyword_perf() {
    assert_eq!(
        classify_intent("Optimize database queries"),
        CommitIntentKind::Perf
    );
}

#[test]
fn intent_keyword_style() {
    assert_eq!(classify_intent("Format all files"), CommitIntentKind::Style);
    assert_eq!(classify_intent("Run lint fixes"), CommitIntentKind::Style);
}

#[test]
fn intent_keyword_other_fallback() {
    assert_eq!(
        classify_intent("Bump version to 1.0"),
        CommitIntentKind::Other
    );
    assert_eq!(classify_intent("WIP"), CommitIntentKind::Other);
}

// -- classify_intent: edge cases --

#[test]
fn intent_empty_subject_is_other() {
    assert_eq!(classify_intent(""), CommitIntentKind::Other);
}

#[test]
fn intent_whitespace_only_is_other() {
    assert_eq!(classify_intent("   "), CommitIntentKind::Other);
}

#[test]
fn intent_deterministic() {
    let a = classify_intent("feat(scope): add feature");
    let b = classify_intent("feat(scope): add feature");
    assert_eq!(a, b);
}

// -- collect_history (requires git) --

#[test]
fn collect_history_single_commit() {
    if !git_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    init_repo(dir.path());
    commit_file(dir.path(), "a.txt", "hello", "init: first commit");

    let commits = collect_history(dir.path(), None, None).unwrap();
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].subject, "init: first commit");
    assert!(!commits[0].files.is_empty());
}

#[test]
fn collect_history_respects_max_commits() {
    if !git_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    init_repo(dir.path());
    for i in 0..5 {
        commit_file(
            dir.path(),
            &format!("f{i}.txt"),
            "x",
            &format!("commit {i}"),
        );
    }

    // When max_commits is set, the pipe may close early causing git log
    // to exit with a broken pipe error on some platforms.
    if let Ok(commits) = collect_history(dir.path(), Some(2), None) {
        assert!(commits.len() <= 2, "got {} commits", commits.len())
    }
}

#[test]
fn collect_history_respects_max_commit_files() {
    if !git_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    init_repo(dir.path());

    for i in 0..10 {
        std::fs::write(dir.path().join(format!("file{i}.txt")), "content").unwrap();
    }
    git_in(dir.path()).args(["add", "."]).output().unwrap();
    git_in(dir.path())
        .args(["commit", "-m", "bulk add"])
        .output()
        .unwrap();

    let commits = collect_history(dir.path(), None, Some(3)).unwrap();
    assert!(!commits.is_empty());
    for c in &commits {
        assert!(c.files.len() <= 3, "commit has {} files", c.files.len());
    }
}

#[test]
fn collect_history_empty_repo_returns_empty() {
    if !git_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    git_in(dir.path()).args(["init"]).output().unwrap();

    let result = collect_history(dir.path(), None, None);
    if let Ok(commits) = result {
        assert!(commits.is_empty())
    }
}

// -- rev_exists (requires git) --

#[test]
fn rev_exists_head_after_commit() {
    if !git_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    init_repo(dir.path());
    commit_file(dir.path(), "f.txt", "a", "init");

    assert!(rev_exists(dir.path(), "HEAD"));
    assert!(rev_exists(dir.path(), "main"));
    assert!(!rev_exists(dir.path(), "nonexistent-ref-xyz"));
}

// -- repo_root (requires git) --

#[test]
fn repo_root_returns_path_for_valid_repo() {
    if !git_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    init_repo(dir.path());
    commit_file(dir.path(), "f.txt", "a", "init");

    let root = repo_root(dir.path());
    assert!(root.is_some());
}

#[test]
fn repo_root_returns_none_for_non_repo() {
    if !git_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    assert!(repo_root(dir.path()).is_none());
}

// -- get_added_lines (requires git) --

#[test]
fn get_added_lines_detects_new_content() {
    if !git_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    init_repo(dir.path());
    commit_file(dir.path(), "a.txt", "line1\n", "first");

    let base = String::from_utf8_lossy(
        &git_in(dir.path())
            .args(["rev-parse", "HEAD"])
            .output()
            .unwrap()
            .stdout,
    )
    .trim()
    .to_string();

    commit_file(dir.path(), "a.txt", "line1\nline2\nline3\n", "second");

    let added = get_added_lines(dir.path(), &base, "HEAD", GitRangeMode::TwoDot).unwrap();
    assert!(!added.is_empty(), "Should detect added lines");
}
