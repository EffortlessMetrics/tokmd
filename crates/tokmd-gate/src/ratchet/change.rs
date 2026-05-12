//! Ratchet percentage-change calculation.

pub(super) fn percentage_change(baseline_value: Option<f64>, current_value: f64) -> Option<f64> {
    baseline_value.map(|baseline| {
        if baseline == 0.0 {
            if current_value == 0.0 {
                0.0
            } else {
                f64::INFINITY
            }
        } else {
            ((current_value - baseline) / baseline) * 100.0
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_baseline_has_no_percentage_change() {
        assert_eq!(percentage_change(None, 10.0), None);
    }

    #[test]
    fn zero_to_zero_is_no_change() {
        assert_eq!(percentage_change(Some(0.0), 0.0), Some(0.0));
    }

    #[test]
    fn zero_to_nonzero_is_unbounded_increase() {
        let pct = percentage_change(Some(0.0), 1.0).expect("percentage");
        assert!(pct.is_infinite());
        assert!(pct.is_sign_positive());
    }

    #[test]
    fn nonzero_baseline_reports_percent_delta() {
        assert_eq!(percentage_change(Some(10.0), 12.5), Some(25.0));
        assert_eq!(percentage_change(Some(10.0), 7.5), Some(-25.0));
    }
}
