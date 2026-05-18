//! Ratchet policy aggregation.

use super::evaluate::evaluate_ratchet_with_options;
use crate::types::{RatchetConfig, RatchetGateResult, RuleLevel};
use serde_json::Value;

/// Evaluate all ratchet rules against baseline and current receipts.
///
/// # Arguments
/// * `config` - The ratchet configuration with rules
/// * `baseline` - The baseline JSON receipt
/// * `current` - The current JSON receipt
///
/// # Returns
/// A `RatchetGateResult` with overall pass/fail and individual results.
pub fn evaluate_ratchet_policy(
    config: &RatchetConfig,
    baseline: &Value,
    current: &Value,
) -> RatchetGateResult {
    let mut ratchet_results = Vec::with_capacity(config.rules.len());

    for rule in &config.rules {
        let result = evaluate_ratchet_with_options(
            rule,
            baseline,
            current,
            config.allow_missing_baseline,
            config.allow_missing_current,
        );

        let failed_error = !result.passed && result.rule.level == RuleLevel::Error;
        ratchet_results.push(result);

        if config.fail_fast && failed_error {
            break;
        }
    }

    RatchetGateResult::from_results(ratchet_results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{RatchetRule, RuleLevel};
    use serde_json::json;

    fn make_rule(
        pointer: &str,
        max_increase_pct: Option<f64>,
        max_value: Option<f64>,
        level: RuleLevel,
    ) -> RatchetRule {
        RatchetRule {
            pointer: pointer.to_string(),
            max_increase_pct,
            max_value,
            level,
            description: None,
        }
    }

    fn make_config(rules: Vec<RatchetRule>, fail_fast: bool) -> RatchetConfig {
        RatchetConfig {
            rules,
            fail_fast,
            allow_missing_baseline: false,
            allow_missing_current: false,
        }
    }

    #[test]
    fn empty_rules_passes_with_no_results() {
        let config = make_config(vec![], false);
        let result = evaluate_ratchet_policy(&config, &json!({}), &json!({}));
        assert!(result.passed);
        assert!(result.ratchet_results.is_empty());
        assert_eq!(result.errors, 0);
        assert_eq!(result.warnings, 0);
    }

    #[test]
    fn all_rules_pass_when_within_thresholds() {
        let baseline = json!({"a": 10.0, "b": 100.0});
        let current = json!({"a": 10.0, "b": 105.0});
        let config = make_config(
            vec![
                make_rule("/a", Some(10.0), None, RuleLevel::Error),
                make_rule("/b", Some(10.0), None, RuleLevel::Error),
            ],
            false,
        );

        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(result.passed);
        assert_eq!(result.ratchet_results.len(), 2);
        assert_eq!(result.errors, 0);
        assert_eq!(result.warnings, 0);
    }

    #[test]
    fn any_error_fails_overall_when_mixed_with_passes() {
        let baseline = json!({"a": 10.0, "b": 100.0});
        let current = json!({"a": 10.0, "b": 200.0}); // /b is 100% over
        let config = make_config(
            vec![
                make_rule("/a", Some(10.0), None, RuleLevel::Error),
                make_rule("/b", Some(10.0), None, RuleLevel::Error),
            ],
            false,
        );

        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(!result.passed);
        assert_eq!(result.errors, 1);
        assert_eq!(result.ratchet_results.len(), 2);
    }

    #[test]
    fn warn_level_failures_do_not_fail_overall() {
        let baseline = json!({"a": 10.0});
        let current = json!({"a": 20.0});
        let config = make_config(
            vec![make_rule("/a", Some(10.0), None, RuleLevel::Warn)],
            false,
        );

        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(result.passed); // warn-only failures don't fail the gate
        assert_eq!(result.warnings, 1);
        assert_eq!(result.errors, 0);
    }

    #[test]
    fn fail_fast_stops_after_first_error_level_failure() {
        let baseline = json!({"a": 10.0, "b": 10.0, "c": 10.0});
        let current = json!({"a": 20.0, "b": 20.0, "c": 20.0});
        let config = make_config(
            vec![
                make_rule("/a", Some(10.0), None, RuleLevel::Error),
                make_rule("/b", Some(10.0), None, RuleLevel::Error),
                make_rule("/c", Some(10.0), None, RuleLevel::Error),
            ],
            true,
        );

        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(!result.passed);
        assert_eq!(
            result.ratchet_results.len(),
            1,
            "fail_fast should stop after first error"
        );
        assert_eq!(result.errors, 1);
    }

    #[test]
    fn fail_fast_does_not_stop_on_warn_failure() {
        // A failing warn rule should NOT trigger early exit because it's not error-level.
        let baseline = json!({"a": 10.0, "b": 10.0});
        let current = json!({"a": 20.0, "b": 20.0});
        let config = make_config(
            vec![
                make_rule("/a", Some(10.0), None, RuleLevel::Warn), // fails as warn
                make_rule("/b", Some(10.0), None, RuleLevel::Error), // fails as error
            ],
            true,
        );

        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(!result.passed);
        assert_eq!(
            result.ratchet_results.len(),
            2,
            "fail_fast must continue past warn-level failures"
        );
        assert_eq!(result.warnings, 1);
        assert_eq!(result.errors, 1);
    }

    #[test]
    fn fail_fast_does_not_stop_on_pass() {
        let baseline = json!({"a": 10.0, "b": 10.0});
        let current = json!({"a": 10.0, "b": 20.0}); // /a passes, /b fails
        let config = make_config(
            vec![
                make_rule("/a", Some(10.0), None, RuleLevel::Error),
                make_rule("/b", Some(10.0), None, RuleLevel::Error),
            ],
            true,
        );

        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(!result.passed);
        assert_eq!(
            result.ratchet_results.len(),
            2,
            "fail_fast must not stop after a passing rule"
        );
    }

    #[test]
    fn allow_missing_baseline_is_forwarded_to_evaluation() {
        let baseline = json!({}); // no /value
        let current = json!({"value": 5.0});
        let config = RatchetConfig {
            rules: vec![make_rule("/value", Some(10.0), None, RuleLevel::Error)],
            fail_fast: false,
            allow_missing_baseline: true,
            allow_missing_current: false,
        };

        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(result.passed);
        assert_eq!(result.errors, 0);
    }

    #[test]
    fn allow_missing_current_is_forwarded_to_evaluation() {
        let baseline = json!({"value": 5.0});
        let current = json!({}); // no /value
        let config = RatchetConfig {
            rules: vec![make_rule("/value", Some(10.0), None, RuleLevel::Error)],
            fail_fast: false,
            allow_missing_baseline: false,
            allow_missing_current: true,
        };

        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(result.passed);
        assert_eq!(result.errors, 0);
    }

    #[test]
    fn results_preserve_rule_order() {
        let baseline = json!({"a": 10.0, "b": 10.0, "c": 10.0});
        let current = json!({"a": 10.0, "b": 10.0, "c": 10.0});
        let config = make_config(
            vec![
                make_rule("/a", Some(10.0), None, RuleLevel::Error),
                make_rule("/b", Some(10.0), None, RuleLevel::Error),
                make_rule("/c", Some(10.0), None, RuleLevel::Error),
            ],
            false,
        );

        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert_eq!(result.ratchet_results.len(), 3);
        assert_eq!(result.ratchet_results[0].rule.pointer, "/a");
        assert_eq!(result.ratchet_results[1].rule.pointer, "/b");
        assert_eq!(result.ratchet_results[2].rule.pointer, "/c");
    }
}
