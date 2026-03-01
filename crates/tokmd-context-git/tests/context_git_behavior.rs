//! Tests for git integration in context selection: changed-file detection,
//! file prioritization by recency, and feature gating behavior.

#[cfg(feature = "git")]
mod with_git {
    use std::process::Command;
    use tokmd_context_git::{GitScores, compute_git_scores};
    use tokmd_types::{FileKind, FileRow};

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

    fn git(root: &std::path::Path, args: &[&str]) -> Option<()> {
        let out = Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .ok()?;
        if out.status.success() { Some(()) } else { None }
    }

    /// Repo layout:
    ///   hot.rs   — 4 commits, 10 lines  → hotspot = 40
    ///   warm.rs  — 2 commits, 8 lines   → hotspot = 16
    ///   cold.rs  — 1 commit, 20 lines   → hotspot = 20
    fn create_prioritization_repo() -> Option<tempfile::TempDir> {
        let dir = tempfile::tempdir().ok()?;
        let root = dir.path();

        git(root, &["init"])?;
        git(root, &["config", "user.email", "test@test.com"])?;
        git(root, &["config", "user.name", "Test"])?;

        // Commit 1: all three files
        std::fs::write(root.join("hot.rs"), "fn hot() {}\n".repeat(10)).ok()?;
        std::fs::write(root.join("warm.rs"), "fn warm() {}\n".repeat(8)).ok()?;
        std::fs::write(root.join("cold.rs"), "fn cold() {}\n".repeat(20)).ok()?;
        git(root, &["add", "."])?;
        git(root, &["commit", "-m", "initial"])?;

        // Commits 2-4: only hot.rs
        for i in 2..=4 {
            let content = format!("// rev {i}\n") + &"fn hot() {}\n".repeat(10);
            std::fs::write(root.join("hot.rs"), &content).ok()?;
            git(root, &["add", "hot.rs"])?;
            git(root, &["commit", "-m", &format!("hot-{i}")])?;
        }

        // Commit 5: warm.rs second touch
        std::fs::write(
            root.join("warm.rs"),
            "// updated\n".to_owned() + &"fn warm() {}\n".repeat(8),
        )
        .ok()?;
        git(root, &["add", "warm.rs"])?;
        git(root, &["commit", "-m", "warm-2"])?;

        Some(dir)
    }

    // ── Prioritization by commit frequency ──────────────────────────

    #[test]
    fn frequently_changed_files_have_higher_commit_counts() {
        let repo = match create_prioritization_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![
            make_row("hot.rs", 10),
            make_row("warm.rs", 8),
            make_row("cold.rs", 20),
        ];
        let scores = compute_git_scores(repo.path(), &rows, 100, 100).unwrap();

        let hot = scores.commit_counts["hot.rs"];
        let warm = scores.commit_counts["warm.rs"];
        let cold = scores.commit_counts["cold.rs"];
        assert!(
            hot > warm,
            "hot ({hot}) should have more commits than warm ({warm})"
        );
        assert!(
            warm > cold,
            "warm ({warm}) should have more commits than cold ({cold})"
        );
    }

    #[test]
    fn hotspot_ranking_reflects_commit_and_size_combined() {
        let repo = match create_prioritization_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![
            make_row("hot.rs", 10),
            make_row("warm.rs", 8),
            make_row("cold.rs", 20),
        ];
        let scores = compute_git_scores(repo.path(), &rows, 100, 100).unwrap();

        // hot.rs: 10 lines × 4 commits = 40
        // cold.rs: 20 lines × 1 commit = 20
        // warm.rs: 8 lines × 2 commits = 16
        let hot = scores.hotspots["hot.rs"];
        let cold = scores.hotspots["cold.rs"];
        let warm = scores.hotspots["warm.rs"];
        assert!(hot > cold, "hot ({hot}) should outrank cold ({cold})");
        assert!(cold > warm, "cold ({cold}) should outrank warm ({warm})");
    }

    // ── Changed-file detection ──────────────────────────────────────

    #[test]
    fn only_files_present_in_rows_appear_in_scores() {
        let repo = match create_prioritization_repo() {
            Some(r) => r,
            None => return,
        };
        // Only ask about warm.rs — hot.rs and cold.rs should not appear
        let rows = vec![make_row("warm.rs", 8)];
        let scores = compute_git_scores(repo.path(), &rows, 100, 100).unwrap();

        assert!(scores.commit_counts.contains_key("warm.rs"));
        assert!(!scores.commit_counts.contains_key("hot.rs"));
        assert!(!scores.commit_counts.contains_key("cold.rs"));
    }

    #[test]
    fn file_not_in_git_history_absent_from_scores() {
        let repo = match create_prioritization_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![make_row("phantom.rs", 100)];
        let scores = compute_git_scores(repo.path(), &rows, 100, 100).unwrap();

        assert!(!scores.commit_counts.contains_key("phantom.rs"));
        assert!(scores.hotspots.is_empty());
    }

    // ── Determinism ─────────────────────────────────────────────────

    #[test]
    fn git_scores_are_deterministic_across_repeated_calls() {
        let repo = match create_prioritization_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![
            make_row("hot.rs", 10),
            make_row("warm.rs", 8),
            make_row("cold.rs", 20),
        ];

        let s1 = compute_git_scores(repo.path(), &rows, 100, 100).unwrap();
        let s2 = compute_git_scores(repo.path(), &rows, 100, 100).unwrap();
        let s3 = compute_git_scores(repo.path(), &rows, 100, 100).unwrap();

        assert_eq!(s1.commit_counts, s2.commit_counts);
        assert_eq!(s2.commit_counts, s3.commit_counts);
        assert_eq!(s1.hotspots, s2.hotspots);
        assert_eq!(s2.hotspots, s3.hotspots);
    }

    // ── max_commits limiting ────────────────────────────────────────

    #[test]
    fn max_commits_limits_visible_history_window() {
        let repo = match create_prioritization_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![
            make_row("hot.rs", 10),
            make_row("warm.rs", 8),
            make_row("cold.rs", 20),
        ];

        // Full history: hot has 4 commits
        let full = compute_git_scores(repo.path(), &rows, 100, 100).unwrap();
        // Limited to 2 most recent commits
        let limited = match compute_git_scores(repo.path(), &rows, 2, 100) {
            Some(s) => s,
            None => return,
        };

        let full_hot = full.commit_counts.get("hot.rs").copied().unwrap_or(0);
        let limited_hot = limited.commit_counts.get("hot.rs").copied().unwrap_or(0);
        assert!(
            limited_hot <= full_hot,
            "limited ({limited_hot}) should not exceed full ({full_hot})"
        );
    }

    // ── BTreeMap ordering guarantees ────────────────────────────────

    #[test]
    fn scores_maps_maintain_sorted_key_order() {
        let repo = match create_prioritization_repo() {
            Some(r) => r,
            None => return,
        };
        let rows = vec![
            make_row("hot.rs", 10),
            make_row("warm.rs", 8),
            make_row("cold.rs", 20),
        ];
        let scores = compute_git_scores(repo.path(), &rows, 100, 100).unwrap();

        let keys: Vec<&String> = scores.commit_counts.keys().collect();
        let mut sorted_keys = keys.clone();
        sorted_keys.sort();
        assert_eq!(keys, sorted_keys, "BTreeMap keys should be sorted");
    }

    // ── GitScores struct construction ───────────────────────────────

    #[test]
    fn git_scores_can_be_constructed_empty() {
        let scores = GitScores {
            hotspots: Default::default(),
            commit_counts: Default::default(),
        };
        assert!(scores.hotspots.is_empty());
        assert!(scores.commit_counts.is_empty());
    }

    // ── Multi-file commit attribution ───────────────────────────────

    #[test]
    fn multi_file_commit_increments_all_touched_files() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        if git(root, &["init"]).is_none() {
            return;
        }
        git(root, &["config", "user.email", "test@test.com"]).unwrap();
        git(root, &["config", "user.name", "Test"]).unwrap();

        // Single commit touching three files at once
        std::fs::write(root.join("a.rs"), "a").unwrap();
        std::fs::write(root.join("b.rs"), "b").unwrap();
        std::fs::write(root.join("c.rs"), "c").unwrap();
        git(root, &["add", "."]).unwrap();
        git(root, &["commit", "-m", "all-at-once"]).unwrap();

        let rows = vec![
            make_row("a.rs", 1),
            make_row("b.rs", 1),
            make_row("c.rs", 1),
        ];
        let scores = compute_git_scores(root, &rows, 100, 100).unwrap();

        // All three files should have exactly 1 commit
        for file in ["a.rs", "b.rs", "c.rs"] {
            assert_eq!(
                scores.commit_counts.get(file),
                Some(&1),
                "{file} should have 1 commit"
            );
        }
    }

    // ── Subdirectory path handling ──────────────────────────────────

    #[test]
    fn subdirectory_files_matched_with_forward_slash_paths() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        if git(root, &["init"]).is_none() {
            return;
        }
        git(root, &["config", "user.email", "test@test.com"]).unwrap();
        git(root, &["config", "user.name", "Test"]).unwrap();

        std::fs::create_dir_all(root.join("src").join("util")).unwrap();
        std::fs::write(
            root.join("src").join("util").join("helpers.rs"),
            "fn help() {}",
        )
        .unwrap();
        git(root, &["add", "."]).unwrap();
        git(root, &["commit", "-m", "nested"]).unwrap();

        let rows = vec![make_row("src/util/helpers.rs", 1)];
        let scores = compute_git_scores(root, &rows, 100, 100).unwrap();

        assert!(
            scores.commit_counts.contains_key("src/util/helpers.rs"),
            "deeply nested path should match with forward slashes"
        );
    }
}

// ── Feature gating: without git feature ─────────────────────────────────

#[cfg(not(feature = "git"))]
mod without_git {
    use tokmd_context_git::compute_git_scores;
    use tokmd_types::{FileKind, FileRow};

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

    #[test]
    fn without_git_feature_always_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let rows = vec![make_row("main.rs", 100)];
        assert!(
            compute_git_scores(dir.path(), &rows, 100, 100).is_none(),
            "should return None when git feature is disabled"
        );
    }

    #[test]
    fn without_git_feature_returns_none_even_with_empty_rows() {
        let dir = tempfile::tempdir().unwrap();
        let rows: Vec<FileRow> = vec![];
        assert!(compute_git_scores(dir.path(), &rows, 100, 100).is_none());
    }
}
