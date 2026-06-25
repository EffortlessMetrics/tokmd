//! Shared clap `value_parser` validators for numeric flags.
//!
//! These keep out-of-range numeric input from silently degenerating a command's
//! behavior. For example, `--max-file-pct 0` drives the per-file token cap to
//! zero (excluding every file), and `--max-commits 0` scans no git history at
//! all. Validating at parse time turns those silent footguns into a clear,
//! actionable error before any work begins.

/// Parse a budget fraction in the half-open range `(0.0, 1.0]`.
///
/// Used by `--max-file-pct`. A non-finite value, a value at or below `0.0`, or a
/// value above `1.0` is rejected so the caller learns immediately instead of
/// receiving an empty or nonsensical selection.
pub(crate) fn budget_fraction(raw: &str) -> Result<f64, String> {
    let value: f64 = raw
        .parse()
        .map_err(|_| format!("`{raw}` is not a valid number"))?;
    if !value.is_finite() || value <= 0.0 || value > 1.0 {
        return Err(format!(
            "`{raw}` is out of range; expected a fraction greater than 0.0 and at most 1.0"
        ));
    }
    Ok(value)
}

/// Parse a count that must be at least `1`.
///
/// Used by flags such as `--max-commits`, `--max-commit-files`, and
/// `--max-file-tokens`, where `0` produces a degenerate result (no commits
/// scanned, or every file excluded by a zero cap).
pub(crate) fn positive_usize(raw: &str) -> Result<usize, String> {
    let value: usize = raw
        .parse()
        .map_err(|_| format!("`{raw}` is not a valid whole number"))?;
    if value == 0 {
        return Err("value must be at least 1".to_string());
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_fraction_accepts_in_range() {
        assert_eq!(budget_fraction("0.15").unwrap(), 0.15);
        assert_eq!(budget_fraction("1.0").unwrap(), 1.0);
        // A vanishingly small positive value is still in range.
        assert!(budget_fraction("0.0001").is_ok());
    }

    #[test]
    fn budget_fraction_rejects_out_of_range() {
        assert!(budget_fraction("0").is_err());
        assert!(budget_fraction("0.0").is_err());
        assert!(budget_fraction("-0.5").is_err());
        assert!(budget_fraction("1.5").is_err());
        assert!(budget_fraction("10").is_err());
        assert!(budget_fraction("NaN").is_err());
        assert!(budget_fraction("inf").is_err());
        assert!(budget_fraction("abc").is_err());
    }

    #[test]
    fn positive_usize_accepts_one_and_above() {
        assert_eq!(positive_usize("1").unwrap(), 1);
        assert_eq!(positive_usize("1000").unwrap(), 1000);
    }

    #[test]
    fn positive_usize_rejects_zero_and_invalid() {
        assert!(positive_usize("0").is_err());
        assert!(positive_usize("-1").is_err());
        assert!(positive_usize("abc").is_err());
    }
}
