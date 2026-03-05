//! Deep tests for tokmd-gate (w64 batch).
//!
//! Covers:
//! - Policy evaluation edge cases
//! - JSON pointer rule parsing (RFC 6901)
//! - Gate pass/fail logic with mixed levels
//! - Deterministic output ordering
//! - Error handling for malformed rules and TOML
//! - Property: same input → same output
//! - BDD-style Given/When/Then scenarios for policy evaluation
//! - Boundary: empty rules, single rule, max rules
//! - Edge: nested JSON pointers, missing fields, null values
//! - Ratchet edge cases: zero baseline, negative values, infinity

use serde_json::json;
use std::collections::BTreeMap;
use tokmd_gate::{
    GateResult, PolicyConfig, PolicyRule, RatchetConfig, RatchetGateResult, RatchetRule, RuleLevel,
    RuleOperator, RuleResult, evaluate_policy, evaluate_ratchet_policy, resolve_pointer,
};

// ─── Helpers ────────────────────────────────────────────────────────────────

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

fn policy(rules: Vec<PolicyRule>) -> PolicyConfig {
    PolicyConfig {
        rules,
        fail_fast: false,
        allow_missing: false,
    }
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

fn ratchet_cfg(rules: Vec<RatchetRule>) -> RatchetConfig {
    RatchetConfig {
        rules,
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. JSON Pointer resolution edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn pointer_empty_string_returns_whole_document() {
    let doc = json!({"a": 1});
    assert_eq!(resolve_pointer(&doc, ""), Some(&doc));
}

#[test]
fn pointer_root_slash_returns_empty_key() {
    let doc = json!({"": "empty_key"});
    assert_eq!(resolve_pointer(&doc, "/"), Some(&json!("empty_key")));
}

#[test]
fn pointer_deeply_nested_path() {
    let doc = json!({"a": {"b": {"c": {"d": {"e": 42}}}}});
    assert_eq!(resolve_pointer(&doc, "/a/b/c/d/e"), Some(&json!(42)));
}

#[test]
fn pointer_missing_intermediate_returns_none() {
    let doc = json!({"a": 1});
    assert_eq!(resolve_pointer(&doc, "/a/b/c"), None);
}

#[test]
fn pointer_no_leading_slash_returns_none() {
    let doc = json!({"a": 1});
    assert_eq!(resolve_pointer(&doc, "a"), None);
}

#[test]
fn pointer_array_index_out_of_bounds() {
    let doc = json!({"arr": [1, 2, 3]});
    assert_eq!(resolve_pointer(&doc, "/arr/5"), None);
}

#[test]
fn pointer_rfc6901_tilde_escape() {
    let doc = json!({"a/b": {"c~d": 99}});
    assert_eq!(resolve_pointer(&doc, "/a~1b/c~0d"), Some(&json!(99)));
}

#[test]
fn pointer_nested_arrays() {
    let doc = json!({"m": [[10, 20], [30, 40]]});
    assert_eq!(resolve_pointer(&doc, "/m/1/0"), Some(&json!(30)));
}

#[test]
fn pointer_null_value() {
    let doc = json!({"x": null});
    assert_eq!(resolve_pointer(&doc, "/x"), Some(&json!(null)));
}

#[test]
fn pointer_boolean_value() {
    let doc = json!({"flag": true});
    assert_eq!(resolve_pointer(&doc, "/flag"), Some(&json!(true)));
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Policy evaluation boundary conditions
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn empty_policy_always_passes() {
    let receipt = json!({"tokens": 999});
    let result = evaluate_policy(&receipt, &policy(vec![]));
    assert!(result.passed);
    assert_eq!(result.errors, 0);
    assert_eq!(result.warnings, 0);
    assert!(result.rule_results.is_empty());
}

#[test]
fn single_passing_rule() {
    let receipt = json!({"v": 5});
    let result = evaluate_policy(
        &receipt,
        &policy(vec![rule("r", "/v", RuleOperator::Lt, json!(10))]),
    );
    assert!(result.passed);
    assert_eq!(result.rule_results.len(), 1);
    assert!(result.rule_results[0].passed);
}

#[test]
fn single_failing_error_rule() {
    let receipt = json!({"v": 50});
    let result = evaluate_policy(
        &receipt,
        &policy(vec![rule("r", "/v", RuleOperator::Lt, json!(10))]),
    );
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
}

#[test]
fn many_rules_all_pass() {
    let receipt = json!({"a": 1, "b": 2, "c": 3, "d": 4, "e": 5});
    let rules: Vec<PolicyRule> = ["a", "b", "c", "d", "e"]
        .iter()
        .enumerate()
        .map(|(i, k)| {
            rule(
                &format!("r{i}"),
                &format!("/{k}"),
                RuleOperator::Lte,
                json!(10),
            )
        })
        .collect();
    let result = evaluate_policy(&receipt, &policy(rules));
    assert!(result.passed);
    assert_eq!(result.rule_results.len(), 5);
}

#[test]
fn many_rules_one_fails() {
    let receipt = json!({"a": 1, "b": 20, "c": 3});
    let rules = vec![
        rule("r0", "/a", RuleOperator::Lte, json!(10)),
        rule("r1", "/b", RuleOperator::Lte, json!(10)),
        rule("r2", "/c", RuleOperator::Lte, json!(10)),
    ];
    let result = evaluate_policy(&receipt, &policy(rules));
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
    assert_eq!(result.rule_results[1].name, "r1");
    assert!(!result.rule_results[1].passed);
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Mixed error/warn levels
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn warn_only_failures_still_pass_gate() {
    let receipt = json!({"x": 100});
    let rules = vec![warn_rule("w1", "/x", RuleOperator::Lt, json!(50))];
    let result = evaluate_policy(&receipt, &policy(rules));
    assert!(result.passed);
    assert_eq!(result.warnings, 1);
    assert_eq!(result.errors, 0);
}

#[test]
fn mixed_warn_and_error_failures() {
    let receipt = json!({"x": 100, "y": 200});
    let rules = vec![
        warn_rule("w", "/x", RuleOperator::Lt, json!(50)),
        rule("e", "/y", RuleOperator::Lt, json!(50)),
    ];
    let result = evaluate_policy(&receipt, &policy(rules));
    assert!(!result.passed);
    assert_eq!(result.warnings, 1);
    assert_eq!(result.errors, 1);
}

#[test]
fn passing_warn_not_counted() {
    let receipt = json!({"x": 1});
    let rules = vec![warn_rule("w", "/x", RuleOperator::Lt, json!(50))];
    let result = evaluate_policy(&receipt, &policy(rules));
    assert!(result.passed);
    assert_eq!(result.warnings, 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Operator coverage
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn all_numeric_operators() {
    let receipt = json!({"v": 10});
    let cases: Vec<(RuleOperator, serde_json::Value, bool)> = vec![
        (RuleOperator::Gt, json!(5), true),
        (RuleOperator::Gt, json!(10), false),
        (RuleOperator::Gte, json!(10), true),
        (RuleOperator::Gte, json!(11), false),
        (RuleOperator::Lt, json!(15), true),
        (RuleOperator::Lt, json!(10), false),
        (RuleOperator::Lte, json!(10), true),
        (RuleOperator::Lte, json!(9), false),
        (RuleOperator::Eq, json!(10), true),
        (RuleOperator::Eq, json!(11), false),
        (RuleOperator::Ne, json!(11), true),
        (RuleOperator::Ne, json!(10), false),
    ];
    for (op, val, expected_pass) in cases {
        let result = evaluate_policy(
            &receipt,
            &policy(vec![rule("op_test", "/v", op, val.clone())]),
        );
        assert_eq!(
            result.passed, expected_pass,
            "op={op}, val={val}, expected_pass={expected_pass}"
        );
    }
}

#[test]
fn contains_on_string() {
    let receipt = json!({"msg": "hello world"});
    let r = rule("c", "/msg", RuleOperator::Contains, json!("world"));
    let result = evaluate_policy(&receipt, &policy(vec![r]));
    assert!(result.passed);
}

#[test]
fn contains_on_array() {
    let receipt = json!({"tags": ["alpha", "beta"]});
    let r = rule("c", "/tags", RuleOperator::Contains, json!("beta"));
    let result = evaluate_policy(&receipt, &policy(vec![r]));
    assert!(result.passed);
}

#[test]
fn in_operator_with_matching_value() {
    let receipt = json!({"license": "MIT"});
    let r = PolicyRule {
        name: "lic".into(),
        pointer: "/license".into(),
        op: RuleOperator::In,
        value: None,
        values: Some(vec![json!("MIT"), json!("Apache-2.0")]),
        negate: false,
        level: RuleLevel::Error,
        message: None,
    };
    let result = evaluate_policy(&receipt, &policy(vec![r]));
    assert!(result.passed);
}

#[test]
fn in_operator_no_match() {
    let receipt = json!({"license": "GPL"});
    let r = PolicyRule {
        name: "lic".into(),
        pointer: "/license".into(),
        op: RuleOperator::In,
        value: None,
        values: Some(vec![json!("MIT"), json!("Apache-2.0")]),
        negate: false,
        level: RuleLevel::Error,
        message: None,
    };
    let result = evaluate_policy(&receipt, &policy(vec![r]));
    assert!(!result.passed);
}

#[test]
fn exists_operator_present() {
    let receipt = json!({"key": 1});
    let r = PolicyRule {
        name: "ex".into(),
        pointer: "/key".into(),
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
fn exists_operator_missing() {
    let receipt = json!({"key": 1});
    let r = PolicyRule {
        name: "ex".into(),
        pointer: "/missing".into(),
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

#[test]
fn exists_negated_passes_when_absent() {
    let receipt = json!({"key": 1});
    let r = PolicyRule {
        name: "no_secrets".into(),
        pointer: "/secrets".into(),
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

// ═══════════════════════════════════════════════════════════════════════════
// 5. Negate flag
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn negate_flips_numeric_comparison() {
    let receipt = json!({"v": 10});
    let mut r = rule("neg", "/v", RuleOperator::Gt, json!(5));
    r.negate = true;
    let result = evaluate_policy(&receipt, &policy(vec![r]));
    // 10 > 5 is true, negated → false
    assert!(!result.passed);
}

#[test]
fn negate_flips_eq() {
    let receipt = json!({"v": 10});
    let mut r = rule("neg", "/v", RuleOperator::Eq, json!(10));
    r.negate = true;
    let result = evaluate_policy(&receipt, &policy(vec![r]));
    // 10 == 10 is true, negated → false
    assert!(!result.passed);
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. allow_missing
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn allow_missing_passes_when_pointer_absent() {
    let receipt = json!({"a": 1});
    let mut p = policy(vec![rule("r", "/missing", RuleOperator::Eq, json!(42))]);
    p.allow_missing = true;
    let result = evaluate_policy(&receipt, &p);
    assert!(result.passed);
}

#[test]
fn disallow_missing_fails_when_pointer_absent() {
    let receipt = json!({"a": 1});
    let p = policy(vec![rule("r", "/missing", RuleOperator::Eq, json!(42))]);
    let result = evaluate_policy(&receipt, &p);
    assert!(!result.passed);
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. fail_fast
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn fail_fast_stops_after_first_error() {
    let receipt = json!({"a": 100, "b": 200});
    let p = PolicyConfig {
        rules: vec![
            rule("r1", "/a", RuleOperator::Lt, json!(10)),
            rule("r2", "/b", RuleOperator::Lt, json!(10)),
        ],
        fail_fast: true,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &p);
    assert!(!result.passed);
    assert_eq!(
        result.rule_results.len(),
        1,
        "fail_fast should stop after first error"
    );
}

#[test]
fn fail_fast_does_not_stop_on_warn() {
    let receipt = json!({"a": 100, "b": 200});
    let p = PolicyConfig {
        rules: vec![
            warn_rule("w1", "/a", RuleOperator::Lt, json!(10)),
            rule("r2", "/b", RuleOperator::Lt, json!(10)),
        ],
        fail_fast: true,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &p);
    assert!(!result.passed);
    // Warn failure doesn't trigger fail_fast; continues to error rule
    assert_eq!(result.rule_results.len(), 2);
}

#[test]
fn fail_fast_continues_when_passing() {
    let receipt = json!({"a": 1, "b": 2});
    let p = PolicyConfig {
        rules: vec![
            rule("r1", "/a", RuleOperator::Lt, json!(10)),
            rule("r2", "/b", RuleOperator::Lt, json!(10)),
        ],
        fail_fast: true,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &p);
    assert!(result.passed);
    assert_eq!(result.rule_results.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Deterministic output ordering
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn rule_results_preserve_insertion_order() {
    let receipt = json!({"a": 1, "b": 2, "c": 3});
    let rules = vec![
        rule("first", "/a", RuleOperator::Eq, json!(1)),
        rule("second", "/b", RuleOperator::Eq, json!(2)),
        rule("third", "/c", RuleOperator::Eq, json!(3)),
    ];
    let result = evaluate_policy(&receipt, &policy(rules));
    let names: Vec<&str> = result
        .rule_results
        .iter()
        .map(|r| r.name.as_str())
        .collect();
    assert_eq!(names, vec!["first", "second", "third"]);
}

#[test]
fn same_input_same_output_property() {
    let receipt = json!({"tokens": 500, "files": 10});
    let p = policy(vec![
        rule("t", "/tokens", RuleOperator::Lte, json!(1000)),
        rule("f", "/files", RuleOperator::Gte, json!(1)),
    ]);
    let r1 = evaluate_policy(&receipt, &p);
    let r2 = evaluate_policy(&receipt, &p);
    assert_eq!(r1.passed, r2.passed);
    assert_eq!(r1.errors, r2.errors);
    assert_eq!(r1.warnings, r2.warnings);
    assert_eq!(r1.rule_results.len(), r2.rule_results.len());
    for (a, b) in r1.rule_results.iter().zip(r2.rule_results.iter()) {
        assert_eq!(a.name, b.name);
        assert_eq!(a.passed, b.passed);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. TOML parsing edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn parse_empty_toml_gives_default_policy() {
    let p = PolicyConfig::from_toml("").unwrap();
    assert!(p.rules.is_empty());
    assert!(!p.fail_fast);
    assert!(!p.allow_missing);
}

#[test]
fn parse_invalid_toml_returns_error() {
    let result = PolicyConfig::from_toml("[[invalid toml!!");
    assert!(result.is_err());
}

#[test]
fn parse_multiple_rules_toml() {
    let toml = r#"
fail_fast = true
allow_missing = true

[[rules]]
name = "r1"
pointer = "/a"
op = "lte"
value = 100

[[rules]]
name = "r2"
pointer = "/b"
op = "eq"
value = "hello"
"#;
    let p = PolicyConfig::from_toml(toml).unwrap();
    assert!(p.fail_fast);
    assert!(p.allow_missing);
    assert_eq!(p.rules.len(), 2);
    assert_eq!(p.rules[0].op, RuleOperator::Lte);
    assert_eq!(p.rules[1].op, RuleOperator::Eq);
}

#[test]
fn parse_ratchet_config_empty_toml() {
    let cfg = RatchetConfig::from_toml("").unwrap();
    assert!(cfg.rules.is_empty());
    assert!(!cfg.fail_fast);
    assert!(!cfg.allow_missing_baseline);
    assert!(!cfg.allow_missing_current);
}

#[test]
fn parse_ratchet_config_invalid_toml() {
    assert!(RatchetConfig::from_toml("not valid toml {{").is_err());
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. GateResult construction invariants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn gate_result_from_empty_vec() {
    let g = GateResult::from_results(vec![]);
    assert!(g.passed);
    assert_eq!(g.errors, 0);
    assert_eq!(g.warnings, 0);
}

#[test]
fn gate_result_passed_rules_contribute_zero_counts() {
    let results = vec![
        RuleResult {
            name: "r1".into(),
            passed: true,
            level: RuleLevel::Error,
            actual: Some(json!(10)),
            expected: "ok".into(),
            message: None,
        },
        RuleResult {
            name: "r2".into(),
            passed: true,
            level: RuleLevel::Warn,
            actual: Some(json!(20)),
            expected: "ok".into(),
            message: None,
        },
    ];
    let g = GateResult::from_results(results);
    assert!(g.passed);
    assert_eq!(g.errors, 0);
    assert_eq!(g.warnings, 0);
}

#[test]
fn gate_result_counts_errors_and_warnings_separately() {
    let results = vec![
        RuleResult {
            name: "e1".into(),
            passed: false,
            level: RuleLevel::Error,
            actual: None,
            expected: "x".into(),
            message: None,
        },
        RuleResult {
            name: "w1".into(),
            passed: false,
            level: RuleLevel::Warn,
            actual: None,
            expected: "y".into(),
            message: None,
        },
        RuleResult {
            name: "e2".into(),
            passed: false,
            level: RuleLevel::Error,
            actual: None,
            expected: "z".into(),
            message: None,
        },
    ];
    let g = GateResult::from_results(results);
    assert!(!g.passed);
    assert_eq!(g.errors, 2);
    assert_eq!(g.warnings, 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// 11. Ratchet evaluation edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ratchet_zero_baseline_zero_current_passes() {
    let baseline = json!({"v": 0.0});
    let current = json!({"v": 0.0});
    let cfg = ratchet_cfg(vec![ratchet_rule("/v", Some(10.0), None)]);
    let result = evaluate_ratchet_policy(&cfg, &baseline, &current);
    assert!(result.passed);
}

#[test]
fn ratchet_zero_baseline_nonzero_current_fails() {
    let baseline = json!({"v": 0.0});
    let current = json!({"v": 1.0});
    let cfg = ratchet_cfg(vec![ratchet_rule("/v", Some(10.0), None)]);
    let result = evaluate_ratchet_policy(&cfg, &baseline, &current);
    // 0 → 1 is infinite increase
    assert!(!result.passed);
}

#[test]
fn ratchet_decrease_always_passes_pct_check() {
    let baseline = json!({"v": 100.0});
    let current = json!({"v": 50.0});
    let cfg = ratchet_cfg(vec![ratchet_rule("/v", Some(5.0), None)]);
    let result = evaluate_ratchet_policy(&cfg, &baseline, &current);
    assert!(result.passed);
}

#[test]
fn ratchet_max_value_ceiling_enforced() {
    let baseline = json!({"v": 10.0});
    let current = json!({"v": 100.0});
    let cfg = ratchet_cfg(vec![ratchet_rule("/v", None, Some(50.0))]);
    let result = evaluate_ratchet_policy(&cfg, &baseline, &current);
    assert!(!result.passed);
}

#[test]
fn ratchet_max_value_under_ceiling_passes() {
    let baseline = json!({"v": 10.0});
    let current = json!({"v": 45.0});
    let cfg = ratchet_cfg(vec![ratchet_rule("/v", None, Some(50.0))]);
    let result = evaluate_ratchet_policy(&cfg, &baseline, &current);
    assert!(result.passed);
}

#[test]
fn ratchet_missing_current_fails_by_default() {
    let baseline = json!({"v": 10.0});
    let current = json!({});
    let cfg = ratchet_cfg(vec![ratchet_rule("/v", Some(10.0), None)]);
    let result = evaluate_ratchet_policy(&cfg, &baseline, &current);
    assert!(!result.passed);
}

#[test]
fn ratchet_missing_current_passes_when_allowed() {
    let baseline = json!({"v": 10.0});
    let current = json!({});
    let mut cfg = ratchet_cfg(vec![ratchet_rule("/v", Some(10.0), None)]);
    cfg.allow_missing_current = true;
    let result = evaluate_ratchet_policy(&cfg, &baseline, &current);
    assert!(result.passed);
}

#[test]
fn ratchet_missing_baseline_fails_by_default() {
    let baseline = json!({});
    let current = json!({"v": 10.0});
    let cfg = ratchet_cfg(vec![ratchet_rule("/v", Some(10.0), None)]);
    let result = evaluate_ratchet_policy(&cfg, &baseline, &current);
    assert!(!result.passed);
}

#[test]
fn ratchet_missing_baseline_passes_when_allowed() {
    let baseline = json!({});
    let current = json!({"v": 10.0});
    let mut cfg = ratchet_cfg(vec![ratchet_rule("/v", Some(10.0), None)]);
    cfg.allow_missing_baseline = true;
    let result = evaluate_ratchet_policy(&cfg, &baseline, &current);
    assert!(result.passed);
}

#[test]
fn ratchet_fail_fast_stops_after_first_error() {
    let baseline = json!({"a": 10.0, "b": 10.0});
    let current = json!({"a": 100.0, "b": 100.0});
    let mut cfg = ratchet_cfg(vec![
        ratchet_rule("/a", Some(5.0), None),
        ratchet_rule("/b", Some(5.0), None),
    ]);
    cfg.fail_fast = true;
    let result = evaluate_ratchet_policy(&cfg, &baseline, &current);
    assert!(!result.passed);
    assert_eq!(result.ratchet_results.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// 12. RatchetGateResult invariants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ratchet_gate_result_empty_passes() {
    let g = RatchetGateResult::from_results(vec![]);
    assert!(g.passed);
    assert_eq!(g.errors, 0);
    assert_eq!(g.warnings, 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// 13. Custom message propagation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn custom_message_appears_on_failure() {
    let receipt = json!({"v": 100});
    let mut r = rule("r", "/v", RuleOperator::Lt, json!(10));
    r.message = Some("Custom failure message".into());
    let result = evaluate_policy(&receipt, &policy(vec![r]));
    let rr = &result.rule_results[0];
    assert!(!rr.passed);
    assert_eq!(rr.message.as_deref(), Some("Custom failure message"));
}

#[test]
fn custom_message_absent_on_pass() {
    let receipt = json!({"v": 1});
    let mut r = rule("r", "/v", RuleOperator::Lt, json!(10));
    r.message = Some("Should not appear".into());
    let result = evaluate_policy(&receipt, &policy(vec![r]));
    assert!(result.rule_results[0].passed);
    assert!(result.rule_results[0].message.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
// 14. RuleOperator and RuleLevel defaults / Display
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn rule_operator_default_is_eq() {
    assert_eq!(RuleOperator::default(), RuleOperator::Eq);
}

#[test]
fn rule_level_default_is_error() {
    assert_eq!(RuleLevel::default(), RuleLevel::Error);
}

#[test]
fn rule_operator_display_all_variants() {
    let expected: BTreeMap<&str, RuleOperator> = [
        (">", RuleOperator::Gt),
        (">=", RuleOperator::Gte),
        ("<", RuleOperator::Lt),
        ("<=", RuleOperator::Lte),
        ("==", RuleOperator::Eq),
        ("!=", RuleOperator::Ne),
        ("in", RuleOperator::In),
        ("contains", RuleOperator::Contains),
        ("exists", RuleOperator::Exists),
    ]
    .into_iter()
    .collect();

    for (display, op) in &expected {
        assert_eq!(&op.to_string(), display);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 15. BDD-style scenarios
// ═══════════════════════════════════════════════════════════════════════════

/// Given a CI receipt with tokens under budget
/// When evaluating a lte policy
/// Then the gate passes
#[test]
fn bdd_tokens_under_budget_passes() {
    // Given
    let receipt = json!({"derived": {"totals": {"tokens": 50_000}}});
    let p = policy(vec![rule(
        "max_tokens",
        "/derived/totals/tokens",
        RuleOperator::Lte,
        json!(100_000),
    )]);

    // When
    let result = evaluate_policy(&receipt, &p);

    // Then
    assert!(result.passed);
    assert_eq!(result.errors, 0);
}

/// Given a receipt with tokens over budget
/// When evaluating a lte policy
/// Then the gate fails with error count 1
#[test]
fn bdd_tokens_over_budget_fails() {
    // Given
    let receipt = json!({"derived": {"totals": {"tokens": 200_000}}});
    let p = policy(vec![rule(
        "max_tokens",
        "/derived/totals/tokens",
        RuleOperator::Lte,
        json!(100_000),
    )]);

    // When
    let result = evaluate_policy(&receipt, &p);

    // Then
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
}

/// Given a complexity baseline of 10 and current of 10.5 (5%)
/// When ratchet allows up to 10%
/// Then ratchet passes
#[test]
fn bdd_ratchet_within_tolerance() {
    // Given
    let baseline = json!({"complexity": 10.0});
    let current = json!({"complexity": 10.5});
    let cfg = ratchet_cfg(vec![ratchet_rule("/complexity", Some(10.0), None)]);

    // When
    let result = evaluate_ratchet_policy(&cfg, &baseline, &current);

    // Then
    assert!(result.passed);
}

/// Given a complexity baseline of 10 and current of 12 (20%)
/// When ratchet allows up to 10%
/// Then ratchet fails
#[test]
fn bdd_ratchet_over_tolerance() {
    // Given
    let baseline = json!({"complexity": 10.0});
    let current = json!({"complexity": 12.0});
    let cfg = ratchet_cfg(vec![ratchet_rule("/complexity", Some(10.0), None)]);

    // When
    let result = evaluate_ratchet_policy(&cfg, &baseline, &current);

    // Then
    assert!(!result.passed);
}

// ═══════════════════════════════════════════════════════════════════════════
// 16. Numeric string coercion
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn numeric_string_coerced_for_comparison() {
    let receipt = json!({"v": "42"});
    let result = evaluate_policy(
        &receipt,
        &policy(vec![rule("r", "/v", RuleOperator::Lte, json!(100))]),
    );
    assert!(result.passed);
}

#[test]
fn string_eq_comparison_exact() {
    let receipt = json!({"name": "hello"});
    let result = evaluate_policy(
        &receipt,
        &policy(vec![rule("r", "/name", RuleOperator::Eq, json!("hello"))]),
    );
    assert!(result.passed);
}

// ═══════════════════════════════════════════════════════════════════════════
// 17. Serialization round-trip
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn policy_config_serde_roundtrip() {
    let p = policy(vec![
        rule("r1", "/a", RuleOperator::Lte, json!(100)),
        rule("r2", "/b", RuleOperator::Eq, json!("test")),
    ]);
    let json_str = serde_json::to_string(&p).unwrap();
    let p2: PolicyConfig = serde_json::from_str(&json_str).unwrap();
    assert_eq!(p2.rules.len(), 2);
    assert_eq!(p2.rules[0].name, "r1");
    assert_eq!(p2.rules[1].name, "r2");
}

#[test]
fn gate_result_serde_roundtrip() {
    let g = GateResult::from_results(vec![RuleResult {
        name: "test".into(),
        passed: true,
        level: RuleLevel::Error,
        actual: Some(json!(42)),
        expected: "/x <= 100".into(),
        message: None,
    }]);
    let json_str = serde_json::to_string(&g).unwrap();
    let g2: GateResult = serde_json::from_str(&json_str).unwrap();
    assert_eq!(g2.passed, g.passed);
    assert_eq!(g2.rule_results.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// 18. Null and boolean JSON values
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn null_value_eq_null() {
    let receipt = json!({"v": null});
    let result = evaluate_policy(
        &receipt,
        &policy(vec![rule("r", "/v", RuleOperator::Eq, json!(null))]),
    );
    assert!(result.passed);
}

#[test]
fn boolean_eq_true() {
    let receipt = json!({"flag": true});
    let result = evaluate_policy(
        &receipt,
        &policy(vec![rule("r", "/flag", RuleOperator::Eq, json!(true))]),
    );
    assert!(result.passed);
}

#[test]
fn boolean_ne_false() {
    let receipt = json!({"flag": true});
    let result = evaluate_policy(
        &receipt,
        &policy(vec![rule("r", "/flag", RuleOperator::Ne, json!(false))]),
    );
    assert!(result.passed);
}
