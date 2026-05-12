//! Single-rule gate policy evaluation.

use super::compare::{compare_contains, compare_equal, compare_in, compare_numeric};
use crate::pointer::resolve_pointer;
use crate::types::{PolicyRule, RuleOperator, RuleResult};
use serde_json::Value;

/// Evaluate a single rule against a receipt.
pub(super) fn evaluate_rule(receipt: &Value, rule: &PolicyRule, allow_missing: bool) -> RuleResult {
    let resolved = resolve_pointer(receipt, &rule.pointer);

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

    let actual = match resolved {
        Some(v) => v,
        None => {
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
        RuleOperator::Exists => unreachable!(),
    };

    let passed = match &comparison_result {
        Ok(result) => {
            if rule.negate {
                !result
            } else {
                *result
            }
        }
        Err(_) => false,
    };

    let empty_values: Vec<Value> = Vec::new();
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

    let generated_failure_message = comparison_result.err().map(|reason| {
        format!(
            "Rule '{}' failed: {} (pointer='{}', op='{}')",
            rule.name, reason, rule.pointer, rule.op
        )
    });

    RuleResult {
        name: rule.name.clone(),
        passed,
        level: rule.level,
        actual: Some(actual.clone()),
        expected,
        message: if passed {
            None
        } else {
            rule.message.clone().or(generated_failure_message)
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RuleLevel;
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
            negate: true,
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

        let result = evaluate_rule(&receipt, &rule, false);
        assert!(!result.passed);

        let result = evaluate_rule(&receipt, &rule, true);
        assert!(result.passed);
    }

    #[test]
    fn test_strict_gt_lt_boundaries() {
        let receipt = json!({"n": 10});

        let gt_equal = make_rule("gt_equal", "/n", RuleOperator::Gt, json!(10));
        assert!(!evaluate_rule(&receipt, &gt_equal, false).passed);

        let lt_equal = make_rule("lt_equal", "/n", RuleOperator::Lt, json!(10));
        assert!(!evaluate_rule(&receipt, &lt_equal, false).passed);
    }

    #[test]
    fn test_numeric_string_coercion() {
        let receipt = json!({"tokens": "1000"});

        let gt = make_rule("gt", "/tokens", RuleOperator::Gt, json!(500));
        assert!(evaluate_rule(&receipt, &gt, false).passed);

        let lt = make_rule("lt", "/tokens", RuleOperator::Lt, json!(1500));
        assert!(evaluate_rule(&receipt, &lt, false).passed);
    }

    #[test]
    fn test_contains_on_string() {
        let receipt = json!({"text": "hello world"});
        let rule = make_rule("contains", "/text", RuleOperator::Contains, json!("world"));
        assert!(evaluate_rule(&receipt, &rule, false).passed);
    }

    #[test]
    fn test_equality_on_non_scalar_values() {
        let receipt = json!({"arr": [1, 2, 3]});
        let rule = make_rule("eq_arr", "/arr", RuleOperator::Eq, json!([1, 2, 3]));
        assert!(evaluate_rule(&receipt, &rule, false).passed);
    }

    #[test]
    fn test_numeric_epsilon_boundary_is_strict() {
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
        let receipt = json!({"lang": "Rust"});

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
        assert!(
            result.expected.contains("Python") && result.expected.contains("Go"),
            "expected string should contain the list values: got '{}'",
            result.expected
        );
    }

    #[test]
    fn test_comparison_type_error_generates_default_message() {
        let receipt = json!({"count": "not-a-number"});
        let rule = make_rule("numeric_check", "/count", RuleOperator::Lte, json!(10));

        let result = evaluate_rule(&receipt, &rule, false);
        assert!(!result.passed);
        let message = result.message.unwrap_or_default();
        assert!(
            message.contains("actual value is not numeric"),
            "expected diagnostic message, got: {message}"
        );
    }

    #[test]
    fn test_custom_message_overrides_generated_failure_message() {
        let receipt = json!({"count": "not-a-number"});
        let rule = PolicyRule {
            name: "numeric_check".into(),
            pointer: "/count".into(),
            op: RuleOperator::Lte,
            value: Some(json!(10)),
            values: None,
            negate: false,
            level: RuleLevel::Error,
            message: Some("Custom policy message".into()),
        };

        let result = evaluate_rule(&receipt, &rule, false);
        assert!(!result.passed);
        assert_eq!(result.message.as_deref(), Some("Custom policy message"));
    }
}
