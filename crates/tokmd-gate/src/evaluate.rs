//! Policy evaluation logic.

use crate::pointer::resolve_pointer;
use crate::types::{GateResult, PolicyConfig, PolicyRule, RuleLevel, RuleOperator, RuleResult};
use serde_json::Value;

/// Evaluate a policy against a JSON receipt.
///
/// # Arguments
/// * `receipt` - The JSON receipt to evaluate
/// * `policy` - The policy configuration with rules
///
/// # Returns
/// A `GateResult` with pass/fail status and individual rule results.
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

/// Evaluate a single rule against a receipt.
fn evaluate_rule(receipt: &Value, rule: &PolicyRule, allow_missing: bool) -> RuleResult {
    let resolved = resolve_pointer(receipt, &rule.pointer);

    // Handle exists operator specially
    if rule.op == RuleOperator::Exists {
        let exists = resolved.is_some();
        let passed = if rule.negate { !exists } else { exists };
        return RuleResult {
            name: rule.name.clone(),
            passed,
            level: rule.level,
            actual: resolved.cloned(),
            expected: if rule.negate {
                format!("pointer {} to NOT exist", rule.pointer)
            } else {
                format!("pointer {} to exist", rule.pointer)
            },
            message: if passed { None } else { rule.message.clone() },
        };
    }

    // For other operators, we need a value
    let actual = match resolved {
        Some(v) => v,
        None => {
            // Missing value
            if allow_missing {
                return RuleResult {
                    name: rule.name.clone(),
                    passed: true,
                    level: rule.level,
                    actual: None,
                    expected: format!("{} {} {:?}", rule.pointer, rule.op, rule.value),
                    message: None,
                };
            } else {
                return RuleResult {
                    name: rule.name.clone(),
                    passed: false,
                    level: rule.level,
                    actual: None,
                    expected: format!("{} {} {:?}", rule.pointer, rule.op, rule.value),
                    message: Some(format!("Pointer '{}' not found in receipt", rule.pointer)),
                };
            }
        }
    };

    let comparison_result = match rule.op {
        RuleOperator::Gt => compare_numeric(actual, rule.value.as_ref(), |a, b| a > b),
        RuleOperator::Gte => compare_numeric(actual, rule.value.as_ref(), |a, b| a >= b),
        RuleOperator::Lt => compare_numeric(actual, rule.value.as_ref(), |a, b| a < b),
        RuleOperator::Lte => compare_numeric(actual, rule.value.as_ref(), |a, b| a <= b),
        RuleOperator::Eq => compare_equal(actual, rule.value.as_ref()),
        RuleOperator::Ne => compare_equal(actual, rule.value.as_ref()).map(|b| !b),
        RuleOperator::In => compare_in(actual, rule.values.as_ref()),
        RuleOperator::Contains => compare_contains(actual, rule.value.as_ref()),
        RuleOperator::Exists => unreachable!(), // Handled above
    };

    let passed = match comparison_result {
        Ok(result) => {
            if rule.negate {
                !result
            } else {
                result
            }
        }
        Err(_) => false,
    };

    let empty_values: Vec<serde_json::Value> = Vec::new();
    let expected = match rule.op {
        RuleOperator::In => format!(
            "{} in {:?}",
            rule.pointer,
            rule.values.as_ref().unwrap_or(&empty_values)
        ),
        _ => format!(
            "{} {} {}",
            rule.pointer,
            rule.op,
            rule.value
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_default()
        ),
    };

    RuleResult {
        name: rule.name.clone(),
        passed,
        level: rule.level,
        actual: Some(actual.clone()),
        expected,
        message: if passed { None } else { rule.message.clone() },
    }
}

/// Compare two values numerically.
fn compare_numeric<F>(actual: &Value, expected: Option<&Value>, cmp: F) -> Result<bool, ()>
where
    F: Fn(f64, f64) -> bool,
{
    let actual_num = value_to_f64(actual).ok_or(())?;
    let expected_num = expected.and_then(value_to_f64).ok_or(())?;
    Ok(cmp(actual_num, expected_num))
}

/// Compare two values for equality.
fn compare_equal(actual: &Value, expected: Option<&Value>) -> Result<bool, ()> {
    let expected = expected.ok_or(())?;

    // For numeric types, compare as f64 to handle int/float mismatches
    if let (Some(a), Some(b)) = (value_to_f64(actual), value_to_f64(expected)) {
        return Ok((a - b).abs() < f64::EPSILON);
    }

    // For strings, compare case-sensitively
    if let (Some(a), Some(b)) = (actual.as_str(), expected.as_str()) {
        return Ok(a == b);
    }

    // For other types, use JSON equality
    Ok(actual == expected)
}

/// Check if actual value is in a list of expected values.
fn compare_in(actual: &Value, expected: Option<&Vec<Value>>) -> Result<bool, ()> {
    let list = expected.ok_or(())?;

    for item in list {
        if compare_equal(actual, Some(item)).unwrap_or(false) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Check if actual contains expected.
fn compare_contains(actual: &Value, expected: Option<&Value>) -> Result<bool, ()> {
    let expected = expected.ok_or(())?;

    match actual {
        Value::String(s) => {
            let needle = expected.as_str().ok_or(())?;
            Ok(s.contains(needle))
        }
        Value::Array(arr) => {
            for item in arr {
                if compare_equal(item, Some(expected)).unwrap_or(false) {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        _ => Err(()),
    }
}

/// Convert a JSON value to f64 for numeric comparison.
fn value_to_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_numeric_comparisons() {
        let receipt = json!({"tokens": 1000});

        let rule = make_rule("test", "/tokens", RuleOperator::Lt, json!(2000));
        let result = evaluate_rule(&receipt, &rule, false);
        assert!(result.passed);

        let rule = make_rule("test", "/tokens", RuleOperator::Gt, json!(2000));
        let result = evaluate_rule(&receipt, &rule, false);
        assert!(!result.passed);

        let rule = make_rule("test", "/tokens", RuleOperator::Lte, json!(1000));
        let result = evaluate_rule(&receipt, &rule, false);
        assert!(result.passed);

        let rule = make_rule("test", "/tokens", RuleOperator::Gte, json!(1000));
        let result = evaluate_rule(&receipt, &rule, false);
        assert!(result.passed);
    }

    #[test]
    fn test_equality() {
        let receipt = json!({"name": "test", "count": 42});

        let rule = make_rule("test", "/name", RuleOperator::Eq, json!("test"));
        let result = evaluate_rule(&receipt, &rule, false);
        assert!(result.passed);

        let rule = make_rule("test", "/count", RuleOperator::Eq, json!(42));
        let result = evaluate_rule(&receipt, &rule, false);
        assert!(result.passed);

        let rule = make_rule("test", "/name", RuleOperator::Ne, json!("other"));
        let result = evaluate_rule(&receipt, &rule, false);
        assert!(result.passed);
    }

    #[test]
    fn test_in_operator() {
        let receipt = json!({"license": "MIT"});

        let rule = PolicyRule {
            name: "license_check".into(),
            pointer: "/license".into(),
            op: RuleOperator::In,
            value: None,
            values: Some(vec![json!("MIT"), json!("Apache-2.0")]),
            negate: false,
            level: RuleLevel::Error,
            message: None,
        };
        let result = evaluate_rule(&receipt, &rule, false);
        assert!(result.passed);

        let rule = PolicyRule {
            name: "license_check".into(),
            pointer: "/license".into(),
            op: RuleOperator::In,
            value: None,
            values: Some(vec![json!("GPL"), json!("LGPL")]),
            negate: false,
            level: RuleLevel::Error,
            message: None,
        };
        let result = evaluate_rule(&receipt, &rule, false);
        assert!(!result.passed);
    }

    #[test]
    fn test_contains_operator() {
        let receipt = json!({"tags": ["rust", "cli", "tools"]});

        let rule = make_rule("test", "/tags", RuleOperator::Contains, json!("cli"));
        let result = evaluate_rule(&receipt, &rule, false);
        assert!(result.passed);

        let rule = make_rule("test", "/tags", RuleOperator::Contains, json!("python"));
        let result = evaluate_rule(&receipt, &rule, false);
        assert!(!result.passed);
    }

    #[test]
    fn test_exists_operator() {
        let receipt = json!({"license": "MIT"});

        let rule = PolicyRule {
            name: "has_license".into(),
            pointer: "/license".into(),
            op: RuleOperator::Exists,
            value: None,
            values: None,
            negate: false,
            level: RuleLevel::Error,
            message: None,
        };
        let result = evaluate_rule(&receipt, &rule, false);
        assert!(result.passed);

        let rule = PolicyRule {
            name: "no_secrets".into(),
            pointer: "/secrets".into(),
            op: RuleOperator::Exists,
            value: None,
            values: None,
            negate: true, // should NOT exist
            level: RuleLevel::Error,
            message: None,
        };
        let result = evaluate_rule(&receipt, &rule, false);
        assert!(result.passed);
    }

    #[test]
    fn test_negate() {
        let receipt = json!({"count": 100});

        let rule = PolicyRule {
            name: "not_above_50".into(),
            pointer: "/count".into(),
            op: RuleOperator::Gt,
            value: Some(json!(50)),
            values: None,
            negate: true,
            level: RuleLevel::Error,
            message: None,
        };
        let result = evaluate_rule(&receipt, &rule, false);
        assert!(!result.passed); // 100 > 50 is true, negated = false

        let rule = PolicyRule {
            name: "not_above_200".into(),
            pointer: "/count".into(),
            op: RuleOperator::Gt,
            value: Some(json!(200)),
            values: None,
            negate: true,
            level: RuleLevel::Error,
            message: None,
        };
        let result = evaluate_rule(&receipt, &rule, false);
        assert!(result.passed); // 100 > 200 is false, negated = true
    }

    #[test]
    fn test_missing_value() {
        let receipt = json!({"foo": 1});

        let rule = make_rule("test", "/bar", RuleOperator::Eq, json!(1));

        // Without allow_missing - fails
        let result = evaluate_rule(&receipt, &rule, false);
        assert!(!result.passed);

        // With allow_missing - passes
        let result = evaluate_rule(&receipt, &rule, true);
        assert!(result.passed);
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

    #[test]
    fn test_strict_gt_lt_boundaries() {
        // Kills >->>= and <-><= mutants.
        let receipt = json!({"n": 10});

        let gt_equal = make_rule("gt_equal", "/n", RuleOperator::Gt, json!(10));
        assert!(!evaluate_rule(&receipt, &gt_equal, false).passed);

        let lt_equal = make_rule("lt_equal", "/n", RuleOperator::Lt, json!(10));
        assert!(!evaluate_rule(&receipt, &lt_equal, false).passed);
    }

    #[test]
    fn test_numeric_string_coercion() {
        // Kills deletion of Value::String arm in value_to_f64.
        let receipt = json!({"tokens": "1000"});

        let gt = make_rule("gt", "/tokens", RuleOperator::Gt, json!(500));
        assert!(evaluate_rule(&receipt, &gt, false).passed);

        let lt = make_rule("lt", "/tokens", RuleOperator::Lt, json!(1500));
        assert!(evaluate_rule(&receipt, &lt, false).passed);
    }

    #[test]
    fn test_contains_on_string() {
        // Kills deletion of Value::String arm in compare_contains.
        let receipt = json!({"text": "hello world"});
        let rule = make_rule("contains", "/text", RuleOperator::Contains, json!("world"));
        assert!(evaluate_rule(&receipt, &rule, false).passed);
    }

    #[test]
    fn test_equality_on_non_scalar_values() {
        // Kills mutation of the JSON equality fallback.
        let receipt = json!({"arr": [1, 2, 3]});
        let rule = make_rule("eq_arr", "/arr", RuleOperator::Eq, json!([1, 2, 3]));
        assert!(evaluate_rule(&receipt, &rule, false).passed);
    }

    #[test]
    fn test_numeric_epsilon_boundary_is_strict() {
        // Kills < -> <= mutant in numeric equality (abs(a-b) < EPSILON).
        let a = 1.0_f64;
        let b = a + f64::EPSILON;
        let receipt = json!({"x": a});
        let rule = make_rule("eq_eps", "/x", RuleOperator::Eq, json!(b));
        assert!(
            !evaluate_rule(&receipt, &rule, false).passed,
            "difference of exactly EPSILON must not be treated as equal"
        );
    }

    #[test]
    fn test_in_operator_membership() {
        // Additional test to ensure In arm deletion mutant is killed.
        let receipt = json!({"lang": "Rust"});

        let rule = PolicyRule {
            name: "lang_in".into(),
            pointer: "/lang".into(),
            op: RuleOperator::In,
            value: None,
            values: Some(vec![json!("Rust"), json!("Go")]),
            negate: false,
            level: RuleLevel::Error,
            message: None,
        };

        assert!(evaluate_rule(&receipt, &rule, false).passed);
    }

    #[test]
    fn test_in_operator_non_member() {
        // Kills In arm mutant by asserting non-membership fails.
        let receipt = json!({"lang": "Rust"});

        let rule = PolicyRule {
            name: "lang_not_in".into(),
            pointer: "/lang".into(),
            op: RuleOperator::In,
            value: None,
            values: Some(vec![json!("Python"), json!("Go")]),
            negate: false,
            level: RuleLevel::Error,
            message: None,
        };

        assert!(!evaluate_rule(&receipt, &rule, false).passed);
    }

    #[test]
    fn test_in_operator_with_negate() {
        // Test In with negate to ensure proper behavior when inverted.
        let receipt = json!({"lang": "Rust"});

        // Rust IS in the list, but negated → should fail
        let rule = PolicyRule {
            name: "lang_not_in_negate".into(),
            pointer: "/lang".into(),
            op: RuleOperator::In,
            value: None,
            values: Some(vec![json!("Rust"), json!("Go")]),
            negate: true,
            level: RuleLevel::Error,
            message: None,
        };

        assert!(!evaluate_rule(&receipt, &rule, false).passed);
    }

    #[test]
    fn test_in_operator_expected_format() {
        // Kills the "delete match arm RuleOperator::In" mutant in the expected string formatting.
        // The In operator must produce an expected string like "/pointer in [values]",
        // not the generic "/pointer in " format from the wildcard arm.
        let receipt = json!({"lang": "Rust"});

        let rule = PolicyRule {
            name: "lang_in".into(),
            pointer: "/lang".into(),
            op: RuleOperator::In,
            value: None,
            values: Some(vec![json!("Python"), json!("Go")]),
            negate: false,
            level: RuleLevel::Error,
            message: None,
        };

        let result = evaluate_rule(&receipt, &rule, false);
        assert!(!result.passed);
        // The expected string must contain the values list, proving the In arm was used.
        // If the In arm is deleted, expected would be "/lang in " (empty/missing values).
        assert!(
            result.expected.contains("Python") && result.expected.contains("Go"),
            "expected string should contain the list values: got '{}'",
            result.expected
        );
    }
}
