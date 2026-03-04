//! Error boundary tests for tokmd-gate.
//!
//! Tests invalid JSON pointer syntax, type mismatches, empty/missing policies,
//! deeply nested pointers, and ratchet edge cases.

use serde_json::json;
use tokmd_gate::{
    PolicyConfig, PolicyRule, RatchetConfig, RatchetRule, RuleLevel, RuleOperator, evaluate_policy,
    evaluate_ratchet_policy, resolve_pointer,
};

// ── Invalid JSON pointer syntax ──────────────────────────────────────

#[test]
fn pointer_without_leading_slash_returns_none() {
    let doc = json!({"foo": 1});
    assert_eq!(resolve_pointer(&doc, "foo"), None);
}

#[test]
fn pointer_with_trailing_slash_resolves_empty_key() {
    let doc = json!({"foo": {"": 42}});
    assert_eq!(resolve_pointer(&doc, "/foo/"), Some(&json!(42)));
}

#[test]
fn pointer_into_scalar_returns_none() {
    let doc = json!({"n": 42});
    assert_eq!(resolve_pointer(&doc, "/n/deeper"), None);
}

#[test]
fn pointer_array_index_out_of_bounds() {
    let doc = json!({"items": [1, 2, 3]});
    assert_eq!(resolve_pointer(&doc, "/items/99"), None);
}

#[test]
fn pointer_negative_array_index_returns_none() {
    let doc = json!({"items": [1, 2, 3]});
    // "-1" is not a valid array index per RFC 6901
    assert_eq!(resolve_pointer(&doc, "/items/-1"), None);
}

#[test]
fn pointer_non_numeric_array_index_returns_none() {
    let doc = json!({"items": [1, 2, 3]});
    assert_eq!(resolve_pointer(&doc, "/items/abc"), None);
}

// ── Threshold with wrong type ────────────────────────────────────────

#[test]
fn numeric_comparison_on_boolean_value_fails() {
    let receipt = json!({"flag": true});
    let policy = PolicyConfig {
        rules: vec![PolicyRule {
            name: "bool_gt".into(),
            pointer: "/flag".into(),
            op: RuleOperator::Gt,
            value: Some(json!(10)),
            values: None,
            negate: false,
            level: RuleLevel::Error,
            message: None,
        }],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed, "numeric comparison on bool should fail");
}

#[test]
fn numeric_comparison_on_null_value_fails() {
    let receipt = json!({"val": null});
    let policy = PolicyConfig {
        rules: vec![PolicyRule {
            name: "null_lt".into(),
            pointer: "/val".into(),
            op: RuleOperator::Lt,
            value: Some(json!(100)),
            values: None,
            negate: false,
            level: RuleLevel::Error,
            message: None,
        }],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed, "numeric comparison on null should fail");
}

#[test]
fn numeric_comparison_on_array_value_fails() {
    let receipt = json!({"arr": [1, 2, 3]});
    let policy = PolicyConfig {
        rules: vec![PolicyRule {
            name: "arr_gte".into(),
            pointer: "/arr".into(),
            op: RuleOperator::Gte,
            value: Some(json!(5)),
            values: None,
            negate: false,
            level: RuleLevel::Error,
            message: None,
        }],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed, "numeric comparison on array should fail");
}

// ── Empty policy ─────────────────────────────────────────────────────

#[test]
fn empty_toml_string_parses_to_default() {
    let policy = PolicyConfig::from_toml("").unwrap();
    assert!(policy.rules.is_empty());
    assert!(!policy.fail_fast);
    assert!(!policy.allow_missing);
}

#[test]
fn malformed_toml_returns_error() {
    let result = PolicyConfig::from_toml("[[[bad toml");
    assert!(result.is_err());
}

// ── Policy with no rules ─────────────────────────────────────────────

#[test]
fn policy_with_no_rules_passes() {
    let receipt = json!({"anything": 42});
    let policy = PolicyConfig {
        rules: vec![],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed);
    assert_eq!(result.errors, 0);
    assert_eq!(result.warnings, 0);
}

#[test]
fn ratchet_with_no_rules_passes() {
    let config = RatchetConfig {
        rules: vec![],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &json!({}), &json!({}));
    assert!(result.passed);
}

// ── Deep nested pointer ──────────────────────────────────────────────

#[test]
fn deeply_nested_pointer_resolves() {
    let mut doc = json!(42);
    // Build 20-level nesting
    for _ in 0..20 {
        doc = json!({"nested": doc});
    }
    let pointer = "/nested".repeat(20);
    assert_eq!(resolve_pointer(&doc, &pointer), Some(&json!(42)));
}

#[test]
fn deeply_nested_pointer_missing_leaf() {
    let mut doc = json!(42);
    for _ in 0..10 {
        doc = json!({"nested": doc});
    }
    let mut pointer = "/nested".repeat(10);
    pointer.push_str("/missing");
    assert_eq!(resolve_pointer(&doc, &pointer), None);
}

// ── Missing value handling ───────────────────────────────────────────

#[test]
fn missing_pointer_with_allow_missing_passes() {
    let receipt = json!({"a": 1});
    let policy = PolicyConfig {
        rules: vec![PolicyRule {
            name: "check".into(),
            pointer: "/nonexistent".into(),
            op: RuleOperator::Lte,
            value: Some(json!(100)),
            values: None,
            negate: false,
            level: RuleLevel::Error,
            message: None,
        }],
        fail_fast: false,
        allow_missing: true,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed);
}

#[test]
fn missing_pointer_without_allow_missing_fails() {
    let receipt = json!({"a": 1});
    let policy = PolicyConfig {
        rules: vec![PolicyRule {
            name: "check".into(),
            pointer: "/nonexistent".into(),
            op: RuleOperator::Lte,
            value: Some(json!(100)),
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
    assert!(
        result.rule_results[0]
            .message
            .as_ref()
            .unwrap()
            .contains("not found")
    );
}

// ── In operator without values ───────────────────────────────────────

#[test]
fn in_operator_with_no_values_list_fails() {
    let receipt = json!({"lang": "Rust"});
    let policy = PolicyConfig {
        rules: vec![PolicyRule {
            name: "in_no_list".into(),
            pointer: "/lang".into(),
            op: RuleOperator::In,
            value: None,
            values: None, // No values list provided
            negate: false,
            level: RuleLevel::Error,
            message: None,
        }],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed, "in operator with no values should fail");
}

// ── Contains on non-container ────────────────────────────────────────

#[test]
fn contains_on_number_fails() {
    let receipt = json!({"count": 42});
    let policy = PolicyConfig {
        rules: vec![PolicyRule {
            name: "contains_num".into(),
            pointer: "/count".into(),
            op: RuleOperator::Contains,
            value: Some(json!(4)),
            values: None,
            negate: false,
            level: RuleLevel::Error,
            message: None,
        }],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(
        !result.passed,
        "contains on a number should fail (not a string/array)"
    );
}

// ── Ratchet with non-numeric values ──────────────────────────────────

#[test]
fn ratchet_non_numeric_current_value_fails() {
    let baseline = json!({"metric": 10.0});
    let current = json!({"metric": "not a number"});
    let config = RatchetConfig {
        rules: vec![RatchetRule {
            pointer: "/metric".into(),
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
    // "not a number" can't be parsed as f64, so this should work like a missing value
    // Actually "not a number" is not a valid f64, so current_value extraction fails
    assert!(!result.passed);
}

// ── Policy file loading errors ───────────────────────────────────────

#[test]
fn policy_from_nonexistent_file_returns_io_error() {
    let path = std::path::Path::new("/tmp/tokmd-gate-no-such-policy-file.toml");
    let result = PolicyConfig::from_file(path);
    assert!(result.is_err());
}

#[test]
fn ratchet_from_nonexistent_file_returns_io_error() {
    let path = std::path::Path::new("/tmp/tokmd-gate-no-such-ratchet-file.toml");
    let result = RatchetConfig::from_file(path);
    assert!(result.is_err());
}

// ── Escaped pointer tokens ───────────────────────────────────────────

#[test]
fn pointer_with_tilde_escape_sequences() {
    let doc = json!({"a/b": {"c~d": 99}});
    // ~1 -> /, ~0 -> ~
    assert_eq!(resolve_pointer(&doc, "/a~1b/c~0d"), Some(&json!(99)));
    // Without escaping, should not find
    assert_eq!(resolve_pointer(&doc, "/a/b/c~d"), None);
}
