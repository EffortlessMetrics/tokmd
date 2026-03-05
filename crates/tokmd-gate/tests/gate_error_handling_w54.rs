//! Comprehensive error handling and edge case tests for tokmd-gate.

use serde_json::json;
use tokmd_gate::{
    GateResult, PolicyConfig, PolicyRule, RatchetConfig, RatchetGateResult, RatchetRule, RuleLevel,
    RuleOperator, RuleResult, evaluate_policy, evaluate_ratchet_policy, resolve_pointer,
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

// ── Invalid / missing JSON pointer paths ──────────────────────────────

#[test]
fn pointer_without_leading_slash_returns_none() {
    let doc = json!({"foo": 1});
    assert_eq!(resolve_pointer(&doc, "foo"), None);
}

#[test]
fn pointer_to_nonexistent_deep_path() {
    let doc = json!({"a": {"b": 1}});
    assert_eq!(resolve_pointer(&doc, "/a/c/d"), None);
}

#[test]
fn pointer_array_index_out_of_bounds() {
    let doc = json!({"items": [1, 2, 3]});
    assert_eq!(resolve_pointer(&doc, "/items/99"), None);
}

#[test]
fn pointer_negative_index_returns_none() {
    let doc = json!({"items": [1, 2, 3]});
    assert_eq!(resolve_pointer(&doc, "/items/-1"), None);
}

#[test]
fn pointer_on_scalar_returns_none() {
    let doc = json!(42);
    assert_eq!(resolve_pointer(&doc, "/any"), None);
}

#[test]
fn pointer_empty_returns_whole_doc() {
    let doc = json!({"a": 1});
    assert_eq!(resolve_pointer(&doc, ""), Some(&doc));
}

#[test]
fn pointer_with_rfc6901_escapes() {
    let doc = json!({"a/b": {"c~d": 42}});
    assert_eq!(resolve_pointer(&doc, "/a~1b/c~0d"), Some(&json!(42)));
}

// ── Gate with unreachable thresholds ──────────────────────────────────

#[test]
fn gate_unreachable_threshold_always_passes() {
    let receipt = json!({"tokens": 100});
    let policy = make_policy(vec![make_rule(
        "huge_limit",
        "/tokens",
        RuleOperator::Lte,
        json!(999_999_999),
    )]);
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed);
    assert_eq!(result.errors, 0);
}

#[test]
fn gate_impossible_threshold_always_fails() {
    let receipt = json!({"tokens": 100});
    let policy = make_policy(vec![make_rule(
        "impossible",
        "/tokens",
        RuleOperator::Lt,
        json!(0),
    )]);
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
}

// ── Empty rule set ────────────────────────────────────────────────────

#[test]
fn gate_empty_rules_passes() {
    let receipt = json!({"tokens": 100});
    let policy = make_policy(vec![]);
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed);
    assert_eq!(result.errors, 0);
    assert_eq!(result.warnings, 0);
}

#[test]
fn gate_result_from_empty_vec() {
    let result = GateResult::from_results(vec![]);
    assert!(result.passed);
    assert_eq!(result.errors, 0);
    assert_eq!(result.warnings, 0);
    assert!(result.rule_results.is_empty());
}

#[test]
fn ratchet_gate_result_from_empty_vec() {
    let result = RatchetGateResult::from_results(vec![]);
    assert!(result.passed);
    assert_eq!(result.errors, 0);
    assert_eq!(result.warnings, 0);
}

// ── Conflicting / multiple rules ──────────────────────────────────────

#[test]
fn gate_conflicting_rules_both_evaluated() {
    let receipt = json!({"tokens": 500});
    let policy = make_policy(vec![
        make_rule("must_be_low", "/tokens", RuleOperator::Lt, json!(100)),
        make_rule("must_be_high", "/tokens", RuleOperator::Gt, json!(1000)),
    ]);
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
    assert_eq!(result.errors, 2);
}

#[test]
fn gate_mix_of_pass_and_fail() {
    let receipt = json!({"tokens": 500, "files": 10});
    let policy = make_policy(vec![
        make_rule("tokens_ok", "/tokens", RuleOperator::Lte, json!(1000)),
        make_rule("files_fail", "/files", RuleOperator::Gt, json!(100)),
    ]);
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
}

#[test]
fn gate_warns_do_not_fail() {
    let receipt = json!({"tokens": 2000});
    let policy = PolicyConfig {
        rules: vec![PolicyRule {
            name: "soft_limit".into(),
            pointer: "/tokens".into(),
            op: RuleOperator::Lte,
            value: Some(json!(1000)),
            values: None,
            negate: false,
            level: RuleLevel::Warn,
            message: None,
        }],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed);
    assert_eq!(result.warnings, 1);
    assert_eq!(result.errors, 0);
}

// ── fail_fast behavior ────────────────────────────────────────────────

#[test]
fn gate_fail_fast_stops_on_first_error() {
    let receipt = json!({"a": 100, "b": 200});
    let policy = PolicyConfig {
        rules: vec![
            make_rule("a_fail", "/a", RuleOperator::Lt, json!(50)),
            make_rule("b_fail", "/b", RuleOperator::Lt, json!(50)),
        ],
        fail_fast: true,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
    // Only 1 rule evaluated because fail_fast stopped after the first error
    assert_eq!(result.rule_results.len(), 1);
    assert_eq!(result.errors, 1);
}

// ── allow_missing behavior ────────────────────────────────────────────

#[test]
fn gate_missing_pointer_fails_when_not_allowed() {
    let receipt = json!({"tokens": 100});
    let policy = PolicyConfig {
        rules: vec![make_rule(
            "missing",
            "/nonexistent",
            RuleOperator::Lte,
            json!(100),
        )],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
}

#[test]
fn gate_missing_pointer_passes_when_allowed() {
    let receipt = json!({"tokens": 100});
    let policy = PolicyConfig {
        rules: vec![make_rule(
            "missing_ok",
            "/nonexistent",
            RuleOperator::Lte,
            json!(100),
        )],
        fail_fast: false,
        allow_missing: true,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed);
    assert_eq!(result.errors, 0);
}

// ── Negate behavior ──────────────────────────────────────────────────

#[test]
fn gate_negate_inverts_result() {
    let receipt = json!({"tokens": 100});
    let policy = make_policy(vec![PolicyRule {
        name: "negate_test".into(),
        pointer: "/tokens".into(),
        op: RuleOperator::Gt,
        value: Some(json!(1000)),
        negate: true, // 100 > 1000 is false → negated → true
        values: None,
        level: RuleLevel::Error,
        message: None,
    }]);
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed);
}

// ── Exists operator ──────────────────────────────────────────────────

#[test]
fn gate_exists_operator_passes_when_present() {
    let receipt = json!({"license": "MIT"});
    let policy = make_policy(vec![PolicyRule {
        name: "has_license".into(),
        pointer: "/license".into(),
        op: RuleOperator::Exists,
        value: None,
        values: None,
        negate: false,
        level: RuleLevel::Error,
        message: None,
    }]);
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed);
}

#[test]
fn gate_exists_operator_fails_when_absent() {
    let receipt = json!({"tokens": 100});
    let policy = make_policy(vec![PolicyRule {
        name: "has_license".into(),
        pointer: "/license".into(),
        op: RuleOperator::Exists,
        value: None,
        values: None,
        negate: false,
        level: RuleLevel::Error,
        message: None,
    }]);
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
}

// ── Contains operator ────────────────────────────────────────────────

#[test]
fn gate_contains_string() {
    let receipt = json!({"name": "hello world"});
    let policy = make_policy(vec![PolicyRule {
        name: "contains_test".into(),
        pointer: "/name".into(),
        op: RuleOperator::Contains,
        value: Some(json!("world")),
        values: None,
        negate: false,
        level: RuleLevel::Error,
        message: None,
    }]);
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed);
}

// ── Type mismatch in comparisons ─────────────────────────────────────

#[test]
fn gate_numeric_op_on_string_fails() {
    let receipt = json!({"name": "hello"});
    let policy = make_policy(vec![make_rule(
        "bad_type",
        "/name",
        RuleOperator::Gt,
        json!(100),
    )]);
    let result = evaluate_policy(&receipt, &policy);
    // String can't be compared numerically → fails
    assert!(!result.passed);
}

// ── PolicyConfig parsing edge cases ──────────────────────────────────

#[test]
fn policy_config_from_toml_empty() {
    let policy = PolicyConfig::from_toml("").unwrap();
    assert!(policy.rules.is_empty());
    assert!(!policy.fail_fast);
    assert!(!policy.allow_missing);
}

#[test]
fn policy_config_from_toml_invalid() {
    let result = PolicyConfig::from_toml("this is not valid toml {{{");
    assert!(result.is_err());
}

#[test]
fn policy_config_from_file_nonexistent() {
    let result = PolicyConfig::from_file(std::path::Path::new("/nonexistent/w54_policy.toml"));
    assert!(result.is_err());
}

// ── Ratchet edge cases ───────────────────────────────────────────────

#[test]
fn ratchet_missing_current_value_fails() {
    let baseline = json!({"complexity": 10.0});
    let current = json!({}); // missing complexity
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
fn ratchet_missing_baseline_with_max_value_only() {
    let baseline = json!({}); // missing
    let current = json!({"complexity": 5.0});
    let config = RatchetConfig {
        rules: vec![RatchetRule {
            pointer: "/complexity".into(),
            max_increase_pct: None,
            max_value: Some(10.0),
            level: RuleLevel::Error,
            description: None,
        }],
        fail_fast: false,
        allow_missing_baseline: true,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    // max_value=10, current=5 → passes
    assert!(result.passed);
}

#[test]
fn ratchet_exceeds_max_value_ceiling() {
    let baseline = json!({"complexity": 5.0});
    let current = json!({"complexity": 20.0});
    let config = RatchetConfig {
        rules: vec![RatchetRule {
            pointer: "/complexity".into(),
            max_increase_pct: Some(500.0), // very lenient percentage
            max_value: Some(15.0),         // but hard ceiling at 15
            level: RuleLevel::Error,
            description: None,
        }],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(!result.passed, "should fail due to max_value ceiling");
}

#[test]
fn ratchet_empty_rules_passes() {
    let config = RatchetConfig {
        rules: vec![],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &json!({}), &json!({}));
    assert!(result.passed);
    assert_eq!(result.errors, 0);
}

#[test]
fn ratchet_zero_baseline_to_nonzero_current() {
    let baseline = json!({"val": 0.0});
    let current = json!({"val": 5.0});
    let config = RatchetConfig {
        rules: vec![RatchetRule {
            pointer: "/val".into(),
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
    // 0 → 5 = infinity% increase → should fail
    assert!(!result.passed);
}

// ── GateResult determinism ──────────────────────────────────────────

#[test]
fn gate_result_deterministic_counts() {
    let results = vec![
        RuleResult {
            name: "r1".into(),
            passed: true,
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
            expected: "x".into(),
            message: Some("fail".into()),
        },
        RuleResult {
            name: "r3".into(),
            passed: false,
            level: RuleLevel::Warn,
            actual: None,
            expected: "x".into(),
            message: Some("warn".into()),
        },
    ];

    let gate = GateResult::from_results(results);
    assert!(!gate.passed);
    assert_eq!(gate.errors, 1);
    assert_eq!(gate.warnings, 1);
    assert_eq!(gate.rule_results.len(), 3);
}

// ── RuleOperator Display ──────────────────────────────────────────────

#[test]
fn all_operators_have_display() {
    let ops = [
        (RuleOperator::Gt, ">"),
        (RuleOperator::Gte, ">="),
        (RuleOperator::Lt, "<"),
        (RuleOperator::Lte, "<="),
        (RuleOperator::Eq, "=="),
        (RuleOperator::Ne, "!="),
        (RuleOperator::In, "in"),
        (RuleOperator::Contains, "contains"),
        (RuleOperator::Exists, "exists"),
    ];
    for (op, expected) in ops {
        assert_eq!(op.to_string(), expected, "failed for {op:?}");
    }
}

// ── RuleOperator / RuleLevel defaults ─────────────────────────────────

#[test]
fn rule_operator_default_is_eq() {
    assert_eq!(RuleOperator::default(), RuleOperator::Eq);
}

#[test]
fn rule_level_default_is_error() {
    assert_eq!(RuleLevel::default(), RuleLevel::Error);
}
