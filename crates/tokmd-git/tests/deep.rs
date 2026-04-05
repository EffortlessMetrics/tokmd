//! Deep tests for tokmd-git: history analysis, hotspot detection,
//! coupling patterns, bus factor, freshness, and determinism.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use tokmd_git::{
    CommitIntentKind, GitCommit, GitRangeMode, classify_intent, collect_history, get_added_lines,
    git_available, repo_root, resolve_base_ref, rev_exists,
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

fn head_sha(dir: &Path) -> String {
    let out = git_in(dir)
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("rev-parse");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
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
        "deep-{}-{}-{:?}",
        tag,
        std::process::id(),
        std::thread::current().id(),
    );
    let dir = std::env::temp_dir().join(format!("tokmd-git-deep-{}", id));
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
        .args(["config", "user.email", "deep@test.com"])
        .output()
        .ok()?;
    git_in(&dir)
        .args(["config", "user.name", "Deep Tester"])
        .output()
        .ok()?;

    // Seed commit
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

fn commit_at(dir: &Path, msg: &str, timestamp: u64) {
    git_in(dir).args(["add", "."]).output().unwrap();
    let date = format!("{} +0000", timestamp);
    git_in(dir)
        .env("GIT_AUTHOR_DATE", &date)
        .env("GIT_COMMITTER_DATE", &date)
        .args(["commit", "-m", msg])
        .output()
        .unwrap();
}

fn commit_as(dir: &Path, msg: &str, email: &str, name: &str) {
    git_in(dir).args(["add", "."]).output().unwrap();
    git_in(dir)
        .args([
            "-c",
            &format!("user.email={}", email),
            "-c",
            &format!("user.name={}", name),
            "commit",
            "-m",
            msg,
        ])
        .output()
        .unwrap();
}

// ===========================================================================
// 1. Hotspot identification (files changed most frequently)
// ===========================================================================

#[test]
fn test_hotspot_most_changed_file() {
    let repo = make_repo("hotspot").expect("repo");

    // Create 3 files; modify "hot.txt" in every subsequent commit
    std::fs::write(repo.path.join("hot.txt"), "v1").unwrap();
    std::fs::write(repo.path.join("cold.txt"), "v1").unwrap();
    commit(&repo.path, "add hot and cold");

    for i in 2..=5 {
        std::fs::write(repo.path.join("hot.txt"), format!("v{i}")).unwrap();
        commit(&repo.path, &format!("edit hot v{i}"));
    }

    let commits = collect_history(&repo.path, None, None).expect("history");
    let mut freq: BTreeMap<&str, usize> = BTreeMap::new();
    for c in &commits {
        for f in &c.files {
            *freq.entry(f.as_str()).or_default() += 1;
        }
    }

    assert!(
        freq.get("hot.txt").copied().unwrap_or(0) > freq.get("cold.txt").copied().unwrap_or(0),
        "hot.txt should appear more often than cold.txt: {:?}",
        freq
    );
}

#[test]
fn test_hotspot_change_frequency_order() {
    let repo = make_repo("hotfreq").expect("repo");

    std::fs::write(repo.path.join("a.txt"), "1").unwrap();
    std::fs::write(repo.path.join("b.txt"), "1").unwrap();
    std::fs::write(repo.path.join("c.txt"), "1").unwrap();
    commit(&repo.path, "add abc");

    // a: 3 more edits, b: 1 more edit, c: 0 more edits
    for i in 2..=4 {
        std::fs::write(repo.path.join("a.txt"), format!("{i}")).unwrap();
        commit(&repo.path, &format!("edit a {i}"));
    }
    std::fs::write(repo.path.join("b.txt"), "2").unwrap();
    commit(&repo.path, "edit b");

    let commits = collect_history(&repo.path, None, None).expect("history");
    let mut freq: BTreeMap<&str, usize> = BTreeMap::new();
    for c in &commits {
        for f in &c.files {
            *freq.entry(f.as_str()).or_default() += 1;
        }
    }

    let a = freq.get("a.txt").copied().unwrap_or(0);
    let b = freq.get("b.txt").copied().unwrap_or(0);
    let c = freq.get("c.txt").copied().unwrap_or(0);
    assert!(a > b && b > c, "Expected a({a}) > b({b}) > c({c})");
}

// ===========================================================================
// 2. Coupling detection (files changed together)
// ===========================================================================

#[test]
fn test_coupling_files_changed_together() {
    let repo = make_repo("coupling").expect("repo");

    // Create coupled pair and an independent file
    for _ in 0..3 {
        std::fs::write(repo.path.join("api.rs"), rand_content()).unwrap();
        std::fs::write(repo.path.join("api_test.rs"), rand_content()).unwrap();
        commit(&repo.path, "coupled change");
    }
    std::fs::write(repo.path.join("readme.md"), "docs").unwrap();
    commit(&repo.path, "add readme");

    let commits = collect_history(&repo.path, None, None).expect("history");
    let mut cooccur: BTreeMap<(String, String), usize> = BTreeMap::new();
    for c in &commits {
        let files: Vec<&String> = c.files.iter().collect();
        for i in 0..files.len() {
            for j in (i + 1)..files.len() {
                let pair = if files[i] < files[j] {
                    (files[i].clone(), files[j].clone())
                } else {
                    (files[j].clone(), files[i].clone())
                };
                *cooccur.entry(pair).or_default() += 1;
            }
        }
    }

    let api_pair = ("api.rs".to_string(), "api_test.rs".to_string());
    assert!(
        cooccur.get(&api_pair).copied().unwrap_or(0) >= 3,
        "api.rs and api_test.rs should be coupled: {:?}",
        cooccur
    );
}

fn rand_content() -> String {
    format!(
        "content-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )
}

// ===========================================================================
// 3. Bus factor (number of unique authors)
// ===========================================================================

#[test]
fn test_bus_factor_single_author() {
    let repo = make_repo("bus1").expect("repo");

    std::fs::write(repo.path.join("code.rs"), "fn main(){}").unwrap();
    commit(&repo.path, "add code");

    let commits = collect_history(&repo.path, None, None).expect("history");
    let authors: BTreeSet<&str> = commits.iter().map(|c| c.author.as_str()).collect();
    assert_eq!(authors.len(), 1, "Single author → bus factor 1");
}

#[test]
fn test_bus_factor_multiple_authors() {
    let repo = make_repo("bus2").expect("repo");

    std::fs::write(repo.path.join("a.rs"), "fn a(){}").unwrap();
    commit_as(&repo.path, "alice commit", "alice@test.com", "Alice");

    std::fs::write(repo.path.join("b.rs"), "fn b(){}").unwrap();
    commit_as(&repo.path, "bob commit", "bob@test.com", "Bob");

    std::fs::write(repo.path.join("c.rs"), "fn c(){}").unwrap();
    commit_as(&repo.path, "carol commit", "carol@test.com", "Carol");

    let commits = collect_history(&repo.path, None, None).expect("history");
    let authors: BTreeSet<&str> = commits.iter().map(|c| c.author.as_str()).collect();
    assert!(
        authors.len() >= 3,
        "Expected ≥3 distinct authors, got {:?}",
        authors
    );
}

#[test]
fn test_author_extraction_email() {
    let repo = make_repo("author-email").expect("repo");

    std::fs::write(repo.path.join("x.txt"), "x").unwrap();
    commit_as(&repo.path, "author test", "specific@corp.io", "Specific");

    let commits = collect_history(&repo.path, None, None).expect("history");
    let last = commits.first().expect("at least one commit");
    assert_eq!(last.author, "specific@corp.io");
}

// ===========================================================================
// 4. Code freshness (recency of last change)
// ===========================================================================

#[test]
fn test_freshness_latest_commit_has_greatest_timestamp() {
    let repo = make_repo("fresh").expect("repo");

    std::fs::write(repo.path.join("f.txt"), "1").unwrap();
    commit_at(&repo.path, "c1", 2000000000);

    std::fs::write(repo.path.join("f.txt"), "2").unwrap();
    commit_at(&repo.path, "c2", 2000000010);

    let commits = collect_history(&repo.path, None, None).expect("history");
    // git log returns newest first
    assert!(
        commits[0].timestamp >= commits[1].timestamp,
        "First commit in log should be newest"
    );
}

#[test]
fn test_freshness_per_file() {
    let repo = make_repo("freshfile").expect("repo");

    std::fs::write(repo.path.join("old.txt"), "old").unwrap();
    commit_at(&repo.path, "add old", 2000000000);

    std::fs::write(repo.path.join("new.txt"), "new").unwrap();
    commit_at(&repo.path, "add new", 2000000010);

    let commits = collect_history(&repo.path, None, None).expect("history");
    let mut file_last_touch: BTreeMap<&str, i64> = BTreeMap::new();
    for c in &commits {
        for f in &c.files {
            file_last_touch
                .entry(f.as_str())
                .and_modify(|ts| *ts = (*ts).max(c.timestamp))
                .or_insert(c.timestamp);
        }
    }

    assert!(
        file_last_touch["new.txt"] >= file_last_touch["old.txt"],
        "new.txt should be fresher"
    );
}

// ===========================================================================
// 5. Commit counting
// ===========================================================================

#[test]
fn test_commit_count_exact() {
    let repo = make_repo("count").expect("repo");

    for i in 1..=7 {
        std::fs::write(repo.path.join("f.txt"), format!("{i}")).unwrap();
        commit(&repo.path, &format!("c{i}"));
    }

    let commits = collect_history(&repo.path, None, None).expect("history");
    // seed + 7 = 8
    assert_eq!(commits.len(), 8, "Expected 8 commits (1 seed + 7)");
}

#[test]
fn test_commit_count_with_max() {
    let repo = make_repo("countmax").expect("repo");

    for i in 1..=5 {
        std::fs::write(repo.path.join("f.txt"), format!("{i}")).unwrap();
        commit(&repo.path, &format!("c{i}"));
    }

    // Ask for exactly 3 commits
    let result = collect_history(&repo.path, Some(3), None);
    if let Ok(commits) = result {
        assert!(commits.len() <= 3, "Should respect max_commits=3");
    }
}

// ===========================================================================
// 6. Empty and single-commit repos
// ===========================================================================

#[test]
fn test_single_commit_repo() {
    let repo = make_repo("single").expect("repo");
    // Only the seed commit exists
    let commits = collect_history(&repo.path, None, None).expect("history");
    assert_eq!(commits.len(), 1, "Seed commit only");
    assert_eq!(commits[0].subject, "seed");
}

#[test]
fn test_single_commit_has_hash() {
    let repo = make_repo("hash").expect("repo");
    let commits = collect_history(&repo.path, None, None).expect("history");
    assert!(commits[0].hash.is_some(), "Commit should have a hash");
    let h = commits[0].hash.as_ref().unwrap();
    assert_eq!(h.len(), 40, "SHA-1 hash should be 40 hex chars");
    assert!(
        h.chars().all(|c| c.is_ascii_hexdigit()),
        "Hash should be hex"
    );
}

// ===========================================================================
// 7. Many commits
// ===========================================================================

#[test]
fn test_many_commits() {
    let repo = make_repo("many").expect("repo");

    for i in 1..=20 {
        std::fs::write(repo.path.join("file.txt"), format!("{i}")).unwrap();
        commit(&repo.path, &format!("commit {i}"));
    }

    let commits = collect_history(&repo.path, None, None).expect("history");
    assert_eq!(commits.len(), 21, "seed + 20 = 21 commits");
}

// ===========================================================================
// 8. Merge commits
// ===========================================================================

#[test]
fn test_merge_commit_in_history() {
    let repo = make_repo("merge").expect("repo");

    // Remember the current branch name
    let branch_out = git_in(&repo.path)
        .args(["branch", "--show-current"])
        .output()
        .unwrap();
    let main_branch = String::from_utf8_lossy(&branch_out.stdout)
        .trim()
        .to_string();

    // Create a feature branch
    git_in(&repo.path)
        .args(["checkout", "-b", "feature"])
        .output()
        .unwrap();

    std::fs::write(repo.path.join("feature.txt"), "feature").unwrap();
    commit(&repo.path, "feature commit");

    // Go back to original branch
    git_in(&repo.path)
        .args(["checkout", &main_branch])
        .output()
        .unwrap();

    std::fs::write(repo.path.join("main.txt"), "main work").unwrap();
    commit(&repo.path, "main commit");

    // Use --no-ff to force a merge commit (avoid fast-forward)
    let merge = git_in(&repo.path)
        .args(["merge", "feature", "--no-ff", "--no-edit"])
        .output()
        .unwrap();

    assert!(merge.status.success(), "Merge should succeed");

    let commits = collect_history(&repo.path, None, None).expect("history");
    // We expect at least seed + feature + main = 3 commits.
    // With --no-ff there should also be a merge commit (possibly without files).
    assert!(
        commits.len() >= 3,
        "Expect at least 3 commits, got {}",
        commits.len()
    );

    // Verify both branches' commits are reachable
    let subjects: Vec<&str> = commits.iter().map(|c| c.subject.as_str()).collect();
    assert!(
        subjects.contains(&"feature commit"),
        "Should contain feature commit: {:?}",
        subjects
    );
}

// ===========================================================================
// 9. Deterministic output
// ===========================================================================

#[test]
fn test_deterministic_history_output() {
    let repo = make_repo("determ").expect("repo");

    std::fs::write(repo.path.join("a.txt"), "a").unwrap();
    commit(&repo.path, "add a");
    std::fs::write(repo.path.join("b.txt"), "b").unwrap();
    commit(&repo.path, "add b");

    let run1 = collect_history(&repo.path, None, None).expect("run1");
    let run2 = collect_history(&repo.path, None, None).expect("run2");

    assert_eq!(run1.len(), run2.len(), "Same number of commits");
    for (c1, c2) in run1.iter().zip(run2.iter()) {
        assert_eq!(c1.timestamp, c2.timestamp);
        assert_eq!(c1.author, c2.author);
        assert_eq!(c1.hash, c2.hash);
        assert_eq!(c1.subject, c2.subject);
        assert_eq!(c1.files, c2.files);
    }
}

#[test]
fn test_deterministic_added_lines() {
    let repo = make_repo("determ-lines").expect("repo");
    let base = head_sha(&repo.path);

    std::fs::write(repo.path.join("x.txt"), "line1\nline2\nline3\n").unwrap();
    commit(&repo.path, "add x");
    let head = head_sha(&repo.path);

    let r1 = get_added_lines(&repo.path, &base, &head, GitRangeMode::TwoDot).expect("r1");
    let r2 = get_added_lines(&repo.path, &base, &head, GitRangeMode::TwoDot).expect("r2");
    assert_eq!(r1, r2, "get_added_lines should be deterministic");
}

// ===========================================================================
// 10. Path filtering via get_added_lines
// ===========================================================================

#[test]
fn test_added_lines_only_shows_changed_files() {
    let repo = make_repo("pathfilter").expect("repo");

    std::fs::write(repo.path.join("changed.txt"), "yes\n").unwrap();
    std::fs::write(repo.path.join("unchanged.txt"), "static\n").unwrap();
    commit(&repo.path, "add files");
    let mid = head_sha(&repo.path);

    // Only change changed.txt
    std::fs::write(repo.path.join("changed.txt"), "yes updated\n").unwrap();
    commit(&repo.path, "update changed");
    let head = head_sha(&repo.path);

    let result = get_added_lines(&repo.path, &mid, &head, GitRangeMode::TwoDot).expect("diff");
    assert!(
        result.contains_key(&PathBuf::from("changed.txt")),
        "changed.txt should appear"
    );
    assert!(
        !result.contains_key(&PathBuf::from("unchanged.txt")),
        "unchanged.txt should not appear"
    );
}

// ===========================================================================
// 11. Classify intent — exhaustive conventional types
// ===========================================================================

#[test]
fn test_classify_all_conventional_types() {
    let cases = vec![
        ("feat: add login", CommitIntentKind::Feat),
        ("feature: add login", CommitIntentKind::Feat),
        ("fix: null pointer", CommitIntentKind::Fix),
        ("bugfix: handle edge case", CommitIntentKind::Fix),
        ("hotfix: critical patch", CommitIntentKind::Fix),
        ("refactor: extract method", CommitIntentKind::Refactor),
        ("docs: update readme", CommitIntentKind::Docs),
        ("doc: api reference", CommitIntentKind::Docs),
        ("test: add unit tests", CommitIntentKind::Test),
        ("tests: add integration tests", CommitIntentKind::Test),
        ("chore: bump version", CommitIntentKind::Chore),
        ("ci: update workflow", CommitIntentKind::Ci),
        ("build: update deps", CommitIntentKind::Build),
        ("perf: optimize query", CommitIntentKind::Perf),
        ("style: format code", CommitIntentKind::Style),
        ("revert: undo change", CommitIntentKind::Revert),
    ];

    for (subject, expected) in cases {
        assert_eq!(classify_intent(subject), expected, "Failed for: {subject}");
    }
}

#[test]
fn test_classify_conventional_with_scope() {
    assert_eq!(
        classify_intent("feat(auth): add OAuth"),
        CommitIntentKind::Feat
    );
    assert_eq!(
        classify_intent("fix(parser): handle empty input"),
        CommitIntentKind::Fix
    );
    assert_eq!(
        classify_intent("refactor(core): simplify loop"),
        CommitIntentKind::Refactor
    );
}

#[test]
fn test_classify_conventional_breaking() {
    assert_eq!(
        classify_intent("feat!: breaking change"),
        CommitIntentKind::Feat
    );
    assert_eq!(
        classify_intent("fix(api)!: remove deprecated"),
        CommitIntentKind::Fix
    );
}

#[test]
fn test_classify_empty_string() {
    assert_eq!(classify_intent(""), CommitIntentKind::Other);
}

#[test]
fn test_classify_whitespace_only() {
    assert_eq!(classify_intent("   "), CommitIntentKind::Other);
}

#[test]
fn test_classify_unknown_conventional_falls_to_keyword() {
    // "release:" is not a known conventional type, so falls through to keyword
    assert_eq!(classify_intent("release: v1.0.0"), CommitIntentKind::Other);
}

#[test]
fn test_classify_keyword_heuristic_all() {
    let cases = vec![
        ("Fix null pointer bug", CommitIntentKind::Fix),
        ("Bug in parser", CommitIntentKind::Fix),
        ("Patch for edge case", CommitIntentKind::Fix),
        ("Add new feature", CommitIntentKind::Feat),
        ("Implement caching", CommitIntentKind::Feat),
        ("Introduce validation", CommitIntentKind::Feat),
        ("Refactor database layer", CommitIntentKind::Refactor),
        ("Update doc strings", CommitIntentKind::Docs),
        ("Update the readme", CommitIntentKind::Docs),
        ("Write test coverage report", CommitIntentKind::Test),
        ("Optimize performance", CommitIntentKind::Perf),
        ("Style improvements", CommitIntentKind::Style),
        ("Format with prettier", CommitIntentKind::Style),
        ("Lint fixes", CommitIntentKind::Style),
        ("Update CI pipeline", CommitIntentKind::Ci),
        ("Build system changes", CommitIntentKind::Build),
        ("Update deps", CommitIntentKind::Build),
        ("Chore: cleanup temp files", CommitIntentKind::Chore),
        ("Cleanup unused imports", CommitIntentKind::Chore),
    ];

    for (subject, expected) in cases {
        assert_eq!(
            classify_intent(subject),
            expected,
            "Failed keyword heuristic for: {subject}"
        );
    }
}

#[test]
fn test_classify_revert_patterns() {
    assert_eq!(
        classify_intent("Revert \"feat: add login\""),
        CommitIntentKind::Revert
    );
    assert_eq!(
        classify_intent("revert: undo last change"),
        CommitIntentKind::Revert
    );
}

#[test]
fn test_classify_case_insensitive_conventional() {
    assert_eq!(classify_intent("FEAT: shout"), CommitIntentKind::Feat);
    assert_eq!(classify_intent("FIX: loud fix"), CommitIntentKind::Fix);
}

// ===========================================================================
// 12. GitCommit struct properties
// ===========================================================================

#[test]
fn test_git_commit_clone() {
    let c = GitCommit {
        timestamp: 1000,
        author: "test@test.com".to_string(),
        hash: Some("abc123".to_string()),
        subject: "hello".to_string(),
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
fn test_git_commit_debug() {
    let c = GitCommit {
        timestamp: 42,
        author: "a@b.com".to_string(),
        hash: None,
        subject: "test".to_string(),
        files: vec![],
    };
    let debug = format!("{:?}", c);
    assert!(debug.contains("42"));
    assert!(debug.contains("a@b.com"));
}

// ===========================================================================
// 13. GitRangeMode properties
// ===========================================================================

#[test]
fn test_range_mode_format_two_dot() {
    assert_eq!(GitRangeMode::TwoDot.format("a", "b"), "a..b");
}

#[test]
fn test_range_mode_format_three_dot() {
    assert_eq!(GitRangeMode::ThreeDot.format("a", "b"), "a...b");
}

#[test]
fn test_range_mode_default() {
    assert_eq!(GitRangeMode::default(), GitRangeMode::TwoDot);
}

#[test]
fn test_range_mode_eq() {
    assert_eq!(GitRangeMode::TwoDot, GitRangeMode::TwoDot);
    assert_ne!(GitRangeMode::TwoDot, GitRangeMode::ThreeDot);
}

#[test]
fn test_range_mode_copy() {
    let m = GitRangeMode::ThreeDot;
    let m2 = m; // Copy
    assert_eq!(m, m2);
}

// ===========================================================================
// 14. Subject line with pipes (delimiter boundary)
// ===========================================================================

#[test]
fn test_subject_with_pipe_chars() {
    let repo = make_repo("pipe").expect("repo");

    std::fs::write(repo.path.join("p.txt"), "pipes").unwrap();
    commit(&repo.path, "fix: handle A|B|C pattern");

    let commits = collect_history(&repo.path, None, None).expect("history");
    let latest = &commits[0];
    assert!(
        latest.subject.contains("|"),
        "Subject should preserve pipe characters: {}",
        latest.subject
    );
}

// ===========================================================================
// 15. Nested directory paths in commits
// ===========================================================================

#[test]
fn test_nested_paths_in_commit_files() {
    let repo = make_repo("nested").expect("repo");

    let deep = repo.path.join("src").join("core").join("util");
    std::fs::create_dir_all(&deep).unwrap();
    std::fs::write(deep.join("helpers.rs"), "fn help(){}").unwrap();
    commit(&repo.path, "add nested file");

    let commits = collect_history(&repo.path, None, None).expect("history");
    let latest = &commits[0];
    let has_nested = latest.files.iter().any(|f| {
        f.contains("src/core/util/helpers.rs") || f.contains("src\\core\\util\\helpers.rs")
    });
    assert!(has_nested, "Should have nested path: {:?}", latest.files);
}

// ===========================================================================
// 16. repo_root and rev_exists
// ===========================================================================

#[test]
fn test_repo_root_valid() {
    let repo = make_repo("root").expect("repo");
    let root = repo_root(&repo.path);
    assert!(root.is_some(), "Should find root");
}

#[test]
fn test_rev_exists_head() {
    let repo = make_repo("revhead").expect("repo");
    assert!(rev_exists(&repo.path, "HEAD"));
}

#[test]
fn test_rev_exists_nonexistent() {
    let repo = make_repo("revno").expect("repo");
    assert!(!rev_exists(&repo.path, "nonexistent-branch-xyz"));
}

#[test]
fn test_rev_exists_specific_sha() {
    let repo = make_repo("revsha").expect("repo");
    let sha = head_sha(&repo.path);
    assert!(rev_exists(&repo.path, &sha));
}

// ===========================================================================
// 17. resolve_base_ref
// ===========================================================================

#[test]
fn test_resolve_base_ref_head() {
    let repo = make_repo("resolve-head").expect("repo");
    assert_eq!(
        resolve_base_ref(&repo.path, "HEAD"),
        Some("HEAD".to_string())
    );
}

#[test]
fn test_resolve_base_ref_explicit_nonexistent_returns_none() {
    let repo = make_repo("resolve-noref").expect("repo");
    // Non-"main" ref that doesn't exist should return None immediately
    assert_eq!(resolve_base_ref(&repo.path, "does-not-exist"), None);
}

// ===========================================================================
// 18. get_added_lines with three-dot mode
// ===========================================================================

#[test]
fn test_added_lines_three_dot_mode() {
    let repo = make_repo("threedot").expect("repo");
    let base = head_sha(&repo.path);

    std::fs::write(repo.path.join("td.txt"), "three dot\n").unwrap();
    commit(&repo.path, "three dot commit");
    let head = head_sha(&repo.path);

    let result =
        get_added_lines(&repo.path, &base, &head, GitRangeMode::ThreeDot).expect("three dot diff");
    assert!(
        result.contains_key(&PathBuf::from("td.txt")),
        "Should find td.txt in three-dot diff"
    );
}

// ===========================================================================
// 19. Multi-hunk changes
// ===========================================================================

#[test]
fn test_added_lines_multi_hunk() {
    let repo = make_repo("multihunk").expect("repo");

    // Create a file with several lines
    let content: String = (1..=20).map(|i| format!("line{i}\n")).collect();
    std::fs::write(repo.path.join("big.txt"), &content).unwrap();
    commit(&repo.path, "add big file");
    let base = head_sha(&repo.path);

    // Modify lines at the beginning and end to create multiple hunks
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    lines[0] = "CHANGED1".to_string();
    lines[19] = "CHANGED20".to_string();
    std::fs::write(repo.path.join("big.txt"), lines.join("\n") + "\n").unwrap();
    commit(&repo.path, "multi-hunk change");
    let head = head_sha(&repo.path);

    let result = get_added_lines(&repo.path, &base, &head, GitRangeMode::TwoDot).expect("diff");
    let big_lines = &result[&PathBuf::from("big.txt")];
    assert!(big_lines.contains(&1), "Should have line 1 changed");
    assert!(big_lines.contains(&20), "Should have line 20 changed");
}

// ===========================================================================
// 20. Commit subjects with special characters
// ===========================================================================

#[test]
fn test_commit_subject_special_chars() {
    let repo = make_repo("special").expect("repo");

    let subjects = [
        "fix: handle <html> tags",
        "feat: support \"quoted\" strings",
        "chore: update 100% of deps",
    ];

    for s in &subjects {
        std::fs::write(repo.path.join("s.txt"), *s).unwrap();
        commit(&repo.path, s);
    }

    let commits = collect_history(&repo.path, None, None).expect("history");
    // Just verify we can collect without errors and subjects are non-empty
    for c in &commits {
        assert!(!c.subject.is_empty(), "Subject should not be empty");
    }
}

// ===========================================================================
// 21. History with deleted files
// ===========================================================================

#[test]
fn test_history_includes_deleted_file_commits() {
    let repo = make_repo("delete").expect("repo");

    std::fs::write(repo.path.join("temp.txt"), "temporary").unwrap();
    commit(&repo.path, "add temp");

    git_in(&repo.path)
        .args(["rm", "temp.txt"])
        .output()
        .unwrap();
    commit(&repo.path, "remove temp");

    let commits = collect_history(&repo.path, None, None).expect("history");
    let has_temp = commits
        .iter()
        .any(|c| c.files.iter().any(|f| f == "temp.txt"));
    assert!(has_temp, "Should reference temp.txt in history");
}

// ===========================================================================
// 22. Timestamps are positive for all commits
// ===========================================================================

#[test]
fn test_all_timestamps_positive() {
    let repo = make_repo("timestamps").expect("repo");

    for i in 1..=3 {
        std::fs::write(repo.path.join("f.txt"), format!("{i}")).unwrap();
        commit(&repo.path, &format!("c{i}"));
    }

    let commits = collect_history(&repo.path, None, None).expect("history");
    for c in &commits {
        assert!(
            c.timestamp > 0,
            "Timestamp should be positive: {}",
            c.timestamp
        );
    }
}

// ===========================================================================
// 23. All commits have non-empty author
// ===========================================================================

#[test]
fn test_all_authors_nonempty() {
    let repo = make_repo("authors").expect("repo");

    std::fs::write(repo.path.join("f.txt"), "1").unwrap();
    commit_as(&repo.path, "alice", "alice@co.com", "Alice");
    std::fs::write(repo.path.join("f.txt"), "2").unwrap();
    commit_as(&repo.path, "bob", "bob@co.com", "Bob");

    let commits = collect_history(&repo.path, None, None).expect("history");
    for c in &commits {
        assert!(!c.author.is_empty(), "Author should not be empty");
    }
}

// ===========================================================================
// 24. Hashes are unique
// ===========================================================================

#[test]
fn test_commit_hashes_unique() {
    let repo = make_repo("uniq-hash").expect("repo");

    for i in 1..=5 {
        std::fs::write(repo.path.join("f.txt"), format!("{i}")).unwrap();
        commit(&repo.path, &format!("c{i}"));
    }

    let commits = collect_history(&repo.path, None, None).expect("history");
    let hashes: Vec<&str> = commits.iter().filter_map(|c| c.hash.as_deref()).collect();
    let unique: BTreeSet<&str> = hashes.iter().copied().collect();
    assert_eq!(hashes.len(), unique.len(), "All hashes should be unique");
}

// ===========================================================================
// 25. git_available
// ===========================================================================

#[test]
fn test_git_available() {
    assert!(git_available(), "Git should be available in test env");
}

// ===========================================================================
// 26. History ordering (reverse chronological)
// ===========================================================================

#[test]
fn test_history_reverse_chronological() {
    let repo = make_repo("order").expect("repo");

    for i in 1..=5 {
        std::fs::write(repo.path.join("f.txt"), format!("{i}")).unwrap();
        commit_at(&repo.path, &format!("c{i}"), 2000000000 + (i * 10));
    }

    let commits = collect_history(&repo.path, None, None).expect("history");

    for w in commits.windows(2) {
        assert!(
            w[0].timestamp >= w[1].timestamp,
            "Commits should be in reverse chronological order"
        );
    }
}

// ===========================================================================
// 27. max_commit_files with multi-file commit
// ===========================================================================

#[test]
fn test_max_commit_files_truncation() {
    let repo = make_repo("maxfiles").expect("repo");

    for i in 1..=10 {
        std::fs::write(repo.path.join(format!("f{i}.txt")), format!("{i}")).unwrap();
    }
    commit(&repo.path, "ten files");

    let commits = collect_history(&repo.path, None, Some(3)).expect("history");
    let latest = &commits[0];
    assert!(
        latest.files.len() <= 3,
        "Should truncate to ≤3 files, got {}",
        latest.files.len()
    );
}

// ===========================================================================
// 28. Classify intent priority: revert > fix > feat
// ===========================================================================

#[test]
fn test_classify_priority_revert_over_fix() {
    // "Revert" at start should be revert even with "fix" in body
    assert_eq!(
        classify_intent("Revert \"fix: something\""),
        CommitIntentKind::Revert
    );
}

#[test]
fn test_classify_priority_fix_over_feat() {
    // Keyword "fix" should win over "add" when "fix" appears as word
    assert_eq!(
        classify_intent("Fix add button crash"),
        CommitIntentKind::Fix
    );
}

// ===========================================================================
// 29. Word boundary matching
// ===========================================================================

#[test]
fn test_classify_no_false_positive_prefix() {
    // "documentation" contains "doc" as a substring, but after "doc" comes 'u'
    // which is alphanumeric → word boundary check fails → no match
    let result = classify_intent("Update documentation site");
    assert_eq!(result, CommitIntentKind::Other);
}

#[test]
fn test_classify_no_false_positive_suffix() {
    // "perform" contains "perf" as prefix but not at word boundary end
    let result = classify_intent("Perform the migration");
    // "perf" at start of "perform" — after "perf" comes 'o' which is alphanumeric
    // so after_ok = false → should NOT match
    assert_eq!(result, CommitIntentKind::Other);
}

// ===========================================================================
// 30. Integration: classify real git history subjects
// ===========================================================================

#[test]
fn test_classify_real_commit_subjects() {
    let repo = make_repo("classify-real").expect("repo");

    let subjects_and_expected = vec![
        ("feat: add user auth", CommitIntentKind::Feat),
        ("fix: resolve memory leak", CommitIntentKind::Fix),
        ("chore: bump dependencies", CommitIntentKind::Chore),
        ("docs: update API guide", CommitIntentKind::Docs),
        ("test: add regression test", CommitIntentKind::Test),
    ];

    for (subject, _) in &subjects_and_expected {
        std::fs::write(repo.path.join("f.txt"), *subject).unwrap();
        commit(&repo.path, subject);
    }

    let commits = collect_history(&repo.path, None, None).expect("history");
    for c in &commits {
        let intent = classify_intent(&c.subject);
        // Just verify classification doesn't panic and produces valid output
        assert!(
            matches!(
                intent,
                CommitIntentKind::Feat
                    | CommitIntentKind::Fix
                    | CommitIntentKind::Refactor
                    | CommitIntentKind::Docs
                    | CommitIntentKind::Test
                    | CommitIntentKind::Chore
                    | CommitIntentKind::Ci
                    | CommitIntentKind::Build
                    | CommitIntentKind::Perf
                    | CommitIntentKind::Style
                    | CommitIntentKind::Revert
                    | CommitIntentKind::Other
            ),
            "Should classify to a valid kind"
        );
    }
}

// ===========================================================================
// 31. Added lines: empty file yields no lines
// ===========================================================================

#[test]
fn test_added_lines_empty_file() {
    let repo = make_repo("emptyfile").expect("repo");
    let base = head_sha(&repo.path);

    std::fs::write(repo.path.join("empty.txt"), "").unwrap();
    commit(&repo.path, "add empty file");
    let head = head_sha(&repo.path);

    let result = get_added_lines(&repo.path, &base, &head, GitRangeMode::TwoDot).expect("diff");
    // Empty file has no added lines
    if let Some(lines) = result.get(&PathBuf::from("empty.txt")) {
        assert!(lines.is_empty(), "Empty file should have no added lines");
    }
}

// ===========================================================================
// 32. BTreeMap ordering of get_added_lines keys
// ===========================================================================

#[test]
fn test_added_lines_btreemap_ordered() {
    let repo = make_repo("btree").expect("repo");
    let base = head_sha(&repo.path);

    std::fs::write(repo.path.join("z.txt"), "z\n").unwrap();
    std::fs::write(repo.path.join("a.txt"), "a\n").unwrap();
    std::fs::write(repo.path.join("m.txt"), "m\n").unwrap();
    commit(&repo.path, "add files");
    let head = head_sha(&repo.path);

    let result = get_added_lines(&repo.path, &base, &head, GitRangeMode::TwoDot).expect("diff");
    let keys: Vec<&PathBuf> = result.keys().collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(keys, sorted, "BTreeMap keys should be sorted");
}
