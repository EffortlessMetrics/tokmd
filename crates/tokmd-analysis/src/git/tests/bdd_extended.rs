//! Extended BDD-style scenario tests for ``tokmd-analysis` Git module`.
//!
//! Additional coverage for churn falling trend, coupling sort order,
//! intent unknown percentage, bus factor sorting, and freshness edge cases.

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
// Scenario: Churn with decreasing activity has falling trend
// ===========================================================================
#[test]
fn scenario_churn_decreasing_activity_falling() {
    // Given: decreasing commits per week: 5, 4, 3, 2, 1
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let mut commits = Vec::new();
    for w in 1..=5i64 {
        let count = 6 - w; // 5, 4, 3, 2, 1
        for _ in 0..count {
            commits.push(commit(w * WEEK, "alice", "feat: less", &["src/lib.rs"]));
        }
    }

    // When
    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));

    // Then: negative slope → falling
    let trend = report.per_module.get("src").expect("module present");
    assert!(
        trend.slope < 0.0,
        "decreasing activity should have negative slope, got {}",
        trend.slope
    );
    assert_eq!(trend.classification, TrendClass::Falling);
}

// ===========================================================================
// Scenario: Churn with multiple modules tracks each independently
// ===========================================================================
#[test]
fn scenario_churn_multiple_modules_independent() {
    // Given: two modules, one active and one dormant
    let exp = export(vec![
        file_row("api/handler.rs", "api", 100),
        file_row("db/query.rs", "db", 80),
    ]);
    let commits = vec![
        commit(WEEK, "alice", "feat: api1", &["api/handler.rs"]),
        commit(2 * WEEK, "alice", "feat: api2", &["api/handler.rs"]),
        commit(3 * WEEK, "alice", "feat: api3", &["api/handler.rs"]),
        commit(WEEK, "bob", "feat: db", &["db/query.rs"]),
    ];

    // When
    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));

    // Then: both modules tracked
    assert!(report.per_module.contains_key("api"));
    assert!(report.per_module.contains_key("db"));
    // db has only 1 data point → flat
    let db_trend = report.per_module.get("db").unwrap();
    assert_eq!(db_trend.slope, 0.0);
    assert_eq!(db_trend.classification, TrendClass::Flat);
}

// ===========================================================================
// Scenario: Coupling rows sorted by count descending
// ===========================================================================
#[test]
fn scenario_coupling_sorted_by_count_desc() {
    // Given: three modules with different coupling strengths
    let exp = export(vec![
        file_row("a/f.rs", "a", 50),
        file_row("b/f.rs", "b", 50),
        file_row("c/f.rs", "c", 50),
    ]);
    let commits = vec![
        // a+b coupled 3 times
        commit(1000, "alice", "feat: ab1", &["a/f.rs", "b/f.rs"]),
        commit(2000, "alice", "feat: ab2", &["a/f.rs", "b/f.rs"]),
        commit(3000, "alice", "feat: ab3", &["a/f.rs", "b/f.rs"]),
        // a+c coupled 1 time
        commit(4000, "bob", "feat: ac1", &["a/f.rs", "c/f.rs"]),
        // b+c coupled 2 times
        commit(5000, "bob", "feat: bc1", &["b/f.rs", "c/f.rs"]),
        commit(6000, "bob", "feat: bc2", &["b/f.rs", "c/f.rs"]),
    ];

    // When
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    // Then: coupling sorted by count desc: a-b(3), b-c(2), a-c(1)
    assert_eq!(report.coupling.len(), 3);
    assert_eq!(report.coupling[0].count, 3);
    assert_eq!(report.coupling[1].count, 2);
    assert_eq!(report.coupling[2].count, 1);
}

// ===========================================================================
// Scenario: Intent with unconventional messages yields high unknown_pct
// ===========================================================================
#[test]
fn scenario_intent_unconventional_messages_high_unknown_pct() {
    // Given: commits with non-conventional messages
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits = vec![
        commit(1000, "alice", "updated the thing", &["src/lib.rs"]),
        commit(2000, "bob", "misc changes", &["src/lib.rs"]),
        commit(3000, "charlie", "wip", &["src/lib.rs"]),
        commit(4000, "alice", "feat: one conventional", &["src/lib.rs"]),
    ];

    // When
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    // Then: 3 out of 4 are "other" → unknown_pct = 0.75
    let intent = report.intent.as_ref().expect("intent present");
    assert_eq!(intent.overall.other, 3);
    assert_eq!(intent.overall.feat, 1);
    assert_eq!(intent.overall.total, 4);
    assert_eq!(intent.unknown_pct, 0.75);
}

// ===========================================================================
// Scenario: Bus factor sorted by authors ascending then module name
// ===========================================================================
#[test]
fn scenario_bus_factor_sort_order() {
    // Given: three modules with varying author counts
    let exp = export(vec![
        file_row("z/f.rs", "z", 50),
        file_row("a/f.rs", "a", 50),
        file_row("m/f.rs", "m", 50),
    ]);
    let commits = vec![
        // z: 2 authors
        commit(1000, "alice", "feat: z", &["z/f.rs"]),
        commit(2000, "bob", "fix: z", &["z/f.rs"]),
        // a: 2 authors (same count as z, but "a" < "z" alphabetically)
        commit(3000, "charlie", "feat: a", &["a/f.rs"]),
        commit(4000, "dave", "fix: a", &["a/f.rs"]),
        // m: 1 author
        commit(5000, "eve", "feat: m", &["m/f.rs"]),
    ];

    // When
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    // Then: sorted by authors asc, then module name asc
    assert_eq!(report.bus_factor.len(), 3);
    assert_eq!(report.bus_factor[0].module, "m"); // 1 author
    assert_eq!(report.bus_factor[0].authors, 1);
    assert_eq!(report.bus_factor[1].module, "a"); // 2 authors, "a" < "z"
    assert_eq!(report.bus_factor[1].authors, 2);
    assert_eq!(report.bus_factor[2].module, "z"); // 2 authors, "z" > "a"
    assert_eq!(report.bus_factor[2].authors, 2);
}

// ===========================================================================
// Scenario: Freshness all stale files yields 100% stale
// ===========================================================================
#[test]
fn scenario_freshness_all_stale() {
    // Given: all files changed over 365 days ago
    let now = 1000 * DAY;
    let exp = export(vec![
        file_row("src/a.rs", "src", 50),
        file_row("src/b.rs", "src", 50),
    ]);
    let commits = vec![
        commit(now - 500 * DAY, "alice", "feat: a", &["src/a.rs"]),
        commit(now - 400 * DAY, "bob", "feat: b", &["src/b.rs"]),
        // Add a recent commit on an untracked file to set max_ts to now
        commit(now, "charlie", "feat: other", &["untracked.rs"]),
    ];

    // When
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    // Then: all tracked files are stale
    assert_eq!(report.freshness.stale_files, 2);
    assert_eq!(report.freshness.total_files, 2);
    assert_eq!(report.freshness.stale_pct, 1.0);
}

// ===========================================================================
// Scenario: Freshness module p90 and avg reflect data
// ===========================================================================
#[test]
fn scenario_freshness_module_p90_positive() {
    // Given: a module with files of different ages
    let now = 500 * DAY;
    let exp = export(vec![
        file_row("src/a.rs", "src", 50),
        file_row("src/b.rs", "src", 50),
        file_row("src/c.rs", "src", 50),
    ]);
    let commits = vec![
        commit(now, "alice", "feat: fresh", &["src/a.rs"]),
        commit(now - 100 * DAY, "bob", "feat: medium", &["src/b.rs"]),
        commit(now - 200 * DAY, "charlie", "feat: old", &["src/c.rs"]),
    ];

    // When
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    // Then: module freshness has positive avg_days and p90
    assert_eq!(report.freshness.by_module.len(), 1);
    let module = &report.freshness.by_module[0];
    assert_eq!(module.module, "src");
    assert!(module.avg_days > 0.0, "avg_days should be positive");
    assert!(module.p90_days > 0.0, "p90_days should be positive");
    assert!(module.p90_days >= module.avg_days, "p90 should be >= avg");
}

// ===========================================================================
// Scenario: Churn r2 is between 0.0 and 1.0
// ===========================================================================
#[test]
fn scenario_churn_r2_bounded() {
    // Given: multiple commits over several weeks
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits: Vec<GitCommit> = (1..=10)
        .map(|i| commit(i * WEEK, "alice", "feat: weekly", &["src/lib.rs"]))
        .collect();

    // When
    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));

    // Then: r2 is in [0.0, 1.0]
    let trend = report.per_module.get("src").expect("module present");
    assert!(
        trend.r2 >= 0.0 && trend.r2 <= 1.0,
        "r2 should be bounded [0,1], got {}",
        trend.r2
    );
}
