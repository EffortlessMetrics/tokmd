//! Single-rule ratchet evaluation.

use super::change::percentage_change;
use crate::numeric::value_to_f64;
use crate::pointer::resolve_pointer;
use crate::types::{RatchetResult, RatchetRule};
use serde_json::Value;

/// Evaluate a single ratchet rule against baseline and current values.
///
/// This is a convenience wrapper around [`evaluate_ratchet_with_options`] with
/// strict missing value handling (fails if baseline or current values are missing).
///
/// # Arguments
/// * `rule` - The ratchet rule to evaluate
/// * `baseline` - The baseline JSON receipt to compare against
/// * `current` - The current JSON receipt to check
///
/// # Returns
/// A `RatchetResult` with pass/fail status and detailed information.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn evaluate_ratchet(
    rule: &RatchetRule,
    baseline: &Value,
    current: &Value,
) -> RatchetResult {
    evaluate_ratchet_with_options(rule, baseline, current, false, false)
}

/// Evaluate a ratchet rule with configurable missing value handling.
///
/// # Arguments
/// * `rule` - The ratchet rule to evaluate
/// * `baseline` - The baseline JSON receipt
/// * `current` - The current JSON receipt
/// * `allow_missing_baseline` - Treat missing baseline as pass
/// * `allow_missing_current` - Treat missing current value as pass
///
/// # Returns
/// A `RatchetResult` with pass/fail status.
pub(super) fn evaluate_ratchet_with_options(
    rule: &RatchetRule,
    baseline: &Value,
    current: &Value,
    allow_missing_baseline: bool,
    allow_missing_current: bool,
) -> RatchetResult {
    let baseline_resolved = resolve_pointer(baseline, &rule.pointer);
    let current_resolved = resolve_pointer(current, &rule.pointer);

    // Handle missing current value.
    let current_value = match current_resolved.and_then(value_to_f64) {
        Some(v) => v,
        None => {
            if allow_missing_current {
                return RatchetResult {
                    rule: rule.clone(),
                    passed: true,
                    baseline_value: baseline_resolved.and_then(value_to_f64),
                    current_value: f64::NAN,
                    change_pct: None,
                    message: format!(
                        "Current value not found at pointer '{}' (allowed)",
                        rule.pointer
                    ),
                };
            } else {
                return RatchetResult {
                    rule: rule.clone(),
                    passed: false,
                    baseline_value: baseline_resolved.and_then(value_to_f64),
                    current_value: f64::NAN,
                    change_pct: None,
                    message: format!(
                        "Current value not found or not numeric at pointer '{}'",
                        rule.pointer
                    ),
                };
            }
        }
    };

    // Extract baseline value.
    let baseline_value = baseline_resolved.and_then(value_to_f64);

    let change_pct = percentage_change(baseline_value, current_value);

    // Check max_value constraint (absolute ceiling).
    if let Some(max_val) = rule.max_value
        && current_value > max_val
    {
        return RatchetResult {
            rule: rule.clone(),
            passed: false,
            baseline_value,
            current_value,
            change_pct,
            message: format!(
                "Current value {} exceeds maximum allowed value {}",
                current_value, max_val
            ),
        };
    }

    // Check max_increase_pct constraint (relative to baseline).
    if let Some(max_inc_pct) = rule.max_increase_pct {
        match baseline_value {
            Some(_bv) => {
                let pct = change_pct.unwrap_or(0.0);
                if pct > max_inc_pct {
                    return RatchetResult {
                        rule: rule.clone(),
                        passed: false,
                        baseline_value,
                        current_value,
                        change_pct,
                        message: format!(
                            "Increase of {:.2}% exceeds maximum allowed increase of {:.2}%",
                            pct, max_inc_pct
                        ),
                    };
                }
            }
            None => {
                // No baseline value - can't evaluate percentage increase.
                if allow_missing_baseline {
                    let desc = rule
                        .description
                        .as_deref()
                        .unwrap_or("Ratchet check passed (no baseline)");
                    return RatchetResult {
                        rule: rule.clone(),
                        passed: true,
                        baseline_value: None,
                        current_value,
                        change_pct: None,
                        message: format!("{}: current value = {}", desc, current_value),
                    };
                } else {
                    return RatchetResult {
                        rule: rule.clone(),
                        passed: false,
                        baseline_value: None,
                        current_value,
                        change_pct: None,
                        message: format!(
                            "Baseline value not found at pointer '{}', cannot evaluate percentage increase",
                            rule.pointer
                        ),
                    };
                }
            }
        }
    }

    // All checks passed.
    let desc = rule
        .description
        .as_deref()
        .unwrap_or("Ratchet check passed");

    let message = match (baseline_value, change_pct) {
        (Some(bv), Some(pct)) => format!("{}: {} -> {} ({:+.2}%)", desc, bv, current_value, pct),
        _ => format!("{}: current value = {}", desc, current_value),
    };

    RatchetResult {
        rule: rule.clone(),
        passed: true,
        baseline_value,
        current_value,
        change_pct,
        message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RuleLevel;
    use serde_json::json;

    fn make_rule(
        pointer: &str,
        max_increase_pct: Option<f64>,
        max_value: Option<f64>,
    ) -> RatchetRule {
        RatchetRule {
            pointer: pointer.to_string(),
            max_increase_pct,
            max_value,
            level: RuleLevel::Error,
            description: None,
        }
    }

    // ── evaluate_ratchet (strict wrapper) ──────────────────────────────
    #[test]
    fn passes_when_current_is_within_max_increase() {
        let rule = make_rule("/metric", Some(10.0), None);
        let baseline = json!({"metric": 100.0});
        let current = json!({"metric": 105.0}); // 5% increase

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(result.passed);
        assert_eq!(result.baseline_value, Some(100.0));
        assert_eq!(result.current_value, 105.0);
        let pct = result.change_pct.expect("change_pct should be set");
        assert!((pct - 5.0).abs() < 1e-9);
    }

    #[test]
    fn fails_when_current_exceeds_max_increase() {
        let rule = make_rule("/metric", Some(10.0), None);
        let baseline = json!({"metric": 100.0});
        let current = json!({"metric": 150.0}); // 50% increase

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(!result.passed);
        assert!(
            result.message.contains("50.00%"),
            "expected message to mention 50.00%, got: {}",
            result.message
        );
        assert!(result.message.contains("exceeds"));
    }

    #[test]
    fn passes_at_exact_max_increase_boundary() {
        let rule = make_rule("/metric", Some(10.0), None);
        let baseline = json!({"metric": 100.0});
        let current = json!({"metric": 110.0}); // exactly 10%

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(
            result.passed,
            "10% increase against 10% threshold should pass (>, not >=)"
        );
    }

    #[test]
    fn fails_just_over_max_increase_boundary() {
        let rule = make_rule("/metric", Some(10.0), None);
        let baseline = json!({"metric": 100.0});
        let current = json!({"metric": 110.01});

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(!result.passed);
    }

    #[test]
    fn fails_when_current_exceeds_absolute_max_value() {
        let rule = make_rule("/metric", None, Some(50.0));
        let baseline = json!({"metric": 10.0});
        let current = json!({"metric": 100.0});

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(!result.passed);
        assert!(result.message.contains("exceeds maximum allowed value"));
        assert!(result.message.contains("100"));
        assert!(result.message.contains("50"));
    }

    #[test]
    fn passes_at_exact_max_value_boundary() {
        let rule = make_rule("/metric", None, Some(50.0));
        let baseline = json!({"metric": 10.0});
        let current = json!({"metric": 50.0});

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(result.passed, "exact max should pass (>, not >=)");
    }

    #[test]
    fn fails_when_current_value_missing_by_default() {
        let rule = make_rule("/metric", Some(10.0), None);
        let baseline = json!({"metric": 100.0});
        let current = json!({"other": 0});

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(!result.passed);
        assert!(result.current_value.is_nan());
        assert!(result.message.contains("Current value not found"));
    }

    #[test]
    fn fails_when_current_value_not_numeric() {
        let rule = make_rule("/metric", Some(10.0), None);
        let baseline = json!({"metric": 100.0});
        let current = json!({"metric": [1, 2, 3]}); // array, not numeric

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(!result.passed);
        assert!(result.current_value.is_nan());
        assert!(result.message.contains("not numeric") || result.message.contains("not found"));
    }

    #[test]
    fn fails_when_baseline_missing_for_max_increase_rule() {
        let rule = make_rule("/metric", Some(10.0), None);
        let baseline = json!({"other": 0});
        let current = json!({"metric": 100.0});

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(!result.passed);
        assert_eq!(result.baseline_value, None);
        assert!(result.message.contains("Baseline value not found"));
    }

    #[test]
    fn passes_when_only_max_value_rule_and_no_baseline() {
        // No baseline needed if only an absolute ceiling is checked.
        let rule = make_rule("/metric", None, Some(1000.0));
        let baseline = json!({});
        let current = json!({"metric": 500.0});

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(result.passed);
        assert_eq!(result.baseline_value, None);
        assert_eq!(result.current_value, 500.0);
    }

    #[test]
    fn coerces_numeric_strings_for_baseline_and_current() {
        let rule = make_rule("/metric", Some(10.0), None);
        let baseline = json!({"metric": "100"});
        let current = json!({"metric": "105"});

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(result.passed);
        assert_eq!(result.baseline_value, Some(100.0));
        assert_eq!(result.current_value, 105.0);
    }

    #[test]
    fn uses_rule_description_in_success_message() {
        let rule = RatchetRule {
            pointer: "/metric".into(),
            max_increase_pct: Some(10.0),
            max_value: None,
            level: RuleLevel::Error,
            description: Some("Cyclomatic complexity".into()),
        };
        let baseline = json!({"metric": 10.0});
        let current = json!({"metric": 10.5});

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(result.passed);
        assert!(result.message.contains("Cyclomatic complexity"));
    }

    #[test]
    fn default_success_message_when_no_description() {
        let rule = make_rule("/metric", Some(10.0), None);
        let baseline = json!({"metric": 10.0});
        let current = json!({"metric": 10.5});

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(result.passed);
        assert!(result.message.contains("Ratchet check passed"));
    }

    #[test]
    fn negative_change_is_reported_and_passes() {
        let rule = make_rule("/metric", Some(10.0), None);
        let baseline = json!({"metric": 100.0});
        let current = json!({"metric": 80.0});

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(result.passed);
        let pct = result.change_pct.expect("change_pct");
        assert!(pct < 0.0, "expected negative change_pct, got {pct}");
    }

    #[test]
    fn zero_baseline_with_nonzero_current_is_infinite_increase() {
        let rule = make_rule("/metric", Some(10.0), None);
        let baseline = json!({"metric": 0});
        let current = json!({"metric": 5});

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(!result.passed);
        assert!(
            result
                .change_pct
                .expect("change_pct should be Some")
                .is_infinite()
        );
    }

    #[test]
    fn zero_baseline_and_zero_current_passes() {
        let rule = make_rule("/metric", Some(10.0), None);
        let baseline = json!({"metric": 0});
        let current = json!({"metric": 0});

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(result.passed);
        assert_eq!(result.change_pct, Some(0.0));
    }

    #[test]
    fn max_value_takes_priority_over_max_increase_check() {
        // If both rules are present and max_value is exceeded, fail with max_value message
        // BEFORE evaluating percentage (which would also fail).
        let rule = make_rule("/metric", Some(10.0), Some(50.0));
        let baseline = json!({"metric": 100.0});
        let current = json!({"metric": 200.0}); // 100% over baseline, 4x max_value

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(!result.passed);
        assert!(
            result.message.contains("exceeds maximum allowed value"),
            "max_value check should win; got: {}",
            result.message
        );
    }

    // ── evaluate_ratchet_with_options ──────────────────────────────────
    #[test]
    fn allow_missing_current_returns_pass_with_baseline_recorded() {
        let rule = make_rule("/metric", Some(10.0), None);
        let baseline = json!({"metric": 42.0});
        let current = json!({});

        let result = evaluate_ratchet_with_options(&rule, &baseline, &current, false, true);
        assert!(result.passed);
        assert_eq!(result.baseline_value, Some(42.0));
        assert!(result.current_value.is_nan());
        assert!(result.change_pct.is_none());
        assert!(result.message.contains("(allowed)"));
    }

    #[test]
    fn allow_missing_baseline_returns_pass_with_current_recorded() {
        let rule = make_rule("/metric", Some(10.0), None);
        let baseline = json!({});
        let current = json!({"metric": 7.0});

        let result = evaluate_ratchet_with_options(&rule, &baseline, &current, true, false);
        assert!(result.passed);
        assert_eq!(result.baseline_value, None);
        assert_eq!(result.current_value, 7.0);
        assert!(result.change_pct.is_none());
    }

    #[test]
    fn allow_missing_baseline_uses_description_when_provided() {
        let rule = RatchetRule {
            pointer: "/metric".into(),
            max_increase_pct: Some(10.0),
            max_value: None,
            level: RuleLevel::Error,
            description: Some("My check".into()),
        };
        let baseline = json!({});
        let current = json!({"metric": 7.0});

        let result = evaluate_ratchet_with_options(&rule, &baseline, &current, true, false);
        assert!(result.passed);
        assert!(result.message.contains("My check"));
    }

    #[test]
    fn allow_missing_current_does_not_mask_disallowed_baseline() {
        // allow_missing_current=true, allow_missing_baseline=false — current path wins.
        let rule = make_rule("/metric", Some(10.0), None);
        let baseline = json!({});
        let current = json!({});

        let result = evaluate_ratchet_with_options(&rule, &baseline, &current, false, true);
        // current is missing first, so allowed-current branch returns passed=true
        assert!(result.passed);
        assert!(result.message.contains("(allowed)"));
    }

    #[test]
    fn strict_mode_fails_when_both_baseline_and_current_missing() {
        let rule = make_rule("/metric", Some(10.0), None);
        let baseline = json!({});
        let current = json!({});

        let result = evaluate_ratchet_with_options(&rule, &baseline, &current, false, false);
        assert!(!result.passed);
        assert!(result.current_value.is_nan());
    }

    #[test]
    fn max_value_only_rule_passes_when_baseline_missing_strict() {
        // A max_value-only rule has no baseline dependency, so it passes
        // even when baseline is missing and allow_missing_baseline is false.
        let rule = make_rule("/metric", None, Some(100.0));
        let baseline = json!({});
        let current = json!({"metric": 50.0});

        let result = evaluate_ratchet_with_options(&rule, &baseline, &current, false, false);
        assert!(result.passed);
    }
}
