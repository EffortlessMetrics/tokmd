//! Property-based tests for tokmd-cockpit (W50 expansion).
//!
//! Verifies cockpit computation functions with arbitrary inputs:
//! composition, contracts, code health, risk, review plans, and trends.

use proptest::prelude::*;
use tokmd_cockpit::{
    FileStat, TrendDirection, compute_code_health, compute_composition, compute_metric_trend,
    compute_risk, detect_contracts, generate_review_plan,
};

// ── Strategies ───────────────────────────────────────────────────────────────

fn arb_file_stat() -> impl Strategy<Value = FileStat> {
    ("[a-z_/]{1,40}\\.[a-z]{1,4}", 0usize..2000, 0usize..2000).prop_map(
        |(path, insertions, deletions)| FileStat {
            path,
            insertions,
            deletions,
        },
    )
}

fn arb_file_path() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("src/main.rs".to_string()),
        Just("src/lib.rs".to_string()),
        Just("tests/test_foo.rs".to_string()),
        Just("README.md".to_string()),
        Just("Cargo.toml".to_string()),
        Just("src/commands/gate.rs".to_string()),
        Just("docs/schema.json".to_string()),
        Just("crates/tokmd-config/src/lib.rs".to_string()),
        "[a-z_/]{1,30}\\.[a-z]{1,4}",
    ]
}

// ── Composition tests ────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn composition_never_panics(files in prop::collection::vec(arb_file_path(), 0..50)) {
        let _ = compute_composition(&files);
    }

    #[test]
    fn composition_percentages_sum_to_one_or_zero(
        files in prop::collection::vec(arb_file_path(), 0..50)
    ) {
        let comp = compute_composition(&files);
        let sum = comp.code_pct + comp.test_pct + comp.docs_pct + comp.config_pct;
        // Sum should be ~1.0 or 0.0 if no categorized files
        prop_assert!(
            (sum - 1.0).abs() < 0.001 || sum == 0.0,
            "Percentages sum to {} (expected ~1.0 or 0.0)", sum
        );
    }

    #[test]
    fn composition_empty_is_zero(_dummy in 0u8..1) {
        let comp = compute_composition::<String>(&[]);
        prop_assert_eq!(comp.code_pct, 0.0);
        prop_assert_eq!(comp.test_pct, 0.0);
        prop_assert_eq!(comp.test_ratio, 0.0);
    }

    #[test]
    fn composition_test_ratio_non_negative(
        files in prop::collection::vec(arb_file_path(), 0..50)
    ) {
        let comp = compute_composition(&files);
        prop_assert!(comp.test_ratio >= 0.0, "test_ratio should be non-negative");
    }
}

// ── Contracts tests ──────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn detect_contracts_never_panics(
        files in prop::collection::vec(arb_file_path(), 0..50)
    ) {
        let _ = detect_contracts(&files);
    }

    #[test]
    fn detect_contracts_breaking_bounded(
        files in prop::collection::vec(arb_file_path(), 0..50)
    ) {
        let contracts = detect_contracts(&files);
        // breaking_indicators is at most api_changed(1) + schema_changed(1)
        prop_assert!(contracts.breaking_indicators <= 2);
    }

    #[test]
    fn lib_rs_triggers_api_changed(prefix in "[a-z]{1,10}") {
        let files = vec![format!("{}/lib.rs", prefix)];
        let contracts = detect_contracts(&files);
        prop_assert!(contracts.api_changed);
    }
}

// ── Code Health tests ────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn code_health_never_panics(
        stats in prop::collection::vec(arb_file_stat(), 0..30)
    ) {
        let contracts = detect_contracts::<String>(&[]);
        let _ = compute_code_health(&stats, &contracts);
    }

    #[test]
    fn code_health_score_bounded(
        stats in prop::collection::vec(arb_file_stat(), 0..30)
    ) {
        let contracts = detect_contracts::<String>(&[]);
        let health = compute_code_health(&stats, &contracts);
        prop_assert!(health.score <= 100, "Health score {} exceeds 100", health.score);
    }

    #[test]
    fn code_health_grade_valid(
        stats in prop::collection::vec(arb_file_stat(), 0..30)
    ) {
        let contracts = detect_contracts::<String>(&[]);
        let health = compute_code_health(&stats, &contracts);
        prop_assert!(
            ["A", "B", "C", "D", "F"].contains(&health.grade.as_str()),
            "Invalid grade: {}", health.grade
        );
    }

    #[test]
    fn empty_stats_perfect_health(_dummy in 0u8..1) {
        let contracts = detect_contracts::<String>(&[]);
        let health = compute_code_health(&[], &contracts);
        prop_assert_eq!(health.score, 100);
        prop_assert_eq!(health.grade, "A");
        prop_assert_eq!(health.large_files_touched, 0);
    }
}

// ── Risk tests ───────────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn risk_never_panics(
        stats in prop::collection::vec(arb_file_stat(), 0..30)
    ) {
        let contracts = detect_contracts::<String>(&[]);
        let health = compute_code_health(&stats, &contracts);
        let _ = compute_risk(stats.clone(), &contracts, &health);
    }

    #[test]
    fn risk_score_bounded(
        stats in prop::collection::vec(arb_file_stat(), 0..30)
    ) {
        let contracts = detect_contracts::<String>(&[]);
        let health = compute_code_health(&stats, &contracts);
        let risk = compute_risk(stats.clone(), &contracts, &health);
        prop_assert!(risk.score <= 100, "Risk score {} exceeds 100", risk.score);
    }
}

// ── Review Plan tests ────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn review_plan_never_panics(
        stats in prop::collection::vec(arb_file_stat(), 0..30)
    ) {
        let contracts = detect_contracts::<String>(&[]);
        let _ = generate_review_plan(&stats, &contracts);
    }

    #[test]
    fn review_plan_same_count_as_inputs(
        stats in prop::collection::vec(arb_file_stat(), 0..30)
    ) {
        let contracts = detect_contracts::<String>(&[]);
        let plan = generate_review_plan(&stats, &contracts);
        prop_assert_eq!(plan.len(), stats.len());
    }

    #[test]
    fn review_plan_sorted_by_priority(
        stats in prop::collection::vec(arb_file_stat(), 0..30)
    ) {
        let contracts = detect_contracts::<String>(&[]);
        let plan = generate_review_plan(&stats, &contracts);
        for window in plan.windows(2) {
            prop_assert!(
                window[0].priority <= window[1].priority,
                "Plan not sorted: {} > {}", window[0].priority, window[1].priority
            );
        }
    }
}

// ── Trend computation tests ──────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn metric_trend_never_panics(
        current in -1_000_000.0f64..1_000_000.0,
        previous in -1_000_000.0f64..1_000_000.0,
        higher_is_better in any::<bool>()
    ) {
        let _ = compute_metric_trend(current, previous, higher_is_better);
    }

    #[test]
    fn equal_values_are_stable(value in 0.0f64..1_000.0) {
        let trend = compute_metric_trend(value, value, true);
        prop_assert_eq!(trend.direction, TrendDirection::Stable);
        prop_assert!((trend.delta).abs() < f64::EPSILON);
    }

    #[test]
    fn higher_is_better_direction(
        base in 10.0f64..1_000.0,
        increase in 2.0f64..500.0
    ) {
        let trend = compute_metric_trend(base + increase, base, true);
        prop_assert_eq!(trend.direction, TrendDirection::Improving);

        let trend2 = compute_metric_trend(base - increase, base, true);
        prop_assert_eq!(trend2.direction, TrendDirection::Degrading);
    }

    #[test]
    fn lower_is_better_direction(
        base in 10.0f64..1_000.0,
        decrease in 2.0f64..500.0
    ) {
        let trend = compute_metric_trend(base - decrease, base, false);
        prop_assert_eq!(trend.direction, TrendDirection::Improving);

        let trend2 = compute_metric_trend(base + decrease, base, false);
        prop_assert_eq!(trend2.direction, TrendDirection::Degrading);
    }

    #[test]
    fn trend_delta_is_current_minus_previous(
        current in 0.0f64..1_000.0,
        previous in 0.0f64..1_000.0
    ) {
        let trend = compute_metric_trend(current, previous, true);
        let expected_delta = current - previous;
        prop_assert!(
            (trend.delta - expected_delta).abs() < f64::EPSILON,
            "delta {} != expected {}", trend.delta, expected_delta
        );
    }
}
