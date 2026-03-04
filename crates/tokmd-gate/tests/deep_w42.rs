//! Wave 42 deep tests for tokmd-gate.
//!
//! Covers:
//! - JSON pointer rule evaluation with all operators
//! - Policy file parsing (TOML format)
//! - Gate pass/fail logic with fail_fast and allow_missing
//! - Multiple rules with mixed error/warn levels
//! - Threshold comparisons (gt, lt, gte, lte, eq, ne)
//! - Edge cases: missing fields, null values, nested objects
//! - Complex policies with many rules
//! - Negate flag inversion
//! - Contains and In operators
//! - Ratchet evaluation edge cases

use serde_json::json;
use tokmd_gate::{
    GateResult, PolicyConfig, PolicyRule, RatchetConfig, RatchetGateResult, RatchetRule, RuleLevel,
    RuleOperator, RuleResult, evaluate_policy, evaluate_ratchet_policy, resolve_pointer,
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

// =========================================================================
// 1. JSON Pointer resolution
// =========================================================================

#[test]
fn pointer_deeply_nested_object() {
    let doc = json!({"a": {"b": {"c": {"d": 99}}}});
    assert_eq!(resolve_pointer(&doc, "/a/b/c/d"), Some(&json!(99)));
}

#[test]
fn pointer_array_in_nested_object() {
    let doc = json!({"data": {"items": [10, 20, 30]}});
    assert_eq!(resolve_pointer(&doc, "/data/items/1"), Some(&json!(20)));
}

#[test]
fn pointer_null_value_is_some() {
    let doc = json!({"x": null});
    assert_eq!(resolve_pointer(&doc, "/x"), Some(&json!(null)));
}

#[test]
fn pointer_empty_string_key() {
    let doc = json!({"": {"nested": 1}});
    assert_eq!(resolve_pointer(&doc, "//nested"), Some(&json!(1)));
}

#[test]
fn pointer_rfc6901_escape_tilde_and_slash() {
    let doc = json!({"a/b": {"c~d": 42}});
    assert_eq!(resolve_pointer(&doc, "/a~1b/c~0d"), Some(&json!(42)));
}

#[test]
fn pointer_into_scalar_returns_none() {
    let doc = json!({"x": 42});
    assert_eq!(resolve_pointer(&doc, "/x/y"), None);
}

// =========================================================================
// 2. Operator comparisons – gt, lt, gte, lte, eq, ne
// =========================================================================

#[test]
fn op_gt_pass_and_fail() {
    let r = json!({"v": 10});
    let res = eval(&r, vec![rule("gt", "/v", RuleOperator::Gt, json!(5))]);
    assert!(res.passed);

    let res = eval(&r, vec![rule("gt", "/v", RuleOperator::Gt, json!(10))]);
    assert!(!res.passed);
}

#[test]
fn op_lt_pass_and_fail() {
    let r = json!({"v": 3});
    let res = eval(&r, vec![rule("lt", "/v", RuleOperator::Lt, json!(5))]);
    assert!(res.passed);

    let res = eval(&r, vec![rule("lt", "/v", RuleOperator::Lt, json!(3))]);
    assert!(!res.passed);
}

#[test]
fn op_gte_boundary() {
    let r = json!({"v": 10});
    let res = eval(&r, vec![rule("gte", "/v", RuleOperator::Gte, json!(10))]);
    assert!(res.passed);

    let res = eval(&r, vec![rule("gte", "/v", RuleOperator::Gte, json!(11))]);
    assert!(!res.passed);
}

#[test]
fn op_lte_boundary() {
    let r = json!({"v": 10});
    let res = eval(&r, vec![rule("lte", "/v", RuleOperator::Lte, json!(10))]);
    assert!(res.passed);

    let res = eval(&r, vec![rule("lte", "/v", RuleOperator::Lte, json!(9))]);
    assert!(!res.passed);
}

#[test]
fn op_eq_integer() {
    let r = json!({"v": 42});
    let res = eval(&r, vec![rule("eq", "/v", RuleOperator::Eq, json!(42))]);
    assert!(res.passed);

    let res = eval(&r, vec![rule("eq", "/v", RuleOperator::Eq, json!(43))]);
    assert!(!res.passed);
}

#[test]
fn op_eq_string() {
    let r = json!({"lang": "Rust"});
    let res = eval(
        &r,
        vec![rule("eq", "/lang", RuleOperator::Eq, json!("Rust"))],
    );
    assert!(res.passed);
}

#[test]
fn op_ne_pass_and_fail() {
    let r = json!({"v": 10});
    let res = eval(&r, vec![rule("ne", "/v", RuleOperator::Ne, json!(20))]);
    assert!(res.passed);

    let res = eval(&r, vec![rule("ne", "/v", RuleOperator::Ne, json!(10))]);
    assert!(!res.passed);
}

// =========================================================================
// 3. Exists operator
// =========================================================================

#[test]
fn op_exists_present() {
    let r = json!({"key": 1});
    let mut p = rule("ex", "/key", RuleOperator::Exists, json!(null));
    p.value = None;
    let res = eval(&r, vec![p]);
    assert!(res.passed);
}

#[test]
fn op_exists_absent() {
    let r = json!({"key": 1});
    let mut p = rule("ex", "/missing", RuleOperator::Exists, json!(null));
    p.value = None;
    let policy = PolicyConfig {
        rules: vec![p],
        fail_fast: false,
        allow_missing: false,
    };
    let res = evaluate_policy(&r, &policy);
    assert!(!res.passed);
}

#[test]
fn op_exists_negated() {
    let r = json!({"key": 1});
    let mut p = rule("ex", "/missing", RuleOperator::Exists, json!(null));
    p.value = None;
    p.negate = true;
    let res = eval(&r, vec![p]);
    assert!(res.passed);
}

// =========================================================================
// 4. Contains operator
// =========================================================================

#[test]
fn op_contains_string() {
    let r = json!({"desc": "hello world"});
    let res = eval(
        &r,
        vec![rule("c", "/desc", RuleOperator::Contains, json!("world"))],
    );
    assert!(res.passed);

    let res = eval(
        &r,
        vec![rule("c", "/desc", RuleOperator::Contains, json!("xyz"))],
    );
    assert!(!res.passed);
}

#[test]
fn op_contains_array() {
    let r = json!({"tags": ["alpha", "beta"]});
    let res = eval(
        &r,
        vec![rule("c", "/tags", RuleOperator::Contains, json!("alpha"))],
    );
    assert!(res.passed);

    let res = eval(
        &r,
        vec![rule("c", "/tags", RuleOperator::Contains, json!("gamma"))],
    );
    assert!(!res.passed);
}

// =========================================================================
// 5. In operator
// =========================================================================

#[test]
fn op_in_match() {
    let r = json!({"status": "active"});
    let mut p = rule("in", "/status", RuleOperator::In, json!(null));
    p.value = None;
    p.values = Some(vec![json!("active"), json!("pending")]);
    let res = eval(&r, vec![p]);
    assert!(res.passed);
}

#[test]
fn op_in_no_match() {
    let r = json!({"status": "archived"});
    let mut p = rule("in", "/status", RuleOperator::In, json!(null));
    p.value = None;
    p.values = Some(vec![json!("active"), json!("pending")]);
    let res = eval(&r, vec![p]);
    assert!(!res.passed);
}

// =========================================================================
// 6. Negate flag
// =========================================================================

#[test]
fn negate_inverts_lte_result() {
    let r = json!({"v": 5});
    let mut p = rule("n", "/v", RuleOperator::Lte, json!(10));
    p.negate = true;
    // 5 <= 10 is true, negated = false
    let res = eval(&r, vec![p]);
    assert!(!res.passed);
}

#[test]
fn negate_inverts_eq_result() {
    let r = json!({"v": 5});
    let mut p = rule("n", "/v", RuleOperator::Eq, json!(5));
    p.negate = true;
    let res = eval(&r, vec![p]);
    assert!(!res.passed);
}

// =========================================================================
// 7. Missing values, allow_missing
// =========================================================================

#[test]
fn missing_pointer_fails_by_default() {
    let r = json!({"a": 1});
    let res = eval(&r, vec![rule("m", "/b", RuleOperator::Eq, json!(1))]);
    assert!(!res.passed);
    assert_eq!(res.errors, 1);
}

#[test]
fn allow_missing_passes_on_absent_pointer() {
    let r = json!({"a": 1});
    let policy = PolicyConfig {
        rules: vec![rule("m", "/b", RuleOperator::Eq, json!(1))],
        fail_fast: false,
        allow_missing: true,
    };
    let res = evaluate_policy(&r, &policy);
    assert!(res.passed);
}

// =========================================================================
// 8. Warn level does not fail gate
// =========================================================================

#[test]
fn warn_rules_do_not_fail_gate() {
    let r = json!({"v": 999});
    let mut p = rule("w", "/v", RuleOperator::Lte, json!(100));
    p.level = RuleLevel::Warn;
    let res = eval(&r, vec![p]);
    assert!(res.passed);
    assert_eq!(res.warnings, 1);
    assert_eq!(res.errors, 0);
}

// =========================================================================
// 9. Fail-fast stops on first error
// =========================================================================

#[test]
fn fail_fast_stops_after_first_error() {
    let r = json!({"a": 999, "b": 999});
    let policy = PolicyConfig {
        rules: vec![
            rule("r1", "/a", RuleOperator::Lte, json!(10)),
            rule("r2", "/b", RuleOperator::Lte, json!(10)),
        ],
        fail_fast: true,
        allow_missing: false,
    };
    let res = evaluate_policy(&r, &policy);
    assert!(!res.passed);
    // Only 1 rule evaluated because of fail_fast
    assert_eq!(res.rule_results.len(), 1);
}

#[test]
fn fail_fast_does_not_stop_on_warn() {
    let r = json!({"a": 999, "b": 999});
    let mut w = rule("w1", "/a", RuleOperator::Lte, json!(10));
    w.level = RuleLevel::Warn;
    let e = rule("e1", "/b", RuleOperator::Lte, json!(10));
    let policy = PolicyConfig {
        rules: vec![w, e],
        fail_fast: true,
        allow_missing: false,
    };
    let res = evaluate_policy(&r, &policy);
    // Both rules evaluated: warn doesn't trigger fail_fast
    assert_eq!(res.rule_results.len(), 2);
}

// =========================================================================
// 10. Multiple rules mixed pass/fail
// =========================================================================

#[test]
fn multiple_rules_mixed_levels() {
    let r = json!({"tokens": 2000, "files": 3});
    let rules = vec![
        rule("max_tokens", "/tokens", RuleOperator::Lte, json!(1000)),
        rule("min_files", "/files", RuleOperator::Gte, json!(1)),
    ];
    let res = eval(&r, rules);
    assert!(!res.passed);
    assert_eq!(res.errors, 1);
    assert_eq!(res.rule_results.len(), 2);
    assert!(!res.rule_results[0].passed);
    assert!(res.rule_results[1].passed);
}

// =========================================================================
// 11. TOML parsing – complex policy
// =========================================================================

#[test]
fn toml_complex_policy_with_all_options() {
    let toml = r#"
fail_fast = true
allow_missing = true

[[rules]]
name = "max_tokens"
pointer = "/derived/totals/tokens"
op = "lte"
value = 500000
level = "error"
message = "Too many tokens"

[[rules]]
name = "has_license"
pointer = "/license/effective"
op = "exists"
level = "warn"

[[rules]]
name = "lang_in_set"
pointer = "/lang"
op = "in"
values = ["Rust", "Go", "Python"]
level = "error"
"#;
    let policy = PolicyConfig::from_toml(toml).unwrap();
    assert!(policy.fail_fast);
    assert!(policy.allow_missing);
    assert_eq!(policy.rules.len(), 3);
    assert_eq!(policy.rules[0].op, RuleOperator::Lte);
    assert_eq!(policy.rules[1].op, RuleOperator::Exists);
    assert_eq!(policy.rules[2].op, RuleOperator::In);
    assert!(policy.rules[2].values.is_some());
    assert_eq!(policy.rules[2].values.as_ref().unwrap().len(), 3);
}

#[test]
fn toml_minimal_policy() {
    let toml = r#"
[[rules]]
name = "check"
pointer = "/x"
op = "eq"
value = 1
"#;
    let policy = PolicyConfig::from_toml(toml).unwrap();
    assert!(!policy.fail_fast);
    assert!(!policy.allow_missing);
    assert_eq!(policy.rules.len(), 1);
    assert_eq!(policy.rules[0].level, RuleLevel::Error); // default
}

#[test]
fn toml_invalid_returns_error() {
    let toml = "not valid toml [[[[";
    assert!(PolicyConfig::from_toml(toml).is_err());
}

// =========================================================================
// 12. GateResult construction invariants
// =========================================================================

#[test]
fn gate_result_only_errors_fail() {
    let results = vec![
        RuleResult {
            name: "pass".into(),
            passed: true,
            level: RuleLevel::Error,
            actual: None,
            expected: String::new(),
            message: None,
        },
        RuleResult {
            name: "warn".into(),
            passed: false,
            level: RuleLevel::Warn,
            actual: None,
            expected: String::new(),
            message: None,
        },
    ];
    let gate = GateResult::from_results(results);
    assert!(gate.passed); // only warns, no errors
    assert_eq!(gate.warnings, 1);
}

#[test]
fn gate_result_single_error_fails() {
    let results = vec![RuleResult {
        name: "err".into(),
        passed: false,
        level: RuleLevel::Error,
        actual: None,
        expected: String::new(),
        message: None,
    }];
    let gate = GateResult::from_results(results);
    assert!(!gate.passed);
    assert_eq!(gate.errors, 1);
}

// =========================================================================
// 13. Numeric comparison with float values
// =========================================================================

#[test]
fn float_comparison_lte() {
    let r = json!({"ratio": 0.85});
    let res = eval(&r, vec![rule("f", "/ratio", RuleOperator::Lte, json!(0.9))]);
    assert!(res.passed);
}

#[test]
fn float_comparison_gt() {
    let r = json!({"ratio": 0.95});
    let res = eval(&r, vec![rule("f", "/ratio", RuleOperator::Gt, json!(0.9))]);
    assert!(res.passed);
}

// =========================================================================
// 14. String-parseable numbers
// =========================================================================

#[test]
fn string_numeric_comparison() {
    // evaluate.rs value_to_f64 parses strings as numbers
    let r = json!({"v": "42"});
    let res = eval(&r, vec![rule("s", "/v", RuleOperator::Eq, json!(42))]);
    assert!(res.passed);
}

// =========================================================================
// 15. Contains on non-string/non-array fails gracefully
// =========================================================================

#[test]
fn contains_on_number_fails() {
    let r = json!({"v": 42});
    let res = eval(
        &r,
        vec![rule("c", "/v", RuleOperator::Contains, json!("2"))],
    );
    assert!(!res.passed);
}

// =========================================================================
// 16-20. Ratchet evaluation
// =========================================================================

#[test]
fn ratchet_no_regression_passes() {
    let baseline = json!({"complexity": 10.0});
    let current = json!({"complexity": 10.0});
    let config = RatchetConfig {
        rules: vec![ratchet_rule("/complexity", Some(5.0), None)],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let res = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(res.passed);
    assert_eq!(res.ratchet_results[0].change_pct, Some(0.0));
}

#[test]
fn ratchet_exceeds_max_increase_pct_fails() {
    let baseline = json!({"complexity": 10.0});
    let current = json!({"complexity": 12.0}); // 20% increase
    let config = RatchetConfig {
        rules: vec![ratchet_rule("/complexity", Some(10.0), None)],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let res = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(!res.passed);
}

#[test]
fn ratchet_max_value_ceiling() {
    let baseline = json!({"lines": 100.0});
    let current = json!({"lines": 200.0});
    let config = RatchetConfig {
        rules: vec![ratchet_rule("/lines", None, Some(150.0))],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let res = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(!res.passed);
}

#[test]
fn ratchet_missing_current_strict_fails() {
    let baseline = json!({"v": 10.0});
    let current = json!({});
    let config = RatchetConfig {
        rules: vec![ratchet_rule("/v", Some(5.0), None)],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let res = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(!res.passed);
}

#[test]
fn ratchet_missing_current_lenient_passes() {
    let baseline = json!({"v": 10.0});
    let current = json!({});
    let config = RatchetConfig {
        rules: vec![ratchet_rule("/v", Some(5.0), None)],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: true,
    };
    let res = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(res.passed);
}

#[test]
fn ratchet_missing_baseline_strict_fails() {
    let baseline = json!({});
    let current = json!({"v": 10.0});
    let config = RatchetConfig {
        rules: vec![ratchet_rule("/v", Some(5.0), None)],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let res = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(!res.passed);
}

#[test]
fn ratchet_missing_baseline_lenient_passes() {
    let baseline = json!({});
    let current = json!({"v": 10.0});
    let config = RatchetConfig {
        rules: vec![ratchet_rule("/v", Some(5.0), None)],
        fail_fast: false,
        allow_missing_baseline: true,
        allow_missing_current: false,
    };
    let res = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(res.passed);
}

#[test]
fn ratchet_zero_baseline_nonzero_current_is_infinity() {
    let baseline = json!({"v": 0.0});
    let current = json!({"v": 5.0});
    let config = RatchetConfig {
        rules: vec![ratchet_rule("/v", Some(100.0), None)],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let res = evaluate_ratchet_policy(&config, &baseline, &current);
    // Infinity > 100.0, so should fail
    assert!(!res.passed);
}

#[test]
fn ratchet_zero_to_zero_is_zero_pct() {
    let baseline = json!({"v": 0.0});
    let current = json!({"v": 0.0});
    let config = RatchetConfig {
        rules: vec![ratchet_rule("/v", Some(5.0), None)],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let res = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(res.passed);
    assert_eq!(res.ratchet_results[0].change_pct, Some(0.0));
}

#[test]
fn ratchet_config_from_toml_roundtrip() {
    let toml = r#"
fail_fast = true
allow_missing_baseline = true
allow_missing_current = false

[[rules]]
pointer = "/complexity/avg"
max_increase_pct = 5.0
level = "warn"
description = "Keep complexity low"
"#;
    let config = RatchetConfig::from_toml(toml).unwrap();
    assert!(config.fail_fast);
    assert!(config.allow_missing_baseline);
    assert!(!config.allow_missing_current);
    assert_eq!(config.rules.len(), 1);
    assert_eq!(config.rules[0].level, RuleLevel::Warn);
    assert_eq!(
        config.rules[0].description.as_deref(),
        Some("Keep complexity low")
    );
}

#[test]
fn ratchet_gate_result_mixed_warn_and_error() {
    let baseline = json!({"a": 10.0, "b": 10.0});
    let current = json!({"a": 20.0, "b": 20.0}); // 100% increase on both

    let mut warn_rule = ratchet_rule("/a", Some(50.0), None);
    warn_rule.level = RuleLevel::Warn;
    let error_rule = ratchet_rule("/b", Some(50.0), None);

    let config = RatchetConfig {
        rules: vec![warn_rule, error_rule],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let res = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(!res.passed);
    assert_eq!(res.errors, 1);
    assert_eq!(res.warnings, 1);
}

#[test]
fn ratchet_fail_fast_stops_on_error() {
    let baseline = json!({"a": 10.0, "b": 10.0});
    let current = json!({"a": 100.0, "b": 100.0}); // huge increase

    let config = RatchetConfig {
        rules: vec![
            ratchet_rule("/a", Some(5.0), None),
            ratchet_rule("/b", Some(5.0), None),
        ],
        fail_fast: true,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let res = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(!res.passed);
    assert_eq!(res.ratchet_results.len(), 1);
}

// =========================================================================
// 21. Empty policy always passes
// =========================================================================

#[test]
fn empty_policy_passes() {
    let r = json!({"anything": true});
    let policy = PolicyConfig::default();
    let res = evaluate_policy(&r, &policy);
    assert!(res.passed);
    assert_eq!(res.errors, 0);
}

// =========================================================================
// 22. Custom message on failure
// =========================================================================

#[test]
fn custom_message_on_failure() {
    let r = json!({"v": 999});
    let mut p = rule("m", "/v", RuleOperator::Lte, json!(10));
    p.message = Some("Value way too high".into());
    let res = eval(&r, vec![p]);
    assert_eq!(
        res.rule_results[0].message.as_deref(),
        Some("Value way too high")
    );
}

#[test]
fn no_message_on_pass() {
    let r = json!({"v": 1});
    let mut p = rule("m", "/v", RuleOperator::Lte, json!(10));
    p.message = Some("Should not appear".into());
    let res = eval(&r, vec![p]);
    assert!(res.rule_results[0].message.is_none());
}

// =========================================================================
// 23. Operator serialization round-trip via TOML
// =========================================================================

#[test]
fn toml_all_operators() {
    for (op_str, expected_op) in [
        ("gt", RuleOperator::Gt),
        ("gte", RuleOperator::Gte),
        ("lt", RuleOperator::Lt),
        ("lte", RuleOperator::Lte),
        ("eq", RuleOperator::Eq),
        ("ne", RuleOperator::Ne),
        ("contains", RuleOperator::Contains),
        ("exists", RuleOperator::Exists),
    ] {
        let toml = format!(
            r#"
[[rules]]
name = "op_test"
pointer = "/x"
op = "{op_str}"
"#
        );
        let policy = PolicyConfig::from_toml(&toml).unwrap();
        assert_eq!(
            policy.rules[0].op, expected_op,
            "operator mismatch for {op_str}"
        );
    }
}

// =========================================================================
// 24. RatchetGateResult from_results empty
// =========================================================================

#[test]
fn ratchet_gate_empty_is_pass() {
    let res = RatchetGateResult::from_results(vec![]);
    assert!(res.passed);
    assert_eq!(res.errors, 0);
    assert_eq!(res.warnings, 0);
}
