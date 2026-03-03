//! Deep tests for tokmd-gate.
//!
//! Targets areas not fully covered by existing test files:
//! - Comparison operators with `None` value (missing value field)
//! - Policy evaluation on empty/minimal JSON documents
//! - Ratchet evaluation with negative, zero, and extreme values
//! - Ratchet `from_file` roundtrip with actual temp file
//! - Negate combined with allow_missing
//! - Multiple rules mixing error and warn levels
//! - GateResult / RatchetGateResult invariants under varied inputs
//! - Ratchet percentage calculation accuracy
//! - Contains on nested array/string edge cases
//! - In operator with mixed-type value lists
//! - Exists operator on null values

use serde_json::json;
use tokmd_gate::{
    GateResult, PolicyConfig, PolicyRule, RatchetConfig, RatchetGateResult, RatchetResult,
    RatchetRule, RuleLevel, RuleOperator, RuleResult, evaluate_policy, evaluate_ratchet_policy,
    resolve_pointer,
};

// =========================================================================
// Helpers
// =========================================================================

fn make_rule(name: &str, pointer: &str, op: RuleOperator, value: serde_json::Value) -> PolicyRule {
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

fn eval_single(receipt: &serde_json::Value, rule: &PolicyRule) -> RuleResult {
    let policy = PolicyConfig {
        rules: vec![rule.clone()],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(receipt, &policy);
    result.rule_results.into_iter().next().unwrap()
}

fn make_ratchet_rule(
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

// =========================================================================
// 1. Comparison operators with None value field
// =========================================================================

mod missing_value_field {
    use super::*;

    #[test]
    fn gt_with_no_value_fails_gracefully() {
        let receipt = json!({"x": 10});
        let rule = PolicyRule {
            name: "gt_no_val".into(),
            pointer: "/x".into(),
            op: RuleOperator::Gt,
            value: None,
            values: None,
            negate: false,
            level: RuleLevel::Error,
            message: None,
        };
        let result = eval_single(&receipt, &rule);
        assert!(!result.passed, "Gt with no expected value should fail");
    }

    #[test]
    fn eq_with_no_value_fails_gracefully() {
        let receipt = json!({"x": 10});
        let rule = PolicyRule {
            name: "eq_no_val".into(),
            pointer: "/x".into(),
            op: RuleOperator::Eq,
            value: None,
            values: None,
            negate: false,
            level: RuleLevel::Error,
            message: None,
        };
        let result = eval_single(&receipt, &rule);
        assert!(!result.passed, "Eq with no expected value should fail");
    }

    #[test]
    fn in_with_no_values_list_fails_gracefully() {
        let receipt = json!({"x": "a"});
        let rule = PolicyRule {
            name: "in_no_vals".into(),
            pointer: "/x".into(),
            op: RuleOperator::In,
            value: None,
            values: None,
            negate: false,
            level: RuleLevel::Error,
            message: None,
        };
        let result = eval_single(&receipt, &rule);
        assert!(!result.passed, "In with no values list should fail");
    }

    #[test]
    fn contains_with_no_value_fails_gracefully() {
        let receipt = json!({"arr": [1, 2, 3]});
        let rule = PolicyRule {
            name: "contains_no_val".into(),
            pointer: "/arr".into(),
            op: RuleOperator::Contains,
            value: None,
            values: None,
            negate: false,
            level: RuleLevel::Error,
            message: None,
        };
        let result = eval_single(&receipt, &rule);
        assert!(
            !result.passed,
            "Contains with no expected value should fail"
        );
    }
}

// =========================================================================
// 2. JSON pointer resolution on edge-case documents
// =========================================================================

mod pointer_deep {
    use super::*;

    #[test]
    fn pointer_on_empty_object() {
        let doc = json!({});
        assert_eq!(resolve_pointer(&doc, ""), Some(&doc));
        assert_eq!(resolve_pointer(&doc, "/anything"), None);
    }

    #[test]
    fn pointer_on_scalar_root() {
        let doc = json!(42);
        assert_eq!(resolve_pointer(&doc, ""), Some(&json!(42)));
        assert_eq!(resolve_pointer(&doc, "/x"), None);
    }

    #[test]
    fn pointer_on_array_root() {
        let doc = json!([1, 2, 3]);
        assert_eq!(resolve_pointer(&doc, ""), Some(&doc));
        assert_eq!(resolve_pointer(&doc, "/0"), Some(&json!(1)));
        assert_eq!(resolve_pointer(&doc, "/2"), Some(&json!(3)));
        assert_eq!(resolve_pointer(&doc, "/key"), None);
    }

    #[test]
    fn pointer_on_null_root() {
        let doc = json!(null);
        assert_eq!(resolve_pointer(&doc, ""), Some(&json!(null)));
        assert_eq!(resolve_pointer(&doc, "/x"), None);
    }

    #[test]
    fn pointer_on_string_root() {
        let doc = json!("hello");
        assert_eq!(resolve_pointer(&doc, ""), Some(&json!("hello")));
        assert_eq!(resolve_pointer(&doc, "/0"), None);
    }

    #[test]
    fn pointer_on_deeply_nested_array_of_objects() {
        let doc = json!({
            "results": [
                {"metrics": [{"name": "loc", "value": 100}]},
                {"metrics": [{"name": "tokens", "value": 200}]}
            ]
        });
        assert_eq!(
            resolve_pointer(&doc, "/results/0/metrics/0/value"),
            Some(&json!(100))
        );
        assert_eq!(
            resolve_pointer(&doc, "/results/1/metrics/0/name"),
            Some(&json!("tokens"))
        );
    }

    #[test]
    fn pointer_double_escape_round_trip() {
        // Key with both ~ and / characters
        let doc = json!({"a~b/c": 42});
        assert_eq!(resolve_pointer(&doc, "/a~0b~1c"), Some(&json!(42)));
    }

    #[test]
    fn pointer_to_false_value() {
        let doc = json!({"flag": false});
        assert_eq!(resolve_pointer(&doc, "/flag"), Some(&json!(false)));
    }

    #[test]
    fn pointer_to_zero_value() {
        let doc = json!({"count": 0});
        assert_eq!(resolve_pointer(&doc, "/count"), Some(&json!(0)));
    }

    #[test]
    fn pointer_to_empty_string_value() {
        let doc = json!({"name": ""});
        assert_eq!(resolve_pointer(&doc, "/name"), Some(&json!("")));
    }

    #[test]
    fn pointer_to_empty_array_value() {
        let doc = json!({"items": []});
        assert_eq!(resolve_pointer(&doc, "/items"), Some(&json!([])));
        assert_eq!(resolve_pointer(&doc, "/items/0"), None);
    }
}

// =========================================================================
// 3. All comparison operators at boundaries
// =========================================================================

mod comparison_boundaries {
    use super::*;

    #[test]
    fn all_operators_at_equality_boundary() {
        let receipt = json!({"n": 42});
        let v = json!(42);

        assert!(
            !eval_single(
                &receipt,
                &make_rule("gt", "/n", RuleOperator::Gt, v.clone())
            )
            .passed
        );
        assert!(
            eval_single(
                &receipt,
                &make_rule("gte", "/n", RuleOperator::Gte, v.clone())
            )
            .passed
        );
        assert!(
            !eval_single(
                &receipt,
                &make_rule("lt", "/n", RuleOperator::Lt, v.clone())
            )
            .passed
        );
        assert!(
            eval_single(
                &receipt,
                &make_rule("lte", "/n", RuleOperator::Lte, v.clone())
            )
            .passed
        );
        assert!(
            eval_single(
                &receipt,
                &make_rule("eq", "/n", RuleOperator::Eq, v.clone())
            )
            .passed
        );
        assert!(!eval_single(&receipt, &make_rule("ne", "/n", RuleOperator::Ne, v)).passed);
    }

    #[test]
    fn float_boundary_just_below_and_above() {
        let receipt = json!({"val": 10.0});

        let just_above = json!(10.0 + 1e-10);
        let just_below = json!(10.0 - 1e-10);

        // val < just_above should be true
        assert!(
            eval_single(
                &receipt,
                &make_rule("lt", "/val", RuleOperator::Lt, just_above)
            )
            .passed
        );
        // val > just_below should be true
        assert!(
            eval_single(
                &receipt,
                &make_rule("gt", "/val", RuleOperator::Gt, just_below)
            )
            .passed
        );
    }

    #[test]
    fn negative_number_comparisons() {
        let receipt = json!({"n": -5});
        assert!(
            eval_single(
                &receipt,
                &make_rule("gt_neg", "/n", RuleOperator::Gt, json!(-10))
            )
            .passed
        );
        assert!(
            eval_single(
                &receipt,
                &make_rule("lt_neg", "/n", RuleOperator::Lt, json!(0))
            )
            .passed
        );
        assert!(
            eval_single(
                &receipt,
                &make_rule("eq_neg", "/n", RuleOperator::Eq, json!(-5))
            )
            .passed
        );
    }

    #[test]
    fn large_number_comparisons() {
        let receipt = json!({"big": 1_000_000_000i64});
        assert!(
            eval_single(
                &receipt,
                &make_rule("gt", "/big", RuleOperator::Gt, json!(999_999_999))
            )
            .passed
        );
        assert!(
            eval_single(
                &receipt,
                &make_rule("lte", "/big", RuleOperator::Lte, json!(1_000_000_000i64))
            )
            .passed
        );
    }
}

// =========================================================================
// 4. Threshold / multiple rules mixing error and warn
// =========================================================================

mod mixed_level_rules {
    use super::*;

    #[test]
    fn multiple_errors_multiple_warns_all_failing() {
        let receipt = json!({"a": 100, "b": 200, "c": 300});
        let policy = PolicyConfig {
            rules: vec![
                PolicyRule {
                    level: RuleLevel::Error,
                    ..make_rule("err1", "/a", RuleOperator::Lt, json!(50))
                },
                PolicyRule {
                    level: RuleLevel::Warn,
                    ..make_rule("warn1", "/b", RuleOperator::Lt, json!(50))
                },
                PolicyRule {
                    level: RuleLevel::Error,
                    ..make_rule("err2", "/c", RuleOperator::Lt, json!(50))
                },
                PolicyRule {
                    level: RuleLevel::Warn,
                    ..make_rule("warn2", "/a", RuleOperator::Lt, json!(50))
                },
            ],
            fail_fast: false,
            allow_missing: false,
        };

        let result = evaluate_policy(&receipt, &policy);
        assert!(!result.passed);
        assert_eq!(result.errors, 2);
        assert_eq!(result.warnings, 2);
        assert_eq!(result.rule_results.len(), 4);
    }

    #[test]
    fn all_passing_rules_mixed_levels() {
        let receipt = json!({"a": 1, "b": 2});
        let policy = PolicyConfig {
            rules: vec![
                PolicyRule {
                    level: RuleLevel::Error,
                    ..make_rule("err", "/a", RuleOperator::Gte, json!(1))
                },
                PolicyRule {
                    level: RuleLevel::Warn,
                    ..make_rule("warn", "/b", RuleOperator::Gte, json!(1))
                },
            ],
            fail_fast: false,
            allow_missing: false,
        };

        let result = evaluate_policy(&receipt, &policy);
        assert!(result.passed);
        assert_eq!(result.errors, 0);
        assert_eq!(result.warnings, 0);
    }
}

// =========================================================================
// 5. Negate combined with allow_missing
// =========================================================================

mod negate_and_allow_missing {
    use super::*;

    #[test]
    fn negate_exists_on_missing_field_with_allow_missing_true() {
        let receipt = json!({"x": 1});
        let policy = PolicyConfig {
            rules: vec![PolicyRule {
                name: "no_secrets".into(),
                pointer: "/secrets".into(),
                op: RuleOperator::Exists,
                value: None,
                values: None,
                negate: true,
                level: RuleLevel::Error,
                message: None,
            }],
            fail_fast: false,
            allow_missing: true,
        };
        let result = evaluate_policy(&receipt, &policy);
        // Negated exists on missing field: exists=false, negated=true → pass
        assert!(result.passed);
    }

    #[test]
    fn negate_gt_on_missing_field_with_allow_missing_false() {
        let receipt = json!({"x": 1});
        let policy = PolicyConfig {
            rules: vec![PolicyRule {
                negate: true,
                ..make_rule("neg_gt", "/missing", RuleOperator::Gt, json!(5))
            }],
            fail_fast: false,
            allow_missing: false,
        };
        let result = evaluate_policy(&receipt, &policy);
        // Missing field + allow_missing=false → fail regardless of negate
        assert!(!result.passed);
    }

    #[test]
    fn negate_gt_on_missing_field_with_allow_missing_true() {
        let receipt = json!({"x": 1});
        let policy = PolicyConfig {
            rules: vec![PolicyRule {
                negate: true,
                ..make_rule("neg_gt", "/missing", RuleOperator::Gt, json!(5))
            }],
            fail_fast: false,
            allow_missing: true,
        };
        let result = evaluate_policy(&receipt, &policy);
        // Missing field + allow_missing=true → pass
        assert!(result.passed);
    }
}

// =========================================================================
// 6. Missing field handling on all non-Exists operators
// =========================================================================

mod missing_field_operators {
    use super::*;

    #[test]
    fn all_comparison_operators_fail_on_missing_field() {
        let receipt = json!({"x": 1});
        let operators = [
            RuleOperator::Gt,
            RuleOperator::Gte,
            RuleOperator::Lt,
            RuleOperator::Lte,
            RuleOperator::Eq,
            RuleOperator::Ne,
        ];

        for op in &operators {
            let rule = make_rule("test", "/missing", *op, json!(1));
            let result = eval_single(&receipt, &rule);
            assert!(
                !result.passed,
                "Operator {:?} should fail on missing field",
                op
            );
            assert!(
                result.message.as_ref().unwrap().contains("not found"),
                "Missing field message should mention 'not found' for {:?}",
                op
            );
        }
    }
}

// =========================================================================
// 7. Type mismatch handling
// =========================================================================

mod type_mismatch {
    use super::*;

    #[test]
    fn gt_on_boolean_value_fails() {
        let receipt = json!({"flag": true});
        let rule = make_rule("test", "/flag", RuleOperator::Gt, json!(0));
        assert!(!eval_single(&receipt, &rule).passed);
    }

    #[test]
    fn gt_on_null_value_fails() {
        let receipt = json!({"val": null});
        let rule = make_rule("test", "/val", RuleOperator::Gt, json!(0));
        assert!(!eval_single(&receipt, &rule).passed);
    }

    #[test]
    fn gt_on_array_value_fails() {
        let receipt = json!({"arr": [1, 2, 3]});
        let rule = make_rule("test", "/arr", RuleOperator::Gt, json!(0));
        assert!(!eval_single(&receipt, &rule).passed);
    }

    #[test]
    fn gt_on_object_value_fails() {
        let receipt = json!({"obj": {"a": 1}});
        let rule = make_rule("test", "/obj", RuleOperator::Gt, json!(0));
        assert!(!eval_single(&receipt, &rule).passed);
    }

    #[test]
    fn eq_between_number_and_string_of_same_digits() {
        // "42" as string vs 42 as number: compare_equal falls through string check
        // (one side is not a string), then tries numeric coercion via value_to_f64.
        // value_to_f64 parses string "42" → 42.0, number 42 → 42.0, so they match.
        let receipt = json!({"val": "42"});
        let rule = make_rule("test", "/val", RuleOperator::Eq, json!(42));
        let result = eval_single(&receipt, &rule);
        assert!(
            result.passed,
            "string '42' should coerce to number 42 via value_to_f64"
        );
    }

    #[test]
    fn eq_between_non_numeric_string_and_number() {
        // "hello" cannot be parsed to f64, so numeric comparison fails;
        // falls through to JSON equality which is false (different types).
        let receipt = json!({"val": "hello"});
        let rule = make_rule("test", "/val", RuleOperator::Eq, json!(42));
        let result = eval_single(&receipt, &rule);
        assert!(
            !result.passed,
            "non-numeric string should not equal a number"
        );
    }
}

// =========================================================================
// 8. Edge cases: empty rules, empty data, null values
// =========================================================================

mod edge_cases_empty {
    use super::*;

    #[test]
    fn empty_rules_empty_data() {
        let receipt = json!({});
        let policy = PolicyConfig::default();
        let result = evaluate_policy(&receipt, &policy);
        assert!(result.passed);
        assert_eq!(result.rule_results.len(), 0);
    }

    #[test]
    fn policy_on_null_root_document() {
        let receipt = json!(null);
        let rule = make_rule("test", "/x", RuleOperator::Eq, json!(1));
        let result = eval_single(&receipt, &rule);
        assert!(!result.passed);
    }

    #[test]
    fn exists_on_null_value_passes() {
        // null is a valid JSON value — the pointer resolves, so exists is true
        let receipt = json!({"key": null});
        let rule = PolicyRule {
            name: "exists_null".into(),
            pointer: "/key".into(),
            op: RuleOperator::Exists,
            value: None,
            values: None,
            negate: false,
            level: RuleLevel::Error,
            message: None,
        };
        assert!(eval_single(&receipt, &rule).passed);
    }

    #[test]
    fn gate_result_from_single_passed_error() {
        let results = vec![RuleResult {
            name: "r1".into(),
            passed: true,
            level: RuleLevel::Error,
            actual: Some(json!(1)),
            expected: "/x >= 0".into(),
            message: None,
        }];
        let gate = GateResult::from_results(results);
        assert!(gate.passed);
        assert_eq!(gate.errors, 0);
        assert_eq!(gate.warnings, 0);
        assert_eq!(gate.rule_results.len(), 1);
    }

    #[test]
    fn gate_result_from_single_failed_error() {
        let results = vec![RuleResult {
            name: "r1".into(),
            passed: false,
            level: RuleLevel::Error,
            actual: Some(json!(-1)),
            expected: "/x >= 0".into(),
            message: Some("below zero".into()),
        }];
        let gate = GateResult::from_results(results);
        assert!(!gate.passed);
        assert_eq!(gate.errors, 1);
        assert_eq!(gate.warnings, 0);
    }

    #[test]
    fn ratchet_gate_result_from_single_passed() {
        let results = vec![RatchetResult {
            rule: make_ratchet_rule("/x", Some(10.0), None),
            passed: true,
            baseline_value: Some(10.0),
            current_value: 10.5,
            change_pct: Some(5.0),
            message: "ok".into(),
        }];
        let gate = RatchetGateResult::from_results(results);
        assert!(gate.passed);
        assert_eq!(gate.errors, 0);
    }
}

// =========================================================================
// 9. Contains on edge-case strings and arrays
// =========================================================================

mod contains_deep {
    use super::*;

    #[test]
    fn contains_unicode_substring() {
        let receipt = json!({"text": "hello 世界"});
        let rule = make_rule("test", "/text", RuleOperator::Contains, json!("世界"));
        assert!(eval_single(&receipt, &rule).passed);
    }

    #[test]
    fn contains_array_with_mixed_types() {
        let receipt = json!({"arr": [1, "two", true, null]});

        assert!(
            eval_single(
                &receipt,
                &make_rule("str", "/arr", RuleOperator::Contains, json!("two"))
            )
            .passed
        );
        assert!(
            eval_single(
                &receipt,
                &make_rule("num", "/arr", RuleOperator::Contains, json!(1))
            )
            .passed
        );
        assert!(
            eval_single(
                &receipt,
                &make_rule("bool", "/arr", RuleOperator::Contains, json!(true))
            )
            .passed
        );
    }

    #[test]
    fn contains_on_nested_array_element_via_pointer() {
        let receipt = json!({"data": {"tags": ["alpha", "beta"]}});
        let rule = make_rule("test", "/data/tags", RuleOperator::Contains, json!("beta"));
        assert!(eval_single(&receipt, &rule).passed);
    }
}

// =========================================================================
// 10. In operator with mixed-type value lists
// =========================================================================

mod in_operator_deep {
    use super::*;

    #[test]
    fn in_with_mixed_numeric_types() {
        // Integer actual vs float in values list
        let receipt = json!({"val": 42});
        let rule = PolicyRule {
            name: "test".into(),
            pointer: "/val".into(),
            op: RuleOperator::In,
            value: None,
            values: Some(vec![json!(42.0), json!(43.0)]),
            negate: false,
            level: RuleLevel::Error,
            message: None,
        };
        assert!(eval_single(&receipt, &rule).passed);
    }

    #[test]
    fn in_with_boolean_values() {
        let receipt = json!({"flag": true});
        let rule = PolicyRule {
            name: "test".into(),
            pointer: "/flag".into(),
            op: RuleOperator::In,
            value: None,
            values: Some(vec![json!(true), json!(false)]),
            negate: false,
            level: RuleLevel::Error,
            message: None,
        };
        assert!(eval_single(&receipt, &rule).passed);
    }

    #[test]
    fn in_with_null_in_list() {
        let receipt = json!({"val": null});
        let rule = PolicyRule {
            name: "test".into(),
            pointer: "/val".into(),
            op: RuleOperator::In,
            value: None,
            values: Some(vec![json!(null), json!("other")]),
            negate: false,
            level: RuleLevel::Error,
            message: None,
        };
        assert!(eval_single(&receipt, &rule).passed);
    }
}

// =========================================================================
// 11. Ratchet evaluation edge cases
// =========================================================================

mod ratchet_deep {
    use super::*;

    #[test]
    fn ratchet_negative_values_decrease_is_ok() {
        let baseline = json!({"val": -10.0});
        let current = json!({"val": -15.0}); // Went more negative (decrease)
        let config = RatchetConfig {
            rules: vec![make_ratchet_rule("/val", Some(10.0), None)],
            fail_fast: false,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };
        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        // -15 from -10 is a 50% decrease in absolute terms,
        // but the percentage is (-15 - (-10)) / (-10) * 100 = 50%
        // So the change is +50% (it got worse in negative direction)
        assert!(!result.passed);
    }

    #[test]
    fn ratchet_both_missing_baseline_and_current_allowed() {
        let baseline = json!({});
        let current = json!({});
        let config = RatchetConfig {
            rules: vec![make_ratchet_rule("/missing", Some(10.0), None)],
            fail_fast: false,
            allow_missing_baseline: true,
            allow_missing_current: true,
        };
        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        // Missing current is allowed → pass
        assert!(result.passed);
    }

    #[test]
    fn ratchet_max_value_zero() {
        let baseline = json!({"val": 0});
        let current = json!({"val": 0});
        let config = RatchetConfig {
            rules: vec![make_ratchet_rule("/val", None, Some(0.0))],
            fail_fast: false,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };
        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        // val=0 is not > max_value=0, so passes
        assert!(result.passed);
    }

    #[test]
    fn ratchet_max_value_just_exceeded() {
        let baseline = json!({"val": 50});
        let current = json!({"val": 100.001});
        let config = RatchetConfig {
            rules: vec![make_ratchet_rule("/val", None, Some(100.0))],
            fail_fast: false,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };
        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(!result.passed);
    }

    #[test]
    fn ratchet_percentage_calculation_accuracy() {
        let baseline = json!({"val": 200.0});
        let current = json!({"val": 210.0}); // 5% increase

        let config = RatchetConfig {
            rules: vec![make_ratchet_rule("/val", Some(10.0), None)],
            fail_fast: false,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };
        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(result.passed);

        let rr = &result.ratchet_results[0];
        assert_eq!(rr.baseline_value, Some(200.0));
        assert_eq!(rr.current_value, 210.0);
        let pct = rr.change_pct.unwrap();
        assert!((pct - 5.0).abs() < 0.001, "Expected ~5%, got {}", pct);
    }

    #[test]
    fn ratchet_no_constraints_at_all_passes() {
        // Rule with neither max_increase_pct nor max_value
        let baseline = json!({"val": 100});
        let current = json!({"val": 9999});
        let config = RatchetConfig {
            rules: vec![make_ratchet_rule("/val", None, None)],
            fail_fast: false,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };
        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(result.passed, "No constraints means always pass");
    }

    #[test]
    fn ratchet_warn_level_does_not_block() {
        let baseline = json!({"val": 10.0});
        let current = json!({"val": 100.0}); // 900% increase
        let config = RatchetConfig {
            rules: vec![RatchetRule {
                pointer: "/val".into(),
                max_increase_pct: Some(5.0),
                max_value: None,
                level: RuleLevel::Warn,
                description: None,
            }],
            fail_fast: false,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };
        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(result.passed, "Warn-level ratchet failures don't block");
        assert_eq!(result.warnings, 1);
        assert_eq!(result.errors, 0);
    }

    #[test]
    fn ratchet_fail_fast_does_not_stop_on_warn() {
        let baseline = json!({"a": 10.0, "b": 10.0});
        let current = json!({"a": 100.0, "b": 100.0}); // Both huge increases

        let config = RatchetConfig {
            rules: vec![
                RatchetRule {
                    pointer: "/a".into(),
                    max_increase_pct: Some(5.0),
                    max_value: None,
                    level: RuleLevel::Warn, // Warn, not error
                    description: None,
                },
                RatchetRule {
                    pointer: "/b".into(),
                    max_increase_pct: Some(5.0),
                    max_value: None,
                    level: RuleLevel::Error, // Error
                    description: None,
                },
            ],
            fail_fast: true,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };
        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(!result.passed);
        // fail_fast should not stop on warn, so both rules evaluated
        assert_eq!(result.ratchet_results.len(), 2);
        assert_eq!(result.warnings, 1);
        assert_eq!(result.errors, 1);
    }
}

// =========================================================================
// 12. Serialization / deserialization roundtrips
// =========================================================================

mod serde_deep {
    use super::*;

    #[test]
    fn policy_rule_all_operators_roundtrip_json() {
        let operators = [
            RuleOperator::Gt,
            RuleOperator::Gte,
            RuleOperator::Lt,
            RuleOperator::Lte,
            RuleOperator::Eq,
            RuleOperator::Ne,
            RuleOperator::In,
            RuleOperator::Contains,
            RuleOperator::Exists,
        ];
        for op in &operators {
            let json_str = serde_json::to_string(op).unwrap();
            let back: RuleOperator = serde_json::from_str(&json_str).unwrap();
            assert_eq!(*op, back, "roundtrip failed for {:?}", op);
        }
    }

    #[test]
    fn rule_level_roundtrip_json() {
        for level in &[RuleLevel::Error, RuleLevel::Warn] {
            let json_str = serde_json::to_string(level).unwrap();
            let back: RuleLevel = serde_json::from_str(&json_str).unwrap();
            assert_eq!(*level, back);
        }
    }

    #[test]
    fn full_policy_config_roundtrip_toml_json() {
        let toml = r#"
fail_fast = true
allow_missing = true

[[rules]]
name = "check1"
pointer = "/a/b"
op = "gte"
value = 100
level = "error"
message = "Too low"
negate = true

[[rules]]
name = "check2"
pointer = "/tags"
op = "in"
values = ["x", "y"]
level = "warn"
"#;
        let config = PolicyConfig::from_toml(toml).unwrap();

        // Roundtrip through JSON
        let json_str = serde_json::to_string(&config).unwrap();
        let back: PolicyConfig = serde_json::from_str(&json_str).unwrap();

        assert_eq!(back.fail_fast, config.fail_fast);
        assert_eq!(back.allow_missing, config.allow_missing);
        assert_eq!(back.rules.len(), config.rules.len());
        assert_eq!(back.rules[0].name, "check1");
        assert_eq!(back.rules[0].op, RuleOperator::Gte);
        assert!(back.rules[0].negate);
        assert_eq!(back.rules[1].op, RuleOperator::In);
        assert_eq!(back.rules[1].level, RuleLevel::Warn);
    }

    #[test]
    fn ratchet_config_roundtrip_toml_json() {
        let toml = r#"
fail_fast = false
allow_missing_baseline = true
allow_missing_current = false

[[rules]]
pointer = "/complexity"
max_increase_pct = 5.0
max_value = 20.0
level = "error"
description = "Keep it low"
"#;
        let config = RatchetConfig::from_toml(toml).unwrap();
        let json_str = serde_json::to_string(&config).unwrap();
        let back: RatchetConfig = serde_json::from_str(&json_str).unwrap();

        assert_eq!(back.rules.len(), 1);
        assert!(back.allow_missing_baseline);
        assert!(!back.allow_missing_current);
        assert_eq!(back.rules[0].max_increase_pct, Some(5.0));
        assert_eq!(back.rules[0].max_value, Some(20.0));
        assert_eq!(back.rules[0].description.as_deref(), Some("Keep it low"));
    }

    #[test]
    fn gate_result_with_mixed_results_roundtrip() {
        let gate = GateResult::from_results(vec![
            RuleResult {
                name: "pass_err".into(),
                passed: true,
                level: RuleLevel::Error,
                actual: Some(json!(42)),
                expected: "/n >= 0".into(),
                message: None,
            },
            RuleResult {
                name: "fail_warn".into(),
                passed: false,
                level: RuleLevel::Warn,
                actual: Some(json!("bad")),
                expected: "/s == good".into(),
                message: Some("not good".into()),
            },
            RuleResult {
                name: "fail_err".into(),
                passed: false,
                level: RuleLevel::Error,
                actual: None,
                expected: "/missing exists".into(),
                message: Some("not found".into()),
            },
        ]);

        let json_str = serde_json::to_string(&gate).unwrap();
        let back: GateResult = serde_json::from_str(&json_str).unwrap();
        assert!(!back.passed);
        assert_eq!(back.errors, 1);
        assert_eq!(back.warnings, 1);
        assert_eq!(back.rule_results.len(), 3);
    }
}

// =========================================================================
// 13. RatchetConfig from_file roundtrip
// =========================================================================

mod ratchet_file {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn ratchet_config_from_file_roundtrip() {
        let toml = r#"
fail_fast = true
allow_missing_baseline = false
allow_missing_current = true

[[rules]]
pointer = "/loc"
max_increase_pct = 15.0
level = "warn"
description = "LOC growth"
"#;
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("tokmd-gate-ratchet-deep-{nanos}.toml"));
        std::fs::write(&path, toml).unwrap();

        let config = RatchetConfig::from_file(&path).unwrap();
        let _ = std::fs::remove_file(&path);

        assert!(config.fail_fast);
        assert!(!config.allow_missing_baseline);
        assert!(config.allow_missing_current);
        assert_eq!(config.rules.len(), 1);
        assert_eq!(config.rules[0].pointer, "/loc");
        assert_eq!(config.rules[0].level, RuleLevel::Warn);
    }
}

// =========================================================================
// 14. Property test: valid pointer always resolves or returns None
// =========================================================================

mod pointer_property {
    use super::*;

    #[test]
    fn valid_pointer_on_matching_structure_always_resolves() {
        // Build a document and verify pointers resolve correctly
        let doc = json!({
            "a": {"b": {"c": 1}},
            "d": [10, 20, 30],
            "e": null,
            "f": true,
            "g": "text"
        });

        let cases: Vec<(&str, serde_json::Value)> = vec![
            ("/a/b/c", json!(1)),
            ("/d/0", json!(10)),
            ("/d/2", json!(30)),
            ("/e", json!(null)),
            ("/f", json!(true)),
            ("/g", json!("text")),
        ];

        for (pointer, expected) in &cases {
            let result = resolve_pointer(&doc, pointer);
            assert_eq!(
                result,
                Some(expected),
                "Pointer {} should resolve to {:?}",
                pointer,
                expected
            );
        }
    }

    #[test]
    fn invalid_pointers_always_return_none() {
        let doc = json!({"a": 1});
        let invalid = vec![
            "no_leading_slash",
            "/nonexistent",
            "/a/b", // a is a number, not an object
            "/a/0", // a is not an array
        ];

        for pointer in &invalid {
            assert_eq!(
                resolve_pointer(&doc, pointer),
                None,
                "Pointer {} should return None",
                pointer
            );
        }
    }
}

// =========================================================================
// 15. Fail-fast interaction with warn-only rules
// =========================================================================

mod fail_fast_deep {
    use super::*;

    #[test]
    fn fail_fast_continues_past_warn_failure_to_reach_error() {
        let receipt = json!({"a": 100, "b": 100, "c": 100});
        let policy = PolicyConfig {
            rules: vec![
                PolicyRule {
                    level: RuleLevel::Warn,
                    ..make_rule("w1", "/a", RuleOperator::Lt, json!(50))
                },
                PolicyRule {
                    level: RuleLevel::Warn,
                    ..make_rule("w2", "/b", RuleOperator::Lt, json!(50))
                },
                PolicyRule {
                    level: RuleLevel::Error,
                    ..make_rule("e1", "/c", RuleOperator::Lt, json!(50))
                },
            ],
            fail_fast: true,
            allow_missing: false,
        };

        let result = evaluate_policy(&receipt, &policy);
        assert!(!result.passed);
        // All 3 rules should be evaluated: 2 warns don't stop, error does
        assert_eq!(result.rule_results.len(), 3);
        assert_eq!(result.warnings, 2);
        assert_eq!(result.errors, 1);
    }
}
