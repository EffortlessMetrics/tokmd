//! Deep tests for ``tokmd-analysis` Git module`.
//!
//! Exercises build_git_report and build_predictive_churn_report with edge
//! cases, serialization roundtrips, and multi-module scenarios.

use std::path::Path;

use super::super::{build_git_report, build_predictive_churn_report};
use tokmd_analysis_types::TrendClass;
use tokmd_git::GitCommit;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

const DAY: i64 = 86_400;
const WEEK: i64 = 7 * DAY;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn file_row(path: &str, module: &str, lines: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Parent,
        code: lines,
        comments: 0,
        blanks: 0,
        lines,
        bytes: lines * 40,
        tokens: lines * 3,
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

// ===========================================================================
// 1. Empty export with non-empty commits → report uses zero rows
// ===========================================================================
#[test]
fn empty_export_ignores_all_commits() {
    let exp = export(vec![]);
    let commits = vec![commit(1000, "a", "feat: x", &["foo.rs"])];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.commits_scanned, 1);
    assert_eq!(report.files_seen, 0);
    assert!(report.hotspots.is_empty());
    assert!(report.coupling.is_empty());
}

// ===========================================================================
// 2. Many authors across multiple modules → bus factor reflects each module
// ===========================================================================
#[test]
fn bus_factor_multiple_modules() {
    let exp = export(vec![
        file_row("a/x.rs", "a", 10),
        file_row("b/y.rs", "b", 20),
        file_row("c/z.rs", "c", 30),
    ]);
    let commits = vec![
        commit(1000, "alice", "feat: a", &["a/x.rs"]),
        commit(2000, "bob", "feat: b", &["b/y.rs"]),
        commit(3000, "charlie", "feat: b", &["b/y.rs"]),
        commit(4000, "dave", "feat: c", &["c/z.rs"]),
        commit(5000, "eve", "feat: c", &["c/z.rs"]),
        commit(6000, "frank", "feat: c", &["c/z.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.bus_factor.len(), 3);
    // Sorted by authors ascending: a(1), b(2), c(3)
    assert_eq!(report.bus_factor[0].authors, 1);
    assert_eq!(report.bus_factor[1].authors, 2);
    assert_eq!(report.bus_factor[2].authors, 3);
}

// ===========================================================================
// 3. Duplicate author in same module → counted once
// ===========================================================================
#[test]
fn bus_factor_deduplicates_authors() {
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits = vec![
        commit(1000, "alice", "feat: 1", &["src/lib.rs"]),
        commit(2000, "alice", "feat: 2", &["src/lib.rs"]),
        commit(3000, "alice", "feat: 3", &["src/lib.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.bus_factor.len(), 1);
    assert_eq!(report.bus_factor[0].authors, 1);
}

// ===========================================================================
// 4. Hotspot with zero-line file → score is zero
// ===========================================================================
#[test]
fn hotspot_zero_lines_zero_score() {
    let exp = export(vec![file_row("src/empty.rs", "src", 0)]);
    let commits = vec![commit(1000, "alice", "feat: empty", &["src/empty.rs"])];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.hotspots.len(), 1);
    assert_eq!(report.hotspots[0].score, 0);
}

// ===========================================================================
// 5. Freshness with all stale files → stale_pct == 1.0
// ===========================================================================
#[test]
fn freshness_all_stale() {
    let now = 1000 * DAY;
    let exp = export(vec![
        file_row("src/a.rs", "src", 50),
        file_row("src/b.rs", "src", 50),
    ]);
    // Make a.rs 500 days old; unknown.rs advances max_ts to now.
    let very_old = vec![
        commit(now - 500 * DAY, "alice", "feat: a", &["src/a.rs"]),
        commit(now, "bob", "feat: trigger_ts", &["unknown.rs"]),
    ];
    // a.rs is 500 days old, but unknown.rs is not in export. Only a.rs is tracked.
    // Wait - commits on unknown files still advance max_ts.
    let report = build_git_report(Path::new("."), &exp, &very_old).unwrap();
    // a.rs is the only file seen. 500 days > 365 → stale
    assert_eq!(report.freshness.stale_files, 1);
    assert_eq!(report.freshness.total_files, 1);
    assert_eq!(report.freshness.stale_pct, 1.0);
}

// ===========================================================================
// 6. Coupling sorted by count descending
// ===========================================================================
#[test]
fn coupling_sorted_by_count_desc() {
    let exp = export(vec![
        file_row("a/x.rs", "a", 10),
        file_row("b/y.rs", "b", 20),
        file_row("c/z.rs", "c", 30),
    ]);
    // a+b coupled 3 times, a+c coupled 1 time
    let commits = vec![
        commit(1000, "alice", "feat: ab", &["a/x.rs", "b/y.rs"]),
        commit(2000, "bob", "feat: ab", &["a/x.rs", "b/y.rs"]),
        commit(3000, "charlie", "feat: ab", &["a/x.rs", "b/y.rs"]),
        commit(4000, "dave", "feat: ac", &["a/x.rs", "c/z.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.coupling.len(), 2);
    assert_eq!(report.coupling[0].count, 3); // a-b
    assert_eq!(report.coupling[1].count, 1); // a-c
}

// ===========================================================================
// 7. Coupling with three modules in one commit generates all pairs
// ===========================================================================
#[test]
fn coupling_three_modules_all_pairs() {
    let exp = export(vec![
        file_row("a/x.rs", "a", 10),
        file_row("b/y.rs", "b", 20),
        file_row("c/z.rs", "c", 30),
    ]);
    let commits = vec![commit(
        1000,
        "alice",
        "feat: all",
        &["a/x.rs", "b/y.rs", "c/z.rs"],
    )];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    // 3 modules → C(3,2) = 3 pairs
    assert_eq!(report.coupling.len(), 3);
}

// ===========================================================================
// 8. GitReport JSON serialization roundtrip
// ===========================================================================
#[test]
fn git_report_serialization_roundtrip() {
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits = vec![
        commit(1000, "alice", "feat: init", &["src/lib.rs"]),
        commit(2000, "bob", "fix: bug", &["src/lib.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let json = serde_json::to_string(&report).unwrap();
    let deser: tokmd_analysis_types::GitReport = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.commits_scanned, report.commits_scanned);
    assert_eq!(deser.files_seen, report.files_seen);
    assert_eq!(deser.hotspots.len(), report.hotspots.len());
    assert_eq!(deser.bus_factor.len(), report.bus_factor.len());
}

// ===========================================================================
// 9. PredictiveChurnReport JSON serialization roundtrip
// ===========================================================================
#[test]
fn churn_report_serialization_roundtrip() {
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits: Vec<GitCommit> = (1..=5)
        .map(|i| commit(i * WEEK, "alice", "feat: weekly", &["src/lib.rs"]))
        .collect();
    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));
    let json = serde_json::to_string(&report).unwrap();
    let deser: tokmd_analysis_types::PredictiveChurnReport = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.per_module.len(), report.per_module.len());
    let orig = report.per_module.get("src").unwrap();
    let rt = deser.per_module.get("src").unwrap();
    assert_eq!(rt.classification, orig.classification);
}

// ===========================================================================
// 10. Age distribution buckets cover all files
// ===========================================================================
#[test]
fn age_distribution_total_files_equals_bucket_sum() {
    let now = 1000 * DAY;
    let exp = export(vec![
        file_row("src/a.rs", "src", 50),
        file_row("src/b.rs", "src", 50),
        file_row("src/c.rs", "src", 50),
        file_row("src/d.rs", "src", 50),
    ]);
    let commits = vec![
        commit(now, "alice", "feat: a", &["src/a.rs"]),
        commit(now - 50 * DAY, "bob", "feat: b", &["src/b.rs"]),
        commit(now - 150 * DAY, "charlie", "feat: c", &["src/c.rs"]),
        commit(now - 400 * DAY, "dave", "feat: d", &["src/d.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let age = report.age_distribution.as_ref().unwrap();
    let total: usize = age.buckets.iter().map(|b| b.files).sum();
    assert_eq!(total, 4);
}

// ===========================================================================
// 11. Intent with only "other" commits → unknown_pct == 1.0
// ===========================================================================
#[test]
fn intent_all_unknown_commits() {
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits = vec![
        commit(1000, "alice", "initial commit", &["src/lib.rs"]),
        commit(2000, "bob", "update stuff", &["src/lib.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let intent = report.intent.as_ref().unwrap();
    assert_eq!(intent.overall.other, 2);
    assert_eq!(intent.overall.total, 2);
    assert_eq!(intent.unknown_pct, 1.0);
}

// ===========================================================================
// 12. Intent corrective_ratio is None when no commits
// ===========================================================================
#[test]
fn intent_corrective_ratio_zero_commits() {
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits: Vec<GitCommit> = vec![];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let intent = report.intent.as_ref().unwrap();
    assert_eq!(intent.corrective_ratio, None);
}

// ===========================================================================
// 13. Churn decreasing activity → falling trend
// ===========================================================================
#[test]
fn churn_decreasing_activity_falling() {
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    // Decreasing commits per week: 5, 4, 3, 2, 1
    let mut commits = Vec::new();
    for w in 1..=5i64 {
        let count = (6 - w) as usize;
        for _ in 0..count {
            commits.push(commit(w * WEEK, "alice", "feat: less", &["src/lib.rs"]));
        }
    }
    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));
    let trend = report.per_module.get("src").unwrap();
    assert!(
        trend.slope < 0.0,
        "decreasing activity should have negative slope"
    );
    assert_eq!(trend.classification, TrendClass::Falling);
}

// ===========================================================================
// 14. Churn with multiple modules → separate trends
// ===========================================================================
#[test]
fn churn_multiple_modules_independent() {
    let exp = export(vec![
        file_row("a/x.rs", "a", 10),
        file_row("b/y.rs", "b", 20),
    ]);
    // Module a: increasing, module b: single commit
    let mut commits = Vec::new();
    for w in 1..=4i64 {
        for _ in 0..w {
            commits.push(commit(w * WEEK, "alice", "feat: a", &["a/x.rs"]));
        }
    }
    commits.push(commit(WEEK, "bob", "feat: b", &["b/y.rs"]));
    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));
    assert!(report.per_module.contains_key("a"));
    assert!(report.per_module.contains_key("b"));
    let a = report.per_module.get("a").unwrap();
    assert!(a.slope > 0.0);
    let b = report.per_module.get("b").unwrap();
    assert_eq!(b.slope, 0.0); // single data point
}

// ===========================================================================
// 15. Hotspot handles large number of files
// ===========================================================================
#[test]
fn hotspot_many_files() {
    let rows: Vec<FileRow> = (0..50)
        .map(|i| file_row(&format!("src/f{i}.rs"), "src", (i + 1) * 10))
        .collect();
    let exp = export(rows);
    let commits: Vec<GitCommit> = (0..50)
        .map(|i| {
            commit(
                (i + 1) * 1000,
                "alice",
                "feat: x",
                &[&format!("src/f{i}.rs")],
            )
        })
        .collect();
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.hotspots.len(), 50);
    // Sorted descending by score; first should be f49 with 500 lines
    assert_eq!(report.hotspots[0].path, "src/f49.rs");
}

// ===========================================================================
// 16. Freshness module p90 computation
// ===========================================================================
#[test]
fn freshness_module_has_p90() {
    let now = 1000 * DAY;
    let rows: Vec<FileRow> = (0..10)
        .map(|i| file_row(&format!("src/f{i}.rs"), "src", 10))
        .collect();
    let exp = export(rows);
    let commits: Vec<GitCommit> = (0..10)
        .map(|i| {
            commit(
                now - (i * 30 * DAY),
                "alice",
                "feat",
                &[&format!("src/f{i}.rs")],
            )
        })
        .collect();
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.freshness.by_module.len(), 1);
    let m = &report.freshness.by_module[0];
    assert!(m.p90_days >= m.avg_days, "p90 >= avg");
}

// ===========================================================================
// 17. Coupling jaccard never exceeds 1.0
// ===========================================================================
#[test]
fn coupling_jaccard_bounded() {
    let exp = export(vec![
        file_row("a/x.rs", "a", 10),
        file_row("b/y.rs", "b", 20),
    ]);
    let commits = vec![
        commit(1000, "alice", "feat: ab", &["a/x.rs", "b/y.rs"]),
        commit(2000, "bob", "feat: ab", &["a/x.rs", "b/y.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    for c in &report.coupling {
        if let Some(j) = c.jaccard {
            assert!(j <= 1.0, "jaccard {j} should be <= 1.0");
            assert!(j > 0.0, "jaccard {j} should be > 0.0");
        }
    }
}

// ===========================================================================
// 18. Coupling lift is positive for correlated modules
// ===========================================================================
#[test]
fn coupling_lift_positive() {
    let exp = export(vec![
        file_row("a/x.rs", "a", 10),
        file_row("b/y.rs", "b", 20),
    ]);
    let commits = vec![
        commit(1000, "alice", "feat: ab", &["a/x.rs", "b/y.rs"]),
        commit(2000, "bob", "fix: a", &["a/x.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.coupling.len(), 1);
    let lift = report.coupling[0].lift.unwrap();
    assert!(lift > 0.0);
}

// ===========================================================================
// 19. Churn R² in [0, 1]
// ===========================================================================
#[test]
fn churn_r2_bounded() {
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits: Vec<GitCommit> = (1..=10)
        .map(|i| commit(i * WEEK, "alice", "feat", &["src/lib.rs"]))
        .collect();
    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));
    let trend = report.per_module.get("src").unwrap();
    assert!(trend.r2 >= 0.0 && trend.r2 <= 1.0, "r2 = {}", trend.r2);
}

// ===========================================================================
// 20. Intent with all conventional prefixes
// ===========================================================================
#[test]
fn intent_all_conventional_prefixes() {
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits = vec![
        commit(1000, "a", "feat: new feature", &["src/lib.rs"]),
        commit(2000, "a", "fix: bug fix", &["src/lib.rs"]),
        commit(3000, "a", "docs: update docs", &["src/lib.rs"]),
        commit(4000, "a", "test: add tests", &["src/lib.rs"]),
        commit(5000, "a", "chore: cleanup", &["src/lib.rs"]),
        commit(6000, "a", "refactor: refactoring", &["src/lib.rs"]),
        commit(7000, "a", "ci: pipeline", &["src/lib.rs"]),
        commit(8000, "a", "perf: optimize", &["src/lib.rs"]),
        commit(9000, "a", "style: format", &["src/lib.rs"]),
        commit(10000, "a", "build: deps", &["src/lib.rs"]),
        commit(11000, "a", "revert: undo", &["src/lib.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let intent = report.intent.as_ref().unwrap();
    assert_eq!(intent.overall.feat, 1);
    assert_eq!(intent.overall.fix, 1);
    assert_eq!(intent.overall.docs, 1);
    assert_eq!(intent.overall.test, 1);
    assert_eq!(intent.overall.chore, 1);
    assert_eq!(intent.overall.refactor, 1);
    assert_eq!(intent.overall.ci, 1);
    assert_eq!(intent.overall.perf, 1);
    assert_eq!(intent.overall.style, 1);
    assert_eq!(intent.overall.build, 1);
    assert_eq!(intent.overall.revert, 1);
    assert_eq!(intent.overall.total, 11);
    assert_eq!(intent.overall.other, 0);
    assert_eq!(intent.unknown_pct, 0.0);
}

// ===========================================================================
// 21. Refresh trend flat when no commits
// ===========================================================================
#[test]
fn refresh_trend_flat_no_commits() {
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let report = build_git_report(Path::new("."), &exp, &[]).unwrap();
    let age = report.age_distribution.as_ref().unwrap();
    assert_eq!(age.refresh_trend, TrendClass::Flat);
    assert_eq!(age.recent_refreshes, 0);
    assert_eq!(age.prior_refreshes, 0);
}

// ===========================================================================
// 22. Multiple files in same module → bus factor still 1 module
// ===========================================================================
#[test]
fn bus_factor_multiple_files_same_module() {
    let exp = export(vec![
        file_row("src/a.rs", "src", 50),
        file_row("src/b.rs", "src", 50),
        file_row("src/c.rs", "src", 50),
    ]);
    let commits = vec![
        commit(1000, "alice", "feat: a", &["src/a.rs"]),
        commit(2000, "bob", "feat: b", &["src/b.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.bus_factor.len(), 1);
    assert_eq!(report.bus_factor[0].module, "src");
    assert_eq!(report.bus_factor[0].authors, 2);
}

// ===========================================================================
// 23. Churn report empty when no files match commits
// ===========================================================================
#[test]
fn churn_no_matching_files() {
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits = vec![commit(WEEK, "alice", "feat", &["other/file.rs"])];
    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));
    assert!(report.per_module.is_empty());
}

// ===========================================================================
// 24. Coupling n_left and n_right fields populated
// ===========================================================================
#[test]
fn coupling_n_left_n_right_populated() {
    let exp = export(vec![
        file_row("a/x.rs", "a", 10),
        file_row("b/y.rs", "b", 20),
    ]);
    let commits = vec![
        commit(1000, "alice", "feat: ab", &["a/x.rs", "b/y.rs"]),
        commit(2000, "bob", "feat: a", &["a/x.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.coupling.len(), 1);
    assert_eq!(report.coupling[0].n_left, Some(2)); // a touched 2 times
    assert_eq!(report.coupling[0].n_right, Some(1)); // b touched 1 time
}
