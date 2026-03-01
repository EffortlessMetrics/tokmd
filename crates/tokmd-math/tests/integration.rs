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

#[test]
fn safe_ratio_result_is_rounded_to_four_decimals() {
    // 1/3 = 0.33333... should become 0.3333
    let got = safe_ratio(1, 3);
    assert_eq!(got, 0.3333);
}

#[test]
fn percentile_p50_on_odd_length_returns_middle_element() {
    let values = [10usize, 20, 30, 40, 50];
    let got = percentile(&values, 0.5);
    assert_eq!(got, 30.0);
}

#[test]
fn gini_increases_with_inequality() {
    let uniform = [10usize, 10, 10, 10];
    let moderate = [1usize, 5, 10, 20];
    let extreme = [0usize, 0, 0, 100];

    let g_uniform = gini_coefficient(&uniform);
    let g_moderate = gini_coefficient(&moderate);
    let g_extreme = gini_coefficient(&extreme);

    assert!(g_uniform < g_moderate);
    assert!(g_moderate < g_extreme);
}

#[test]
fn round_f64_chained_with_safe_ratio_produces_stable_output() {
    let r1 = safe_ratio(7, 13);
    let r2 = safe_ratio(7, 13);
    let rounded1 = round_f64(r1 * 100.0, 2);
    let rounded2 = round_f64(r2 * 100.0, 2);
    assert_eq!(rounded1, rounded2);
}

#[test]
fn all_percentile_milestones_are_monotonic() {
    let values = [1usize, 3, 5, 7, 9, 11, 13, 15, 17, 19];
    let pcts = [0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0];
    let results: Vec<f64> = pcts.iter().map(|p| percentile(&values, *p)).collect();
    for w in results.windows(2) {
        assert!(
            w[0] <= w[1],
            "percentile not monotonic: {} > {}",
            w[0],
            w[1]
        );
    }
}

#[test]
fn large_values_do_not_overflow_safe_ratio() {
    let got = safe_ratio(usize::MAX / 2, usize::MAX);
    assert!(got > 0.0);
    assert!(got < 1.0);
}

#[test]
fn gini_and_percentile_agree_on_equality_for_uniform_input() {
    let values = [50usize, 50, 50, 50, 50];
    assert!(gini_coefficient(&values).abs() < 1e-10);
    assert_eq!(percentile(&values, 0.0), percentile(&values, 1.0));
}
