use tokmd_types::cockpit::TrendDirection;

/// Format a float with a sign prefix.
pub fn format_signed_f64(value: f64) -> String {
    if value > 0.0 {
        format!("+{value:.2}")
    } else {
        format!("{value:.2}")
    }
}

/// Human-readable label for a trend direction.
pub fn trend_direction_label(direction: TrendDirection) -> &'static str {
    match direction {
        TrendDirection::Improving => "improving",
        TrendDirection::Stable => "stable",
        TrendDirection::Degrading => "degrading",
    }
}

/// Render a sparkline string from a slice of values.
pub fn sparkline(values: &[f64]) -> String {
    if values.is_empty() {
        return String::new();
    }

    const BARS: &[char] = &[
        '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}',
        '\u{2588}',
    ];
    let min = values
        .iter()
        .copied()
        .fold(f64::INFINITY, |acc, v| acc.min(v));
    let max = values
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, |acc, v| acc.max(v));

    if !min.is_finite() || !max.is_finite() {
        return String::new();
    }

    if (max - min).abs() < f64::EPSILON {
        return std::iter::repeat_n(BARS[3], values.len()).collect();
    }

    let span = max - min;
    values
        .iter()
        .map(|v| {
            let norm = ((v - min) / span).clamp(0.0, 1.0);
            let idx = (norm * (BARS.len() as f64 - 1.0)).round() as usize;
            BARS[idx]
        })
        .collect()
}

/// Return the current time as an ISO 8601 string.
pub fn now_iso8601() -> String {
    let now = time::OffsetDateTime::now_utc();
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        now.year(),
        now.month() as u8,
        now.day(),
        now.hour(),
        now.minute(),
        now.second(),
    )
}

/// Round a float to two decimal places.
pub fn round_pct(val: f64) -> f64 {
    (val * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_pct_basic() {
        assert_eq!(round_pct(0.123456), 0.12);
        assert_eq!(round_pct(0.999), 1.0);
        assert_eq!(round_pct(0.0), 0.0);
    }

    #[test]
    fn round_pct_rounding_up() {
        assert_eq!(round_pct(0.125), 0.13);
    }

    #[test]
    fn round_pct_negative() {
        assert_eq!(round_pct(-0.567), -0.57);
    }

    #[test]
    fn signed_float_formatting_marks_positive_values() {
        assert_eq!(format_signed_f64(5.0), "+5.00");
        assert_eq!(format_signed_f64(0.5), "+0.50");
        assert_eq!(format_signed_f64(-2.50), "-2.50");
        assert_eq!(format_signed_f64(0.0), "0.00");
    }

    #[test]
    fn trend_direction_labels_are_stable() {
        assert_eq!(
            trend_direction_label(TrendDirection::Improving),
            "improving"
        );
        assert_eq!(trend_direction_label(TrendDirection::Stable), "stable");
        assert_eq!(
            trend_direction_label(TrendDirection::Degrading),
            "degrading"
        );
    }

    #[test]
    fn sparkline_handles_empty_and_single_value_inputs() {
        assert_eq!(sparkline(&[]), "");
        assert_eq!(sparkline(&[5.0]).chars().count(), 1);
    }

    #[test]
    fn sparkline_scales_ascending_values() {
        let result = sparkline(&[0.0, 25.0, 50.0, 75.0, 100.0]);
        assert_eq!(result.chars().count(), 5);
        let chars: Vec<char> = result.chars().collect();
        assert_eq!(chars[0], '\u{2581}');
        assert_eq!(chars[4], '\u{2588}');
    }

    #[test]
    fn sparkline_constant_values_use_same_bar() {
        let result = sparkline(&[42.0, 42.0, 42.0]);
        assert_eq!(result.chars().count(), 3);
        let chars: Vec<char> = result.chars().collect();
        assert_eq!(chars[0], chars[1]);
        assert_eq!(chars[1], chars[2]);
    }

    #[test]
    fn now_iso8601_shape_is_stable() {
        let ts = now_iso8601();
        assert!(ts.ends_with('Z'));
        assert!(ts.contains('T'));
        assert_eq!(ts.len(), 20);
    }
}
