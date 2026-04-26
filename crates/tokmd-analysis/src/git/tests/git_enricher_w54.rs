//! W54: Comprehensive enricher coverage for ``tokmd-analysis` Git module`.
//!
//! Targets git report construction, hotspot detection, coupling analysis,
//! freshness, bus factor, churn prediction, intent classification,
//! edge cases, and deterministic ordering.

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
// 1. Empty commits list → zero counts
// ===========================================================================
#[test]
fn empty_commits_zero_counts() {
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let report = build_git_report(Path::new("."), &exp, &[]).unwrap();
    assert_eq!(report.commits_scanned, 0);
    assert_eq!(report.files_seen, 0);
    assert!(report.hotspots.is_empty());
    assert!(report.bus_factor.is_empty());
}

// ===========================================================================
// 2. Single commit, single file → hotspot with score = lines * 1
// ===========================================================================
#[test]
fn single_commit_single_file_hotspot() {
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits = vec![commit(1000, "alice", "feat: init", &["src/lib.rs"])];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.hotspots.len(), 1);
    assert_eq!(report.hotspots[0].score, 100); // 100 lines * 1 commit
    assert_eq!(report.hotspots[0].commits, 1);
}

// ===========================================================================
// 3. Hotspot score scales with commit count
// ===========================================================================
#[test]
fn hotspot_score_scales_with_commits() {
    let exp = export(vec![file_row("src/lib.rs", "src", 50)]);
    let commits = vec![
        commit(1000, "alice", "feat: a", &["src/lib.rs"]),
        commit(2000, "bob", "fix: b", &["src/lib.rs"]),
        commit(3000, "alice", "refactor: c", &["src/lib.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.hotspots[0].score, 150); // 50 * 3
}

// ===========================================================================
// 4. Hotspots sorted by score descending
// ===========================================================================
#[test]
fn hotspots_sorted_desc() {
    let exp = export(vec![
        file_row("hot.rs", "src", 200),
        file_row("cold.rs", "src", 10),
    ]);
    let commits = vec![
        commit(1000, "a", "feat", &["hot.rs", "cold.rs"]),
        commit(2000, "a", "fix", &["hot.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.hotspots[0].path, "hot.rs");
    assert!(report.hotspots[0].score > report.hotspots[1].score);
}

// ===========================================================================
// 5. Bus factor: single author → 1
// ===========================================================================
#[test]
fn bus_factor_single_author() {
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits = vec![
        commit(1000, "alice", "feat: a", &["src/lib.rs"]),
        commit(2000, "alice", "fix: b", &["src/lib.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let bf = report
        .bus_factor
        .iter()
        .find(|b| b.module == "src")
        .unwrap();
    assert_eq!(bf.authors, 1);
}

// ===========================================================================
// 6. Bus factor: multiple authors counted per module
// ===========================================================================
#[test]
fn bus_factor_multiple_authors() {
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits = vec![
        commit(1000, "alice", "feat: a", &["src/lib.rs"]),
        commit(2000, "bob", "fix: b", &["src/lib.rs"]),
        commit(3000, "charlie", "chore: c", &["src/lib.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let bf = report
        .bus_factor
        .iter()
        .find(|b| b.module == "src")
        .unwrap();
    assert_eq!(bf.authors, 3);
}

// ===========================================================================
// 7. Bus factor sorted by author count ascending
// ===========================================================================
#[test]
fn bus_factor_sorted_ascending() {
    let exp = export(vec![
        file_row("a/x.rs", "a", 10),
        file_row("b/y.rs", "b", 10),
    ]);
    let commits = vec![
        commit(1000, "alice", "feat", &["a/x.rs", "b/y.rs"]),
        commit(2000, "bob", "fix", &["b/y.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert!(report.bus_factor[0].authors <= report.bus_factor[1].authors);
}

// ===========================================================================
// 8. Coupling: two modules changed together
// ===========================================================================
#[test]
fn coupling_two_modules() {
    let exp = export(vec![
        file_row("a/x.rs", "a", 10),
        file_row("b/y.rs", "b", 20),
    ]);
    let commits = vec![
        commit(1000, "alice", "feat: both", &["a/x.rs", "b/y.rs"]),
        commit(2000, "bob", "fix: both", &["a/x.rs", "b/y.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.coupling.len(), 1);
    assert_eq!(report.coupling[0].count, 2);
    assert!(report.coupling[0].jaccard.is_some());
}

// ===========================================================================
// 9. Coupling: single module changes → no coupling rows
// ===========================================================================
#[test]
fn coupling_single_module_no_rows() {
    let exp = export(vec![
        file_row("a/x.rs", "a", 10),
        file_row("a/y.rs", "a", 20),
    ]);
    let commits = vec![commit(1000, "alice", "feat", &["a/x.rs", "a/y.rs"])];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert!(report.coupling.is_empty());
}

// ===========================================================================
// 10. Freshness: all files within threshold → 0 stale
// ===========================================================================
#[test]
fn freshness_all_recent() {
    let now = 400 * DAY;
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits = vec![commit(now - 10 * DAY, "alice", "feat", &["src/lib.rs"])];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.freshness.stale_files, 0);
    assert_eq!(report.freshness.stale_pct, 0.0);
}

// ===========================================================================
// 11. Freshness: file older than 365 days → stale
// ===========================================================================
#[test]
fn freshness_stale_file() {
    let now = 800 * DAY;
    let exp = export(vec![
        file_row("old.rs", "src", 50),
        file_row("new.rs", "src", 50),
    ]);
    // Need a recent commit so max_ts is "now", making old.rs stale relative to it
    let commits = vec![
        commit(now, "bob", "feat: new", &["new.rs"]),
        commit(now - 400 * DAY, "alice", "feat: old", &["old.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.freshness.stale_files, 1);
    assert_eq!(report.freshness.total_files, 2);
}

// ===========================================================================
// 12. Freshness module rows sorted alphabetically
// ===========================================================================
#[test]
fn freshness_modules_sorted() {
    let now = 500 * DAY;
    let exp = export(vec![
        file_row("z/a.rs", "z", 10),
        file_row("a/b.rs", "a", 10),
    ]);
    let commits = vec![commit(
        now - 10 * DAY,
        "alice",
        "feat",
        &["z/a.rs", "a/b.rs"],
    )];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    if report.freshness.by_module.len() >= 2 {
        assert!(report.freshness.by_module[0].module < report.freshness.by_module[1].module);
    }
}

// ===========================================================================
// 13. Commit intent classification
// ===========================================================================
#[test]
fn intent_report_present() {
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits = vec![
        commit(1000, "a", "feat: add x", &["src/lib.rs"]),
        commit(2000, "b", "fix: bug y", &["src/lib.rs"]),
        commit(3000, "c", "chore: cleanup", &["src/lib.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let intent = report.intent.as_ref().unwrap();
    assert_eq!(intent.overall.total, 3);
}

// ===========================================================================
// 14. Corrective ratio computed
// ===========================================================================
#[test]
fn intent_corrective_ratio() {
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits = vec![
        commit(1000, "a", "feat: add", &["src/lib.rs"]),
        commit(2000, "b", "fix: bug", &["src/lib.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let intent = report.intent.as_ref().unwrap();
    assert!(intent.corrective_ratio.is_some());
    let ratio = intent.corrective_ratio.unwrap();
    assert!((0.0..=1.0).contains(&ratio));
}

// ===========================================================================
// 15. Code age distribution buckets
// ===========================================================================
#[test]
fn age_distribution_has_buckets() {
    let now = 500 * DAY;
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits = vec![commit(now - 15 * DAY, "a", "feat", &["src/lib.rs"])];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let age = report.age_distribution.as_ref().unwrap();
    assert_eq!(age.buckets.len(), 5);
    // File is 15 days old → in "0-30d" bucket
    assert!(age.buckets[0].files > 0);
}

// ===========================================================================
// 16. Predictive churn: empty commits → empty report
// ===========================================================================
#[test]
fn churn_empty_commits() {
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let report = build_predictive_churn_report(&exp, &[], Path::new("."));
    assert!(report.per_module.is_empty());
}

// ===========================================================================
// 17. Predictive churn: single commit → flat trend
// ===========================================================================
#[test]
fn churn_single_commit_flat() {
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits = vec![commit(WEEK, "alice", "feat", &["src/lib.rs"])];
    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));
    let trend = report.per_module.get("src").unwrap();
    assert_eq!(trend.classification, TrendClass::Flat);
}

// ===========================================================================
// 18. Predictive churn: increasing commits → rising
// ===========================================================================
#[test]
fn churn_rising_trend() {
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let mut commits = Vec::new();
    // Ramp up: more commits per week
    for w in 1..=10 {
        for _ in 0..w {
            commits.push(commit(w * WEEK, "a", "feat", &["src/lib.rs"]));
        }
    }
    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));
    let trend = report.per_module.get("src").unwrap();
    assert!(trend.slope > 0.0);
    assert_eq!(trend.classification, TrendClass::Rising);
}

// ===========================================================================
// 19. Unmatched files in commits are ignored
// ===========================================================================
#[test]
fn unmatched_files_ignored() {
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits = vec![commit(1000, "a", "feat", &["nonexistent.rs"])];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.files_seen, 0);
    assert!(report.hotspots.is_empty());
}

// ===========================================================================
// 20. Child rows filtered from git analysis
// ===========================================================================
#[test]
fn child_rows_filtered() {
    let exp = ExportData {
        rows: vec![FileRow {
            path: "embedded.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Child,
            code: 50,
            comments: 0,
            blanks: 0,
            lines: 50,
            bytes: 2000,
            tokens: 150,
        }],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let commits = vec![commit(1000, "a", "feat", &["embedded.rs"])];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.files_seen, 0);
}

// ===========================================================================
// 21. Git report deterministic
// ===========================================================================
#[test]
fn git_report_deterministic() {
    let exp = export(vec![
        file_row("a.rs", "src", 100),
        file_row("b.rs", "src", 200),
    ]);
    let commits = vec![
        commit(1000, "alice", "feat: a", &["a.rs", "b.rs"]),
        commit(2000, "bob", "fix: b", &["b.rs"]),
    ];
    let r1 = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let r2 = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(r1.hotspots.len(), r2.hotspots.len());
    for (a, b) in r1.hotspots.iter().zip(r2.hotspots.iter()) {
        assert_eq!(a.path, b.path);
        assert_eq!(a.score, b.score);
    }
}

// ===========================================================================
// 22. Coupling with lift metric
// ===========================================================================
#[test]
fn coupling_has_lift() {
    let exp = export(vec![
        file_row("a/x.rs", "a", 10),
        file_row("b/y.rs", "b", 20),
    ]);
    let commits = vec![
        commit(1000, "alice", "feat", &["a/x.rs", "b/y.rs"]),
        commit(2000, "bob", "fix", &["a/x.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.coupling.len(), 1);
    assert!(report.coupling[0].lift.is_some());
}

// ===========================================================================
// 23. Churn multi-module tracks separately
// ===========================================================================
#[test]
fn churn_multi_module_separate() {
    let exp = export(vec![
        file_row("a/x.rs", "a", 10),
        file_row("b/y.rs", "b", 20),
    ]);
    let commits = vec![
        commit(WEEK, "alice", "feat", &["a/x.rs"]),
        commit(2 * WEEK, "bob", "fix", &["b/y.rs"]),
    ];
    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));
    assert!(report.per_module.contains_key("a"));
    assert!(report.per_module.contains_key("b"));
}
