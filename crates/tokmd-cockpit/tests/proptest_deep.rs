//! Deep property-based tests for tokmd-cockpit.
//!
//! Covers: composition invariants, health/risk score bounds,
//! sparkline character validity, trend symmetry, and format helpers.

use proptest::prelude::*;
use tokmd_cockpit::*;

// =========================================================================
// Strategies
// =========================================================================

fn file_ext_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(".rs".to_string()),
        Just(".js".to_string()),
        Just(".ts".to_string()),
        Just(".py".to_string()),
        Just(".md".to_string()),
        Just(".toml".to_string()),
        Just(".json".to_string()),
    ]
}

fn file_path_strategy() -> impl Strategy<Value = String> {
    (
        prop_oneof![
            Just("src/".to_string()),
            Just("tests/".to_string()),
            Just("docs/".to_string()),
            Just("".to_string()),
        ],
        "[a-z]{1,10}",
        file_ext_strategy(),
    )
        .prop_map(|(dir, name, ext)| format!("{}{}{}", dir, name, ext))
}

fn file_stat_strategy() -> impl Strategy<Value = FileStat> {
    (file_path_strategy(), 0..5000usize, 0..5000usize).prop_map(|(path, ins, del)| FileStat {
        path,
        insertions: ins,
        deletions: del,
    })
}

// =========================================================================
// Composition: all percentages bounded and deterministic
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn composition_is_deterministic(
        paths in prop::collection::vec(file_path_strategy(), 0..30),
    ) {
        let refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
        let c1 = compute_composition(&refs);
        let c2 = compute_composition(&refs);
        prop_assert!((c1.code_pct - c2.code_pct).abs() < f64::EPSILON);
        prop_assert!((c1.test_pct - c2.test_pct).abs() < f64::EPSILON);
        prop_assert!((c1.docs_pct - c2.docs_pct).abs() < f64::EPSILON);
        prop_assert!((c1.config_pct - c2.config_pct).abs() < f64::EPSILON);
    }

    #[test]
    fn composition_individual_pct_bounded(
        paths in prop::collection::vec(file_path_strategy(), 1..30),
    ) {
        let refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
        let comp = compute_composition(&refs);
        prop_assert!(comp.code_pct >= 0.0 && comp.code_pct <= 1.0);
        prop_assert!(comp.test_pct >= 0.0 && comp.test_pct <= 1.0);
        prop_assert!(comp.docs_pct >= 0.0 && comp.docs_pct <= 1.0);
        prop_assert!(comp.config_pct >= 0.0 && comp.config_pct <= 1.0);
        prop_assert!(comp.test_ratio >= 0.0);
    }

    #[test]
    fn composition_empty_input(paths in Just(vec![] as Vec<String>)) {
        let refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
        let comp = compute_composition(&refs);
        prop_assert!((comp.code_pct - 0.0).abs() < f64::EPSILON);
        prop_assert!((comp.test_pct - 0.0).abs() < f64::EPSILON);
    }
}

// =========================================================================
// Health: score monotonically worsens with more breaking indicators
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn health_worsens_with_more_breaking(
        stats in prop::collection::vec(file_stat_strategy(), 0..10),
        breaking_small in 0usize..2,
    ) {
        let breaking_large = breaking_small + 2;
        let contracts_small = Contracts {
            api_changed: breaking_small > 0,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: breaking_small,
        };
        let contracts_large = Contracts {
            api_changed: true,
            cli_changed: true,
            schema_changed: breaking_large > 2,
            breaking_indicators: breaking_large,
        };
        let health_small = compute_code_health(&stats, &contracts_small);
        let health_large = compute_code_health(&stats, &contracts_large);
        prop_assert!(
            health_large.score <= health_small.score,
            "More breaking ({}) should give <= score than fewer ({}): {} vs {}",
            breaking_large, breaking_small, health_large.score, health_small.score
        );
    }
}

// =========================================================================
// Sparkline: only valid block characters
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn sparkline_chars_are_block_elements(
        values in prop::collection::vec(0.0f64..100.0, 1..20),
    ) {
        let result = sparkline(&values);
        let valid_chars: Vec<char> = ('\u{2581}'..='\u{2588}').collect();
        for ch in result.chars() {
            prop_assert!(
                valid_chars.contains(&ch),
                "Sparkline char '{}' (U+{:04X}) not a block element", ch, ch as u32
            );
        }
    }

    #[test]
    fn sparkline_constant_values_all_same_char(
        value in 0.0f64..100.0,
        len in 1usize..20,
    ) {
        let values: Vec<f64> = vec![value; len];
        let result = sparkline(&values);
        let chars: Vec<char> = result.chars().collect();
        if !chars.is_empty() {
            let first = chars[0];
            prop_assert!(chars.iter().all(|&c| c == first),
                "Constant values should produce identical chars");
        }
    }

    #[test]
    fn sparkline_empty_input_empty_output(values in Just(vec![] as Vec<f64>)) {
        let result = sparkline(&values);
        prop_assert!(result.is_empty());
    }
}

// =========================================================================
// Trend: symmetric reversal
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn trend_delta_negation(current in -500.0f64..500.0, previous in -500.0f64..500.0) {
        let t1 = compute_metric_trend(current, previous, true);
        let t2 = compute_metric_trend(previous, current, true);
        prop_assert!(
            (t1.delta + t2.delta).abs() < 0.001,
            "Reversed trend deltas should negate: {} + {} != 0",
            t1.delta, t2.delta
        );
    }

    #[test]
    fn trend_direction_reversal(
        current in 0.0f64..100.0,
        previous in 0.0f64..100.0,
    ) {
        prop_assume!((current - previous).abs() >= 1.0);
        let t_fwd = compute_metric_trend(current, previous, true);
        let t_rev = compute_metric_trend(previous, current, true);
        // If forward is improving, reverse should be degrading
        if t_fwd.direction == TrendDirection::Improving {
            prop_assert_eq!(t_rev.direction, TrendDirection::Degrading);
        } else if t_fwd.direction == TrendDirection::Degrading {
            prop_assert_eq!(t_rev.direction, TrendDirection::Improving);
        }
    }
}

// =========================================================================
// Format helpers
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn format_signed_zero_has_plus(val in Just(0.0f64)) {
        let formatted = format_signed_f64(val);
        prop_assert!(formatted.starts_with('+') || formatted.starts_with('0'),
            "Zero should format as +0.0 or similar: {}", formatted);
    }

    #[test]
    fn round_pct_bounded(val in -1.0f64..1.0) {
        let rounded = round_pct(val);
        // round_pct multiplies by 100, rounds, divides by 100
        // Result should be within ±0.01 of input (rounding error)
        prop_assert!(
            (rounded - val).abs() < 0.01,
            "round_pct({}) = {} too far from input", val, rounded
        );
    }

    #[test]
    fn round_pct_preserves_sign(val in -100.0f64..100.0) {
        let rounded = round_pct(val);
        if val > 0.005 {
            prop_assert!(rounded >= 0.0, "Positive value {} rounded to negative {}", val, rounded);
        } else if val < -0.005 {
            prop_assert!(rounded <= 0.0, "Negative value {} rounded to positive {}", val, rounded);
        }
    }
}

// =========================================================================
// detect_contracts: determinism and consistency
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn detect_contracts_deterministic(
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

    #[test]
    fn detect_contracts_breaking_count_bounded(
        paths in prop::collection::vec(file_path_strategy(), 0..20),
    ) {
        let refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
        let contracts = detect_contracts(&refs);
        // breaking_indicators should be sum of booleans (0-3)
        let expected_max = (contracts.api_changed as usize)
            + (contracts.cli_changed as usize)
            + (contracts.schema_changed as usize);
        prop_assert!(
            contracts.breaking_indicators <= expected_max,
            "breaking_indicators {} > expected max {}",
            contracts.breaking_indicators, expected_max
        );
    }
}
