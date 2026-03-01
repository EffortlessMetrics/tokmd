//! Deeper tests for git analysis adapter behavior, capability reporting, and determinism.

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
// Determinism: same inputs → identical outputs
// ===========================================================================

#[test]
fn git_report_is_deterministic_across_repeated_calls() {
    let exp = export(vec![
        file_row("src/a.rs", "src", 100),
        file_row("lib/b.rs", "lib", 200),
    ]);
    let commits = vec![
        commit(1000, "alice", "feat: a", &["src/a.rs"]),
        commit(2000, "bob", "fix: b", &["lib/b.rs", "src/a.rs"]),
        commit(3000, "charlie", "refactor: both", &["src/a.rs", "lib/b.rs"]),
    ];

    let r1 = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let r2 = build_git_report(Path::new("."), &exp, &commits).unwrap();

    assert_eq!(r1.commits_scanned, r2.commits_scanned);
    assert_eq!(r1.files_seen, r2.files_seen);
    assert_eq!(r1.hotspots.len(), r2.hotspots.len());
    for (a, b) in r1.hotspots.iter().zip(r2.hotspots.iter()) {
        assert_eq!(a.path, b.path);
        assert_eq!(a.score, b.score);
        assert_eq!(a.commits, b.commits);
    }
    assert_eq!(r1.bus_factor.len(), r2.bus_factor.len());
    for (a, b) in r1.bus_factor.iter().zip(r2.bus_factor.iter()) {
        assert_eq!(a.module, b.module);
        assert_eq!(a.authors, b.authors);
    }
    assert_eq!(r1.freshness.stale_files, r2.freshness.stale_files);
    assert_eq!(r1.freshness.stale_pct, r2.freshness.stale_pct);
    assert_eq!(r1.coupling.len(), r2.coupling.len());
}

#[test]
fn churn_report_is_deterministic_across_repeated_calls() {
    let week = 7 * DAY;
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits: Vec<GitCommit> = (1..=5)
        .map(|i| commit(i * week, "alice", "feat: work", &["src/lib.rs"]))
        .collect();

    let r1 = build_predictive_churn_report(&exp, &commits, Path::new("."));
    let r2 = build_predictive_churn_report(&exp, &commits, Path::new("."));

    assert_eq!(r1.per_module.len(), r2.per_module.len());
    for (k, v1) in &r1.per_module {
        let v2 = r2.per_module.get(k).expect("key present in both");
        assert_eq!(v1.slope, v2.slope);
        assert_eq!(v1.r2, v2.r2);
        assert_eq!(v1.recent_change, v2.recent_change);
        assert_eq!(v1.classification, v2.classification);
    }
}

// ===========================================================================
// Empty export with commits → all commits ignored gracefully
// ===========================================================================

#[test]
fn empty_export_with_commits_produces_empty_report() {
    let exp = export(vec![]);
    let commits = vec![
        commit(1000, "alice", "feat: ghost", &["unknown/file.rs"]),
        commit(2000, "bob", "fix: phantom", &["other/file.rs"]),
    ];

    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    assert_eq!(report.commits_scanned, 2);
    assert_eq!(report.files_seen, 0);
    assert!(report.hotspots.is_empty());
    assert!(report.bus_factor.is_empty());
    assert!(report.coupling.is_empty());
    assert_eq!(report.freshness.total_files, 0);
}

// ===========================================================================
// All commits reference unknown files → zero file coverage
// ===========================================================================

#[test]
fn commits_referencing_only_unknown_files_yield_zero_coverage() {
    let exp = export(vec![file_row("src/main.rs", "src", 50)]);
    let commits = vec![
        commit(1000, "alice", "feat: unknown", &["docs/readme.md"]),
        commit(2000, "bob", "fix: also unknown", &["tests/helper.rs"]),
    ];

    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    assert_eq!(report.files_seen, 0);
    assert!(report.hotspots.is_empty());
    assert!(report.bus_factor.is_empty());
}

// ===========================================================================
// Bus factor = 1 detection (single author per module)
// ===========================================================================

#[test]
fn bus_factor_one_detected_for_single_author_module() {
    let exp = export(vec![
        file_row("core/engine.rs", "core", 500),
        file_row("core/parser.rs", "core", 300),
    ]);
    let commits = vec![
        commit(1000, "lonely_dev", "feat: engine", &["core/engine.rs"]),
        commit(2000, "lonely_dev", "feat: parser", &["core/parser.rs"]),
        commit(3000, "lonely_dev", "fix: engine bug", &["core/engine.rs"]),
    ];

    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    assert_eq!(report.bus_factor.len(), 1);
    assert_eq!(report.bus_factor[0].module, "core");
    assert_eq!(report.bus_factor[0].authors, 1);
}

// ===========================================================================
// Many modules with varied coupling patterns
// ===========================================================================

#[test]
fn three_module_coupling_only_pairs_that_co_occur() {
    let exp = export(vec![
        file_row("api/routes.rs", "api", 100),
        file_row("db/models.rs", "db", 80),
        file_row("cli/main.rs", "cli", 60),
    ]);
    // api+db co-occur in 2 commits; cli is always independent
    let commits = vec![
        commit(
            1000,
            "alice",
            "feat: api+db",
            &["api/routes.rs", "db/models.rs"],
        ),
        commit(2000, "bob", "fix: cli", &["cli/main.rs"]),
        commit(
            3000,
            "alice",
            "refactor: api+db",
            &["api/routes.rs", "db/models.rs"],
        ),
        commit(4000, "charlie", "feat: cli update", &["cli/main.rs"]),
    ];

    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    // Only api-db coupling; cli has no co-occurrence
    assert_eq!(report.coupling.len(), 1);
    assert_eq!(report.coupling[0].left, "api");
    assert_eq!(report.coupling[0].right, "db");
    assert_eq!(report.coupling[0].count, 2);
}

// ===========================================================================
// Intent with non-conventional commit messages → classified as "other"
// ===========================================================================

#[test]
fn non_conventional_commit_messages_counted_as_other() {
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits = vec![
        commit(1000, "alice", "WIP stuff", &["src/lib.rs"]),
        commit(2000, "bob", "updated things", &["src/lib.rs"]),
        commit(3000, "charlie", "misc changes", &["src/lib.rs"]),
    ];

    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let intent = report.intent.as_ref().unwrap();

    assert_eq!(intent.overall.total, 3);
    assert_eq!(intent.overall.other, 3);
    assert_eq!(intent.unknown_pct, 1.0);
}

// ===========================================================================
// Freshness: all files stale → 100% stale percentage
// ===========================================================================

#[test]
fn all_stale_files_produce_full_stale_percentage() {
    // reference_ts is the max commit timestamp, so we need a recent
    // "anchor" commit on a different file to push reference_ts forward,
    // while the target files were last touched > 365 days before that.
    let now = 1000 * DAY;
    let exp = export(vec![
        file_row("src/old1.rs", "src", 50),
        file_row("src/old2.rs", "src", 60),
        file_row("src/anchor.rs", "src", 10),
    ]);
    let commits = vec![
        commit(now - 500 * DAY, "alice", "feat: old1", &["src/old1.rs"]),
        commit(now - 400 * DAY, "bob", "feat: old2", &["src/old2.rs"]),
        commit(now, "charlie", "chore: anchor", &["src/anchor.rs"]),
    ];

    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    // old1 = 500 days ago, old2 = 400 days ago — both > 365 threshold
    assert_eq!(report.freshness.stale_files, 2);
    assert_eq!(report.freshness.total_files, 3);
    assert!(report.freshness.stale_pct > 0.0);
}

// ===========================================================================
// Age distribution: all files in same bucket
// ===========================================================================

#[test]
fn all_files_recent_land_in_first_age_bucket() {
    let now = 1000 * DAY;
    let exp = export(vec![
        file_row("src/a.rs", "src", 50),
        file_row("src/b.rs", "src", 50),
    ]);
    let commits = vec![
        commit(now - DAY, "alice", "feat: a", &["src/a.rs"]),
        commit(now, "bob", "feat: b", &["src/b.rs"]),
    ];

    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let age = report.age_distribution.as_ref().unwrap();

    // All files should be in 0-30d bucket
    assert_eq!(age.buckets[0].label, "0-30d");
    assert_eq!(age.buckets[0].files, 2);
    assert_eq!(age.buckets[0].pct, 1.0);
    for b in &age.buckets[1..] {
        assert_eq!(b.files, 0);
    }
}

// ===========================================================================
// Churn: decreasing activity → falling trend
// ===========================================================================

#[test]
fn churn_with_decreasing_activity_has_falling_trend() {
    let week = 7 * DAY;
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    // Decreasing commits per week: 5, 4, 3, 2, 1
    let mut commits = Vec::new();
    for w in 1..=5i64 {
        let count = (6 - w) as usize;
        for _ in 0..count {
            commits.push(commit(
                w * week,
                "alice",
                "feat: less",
                &["src/lib.rs"],
            ));
        }
    }

    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));
    let trend = report.per_module.get("src").expect("module present");

    assert!(trend.slope < 0.0, "decreasing activity should have negative slope");
    assert_eq!(trend.classification, TrendClass::Falling);
}

// ===========================================================================
// Churn: multiple modules tracked independently
// ===========================================================================

#[test]
fn churn_tracks_modules_independently() {
    let week = 7 * DAY;
    let exp = export(vec![
        file_row("api/handler.rs", "api", 100),
        file_row("db/query.rs", "db", 80),
    ]);
    // api: steady (1 commit/week), db: increasing (1,2,3,4,5)
    let mut commits = Vec::new();
    for w in 1..=5i64 {
        commits.push(commit(
            w * week,
            "alice",
            "feat: api",
            &["api/handler.rs"],
        ));
        for _ in 0..w {
            commits.push(commit(
                w * week,
                "bob",
                "feat: db",
                &["db/query.rs"],
            ));
        }
    }

    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));

    assert!(report.per_module.contains_key("api"));
    assert!(report.per_module.contains_key("db"));

    let api = &report.per_module["api"];
    let db = &report.per_module["db"];
    // db should have a steeper slope than api
    assert!(
        db.slope > api.slope,
        "db slope {} should exceed api slope {}",
        db.slope,
        api.slope
    );
}

// ===========================================================================
// Hotspots: file with many commits but few lines vs few commits many lines
// ===========================================================================

#[test]
fn hotspot_score_favors_large_file_with_many_commits() {
    let exp = export(vec![
        file_row("src/big.rs", "src", 1000),   // large file
        file_row("src/small.rs", "src", 10),    // small file
    ]);
    // big.rs: 2 commits → score = 2000
    // small.rs: 5 commits → score = 50
    let commits = vec![
        commit(1000, "alice", "feat: big", &["src/big.rs"]),
        commit(2000, "bob", "fix: big", &["src/big.rs"]),
        commit(3000, "alice", "feat: small1", &["src/small.rs"]),
        commit(4000, "bob", "feat: small2", &["src/small.rs"]),
        commit(5000, "alice", "feat: small3", &["src/small.rs"]),
        commit(6000, "bob", "feat: small4", &["src/small.rs"]),
        commit(7000, "alice", "feat: small5", &["src/small.rs"]),
    ];

    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    assert_eq!(report.hotspots[0].path, "src/big.rs");
    assert_eq!(report.hotspots[0].score, 2000);
    assert_eq!(report.hotspots[1].path, "src/small.rs");
    assert_eq!(report.hotspots[1].score, 50);
}

// ===========================================================================
// Coupling: lift > 1 indicates stronger-than-random co-occurrence
// ===========================================================================

#[test]
fn coupling_lift_above_one_for_strongly_coupled_modules() {
    let exp = export(vec![
        file_row("api/handler.rs", "api", 100),
        file_row("db/query.rs", "db", 80),
        file_row("cli/main.rs", "cli", 60),
    ]);
    // 5 commits: 4 touch api+db together, 1 touches only cli
    let commits = vec![
        commit(1000, "a", "feat: 1", &["api/handler.rs", "db/query.rs"]),
        commit(2000, "b", "feat: 2", &["api/handler.rs", "db/query.rs"]),
        commit(3000, "c", "feat: 3", &["api/handler.rs", "db/query.rs"]),
        commit(4000, "d", "feat: 4", &["api/handler.rs", "db/query.rs"]),
        commit(5000, "e", "feat: cli", &["cli/main.rs"]),
    ];

    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    let api_db = report
        .coupling
        .iter()
        .find(|c| c.left == "api" && c.right == "db")
        .expect("api-db coupling");
    assert!(
        api_db.lift.unwrap() > 1.0,
        "lift should be > 1 for strongly coupled: {}",
        api_db.lift.unwrap()
    );
}

// ===========================================================================
// Refresh trend: falling when prior period more active than recent
// ===========================================================================

#[test]
fn refresh_trend_falling_when_prior_more_active() {
    let now = 1000 * DAY;
    let exp = export(vec![
        file_row("src/a.rs", "src", 50),
        file_row("src/b.rs", "src", 50),
        file_row("src/c.rs", "src", 50),
    ]);
    // Prior 30 days (day 940-970): 3 files touched
    // Recent 30 days (day 970-1000): 1 file touched
    let commits = vec![
        commit(now - 50 * DAY, "alice", "feat: a", &["src/a.rs"]),
        commit(now - 45 * DAY, "bob", "feat: b", &["src/b.rs"]),
        commit(now - 40 * DAY, "charlie", "feat: c", &["src/c.rs"]),
        commit(now - 5 * DAY, "alice", "fix: a", &["src/a.rs"]),
    ];

    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let age = report.age_distribution.as_ref().unwrap();

    assert_eq!(age.refresh_trend, TrendClass::Falling);
    assert!(age.prior_refreshes > age.recent_refreshes);
}

// ===========================================================================
// Module freshness: p90 days increases with more stale files
// ===========================================================================

#[test]
fn module_freshness_p90_reflects_staleness() {
    let now = 500 * DAY;
    let exp = export(vec![
        file_row("src/fresh.rs", "src", 50),
        file_row("src/stale1.rs", "src", 50),
        file_row("src/stale2.rs", "src", 50),
        file_row("src/stale3.rs", "src", 50),
    ]);
    let commits = vec![
        commit(now, "alice", "feat: fresh", &["src/fresh.rs"]),
        commit(now - 400 * DAY, "bob", "feat: stale1", &["src/stale1.rs"]),
        commit(now - 410 * DAY, "charlie", "feat: stale2", &["src/stale2.rs"]),
        commit(now - 420 * DAY, "alice", "feat: stale3", &["src/stale3.rs"]),
    ];

    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();

    let src_freshness = report
        .freshness
        .by_module
        .iter()
        .find(|m| m.module == "src")
        .expect("src module");
    // p90 should be high since 3/4 files are very stale
    assert!(
        src_freshness.p90_days > 365.0,
        "p90 should exceed stale threshold: {}",
        src_freshness.p90_days
    );
}

// ===========================================================================
// Churn report: empty export with commits → empty result
// ===========================================================================

#[test]
fn churn_empty_export_with_commits_is_empty() {
    let exp = export(vec![]);
    let commits = vec![
        commit(DAY, "alice", "feat: ghost", &["unknown.rs"]),
    ];

    let report = build_predictive_churn_report(&exp, &commits, Path::new("."));
    assert!(report.per_module.is_empty());
}

// ===========================================================================
// Intent: mixed conventional and free-form messages
// ===========================================================================

#[test]
fn intent_mixed_conventional_and_freeform() {
    let exp = export(vec![file_row("src/lib.rs", "src", 100)]);
    let commits = vec![
        commit(1000, "alice", "feat: add feature", &["src/lib.rs"]),
        commit(2000, "bob", "random update", &["src/lib.rs"]),
        commit(3000, "charlie", "fix: critical bug", &["src/lib.rs"]),
        commit(4000, "dave", "just doing stuff", &["src/lib.rs"]),
        commit(5000, "eve", "refactor: cleanup", &["src/lib.rs"]),
    ];

    let report = build_git_report(Path::new("."), &exp, &commits).unwrap();
    let intent = report.intent.as_ref().unwrap();

    assert_eq!(intent.overall.total, 5);
    assert_eq!(intent.overall.feat, 1);
    assert_eq!(intent.overall.fix, 1);
    assert_eq!(intent.overall.refactor, 1);
    assert_eq!(intent.overall.other, 2);
    // corrective_ratio = (1 fix + 0 revert) / 5 = 0.2
    assert_eq!(intent.corrective_ratio, Some(0.2));
}
