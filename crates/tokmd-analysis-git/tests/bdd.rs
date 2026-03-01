//! BDD-style scenario tests for `tokmd-analysis-git`.
//!
//! Each test follows Given / When / Then structure exercising the public API:
//! - `build_git_report`
//! - `build_predictive_churn_report`

use std::path::Path;

use tokmd_analysis_git::{build_git_report, build_predictive_churn_report};
use tokmd_analysis_types::TrendClass;
use tokmd_git::GitCommit;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

const DAY: i64 = 86_400;

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
// Scenario: Empty commits produce empty git report
// ===========================================================================
#[test]
fn scenario_empty_commits_produce_empty_report() {
    // Given: an export with one file, but no commits
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits: Vec<GitCommit> = vec![];

    // When: we build the git report
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    // Then: all sections are empty
    assert_eq!(report.commits_scanned, 0);
    assert_eq!(report.files_seen, 0);
    assert!(report.hotspots.is_empty());
    assert!(report.bus_factor.is_empty());
    assert_eq!(report.freshness.total_files, 0);
    assert!(report.coupling.is_empty());
}

// ===========================================================================
// Scenario: Single file hotspot score equals lines * commits
// ===========================================================================
#[test]
fn scenario_hotspot_score_equals_lines_times_commits() {
    // Given: one file with 200 lines and 3 commits touching it
    let exp = export(vec![file_row("src/lib.rs", "src", 200)]);
    let commits = vec![
        commit(1000, "alice", "feat: init", &["src/lib.rs"]),
        commit(2000, "bob", "fix: bug", &["src/lib.rs"]),
        commit(3000, "alice", "refactor: cleanup", &["src/lib.rs"]),
    ];

    // When: we build the git report
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    // Then: hotspot score = 200 * 3 = 600
    assert_eq!(report.hotspots.len(), 1);
    assert_eq!(report.hotspots[0].commits, 3);
    assert_eq!(report.hotspots[0].lines, 200);
    assert_eq!(report.hotspots[0].score, 600);
}

// ===========================================================================
// Scenario: Hotspots sorted by score descending
// ===========================================================================
#[test]
fn scenario_hotspots_sorted_by_score_desc() {
    // Given: two files, one touched more often
    let exp = export(vec![
        file_row("src/a.rs", "src", 100),
        file_row("src/b.rs", "src", 50),
    ]);
    let commits = vec![
        commit(1000, "alice", "feat: a", &["src/a.rs"]),
        commit(2000, "bob", "feat: b", &["src/b.rs", "src/a.rs"]),
        commit(3000, "alice", "fix: b", &["src/b.rs"]),
        commit(4000, "alice", "fix: b2", &["src/b.rs"]),
    ];

    // When
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    // Then: a.rs score = 100*2=200, b.rs score = 50*3=150 → a.rs first
    assert_eq!(report.hotspots[0].path, "src/a.rs");
    assert_eq!(report.hotspots[0].score, 200);
    assert_eq!(report.hotspots[1].path, "src/b.rs");
    assert_eq!(report.hotspots[1].score, 150);
}

// ===========================================================================
// Scenario: Bus factor counts unique authors per module
// ===========================================================================
#[test]
fn scenario_bus_factor_unique_authors() {
    // Given: two modules, one with 1 author, one with 3
    let exp = export(vec![
        file_row("src/a.rs", "src", 100),
        file_row("lib/b.rs", "lib", 50),
    ]);
    let commits = vec![
        commit(1000, "alice", "feat: a", &["src/a.rs"]),
        commit(2000, "bob", "feat: b", &["lib/b.rs"]),
        commit(3000, "charlie", "fix: b", &["lib/b.rs"]),
        commit(4000, "alice", "fix: b2", &["lib/b.rs"]),
    ];

    // When
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    // Then: bus_factor sorted by author count ascending
    assert_eq!(report.bus_factor.len(), 2);
    // src has 1 author (alice), lib has 3 (alice, bob, charlie)
    assert_eq!(report.bus_factor[0].module, "src");
    assert_eq!(report.bus_factor[0].authors, 1);
    assert_eq!(report.bus_factor[1].module, "lib");
    assert_eq!(report.bus_factor[1].authors, 3);
}

// ===========================================================================
// Scenario: Freshness detects stale files (>365 days old)
// ===========================================================================
#[test]
fn scenario_freshness_stale_files() {
    // Given: one file changed 400 days ago, one changed recently
    let exp = export(vec![
        file_row("src/old.rs", "src", 50),
        file_row("src/new.rs", "src", 50),
    ]);
    let reference_ts = 500 * DAY;
    let commits = vec![
        commit(100 * DAY, "alice", "feat: old", &["src/old.rs"]),
        commit(reference_ts, "bob", "feat: new", &["src/new.rs"]),
    ];

    // When
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    // Then: 1 stale file (old.rs is 400 days old), 1 fresh
    assert_eq!(report.freshness.total_files, 2);
    assert_eq!(report.freshness.stale_files, 1);
    assert_eq!(report.freshness.threshold_days, 365);
    assert!(report.freshness.stale_pct > 0.0);
    assert!(report.freshness.stale_pct < 1.0);
}

// ===========================================================================
// Scenario: Freshness with all fresh files shows zero stale
// ===========================================================================
#[test]
fn scenario_freshness_all_fresh() {
    // Given: all files changed within the last 30 days
    let now = 1000 * DAY;
    let exp = export(vec![
        file_row("src/a.rs", "src", 50),
        file_row("src/b.rs", "src", 50),
    ]);
    let commits = vec![
        commit(now - 5 * DAY, "alice", "feat: a", &["src/a.rs"]),
        commit(now, "bob", "feat: b", &["src/b.rs"]),
    ];

    // When
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    // Then: no stale files
    assert_eq!(report.freshness.stale_files, 0);
    assert_eq!(report.freshness.stale_pct, 0.0);
}

// ===========================================================================
// Scenario: Module freshness rows present per module
// ===========================================================================
#[test]
fn scenario_freshness_by_module() {
    // Given: two modules with different freshness
    let now = 500 * DAY;
    let exp = export(vec![
        file_row("api/handler.rs", "api", 100),
        file_row("db/query.rs", "db", 80),
    ]);
    let commits = vec![
        commit(now, "alice", "feat: api", &["api/handler.rs"]),
        commit(100 * DAY, "bob", "feat: db", &["db/query.rs"]),
    ];

    // When
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    // Then: two module freshness rows, sorted by module name
    assert_eq!(report.freshness.by_module.len(), 2);
    assert_eq!(report.freshness.by_module[0].module, "api");
    assert_eq!(report.freshness.by_module[1].module, "db");
    // db should have higher avg_days than api
    assert!(report.freshness.by_module[1].avg_days > report.freshness.by_module[0].avg_days);
}

// ===========================================================================
// Scenario: Coupling detects modules changed together
// ===========================================================================
#[test]
fn scenario_coupling_modules_changed_together() {
    // Given: commits that always touch both api and db modules
    let exp = export(vec![
        file_row("api/handler.rs", "api", 100),
        file_row("db/query.rs", "db", 80),
    ]);
    let commits = vec![
        commit(
            1000,
            "alice",
            "feat: both",
            &["api/handler.rs", "db/query.rs"],
        ),
        commit(2000, "bob", "fix: both", &["api/handler.rs", "db/query.rs"]),
        commit(
            3000,
            "alice",
            "refactor: both",
            &["api/handler.rs", "db/query.rs"],
        ),
    ];

    // When
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    // Then: one coupling row linking api <-> db
    assert_eq!(report.coupling.len(), 1);
    assert_eq!(report.coupling[0].count, 3);
    // Jaccard: 3 / (3+3-3) = 1.0 (perfect coupling)
    assert_eq!(report.coupling[0].jaccard, Some(1.0));
}

// ===========================================================================
// Scenario: No coupling when modules never share commits
// ===========================================================================
#[test]
fn scenario_no_coupling_independent_modules() {
    // Given: commits that each touch only one module
    let exp = export(vec![
        file_row("api/handler.rs", "api", 100),
        file_row("db/query.rs", "db", 80),
    ]);
    let commits = vec![
        commit(1000, "alice", "feat: api", &["api/handler.rs"]),
        commit(2000, "bob", "feat: db", &["db/query.rs"]),
    ];

    // When
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    // Then: no coupling rows
    assert!(report.coupling.is_empty());
}

// ===========================================================================
// Scenario: Intent report classifies conventional commits
// ===========================================================================
#[test]
fn scenario_intent_report_conventional_commits() {
    // Given: conventional commit messages
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits = vec![
        commit(1000, "alice", "feat: add login", &["src/lib.rs"]),
        commit(2000, "bob", "fix: null pointer", &["src/lib.rs"]),
        commit(3000, "alice", "docs: update readme", &["src/lib.rs"]),
        commit(4000, "charlie", "fix: memory leak", &["src/lib.rs"]),
    ];

    // When
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    // Then: intent report counts by category
    let intent = report.intent.as_ref().expect("intent present");
    assert_eq!(intent.overall.feat, 1);
    assert_eq!(intent.overall.fix, 2);
    assert_eq!(intent.overall.docs, 1);
    assert_eq!(intent.overall.total, 4);
}

// ===========================================================================
// Scenario: Corrective ratio reflects fix and revert commits
// ===========================================================================
#[test]
fn scenario_corrective_ratio() {
    // Given: 2 fixes, 1 revert, 1 feat out of 4 total
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits = vec![
        commit(1000, "alice", "feat: init", &["src/lib.rs"]),
        commit(2000, "bob", "fix: bug1", &["src/lib.rs"]),
        commit(3000, "charlie", "fix: bug2", &["src/lib.rs"]),
        commit(4000, "alice", "revert: bad change", &["src/lib.rs"]),
    ];

    // When
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    // Then: corrective_ratio = (2 fixes + 1 revert) / 4 = 0.75
    let intent = report.intent.as_ref().unwrap();
    assert_eq!(intent.corrective_ratio, Some(0.75));
}

// ===========================================================================
// Scenario: Code age distribution has 5 buckets
// ===========================================================================
#[test]
fn scenario_code_age_distribution_buckets() {
    // Given: files with varying ages
    let now = 500 * DAY;
    let exp = export(vec![
        file_row("src/a.rs", "src", 50),
        file_row("src/b.rs", "src", 50),
    ]);
    let commits = vec![
        commit(now, "alice", "feat: recent", &["src/a.rs"]),
        commit(now - 400 * DAY, "bob", "feat: old", &["src/b.rs"]),
    ];

    // When
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    // Then: age distribution has 5 buckets
    let age = report.age_distribution.as_ref().unwrap();
    assert_eq!(age.buckets.len(), 5);
    assert_eq!(age.buckets[0].label, "0-30d");
    assert_eq!(age.buckets[4].label, "366d+");
}

// ===========================================================================
// Scenario: Age distribution percentages sum to ~1.0
// ===========================================================================
#[test]
fn scenario_age_distribution_pct_sum() {
    // Given: several files with different ages
    let now = 500 * DAY;
    let exp = export(vec![
        file_row("src/a.rs", "src", 50),
        file_row("src/b.rs", "src", 50),
        file_row("src/c.rs", "src", 50),
    ]);
    let commits = vec![
        commit(now, "alice", "feat: a", &["src/a.rs"]),
        commit(now - 60 * DAY, "bob", "feat: b", &["src/b.rs"]),
        commit(now - 400 * DAY, "charlie", "feat: c", &["src/c.rs"]),
    ];

    // When
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    // Then: bucket percentages sum to approximately 1.0
    let age = report.age_distribution.as_ref().unwrap();
    let total_pct: f64 = age.buckets.iter().map(|b| b.pct).sum();
    assert!((total_pct - 1.0).abs() < 0.01, "pct sum {total_pct} ≈ 1.0");
}

// ===========================================================================
// Scenario: Commits touching unknown files are ignored
// ===========================================================================
#[test]
fn scenario_unknown_files_ignored() {
    // Given: a commit that touches a file not in the export
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits = vec![
        commit(1000, "alice", "feat: init", &["src/lib.rs"]),
        commit(2000, "bob", "feat: unknown", &["unknown/file.rs"]),
    ];

    // When
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    // Then: only 1 file seen (the unknown file is skipped)
    assert_eq!(report.commits_scanned, 2);
    assert_eq!(report.files_seen, 1);
    assert_eq!(report.hotspots.len(), 1);
    assert_eq!(report.hotspots[0].path, "src/lib.rs");
}

// ===========================================================================
// Scenario: Child file kind rows are excluded from analysis
// ===========================================================================
#[test]
fn scenario_child_file_kind_excluded() {
    // Given: an export with a Child row
    let mut rows = vec![file_row("src/lib.rs", "src", 100)];
    rows.push(FileRow {
        path: "src/lib.rs".to_string(),
        module: "src".to_string(),
        lang: "Markdown".to_string(),
        kind: FileKind::Child,
        code: 5,
        comments: 0,
        blanks: 0,
        lines: 5,
        bytes: 50,
        tokens: 10,
    });
    let exp = export(rows);
    let commits = vec![commit(1000, "alice", "feat: init", &["src/lib.rs"])];

    // When
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    // Then: only the Parent row counts, lines = 100
    assert_eq!(report.hotspots.len(), 1);
    assert_eq!(report.hotspots[0].lines, 100);
}

// ===========================================================================
// Scenario: Git path normalization handles backslash and ./ prefix
// ===========================================================================
#[test]
fn scenario_backslash_paths_normalized() {
    // Given: commits with backslash paths
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits = vec![
        commit(1000, "alice", "feat: init", &["src\\lib.rs"]),
        commit(2000, "bob", "fix: it", &["./src/lib.rs"]),
    ];

    // When
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    // Then: both commits map to the same file
    assert_eq!(report.hotspots.len(), 1);
    assert_eq!(report.hotspots[0].commits, 2);
}

// ===========================================================================
// Scenario: Predictive churn with empty commits returns empty report
// ===========================================================================
#[test]
fn scenario_churn_empty_commits() {
    // Given: an export with files, but no commits
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);

    // When
    let report = build_predictive_churn_report(&exp, &[], Path::new("."));

    // Then: per_module is empty
    assert!(report.per_module.is_empty());
}

// ===========================================================================
// Scenario: Churn with steady commits has flat or non-negative slope
// ===========================================================================
#[test]
fn scenario_churn_steady_commits() {
    // Given: one commit per week for 5 weeks
    let week = 7 * DAY;
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits: Vec<GitCommit> = (1..=5)
        .map(|i| commit(i * week, "alice", "feat: weekly", &["src/lib.rs"]))
        .collect();

    // When
    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));

    // Then: the module is present and slope is flat (1 commit/week constant)
    let trend = report.per_module.get("src").expect("module present");
    // Constant rate → slope ≈ 0
    assert!(
        trend.slope.abs() < 0.1,
        "constant rate should have near-zero slope, got {}",
        trend.slope
    );
    assert_eq!(trend.classification, TrendClass::Flat);
}

// ===========================================================================
// Scenario: Churn with increasing activity has rising trend
// ===========================================================================
#[test]
fn scenario_churn_rising_trend() {
    // Given: increasing commits per week: 1, 2, 3, 4, 5
    let week = 7 * DAY;
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let mut commits = Vec::new();
    for w in 1..=5i64 {
        for _ in 0..w {
            commits.push(commit(w * week, "alice", "feat: more", &["src/lib.rs"]));
        }
    }

    // When
    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));

    // Then: positive slope → rising
    let trend = report.per_module.get("src").expect("module present");
    assert!(
        trend.slope > 0.0,
        "increasing activity should have positive slope"
    );
    assert_eq!(trend.classification, TrendClass::Rising);
}

// ===========================================================================
// Scenario: Churn with single commit has flat trend
// ===========================================================================
#[test]
fn scenario_churn_single_commit_flat() {
    // Given: a single commit (< 2 data points)
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits = vec![commit(7 * DAY, "alice", "feat: init", &["src/lib.rs"])];

    // When
    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));

    // Then: slope is 0.0 (regression needs ≥2 points)
    let trend = report.per_module.get("src").expect("module present");
    assert_eq!(trend.slope, 0.0);
    assert_eq!(trend.classification, TrendClass::Flat);
}

// ===========================================================================
// Scenario: Refresh trend rising when only recent activity
// ===========================================================================
#[test]
fn scenario_refresh_trend_rising() {
    // Given: all commits in the last 30 days, none before
    let now = 1000 * DAY;
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits = vec![
        commit(now - 5 * DAY, "alice", "feat: recent1", &["src/lib.rs"]),
        commit(now, "bob", "feat: recent2", &["src/lib.rs"]),
    ];

    // When
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    // Then: refresh_trend is Rising (recent > 0, prior = 0)
    let age = report.age_distribution.as_ref().unwrap();
    assert_eq!(age.refresh_trend, TrendClass::Rising);
    assert!(age.recent_refreshes > 0);
    assert_eq!(age.prior_refreshes, 0);
}

// ===========================================================================
// Scenario: Intent by_module attributes intents per touched module
// ===========================================================================
#[test]
fn scenario_intent_by_module() {
    // Given: commits touching different modules
    let exp = export(vec![
        file_row("api/handler.rs", "api", 100),
        file_row("db/query.rs", "db", 80),
    ]);
    let commits = vec![
        commit(1000, "alice", "feat: api feature", &["api/handler.rs"]),
        commit(2000, "bob", "fix: db bug", &["db/query.rs"]),
        commit(
            3000,
            "charlie",
            "feat: both",
            &["api/handler.rs", "db/query.rs"],
        ),
    ];

    // When
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    // Then: each module gets intent attribution
    let intent = report.intent.as_ref().unwrap();
    assert_eq!(intent.by_module.len(), 2);
    let api_intent = intent.by_module.iter().find(|m| m.module == "api").unwrap();
    let db_intent = intent.by_module.iter().find(|m| m.module == "db").unwrap();
    // api: 2 feats
    assert_eq!(api_intent.counts.feat, 2);
    // db: 1 fix + 1 feat
    assert_eq!(db_intent.counts.fix, 1);
    assert_eq!(db_intent.counts.feat, 1);
}

// ===========================================================================
// Scenario: Coupling jaccard and lift are computed
// ===========================================================================
#[test]
fn scenario_coupling_metrics_computed() {
    // Given: 4 commits, 2 touching both api+db, 1 touching only api, 1 only db
    let exp = export(vec![
        file_row("api/handler.rs", "api", 100),
        file_row("db/query.rs", "db", 80),
    ]);
    let commits = vec![
        commit(
            1000,
            "alice",
            "feat: both1",
            &["api/handler.rs", "db/query.rs"],
        ),
        commit(
            2000,
            "bob",
            "feat: both2",
            &["api/handler.rs", "db/query.rs"],
        ),
        commit(3000, "alice", "fix: api only", &["api/handler.rs"]),
        commit(4000, "bob", "fix: db only", &["db/query.rs"]),
    ];

    // When
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    // Then: coupling row with computed jaccard and lift
    assert_eq!(report.coupling.len(), 1);
    let c = &report.coupling[0];
    assert_eq!(c.count, 2); // 2 shared commits
    assert_eq!(c.n_left, Some(3)); // api: 3 touches
    assert_eq!(c.n_right, Some(3)); // db: 3 touches
    // Jaccard: 2 / (3+3-2) = 0.5
    assert_eq!(c.jaccard, Some(0.5));
    // Lift: (2*4) / (3*3) = 8/9 ≈ 0.8889
    let lift = c.lift.unwrap();
    assert!(
        (lift - 0.8889).abs() < 0.001,
        "lift should be ~0.889, got {lift}"
    );
}

// ===========================================================================
// Scenario: files_seen counts distinct files that appear in commits
// ===========================================================================
#[test]
fn scenario_files_seen_counts_distinct() {
    // Given: 3 commits touching 2 files
    let exp = export(vec![
        file_row("src/a.rs", "src", 50),
        file_row("src/b.rs", "src", 30),
    ]);
    let commits = vec![
        commit(1000, "alice", "feat: a", &["src/a.rs"]),
        commit(2000, "bob", "feat: b", &["src/b.rs"]),
        commit(3000, "alice", "fix: a", &["src/a.rs"]),
    ];

    // When
    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    // Then: 2 distinct files seen
    assert_eq!(report.files_seen, 2);
    assert_eq!(report.commits_scanned, 3);
}
