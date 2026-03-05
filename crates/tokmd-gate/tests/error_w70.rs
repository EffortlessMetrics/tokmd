//! Error handling edge-case tests for tokmd-gate (w70).
//!
//! Tests policy evaluation with invalid pointers, type mismatches,
//! missing fields, malformed TOML, and boundary conditions.

use serde_json::{Value, json};
use tokmd_gate::{
    GateResult, PolicyConfig, PolicyRule, RuleLevel, RuleOperator, evaluate_policy,
    evaluate_ratchet_policy, resolve_pointer, RatchetConfig, RatchetRule,
};

// ============================================================================
// Helpers
// ============================================================================

fn rule(name: &str, pointer: &str, op: RuleOperator, value: Value) -> PolicyRule {
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

fn policy(rules: Vec<PolicyRule>) -> PolicyConfig {
    PolicyConfig {
        rules,
        fail_fast: false,
        allow_missing: false,
    }
}

fn eval(receipt: &Value, rules: Vec<PolicyRule>) -> GateResult {
    evaluate_policy(receipt, &policy(rules))
}

// ============================================================================
// 1. Invalid / missing JSON pointer paths
// ============================================================================

#[test]
fn pointer_to_nonexistent_key_returns_none() {
    let receipt = json!({"a": 1});
    assert_eq!(resolve_pointer(&receipt, "/nonexistent"), None);
}

#[test]
fn pointer_deep_nonexistent_path_returns_none() {
    let receipt = json!({"a": {"b": 1}});
    assert_eq!(resolve_pointer(&receipt, "/a/b/c/d/e"), None);
}

#[test]
fn pointer_into_scalar_returns_none() {
    let receipt = json!({"val": 42});
    assert_eq!(resolve_pointer(&receipt, "/val/child"), None);
}

#[test]
fn pointer_with_out_of_bounds_array_index_returns_none() {
    let receipt = json!({"items": [1, 2, 3]});
    assert_eq!(resolve_pointer(&receipt, "/items/99"), None);
}

#[test]
fn rule_on_missing_pointer_fails_when_allow_missing_false() {
    let receipt = json!({"tokens": 100});
    let result = eval(
        &receipt,
        vec![rule("check_missing", "/nonexistent/path", RuleOperator::Lte, json!(500))],
    );
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
    // The rule result should mention the pointer not being found
    let msg = result.rule_results[0].message.as_deref().unwrap_or("");
    assert!(
        msg.contains("not found"),
        "error message should mention 'not found', got: {msg}"
    );
}

#[test]
fn rule_on_missing_pointer_passes_when_allow_missing_true() {
    let receipt = json!({"tokens": 100});
    let policy = PolicyConfig {
        rules: vec![rule(
            "check_missing",
            "/nonexistent/path",
            RuleOperator::Lte,
            json!(500),
        )],
        fail_fast: false,
        allow_missing: true,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed);
    assert_eq!(result.errors, 0);
}

// ============================================================================
// 2. Type mismatches in comparisons
// ============================================================================

#[test]
fn numeric_comparison_against_string_value_fails() {
    let receipt = json!({"metric": "not_a_number"});
    let result = eval(
        &receipt,
        vec![rule("num_check", "/metric", RuleOperator::Gt, json!(10))],
    );
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
}

#[test]
fn numeric_comparison_against_boolean_value_fails() {
    let receipt = json!({"metric": true});
    let result = eval(
        &receipt,
        vec![rule("num_check", "/metric", RuleOperator::Lte, json!(100))],
    );
    assert!(!result.passed);
}

#[test]
fn numeric_comparison_against_null_value_fails() {
    let receipt = json!({"metric": null});
    let result = eval(
        &receipt,
        vec![rule("null_check", "/metric", RuleOperator::Gt, json!(0))],
    );
    // null pointer resolution returns Some(null), comparison should fail
    assert!(!result.passed);
}

#[test]
fn numeric_comparison_against_array_value_fails() {
    let receipt = json!({"metric": [1, 2, 3]});
    let result = eval(
        &receipt,
        vec![rule("arr_check", "/metric", RuleOperator::Lt, json!(10))],
    );
    assert!(!result.passed);
}

#[test]
fn numeric_comparison_against_object_value_fails() {
    let receipt = json!({"metric": {"nested": 42}});
    let result = eval(
        &receipt,
        vec![rule("obj_check", "/metric", RuleOperator::Eq, json!(42))],
    );
    assert!(!result.passed);
}

// ============================================================================
// 3. Malformed TOML policy parsing
// ============================================================================

#[test]
fn from_toml_with_invalid_toml_returns_error() {
    let bad_toml = "this is not valid toml [[[";
    let result = PolicyConfig::from_toml(bad_toml);
    assert!(result.is_err());
}

#[test]
fn from_toml_with_unknown_operator_returns_error() {
    let toml = r#"
[[rules]]
name = "test"
pointer = "/x"
op = "bogus_operator"
value = 42
"#;
    let result = PolicyConfig::from_toml(toml);
    assert!(result.is_err());
}

#[test]
fn from_file_with_nonexistent_path_returns_io_error() {
    let result =
        PolicyConfig::from_file(std::path::Path::new("/nonexistent/w70_policy_file.toml"));
    assert!(result.is_err());
}

// ============================================================================
// 4. Edge cases in rule evaluation
// ============================================================================

#[test]
fn empty_policy_with_no_rules_passes() {
    let receipt = json!({"anything": 42});
    let result = evaluate_policy(&receipt, &policy(vec![]));
    assert!(result.passed);
    assert_eq!(result.errors, 0);
    assert_eq!(result.warnings, 0);
    assert!(result.rule_results.is_empty());
}

#[test]
fn exists_operator_on_missing_key_fails() {
    let receipt = json!({"a": 1});
    let mut r = rule("check_exists", "/missing_key", RuleOperator::Exists, json!(null));
    r.value = None;
    r.op = RuleOperator::Exists;
    let result = eval(&receipt, vec![r]);
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
}

#[test]
fn exists_operator_on_present_key_passes() {
    let receipt = json!({"present": "value"});
    let mut r = rule("check_exists", "/present", RuleOperator::Exists, json!(null));
    r.value = None;
    let result = eval(&receipt, vec![r]);
    assert!(result.passed);
}

#[test]
fn negated_exists_on_present_key_fails() {
    let receipt = json!({"present": "value"});
    let r = PolicyRule {
        name: "no_present".into(),
        pointer: "/present".into(),
        op: RuleOperator::Exists,
        value: None,
        values: None,
        negate: true,
        level: RuleLevel::Error,
        message: None,
    };
    let result = eval(&receipt, vec![r]);
    assert!(!result.passed);
}

// ============================================================================
// 5. Ratchet error paths
// ============================================================================

#[test]
fn ratchet_missing_current_value_fails() {
    let baseline = json!({"complexity": 10.0});
    let current = json!({}); // Missing the metric
    let config = RatchetConfig {
        rules: vec![RatchetRule {
            pointer: "/complexity".into(),
            max_increase_pct: Some(10.0),
            max_value: None,
            level: RuleLevel::Error,
            description: None,
        }],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
}

#[test]
fn ratchet_missing_baseline_value_fails_when_not_allowed() {
    let baseline = json!({}); // Missing baseline
    let current = json!({"complexity": 10.0});
    let config = RatchetConfig {
        rules: vec![RatchetRule {
            pointer: "/complexity".into(),
            max_increase_pct: Some(10.0),
            max_value: None,
            level: RuleLevel::Error,
            description: None,
        }],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(!result.passed);
}

#[test]
fn ratchet_from_toml_with_invalid_toml_returns_error() {
    let bad_toml = "not valid [[[";
    let result = RatchetConfig::from_toml(bad_toml);
    assert!(result.is_err());
}
