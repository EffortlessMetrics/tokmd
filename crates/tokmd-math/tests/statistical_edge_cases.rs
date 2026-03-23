//! Edge-case tests for statistical helpers in tokmd-math.

use proptest::prelude::*;
use tokmd_math::{gini_coefficient, percentile, round_f64, safe_ratio};

// ── Median with all identical values ────────────────────────────────

#[test]
fn median_of_identical_values_returns_that_value() {
    let values = [5usize; 100];
    assert_eq!(percentile(&values, 0.5), 5.0);
}

#[test]
fn median_of_identical_zeros_returns_zero() {
    let values = [0usize; 50];
    assert_eq!(percentile(&values, 0.5), 0.0);
}

#[test]
fn median_of_identical_large_value() {
    let values = [999_999usize; 10];
    assert_eq!(percentile(&values, 0.5), 999_999.0);
}

// ── Percentile with 1 element ───────────────────────────────────────

#[test]
fn percentile_single_element_at_every_quantile() {
    for pct in [0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0] {
        assert_eq!(percentile(&[42], pct), 42.0, "pct={pct}");
    }
}

#[test]
fn percentile_single_zero_element() {
    assert_eq!(percentile(&[0], 0.5), 0.0);
}

// ── Standard deviation / variance proxy (Gini with 0 variance) ─────

#[test]
fn gini_zero_variance_uniform_small() {
    let values = [10usize, 10, 10];
    assert!(gini_coefficient(&values).abs() < 1e-10);
}

#[test]
fn gini_zero_variance_uniform_large() {
    let values = vec![42usize; 1000];
    assert!(gini_coefficient(&values).abs() < 1e-10);
}

// ── COCOMO-like extreme inputs (ratio at 0 LOC and very large LOC) ──

#[test]
fn safe_ratio_zero_loc_numerator() {
    assert_eq!(safe_ratio(0, 1_000_000), 0.0);
}

#[test]
fn safe_ratio_very_large_numerator() {
    let ratio = safe_ratio(10_000_000, 1);
    assert_eq!(ratio, 10_000_000.0);
}

#[test]
fn safe_ratio_very_large_both() {
    let ratio = safe_ratio(1_000_000, 1_000_000);
    assert_eq!(ratio, 1.0);
}

#[test]
fn safe_ratio_large_numerator_small_denominator() {
    let ratio = safe_ratio(999_999, 3);
    assert!(ratio > 100_000.0);
    assert!(ratio.is_finite());
}

// ── Ratio calculations with zero denominators ───────────────────────

#[test]
fn safe_ratio_zero_zero() {
    assert_eq!(safe_ratio(0, 0), 0.0);
}

#[test]
fn safe_ratio_nonzero_zero() {
    assert_eq!(safe_ratio(42, 0), 0.0);
}

#[test]
fn safe_ratio_max_usize_zero() {
    assert_eq!(safe_ratio(usize::MAX, 0), 0.0);
}

// ── round_f64 edge cases ────────────────────────────────────────────

#[test]
fn round_f64_exact_half_rounds_away_from_zero() {
    // Rust's f64::round uses "round half away from zero"
    assert_eq!(round_f64(0.5, 0), 1.0);
    assert_eq!(round_f64(-0.5, 0), -1.0);
    assert_eq!(round_f64(2.5, 0), 3.0);
}

#[test]
fn round_f64_high_precision() {
    let val = round_f64(1.0 / 3.0, 10);
    assert!((val - 0.3333333333).abs() < 1e-10);
}

// ── Gini with extreme inequality ────────────────────────────────────

#[test]
fn gini_extreme_inequality() {
    let mut values = vec![0usize; 99];
    values.push(1_000_000);
    values.sort();
    let g = gini_coefficient(&values);
    assert!(g > 0.9, "extreme inequality should yield high Gini: {g}");
}

#[test]
fn gini_two_element_max_inequality() {
    let g = gini_coefficient(&[0, 100]);
    assert!((g - 0.5).abs() < 1e-10);
}

// ── Percentile boundary behavior ────────────────────────────────────

#[test]
fn percentile_two_elements_boundary() {
    let values = [1usize, 100];
    assert_eq!(percentile(&values, 0.0), 1.0);
    assert_eq!(percentile(&values, 1.0), 100.0);
}

#[test]
fn percentile_large_sorted_slice() {
    let values: Vec<usize> = (1..=1000).collect();
    assert_eq!(percentile(&values, 0.0), 1.0);
    assert_eq!(percentile(&values, 1.0), 1000.0);
    let median = percentile(&values, 0.5);
    assert!((1.0..=1000.0).contains(&median));
}

// ── Property test: mean proxy is between min and max ────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    #[test]
    fn mean_is_between_min_and_max(
        values in prop::collection::vec(1usize..10000, 2..100)
    ) {
        let sum: f64 = values.iter().map(|v| *v as f64).sum();
        let mean = sum / values.len() as f64;
        let min_val = values.iter().min().copied().unwrap_or(0) as f64;
        let max_val = values.iter().max().copied().unwrap_or(0) as f64;
        prop_assert!(mean >= min_val, "mean {mean} < min {min_val}");
        prop_assert!(mean <= max_val, "mean {mean} > max {max_val}");
    }

    #[test]
    fn safe_ratio_zero_denom_always_zero(numer in 0usize..1_000_000) {
        prop_assert_eq!(safe_ratio(numer, 0), 0.0);
    }

    #[test]
    fn gini_uniform_always_zero(value in 1usize..10000, len in 2usize..50) {
        let values = vec![value; len];
        let g = gini_coefficient(&values);
        prop_assert!(g.abs() < 1e-10, "uniform Gini should be ~0, got {g}");
    }

    #[test]
    fn percentile_identical_values_always_returns_value(
        value in 0usize..10000,
        len in 1usize..100,
        pct in 0.0f64..=1.0
    ) {
        let values = vec![value; len];
        prop_assert_eq!(percentile(&values, pct), value as f64);
    }
}
