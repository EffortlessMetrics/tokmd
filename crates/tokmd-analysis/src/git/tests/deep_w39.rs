//! Deep tests for `tokmd-analysis` Git module: hotspots, coupling, freshness, churn, intent.

use std::path::Path;

use super::super::{build_git_report, build_predictive_churn_report};
use tokmd_git::GitCommit;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ─── helpers ───────────────────────────────────────────────────

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

const DAY: i64 = 86_400;

// ─── hotspot detection ─────────────────────────────────────────

#[test]
fn hotspot_score_is_lines_times_commits() {
    let export = make_export(vec![make_row("src/lib.rs", "src", 100)]);
    let commits = vec![
        make_commit(DAY, "a", "feat: init", &["src/lib.rs"]),
        make_commit(2 * DAY, "a", "fix: bug", &["src/lib.rs"]),
        make_commit(3 * DAY, "a", "refactor: clean", &["src/lib.rs"]),
    ];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    assert_eq!(report.hotspots.len(), 1);
    assert_eq!(report.hotspots[0].commits, 3);
    assert_eq!(report.hotspots[0].lines, 100);
    assert_eq!(report.hotspots[0].score, 300); // 100 * 3
}

#[test]
fn hotspots_sorted_by_score_descending() {
    let export = make_export(vec![
        make_row("src/a.rs", "src", 50),
        make_row("src/b.rs", "src", 200),
    ]);
    let commits = vec![
        make_commit(DAY, "a", "c1", &["src/a.rs", "src/b.rs"]),
        make_commit(2 * DAY, "a", "c2", &["src/a.rs"]),
    ];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    // a.rs: 50 * 2 = 100, b.rs: 200 * 1 = 200
    assert_eq!(report.hotspots[0].path, "src/b.rs");
    assert_eq!(report.hotspots[0].score, 200);
    assert_eq!(report.hotspots[1].path, "src/a.rs");
    assert_eq!(report.hotspots[1].score, 100);
}

#[test]
fn hotspot_unmatched_files_ignored() {
    let export = make_export(vec![make_row("src/lib.rs", "src", 50)]);
    let commits = vec![make_commit(DAY, "a", "c1", &["unknown.rs"])];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    assert!(report.hotspots.is_empty());
    assert_eq!(report.files_seen, 0);
}

// ─── coupling analysis ─────────────────────────────────────────

#[test]
fn coupling_detects_co_changed_modules() {
    let export = make_export(vec![
        make_row("src/a.rs", "src", 50),
        make_row("tests/t.rs", "tests", 30),
    ]);
    let commits = vec![
        make_commit(DAY, "a", "c1", &["src/a.rs", "tests/t.rs"]),
        make_commit(2 * DAY, "a", "c2", &["src/a.rs", "tests/t.rs"]),
    ];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    assert_eq!(report.coupling.len(), 1);
    assert_eq!(report.coupling[0].count, 2);
    assert_eq!(report.coupling[0].left, "src");
    assert_eq!(report.coupling[0].right, "tests");
}

#[test]
fn coupling_jaccard_and_lift_computed() {
    let export = make_export(vec![
        make_row("src/a.rs", "src", 50),
        make_row("tests/t.rs", "tests", 30),
    ]);
    // Both modules touched in every commit → perfect coupling
    let commits = vec![
        make_commit(DAY, "a", "c1", &["src/a.rs", "tests/t.rs"]),
        make_commit(2 * DAY, "a", "c2", &["src/a.rs", "tests/t.rs"]),
    ];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    assert!(report.coupling[0].jaccard.unwrap() > 0.0);
    assert!(report.coupling[0].lift.is_some());
}

#[test]
fn coupling_empty_when_single_module() {
    let export = make_export(vec![
        make_row("src/a.rs", "src", 50),
        make_row("src/b.rs", "src", 60),
    ]);
    let commits = vec![make_commit(DAY, "a", "c1", &["src/a.rs", "src/b.rs"])];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    // Both files in same module → no inter-module coupling pairs
    assert!(report.coupling.is_empty());
}

// ─── freshness calculation ─────────────────────────────────────

#[test]
fn freshness_all_recent() {
    let now = 400 * DAY;
    let export = make_export(vec![make_row("src/lib.rs", "src", 50)]);
    let commits = vec![make_commit(now - DAY, "a", "c1", &["src/lib.rs"])];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    assert_eq!(report.freshness.stale_files, 0);
    assert_eq!(report.freshness.total_files, 1);
    assert_eq!(report.freshness.stale_pct, 0.0);
}

#[test]
fn freshness_stale_files_detected() {
    let now = 800 * DAY;
    let export = make_export(vec![
        make_row("src/old.rs", "src", 50),
        make_row("src/new.rs", "src", 50),
    ]);
    let commits = vec![
        // old.rs was last changed 400 days ago (> 365 threshold)
        make_commit(now - 400 * DAY, "a", "c1", &["src/old.rs"]),
        // new.rs was changed recently
        make_commit(now, "a", "c2", &["src/new.rs"]),
    ];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    assert_eq!(report.freshness.stale_files, 1);
    assert_eq!(report.freshness.total_files, 2);
    assert_eq!(report.freshness.stale_pct, 0.5);
}

#[test]
fn freshness_module_breakdown() {
    let now = 800 * DAY;
    let export = make_export(vec![
        make_row("src/a.rs", "src", 50),
        make_row("tests/b.rs", "tests", 30),
    ]);
    let commits = vec![
        make_commit(now - 10 * DAY, "a", "c1", &["src/a.rs"]),
        make_commit(now - 500 * DAY, "b", "c2", &["tests/b.rs"]),
    ];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    assert_eq!(report.freshness.by_module.len(), 2);
    // Tests module should have high stale_pct
    let tests_row = report
        .freshness
        .by_module
        .iter()
        .find(|m| m.module == "tests")
        .unwrap();
    assert_eq!(tests_row.stale_pct, 1.0);
}

// ─── bus factor ────────────────────────────────────────────────

#[test]
fn bus_factor_counts_unique_authors() {
    let export = make_export(vec![
        make_row("src/a.rs", "src", 50),
        make_row("src/b.rs", "src", 60),
    ]);
    let commits = vec![
        make_commit(DAY, "alice", "c1", &["src/a.rs"]),
        make_commit(2 * DAY, "bob", "c2", &["src/b.rs"]),
        make_commit(3 * DAY, "alice", "c3", &["src/a.rs"]),
    ];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    let src_bus = report
        .bus_factor
        .iter()
        .find(|b| b.module == "src")
        .unwrap();
    assert_eq!(src_bus.authors, 2); // alice and bob
}

// ─── commit intent classification ──────────────────────────────

#[test]
fn intent_counts_feat_and_fix() {
    let export = make_export(vec![make_row("src/lib.rs", "src", 50)]);
    let commits = vec![
        make_commit(DAY, "a", "feat: add feature", &["src/lib.rs"]),
        make_commit(2 * DAY, "a", "fix: resolve bug", &["src/lib.rs"]),
        make_commit(3 * DAY, "a", "fix: another bug", &["src/lib.rs"]),
    ];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    let intent = report.intent.unwrap();
    assert_eq!(intent.overall.feat, 1);
    assert_eq!(intent.overall.fix, 2);
    assert_eq!(intent.overall.total, 3);
}

#[test]
fn intent_corrective_ratio() {
    let export = make_export(vec![make_row("src/lib.rs", "src", 50)]);
    let commits = vec![
        make_commit(DAY, "a", "feat: new", &["src/lib.rs"]),
        make_commit(2 * DAY, "a", "fix: oops", &["src/lib.rs"]),
        make_commit(3 * DAY, "a", "Revert \"feat: new\"", &["src/lib.rs"]),
        make_commit(4 * DAY, "a", "docs: update", &["src/lib.rs"]),
    ];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    let intent = report.intent.unwrap();
    // corrective = (fix + revert) / total = 2 / 4 = 0.5
    assert_eq!(intent.corrective_ratio.unwrap(), 0.5);
}

#[test]
fn intent_unknown_pct_for_unrecognized() {
    let export = make_export(vec![make_row("src/lib.rs", "src", 50)]);
    let commits = vec![
        make_commit(DAY, "a", "WIP something", &["src/lib.rs"]),
        make_commit(2 * DAY, "a", "misc changes", &["src/lib.rs"]),
    ];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    let intent = report.intent.unwrap();
    assert_eq!(intent.unknown_pct, 1.0); // both unrecognized
}

// ─── edge cases ────────────────────────────────────────────────

#[test]
fn no_commits_produces_empty_report() {
    let export = make_export(vec![make_row("src/lib.rs", "src", 50)]);
    let commits: Vec<GitCommit> = vec![];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    assert_eq!(report.commits_scanned, 0);
    assert_eq!(report.files_seen, 0);
    assert!(report.hotspots.is_empty());
    assert!(report.bus_factor.is_empty());
    assert!(report.coupling.is_empty());
}

#[test]
fn single_commit_single_file() {
    let export = make_export(vec![make_row("src/main.rs", "src", 10)]);
    let commits = vec![make_commit(DAY, "alice", "feat: init", &["src/main.rs"])];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    assert_eq!(report.commits_scanned, 1);
    assert_eq!(report.files_seen, 1);
    assert_eq!(report.hotspots[0].score, 10); // 10 * 1
}

#[test]
fn backslash_paths_normalized() {
    let export = make_export(vec![make_row("src/lib.rs", "src", 20)]);
    let commits = vec![make_commit(DAY, "a", "c1", &["src\\lib.rs"])];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    assert_eq!(report.hotspots.len(), 1);
    assert_eq!(report.hotspots[0].path, "src/lib.rs");
}

#[test]
fn dot_slash_prefix_stripped() {
    let export = make_export(vec![make_row("src/lib.rs", "src", 15)]);
    let commits = vec![make_commit(DAY, "a", "c1", &["./src/lib.rs"])];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    assert_eq!(report.hotspots.len(), 1);
}

// ─── code age distribution ─────────────────────────────────────

#[test]
fn age_distribution_buckets_populated() {
    let now = 500 * DAY;
    let export = make_export(vec![
        make_row("src/new.rs", "src", 10),
        make_row("src/old.rs", "src", 10),
    ]);
    let commits = vec![
        make_commit(now, "a", "c1", &["src/new.rs"]), // 0 days old
        make_commit(now - 400 * DAY, "a", "c2", &["src/old.rs"]), // 400 days old
    ];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    let dist = report.age_distribution.unwrap();
    assert_eq!(dist.buckets.len(), 5);
    // Sum of file counts should equal total tracked files
    let total: usize = dist.buckets.iter().map(|b| b.files).sum();
    assert_eq!(total, 2);
}

// ─── churn prediction ──────────────────────────────────────────

#[test]
fn churn_rising_trend_detected() {
    let week = 7 * DAY;
    let export = make_export(vec![make_row("src/lib.rs", "src", 50)]);
    // Increasing number of commits per week → rising slope
    let mut commits = Vec::new();
    for w in 1i64..=10 {
        for c in 0..w {
            commits.push(make_commit(w * week + c * 100, "a", "c", &["src/lib.rs"]));
        }
    }
    let report = build_predictive_churn_report(&export, &commits, Path::new("."));
    let trend = report.per_module.get("src").unwrap();
    assert!(
        trend.slope > 0.0,
        "Expected positive slope for rising churn"
    );
}

#[test]
fn churn_flat_trend_for_constant_activity() {
    let week = 7 * DAY;
    let export = make_export(vec![make_row("src/lib.rs", "src", 50)]);
    // Exactly 1 commit per week → flat
    let commits: Vec<GitCommit> = (1..=5)
        .map(|w| make_commit(w as i64 * week, "a", "c", &["src/lib.rs"]))
        .collect();
    let report = build_predictive_churn_report(&export, &commits, Path::new("."));
    let trend = report.per_module.get("src").unwrap();
    // With constant 1 commit per week, slope should be ~0
    assert!(
        trend.slope.abs() < 0.1,
        "Expected near-zero slope, got {}",
        trend.slope
    );
}

#[test]
fn churn_empty_commits_yields_empty_report() {
    let export = make_export(vec![make_row("src/lib.rs", "src", 50)]);
    let report = build_predictive_churn_report(&export, &[], Path::new("."));
    assert!(report.per_module.is_empty());
}

#[test]
fn churn_child_rows_excluded() {
    let week = 7 * DAY;
    let mut export = make_export(vec![]);
    export.rows.push(FileRow {
        path: "src/lib.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Child,
        code: 50,
        comments: 0,
        blanks: 0,
        lines: 50,
        bytes: 500,
        tokens: 250,
    });
    let commits = vec![make_commit(week, "a", "c", &["src/lib.rs"])];
    let report = build_predictive_churn_report(&export, &commits, Path::new("."));
    assert!(report.per_module.is_empty());
}

// ─── commits_scanned / files_seen counters ─────────────────────

#[test]
fn commits_scanned_counts_all_commits() {
    let export = make_export(vec![make_row("src/lib.rs", "src", 10)]);
    let commits = vec![
        make_commit(DAY, "a", "c1", &["src/lib.rs"]),
        make_commit(2 * DAY, "a", "c2", &["other.rs"]),
        make_commit(3 * DAY, "a", "c3", &["src/lib.rs"]),
    ];
    let report = build_git_report(Path::new("."), &export, &commits).unwrap();
    assert_eq!(report.commits_scanned, 3);
    assert_eq!(report.files_seen, 1); // only src/lib.rs matched
}
