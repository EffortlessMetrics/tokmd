//! BDD-style scenario tests for tokmd-git.
//!
//! Each test follows Given/When/Then structure to document behaviour
//! in a human-readable way.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use tokmd_git::{
    classify_intent, collect_history, git_available, repo_root, GitCommit, GitRangeMode,
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

/// Create a minimal git repo with one seed commit.
fn make_repo(suffix: &str) -> Option<TempGitRepo> {
    if !git_available() {
        return None;
    }
    let id = format!(
        "{}-{:?}-{}-bdd-{}",
        std::process::id(),
        std::thread::current().id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0),
        suffix
    );
    let dir = std::env::temp_dir().join(format!("tokmd-git-bdd-{}", id));
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
        .args(["config", "user.email", "bdd@test.com"])
        .output()
        .ok()?;
    git_in(&dir)
        .args(["config", "user.name", "BDD Tester"])
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
// Scenario: repo_root detection
// ============================================================================

#[test]
fn scenario_repo_root_from_root_directory() {
    // Given a freshly initialised git repository
    let repo = make_repo("root-detect").expect("repo");
    // When repo_root is called on the repo root
    let root = repo_root(&repo.path);
    // Then it returns a path that contains .git
    let root = root.expect("should find root");
    assert!(root.join(".git").exists());
}

#[test]
fn scenario_repo_root_from_deeply_nested_subdir() {
    // Given a repo with deeply nested subdirectories
    let repo = make_repo("nested").expect("repo");
    let deep = repo.path.join("a").join("b").join("c").join("d");
    std::fs::create_dir_all(&deep).unwrap();
    // When repo_root is called on the deepest subdirectory
    let root = repo_root(&deep);
    // Then it returns the actual repo root
    let root = root.expect("should find root");
    let expected = repo.path.canonicalize().unwrap();
    let actual = root.canonicalize().unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn scenario_repo_root_outside_any_repo() {
    // Given a plain directory with no .git
    let dir = tempfile::tempdir().unwrap();
    // When repo_root is called
    let result = repo_root(dir.path());
    // Then it returns None (or Some if the temp dir happens to be inside a repo)
    if let Some(found) = result {
        assert_ne!(
            found.canonicalize().ok(),
            dir.path().canonicalize().ok(),
            "should not treat the temp dir itself as a repo root"
        );
    }
}

// ============================================================================
// Scenario: collect_history parsing
// ============================================================================

#[test]
fn scenario_collect_history_returns_commits_in_reverse_chronological_order() {
    // Given a repo with 3 sequential commits
    let repo = make_repo("chrono").expect("repo");
    std::fs::write(repo.path.join("a.txt"), "a").unwrap();
    git_in(&repo.path).args(["add", "."]).output().unwrap();
    git_in(&repo.path)
        .args(["commit", "-m", "second"])
        .output()
        .unwrap();

    std::fs::write(repo.path.join("b.txt"), "b").unwrap();
    git_in(&repo.path).args(["add", "."]).output().unwrap();
    git_in(&repo.path)
        .args(["commit", "-m", "third"])
        .output()
        .unwrap();

    let root = repo_root(&repo.path).unwrap();
    // When we collect history
    let commits = collect_history(&root, None, None).unwrap();
    // Then commits arrive in reverse chronological order (newest first)
    assert_eq!(commits.len(), 3);
    for window in commits.windows(2) {
        assert!(
            window[0].timestamp >= window[1].timestamp,
            "commits should be newest-first: {} >= {}",
            window[0].timestamp,
            window[1].timestamp
        );
    }
}

#[test]
fn scenario_collect_history_records_author_email() {
    // Given a repo with a known author email
    let repo = make_repo("author").expect("repo");
    let root = repo_root(&repo.path).unwrap();
    // When we collect history
    let commits = collect_history(&root, None, None).unwrap();
    // Then every commit has the configured author email
    for c in &commits {
        assert_eq!(c.author, "bdd@test.com");
    }
}

#[test]
fn scenario_collect_history_captures_commit_hash() {
    // Given a repo with commits
    let repo = make_repo("hash").expect("repo");
    let root = repo_root(&repo.path).unwrap();
    // When we collect history
    let commits = collect_history(&root, None, None).unwrap();
    // Then each commit has a 40-hex-char hash
    for c in &commits {
        let h = c.hash.as_ref().expect("hash should be present");
        assert_eq!(h.len(), 40, "git SHA should be 40 hex chars");
        assert!(h.chars().all(|ch| ch.is_ascii_hexdigit()));
    }
}

#[test]
fn scenario_collect_history_captures_subject_line() {
    // Given a repo with a commit whose subject is "seed commit"
    let repo = make_repo("subject").expect("repo");
    let root = repo_root(&repo.path).unwrap();
    // When we collect history
    let commits = collect_history(&root, None, None).unwrap();
    // Then the subject is captured
    assert_eq!(commits[0].subject, "seed commit");
}

#[test]
fn scenario_collect_history_each_commit_has_positive_timestamp() {
    // Given a repo
    let repo = make_repo("ts").expect("repo");
    let root = repo_root(&repo.path).unwrap();
    // When we collect history
    let commits = collect_history(&root, None, None).unwrap();
    // Then every timestamp is positive
    for c in &commits {
        assert!(c.timestamp > 0, "timestamp should be positive");
    }
}

#[test]
fn scenario_collect_history_with_max_commit_files_preserves_commit_count() {
    // Given a repo with two commits, each touching a different file
    let repo = make_repo("files-limit").expect("repo");
    for name in &["x.txt", "y.txt"] {
        std::fs::write(repo.path.join(name), name).unwrap();
        git_in(&repo.path).args(["add", "."]).output().unwrap();
        git_in(&repo.path)
            .args(["commit", "-m", &format!("add {}", name)])
            .output()
            .unwrap();
    }
    let root = repo_root(&repo.path).unwrap();
    // When we limit files per commit to 0
    let commits = collect_history(&root, None, Some(0)).unwrap();
    // Then all commits are still returned, just with empty file lists
    assert_eq!(commits.len(), 3); // seed + x + y
    for c in &commits {
        assert!(c.files.is_empty());
    }
}

// ============================================================================
// Scenario: GitRangeMode formatting
// ============================================================================

#[test]
fn scenario_two_dot_range_format() {
    // Given TwoDot mode with base "v1.0" and head "v2.0"
    // When formatted
    let range = GitRangeMode::TwoDot.format("v1.0", "v2.0");
    // Then it produces "v1.0..v2.0"
    assert_eq!(range, "v1.0..v2.0");
}

#[test]
fn scenario_three_dot_range_format() {
    // Given ThreeDot mode with base "origin/main" and head "feature"
    // When formatted
    let range = GitRangeMode::ThreeDot.format("origin/main", "feature");
    // Then it produces "origin/main...feature"
    assert_eq!(range, "origin/main...feature");
}

#[test]
fn scenario_range_format_preserves_special_characters() {
    // Given refs with special chars (tags, slashes)
    let range = GitRangeMode::TwoDot.format("refs/tags/v1.0-rc.1", "refs/heads/feat/foo");
    // Then all characters are preserved verbatim
    assert_eq!(range, "refs/tags/v1.0-rc.1..refs/heads/feat/foo");
}

// ============================================================================
// Scenario: classify_intent – conventional commits
// ============================================================================

#[test]
fn scenario_classify_feat_conventional() {
    assert_eq!(classify_intent("feat: add login"), CommitIntentKind::Feat);
    assert_eq!(
        classify_intent("feat(auth): add login"),
        CommitIntentKind::Feat
    );
    assert_eq!(
        classify_intent("feat!: breaking change"),
        CommitIntentKind::Feat
    );
    assert_eq!(
        classify_intent("feature: new widget"),
        CommitIntentKind::Feat
    );
}

#[test]
fn scenario_classify_fix_conventional() {
    assert_eq!(
        classify_intent("fix: null pointer"),
        CommitIntentKind::Fix
    );
    assert_eq!(
        classify_intent("bugfix: crash on start"),
        CommitIntentKind::Fix
    );
    assert_eq!(
        classify_intent("hotfix: urgent patch"),
        CommitIntentKind::Fix
    );
    assert_eq!(
        classify_intent("fix(core)!: breaking fix"),
        CommitIntentKind::Fix
    );
}

#[test]
fn scenario_classify_refactor_conventional() {
    assert_eq!(
        classify_intent("refactor: extract method"),
        CommitIntentKind::Refactor
    );
    assert_eq!(
        classify_intent("refactor(api): simplify handler"),
        CommitIntentKind::Refactor
    );
}

#[test]
fn scenario_classify_docs_conventional() {
    assert_eq!(
        classify_intent("docs: update readme"),
        CommitIntentKind::Docs
    );
    assert_eq!(
        classify_intent("doc: add example"),
        CommitIntentKind::Docs
    );
}

#[test]
fn scenario_classify_test_conventional() {
    assert_eq!(
        classify_intent("test: add unit tests"),
        CommitIntentKind::Test
    );
    assert_eq!(
        classify_intent("tests: integration"),
        CommitIntentKind::Test
    );
}

#[test]
fn scenario_classify_chore_ci_build_perf_style() {
    assert_eq!(
        classify_intent("chore: bump deps"),
        CommitIntentKind::Chore
    );
    assert_eq!(
        classify_intent("ci: fix pipeline"),
        CommitIntentKind::Ci
    );
    assert_eq!(
        classify_intent("build: update makefile"),
        CommitIntentKind::Build
    );
    assert_eq!(
        classify_intent("perf: optimize query"),
        CommitIntentKind::Perf
    );
    assert_eq!(
        classify_intent("style: fix formatting"),
        CommitIntentKind::Style
    );
}

#[test]
fn scenario_classify_revert_patterns() {
    // Revert with conventional prefix
    assert_eq!(
        classify_intent("revert: undo change"),
        CommitIntentKind::Revert
    );
    // GitHub-style revert
    assert_eq!(
        classify_intent("Revert \"feat: add login\""),
        CommitIntentKind::Revert
    );
    // revert: prefix (colon-first path)
    assert_eq!(
        classify_intent("revert: something"),
        CommitIntentKind::Revert
    );
}

// ============================================================================
// Scenario: classify_intent – keyword heuristic fallback
// ============================================================================

#[test]
fn scenario_classify_keyword_fix() {
    assert_eq!(
        classify_intent("Fix crash on startup"),
        CommitIntentKind::Fix
    );
    assert_eq!(
        classify_intent("resolved a bug in parser"),
        CommitIntentKind::Fix
    );
    assert_eq!(
        classify_intent("apply hotfix for prod"),
        CommitIntentKind::Fix
    );
}

#[test]
fn scenario_classify_keyword_feat() {
    assert_eq!(
        classify_intent("Add user authentication"),
        CommitIntentKind::Feat
    );
    assert_eq!(
        classify_intent("Implement caching layer"),
        CommitIntentKind::Feat
    );
    assert_eq!(
        classify_intent("Introduce new API endpoint"),
        CommitIntentKind::Feat
    );
}

#[test]
fn scenario_classify_keyword_refactor() {
    assert_eq!(
        classify_intent("Refactor database module"),
        CommitIntentKind::Refactor
    );
    assert_eq!(
        classify_intent("Restructure project layout"),
        CommitIntentKind::Refactor
    );
}

#[test]
fn scenario_classify_keyword_docs() {
    assert_eq!(
        classify_intent("Update doc for API"),
        CommitIntentKind::Docs
    );
    assert_eq!(
        classify_intent("Improve readme instructions"),
        CommitIntentKind::Docs
    );
}

#[test]
fn scenario_classify_keyword_perf() {
    assert_eq!(
        classify_intent("Optimize database queries"),
        CommitIntentKind::Perf
    );
    assert_eq!(
        classify_intent("Improve performance of parser"),
        CommitIntentKind::Perf
    );
}

#[test]
fn scenario_classify_keyword_style_lint() {
    assert_eq!(
        classify_intent("Run lint on codebase"),
        CommitIntentKind::Style
    );
    assert_eq!(
        classify_intent("Format code with rustfmt"),
        CommitIntentKind::Style
    );
}

#[test]
fn scenario_classify_keyword_ci_build() {
    assert_eq!(
        classify_intent("Update CI pipeline config"),
        CommitIntentKind::Ci
    );
    assert_eq!(
        classify_intent("Update build scripts"),
        CommitIntentKind::Build
    );
    assert_eq!(
        classify_intent("Bump deps to latest"),
        CommitIntentKind::Build
    );
}

#[test]
fn scenario_classify_keyword_chore() {
    assert_eq!(
        classify_intent("Cleanup unused imports"),
        CommitIntentKind::Chore
    );
}

#[test]
fn scenario_classify_other_for_unrecognised() {
    assert_eq!(
        classify_intent("Initial commit"),
        CommitIntentKind::Other
    );
    assert_eq!(
        classify_intent("WIP save progress"),
        CommitIntentKind::Other
    );
    assert_eq!(classify_intent("v2.0.0"), CommitIntentKind::Other);
}

#[test]
fn scenario_classify_empty_and_whitespace() {
    assert_eq!(classify_intent(""), CommitIntentKind::Other);
    assert_eq!(classify_intent("   "), CommitIntentKind::Other);
    assert_eq!(classify_intent("\t\n"), CommitIntentKind::Other);
}

// ============================================================================
// Scenario: classify_intent – case insensitivity
// ============================================================================

#[test]
fn scenario_classify_case_insensitive_conventional() {
    assert_eq!(classify_intent("FEAT: loud"), CommitIntentKind::Feat);
    assert_eq!(classify_intent("Fix: mixed"), CommitIntentKind::Fix);
    assert_eq!(
        classify_intent("REFACTOR: all caps"),
        CommitIntentKind::Refactor
    );
}

// ============================================================================
// Scenario: classify_intent – word boundary matching
// ============================================================================

#[test]
fn scenario_classify_word_boundary_no_false_positive() {
    // "prefix" contains "fix" but should NOT match as a fix commit
    assert_ne!(
        classify_intent("prefix the module name"),
        CommitIntentKind::Fix,
        "'prefix' should not trigger fix"
    );
    // "testing" starts with "test" — contains_word should still match
    // because the 'ing' after 'test' makes it not a word boundary
    // Actually: "testing" – 'test' at index 0, after_idx=4, char='i' is alphanumeric → no match
    assert_ne!(
        classify_intent("Stop testing in production"),
        CommitIntentKind::Test,
        "'testing' should not trigger test"
    );
}

// ============================================================================
// Scenario: get_added_lines integration
// ============================================================================

#[test]
fn scenario_get_added_lines_multi_hunk_same_file() {
    // Given a file with content modified in two disjoint regions
    let repo = make_repo("multi-hunk").expect("repo");
    let _base = head_sha(&repo.path);

    // Create a file with 10 lines
    let content: String = (1..=10).map(|i| format!("line{}\n", i)).collect();
    std::fs::write(repo.path.join("multi.txt"), &content).unwrap();
    git_in(&repo.path)
        .args(["add", "multi.txt"])
        .output()
        .unwrap();
    git_in(&repo.path)
        .args(["commit", "-m", "add multi.txt"])
        .output()
        .unwrap();

    // Modify lines 2 and 8 (creating two hunks)
    let mut lines: Vec<String> = (1..=10).map(|i| format!("line{}", i)).collect();
    lines[1] = "CHANGED2".to_string();
    lines[7] = "CHANGED8".to_string();
    let new_content = lines.join("\n") + "\n";
    std::fs::write(repo.path.join("multi.txt"), &new_content).unwrap();
    git_in(&repo.path)
        .args(["add", "multi.txt"])
        .output()
        .unwrap();
    git_in(&repo.path)
        .args(["commit", "-m", "modify two regions"])
        .output()
        .unwrap();

    let mid_sha = head_sha(&repo.path);

    // When we get added lines between base+1 and latest
    // We need the SHA after "add multi.txt" as base
    let parent_output = git_in(&repo.path)
        .args(["rev-parse", &format!("{}~1", mid_sha)])
        .output()
        .unwrap();
    let parent_sha = String::from_utf8_lossy(&parent_output.stdout)
        .trim()
        .to_string();

    let result =
        tokmd_git::get_added_lines(&repo.path, &parent_sha, &mid_sha, GitRangeMode::TwoDot)
            .unwrap();

    // Then lines in the modified regions are reported
    let key = PathBuf::from("multi.txt");
    assert!(result.contains_key(&key), "Should contain multi.txt");
    let lines = &result[&key];
    assert!(lines.contains(&2), "Should contain modified line 2");
    assert!(lines.contains(&8), "Should contain modified line 8");
}

#[test]
fn scenario_get_added_lines_three_dot_mode() {
    // Given a repo with a branch diverging from main
    let repo = make_repo("three-dot").expect("repo");
    let base = head_sha(&repo.path);

    // Create a branch and add a file
    git_in(&repo.path)
        .args(["checkout", "-b", "feature"])
        .output()
        .unwrap();
    std::fs::write(repo.path.join("feat.txt"), "feature\n").unwrap();
    git_in(&repo.path)
        .args(["add", "feat.txt"])
        .output()
        .unwrap();
    git_in(&repo.path)
        .args(["commit", "-m", "feature work"])
        .output()
        .unwrap();
    let feat_sha = head_sha(&repo.path);

    // When using ThreeDot mode (symmetric difference)
    let result =
        tokmd_git::get_added_lines(&repo.path, &base, &feat_sha, GitRangeMode::ThreeDot).unwrap();

    // Then the feature file's lines appear
    let key = PathBuf::from("feat.txt");
    assert!(result.contains_key(&key));
    let expected: BTreeSet<usize> = [1].into_iter().collect();
    assert_eq!(result[&key], expected);
}

// ============================================================================
// Scenario: collect_history with subject containing pipe characters
// ============================================================================

#[test]
fn scenario_collect_history_subject_with_pipes() {
    // Given a commit whose subject contains pipe characters
    let repo = make_repo("pipe-subject").expect("repo");
    std::fs::write(repo.path.join("p.txt"), "p").unwrap();
    git_in(&repo.path).args(["add", "."]).output().unwrap();
    git_in(&repo.path)
        .args(["commit", "-m", "fix: handle A | B | C"])
        .output()
        .unwrap();

    let root = repo_root(&repo.path).unwrap();
    // When we collect history
    let commits = collect_history(&root, None, None).unwrap();
    // Then the subject is captured intact (splitn(4, '|') preserves trailing pipes)
    let latest = &commits[0];
    assert_eq!(latest.subject, "fix: handle A | B | C");
}

// ============================================================================
// Scenario: GitCommit Debug trait
// ============================================================================

#[test]
fn scenario_git_commit_is_debug_printable() {
    let commit = GitCommit {
        timestamp: 1700000000,
        author: "dev@example.com".to_string(),
        hash: Some("abc123".to_string()),
        subject: "feat: hello".to_string(),
        files: vec!["src/main.rs".to_string()],
    };
    let debug = format!("{:?}", commit);
    assert!(debug.contains("1700000000"));
    assert!(debug.contains("dev@example.com"));
    assert!(debug.contains("abc123"));
    assert!(debug.contains("feat: hello"));
}

// ============================================================================
// Scenario: GitCommit Clone
// ============================================================================

#[test]
fn scenario_git_commit_clone_is_independent() {
    let original = GitCommit {
        timestamp: 100,
        author: "a@b.com".to_string(),
        hash: Some("deadbeef".to_string()),
        subject: "init".to_string(),
        files: vec!["f.rs".to_string()],
    };
    let mut cloned = original.clone();
    cloned.timestamp = 200;
    cloned.files.push("g.rs".to_string());
    assert_eq!(original.timestamp, 100);
    assert_eq!(original.files.len(), 1);
}
