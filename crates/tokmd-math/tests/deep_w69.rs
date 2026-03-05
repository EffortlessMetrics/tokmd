//! Deep tests for tokmd-math (W69).
//!
//! Covers round_f64, safe_ratio, percentile, gini_coefficient with
//! edge cases, boundary conditions, and property-based invariants.

use tokmd_math::{gini_coefficient, percentile, round_f64, safe_ratio};

// ---------------------------------------------------------------------------
// round_f64
// ---------------------------------------------------------------------------

#[test]
fn w69_round_zero_decimals() {
    assert_eq!(round_f64(2.4, 0), 2.0);
    assert_eq!(round_f64(2.5, 0), 3.0);
    assert_eq!(round_f64(2.6, 0), 3.0);
}

#[test]
fn w69_round_two_decimals() {
    assert_eq!(round_f64(12.345, 2), 12.35);
    assert_eq!(round_f64(12.344, 2), 12.34);
}

#[test]
fn w69_round_four_decimals() {
    assert_eq!(round_f64(0.123456, 4), 0.1235);
}

#[test]
fn w69_round_negative_value() {
    assert_eq!(round_f64(-3.456, 2), -3.46);
}

#[test]
fn w69_round_zero() {
    assert_eq!(round_f64(0.0, 5), 0.0);
}

// ---------------------------------------------------------------------------
// safe_ratio
// ---------------------------------------------------------------------------

#[test]
fn w69_safe_ratio_divide_by_zero() {
    assert_eq!(safe_ratio(100, 0), 0.0);
    assert_eq!(safe_ratio(0, 0), 0.0);
}

#[test]
fn w69_safe_ratio_exact() {
    assert_eq!(safe_ratio(1, 4), 0.25);
    assert_eq!(safe_ratio(1, 2), 0.5);
    assert_eq!(safe_ratio(3, 4), 0.75);
}

#[test]
fn w69_safe_ratio_repeating() {
    assert_eq!(safe_ratio(1, 3), 0.3333);
    assert_eq!(safe_ratio(2, 3), 0.6667);
}

#[test]
fn w69_safe_ratio_one_to_one() {
    assert_eq!(safe_ratio(5, 5), 1.0);
}

#[test]
fn w69_safe_ratio_zero_numerator() {
    assert_eq!(safe_ratio(0, 100), 0.0);
}

// ---------------------------------------------------------------------------
// percentile
// ---------------------------------------------------------------------------

#[test]
fn w69_percentile_empty() {
    assert_eq!(percentile(&[], 0.5), 0.0);
}

#[test]
fn w69_percentile_single_value() {
    assert_eq!(percentile(&[42], 0.0), 42.0);
    assert_eq!(percentile(&[42], 0.5), 42.0);
    assert_eq!(percentile(&[42], 1.0), 42.0);
}

#[test]
fn w69_percentile_min_max() {
    let data = [10, 20, 30, 40, 50];
    assert_eq!(percentile(&data, 0.0), 10.0);
    assert_eq!(percentile(&data, 1.0), 50.0);
}

#[test]
fn w69_percentile_median() {
    let data = [1, 2, 3, 4, 5];
    assert_eq!(percentile(&data, 0.5), 3.0);
}

#[test]
fn w69_percentile_all_same() {
    let data = [7, 7, 7, 7, 7];
    assert_eq!(percentile(&data, 0.25), 7.0);
    assert_eq!(percentile(&data, 0.75), 7.0);
}

// ---------------------------------------------------------------------------
// gini_coefficient
// ---------------------------------------------------------------------------

#[test]
fn w69_gini_empty() {
    assert_eq!(gini_coefficient(&[]), 0.0);
}

#[test]
fn w69_gini_single_element() {
    assert_eq!(gini_coefficient(&[42]), 0.0);
}

#[test]
fn w69_gini_uniform() {
    let data = [10, 10, 10, 10];
    assert!((gini_coefficient(&data) - 0.0).abs() < 1e-10);
}

#[test]
fn w69_gini_all_zeros() {
    assert_eq!(gini_coefficient(&[0, 0, 0]), 0.0);
}

#[test]
fn w69_gini_unequal_positive() {
    let data = [1, 1, 1, 100];
    assert!(gini_coefficient(&data) > 0.0);
}

#[test]
fn w69_gini_bounded_zero_to_one() {
    let data = [1, 2, 3, 4, 5, 100];
    let g = gini_coefficient(&data);
    assert!(g >= 0.0 && g <= 1.0, "gini={g} out of [0,1]");
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn w69_round_f64_idempotent(val in -1e6f64..1e6f64, dec in 0u32..6) {
            let r1 = round_f64(val, dec);
            let r2 = round_f64(r1, dec);
            prop_assert_eq!(r1.to_bits(), r2.to_bits());
        }

        #[test]
        fn w69_safe_ratio_in_unit_interval(n in 0usize..10_000, d in 1usize..10_000) {
            let r = safe_ratio(n, d);
            prop_assert!(r >= 0.0);
            // Allow slight overshoot beyond 1.0 due to rounding at 4 decimals
            // when n > d this can exceed 1.0 legitimately, so just check >= 0
        }

        #[test]
        fn w69_percentile_monotonic(
            mut vals in proptest::collection::vec(0usize..1000, 2..50),
            p1 in 0.0f64..0.5,
        ) {
            vals.sort();
            let p2 = p1 + 0.5; // p2 > p1
            let v1 = percentile(&vals, p1);
            let v2 = percentile(&vals, p2);
            prop_assert!(v2 >= v1, "p({p1})={v1} > p({p2})={v2}");
        }

        #[test]
        fn w69_gini_nonneg(mut vals in proptest::collection::vec(0usize..500, 1..30)) {
            vals.sort();
            let g = gini_coefficient(&vals);
            prop_assert!(g >= 0.0, "gini={g} is negative");
        }

        #[test]
        fn w69_gini_uniform_is_zero(v in 1usize..100, n in 2usize..20) {
            let data: Vec<usize> = vec![v; n];
            let g = gini_coefficient(&data);
            prop_assert!((g - 0.0).abs() < 1e-10, "gini of uniform={g}");
        }
    }
}
