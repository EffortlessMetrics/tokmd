//! Wave-59 depth tests for statistical functions — edge cases, known values,
//! overflow behaviour, and COCOMO-style estimation inputs.

use tokmd_math::{gini_coefficient, percentile, round_f64, safe_ratio};

// ── round_f64 ────────────────────────────────────────────────────────────

#[test]
fn round_zero_decimals_truncates_fraction() {
    assert_eq!(round_f64(2.4, 0), 2.0);
    assert_eq!(round_f64(2.5, 0), 3.0); // banker's rounding not used
    assert_eq!(round_f64(2.6, 0), 3.0);
}

#[test]
fn round_high_precision_preserves_digits() {
    assert_eq!(round_f64(1.123_456_789, 6), 1.123_457);
    assert_eq!(round_f64(1.123_456_789, 8), 1.123_456_79);
}

#[test]
fn round_negative_values() {
    assert_eq!(round_f64(-3.456, 2), -3.46);
    assert_eq!(round_f64(-0.005, 2), -0.01);
    assert_eq!(round_f64(-100.999, 1), -101.0);
}

#[test]
fn round_nan_returns_nan() {
    assert!(round_f64(f64::NAN, 2).is_nan());
}

#[test]
fn round_infinity_returns_infinity() {
    assert_eq!(round_f64(f64::INFINITY, 3), f64::INFINITY);
    assert_eq!(round_f64(f64::NEG_INFINITY, 3), f64::NEG_INFINITY);
}

#[test]
fn round_very_large_value() {
    let big = 1e15;
    let result = round_f64(big, 2);
    assert_eq!(result, big);
}

#[test]
fn round_zero_is_zero() {
    assert_eq!(round_f64(0.0, 5), 0.0);
    assert_eq!(round_f64(-0.0, 3), 0.0);
}

// ── safe_ratio ───────────────────────────────────────────────────────────

#[test]
fn safe_ratio_zero_numerator() {
    assert_eq!(safe_ratio(0, 100), 0.0);
}

#[test]
fn safe_ratio_equal_values_gives_one() {
    assert_eq!(safe_ratio(42, 42), 1.0);
}

#[test]
fn safe_ratio_both_zero() {
    assert_eq!(safe_ratio(0, 0), 0.0);
}

#[test]
fn safe_ratio_large_values() {
    let r = safe_ratio(usize::MAX, usize::MAX);
    assert_eq!(r, 1.0);
}

#[test]
fn safe_ratio_one_over_three() {
    assert_eq!(safe_ratio(1, 3), 0.3333);
}

#[test]
fn safe_ratio_two_over_three() {
    assert_eq!(safe_ratio(2, 3), 0.6667);
}

#[test]
fn safe_ratio_numerator_greater_than_denominator() {
    let r = safe_ratio(10, 3);
    assert_eq!(r, 3.3333);
}

// ── percentile ───────────────────────────────────────────────────────────

#[test]
fn percentile_empty_slice_returns_zero() {
    assert_eq!(percentile(&[], 0.5), 0.0);
}

#[test]
fn percentile_single_element() {
    assert_eq!(percentile(std::slice::from_ref(&42usize), 0.0), 42.0);
    assert_eq!(percentile(std::slice::from_ref(&42usize), 0.5), 42.0);
    assert_eq!(percentile(std::slice::from_ref(&42usize), 1.0), 42.0);
}

#[test]
fn percentile_two_elements() {
    let data = [10, 20];
    assert_eq!(percentile(&data, 0.0), 10.0);
    assert_eq!(percentile(&data, 1.0), 20.0);
}

#[test]
fn percentile_median_of_odd_count() {
    let data = [1, 2, 3, 4, 5];
    assert_eq!(percentile(&data, 0.5), 3.0);
}

#[test]
fn percentile_median_of_even_count() {
    let data = [10, 20, 30, 40];
    // ceil-based indexing: idx = ceil(0.5 * 3) = 2 → 30
    assert_eq!(percentile(&data, 0.5), 30.0);
}

#[test]
fn percentile_all_same() {
    let data = [7, 7, 7, 7, 7];
    assert_eq!(percentile(&data, 0.0), 7.0);
    assert_eq!(percentile(&data, 0.5), 7.0);
    assert_eq!(percentile(&data, 1.0), 7.0);
}

#[test]
fn percentile_all_zeros() {
    let data = [0, 0, 0, 0];
    assert_eq!(percentile(&data, 0.5), 0.0);
}

#[test]
fn percentile_known_quartiles() {
    let data: Vec<usize> = (1..=100).collect();
    assert_eq!(percentile(&data, 0.0), 1.0);
    assert_eq!(percentile(&data, 1.0), 100.0);
    // p25: ceil(0.25 * 99) = 25 → data[25] = 26
    assert_eq!(percentile(&data, 0.25), 26.0);
}

#[test]
fn percentile_monotonic_across_range() {
    let data: Vec<usize> = (0..50).collect();
    let mut prev = percentile(&data, 0.0);
    for i in 1..=20 {
        let pct = i as f64 / 20.0;
        let cur = percentile(&data, pct);
        assert!(
            cur >= prev,
            "percentile should be monotonically non-decreasing"
        );
        prev = cur;
    }
}

// ── gini_coefficient ─────────────────────────────────────────────────────

#[test]
fn gini_empty_returns_zero() {
    assert_eq!(gini_coefficient(&[]), 0.0);
}

#[test]
fn gini_single_element_returns_zero() {
    assert_eq!(gini_coefficient(std::slice::from_ref(&1000usize)), 0.0);
}

#[test]
fn gini_all_zeros_returns_zero() {
    assert_eq!(gini_coefficient(&[0, 0, 0, 0, 0]), 0.0);
}

#[test]
fn gini_uniform_distribution_is_zero() {
    assert!((gini_coefficient(&[10, 10, 10, 10]) - 0.0).abs() < 1e-10);
}

#[test]
fn gini_maximum_inequality() {
    // [0, 0, 0, N] should approach 0.75 for 4 elements
    let g = gini_coefficient(&[0, 0, 0, 1000]);
    assert!(
        g > 0.7,
        "near-maximum inequality should produce gini > 0.7, got {g}"
    );
    assert!(g <= 1.0, "gini should never exceed 1.0");
}

#[test]
fn gini_known_value_three_elements() {
    // [1, 2, 3]: Gini = (2*1+2*2*2+2*3*3 - (3+1)*(1+2+3)) / (3*6)
    // Manual: accum = (2*1-4)*1 + (2*2-4)*2 + (2*3-4)*3 = -2+0+6 = 4
    // gini = 4 / (3*6) = 4/18 ≈ 0.2222
    let g = gini_coefficient(&[1, 2, 3]);
    assert!(
        (g - 2.0 / 9.0).abs() < 1e-10,
        "expected 2/9 ≈ 0.2222, got {g}"
    );
}

#[test]
fn gini_bounded_zero_to_one() {
    let cases: &[&[usize]] = &[
        &[1, 1, 1],
        &[0, 0, 100],
        &[1, 2, 3, 4, 5],
        &[1, 100],
        &[1, 1, 1, 1, 1, 1, 1000],
    ];
    for data in cases {
        let g = gini_coefficient(data);
        assert!(g >= 0.0, "gini should be >= 0.0 for {data:?}, got {g}");
        assert!(g <= 1.0, "gini should be <= 1.0 for {data:?}, got {g}");
    }
}

#[test]
fn gini_scale_invariant() {
    let base = [1usize, 2, 5, 10];
    let scaled: Vec<usize> = base.iter().map(|v| v * 1000).collect();
    let g1 = gini_coefficient(&base);
    let g2 = gini_coefficient(&scaled);
    assert!((g1 - g2).abs() < 1e-10, "gini should be scale-invariant");
}

#[test]
fn gini_two_elements_asymmetric() {
    // [1, 99]: accum = (2*1-3)*1 + (2*2-3)*99 = -1+99 = 98
    // gini = 98 / (2*100) = 0.49
    let g = gini_coefficient(&[1, 99]);
    assert!((g - 0.49).abs() < 1e-10, "expected 0.49, got {g}");
}

// ── COCOMO-style composition ─────────────────────────────────────────────

#[test]
fn cocomo_effort_round_trip() {
    // COCOMO basic: effort = a * (KLOC)^b
    // Verify round_f64 + safe_ratio compose for typical metric pipelines
    let total_lines: usize = 15_000;
    let code_lines: usize = 10_500;
    let ratio = safe_ratio(code_lines, total_lines);
    assert_eq!(ratio, 0.7);

    let kloc = code_lines as f64 / 1000.0;
    let effort = round_f64(2.4 * kloc.powf(1.05), 2);
    assert!(effort > 0.0);
    assert!(effort.is_finite());
}

// ── Determinism ──────────────────────────────────────────────────────────

#[test]
fn all_functions_deterministic_over_100_iterations() {
    let data = [1usize, 5, 10, 20, 50];
    for _ in 0..100 {
        assert_eq!(round_f64(1.23456, 3), 1.235);
        assert_eq!(safe_ratio(7, 13), 0.5385);
        assert_eq!(percentile(&data, 0.5), 10.0);
        let g = gini_coefficient(&data);
        assert!((g - gini_coefficient(&data)).abs() < 1e-10);
    }
}
