//! Deep tests for tokmd-analysis-git: git report building, coupling,
//! freshness, intent classification, and predictive churn.

use std::path::Path;

use tokmd_analysis_git::{build_git_report, build_predictive_churn_report};
use tokmd_analysis_types::TrendClass;
use tokmd_git::GitCommit;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

const SECONDS_PER_DAY: i64 = 86_400;
const SECONDS_PER_WEEK: i64 = 7 * SECONDS_PER_DAY;

fn make_row(path: &str, module: &str, lines: usize) -> FileRow {
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

fn make_export(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn make_commit(ts: i64, author: &str, subject: &str, files: &[&str]) -> GitCommit {
    GitCommit {
        timestamp: ts,
        author: author.to_string(),
        hash: None,
        subject: subject.to_string(),
        files: files.iter().map(|s| s.to_string()).collect(),
    }
}

// ===========================================================================
// 1. build_git_report – basic behaviour
// ===========================================================================

#[test]
fn git_report_empty_commits_empty_export() {
    let export = make_export(vec![]);
    let report = build_git_report(Path::new("."), &export, &[]).unwrap();
    assert_eq!(report.commits_scanned, 0);
    assert_eq!(report.files_seen, 0);
    assert!(report.hotspots.is_empty());
    assert!(report.bus_factor.is_empty());
    assert!(report.coupling.is_empty());
}

#[test]
fn git_report_counts_commits_and_files() {
    let export = make_export(vec![
        make_row("src/main.rs", "core", 100),
        make_row("src/lib.rs", "core", 200),
    ]);
    let commits = vec![
        make_commit(1000, "a@x.com", "feat: init", &["src/main.rs"]),
        make_commit(2000, "b@x.com", "fix: bug", &["src/main.rs", "src/lib.rs"]),
    ];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    assert_eq!(report.commits_scanned, 2);
    assert_eq!(report.files_seen, 2);
}

// ===========================================================================
// 2. Hotspot detection
// ===========================================================================

#[test]
fn hotspots_sorted_by_score_descending() {
    let export = make_export(vec![
        make_row("small.rs", "core", 10),
        make_row("big.rs", "core", 1000),
    ]);
    // small.rs: 10 lines * 5 commits = 50
    // big.rs: 1000 lines * 1 commit = 1000
    let commits: Vec<GitCommit> = (0..5)
        .map(|i| make_commit(i * 1000, "a@x.com", "c", &["small.rs"]))
        .chain(std::iter::once(make_commit(
            6000,
            "a@x.com",
            "c",
            &["big.rs"],
        )))
        .collect();
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    assert!(report.hotspots.len() == 2);
    // big.rs (score=1000) should come before small.rs (score=50)
    assert_eq!(report.hotspots[0].path, "big.rs");
    assert_eq!(report.hotspots[0].score, 1000);
    assert_eq!(report.hotspots[1].path, "small.rs");
    assert_eq!(report.hotspots[1].score, 50);
}

#[test]
fn hotspot_score_is_lines_times_commits() {
    let export = make_export(vec![make_row("f.rs", "core", 42)]);
    let commits = vec![
        make_commit(1000, "a@x.com", "c1", &["f.rs"]),
        make_commit(2000, "a@x.com", "c2", &["f.rs"]),
        make_commit(3000, "a@x.com", "c3", &["f.rs"]),
    ];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    assert_eq!(report.hotspots[0].score, 42 * 3);
    assert_eq!(report.hotspots[0].commits, 3);
    assert_eq!(report.hotspots[0].lines, 42);
}

#[test]
fn files_outside_export_not_in_hotspots() {
    let export = make_export(vec![make_row("tracked.rs", "core", 50)]);
    let commits = vec![make_commit(
        1000,
        "a@x.com",
        "c",
        &["tracked.rs", "untracked.rs"],
    )];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    assert_eq!(report.hotspots.len(), 1);
    assert_eq!(report.hotspots[0].path, "tracked.rs");
}

// ===========================================================================
// 3. Bus factor
// ===========================================================================

#[test]
fn bus_factor_counts_unique_authors() {
    let export = make_export(vec![make_row("f.rs", "core", 100)]);
    let commits = vec![
        make_commit(1000, "alice@x.com", "c1", &["f.rs"]),
        make_commit(2000, "bob@x.com", "c2", &["f.rs"]),
        make_commit(3000, "alice@x.com", "c3", &["f.rs"]),
    ];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    let core_bf = report
        .bus_factor
        .iter()
        .find(|b| b.module == "core")
        .unwrap();
    assert_eq!(core_bf.authors, 2); // alice + bob
}

#[test]
fn bus_factor_sorted_by_author_count_ascending() {
    let export = make_export(vec![
        make_row("a.rs", "alpha", 50),
        make_row("b.rs", "beta", 50),
    ]);
    let commits = vec![
        make_commit(1000, "alice@x.com", "c1", &["a.rs"]),
        make_commit(2000, "bob@x.com", "c2", &["a.rs"]),
        make_commit(3000, "alice@x.com", "c3", &["b.rs"]),
    ];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    // beta has 1 author, alpha has 2
    assert!(report.bus_factor.len() == 2);
    assert_eq!(report.bus_factor[0].module, "beta"); // 1 author first
    assert_eq!(report.bus_factor[1].module, "alpha"); // 2 authors
}

// ===========================================================================
// 4. Coupling analysis
// ===========================================================================

#[test]
fn coupling_detected_between_co_changed_modules() {
    let export = make_export(vec![
        make_row("a.rs", "mod_a", 50),
        make_row("b.rs", "mod_b", 50),
    ]);
    // Both files changed together in one commit
    let commits = vec![make_commit(1000, "a@x.com", "c", &["a.rs", "b.rs"])];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    assert_eq!(report.coupling.len(), 1);
    assert_eq!(report.coupling[0].count, 1);
    assert!(report.coupling[0].jaccard.is_some());
}

#[test]
fn no_coupling_when_modules_never_co_change() {
    let export = make_export(vec![
        make_row("a.rs", "mod_a", 50),
        make_row("b.rs", "mod_b", 50),
    ]);
    let commits = vec![
        make_commit(1000, "a@x.com", "c1", &["a.rs"]),
        make_commit(2000, "a@x.com", "c2", &["b.rs"]),
    ];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    assert!(
        report.coupling.is_empty(),
        "modules that never co-change should have no coupling"
    );
}

#[test]
fn coupling_sorted_by_count_descending() {
    let export = make_export(vec![
        make_row("a.rs", "mod_a", 50),
        make_row("b.rs", "mod_b", 50),
        make_row("c.rs", "mod_c", 50),
    ]);
    // mod_a + mod_b co-change 3 times, mod_a + mod_c co-change 1 time
    let commits = vec![
        make_commit(1000, "x@x.com", "c1", &["a.rs", "b.rs"]),
        make_commit(2000, "x@x.com", "c2", &["a.rs", "b.rs"]),
        make_commit(3000, "x@x.com", "c3", &["a.rs", "b.rs"]),
        make_commit(4000, "x@x.com", "c4", &["a.rs", "c.rs"]),
    ];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    assert!(report.coupling.len() >= 2);
    assert!(report.coupling[0].count >= report.coupling[1].count);
}

#[test]
fn coupling_jaccard_within_valid_range() {
    let export = make_export(vec![
        make_row("a.rs", "mod_a", 50),
        make_row("b.rs", "mod_b", 50),
    ]);
    let commits = vec![
        make_commit(1000, "x@x.com", "c1", &["a.rs", "b.rs"]),
        make_commit(2000, "x@x.com", "c2", &["a.rs"]),
    ];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    for row in &report.coupling {
        if let Some(j) = row.jaccard {
            assert!(j > 0.0 && j <= 1.0, "jaccard {j} out of range");
        }
    }
}

// ===========================================================================
// 5. Freshness metrics
// ===========================================================================

#[test]
fn freshness_all_recent_zero_stale() {
    let now = 1_700_000_000i64;
    let export = make_export(vec![make_row("f.rs", "core", 100)]);
    let commits = vec![make_commit(now, "a@x.com", "c", &["f.rs"])];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    assert_eq!(report.freshness.stale_files, 0);
    assert_eq!(report.freshness.stale_pct, 0.0);
}

#[test]
fn freshness_old_file_counted_as_stale() {
    let now = 1_700_000_000i64;
    let old = now - (400 * SECONDS_PER_DAY); // 400 days ago > 365 threshold
    let export = make_export(vec![
        make_row("old.rs", "core", 50),
        make_row("new.rs", "core", 50),
    ]);
    // Need a recent commit so max_ts is "now" (freshness is relative to max_ts)
    let commits = vec![
        make_commit(old, "a@x.com", "c1", &["old.rs"]),
        make_commit(now, "a@x.com", "c2", &["new.rs"]),
    ];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    assert_eq!(report.freshness.stale_files, 1);
    assert!(report.freshness.stale_pct > 0.0);
}

#[test]
fn freshness_total_files_matches_tracked() {
    let export = make_export(vec![
        make_row("a.rs", "core", 10),
        make_row("b.rs", "core", 20),
    ]);
    let commits = vec![
        make_commit(1000, "a@x.com", "c1", &["a.rs"]),
        make_commit(2000, "a@x.com", "c2", &["b.rs"]),
    ];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    assert_eq!(report.freshness.total_files, 2);
}

// ===========================================================================
// 6. Intent report
// ===========================================================================

#[test]
fn intent_report_counts_conventional_commits() {
    let export = make_export(vec![make_row("f.rs", "core", 100)]);
    let commits = vec![
        make_commit(1000, "a@x.com", "feat: new feature", &["f.rs"]),
        make_commit(2000, "a@x.com", "fix: bug fix", &["f.rs"]),
        make_commit(3000, "a@x.com", "docs: update docs", &["f.rs"]),
    ];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    let intent = report.intent.unwrap();
    assert_eq!(intent.overall.feat, 1);
    assert_eq!(intent.overall.fix, 1);
    assert_eq!(intent.overall.docs, 1);
    assert_eq!(intent.overall.total, 3);
}

#[test]
fn intent_report_corrective_ratio() {
    let export = make_export(vec![make_row("f.rs", "core", 100)]);
    let commits = vec![
        make_commit(1000, "a@x.com", "feat: add", &["f.rs"]),
        make_commit(2000, "a@x.com", "fix: bug", &["f.rs"]),
        make_commit(3000, "a@x.com", "fix: another", &["f.rs"]),
        make_commit(4000, "a@x.com", "revert: undo", &["f.rs"]),
    ];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    let intent = report.intent.unwrap();
    // corrective = (fix + revert) / total = (2 + 1) / 4 = 0.75
    let ratio = intent.corrective_ratio.unwrap();
    assert!((ratio - 0.75).abs() < 0.01);
}

#[test]
fn intent_report_unknown_pct() {
    let export = make_export(vec![make_row("f.rs", "core", 100)]);
    let commits = vec![
        make_commit(1000, "a@x.com", "Initial commit", &["f.rs"]),
        make_commit(2000, "a@x.com", "WIP", &["f.rs"]),
    ];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    let intent = report.intent.unwrap();
    assert_eq!(intent.overall.other, 2);
    assert!((intent.unknown_pct - 1.0).abs() < 0.01);
}

// ===========================================================================
// 7. Predictive churn
// ===========================================================================

#[test]
fn churn_report_empty_commits() {
    let export = make_export(vec![make_row("f.rs", "core", 100)]);
    let report = build_predictive_churn_report(&export, &[], Path::new("."));
    assert!(report.per_module.is_empty());
}

#[test]
fn churn_report_single_module_flat_slope() {
    let export = make_export(vec![make_row("f.rs", "core", 100)]);
    // Uniform activity across weeks
    let commits: Vec<GitCommit> = (1..=4)
        .map(|w| make_commit(w * SECONDS_PER_WEEK, "a@x.com", "c", &["f.rs"]))
        .collect();
    let report = build_predictive_churn_report(&export, &commits, Path::new("."));
    let trend = report.per_module.get("core").unwrap();
    // Flat: one commit per week → slope ≈ 0
    assert!(
        trend.slope.abs() < 0.1,
        "uniform activity should have near-zero slope, got {}",
        trend.slope
    );
}

#[test]
fn churn_report_rising_trend() {
    let export = make_export(vec![make_row("f.rs", "core", 100)]);
    // Increasing activity: 1 commit week 1, 2 week 2, 3 week 3
    let mut commits = vec![];
    for w in 1..=3i64 {
        for _ in 0..w {
            commits.push(make_commit(
                w * SECONDS_PER_WEEK + commits.len() as i64,
                "a@x.com",
                "c",
                &["f.rs"],
            ));
        }
    }
    let report = build_predictive_churn_report(&export, &commits, Path::new("."));
    let trend = report.per_module.get("core").unwrap();
    assert_eq!(trend.classification, TrendClass::Rising);
}

#[test]
fn churn_report_falling_trend() {
    let export = make_export(vec![make_row("f.rs", "core", 100)]);
    // Decreasing activity: 3 commits week 1, 2 week 2, 1 week 3
    let mut commits = vec![];
    for w in 1..=3i64 {
        let count = 4 - w; // 3, 2, 1
        for j in 0..count {
            commits.push(make_commit(
                w * SECONDS_PER_WEEK + j,
                "a@x.com",
                "c",
                &["f.rs"],
            ));
        }
    }
    let report = build_predictive_churn_report(&export, &commits, Path::new("."));
    let trend = report.per_module.get("core").unwrap();
    assert_eq!(trend.classification, TrendClass::Falling);
}

#[test]
fn churn_report_r2_in_valid_range() {
    let export = make_export(vec![make_row("f.rs", "core", 100)]);
    let commits: Vec<GitCommit> = (1..=5)
        .map(|w| make_commit(w * SECONDS_PER_WEEK, "a@x.com", "c", &["f.rs"]))
        .collect();
    let report = build_predictive_churn_report(&export, &commits, Path::new("."));
    if let Some(trend) = report.per_module.get("core") {
        assert!(
            trend.r2 >= 0.0 && trend.r2 <= 1.0,
            "r2 should be in [0,1], got {}",
            trend.r2
        );
    }
}

#[test]
fn churn_report_child_rows_excluded() {
    let rows = vec![FileRow {
        path: "f.rs".to_string(),
        module: "core".to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Child, // should be filtered out
        code: 100,
        comments: 0,
        blanks: 0,
        lines: 100,
        bytes: 1000,
        tokens: 500,
    }];
    let export = make_export(rows);
    let commits = vec![make_commit(SECONDS_PER_WEEK, "a@x.com", "c", &["f.rs"])];
    let report = build_predictive_churn_report(&export, &commits, Path::new("."));
    assert!(
        report.per_module.is_empty(),
        "child rows should not produce churn data"
    );
}

// ===========================================================================
// 8. Age distribution
// ===========================================================================

#[test]
fn age_distribution_present_in_report() {
    let now = 1_700_000_000i64;
    let export = make_export(vec![make_row("f.rs", "core", 100)]);
    let commits = vec![make_commit(now, "a@x.com", "c", &["f.rs"])];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    let age = report.age_distribution.unwrap();
    assert_eq!(age.buckets.len(), 5); // 5 standard buckets
    let total_pct: f64 = age.buckets.iter().map(|b| b.pct).sum();
    assert!(
        (total_pct - 1.0).abs() < 0.01,
        "bucket percentages should sum to ~1.0, got {total_pct}"
    );
}

// ===========================================================================
// 9. Edge cases
// ===========================================================================

#[test]
fn git_report_with_no_matching_files() {
    let export = make_export(vec![make_row("tracked.rs", "core", 100)]);
    let commits = vec![make_commit(1000, "a@x.com", "c", &["untracked.rs"])];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    assert_eq!(report.commits_scanned, 1);
    assert_eq!(report.files_seen, 0);
    assert!(report.hotspots.is_empty());
}

#[test]
fn git_report_deterministic_output() {
    let export = make_export(vec![
        make_row("b.rs", "beta", 200),
        make_row("a.rs", "alpha", 100),
    ]);
    let commits = vec![
        make_commit(1000, "x@x.com", "c1", &["a.rs", "b.rs"]),
        make_commit(2000, "y@x.com", "c2", &["a.rs"]),
    ];
    let r1 = build_git_report(Path::new("."), &export, &commits).unwrap();
    let r2 = build_git_report(Path::new("."), &export, &commits).unwrap();
    // Hotspot order should be deterministic
    assert_eq!(r1.hotspots.len(), r2.hotspots.len());
    for (a, b) in r1.hotspots.iter().zip(r2.hotspots.iter()) {
        assert_eq!(a.path, b.path);
        assert_eq!(a.score, b.score);
    }
    // Bus factor order should be deterministic
    for (a, b) in r1.bus_factor.iter().zip(r2.bus_factor.iter()) {
        assert_eq!(a.module, b.module);
        assert_eq!(a.authors, b.authors);
    }
}
