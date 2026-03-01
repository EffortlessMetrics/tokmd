use proptest::prelude::*;
use tokmd_math::{gini_coefficient, percentile, round_f64, safe_ratio};

proptest! {
    #[test]
    fn percentile_empty_is_zero(pct in 0.0f64..=1.0) {
        prop_assert_eq!(percentile(&[], pct), 0.0);
    }

    #[test]
    fn percentile_is_within_bounds(mut values in prop::collection::vec(0usize..10000, 1..100),
                                   pct in 0.0f64..=1.0) {
        values.sort();
        let got = percentile(&values, pct);
        prop_assert!(got >= *values.first().unwrap() as f64);
        prop_assert!(got <= *values.last().unwrap() as f64);
    }

    #[test]
    fn percentile_is_monotonic(mut values in prop::collection::vec(0usize..10000, 2..100),
                               pct1 in 0.0f64..=1.0,
                               pct2 in 0.0f64..=1.0) {
        values.sort();
        let p1 = percentile(&values, pct1);
        let p2 = percentile(&values, pct2);
        if pct1 <= pct2 {
            prop_assert!(p1 <= p2);
        } else {
            prop_assert!(p1 >= p2);
        }
    }

    #[test]
    fn gini_empty_is_zero(_dummy in 0u8..1) {
        prop_assert_eq!(gini_coefficient(&[]), 0.0);
    }

    #[test]
    fn gini_is_bounded(values in prop::collection::vec(0usize..1000, 1..100)) {
        let mut sorted = values;
        sorted.sort();
        let gini = gini_coefficient(&sorted);
        prop_assert!(gini >= 0.0);
        prop_assert!(gini <= 1.0);
    }

    #[test]
    fn gini_uniform_is_near_zero(value in 1usize..1000, len in 2usize..100) {
        let values = vec![value; len];
        let gini = gini_coefficient(&values);
        prop_assert!(gini.abs() < 0.0001);
    }

    #[test]
    fn safe_ratio_zero_denominator_is_zero(numer in 0usize..10000) {
        prop_assert_eq!(safe_ratio(numer, 0), 0.0);
    }

    #[test]
    fn safe_ratio_identity_is_one(value in 1usize..10000) {
        prop_assert_eq!(safe_ratio(value, value), 1.0);
    }

    #[test]
    fn round_f64_is_idempotent(value in -1000.0f64..1000.0, decimals in 0u32..8) {
        let once = round_f64(value, decimals);
        let twice = round_f64(once, decimals);
        prop_assert!((once - twice).abs() < 1e-10);
    }

    #[test]
    fn round_f64_preserves_integers(value in -1000i64..1000, decimals in 0u32..8) {
        let f = value as f64;
        prop_assert_eq!(round_f64(f, decimals), f);
    }
}
