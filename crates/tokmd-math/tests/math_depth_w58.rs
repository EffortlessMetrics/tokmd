//! Depth tests for tokmd-math (w58).
//!
//! Covers gini coefficient distributions, median/percentile edge cases,
//! deterministic rounding, and property-based invariants.

use tokmd_math::{gini_coefficient, percentile, round_f64, safe_ratio};

// ── gini_coefficient ────────────────────────────────────────────────

#[test]
fn gini_uniform_distribution_is_zero() {
    assert!((gini_coefficient(&[10, 10, 10, 10, 10]) - 0.0).abs() < 1e-10);
}

#[test]
fn gini_perfectly_unequal_two_elements() {
    // [0, N] is maximally unequal for 2 items → gini = 0.5
    let g = gini_coefficient(&[0, 100]);
    assert!((g - 0.5).abs() < 1e-10);
}

#[test]
fn gini_heavily_skewed() {
    let g = gini_coefficient(&[0, 0, 0, 0, 1000]);
    assert!(g > 0.7, "heavily skewed should have high gini, got {g}");
}

#[test]
fn gini_single_item_is_zero() {
    assert_eq!(gini_coefficient(&[999]), 0.0);
}

#[test]
fn gini_empty_is_zero() {
    assert_eq!(gini_coefficient(&[]), 0.0);
}

#[test]
fn gini_all_zeros_is_zero() {
    assert_eq!(gini_coefficient(&[0, 0, 0, 0]), 0.0);
}

#[test]
fn gini_two_equal_nonzero() {
    assert!((gini_coefficient(&[50, 50]) - 0.0).abs() < 1e-10);
}

#[test]
fn gini_monotonic_increase() {
    // Adding more inequality should increase or maintain gini
    let g1 = gini_coefficient(&[1, 2, 3]);
    let g2 = gini_coefficient(&[1, 2, 100]);
    assert!(g2 > g1, "more skew should increase gini");
}

#[test]
fn gini_large_uniform_is_zero() {
    let data: Vec<usize> = vec![42; 1000];
    assert!((gini_coefficient(&data) - 0.0).abs() < 1e-10);
}

// ── percentile / median ─────────────────────────────────────────────

#[test]
fn percentile_empty_returns_zero() {
    assert_eq!(percentile(&[], 0.5), 0.0);
}

#[test]
fn percentile_single_element() {
    assert_eq!(percentile(&[77], 0.0), 77.0);
    assert_eq!(percentile(&[77], 0.5), 77.0);
    assert_eq!(percentile(&[77], 1.0), 77.0);
}

#[test]
fn median_odd_length() {
    assert_eq!(percentile(&[1, 2, 3, 4, 5], 0.5), 3.0);
}

#[test]
fn median_even_length() {
    // With ceil-based indexing, the median of 4 elements picks index ceil(1.5)=2
    let result = percentile(&[10, 20, 30, 40], 0.5);
    assert!(
        result == 20.0 || result == 30.0,
        "median should be 20 or 30, got {result}"
    );
}

#[test]
fn percentile_p0_is_minimum() {
    assert_eq!(percentile(&[5, 10, 15, 20], 0.0), 5.0);
}

#[test]
fn percentile_p100_is_maximum() {
    assert_eq!(percentile(&[5, 10, 15, 20], 1.0), 20.0);
}

#[test]
fn percentile_p90_near_top() {
    let result = percentile(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10], 0.9);
    assert!(result >= 9.0, "p90 should be near top, got {result}");
}

#[test]
fn percentile_p10_near_bottom() {
    let result = percentile(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10], 0.1);
    assert!(result <= 2.0, "p10 should be near bottom, got {result}");
}

#[test]
fn percentile_two_elements() {
    assert_eq!(percentile(&[0, 100], 0.0), 0.0);
    assert_eq!(percentile(&[0, 100], 1.0), 100.0);
}

// ── round_f64 ───────────────────────────────────────────────────────

#[test]
fn round_zero_decimals() {
    assert_eq!(round_f64(2.4, 0), 2.0);
    assert_eq!(round_f64(2.5, 0), 3.0);
    assert_eq!(round_f64(2.6, 0), 3.0);
}

#[test]
fn round_many_decimals() {
    assert_eq!(round_f64(1.123_456_789, 6), 1.123_457);
}

#[test]
fn round_negative_values() {
    assert_eq!(round_f64(-1.5, 0), -2.0);
    assert_eq!(round_f64(-1.234, 2), -1.23);
}

#[test]
fn round_zero_value() {
    assert_eq!(round_f64(0.0, 5), 0.0);
}

#[test]
fn round_idempotent_for_integers() {
    assert_eq!(round_f64(42.0, 0), 42.0);
    assert_eq!(round_f64(42.0, 3), 42.0);
}

// ── safe_ratio ──────────────────────────────────────────────────────

#[test]
fn safe_ratio_zero_denominator() {
    assert_eq!(safe_ratio(100, 0), 0.0);
}

#[test]
fn safe_ratio_zero_numerator() {
    assert_eq!(safe_ratio(0, 100), 0.0);
}

#[test]
fn safe_ratio_equal_values() {
    assert_eq!(safe_ratio(7, 7), 1.0);
}

#[test]
fn safe_ratio_rounds_to_4_decimals() {
    // 1/3 = 0.3333...
    assert_eq!(safe_ratio(1, 3), 0.3333);
    // 2/3 = 0.6666...
    assert_eq!(safe_ratio(2, 3), 0.6667);
}

#[test]
fn safe_ratio_large_values() {
    let r = safe_ratio(1_000_000, 3_000_000);
    assert!((r - 0.3333).abs() < 0.001);
}

// ── property-based tests ────────────────────────────────────────────

mod proptests {
    use proptest::prelude::*;
    use tokmd_math::{gini_coefficient, percentile, round_f64, safe_ratio};

    proptest! {
        #[test]
        fn gini_always_in_0_1(values in proptest::collection::vec(0_usize..1000, 1..50)) {
            let mut sorted = values;
            sorted.sort();
            let g = gini_coefficient(&sorted);
            prop_assert!(g >= 0.0, "gini must be >= 0, got {}", g);
            prop_assert!(g <= 1.0, "gini must be <= 1, got {}", g);
        }

        #[test]
        fn gini_uniform_is_zero(val in 1_usize..1000, len in 2_usize..20) {
            let data = vec![val; len];
            let g = gini_coefficient(&data);
            prop_assert!((g - 0.0).abs() < 1e-10, "uniform gini should be 0, got {}", g);
        }

        #[test]
        fn percentile_within_range(values in proptest::collection::vec(0_usize..10000, 1..100), pct in 0.0_f64..=1.0) {
            let mut sorted = values;
            sorted.sort();
            let p = percentile(&sorted, pct);
            let min = *sorted.first().unwrap() as f64;
            let max = *sorted.last().unwrap() as f64;
            prop_assert!(p >= min, "percentile {} < min {}", p, min);
            prop_assert!(p <= max, "percentile {} > max {}", p, max);
        }

        #[test]
        fn round_idempotent(value in -1000.0_f64..1000.0, decimals in 0_u32..8) {
            let once = round_f64(value, decimals);
            let twice = round_f64(once, decimals);
            prop_assert!((once - twice).abs() < 1e-12, "rounding not idempotent: {} vs {}", once, twice);
        }

        #[test]
        fn safe_ratio_bounded(n in 0_usize..10000, d in 1_usize..10000) {
            let r = safe_ratio(n, d);
            prop_assert!(r >= 0.0);
            if n <= d {
                prop_assert!(r <= 1.0001, "ratio {} should be <= 1 for n<=d", r);
            }
        }
    }
}
