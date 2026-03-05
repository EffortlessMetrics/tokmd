//! Depth tests for tokmd-gate (w60).
//!
//! BDD-style tests covering:
//! - Policy evaluation rules with all comparison operators
//! - JSON pointer navigation edge cases
//! - Ratchet logic (baseline comparison, degradation detection)
//! - Complex nested policies
//! - Negation, fail-fast, allow-missing semantics
//! - Property-based tests for gate evaluation determinism

use serde_json::{json, Value};
use tokmd_gate::{
    GateResult, PolicyConfig, PolicyRule, RatchetConfig, RatchetGateResult, RatchetRule,
    RuleLevel, RuleOperator, RuleResult, evaluate_policy, evaluate_ratchet_policy,
    resolve_pointer,
};

// ═══════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════

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

fn warn_rule(name: &str, pointer: &str, op: RuleOperator, value: Value) -> PolicyRule {
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

fn ratchet_rule(pointer: &str, max_increase_pct: Option<f64>, max_value: Option<f64>) -> RatchetRule {
    RatchetRule {
        pointer: pointer.into(),
        max_increase_pct,
        max_value,
        level: RuleLevel::Error,
        description: None,
    }
}

fn ratchet_config(rules: Vec<RatchetRule>) -> RatchetConfig {
    RatchetConfig {
        rules,
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    }
}

// ═══════════════════════════════════════════════════════════════════
// 1. JSON Pointer resolution edge cases
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_empty_pointer_when_resolved_then_whole_doc_returned() {
    let doc = json!({"a": 1, "b": 2});
    assert_eq!(resolve_pointer(&doc, ""), Some(&doc));
}

#[test]
fn given_pointer_to_null_value_when_resolved_then_null_returned() {
    let doc = json!({"key": null});
    assert_eq!(resolve_pointer(&doc, "/key"), Some(&Value::Null));
}

#[test]
fn given_pointer_to_boolean_when_resolved_then_bool_returned() {
    let doc = json!({"enabled": true, "disabled": false});
    assert_eq!(resolve_pointer(&doc, "/enabled"), Some(&json!(true)));
    assert_eq!(resolve_pointer(&doc, "/disabled"), Some(&json!(false)));
}

#[test]
fn given_deeply_nested_path_when_resolved_then_leaf_found() {
    let doc = json!({"a": {"b": {"c": {"d": {"e": 42}}}}});
    assert_eq!(resolve_pointer(&doc, "/a/b/c/d/e"), Some(&json!(42)));
}

#[test]
fn given_array_of_objects_when_index_resolved_then_object_returned() {
    let doc = json!({"items": [{"id": 1}, {"id": 2}]});
    assert_eq!(resolve_pointer(&doc, "/items/0/id"), Some(&json!(1)));
    assert_eq!(resolve_pointer(&doc, "/items/1/id"), Some(&json!(2)));
}

#[test]
fn given_nested_array_when_multi_index_resolved_then_value_found() {
    let doc = json!({"matrix": [[10, 20], [30, 40]]});
    assert_eq!(resolve_pointer(&doc, "/matrix/1/0"), Some(&json!(30)));
}

#[test]
fn given_missing_intermediate_key_when_resolved_then_none() {
    let doc = json!({"a": {"b": 1}});
    assert_eq!(resolve_pointer(&doc, "/a/missing/deep"), None);
}

#[test]
fn given_pointer_without_leading_slash_when_resolved_then_none() {
    let doc = json!({"key": 1});
    assert_eq!(resolve_pointer(&doc, "key"), None);
}

#[test]
fn given_array_out_of_bounds_when_resolved_then_none() {
    let doc = json!({"arr": [1, 2, 3]});
    assert_eq!(resolve_pointer(&doc, "/arr/5"), None);
}

#[test]
fn given_escaped_tilde_in_key_when_resolved_then_correct_value() {
    let doc = json!({"a~b": 99});
    assert_eq!(resolve_pointer(&doc, "/a~0b"), Some(&json!(99)));
}

#[test]
fn given_escaped_slash_in_key_when_resolved_then_correct_value() {
    let doc = json!({"a/b": 77});
    assert_eq!(resolve_pointer(&doc, "/a~1b"), Some(&json!(77)));
}

#[test]
fn given_empty_string_key_when_resolved_via_slash_then_found() {
    let doc = json!({"": 42});
    assert_eq!(resolve_pointer(&doc, "/"), Some(&json!(42)));
}

// ═══════════════════════════════════════════════════════════════════
// 2. Comparison operators (gt, lt, eq, gte, lte, ne)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_gt_when_actual_above_threshold_then_pass() {
    let receipt = json!({"score": 90});
    let result = eval(&receipt, vec![rule("high_score", "/score", RuleOperator::Gt, json!(50))]);
    assert!(result.passed);
    assert_eq!(result.errors, 0);
}

#[test]
fn given_gt_when_actual_equals_threshold_then_fail() {
    let receipt = json!({"score": 50});
    let result = eval(&receipt, vec![rule("score_check", "/score", RuleOperator::Gt, json!(50))]);
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
}

#[test]
fn given_lt_when_actual_below_threshold_then_pass() {
    let receipt = json!({"errors": 3});
    let result = eval(&receipt, vec![rule("low_errors", "/errors", RuleOperator::Lt, json!(10))]);
    assert!(result.passed);
}

#[test]
fn given_lt_when_actual_equals_threshold_then_fail() {
    let receipt = json!({"errors": 10});
    let result = eval(&receipt, vec![rule("err", "/errors", RuleOperator::Lt, json!(10))]);
    assert!(!result.passed);
}

#[test]
fn given_gte_when_actual_equals_threshold_then_pass() {
    let receipt = json!({"coverage": 80.0});
    let result = eval(&receipt, vec![rule("cov", "/coverage", RuleOperator::Gte, json!(80.0))]);
    assert!(result.passed);
}

#[test]
fn given_lte_when_actual_equals_threshold_then_pass() {
    let receipt = json!({"tokens": 1000});
    let result = eval(&receipt, vec![rule("tok", "/tokens", RuleOperator::Lte, json!(1000))]);
    assert!(result.passed);
}

#[test]
fn given_lte_when_actual_above_threshold_then_fail() {
    let receipt = json!({"tokens": 1001});
    let result = eval(&receipt, vec![rule("tok", "/tokens", RuleOperator::Lte, json!(1000))]);
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
}

#[test]
fn given_eq_when_values_match_then_pass() {
    let receipt = json!({"lang": "Rust"});
    let result = eval(&receipt, vec![rule("lang", "/lang", RuleOperator::Eq, json!("Rust"))]);
    assert!(result.passed);
}

#[test]
fn given_eq_when_values_differ_then_fail() {
    let receipt = json!({"lang": "Python"});
    let result = eval(&receipt, vec![rule("lang", "/lang", RuleOperator::Eq, json!("Rust"))]);
    assert!(!result.passed);
}

#[test]
fn given_ne_when_values_differ_then_pass() {
    let receipt = json!({"status": "ok"});
    let result = eval(&receipt, vec![rule("no_err", "/status", RuleOperator::Ne, json!("error"))]);
    assert!(result.passed);
}

#[test]
fn given_ne_when_values_match_then_fail() {
    let receipt = json!({"status": "error"});
    let result = eval(&receipt, vec![rule("no_err", "/status", RuleOperator::Ne, json!("error"))]);
    assert!(!result.passed);
}

// ═══════════════════════════════════════════════════════════════════
// 3. In / Contains / Exists operators
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_in_op_when_value_in_list_then_pass() {
    let receipt = json!({"format": "json"});
    let r = PolicyRule {
        name: "fmt_check".into(),
        pointer: "/format".into(),
        op: RuleOperator::In,
        value: None,
        values: Some(vec![json!("json"), json!("csv"), json!("jsonl")]),
        negate: false,
        level: RuleLevel::Error,
        message: None,
    };
    let result = evaluate_policy(&receipt, &policy(vec![r]));
    assert!(result.passed);
}

#[test]
fn given_in_op_when_value_not_in_list_then_fail() {
    let receipt = json!({"format": "xml"});
    let r = PolicyRule {
        name: "fmt_check".into(),
        pointer: "/format".into(),
        op: RuleOperator::In,
        value: None,
        values: Some(vec![json!("json"), json!("csv")]),
        negate: false,
        level: RuleLevel::Error,
        message: None,
    };
    let result = evaluate_policy(&receipt, &policy(vec![r]));
    assert!(!result.passed);
}

#[test]
fn given_contains_op_when_string_has_substring_then_pass() {
    let receipt = json!({"path": "src/main.rs"});
    let result = eval(&receipt, vec![rule("has_src", "/path", RuleOperator::Contains, json!("src"))]);
    assert!(result.passed);
}

#[test]
fn given_contains_op_when_array_has_element_then_pass() {
    let receipt = json!({"langs": ["Rust", "Python", "Go"]});
    let result = eval(&receipt, vec![rule("has_rust", "/langs", RuleOperator::Contains, json!("Rust"))]);
    assert!(result.passed);
}

#[test]
fn given_exists_op_when_key_present_then_pass() {
    let receipt = json!({"license": "MIT"});
    let r = PolicyRule {
        name: "has_license".into(),
        pointer: "/license".into(),
        op: RuleOperator::Exists,
        value: None,
        values: None,
        negate: false,
        level: RuleLevel::Error,
        message: None,
    };
    let result = evaluate_policy(&receipt, &policy(vec![r]));
    assert!(result.passed);
}

#[test]
fn given_exists_op_when_key_absent_then_fail() {
    let receipt = json!({"name": "test"});
    let r = PolicyRule {
        name: "has_license".into(),
        pointer: "/license".into(),
        op: RuleOperator::Exists,
        value: None,
        values: None,
        negate: false,
        level: RuleLevel::Error,
        message: None,
    };
    let result = evaluate_policy(&receipt, &policy(vec![r]));
    assert!(!result.passed);
}

// ═══════════════════════════════════════════════════════════════════
// 4. Negation semantics
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_negated_lte_when_below_threshold_then_fail() {
    let receipt = json!({"tokens": 500});
    let r = PolicyRule {
        name: "not_low".into(),
        pointer: "/tokens".into(),
        op: RuleOperator::Lte,
        value: Some(json!(1000)),
        values: None,
        negate: true,
        level: RuleLevel::Error,
        message: None,
    };
    let result = evaluate_policy(&receipt, &policy(vec![r]));
    assert!(!result.passed);
}

#[test]
fn given_negated_exists_when_key_absent_then_pass() {
    let receipt = json!({"name": "test"});
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
    let result = evaluate_policy(&receipt, &policy(vec![r]));
    assert!(result.passed);
}

// ═══════════════════════════════════════════════════════════════════
// 5. Fail-fast and allow-missing
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_fail_fast_when_first_rule_fails_then_remaining_skipped() {
    let receipt = json!({"a": 100, "b": 200});
    let pol = PolicyConfig {
        rules: vec![
            rule("fail_a", "/a", RuleOperator::Lt, json!(50)),
            rule("check_b", "/b", RuleOperator::Lt, json!(300)),
        ],
        fail_fast: true,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &pol);
    assert!(!result.passed);
    // With fail_fast, only first rule should be evaluated
    assert_eq!(result.rule_results.len(), 1);
}

#[test]
fn given_no_fail_fast_when_first_fails_then_all_evaluated() {
    let receipt = json!({"a": 100, "b": 200});
    let pol = PolicyConfig {
        rules: vec![
            rule("fail_a", "/a", RuleOperator::Lt, json!(50)),
            rule("check_b", "/b", RuleOperator::Lt, json!(300)),
        ],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &pol);
    assert!(!result.passed);
    assert_eq!(result.rule_results.len(), 2);
}

#[test]
fn given_allow_missing_when_pointer_absent_then_pass() {
    let receipt = json!({"a": 1});
    let pol = PolicyConfig {
        rules: vec![rule("miss", "/nonexistent", RuleOperator::Lte, json!(100))],
        fail_fast: false,
        allow_missing: true,
    };
    let result = evaluate_policy(&receipt, &pol);
    assert!(result.passed);
}

#[test]
fn given_no_allow_missing_when_pointer_absent_then_fail() {
    let receipt = json!({"a": 1});
    let pol = PolicyConfig {
        rules: vec![rule("miss", "/nonexistent", RuleOperator::Lte, json!(100))],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &pol);
    assert!(!result.passed);
}

// ═══════════════════════════════════════════════════════════════════
// 6. Warning rules don't fail the gate
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_warn_rule_when_fails_then_gate_still_passes() {
    let receipt = json!({"tokens": 2000});
    let result = evaluate_policy(
        &receipt,
        &policy(vec![warn_rule("warn_tok", "/tokens", RuleOperator::Lte, json!(1000))]),
    );
    assert!(result.passed);
    assert_eq!(result.warnings, 1);
    assert_eq!(result.errors, 0);
}

#[test]
fn given_mixed_warn_and_error_when_only_warn_fails_then_pass() {
    let receipt = json!({"tokens": 2000, "files": 5});
    let rules = vec![
        warn_rule("w_tok", "/tokens", RuleOperator::Lte, json!(1000)),
        rule("e_files", "/files", RuleOperator::Lte, json!(100)),
    ];
    let result = evaluate_policy(&receipt, &policy(rules));
    assert!(result.passed);
    assert_eq!(result.warnings, 1);
    assert_eq!(result.errors, 0);
}

#[test]
fn given_mixed_warn_and_error_when_error_fails_then_gate_fails() {
    let receipt = json!({"tokens": 2000, "files": 200});
    let rules = vec![
        warn_rule("w_tok", "/tokens", RuleOperator::Lte, json!(1000)),
        rule("e_files", "/files", RuleOperator::Lte, json!(100)),
    ];
    let result = evaluate_policy(&receipt, &policy(rules));
    assert!(!result.passed);
    assert_eq!(result.warnings, 1);
    assert_eq!(result.errors, 1);
}

// ═══════════════════════════════════════════════════════════════════
// 7. Complex nested policy evaluation
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_nested_receipt_when_multiple_deep_rules_then_correct_outcome() {
    let receipt = json!({
        "derived": {
            "totals": { "code": 5000, "tokens": 15000 },
            "density": { "comment_ratio": 0.25 }
        },
        "meta": { "schema_version": 2 }
    });
    let rules = vec![
        rule("max_code", "/derived/totals/code", RuleOperator::Lte, json!(10000)),
        rule("max_tokens", "/derived/totals/tokens", RuleOperator::Lte, json!(500000)),
        rule("min_comment", "/derived/density/comment_ratio", RuleOperator::Gte, json!(0.1)),
        rule("schema_v", "/meta/schema_version", RuleOperator::Eq, json!(2)),
    ];
    let result = evaluate_policy(&receipt, &policy(rules));
    assert!(result.passed);
    assert_eq!(result.rule_results.len(), 4);
}

#[test]
fn given_complex_receipt_when_one_nested_rule_fails_then_gate_fails() {
    let receipt = json!({
        "analysis": {
            "complexity": { "avg_cyclomatic": 15.0, "max_cyclomatic": 50 },
            "coverage": { "line_pct": 0.45 }
        }
    });
    let rules = vec![
        rule("avg_cx", "/analysis/complexity/avg_cyclomatic", RuleOperator::Lte, json!(20.0)),
        rule("max_cx", "/analysis/complexity/max_cyclomatic", RuleOperator::Lte, json!(30)),
        rule("cov", "/analysis/coverage/line_pct", RuleOperator::Gte, json!(0.50)),
    ];
    let result = evaluate_policy(&receipt, &policy(rules));
    assert!(!result.passed);
    // max_cx (50 > 30) and cov (0.45 < 0.50) both fail
    assert_eq!(result.errors, 2);
}

// ═══════════════════════════════════════════════════════════════════
// 8. Ratchet logic: baseline comparison
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_ratchet_within_threshold_when_evaluated_then_pass() {
    let baseline = json!({"complexity": 10.0});
    let current = json!({"complexity": 10.5}); // 5% increase
    let config = ratchet_config(vec![ratchet_rule("/complexity", Some(10.0), None)]);
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(result.passed);
    assert_eq!(result.errors, 0);
}

#[test]
fn given_ratchet_exceeds_threshold_when_evaluated_then_fail() {
    let baseline = json!({"complexity": 10.0});
    let current = json!({"complexity": 12.0}); // 20% increase
    let config = ratchet_config(vec![ratchet_rule("/complexity", Some(10.0), None)]);
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
}

#[test]
fn given_ratchet_max_value_exceeded_when_evaluated_then_fail() {
    let baseline = json!({"complexity": 10.0});
    let current = json!({"complexity": 25.0});
    let config = ratchet_config(vec![ratchet_rule("/complexity", None, Some(20.0))]);
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(!result.passed);
}

#[test]
fn given_ratchet_max_value_not_exceeded_when_evaluated_then_pass() {
    let baseline = json!({"complexity": 10.0});
    let current = json!({"complexity": 18.0});
    let config = ratchet_config(vec![ratchet_rule("/complexity", None, Some(20.0))]);
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(result.passed);
}

#[test]
fn given_ratchet_both_constraints_when_pct_fails_then_fail() {
    let baseline = json!({"complexity": 10.0});
    let current = json!({"complexity": 15.0}); // 50% increase, but under max_value
    let config = ratchet_config(vec![ratchet_rule("/complexity", Some(10.0), Some(20.0))]);
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(!result.passed);
}

#[test]
fn given_ratchet_improvement_when_metric_decreases_then_pass() {
    let baseline = json!({"complexity": 20.0});
    let current = json!({"complexity": 15.0}); // decreased
    let config = ratchet_config(vec![ratchet_rule("/complexity", Some(10.0), None)]);
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(result.passed);
}

#[test]
fn given_ratchet_missing_baseline_when_strict_then_fail() {
    let baseline = json!({});
    let current = json!({"complexity": 10.0});
    let config = ratchet_config(vec![ratchet_rule("/complexity", Some(10.0), None)]);
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(!result.passed);
}

#[test]
fn given_ratchet_missing_baseline_when_allowed_then_pass() {
    let baseline = json!({});
    let current = json!({"complexity": 10.0});
    let config = RatchetConfig {
        rules: vec![ratchet_rule("/complexity", Some(10.0), None)],
        fail_fast: false,
        allow_missing_baseline: true,
        allow_missing_current: false,
    };
    // allow_missing_baseline only affects evaluate_ratchet_with_options, not the main policy fn
    // The main evaluate_ratchet_policy uses the config's flags
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(result.passed);
}

#[test]
fn given_ratchet_missing_current_when_strict_then_fail() {
    let baseline = json!({"complexity": 10.0});
    let current = json!({});
    let config = ratchet_config(vec![ratchet_rule("/complexity", Some(10.0), None)]);
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(!result.passed);
}

#[test]
fn given_ratchet_zero_baseline_with_zero_current_then_pass() {
    let baseline = json!({"complexity": 0.0});
    let current = json!({"complexity": 0.0});
    let config = ratchet_config(vec![ratchet_rule("/complexity", Some(10.0), None)]);
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(result.passed);
}

// ═══════════════════════════════════════════════════════════════════
// 9. GateResult / RatchetGateResult construction
// ═══════════════════════════════════════════════════════════════════

#[test]
fn gate_result_from_all_passing_rules() {
    let results = vec![
        RuleResult { name: "r1".into(), passed: true, level: RuleLevel::Error, actual: None, expected: "x".into(), message: None },
        RuleResult { name: "r2".into(), passed: true, level: RuleLevel::Warn, actual: None, expected: "y".into(), message: None },
    ];
    let gate = GateResult::from_results(results);
    assert!(gate.passed);
    assert_eq!(gate.errors, 0);
    assert_eq!(gate.warnings, 0);
}

#[test]
fn gate_result_counts_errors_and_warnings_separately() {
    let results = vec![
        RuleResult { name: "e1".into(), passed: false, level: RuleLevel::Error, actual: None, expected: "x".into(), message: None },
        RuleResult { name: "w1".into(), passed: false, level: RuleLevel::Warn, actual: None, expected: "y".into(), message: None },
        RuleResult { name: "w2".into(), passed: false, level: RuleLevel::Warn, actual: None, expected: "z".into(), message: None },
    ];
    let gate = GateResult::from_results(results);
    assert!(!gate.passed);
    assert_eq!(gate.errors, 1);
    assert_eq!(gate.warnings, 2);
}

#[test]
fn ratchet_gate_result_from_empty_is_pass() {
    let result = RatchetGateResult::from_results(vec![]);
    assert!(result.passed);
    assert_eq!(result.errors, 0);
    assert_eq!(result.warnings, 0);
}

// ═══════════════════════════════════════════════════════════════════
// 10. TOML parsing
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_valid_policy_toml_when_parsed_then_all_fields_set() {
    let toml = r#"
fail_fast = true
allow_missing = true

[[rules]]
name = "max_tokens"
pointer = "/tokens"
op = "lte"
value = 100000
level = "error"
message = "Too many tokens"

[[rules]]
name = "has_license"
pointer = "/license"
op = "exists"
level = "warn"
"#;
    let config = PolicyConfig::from_toml(toml).unwrap();
    assert!(config.fail_fast);
    assert!(config.allow_missing);
    assert_eq!(config.rules.len(), 2);
    assert_eq!(config.rules[0].name, "max_tokens");
    assert_eq!(config.rules[0].op, RuleOperator::Lte);
    assert_eq!(config.rules[1].op, RuleOperator::Exists);
    assert_eq!(config.rules[1].level, RuleLevel::Warn);
}

#[test]
fn given_empty_toml_when_parsed_then_defaults_applied() {
    let config = PolicyConfig::from_toml("").unwrap();
    assert!(config.rules.is_empty());
    assert!(!config.fail_fast);
    assert!(!config.allow_missing);
}

#[test]
fn given_invalid_toml_when_parsed_then_error() {
    let result = PolicyConfig::from_toml("not valid [[[ toml");
    assert!(result.is_err());
}

#[test]
fn given_ratchet_toml_when_parsed_then_all_fields_set() {
    let toml = r#"
fail_fast = true
allow_missing_baseline = true
allow_missing_current = false

[[rules]]
pointer = "/complexity/avg"
max_increase_pct = 5.0
max_value = 25.0
level = "error"
description = "Complexity must not regress"
"#;
    let config = RatchetConfig::from_toml(toml).unwrap();
    assert!(config.fail_fast);
    assert!(config.allow_missing_baseline);
    assert!(!config.allow_missing_current);
    assert_eq!(config.rules.len(), 1);
    assert_eq!(config.rules[0].pointer, "/complexity/avg");
    assert_eq!(config.rules[0].max_increase_pct, Some(5.0));
    assert_eq!(config.rules[0].max_value, Some(25.0));
}

// ═══════════════════════════════════════════════════════════════════
// 11. Serde roundtrip tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn rule_operator_serde_roundtrip_all_variants() {
    let variants = [
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
    for variant in variants {
        let json = serde_json::to_string(&variant).unwrap();
        let back: RuleOperator = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn rule_level_serde_roundtrip() {
    for variant in [RuleLevel::Warn, RuleLevel::Error] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: RuleLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn gate_result_json_roundtrip() {
    let gate = GateResult::from_results(vec![
        RuleResult { name: "r1".into(), passed: true, level: RuleLevel::Error, actual: Some(json!(42)), expected: "lte 100".into(), message: None },
    ]);
    let json = serde_json::to_string(&gate).unwrap();
    let back: GateResult = serde_json::from_str(&json).unwrap();
    assert_eq!(back.passed, gate.passed);
    assert_eq!(back.errors, gate.errors);
    assert_eq!(back.rule_results.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════
// 12. Custom message propagation
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_custom_message_when_rule_fails_then_message_in_result() {
    let receipt = json!({"tokens": 9999});
    let r = PolicyRule {
        name: "tok".into(),
        pointer: "/tokens".into(),
        op: RuleOperator::Lte,
        value: Some(json!(100)),
        values: None,
        negate: false,
        level: RuleLevel::Error,
        message: Some("Way too many tokens!".into()),
    };
    let result = evaluate_policy(&receipt, &policy(vec![r]));
    assert!(!result.passed);
    assert_eq!(
        result.rule_results[0].message.as_deref(),
        Some("Way too many tokens!")
    );
}

#[test]
fn given_custom_message_when_rule_passes_then_message_is_none() {
    let receipt = json!({"tokens": 5});
    let r = PolicyRule {
        name: "tok".into(),
        pointer: "/tokens".into(),
        op: RuleOperator::Lte,
        value: Some(json!(100)),
        values: None,
        negate: false,
        level: RuleLevel::Error,
        message: Some("Way too many tokens!".into()),
    };
    let result = evaluate_policy(&receipt, &policy(vec![r]));
    assert!(result.passed);
    assert!(result.rule_results[0].message.is_none());
}

// ═══════════════════════════════════════════════════════════════════
// 13. Property tests for determinism
// ═══════════════════════════════════════════════════════════════════

mod properties {
    use super::*;
    use proptest::prelude::*;

    fn arb_op() -> impl Strategy<Value = RuleOperator> {
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
        #[test]
        fn evaluate_same_input_gives_same_output(val in 0i64..10000) {
            let receipt = json!({"v": val});
            let r = rule("check", "/v", RuleOperator::Lte, json!(5000));
            let a = evaluate_policy(&receipt, &policy(vec![r.clone()]));
            let b = evaluate_policy(&receipt, &policy(vec![r]));
            prop_assert_eq!(a.passed, b.passed);
            prop_assert_eq!(a.errors, b.errors);
            prop_assert_eq!(a.warnings, b.warnings);
        }

        #[test]
        fn ratchet_deterministic(base in 1.0f64..100.0, curr in 1.0f64..100.0) {
            let baseline = json!({"m": base});
            let current = json!({"m": curr});
            let config = ratchet_config(vec![ratchet_rule("/m", Some(20.0), None)]);
            let a = evaluate_ratchet_policy(&config, &baseline, &current);
            let b = evaluate_ratchet_policy(&config, &baseline, &current);
            prop_assert_eq!(a.passed, b.passed);
            prop_assert_eq!(a.errors, b.errors);
        }

        #[test]
        fn empty_policy_always_passes(val in 0i64..10000) {
            let receipt = json!({"v": val});
            let result = evaluate_policy(&receipt, &PolicyConfig::default());
            prop_assert!(result.passed);
            prop_assert_eq!(result.errors, 0);
        }

        #[test]
        fn pointer_resolution_deterministic(idx in 0usize..5) {
            let doc = json!({"arr": [0, 1, 2, 3, 4]});
            let ptr = format!("/arr/{idx}");
            let a = resolve_pointer(&doc, &ptr);
            let b = resolve_pointer(&doc, &ptr);
            prop_assert_eq!(a, b);
        }

        #[test]
        fn operator_display_roundtrip(op in arb_op()) {
            let display = op.to_string();
            prop_assert!(!display.is_empty());
        }
    }
}
