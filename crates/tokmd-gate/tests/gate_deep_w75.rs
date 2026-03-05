//! Deep tests for `tokmd-gate` – policy evaluation engine (w75).
//!
//! Covers all operators, JSON pointer resolution, ratchet mode,
//! policy file loading, multi-rule AND logic, output format, and error handling.

use serde_json::{json, Value};
use std::io::Write;
use tokmd_gate::{
    evaluate_policy, evaluate_ratchet_policy, resolve_pointer, GateResult, PolicyConfig,
    PolicyRule, RatchetConfig, RatchetRule, RuleLevel, RuleOperator,
};

// ── Helpers ─────────────────────────────────────────────────────────────────

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

fn ratchet_rule(pointer: &str, max_inc: Option<f64>, max_val: Option<f64>) -> RatchetRule {
    RatchetRule {
        pointer: pointer.into(),
        max_increase_pct: max_inc,
        max_value: max_val,
        level: RuleLevel::Error,
        description: None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. Operator tests – every operator exercised
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn op_gt_pass_and_fail() {
    let receipt = json!({"v": 10});
    let pass = evaluate_policy(&receipt, &policy(vec![rule("gt", "/v", RuleOperator::Gt, json!(5))]));
    assert!(pass.passed);
    let fail = evaluate_policy(&receipt, &policy(vec![rule("gt", "/v", RuleOperator::Gt, json!(10))]));
    assert!(!fail.passed);
}

#[test]
fn op_gte_pass_at_boundary() {
    let receipt = json!({"v": 10});
    let pass = evaluate_policy(&receipt, &policy(vec![rule("gte", "/v", RuleOperator::Gte, json!(10))]));
    assert!(pass.passed);
    let fail = evaluate_policy(&receipt, &policy(vec![rule("gte", "/v", RuleOperator::Gte, json!(11))]));
    assert!(!fail.passed);
}

#[test]
fn op_lt_pass_and_fail() {
    let receipt = json!({"v": 5});
    let pass = evaluate_policy(&receipt, &policy(vec![rule("lt", "/v", RuleOperator::Lt, json!(10))]));
    assert!(pass.passed);
    let fail = evaluate_policy(&receipt, &policy(vec![rule("lt", "/v", RuleOperator::Lt, json!(5))]));
    assert!(!fail.passed);
}

#[test]
fn op_lte_pass_at_boundary() {
    let receipt = json!({"v": 10});
    let pass = evaluate_policy(&receipt, &policy(vec![rule("lte", "/v", RuleOperator::Lte, json!(10))]));
    assert!(pass.passed);
    let fail = evaluate_policy(&receipt, &policy(vec![rule("lte", "/v", RuleOperator::Lte, json!(9))]));
    assert!(!fail.passed);
}

#[test]
fn op_eq_string_and_number() {
    let receipt = json!({"name": "rust", "count": 42});
    let str_pass = evaluate_policy(&receipt, &policy(vec![rule("eq_s", "/name", RuleOperator::Eq, json!("rust"))]));
    assert!(str_pass.passed);
    let num_pass = evaluate_policy(&receipt, &policy(vec![rule("eq_n", "/count", RuleOperator::Eq, json!(42))]));
    assert!(num_pass.passed);
    let str_fail = evaluate_policy(&receipt, &policy(vec![rule("eq_f", "/name", RuleOperator::Eq, json!("go"))]));
    assert!(!str_fail.passed);
}

#[test]
fn op_neq_pass_and_fail() {
    let receipt = json!({"lang": "rust"});
    let pass = evaluate_policy(&receipt, &policy(vec![rule("ne", "/lang", RuleOperator::Ne, json!("go"))]));
    assert!(pass.passed);
    let fail = evaluate_policy(&receipt, &policy(vec![rule("ne", "/lang", RuleOperator::Ne, json!("rust"))]));
    assert!(!fail.passed);
}

#[test]
fn op_contains_string_substring() {
    let receipt = json!({"msg": "hello world"});
    let pass = evaluate_policy(&receipt, &policy(vec![rule("c", "/msg", RuleOperator::Contains, json!("world"))]));
    assert!(pass.passed);
    let fail = evaluate_policy(&receipt, &policy(vec![rule("c", "/msg", RuleOperator::Contains, json!("mars"))]));
    assert!(!fail.passed);
}

#[test]
fn op_contains_array_membership() {
    let receipt = json!({"tags": ["alpha", "beta", "gamma"]});
    let pass = evaluate_policy(&receipt, &policy(vec![rule("c", "/tags", RuleOperator::Contains, json!("beta"))]));
    assert!(pass.passed);
    let fail = evaluate_policy(&receipt, &policy(vec![rule("c", "/tags", RuleOperator::Contains, json!("delta"))]));
    assert!(!fail.passed);
}

#[test]
fn op_in_membership() {
    let receipt = json!({"lang": "rust"});
    let mut r = rule("in", "/lang", RuleOperator::In, json!(null));
    r.value = None;
    r.values = Some(vec![json!("rust"), json!("go"), json!("python")]);
    let pass = evaluate_policy(&receipt, &policy(vec![r]));
    assert!(pass.passed);

    let mut r2 = rule("in", "/lang", RuleOperator::In, json!(null));
    r2.value = None;
    r2.values = Some(vec![json!("java"), json!("c")]);
    let fail = evaluate_policy(&receipt, &policy(vec![r2]));
    assert!(!fail.passed);
}

#[test]
fn op_exists_present_and_absent() {
    let receipt = json!({"meta": {"key": 1}});
    let mut exists_rule = rule("ex", "/meta/key", RuleOperator::Exists, json!(null));
    exists_rule.value = None;
    let pass = evaluate_policy(&receipt, &policy(vec![exists_rule]));
    assert!(pass.passed);

    let mut missing_rule = rule("ex", "/meta/nope", RuleOperator::Exists, json!(null));
    missing_rule.value = None;
    let fail = evaluate_policy(&receipt, &policy(vec![missing_rule]));
    assert!(!fail.passed);
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. JSON pointer resolution on nested objects
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn pointer_deep_nesting() {
    let doc = json!({"a": {"b": {"c": {"d": 99}}}});
    assert_eq!(resolve_pointer(&doc, "/a/b/c/d"), Some(&json!(99)));
}

#[test]
fn pointer_array_index() {
    let doc = json!({"items": [{"id": 1}, {"id": 2}]});
    assert_eq!(resolve_pointer(&doc, "/items/0/id"), Some(&json!(1)));
    assert_eq!(resolve_pointer(&doc, "/items/1/id"), Some(&json!(2)));
    assert_eq!(resolve_pointer(&doc, "/items/5"), None);
}

#[test]
fn pointer_rfc6901_escape_sequences() {
    let doc = json!({"a/b": {"c~d": 42}});
    assert_eq!(resolve_pointer(&doc, "/a~1b/c~0d"), Some(&json!(42)));
}

#[test]
fn pointer_empty_returns_whole_document() {
    let doc = json!({"x": 1});
    assert_eq!(resolve_pointer(&doc, ""), Some(&doc));
}

#[test]
fn pointer_no_leading_slash_returns_none() {
    let doc = json!({"x": 1});
    assert_eq!(resolve_pointer(&doc, "x"), None);
}

#[test]
fn pointer_missing_intermediate_key() {
    let doc = json!({"a": 1});
    assert_eq!(resolve_pointer(&doc, "/a/b/c"), None);
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Ratchet mode – no regression from baseline
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ratchet_within_tolerance_passes() {
    let baseline = json!({"complexity": 100.0});
    let current = json!({"complexity": 105.0}); // +5%
    let config = RatchetConfig {
        rules: vec![ratchet_rule("/complexity", Some(10.0), None)],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(result.passed);
    assert_eq!(result.errors, 0);
}

#[test]
fn ratchet_exceeds_tolerance_fails() {
    let baseline = json!({"complexity": 100.0});
    let current = json!({"complexity": 125.0}); // +25%
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
fn ratchet_absolute_ceiling_exceeded() {
    let baseline = json!({"tokens": 500});
    let current = json!({"tokens": 2500});
    let config = RatchetConfig {
        rules: vec![ratchet_rule("/tokens", None, Some(2000.0))],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(!result.passed);
}

#[test]
fn ratchet_missing_baseline_strict_fails() {
    let baseline = json!({});
    let current = json!({"x": 10.0});
    let config = RatchetConfig {
        rules: vec![ratchet_rule("/x", Some(5.0), None)],
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
    let current = json!({"x": 10.0});
    let config = RatchetConfig {
        rules: vec![ratchet_rule("/x", Some(5.0), None)],
        fail_fast: false,
        allow_missing_baseline: true,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(result.passed);
}

#[test]
fn ratchet_zero_baseline_same_value_passes() {
    let baseline = json!({"c": 0});
    let current = json!({"c": 0});
    let config = RatchetConfig {
        rules: vec![ratchet_rule("/c", Some(10.0), None)],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(result.passed);
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Policy file loading and validation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn policy_from_toml_string() {
    let toml = r#"
fail_fast = true
allow_missing = true

[[rules]]
name = "budget"
pointer = "/tokens"
op = "lte"
value = 100000
level = "error"

[[rules]]
name = "has_tests"
pointer = "/test_ratio"
op = "gte"
value = 0.1
level = "warn"
"#;
    let p = PolicyConfig::from_toml(toml).unwrap();
    assert!(p.fail_fast);
    assert!(p.allow_missing);
    assert_eq!(p.rules.len(), 2);
    assert_eq!(p.rules[0].op, RuleOperator::Lte);
    assert_eq!(p.rules[1].level, RuleLevel::Warn);
}

#[test]
fn policy_from_file_roundtrip() {
    let toml = r#"
[[rules]]
name = "file_check"
pointer = "/files"
op = "gt"
value = 0
"#;
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    tmp.write_all(toml.as_bytes()).unwrap();
    tmp.flush().unwrap();

    let p = PolicyConfig::from_file(tmp.path()).unwrap();
    assert_eq!(p.rules.len(), 1);
    assert_eq!(p.rules[0].name, "file_check");
    assert_eq!(p.rules[0].op, RuleOperator::Gt);
}

#[test]
fn policy_from_file_nonexistent_errors() {
    let result = PolicyConfig::from_file(std::path::Path::new("/nonexistent/policy.toml"));
    assert!(result.is_err());
}

#[test]
fn policy_from_toml_invalid_syntax_errors() {
    let result = PolicyConfig::from_toml("this is not valid toml {{{");
    assert!(result.is_err());
}

#[test]
fn ratchet_config_from_toml_string() {
    let toml = r#"
fail_fast = false
allow_missing_baseline = true
allow_missing_current = false

[[rules]]
pointer = "/complexity"
max_increase_pct = 15.0
level = "error"
description = "Complexity cap"
"#;
    let c = RatchetConfig::from_toml(toml).unwrap();
    assert!(!c.fail_fast);
    assert!(c.allow_missing_baseline);
    assert!(!c.allow_missing_current);
    assert_eq!(c.rules[0].description.as_deref(), Some("Complexity cap"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Gate with multiple rules (AND logic)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn multiple_rules_all_pass() {
    let receipt = json!({"tokens": 500, "files": 10, "lang": "rust"});
    let p = policy(vec![
        rule("tok", "/tokens", RuleOperator::Lte, json!(1000)),
        rule("files", "/files", RuleOperator::Gte, json!(1)),
        rule("lang", "/lang", RuleOperator::Eq, json!("rust")),
    ]);
    let result = evaluate_policy(&receipt, &p);
    assert!(result.passed);
    assert_eq!(result.errors, 0);
    assert_eq!(result.rule_results.len(), 3);
}

#[test]
fn multiple_rules_one_fails() {
    let receipt = json!({"tokens": 5000, "files": 10});
    let p = policy(vec![
        rule("tok", "/tokens", RuleOperator::Lte, json!(1000)),
        rule("files", "/files", RuleOperator::Gte, json!(1)),
    ]);
    let result = evaluate_policy(&receipt, &p);
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
    assert_eq!(result.rule_results.len(), 2);
}

#[test]
fn fail_fast_stops_after_first_error() {
    let receipt = json!({"a": 100, "b": 200});
    let p = PolicyConfig {
        rules: vec![
            rule("a", "/a", RuleOperator::Lte, json!(50)),
            rule("b", "/b", RuleOperator::Lte, json!(50)),
        ],
        fail_fast: true,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &p);
    assert!(!result.passed);
    assert_eq!(result.rule_results.len(), 1); // stopped after first
}

#[test]
fn warn_level_does_not_fail_gate() {
    let receipt = json!({"tokens": 2000});
    let mut r = rule("tok", "/tokens", RuleOperator::Lte, json!(1000));
    r.level = RuleLevel::Warn;
    let result = evaluate_policy(&receipt, &policy(vec![r]));
    assert!(result.passed);
    assert_eq!(result.warnings, 1);
    assert_eq!(result.errors, 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Gate output format
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn gate_result_from_empty_rules() {
    let result = GateResult::from_results(vec![]);
    assert!(result.passed);
    assert_eq!(result.errors, 0);
    assert_eq!(result.warnings, 0);
    assert!(result.rule_results.is_empty());
}

#[test]
fn gate_result_serializes_to_json() {
    let receipt = json!({"v": 10});
    let p = policy(vec![rule("check", "/v", RuleOperator::Lte, json!(100))]);
    let result = evaluate_policy(&receipt, &p);
    let json_str = serde_json::to_string(&result).unwrap();
    let parsed: Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed["passed"], json!(true));
    assert!(parsed["rule_results"].is_array());
    assert!(parsed["errors"].is_number());
    assert!(parsed["warnings"].is_number());
}

#[test]
fn rule_result_contains_actual_and_expected() {
    let receipt = json!({"v": 50});
    let p = policy(vec![rule("check", "/v", RuleOperator::Gt, json!(100))]);
    let result = evaluate_policy(&receipt, &p);
    let rr = &result.rule_results[0];
    assert!(!rr.passed);
    assert_eq!(rr.actual, Some(json!(50)));
    assert!(rr.expected.contains("/v"));
    assert!(rr.expected.contains(">"));
}

#[test]
fn ratchet_result_serializes_to_json() {
    let baseline = json!({"x": 10.0});
    let current = json!({"x": 11.0});
    let config = RatchetConfig {
        rules: vec![ratchet_rule("/x", Some(20.0), None)],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    let json_str = serde_json::to_string(&result).unwrap();
    let parsed: Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed["passed"], json!(true));
    assert!(parsed["ratchet_results"].is_array());
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Error handling for invalid pointers / missing values
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn missing_pointer_strict_fails_rule() {
    let receipt = json!({"a": 1});
    let p = policy(vec![rule("check", "/nonexistent", RuleOperator::Eq, json!(1))]);
    let result = evaluate_policy(&receipt, &p);
    assert!(!result.passed);
    let rr = &result.rule_results[0];
    assert!(!rr.passed);
    assert!(rr.message.as_ref().unwrap().contains("not found"));
}

#[test]
fn missing_pointer_allow_missing_passes() {
    let receipt = json!({"a": 1});
    let mut p = policy(vec![rule("check", "/nonexistent", RuleOperator::Eq, json!(1))]);
    p.allow_missing = true;
    let result = evaluate_policy(&receipt, &p);
    assert!(result.passed);
}

#[test]
fn negate_flag_inverts_result() {
    let receipt = json!({"v": 10});
    let mut r = rule("neg", "/v", RuleOperator::Eq, json!(10));
    r.negate = true;
    let result = evaluate_policy(&receipt, &policy(vec![r]));
    // v == 10 is true, but negated → false → fails
    assert!(!result.passed);
}

#[test]
fn negate_exists_means_should_not_exist() {
    let receipt = json!({"a": 1});
    let mut r = rule("neg_ex", "/a", RuleOperator::Exists, json!(null));
    r.value = None;
    r.negate = true;
    let result = evaluate_policy(&receipt, &policy(vec![r]));
    // /a exists, negated → should not exist → fail
    assert!(!result.passed);

    // Missing path + negate → should pass
    let mut r2 = rule("neg_ex2", "/missing", RuleOperator::Exists, json!(null));
    r2.value = None;
    r2.negate = true;
    let result2 = evaluate_policy(&receipt, &policy(vec![r2]));
    assert!(result2.passed);
}

#[test]
fn custom_failure_message_propagated() {
    let receipt = json!({"v": 999});
    let mut r = rule("check", "/v", RuleOperator::Lte, json!(100));
    r.message = Some("Value exceeds budget!".into());
    let result = evaluate_policy(&receipt, &policy(vec![r]));
    let rr = &result.rule_results[0];
    assert_eq!(rr.message.as_deref(), Some("Value exceeds budget!"));
}

#[test]
fn numeric_comparison_with_string_value() {
    // Numeric strings should be coerced to f64
    let receipt = json!({"v": "42"});
    let p = policy(vec![rule("check", "/v", RuleOperator::Gt, json!(40))]);
    let result = evaluate_policy(&receipt, &p);
    assert!(result.passed);
}
