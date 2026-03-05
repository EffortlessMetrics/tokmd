//! W62 depth tests for tokmd-math: comprehensive coverage of all statistical helpers.

use tokmd_math::{gini_coefficient, percentile, round_f64, safe_ratio};

// ── round_f64 ──────────────────────────────────────────────────────────────

#[test]
fn round_f64_zero_decimals() {
    assert_eq!(round_f64(2.4, 0), 2.0);
    assert_eq!(round_f64(2.5, 0), 3.0);
    assert_eq!(round_f64(2.6, 0), 3.0);
}

#[test]
fn round_f64_one_decimal() {
    assert_eq!(round_f64(1.05, 1), 1.1);
    assert_eq!(round_f64(1.04, 1), 1.0);
}

#[test]
fn round_f64_many_decimals() {
    assert_eq!(round_f64(1.123456789, 8), 1.12345679);
}

#[test]
fn round_f64_negative_value() {
    assert_eq!(round_f64(-1.5, 0), -2.0);
    assert_eq!(round_f64(-1.555, 2), -1.56);
}

#[test]
fn round_f64_zero() {
    assert_eq!(round_f64(0.0, 5), 0.0);
}

#[test]
fn round_f64_large_value() {
    // Large value should not overflow or lose precision catastrophically.
    let v = round_f64(1e15 + 0.123, 2);
    assert!(v > 1e15 - 1.0);
}

#[test]
fn round_f64_very_small_value() {
    assert_eq!(round_f64(0.000001, 6), 0.000001);
    assert_eq!(round_f64(0.000001, 5), 0.0);
}

#[test]
fn round_f64_infinity() {
    assert!(round_f64(f64::INFINITY, 2).is_infinite());
    assert!(round_f64(f64::NEG_INFINITY, 2).is_infinite());
}

#[test]
fn round_f64_nan() {
    assert!(round_f64(f64::NAN, 2).is_nan());
}

#[test]
fn round_f64_deterministic() {
    for _ in 0..100 {
        assert_eq!(round_f64(1.23456789, 4), 1.2346);
    }
}

// ── safe_ratio ─────────────────────────────────────────────────────────────

#[test]
fn safe_ratio_zero_denominator() {
    assert_eq!(safe_ratio(0, 0), 0.0);
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
fn safe_ratio_common_fractions() {
    assert_eq!(safe_ratio(1, 4), 0.25);
    assert_eq!(safe_ratio(1, 2), 0.5);
    assert_eq!(safe_ratio(3, 4), 0.75);
}

#[test]
fn safe_ratio_repeating_decimal() {
    assert_eq!(safe_ratio(1, 3), 0.3333);
    assert_eq!(safe_ratio(2, 3), 0.6667);
}

#[test]
fn safe_ratio_greater_than_one() {
    assert_eq!(safe_ratio(10, 3), 3.3333);
}

#[test]
fn safe_ratio_large_values() {
    // Should not panic or overflow with large usize values.
    let r = safe_ratio(1_000_000, 3_000_000);
    assert!((r - 0.3333).abs() < 0.001);
}

#[test]
fn safe_ratio_deterministic() {
    for _ in 0..100 {
        assert_eq!(safe_ratio(7, 11), safe_ratio(7, 11));
    }
}

// ── percentile ─────────────────────────────────────────────────────────────

#[test]
fn percentile_empty_slice() {
    assert_eq!(percentile(&[], 0.5), 0.0);
}

#[test]
fn percentile_single_element() {
    assert_eq!(percentile(&[42], 0.0), 42.0);
    assert_eq!(percentile(&[42], 0.5), 42.0);
    assert_eq!(percentile(&[42], 1.0), 42.0);
}

#[test]
fn percentile_two_elements() {
    assert_eq!(percentile(&[10, 20], 0.0), 10.0);
    assert_eq!(percentile(&[10, 20], 1.0), 20.0);
}

#[test]
fn percentile_median_odd_count() {
    assert_eq!(percentile(&[1, 2, 3, 4, 5], 0.5), 3.0);
}

#[test]
fn percentile_median_even_count() {
    // Ceiling index strategy: p50 of [1,2,3,4] => index ceil(0.5*3)=ceil(1.5)=2 => value 3
    assert_eq!(percentile(&[1, 2, 3, 4], 0.5), 3.0);
}

#[test]
fn percentile_p0() {
    assert_eq!(percentile(&[10, 20, 30, 40, 50], 0.0), 10.0);
}

#[test]
fn percentile_p100() {
    assert_eq!(percentile(&[10, 20, 30, 40, 50], 1.0), 50.0);
}

#[test]
fn percentile_p25_p75() {
    let data = [10, 20, 30, 40, 50];
    let p25 = percentile(&data, 0.25);
    let p75 = percentile(&data, 0.75);
    assert!(p25 <= p75);
}

#[test]
fn percentile_all_same_values() {
    assert_eq!(percentile(&[5, 5, 5, 5, 5], 0.5), 5.0);
}

#[test]
fn percentile_large_dataset() {
    let data: Vec<usize> = (1..=1000).collect();
    let p50 = percentile(&data, 0.5);
    assert!(p50 >= 1.0 && p50 <= 1000.0);
}

#[test]
fn percentile_monotonic_in_pct() {
    let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let p10 = percentile(&data, 0.1);
    let p50 = percentile(&data, 0.5);
    let p90 = percentile(&data, 0.9);
    assert!(p10 <= p50);
    assert!(p50 <= p90);
}

#[test]
fn percentile_deterministic() {
    let data = [3, 7, 11, 15, 19];
    for _ in 0..100 {
        assert_eq!(percentile(&data, 0.5), percentile(&data, 0.5));
    }
}

// ── gini_coefficient ───────────────────────────────────────────────────────

#[test]
fn gini_empty() {
    assert_eq!(gini_coefficient(&[]), 0.0);
}

#[test]
fn gini_single_element() {
    assert_eq!(gini_coefficient(&[42]), 0.0);
}

#[test]
fn gini_all_equal() {
    assert!((gini_coefficient(&[10, 10, 10, 10]) - 0.0).abs() < 1e-10);
}

#[test]
fn gini_all_zeros() {
    assert_eq!(gini_coefficient(&[0, 0, 0]), 0.0);
}

#[test]
fn gini_maximum_inequality() {
    // One person has all; Gini should be close to (n-1)/n for sorted [0,0,...,X]
    let g = gini_coefficient(&[0, 0, 0, 100]);
    assert!(g > 0.5, "expected high gini, got {g}");
}

#[test]
fn gini_in_unit_range() {
    let g = gini_coefficient(&[1, 2, 5, 10, 100]);
    assert!(g >= 0.0 && g <= 1.0, "gini should be in [0,1], got {g}");
}

#[test]
fn gini_two_elements_equal() {
    assert!((gini_coefficient(&[5, 5]) - 0.0).abs() < 1e-10);
}

#[test]
fn gini_two_elements_unequal() {
    let g = gini_coefficient(&[1, 100]);
    assert!(g > 0.0);
}

#[test]
fn gini_increasing_inequality() {
    // More skewed distributions should have higher Gini.
    let g_equal = gini_coefficient(&[10, 10, 10, 10]);
    let g_slight = gini_coefficient(&[5, 10, 10, 15]);
    let g_extreme = gini_coefficient(&[1, 1, 1, 100]);
    assert!(g_equal <= g_slight);
    assert!(g_slight < g_extreme);
}

#[test]
fn gini_large_uniform() {
    let data: Vec<usize> = vec![100; 1000];
    assert!((gini_coefficient(&data) - 0.0).abs() < 1e-10);
}

#[test]
fn gini_deterministic() {
    let data = [1, 3, 5, 7, 100];
    let first = gini_coefficient(&data);
    for _ in 0..100 {
        assert_eq!(gini_coefficient(&data), first);
    }
}

// ── proptest ───────────────────────────────────────────────────────────────

mod property_tests {
    use proptest::prelude::*;
    use tokmd_math::{gini_coefficient, percentile, round_f64, safe_ratio};

    proptest! {
        #[test]
        fn round_f64_idempotent(val in -1e6f64..1e6f64, dec in 0u32..8) {
            let once = round_f64(val, dec);
            let twice = round_f64(once, dec);
            prop_assert!((once - twice).abs() < 1e-12,
                "round_f64 not idempotent: once={once}, twice={twice}");
        }

        #[test]
        fn safe_ratio_non_negative(n in 0usize..100_000, d in 0usize..100_000) {
            let r = safe_ratio(n, d);
            prop_assert!(r >= 0.0, "safe_ratio returned negative: {r}");
        }

        #[test]
        fn safe_ratio_bounded_when_numer_leq_denom(n in 0usize..100, d in 1usize..100) {
            let n = n.min(d);
            let r = safe_ratio(n, d);
            prop_assert!(r <= 1.0001, "ratio > 1 when numer <= denom: {r}");
        }

        #[test]
        fn percentile_in_range(
            data in proptest::collection::vec(0usize..10_000, 1..100),
            pct in 0.0f64..=1.0,
        ) {
            let mut sorted = data.clone();
            sorted.sort();
            let p = percentile(&sorted, pct);
            let min = *sorted.first().unwrap() as f64;
            let max = *sorted.last().unwrap() as f64;
            prop_assert!(p >= min && p <= max,
                "percentile {p} outside [{min}, {max}]");
        }

        #[test]
        fn percentile_monotonic(
            data in proptest::collection::vec(1usize..1000, 2..50),
        ) {
            let mut sorted = data.clone();
            sorted.sort();
            let p25 = percentile(&sorted, 0.25);
            let p75 = percentile(&sorted, 0.75);
            prop_assert!(p25 <= p75, "p25={p25} > p75={p75}");
        }

        #[test]
        fn gini_in_zero_one(
            data in proptest::collection::vec(0usize..10_000, 1..100),
        ) {
            let mut sorted = data.clone();
            sorted.sort();
            let g = gini_coefficient(&sorted);
            prop_assert!(g >= 0.0 && g <= 1.0,
                "gini outside [0,1]: {g}");
        }

        #[test]
        fn gini_zero_for_uniform(val in 1usize..10_000, n in 2usize..50) {
            let data = vec![val; n];
            let g = gini_coefficient(&data);
            prop_assert!((g - 0.0).abs() < 1e-10,
                "gini for uniform data should be 0, got {g}");
        }
    }
}
