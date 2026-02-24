//! Deterministic numeric and statistical helpers.

#![forbid(unsafe_code)]

/// Round a floating point value to `decimals` decimal places.
#[must_use]
pub fn round_f64(value: f64, decimals: u32) -> f64 {
    let factor = 10f64.powi(decimals as i32);
    (value * factor).round() / factor
}

/// Return a 4-decimal ratio and guard division by zero.
#[must_use]
pub fn safe_ratio(numer: usize, denom: usize) -> f64 {
    if denom == 0 {
        0.0
    } else {
        round_f64(numer as f64 / denom as f64, 4)
    }
}

/// Return the `pct` percentile from an ascending-sorted integer slice.
#[must_use]
pub fn percentile(sorted: &[usize], pct: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = (pct * (sorted.len() as f64 - 1.0)).ceil() as usize;
    sorted[idx.min(sorted.len() - 1)] as f64
}

/// Return the Gini coefficient for an ascending-sorted integer slice.
#[must_use]
pub fn gini_coefficient(sorted: &[usize]) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let n = sorted.len() as f64;
    let sum: f64 = sorted.iter().map(|v| *v as f64).sum();
    if sum == 0.0 {
        return 0.0;
    }
    let mut accum = 0.0;
    for (i, value) in sorted.iter().enumerate() {
        let i = i as f64 + 1.0;
        accum += (2.0 * i - n - 1.0) * (*value as f64);
    }
    accum / (n * sum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_f64_rounds_expected_precision() {
        // Avoid PI-like literals: Nix clippy denies clippy::approx_constant and
        // lints test targets.
        let value = 12.34567;
        assert_eq!(round_f64(value, 2), 12.35);
        assert_eq!(round_f64(value, 4), 12.3457);
    }

    #[test]
    fn safe_ratio_guards_divide_by_zero() {
        assert_eq!(safe_ratio(5, 0), 0.0);
        assert_eq!(safe_ratio(1, 4), 0.25);
    }

    #[test]
    fn percentile_returns_expected_values() {
        let values = [10usize, 20, 30, 40, 50];
        assert_eq!(percentile(&values, 0.0), 10.0);
        assert_eq!(percentile(&values, 0.9), 50.0);
    }

    #[test]
    fn gini_coefficient_handles_empty_and_uniform() {
        assert_eq!(gini_coefficient(&[]), 0.0);
        assert!((gini_coefficient(&[5, 5, 5, 5]) - 0.0).abs() < 1e-10);
    }
}
