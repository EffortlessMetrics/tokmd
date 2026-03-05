//! Wave-59 property-based tests for tokmd-math — statistical bounds,
//! sorting invariants, and algebraic properties.

use proptest::prelude::*;
use tokmd_math::{gini_coefficient, percentile, round_f64, safe_ratio};

// ── round_f64 properties ─────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn round_idempotent(val in -1e6f64..1e6, dec in 0u32..10) {
        let once = round_f64(val, dec);
        let twice = round_f64(once, dec);
        prop_assert!(
            (once - twice).abs() < f64::EPSILON,
            "round_f64 must be idempotent: once={once}, twice={twice}"
        );
    }

    #[test]
    fn round_preserves_finiteness(val in -1e15f64..1e15, dec in 0u32..10) {
        let r = round_f64(val, dec);
        prop_assert!(r.is_finite(), "round_f64 of finite input should be finite");
    }

    #[test]
    fn round_zero_decimals_is_integer(val in -1e6f64..1e6) {
        let r = round_f64(val, 0);
        prop_assert_eq!(r, r.round(), "zero-decimal rounding should be integer");
    }

    // ── safe_ratio properties ────────────────────────────────────────────

    #[test]
    fn safe_ratio_non_negative(n in 0usize..1_000_000, d in 0usize..1_000_000) {
        let r = safe_ratio(n, d);
        prop_assert!(r >= 0.0, "safe_ratio must be non-negative, got {r}");
    }

    #[test]
    fn safe_ratio_finite(n in 0usize..=usize::MAX, d in 0usize..=usize::MAX) {
        let r = safe_ratio(n, d);
        prop_assert!(r.is_finite(), "safe_ratio must always be finite");
    }

    #[test]
    fn safe_ratio_zero_denom_always_zero(n in 0usize..1_000_000) {
        prop_assert_eq!(safe_ratio(n, 0), 0.0);
    }

    // ── percentile properties ────────────────────────────────────────────

    #[test]
    fn percentile_bounded_by_min_max(
        mut data in prop::collection::vec(0usize..10_000, 1..100),
        pct in 0.0f64..=1.0,
    ) {
        data.sort();
        let p = percentile(&data, pct);
        let min = *data.first().unwrap() as f64;
        let max = *data.last().unwrap() as f64;
        prop_assert!(p >= min, "percentile {p} < min {min}");
        prop_assert!(p <= max, "percentile {p} > max {max}");
    }

    #[test]
    fn percentile_zero_is_minimum(
        mut data in prop::collection::vec(0usize..10_000, 1..50),
    ) {
        data.sort();
        let p = percentile(&data, 0.0);
        prop_assert_eq!(p, *data.first().unwrap() as f64);
    }

    #[test]
    fn percentile_one_is_maximum(
        mut data in prop::collection::vec(0usize..10_000, 1..50),
    ) {
        data.sort();
        let p = percentile(&data, 1.0);
        prop_assert_eq!(p, *data.last().unwrap() as f64);
    }

    #[test]
    fn percentile_monotonic(
        mut data in prop::collection::vec(0usize..10_000, 2..50),
    ) {
        data.sort();
        let mut prev = percentile(&data, 0.0);
        for i in 1..=10 {
            let pct = i as f64 / 10.0;
            let cur = percentile(&data, pct);
            prop_assert!(cur >= prev, "percentile not monotonic: p({})={} < p({})={}", pct, cur, (i-1) as f64 / 10.0, prev);
            prev = cur;
        }
    }

    // ── gini_coefficient properties ──────────────────────────────────────

    #[test]
    fn gini_bounded(
        mut data in prop::collection::vec(0usize..10_000, 1..100),
    ) {
        data.sort();
        let g = gini_coefficient(&data);
        prop_assert!(g >= 0.0, "gini must be >= 0, got {g}");
        prop_assert!(g <= 1.0, "gini must be <= 1, got {g}");
    }

    #[test]
    fn gini_uniform_is_zero(val in 1usize..10_000, len in 2usize..50) {
        let data = vec![val; len];
        let g = gini_coefficient(&data);
        prop_assert!(
            g.abs() < 1e-10,
            "uniform distribution should have gini ≈ 0, got {g}"
        );
    }

    #[test]
    fn gini_scale_invariant(
        mut data in prop::collection::vec(1usize..1_000, 2..30),
        factor in 1usize..100,
    ) {
        data.sort();
        let g1 = gini_coefficient(&data);
        let scaled: Vec<usize> = data.iter().map(|v| v * factor).collect();
        let g2 = gini_coefficient(&scaled);
        prop_assert!(
            (g1 - g2).abs() < 1e-10,
            "gini should be scale-invariant: g1={g1}, g2={g2}"
        );
    }

    #[test]
    fn gini_deterministic(
        mut data in prop::collection::vec(0usize..5_000, 1..50),
    ) {
        data.sort();
        let g1 = gini_coefficient(&data);
        let g2 = gini_coefficient(&data);
        prop_assert_eq!(g1.to_bits(), g2.to_bits(), "gini must be deterministic");
    }
}
