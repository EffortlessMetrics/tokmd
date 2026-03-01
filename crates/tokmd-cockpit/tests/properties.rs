//! Property-based tests for cockpit functions using `proptest`.

use proptest::prelude::*;
use tokmd_cockpit::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

#[allow(dead_code)]
fn make_file_stat(path: &str, insertions: usize, deletions: usize) -> FileStat {
    FileStat {
        path: path.to_string(),
        insertions,
        deletions,
    }
}

/// Strategy for generating a valid file extension.
fn file_ext_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(".rs".to_string()),
        Just(".js".to_string()),
        Just(".ts".to_string()),
        Just(".py".to_string()),
        Just(".md".to_string()),
        Just(".toml".to_string()),
        Just(".json".to_string()),
        Just(".yml".to_string()),
        Just(".yaml".to_string()),
        Just(".txt".to_string()),
    ]
}

/// Strategy for generating a file path.
fn file_path_strategy() -> impl Strategy<Value = String> {
    (
        prop_oneof![
            Just("src/".to_string()),
            Just("tests/".to_string()),
            Just("docs/".to_string()),
            Just("crates/tokmd/src/commands/".to_string()),
            Just("crates/tokmd-config/".to_string()),
            Just("".to_string()),
        ],
        "[a-z]{1,10}",
        file_ext_strategy(),
    )
        .prop_map(|(dir, name, ext)| format!("{}{}{}", dir, name, ext))
}

/// Strategy for generating FileStat values.
fn file_stat_strategy() -> impl Strategy<Value = FileStat> {
    (file_path_strategy(), 0..5000usize, 0..5000usize).prop_map(|(path, ins, del)| FileStat {
        path,
        insertions: ins,
        deletions: del,
    })
}

// ===========================================================================
// Property: Composition percentages always sum to <= 1.0
// ===========================================================================

proptest! {
    #[test]
    fn prop_composition_percentages_sum_to_one_or_less(
        paths in prop::collection::vec(file_path_strategy(), 0..50)
    ) {
        let refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
        let comp = compute_composition(&refs);

        let sum = comp.code_pct + comp.test_pct + comp.docs_pct + comp.config_pct;
        // Sum should be <= 1.0 (some files may be uncategorized)
        prop_assert!(sum <= 1.001, "sum was {}", sum);
        // Each percentage should be [0, 1]
        prop_assert!(comp.code_pct >= 0.0 && comp.code_pct <= 1.0);
        prop_assert!(comp.test_pct >= 0.0 && comp.test_pct <= 1.0);
        prop_assert!(comp.docs_pct >= 0.0 && comp.docs_pct <= 1.0);
        prop_assert!(comp.config_pct >= 0.0 && comp.config_pct <= 1.0);
    }
}

// ===========================================================================
// Property: Composition test_ratio is non-negative
// ===========================================================================

proptest! {
    #[test]
    fn prop_test_ratio_non_negative(
        paths in prop::collection::vec(file_path_strategy(), 0..50)
    ) {
        let refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
        let comp = compute_composition(&refs);

        prop_assert!(comp.test_ratio >= 0.0, "test_ratio was {}", comp.test_ratio);
    }
}

// ===========================================================================
// Property: Code health score is bounded [0, 100]
// ===========================================================================

proptest! {
    #[test]
    fn prop_health_score_bounded(
        stats in prop::collection::vec(file_stat_strategy(), 0..20),
        breaking in 0..5usize,
    ) {
        let contracts = Contracts {
            api_changed: breaking > 0,
            cli_changed: false,
            schema_changed: breaking > 1,
            breaking_indicators: breaking,
        };
        let health = compute_code_health(&stats, &contracts);

        prop_assert!(health.score <= 100, "score was {}", health.score);
        prop_assert!(
            matches!(
                health.grade.as_str(),
                "A" | "B" | "C" | "D" | "F"
            ),
            "unexpected grade: {}",
            health.grade
        );
    }
}

// ===========================================================================
// Property: Risk score is bounded [0, 100]
// ===========================================================================

proptest! {
    #[test]
    fn prop_risk_score_bounded(
        stats in prop::collection::vec(file_stat_strategy(), 0..20),
    ) {
        let contracts = Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        };
        let health = compute_code_health(&stats, &contracts);
        let risk = compute_risk(&stats, &contracts, &health);

        prop_assert!(risk.score <= 100, "risk score was {}", risk.score);
    }
}

// ===========================================================================
// Property: Review plan items match file count
// ===========================================================================

proptest! {
    #[test]
    fn prop_review_plan_has_entry_per_file(
        stats in prop::collection::vec(file_stat_strategy(), 0..20),
    ) {
        let contracts = Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        };
        let plan = generate_review_plan(&stats, &contracts);

        prop_assert_eq!(plan.len(), stats.len());
    }
}

// ===========================================================================
// Property: Review plan priorities are valid (1, 2, or 3)
// ===========================================================================

proptest! {
    #[test]
    fn prop_review_plan_priorities_valid(
        stats in prop::collection::vec(file_stat_strategy(), 1..20),
    ) {
        let contracts = Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        };
        let plan = generate_review_plan(&stats, &contracts);

        for item in &plan {
            prop_assert!(
                item.priority >= 1 && item.priority <= 3,
                "invalid priority {} for {}",
                item.priority,
                item.path
            );
        }
    }
}

// ===========================================================================
// Property: Review plan is sorted by priority
// ===========================================================================

proptest! {
    #[test]
    fn prop_review_plan_sorted_by_priority(
        stats in prop::collection::vec(file_stat_strategy(), 0..20),
    ) {
        let contracts = Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        };
        let plan = generate_review_plan(&stats, &contracts);

        for window in plan.windows(2) {
            prop_assert!(
                window[0].priority <= window[1].priority,
                "review plan not sorted: {} > {}",
                window[0].priority,
                window[1].priority
            );
        }
    }
}

// ===========================================================================
// Property: Trend metric delta is always current - previous
// ===========================================================================

proptest! {
    #[test]
    fn prop_trend_delta_is_current_minus_previous(
        current in -1000.0f64..1000.0,
        previous in -1000.0f64..1000.0,
        higher_is_better in proptest::bool::ANY,
    ) {
        let trend = compute_metric_trend(current, previous, higher_is_better);

        let expected_delta = current - previous;
        prop_assert!(
            (trend.delta - expected_delta).abs() < 0.001,
            "delta {} != expected {}",
            trend.delta,
            expected_delta
        );
    }
}

// ===========================================================================
// Property: Trend direction consistency with higher_is_better=true
// ===========================================================================

proptest! {
    #[test]
    fn prop_trend_direction_consistent_higher_better(
        current in 0.0f64..100.0,
        previous in 0.0f64..100.0,
    ) {
        let trend = compute_metric_trend(current, previous, true);

        if (current - previous).abs() < 1.0 {
            prop_assert_eq!(trend.direction, TrendDirection::Stable);
        } else if current > previous {
            prop_assert_eq!(trend.direction, TrendDirection::Improving);
        } else {
            prop_assert_eq!(trend.direction, TrendDirection::Degrading);
        }
    }
}

// ===========================================================================
// Property: Trend direction consistency with higher_is_better=false
// ===========================================================================

proptest! {
    #[test]
    fn prop_trend_direction_consistent_lower_better(
        current in 0.0f64..100.0,
        previous in 0.0f64..100.0,
    ) {
        let trend = compute_metric_trend(current, previous, false);

        if (current - previous).abs() < 1.0 {
            prop_assert_eq!(trend.direction, TrendDirection::Stable);
        } else if current < previous {
            prop_assert_eq!(trend.direction, TrendDirection::Improving);
        } else {
            prop_assert_eq!(trend.direction, TrendDirection::Degrading);
        }
    }
}

// ===========================================================================
// Property: Sparkline length matches input length
// ===========================================================================

proptest! {
    #[test]
    fn prop_sparkline_length_matches_input(
        values in prop::collection::vec(0.0f64..100.0, 0..20),
    ) {
        let result = sparkline(&values);
        prop_assert_eq!(result.chars().count(), values.len());
    }
}

// ===========================================================================
// Property: round_pct is idempotent
// ===========================================================================

proptest! {
    #[test]
    fn prop_round_pct_idempotent(val in -100.0f64..100.0) {
        let once = round_pct(val);
        let twice = round_pct(once);
        prop_assert!((once - twice).abs() < f64::EPSILON,
            "round_pct not idempotent: {} -> {} -> {}", val, once, twice);
    }
}

// ===========================================================================
// Property: format_signed_f64 starts with + for positive values
// ===========================================================================

proptest! {
    #[test]
    fn prop_format_signed_positive_has_plus(val in 0.01f64..1000.0) {
        let formatted = format_signed_f64(val);
        prop_assert!(formatted.starts_with('+'), "expected + prefix: {}", formatted);
    }

    #[test]
    fn prop_format_signed_negative_has_minus(val in -1000.0f64..-0.01) {
        let formatted = format_signed_f64(val);
        prop_assert!(formatted.starts_with('-'), "expected - prefix: {}", formatted);
    }
}

// ===========================================================================
// Property: Determinism hash is always 64 hex characters
// ===========================================================================

proptest! {
    #[test]
    fn prop_hash_is_64_hex(
        content_a in "[a-z]{0,100}",
        content_b in "[a-z]{0,100}",
    ) {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), &content_a).unwrap();
        std::fs::write(dir.path().join("b.rs"), &content_b).unwrap();

        let hash = tokmd_cockpit::determinism::hash_files_from_paths(
            dir.path(),
            &["a.rs", "b.rs"],
        ).unwrap();

        prop_assert_eq!(hash.len(), 64);
        prop_assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

// ===========================================================================
// Property: Determinism hash is order-independent
// ===========================================================================

proptest! {
    #[test]
    fn prop_hash_order_independent(
        content_a in "[a-z]{1,50}",
        content_b in "[a-z]{1,50}",
    ) {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), &content_a).unwrap();
        std::fs::write(dir.path().join("b.rs"), &content_b).unwrap();

        let h1 = tokmd_cockpit::determinism::hash_files_from_paths(
            dir.path(), &["a.rs", "b.rs"],
        ).unwrap();
        let h2 = tokmd_cockpit::determinism::hash_files_from_paths(
            dir.path(), &["b.rs", "a.rs"],
        ).unwrap();

        prop_assert_eq!(h1, h2);
    }
}

// ===========================================================================
// Property: Contracts detection is pure (same input -> same output)
// ===========================================================================

proptest! {
    #[test]
    fn prop_contracts_detection_pure(
        paths in prop::collection::vec(file_path_strategy(), 0..20),
    ) {
        let refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
        let c1 = detect_contracts(&refs);
        let c2 = detect_contracts(&refs);

        prop_assert_eq!(c1.api_changed, c2.api_changed);
        prop_assert_eq!(c1.cli_changed, c2.cli_changed);
        prop_assert_eq!(c1.schema_changed, c2.schema_changed);
        prop_assert_eq!(c1.breaking_indicators, c2.breaking_indicators);
    }
}

// ===========================================================================
// Property: Empty file stats produce zero-risk and full-health
// ===========================================================================

proptest! {
    #[test]
    fn prop_empty_stats_baseline(breaking in 0..3usize) {
        let stats: Vec<FileStat> = Vec::new();
        let contracts = Contracts {
            api_changed: breaking > 0,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: breaking,
        };
        let health = compute_code_health(&stats, &contracts);
        let risk = compute_risk(&stats, &contracts, &health);

        prop_assert_eq!(health.large_files_touched, 0);
        prop_assert!(health.warnings.is_empty());
        prop_assert!(risk.hotspots_touched.is_empty());
    }
}
