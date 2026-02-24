use tokmd_math::{gini_coefficient, percentile, round_f64, safe_ratio};

#[test]
fn stats_pipeline_is_deterministic_for_same_input() {
    let values = [1usize, 3, 8, 21, 34, 55];

    let p90_a = percentile(&values, 0.90);
    let p90_b = percentile(&values, 0.90);
    let gini_a = gini_coefficient(&values);
    let gini_b = gini_coefficient(&values);

    assert_eq!(p90_a, p90_b);
    assert_eq!(gini_a, gini_b);
}

#[test]
fn ratio_then_round_can_be_used_for_percentage_display() {
    let ratio = safe_ratio(3, 8);
    let pct = round_f64(ratio * 100.0, 2);
    assert_eq!(ratio, 0.375);
    assert_eq!(pct, 37.5);
}

#[test]
fn percentile_bounds_match_input_range() {
    let values = [2usize, 4, 6, 8];
    let low = percentile(&values, 0.0);
    let high = percentile(&values, 1.0);
    assert_eq!(low, 2.0);
    assert_eq!(high, 8.0);
}

#[test]
fn extreme_inequality_has_high_gini() {
    let values = [0usize, 0, 0, 1000];
    let gini = gini_coefficient(&values);
    assert!(gini > 0.7);
}
