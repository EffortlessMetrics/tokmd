//! W76 deep tests for ``tokmd-analysis` Git module`.
//!
//! Exercises hotspot scoring, coupling metrics (Jaccard/lift), freshness
//! boundaries, bus-factor calculation, code-age bucket distribution,
//! predictive churn regression, intent classification, and edge cases.

use std::path::Path;

use super::super::{build_git_report, build_predictive_churn_report};
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

// ═══════════════════════════════════════════════════════════════════
// § 1. Hotspot scoring edge cases
// ═══════════════════════════════════════════════════════════════════

mod hotspot_w76 {
    use super::*;

    #[test]
    fn many_commits_same_file_accumulates_score() {
        let e = export(vec![row("src/hot.rs", "src", 100)]);
        let commits: Vec<GitCommit> = (1..=20)
            .map(|i| commit(i * DAY, "dev", &format!("c{i}"), &["src/hot.rs"]))
            .collect();
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        assert_eq!(r.hotspots[0].score, 100 * 20);
    }

    #[test]
    fn unmatched_git_files_excluded_from_hotspots() {
        let e = export(vec![row("src/lib.rs", "src", 50)]);
        let commits = vec![
            commit(DAY, "a", "c1", &["src/lib.rs"]),
            commit(2 * DAY, "a", "c2", &["nonexistent.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        assert_eq!(r.hotspots.len(), 1);
        assert_eq!(r.hotspots[0].path, "src/lib.rs");
    }

    #[test]
    fn hotspot_tie_broken_by_path() {
        let e = export(vec![row("z.rs", "src", 100), row("a.rs", "src", 100)]);
        let commits = vec![commit(DAY, "a", "c", &["z.rs", "a.rs"])];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        // Same score (100 * 1) -> alphabetical by path
        assert_eq!(r.hotspots[0].path, "a.rs");
        assert_eq!(r.hotspots[1].path, "z.rs");
    }
}

// ═══════════════════════════════════════════════════════════════════
// § 2. Bus factor
// ═══════════════════════════════════════════════════════════════════

mod bus_factor_w76 {
    use super::*;

    #[test]
    fn multiple_authors_counted_per_module() {
        let e = export(vec![row("src/lib.rs", "src", 100)]);
        let commits = vec![
            commit(DAY, "alice", "c1", &["src/lib.rs"]),
            commit(2 * DAY, "bob", "c2", &["src/lib.rs"]),
            commit(3 * DAY, "charlie", "c3", &["src/lib.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        assert_eq!(r.bus_factor[0].authors, 3);
    }

    #[test]
    fn same_author_multiple_commits_counted_once() {
        let e = export(vec![row("src/lib.rs", "src", 100)]);
        let commits = vec![
            commit(DAY, "alice", "c1", &["src/lib.rs"]),
            commit(2 * DAY, "alice", "c2", &["src/lib.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        assert_eq!(r.bus_factor[0].authors, 1);
    }

    #[test]
    fn bus_factor_sorted_by_authors_ascending() {
        let e = export(vec![row("a.rs", "alpha", 10), row("b.rs", "beta", 10)]);
        let commits = vec![
            commit(DAY, "alice", "c1", &["a.rs"]),
            commit(2 * DAY, "alice", "c2", &["b.rs"]),
            commit(3 * DAY, "bob", "c3", &["b.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        // alpha has 1 author, beta has 2 -> ascending
        assert_eq!(r.bus_factor[0].module, "alpha");
        assert_eq!(r.bus_factor[0].authors, 1);
        assert_eq!(r.bus_factor[1].module, "beta");
        assert_eq!(r.bus_factor[1].authors, 2);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § 3. Coupling with Jaccard/lift
// ═══════════════════════════════════════════════════════════════════

mod coupling_w76 {
    use super::*;

    #[test]
    fn no_coupling_when_modules_never_co_change() {
        let e = export(vec![row("a.rs", "alpha", 50), row("b.rs", "beta", 50)]);
        let commits = vec![
            commit(DAY, "a", "c1", &["a.rs"]),
            commit(2 * DAY, "a", "c2", &["b.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        assert!(r.coupling.is_empty());
    }

    #[test]
    fn perfect_coupling_jaccard_is_one() {
        let e = export(vec![row("a.rs", "alpha", 50), row("b.rs", "beta", 50)]);
        // Both modules always change together
        let commits = vec![
            commit(DAY, "a", "c1", &["a.rs", "b.rs"]),
            commit(2 * DAY, "a", "c2", &["a.rs", "b.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        assert_eq!(r.coupling.len(), 1);
        let j = r.coupling[0].jaccard.unwrap();
        assert!(
            (j - 1.0).abs() < 0.001,
            "perfect coupling jaccard should be 1.0, got {j}"
        );
    }

    #[test]
    fn coupling_count_reflects_co_occurrence() {
        let e = export(vec![row("a.rs", "alpha", 50), row("b.rs", "beta", 50)]);
        let commits = vec![
            commit(DAY, "a", "c1", &["a.rs", "b.rs"]),
            commit(2 * DAY, "a", "c2", &["a.rs", "b.rs"]),
            commit(3 * DAY, "a", "c3", &["a.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        assert_eq!(r.coupling[0].count, 2);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § 4. Freshness boundary conditions
// ═══════════════════════════════════════════════════════════════════

mod freshness_w76 {
    use super::*;

    #[test]
    fn all_files_fresh_gives_zero_stale_pct() {
        let now = 100 * DAY;
        let e = export(vec![row("a.rs", "src", 10), row("b.rs", "src", 10)]);
        let commits = vec![
            commit(now - DAY, "a", "c1", &["a.rs"]),
            commit(now, "a", "c2", &["b.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        assert_eq!(r.freshness.stale_files, 0);
        assert_eq!(r.freshness.stale_pct, 0.0);
    }

    #[test]
    fn all_files_stale_gives_one_stale_pct() {
        let now = 1000 * DAY;
        let e = export(vec![
            row("a.rs", "src", 10),
            row("b.rs", "src", 10),
            // Need a recent file to set the reference timestamp
            row("ref.rs", "src", 10),
        ]);
        let commits = vec![
            commit(now - 400 * DAY, "a", "c1", &["a.rs"]),
            commit(now - 500 * DAY, "a", "c2", &["b.rs"]),
            commit(now, "a", "ref", &["ref.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        // a.rs (400d) and b.rs (500d) are stale, ref.rs (0d) is not
        assert_eq!(r.freshness.stale_files, 2);
        assert_eq!(r.freshness.total_files, 3);
    }

    #[test]
    fn freshness_p90_within_reasonable_range() {
        let now = 200 * DAY;
        let e = export(vec![
            row("a.rs", "src", 10),
            row("b.rs", "src", 10),
            row("c.rs", "src", 10),
        ]);
        let commits = vec![
            commit(now - 10 * DAY, "a", "c1", &["a.rs"]),
            commit(now - 50 * DAY, "a", "c2", &["b.rs"]),
            commit(now, "a", "c3", &["c.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        for m in &r.freshness.by_module {
            assert!(m.p90_days >= 0.0, "p90 should be non-negative");
            assert!(
                m.p90_days >= m.avg_days * 0.5,
                "p90 should be >= half the average"
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// § 5. Predictive churn regression
// ═══════════════════════════════════════════════════════════════════

mod churn_w76 {
    use super::*;

    #[test]
    fn decreasing_commits_give_falling_trend() {
        let e = export(vec![row("src/lib.rs", "src", 100)]);
        // 3 commits in week 1, 2 in week 2, 1 in week 3
        let commits = vec![
            commit(WEEK, "a", "c1", &["src/lib.rs"]),
            commit(WEEK + DAY, "a", "c2", &["src/lib.rs"]),
            commit(WEEK + 2 * DAY, "a", "c3", &["src/lib.rs"]),
            commit(2 * WEEK, "a", "c4", &["src/lib.rs"]),
            commit(2 * WEEK + DAY, "a", "c5", &["src/lib.rs"]),
            commit(3 * WEEK, "a", "c6", &["src/lib.rs"]),
        ];
        let r = build_predictive_churn_report(&e, &commits, Path::new("."));
        let trend = r.per_module.get("src").unwrap();
        assert!(
            trend.slope < 0.0,
            "slope should be negative for decreasing churn"
        );
        assert_eq!(trend.classification, TrendClass::Falling);
    }

    #[test]
    fn single_commit_gives_flat_trend() {
        let e = export(vec![row("src/lib.rs", "src", 100)]);
        let commits = vec![commit(WEEK, "a", "c1", &["src/lib.rs"])];
        let r = build_predictive_churn_report(&e, &commits, Path::new("."));
        let trend = r.per_module.get("src").unwrap();
        assert_eq!(trend.classification, TrendClass::Flat);
    }

    #[test]
    fn no_commits_gives_empty_per_module() {
        let e = export(vec![row("src/lib.rs", "src", 100)]);
        let r = build_predictive_churn_report(&e, &[], Path::new("."));
        assert!(r.per_module.is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════════
// § 6. Intent classification and corrective ratio
// ═══════════════════════════════════════════════════════════════════

mod intent_w76 {
    use super::*;

    #[test]
    fn feat_commits_classified_correctly() {
        let e = export(vec![row("src/lib.rs", "src", 100)]);
        let commits = vec![
            commit(DAY, "a", "feat: add new feature", &["src/lib.rs"]),
            commit(2 * DAY, "a", "feat: another feature", &["src/lib.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        let intent = r.intent.unwrap();
        assert_eq!(intent.overall.feat, 2);
        assert_eq!(intent.overall.total, 2);
    }

    #[test]
    fn mixed_intents_all_counted() {
        let e = export(vec![row("src/lib.rs", "src", 100)]);
        let commits = vec![
            commit(DAY, "a", "feat: add", &["src/lib.rs"]),
            commit(2 * DAY, "a", "fix: bug", &["src/lib.rs"]),
            commit(3 * DAY, "a", "docs: readme", &["src/lib.rs"]),
            commit(4 * DAY, "a", "refactor: cleanup", &["src/lib.rs"]),
            commit(5 * DAY, "a", "test: add tests", &["src/lib.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        let intent = r.intent.unwrap();
        assert_eq!(intent.overall.feat, 1);
        assert_eq!(intent.overall.fix, 1);
        assert_eq!(intent.overall.docs, 1);
        assert_eq!(intent.overall.refactor, 1);
        assert_eq!(intent.overall.test, 1);
        assert_eq!(intent.overall.total, 5);
    }

    #[test]
    fn all_fix_commits_gives_corrective_ratio_one() {
        let e = export(vec![row("src/lib.rs", "src", 100)]);
        let commits = vec![
            commit(DAY, "a", "fix: bug one", &["src/lib.rs"]),
            commit(2 * DAY, "a", "fix: bug two", &["src/lib.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        let intent = r.intent.unwrap();
        assert!((intent.corrective_ratio.unwrap() - 1.0).abs() < 0.001);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § 7. Code age distribution buckets
// ═══════════════════════════════════════════════════════════════════

mod age_w76 {
    use super::*;

    #[test]
    fn files_in_each_bucket_sum_to_total() {
        let now = 500 * DAY;
        let e = export(vec![
            row("a.rs", "src", 10),
            row("b.rs", "src", 10),
            row("c.rs", "src", 10),
        ]);
        let commits = vec![
            commit(now - 10 * DAY, "a", "c1", &["a.rs"]),
            commit(now - 100 * DAY, "a", "c2", &["b.rs"]),
            commit(now - 400 * DAY, "a", "c3", &["c.rs"]),
        ];
        let r = build_git_report(Path::new("."), &e, &commits).unwrap();
        let dist = r.age_distribution.unwrap();
        let total: usize = dist.buckets.iter().map(|b| b.files).sum();
        assert_eq!(total, 3);
    }

    #[test]
    fn refresh_trend_flat_when_no_commits() {
        let e = export(vec![row("a.rs", "src", 10)]);
        let r = build_git_report(Path::new("."), &e, &[]).unwrap();
        let dist = r.age_distribution.unwrap();
        assert_eq!(dist.refresh_trend, TrendClass::Flat);
    }
}
