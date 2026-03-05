//! W68 deep tests for `tokmd-analysis-git`.
//!
//! Exercises hotspot detection, coupling metrics, freshness computation,
//! bus factor, code age distribution, intent classification, predictive
//! churn, and edge cases (no commits, single commit, backslash paths).

use std::path::Path;

use tokmd_analysis_git::{build_git_report, build_predictive_churn_report};
use tokmd_analysis_types::TrendClass;
use tokmd_git::GitCommit;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────

const DAY: i64 = 86_400;
const WEEK: i64 = 7 * DAY;

fn row(path: &str, module: &str, lines: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
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

fn export(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn commit(ts: i64, author: &str, subject: &str, files: &[&str]) -> GitCommit {
    GitCommit {
        timestamp: ts,
        author: author.to_string(),
        hash: None,
        subject: subject.to_string(),
        files: files.iter().map(|s| s.to_string()).collect(),
    }
}

// ── Hotspot detection ───────────────────────────────────────────

mod hotspot_w68 {
    use super::*;

    #[test]
    fn score_equals_lines_times_commits() {
        let e = export(vec![row("src/lib.rs", "src", 200)]);
        let commits = vec![
            commit(DAY, "a", "c1", &["src/lib.rs"]),
            commit(2 * DAY, "a", "c2", &["src/lib.rs"]),
            commit(3 * DAY, "a", "c3", &["src/lib.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        assert_eq!(r.hotspots.len(), 1);
        assert_eq!(r.hotspots[0].score, 200 * 3);
        assert_eq!(r.hotspots[0].commits, 3);
        assert_eq!(r.hotspots[0].lines, 200);
    }

    #[test]
    fn hotspots_sorted_desc_by_score() {
        let e = export(vec![
            row("src/big.rs", "src", 300),
            row("src/small.rs", "src", 10),
        ]);
        let commits = vec![
            commit(DAY, "a", "c1", &["src/big.rs"]),
            commit(2 * DAY, "a", "c2", &["src/small.rs", "src/big.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        assert!(r.hotspots[0].score > r.hotspots[1].score);
        assert_eq!(r.hotspots[0].path, "src/big.rs");
    }

    #[test]
    fn backslash_paths_normalised_to_forward_slash() {
        let e = export(vec![row("src/lib.rs", "src", 50)]);
        let commits = vec![commit(DAY, "a", "c", &["src\\lib.rs"])];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        assert_eq!(r.hotspots.len(), 1);
        assert_eq!(r.hotspots[0].path, "src/lib.rs");
    }

    #[test]
    fn dot_slash_prefix_stripped() {
        let e = export(vec![row("src/lib.rs", "src", 50)]);
        let commits = vec![commit(DAY, "a", "c", &["./src/lib.rs"])];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        assert_eq!(r.hotspots.len(), 1);
    }

    #[test]
    fn files_seen_counts_unique_matched_files() {
        let e = export(vec![row("src/a.rs", "src", 10), row("src/b.rs", "src", 10)]);
        let commits = vec![
            commit(DAY, "a", "c1", &["src/a.rs"]),
            commit(2 * DAY, "a", "c2", &["src/a.rs", "src/b.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        assert_eq!(r.files_seen, 2);
    }
}

// ── Coupling analysis ───────────────────────────────────────────

mod coupling_w68 {
    use super::*;

    #[test]
    fn coupling_jaccard_range_zero_to_one() {
        let e = export(vec![
            row("src/a.rs", "alpha", 50),
            row("src/b.rs", "beta", 50),
        ]);
        let commits = vec![
            commit(DAY, "a", "c1", &["src/a.rs", "src/b.rs"]),
            commit(2 * DAY, "a", "c2", &["src/a.rs"]),
            commit(3 * DAY, "a", "c3", &["src/b.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        for c in &r.coupling {
            let j = c.jaccard.unwrap();
            assert!(j > 0.0 && j <= 1.0, "jaccard={j} out of range");
        }
    }

    #[test]
    fn coupling_lift_above_one_when_correlated() {
        let e = export(vec![
            row("src/a.rs", "alpha", 50),
            row("src/b.rs", "beta", 50),
        ]);
        // Every commit touches both modules → perfectly correlated
        let commits = vec![
            commit(DAY, "a", "c1", &["src/a.rs", "src/b.rs"]),
            commit(2 * DAY, "a", "c2", &["src/a.rs", "src/b.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        assert_eq!(r.coupling.len(), 1);
        let lift = r.coupling[0].lift.unwrap();
        assert!(
            lift >= 1.0,
            "perfect co-occurrence should give lift >= 1.0, got {lift}"
        );
    }

    #[test]
    fn coupling_n_left_n_right_populated() {
        let e = export(vec![
            row("src/a.rs", "alpha", 50),
            row("src/b.rs", "beta", 50),
        ]);
        let commits = vec![
            commit(DAY, "a", "c1", &["src/a.rs", "src/b.rs"]),
            commit(2 * DAY, "a", "c2", &["src/a.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        assert_eq!(r.coupling.len(), 1);
        assert_eq!(r.coupling[0].n_left.unwrap(), 2); // alpha touched 2 times
        assert_eq!(r.coupling[0].n_right.unwrap(), 1); // beta touched 1 time
    }

    #[test]
    fn three_modules_produce_three_pairs() {
        let e = export(vec![
            row("a.rs", "alpha", 10),
            row("b.rs", "beta", 10),
            row("c.rs", "gamma", 10),
        ]);
        // One commit touches all three → C(3,2)=3 pairs
        let commits = vec![commit(DAY, "a", "c1", &["a.rs", "b.rs", "c.rs"])];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        assert_eq!(r.coupling.len(), 3);
    }

    #[test]
    fn coupling_pair_keys_are_ordered() {
        let e = export(vec![row("z.rs", "zulu", 10), row("a.rs", "alpha", 10)]);
        let commits = vec![commit(DAY, "a", "c1", &["z.rs", "a.rs"])];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        assert_eq!(r.coupling.len(), 1);
        assert!(
            r.coupling[0].left <= r.coupling[0].right,
            "left should be <= right"
        );
    }
}

// ── Freshness computation ───────────────────────────────────────

mod freshness_w68 {
    use super::*;

    #[test]
    fn stale_threshold_is_365_days() {
        let e = export(vec![row("src/lib.rs", "src", 100)]);
        let commits = vec![commit(DAY, "a", "old", &["src/lib.rs"])];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        assert_eq!(r.freshness.threshold_days, 365);
    }

    #[test]
    fn file_at_exactly_365_days_not_stale() {
        let reference = 400 * DAY;
        let file_ts = reference - 365 * DAY;
        let e = export(vec![
            row("src/lib.rs", "src", 100),
            row("src/new.rs", "src", 50),
        ]);
        // Need a recent commit to set max_ts as our reference
        let commits = vec![
            commit(file_ts, "a", "old", &["src/lib.rs"]),
            commit(reference, "a", "recent", &["src/new.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        // 365 days is the exact boundary; > threshold is stale
        assert_eq!(r.freshness.stale_files, 0);
    }

    #[test]
    fn file_at_366_days_is_stale() {
        let reference = 500 * DAY;
        let file_ts = reference - 366 * DAY;
        let e = export(vec![
            row("src/lib.rs", "src", 100),
            row("src/new.rs", "src", 50),
        ]);
        // Need a recent commit to set max_ts as our reference
        let commits = vec![
            commit(file_ts, "a", "old", &["src/lib.rs"]),
            commit(reference, "a", "recent", &["src/new.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        assert_eq!(r.freshness.stale_files, 1);
    }

    #[test]
    fn freshness_module_rows_sorted_by_module_name() {
        let e = export(vec![
            row("src/z.rs", "zulu", 10),
            row("src/a.rs", "alpha", 10),
            row("src/m.rs", "mike", 10),
        ]);
        let now = 100 * DAY;
        let commits = vec![commit(
            now - DAY,
            "a",
            "c",
            &["src/z.rs", "src/a.rs", "src/m.rs"],
        )];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        let modules: Vec<&str> = r
            .freshness
            .by_module
            .iter()
            .map(|m| m.module.as_str())
            .collect();
        let mut sorted = modules.clone();
        sorted.sort();
        assert_eq!(modules, sorted);
    }

    #[test]
    fn stale_pct_correct_when_half_stale() {
        let now = 500 * DAY;
        let e = export(vec![row("old.rs", "src", 100), row("new.rs", "src", 100)]);
        let commits = vec![
            commit(now - 400 * DAY, "a", "old", &["old.rs"]),
            commit(now - 10 * DAY, "a", "new", &["new.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        assert_eq!(r.freshness.stale_files, 1);
        assert_eq!(r.freshness.total_files, 2);
        assert!((r.freshness.stale_pct - 0.5).abs() < 0.001);
    }
}

// ── Code age distribution ───────────────────────────────────────

mod age_distribution_w68 {
    use super::*;

    #[test]
    fn five_buckets_always_present() {
        let e = export(vec![row("src/lib.rs", "src", 100)]);
        let commits = vec![commit(DAY, "a", "c", &["src/lib.rs"])];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        let dist = r.age_distribution.unwrap();
        assert_eq!(dist.buckets.len(), 5);
    }

    #[test]
    fn bucket_pcts_sum_to_one() {
        let now = 500 * DAY;
        let e = export(vec![row("a.rs", "src", 10), row("b.rs", "src", 10)]);
        let commits = vec![
            commit(now - 10 * DAY, "a", "c1", &["a.rs"]),
            commit(now - 200 * DAY, "a", "c2", &["b.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        let dist = r.age_distribution.unwrap();
        let total_pct: f64 = dist.buckets.iter().map(|b| b.pct).sum();
        assert!(
            (total_pct - 1.0).abs() < 0.01,
            "bucket pcts should sum to ~1.0, got {total_pct}"
        );
    }

    #[test]
    fn refresh_trend_rising_when_all_recent() {
        let now = 100 * DAY;
        let e = export(vec![row("a.rs", "src", 10)]);
        // Commit in the recent 30-day window, none in prior window
        let commits = vec![commit(now - 5 * DAY, "a", "c", &["a.rs"])];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        let dist = r.age_distribution.unwrap();
        assert_eq!(dist.refresh_trend, TrendClass::Rising);
    }
}

// ── Edge cases ──────────────────────────────────────────────────

mod edge_cases_w68 {
    use super::*;

    #[test]
    fn no_commits_all_fields_empty() {
        let e = export(vec![row("src/lib.rs", "src", 100)]);
        let r = build_git_report(Path::new("."), &e, &[]).unwrap();
        assert_eq!(r.commits_scanned, 0);
        assert_eq!(r.files_seen, 0);
        assert!(r.hotspots.is_empty());
        assert!(r.bus_factor.is_empty());
        assert!(r.coupling.is_empty());
        assert_eq!(r.freshness.total_files, 0);
    }

    #[test]
    fn single_commit_single_file() {
        let e = export(vec![row("main.rs", "root", 42)]);
        let commits = vec![commit(DAY, "alice", "feat: init", &["main.rs"])];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        assert_eq!(r.commits_scanned, 1);
        assert_eq!(r.files_seen, 1);
        assert_eq!(r.hotspots.len(), 1);
        assert_eq!(r.bus_factor.len(), 1);
        assert_eq!(r.bus_factor[0].authors, 1);
    }

    #[test]
    fn empty_export_with_commits_produces_empty_report() {
        let e = export(vec![]);
        let commits = vec![
            commit(DAY, "a", "c1", &["src/lib.rs"]),
            commit(2 * DAY, "b", "c2", &["src/main.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        assert_eq!(r.commits_scanned, 2);
        assert_eq!(r.files_seen, 0);
        assert!(r.hotspots.is_empty());
    }

    #[test]
    fn child_file_rows_excluded_from_hotspots() {
        let mut e = export(vec![row("src/lib.rs", "src", 100)]);
        e.rows.push(FileRow {
            path: "src/embedded.html".to_string(),
            module: "src".to_string(),
            lang: "HTML".to_string(),
            kind: FileKind::Child,
            code: 50,
            comments: 0,
            blanks: 0,
            lines: 50,
            bytes: 500,
            tokens: 100,
        });
        let commits = vec![commit(DAY, "a", "c", &["src/lib.rs", "src/embedded.html"])];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        // Only parent rows should appear in hotspots
        assert_eq!(r.hotspots.len(), 1);
        assert_eq!(r.hotspots[0].path, "src/lib.rs");
    }
}

// ── Predictive churn ────────────────────────────────────────────

mod churn_w68 {
    use super::*;

    #[test]
    fn increasing_commits_give_positive_slope() {
        let e = export(vec![row("src/lib.rs", "src", 100)]);
        // 1 commit in week 1, 2 in week 2, 3 in week 3
        let commits = vec![
            commit(WEEK, "a", "c1", &["src/lib.rs"]),
            commit(2 * WEEK, "a", "c2", &["src/lib.rs"]),
            commit(2 * WEEK + DAY, "a", "c3", &["src/lib.rs"]),
            commit(3 * WEEK, "a", "c4", &["src/lib.rs"]),
            commit(3 * WEEK + DAY, "a", "c5", &["src/lib.rs"]),
            commit(3 * WEEK + 2 * DAY, "a", "c6", &["src/lib.rs"]),
        ];
        let r = build_predictive_churn_report(&e, &commits, Path::new("."));
        let trend = r.per_module.get("src").unwrap();
        assert!(
            trend.slope > 0.0,
            "slope should be positive for increasing churn"
        );
        assert_eq!(trend.classification, TrendClass::Rising);
    }

    #[test]
    fn constant_commits_give_flat_trend() {
        let e = export(vec![row("src/lib.rs", "src", 100)]);
        // Exactly 1 commit per week
        let commits: Vec<GitCommit> = (1..=5)
            .map(|w| commit(w * WEEK, "a", "c", &["src/lib.rs"]))
            .collect();
        let r = build_predictive_churn_report(&e, &commits, Path::new("."));
        let trend = r.per_module.get("src").unwrap();
        assert_eq!(trend.classification, TrendClass::Flat);
    }

    #[test]
    fn churn_r2_in_valid_range() {
        let e = export(vec![row("src/lib.rs", "src", 100)]);
        let commits: Vec<GitCommit> = (1..=10)
            .map(|w| commit(w * WEEK, "a", "c", &["src/lib.rs"]))
            .collect();
        let r = build_predictive_churn_report(&e, &commits, Path::new("."));
        let trend = r.per_module.get("src").unwrap();
        assert!(
            trend.r2 >= 0.0 && trend.r2 <= 1.0,
            "r2={} out of range",
            trend.r2
        );
    }

    #[test]
    fn churn_per_module_keys_match_export_modules() {
        let e = export(vec![
            row("src/a.rs", "mod_a", 50),
            row("src/b.rs", "mod_b", 50),
        ]);
        let commits = vec![
            commit(WEEK, "a", "c1", &["src/a.rs"]),
            commit(2 * WEEK, "a", "c2", &["src/b.rs"]),
        ];
        let r = build_predictive_churn_report(&e, &commits, Path::new("."));
        assert!(r.per_module.contains_key("mod_a"));
        assert!(r.per_module.contains_key("mod_b"));
    }
}

// ── Intent classification ───────────────────────────────────────

mod intent_w68 {
    use super::*;

    #[test]
    fn corrective_ratio_zero_when_no_fixes() {
        let e = export(vec![row("src/lib.rs", "src", 100)]);
        let commits = vec![
            commit(DAY, "a", "feat: add feature", &["src/lib.rs"]),
            commit(2 * DAY, "a", "docs: readme", &["src/lib.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        let intent = r.intent.unwrap();
        assert_eq!(intent.corrective_ratio.unwrap(), 0.0);
    }

    #[test]
    fn corrective_ratio_includes_fix_and_revert() {
        let e = export(vec![row("src/lib.rs", "src", 100)]);
        let commits = vec![
            commit(DAY, "a", "feat: add", &["src/lib.rs"]),
            commit(2 * DAY, "a", "fix: bug", &["src/lib.rs"]),
            commit(3 * DAY, "a", "Revert \"feat: add\"", &["src/lib.rs"]),
            commit(4 * DAY, "a", "chore: cleanup", &["src/lib.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        let intent = r.intent.unwrap();
        // 2 corrective out of 4 total = 0.5
        assert!((intent.corrective_ratio.unwrap() - 0.5).abs() < 0.001);
    }

    #[test]
    fn intent_by_module_populated() {
        let e = export(vec![
            row("src/a.rs", "mod_a", 50),
            row("src/b.rs", "mod_b", 50),
        ]);
        let commits = vec![
            commit(DAY, "a", "feat: add", &["src/a.rs"]),
            commit(2 * DAY, "a", "fix: bug", &["src/b.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        let intent = r.intent.unwrap();
        assert_eq!(intent.by_module.len(), 2);
        let modules: Vec<&str> = intent.by_module.iter().map(|m| m.module.as_str()).collect();
        assert!(modules.contains(&"mod_a"));
        assert!(modules.contains(&"mod_b"));
    }

    #[test]
    fn unknown_pct_one_when_all_other() {
        let e = export(vec![row("src/lib.rs", "src", 100)]);
        let commits = vec![
            commit(DAY, "a", "random commit message", &["src/lib.rs"]),
            commit(2 * DAY, "a", "another random message", &["src/lib.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        let intent = r.intent.unwrap();
        assert!((intent.unknown_pct - 1.0).abs() < 0.001);
    }
}
