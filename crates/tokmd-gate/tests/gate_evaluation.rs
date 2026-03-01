//! Deeper tests for gate policy evaluation, JSON pointer resolution,
//! operator behavior, and threshold semantics.

use serde_json::json;
use tokmd_gate::{
    PolicyConfig, PolicyRule, RatchetConfig, RatchetRule, RuleLevel, RuleOperator, evaluate_policy,
    evaluate_ratchet_policy, resolve_pointer,
};

// ============================================================================
// Helpers
// ============================================================================

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

fn make_warn_rule(
    name: &str,
    pointer: &str,
    op: RuleOperator,
    value: serde_json::Value,
) -> PolicyRule {
    PolicyRule {
        name: name.into(),
        pointer: pointer.into(),
        op,
        value: Some(value),
        values: None,
        negate: false,
        level: RuleLevel::Warn,
        message: None,
    }
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

// ============================================================================
// JSON Pointer: complex nested paths
// ============================================================================

#[test]
fn pointer_deeply_nested() {
    let doc = json!({
        "a": {"b": {"c": {"d": {"e": 42}}}}
    });
    assert_eq!(resolve_pointer(&doc, "/a/b/c/d/e"), Some(&json!(42)));
}

#[test]
fn pointer_array_within_object() {
    let doc = json!({
        "data": {
            "items": [
                {"name": "first", "value": 1},
                {"name": "second", "value": 2}
            ]
        }
    });
    assert_eq!(
        resolve_pointer(&doc, "/data/items/0/name"),
        Some(&json!("first"))
    );
    assert_eq!(
        resolve_pointer(&doc, "/data/items/1/value"),
        Some(&json!(2))
    );
}

#[test]
fn pointer_nested_arrays() {
    let doc = json!({
        "grid": [[10, 20, 30], [40, 50, 60]]
    });
    assert_eq!(resolve_pointer(&doc, "/grid/0/2"), Some(&json!(30)));
    assert_eq!(resolve_pointer(&doc, "/grid/1/0"), Some(&json!(40)));
}

#[test]
fn pointer_missing_intermediate_key() {
    let doc = json!({"a": {"b": 1}});
    assert_eq!(resolve_pointer(&doc, "/a/c/d"), None);
}

#[test]
fn pointer_array_out_of_bounds() {
    let doc = json!({"items": [1, 2, 3]});
    assert_eq!(resolve_pointer(&doc, "/items/5"), None);
}

#[test]
fn pointer_into_scalar() {
    let doc = json!({"x": 42});
    assert_eq!(resolve_pointer(&doc, "/x/deeper"), None);
}

#[test]
fn pointer_empty_key() {
    // RFC 6901: "/" refers to the empty string key
    let doc = json!({"": "empty_key_value"});
    assert_eq!(resolve_pointer(&doc, "/"), Some(&json!("empty_key_value")));
}

#[test]
fn pointer_rfc6901_escaping() {
    // ~0 → ~, ~1 → /
    let doc = json!({"a/b": {"c~d": 99}});
    assert_eq!(resolve_pointer(&doc, "/a~1b/c~0d"), Some(&json!(99)));
}

#[test]
fn pointer_empty_string_returns_whole_doc() {
    let doc = json!({"any": "thing"});
    assert_eq!(resolve_pointer(&doc, ""), Some(&doc));
}

#[test]
fn pointer_no_leading_slash_returns_none() {
    let doc = json!({"a": 1});
    assert_eq!(resolve_pointer(&doc, "a"), None);
}

#[test]
fn pointer_null_value() {
    let doc = json!({"nullable": null});
    assert_eq!(resolve_pointer(&doc, "/nullable"), Some(&json!(null)));
}

#[test]
fn pointer_boolean_value() {
    let doc = json!({"flag": true});
    assert_eq!(resolve_pointer(&doc, "/flag"), Some(&json!(true)));
}

// ============================================================================
// Operator: gt, lt, gte, lte
// ============================================================================

#[test]
fn gt_strictly_greater() {
    let receipt = json!({"n": 10});
    // 10 > 10 = false
    assert!(
        !evaluate_policy(
            &receipt,
            &PolicyConfig {
                rules: vec![make_rule("gt", "/n", RuleOperator::Gt, json!(10))],
                ..Default::default()
            }
        )
        .passed
    );
    // 10 > 9 = true
    assert!(
        evaluate_policy(
            &receipt,
            &PolicyConfig {
                rules: vec![make_rule("gt", "/n", RuleOperator::Gt, json!(9))],
                ..Default::default()
            }
        )
        .passed
    );
}

#[test]
fn lt_strictly_less() {
    let receipt = json!({"n": 10});
    // 10 < 10 = false
    assert!(
        !evaluate_policy(
            &receipt,
            &PolicyConfig {
                rules: vec![make_rule("lt", "/n", RuleOperator::Lt, json!(10))],
                ..Default::default()
            }
        )
        .passed
    );
    // 10 < 11 = true
    assert!(
        evaluate_policy(
            &receipt,
            &PolicyConfig {
                rules: vec![make_rule("lt", "/n", RuleOperator::Lt, json!(11))],
                ..Default::default()
            }
        )
        .passed
    );
}

#[test]
fn gte_includes_equal() {
    let receipt = json!({"n": 10});
    assert!(
        evaluate_policy(
            &receipt,
            &PolicyConfig {
                rules: vec![make_rule("gte", "/n", RuleOperator::Gte, json!(10))],
                ..Default::default()
            }
        )
        .passed
    );
    assert!(
        evaluate_policy(
            &receipt,
            &PolicyConfig {
                rules: vec![make_rule("gte", "/n", RuleOperator::Gte, json!(9))],
                ..Default::default()
            }
        )
        .passed
    );
    assert!(
        !evaluate_policy(
            &receipt,
            &PolicyConfig {
                rules: vec![make_rule("gte", "/n", RuleOperator::Gte, json!(11))],
                ..Default::default()
            }
        )
        .passed
    );
}

#[test]
fn lte_includes_equal() {
    let receipt = json!({"n": 10});
    assert!(
        evaluate_policy(
            &receipt,
            &PolicyConfig {
                rules: vec![make_rule("lte", "/n", RuleOperator::Lte, json!(10))],
                ..Default::default()
            }
        )
        .passed
    );
    assert!(
        evaluate_policy(
            &receipt,
            &PolicyConfig {
                rules: vec![make_rule("lte", "/n", RuleOperator::Lte, json!(11))],
                ..Default::default()
            }
        )
        .passed
    );
    assert!(
        !evaluate_policy(
            &receipt,
            &PolicyConfig {
                rules: vec![make_rule("lte", "/n", RuleOperator::Lte, json!(9))],
                ..Default::default()
            }
        )
        .passed
    );
}

// ============================================================================
// Operator: eq, ne
// ============================================================================

#[test]
fn eq_string_case_sensitive() {
    let receipt = json!({"name": "Rust"});
    assert!(
        evaluate_policy(
            &receipt,
            &PolicyConfig {
                rules: vec![make_rule("eq", "/name", RuleOperator::Eq, json!("Rust"))],
                ..Default::default()
            }
        )
        .passed
    );
    // Case mismatch
    assert!(
        !evaluate_policy(
            &receipt,
            &PolicyConfig {
                rules: vec![make_rule("eq", "/name", RuleOperator::Eq, json!("rust"))],
                ..Default::default()
            }
        )
        .passed
    );
}

#[test]
fn eq_integer_float_crossover() {
    let receipt = json!({"n": 42});
    // 42 (int) == 42.0 (float) should be equal via f64 comparison
    assert!(
        evaluate_policy(
            &receipt,
            &PolicyConfig {
                rules: vec![make_rule("eq", "/n", RuleOperator::Eq, json!(42.0))],
                ..Default::default()
            }
        )
        .passed
    );
}

#[test]
fn eq_boolean() {
    let receipt = json!({"flag": true});
    assert!(
        evaluate_policy(
            &receipt,
            &PolicyConfig {
                rules: vec![make_rule("eq", "/flag", RuleOperator::Eq, json!(true))],
                ..Default::default()
            }
        )
        .passed
    );
    assert!(
        !evaluate_policy(
            &receipt,
            &PolicyConfig {
                rules: vec![make_rule("eq", "/flag", RuleOperator::Eq, json!(false))],
                ..Default::default()
            }
        )
        .passed
    );
}

#[test]
fn ne_operator() {
    let receipt = json!({"x": 5});
    assert!(
        evaluate_policy(
            &receipt,
            &PolicyConfig {
                rules: vec![make_rule("ne", "/x", RuleOperator::Ne, json!(10))],
                ..Default::default()
            }
        )
        .passed
    );
    assert!(
        !evaluate_policy(
            &receipt,
            &PolicyConfig {
                rules: vec![make_rule("ne", "/x", RuleOperator::Ne, json!(5))],
                ..Default::default()
            }
        )
        .passed
    );
}

// ============================================================================
// Operator: contains
// ============================================================================

#[test]
fn contains_string_substring() {
    let receipt = json!({"desc": "hello world foo"});
    assert!(
        evaluate_policy(
            &receipt,
            &PolicyConfig {
                rules: vec![make_rule(
                    "c",
                    "/desc",
                    RuleOperator::Contains,
                    json!("world"),
                )],
                ..Default::default()
            }
        )
        .passed
    );
    assert!(
        !evaluate_policy(
            &receipt,
            &PolicyConfig {
                rules: vec![make_rule(
                    "c",
                    "/desc",
                    RuleOperator::Contains,
                    json!("missing"),
                )],
                ..Default::default()
            }
        )
        .passed
    );
}

#[test]
fn contains_array_element() {
    let receipt = json!({"tags": ["rust", "cli", "tools"]});
    assert!(
        evaluate_policy(
            &receipt,
            &PolicyConfig {
                rules: vec![make_rule(
                    "c",
                    "/tags",
                    RuleOperator::Contains,
                    json!("cli"),
                )],
                ..Default::default()
            }
        )
        .passed
    );
    assert!(
        !evaluate_policy(
            &receipt,
            &PolicyConfig {
                rules: vec![make_rule(
                    "c",
                    "/tags",
                    RuleOperator::Contains,
                    json!("python"),
                )],
                ..Default::default()
            }
        )
        .passed
    );
}

#[test]
fn contains_array_numeric_element() {
    let receipt = json!({"codes": [200, 404, 500]});
    assert!(
        evaluate_policy(
            &receipt,
            &PolicyConfig {
                rules: vec![make_rule("c", "/codes", RuleOperator::Contains, json!(404),)],
                ..Default::default()
            }
        )
        .passed
    );
}

#[test]
fn contains_on_non_string_non_array_fails() {
    // contains on a number should fail (not pass)
    let receipt = json!({"n": 42});
    let result = evaluate_policy(
        &receipt,
        &PolicyConfig {
            rules: vec![make_rule("c", "/n", RuleOperator::Contains, json!(4))],
            ..Default::default()
        },
    );
    assert!(!result.passed);
}

// ============================================================================
// Operator: in
// ============================================================================

#[test]
fn in_operator_string_match() {
    let receipt = json!({"license": "MIT"});
    let rule = PolicyRule {
        name: "in_test".into(),
        pointer: "/license".into(),
        op: RuleOperator::In,
        value: None,
        values: Some(vec![
            json!("MIT"),
            json!("Apache-2.0"),
            json!("BSD-3-Clause"),
        ]),
        negate: false,
        level: RuleLevel::Error,
        message: None,
    };
    let result = evaluate_policy(
        &receipt,
        &PolicyConfig {
            rules: vec![rule],
            ..Default::default()
        },
    );
    assert!(result.passed);
}

#[test]
fn in_operator_numeric_match() {
    let receipt = json!({"code": 200});
    let rule = PolicyRule {
        name: "in_test".into(),
        pointer: "/code".into(),
        op: RuleOperator::In,
        value: None,
        values: Some(vec![json!(200), json!(201), json!(204)]),
        negate: false,
        level: RuleLevel::Error,
        message: None,
    };
    let result = evaluate_policy(
        &receipt,
        &PolicyConfig {
            rules: vec![rule],
            ..Default::default()
        },
    );
    assert!(result.passed);
}

#[test]
fn in_operator_no_match() {
    let receipt = json!({"license": "GPL"});
    let rule = PolicyRule {
        name: "in_test".into(),
        pointer: "/license".into(),
        op: RuleOperator::In,
        value: None,
        values: Some(vec![json!("MIT"), json!("Apache-2.0")]),
        negate: false,
        level: RuleLevel::Error,
        message: None,
    };
    let result = evaluate_policy(
        &receipt,
        &PolicyConfig {
            rules: vec![rule],
            ..Default::default()
        },
    );
    assert!(!result.passed);
}

// ============================================================================
// Operator: exists
// ============================================================================

#[test]
fn exists_present_field() {
    let receipt = json!({"metadata": {"license": "MIT"}});
    let rule = PolicyRule {
        name: "exists".into(),
        pointer: "/metadata/license".into(),
        op: RuleOperator::Exists,
        value: None,
        values: None,
        negate: false,
        level: RuleLevel::Error,
        message: None,
    };
    let result = evaluate_policy(
        &receipt,
        &PolicyConfig {
            rules: vec![rule],
            ..Default::default()
        },
    );
    assert!(result.passed);
}

#[test]
fn exists_absent_field() {
    let receipt = json!({"metadata": {}});
    let rule = PolicyRule {
        name: "exists".into(),
        pointer: "/metadata/license".into(),
        op: RuleOperator::Exists,
        value: None,
        values: None,
        negate: false,
        level: RuleLevel::Error,
        message: None,
    };
    let result = evaluate_policy(
        &receipt,
        &PolicyConfig {
            rules: vec![rule],
            ..Default::default()
        },
    );
    assert!(!result.passed);
}

#[test]
fn exists_negated_absent_is_pass() {
    let receipt = json!({"data": {}});
    let rule = PolicyRule {
        name: "not_exists".into(),
        pointer: "/data/secrets".into(),
        op: RuleOperator::Exists,
        value: None,
        values: None,
        negate: true,
        level: RuleLevel::Error,
        message: None,
    };
    let result = evaluate_policy(
        &receipt,
        &PolicyConfig {
            rules: vec![rule],
            ..Default::default()
        },
    );
    assert!(result.passed);
}

#[test]
fn exists_null_value_counts_as_present() {
    let receipt = json!({"nullable": null});
    let rule = PolicyRule {
        name: "exists_null".into(),
        pointer: "/nullable".into(),
        op: RuleOperator::Exists,
        value: None,
        values: None,
        negate: false,
        level: RuleLevel::Error,
        message: None,
    };
    let result = evaluate_policy(
        &receipt,
        &PolicyConfig {
            rules: vec![rule],
            ..Default::default()
        },
    );
    assert!(result.passed, "null value should count as existing");
}

// ============================================================================
// Type coercion edge cases
// ============================================================================

#[test]
fn string_to_number_coercion() {
    let receipt = json!({"value": "42"});
    let result = evaluate_policy(
        &receipt,
        &PolicyConfig {
            rules: vec![make_rule("gt", "/value", RuleOperator::Gt, json!(41))],
            ..Default::default()
        },
    );
    assert!(result.passed, "string '42' should coerce to numeric 42");
}

#[test]
fn float_string_coercion() {
    let receipt = json!({"ratio": "0.75"});
    let result = evaluate_policy(
        &receipt,
        &PolicyConfig {
            rules: vec![make_rule("gt", "/ratio", RuleOperator::Gt, json!(0.5))],
            ..Default::default()
        },
    );
    assert!(result.passed);
}

#[test]
fn non_numeric_string_comparison_fails() {
    let receipt = json!({"name": "hello"});
    let result = evaluate_policy(
        &receipt,
        &PolicyConfig {
            rules: vec![make_rule("gt", "/name", RuleOperator::Gt, json!(5))],
            ..Default::default()
        },
    );
    assert!(
        !result.passed,
        "non-numeric string should fail numeric comparison"
    );
}

#[test]
fn eq_array_values() {
    let receipt = json!({"data": [1, 2, 3]});
    let result = evaluate_policy(
        &receipt,
        &PolicyConfig {
            rules: vec![make_rule("eq", "/data", RuleOperator::Eq, json!([1, 2, 3]))],
            ..Default::default()
        },
    );
    assert!(result.passed);
}

#[test]
fn eq_object_values() {
    let receipt = json!({"meta": {"k": "v"}});
    let result = evaluate_policy(
        &receipt,
        &PolicyConfig {
            rules: vec![make_rule(
                "eq",
                "/meta",
                RuleOperator::Eq,
                json!({"k": "v"}),
            )],
            ..Default::default()
        },
    );
    assert!(result.passed);
}

// ============================================================================
// Missing fields behavior
// ============================================================================

#[test]
fn missing_field_without_allow_missing_fails() {
    let receipt = json!({"a": 1});
    let policy = PolicyConfig {
        rules: vec![make_rule("check", "/missing", RuleOperator::Eq, json!(1))],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
    assert!(
        result.rule_results[0]
            .message
            .as_ref()
            .unwrap()
            .contains("not found")
    );
}

#[test]
fn missing_field_with_allow_missing_passes() {
    let receipt = json!({"a": 1});
    let policy = PolicyConfig {
        rules: vec![make_rule("check", "/missing", RuleOperator::Eq, json!(1))],
        fail_fast: false,
        allow_missing: true,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed);
}

#[test]
fn missing_deep_field() {
    let receipt = json!({"a": {"b": 1}});
    let policy = PolicyConfig {
        rules: vec![make_rule("deep", "/a/b/c/d/e", RuleOperator::Eq, json!(1))],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
}

// ============================================================================
// Gate pass/fail/warn threshold behavior
// ============================================================================

#[test]
fn warn_rules_do_not_fail_gate() {
    let receipt = json!({"tokens": 1000000});
    let policy = PolicyConfig {
        rules: vec![make_warn_rule(
            "token_warn",
            "/tokens",
            RuleOperator::Lte,
            json!(500000),
        )],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed, "warn-level rule should not fail the gate");
    assert_eq!(result.warnings, 1);
    assert_eq!(result.errors, 0);
}

#[test]
fn mixed_error_and_warn_only_errors_fail() {
    let receipt = json!({"a": 100, "b": 200});
    let policy = PolicyConfig {
        rules: vec![
            // Error rule: passes
            make_rule("a_ok", "/a", RuleOperator::Lte, json!(200)),
            // Warn rule: fails (200 > 150)
            make_warn_rule("b_high", "/b", RuleOperator::Lte, json!(150)),
        ],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed, "only warn failed, gate should pass");
    assert_eq!(result.warnings, 1);
    assert_eq!(result.errors, 0);
}

#[test]
fn single_error_failure_fails_gate() {
    let receipt = json!({"a": 100, "b": 200});
    let policy = PolicyConfig {
        rules: vec![
            make_rule("a_ok", "/a", RuleOperator::Lte, json!(200)),
            make_rule("b_fail", "/b", RuleOperator::Lte, json!(150)),
        ],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
}

#[test]
fn empty_policy_always_passes() {
    let receipt = json!({"anything": true});
    let policy = PolicyConfig::default();
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed);
    assert_eq!(result.rule_results.len(), 0);
}

// ============================================================================
// fail_fast semantics
// ============================================================================

#[test]
fn fail_fast_stops_on_first_error() {
    let receipt = json!({"a": 100, "b": 200, "c": 300});
    let policy = PolicyConfig {
        rules: vec![
            make_rule("a_fail", "/a", RuleOperator::Gt, json!(200)),
            make_rule("b_fail", "/b", RuleOperator::Gt, json!(300)),
            make_rule("c_fail", "/c", RuleOperator::Gt, json!(400)),
        ],
        fail_fast: true,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
    assert_eq!(
        result.rule_results.len(),
        1,
        "should stop after first error"
    );
}

#[test]
fn fail_fast_does_not_stop_on_warn() {
    let receipt = json!({"a": 100, "b": 200});
    let policy = PolicyConfig {
        rules: vec![
            make_warn_rule("a_warn", "/a", RuleOperator::Gt, json!(200)),
            make_rule("b_fail", "/b", RuleOperator::Gt, json!(300)),
        ],
        fail_fast: true,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    // warn doesn't trigger fail_fast, so both rules should be evaluated
    assert!(!result.passed);
    assert_eq!(result.rule_results.len(), 2);
}

// ============================================================================
// Negate modifier
// ============================================================================

#[test]
fn negate_inverts_gt_result() {
    let receipt = json!({"n": 100});
    let rule = PolicyRule {
        name: "negate_gt".into(),
        pointer: "/n".into(),
        op: RuleOperator::Gt,
        value: Some(json!(50)),
        values: None,
        negate: true,
        level: RuleLevel::Error,
        message: None,
    };
    let result = evaluate_policy(
        &receipt,
        &PolicyConfig {
            rules: vec![rule],
            ..Default::default()
        },
    );
    assert!(!result.passed, "100 > 50 negated should fail");
}

#[test]
fn negate_inverts_contains() {
    let receipt = json!({"tags": ["rust", "cli"]});
    let rule = PolicyRule {
        name: "no_python".into(),
        pointer: "/tags".into(),
        op: RuleOperator::Contains,
        value: Some(json!("python")),
        values: None,
        negate: true,
        level: RuleLevel::Error,
        message: None,
    };
    let result = evaluate_policy(
        &receipt,
        &PolicyConfig {
            rules: vec![rule],
            ..Default::default()
        },
    );
    assert!(result.passed, "tags don't contain 'python', negated → pass");
}

// ============================================================================
// Ratchet gate: threshold behavior
// ============================================================================

#[test]
fn ratchet_pass_when_within_threshold() {
    let baseline = json!({"complexity": 10.0});
    let current = json!({"complexity": 10.5}); // 5% increase
    let config = RatchetConfig {
        rules: vec![make_ratchet_rule("/complexity", Some(10.0), None)],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(result.passed);
}

#[test]
fn ratchet_fail_when_over_threshold() {
    let baseline = json!({"complexity": 10.0});
    let current = json!({"complexity": 12.0}); // 20% > 10% limit
    let config = RatchetConfig {
        rules: vec![make_ratchet_rule("/complexity", Some(10.0), None)],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
}

#[test]
fn ratchet_max_value_ceiling() {
    let baseline = json!({"tokens": 400000});
    let current = json!({"tokens": 600000}); // Under 100% increase, but over max
    let config = RatchetConfig {
        rules: vec![make_ratchet_rule("/tokens", Some(100.0), Some(500000.0))],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(!result.passed, "should fail due to max_value ceiling");
}

#[test]
fn ratchet_warn_does_not_fail() {
    let baseline = json!({"complexity": 10.0});
    let current = json!({"complexity": 20.0}); // 100% increase
    let config = RatchetConfig {
        rules: vec![RatchetRule {
            pointer: "/complexity".to_string(),
            max_increase_pct: Some(10.0),
            max_value: None,
            level: RuleLevel::Warn,
            description: None,
        }],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(result.passed, "warn-level ratchet should not fail gate");
    assert_eq!(result.warnings, 1);
}

#[test]
fn ratchet_missing_baseline_strict_fails() {
    let baseline = json!({});
    let current = json!({"complexity": 5.0});
    let config = RatchetConfig {
        rules: vec![make_ratchet_rule("/complexity", Some(10.0), None)],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(!result.passed);
}

#[test]
fn ratchet_missing_baseline_lenient_passes() {
    let baseline = json!({});
    let current = json!({"complexity": 5.0});
    let config = RatchetConfig {
        rules: vec![make_ratchet_rule("/complexity", Some(10.0), None)],
        fail_fast: false,
        allow_missing_baseline: true,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(result.passed);
}

#[test]
fn ratchet_missing_current_strict_fails() {
    let baseline = json!({"complexity": 5.0});
    let current = json!({});
    let config = RatchetConfig {
        rules: vec![make_ratchet_rule("/complexity", Some(10.0), None)],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(!result.passed);
}

#[test]
fn ratchet_missing_current_lenient_passes() {
    let baseline = json!({"complexity": 5.0});
    let current = json!({});
    let config = RatchetConfig {
        rules: vec![make_ratchet_rule("/complexity", Some(10.0), None)],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: true,
    };
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(result.passed);
}

#[test]
fn ratchet_zero_baseline_same_current() {
    let baseline = json!({"count": 0});
    let current = json!({"count": 0});
    let config = RatchetConfig {
        rules: vec![make_ratchet_rule("/count", Some(10.0), None)],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(result.passed);
}

#[test]
fn ratchet_zero_baseline_nonzero_current_fails() {
    let baseline = json!({"count": 0});
    let current = json!({"count": 1});
    let config = RatchetConfig {
        rules: vec![make_ratchet_rule("/count", Some(10.0), None)],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    // 0 → 1 = infinite increase, should fail
    assert!(!result.passed);
}

// ============================================================================
// Complex nested pointer paths in policy evaluation
// ============================================================================

#[test]
fn policy_with_deep_pointer() {
    let receipt = json!({
        "analysis": {
            "derived": {
                "metrics": {
                    "complexity": {
                        "avg_cyclomatic": 4.5
                    }
                }
            }
        }
    });
    let policy = PolicyConfig {
        rules: vec![make_rule(
            "complexity",
            "/analysis/derived/metrics/complexity/avg_cyclomatic",
            RuleOperator::Lte,
            json!(10.0),
        )],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed);
}

#[test]
fn policy_with_array_index_pointer() {
    let receipt = json!({
        "languages": [
            {"name": "Rust", "code": 8000},
            {"name": "Python", "code": 2000}
        ]
    });
    let policy = PolicyConfig {
        rules: vec![
            make_rule(
                "top_lang",
                "/languages/0/name",
                RuleOperator::Eq,
                json!("Rust"),
            ),
            make_rule(
                "top_code",
                "/languages/0/code",
                RuleOperator::Gte,
                json!(5000),
            ),
        ],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed);
}

// ============================================================================
// GateResult / RuleResult properties
// ============================================================================

#[test]
fn rule_result_contains_actual_value() {
    let receipt = json!({"n": 42});
    let policy = PolicyConfig {
        rules: vec![make_rule("check", "/n", RuleOperator::Gt, json!(100))],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
    let rr = &result.rule_results[0];
    assert_eq!(rr.actual, Some(json!(42)));
    assert!(!rr.passed);
}

#[test]
fn rule_result_custom_message_on_failure() {
    let receipt = json!({"n": 42});
    let rule = PolicyRule {
        name: "custom_msg".into(),
        pointer: "/n".into(),
        op: RuleOperator::Gt,
        value: Some(json!(100)),
        values: None,
        negate: false,
        level: RuleLevel::Error,
        message: Some("Value too low!".into()),
    };
    let result = evaluate_policy(
        &receipt,
        &PolicyConfig {
            rules: vec![rule],
            ..Default::default()
        },
    );
    assert_eq!(
        result.rule_results[0].message.as_deref(),
        Some("Value too low!")
    );
}

#[test]
fn rule_result_no_message_on_pass() {
    let receipt = json!({"n": 200});
    let rule = PolicyRule {
        name: "check".into(),
        pointer: "/n".into(),
        op: RuleOperator::Gt,
        value: Some(json!(100)),
        values: None,
        negate: false,
        level: RuleLevel::Error,
        message: Some("Should not appear".into()),
    };
    let result = evaluate_policy(
        &receipt,
        &PolicyConfig {
            rules: vec![rule],
            ..Default::default()
        },
    );
    assert!(result.passed);
    assert!(
        result.rule_results[0].message.is_none(),
        "message should be None when rule passes"
    );
}

// ============================================================================
// Serialization round-trip
// ============================================================================

#[test]
fn gate_result_serde_roundtrip() {
    let receipt = json!({"tokens": 100, "code": 500});
    let policy = PolicyConfig {
        rules: vec![
            make_rule("t", "/tokens", RuleOperator::Lte, json!(1000)),
            make_warn_rule("c", "/code", RuleOperator::Gte, json!(100)),
        ],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    let json_str = serde_json::to_string(&result).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed["passed"], json!(true));
    assert_eq!(parsed["errors"], json!(0));
    assert_eq!(parsed["warnings"], json!(0));
}

#[test]
fn ratchet_gate_result_serde_roundtrip() {
    let baseline = json!({"x": 10.0});
    let current = json!({"x": 10.5});
    let config = RatchetConfig {
        rules: vec![make_ratchet_rule("/x", Some(20.0), None)],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    let json_str = serde_json::to_string(&result).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed["passed"], json!(true));
}

// ============================================================================
// TOML config parsing edge cases
// ============================================================================

#[test]
fn toml_all_operators() {
    let toml = r#"
[[rules]]
name = "gt_rule"
pointer = "/a"
op = "gt"
value = 1

[[rules]]
name = "lt_rule"
pointer = "/b"
op = "lt"
value = 100

[[rules]]
name = "gte_rule"
pointer = "/c"
op = "gte"
value = 5

[[rules]]
name = "lte_rule"
pointer = "/d"
op = "lte"
value = 50

[[rules]]
name = "eq_rule"
pointer = "/e"
op = "eq"
value = "hello"

[[rules]]
name = "ne_rule"
pointer = "/f"
op = "ne"
value = "bad"

[[rules]]
name = "contains_rule"
pointer = "/g"
op = "contains"
value = "needle"

[[rules]]
name = "exists_rule"
pointer = "/h"
op = "exists"
"#;
    let policy = PolicyConfig::from_toml(toml).unwrap();
    assert_eq!(policy.rules.len(), 8);
    assert_eq!(policy.rules[0].op, RuleOperator::Gt);
    assert_eq!(policy.rules[1].op, RuleOperator::Lt);
    assert_eq!(policy.rules[2].op, RuleOperator::Gte);
    assert_eq!(policy.rules[3].op, RuleOperator::Lte);
    assert_eq!(policy.rules[4].op, RuleOperator::Eq);
    assert_eq!(policy.rules[5].op, RuleOperator::Ne);
    assert_eq!(policy.rules[6].op, RuleOperator::Contains);
    assert_eq!(policy.rules[7].op, RuleOperator::Exists);
}

#[test]
fn toml_negate_and_level() {
    let toml = r#"
[[rules]]
name = "negated_warn"
pointer = "/x"
op = "gt"
value = 100
negate = true
level = "warn"
message = "custom"
"#;
    let policy = PolicyConfig::from_toml(toml).unwrap();
    assert!(policy.rules[0].negate);
    assert_eq!(policy.rules[0].level, RuleLevel::Warn);
    assert_eq!(policy.rules[0].message.as_deref(), Some("custom"));
}
