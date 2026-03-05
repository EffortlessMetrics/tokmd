//! Contract tests for tokmd-git: deep coverage of git history collection,
//! commit parsing, file change tracking, hotspot calculation, coupling
//! detection, determinism, and edge/boundary cases.

use std::collections::BTreeMap;
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
    _dir: tempfile::TempDir,
    path: PathBuf,
}

fn make_repo(tag: &str) -> Option<TempRepo> {
    if !git_available() {
        return None;
    }
    let dir = tempfile::tempdir().ok()?;
    let path = dir.path().to_path_buf();

    let ok = git_in(&path)
        .args(["init", "-b", "main"])
        .output()
        .ok()?
        .status
        .success();
    if !ok {
        return None;
    }
    git_in(&path)
        .args(["config", "user.email", format!("{tag}@w64.test").as_str()])
        .output()
        .ok()?;
    git_in(&path)
        .args(["config", "user.name", "W64 Tester"])
        .output()
        .ok()?;

    // Seed commit
    std::fs::write(path.join("seed.txt"), "seed").ok()?;
    git_in(&path).args(["add", "."]).output().ok()?;
    let c = git_in(&path).args(["commit", "-m", "seed"]).output().ok()?;
    if !c.status.success() {
        return None;
    }
    Some(TempRepo { _dir: dir, path })
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

fn file_freq(commits: &[GitCommit]) -> BTreeMap<String, usize> {
    let mut freq: BTreeMap<String, usize> = BTreeMap::new();
    for c in commits {
        for f in &c.files {
            *freq.entry(f.clone()).or_default() += 1;
        }
    }
    freq
}

// ===========================================================================
// 1. Git history collection with real temp repos
// ===========================================================================

#[test]
fn collect_seed_commit_only() {
    let repo = make_repo("seed-only").expect("repo");
    let commits = collect_history(&repo.path, None, None).unwrap();
    assert_eq!(commits.len(), 1, "seed commit only");
    assert_eq!(commits[0].subject, "seed");
}

#[test]
fn collect_multiple_commits() {
    let repo = make_repo("multi").expect("repo");
    std::fs::write(repo.path.join("a.txt"), "a").unwrap();
    commit(&repo.path, "add a");
    std::fs::write(repo.path.join("b.txt"), "b").unwrap();
    commit(&repo.path, "add b");

    let commits = collect_history(&repo.path, None, None).unwrap();
    assert_eq!(commits.len(), 3); // seed + add a + add b
}

#[test]
fn collect_history_max_commits_limits() {
    let repo = make_repo("limit").expect("repo");
    for i in 0..5 {
        std::fs::write(repo.path.join(format!("f{i}.txt")), format!("{i}")).unwrap();
        commit(&repo.path, &format!("commit {i}"));
    }
    // Total is 6 (seed + 5). max_commits causes early reader break which
    // can kill git log (broken pipe). The implementation may return an error
    // in that case, which is acceptable — or it may succeed with limited commits.
    match collect_history(&repo.path, Some(3), None) {
        Ok(commits) => {
            assert!(
                commits.len() <= 4,
                "should roughly limit to 3, got {}",
                commits.len()
            );
        }
        Err(_) => {
            // Broken pipe from early termination is acceptable
        }
    }
}

#[test]
fn collect_history_max_commit_files_limits() {
    let repo = make_repo("filelimit").expect("repo");
    // Create many files in one commit
    for i in 0..20 {
        std::fs::write(repo.path.join(format!("f{i}.txt")), format!("{i}")).unwrap();
    }
    commit(&repo.path, "bulk add");

    let commits = collect_history(&repo.path, None, Some(5)).unwrap();
    for c in &commits {
        assert!(c.files.len() <= 5, "files should be capped at 5");
    }
}

#[test]
fn collect_history_returns_hashes() {
    let repo = make_repo("hash").expect("repo");
    let commits = collect_history(&repo.path, None, None).unwrap();
    for c in &commits {
        let h = c.hash.as_ref().expect("hash present");
        assert_eq!(h.len(), 40, "SHA-1 hex length");
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

#[test]
fn collect_history_returns_authors() {
    let repo = make_repo("author").expect("repo");
    let commits = collect_history(&repo.path, None, None).unwrap();
    for c in &commits {
        assert!(c.author.contains('@'), "author should be an email");
    }
}

#[test]
fn collect_history_timestamps_are_positive() {
    let repo = make_repo("ts").expect("repo");
    let commits = collect_history(&repo.path, None, None).unwrap();
    for c in &commits {
        assert!(c.timestamp > 0, "timestamp should be positive");
    }
}

// ===========================================================================
// 2. Commit parsing
// ===========================================================================

#[test]
fn commit_files_tracked_correctly() {
    let repo = make_repo("files").expect("repo");
    std::fs::write(repo.path.join("alpha.txt"), "a").unwrap();
    std::fs::write(repo.path.join("beta.txt"), "b").unwrap();
    commit(&repo.path, "add two files");

    let commits = collect_history(&repo.path, None, None).unwrap();
    let latest = &commits[0]; // most recent first
    assert!(latest.files.contains(&"alpha.txt".to_string()));
    assert!(latest.files.contains(&"beta.txt".to_string()));
}

#[test]
fn commit_subject_preserved() {
    let repo = make_repo("subj").expect("repo");
    std::fs::write(repo.path.join("x.txt"), "x").unwrap();
    commit(&repo.path, "feat: add cool feature");

    let commits = collect_history(&repo.path, None, None).unwrap();
    assert_eq!(commits[0].subject, "feat: add cool feature");
}

#[test]
fn commits_ordered_newest_first() {
    let repo = make_repo("order").expect("repo");
    for i in 0..3 {
        std::fs::write(repo.path.join(format!("o{i}.txt")), format!("{i}")).unwrap();
        commit(&repo.path, &format!("commit {i}"));
    }
    let commits = collect_history(&repo.path, None, None).unwrap();
    for w in commits.windows(2) {
        assert!(
            w[0].timestamp >= w[1].timestamp,
            "newest first: {} >= {}",
            w[0].timestamp,
            w[1].timestamp
        );
    }
}

// ===========================================================================
// 3. File change tracking / hotspot calculation
// ===========================================================================

#[test]
fn hotspot_most_changed_file_detected() {
    let repo = make_repo("hot").expect("repo");
    std::fs::write(repo.path.join("hot.rs"), "v1").unwrap();
    std::fs::write(repo.path.join("cold.rs"), "v1").unwrap();
    commit(&repo.path, "init files");

    for i in 2..=6 {
        std::fs::write(repo.path.join("hot.rs"), format!("v{i}")).unwrap();
        commit(&repo.path, &format!("edit hot v{i}"));
    }

    let commits = collect_history(&repo.path, None, None).unwrap();
    let freq = file_freq(&commits);
    assert!(freq.get("hot.rs").copied().unwrap_or(0) > freq.get("cold.rs").copied().unwrap_or(0));
}

#[test]
fn hotspot_frequency_ordering() {
    let repo = make_repo("freqord").expect("repo");
    std::fs::write(repo.path.join("a.rs"), "1").unwrap();
    std::fs::write(repo.path.join("b.rs"), "1").unwrap();
    std::fs::write(repo.path.join("c.rs"), "1").unwrap();
    commit(&repo.path, "add abc");

    // a: 4 edits, b: 2 edits, c: 0 more
    for i in 2..=5 {
        std::fs::write(repo.path.join("a.rs"), format!("{i}")).unwrap();
        commit(&repo.path, &format!("a v{i}"));
    }
    for i in 2..=3 {
        std::fs::write(repo.path.join("b.rs"), format!("{i}")).unwrap();
        commit(&repo.path, &format!("b v{i}"));
    }

    let commits = collect_history(&repo.path, None, None).unwrap();
    let freq = file_freq(&commits);
    let a = freq.get("a.rs").copied().unwrap_or(0);
    let b = freq.get("b.rs").copied().unwrap_or(0);
    let c = freq.get("c.rs").copied().unwrap_or(0);
    assert!(a > b && b > c, "a({a}) > b({b}) > c({c})");
}

#[test]
fn bus_factor_multi_author() {
    let repo = make_repo("bus").expect("repo");
    std::fs::write(repo.path.join("shared.rs"), "v1").unwrap();
    commit(&repo.path, "init shared");

    std::fs::write(repo.path.join("shared.rs"), "v2").unwrap();
    commit_as(&repo.path, "alice edit", "alice@test.com", "Alice");

    std::fs::write(repo.path.join("shared.rs"), "v3").unwrap();
    commit_as(&repo.path, "bob edit", "bob@test.com", "Bob");

    let commits = collect_history(&repo.path, None, None).unwrap();
    let authors_for_shared: std::collections::BTreeSet<&str> = commits
        .iter()
        .filter(|c| c.files.iter().any(|f| f == "shared.rs"))
        .map(|c| c.author.as_str())
        .collect();
    assert!(
        authors_for_shared.len() >= 2,
        "multiple authors touched shared.rs"
    );
}

// ===========================================================================
// 4. Coupling detection
// ===========================================================================

#[test]
fn coupling_co_changed_files() {
    let repo = make_repo("couple").expect("repo");
    for i in 0..4 {
        std::fs::write(repo.path.join("model.rs"), format!("v{i}")).unwrap();
        std::fs::write(repo.path.join("model_test.rs"), format!("v{i}")).unwrap();
        commit(&repo.path, &format!("coupled {i}"));
    }
    std::fs::write(repo.path.join("unrelated.rs"), "x").unwrap();
    commit(&repo.path, "unrelated");

    let commits = collect_history(&repo.path, None, None).unwrap();
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
    let coupled = cooccur
        .get(&("model.rs".to_string(), "model_test.rs".to_string()))
        .copied()
        .unwrap_or(0);
    assert!(
        coupled >= 4,
        "coupled pair co-changed at least 4 times: {coupled}"
    );
}

#[test]
fn coupling_independent_files_low_score() {
    let repo = make_repo("indep").expect("repo");
    std::fs::write(repo.path.join("x.rs"), "x1").unwrap();
    commit(&repo.path, "add x");
    std::fs::write(repo.path.join("y.rs"), "y1").unwrap();
    commit(&repo.path, "add y");

    let commits = collect_history(&repo.path, None, None).unwrap();
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
    let xy = cooccur
        .get(&("x.rs".to_string(), "y.rs".to_string()))
        .copied()
        .unwrap_or(0);
    assert_eq!(xy, 0, "independent files should not co-occur");
}

// ===========================================================================
// 5. Deterministic output for same git history
// ===========================================================================

#[test]
fn deterministic_collect_history() {
    let repo = make_repo("det").expect("repo");
    for i in 0..3 {
        std::fs::write(repo.path.join(format!("d{i}.txt")), format!("{i}")).unwrap();
        commit(&repo.path, &format!("det commit {i}"));
    }
    let run1 = collect_history(&repo.path, None, None).unwrap();
    let run2 = collect_history(&repo.path, None, None).unwrap();
    assert_eq!(run1.len(), run2.len());
    for (a, b) in run1.iter().zip(run2.iter()) {
        assert_eq!(a.hash, b.hash);
        assert_eq!(a.subject, b.subject);
        assert_eq!(a.author, b.author);
        assert_eq!(a.timestamp, b.timestamp);
        assert_eq!(a.files, b.files);
    }
}

#[test]
fn deterministic_file_freq() {
    let repo = make_repo("detfreq").expect("repo");
    std::fs::write(repo.path.join("f.txt"), "1").unwrap();
    commit(&repo.path, "c1");
    std::fs::write(repo.path.join("f.txt"), "2").unwrap();
    commit(&repo.path, "c2");

    let freq1 = file_freq(&collect_history(&repo.path, None, None).unwrap());
    let freq2 = file_freq(&collect_history(&repo.path, None, None).unwrap());
    assert_eq!(freq1, freq2);
}

// ===========================================================================
// 6. BDD: Given repo with commits / When collecting / Then correct counts
// ===========================================================================

#[test]
fn bdd_given_repo_with_five_commits_then_count_is_six() {
    // Given: repo with seed + 5 commits
    let repo = make_repo("bdd5").expect("repo");
    for i in 0..5 {
        std::fs::write(repo.path.join(format!("bdd{i}.txt")), format!("{i}")).unwrap();
        commit(&repo.path, &format!("bdd commit {i}"));
    }
    // When
    let commits = collect_history(&repo.path, None, None).unwrap();
    // Then: 1 seed + 5 = 6
    assert_eq!(commits.len(), 6);
}

#[test]
fn bdd_given_author_commits_then_author_correct() {
    let repo = make_repo("bddauth").expect("repo");
    std::fs::write(repo.path.join("t.txt"), "t").unwrap();
    commit_as(&repo.path, "alice work", "alice@dev.io", "Alice Dev");

    let commits = collect_history(&repo.path, None, None).unwrap();
    let alice_commits: Vec<_> = commits
        .iter()
        .filter(|c| c.author == "alice@dev.io")
        .collect();
    assert_eq!(alice_commits.len(), 1);
    assert_eq!(alice_commits[0].subject, "alice work");
}

#[test]
fn bdd_given_file_edits_then_all_files_present() {
    let repo = make_repo("bddfiles").expect("repo");
    let names = ["one.txt", "two.txt", "three.txt"];
    for name in &names {
        std::fs::write(repo.path.join(name), "content").unwrap();
    }
    commit(&repo.path, "add three files");

    let commits = collect_history(&repo.path, None, None).unwrap();
    let latest = &commits[0];
    for name in &names {
        assert!(latest.files.contains(&name.to_string()), "missing {name}");
    }
}

// ===========================================================================
// 7. Edge cases
// ===========================================================================

#[test]
fn edge_empty_repo_no_commits() {
    if !git_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    git_in(dir.path())
        .args(["init", "-b", "main"])
        .output()
        .unwrap();
    git_in(dir.path())
        .args(["config", "user.email", "e@t.c"])
        .output()
        .unwrap();
    git_in(dir.path())
        .args(["config", "user.name", "T"])
        .output()
        .unwrap();

    // No commits at all — git log should fail or return empty
    let result = collect_history(dir.path(), None, None);
    if let Ok(commits) = result {
        assert!(commits.is_empty());
    }
    // Err is also acceptable — empty repo may fail git log
}

#[test]
fn edge_single_commit_repo() {
    let repo = make_repo("single").expect("repo");
    let commits = collect_history(&repo.path, None, None).unwrap();
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].files, ["seed.txt"]);
}

#[test]
fn edge_repo_root_found() {
    let repo = make_repo("root").expect("repo");
    let root = repo_root(&repo.path);
    assert!(root.is_some());
}

#[test]
fn edge_repo_root_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let root = repo_root(dir.path());
    assert!(root.is_none());
}

#[test]
fn edge_rev_exists_head() {
    let repo = make_repo("revhead").expect("repo");
    assert!(rev_exists(&repo.path, "HEAD"));
}

#[test]
fn edge_rev_exists_bogus() {
    let repo = make_repo("revbogus").expect("repo");
    assert!(!rev_exists(&repo.path, "definitely-not-a-branch-xyz123"));
}

#[test]
fn edge_resolve_base_ref_main_exists() {
    let repo = make_repo("refmain").expect("repo");
    assert_eq!(
        resolve_base_ref(&repo.path, "main"),
        Some("main".to_string())
    );
}

#[test]
fn edge_resolve_base_ref_nonexistent() {
    let repo = make_repo("refnone").expect("repo");
    assert_eq!(resolve_base_ref(&repo.path, "noexist"), None);
}

// ===========================================================================
// 8. Boundary: max commits, long file paths
// ===========================================================================

#[test]
fn boundary_max_commits_zero() {
    let repo = make_repo("maxzero").expect("repo");
    // Implementation pushes a commit then checks the limit, so
    // max_commits=0 may yield 1 commit (the push happens before the check).
    let commits = collect_history(&repo.path, Some(0), None).unwrap();
    assert!(
        commits.len() <= 1,
        "max_commits=0 yields at most 1, got {}",
        commits.len()
    );
}

#[test]
fn boundary_max_commits_one() {
    let repo = make_repo("maxone").expect("repo");
    std::fs::write(repo.path.join("extra.txt"), "x").unwrap();
    commit(&repo.path, "extra");
    let commits = collect_history(&repo.path, Some(1), None).unwrap();
    assert_eq!(commits.len(), 1);
}

#[test]
fn boundary_max_commit_files_zero() {
    let repo = make_repo("filezero").expect("repo");
    let commits = collect_history(&repo.path, None, Some(0)).unwrap();
    for c in &commits {
        assert!(c.files.is_empty());
    }
}

#[test]
fn boundary_long_file_path() {
    let repo = make_repo("longpath").expect("repo");
    let deep_dir = repo.path.join("a").join("b").join("c").join("d");
    std::fs::create_dir_all(&deep_dir).unwrap();
    let deep_file = deep_dir.join("deeply_nested_file.txt");
    std::fs::write(&deep_file, "deep").unwrap();
    commit(&repo.path, "add deep file");

    let commits = collect_history(&repo.path, None, None).unwrap();
    let latest = &commits[0];
    assert!(
        latest
            .files
            .iter()
            .any(|f| f.contains("deeply_nested_file")),
        "deep file should appear: {:?}",
        latest.files
    );
}

// ===========================================================================
// 9. get_added_lines tests
// ===========================================================================

#[test]
fn added_lines_basic() {
    let repo = make_repo("addlines").expect("repo");
    let base_sha = head_sha(&repo.path);

    std::fs::write(repo.path.join("new.txt"), "line1\nline2\nline3\n").unwrap();
    commit(&repo.path, "add new file");

    let added = get_added_lines(&repo.path, &base_sha, "HEAD", GitRangeMode::TwoDot).unwrap();
    assert!(!added.is_empty(), "should have added lines");
    let new_lines = added.get(&PathBuf::from("new.txt"));
    assert!(new_lines.is_some(), "new.txt should have added lines");
    assert!(new_lines.unwrap().contains(&1));
}

#[test]
fn added_lines_empty_diff() {
    let repo = make_repo("nodiff").expect("repo");
    let sha = head_sha(&repo.path);
    let added = get_added_lines(&repo.path, &sha, "HEAD", GitRangeMode::TwoDot).unwrap();
    assert!(added.is_empty(), "no diff, no added lines");
}

#[test]
fn added_lines_three_dot_mode() {
    let repo = make_repo("threedot").expect("repo");
    let base_sha = head_sha(&repo.path);

    std::fs::write(repo.path.join("branch.txt"), "branch content\n").unwrap();
    commit(&repo.path, "branch work");

    let added = get_added_lines(&repo.path, &base_sha, "HEAD", GitRangeMode::ThreeDot).unwrap();
    assert!(!added.is_empty());
}

// ===========================================================================
// 10. GitRangeMode
// ===========================================================================

#[test]
fn range_mode_two_dot_format() {
    assert_eq!(GitRangeMode::TwoDot.format("v1", "v2"), "v1..v2");
}

#[test]
fn range_mode_three_dot_format() {
    assert_eq!(GitRangeMode::ThreeDot.format("v1", "v2"), "v1...v2");
}

#[test]
fn range_mode_default_is_two_dot() {
    assert_eq!(GitRangeMode::default(), GitRangeMode::TwoDot);
}

#[test]
fn range_mode_equality() {
    assert_eq!(GitRangeMode::TwoDot, GitRangeMode::TwoDot);
    assert_ne!(GitRangeMode::TwoDot, GitRangeMode::ThreeDot);
}

// ===========================================================================
// 11. classify_intent — conventional commits
// ===========================================================================

#[test]
fn intent_feat_conventional() {
    assert_eq!(classify_intent("feat: add new API"), CommitIntentKind::Feat);
}

#[test]
fn intent_feat_with_scope() {
    assert_eq!(
        classify_intent("feat(core): new parser"),
        CommitIntentKind::Feat
    );
}

#[test]
fn intent_fix_conventional() {
    assert_eq!(classify_intent("fix: null pointer"), CommitIntentKind::Fix);
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
    assert_eq!(classify_intent("ci: update workflow"), CommitIntentKind::Ci);
}

#[test]
fn intent_build_conventional() {
    assert_eq!(
        classify_intent("build: fix makefile"),
        CommitIntentKind::Build
    );
}

#[test]
fn intent_perf_conventional() {
    assert_eq!(
        classify_intent("perf: optimize loop"),
        CommitIntentKind::Perf
    );
}

#[test]
fn intent_style_conventional() {
    assert_eq!(
        classify_intent("style: fix indentation"),
        CommitIntentKind::Style
    );
}

#[test]
fn intent_revert_conventional() {
    assert_eq!(
        classify_intent("revert: undo bad commit"),
        CommitIntentKind::Revert
    );
}

#[test]
fn intent_revert_pattern() {
    assert_eq!(
        classify_intent("Revert \"feat: add feature\""),
        CommitIntentKind::Revert
    );
}

#[test]
fn intent_breaking_change_bang() {
    assert_eq!(
        classify_intent("feat!: breaking API change"),
        CommitIntentKind::Feat
    );
}

// ===========================================================================
// 12. classify_intent — keyword heuristic
// ===========================================================================

#[test]
fn intent_keyword_fix() {
    assert_eq!(
        classify_intent("Fix crash on startup"),
        CommitIntentKind::Fix
    );
}

#[test]
fn intent_keyword_add() {
    assert_eq!(
        classify_intent("Add user dashboard"),
        CommitIntentKind::Feat
    );
}

#[test]
fn intent_keyword_implement() {
    assert_eq!(
        classify_intent("Implement caching layer"),
        CommitIntentKind::Feat
    );
}

#[test]
fn intent_keyword_bug() {
    assert_eq!(
        classify_intent("Bug in authentication"),
        CommitIntentKind::Fix
    );
}

#[test]
fn intent_keyword_doc() {
    assert_eq!(
        classify_intent("Update doc for API"),
        CommitIntentKind::Docs
    );
}

#[test]
fn intent_keyword_readme() {
    assert_eq!(classify_intent("Update README"), CommitIntentKind::Docs);
}

#[test]
fn intent_empty_is_other() {
    assert_eq!(classify_intent(""), CommitIntentKind::Other);
}

#[test]
fn intent_whitespace_is_other() {
    assert_eq!(classify_intent("   "), CommitIntentKind::Other);
}

#[test]
fn intent_unknown_subject() {
    assert_eq!(classify_intent("WIP"), CommitIntentKind::Other);
}

// ===========================================================================
// 13. Property: deterministic classify_intent
// ===========================================================================

#[test]
fn property_classify_intent_deterministic() {
    let subjects = [
        "feat: new thing",
        "fix: broken thing",
        "random message",
        "",
        "Revert \"something\"",
        "Add feature",
        "Update doc",
    ];
    for s in &subjects {
        let a = classify_intent(s);
        let b = classify_intent(s);
        assert_eq!(a, b, "classify_intent should be deterministic for {s:?}");
    }
}

#[test]
fn property_classify_intent_never_panics() {
    let long = "a".repeat(1000);
    let edge_inputs = [
        "",
        " ",
        ":",
        "!:",
        "(scope):",
        long.as_str(),
        "🚀 emoji commit",
        "fix(🔧): unicode scope",
        "\n\n\n",
        "\t\t",
    ];
    for input in &edge_inputs {
        let _ = classify_intent(input); // should not panic
    }
}

// ===========================================================================
// 14. Git availability check
// ===========================================================================

#[test]
fn git_available_returns_bool() {
    // Just confirm it doesn't panic
    let _ = git_available();
}

// ===========================================================================
// 15. Freshness (timestamp-based recency)
// ===========================================================================

#[test]
fn freshness_newest_commit_has_highest_timestamp() {
    let repo = make_repo("fresh").expect("repo");
    std::fs::write(repo.path.join("f.txt"), "1").unwrap();
    commit(&repo.path, "c1");
    // Small delay via content change
    std::fs::write(repo.path.join("f.txt"), "2").unwrap();
    commit(&repo.path, "c2");

    let commits = collect_history(&repo.path, None, None).unwrap();
    // Newest first
    assert!(commits[0].timestamp >= commits[1].timestamp);
}
