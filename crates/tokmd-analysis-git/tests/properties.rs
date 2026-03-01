//! Property-based tests for `tokmd-analysis-git`.
//!
//! Uses `proptest` to verify invariants that must hold for all inputs.

use std::path::Path;

use proptest::prelude::*;

use tokmd_analysis_git::{build_git_report, build_predictive_churn_report};
use tokmd_analysis_types::TrendClass;
use tokmd_git::GitCommit;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

const DAY: i64 = 86_400;
const WEEK: i64 = 7 * DAY;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_file_row() -> impl Strategy<Value = FileRow> {
    (
        "[a-z]{1,4}/[a-z]{1,6}\\.rs", // path
        "[a-z]{1,4}",                 // module
        1..500usize,                  // lines
    )
        .prop_map(|(path, module, lines)| FileRow {
            path,
            module,
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: lines,
            comments: 0,
            blanks: 0,
            lines,
            bytes: lines * 40,
            tokens: lines * 3,
        })
}

fn arb_export() -> impl Strategy<Value = (ExportData, Vec<String>)> {
    proptest::collection::vec(arb_file_row(), 1..8).prop_map(|rows| {
        let paths: Vec<String> = rows.iter().map(|r| r.path.clone()).collect();
        let export = ExportData {
            rows,
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        };
        (export, paths)
    })
}

fn arb_commit(paths: Vec<String>) -> impl Strategy<Value = GitCommit> {
    let n = paths.len();
    (
        1..200i64,               // timestamp (in weeks)
        "[a-z]{3,6}@test\\.com", // author
        prop_oneof![
            Just("feat: feature".to_string()),
            Just("fix: bugfix".to_string()),
            Just("refactor: cleanup".to_string()),
            Just("docs: update".to_string()),
            Just("chore: maintenance".to_string()),
        ],
        proptest::collection::vec(0..n, 1..=n.clamp(1, 4)), // file indices
    )
        .prop_map(move |(ts_weeks, author, subject, indices)| {
            let files: Vec<String> = indices
                .into_iter()
                .map(|i| paths[i % paths.len()].clone())
                .collect();
            GitCommit {
                timestamp: ts_weeks * WEEK,
                author,
                hash: None,
                subject,
                files,
            }
        })
}

fn arb_scenario() -> impl Strategy<Value = (ExportData, Vec<GitCommit>)> {
    arb_export().prop_flat_map(|(export, paths)| {
        let commits_strategy = proptest::collection::vec(arb_commit(paths), 0..20);
        (Just(export), commits_strategy)
    })
}

// ===========================================================================
// Property: commits_scanned always equals input length
// ===========================================================================
proptest! {
    #[test]
    fn prop_commits_scanned_equals_input(
        (export, commits) in arb_scenario()
    ) {
        let report = build_git_report(Path::new("."), &export, &commits).unwrap();
        prop_assert_eq!(report.commits_scanned, commits.len());
    }
}

// ===========================================================================
// Property: files_seen never exceeds export row count
// ===========================================================================
proptest! {
    #[test]
    fn prop_files_seen_bounded(
        (export, commits) in arb_scenario()
    ) {
        let parent_count = export.rows.iter().filter(|r| r.kind == FileKind::Parent).count();
        let report = build_git_report(Path::new("."), &export, &commits).unwrap();
        prop_assert!(
            report.files_seen <= parent_count,
            "files_seen {} > parent rows {}", report.files_seen, parent_count,
        );
    }
}

// ===========================================================================
// Property: hotspot score = lines * commits for each row
// ===========================================================================
proptest! {
    #[test]
    fn prop_hotspot_score_invariant(
        (export, commits) in arb_scenario()
    ) {
        let report = build_git_report(Path::new("."), &export, &commits).unwrap();
        for h in &report.hotspots {
            prop_assert_eq!(
                h.score, h.lines * h.commits,
                "score mismatch for {} : {} != {} * {}", h.path, h.score, h.lines, h.commits,
            );
        }
    }
}

// ===========================================================================
// Property: hotspots are sorted by score descending
// ===========================================================================
proptest! {
    #[test]
    fn prop_hotspots_sorted_desc(
        (export, commits) in arb_scenario()
    ) {
        let report = build_git_report(Path::new("."), &export, &commits).unwrap();
        for window in report.hotspots.windows(2) {
            prop_assert!(
                window[0].score >= window[1].score,
                "hotspots not sorted: {} < {}", window[0].score, window[1].score,
            );
        }
    }
}

// ===========================================================================
// Property: bus_factor sorted by authors ascending then module name
// ===========================================================================
proptest! {
    #[test]
    fn prop_bus_factor_sorted(
        (export, commits) in arb_scenario()
    ) {
        let report = build_git_report(Path::new("."), &export, &commits).unwrap();
        for window in report.bus_factor.windows(2) {
            let ok = window[0].authors < window[1].authors
                || (window[0].authors == window[1].authors && window[0].module <= window[1].module);
            prop_assert!(ok, "bus_factor not sorted: {:?} vs {:?}", window[0], window[1]);
        }
    }
}

// ===========================================================================
// Property: bus_factor author count is always >= 1
// ===========================================================================
proptest! {
    #[test]
    fn prop_bus_factor_authors_positive(
        (export, commits) in arb_scenario()
    ) {
        let report = build_git_report(Path::new("."), &export, &commits).unwrap();
        for bf in &report.bus_factor {
            prop_assert!(bf.authors >= 1, "bus factor for {} has 0 authors", bf.module);
        }
    }
}

// ===========================================================================
// Property: freshness stale_pct in [0.0, 1.0]
// ===========================================================================
proptest! {
    #[test]
    fn prop_freshness_stale_pct_bounded(
        (export, commits) in arb_scenario()
    ) {
        let report = build_git_report(Path::new("."), &export, &commits).unwrap();
        prop_assert!(report.freshness.stale_pct >= 0.0);
        prop_assert!(report.freshness.stale_pct <= 1.0);
    }
}

// ===========================================================================
// Property: freshness stale_files <= total_files
// ===========================================================================
proptest! {
    #[test]
    fn prop_freshness_stale_le_total(
        (export, commits) in arb_scenario()
    ) {
        let report = build_git_report(Path::new("."), &export, &commits).unwrap();
        prop_assert!(
            report.freshness.stale_files <= report.freshness.total_files,
            "stale {} > total {}", report.freshness.stale_files, report.freshness.total_files,
        );
    }
}

// ===========================================================================
// Property: module freshness rows have non-negative values
// ===========================================================================
proptest! {
    #[test]
    fn prop_freshness_module_nonnegative(
        (export, commits) in arb_scenario()
    ) {
        let report = build_git_report(Path::new("."), &export, &commits).unwrap();
        for m in &report.freshness.by_module {
            prop_assert!(m.avg_days >= 0.0, "avg_days negative for {}", m.module);
            prop_assert!(m.p90_days >= 0.0, "p90_days negative for {}", m.module);
            prop_assert!(m.stale_pct >= 0.0 && m.stale_pct <= 1.0);
        }
    }
}

// ===========================================================================
// Property: coupling jaccard in (0, 1]
// ===========================================================================
proptest! {
    #[test]
    fn prop_coupling_jaccard_bounded(
        (export, commits) in arb_scenario()
    ) {
        let report = build_git_report(Path::new("."), &export, &commits).unwrap();
        for c in &report.coupling {
            if let Some(j) = c.jaccard {
                prop_assert!(j > 0.0 && j <= 1.0, "jaccard out of range: {}", j);
            }
        }
    }
}

// ===========================================================================
// Property: coupling lift is positive when present
// ===========================================================================
proptest! {
    #[test]
    fn prop_coupling_lift_positive(
        (export, commits) in arb_scenario()
    ) {
        let report = build_git_report(Path::new("."), &export, &commits).unwrap();
        for c in &report.coupling {
            if let Some(l) = c.lift {
                prop_assert!(l > 0.0, "lift should be positive, got {}", l);
            }
        }
    }
}

// ===========================================================================
// Property: coupling rows are sorted by count descending
// ===========================================================================
proptest! {
    #[test]
    fn prop_coupling_sorted_desc(
        (export, commits) in arb_scenario()
    ) {
        let report = build_git_report(Path::new("."), &export, &commits).unwrap();
        for window in report.coupling.windows(2) {
            prop_assert!(
                window[0].count >= window[1].count,
                "coupling not sorted: {} < {}", window[0].count, window[1].count,
            );
        }
    }
}

// ===========================================================================
// Property: age distribution has exactly 5 buckets
// ===========================================================================
proptest! {
    #[test]
    fn prop_age_distribution_5_buckets(
        (export, commits) in arb_scenario()
    ) {
        let report = build_git_report(Path::new("."), &export, &commits).unwrap();
        if let Some(age) = &report.age_distribution {
            prop_assert_eq!(age.buckets.len(), 5);
        }
    }
}

// ===========================================================================
// Property: age distribution percentages sum to ~1.0 when files present
// ===========================================================================
proptest! {
    #[test]
    fn prop_age_distribution_pct_sum(
        (export, commits) in arb_scenario()
    ) {
        let report = build_git_report(Path::new("."), &export, &commits).unwrap();
        if let Some(age) = &report.age_distribution {
            let total: f64 = age.buckets.iter().map(|b| b.pct).sum();
            let total_files: usize = age.buckets.iter().map(|b| b.files).sum();
            if total_files > 0 {
                prop_assert!(
                    (total - 1.0).abs() < 0.01,
                    "pct sum {} not close to 1.0", total,
                );
            }
        }
    }
}

// ===========================================================================
// Property: age distribution bucket percentages are non-negative
// ===========================================================================
proptest! {
    #[test]
    fn prop_age_distribution_pct_nonneg(
        (export, commits) in arb_scenario()
    ) {
        let report = build_git_report(Path::new("."), &export, &commits).unwrap();
        if let Some(age) = &report.age_distribution {
            for b in &age.buckets {
                prop_assert!(b.pct >= 0.0, "negative pct in bucket {}", b.label);
            }
        }
    }
}

// ===========================================================================
// Property: refresh_trend is a valid TrendClass
// ===========================================================================
proptest! {
    #[test]
    fn prop_refresh_trend_valid(
        (export, commits) in arb_scenario()
    ) {
        let report = build_git_report(Path::new("."), &export, &commits).unwrap();
        if let Some(age) = &report.age_distribution {
            let _ = match age.refresh_trend {
                TrendClass::Rising | TrendClass::Flat | TrendClass::Falling => true,
            };
        }
    }
}

// ===========================================================================
// Property: intent total equals commits_scanned
// ===========================================================================
proptest! {
    #[test]
    fn prop_intent_total_equals_commits(
        (export, commits) in arb_scenario()
    ) {
        let report = build_git_report(Path::new("."), &export, &commits).unwrap();
        if let Some(intent) = &report.intent {
            prop_assert_eq!(
                intent.overall.total, commits.len(),
                "intent total {} != commits_scanned {}", intent.overall.total, commits.len(),
            );
        }
    }
}

// ===========================================================================
// Property: intent corrective_ratio in [0, 1] when present
// ===========================================================================
proptest! {
    #[test]
    fn prop_intent_corrective_ratio_bounded(
        (export, commits) in arb_scenario()
    ) {
        let report = build_git_report(Path::new("."), &export, &commits).unwrap();
        if let Some(intent) = &report.intent
            && let Some(ratio) = intent.corrective_ratio
        {
            prop_assert!((0.0..=1.0).contains(&ratio), "corrective_ratio {} out of [0,1]", ratio);
        }
    }
}

// ===========================================================================
// Property: empty commits → empty git report fields
// ===========================================================================
proptest! {
    #[test]
    fn prop_empty_commits_empty_report(export_and_paths in arb_export()) {
        let (export, _) = export_and_paths;
        let report = build_git_report(Path::new("."), &export, &[]).unwrap();
        prop_assert_eq!(report.commits_scanned, 0);
        prop_assert_eq!(report.files_seen, 0);
        prop_assert!(report.hotspots.is_empty());
        prop_assert!(report.bus_factor.is_empty());
        prop_assert!(report.coupling.is_empty());
    }
}

// ===========================================================================
// Property: churn per_module keys are subset of export modules
// ===========================================================================
proptest! {
    #[test]
    fn prop_churn_modules_subset(
        (export, commits) in arb_scenario()
    ) {
        let report = build_predictive_churn_report(&export, &commits, Path::new("."));
        let export_modules: std::collections::BTreeSet<String> =
            export.rows.iter().map(|r| r.module.clone()).collect();
        for module in report.per_module.keys() {
            prop_assert!(
                export_modules.contains(module),
                "churn module '{}' not in export", module,
            );
        }
    }
}

// ===========================================================================
// Property: churn classification matches slope sign
// ===========================================================================
proptest! {
    #[test]
    fn prop_churn_classification_matches_slope(
        (export, commits) in arb_scenario()
    ) {
        let report = build_predictive_churn_report(&export, &commits, Path::new("."));
        for trend in report.per_module.values() {
            match trend.classification {
                TrendClass::Rising => prop_assert!(trend.slope > 0.0),
                TrendClass::Falling => prop_assert!(trend.slope < 0.0),
                TrendClass::Flat => prop_assert!(trend.slope.abs() <= 0.01),
            }
        }
    }
}

// ===========================================================================
// Property: churn r2 in [0, 1]
// ===========================================================================
proptest! {
    #[test]
    fn prop_churn_r2_bounded(
        (export, commits) in arb_scenario()
    ) {
        let report = build_predictive_churn_report(&export, &commits, Path::new("."));
        for trend in report.per_module.values() {
            prop_assert!(trend.r2 >= 0.0 && trend.r2 <= 1.0, "r2 out of [0,1]: {}", trend.r2);
        }
    }
}

// ===========================================================================
// Property: git report is deterministic
// ===========================================================================
proptest! {
    #[test]
    fn prop_deterministic(
        (export, commits) in arb_scenario()
    ) {
        let r1 = build_git_report(Path::new("."), &export, &commits).unwrap();
        let r2 = build_git_report(Path::new("."), &export, &commits).unwrap();
        prop_assert_eq!(r1.commits_scanned, r2.commits_scanned);
        prop_assert_eq!(r1.files_seen, r2.files_seen);
        prop_assert_eq!(r1.hotspots.len(), r2.hotspots.len());
        prop_assert_eq!(r1.bus_factor.len(), r2.bus_factor.len());
        prop_assert_eq!(r1.coupling.len(), r2.coupling.len());
    }
}
