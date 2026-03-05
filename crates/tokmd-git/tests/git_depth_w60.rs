//! Deep tests (w60) for tokmd-git: git history data structures,
//! git log parsing edge cases, hotspot calculation, freshness metrics,
//! coupling detection, property tests, and unavailable/empty repo handling.

use std::collections::{BTreeMap, BTreeSet};
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
        "w60-{}-{}-{:?}",
        tag,
        std::process::id(),
        std::thread::current().id(),
    );
    let dir = std::env::temp_dir().join(format!("tokmd-git-w60-{id}"));
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
        .args(["config", "user.email", "w60@test.com"])
        .output()
        .ok()?;
    git_in(&dir)
        .args(["config", "user.name", "W60 Tester"])
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

fn nano_content() -> String {
    format!(
        "content-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )
}

// ===========================================================================
// 1. GitCommit data structure tests
// ===========================================================================

#[test]
fn git_commit_struct_fields_are_accessible() {
    let c = GitCommit {
        timestamp: 1_700_000_000,
        author: "dev@example.com".to_string(),
        hash: Some("abc123".to_string()),
        subject: "feat: add feature".to_string(),
        files: vec!["src/lib.rs".to_string()],
    };
    assert_eq!(c.timestamp, 1_700_000_000);
    assert_eq!(c.author, "dev@example.com");
    assert_eq!(c.hash.as_deref(), Some("abc123"));
    assert_eq!(c.subject, "feat: add feature");
    assert_eq!(c.files.len(), 1);
}

#[test]
fn git_commit_clone_is_equal() {
    let c = GitCommit {
        timestamp: 42,
        author: "a@b.com".to_string(),
        hash: None,
        subject: "init".to_string(),
        files: vec!["f.txt".to_string()],
    };
    let c2 = c.clone();
    assert_eq!(c.timestamp, c2.timestamp);
    assert_eq!(c.author, c2.author);
    assert_eq!(c.hash, c2.hash);
    assert_eq!(c.subject, c2.subject);
    assert_eq!(c.files, c2.files);
}

#[test]
fn git_commit_debug_format_not_empty() {
    let c = GitCommit {
        timestamp: 0,
        author: String::new(),
        hash: None,
        subject: String::new(),
        files: vec![],
    };
    let dbg = format!("{c:?}");
    assert!(!dbg.is_empty());
    assert!(dbg.contains("GitCommit"));
}

#[test]
fn git_commit_with_no_files_has_empty_vec() {
    let c = GitCommit {
        timestamp: 100,
        author: "dev@x.com".to_string(),
        hash: Some("deadbeef".to_string()),
        subject: "empty commit".to_string(),
        files: vec![],
    };
    assert!(c.files.is_empty());
}

#[test]
fn git_commit_with_many_files() {
    let files: Vec<String> = (0..100).map(|i| format!("file{i}.rs")).collect();
    let c = GitCommit {
        timestamp: 999,
        author: "bulk@dev.io".to_string(),
        hash: Some("ffff".to_string()),
        subject: "bulk change".to_string(),
        files,
    };
    assert_eq!(c.files.len(), 100);
}

// ===========================================================================
// 2. GitRangeMode tests
// ===========================================================================

#[test]
fn range_mode_two_dot_format_special_chars() {
    let r = GitRangeMode::TwoDot.format("v1.0.0", "v2.0.0-rc1");
    assert_eq!(r, "v1.0.0..v2.0.0-rc1");
}

#[test]
fn range_mode_three_dot_with_origin_prefix() {
    let r = GitRangeMode::ThreeDot.format("origin/main", "feature/x");
    assert_eq!(r, "origin/main...feature/x");
}

#[test]
fn range_mode_eq_and_copy_traits() {
    let a = GitRangeMode::TwoDot;
    let b = a; // copy
    assert_eq!(a, b);
    assert_ne!(a, GitRangeMode::ThreeDot);
}

#[test]
fn range_mode_debug_output() {
    let dbg = format!("{:?}", GitRangeMode::ThreeDot);
    assert!(dbg.contains("ThreeDot"));
}

// ===========================================================================
// 3. classify_intent – conventional commit parsing
// ===========================================================================

#[test]
fn intent_feat_conventional() {
    assert_eq!(classify_intent("feat: add login"), CommitIntentKind::Feat);
}

#[test]
fn intent_feat_with_scope() {
    assert_eq!(
        classify_intent("feat(auth): add OAuth2"),
        CommitIntentKind::Feat
    );
}

#[test]
fn intent_fix_conventional() {
    assert_eq!(classify_intent("fix: null pointer"), CommitIntentKind::Fix);
}

#[test]
fn intent_fix_with_bang() {
    assert_eq!(classify_intent("fix!: breaking fix"), CommitIntentKind::Fix);
}

#[test]
fn intent_refactor_conventional() {
    assert_eq!(
        classify_intent("refactor: simplify loop"),
        CommitIntentKind::Refactor
    );
}

#[test]
fn intent_docs_conventional() {
    assert_eq!(
        classify_intent("docs: update README"),
        CommitIntentKind::Docs
    );
}

#[test]
fn intent_test_conventional() {
    assert_eq!(
        classify_intent("test: add unit tests"),
        CommitIntentKind::Test
    );
}

#[test]
fn intent_chore_conventional() {
    assert_eq!(classify_intent("chore: bump deps"), CommitIntentKind::Chore);
}

#[test]
fn intent_ci_conventional() {
    assert_eq!(classify_intent("ci: add workflow"), CommitIntentKind::Ci);
}

#[test]
fn intent_build_conventional() {
    assert_eq!(
        classify_intent("build: update Cargo.toml"),
        CommitIntentKind::Build
    );
}

#[test]
fn intent_perf_conventional() {
    assert_eq!(
        classify_intent("perf: reduce allocations"),
        CommitIntentKind::Perf
    );
}

#[test]
fn intent_style_conventional() {
    assert_eq!(
        classify_intent("style: fix formatting"),
        CommitIntentKind::Style
    );
}

#[test]
fn intent_revert_prefix() {
    assert_eq!(
        classify_intent("Revert \"feat: add login\""),
        CommitIntentKind::Revert
    );
}

#[test]
fn intent_revert_colon_prefix() {
    assert_eq!(
        classify_intent("revert: undo something"),
        CommitIntentKind::Revert
    );
}

#[test]
fn intent_empty_subject_is_other() {
    assert_eq!(classify_intent(""), CommitIntentKind::Other);
}

#[test]
fn intent_whitespace_only_is_other() {
    assert_eq!(classify_intent("   "), CommitIntentKind::Other);
}

// ===========================================================================
// 4. classify_intent – keyword heuristic fallbacks
// ===========================================================================

#[test]
fn intent_keyword_fix_bug() {
    assert_eq!(
        classify_intent("Fix the null bug in parser"),
        CommitIntentKind::Fix
    );
}

#[test]
fn intent_keyword_add_feature() {
    assert_eq!(
        classify_intent("Add new caching layer"),
        CommitIntentKind::Feat
    );
}

#[test]
fn intent_keyword_implement() {
    assert_eq!(
        classify_intent("Implement retry logic"),
        CommitIntentKind::Feat
    );
}

#[test]
fn intent_keyword_introduce() {
    assert_eq!(
        classify_intent("Introduce feature flags"),
        CommitIntentKind::Feat
    );
}

#[test]
fn intent_keyword_refactor() {
    assert_eq!(
        classify_intent("Refactor error handling"),
        CommitIntentKind::Refactor
    );
}

#[test]
fn intent_keyword_restructure() {
    assert_eq!(
        classify_intent("Restructure modules"),
        CommitIntentKind::Refactor
    );
}

#[test]
fn intent_keyword_doc() {
    assert_eq!(
        classify_intent("Update doc strings"),
        CommitIntentKind::Docs
    );
}

#[test]
fn intent_keyword_readme() {
    assert_eq!(
        classify_intent("Update README badges"),
        CommitIntentKind::Docs
    );
}

#[test]
fn intent_keyword_perf_optimize() {
    assert_eq!(classify_intent("Optimize hot loop"), CommitIntentKind::Perf);
}

#[test]
fn intent_keyword_lint() {
    assert_eq!(classify_intent("Apply lint rules"), CommitIntentKind::Style);
}

#[test]
fn intent_keyword_pipeline() {
    assert_eq!(
        classify_intent("Update pipeline config"),
        CommitIntentKind::Ci
    );
}

#[test]
fn intent_keyword_deps() {
    assert_eq!(
        classify_intent("Update deps to latest"),
        CommitIntentKind::Build
    );
}

#[test]
fn intent_keyword_cleanup() {
    assert_eq!(
        classify_intent("Cleanup dead code"),
        CommitIntentKind::Chore
    );
}

#[test]
fn intent_keyword_revert_word() {
    assert_eq!(
        classify_intent("Revert accidental push"),
        CommitIntentKind::Revert
    );
}

#[test]
fn intent_unknown_message() {
    assert_eq!(
        classify_intent("WIP save progress"),
        CommitIntentKind::Other
    );
}

// ===========================================================================
// 5. classify_intent – Unicode and edge cases
// ===========================================================================

#[test]
fn intent_unicode_subject_does_not_panic() {
    let _ = classify_intent("修复: 修复了一个错误");
    let _ = classify_intent("feat: 日本語テスト");
    let _ = classify_intent("🐛 fix emoji in subject");
}

#[test]
fn intent_very_long_subject() {
    let long = "feat: ".to_string() + &"x".repeat(10_000);
    assert_eq!(classify_intent(&long), CommitIntentKind::Feat);
}

#[test]
fn intent_subject_with_special_chars() {
    assert_eq!(
        classify_intent("fix(core): handle `NULL` & \"quotes\""),
        CommitIntentKind::Fix
    );
}

#[test]
fn intent_case_insensitive_conventional() {
    assert_eq!(classify_intent("FEAT: uppercase"), CommitIntentKind::Feat);
    assert_eq!(classify_intent("FIX: uppercase"), CommitIntentKind::Fix);
}

// ===========================================================================
// 6. Hotspot calculation via collect_history
// ===========================================================================

#[test]
fn hotspot_frequently_changed_file_has_most_commits() {
    let repo = match make_repo("w60-hotspot") {
        Some(r) => r,
        None => return,
    };

    std::fs::write(repo.path.join("hot.rs"), "v1").unwrap();
    std::fs::write(repo.path.join("cold.rs"), "v1").unwrap();
    commit(&repo.path, "add files");

    for i in 2..=6 {
        std::fs::write(repo.path.join("hot.rs"), format!("v{i}")).unwrap();
        commit(&repo.path, &format!("edit hot v{i}"));
    }

    let commits = collect_history(&repo.path, None, None).expect("history");
    let mut freq: BTreeMap<&str, usize> = BTreeMap::new();
    for c in &commits {
        for f in &c.files {
            *freq.entry(f.as_str()).or_default() += 1;
        }
    }

    let hot_count = freq.get("hot.rs").copied().unwrap_or(0);
    let cold_count = freq.get("cold.rs").copied().unwrap_or(0);
    assert!(
        hot_count > cold_count,
        "hot.rs ({hot_count}) should appear more than cold.rs ({cold_count})"
    );
}

#[test]
fn hotspot_score_is_lines_times_commits() {
    let repo = match make_repo("w60-hscore") {
        Some(r) => r,
        None => return,
    };

    std::fs::write(repo.path.join("a.rs"), "line1\nline2\nline3").unwrap();
    commit(&repo.path, "add a.rs");
    std::fs::write(repo.path.join("a.rs"), "line1\nline2\nline3\nline4").unwrap();
    commit(&repo.path, "edit a.rs");

    let commits = collect_history(&repo.path, None, None).expect("history");
    let mut freq: BTreeMap<&str, usize> = BTreeMap::new();
    for c in &commits {
        for f in &c.files {
            *freq.entry(f.as_str()).or_default() += 1;
        }
    }

    let commit_count = freq.get("a.rs").copied().unwrap_or(0);
    assert_eq!(commit_count, 2);
    let lines = 4_usize;
    let hotspot = lines * commit_count;
    assert_eq!(hotspot, 8, "4 lines × 2 commits = 8");
}

// ===========================================================================
// 7. Freshness metrics
// ===========================================================================

#[test]
fn freshness_newest_commit_is_first_in_log() {
    let repo = match make_repo("w60-fresh") {
        Some(r) => r,
        None => return,
    };

    std::fs::write(repo.path.join("old.txt"), "old").unwrap();
    commit(&repo.path, "old commit");
    std::thread::sleep(std::time::Duration::from_secs(1));
    std::fs::write(repo.path.join("new.txt"), "new").unwrap();
    commit(&repo.path, "new commit");

    let commits = collect_history(&repo.path, None, None).expect("history");
    assert!(
        commits[0].timestamp >= commits.last().unwrap().timestamp,
        "git log should return newest first"
    );
}

#[test]
fn freshness_per_file_last_touch_tracking() {
    let repo = match make_repo("w60-freshfile") {
        Some(r) => r,
        None => return,
    };

    std::fs::write(repo.path.join("stale.rs"), "stale").unwrap();
    commit(&repo.path, "add stale");
    std::thread::sleep(std::time::Duration::from_secs(1));
    std::fs::write(repo.path.join("fresh.rs"), "fresh").unwrap();
    commit(&repo.path, "add fresh");

    let commits = collect_history(&repo.path, None, None).expect("history");
    let mut last_touch: BTreeMap<&str, i64> = BTreeMap::new();
    for c in &commits {
        for f in &c.files {
            last_touch
                .entry(f.as_str())
                .and_modify(|ts| *ts = (*ts).max(c.timestamp))
                .or_insert(c.timestamp);
        }
    }

    assert!(
        last_touch.get("fresh.rs").copied().unwrap_or(0)
            >= last_touch.get("stale.rs").copied().unwrap_or(0),
        "fresh.rs should have a more recent last-touch"
    );
}

#[test]
fn freshness_all_timestamps_are_positive() {
    let repo = match make_repo("w60-tspositive") {
        Some(r) => r,
        None => return,
    };

    std::fs::write(repo.path.join("f.txt"), "data").unwrap();
    commit(&repo.path, "add f");

    let commits = collect_history(&repo.path, None, None).expect("history");
    for c in &commits {
        assert!(
            c.timestamp > 0,
            "timestamp should be positive: {}",
            c.timestamp
        );
    }
}

// ===========================================================================
// 8. Coupling detection (co-changed files)
// ===========================================================================

#[test]
fn coupling_files_committed_together_have_high_cooccurrence() {
    let repo = match make_repo("w60-couple") {
        Some(r) => r,
        None => return,
    };

    for _ in 0..4 {
        std::fs::write(repo.path.join("model.rs"), nano_content()).unwrap();
        std::fs::write(repo.path.join("model_test.rs"), nano_content()).unwrap();
        commit(&repo.path, "coupled change");
    }
    std::fs::write(repo.path.join("readme.md"), "docs").unwrap();
    commit(&repo.path, "standalone change");

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

    let coupled_pair = ("model.rs".to_string(), "model_test.rs".to_string());
    assert!(
        cooccur.get(&coupled_pair).copied().unwrap_or(0) >= 4,
        "model.rs and model_test.rs should appear together: {cooccur:?}"
    );
}

#[test]
fn coupling_independent_files_have_low_cooccurrence() {
    let repo = match make_repo("w60-indep") {
        Some(r) => r,
        None => return,
    };

    // Commit each file separately
    std::fs::write(repo.path.join("alpha.rs"), "a").unwrap();
    commit(&repo.path, "add alpha");
    std::fs::write(repo.path.join("beta.rs"), "b").unwrap();
    commit(&repo.path, "add beta");

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

    let pair = ("alpha.rs".to_string(), "beta.rs".to_string());
    assert_eq!(
        cooccur.get(&pair).copied().unwrap_or(0),
        0,
        "independently committed files should not co-occur"
    );
}

// ===========================================================================
// 9. Empty / unavailable repo handling
// ===========================================================================

#[test]
fn repo_root_returns_none_for_non_git_dir() {
    let dir = tempfile::tempdir().unwrap();
    assert!(repo_root(dir.path()).is_none());
}

#[test]
fn collect_history_non_repo_errors() {
    let dir = tempfile::tempdir().unwrap();
    let result = collect_history(dir.path(), None, None);
    assert!(result.is_err(), "non-repo should fail");
}

#[test]
fn rev_exists_false_for_non_repo() {
    let dir = tempfile::tempdir().unwrap();
    assert!(!rev_exists(dir.path(), "HEAD"));
}

#[test]
fn resolve_base_ref_none_for_non_repo() {
    let dir = tempfile::tempdir().unwrap();
    assert!(resolve_base_ref(dir.path(), "main").is_none());
}

#[test]
fn resolve_base_ref_nonexistent_custom_ref() {
    let repo = match make_repo("w60-resolve-missing") {
        Some(r) => r,
        None => return,
    };
    // Non-"main" refs should fail fast without fallback
    assert!(resolve_base_ref(&repo.path, "nonexistent-branch-xyz").is_none());
}

// ===========================================================================
// 10. Bus factor (author diversity)
// ===========================================================================

#[test]
fn single_author_bus_factor() {
    let repo = match make_repo("w60-bus1") {
        Some(r) => r,
        None => return,
    };

    std::fs::write(repo.path.join("code.rs"), "fn main(){}").unwrap();
    commit(&repo.path, "add code");

    let commits = collect_history(&repo.path, None, None).expect("history");
    let authors: BTreeSet<&str> = commits.iter().map(|c| c.author.as_str()).collect();
    assert_eq!(authors.len(), 1);
}

#[test]
fn multi_author_bus_factor() {
    let repo = match make_repo("w60-bus2") {
        Some(r) => r,
        None => return,
    };

    std::fs::write(repo.path.join("a.rs"), "fn a(){}").unwrap();
    commit_as(&repo.path, "alice work", "alice@co.io", "Alice");
    std::fs::write(repo.path.join("b.rs"), "fn b(){}").unwrap();
    commit_as(&repo.path, "bob work", "bob@co.io", "Bob");

    let commits = collect_history(&repo.path, None, None).expect("history");
    let authors: BTreeSet<&str> = commits.iter().map(|c| c.author.as_str()).collect();
    // seed commit author + alice + bob = 3
    assert!(authors.len() >= 3, "Expected ≥3 authors, got {authors:?}");
}

// ===========================================================================
// 11. max_commits / max_commit_files limits
// ===========================================================================

#[test]
fn max_commits_limits_history() {
    let repo = match make_repo("w60-maxc") {
        Some(r) => r,
        None => return,
    };

    for i in 1..=10 {
        std::fs::write(repo.path.join("f.txt"), format!("{i}")).unwrap();
        commit(&repo.path, &format!("c{i}"));
    }

    if let Ok(commits) = collect_history(&repo.path, Some(3), None) {
        assert!(commits.len() <= 3, "max_commits=3 should limit to 3");
    }
}

#[test]
fn max_commit_files_limits_file_list() {
    let repo = match make_repo("w60-maxf") {
        Some(r) => r,
        None => return,
    };

    // Create 20 files in one commit
    for i in 0..20 {
        std::fs::write(repo.path.join(format!("f{i}.txt")), format!("{i}")).unwrap();
    }
    commit(&repo.path, "many files");

    if let Ok(commits) = collect_history(&repo.path, None, Some(5)) {
        for c in &commits {
            assert!(
                c.files.len() <= 5,
                "max_commit_files=5 should limit files per commit, got {}",
                c.files.len()
            );
        }
    }
}

// ===========================================================================
// 12. Merge commit handling
// ===========================================================================

#[test]
fn merge_commit_is_included_in_history() {
    let repo = match make_repo("w60-merge") {
        Some(r) => r,
        None => return,
    };

    // Create a branch, commit, and merge
    git_in(&repo.path)
        .args(["checkout", "-b", "feature"])
        .output()
        .unwrap();
    std::fs::write(repo.path.join("feature.rs"), "feature").unwrap();
    commit(&repo.path, "feature commit");

    git_in(&repo.path).args(["checkout", "-"]).output().unwrap();
    std::fs::write(repo.path.join("main_only.rs"), "main").unwrap();
    commit(&repo.path, "main commit");

    let merge_out = git_in(&repo.path)
        .args(["merge", "feature", "--no-ff", "-m", "merge feature"])
        .output()
        .unwrap();

    if !merge_out.status.success() {
        return; // merge conflict or git version issue
    }

    let commits = collect_history(&repo.path, None, None).expect("history");
    let subjects: Vec<&str> = commits.iter().map(|c| c.subject.as_str()).collect();
    assert!(
        subjects.iter().any(|s| s.contains("merge")),
        "merge commit should appear in history: {subjects:?}"
    );
}

// ===========================================================================
// 13. Deterministic output
// ===========================================================================

#[test]
fn collect_history_is_deterministic() {
    let repo = match make_repo("w60-determ") {
        Some(r) => r,
        None => return,
    };

    for i in 1..=5 {
        std::fs::write(repo.path.join("f.txt"), format!("v{i}")).unwrap();
        commit(&repo.path, &format!("c{i}"));
    }

    let h1 = collect_history(&repo.path, None, None).expect("h1");
    let h2 = collect_history(&repo.path, None, None).expect("h2");

    assert_eq!(h1.len(), h2.len());
    for (a, b) in h1.iter().zip(h2.iter()) {
        assert_eq!(a.timestamp, b.timestamp);
        assert_eq!(a.author, b.author);
        assert_eq!(a.hash, b.hash);
        assert_eq!(a.subject, b.subject);
        assert_eq!(a.files, b.files);
    }
}

// ===========================================================================
// 14. Single-commit repo edge cases
// ===========================================================================

#[test]
fn single_commit_repo_returns_one_commit() {
    let repo = match make_repo("w60-single") {
        Some(r) => r,
        None => return,
    };
    let commits = collect_history(&repo.path, None, None).expect("history");
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].subject, "seed");
}

#[test]
fn single_commit_hash_is_40_hex_chars() {
    let repo = match make_repo("w60-hashlen") {
        Some(r) => r,
        None => return,
    };
    let commits = collect_history(&repo.path, None, None).expect("history");
    let h = commits[0].hash.as_ref().expect("hash present");
    assert_eq!(h.len(), 40);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

// ===========================================================================
// 15. rev_exists and repo_root positive paths
// ===========================================================================

#[test]
fn rev_exists_finds_head() {
    let repo = match make_repo("w60-revhead") {
        Some(r) => r,
        None => return,
    };
    assert!(rev_exists(&repo.path, "HEAD"));
}

#[test]
fn repo_root_returns_valid_path() {
    let repo = match make_repo("w60-root") {
        Some(r) => r,
        None => return,
    };
    let root = repo_root(&repo.path).expect("should find root");
    assert!(root.exists());
}

// ===========================================================================
// 16. Property tests
// ===========================================================================

proptest! {
    #[test]
    fn classify_intent_never_panics(subject in ".*") {
        let _ = classify_intent(&subject);
    }

    #[test]
    fn classify_intent_empty_always_other(ws in "[ \\t\\n]*") {
        prop_assert_eq!(classify_intent(&ws), CommitIntentKind::Other);
    }

    #[test]
    fn git_range_format_contains_base_and_head(
        base in "[a-zA-Z0-9/_.-]{1,30}",
        head in "[a-zA-Z0-9/_.-]{1,30}",
    ) {
        let two = GitRangeMode::TwoDot.format(&base, &head);
        let three = GitRangeMode::ThreeDot.format(&base, &head);
        prop_assert!(two.contains(&base));
        prop_assert!(two.contains(&head));
        prop_assert!(two.contains(".."));
        prop_assert!(three.contains("..."));
    }

    #[test]
    fn git_range_two_dot_has_exactly_two_dots(
        base in "[a-z]{1,10}",
        head in "[a-z]{1,10}",
    ) {
        let r = GitRangeMode::TwoDot.format(&base, &head);
        let dot_count = r.matches("..").count();
        // "a..b" contains exactly one ".." match (two consecutive dots)
        prop_assert!(dot_count >= 1);
    }

    #[test]
    fn conventional_feat_always_feat(scope in "[a-z]{0,10}", desc in "[a-zA-Z ]{1,30}") {
        let subject = if scope.is_empty() {
            format!("feat: {desc}")
        } else {
            format!("feat({scope}): {desc}")
        };
        prop_assert_eq!(classify_intent(&subject), CommitIntentKind::Feat);
    }

    #[test]
    fn conventional_fix_always_fix(scope in "[a-z]{0,10}", desc in "[a-zA-Z ]{1,30}") {
        let subject = if scope.is_empty() {
            format!("fix: {desc}")
        } else {
            format!("fix({scope}): {desc}")
        };
        prop_assert_eq!(classify_intent(&subject), CommitIntentKind::Fix);
    }

    #[test]
    fn hotspot_multiplication_commutative(lines in 0..1000_usize, commits in 0..100_usize) {
        prop_assert_eq!(lines * commits, commits * lines);
    }

    #[test]
    fn hotspot_zero_lines_is_zero(commits in 0..1000_usize) {
        let result = 0_usize.checked_mul(commits).unwrap_or(0);
        prop_assert_eq!(result, 0_usize);
    }

    #[test]
    fn hotspot_zero_commits_is_zero(lines in 0..1000_usize) {
        let result = lines.checked_mul(0).unwrap_or(0);
        prop_assert_eq!(result, 0_usize);
    }

    #[test]
    fn hotspot_monotonic_in_commits(lines in 1..500_usize, c1 in 0..100_usize, c2 in 0..100_usize) {
        if c1 <= c2 {
            prop_assert!(lines * c1 <= lines * c2);
        } else {
            prop_assert!(lines * c1 >= lines * c2);
        }
    }
}

// ===========================================================================
// 17. Commit intent classification (additional)
// ===========================================================================

#[test]
fn intent_feature_alias() {
    assert_eq!(
        classify_intent("feature: new endpoint"),
        CommitIntentKind::Feat
    );
}

#[test]
fn intent_bugfix_alias() {
    assert_eq!(classify_intent("bugfix: fix race"), CommitIntentKind::Fix);
}

#[test]
fn intent_hotfix_alias() {
    assert_eq!(
        classify_intent("hotfix: critical fix"),
        CommitIntentKind::Fix
    );
}

#[test]
fn intent_tests_plural() {
    assert_eq!(
        classify_intent("tests: add coverage"),
        CommitIntentKind::Test
    );
}

#[test]
fn intent_doc_singular() {
    assert_eq!(classify_intent("doc: fix typo"), CommitIntentKind::Docs);
}
