//! Wave-66 deep tests for tokmd-gate.
//!
//! Coverage targets:
//! - Policy rules with JSON pointer expressions
//! - Pass/fail/warn outcomes for every operator
//! - Negate flag on each operator
//! - Edge cases: empty rules, missing data, invalid pointers
//! - Ratchet evaluation: boundary, zero baseline, allow-missing flags
//! - Determinism property tests

use serde_json::json;
use tokmd_gate::{
    evaluate_policy, evaluate_ratchet_policy, resolve_pointer, GateResult, PolicyConfig,
    PolicyRule, RatchetConfig, RatchetGateResult, RatchetRule, RuleLevel, RuleOperator, RuleResult,
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

fn policy(rules: Vec<PolicyRule>) -> PolicyConfig {
    PolicyConfig {
        rules,
        fail_fast: false,
        allow_missing: false,
    }
}

fn ratchet_rule(pointer: &str, max_inc: Option<f64>, max_val: Option<f64>) -> RatchetRule {
    RatchetRule {
        pointer: pointer.into(),
        max_increase_pct: max_inc,
        max_value: max_val,
        level: RuleLevel::Error,
        description: None,
    }
}

// =========================================================================
// 1. JSON pointer resolution
// =========================================================================

#[test]
fn pointer_deeply_nested() {
    let doc = json!({"a": {"b": {"c": {"d": 99}}}});
    assert_eq!(resolve_pointer(&doc, "/a/b/c/d"), Some(&json!(99)));
}

#[test]
fn pointer_array_in_object() {
    let doc = json!({"items": [{"name": "a"}, {"name": "b"}]});
    assert_eq!(
        resolve_pointer(&doc, "/items/1/name"),
        Some(&json!("b"))
    );
}

#[test]
fn pointer_escaped_tilde_and_slash() {
    let doc = json!({"a/b": {"c~d": 42}});
    assert_eq!(resolve_pointer(&doc, "/a~1b/c~0d"), Some(&json!(42)));
}

#[test]
fn pointer_no_leading_slash_returns_none() {
    let doc = json!({"x": 1});
    assert_eq!(resolve_pointer(&doc, "x"), None);
}

// =========================================================================
// 2. Operator pass/fail coverage
// =========================================================================

#[test]
fn op_gt_pass_and_fail() {
    let receipt = json!({"val": 10});
    let pass = evaluate_policy(&receipt, &policy(vec![rule("gt", "/val", RuleOperator::Gt, json!(5))]));
    assert!(pass.passed);
    let fail = evaluate_policy(&receipt, &policy(vec![rule("gt", "/val", RuleOperator::Gt, json!(10))]));
    assert!(!fail.passed);
}

#[test]
fn op_gte_boundary() {
    let receipt = json!({"val": 10});
    let at = evaluate_policy(&receipt, &policy(vec![rule("gte", "/val", RuleOperator::Gte, json!(10))]));
    assert!(at.passed);
    let above = evaluate_policy(&receipt, &policy(vec![rule("gte", "/val", RuleOperator::Gte, json!(11))]));
    assert!(!above.passed);
}

#[test]
fn op_lt_pass_and_fail() {
    let receipt = json!({"val": 5});
    let pass = evaluate_policy(&receipt, &policy(vec![rule("lt", "/val", RuleOperator::Lt, json!(10))]));
    assert!(pass.passed);
    let fail = evaluate_policy(&receipt, &policy(vec![rule("lt", "/val", RuleOperator::Lt, json!(5))]));
    assert!(!fail.passed);
}

#[test]
fn op_lte_boundary() {
    let receipt = json!({"val": 10});
    let at = evaluate_policy(&receipt, &policy(vec![rule("lte", "/val", RuleOperator::Lte, json!(10))]));
    assert!(at.passed);
    let below = evaluate_policy(&receipt, &policy(vec![rule("lte", "/val", RuleOperator::Lte, json!(9))]));
    assert!(!below.passed);
}

#[test]
fn op_eq_string_and_numeric() {
    let receipt = json!({"lang": "Rust", "count": 42});
    let s = evaluate_policy(&receipt, &policy(vec![rule("eq_s", "/lang", RuleOperator::Eq, json!("Rust"))]));
    assert!(s.passed);
    let n = evaluate_policy(&receipt, &policy(vec![rule("eq_n", "/count", RuleOperator::Eq, json!(42))]));
    assert!(n.passed);
}

#[test]
fn op_ne_pass_and_fail() {
    let receipt = json!({"val": 5});
    let pass = evaluate_policy(&receipt, &policy(vec![rule("ne", "/val", RuleOperator::Ne, json!(6))]));
    assert!(pass.passed);
    let fail = evaluate_policy(&receipt, &policy(vec![rule("ne", "/val", RuleOperator::Ne, json!(5))]));
    assert!(!fail.passed);
}

#[test]
fn op_in_with_values_list() {
    let receipt = json!({"lang": "Rust"});
    let mut r = rule("in", "/lang", RuleOperator::In, json!(null));
    r.value = None;
    r.values = Some(vec![json!("Go"), json!("Rust"), json!("Python")]);
    let pass = evaluate_policy(&receipt, &policy(vec![r.clone()]));
    assert!(pass.passed);

    r.values = Some(vec![json!("Go"), json!("Python")]);
    let fail = evaluate_policy(&receipt, &policy(vec![r]));
    assert!(!fail.passed);
}

#[test]
fn op_contains_string() {
    let receipt = json!({"msg": "hello world"});
    let pass = evaluate_policy(&receipt, &policy(vec![rule("c", "/msg", RuleOperator::Contains, json!("world"))]));
    assert!(pass.passed);
    let fail = evaluate_policy(&receipt, &policy(vec![rule("c", "/msg", RuleOperator::Contains, json!("xyz"))]));
    assert!(!fail.passed);
}

#[test]
fn op_contains_array() {
    let receipt = json!({"tags": ["a", "b", "c"]});
    let pass = evaluate_policy(&receipt, &policy(vec![rule("c", "/tags", RuleOperator::Contains, json!("b"))]));
    assert!(pass.passed);
}

#[test]
fn op_exists_present_and_absent() {
    let receipt = json!({"x": 1});
    let mut r = PolicyRule {
        name: "ex".into(),
        pointer: "/x".into(),
        op: RuleOperator::Exists,
        value: None,
        values: None,
        negate: false,
        level: RuleLevel::Error,
        message: None,
    };
    let pass = evaluate_policy(&receipt, &policy(vec![r.clone()]));
    assert!(pass.passed);

    r.pointer = "/missing".into();
    let fail = evaluate_policy(&receipt, &policy(vec![r]));
    assert!(!fail.passed);
}

// =========================================================================
// 3. Negate flag
// =========================================================================

#[test]
fn negate_flips_result() {
    let receipt = json!({"val": 10});
    let mut r = rule("neg", "/val", RuleOperator::Eq, json!(10));
    r.negate = true;
    let result = evaluate_policy(&receipt, &policy(vec![r]));
    assert!(!result.passed);
}

#[test]
fn negate_exists_absent() {
    let receipt = json!({"x": 1});
    let r = PolicyRule {
        name: "nex".into(),
        pointer: "/missing".into(),
        op: RuleOperator::Exists,
        value: None,
        values: None,
        negate: true,
        level: RuleLevel::Error,
        message: None,
    };
    let result = evaluate_policy(&receipt, &policy(vec![r]));
    assert!(result.passed);
}

// =========================================================================
// 4. Warn level does not fail gate
// =========================================================================

#[test]
fn warn_level_does_not_block_gate() {
    let receipt = json!({"val": 999});
    let mut r = rule("w", "/val", RuleOperator::Lte, json!(10));
    r.level = RuleLevel::Warn;
    let result = evaluate_policy(&receipt, &policy(vec![r]));
    assert!(result.passed);
    assert_eq!(result.warnings, 1);
    assert_eq!(result.errors, 0);
}

// =========================================================================
// 5. Edge cases: empty rules, missing data, allow_missing
// =========================================================================

#[test]
fn empty_rules_passes() {
    let receipt = json!({"x": 1});
    let result = evaluate_policy(&receipt, &policy(vec![]));
    assert!(result.passed);
    assert_eq!(result.rule_results.len(), 0);
}

#[test]
fn missing_pointer_fails_without_allow_missing() {
    let receipt = json!({"x": 1});
    let result = evaluate_policy(&receipt, &policy(vec![rule("m", "/nope", RuleOperator::Eq, json!(1))]));
    assert!(!result.passed);
}

#[test]
fn missing_pointer_passes_with_allow_missing() {
    let receipt = json!({"x": 1});
    let mut p = policy(vec![rule("m", "/nope", RuleOperator::Eq, json!(1))]);
    p.allow_missing = true;
    let result = evaluate_policy(&receipt, &p);
    assert!(result.passed);
}

// =========================================================================
// 6. Fail-fast stops on first error
// =========================================================================

#[test]
fn fail_fast_stops_after_first_error() {
    let receipt = json!({"a": 999, "b": 999});
    let mut p = policy(vec![
        rule("r1", "/a", RuleOperator::Lte, json!(10)),
        rule("r2", "/b", RuleOperator::Lte, json!(10)),
    ]);
    p.fail_fast = true;
    let result = evaluate_policy(&receipt, &p);
    assert_eq!(result.rule_results.len(), 1);
    assert!(!result.passed);
}

// =========================================================================
// 7. Multiple rules mixing error and warn
// =========================================================================

#[test]
fn mixed_error_and_warn_counts() {
    let receipt = json!({"a": 999, "b": 999});
    let mut r1 = rule("err", "/a", RuleOperator::Lte, json!(10));
    r1.level = RuleLevel::Error;
    let mut r2 = rule("wrn", "/b", RuleOperator::Lte, json!(10));
    r2.level = RuleLevel::Warn;
    let result = evaluate_policy(&receipt, &policy(vec![r1, r2]));
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
    assert_eq!(result.warnings, 1);
}

// =========================================================================
// 8. GateResult / RatchetGateResult from_results
// =========================================================================

#[test]
fn gate_result_from_all_pass() {
    let results = vec![RuleResult {
        name: "ok".into(),
        passed: true,
        level: RuleLevel::Error,
        actual: Some(json!(1)),
        expected: "1".into(),
        message: None,
    }];
    let gate = GateResult::from_results(results);
    assert!(gate.passed);
    assert_eq!(gate.errors, 0);
    assert_eq!(gate.warnings, 0);
}

#[test]
fn ratchet_gate_result_empty_passes() {
    let r = RatchetGateResult::from_results(vec![]);
    assert!(r.passed);
    assert_eq!(r.errors, 0);
}

// =========================================================================
// 9. TOML parsing
// =========================================================================

#[test]
fn policy_config_from_toml_multiple_rules() {
    let toml = r#"
fail_fast = true
allow_missing = true

[[rules]]
name = "r1"
pointer = "/a"
op = "gt"
value = 0

[[rules]]
name = "r2"
pointer = "/b"
op = "exists"
level = "warn"
"#;
    let cfg = PolicyConfig::from_toml(toml).unwrap();
    assert!(cfg.fail_fast);
    assert!(cfg.allow_missing);
    assert_eq!(cfg.rules.len(), 2);
    assert_eq!(cfg.rules[0].op, RuleOperator::Gt);
    assert_eq!(cfg.rules[1].op, RuleOperator::Exists);
    assert_eq!(cfg.rules[1].level, RuleLevel::Warn);
}

#[test]
fn ratchet_config_from_toml_with_max_value() {
    let toml = r#"
allow_missing_baseline = true
allow_missing_current = true

[[rules]]
pointer = "/complexity"
max_value = 50.0
level = "error"
description = "absolute ceiling"
"#;
    let cfg = RatchetConfig::from_toml(toml).unwrap();
    assert!(cfg.allow_missing_baseline);
    assert!(cfg.allow_missing_current);
    assert_eq!(cfg.rules[0].max_value, Some(50.0));
    assert_eq!(cfg.rules[0].description.as_deref(), Some("absolute ceiling"));
}

// =========================================================================
// 10. Ratchet evaluation
// =========================================================================

#[test]
fn ratchet_pass_within_threshold() {
    let baseline = json!({"m": 100.0});
    let current = json!({"m": 109.0});
    let cfg = RatchetConfig {
        rules: vec![ratchet_rule("/m", Some(10.0), None)],
        ..Default::default()
    };
    let r = evaluate_ratchet_policy(&cfg, &baseline, &current);
    assert!(r.passed);
}

#[test]
fn ratchet_fail_exceeds_threshold() {
    let baseline = json!({"m": 100.0});
    let current = json!({"m": 120.0});
    let cfg = RatchetConfig {
        rules: vec![ratchet_rule("/m", Some(10.0), None)],
        ..Default::default()
    };
    let r = evaluate_ratchet_policy(&cfg, &baseline, &current);
    assert!(!r.passed);
    assert_eq!(r.errors, 1);
}

#[test]
fn ratchet_max_value_ceiling() {
    let baseline = json!({"m": 10.0});
    let current = json!({"m": 55.0});
    let cfg = RatchetConfig {
        rules: vec![ratchet_rule("/m", None, Some(50.0))],
        ..Default::default()
    };
    let r = evaluate_ratchet_policy(&cfg, &baseline, &current);
    assert!(!r.passed);
}

#[test]
fn ratchet_zero_baseline_same_current_passes() {
    let baseline = json!({"m": 0.0});
    let current = json!({"m": 0.0});
    let cfg = RatchetConfig {
        rules: vec![ratchet_rule("/m", Some(10.0), None)],
        ..Default::default()
    };
    let r = evaluate_ratchet_policy(&cfg, &baseline, &current);
    assert!(r.passed);
}

#[test]
fn ratchet_missing_baseline_fails_by_default() {
    let baseline = json!({});
    let current = json!({"m": 5.0});
    let cfg = RatchetConfig {
        rules: vec![ratchet_rule("/m", Some(10.0), None)],
        allow_missing_baseline: false,
        ..Default::default()
    };
    let r = evaluate_ratchet_policy(&cfg, &baseline, &current);
    assert!(!r.passed);
}

#[test]
fn ratchet_missing_baseline_allowed() {
    let baseline = json!({});
    let current = json!({"m": 5.0});
    let cfg = RatchetConfig {
        rules: vec![ratchet_rule("/m", Some(10.0), None)],
        allow_missing_baseline: true,
        ..Default::default()
    };
    let r = evaluate_ratchet_policy(&cfg, &baseline, &current);
    assert!(r.passed);
}

#[test]
fn ratchet_missing_current_allowed() {
    let baseline = json!({"m": 5.0});
    let current = json!({});
    let cfg = RatchetConfig {
        rules: vec![ratchet_rule("/m", Some(10.0), None)],
        allow_missing_current: true,
        ..Default::default()
    };
    let r = evaluate_ratchet_policy(&cfg, &baseline, &current);
    assert!(r.passed);
}

// =========================================================================
// 11. Property: determinism (same input → same output)
// =========================================================================

#[cfg(test)]
mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn evaluate_policy_deterministic(val in 0i64..10_000) {
            let receipt = json!({"x": val});
            let p = policy(vec![rule("r", "/x", RuleOperator::Lte, json!(5000))]);
            let r1 = evaluate_policy(&receipt, &p);
            let r2 = evaluate_policy(&receipt, &p);
            prop_assert_eq!(r1.passed, r2.passed);
            prop_assert_eq!(r1.errors, r2.errors);
            prop_assert_eq!(r1.warnings, r2.warnings);
        }

        #[test]
        fn ratchet_deterministic(base in 1.0f64..1000.0, current in 1.0f64..1000.0) {
            let b = json!({"m": base});
            let c = json!({"m": current});
            let cfg = RatchetConfig {
                rules: vec![ratchet_rule("/m", Some(10.0), None)],
                ..Default::default()
            };
            let r1 = evaluate_ratchet_policy(&cfg, &b, &c);
            let r2 = evaluate_ratchet_policy(&cfg, &b, &c);
            prop_assert_eq!(r1.passed, r2.passed);
            prop_assert_eq!(r1.errors, r2.errors);
        }
    }
}
