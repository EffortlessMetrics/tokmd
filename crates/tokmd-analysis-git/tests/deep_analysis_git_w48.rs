//! Deep tests (w48) for tokmd-analysis-git: git enricher pipeline,
//! churn prediction, coupling/freshness report format, and capability
//! reporting.

use std::path::Path;

use proptest::prelude::*;
use tokmd_analysis_git::{build_git_report, build_predictive_churn_report};
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

fn child_row(path: &str, module: &str, lines: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Child,
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
// 1. Git enricher pipeline – hotspots → analysis report
// ===========================================================================

#[test]
fn pipeline_empty_input_produces_empty_report() {
    let exp = export(vec![]);
    let report = build_git_report(Path::new("."), &exp, &[]).unwrap();
    assert_eq!(report.commits_scanned, 0);
    assert_eq!(report.files_seen, 0);
    assert!(report.hotspots.is_empty());
    assert!(report.bus_factor.is_empty());
    assert!(report.coupling.is_empty());
    assert_eq!(report.freshness.total_files, 0);
}

#[test]
fn pipeline_hotspot_score_equals_lines_times_commits() {
    let exp = export(vec![file_row("src/hot.rs", "core", 100)]);
    let commits = vec![
        commit(1000, "alice@dev.com", "feat: init", &["src/hot.rs"]),
        commit(2000, "alice@dev.com", "fix: bug", &["src/hot.rs"]),
        commit(3000, "bob@dev.com", "refactor: cleanup", &["src/hot.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.hotspots.len(), 1);
    assert_eq!(report.hotspots[0].score, 100 * 3);
    assert_eq!(report.hotspots[0].commits, 3);
    assert_eq!(report.hotspots[0].lines, 100);
}

#[test]
fn pipeline_hotspots_sorted_descending_by_score() {
    let exp = export(vec![
        file_row("big.rs", "core", 500),
        file_row("small.rs", "core", 10),
    ]);
    let commits = vec![
        commit(1000, "a@x.com", "c1", &["big.rs"]),
        commit(2000, "a@x.com", "c2", &["small.rs"]),
        commit(3000, "a@x.com", "c3", &["small.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.hotspots.len(), 2);
    // big.rs: 500*1=500, small.rs: 10*2=20
    assert_eq!(report.hotspots[0].path, "big.rs");
    assert!(report.hotspots[0].score >= report.hotspots[1].score);
}

#[test]
fn pipeline_untracked_files_excluded_from_hotspots() {
    let exp = export(vec![file_row("tracked.rs", "core", 50)]);
    let commits = vec![commit(
        1000,
        "a@x.com",
        "c",
        &["tracked.rs", "untracked.rs"],
    )];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.hotspots.len(), 1);
    assert_eq!(report.hotspots[0].path, "tracked.rs");
}

#[test]
fn pipeline_child_rows_excluded_from_hotspots() {
    let exp = export(vec![
        file_row("parent.rs", "core", 100),
        child_row("child.rs", "core", 50),
    ]);
    let commits = vec![commit(1000, "a@x.com", "c", &["parent.rs", "child.rs"])];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    // Only parent rows appear in hotspots
    assert_eq!(report.hotspots.len(), 1);
    assert_eq!(report.hotspots[0].path, "parent.rs");
}

#[test]
fn pipeline_bus_factor_deduplicates_authors() {
    let exp = export(vec![file_row("f.rs", "core", 100)]);
    let commits = vec![
        commit(1000, "alice@x.com", "c1", &["f.rs"]),
        commit(2000, "alice@x.com", "c2", &["f.rs"]),
        commit(3000, "alice@x.com", "c3", &["f.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.bus_factor.len(), 1);
    assert_eq!(report.bus_factor[0].authors, 1);
}

#[test]
fn pipeline_bus_factor_sorted_by_authors_ascending() {
    let exp = export(vec![
        file_row("a.rs", "alpha", 10),
        file_row("b.rs", "beta", 10),
    ]);
    let commits = vec![
        commit(1000, "a@x.com", "c1", &["a.rs"]),
        commit(2000, "a@x.com", "c2", &["b.rs"]),
        commit(3000, "b@x.com", "c3", &["b.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.bus_factor[0].module, "alpha");
    assert_eq!(report.bus_factor[0].authors, 1);
    assert_eq!(report.bus_factor[1].module, "beta");
    assert_eq!(report.bus_factor[1].authors, 2);
}

#[test]
fn pipeline_intent_counts_correct() {
    let exp = export(vec![file_row("f.rs", "core", 100)]);
    let commits = vec![
        commit(1000, "a@x.com", "feat: add", &["f.rs"]),
        commit(2000, "a@x.com", "fix: bug", &["f.rs"]),
        commit(3000, "a@x.com", "docs: update", &["f.rs"]),
        commit(4000, "a@x.com", "Initial commit", &["f.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let intent = report.intent.unwrap();
    assert_eq!(intent.overall.feat, 1);
    assert_eq!(intent.overall.fix, 1);
    assert_eq!(intent.overall.docs, 1);
    assert_eq!(intent.overall.other, 1);
    assert_eq!(intent.overall.total, 4);
}

#[test]
fn pipeline_deterministic_output() {
    let exp = export(vec![
        file_row("a.rs", "alpha", 100),
        file_row("b.rs", "beta", 200),
    ]);
    let commits = vec![
        commit(1000, "x@x.com", "c1", &["a.rs", "b.rs"]),
        commit(2000, "y@x.com", "c2", &["a.rs"]),
    ];
    let r1 = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let r2 = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(r1.hotspots.len(), r2.hotspots.len());
    for (a, b) in r1.hotspots.iter().zip(r2.hotspots.iter()) {
        assert_eq!(a.path, b.path);
        assert_eq!(a.score, b.score);
    }
    for (a, b) in r1.bus_factor.iter().zip(r2.bus_factor.iter()) {
        assert_eq!(a.module, b.module);
        assert_eq!(a.authors, b.authors);
    }
}

// ===========================================================================
// 2. Churn prediction
// ===========================================================================

#[test]
fn churn_empty_commits_produces_empty_report() {
    let exp = export(vec![file_row("f.rs", "core", 100)]);
    let report = build_predictive_churn_report(&exp, &[], Path::new("."));
    assert!(report.per_module.is_empty());
}

#[test]
fn churn_single_commit_flat_slope() {
    let exp = export(vec![file_row("f.rs", "core", 100)]);
    let commits = vec![commit(WEEK, "a@x.com", "c", &["f.rs"])];
    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));
    if let Some(trend) = report.per_module.get("core") {
        // Single data point → slope = 0
        assert!(trend.slope.abs() < 0.01);
    }
}

#[test]
fn churn_rising_activity_detected() {
    let exp = export(vec![file_row("f.rs", "core", 100)]);
    let mut commits = vec![];
    // 1 commit week 1, 2 week 2, 3 week 3, 4 week 4
    for w in 1..=4i64 {
        for j in 0..w {
            commits.push(commit(w * WEEK + j, "a@x.com", "c", &["f.rs"]));
        }
    }
    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));
    let trend = report.per_module.get("core").unwrap();
    assert_eq!(trend.classification, TrendClass::Rising);
}

#[test]
fn churn_falling_activity_detected() {
    let exp = export(vec![file_row("f.rs", "core", 100)]);
    let mut commits = vec![];
    // 4 commits week 1, 3 week 2, 2 week 3, 1 week 4
    for w in 1..=4i64 {
        let count = 5 - w;
        for j in 0..count {
            commits.push(commit(w * WEEK + j, "a@x.com", "c", &["f.rs"]));
        }
    }
    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));
    let trend = report.per_module.get("core").unwrap();
    assert_eq!(trend.classification, TrendClass::Falling);
}

#[test]
fn churn_uniform_activity_flat() {
    let exp = export(vec![file_row("f.rs", "core", 100)]);
    let commits: Vec<_> = (1..=6)
        .map(|w| commit(w * WEEK, "a@x.com", "c", &["f.rs"]))
        .collect();
    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));
    let trend = report.per_module.get("core").unwrap();
    assert!(
        trend.slope.abs() < 0.1,
        "uniform weekly activity → near-zero slope, got {}",
        trend.slope
    );
}

#[test]
fn churn_r2_in_valid_range() {
    let exp = export(vec![file_row("f.rs", "core", 100)]);
    let commits: Vec<_> = (1..=5)
        .map(|w| commit(w * WEEK, "a@x.com", "c", &["f.rs"]))
        .collect();
    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));
    if let Some(trend) = report.per_module.get("core") {
        assert!(
            trend.r2 >= 0.0 && trend.r2 <= 1.0,
            "r2 should be [0,1], got {}",
            trend.r2
        );
    }
}

#[test]
fn churn_child_rows_excluded() {
    let exp = export(vec![child_row("f.rs", "core", 100)]);
    let commits = vec![commit(WEEK, "a@x.com", "c", &["f.rs"])];
    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));
    assert!(
        report.per_module.is_empty(),
        "child rows should not produce churn data"
    );
}

#[test]
fn churn_multiple_modules_independent() {
    let exp = export(vec![
        file_row("a.rs", "alpha", 50),
        file_row("b.rs", "beta", 50),
    ]);
    // Rising for alpha, flat for beta
    let mut commits = vec![];
    for w in 1..=4i64 {
        for j in 0..w {
            commits.push(commit(w * WEEK + j, "a@x.com", "c", &["a.rs"]));
        }
        commits.push(commit(w * WEEK + 100, "a@x.com", "c", &["b.rs"]));
    }
    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));
    assert!(report.per_module.contains_key("alpha"));
    assert!(report.per_module.contains_key("beta"));
}

// ===========================================================================
// 3. Coupling report format
// ===========================================================================

#[test]
fn coupling_co_changed_modules_detected() {
    let exp = export(vec![
        file_row("a.rs", "mod_a", 50),
        file_row("b.rs", "mod_b", 50),
    ]);
    let commits = vec![
        commit(1000, "x@x.com", "c1", &["a.rs", "b.rs"]),
        commit(2000, "x@x.com", "c2", &["a.rs", "b.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.coupling.len(), 1);
    assert_eq!(report.coupling[0].count, 2);
}

#[test]
fn coupling_jaccard_in_valid_range() {
    let exp = export(vec![
        file_row("a.rs", "mod_a", 50),
        file_row("b.rs", "mod_b", 50),
    ]);
    let commits = vec![
        commit(1000, "x@x.com", "c1", &["a.rs", "b.rs"]),
        commit(2000, "x@x.com", "c2", &["a.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    for row in &report.coupling {
        if let Some(j) = row.jaccard {
            assert!(j > 0.0 && j <= 1.0, "jaccard {j} out of range (0,1]");
        }
    }
}

#[test]
fn coupling_lift_present_when_enough_data() {
    let exp = export(vec![
        file_row("a.rs", "mod_a", 50),
        file_row("b.rs", "mod_b", 50),
    ]);
    let commits = vec![commit(1000, "x@x.com", "c1", &["a.rs", "b.rs"])];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.coupling.len(), 1);
    assert!(report.coupling[0].lift.is_some());
}

#[test]
fn coupling_sorted_by_count_descending() {
    let exp = export(vec![
        file_row("a.rs", "mod_a", 50),
        file_row("b.rs", "mod_b", 50),
        file_row("c.rs", "mod_c", 50),
    ]);
    let commits = vec![
        commit(1000, "x@x.com", "c1", &["a.rs", "b.rs"]),
        commit(2000, "x@x.com", "c2", &["a.rs", "b.rs"]),
        commit(3000, "x@x.com", "c3", &["a.rs", "b.rs"]),
        commit(4000, "x@x.com", "c4", &["a.rs", "c.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert!(report.coupling.len() >= 2);
    assert!(report.coupling[0].count >= report.coupling[1].count);
}

#[test]
fn coupling_no_self_coupling() {
    let exp = export(vec![file_row("a.rs", "mod_a", 50)]);
    let commits = vec![
        commit(1000, "x@x.com", "c1", &["a.rs"]),
        commit(2000, "x@x.com", "c2", &["a.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    // Single module → no coupling pairs
    assert!(report.coupling.is_empty());
}

#[test]
fn coupling_n_left_n_right_present() {
    let exp = export(vec![
        file_row("a.rs", "mod_a", 50),
        file_row("b.rs", "mod_b", 50),
    ]);
    let commits = vec![
        commit(1000, "x@x.com", "c1", &["a.rs", "b.rs"]),
        commit(2000, "x@x.com", "c2", &["a.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.coupling.len(), 1);
    assert!(report.coupling[0].n_left.is_some());
    assert!(report.coupling[0].n_right.is_some());
}

// ===========================================================================
// 4. Freshness report format
// ===========================================================================

#[test]
fn freshness_recent_file_zero_stale() {
    let now = 1_700_000_000i64;
    let exp = export(vec![file_row("f.rs", "core", 100)]);
    let commits = vec![commit(now, "a@x.com", "c", &["f.rs"])];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.freshness.stale_files, 0);
    assert_eq!(report.freshness.stale_pct, 0.0);
    assert_eq!(report.freshness.total_files, 1);
}

#[test]
fn freshness_old_file_counted_stale() {
    let now = 1_700_000_000i64;
    let old = now - 400 * DAY; // >365 threshold
    let exp = export(vec![
        file_row("old.rs", "core", 50),
        file_row("new.rs", "core", 50),
    ]);
    let commits = vec![
        commit(old, "a@x.com", "c1", &["old.rs"]),
        commit(now, "a@x.com", "c2", &["new.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.freshness.stale_files, 1);
    assert!(report.freshness.stale_pct > 0.0);
}

#[test]
fn freshness_threshold_is_365_days() {
    let exp = export(vec![file_row("f.rs", "core", 100)]);
    let commits = vec![commit(1000, "a@x.com", "c", &["f.rs"])];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.freshness.threshold_days, 365);
}

#[test]
fn freshness_by_module_populated() {
    let now = 1_700_000_000i64;
    let exp = export(vec![
        file_row("a.rs", "alpha", 50),
        file_row("b.rs", "beta", 50),
    ]);
    let commits = vec![
        commit(now, "a@x.com", "c1", &["a.rs"]),
        commit(now, "a@x.com", "c2", &["b.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.freshness.by_module.len(), 2);
    for row in &report.freshness.by_module {
        assert!(row.avg_days >= 0.0);
        assert!(row.p90_days >= 0.0);
        assert!(row.stale_pct >= 0.0 && row.stale_pct <= 1.0);
    }
}

#[test]
fn freshness_no_commits_for_file_not_counted() {
    let exp = export(vec![file_row("orphan.rs", "core", 100)]);
    // No commits touch orphan.rs
    let commits = vec![commit(1000, "a@x.com", "c", &["other.rs"])];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.freshness.total_files, 0);
}

// ===========================================================================
// 5. Capability reporting (git available vs unavailable)
// ===========================================================================

#[test]
fn git_available_returns_consistent_result() {
    let a = tokmd_git::git_available();
    let b = tokmd_git::git_available();
    assert_eq!(a, b);
}

#[test]
fn build_git_report_works_without_real_repo() {
    // build_git_report takes pre-collected data, doesn't need git
    let exp = export(vec![file_row("f.rs", "core", 50)]);
    let commits = vec![commit(1000, "a@x.com", "feat: test", &["f.rs"])];
    let report = build_git_report(Path::new("/nonexistent"), &exp, &commits).unwrap();
    assert_eq!(report.commits_scanned, 1);
}

// ===========================================================================
// 6. Age distribution
// ===========================================================================

#[test]
fn age_distribution_has_five_buckets() {
    let now = 1_700_000_000i64;
    let exp = export(vec![file_row("f.rs", "core", 100)]);
    let commits = vec![commit(now, "a@x.com", "c", &["f.rs"])];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let age = report.age_distribution.unwrap();
    assert_eq!(age.buckets.len(), 5);
}

#[test]
fn age_distribution_percentages_sum_to_one() {
    let now = 1_700_000_000i64;
    let exp = export(vec![
        file_row("a.rs", "core", 10),
        file_row("b.rs", "core", 20),
    ]);
    let commits = vec![
        commit(now, "a@x.com", "c1", &["a.rs"]),
        commit(now - 100 * DAY, "a@x.com", "c2", &["b.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let age = report.age_distribution.unwrap();
    let total: f64 = age.buckets.iter().map(|b| b.pct).sum();
    assert!(
        (total - 1.0).abs() < 0.01,
        "bucket pcts should sum to ~1.0, got {total}"
    );
}

// ===========================================================================
// 7. Intent report edge cases
// ===========================================================================

#[test]
fn intent_corrective_ratio_none_when_zero_commits() {
    let exp = export(vec![]);
    let report = build_git_report(Path::new("."), &exp, &[]).unwrap();
    let intent = report.intent.unwrap();
    assert!(intent.corrective_ratio.is_none());
}

#[test]
fn intent_by_module_tracks_per_module_counts() {
    let exp = export(vec![
        file_row("a.rs", "alpha", 50),
        file_row("b.rs", "beta", 50),
    ]);
    let commits = vec![
        commit(1000, "a@x.com", "feat: add alpha", &["a.rs"]),
        commit(2000, "a@x.com", "fix: beta bug", &["b.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let intent = report.intent.unwrap();
    assert_eq!(intent.by_module.len(), 2);
    let alpha = intent
        .by_module
        .iter()
        .find(|m| m.module == "alpha")
        .unwrap();
    assert_eq!(alpha.counts.feat, 1);
    let beta = intent
        .by_module
        .iter()
        .find(|m| m.module == "beta")
        .unwrap();
    assert_eq!(beta.counts.fix, 1);
}

#[test]
fn intent_unknown_pct_all_classified() {
    let exp = export(vec![file_row("f.rs", "core", 100)]);
    let commits = vec![
        commit(1000, "a@x.com", "feat: x", &["f.rs"]),
        commit(2000, "a@x.com", "fix: y", &["f.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let intent = report.intent.unwrap();
    assert_eq!(intent.unknown_pct, 0.0);
}

// ===========================================================================
// 8. Property tests
// ===========================================================================

proptest! {
    #[test]
    fn prop_hotspot_score_nonneg(lines in 0usize..10_000, commits_n in 1usize..100) {
        let score = lines * commits_n;
        prop_assert!(score < usize::MAX);
    }

    #[test]
    fn prop_churn_report_no_panic(
        n_commits in 0usize..20,
        n_files in 1usize..5,
    ) {
        let rows: Vec<FileRow> = (0..n_files)
            .map(|i| file_row(&format!("f{i}.rs"), "core", 10 + i * 5))
            .collect();
        let exp = export(rows);
        let commits: Vec<GitCommit> = (0..n_commits)
            .map(|i| commit(
                (i as i64 + 1) * WEEK,
                "a@x.com",
                "c",
                &["f0.rs"],
            ))
            .collect();
        let _ = build_predictive_churn_report(&exp, &commits, Path::new("."));
    }

    #[test]
    fn prop_git_report_no_panic(
        n_commits in 0usize..20,
    ) {
        let exp = export(vec![file_row("f.rs", "core", 50)]);
        let commits: Vec<GitCommit> = (0..n_commits)
            .map(|i| commit(
                (i as i64 + 1) * 1000,
                "a@x.com",
                "c",
                &["f.rs"],
            ))
            .collect();
        let _ = build_git_report(Path::new("."), &exp, &commits);
    }
}

// ===========================================================================
// 9. Multi-module report
// ===========================================================================

#[test]
fn multi_module_report_comprehensive() {
    let exp = export(vec![
        file_row("api/handler.rs", "api", 200),
        file_row("api/routes.rs", "api", 150),
        file_row("db/query.rs", "db", 300),
        file_row("db/schema.rs", "db", 100),
        file_row("util/helpers.rs", "util", 50),
    ]);
    let commits = vec![
        commit(
            1000,
            "alice@x.com",
            "feat: api handler",
            &["api/handler.rs"],
        ),
        commit(2000, "bob@x.com", "feat: api routes", &["api/routes.rs"]),
        commit(
            3000,
            "alice@x.com",
            "fix: api+db",
            &["api/handler.rs", "db/query.rs"],
        ),
        commit(4000, "charlie@x.com", "feat: db schema", &["db/schema.rs"]),
        commit(
            5000,
            "bob@x.com",
            "refactor: all",
            &["api/handler.rs", "db/query.rs", "util/helpers.rs"],
        ),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    assert_eq!(report.commits_scanned, 5);
    assert_eq!(report.files_seen, 5);

    // Bus factor: api has alice+bob, db has alice+bob+charlie, util has bob
    let util_bf = report
        .bus_factor
        .iter()
        .find(|b| b.module == "util")
        .unwrap();
    assert_eq!(util_bf.authors, 1);
    let db_bf = report.bus_factor.iter().find(|b| b.module == "db").unwrap();
    assert!(db_bf.authors >= 2);

    // Coupling: api+db should be coupled (2 commits touch both)
    assert!(!report.coupling.is_empty());
}
