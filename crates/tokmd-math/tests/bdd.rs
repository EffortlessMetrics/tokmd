use tokmd_math::{gini_coefficient, percentile, round_f64, safe_ratio};

#[test]
fn given_empty_series_when_percentile_is_requested_then_zero_is_returned() {
    let values: [usize; 0] = [];
    let got = percentile(&values, 0.5);
    assert_eq!(got, 0.0);
}

#[test]
fn given_uniform_distribution_when_gini_is_computed_then_result_is_zero() {
    let values = [42usize, 42, 42, 42];
    let got = gini_coefficient(&values);
    assert!(got.abs() < 1e-10);
}

#[test]
fn given_zero_denominator_when_safe_ratio_is_used_then_result_is_zero() {
    let got = safe_ratio(99, 0);
    assert_eq!(got, 0.0);
}

#[test]
fn given_fraction_when_rounding_then_requested_precision_is_applied() {
    let got = round_f64(12.34567, 3);
    assert_eq!(got, 12.346);
}

// --- round_f64 edge cases ---

#[test]
fn given_zero_decimals_when_rounding_then_integer_is_returned() {
    assert_eq!(round_f64(3.7, 0), 4.0);
    assert_eq!(round_f64(3.4, 0), 3.0);
}

#[test]
fn given_negative_value_when_rounding_then_sign_is_preserved() {
    assert_eq!(round_f64(-2.555, 2), -2.56);
    assert_eq!(round_f64(-0.001, 2), 0.0);
}

#[test]
fn given_nan_when_rounding_then_nan_is_returned() {
    let got = round_f64(f64::NAN, 2);
    assert!(got.is_nan());
}

#[test]
fn given_infinity_when_rounding_then_infinity_is_returned() {
    assert_eq!(round_f64(f64::INFINITY, 2), f64::INFINITY);
    assert_eq!(round_f64(f64::NEG_INFINITY, 2), f64::NEG_INFINITY);
}

#[test]
fn given_zero_when_rounding_then_zero_is_returned() {
    assert_eq!(round_f64(0.0, 5), 0.0);
}

// --- safe_ratio edge cases ---

#[test]
fn given_zero_numerator_when_safe_ratio_then_zero_is_returned() {
    assert_eq!(safe_ratio(0, 100), 0.0);
}

#[test]
fn given_numerator_greater_than_denominator_when_safe_ratio_then_ratio_exceeds_one() {
    let got = safe_ratio(10, 3);
    assert!(got > 1.0);
}

#[test]
fn given_equal_numerator_and_denominator_when_safe_ratio_then_one_is_returned() {
    assert_eq!(safe_ratio(42, 42), 1.0);
}

#[test]
fn given_both_zero_when_safe_ratio_then_zero_is_returned() {
    assert_eq!(safe_ratio(0, 0), 0.0);
}

// --- percentile edge cases ---

#[test]
fn given_single_element_when_percentile_is_requested_then_that_element_is_returned() {
    assert_eq!(percentile(&[42], 0.0), 42.0);
    assert_eq!(percentile(&[42], 0.5), 42.0);
    assert_eq!(percentile(&[42], 1.0), 42.0);
}

#[test]
fn given_all_same_values_when_any_percentile_then_that_value_is_returned() {
    let values = [7usize, 7, 7, 7, 7];
    assert_eq!(percentile(&values, 0.0), 7.0);
    assert_eq!(percentile(&values, 0.5), 7.0);
    assert_eq!(percentile(&values, 1.0), 7.0);
}

#[test]
fn given_two_elements_when_median_percentile_then_upper_element_is_returned() {
    let values = [10usize, 20];
    let got = percentile(&values, 0.5);
    assert_eq!(got, 20.0);
}

#[test]
fn given_sorted_slice_when_0th_percentile_then_min_is_returned() {
    let values = [1usize, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    assert_eq!(percentile(&values, 0.0), 1.0);
}

#[test]
fn given_sorted_slice_when_100th_percentile_then_max_is_returned() {
    let values = [1usize, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    assert_eq!(percentile(&values, 1.0), 10.0);
}

// --- gini_coefficient edge cases ---

#[test]
fn given_single_element_when_gini_then_zero_is_returned() {
    assert_eq!(gini_coefficient(&[100]), 0.0);
}

#[test]
fn given_all_zeros_when_gini_then_zero_is_returned() {
    assert_eq!(gini_coefficient(&[0, 0, 0, 0]), 0.0);
}

#[test]
fn given_two_elements_with_max_inequality_when_gini_then_result_is_half() {
    // For [0, N], Gini = (2*2 - 2 - 1)*N / (2*N) = N/(2N) = 0.5
    let gini = gini_coefficient(&[0, 1000]);
    assert!((gini - 0.5).abs() < 1e-10);
}

#[test]
fn given_ascending_sequence_when_gini_then_result_reflects_moderate_inequality() {
    let values = [1usize, 2, 3, 4, 5];
    let gini = gini_coefficient(&values);
    assert!(gini > 0.0);
    assert!(gini < 0.5);
}
