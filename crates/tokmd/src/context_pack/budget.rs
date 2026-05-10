//! Budget parsing for context and handoff token limits.

/// Parse a budget string with optional k/m/g suffix into token count.
///
/// Accepts:
/// - Plain numbers: "50000"
/// - Suffix `k` (x1,000): "128k", "1.5k"
/// - Suffix `m` (x1,000,000): "1m", "0.5m"
/// - Suffix `g` (x1,000,000,000): "1g", "0.5g"
/// - Keywords `unlimited` or `max`: returns `usize::MAX`
pub fn parse_budget(budget: &str) -> anyhow::Result<usize> {
    let input = budget.trim().to_lowercase();

    if input == "unlimited" || input == "max" {
        return Ok(usize::MAX);
    }

    let (num_str, multiplier) = if let Some(num) = input.strip_suffix('k') {
        (num.trim(), 1_000.0)
    } else if let Some(num) = input.strip_suffix('m') {
        (num.trim(), 1_000_000.0)
    } else if let Some(num) = input.strip_suffix('g') {
        (num.trim(), 1_000_000_000.0)
    } else {
        (input.as_str(), 1.0)
    };

    let n: f64 = num_str.parse().map_err(|_| {
        anyhow::anyhow!(
            "Invalid budget '{}': expected <number>[k|m|g] or 'unlimited' (examples: 128k, 1m, 1g, unlimited)",
            budget.trim()
        )
    })?;

    let result = n * multiplier;
    if !result.is_finite() || result < 0.0 {
        anyhow::bail!(
            "Invalid budget '{}': value must be a finite non-negative number",
            budget.trim()
        );
    }
    if result > usize::MAX as f64 {
        anyhow::bail!(
            "Invalid budget '{}': value overflows (max is {})",
            budget.trim(),
            usize::MAX
        );
    }

    Ok(result as usize)
}

#[cfg(test)]
mod tests {
    use super::parse_budget;

    #[test]
    fn parses_common_suffixes() {
        assert_eq!(
            parse_budget("128k").expect("failed to parse valid budget string '128k'"),
            128_000
        );
        assert_eq!(
            parse_budget("1m").expect("failed to parse valid budget string '1m'"),
            1_000_000
        );
        assert_eq!(
            parse_budget("50000").expect("failed to parse valid budget string '50000'"),
            50_000
        );
        assert_eq!(
            parse_budget("1.5k").expect("failed to parse valid budget string '1.5k'"),
            1_500
        );
    }

    #[test]
    fn parses_g_suffix() {
        assert_eq!(
            parse_budget("1g").expect("failed to parse valid budget string '1g'"),
            1_000_000_000
        );
        assert_eq!(
            parse_budget("0.5g").expect("failed to parse valid budget string '0.5g'"),
            500_000_000
        );
        assert_eq!(
            parse_budget("2G").expect("failed to parse valid budget string '2G'"),
            2_000_000_000
        );
    }

    #[test]
    fn parses_unlimited_keywords() {
        assert_eq!(
            parse_budget("unlimited").expect("failed to parse valid budget string 'unlimited'"),
            usize::MAX
        );
        assert_eq!(
            parse_budget("max").expect("failed to parse valid budget string 'max'"),
            usize::MAX
        );
        assert_eq!(
            parse_budget("UNLIMITED").expect("failed to parse valid budget string 'UNLIMITED'"),
            usize::MAX
        );
        assert_eq!(
            parse_budget("MAX").expect("failed to parse valid budget string 'MAX'"),
            usize::MAX
        );
        assert_eq!(
            parse_budget("  unlimited  ")
                .expect("failed to parse padded budget string '  unlimited  '"),
            usize::MAX
        );
    }

    #[test]
    fn trims_whitespace() {
        assert_eq!(
            parse_budget("  10k  ").expect("failed to parse padded budget string '  10k  '"),
            10_000
        );
        assert_eq!(
            parse_budget(" 5m ").expect("failed to parse padded budget string ' 5m '"),
            5_000_000
        );
    }

    #[test]
    fn suffixes_are_case_insensitive() {
        assert_eq!(
            parse_budget("10K").expect("failed to parse valid budget string '10K'"),
            10_000
        );
        assert_eq!(
            parse_budget("2M").expect("failed to parse valid budget string '2M'"),
            2_000_000
        );
    }

    #[test]
    fn suffixes_multiply() {
        assert_eq!(
            parse_budget("2k").expect("failed to parse valid budget string '2k'"),
            2_000
        );
        assert_eq!(
            parse_budget("0.5k").expect("failed to parse valid budget string '0.5k'"),
            500
        );
        assert_eq!(
            parse_budget("2m").expect("failed to parse valid budget string '2m'"),
            2_000_000
        );
        assert_eq!(
            parse_budget("0.5m").expect("failed to parse valid budget string '0.5m'"),
            500_000
        );
    }

    #[test]
    fn rejects_alpha_input_with_guidance() {
        let err = parse_budget("abc").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("Invalid budget"),
            "Expected guidance message, got: {msg}"
        );
        assert!(
            msg.contains("128k"),
            "Expected example in guidance, got: {msg}"
        );
    }

    #[test]
    fn rejects_invalid_suffix_with_guidance() {
        let err = parse_budget("1x").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("Invalid budget"),
            "Expected guidance message, got: {msg}"
        );
    }

    #[test]
    fn rejects_empty_input() {
        assert!(parse_budget("").is_err());
    }

    #[test]
    fn rejects_suffix_only_input() {
        assert!(parse_budget("k").is_err());
        assert!(parse_budget("m").is_err());
        assert!(parse_budget("g").is_err());
    }

    #[test]
    fn rejects_negative_values() {
        assert!(parse_budget("-1").is_err());
        assert!(parse_budget("-1k").is_err());
    }

    #[test]
    fn rejects_non_finite_values() {
        assert!(parse_budget("NaN").is_err());
        assert!(parse_budget("inf").is_err());
        assert!(parse_budget("-inf").is_err());
    }

    #[test]
    fn rejects_overflow() {
        let err = parse_budget("999999999999g").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("overflows"),
            "Expected overflow message, got: {msg}"
        );
    }
}
