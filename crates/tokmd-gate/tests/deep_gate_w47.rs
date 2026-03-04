//! Wave 47 deep integration and property tests for tokmd-gate.
//!
//! Covers:
//! - JSON pointer rule evaluation (dot notation paths)
//! - Threshold comparisons (gt, lt, eq, gte, lte)
//! - Multiple gate rules with mixed pass/fail
//! - Missing pointer paths (should fail gracefully)
//! - Gate result aggregation (all pass, some fail, all fail)
//! - Property test: valid rules always produce a result (never panic)
//! - Edge cases: empty rules, deeply nested pointers, numeric vs string comparisons

use proptest::prelude::*;
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

// =========================================================================
// 1. JSON pointer rule evaluation
// =========================================================================

#[test]
fn pointer_simple_key() {
    let doc = json!({"tokens": 42});
    assert_eq!(resolve_pointer(&doc, "/tokens"), Some(&json!(42)));
}

#[test]
fn pointer_nested_path() {
    let doc = json!({"derived": {"totals": {"tokens": 500}}});
    assert_eq!(
        resolve_pointer(&doc, "/derived/totals/tokens"),
        Some(&json!(500))
    );
}

#[test]
fn pointer_array_index() {
    let doc = json!({"items": [10, 20, 30]});
    assert_eq!(resolve_pointer(&doc, "/items/1"), Some(&json!(20)));
}

#[test]
fn pointer_deeply_nested_5_levels() {
    let doc = json!({"a": {"b": {"c": {"d": {"e": 99}}}}});
    assert_eq!(resolve_pointer(&doc, "/a/b/c/d/e"), Some(&json!(99)));
}

#[test]
fn pointer_rfc6901_tilde_escape() {
    let doc = json!({"a~b": 1});
    assert_eq!(resolve_pointer(&doc, "/a~0b"), Some(&json!(1)));
}

#[test]
fn pointer_rfc6901_slash_escape() {
    let doc = json!({"a/b": 2});
    assert_eq!(resolve_pointer(&doc, "/a~1b"), Some(&json!(2)));
}

#[test]
fn pointer_empty_returns_whole_doc() {
    let doc = json!({"x": 1});
    assert_eq!(resolve_pointer(&doc, ""), Some(&doc));
}

#[test]
fn pointer_missing_returns_none() {
    let doc = json!({"x": 1});
    assert_eq!(resolve_pointer(&doc, "/y"), None);
}

#[test]
fn pointer_no_leading_slash_returns_none() {
    let doc = json!({"x": 1});
    assert_eq!(resolve_pointer(&doc, "x"), None);
}

#[test]
fn pointer_into_scalar_returns_none() {
    let doc = json!({"x": 42});
    assert_eq!(resolve_pointer(&doc, "/x/y"), None);
}

// =========================================================================
// 2. Threshold comparisons
// =========================================================================

#[test]
fn threshold_gt_pass() {
    let receipt = json!({"val": 100});
    let result = eval(
        &receipt,
        vec![rule("gt", "/val", RuleOperator::Gt, json!(50))],
    );
    assert!(result.passed);
}

#[test]
fn threshold_gt_fail() {
    let receipt = json!({"val": 30});
    let result = eval(
        &receipt,
        vec![rule("gt", "/val", RuleOperator::Gt, json!(50))],
    );
    assert!(!result.passed);
}

#[test]
fn threshold_lt_pass() {
    let receipt = json!({"val": 10});
    let result = eval(
        &receipt,
        vec![rule("lt", "/val", RuleOperator::Lt, json!(50))],
    );
    assert!(result.passed);
}

#[test]
fn threshold_lt_fail() {
    let receipt = json!({"val": 100});
    let result = eval(
        &receipt,
        vec![rule("lt", "/val", RuleOperator::Lt, json!(50))],
    );
    assert!(!result.passed);
}

#[test]
fn threshold_eq_pass() {
    let receipt = json!({"val": 42});
    let result = eval(
        &receipt,
        vec![rule("eq", "/val", RuleOperator::Eq, json!(42))],
    );
    assert!(result.passed);
}

#[test]
fn threshold_eq_fail() {
    let receipt = json!({"val": 43});
    let result = eval(
        &receipt,
        vec![rule("eq", "/val", RuleOperator::Eq, json!(42))],
    );
    assert!(!result.passed);
}

#[test]
fn threshold_gte_boundary() {
    let receipt = json!({"val": 50});
    let result = eval(
        &receipt,
        vec![rule("gte", "/val", RuleOperator::Gte, json!(50))],
    );
    assert!(result.passed);
}

#[test]
fn threshold_lte_boundary() {
    let receipt = json!({"val": 50});
    let result = eval(
        &receipt,
        vec![rule("lte", "/val", RuleOperator::Lte, json!(50))],
    );
    assert!(result.passed);
}

#[test]
fn threshold_ne_pass() {
    let receipt = json!({"val": 10});
    let result = eval(
        &receipt,
        vec![rule("ne", "/val", RuleOperator::Ne, json!(20))],
    );
    assert!(result.passed);
}

#[test]
fn threshold_ne_fail() {
    let receipt = json!({"val": 20});
    let result = eval(
        &receipt,
        vec![rule("ne", "/val", RuleOperator::Ne, json!(20))],
    );
    assert!(!result.passed);
}

// =========================================================================
// 3. Multiple gate rules with mixed pass/fail
// =========================================================================

#[test]
fn mixed_rules_all_pass() {
    let receipt = json!({"tokens": 100, "files": 5});
    let result = eval(
        &receipt,
        vec![
            rule("max_tokens", "/tokens", RuleOperator::Lte, json!(1000)),
            rule("min_files", "/files", RuleOperator::Gte, json!(1)),
        ],
    );
    assert!(result.passed);
    assert_eq!(result.errors, 0);
    assert_eq!(result.rule_results.len(), 2);
}

#[test]
fn mixed_rules_one_fail() {
    let receipt = json!({"tokens": 2000, "files": 5});
    let result = eval(
        &receipt,
        vec![
            rule("max_tokens", "/tokens", RuleOperator::Lte, json!(1000)),
            rule("min_files", "/files", RuleOperator::Gte, json!(1)),
        ],
    );
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
}

#[test]
fn mixed_rules_all_fail() {
    let receipt = json!({"tokens": 2000, "files": 0});
    let result = eval(
        &receipt,
        vec![
            rule("max_tokens", "/tokens", RuleOperator::Lte, json!(1000)),
            rule("min_files", "/files", RuleOperator::Gt, json!(0)),
        ],
    );
    assert!(!result.passed);
    assert_eq!(result.errors, 2);
}

#[test]
fn mixed_error_and_warn_only_errors_fail() {
    let receipt = json!({"tokens": 2000, "files": 0});
    let result = eval(
        &receipt,
        vec![
            warn_rule("token_warn", "/tokens", RuleOperator::Lte, json!(1000)),
            warn_rule("file_warn", "/files", RuleOperator::Gt, json!(0)),
        ],
    );
    // Warnings don't fail the gate
    assert!(result.passed);
    assert_eq!(result.warnings, 2);
    assert_eq!(result.errors, 0);
}

// =========================================================================
// 4. Missing pointer paths
// =========================================================================

#[test]
fn missing_pointer_fails_when_not_allowed() {
    let receipt = json!({"tokens": 100});
    let policy = PolicyConfig {
        rules: vec![rule(
            "missing",
            "/nonexistent",
            RuleOperator::Lte,
            json!(1000),
        )],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
    assert!(
        result.rule_results[0]
            .message
            .as_ref()
            .unwrap()
            .contains("not found")
    );
}

#[test]
fn missing_pointer_passes_when_allowed() {
    let receipt = json!({"tokens": 100});
    let policy = PolicyConfig {
        rules: vec![rule(
            "missing",
            "/nonexistent",
            RuleOperator::Lte,
            json!(1000),
        )],
        fail_fast: false,
        allow_missing: true,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed);
    assert_eq!(result.errors, 0);
}

#[test]
fn missing_nested_pointer_fails() {
    let receipt = json!({"a": {"b": 1}});
    let result = eval(
        &receipt,
        vec![rule("deep", "/a/c/d", RuleOperator::Eq, json!(1))],
    );
    assert!(!result.passed);
}

// =========================================================================
// 5. Gate result aggregation
// =========================================================================

#[test]
fn gate_result_from_empty_rules_passes() {
    let result = GateResult::from_results(vec![]);
    assert!(result.passed);
    assert_eq!(result.errors, 0);
    assert_eq!(result.warnings, 0);
}

#[test]
fn gate_result_counts_errors_correctly() {
    let results = vec![
        RuleResult {
            name: "r1".into(),
            passed: false,
            level: RuleLevel::Error,
            actual: None,
            expected: "x".into(),
            message: None,
        },
        RuleResult {
            name: "r2".into(),
            passed: false,
            level: RuleLevel::Error,
            actual: None,
            expected: "y".into(),
            message: None,
        },
        RuleResult {
            name: "r3".into(),
            passed: true,
            level: RuleLevel::Error,
            actual: None,
            expected: "z".into(),
            message: None,
        },
    ];
    let gate = GateResult::from_results(results);
    assert!(!gate.passed);
    assert_eq!(gate.errors, 2);
    assert_eq!(gate.warnings, 0);
}

#[test]
fn gate_result_counts_warnings_separately() {
    let results = vec![
        RuleResult {
            name: "w1".into(),
            passed: false,
            level: RuleLevel::Warn,
            actual: None,
            expected: "x".into(),
            message: None,
        },
        RuleResult {
            name: "e1".into(),
            passed: false,
            level: RuleLevel::Error,
            actual: None,
            expected: "y".into(),
            message: None,
        },
    ];
    let gate = GateResult::from_results(results);
    assert!(!gate.passed);
    assert_eq!(gate.errors, 1);
    assert_eq!(gate.warnings, 1);
}

#[test]
fn gate_result_passed_warnings_not_counted() {
    let results = vec![RuleResult {
        name: "w1".into(),
        passed: true,
        level: RuleLevel::Warn,
        actual: None,
        expected: "x".into(),
        message: None,
    }];
    let gate = GateResult::from_results(results);
    assert!(gate.passed);
    assert_eq!(gate.warnings, 0);
}

// =========================================================================
// 6. Edge cases
// =========================================================================

#[test]
fn empty_rules_policy_passes() {
    let receipt = json!({"anything": true});
    let policy = PolicyConfig::default();
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed);
}

#[test]
fn exists_operator_pass() {
    let receipt = json!({"key": "value"});
    let mut r = rule("exists", "/key", RuleOperator::Exists, json!(null));
    r.value = None;
    let result = eval(&receipt, vec![r]);
    assert!(result.passed);
}

#[test]
fn exists_operator_fail() {
    let receipt = json!({"key": "value"});
    let mut r = rule("not_exists", "/missing", RuleOperator::Exists, json!(null));
    r.value = None;
    let result = eval(&receipt, vec![r]);
    assert!(!result.passed);
}

#[test]
fn exists_negated_pass() {
    let receipt = json!({"key": "value"});
    let r = PolicyRule {
        name: "no_secret".into(),
        pointer: "/secret".into(),
        op: RuleOperator::Exists,
        value: None,
        values: None,
        negate: true,
        level: RuleLevel::Error,
        message: None,
    };
    let result = eval(&receipt, vec![r]);
    assert!(result.passed);
}

#[test]
fn contains_string_pass() {
    let receipt = json!({"name": "hello world"});
    let result = eval(
        &receipt,
        vec![rule(
            "has_hello",
            "/name",
            RuleOperator::Contains,
            json!("hello"),
        )],
    );
    assert!(result.passed);
}

#[test]
fn contains_string_fail() {
    let receipt = json!({"name": "hello world"});
    let result = eval(
        &receipt,
        vec![rule(
            "has_foo",
            "/name",
            RuleOperator::Contains,
            json!("foo"),
        )],
    );
    assert!(!result.passed);
}

#[test]
fn contains_array_pass() {
    let receipt = json!({"tags": ["rust", "cli", "tool"]});
    let result = eval(
        &receipt,
        vec![rule(
            "has_rust",
            "/tags",
            RuleOperator::Contains,
            json!("rust"),
        )],
    );
    assert!(result.passed);
}

#[test]
fn in_operator_pass() {
    let receipt = json!({"lang": "rust"});
    let r = PolicyRule {
        name: "lang_check".into(),
        pointer: "/lang".into(),
        op: RuleOperator::In,
        value: None,
        values: Some(vec![json!("rust"), json!("go"), json!("python")]),
        negate: false,
        level: RuleLevel::Error,
        message: None,
    };
    let result = eval(&receipt, vec![r]);
    assert!(result.passed);
}

#[test]
fn in_operator_fail() {
    let receipt = json!({"lang": "java"});
    let r = PolicyRule {
        name: "lang_check".into(),
        pointer: "/lang".into(),
        op: RuleOperator::In,
        value: None,
        values: Some(vec![json!("rust"), json!("go"), json!("python")]),
        negate: false,
        level: RuleLevel::Error,
        message: None,
    };
    let result = eval(&receipt, vec![r]);
    assert!(!result.passed);
}

#[test]
fn numeric_string_compared_as_number() {
    // String "100" should be parsed as f64 for numeric comparison
    let receipt = json!({"val": "100"});
    let result = eval(
        &receipt,
        vec![rule("lte", "/val", RuleOperator::Lte, json!(200))],
    );
    assert!(result.passed);
}

#[test]
fn negate_inverts_result() {
    let receipt = json!({"val": 100});
    let r = PolicyRule {
        name: "not_gt".into(),
        pointer: "/val".into(),
        op: RuleOperator::Gt,
        value: Some(json!(50)),
        values: None,
        negate: true,
        level: RuleLevel::Error,
        message: None,
    };
    // val > 50 is true, negated → false → fail
    let result = eval(&receipt, vec![r]);
    assert!(!result.passed);
}

#[test]
fn fail_fast_stops_after_first_error() {
    let receipt = json!({"a": 100, "b": 200});
    let policy = PolicyConfig {
        rules: vec![
            rule("a_check", "/a", RuleOperator::Lt, json!(50)), // fails
            rule("b_check", "/b", RuleOperator::Lt, json!(500)), // would pass
        ],
        fail_fast: true,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
    // Only 1 rule evaluated due to fail_fast
    assert_eq!(result.rule_results.len(), 1);
}

#[test]
fn custom_message_on_failure() {
    let receipt = json!({"val": 2000});
    let r = PolicyRule {
        name: "max".into(),
        pointer: "/val".into(),
        op: RuleOperator::Lte,
        value: Some(json!(1000)),
        values: None,
        negate: false,
        level: RuleLevel::Error,
        message: Some("Value too high!".into()),
    };
    let result = eval(&receipt, vec![r]);
    assert_eq!(
        result.rule_results[0].message.as_deref(),
        Some("Value too high!")
    );
}

#[test]
fn custom_message_not_shown_on_pass() {
    let receipt = json!({"val": 100});
    let r = PolicyRule {
        name: "max".into(),
        pointer: "/val".into(),
        op: RuleOperator::Lte,
        value: Some(json!(1000)),
        values: None,
        negate: false,
        level: RuleLevel::Error,
        message: Some("Value too high!".into()),
    };
    let result = eval(&receipt, vec![r]);
    assert!(result.rule_results[0].message.is_none());
}

// =========================================================================
// 7. TOML parsing
// =========================================================================

#[test]
fn toml_roundtrip_complex_policy() {
    let toml = r#"
fail_fast = true
allow_missing = false

[[rules]]
name = "max_tokens"
pointer = "/tokens"
op = "lte"
value = 500000
level = "error"
message = "Too many tokens"

[[rules]]
name = "min_coverage"
pointer = "/coverage"
op = "gte"
value = 0.8
level = "warn"

[[rules]]
name = "has_license"
pointer = "/license"
op = "exists"
level = "error"
"#;
    let policy = PolicyConfig::from_toml(toml).unwrap();
    assert!(policy.fail_fast);
    assert!(!policy.allow_missing);
    assert_eq!(policy.rules.len(), 3);
    assert_eq!(policy.rules[0].op, RuleOperator::Lte);
    assert_eq!(policy.rules[1].op, RuleOperator::Gte);
    assert_eq!(policy.rules[1].level, RuleLevel::Warn);
    assert_eq!(policy.rules[2].op, RuleOperator::Exists);
}

#[test]
fn toml_defaults_applied() {
    let toml = r#"
[[rules]]
name = "check"
pointer = "/val"
op = "eq"
value = 1
"#;
    let policy = PolicyConfig::from_toml(toml).unwrap();
    assert!(!policy.fail_fast);
    assert!(!policy.allow_missing);
    assert_eq!(policy.rules[0].level, RuleLevel::Error);
    assert!(!policy.rules[0].negate);
}

// =========================================================================
// 8. Ratchet evaluation
// =========================================================================

#[test]
fn ratchet_within_threshold_passes() {
    let baseline = json!({"complexity": 10.0});
    let current = json!({"complexity": 10.5}); // 5% increase
    let config = RatchetConfig {
        rules: vec![ratchet_rule("/complexity", Some(10.0), None)],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(result.passed);
}

#[test]
fn ratchet_over_threshold_fails() {
    let baseline = json!({"complexity": 10.0});
    let current = json!({"complexity": 15.0}); // 50% increase
    let config = RatchetConfig {
        rules: vec![ratchet_rule("/complexity", Some(10.0), None)],
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
    let baseline = json!({"complexity": 5.0});
    let current = json!({"complexity": 25.0}); // under max_value of 30
    let config = RatchetConfig {
        rules: vec![ratchet_rule("/complexity", None, Some(30.0))],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(result.passed);
}

#[test]
fn ratchet_max_value_exceeded() {
    let baseline = json!({"complexity": 5.0});
    let current = json!({"complexity": 35.0}); // exceeds max_value 30
    let config = RatchetConfig {
        rules: vec![ratchet_rule("/complexity", None, Some(30.0))],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(!result.passed);
}

#[test]
fn ratchet_empty_rules_passes() {
    let baseline = json!({});
    let current = json!({});
    let config = RatchetConfig::default();
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(result.passed);
    assert_eq!(result.errors, 0);
}

#[test]
fn ratchet_gate_result_aggregation() {
    let result = RatchetGateResult::from_results(vec![]);
    assert!(result.passed);
    assert_eq!(result.errors, 0);
    assert_eq!(result.warnings, 0);
}

// =========================================================================
// 9. Property tests
// =========================================================================

fn arb_operator() -> impl Strategy<Value = RuleOperator> {
    prop_oneof![
        Just(RuleOperator::Gt),
        Just(RuleOperator::Gte),
        Just(RuleOperator::Lt),
        Just(RuleOperator::Lte),
        Just(RuleOperator::Eq),
        Just(RuleOperator::Ne),
    ]
}

proptest! {
    /// Any valid rule with a numeric receipt value always produces a result
    /// without panicking.
    #[test]
    fn prop_valid_rule_never_panics(
        actual in -1000.0f64..1000.0,
        threshold in -1000.0f64..1000.0,
        op in arb_operator(),
    ) {
        let receipt = json!({"val": actual});
        let result = eval(&receipt, vec![rule("check", "/val", op, json!(threshold))]);
        // Result always has exactly one rule result
        prop_assert_eq!(result.rule_results.len(), 1);
        // errors + warnings ≤ total rules
        prop_assert!(result.errors + result.warnings <= 1);
    }

    /// Empty policy always passes regardless of receipt content.
    #[test]
    fn prop_empty_policy_always_passes(val in -1000i64..1000) {
        let receipt = json!({"x": val});
        let policy = PolicyConfig::default();
        let result = evaluate_policy(&receipt, &policy);
        prop_assert!(result.passed);
    }

    /// Missing pointer with allow_missing always passes.
    #[test]
    fn prop_allow_missing_always_passes(
        op in arb_operator(),
        threshold in -100.0f64..100.0,
    ) {
        let receipt = json!({}); // empty receipt
        let policy = PolicyConfig {
            rules: vec![rule("check", "/missing", op, json!(threshold))],
            fail_fast: false,
            allow_missing: true,
        };
        let result = evaluate_policy(&receipt, &policy);
        prop_assert!(result.passed);
    }

    /// GateResult error count matches number of failed error-level rules.
    #[test]
    fn prop_gate_result_error_count_consistent(
        pass_count in 0usize..10,
        fail_count in 0usize..10,
        warn_count in 0usize..10,
    ) {
        let mut results = Vec::new();
        for i in 0..pass_count {
            results.push(RuleResult {
                name: format!("pass_{i}"),
                passed: true,
                level: RuleLevel::Error,
                actual: None,
                expected: "x".into(),
                message: None,
            });
        }
        for i in 0..fail_count {
            results.push(RuleResult {
                name: format!("fail_{i}"),
                passed: false,
                level: RuleLevel::Error,
                actual: None,
                expected: "x".into(),
                message: None,
            });
        }
        for i in 0..warn_count {
            results.push(RuleResult {
                name: format!("warn_{i}"),
                passed: false,
                level: RuleLevel::Warn,
                actual: None,
                expected: "x".into(),
                message: None,
            });
        }

        let gate = GateResult::from_results(results);
        prop_assert_eq!(gate.errors, fail_count);
        prop_assert_eq!(gate.warnings, warn_count);
        prop_assert_eq!(gate.passed, fail_count == 0);
    }

    /// Resolve pointer on empty string always returns the whole doc.
    #[test]
    fn prop_empty_pointer_returns_root(val in -1000i64..1000) {
        let doc = json!({"val": val});
        prop_assert_eq!(resolve_pointer(&doc, ""), Some(&doc));
    }
}
