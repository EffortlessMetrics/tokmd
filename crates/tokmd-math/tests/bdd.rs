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
