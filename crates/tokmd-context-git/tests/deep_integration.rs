#![cfg(feature = "git")]
//! Deep integration tests exercising `compute_git_scores` end-to-end.
//!
//! Covers: multi-file repos, subdirectories, scoring correctness,
//! limit parameters, determinism, ordering, and structural invariants.

#[cfg(feature = "git")]
mod with_git {
    use std::collections::BTreeMap;
    use std::process::Command;
    use tokmd_context_git::{GitScores, compute_git_scores};
    use tokmd_types::{FileKind, FileRow};

    // ── helpers ─────────────────────────────────────────────────────

    fn make_row(path: &str, lines: usize) -> FileRow {
        FileRow {
            path: path.to_string(),
            module: "(root)".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: lines,
            comments: 0,
            blanks: 0,
            lines,
            bytes: lines * 10,
            tokens: lines * 5,
        }
    }

    fn make_child_row(path: &str, lines: usize) -> FileRow {
        FileRow {
            path: path.to_string(),
            module: "(root)".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Child,
            code: lines,
            comments: 0,
            blanks: 0,
            lines,
            bytes: lines * 10,
            tokens: lines * 5,
        }
    }

    fn git(root: &std::path::Path, args: &[&str]) -> Option<()> {
        let out = Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .ok()?;
        if out.status.success() { Some(()) } else { None }
    }

    /// Repo with:
    ///   main.rs – 2 commits (3 then 4 lines)
    ///   lib.rs  – 1 commit (5 lines)
    ///   util.rs – 1 commit (2 lines, same commit as lib.rs)
    fn create_standard_repo() -> Option<tempfile::TempDir> {
        let dir = tempfile::tempdir().ok()?;
        let root = dir.path();

        git(root, &["init"])?;
        git(root, &["config", "user.email", "test@test.com"])?;
        git(root, &["config", "user.name", "Test"])?;

        std::fs::write(root.join("main.rs"), "1\n2\n3").ok()?;
        git(root, &["add", "."])?;
        git(root, &["commit", "-m", "c1"])?;

        std::fs::write(root.join("main.rs"), "1\n2\n3\n4").ok()?;
        git(root, &["add", "."])?;
        git(root, &["commit", "-m", "c2"])?;

        std::fs::write(root.join("lib.rs"), "1\n2\n3\n4\n5").ok()?;
        std::fs::write(root.join("util.rs"), "1\n2").ok()?;
        git(root, &["add", "."])?;
        git(root, &["commit", "-m", "c3"])?;

        Some(dir)
    }

    /// Repo with files in subdirectories
    fn create_nested_repo() -> Option<tempfile::TempDir> {
        let dir = tempfile::tempdir().ok()?;
        let root = dir.path();

        git(root, &["init"])?;
        git(root, &["config", "user.email", "test@test.com"])?;
        git(root, &["config", "user.name", "Test"])?;

        std::fs::create_dir_all(root.join("src")).ok()?;
        std::fs::create_dir_all(root.join("tests")).ok()?;

        std::fs::write(root.join("src/main.rs"), "fn main() {}\n").ok()?;
        std::fs::write(root.join("src/lib.rs"), "pub mod utils;\n").ok()?;
        git(root, &["add", "."])?;
        git(root, &["commit", "-m", "init src"])?;

        std::fs::write(root.join("tests/test.rs"), "#[test]\nfn it_works() {}\n").ok()?;
        git(root, &["add", "."])?;
        git(root, &["commit", "-m", "add tests"])?;

        // Modify src/main.rs again
        std::fs::write(
            root.join("src/main.rs"),
            "fn main() {\n    println!(\"hi\");\n}\n",
        )
        .ok()?;
        git(root, &["add", "."])?;
        git(root, &["commit", "-m", "update main"])?;

        Some(dir)
    }

    /// Repo with many commits to a single file
    fn create_many_commits_repo() -> Option<tempfile::TempDir> {
        let dir = tempfile::tempdir().ok()?;
        let root = dir.path();

        git(root, &["init"])?;
        git(root, &["config", "user.email", "test@test.com"])?;
        git(root, &["config", "user.name", "Test"])?;

        for i in 1..=5 {
            let content: String = (1..=i).map(|n| format!("line {n}\n")).collect();
            std::fs::write(root.join("hot.rs"), &content).ok()?;
            git(root, &["add", "."])?;
            git(root, &["commit", "-m", &format!("commit {i}")])?;
        }

        // One commit for a cold file
        std::fs::write(root.join("cold.rs"), "stable\n").ok()?;
        git(root, &["add", "."])?;
        git(root, &["commit", "-m", "add cold"])?;

        Some(dir)
    }

    // ── 1. Basic scoring correctness ────────────────────────────────

    #[test]
    fn standard_repo_commit_counts() {
        let repo = match create_standard_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![
            make_row("main.rs", 4),
            make_row("lib.rs", 5),
            make_row("util.rs", 2),
        ];
        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };
        assert_eq!(scores.commit_counts.get("main.rs"), Some(&2));
        assert_eq!(scores.commit_counts.get("lib.rs"), Some(&1));
        assert_eq!(scores.commit_counts.get("util.rs"), Some(&1));
    }

    #[test]
    fn standard_repo_hotspots() {
        let repo = match create_standard_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![
            make_row("main.rs", 4),
            make_row("lib.rs", 5),
            make_row("util.rs", 2),
        ];
        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };
        assert_eq!(scores.hotspots.get("main.rs"), Some(&8)); // 4*2
        assert_eq!(scores.hotspots.get("lib.rs"), Some(&5)); // 5*1
        assert_eq!(scores.hotspots.get("util.rs"), Some(&2)); // 2*1
    }

    #[test]
    fn standard_repo_hotspot_ranking() {
        let repo = match create_standard_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![
            make_row("main.rs", 4),
            make_row("lib.rs", 5),
            make_row("util.rs", 2),
        ];
        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };
        assert!(scores.hotspots["main.rs"] > scores.hotspots["lib.rs"]);
        assert!(scores.hotspots["lib.rs"] > scores.hotspots["util.rs"]);
    }

    // ── 2. Nested directory paths ───────────────────────────────────

    #[test]
    fn nested_repo_subdirectory_paths() {
        let repo = match create_nested_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![
            make_row("src/main.rs", 3),
            make_row("src/lib.rs", 1),
            make_row("tests/test.rs", 2),
        ];
        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };
        // src/main.rs has 2 commits, src/lib.rs has 1, tests/test.rs has 1
        assert_eq!(scores.commit_counts.get("src/main.rs"), Some(&2));
        assert_eq!(scores.commit_counts.get("src/lib.rs"), Some(&1));
        assert_eq!(scores.commit_counts.get("tests/test.rs"), Some(&1));
    }

    #[test]
    fn nested_repo_hotspot_calculation() {
        let repo = match create_nested_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![make_row("src/main.rs", 3), make_row("tests/test.rs", 2)];
        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };
        assert_eq!(scores.hotspots.get("src/main.rs"), Some(&6)); // 3*2
        assert_eq!(scores.hotspots.get("tests/test.rs"), Some(&2)); // 2*1
    }

    // ── 3. Many commits (hot vs cold) ───────────────────────────────

    #[test]
    fn many_commits_hot_file_ranks_higher() {
        let repo = match create_many_commits_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![make_row("hot.rs", 5), make_row("cold.rs", 1)];
        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };
        assert!(
            scores.hotspots.get("hot.rs").copied().unwrap_or(0)
                > scores.hotspots.get("cold.rs").copied().unwrap_or(0),
            "hot file should rank higher"
        );
    }

    #[test]
    fn many_commits_commit_count_for_hot_file() {
        let repo = match create_many_commits_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![make_row("hot.rs", 5), make_row("cold.rs", 1)];
        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };
        assert_eq!(scores.commit_counts.get("hot.rs"), Some(&5));
        assert_eq!(scores.commit_counts.get("cold.rs"), Some(&1));
    }

    // ── 4. Child rows filtering ─────────────────────────────────────

    #[test]
    fn child_rows_excluded_from_scoring() {
        let repo = match create_standard_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![make_child_row("main.rs", 4), make_row("lib.rs", 5)];
        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };
        assert!(!scores.commit_counts.contains_key("main.rs"));
        assert!(scores.commit_counts.contains_key("lib.rs"));
    }

    #[test]
    fn all_child_rows_produces_empty_maps() {
        let repo = match create_standard_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![make_child_row("main.rs", 4), make_child_row("lib.rs", 5)];
        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };
        assert!(scores.commit_counts.is_empty());
        assert!(scores.hotspots.is_empty());
    }

    // ── 5. Non-repo and empty repo ──────────────────────────────────

    #[test]
    fn non_git_directory_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let rows = vec![make_row("foo.rs", 10)];
        assert!(compute_git_scores(dir.path(), &rows, 100, 100).is_none());
    }

    #[test]
    fn non_git_directory_empty_rows_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let rows: Vec<FileRow> = vec![];
        assert!(compute_git_scores(dir.path(), &rows, 100, 100).is_none());
    }

    #[test]
    fn empty_rows_with_valid_repo() {
        let repo = match create_standard_repo() {
            Some(r) => r,
            None => return,
        };
        let rows: Vec<FileRow> = vec![];
        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };
        assert!(scores.commit_counts.is_empty());
        assert!(scores.hotspots.is_empty());
    }

    // ── 6. Untracked files ──────────────────────────────────────────

    #[test]
    fn file_not_in_git_absent_from_results() {
        let repo = match create_standard_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![make_row("nonexistent.rs", 100)];
        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };
        assert!(!scores.commit_counts.contains_key("nonexistent.rs"));
        assert!(scores.hotspots.is_empty());
    }

    #[test]
    fn mix_of_tracked_and_untracked() {
        let repo = match create_standard_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![
            make_row("main.rs", 4),   // tracked
            make_row("ghost.rs", 50), // not in git
        ];
        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };
        assert!(scores.commit_counts.contains_key("main.rs"));
        assert!(!scores.commit_counts.contains_key("ghost.rs"));
    }

    // ── 7. Zero lines edge case ─────────────────────────────────────

    #[test]
    fn zero_lines_produces_zero_hotspot() {
        let repo = match create_standard_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![make_row("main.rs", 0)];
        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };
        assert_eq!(scores.hotspots.get("main.rs"), Some(&0));
        // But commit count should still be present
        assert!(scores.commit_counts.get("main.rs").unwrap_or(&0) > &0);
    }

    // ── 8. Determinism ──────────────────────────────────────────────

    #[test]
    fn scores_are_deterministic() {
        let repo = match create_standard_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![
            make_row("main.rs", 4),
            make_row("lib.rs", 5),
            make_row("util.rs", 2),
        ];
        let s1 = compute_git_scores(repo.path(), &rows, 100, 100);
        let s2 = compute_git_scores(repo.path(), &rows, 100, 100);
        match (s1, s2) {
            (Some(a), Some(b)) => {
                assert_eq!(
                    a.commit_counts.keys().collect::<Vec<_>>(),
                    b.commit_counts.keys().collect::<Vec<_>>()
                );
                assert_eq!(
                    a.hotspots.keys().collect::<Vec<_>>(),
                    b.hotspots.keys().collect::<Vec<_>>()
                );
                for key in a.hotspots.keys() {
                    assert_eq!(a.hotspots[key], b.hotspots[key]);
                }
                for key in a.commit_counts.keys() {
                    assert_eq!(a.commit_counts[key], b.commit_counts[key]);
                }
            }
            (None, None) => {} // both None is acceptable
            _ => panic!("determinism violated: one returned Some, other None"),
        }
    }

    // ── 9. BTreeMap ordering of results ─────────────────────────────

    #[test]
    fn results_keys_are_lexicographically_sorted() {
        let repo = match create_standard_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![
            make_row("util.rs", 2),
            make_row("lib.rs", 5),
            make_row("main.rs", 4),
        ];
        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };
        let keys: Vec<&String> = scores.commit_counts.keys().collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted, "BTreeMap keys should be sorted");
    }

    // ── 10. Structural invariant: hotspot keys ⊆ commit_count keys ──

    #[test]
    fn hotspot_keys_subset_of_commit_count_keys() {
        let repo = match create_standard_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![
            make_row("main.rs", 4),
            make_row("lib.rs", 5),
            make_row("util.rs", 2),
        ];
        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };
        for key in scores.hotspots.keys() {
            assert!(
                scores.commit_counts.contains_key(key),
                "hotspot key {key} missing from commit_counts"
            );
        }
    }

    // ── 11. Max commits parameter ───────────────────────────────────

    #[test]
    fn max_commits_one_limits_history_depth() {
        let repo = match create_standard_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![
            make_row("main.rs", 4),
            make_row("lib.rs", 5),
            make_row("util.rs", 2),
        ];
        let Some(scores) = compute_git_scores(repo.path(), &rows, 1, 100) else {
            return;
        };
        // With only 1 commit (most recent = c3), main.rs should not appear
        // c3 only touched lib.rs and util.rs
        let main_count = scores.commit_counts.get("main.rs").copied().unwrap_or(0);
        assert!(
            main_count <= 1,
            "max_commits=1 should limit main.rs commits, got {main_count}"
        );
    }

    // ── 12. GitScores struct direct construction ────────────────────

    #[test]
    fn git_scores_struct_with_realistic_data() {
        let mut hotspots = BTreeMap::new();
        hotspots.insert("src/main.rs".to_string(), 500usize); // 100 lines * 5 commits
        hotspots.insert("src/lib.rs".to_string(), 200); // 100 lines * 2 commits
        hotspots.insert("tests/test.rs".to_string(), 50); // 50 lines * 1 commit

        let mut commit_counts = BTreeMap::new();
        commit_counts.insert("src/main.rs".to_string(), 5usize);
        commit_counts.insert("src/lib.rs".to_string(), 2);
        commit_counts.insert("tests/test.rs".to_string(), 1);

        let scores = GitScores {
            hotspots,
            commit_counts,
        };

        assert_eq!(scores.hotspots.len(), 3);
        assert_eq!(scores.commit_counts.len(), 3);
        assert_eq!(scores.hotspots["src/main.rs"], 500);
        assert!(scores.hotspots["src/main.rs"] > scores.hotspots["src/lib.rs"]);
    }

    #[test]
    fn git_scores_empty_hotspots_nonempty_commits() {
        let scores = GitScores {
            hotspots: BTreeMap::new(),
            commit_counts: {
                let mut m = BTreeMap::new();
                m.insert("a.rs".to_string(), 1usize);
                m
            },
        };
        assert!(scores.hotspots.is_empty());
        assert!(!scores.commit_counts.is_empty());
    }

    // ── 13. Score with different line counts for same commit count ───

    #[test]
    fn same_commits_different_lines_produces_different_hotspots() {
        let repo = match create_standard_repo() {
            Some(r) => r,
            None => return,
        };
        // lib.rs and util.rs both have 1 commit but different line counts
        let rows = vec![make_row("lib.rs", 100), make_row("util.rs", 10)];
        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };
        // With 1 commit each: lib.rs hotspot = 100, util.rs hotspot = 10
        assert!(
            scores.hotspots.get("lib.rs").copied().unwrap_or(0)
                > scores.hotspots.get("util.rs").copied().unwrap_or(0),
        );
    }

    // ── 14. Score returns Some for valid repo ───────────────────────

    #[test]
    fn valid_repo_with_data_returns_some() {
        let repo = match create_standard_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![make_row("main.rs", 4)];
        let result = compute_git_scores(repo.path(), &rows, 100, 100);
        assert!(result.is_some(), "valid repo should return Some");
    }

    #[test]
    fn valid_repo_returns_nonempty_scores() {
        let repo = match create_standard_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![make_row("main.rs", 4)];
        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };
        assert!(!scores.commit_counts.is_empty());
        assert!(!scores.hotspots.is_empty());
    }
}

// ===========================================================================
// Tests that work without git feature
// ===========================================================================

mod no_feature_required {
    use std::collections::BTreeMap;
    use tokmd_context_git::GitScores;

    #[test]
    fn git_scores_both_empty() {
        let scores = GitScores {
            hotspots: BTreeMap::new(),
            commit_counts: BTreeMap::new(),
        };
        assert!(scores.hotspots.is_empty());
        assert!(scores.commit_counts.is_empty());
    }

    #[test]
    fn git_scores_keys_are_sorted() {
        let mut hotspots = BTreeMap::new();
        hotspots.insert("z.rs".to_string(), 1usize);
        hotspots.insert("a.rs".to_string(), 2);
        hotspots.insert("m.rs".to_string(), 3);

        let scores = GitScores {
            hotspots,
            commit_counts: BTreeMap::new(),
        };

        let keys: Vec<&String> = scores.hotspots.keys().collect();
        assert_eq!(keys, vec!["a.rs", "m.rs", "z.rs"]);
    }

    #[test]
    fn git_scores_large_values() {
        let mut hotspots = BTreeMap::new();
        hotspots.insert("big.rs".to_string(), 100_000 * 10_000);
        let scores = GitScores {
            hotspots,
            commit_counts: BTreeMap::new(),
        };
        assert_eq!(scores.hotspots["big.rs"], 1_000_000_000);
    }

    #[test]
    fn git_scores_duplicate_insert_overwrites() {
        let mut hotspots = BTreeMap::new();
        hotspots.insert("a.rs".to_string(), 10usize);
        hotspots.insert("a.rs".to_string(), 20);
        let scores = GitScores {
            hotspots,
            commit_counts: BTreeMap::new(),
        };
        assert_eq!(scores.hotspots["a.rs"], 20);
        assert_eq!(scores.hotspots.len(), 1);
    }

    #[test]
    fn git_scores_many_files() {
        let hotspots: BTreeMap<String, usize> = (0..100)
            .map(|i| (format!("file_{i:03}.rs"), i * 10))
            .collect();
        let scores = GitScores {
            hotspots,
            commit_counts: BTreeMap::new(),
        };
        assert_eq!(scores.hotspots.len(), 100);
        // First key should be file_000.rs
        assert_eq!(scores.hotspots.keys().next().unwrap(), "file_000.rs");
        // Last key should be file_099.rs
        assert_eq!(scores.hotspots.keys().last().unwrap(), "file_099.rs");
    }
}
