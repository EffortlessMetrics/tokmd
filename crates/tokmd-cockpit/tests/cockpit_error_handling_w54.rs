//! Comprehensive error handling and edge case tests for tokmd-cockpit.

use tokmd_cockpit::determinism::{hash_cargo_lock, hash_files_from_paths};
use tokmd_cockpit::{
    FileStat, TrendDirection, compute_code_health, compute_composition, compute_metric_trend,
    compute_risk, detect_contracts, format_signed_f64, generate_review_plan, round_pct, sparkline,
    trend_direction_label,
};

// ── compute_composition edge cases ────────────────────────────────────

#[test]
fn composition_empty_files() {
    let files: Vec<&str> = vec![];
    let c = compute_composition(&files);
    assert_eq!(c.code_pct, 0.0);
    assert_eq!(c.test_pct, 0.0);
    assert_eq!(c.docs_pct, 0.0);
    assert_eq!(c.config_pct, 0.0);
    assert_eq!(c.test_ratio, 0.0);
}

#[test]
fn composition_only_test_files() {
    let files = vec!["src/test_main.rs", "tests/test_helper.rs"];
    let c = compute_composition(&files);
    assert_eq!(c.test_ratio, 1.0, "all test, no code → ratio 1.0");
    assert!(c.test_pct > 0.0);
    assert_eq!(c.code_pct, 0.0);
}

#[test]
fn composition_only_docs() {
    let files = vec!["README.md", "docs/guide.md"];
    let c = compute_composition(&files);
    assert_eq!(c.docs_pct, 1.0);
    assert_eq!(c.code_pct, 0.0);
    assert_eq!(c.test_ratio, 0.0);
}

#[test]
fn composition_mixed_files() {
    let files = vec![
        "src/lib.rs",
        "src/tests/test_lib.rs",
        "README.md",
        "Cargo.toml",
    ];
    let c = compute_composition(&files);
    assert!(c.code_pct > 0.0);
    assert!(c.test_pct > 0.0);
    assert!(c.docs_pct > 0.0);
    assert!(c.config_pct > 0.0);
}

#[test]
fn composition_unrecognized_extensions() {
    let files = vec!["data.bin", "image.png", "archive.tar.gz"];
    let c = compute_composition(&files);
    // None of these match known categories
    assert_eq!(c.code_pct, 0.0);
    assert_eq!(c.test_pct, 0.0);
    assert_eq!(c.test_ratio, 0.0);
}

// ── detect_contracts edge cases ───────────────────────────────────────

#[test]
fn contracts_empty_files() {
    let files: Vec<&str> = vec![];
    let c = detect_contracts(&files);
    assert!(!c.api_changed);
    assert!(!c.cli_changed);
    assert!(!c.schema_changed);
    assert_eq!(c.breaking_indicators, 0);
}

#[test]
fn contracts_api_change_detected() {
    let files = vec!["crates/tokmd-core/src/lib.rs"];
    let c = detect_contracts(&files);
    assert!(c.api_changed);
    assert_eq!(c.breaking_indicators, 1);
}

#[test]
fn contracts_schema_change_detected() {
    let files = vec!["docs/schema.json"];
    let c = detect_contracts(&files);
    assert!(c.schema_changed);
    assert_eq!(c.breaking_indicators, 1);
}

#[test]
fn contracts_cli_change_detected() {
    let files = vec!["crates/tokmd/src/commands/lang.rs"];
    let c = detect_contracts(&files);
    assert!(c.cli_changed);
}

#[test]
fn contracts_all_changes() {
    let files = vec![
        "crates/tokmd-core/src/lib.rs",
        "crates/tokmd/src/commands/gate.rs",
        "docs/schema.json",
    ];
    let c = detect_contracts(&files);
    assert!(c.api_changed);
    assert!(c.cli_changed);
    assert!(c.schema_changed);
    assert_eq!(c.breaking_indicators, 2); // api + schema
}

// ── compute_code_health edge cases ────────────────────────────────────

#[test]
fn code_health_empty_file_stats() {
    let contracts = detect_contracts::<&str>(&[]);
    let h = compute_code_health(&[], &contracts);
    assert_eq!(h.score, 100);
    assert_eq!(h.grade, "A");
    assert_eq!(h.large_files_touched, 0);
    assert_eq!(h.avg_file_size, 0);
    assert!(h.warnings.is_empty());
}

#[test]
fn code_health_with_large_files() {
    let stats = vec![
        FileStat {
            path: "big.rs".into(),
            insertions: 400,
            deletions: 200,
        },
        FileStat {
            path: "small.rs".into(),
            insertions: 10,
            deletions: 5,
        },
    ];
    let contracts = detect_contracts::<&str>(&[]);
    let h = compute_code_health(&stats, &contracts);
    assert_eq!(h.large_files_touched, 1);
    assert!(h.warnings.len() == 1);
}

#[test]
fn code_health_with_breaking_contracts() {
    let stats = vec![FileStat {
        path: "lib.rs".into(),
        insertions: 10,
        deletions: 5,
    }];
    let contracts = detect_contracts(&["crates/tokmd-core/src/lib.rs", "docs/schema.json"]);
    let h = compute_code_health(&stats, &contracts);
    // breaking_indicators > 0 → score reduced by 20
    assert!(h.score <= 80);
}

// ── compute_risk edge cases ──────────────────────────────────────────

#[test]
fn risk_empty_stats() {
    let contracts = detect_contracts::<&str>(&[]);
    let health = compute_code_health(&[], &contracts);
    let r = compute_risk(&[], &contracts, &health);
    assert!(r.hotspots_touched.is_empty());
    assert_eq!(r.score, 0);
}

#[test]
fn risk_with_hotspots() {
    let stats = vec![FileStat {
        path: "hot.rs".into(),
        insertions: 200,
        deletions: 200,
    }];
    let contracts = detect_contracts::<&str>(&[]);
    let health = compute_code_health(&stats, &contracts);
    let r = compute_risk(&stats, &contracts, &health);
    assert!(!r.hotspots_touched.is_empty());
    assert!(r.score > 0);
}

// ── compute_metric_trend edge cases ──────────────────────────────────

#[test]
fn trend_stable_when_no_change() {
    let t = compute_metric_trend(50.0, 50.0, true);
    assert_eq!(t.direction, TrendDirection::Stable);
    assert_eq!(t.delta, 0.0);
}

#[test]
fn trend_improving_when_higher_is_better_and_increases() {
    let t = compute_metric_trend(80.0, 60.0, true);
    assert_eq!(t.direction, TrendDirection::Improving);
    assert!(t.delta > 0.0);
}

#[test]
fn trend_degrading_when_higher_is_better_and_decreases() {
    let t = compute_metric_trend(40.0, 60.0, true);
    assert_eq!(t.direction, TrendDirection::Degrading);
    assert!(t.delta < 0.0);
}

#[test]
fn trend_improving_when_lower_is_better_and_decreases() {
    let t = compute_metric_trend(20.0, 40.0, false);
    assert_eq!(t.direction, TrendDirection::Improving);
}

#[test]
fn trend_from_zero_previous() {
    let t = compute_metric_trend(10.0, 0.0, true);
    assert_eq!(t.delta_pct, round_pct(100.0));
}

#[test]
fn trend_both_zero() {
    let t = compute_metric_trend(0.0, 0.0, true);
    assert_eq!(t.direction, TrendDirection::Stable);
    assert_eq!(t.delta_pct, 0.0);
}

// ── generate_review_plan edge cases ──────────────────────────────────

#[test]
fn review_plan_empty_stats() {
    let contracts = detect_contracts::<&str>(&[]);
    let plan = generate_review_plan(&[], &contracts);
    assert!(plan.is_empty());
}

#[test]
fn review_plan_sorted_by_priority() {
    let stats = vec![
        FileStat {
            path: "small.rs".into(),
            insertions: 10,
            deletions: 5,
        },
        FileStat {
            path: "huge.rs".into(),
            insertions: 300,
            deletions: 100,
        },
        FileStat {
            path: "medium.rs".into(),
            insertions: 60,
            deletions: 20,
        },
    ];
    let contracts = detect_contracts::<&str>(&[]);
    let plan = generate_review_plan(&stats, &contracts);
    assert_eq!(plan.len(), 3);
    // Should be sorted by priority (lower number = higher priority)
    assert!(plan[0].priority <= plan[1].priority);
    assert!(plan[1].priority <= plan[2].priority);
}

// ── Utility function edge cases ──────────────────────────────────────

#[test]
fn round_pct_basic() {
    assert_eq!(round_pct(0.7777), 0.78);
    assert_eq!(round_pct(1.0), 1.0);
    assert_eq!(round_pct(0.0), 0.0);
}

#[test]
fn format_signed_f64_positive() {
    assert_eq!(format_signed_f64(3.15), "+3.15");
}

#[test]
fn format_signed_f64_negative() {
    assert_eq!(format_signed_f64(-2.5), "-2.50");
}

#[test]
fn format_signed_f64_zero() {
    assert_eq!(format_signed_f64(0.0), "0.00");
}

#[test]
fn trend_direction_labels() {
    assert_eq!(
        trend_direction_label(TrendDirection::Improving),
        "improving"
    );
    assert_eq!(trend_direction_label(TrendDirection::Stable), "stable");
    assert_eq!(
        trend_direction_label(TrendDirection::Degrading),
        "degrading"
    );
}

#[test]
fn sparkline_empty() {
    assert_eq!(sparkline(&[]), "");
}

#[test]
fn sparkline_single_value() {
    let s = sparkline(&[5.0]);
    assert_eq!(s.chars().count(), 1);
}

#[test]
fn sparkline_constant_values() {
    let s = sparkline(&[3.0, 3.0, 3.0]);
    // All same value → all same bar character
    let chars: Vec<char> = s.chars().collect();
    assert_eq!(chars.len(), 3);
    assert_eq!(chars[0], chars[1]);
    assert_eq!(chars[1], chars[2]);
}

// ── Determinism hash edge cases ──────────────────────────────────────

#[test]
fn hash_empty_file_list() {
    let dir = tempfile::tempdir().unwrap();
    let h = hash_files_from_paths(dir.path(), &[]).unwrap();
    assert_eq!(h.len(), 64, "BLAKE3 hex digest is 64 chars");
}

#[test]
fn hash_missing_file_skipped() {
    let dir = tempfile::tempdir().unwrap();
    // File doesn't exist → skipped (NotFound)
    let h = hash_files_from_paths(dir.path(), &["nonexistent.rs"]).unwrap();
    assert_eq!(h.len(), 64);
}

#[test]
fn hash_cargo_lock_absent() {
    let dir = tempfile::tempdir().unwrap();
    let result = hash_cargo_lock(dir.path()).unwrap();
    assert!(result.is_none());
}

#[test]
fn hash_cargo_lock_present() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("Cargo.lock"),
        "[[package]]\nname = \"test\"\n",
    )
    .unwrap();
    let result = hash_cargo_lock(dir.path()).unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().len(), 64);
}

#[test]
fn hash_deduplicates_paths() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.rs"), "fn main() {}").unwrap();
    let h1 = hash_files_from_paths(dir.path(), &["a.rs"]).unwrap();
    let h2 = hash_files_from_paths(dir.path(), &["a.rs", "a.rs"]).unwrap();
    assert_eq!(h1, h2, "duplicate paths should be deduplicated");
}

#[test]
fn hash_excludes_tokmd_directory() {
    let dir = tempfile::tempdir().unwrap();
    // .tokmd/ paths should be auto-excluded
    let h = hash_files_from_paths(dir.path(), &[".tokmd/baseline.json"]).unwrap();
    let h_empty = hash_files_from_paths(dir.path(), &[]).unwrap();
    assert_eq!(h, h_empty, ".tokmd/ files should be excluded from hash");
}

#[test]
fn hash_excludes_target_directory() {
    let dir = tempfile::tempdir().unwrap();
    let h = hash_files_from_paths(dir.path(), &["target/debug/build"]).unwrap();
    let h_empty = hash_files_from_paths(dir.path(), &[]).unwrap();
    assert_eq!(h, h_empty, "target/ files should be excluded from hash");
}

#[test]
fn hash_excludes_git_directory() {
    let dir = tempfile::tempdir().unwrap();
    let h = hash_files_from_paths(dir.path(), &[".git/HEAD"]).unwrap();
    let h_empty = hash_files_from_paths(dir.path(), &[]).unwrap();
    assert_eq!(h, h_empty, ".git/ files should be excluded from hash");
}
