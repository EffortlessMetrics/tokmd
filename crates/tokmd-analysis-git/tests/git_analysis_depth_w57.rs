//! Depth tests for `tokmd-analysis-git` — w57
//!
//! Covers hotspot detection, freshness scoring, coupling detection,
//! churn calculation, empty/single/many commit scenarios, deterministic
//! ordering, and serde roundtrips.

use std::path::Path;

use tokmd_analysis_git::{build_git_report, build_predictive_churn_report};
use tokmd_analysis_types::TrendClass;
use tokmd_git::GitCommit;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

const DAY: i64 = 86_400;
const WEEK: i64 = 7 * DAY;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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
// Hotspot detection
// ===========================================================================

// 1. Single file with many commits is a hotspot
#[test]
fn hotspot_single_file_many_commits() {
    let exp = export(vec![row("src/hot.rs", "src", 500)]);
    let commits: Vec<GitCommit> = (0..10)
        .map(|i| commit(1000 + i * DAY, "alice", "feat: change", &["src/hot.rs"]))
        .collect();
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.hotspots.len(), 1);
    assert_eq!(report.hotspots[0].commits, 10);
    assert_eq!(report.hotspots[0].lines, 500);
    assert_eq!(report.hotspots[0].score, 500 * 10);
}

// 2. Hotspot ordering: higher score first
#[test]
fn hotspot_ordering_by_score_desc() {
    let exp = export(vec![
        row("src/big.rs", "src", 1000),
        row("src/small.rs", "src", 10),
    ]);
    let commits = vec![
        commit(1000, "a", "feat: x", &["src/big.rs"]),
        commit(2000, "a", "feat: y", &["src/small.rs"]),
        commit(3000, "a", "feat: z", &["src/small.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.hotspots.len(), 2);
    // big.rs: 1000*1=1000, small.rs: 10*2=20
    assert_eq!(report.hotspots[0].path, "src/big.rs");
    assert!(report.hotspots[0].score > report.hotspots[1].score);
}

// 3. Hotspot score = lines * commits
#[test]
fn hotspot_score_formula() {
    let exp = export(vec![row("lib.rs", "root", 42)]);
    let commits = vec![
        commit(1000, "a", "feat: a", &["lib.rs"]),
        commit(2000, "b", "fix: b", &["lib.rs"]),
        commit(3000, "c", "chore: c", &["lib.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.hotspots[0].score, 42 * 3);
}

// 4. Files not in export are excluded from hotspots
#[test]
fn hotspot_excludes_unmapped_files() {
    let exp = export(vec![row("src/a.rs", "src", 100)]);
    let commits = vec![commit(1000, "a", "feat: x", &["src/a.rs", "unknown.rs"])];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.hotspots.len(), 1);
    assert_eq!(report.hotspots[0].path, "src/a.rs");
}

// 5. Deterministic hotspot tie-breaking by path
#[test]
fn hotspot_tiebreak_by_path() {
    let exp = export(vec![row("b.rs", "root", 100), row("a.rs", "root", 100)]);
    let commits = vec![commit(1000, "x", "feat: x", &["a.rs", "b.rs"])];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    // Same score (100*1=100), so sorted by path ascending
    assert_eq!(report.hotspots[0].path, "a.rs");
    assert_eq!(report.hotspots[1].path, "b.rs");
}

// ===========================================================================
// Freshness scoring
// ===========================================================================

// 6. All files touched recently → zero stale
#[test]
fn freshness_all_recent() {
    let now = 400 * DAY;
    let exp = export(vec![row("a.rs", "src", 10), row("b.rs", "src", 20)]);
    let commits = vec![
        commit(now - 10 * DAY, "a", "feat: a", &["a.rs"]),
        commit(now, "b", "feat: b", &["b.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.freshness.stale_files, 0);
    assert_eq!(report.freshness.stale_pct, 0.0);
}

// 7. All files older than 365 days → all stale
#[test]
fn freshness_all_stale() {
    let now = 1000 * DAY;
    let exp = export(vec![row("a.rs", "src", 10), row("b.rs", "src", 20)]);
    // Push max_ts to `now` via an unrelated commit, so tracked files are stale
    let commits = vec![
        commit(now, "x", "chore: bump", &["unrelated.rs"]),
        commit(100 * DAY, "a", "feat: a", &["a.rs"]),
        commit(50 * DAY, "b", "feat: b", &["b.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.freshness.stale_files, 2);
    assert_eq!(report.freshness.total_files, 2);
}

// 8. Mixed freshness: one stale, one fresh
#[test]
fn freshness_mixed() {
    let now = 800 * DAY;
    let exp = export(vec![row("old.rs", "src", 10), row("new.rs", "src", 20)]);
    let commits = vec![
        commit(now - 400 * DAY, "a", "feat: old", &["old.rs"]),
        commit(now, "b", "feat: new", &["new.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.freshness.stale_files, 1);
    assert_eq!(report.freshness.total_files, 2);
}

// 9. Module freshness rows are sorted by module name
#[test]
fn freshness_modules_sorted() {
    let now = 400 * DAY;
    let exp = export(vec![
        row("z/a.rs", "z", 10),
        row("a/b.rs", "a", 20),
        row("m/c.rs", "m", 30),
    ]);
    let commits = vec![commit(now, "x", "feat: x", &["z/a.rs", "a/b.rs", "m/c.rs"])];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let modules: Vec<&str> = report
        .freshness
        .by_module
        .iter()
        .map(|m| m.module.as_str())
        .collect();
    assert_eq!(modules, vec!["a", "m", "z"]);
}

// ===========================================================================
// Coupling detection
// ===========================================================================

// 10. Two modules always changed together → coupling detected
#[test]
fn coupling_always_together() {
    let exp = export(vec![row("a/x.rs", "a", 10), row("b/y.rs", "b", 20)]);
    let commits = vec![
        commit(1000, "x", "feat: x", &["a/x.rs", "b/y.rs"]),
        commit(2000, "x", "feat: y", &["a/x.rs", "b/y.rs"]),
        commit(3000, "x", "feat: z", &["a/x.rs", "b/y.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.coupling.len(), 1);
    assert_eq!(report.coupling[0].count, 3);
    assert!(report.coupling[0].jaccard.unwrap() > 0.0);
}

// 11. Single-module commits → no coupling
#[test]
fn coupling_single_module_no_pairs() {
    let exp = export(vec![row("a/x.rs", "a", 10), row("b/y.rs", "b", 20)]);
    let commits = vec![
        commit(1000, "x", "feat: a only", &["a/x.rs"]),
        commit(2000, "x", "feat: b only", &["b/y.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert!(report.coupling.is_empty());
}

// 12. Coupling rows sorted by count descending
#[test]
fn coupling_sorted_by_count_desc() {
    let exp = export(vec![
        row("a/x.rs", "a", 10),
        row("b/y.rs", "b", 20),
        row("c/z.rs", "c", 30),
    ]);
    let commits = vec![
        commit(1000, "x", "feat: ab", &["a/x.rs", "b/y.rs"]),
        commit(2000, "x", "feat: bc", &["b/y.rs", "c/z.rs"]),
        commit(3000, "x", "feat: bc2", &["b/y.rs", "c/z.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert!(report.coupling.len() >= 2);
    assert!(report.coupling[0].count >= report.coupling[1].count);
}

// 13. Coupling pair ordering is canonical (left <= right)
#[test]
fn coupling_canonical_pair_order() {
    let exp = export(vec![row("z/a.rs", "z", 10), row("a/b.rs", "a", 20)]);
    let commits = vec![commit(1000, "x", "feat: x", &["z/a.rs", "a/b.rs"])];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.coupling.len(), 1);
    assert!(report.coupling[0].left <= report.coupling[0].right);
}

// ===========================================================================
// Churn calculation
// ===========================================================================

// 14. No commits → empty churn report
#[test]
fn churn_no_commits() {
    let exp = export(vec![row("a.rs", "src", 100)]);
    let report = build_predictive_churn_report(&exp, &[], Path::new("."));
    assert!(report.per_module.is_empty());
}

// 15. Single commit → flat trend (single point regression)
#[test]
fn churn_single_commit_flat() {
    let exp = export(vec![row("a.rs", "src", 100)]);
    let commits = vec![commit(WEEK, "a", "feat: x", &["a.rs"])];
    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));
    let trend = report.per_module.get("src").unwrap();
    assert!(matches!(trend.classification, TrendClass::Flat));
}

// 16. Increasing activity → rising trend
#[test]
fn churn_rising_activity() {
    let exp = export(vec![row("a.rs", "src", 100)]);
    let mut commits = Vec::new();
    // Week 1: 1 commit, Week 2: 2 commits, ..., Week 10: 10 commits
    for week in 1..=10i64 {
        for _ in 0..week {
            commits.push(commit(week * WEEK + DAY, "a", "feat: x", &["a.rs"]));
        }
    }
    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));
    let trend = report.per_module.get("src").unwrap();
    assert!(trend.slope > 0.0);
    assert!(matches!(trend.classification, TrendClass::Rising));
}

// 17. Churn modules are BTreeMap-ordered (deterministic)
#[test]
fn churn_module_ordering_deterministic() {
    let exp = export(vec![
        row("z/a.rs", "z", 10),
        row("a/b.rs", "a", 20),
        row("m/c.rs", "m", 30),
    ]);
    let commits = vec![
        commit(WEEK, "x", "feat: x", &["z/a.rs", "a/b.rs", "m/c.rs"]),
        commit(2 * WEEK, "x", "feat: y", &["z/a.rs", "a/b.rs", "m/c.rs"]),
    ];
    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));
    let keys: Vec<&String> = report.per_module.keys().collect();
    assert_eq!(keys, vec!["a", "m", "z"]);
}

// ===========================================================================
// Empty / single / many commits
// ===========================================================================

// 18. Empty git history → empty report
#[test]
fn empty_history_empty_report() {
    let exp = export(vec![row("a.rs", "src", 50)]);
    let report = build_git_report(Path::new("."), &exp, &[]).unwrap();
    assert_eq!(report.commits_scanned, 0);
    assert_eq!(report.files_seen, 0);
    assert!(report.hotspots.is_empty());
    assert!(report.coupling.is_empty());
    assert_eq!(report.freshness.total_files, 0);
}

// 19. Single commit produces valid report
#[test]
fn single_commit_valid_report() {
    let exp = export(vec![row("main.rs", "root", 200)]);
    let commits = vec![commit(5000, "alice", "feat: init", &["main.rs"])];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.commits_scanned, 1);
    assert_eq!(report.files_seen, 1);
    assert_eq!(report.hotspots.len(), 1);
    assert_eq!(report.hotspots[0].commits, 1);
}

// 20. Many commits (100+) do not panic
#[test]
fn many_commits_no_panic() {
    let exp = export(vec![row("a.rs", "src", 100), row("b.rs", "src", 200)]);
    let commits: Vec<GitCommit> = (0..200)
        .map(|i| {
            let files = if i % 2 == 0 {
                vec!["a.rs"]
            } else {
                vec!["b.rs"]
            };
            commit(1000 + i * DAY, "dev", "chore: update", &files)
        })
        .collect();
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.commits_scanned, 200);
    assert_eq!(report.files_seen, 2);
}

// ===========================================================================
// Deterministic ordering
// ===========================================================================

// 21. Bus factor sorted by authors ascending, then module name
#[test]
fn bus_factor_deterministic_sort() {
    let exp = export(vec![row("z/a.rs", "z", 10), row("a/b.rs", "a", 20)]);
    let commits = vec![
        commit(1000, "alice", "feat: x", &["z/a.rs"]),
        commit(2000, "bob", "feat: y", &["a/b.rs"]),
        commit(3000, "alice", "feat: z", &["a/b.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    // z has 1 author, a has 2 authors
    assert_eq!(report.bus_factor[0].module, "z");
    assert_eq!(report.bus_factor[0].authors, 1);
    assert_eq!(report.bus_factor[1].module, "a");
    assert_eq!(report.bus_factor[1].authors, 2);
}

// 22. Running the same inputs twice produces identical results
#[test]
fn deterministic_full_report() {
    let exp = export(vec![row("a.rs", "src", 50), row("b.rs", "lib", 80)]);
    let commits = vec![
        commit(1000, "alice", "feat: a", &["a.rs"]),
        commit(2000, "bob", "fix: b", &["b.rs"]),
        commit(3000, "alice", "chore: both", &["a.rs", "b.rs"]),
    ];
    let r1 = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let r2 = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let j1 = serde_json::to_string(&r1).unwrap();
    let j2 = serde_json::to_string(&r2).unwrap();
    assert_eq!(j1, j2);
}

// ===========================================================================
// Serde roundtrips
// ===========================================================================

// 23. GitReport roundtrip
#[test]
fn serde_git_report_roundtrip() {
    let exp = export(vec![row("a.rs", "src", 100), row("b.rs", "lib", 200)]);
    let commits = vec![
        commit(1000, "alice", "feat: a", &["a.rs"]),
        commit(2000, "bob", "fix: b", &["b.rs", "a.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let json = serde_json::to_string_pretty(&report).unwrap();
    let deser: tokmd_analysis_types::GitReport = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.commits_scanned, report.commits_scanned);
    assert_eq!(deser.files_seen, report.files_seen);
    assert_eq!(deser.hotspots.len(), report.hotspots.len());
    assert_eq!(deser.bus_factor.len(), report.bus_factor.len());
    assert_eq!(deser.coupling.len(), report.coupling.len());
}

// 24. PredictiveChurnReport roundtrip
#[test]
fn serde_churn_report_roundtrip() {
    let exp = export(vec![row("a.rs", "src", 100)]);
    let commits = vec![
        commit(WEEK, "a", "feat: a", &["a.rs"]),
        commit(2 * WEEK, "a", "feat: b", &["a.rs"]),
        commit(3 * WEEK, "a", "feat: c", &["a.rs"]),
    ];
    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));
    let json = serde_json::to_string(&report).unwrap();
    let deser: tokmd_analysis_types::PredictiveChurnReport = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.per_module.len(), report.per_module.len());
    let trend = deser.per_module.get("src").unwrap();
    assert_eq!(trend.slope, report.per_module["src"].slope);
}

// 25. FreshnessReport roundtrip via GitReport
#[test]
fn serde_freshness_roundtrip() {
    let now = 400 * DAY;
    let exp = export(vec![row("a.rs", "src", 50)]);
    let commits = vec![commit(now - 10 * DAY, "a", "feat: a", &["a.rs"])];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let json = serde_json::to_string(&report.freshness).unwrap();
    let deser: tokmd_analysis_types::FreshnessReport = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.total_files, report.freshness.total_files);
    assert_eq!(deser.stale_files, report.freshness.stale_files);
    assert_eq!(deser.by_module.len(), report.freshness.by_module.len());
}

// ===========================================================================
// Code age distribution
// ===========================================================================

// 26. Age buckets sum to total files
#[test]
fn age_buckets_sum_to_total() {
    let now = 800 * DAY;
    let exp = export(vec![
        row("a.rs", "src", 10),
        row("b.rs", "src", 20),
        row("c.rs", "src", 30),
    ]);
    let commits = vec![
        commit(now, "a", "feat: a", &["a.rs"]),
        commit(now - 100 * DAY, "b", "feat: b", &["b.rs"]),
        commit(now - 400 * DAY, "c", "feat: c", &["c.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let dist = report.age_distribution.as_ref().unwrap();
    let sum: usize = dist.buckets.iter().map(|b| b.files).sum();
    assert_eq!(sum, 3);
}

// 27. Age percentages sum to ~1.0
#[test]
fn age_pct_sum_approximately_one() {
    let now = 800 * DAY;
    let exp = export(vec![row("a.rs", "src", 10), row("b.rs", "lib", 20)]);
    let commits = vec![
        commit(now, "a", "feat: a", &["a.rs"]),
        commit(now - 200 * DAY, "b", "feat: b", &["b.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let dist = report.age_distribution.as_ref().unwrap();
    let pct_sum: f64 = dist.buckets.iter().map(|b| b.pct).sum();
    assert!((pct_sum - 1.0).abs() < 0.01);
}

// ===========================================================================
// Intent classification
// ===========================================================================

// 28. Intent report classifies conventional commits
#[test]
fn intent_conventional_commits() {
    let exp = export(vec![row("a.rs", "src", 100)]);
    let commits = vec![
        commit(1000, "a", "feat: add feature", &["a.rs"]),
        commit(2000, "b", "fix: bug fix", &["a.rs"]),
        commit(3000, "c", "docs: update readme", &["a.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let intent = report.intent.as_ref().unwrap();
    assert_eq!(intent.overall.feat, 1);
    assert_eq!(intent.overall.fix, 1);
    assert_eq!(intent.overall.docs, 1);
    assert_eq!(intent.overall.total, 3);
}

// 29. Corrective ratio = (fix + revert) / total
#[test]
fn intent_corrective_ratio() {
    let exp = export(vec![row("a.rs", "src", 100)]);
    let commits = vec![
        commit(1000, "a", "feat: feature", &["a.rs"]),
        commit(2000, "b", "fix: bug", &["a.rs"]),
        commit(3000, "c", "fix: another bug", &["a.rs"]),
        commit(4000, "d", "Revert \"feat: feature\"", &["a.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let intent = report.intent.as_ref().unwrap();
    // corrective = (2 fix + 1 revert) / 4 total = 0.75
    let ratio = intent.corrective_ratio.unwrap();
    assert!((ratio - 0.75).abs() < 0.001);
}

// 30. Intent by_module is sorted alphabetically
#[test]
fn intent_by_module_sorted() {
    let exp = export(vec![row("z/a.rs", "z", 10), row("a/b.rs", "a", 20)]);
    let commits = vec![
        commit(1000, "x", "feat: x", &["z/a.rs"]),
        commit(2000, "x", "fix: y", &["a/b.rs"]),
    ];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let intent = report.intent.as_ref().unwrap();
    let modules: Vec<&str> = intent.by_module.iter().map(|m| m.module.as_str()).collect();
    assert_eq!(modules, vec!["a", "z"]);
}

// ===========================================================================
// Edge cases
// ===========================================================================

// 31. Backslash paths in commits get normalized
#[test]
fn backslash_paths_normalized() {
    let exp = export(vec![row("src/lib.rs", "src", 100)]);
    let commits = vec![commit(1000, "a", "feat: x", &["src\\lib.rs"])];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.hotspots.len(), 1);
    assert_eq!(report.hotspots[0].path, "src/lib.rs");
}

// 32. Dot-prefixed paths get stripped
#[test]
fn dot_prefix_stripped() {
    let exp = export(vec![row("lib.rs", "root", 50)]);
    let commits = vec![commit(1000, "a", "feat: x", &["./lib.rs"])];
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    assert_eq!(report.hotspots.len(), 1);
}

// 33. Churn with unmapped files → only tracked modules appear
#[test]
fn churn_unmapped_files_ignored() {
    let exp = export(vec![row("a.rs", "src", 100)]);
    let commits = vec![
        commit(WEEK, "a", "feat: x", &["a.rs", "unknown.rs"]),
        commit(2 * WEEK, "a", "feat: y", &["unknown_only.rs"]),
    ];
    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));
    assert_eq!(report.per_module.len(), 1);
    assert!(report.per_module.contains_key("src"));
}
