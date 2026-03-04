//! Numerical stability and precision tests for tokmd-math.

use proptest::prelude::*;
use tokmd_math::{gini_coefficient, percentile, round_f64, safe_ratio};

// ── Very large numbers ──────────────────────────────────────────────

#[test]
fn safe_ratio_large_values_no_panic() {
    let large = usize::MAX / 2;
    let ratio = safe_ratio(large, large);
    assert_eq!(ratio, 1.0);
}

#[test]
fn safe_ratio_near_max_usize() {
    let ratio = safe_ratio(usize::MAX, 1);
    assert!(ratio.is_finite());
    assert!(ratio > 0.0);
}

#[test]
fn safe_ratio_max_over_max_is_one() {
    assert_eq!(safe_ratio(usize::MAX, usize::MAX), 1.0);
}

#[test]
fn percentile_with_large_values() {
    let values = [usize::MAX - 2, usize::MAX - 1, usize::MAX];
    let p = percentile(&values, 0.5);
    assert!(p.is_finite());
    assert!(p > 0.0);
}

#[test]
fn gini_with_large_values_no_panic() {
    let values = [1usize, usize::MAX / 1000, usize::MAX / 100];
    let g = gini_coefficient(&values);
    assert!(g.is_finite());
    assert!(g >= 0.0);
}

// ── Floating point edge cases ───────────────────────────────────────

#[test]
fn round_f64_nan_stays_nan() {
    assert!(round_f64(f64::NAN, 2).is_nan());
}

#[test]
fn round_f64_infinity_stays_infinity() {
    assert_eq!(round_f64(f64::INFINITY, 3), f64::INFINITY);
    assert_eq!(round_f64(f64::NEG_INFINITY, 3), f64::NEG_INFINITY);
}

#[test]
fn round_f64_negative_zero() {
    let val = round_f64(-0.0, 5);
    assert_eq!(val, 0.0);
}

#[test]
fn round_f64_very_small_positive() {
    let val = round_f64(f64::MIN_POSITIVE, 10);
    assert!(val.is_finite());
}

#[test]
fn round_f64_subnormal() {
    let subnormal = 5e-324_f64;
    let val = round_f64(subnormal, 5);
    assert!(val.is_finite());
}

#[test]
fn round_f64_large_magnitude() {
    let val = round_f64(1e15, 2);
    assert!(val.is_finite());
    assert_eq!(val, 1e15);
}

#[test]
fn round_f64_max_decimals_does_not_overflow() {
    // u32::MAX as i32 is negative, but powi should still return finite
    let val = round_f64(1.5, 15);
    assert!(val.is_finite());
}

// ── Accumulator overflow prevention ─────────────────────────────────

#[test]
fn gini_many_large_values_no_overflow() {
    let values = vec![usize::MAX / 10000; 100];
    let g = gini_coefficient(&values);
    assert!(g.is_finite());
    // Uniform distribution — should be near zero
    assert!(g.abs() < 1e-5);
}

#[test]
fn gini_increasing_large_values_is_finite() {
    let values: Vec<usize> = (1..=100).map(|i| i * (usize::MAX / 100_000)).collect();
    let g = gini_coefficient(&values);
    assert!(g.is_finite());
    assert!(g >= 0.0);
}

#[test]
fn percentile_thousand_elements_no_issue() {
    let values: Vec<usize> = (0..1000).collect();
    let p50 = percentile(&values, 0.5);
    assert!(p50.is_finite());
    assert!(p50 >= 0.0 && p50 <= 999.0);
}

// ── safe_ratio precision ────────────────────────────────────────────

#[test]
fn safe_ratio_precision_thirds() {
    assert_eq!(safe_ratio(1, 3), 0.3333);
    assert_eq!(safe_ratio(2, 3), 0.6667);
}

#[test]
fn safe_ratio_precision_sevenths() {
    let ratio = safe_ratio(1, 7);
    // 1/7 ≈ 0.142857... → rounds to 0.1429
    assert_eq!(ratio, 0.1429);
}

// ── Property tests: no panics on valid input ────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn round_f64_never_panics(value in prop::num::f64::ANY, decimals in 0u32..20) {
        let _ = round_f64(value, decimals);
    }

    #[test]
    fn safe_ratio_never_panics(numer in 0usize..=usize::MAX, denom in 0usize..=usize::MAX) {
        let r = safe_ratio(numer, denom);
        if denom == 0 {
            prop_assert_eq!(r, 0.0);
        } else {
            prop_assert!(r.is_finite());
        }
    }

    #[test]
    fn percentile_never_panics(
        mut values in prop::collection::vec(0usize..=1_000_000_000, 0..200),
        pct in 0.0f64..=1.0
    ) {
        values.sort();
        let p = percentile(&values, pct);
        prop_assert!(p.is_finite() || values.is_empty());
    }

    #[test]
    fn gini_never_panics(
        values in prop::collection::vec(0usize..=1_000_000, 0..200)
    ) {
        let mut sorted = values;
        sorted.sort();
        let g = gini_coefficient(&sorted);
        prop_assert!(g.is_finite());
    }

    #[test]
    fn round_f64_finite_input_finite_output(
        value in -1e12f64..1e12,
        decimals in 0u32..10
    ) {
        let result = round_f64(value, decimals);
        prop_assert!(result.is_finite(), "finite input should yield finite output");
    }

    #[test]
    fn safe_ratio_result_is_always_finite_or_zero(
        numer in 0usize..10_000_000,
        denom in 0usize..10_000_000
    ) {
        let r = safe_ratio(numer, denom);
        prop_assert!(r.is_finite());
        prop_assert!(r >= 0.0);
    }
}
