//! Policy evaluation logic.

mod compare;
mod rule;

use crate::types::{GateResult, PolicyConfig, RuleLevel};
use rule::evaluate_rule;
use serde_json::Value;

/// Evaluate a policy against a JSON receipt.
///
/// # Arguments
/// * `receipt` - The JSON receipt to evaluate
/// * `policy` - The policy configuration with rules
///
/// # Returns
/// A `GateResult` with pass/fail status and individual rule results.
///
/// # Examples
///
/// ```
/// use serde_json::json;
/// use tokmd_gate::{evaluate_policy, PolicyConfig, PolicyRule, RuleOperator, RuleLevel};
///
/// let receipt = json!({"tokens": 500, "files": 10});
/// let policy = PolicyConfig {
///     rules: vec![PolicyRule {
///         name: "max_tokens".into(),
///         pointer: "/tokens".into(),
///         op: RuleOperator::Lte,
///         value: Some(json!(1000)),
///         values: None,
///         negate: false,
///         level: RuleLevel::Error,
///         message: None,
///     }],
///     fail_fast: false,
///     allow_missing: false,
/// };
///
/// let result = evaluate_policy(&receipt, &policy);
/// assert!(result.passed);
/// assert_eq!(result.errors, 0);
/// ```
pub fn evaluate_policy(receipt: &Value, policy: &PolicyConfig) -> GateResult {
    let mut rule_results = Vec::with_capacity(policy.rules.len());

    for rule in &policy.rules {
        let result = evaluate_rule(receipt, rule, policy.allow_missing);
        let failed_error = !result.passed && result.level == RuleLevel::Error;

        rule_results.push(result);

        if policy.fail_fast && failed_error {
            break;
        }
    }

    GateResult::from_results(rule_results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PolicyRule, RuleOperator};
    use serde_json::json;

    fn make_rule(name: &str, pointer: &str, op: RuleOperator, value: Value) -> PolicyRule {
        PolicyRule {
            name: name.into(),
            pointer: pointer.into(),
            op,
            value: Some(value),
            values: None,
            negate: false,
            level: RuleLevel::Error,
            message: None,
        }
    }

    #[test]
    fn test_full_policy() {
        let receipt = json!({
            "derived": {
                "totals": {"tokens": 100000, "code": 5000}
            },
            "license": {"effective": "MIT"}
        });

        let policy = PolicyConfig {
            rules: vec![
                make_rule(
                    "max_tokens",
                    "/derived/totals/tokens",
                    RuleOperator::Lte,
                    json!(500000),
                ),
                make_rule(
                    "min_code",
                    "/derived/totals/code",
                    RuleOperator::Gte,
                    json!(100),
                ),
            ],
            fail_fast: false,
            allow_missing: false,
        };

        let result = evaluate_policy(&receipt, &policy);
        assert!(result.passed);
        assert_eq!(result.errors, 0);
        assert_eq!(result.warnings, 0);
    }

    #[test]
    fn test_fail_fast() {
        let receipt = json!({"a": 1, "b": 2});

        let policy = PolicyConfig {
            rules: vec![
                make_rule("first", "/a", RuleOperator::Gt, json!(10)), // fails
                make_rule("second", "/b", RuleOperator::Gt, json!(10)), // also fails
            ],
            fail_fast: true,
            allow_missing: false,
        };

        let result = evaluate_policy(&receipt, &policy);
        assert!(!result.passed);
        // Only one rule evaluated due to fail_fast
        assert_eq!(result.rule_results.len(), 1);
    }

    #[test]
    fn test_fail_fast_does_not_stop_on_pass() {
        // If fail_fast is true, we should NOT stop on a passing rule, even if its level is Error.
        // This kills the `failed_error` &&/|| mutants.
        let receipt = json!({"a": 1, "b": 2});

        let policy = PolicyConfig {
            rules: vec![
                make_rule("first_passes", "/a", RuleOperator::Gt, json!(0)), // passes
                make_rule("second_fails", "/b", RuleOperator::Gt, json!(10)), // fails
            ],
            fail_fast: true,
            allow_missing: false,
        };

        let result = evaluate_policy(&receipt, &policy);
        assert!(!result.passed);
        assert_eq!(
            result.rule_results.len(),
            2,
            "fail_fast must not stop after a passing rule"
        );
        assert_eq!(result.errors, 1);
    }

    #[test]
    fn test_no_fail_fast_evaluates_all_rules_even_after_failure() {
        // Kills the `if policy.fail_fast && failed_error` &&/|| mutant.
        let receipt = json!({"a": 1, "b": 2});

        let policy = PolicyConfig {
            rules: vec![
                make_rule("first_fails", "/a", RuleOperator::Gt, json!(10)), // fails
                make_rule("second_passes", "/b", RuleOperator::Gt, json!(0)), // passes
            ],
            fail_fast: false,
            allow_missing: false,
        };

        let result = evaluate_policy(&receipt, &policy);
        assert!(!result.passed);
        assert_eq!(
            result.rule_results.len(),
            2,
            "when fail_fast is false we should evaluate all rules"
        );
        assert_eq!(result.errors, 1);
    }
}
