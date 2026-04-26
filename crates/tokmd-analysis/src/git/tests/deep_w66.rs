//! W66 deep tests for ``tokmd-analysis` Git module`.
//!
//! Exercises hotspot calculation, coupling metrics, freshness, bus factor,
//! intent classification, churn trends, and determinism.

use std::path::Path;

use super::super::{build_git_report, build_predictive_churn_report};
use tokmd_git::GitCommit;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────

const DAY: i64 = 86_400;

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

// ── Hotspot edge cases ──────────────────────────────────────────

mod hotspot_w66 {
    use super::*;

    #[test]
    fn empty_commits_produces_no_hotspots() {
        let export = make_export(vec![make_row("src/lib.rs", "src", 100)]);
        let r = build_git_report(Path::new("."), &export, &[]).unwrap();
        assert!(r.hotspots.is_empty());
        assert_eq!(r.commits_scanned, 0);
    }

    #[test]
    fn commits_referencing_unknown_files_ignored() {
        let export = make_export(vec![make_row("src/lib.rs", "src", 100)]);
        let commits = vec![make_commit(DAY, "a", "c1", &["unknown.rs"])];
        let r = build_git_report(Path::new("."), &export, &commits).unwrap();
        assert!(r.hotspots.is_empty());
    }

    #[test]
    fn hotspot_with_single_commit() {
        let export = make_export(vec![make_row("src/lib.rs", "src", 50)]);
        let commits = vec![make_commit(DAY, "a", "init", &["src/lib.rs"])];
        let r = build_git_report(Path::new("."), &export, &commits).unwrap();
        assert_eq!(r.hotspots.len(), 1);
        assert_eq!(r.hotspots[0].score, 50);
    }

    #[test]
    fn hotspot_tiebreak_by_path() {
        let export = make_export(vec![
            make_row("src/a.rs", "src", 100),
            make_row("src/b.rs", "src", 100),
        ]);
        let commits = vec![make_commit(DAY, "a", "c", &["src/a.rs", "src/b.rs"])];
        let r = build_git_report(Path::new("."), &export, &commits).unwrap();
        assert_eq!(r.hotspots[0].path, "src/a.rs");
        assert_eq!(r.hotspots[1].path, "src/b.rs");
    }
}

// ── Bus factor ──────────────────────────────────────────────────

mod bus_factor_w66 {
    use super::*;

    #[test]
    fn single_author_gives_bus_factor_one() {
        let export = make_export(vec![make_row("src/lib.rs", "src", 100)]);
        let commits = vec![
            make_commit(DAY, "alice", "c1", &["src/lib.rs"]),
            make_commit(2 * DAY, "alice", "c2", &["src/lib.rs"]),
        ];
        let r = build_git_report(Path::new("."), &export, &commits).unwrap();
        assert_eq!(r.bus_factor[0].authors, 1);
    }

    #[test]
    fn multiple_authors_counted() {
        let export = make_export(vec![make_row("src/lib.rs", "src", 100)]);
        let commits = vec![
            make_commit(DAY, "alice", "c1", &["src/lib.rs"]),
            make_commit(2 * DAY, "bob", "c2", &["src/lib.rs"]),
            make_commit(3 * DAY, "carol", "c3", &["src/lib.rs"]),
        ];
        let r = build_git_report(Path::new("."), &export, &commits).unwrap();
        assert_eq!(r.bus_factor[0].authors, 3);
    }

    #[test]
    fn bus_factor_sorted_ascending_by_authors() {
        let export = make_export(vec![
            make_row("src/a.rs", "mod_a", 50),
            make_row("src/b.rs", "mod_b", 50),
        ]);
        let commits = vec![
            make_commit(DAY, "alice", "c1", &["src/a.rs"]),
            make_commit(2 * DAY, "bob", "c2", &["src/b.rs"]),
            make_commit(3 * DAY, "carol", "c3", &["src/b.rs"]),
        ];
        let r = build_git_report(Path::new("."), &export, &commits).unwrap();
        assert_eq!(r.bus_factor[0].module, "mod_a");
        assert_eq!(r.bus_factor[0].authors, 1);
        assert_eq!(r.bus_factor[1].module, "mod_b");
        assert_eq!(r.bus_factor[1].authors, 2);
    }
}

// ── Coupling metric determinism ─────────────────────────────────

mod coupling_w66 {
    use super::*;

    #[test]
    fn no_coupling_when_single_module() {
        let export = make_export(vec![
            make_row("src/a.rs", "src", 50),
            make_row("src/b.rs", "src", 50),
        ]);
        let commits = vec![make_commit(DAY, "a", "c", &["src/a.rs", "src/b.rs"])];
        let r = build_git_report(Path::new("."), &export, &commits).unwrap();
        assert!(r.coupling.is_empty());
    }

    #[test]
    fn coupling_detected_across_modules() {
        let export = make_export(vec![
            make_row("src/a.rs", "mod_a", 50),
            make_row("src/b.rs", "mod_b", 50),
        ]);
        let commits = vec![make_commit(DAY, "a", "c", &["src/a.rs", "src/b.rs"])];
        let r = build_git_report(Path::new("."), &export, &commits).unwrap();
        assert_eq!(r.coupling.len(), 1);
        assert_eq!(r.coupling[0].count, 1);
    }

    #[test]
    fn coupling_sorted_by_count_desc() {
        let export = make_export(vec![
            make_row("src/a.rs", "alpha", 50),
            make_row("src/b.rs", "beta", 50),
            make_row("src/c.rs", "gamma", 50),
        ]);
        let commits = vec![
            make_commit(DAY, "a", "c1", &["src/a.rs", "src/b.rs"]),
            make_commit(2 * DAY, "a", "c2", &["src/a.rs", "src/b.rs"]),
            make_commit(3 * DAY, "a", "c3", &["src/a.rs", "src/c.rs"]),
        ];
        let r = build_git_report(Path::new("."), &export, &commits).unwrap();
        assert!(r.coupling.len() >= 2);
        assert!(r.coupling[0].count >= r.coupling[1].count);
    }

    #[test]
    fn coupling_jaccard_present() {
        let export = make_export(vec![
            make_row("src/a.rs", "mod_a", 50),
            make_row("src/b.rs", "mod_b", 50),
        ]);
        let commits = vec![
            make_commit(DAY, "a", "c1", &["src/a.rs", "src/b.rs"]),
            make_commit(2 * DAY, "a", "c2", &["src/a.rs"]),
        ];
        let r = build_git_report(Path::new("."), &export, &commits).unwrap();
        assert!(!r.coupling.is_empty());
        assert!(r.coupling[0].jaccard.is_some());
        let j = r.coupling[0].jaccard.unwrap();
        assert!(j > 0.0 && j <= 1.0);
    }
}

// ── Freshness ───────────────────────────────────────────────────

mod freshness_w66 {
    use super::*;

    #[test]
    fn all_recent_files_zero_stale() {
        let now = 400 * DAY;
        let export = make_export(vec![make_row("src/lib.rs", "src", 100)]);
        let commits = vec![make_commit(now - DAY, "a", "recent", &["src/lib.rs"])];
        let r = build_git_report(Path::new("."), &export, &commits).unwrap();
        assert_eq!(r.freshness.stale_files, 0);
        assert_eq!(r.freshness.stale_pct, 0.0);
    }

    #[test]
    fn old_file_marked_stale() {
        let export = make_export(vec![
            make_row("src/lib.rs", "src", 100),
            make_row("src/new.rs", "src", 50),
        ]);
        let commits = vec![
            make_commit(DAY, "a", "old", &["src/lib.rs"]),
            make_commit(400 * DAY, "a", "recent", &["src/new.rs"]),
        ];
        let r = build_git_report(Path::new("."), &export, &commits).unwrap();
        assert_eq!(r.freshness.stale_files, 1);
        assert!(r.freshness.stale_pct > 0.0);
    }

    #[test]
    fn freshness_by_module_populated() {
        let now = 100 * DAY;
        let export = make_export(vec![
            make_row("src/a.rs", "mod_a", 50),
            make_row("src/b.rs", "mod_b", 50),
        ]);
        let commits = vec![
            make_commit(now - 10 * DAY, "a", "c1", &["src/a.rs"]),
            make_commit(now - 20 * DAY, "a", "c2", &["src/b.rs"]),
        ];
        let r = build_git_report(Path::new("."), &export, &commits).unwrap();
        assert_eq!(r.freshness.by_module.len(), 2);
    }
}

// ── Churn trends ────────────────────────────────────────────────

mod churn_w66 {
    use super::*;

    #[test]
    fn empty_commits_produces_empty_churn() {
        let export = make_export(vec![make_row("src/lib.rs", "src", 100)]);
        let r = build_predictive_churn_report(&export, &[], Path::new("."));
        assert!(r.per_module.is_empty());
    }

    #[test]
    fn single_commit_produces_flat_trend() {
        let export = make_export(vec![make_row("src/lib.rs", "src", 100)]);
        let commits = vec![make_commit(DAY, "a", "init", &["src/lib.rs"])];
        let r = build_predictive_churn_report(&export, &commits, Path::new("."));
        let trend = r.per_module.get("src").unwrap();
        assert_eq!(trend.slope, 0.0);
    }

    #[test]
    fn churn_report_deterministic() {
        let export = make_export(vec![make_row("src/lib.rs", "src", 100)]);
        let week = 7 * DAY;
        let commits = vec![
            make_commit(week, "a", "c1", &["src/lib.rs"]),
            make_commit(2 * week, "a", "c2", &["src/lib.rs"]),
            make_commit(3 * week, "a", "c3", &["src/lib.rs"]),
        ];
        let r1 = build_predictive_churn_report(&export, &commits, Path::new("."));
        let r2 = build_predictive_churn_report(&export, &commits, Path::new("."));
        assert_eq!(
            serde_json::to_string(&r1).unwrap(),
            serde_json::to_string(&r2).unwrap(),
        );
    }
}

// ── Intent classification ───────────────────────────────────────

mod intent_w66 {
    use super::*;

    #[test]
    fn feat_commits_classified() {
        let export = make_export(vec![make_row("src/lib.rs", "src", 100)]);
        let commits = vec![
            make_commit(DAY, "a", "feat: new thing", &["src/lib.rs"]),
            make_commit(2 * DAY, "a", "fix: broken thing", &["src/lib.rs"]),
        ];
        let r = build_git_report(Path::new("."), &export, &commits).unwrap();
        let intent = r.intent.unwrap();
        assert_eq!(intent.overall.total, 2);
    }

    #[test]
    fn empty_commits_gives_zero_intent() {
        let export = make_export(vec![make_row("src/lib.rs", "src", 100)]);
        let r = build_git_report(Path::new("."), &export, &[]).unwrap();
        let intent = r.intent.unwrap();
        assert_eq!(intent.overall.total, 0);
        assert_eq!(intent.unknown_pct, 0.0);
    }

    #[test]
    fn corrective_ratio_present_when_commits_exist() {
        let export = make_export(vec![make_row("src/lib.rs", "src", 100)]);
        let commits = vec![
            make_commit(DAY, "a", "feat: add", &["src/lib.rs"]),
            make_commit(2 * DAY, "a", "fix: bug", &["src/lib.rs"]),
        ];
        let r = build_git_report(Path::new("."), &export, &commits).unwrap();
        let intent = r.intent.unwrap();
        assert!(intent.corrective_ratio.is_some());
    }
}

// ── Determinism ─────────────────────────────────────────────────

mod determinism_w66 {
    use super::*;

    #[test]
    fn git_report_deterministic() {
        let export = make_export(vec![
            make_row("src/a.rs", "mod_a", 50),
            make_row("src/b.rs", "mod_b", 80),
        ]);
        let commits = vec![
            make_commit(DAY, "alice", "feat: init", &["src/a.rs", "src/b.rs"]),
            make_commit(2 * DAY, "bob", "fix: bug", &["src/a.rs"]),
        ];
        let r1 = build_git_report(Path::new("."), &export, &commits).unwrap();
        let r2 = build_git_report(Path::new("."), &export, &commits).unwrap();
        assert_eq!(
            serde_json::to_string(&r1).unwrap(),
            serde_json::to_string(&r2).unwrap(),
        );
    }
}
