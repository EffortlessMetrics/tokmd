//! Comprehensive depth tests for tokmd-math – wave 55.

use tokmd_math::{gini_coefficient, percentile, round_f64, safe_ratio};

// ── round_f64 ───────────────────────────────────────────────────────

#[test]
fn round_zero_decimals_truncates() {
    assert_eq!(round_f64(2.4, 0), 2.0);
    assert_eq!(round_f64(2.5, 0), 3.0);
    assert_eq!(round_f64(2.6, 0), 3.0);
}

#[test]
fn round_one_decimal() {
    assert_eq!(round_f64(1.25, 1), 1.3);
    assert_eq!(round_f64(1.24, 1), 1.2);
}

#[test]
fn round_many_decimals_preserves_value() {
    let v = 1.123_456_789;
    assert_eq!(round_f64(v, 8), 1.12345679);
    assert_eq!(round_f64(v, 6), 1.123457);
}

#[test]
fn round_negative_values() {
    assert_eq!(round_f64(-2.555, 2), -2.56);
    assert_eq!(round_f64(-1.5, 0), -2.0);
}

#[test]
fn round_zero_stays_zero() {
    assert_eq!(round_f64(0.0, 5), 0.0);
}

#[test]
fn round_very_large_value() {
    let big = 1e15;
    assert_eq!(round_f64(big, 2), big);
}

#[test]
fn round_nan_stays_nan() {
    assert!(round_f64(f64::NAN, 2).is_nan());
}

#[test]
fn round_infinity_stays_infinity() {
    assert_eq!(round_f64(f64::INFINITY, 2), f64::INFINITY);
    assert_eq!(round_f64(f64::NEG_INFINITY, 2), f64::NEG_INFINITY);
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
fn safe_ratio_one_third() {
    assert_eq!(safe_ratio(1, 3), 0.3333);
}

#[test]
fn safe_ratio_two_thirds() {
    assert_eq!(safe_ratio(2, 3), 0.6667);
}

#[test]
fn safe_ratio_large_values() {
    let r = safe_ratio(1_000_000, 3_000_000);
    assert_eq!(r, 0.3333);
}

#[test]
fn safe_ratio_denominator_one() {
    assert_eq!(safe_ratio(42, 1), 42.0);
}

// ── percentile ──────────────────────────────────────────────────────

#[test]
fn percentile_empty_returns_zero() {
    assert_eq!(percentile(&[], 0.5), 0.0);
}

#[test]
fn percentile_single_element() {
    assert_eq!(percentile(&[99], 0.0), 99.0);
    assert_eq!(percentile(&[99], 0.5), 99.0);
    assert_eq!(percentile(&[99], 1.0), 99.0);
}

#[test]
fn percentile_median_odd_count() {
    assert_eq!(percentile(&[1, 2, 3, 4, 5], 0.5), 3.0);
}

#[test]
fn percentile_median_even_count() {
    // With ceil-based indexing, p50 of [1,2,3,4] → index ceil(0.5*3)=2 → value 3
    assert_eq!(percentile(&[1, 2, 3, 4], 0.5), 3.0);
}

#[test]
fn percentile_p0_returns_minimum() {
    assert_eq!(percentile(&[10, 20, 30], 0.0), 10.0);
}

#[test]
fn percentile_p100_returns_maximum() {
    assert_eq!(percentile(&[10, 20, 30], 1.0), 30.0);
}

#[test]
fn percentile_p90_large_set() {
    let data: Vec<usize> = (1..=100).collect();
    assert_eq!(percentile(&data, 0.9), 91.0);
}

// ── gini_coefficient ────────────────────────────────────────────────

#[test]
fn gini_empty_is_zero() {
    assert_eq!(gini_coefficient(&[]), 0.0);
}

#[test]
fn gini_single_element_is_zero() {
    assert_eq!(gini_coefficient(&[42]), 0.0);
}

#[test]
fn gini_all_zeros_is_zero() {
    assert_eq!(gini_coefficient(&[0, 0, 0, 0]), 0.0);
}

#[test]
fn gini_uniform_is_zero() {
    let g = gini_coefficient(&[10, 10, 10, 10]);
    assert!(g.abs() < 1e-10, "expected ~0, got {g}");
}

#[test]
fn gini_maximal_inequality() {
    // One element holds all the value — Gini should approach (n-1)/n
    let g = gini_coefficient(&[0, 0, 0, 1000]);
    assert!(g > 0.7, "expected high gini, got {g}");
}

#[test]
fn gini_moderate_inequality() {
    let g = gini_coefficient(&[1, 2, 3, 4]);
    assert!(g > 0.0 && g < 1.0, "expected 0 < gini < 1, got {g}");
}

#[test]
fn gini_two_elements_equal() {
    let g = gini_coefficient(&[5, 5]);
    assert!(g.abs() < 1e-10);
}

#[test]
fn gini_two_elements_unequal() {
    let g = gini_coefficient(&[0, 10]);
    assert!(g > 0.0, "expected positive gini for unequal pair, got {g}");
}

#[test]
fn gini_is_bounded_zero_to_one() {
    for data in [vec![1, 1, 1], vec![1, 2, 3], vec![1, 100], vec![0, 0, 0, 1]] {
        let g = gini_coefficient(&data);
        assert!(
            (0.0..=1.0).contains(&g),
            "gini {g} out of [0,1] for {data:?}"
        );
    }
}

#[test]
fn gini_deterministic_across_calls() {
    let data = [1, 3, 5, 7, 9];
    let a = gini_coefficient(&data);
    let b = gini_coefficient(&data);
    assert_eq!(a, b);
}
