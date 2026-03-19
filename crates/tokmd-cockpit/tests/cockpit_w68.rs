//! W68 integration tests for tokmd-cockpit: composition, contracts,
//! health, risk, review plan, trend computation, and evidence helpers.

use tokmd_cockpit::{
    FileStat, TrendDirection, compute_code_health, compute_composition, compute_metric_trend,
    compute_risk, detect_contracts, format_signed_f64, generate_review_plan, round_pct, sparkline,
    trend_direction_label,
};

// ── Helper ────────────────────────────────────────────────────────────────

fn stat(path: &str, ins: usize, del: usize) -> FileStat {
    FileStat {
        path: path.to_string(),
        insertions: ins,
        deletions: del,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. compute_composition
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn composition_empty_files() {
    let comp = compute_composition::<&str>(&[]);
    assert_eq!(comp.code_pct, 0.0);
    assert_eq!(comp.test_pct, 0.0);
    assert_eq!(comp.test_ratio, 0.0);
}

#[test]
fn composition_code_only() {
    let files = vec!["src/lib.rs", "src/main.rs"];
    let comp = compute_composition(&files);
    assert!(comp.code_pct > 0.0);
    assert_eq!(comp.test_pct, 0.0);
}

#[test]
fn composition_with_tests() {
    let files = vec!["src/lib.rs", "tests/integration_test.rs"];
    let comp = compute_composition(&files);
    assert!(comp.code_pct > 0.0);
    assert!(comp.test_pct > 0.0);
    assert!(comp.test_ratio > 0.0);
}

#[test]
fn composition_docs_and_config() {
    let files = vec!["README.md", "docs/guide.md", "Cargo.toml", "config.yml"];
    let comp = compute_composition(&files);
    assert!(comp.docs_pct > 0.0);
    assert!(comp.config_pct > 0.0);
    assert_eq!(comp.code_pct, 0.0);
}

#[test]
fn composition_test_ratio_with_only_tests() {
    let files = vec!["tests/test_a.rs", "tests/test_b.rs"];
    let comp = compute_composition(&files);
    assert_eq!(comp.test_ratio, 1.0);
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. detect_contracts
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn contracts_detects_api_change() {
    let files = vec!["crates/tokmd/src/lib.rs"];
    let contracts = detect_contracts(&files);
    assert!(contracts.api_changed);
    assert_eq!(contracts.breaking_indicators, 1);
}

#[test]
fn contracts_detects_cli_change() {
    let files = vec!["crates/tokmd/src/commands/gate.rs"];
    let contracts = detect_contracts(&files);
    assert!(contracts.cli_changed);
}

#[test]
fn contracts_detects_schema_change() {
    let files = vec!["docs/schema.json"];
    let contracts = detect_contracts(&files);
    assert!(contracts.schema_changed);
    assert_eq!(contracts.breaking_indicators, 1);
}

#[test]
fn contracts_no_changes() {
    let files = vec!["src/utils.rs", "benches/perf.rs"];
    let contracts = detect_contracts(&files);
    assert!(!contracts.api_changed);
    assert!(!contracts.cli_changed);
    assert!(!contracts.schema_changed);
    assert_eq!(contracts.breaking_indicators, 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. compute_code_health
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn health_perfect_for_small_changes() {
    let stats = vec![stat("a.rs", 10, 5)];
    let contracts = detect_contracts::<&str>(&[]);
    let health = compute_code_health(&stats, &contracts);
    assert_eq!(health.score, 100);
    assert_eq!(health.grade, "A");
    assert_eq!(health.large_files_touched, 0);
}

#[test]
fn health_degrades_with_large_files() {
    let stats = vec![
        stat("a.rs", 300, 300),
        stat("b.rs", 400, 200),
        stat("c.rs", 500, 100),
    ];
    let contracts = detect_contracts::<&str>(&[]);
    let health = compute_code_health(&stats, &contracts);
    assert!(health.large_files_touched >= 2);
    assert!(health.score < 100);
}

#[test]
fn health_penalizes_breaking_contracts() {
    let stats = vec![stat("lib.rs", 10, 5)];
    let files = vec!["crates/tokmd/src/lib.rs"];
    let contracts = detect_contracts(&files);
    let health = compute_code_health(&stats, &contracts);
    assert!(health.score < 100);
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. compute_risk
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn risk_low_for_small_changes() {
    let stats = vec![stat("a.rs", 10, 5)];
    let contracts = detect_contracts::<&str>(&[]);
    let health = compute_code_health(&stats, &contracts);
    let risk = compute_risk(&stats, &contracts, &health);
    assert_eq!(risk.score, 0);
}

#[test]
fn risk_increases_with_hotspots() {
    let stats = vec![stat("hot1.rs", 200, 200), stat("hot2.rs", 200, 200)];
    let contracts = detect_contracts::<&str>(&[]);
    let health = compute_code_health(&stats, &contracts);
    let risk = compute_risk(&stats, &contracts, &health);
    assert!(risk.score > 0);
    assert!(!risk.hotspots_touched.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. generate_review_plan
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn review_plan_sorted_by_priority() {
    let stats = vec![
        stat("small.rs", 5, 5),
        stat("big.rs", 200, 100),
        stat("medium.rs", 40, 20),
    ];
    let contracts = detect_contracts::<&str>(&[]);
    let plan = generate_review_plan(&stats, &contracts);
    assert_eq!(plan.len(), 3);
    // Sorted by priority (lower number = higher priority)
    assert!(plan[0].priority <= plan[1].priority);
    assert!(plan[1].priority <= plan[2].priority);
}

#[test]
fn review_plan_empty_for_no_files() {
    let contracts = detect_contracts::<&str>(&[]);
    let plan = generate_review_plan(&[], &contracts);
    assert!(plan.is_empty());
}

#[test]
fn review_plan_assigns_complexity() {
    let stats = vec![stat("huge.rs", 300, 100)];
    let contracts = detect_contracts::<&str>(&[]);
    let plan = generate_review_plan(&stats, &contracts);
    assert_eq!(plan.len(), 1);
    assert!(plan[0].complexity.unwrap() >= 3);
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. compute_metric_trend
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn trend_improving_when_higher_is_better() {
    let trend = compute_metric_trend(90.0, 80.0, true);
    assert_eq!(trend.direction, TrendDirection::Improving);
    assert!(trend.delta > 0.0);
}

#[test]
fn trend_degrading_when_lower_is_better() {
    let trend = compute_metric_trend(50.0, 40.0, false);
    assert_eq!(trend.direction, TrendDirection::Degrading);
}

#[test]
fn trend_stable_when_small_delta() {
    let trend = compute_metric_trend(80.0, 80.5, true);
    assert_eq!(trend.direction, TrendDirection::Stable);
}

#[test]
fn trend_zero_previous() {
    let trend = compute_metric_trend(10.0, 0.0, true);
    assert_eq!(trend.delta_pct, 100.0);
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Utility helpers
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn round_pct_two_decimals() {
    assert_eq!(round_pct(0.123456), 0.12);
    assert_eq!(round_pct(0.999), 1.0);
    assert_eq!(round_pct(0.0), 0.0);
}

#[test]
fn format_signed_positive_and_negative() {
    assert_eq!(format_signed_f64(5.0), "+5.00");
    assert_eq!(format_signed_f64(-2.5), "-2.50");
    assert_eq!(format_signed_f64(0.0), "0.00");
}

#[test]
fn trend_labels() {
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
fn sparkline_multiple_values() {
    let s = sparkline(&[1.0, 5.0, 3.0, 8.0]);
    assert_eq!(s.chars().count(), 4);
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Determinism
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn composition_deterministic() {
    let files = vec!["src/lib.rs", "tests/test.rs", "README.md", "Cargo.toml"];
    let c1 = compute_composition(&files);
    let c2 = compute_composition(&files);
    assert_eq!(c1.code_pct, c2.code_pct);
    assert_eq!(c1.test_pct, c2.test_pct);
    assert_eq!(c1.docs_pct, c2.docs_pct);
    assert_eq!(c1.config_pct, c2.config_pct);
    assert_eq!(c1.test_ratio, c2.test_ratio);
}
