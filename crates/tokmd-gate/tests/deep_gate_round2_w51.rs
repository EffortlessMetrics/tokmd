//! Wave 51 deep round-2 tests for tokmd-gate.
//!
//! Covers:
//! - Gate policy evaluation with TOML-defined rules
//! - JSON pointer resolution on nested receipt JSON
//! - Ratchet behavior (value can only improve, not regress)
//! - Gate status aggregation (all-pass, any-fail, mixed)
//! - Threshold comparison operators (gt, lt, gte, lte, eq, ne)
//! - Missing JSON pointer paths (should not panic)
//! - Gate report deterministic serialization

use serde_json::json;
use tokmd_gate::{
    GateResult, PolicyConfig, PolicyRule, RatchetConfig, RatchetGateResult, RatchetRule, RuleLevel,
    RuleOperator, evaluate_policy, evaluate_ratchet_policy, resolve_pointer,
};

// =========================================================================
// Helpers
// =========================================================================

fn rule(name: &str, pointer: &str, op: RuleOperator, value: serde_json::Value) -> PolicyRule {
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

fn warn_rule(name: &str, pointer: &str, op: RuleOperator, value: serde_json::Value) -> PolicyRule {
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

fn eval(receipt: &serde_json::Value, rules: Vec<PolicyRule>) -> GateResult {
    let policy = PolicyConfig {
        rules,
        fail_fast: false,
        allow_missing: false,
    };
    evaluate_policy(receipt, &policy)
}

fn ratchet_rule(pointer: &str, max_inc: Option<f64>, max_val: Option<f64>) -> RatchetRule {
    RatchetRule {
        pointer: pointer.to_string(),
        max_increase_pct: max_inc,
        max_value: max_val,
        level: RuleLevel::Error,
        description: None,
    }
}

fn eval_ratchet(
    rules: Vec<RatchetRule>,
    baseline: &serde_json::Value,
    current: &serde_json::Value,
) -> RatchetGateResult {
    let config = RatchetConfig {
        rules,
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    evaluate_ratchet_policy(&config, baseline, current)
}

// =========================================================================
// 1. Gate policy evaluation with TOML-defined rules
// =========================================================================

#[test]
fn toml_policy_single_rule_pass() {
    let policy = PolicyConfig::from_toml(
        r#"
[[rules]]
name = "max_tokens"
pointer = "/tokens"
op = "lte"
value = 1000
"#,
    )
    .unwrap();
    let receipt = json!({"tokens": 500});
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed);
    assert_eq!(result.errors, 0);
}

#[test]
fn toml_policy_single_rule_fail() {
    let policy = PolicyConfig::from_toml(
        r#"
[[rules]]
name = "max_tokens"
pointer = "/tokens"
op = "lte"
value = 100
"#,
    )
    .unwrap();
    let receipt = json!({"tokens": 500});
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
}

#[test]
fn toml_policy_multiple_rules_mixed() {
    let policy = PolicyConfig::from_toml(
        r#"
[[rules]]
name = "max_tokens"
pointer = "/tokens"
op = "lte"
value = 1000

[[rules]]
name = "min_files"
pointer = "/files"
op = "gte"
value = 5
"#,
    )
    .unwrap();
    let receipt = json!({"tokens": 500, "files": 3});
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
    assert_eq!(result.errors, 1); // files < 5 fails
}

#[test]
fn toml_policy_fail_fast_stops_on_first_error() {
    let policy = PolicyConfig::from_toml(
        r#"
fail_fast = true

[[rules]]
name = "first"
pointer = "/a"
op = "eq"
value = 999

[[rules]]
name = "second"
pointer = "/b"
op = "eq"
value = 999
"#,
    )
    .unwrap();
    let receipt = json!({"a": 1, "b": 2});
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
    // fail_fast should stop after first error
    assert_eq!(result.rule_results.len(), 1);
}

#[test]
fn toml_policy_allow_missing_treats_absent_as_pass() {
    let policy = PolicyConfig::from_toml(
        r#"
allow_missing = true

[[rules]]
name = "check_missing"
pointer = "/nonexistent"
op = "lte"
value = 100
"#,
    )
    .unwrap();
    let receipt = json!({"tokens": 42});
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed);
}

#[test]
fn toml_ratchet_config_parse() {
    let config = RatchetConfig::from_toml(
        r#"
fail_fast = true
allow_missing_baseline = true
allow_missing_current = false

[[rules]]
pointer = "/complexity"
max_increase_pct = 5.0
max_value = 50.0
level = "error"
description = "Complexity must not regress"
"#,
    )
    .unwrap();
    assert!(config.fail_fast);
    assert!(config.allow_missing_baseline);
    assert!(!config.allow_missing_current);
    assert_eq!(config.rules.len(), 1);
    assert_eq!(config.rules[0].pointer, "/complexity");
}

// =========================================================================
// 2. JSON pointer resolution on nested receipt JSON
// =========================================================================

#[test]
fn pointer_deeply_nested() {
    let doc = json!({"a": {"b": {"c": {"d": 42}}}});
    assert_eq!(resolve_pointer(&doc, "/a/b/c/d"), Some(&json!(42)));
}

#[test]
fn pointer_array_index() {
    let doc = json!({"items": [10, 20, 30]});
    assert_eq!(resolve_pointer(&doc, "/items/1"), Some(&json!(20)));
}

#[test]
fn pointer_escaped_tilde_and_slash() {
    let doc = json!({"a/b": {"c~d": 99}});
    assert_eq!(resolve_pointer(&doc, "/a~1b/c~0d"), Some(&json!(99)));
}

#[test]
fn pointer_empty_returns_whole_document() {
    let doc = json!({"x": 1});
    assert_eq!(resolve_pointer(&doc, ""), Some(&doc));
}

#[test]
fn pointer_into_array_of_objects() {
    let doc = json!({"users": [{"name": "alice"}, {"name": "bob"}]});
    assert_eq!(
        resolve_pointer(&doc, "/users/0/name"),
        Some(&json!("alice"))
    );
    assert_eq!(resolve_pointer(&doc, "/users/1/name"), Some(&json!("bob")));
}

// =========================================================================
// 3. Ratchet behavior
// =========================================================================

#[test]
fn ratchet_pass_when_value_improves() {
    let baseline = json!({"complexity": 20.0});
    let current = json!({"complexity": 18.0});
    let result = eval_ratchet(
        vec![ratchet_rule("/complexity", Some(10.0), None)],
        &baseline,
        &current,
    );
    assert!(result.passed);
}

#[test]
fn ratchet_fail_when_regression_exceeds_threshold() {
    let baseline = json!({"complexity": 10.0});
    let current = json!({"complexity": 15.0}); // 50% increase
    let result = eval_ratchet(
        vec![ratchet_rule("/complexity", Some(10.0), None)],
        &baseline,
        &current,
    );
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
}

#[test]
fn ratchet_pass_within_threshold() {
    let baseline = json!({"complexity": 10.0});
    let current = json!({"complexity": 10.9}); // 9% increase, threshold 10%
    let result = eval_ratchet(
        vec![ratchet_rule("/complexity", Some(10.0), None)],
        &baseline,
        &current,
    );
    assert!(result.passed);
}

#[test]
fn ratchet_max_value_ceiling() {
    let baseline = json!({"complexity": 40.0});
    let current = json!({"complexity": 42.0}); // only 5% increase but exceeds ceiling
    let result = eval_ratchet(
        vec![ratchet_rule("/complexity", None, Some(41.0))],
        &baseline,
        &current,
    );
    assert!(!result.passed);
}

#[test]
fn ratchet_missing_baseline_fails_strict() {
    let baseline = json!({});
    let current = json!({"complexity": 10.0});
    let result = eval_ratchet(
        vec![ratchet_rule("/complexity", Some(10.0), None)],
        &baseline,
        &current,
    );
    assert!(!result.passed);
}

#[test]
fn ratchet_missing_baseline_pass_when_allowed() {
    let baseline = json!({});
    let current = json!({"complexity": 10.0});
    let config = RatchetConfig {
        rules: vec![ratchet_rule("/complexity", Some(10.0), None)],
        fail_fast: false,
        allow_missing_baseline: true,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(result.passed);
}

// =========================================================================
// 4. Gate status aggregation
// =========================================================================

#[test]
fn gate_all_pass() {
    let receipt = json!({"a": 10, "b": 20});
    let result = eval(
        &receipt,
        vec![
            rule("r1", "/a", RuleOperator::Lte, json!(100)),
            rule("r2", "/b", RuleOperator::Lte, json!(100)),
        ],
    );
    assert!(result.passed);
    assert_eq!(result.errors, 0);
    assert_eq!(result.warnings, 0);
}

#[test]
fn gate_any_fail() {
    let receipt = json!({"a": 10, "b": 200});
    let result = eval(
        &receipt,
        vec![
            rule("r1", "/a", RuleOperator::Lte, json!(100)),
            rule("r2", "/b", RuleOperator::Lte, json!(100)),
        ],
    );
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
}

#[test]
fn gate_mixed_warn_and_error() {
    let receipt = json!({"a": 200, "b": 200});
    let result = eval(
        &receipt,
        vec![
            warn_rule("w1", "/a", RuleOperator::Lte, json!(100)),
            rule("e1", "/b", RuleOperator::Lte, json!(100)),
        ],
    );
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
    assert_eq!(result.warnings, 1);
}

#[test]
fn gate_only_warnings_still_passes() {
    let receipt = json!({"a": 200});
    let result = eval(
        &receipt,
        vec![warn_rule("w1", "/a", RuleOperator::Lte, json!(100))],
    );
    assert!(result.passed);
    assert_eq!(result.warnings, 1);
    assert_eq!(result.errors, 0);
}

#[test]
fn gate_empty_rules_passes() {
    let result = GateResult::from_results(vec![]);
    assert!(result.passed);
    assert_eq!(result.errors, 0);
    assert_eq!(result.warnings, 0);
}

// =========================================================================
// 5. Threshold comparison operators
// =========================================================================

#[test]
fn operator_gt() {
    let receipt = json!({"v": 10});
    let r = eval(&receipt, vec![rule("gt", "/v", RuleOperator::Gt, json!(5))]);
    assert!(r.passed);
    let r = eval(
        &receipt,
        vec![rule("gt", "/v", RuleOperator::Gt, json!(10))],
    );
    assert!(!r.passed);
}

#[test]
fn operator_lt() {
    let receipt = json!({"v": 5});
    let r = eval(
        &receipt,
        vec![rule("lt", "/v", RuleOperator::Lt, json!(10))],
    );
    assert!(r.passed);
    let r = eval(&receipt, vec![rule("lt", "/v", RuleOperator::Lt, json!(5))]);
    assert!(!r.passed);
}

#[test]
fn operator_gte() {
    let receipt = json!({"v": 10});
    let r = eval(
        &receipt,
        vec![rule("gte", "/v", RuleOperator::Gte, json!(10))],
    );
    assert!(r.passed);
    let r = eval(
        &receipt,
        vec![rule("gte", "/v", RuleOperator::Gte, json!(11))],
    );
    assert!(!r.passed);
}

#[test]
fn operator_lte() {
    let receipt = json!({"v": 10});
    let r = eval(
        &receipt,
        vec![rule("lte", "/v", RuleOperator::Lte, json!(10))],
    );
    assert!(r.passed);
    let r = eval(
        &receipt,
        vec![rule("lte", "/v", RuleOperator::Lte, json!(9))],
    );
    assert!(!r.passed);
}

#[test]
fn operator_eq() {
    let receipt = json!({"v": 42});
    let r = eval(
        &receipt,
        vec![rule("eq", "/v", RuleOperator::Eq, json!(42))],
    );
    assert!(r.passed);
    let r = eval(
        &receipt,
        vec![rule("eq", "/v", RuleOperator::Eq, json!(43))],
    );
    assert!(!r.passed);
}

#[test]
fn operator_ne() {
    let receipt = json!({"v": 42});
    let r = eval(
        &receipt,
        vec![rule("ne", "/v", RuleOperator::Ne, json!(99))],
    );
    assert!(r.passed);
    let r = eval(
        &receipt,
        vec![rule("ne", "/v", RuleOperator::Ne, json!(42))],
    );
    assert!(!r.passed);
}

#[test]
fn operator_exists() {
    let receipt = json!({"present": 1});
    let policy = PolicyConfig {
        rules: vec![PolicyRule {
            name: "exists_check".into(),
            pointer: "/present".into(),
            op: RuleOperator::Exists,
            value: None,
            values: None,
            negate: false,
            level: RuleLevel::Error,
            message: None,
        }],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed);
}

#[test]
fn operator_exists_missing_fails() {
    let receipt = json!({"present": 1});
    let policy = PolicyConfig {
        rules: vec![PolicyRule {
            name: "exists_check".into(),
            pointer: "/missing".into(),
            op: RuleOperator::Exists,
            value: None,
            values: None,
            negate: false,
            level: RuleLevel::Error,
            message: None,
        }],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
}

#[test]
fn operator_in_with_values() {
    let receipt = json!({"lang": "rust"});
    let policy = PolicyConfig {
        rules: vec![PolicyRule {
            name: "lang_check".into(),
            pointer: "/lang".into(),
            op: RuleOperator::In,
            value: None,
            values: Some(vec![json!("rust"), json!("go"), json!("python")]),
            negate: false,
            level: RuleLevel::Error,
            message: None,
        }],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed);
}

// =========================================================================
// 6. Missing JSON pointer paths
// =========================================================================

#[test]
fn missing_pointer_does_not_panic() {
    let receipt = json!({"tokens": 42});
    let result = eval(
        &receipt,
        vec![rule(
            "missing",
            "/nonexistent/deep/path",
            RuleOperator::Lte,
            json!(100),
        )],
    );
    // Should fail (not panic) because pointer is missing
    assert!(!result.passed);
}

#[test]
fn missing_pointer_with_allow_missing() {
    let receipt = json!({"tokens": 42});
    let policy = PolicyConfig {
        rules: vec![rule(
            "missing",
            "/nonexistent",
            RuleOperator::Lte,
            json!(100),
        )],
        fail_fast: false,
        allow_missing: true,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed);
}

#[test]
fn missing_pointer_in_ratchet_does_not_panic() {
    let baseline = json!({"a": 10});
    let current = json!({"a": 10});
    let result = eval_ratchet(
        vec![ratchet_rule("/nonexistent", Some(10.0), None)],
        &baseline,
        &current,
    );
    // Should fail, not panic
    assert!(!result.passed);
}

#[test]
fn pointer_invalid_no_leading_slash() {
    let doc = json!({"a": 1});
    assert_eq!(resolve_pointer(&doc, "a"), None);
}

// =========================================================================
// 7. Gate report deterministic serialization
// =========================================================================

#[test]
fn gate_result_deterministic_json() {
    let receipt = json!({"tokens": 500, "files": 10, "complexity": 8.5});
    let rules = vec![
        rule("max_tokens", "/tokens", RuleOperator::Lte, json!(1000)),
        rule("min_files", "/files", RuleOperator::Gte, json!(1)),
        warn_rule(
            "complexity_warn",
            "/complexity",
            RuleOperator::Lte,
            json!(10.0),
        ),
    ];
    let r1 = eval(&receipt, rules.clone());
    let r2 = eval(&receipt, rules);

    let j1 = serde_json::to_string(&r1).unwrap();
    let j2 = serde_json::to_string(&r2).unwrap();
    assert_eq!(
        j1, j2,
        "Same inputs must produce identical gate result JSON"
    );
}

#[test]
fn ratchet_result_deterministic_json() {
    let baseline = json!({"complexity": 10.0, "tokens": 500});
    let current = json!({"complexity": 10.5, "tokens": 520});
    let rules = vec![
        ratchet_rule("/complexity", Some(10.0), Some(50.0)),
        ratchet_rule("/tokens", Some(20.0), None),
    ];
    let r1 = eval_ratchet(rules.clone(), &baseline, &current);
    let r2 = eval_ratchet(rules, &baseline, &current);

    let j1 = serde_json::to_string(&r1).unwrap();
    let j2 = serde_json::to_string(&r2).unwrap();
    assert_eq!(j1, j2);
}

#[test]
fn gate_result_serde_roundtrip() {
    let receipt = json!({"tokens": 500});
    let result = eval(
        &receipt,
        vec![rule("check", "/tokens", RuleOperator::Lte, json!(1000))],
    );
    let json = serde_json::to_string(&result).unwrap();
    let back: GateResult = serde_json::from_str(&json).unwrap();
    assert_eq!(back.passed, result.passed);
    assert_eq!(back.errors, result.errors);
    assert_eq!(back.warnings, result.warnings);
    assert_eq!(back.rule_results.len(), result.rule_results.len());
}

#[test]
fn ratchet_gate_result_serde_roundtrip() {
    let baseline = json!({"complexity": 10.0});
    let current = json!({"complexity": 10.5});
    let result = eval_ratchet(
        vec![ratchet_rule("/complexity", Some(10.0), None)],
        &baseline,
        &current,
    );
    let json = serde_json::to_string(&result).unwrap();
    let back: RatchetGateResult = serde_json::from_str(&json).unwrap();
    assert_eq!(back.passed, result.passed);
    assert_eq!(back.errors, result.errors);
    assert_eq!(back.ratchet_results.len(), result.ratchet_results.len());
}

// =========================================================================
// 8. Negate flag
// =========================================================================

#[test]
fn negate_inverts_result() {
    let receipt = json!({"v": 42});
    let policy = PolicyConfig {
        rules: vec![PolicyRule {
            name: "negated_eq".into(),
            pointer: "/v".into(),
            op: RuleOperator::Eq,
            value: Some(json!(42)),
            values: None,
            negate: true,
            level: RuleLevel::Error,
            message: None,
        }],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    // v == 42 is true, but negated => false => fail
    assert!(!result.passed);
}

#[test]
fn negate_exists_means_must_not_exist() {
    let receipt = json!({"present": 1});
    let policy = PolicyConfig {
        rules: vec![PolicyRule {
            name: "must_not_exist".into(),
            pointer: "/present".into(),
            op: RuleOperator::Exists,
            value: None,
            values: None,
            negate: true,
            level: RuleLevel::Error,
            message: None,
        }],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
}
