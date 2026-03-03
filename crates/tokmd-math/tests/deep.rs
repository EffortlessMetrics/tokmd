//! Deep tests for tokmd-math: numeric precision, mathematical identities,
//! and edge cases not covered by existing unit/bdd/property tests.

use tokmd_math::{gini_coefficient, percentile, round_f64, safe_ratio};

// ── round_f64: precision and rounding direction ─────────────────────

#[test]
fn round_f64_half_rounds_up() {
    // Rust f64::round uses "round half to even" (banker's rounding) for
    // .5 values, but our implementation multiplies then rounds, so verify
    // the actual behavior is consistent.
    assert_eq!(round_f64(2.5, 0), 3.0);
    assert_eq!(round_f64(3.5, 0), 4.0);
    assert_eq!(round_f64(0.5, 0), 1.0);
}

#[test]
fn round_f64_very_high_precision() {
    let val = 1.123_456_789_012_345;
    let r6 = round_f64(val, 6);
    assert!((r6 - 1.123_457).abs() < 1e-10);
}

#[test]
fn round_f64_large_values() {
    assert_eq!(round_f64(1_000_000.555, 2), 1_000_000.56);
    assert_eq!(round_f64(999_999_999.0, 0), 999_999_999.0);
}

#[test]
fn round_f64_very_small_positive() {
    let tiny = 0.000_001;
    assert_eq!(round_f64(tiny, 6), 0.000_001);
    assert_eq!(round_f64(tiny, 4), 0.0);
}

#[test]
fn round_f64_negative_values_symmetry() {
    // Rounding negative values should mirror positive rounding behavior.
    let pos = round_f64(1.235, 2);
    let neg = round_f64(-1.235, 2);
    assert!((pos + neg).abs() < 1e-10, "pos={pos}, neg={neg}");
}

#[test]
fn round_f64_zero_with_many_decimals() {
    assert_eq!(round_f64(0.0, 0), 0.0);
    assert_eq!(round_f64(0.0, 10), 0.0);
}

#[test]
fn round_f64_already_rounded_is_stable() {
    assert_eq!(round_f64(1.23, 2), 1.23);
    assert_eq!(round_f64(100.0, 5), 100.0);
}

// ── safe_ratio: precision and boundary values ───────────────────────

#[test]
fn safe_ratio_one_over_seven_rounds_to_four_decimals() {
    // 1/7 = 0.142857... should round to 0.1429
    assert_eq!(safe_ratio(1, 7), 0.1429);
}

#[test]
fn safe_ratio_two_over_three() {
    // 2/3 = 0.6666... should round to 0.6667
    assert_eq!(safe_ratio(2, 3), 0.6667);
}

#[test]
fn safe_ratio_small_numerator_large_denominator() {
    // 1/1_000_000 = 0.000001, rounds to 0.0 at 4 decimals
    let r = safe_ratio(1, 1_000_000);
    assert_eq!(r, 0.0);
    // But 1/100 should be nonzero
    let r2 = safe_ratio(1, 100);
    assert_eq!(r2, 0.01);
}

#[test]
fn safe_ratio_large_numerator_equals_denom() {
    assert_eq!(safe_ratio(999_999, 999_999), 1.0);
}

#[test]
fn safe_ratio_numerator_exceeds_denominator() {
    let r = safe_ratio(100, 7);
    assert!(r > 14.0);
    assert!(r < 15.0);
}

#[test]
fn safe_ratio_successive_halving() {
    // 1/2, 1/4, 1/8 should produce decreasing ratios
    let r2 = safe_ratio(1, 2);
    let r4 = safe_ratio(1, 4);
    let r8 = safe_ratio(1, 8);
    assert!(r2 > r4);
    assert!(r4 > r8);
    assert_eq!(r2, 0.5);
    assert_eq!(r4, 0.25);
    assert_eq!(r8, 0.125);
}

// ── percentile: interpolation and known quantiles ───────────────────

#[test]
fn percentile_quartiles_on_10_elements() {
    let values: Vec<usize> = (1..=10).collect();
    let p0 = percentile(&values, 0.0);
    let p25 = percentile(&values, 0.25);
    let p50 = percentile(&values, 0.5);
    let p75 = percentile(&values, 0.75);
    let p100 = percentile(&values, 1.0);

    assert_eq!(p0, 1.0);
    assert_eq!(p100, 10.0);
    assert!(p25 <= p50);
    assert!(p50 <= p75);
}

#[test]
fn percentile_large_dataset_bounds() {
    let values: Vec<usize> = (0..1000).collect();
    let p0 = percentile(&values, 0.0);
    let p100 = percentile(&values, 1.0);
    assert_eq!(p0, 0.0);
    assert_eq!(p100, 999.0);
}

#[test]
fn percentile_large_dataset_monotonic() {
    let values: Vec<usize> = (0..1000).collect();
    let quantiles = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
    let results: Vec<f64> = quantiles.iter().map(|q| percentile(&values, *q)).collect();
    for w in results.windows(2) {
        assert!(w[0] <= w[1], "Not monotonic: {} > {}", w[0], w[1]);
    }
}

#[test]
fn percentile_all_identical_values() {
    let values = [42usize; 100];
    for pct in [0.0, 0.25, 0.5, 0.75, 1.0] {
        assert_eq!(percentile(&values, pct), 42.0);
    }
}

#[test]
fn percentile_two_distinct_values() {
    let values = [0usize, 100];
    assert_eq!(percentile(&values, 0.0), 0.0);
    assert_eq!(percentile(&values, 1.0), 100.0);
}

#[test]
fn percentile_three_elements_median() {
    let values = [10usize, 20, 30];
    let p50 = percentile(&values, 0.5);
    assert_eq!(p50, 20.0);
}

// ── gini_coefficient: known mathematical values ─────────────────────

#[test]
fn gini_two_equal_values_is_zero() {
    assert!((gini_coefficient(&[100, 100])).abs() < 1e-10);
}

#[test]
fn gini_maximal_inequality_two_elements() {
    // [0, N] → Gini = 0.5 for any N > 0
    assert!((gini_coefficient(&[0, 1]) - 0.5).abs() < 1e-10);
    assert!((gini_coefficient(&[0, 1_000_000]) - 0.5).abs() < 1e-10);
}

#[test]
fn gini_maximal_inequality_many_elements() {
    // [0, 0, ..., 0, N] → Gini = (n-1)/n
    let n = 10;
    let mut values = vec![0usize; n];
    values[n - 1] = 1000;
    let expected = (n as f64 - 1.0) / n as f64;
    let gini = gini_coefficient(&values);
    assert!(
        (gini - expected).abs() < 1e-10,
        "expected {expected}, got {gini}"
    );
}

#[test]
fn gini_linear_sequence_known_value() {
    // For [1, 2, 3, ..., n], Gini = (n+1)/(3*(n-1)) - ... but let's just
    // verify it's between 0 and 0.5 for typical sequences.
    let values: Vec<usize> = (1..=100).collect();
    let gini = gini_coefficient(&values);
    assert!(gini > 0.0);
    assert!(gini < 0.5);
}

#[test]
fn gini_exponential_distribution_high() {
    let values = [1usize, 2, 4, 8, 16, 32, 64, 128, 256, 512];
    let gini = gini_coefficient(&values);
    // Exponential distribution should have moderately high Gini
    assert!(gini > 0.4);
    assert!(gini <= 1.0);
}

#[test]
fn gini_single_zero_is_zero() {
    assert_eq!(gini_coefficient(&[0]), 0.0);
}

#[test]
fn gini_large_uniform_is_zero() {
    let values = vec![42usize; 1000];
    assert!(gini_coefficient(&values).abs() < 1e-10);
}

#[test]
fn gini_is_scale_invariant() {
    // Multiplying all values by a constant should not change Gini
    let base = [1usize, 3, 5, 10, 20];
    let scaled: Vec<usize> = base.iter().map(|v| v * 100).collect();
    let g_base = gini_coefficient(&base);
    let g_scaled = gini_coefficient(&scaled);
    assert!(
        (g_base - g_scaled).abs() < 1e-10,
        "Gini should be scale-invariant: base={g_base}, scaled={g_scaled}"
    );
}

#[test]
fn gini_adding_zero_increases_inequality() {
    let without = [10usize, 20, 30];
    let with = [0usize, 10, 20, 30];
    let g_without = gini_coefficient(&without);
    let g_with = gini_coefficient(&with);
    assert!(
        g_with > g_without,
        "Adding a zero should increase inequality"
    );
}

// ── Composability: combining functions ──────────────────────────────

#[test]
fn percentile_gini_pipeline() {
    let values: Vec<usize> = (1..=20).collect();
    let p90 = percentile(&values, 0.9);
    let gini = gini_coefficient(&values);
    let ratio = safe_ratio(p90 as usize, *values.last().unwrap());
    let display = round_f64(ratio * 100.0, 1);

    // All results should be well-defined
    assert!(p90 > 0.0);
    assert!(gini > 0.0 && gini < 1.0);
    assert!(display > 0.0 && display <= 100.0);
}

#[test]
fn safe_ratio_composed_with_round() {
    // Composing safe_ratio → round_f64 should be stable
    let r = safe_ratio(17, 31);
    let r2 = round_f64(r, 2);
    let r4 = round_f64(r, 4);
    assert_eq!(r, r4); // safe_ratio already rounds to 4
    assert!(r2 <= r4 + 0.01); // fewer decimals rounds differently
}
