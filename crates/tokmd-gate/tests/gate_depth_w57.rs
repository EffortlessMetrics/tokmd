//! Depth tests for tokmd-gate (w57).
//!
//! Exercises JSON pointer resolution, threshold-based gates, nested JSON,
//! gate outcomes, missing keys, null values, deterministic ordering,
//! ratchet evaluation, serde roundtrips, and TOML parsing.

use serde_json::json;
use tokmd_gate::*;

// ═══════════════════════════════════════════════════════════════════
// Helper builders
// ═══════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════
// 1. JSON Pointer resolution
// ═══════════════════════════════════════════════════════════════════

#[test]
fn pointer_root() {
    let doc = json!({"a": 1});
    assert_eq!(resolve_pointer(&doc, ""), Some(&doc));
}

#[test]
fn pointer_single_key() {
    let doc = json!({"x": 42});
    assert_eq!(resolve_pointer(&doc, "/x"), Some(&json!(42)));
}

#[test]
fn pointer_nested() {
    let doc = json!({"a": {"b": {"c": 99}}});
    assert_eq!(resolve_pointer(&doc, "/a/b/c"), Some(&json!(99)));
}

#[test]
fn pointer_array_index() {
    let doc = json!({"items": [10, 20, 30]});
    assert_eq!(resolve_pointer(&doc, "/items/1"), Some(&json!(20)));
}

#[test]
fn pointer_missing_key() {
    let doc = json!({"a": 1});
    assert_eq!(resolve_pointer(&doc, "/missing"), None);
}

#[test]
fn pointer_deep_missing() {
    let doc = json!({"a": {"b": 1}});
    assert_eq!(resolve_pointer(&doc, "/a/c/d"), None);
}

// ═══════════════════════════════════════════════════════════════════
// 2. Threshold-based gates (>, <, ==, >=, <=)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn gt_pass() {
    let receipt = json!({"score": 100});
    let p = policy(vec![rule("check", "/score", RuleOperator::Gt, json!(50))]);
    let result = evaluate_policy(&receipt, &p);
    assert!(result.passed);
}

#[test]
fn gt_fail_when_equal() {
    let receipt = json!({"score": 50});
    let p = policy(vec![rule("check", "/score", RuleOperator::Gt, json!(50))]);
    let result = evaluate_policy(&receipt, &p);
    assert!(!result.passed);
}

#[test]
fn lt_pass() {
    let receipt = json!({"score": 10});
    let p = policy(vec![rule("check", "/score", RuleOperator::Lt, json!(50))]);
    assert!(evaluate_policy(&receipt, &p).passed);
}

#[test]
fn lt_fail() {
    let receipt = json!({"score": 100});
    let p = policy(vec![rule("check", "/score", RuleOperator::Lt, json!(50))]);
    assert!(!evaluate_policy(&receipt, &p).passed);
}

#[test]
fn gte_pass_equal() {
    let receipt = json!({"score": 50});
    let p = policy(vec![rule("check", "/score", RuleOperator::Gte, json!(50))]);
    assert!(evaluate_policy(&receipt, &p).passed);
}

#[test]
fn lte_pass_equal() {
    let receipt = json!({"score": 50});
    let p = policy(vec![rule("check", "/score", RuleOperator::Lte, json!(50))]);
    assert!(evaluate_policy(&receipt, &p).passed);
}

#[test]
fn eq_pass() {
    let receipt = json!({"name": "tokmd"});
    let p = policy(vec![rule(
        "check",
        "/name",
        RuleOperator::Eq,
        json!("tokmd"),
    )]);
    assert!(evaluate_policy(&receipt, &p).passed);
}

#[test]
fn ne_pass() {
    let receipt = json!({"name": "other"});
    let p = policy(vec![rule(
        "check",
        "/name",
        RuleOperator::Ne,
        json!("tokmd"),
    )]);
    assert!(evaluate_policy(&receipt, &p).passed);
}

// ═══════════════════════════════════════════════════════════════════
// 3. Nested JSON structures
// ═══════════════════════════════════════════════════════════════════

#[test]
fn deeply_nested_pointer() {
    let receipt = json!({"a": {"b": {"c": {"d": 42}}}});
    let p = policy(vec![rule("deep", "/a/b/c/d", RuleOperator::Eq, json!(42))]);
    assert!(evaluate_policy(&receipt, &p).passed);
}

#[test]
fn nested_array_in_object() {
    let receipt = json!({"data": {"items": [1, 2, 3]}});
    let p = policy(vec![rule(
        "check",
        "/data/items/2",
        RuleOperator::Eq,
        json!(3),
    )]);
    assert!(evaluate_policy(&receipt, &p).passed);
}

// ═══════════════════════════════════════════════════════════════════
// 4. Gate pass/fail/warn outcomes
// ═══════════════════════════════════════════════════════════════════

#[test]
fn error_level_fails_gate() {
    let receipt = json!({"val": 999});
    let p = policy(vec![rule("check", "/val", RuleOperator::Lt, json!(100))]);
    let result = evaluate_policy(&receipt, &p);
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
    assert_eq!(result.warnings, 0);
}

#[test]
fn warn_level_does_not_fail_gate() {
    let receipt = json!({"val": 999});
    let mut r = rule("check", "/val", RuleOperator::Lt, json!(100));
    r.level = RuleLevel::Warn;
    let p = policy(vec![r]);
    let result = evaluate_policy(&receipt, &p);
    assert!(result.passed);
    assert_eq!(result.errors, 0);
    assert_eq!(result.warnings, 1);
}

#[test]
fn mixed_error_and_warn() {
    let receipt = json!({"a": 200, "b": 500});
    let mut warn_rule = rule("warn_a", "/a", RuleOperator::Lt, json!(100));
    warn_rule.level = RuleLevel::Warn;
    let error_rule = rule("err_b", "/b", RuleOperator::Lt, json!(100));
    let p = policy(vec![warn_rule, error_rule]);
    let result = evaluate_policy(&receipt, &p);
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
    assert_eq!(result.warnings, 1);
}

// ═══════════════════════════════════════════════════════════════════
// 5. Missing keys, null values, invalid pointers
// ═══════════════════════════════════════════════════════════════════

#[test]
fn missing_key_fails_by_default() {
    let receipt = json!({"other": 1});
    let p = policy(vec![rule("check", "/missing", RuleOperator::Eq, json!(1))]);
    let result = evaluate_policy(&receipt, &p);
    assert!(!result.passed);
}

#[test]
fn missing_key_passes_with_allow_missing() {
    let receipt = json!({"other": 1});
    let mut p = policy(vec![rule("check", "/missing", RuleOperator::Eq, json!(1))]);
    p.allow_missing = true;
    let result = evaluate_policy(&receipt, &p);
    assert!(result.passed);
}

#[test]
fn null_value_eq_null() {
    let receipt = json!({"val": null});
    let p = policy(vec![rule(
        "check",
        "/val",
        RuleOperator::Eq,
        serde_json::Value::Null,
    )]);
    let result = evaluate_policy(&receipt, &p);
    assert!(result.passed);
}

#[test]
fn exists_operator_pass() {
    let receipt = json!({"present": 42});
    let mut r = rule("check", "/present", RuleOperator::Exists, json!(null));
    r.value = None;
    let p = policy(vec![r]);
    let result = evaluate_policy(&receipt, &p);
    assert!(result.passed);
}

#[test]
fn exists_operator_fail() {
    let receipt = json!({"other": 42});
    let mut r = rule("check", "/missing", RuleOperator::Exists, json!(null));
    r.value = None;
    let p = policy(vec![r]);
    let result = evaluate_policy(&receipt, &p);
    assert!(!result.passed);
}

// ═══════════════════════════════════════════════════════════════════
// 6. Deterministic rule ordering
// ═══════════════════════════════════════════════════════════════════

#[test]
fn rule_results_preserve_input_order() {
    let receipt = json!({"a": 1, "b": 2, "c": 3});
    let p = policy(vec![
        rule("first", "/a", RuleOperator::Eq, json!(1)),
        rule("second", "/b", RuleOperator::Eq, json!(2)),
        rule("third", "/c", RuleOperator::Eq, json!(3)),
    ]);
    let result = evaluate_policy(&receipt, &p);
    assert_eq!(result.rule_results[0].name, "first");
    assert_eq!(result.rule_results[1].name, "second");
    assert_eq!(result.rule_results[2].name, "third");
}

#[test]
fn evaluate_deterministic_across_runs() {
    let receipt = json!({"x": 42, "y": "hello"});
    let p = policy(vec![
        rule("num", "/x", RuleOperator::Lte, json!(100)),
        rule("str", "/y", RuleOperator::Eq, json!("hello")),
    ]);
    let r1 = evaluate_policy(&receipt, &p);
    let r2 = evaluate_policy(&receipt, &p);
    assert_eq!(r1.passed, r2.passed);
    assert_eq!(r1.errors, r2.errors);
    assert_eq!(r1.rule_results.len(), r2.rule_results.len());
}

// ═══════════════════════════════════════════════════════════════════
// 7. Negate flag
// ═══════════════════════════════════════════════════════════════════

#[test]
fn negate_inverts_result() {
    let receipt = json!({"val": 50});
    let mut r = rule("check", "/val", RuleOperator::Gt, json!(100));
    r.negate = true;
    let p = policy(vec![r]);
    // val (50) > 100 is false, negate -> true
    let result = evaluate_policy(&receipt, &p);
    assert!(result.passed);
}

// ═══════════════════════════════════════════════════════════════════
// 8. Contains and In operators
// ═══════════════════════════════════════════════════════════════════

#[test]
fn contains_string() {
    let receipt = json!({"greeting": "hello world"});
    let p = policy(vec![rule(
        "check",
        "/greeting",
        RuleOperator::Contains,
        json!("hello"),
    )]);
    assert!(evaluate_policy(&receipt, &p).passed);
}

#[test]
fn in_operator_pass() {
    let receipt = json!({"lang": "rust"});
    let mut r = rule("check", "/lang", RuleOperator::In, json!(null));
    r.value = None;
    r.values = Some(vec![json!("rust"), json!("go"), json!("python")]);
    let p = policy(vec![r]);
    assert!(evaluate_policy(&receipt, &p).passed);
}

#[test]
fn in_operator_fail() {
    let receipt = json!({"lang": "java"});
    let mut r = rule("check", "/lang", RuleOperator::In, json!(null));
    r.value = None;
    r.values = Some(vec![json!("rust"), json!("go")]);
    let p = policy(vec![r]);
    assert!(!evaluate_policy(&receipt, &p).passed);
}

// ═══════════════════════════════════════════════════════════════════
// 9. Ratchet evaluation
// ═══════════════════════════════════════════════════════════════════

#[test]
fn ratchet_pass_within_tolerance() {
    let baseline = json!({"complexity": 10.0});
    let current = json!({"complexity": 10.5}); // 5%
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
    assert!(result.passed);
}

#[test]
fn ratchet_fail_exceed_pct() {
    let baseline = json!({"complexity": 10.0});
    let current = json!({"complexity": 15.0}); // 50%
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
fn ratchet_max_value_ceiling() {
    let baseline = json!({"complexity": 10.0});
    let current = json!({"complexity": 12.0}); // within 50% pct but over max_value
    let config = RatchetConfig {
        rules: vec![RatchetRule {
            pointer: "/complexity".into(),
            max_increase_pct: Some(50.0),
            max_value: Some(11.0),
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
fn ratchet_warn_level_does_not_fail() {
    let baseline = json!({"complexity": 10.0});
    let current = json!({"complexity": 20.0}); // 100% increase
    let config = RatchetConfig {
        rules: vec![RatchetRule {
            pointer: "/complexity".into(),
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
    assert!(result.passed);
    assert_eq!(result.warnings, 1);
}

// ═══════════════════════════════════════════════════════════════════
// 10. TOML parsing
// ═══════════════════════════════════════════════════════════════════

#[test]
fn policy_from_toml_multiple_rules() {
    let toml = r#"
fail_fast = true
allow_missing = false

[[rules]]
name = "max_tokens"
pointer = "/tokens"
op = "lte"
value = 100000

[[rules]]
name = "min_score"
pointer = "/score"
op = "gte"
value = 80
level = "warn"
"#;
    let p = PolicyConfig::from_toml(toml).unwrap();
    assert!(p.fail_fast);
    assert!(!p.allow_missing);
    assert_eq!(p.rules.len(), 2);
    assert_eq!(p.rules[1].level, RuleLevel::Warn);
}

#[test]
fn ratchet_from_toml() {
    let toml = r#"
allow_missing_baseline = true

[[rules]]
pointer = "/complexity"
max_increase_pct = 5.0
max_value = 20.0
level = "error"
description = "Keep complexity low"
"#;
    let c = RatchetConfig::from_toml(toml).unwrap();
    assert!(c.allow_missing_baseline);
    assert_eq!(c.rules.len(), 1);
    assert_eq!(c.rules[0].max_value, Some(20.0));
}

#[test]
fn policy_from_toml_invalid() {
    let result = PolicyConfig::from_toml("this is not valid toml {{{");
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════
// 11. Serde roundtrips
// ═══════════════════════════════════════════════════════════════════

#[test]
fn gate_result_serde_roundtrip() {
    let gr = GateResult::from_results(vec![RuleResult {
        name: "check".into(),
        passed: true,
        level: RuleLevel::Error,
        actual: Some(json!(42)),
        expected: "<= 100".into(),
        message: None,
    }]);
    let json = serde_json::to_string(&gr).unwrap();
    let parsed: GateResult = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.passed, gr.passed);
    assert_eq!(parsed.rule_results.len(), 1);
}

#[test]
fn ratchet_result_serde_roundtrip() {
    let rr = RatchetResult {
        rule: RatchetRule {
            pointer: "/complexity".into(),
            max_increase_pct: Some(10.0),
            max_value: None,
            level: RuleLevel::Error,
            description: Some("test".into()),
        },
        passed: true,
        baseline_value: Some(10.0),
        current_value: 10.5,
        change_pct: Some(5.0),
        message: "OK".into(),
    };
    let json = serde_json::to_string(&rr).unwrap();
    let parsed: RatchetResult = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.current_value, 10.5);
    assert!(parsed.passed);
}

#[test]
fn rule_operator_serde_roundtrip() {
    let ops = [
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
    for op in &ops {
        let json = serde_json::to_value(op).unwrap();
        let parsed: RuleOperator = serde_json::from_value(json).unwrap();
        assert_eq!(&parsed, op);
    }
}

#[test]
fn rule_level_serde_roundtrip() {
    for level in &[RuleLevel::Error, RuleLevel::Warn] {
        let json = serde_json::to_value(level).unwrap();
        let parsed: RuleLevel = serde_json::from_value(json).unwrap();
        assert_eq!(&parsed, level);
    }
}

// ═══════════════════════════════════════════════════════════════════
// 12. GateResult / RatchetGateResult construction
// ═══════════════════════════════════════════════════════════════════

#[test]
fn gate_result_empty_passes() {
    let gr = GateResult::from_results(vec![]);
    assert!(gr.passed);
    assert_eq!(gr.errors, 0);
    assert_eq!(gr.warnings, 0);
}

#[test]
fn ratchet_gate_result_empty_passes() {
    let rr = RatchetGateResult::from_results(vec![]);
    assert!(rr.passed);
    assert_eq!(rr.errors, 0);
    assert_eq!(rr.warnings, 0);
}

// ═══════════════════════════════════════════════════════════════════
// 13. Fail-fast behavior
// ═══════════════════════════════════════════════════════════════════

#[test]
fn fail_fast_stops_on_first_error() {
    let receipt = json!({"a": 999, "b": 999});
    let p = PolicyConfig {
        rules: vec![
            rule("first", "/a", RuleOperator::Lt, json!(10)),
            rule("second", "/b", RuleOperator::Lt, json!(10)),
        ],
        fail_fast: true,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &p);
    assert!(!result.passed);
    // With fail_fast, may stop after first error
    assert!(result.errors >= 1);
}

// ═══════════════════════════════════════════════════════════════════
// 14. Float comparison precision
// ═══════════════════════════════════════════════════════════════════

#[test]
fn float_equality() {
    let receipt = json!({"val": 0.1 + 0.2});
    let p = policy(vec![rule("check", "/val", RuleOperator::Gte, json!(0.3))]);
    assert!(evaluate_policy(&receipt, &p).passed);
}

#[test]
fn integer_and_float_comparison() {
    let receipt = json!({"val": 42});
    let p = policy(vec![rule("check", "/val", RuleOperator::Eq, json!(42.0))]);
    assert!(evaluate_policy(&receipt, &p).passed);
}
