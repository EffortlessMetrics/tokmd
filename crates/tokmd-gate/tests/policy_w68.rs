//! W68 property-based and integration tests for tokmd-gate policy evaluation.

use serde_json::json;
use tokmd_gate::{
    evaluate_policy, evaluate_ratchet_policy, resolve_pointer, GateResult, PolicyConfig,
    PolicyRule, RatchetConfig, RatchetGateResult, RatchetRule, RuleLevel, RuleOperator,
};

// ── Helper ────────────────────────────────────────────────────────────────

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

// ═══════════════════════════════════════════════════════════════════════════
// 1. JSON Pointer resolution
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn pointer_resolves_nested_object() {
    let doc = json!({"a": {"b": {"c": 99}}});
    assert_eq!(resolve_pointer(&doc, "/a/b/c"), Some(&json!(99)));
}

#[test]
fn pointer_resolves_array_element() {
    let doc = json!({"arr": [10, 20, 30]});
    assert_eq!(resolve_pointer(&doc, "/arr/1"), Some(&json!(20)));
}

#[test]
fn pointer_empty_returns_root() {
    let doc = json!({"x": 1});
    assert_eq!(resolve_pointer(&doc, ""), Some(&doc));
}

#[test]
fn pointer_missing_key_returns_none() {
    let doc = json!({"a": 1});
    assert_eq!(resolve_pointer(&doc, "/b"), None);
}

#[test]
fn pointer_no_leading_slash_returns_none() {
    let doc = json!({"a": 1});
    assert_eq!(resolve_pointer(&doc, "a"), None);
}

#[test]
fn pointer_tilde_escaping() {
    let doc = json!({"a/b": {"c~d": 42}});
    assert_eq!(resolve_pointer(&doc, "/a~1b/c~0d"), Some(&json!(42)));
}

#[test]
fn pointer_nested_array() {
    let doc = json!({"m": [[1, 2], [3, 4]]});
    assert_eq!(resolve_pointer(&doc, "/m/1/0"), Some(&json!(3)));
}

#[test]
fn pointer_out_of_bounds_array() {
    let doc = json!({"arr": [1]});
    assert_eq!(resolve_pointer(&doc, "/arr/5"), None);
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Rule operator evaluation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn op_eq_passes_on_equal() {
    let receipt = json!({"v": 42});
    let p = policy(vec![rule("eq", "/v", RuleOperator::Eq, json!(42))]);
    assert!(evaluate_policy(&receipt, &p).passed);
}

#[test]
fn op_neq_passes_on_different() {
    let receipt = json!({"v": 1});
    let p = policy(vec![rule("ne", "/v", RuleOperator::Ne, json!(2))]);
    assert!(evaluate_policy(&receipt, &p).passed);
}

#[test]
fn op_gt_passes_when_greater() {
    let receipt = json!({"v": 10});
    let p = policy(vec![rule("gt", "/v", RuleOperator::Gt, json!(5))]);
    assert!(evaluate_policy(&receipt, &p).passed);
}

#[test]
fn op_gt_fails_when_equal() {
    let receipt = json!({"v": 5});
    let p = policy(vec![rule("gt", "/v", RuleOperator::Gt, json!(5))]);
    assert!(!evaluate_policy(&receipt, &p).passed);
}

#[test]
fn op_lt_passes_when_less() {
    let receipt = json!({"v": 3});
    let p = policy(vec![rule("lt", "/v", RuleOperator::Lt, json!(10))]);
    assert!(evaluate_policy(&receipt, &p).passed);
}

#[test]
fn op_gte_passes_when_equal() {
    let receipt = json!({"v": 10});
    let p = policy(vec![rule("gte", "/v", RuleOperator::Gte, json!(10))]);
    assert!(evaluate_policy(&receipt, &p).passed);
}

#[test]
fn op_lte_passes_when_equal() {
    let receipt = json!({"v": 10});
    let p = policy(vec![rule("lte", "/v", RuleOperator::Lte, json!(10))]);
    assert!(evaluate_policy(&receipt, &p).passed);
}

#[test]
fn op_lte_fails_when_greater() {
    let receipt = json!({"v": 20});
    let p = policy(vec![rule("lte", "/v", RuleOperator::Lte, json!(10))]);
    assert!(!evaluate_policy(&receipt, &p).passed);
}

#[test]
fn op_contains_string() {
    let receipt = json!({"lang": "Rust is great"});
    let p = policy(vec![rule(
        "has_rust",
        "/lang",
        RuleOperator::Contains,
        json!("Rust"),
    )]);
    assert!(evaluate_policy(&receipt, &p).passed);
}

#[test]
fn op_contains_array() {
    let receipt = json!({"tags": ["rust", "cli", "tokei"]});
    let p = policy(vec![rule(
        "has_cli",
        "/tags",
        RuleOperator::Contains,
        json!("cli"),
    )]);
    assert!(evaluate_policy(&receipt, &p).passed);
}

#[test]
fn op_in_passes_when_value_in_list() {
    let receipt = json!({"status": "pass"});
    let mut r = rule("in_check", "/status", RuleOperator::In, json!(null));
    r.value = None;
    r.values = Some(vec![json!("pass"), json!("warn")]);
    let p = policy(vec![r]);
    assert!(evaluate_policy(&receipt, &p).passed);
}

#[test]
fn op_in_fails_when_absent() {
    let receipt = json!({"status": "fail"});
    let mut r = rule("in_check", "/status", RuleOperator::In, json!(null));
    r.value = None;
    r.values = Some(vec![json!("pass"), json!("warn")]);
    let p = policy(vec![r]);
    assert!(!evaluate_policy(&receipt, &p).passed);
}

#[test]
fn op_exists_passes_when_present() {
    let receipt = json!({"key": 1});
    let mut r = rule("exists", "/key", RuleOperator::Exists, json!(null));
    r.value = None;
    let p = policy(vec![r]);
    assert!(evaluate_policy(&receipt, &p).passed);
}

#[test]
fn op_exists_fails_when_absent() {
    let receipt = json!({"key": 1});
    let mut r = rule("exists", "/missing", RuleOperator::Exists, json!(null));
    r.value = None;
    let p = policy(vec![r]);
    assert!(!evaluate_policy(&receipt, &p).passed);
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Gate verdict logic (pass / fail / warn)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn verdict_all_pass() {
    let receipt = json!({"a": 1, "b": 2});
    let p = policy(vec![
        rule("r1", "/a", RuleOperator::Eq, json!(1)),
        rule("r2", "/b", RuleOperator::Lt, json!(10)),
    ]);
    let result = evaluate_policy(&receipt, &p);
    assert!(result.passed);
    assert_eq!(result.errors, 0);
    assert_eq!(result.warnings, 0);
}

#[test]
fn verdict_error_fails_gate() {
    let receipt = json!({"v": 999});
    let p = policy(vec![rule("max", "/v", RuleOperator::Lte, json!(100))]);
    let result = evaluate_policy(&receipt, &p);
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
}

#[test]
fn verdict_warn_does_not_fail_gate() {
    let receipt = json!({"v": 999});
    let mut r = rule("advisory", "/v", RuleOperator::Lte, json!(100));
    r.level = RuleLevel::Warn;
    let p = policy(vec![r]);
    let result = evaluate_policy(&receipt, &p);
    assert!(result.passed);
    assert_eq!(result.warnings, 1);
    assert_eq!(result.errors, 0);
}

#[test]
fn verdict_mixed_error_and_warn() {
    let receipt = json!({"a": 200, "b": 200});
    let mut warn_rule = rule("w", "/a", RuleOperator::Lte, json!(100));
    warn_rule.level = RuleLevel::Warn;
    let err_rule = rule("e", "/b", RuleOperator::Lte, json!(100));
    let p = policy(vec![warn_rule, err_rule]);
    let result = evaluate_policy(&receipt, &p);
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
    assert_eq!(result.warnings, 1);
}

#[test]
fn gate_result_empty_rules_passes() {
    let result = GateResult::from_results(vec![]);
    assert!(result.passed);
    assert_eq!(result.errors, 0);
    assert_eq!(result.warnings, 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Negate flag
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn negate_inverts_result() {
    let receipt = json!({"v": 5});
    let mut r = rule("neg", "/v", RuleOperator::Eq, json!(5));
    r.negate = true;
    let p = policy(vec![r]);
    assert!(!evaluate_policy(&receipt, &p).passed);
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Missing value handling
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn missing_value_fails_by_default() {
    let receipt = json!({"a": 1});
    let p = policy(vec![rule("miss", "/nope", RuleOperator::Eq, json!(1))]);
    assert!(!evaluate_policy(&receipt, &p).passed);
}

#[test]
fn allow_missing_treats_as_pass() {
    let receipt = json!({"a": 1});
    let mut p = policy(vec![rule("miss", "/nope", RuleOperator::Eq, json!(1))]);
    p.allow_missing = true;
    assert!(evaluate_policy(&receipt, &p).passed);
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Fail-fast
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn fail_fast_stops_after_first_error() {
    let receipt = json!({"a": 999, "b": 999});
    let mut p = policy(vec![
        rule("r1", "/a", RuleOperator::Lte, json!(10)),
        rule("r2", "/b", RuleOperator::Lte, json!(10)),
    ]);
    p.fail_fast = true;
    let result = evaluate_policy(&receipt, &p);
    assert!(!result.passed);
    // Only one rule evaluated due to fail_fast
    assert_eq!(result.rule_results.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. TOML parsing
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn policy_from_toml_roundtrip() {
    let toml = r#"
fail_fast = true
allow_missing = false

[[rules]]
name = "max_tokens"
pointer = "/tokens"
op = "lte"
value = 500000
level = "error"
"#;
    let p = PolicyConfig::from_toml(toml).unwrap();
    assert!(p.fail_fast);
    assert!(!p.allow_missing);
    assert_eq!(p.rules.len(), 1);
    assert_eq!(p.rules[0].op, RuleOperator::Lte);
}

#[test]
fn ratchet_config_from_toml_parses() {
    let toml = r#"
fail_fast = true
allow_missing_baseline = true

[[rules]]
pointer = "/complexity/avg"
max_increase_pct = 5.0
level = "error"
"#;
    let config = RatchetConfig::from_toml(toml).unwrap();
    assert!(config.fail_fast);
    assert!(config.allow_missing_baseline);
    assert_eq!(config.rules.len(), 1);
    assert_eq!(config.rules[0].max_increase_pct, Some(5.0));
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Ratchet evaluation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ratchet_passes_within_threshold() {
    let baseline = json!({"complexity": 10.0});
    let current = json!({"complexity": 10.5});
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
fn ratchet_fails_exceeding_threshold() {
    let baseline = json!({"complexity": 10.0});
    let current = json!({"complexity": 15.0});
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
    let current = json!({"complexity": 25.0});
    let config = RatchetConfig {
        rules: vec![RatchetRule {
            pointer: "/complexity".into(),
            max_increase_pct: None,
            max_value: Some(20.0),
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
fn ratchet_empty_rules_passes() {
    let result = RatchetGateResult::from_results(vec![]);
    assert!(result.passed);
    assert_eq!(result.errors, 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. Proptest: determinism
// ═══════════════════════════════════════════════════════════════════════════

mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn evaluate_is_deterministic(val in 0i64..1_000_000) {
            let receipt = json!({"v": val});
            let p = policy(vec![rule("r", "/v", RuleOperator::Lte, json!(500_000))]);
            let r1 = evaluate_policy(&receipt, &p);
            let r2 = evaluate_policy(&receipt, &p);
            prop_assert_eq!(r1.passed, r2.passed);
            prop_assert_eq!(r1.errors, r2.errors);
            prop_assert_eq!(r1.warnings, r2.warnings);
        }

        #[test]
        fn pointer_resolution_deterministic(idx in 0usize..10) {
            let doc = json!({"arr": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]});
            let ptr = format!("/arr/{}", idx);
            let a = resolve_pointer(&doc, &ptr);
            let b = resolve_pointer(&doc, &ptr);
            prop_assert_eq!(a, b);
        }
    }
}
