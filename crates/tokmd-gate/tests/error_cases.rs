//! Error handling and edge case tests for tokmd-gate.

use serde_json::json;
use tokmd_gate::{
    GateResult, PolicyConfig, PolicyRule, RatchetConfig, RatchetRule, RuleLevel, RuleOperator,
    RuleResult, evaluate_policy, evaluate_ratchet_policy, resolve_pointer,
};

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

fn make_policy(rules: Vec<PolicyRule>) -> PolicyConfig {
    PolicyConfig {
        rules,
        fail_fast: false,
        allow_missing: false,
    }
}

// ── Invalid JSON pointer syntax ────────────────────────────────────

#[test]
fn pointer_without_leading_slash_returns_none() {
    let doc = json!({"foo": 1});
    assert_eq!(resolve_pointer(&doc, "foo"), None);
}

#[test]
fn pointer_with_trailing_slash_resolves_empty_key() {
    let doc = json!({"foo": {"": 42}});
    assert_eq!(resolve_pointer(&doc, "/foo/"), Some(&json!(42)));
}

#[test]
fn pointer_double_slash_resolves_empty_key() {
    let doc = json!({"": {"": "deep"}});
    assert_eq!(resolve_pointer(&doc, "//"), Some(&json!("deep")));
}

#[test]
fn pointer_into_scalar_returns_none() {
    let doc = json!({"x": 42});
    assert_eq!(resolve_pointer(&doc, "/x/deeper"), None);
}

#[test]
fn pointer_non_numeric_array_index_returns_none() {
    let doc = json!({"arr": [1, 2, 3]});
    assert_eq!(resolve_pointer(&doc, "/arr/abc"), None);
}

#[test]
fn pointer_negative_index_returns_none() {
    let doc = json!({"arr": [1, 2, 3]});
    assert_eq!(resolve_pointer(&doc, "/arr/-1"), None);
}

#[test]
fn pointer_out_of_bounds_array_index_returns_none() {
    let doc = json!({"arr": [1, 2, 3]});
    assert_eq!(resolve_pointer(&doc, "/arr/99"), None);
}

#[test]
fn pointer_with_tilde_escapes_resolves_correctly() {
    let doc = json!({"a/b": {"c~d": 99}});
    assert_eq!(resolve_pointer(&doc, "/a~1b/c~0d"), Some(&json!(99)));
}

// ── Missing fields in receipt for gate evaluation ──────────────────

#[test]
fn evaluate_missing_field_with_allow_missing_passes() {
    let receipt = json!({"a": 1});
    let policy = PolicyConfig {
        rules: vec![make_rule(
            "check",
            "/nonexistent",
            RuleOperator::Lte,
            json!(100),
        )],
        fail_fast: false,
        allow_missing: true,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed);
    assert!(result.rule_results[0].passed);
}

#[test]
fn evaluate_missing_field_without_allow_missing_fails() {
    let receipt = json!({"a": 1});
    let policy = make_policy(vec![make_rule(
        "check",
        "/nonexistent",
        RuleOperator::Lte,
        json!(100),
    )]);
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
    let msg = result.rule_results[0].message.as_deref().unwrap();
    assert!(
        msg.contains("not found"),
        "error message should indicate missing pointer: {msg}"
    );
}

#[test]
fn evaluate_deeply_nested_missing_field_fails() {
    let receipt = json!({"a": {"b": 1}});
    let policy = make_policy(vec![make_rule(
        "deep",
        "/a/b/c/d",
        RuleOperator::Eq,
        json!(1),
    )]);
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
}

// ── Empty rule set ─────────────────────────────────────────────────

#[test]
fn empty_policy_rules_always_passes() {
    let receipt = json!({"anything": "at all"});
    let policy = make_policy(vec![]);
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed);
    assert_eq!(result.errors, 0);
    assert_eq!(result.warnings, 0);
    assert!(result.rule_results.is_empty());
}

#[test]
fn empty_ratchet_rules_always_passes() {
    let config = RatchetConfig {
        rules: vec![],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &json!({}), &json!({}));
    assert!(result.passed);
    assert!(result.ratchet_results.is_empty());
}

// ── Contradictory rules ────────────────────────────────────────────

#[test]
fn contradictory_rules_gt_and_lt_same_value_fail() {
    let receipt = json!({"x": 10});
    let policy = make_policy(vec![
        make_rule("must_be_gt_10", "/x", RuleOperator::Gt, json!(10)),
        make_rule("must_be_lt_10", "/x", RuleOperator::Lt, json!(10)),
    ]);
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
    assert_eq!(result.errors, 2, "both rules should fail for x=10");
}

#[test]
fn contradictory_eq_ne_same_value() {
    let receipt = json!({"v": "hello"});
    let policy = make_policy(vec![
        make_rule("eq_hello", "/v", RuleOperator::Eq, json!("hello")),
        make_rule("ne_hello", "/v", RuleOperator::Ne, json!("hello")),
    ]);
    let result = evaluate_policy(&receipt, &policy);
    // One must pass and one must fail
    assert_eq!(result.errors, 1);
    assert!(!result.passed);
}

// ── Malformed threshold values ─────────────────────────────────────

#[test]
fn numeric_comparison_against_string_value_fails_gracefully() {
    let receipt = json!({"count": "not-a-number"});
    let policy = make_policy(vec![make_rule(
        "check",
        "/count",
        RuleOperator::Gt,
        json!(10),
    )]);
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
}

#[test]
fn numeric_comparison_against_boolean_fails_gracefully() {
    let receipt = json!({"flag": true});
    let policy = make_policy(vec![make_rule(
        "check",
        "/flag",
        RuleOperator::Gt,
        json!(0),
    )]);
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
}

#[test]
fn numeric_comparison_against_null_fails_gracefully() {
    let receipt = json!({"val": null});
    let policy = make_policy(vec![make_rule(
        "check",
        "/val",
        RuleOperator::Lte,
        json!(100),
    )]);
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
}

#[test]
fn in_operator_with_no_values_fails() {
    let receipt = json!({"lang": "Rust"});
    let rule = PolicyRule {
        name: "in_empty".into(),
        pointer: "/lang".into(),
        op: RuleOperator::In,
        value: None,
        values: None,
        negate: false,
        level: RuleLevel::Error,
        message: None,
    };
    let policy = make_policy(vec![rule]);
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
}

#[test]
fn in_operator_with_empty_values_list_fails() {
    let receipt = json!({"lang": "Rust"});
    let rule = PolicyRule {
        name: "in_empty_list".into(),
        pointer: "/lang".into(),
        op: RuleOperator::In,
        value: None,
        values: Some(vec![]),
        negate: false,
        level: RuleLevel::Error,
        message: None,
    };
    let policy = make_policy(vec![rule]);
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed, "nothing can be 'in' an empty list");
}

#[test]
fn contains_on_non_container_type_fails() {
    let receipt = json!({"num": 42});
    let policy = make_policy(vec![make_rule(
        "check",
        "/num",
        RuleOperator::Contains,
        json!(4),
    )]);
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
}

// ── TOML parsing errors ────────────────────────────────────────────

#[test]
fn policy_from_invalid_toml_returns_error() {
    let bad_toml = "this is {{ not valid toml";
    let result = PolicyConfig::from_toml(bad_toml);
    assert!(result.is_err());
}

#[test]
fn ratchet_from_invalid_toml_returns_error() {
    let bad_toml = "[[[rules]]] = broken";
    let result = RatchetConfig::from_toml(bad_toml);
    assert!(result.is_err());
}

#[test]
fn policy_from_nonexistent_file_returns_io_error() {
    let result = PolicyConfig::from_file(std::path::Path::new("/nonexistent/policy.toml"));
    assert!(result.is_err());
}

#[test]
fn ratchet_from_nonexistent_file_returns_io_error() {
    let result = RatchetConfig::from_file(std::path::Path::new("/nonexistent/ratchet.toml"));
    assert!(result.is_err());
}

// ── GateResult construction edge cases ─────────────────────────────

#[test]
fn gate_result_all_passing_rules() {
    let results = vec![
        RuleResult {
            name: "r1".into(),
            passed: true,
            level: RuleLevel::Error,
            actual: Some(json!(1)),
            expected: "ok".into(),
            message: None,
        },
        RuleResult {
            name: "r2".into(),
            passed: true,
            level: RuleLevel::Warn,
            actual: Some(json!(2)),
            expected: "ok".into(),
            message: None,
        },
    ];
    let gate = GateResult::from_results(results);
    assert!(gate.passed);
    assert_eq!(gate.errors, 0);
    assert_eq!(gate.warnings, 0);
}

#[test]
fn gate_result_mixed_errors_and_warnings() {
    let results = vec![
        RuleResult {
            name: "err".into(),
            passed: false,
            level: RuleLevel::Error,
            actual: None,
            expected: "x".into(),
            message: Some("boom".into()),
        },
        RuleResult {
            name: "warn".into(),
            passed: false,
            level: RuleLevel::Warn,
            actual: None,
            expected: "x".into(),
            message: Some("heads up".into()),
        },
    ];
    let gate = GateResult::from_results(results);
    assert!(!gate.passed);
    assert_eq!(gate.errors, 1);
    assert_eq!(gate.warnings, 1);
}

// ── Ratchet edge cases ─────────────────────────────────────────────

#[test]
fn ratchet_non_numeric_current_value_fails() {
    let baseline = json!({"metric": 10});
    let current = json!({"metric": "not-a-number"});
    let rule = RatchetRule {
        pointer: "/metric".into(),
        max_increase_pct: Some(10.0),
        max_value: None,
        level: RuleLevel::Error,
        description: None,
    };
    let config = RatchetConfig {
        rules: vec![rule],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(!result.passed);
}

#[test]
fn ratchet_boolean_value_is_not_numeric() {
    let baseline = json!({"flag": true});
    let current = json!({"flag": false});
    let rule = RatchetRule {
        pointer: "/flag".into(),
        max_increase_pct: Some(10.0),
        max_value: None,
        level: RuleLevel::Error,
        description: None,
    };
    let config = RatchetConfig {
        rules: vec![rule],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(!result.passed);
}
