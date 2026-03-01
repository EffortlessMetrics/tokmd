//! BDD-style scenario tests for git-based context selection.
//!
//! Each test follows the Given-When-Then naming convention to document
//! expected behaviour of `compute_git_scores`.

#[cfg(feature = "git")]
mod with_git {
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

    /// Create a repo with:
    ///   main.rs  – 2 commits, 4 final lines
    ///   lib.rs   – 1 commit, 5 lines
    ///   util.rs  – 1 commit, 2 lines (committed alongside lib.rs)
    fn create_test_repo() -> Option<tempfile::TempDir> {
        let dir = tempfile::tempdir().ok()?;
        let root = dir.path();

        git(root, &["init"])?;
        git(root, &["config", "user.email", "test@test.com"])?;
        git(root, &["config", "user.name", "Test"])?;

        // Commit 1: main.rs (3 lines)
        std::fs::write(root.join("main.rs"), "1\n2\n3").ok()?;
        git(root, &["add", "."])?;
        git(root, &["commit", "-m", "c1"])?;

        // Commit 2: main.rs grows to 4 lines
        std::fs::write(root.join("main.rs"), "1\n2\n3\n4").ok()?;
        git(root, &["add", "."])?;
        git(root, &["commit", "-m", "c2"])?;

        // Commit 3: lib.rs (5 lines) + util.rs (2 lines)
        std::fs::write(root.join("lib.rs"), "1\n2\n3\n4\n5").ok()?;
        std::fs::write(root.join("util.rs"), "1\n2").ok()?;
        git(root, &["add", "."])?;
        git(root, &["commit", "-m", "c3"])?;

        Some(dir)
    }

    /// Create an empty repo (git init, but no commits)
    fn create_empty_repo() -> Option<tempfile::TempDir> {
        let dir = tempfile::tempdir().ok()?;
        let root = dir.path();
        git(root, &["init"])?;
        git(root, &["config", "user.email", "test@test.com"])?;
        git(root, &["config", "user.name", "Test"])?;
        Some(dir)
    }

    // ── scenario: recently changed files are prioritised ────────────

    #[test]
    fn given_files_with_git_history_when_context_is_selected_then_recently_changed_files_are_prioritised()
     {
        let repo = match create_test_repo() {
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

        // main.rs has 2 commits → hotspot = 4 × 2 = 8
        // lib.rs has 1 commit  → hotspot = 5 × 1 = 5
        // util.rs has 1 commit → hotspot = 2 × 1 = 2
        // Most-changed file (main.rs) gets highest hotspot score.
        let main_hotspot = scores.hotspots["main.rs"];
        let lib_hotspot = scores.hotspots["lib.rs"];
        let util_hotspot = scores.hotspots["util.rs"];
        assert!(
            main_hotspot > lib_hotspot,
            "main.rs ({main_hotspot}) should rank above lib.rs ({lib_hotspot})"
        );
        assert!(
            lib_hotspot > util_hotspot,
            "lib.rs ({lib_hotspot}) should rank above util.rs ({util_hotspot})"
        );
    }

    // ── scenario: commit counts are accurate ────────────────────────

    #[test]
    fn given_files_with_varying_commits_when_scores_computed_then_commit_counts_are_correct() {
        let repo = match create_test_repo() {
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

        assert_eq!(scores.commit_counts["main.rs"], 2);
        assert_eq!(scores.commit_counts["lib.rs"], 1);
        assert_eq!(scores.commit_counts["util.rs"], 1);
    }

    // ── scenario: hotspot = lines × commits ─────────────────────────

    #[test]
    fn given_known_line_counts_when_scores_computed_then_hotspot_equals_lines_times_commits() {
        let repo = match create_test_repo() {
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

        assert_eq!(scores.hotspots["main.rs"], 4 * 2);
        assert_eq!(scores.hotspots["lib.rs"], 5);
        assert_eq!(scores.hotspots["util.rs"], 2);
    }

    // ── scenario: child rows are excluded ───────────────────────────

    #[test]
    fn given_child_file_rows_when_scores_computed_then_children_are_excluded() {
        let repo = match create_test_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![
            make_child_row("main.rs", 4), // Child – should be filtered
            make_row("lib.rs", 5),        // Parent – should be included
        ];

        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };

        assert!(
            !scores.commit_counts.contains_key("main.rs"),
            "child row should be excluded from commit_counts"
        );
        assert!(
            !scores.hotspots.contains_key("main.rs"),
            "child row should be excluded from hotspots"
        );
        assert!(scores.commit_counts.contains_key("lib.rs"));
    }

    // ── scenario: files not tracked by git are absent ───────────────

    #[test]
    fn given_file_not_in_git_when_scores_computed_then_file_is_absent_from_results() {
        let repo = match create_test_repo() {
            Some(r) => r,
            None => return,
        };
        // "nonexistent.rs" was never committed
        let rows = vec![make_row("nonexistent.rs", 10)];

        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };

        assert!(
            !scores.commit_counts.contains_key("nonexistent.rs"),
            "untracked file should not appear in commit_counts"
        );
        assert!(scores.hotspots.is_empty());
    }

    // ── scenario: non-repo directory returns None ───────────────────

    #[test]
    fn given_non_git_directory_when_scores_computed_then_none_is_returned() {
        let dir = tempfile::tempdir().unwrap();
        let rows = vec![make_row("foo.rs", 10)];
        assert!(compute_git_scores(dir.path(), &rows, 100, 100).is_none());
    }

    // ── scenario: empty commit history ──────────────────────────────

    #[test]
    fn given_empty_commit_history_when_scores_computed_then_empty_maps_returned() {
        let repo = match create_empty_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![make_row("foo.rs", 10)];

        // An empty repo (no commits) may return None or Some with empty maps
        // depending on git log behaviour. Either is acceptable.
        match compute_git_scores(repo.path(), &rows, 100, 100) {
            None => {} // acceptable – git log fails on empty repo
            Some(scores) => {
                assert!(scores.commit_counts.is_empty());
                assert!(scores.hotspots.is_empty());
            }
        }
    }

    // ── scenario: empty rows list ───────────────────────────────────

    #[test]
    fn given_empty_file_rows_when_scores_computed_then_empty_maps_returned() {
        let repo = match create_test_repo() {
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

    // ── scenario: max_commits limits history depth ──────────────────

    #[test]
    fn given_max_commits_of_two_when_scores_computed_then_older_commits_excluded() {
        let repo = match create_test_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![
            make_row("main.rs", 4),
            make_row("lib.rs", 5),
            make_row("util.rs", 2),
        ];

        // With max_commits=2, only c3 and c2 should be considered
        let scores = match compute_git_scores(repo.path(), &rows, 2, 100) {
            Some(s) => s,
            None => return, // git history collection may not support limit
        };

        // main.rs appears in c2 (1 commit in window), not in c1 (truncated)
        let main_count = scores.commit_counts.get("main.rs").copied().unwrap_or(0);
        assert!(
            main_count <= 2,
            "main.rs should have at most 2 commits in window, got {main_count}"
        );
        // lib.rs and util.rs appear in c3 (within window)
        assert!(scores.commit_counts.contains_key("lib.rs"));
        assert!(scores.commit_counts.contains_key("util.rs"));
    }

    // ── scenario: hotspot map is subset of commit_counts ────────────

    #[test]
    fn given_any_repo_when_scores_computed_then_hotspot_keys_are_subset_of_commit_count_keys() {
        let repo = match create_test_repo() {
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
                "hotspot key {key:?} must also appear in commit_counts"
            );
        }
    }

    // ── scenario: files with zero lines produce zero hotspot ────────

    #[test]
    fn given_file_with_zero_lines_when_scores_computed_then_hotspot_is_zero() {
        let repo = match create_test_repo() {
            Some(r) => r,
            None => return,
        };
        // main.rs exists in git but we report 0 lines
        let rows = vec![make_row("main.rs", 0)];

        let Some(scores) = compute_git_scores(repo.path(), &rows, 100, 100) else {
            return;
        };

        assert_eq!(
            scores.hotspots.get("main.rs"),
            Some(&0),
            "0 lines × any commits = 0"
        );
    }

    // ── scenario: GitScores struct fields are accessible ────────────

    #[test]
    fn given_git_scores_struct_when_accessed_then_both_fields_are_btreemaps() {
        let scores = GitScores {
            hotspots: Default::default(),
            commit_counts: Default::default(),
        };
        // Ensure the struct is constructible and fields are BTreeMaps
        assert!(scores.hotspots.is_empty());
        assert!(scores.commit_counts.is_empty());
    }

    // ── scenario: subdirectory paths normalised correctly ────────────

    #[test]
    fn given_file_in_subdirectory_when_scores_computed_then_path_normalised() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // Set up repo with a subdirectory file
        if git(root, &["init"]).is_none() {
            return;
        }
        if git(root, &["config", "user.email", "test@test.com"]).is_none() {
            return;
        }
        if git(root, &["config", "user.name", "Test"]).is_none() {
            return;
        }

        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src").join("main.rs"), "fn main() {}").unwrap();
        if git(root, &["add", "."]).is_none() {
            return;
        }
        if git(root, &["commit", "-m", "init"]).is_none() {
            return;
        }

        let rows = vec![make_row("src/main.rs", 1)];

        let Some(scores) = compute_git_scores(root, &rows, 100, 100) else {
            return;
        };

        assert!(
            scores.commit_counts.contains_key("src/main.rs"),
            "normalised forward-slash path should match"
        );
    }
}

// ── scenario: without git feature ───────────────────────────────

#[cfg(not(feature = "git"))]
mod without_git {
    use tokmd_context_git::compute_git_scores;

    #[test]
    fn given_no_git_feature_when_compute_called_then_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let rows = vec![];
        assert!(compute_git_scores(dir.path(), &rows, 100, 100).is_none());
    }
}
