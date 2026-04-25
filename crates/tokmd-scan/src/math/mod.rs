//! Deterministic numeric and statistical helpers.

#![forbid(unsafe_code)]

/// Round a floating point value to `decimals` decimal places.
///
/// # Examples
///
/// ```
/// use tokmd_scan::round_f64;
///
/// assert_eq!(round_f64(12.34567, 2), 12.35);
/// assert_eq!(round_f64(12.34567, 4), 12.3457);
/// assert_eq!(round_f64(1.5, 0), 2.0);
/// ```
///
/// Rounding at zero decimal places:
///
/// ```
/// use tokmd_scan::round_f64;
///
/// assert_eq!(round_f64(2.4, 0), 2.0);
/// assert_eq!(round_f64(2.6, 0), 3.0);
/// ```
#[must_use]
pub fn round_f64(value: f64, decimals: u32) -> f64 {
    let factor = 10f64.powi(decimals as i32);
    (value * factor).round() / factor
}

/// Return a 4-decimal ratio and guard division by zero.
///
/// # Examples
///
/// ```
/// use tokmd_scan::safe_ratio;
///
/// assert_eq!(safe_ratio(1, 4), 0.25);
/// assert_eq!(safe_ratio(5, 0), 0.0); // division by zero returns 0
/// ```
///
/// Fractional ratios are rounded to four decimal places:
///
/// ```
/// use tokmd_scan::safe_ratio;
///
/// assert_eq!(safe_ratio(1, 3), 0.3333);
/// assert_eq!(safe_ratio(2, 3), 0.6667);
/// ```
#[must_use]
pub fn safe_ratio(numer: usize, denom: usize) -> f64 {
    if denom == 0 {
        0.0
    } else {
        round_f64(numer as f64 / denom as f64, 4)
    }
}

/// Return the `pct` percentile from an ascending-sorted integer slice.
///
/// # Examples
///
/// ```
/// use tokmd_scan::percentile;
///
/// let values = [10, 20, 30, 40, 50];
/// assert_eq!(percentile(&values, 0.0), 10.0);
/// assert_eq!(percentile(&values, 0.9), 50.0);
/// assert_eq!(percentile(&[], 0.5), 0.0); // empty slice returns 0
/// ```
///
/// Computing the median:
///
/// ```
/// use tokmd_scan::percentile;
///
/// let data = [1, 2, 3, 4, 5];
/// assert_eq!(percentile(&data, 0.5), 3.0);
/// ```
#[must_use]
pub fn percentile(sorted: &[usize], pct: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = (pct * (sorted.len() as f64 - 1.0)).ceil() as usize;
    sorted[idx.min(sorted.len() - 1)] as f64
}

/// Return the Gini coefficient for an ascending-sorted integer slice.
///
/// # Examples
///
/// ```
/// use tokmd_scan::gini_coefficient;
///
/// // Perfectly equal distribution has a Gini of 0
/// assert!((gini_coefficient(&[5, 5, 5, 5]) - 0.0).abs() < 1e-10);
///
/// // Empty slice returns 0
/// assert_eq!(gini_coefficient(&[]), 0.0);
///
/// // Unequal distribution produces a positive Gini
/// assert!(gini_coefficient(&[1, 1, 1, 100]) > 0.0);
/// ```
///
/// Single-element and all-zero slices:
///
/// ```
/// use tokmd_scan::gini_coefficient;
///
/// // A single element has zero inequality
/// assert_eq!(gini_coefficient(&[42]), 0.0);
///
/// // All zeros also return 0 (no division by zero)
/// assert_eq!(gini_coefficient(&[0, 0, 0]), 0.0);
/// ```
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
