//! Mutation-hardening tests for `tokmd-math`.
//!
//! Each test targets a known mutation-testing survivor pattern:
//! boundary conditions, operator swaps, off-by-one errors, and
//! boolean-logic flips.

use tokmd_math::{gini_coefficient, percentile, round_f64, safe_ratio};

// ── round_f64 ────────────────────────────────────────────────────────

#[test]
fn round_exact_half_rounds_up_zero_decimals() {
    // 0.5 boundary: mutating `round` to `floor`/`ceil` would fail
    assert_eq!(round_f64(0.5, 0), 1.0);
    assert_eq!(round_f64(1.5, 0), 2.0);
    assert_eq!(round_f64(-0.5, 0), -1.0);
}

#[test]
fn round_just_below_half_stays_down() {
    assert_eq!(round_f64(0.4999, 0), 0.0);
    assert_eq!(round_f64(2.449, 1), 2.4);
}

#[test]
fn round_zero_decimals_preserves_integer() {
    assert_eq!(round_f64(7.0, 0), 7.0);
    assert_eq!(round_f64(0.0, 5), 0.0);
}

#[test]
fn round_large_decimal_places() {
    // If `decimals as i32` were off-by-one, precision would differ
    assert_eq!(round_f64(1.123456789, 8), 1.12345679);
}

// ── safe_ratio ───────────────────────────────────────────────────────

#[test]
fn safe_ratio_zero_divisor_returns_zero_not_panic() {
    assert_eq!(safe_ratio(0, 0), 0.0);
    assert_eq!(safe_ratio(100, 0), 0.0);
}

#[test]
fn safe_ratio_identity() {
    // n/n == 1.0 — swapping `==` to `!=` in the guard would break this
    assert_eq!(safe_ratio(1, 1), 1.0);
    assert_eq!(safe_ratio(999, 999), 1.0);
}

#[test]
fn safe_ratio_rounds_to_four_decimals() {
    assert_eq!(safe_ratio(1, 3), 0.3333);
    assert_eq!(safe_ratio(2, 3), 0.6667);
    assert_eq!(safe_ratio(1, 7), 0.1429);
}

#[test]
fn safe_ratio_zero_numerator() {
    assert_eq!(safe_ratio(0, 42), 0.0);
}

// ── percentile ───────────────────────────────────────────────────────

#[test]
fn percentile_empty_returns_zero() {
    assert_eq!(percentile(&[], 0.0), 0.0);
    assert_eq!(percentile(&[], 0.5), 0.0);
    assert_eq!(percentile(&[], 1.0), 0.0);
}

#[test]
fn percentile_single_element() {
    // Any percentile of a single-element slice must return that element
    assert_eq!(percentile(&[42], 0.0), 42.0);
    assert_eq!(percentile(&[42], 0.5), 42.0);
    assert_eq!(percentile(&[42], 1.0), 42.0);
}

#[test]
fn percentile_0th_and_100th() {
    let data = [10, 20, 30, 40, 50];
    assert_eq!(percentile(&data, 0.0), 10.0);
    assert_eq!(percentile(&data, 1.0), 50.0);
}

#[test]
fn percentile_median_odd_len() {
    assert_eq!(percentile(&[1, 2, 3, 4, 5], 0.5), 3.0);
}

#[test]
fn percentile_median_even_len() {
    // With ceil-based indexing, 0.5 of 4-element slice → idx ceil(1.5)=2 → 30
    assert_eq!(percentile(&[10, 20, 30, 40], 0.5), 30.0);
}

// ── gini_coefficient ─────────────────────────────────────────────────

#[test]
fn gini_empty_returns_zero() {
    assert_eq!(gini_coefficient(&[]), 0.0);
}

#[test]
fn gini_single_element_returns_zero() {
    assert_eq!(gini_coefficient(&[1]), 0.0);
    assert_eq!(gini_coefficient(&[999]), 0.0);
}

#[test]
fn gini_all_zeros_returns_zero() {
    // Guards the `sum == 0.0` early-return path
    assert_eq!(gini_coefficient(&[0, 0, 0, 0]), 0.0);
}

#[test]
fn gini_uniform_returns_zero() {
    let g = gini_coefficient(&[5, 5, 5, 5]);
    assert!(g.abs() < 1e-10, "expected 0, got {g}");
}

#[test]
fn gini_maximal_inequality() {
    // [0, 0, 0, N] gives Gini close to 0.75 for 4 elements
    let g = gini_coefficient(&[0, 0, 0, 100]);
    assert!(g > 0.5, "expected high Gini, got {g}");
    assert!(g <= 1.0, "Gini must be ≤1.0, got {g}");
}

#[test]
fn gini_monotonically_increases_with_inequality() {
    let equal = gini_coefficient(&[10, 10, 10, 10]);
    let moderate = gini_coefficient(&[1, 5, 10, 20]);
    let extreme = gini_coefficient(&[0, 0, 0, 100]);
    assert!(
        equal < moderate,
        "equal ({equal}) should be < moderate ({moderate})"
    );
    assert!(
        moderate < extreme,
        "moderate ({moderate}) should be < extreme ({extreme})"
    );
}

#[test]
fn gini_two_elements_known_value() {
    // [0, N] → Gini = 0.5 for any N>0 with 2 elements
    let g = gini_coefficient(&[0, 10]);
    assert!((g - 0.5).abs() < 1e-10, "expected 0.5, got {g}");
}
