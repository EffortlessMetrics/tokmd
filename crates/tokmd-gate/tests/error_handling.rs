//! Error handling tests for tokmd-gate.
//!
//! Tests error paths for invalid JSON pointers, missing fields in receipts,
//! type mismatches in comparisons, malformed config, and empty rule sets.

use serde_json::json;
use tokmd_gate::{
    GateResult, PolicyConfig, PolicyRule, RatchetConfig, RatchetGateResult, RatchetRule, RuleLevel,
    RuleOperator, RuleResult, evaluate_policy, evaluate_ratchet_policy, resolve_pointer,
};

// =============================================================================
// Invalid JSON Pointer tests
// =============================================================================

#[test]
fn resolve_pointer_missing_leading_slash_returns_none() {
    let doc = json!({"foo": 1});
    assert_eq!(resolve_pointer(&doc, "foo"), None);
}

#[test]
fn resolve_pointer_into_scalar_returns_none() {
    let doc = json!({"foo": 42});
    // Trying to traverse into a scalar value
    assert_eq!(resolve_pointer(&doc, "/foo/bar"), None);
}

#[test]
fn resolve_pointer_invalid_array_index_returns_none() {
    let doc = json!({"arr": [1, 2, 3]});
    assert_eq!(resolve_pointer(&doc, "/arr/not_a_number"), None);
}

#[test]
fn resolve_pointer_out_of_bounds_array_index_returns_none() {
    let doc = json!({"arr": [1, 2, 3]});
    assert_eq!(resolve_pointer(&doc, "/arr/99"), None);
}

#[test]
fn resolve_pointer_deeply_nested_missing_returns_none() {
    let doc = json!({"a": {"b": {"c": 1}}});
    assert_eq!(resolve_pointer(&doc, "/a/b/c/d/e"), None);
}

#[test]
fn resolve_pointer_empty_key_segment() {
    // RFC 6901: "/" refers to the key "" (empty string)
    let doc = json!({"": "empty_key_value"});
    assert_eq!(resolve_pointer(&doc, "/"), Some(&json!("empty_key_value")));
}

// =============================================================================
// PolicyConfig parsing errors
// =============================================================================

#[test]
fn policy_config_malformed_toml_returns_error() {
    let bad = "this is not valid [[[toml";
    let result = PolicyConfig::from_toml(bad);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("TOML"));
}

#[test]
fn policy_config_wrong_type_for_fail_fast_returns_error() {
    let bad = r#"fail_fast = "yes""#;
    let result = PolicyConfig::from_toml(bad);
    assert!(result.is_err());
}

#[test]
fn policy_config_wrong_type_for_rules_returns_error() {
    let bad = r#"rules = "not-an-array""#;
    let result = PolicyConfig::from_toml(bad);
    assert!(result.is_err());
}

#[test]
fn policy_config_rule_missing_name_returns_error() {
    let bad = r#"
[[rules]]
pointer = "/tokens"
op = "lte"
value = 1000
"#;
    let result = PolicyConfig::from_toml(bad);
    assert!(result.is_err());
}

#[test]
fn policy_config_rule_missing_pointer_returns_error() {
    let bad = r#"
[[rules]]
name = "check"
op = "lte"
value = 1000
"#;
    let result = PolicyConfig::from_toml(bad);
    assert!(result.is_err());
}

#[test]
fn policy_config_rule_invalid_operator_returns_error() {
    let bad = r#"
[[rules]]
name = "check"
pointer = "/tokens"
op = "not_an_operator"
value = 1000
"#;
    let result = PolicyConfig::from_toml(bad);
    assert!(result.is_err());
}

#[test]
fn policy_config_from_file_nonexistent_returns_error() {
    let path = std::path::Path::new("/tmp/tokmd-errors-nonexistent-policy.toml");
    let result = PolicyConfig::from_file(path);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("read"));
}

#[test]
fn policy_config_empty_string_parses_to_defaults() {
    let config = PolicyConfig::from_toml("").unwrap();
    assert!(config.rules.is_empty());
    assert!(!config.fail_fast);
    assert!(!config.allow_missing);
}

// =============================================================================
// RatchetConfig parsing errors
// =============================================================================

#[test]
fn ratchet_config_malformed_toml_returns_error() {
    let bad = "this is {{{ not valid toml";
    let result = RatchetConfig::from_toml(bad);
    assert!(result.is_err());
}

#[test]
fn ratchet_config_rule_missing_pointer_returns_error() {
    let bad = r#"
[[rules]]
max_increase_pct = 10.0
"#;
    let result = RatchetConfig::from_toml(bad);
    assert!(result.is_err());
}

#[test]
fn ratchet_config_wrong_type_for_max_increase_pct_returns_error() {
    let bad = r#"
[[rules]]
pointer = "/complexity"
max_increase_pct = "ten percent"
"#;
    let result = RatchetConfig::from_toml(bad);
    assert!(result.is_err());
}

#[test]
fn ratchet_config_from_file_nonexistent_returns_error() {
    let path = std::path::Path::new("/tmp/tokmd-errors-nonexistent-ratchet.toml");
    let result = RatchetConfig::from_file(path);
    assert!(result.is_err());
}

#[test]
fn ratchet_config_empty_string_parses_to_defaults() {
    let config = RatchetConfig::from_toml("").unwrap();
    assert!(config.rules.is_empty());
    assert!(!config.fail_fast);
    assert!(!config.allow_missing_baseline);
    assert!(!config.allow_missing_current);
}

// =============================================================================
// Type mismatch in comparisons
// =============================================================================

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

#[test]
fn numeric_comparison_on_non_numeric_value_fails() {
    // Comparing a boolean with a numeric operator should fail
    let receipt = json!({"flag": true});
    let policy = PolicyConfig {
        rules: vec![make_rule("check", "/flag", RuleOperator::Gt, json!(0))],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
}

#[test]
fn numeric_comparison_on_object_value_fails() {
    let receipt = json!({"nested": {"a": 1}});
    let policy = PolicyConfig {
        rules: vec![make_rule("check", "/nested", RuleOperator::Lte, json!(100))],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
}

#[test]
fn numeric_comparison_on_array_value_fails() {
    let receipt = json!({"items": [1, 2, 3]});
    let policy = PolicyConfig {
        rules: vec![make_rule("check", "/items", RuleOperator::Lt, json!(10))],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
}

#[test]
fn numeric_comparison_on_null_value_fails() {
    let receipt = json!({"val": null});
    let policy = PolicyConfig {
        rules: vec![make_rule("check", "/val", RuleOperator::Gte, json!(0))],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
}

#[test]
fn contains_on_non_container_type_fails() {
    // contains on a number should fail
    let receipt = json!({"count": 42});
    let policy = PolicyConfig {
        rules: vec![make_rule(
            "check",
            "/count",
            RuleOperator::Contains,
            json!("4"),
        )],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
}

#[test]
fn in_operator_without_values_list_fails() {
    let receipt = json!({"lang": "Rust"});
    let rule = PolicyRule {
        name: "check".into(),
        pointer: "/lang".into(),
        op: RuleOperator::In,
        value: None,
        values: None, // No values list provided
        negate: false,
        level: RuleLevel::Error,
        message: None,
    };
    let policy = PolicyConfig {
        rules: vec![rule],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
}

// =============================================================================
// Missing fields in receipts
// =============================================================================

#[test]
fn missing_pointer_path_without_allow_missing_fails() {
    let receipt = json!({"a": 1});
    let policy = PolicyConfig {
        rules: vec![make_rule(
            "check",
            "/nonexistent/path",
            RuleOperator::Eq,
            json!(1),
        )],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
    // Verify the error message mentions the pointer
    let rule_result = &result.rule_results[0];
    assert!(!rule_result.passed);
    assert!(rule_result.actual.is_none());
    assert!(rule_result.message.as_ref().unwrap().contains("not found"));
}

#[test]
fn missing_pointer_path_with_allow_missing_passes() {
    let receipt = json!({"a": 1});
    let policy = PolicyConfig {
        rules: vec![make_rule(
            "check",
            "/nonexistent/path",
            RuleOperator::Eq,
            json!(1),
        )],
        fail_fast: false,
        allow_missing: true,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed);
}

// =============================================================================
// Empty rule sets
// =============================================================================

#[test]
fn empty_policy_rules_always_pass() {
    let receipt = json!({"anything": "here"});
    let policy = PolicyConfig {
        rules: vec![],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed);
    assert_eq!(result.errors, 0);
    assert_eq!(result.warnings, 0);
    assert!(result.rule_results.is_empty());
}

#[test]
fn empty_ratchet_rules_always_pass() {
    let baseline = json!({"a": 1});
    let current = json!({"a": 2});
    let config = RatchetConfig {
        rules: vec![],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(result.passed);
    assert_eq!(result.errors, 0);
    assert_eq!(result.warnings, 0);
    assert!(result.ratchet_results.is_empty());
}

// =============================================================================
// Ratchet evaluation error paths
// =============================================================================

#[test]
fn ratchet_non_numeric_current_value_fails() {
    let baseline = json!({"val": 10.0});
    let current = json!({"val": "not-a-number"});
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
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
    assert!(
        result.ratchet_results[0]
            .message
            .contains("not found or not numeric")
    );
}

#[test]
fn ratchet_non_numeric_baseline_with_pct_check_fails() {
    let baseline = json!({"val": "not-a-number"});
    let current = json!({"val": 10.0});
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
    assert!(!result.passed);
    assert!(
        result.ratchet_results[0]
            .message
            .contains("Baseline value not found")
    );
}

#[test]
fn ratchet_missing_current_pointer_fails_without_allow() {
    let baseline = json!({"val": 10.0});
    let current = json!({}); // pointer /val missing
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
    assert!(!result.passed);
}

#[test]
fn ratchet_missing_current_pointer_passes_with_allow() {
    let baseline = json!({"val": 10.0});
    let current = json!({});
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
        allow_missing_current: true,
    };
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(result.passed);
}

// =============================================================================
// GateResult / RatchetGateResult construction edge cases
// =============================================================================

#[test]
fn gate_result_mixed_errors_and_warnings() {
    let results = vec![
        RuleResult {
            name: "err1".into(),
            passed: false,
            level: RuleLevel::Error,
            actual: None,
            expected: "x".into(),
            message: Some("error".into()),
        },
        RuleResult {
            name: "warn1".into(),
            passed: false,
            level: RuleLevel::Warn,
            actual: None,
            expected: "x".into(),
            message: Some("warning".into()),
        },
        RuleResult {
            name: "pass1".into(),
            passed: true,
            level: RuleLevel::Error,
            actual: Some(json!(1)),
            expected: "x".into(),
            message: None,
        },
    ];
    let gate = GateResult::from_results(results);
    assert!(!gate.passed); // Has errors
    assert_eq!(gate.errors, 1);
    assert_eq!(gate.warnings, 1);
    assert_eq!(gate.rule_results.len(), 3);
}

#[test]
fn ratchet_gate_result_all_warnings_still_passes() {
    let results = vec![tokmd_gate::RatchetResult {
        rule: RatchetRule {
            pointer: "/x".into(),
            max_increase_pct: Some(5.0),
            max_value: None,
            level: RuleLevel::Warn,
            description: None,
        },
        passed: false,
        baseline_value: Some(10.0),
        current_value: 20.0,
        change_pct: Some(100.0),
        message: "regression".into(),
    }];
    let gate = RatchetGateResult::from_results(results);
    assert!(gate.passed); // Only warnings
    assert_eq!(gate.warnings, 1);
    assert_eq!(gate.errors, 0);
}

// =============================================================================
// Exists operator edge cases
// =============================================================================

#[test]
fn exists_on_null_value_still_exists() {
    // A key with null value exists in JSON
    let receipt = json!({"key": null});
    let rule = PolicyRule {
        name: "check".into(),
        pointer: "/key".into(),
        op: RuleOperator::Exists,
        value: None,
        values: None,
        negate: false,
        level: RuleLevel::Error,
        message: None,
    };
    let policy = PolicyConfig {
        rules: vec![rule],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed);
}

#[test]
fn not_exists_on_present_key_fails() {
    let receipt = json!({"key": "value"});
    let rule = PolicyRule {
        name: "check".into(),
        pointer: "/key".into(),
        op: RuleOperator::Exists,
        value: None,
        values: None,
        negate: true, // should NOT exist
        level: RuleLevel::Error,
        message: Some("key should not exist".into()),
    };
    let policy = PolicyConfig {
        rules: vec![rule],
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
            .contains("should not exist")
    );
}

// =============================================================================
// Comparison with missing expected value
// =============================================================================

#[test]
fn eq_without_value_fails() {
    let receipt = json!({"count": 42});
    let rule = PolicyRule {
        name: "check".into(),
        pointer: "/count".into(),
        op: RuleOperator::Eq,
        value: None, // No expected value
        values: None,
        negate: false,
        level: RuleLevel::Error,
        message: None,
    };
    let policy = PolicyConfig {
        rules: vec![rule],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
}

#[test]
fn lt_without_value_fails() {
    let receipt = json!({"count": 42});
    let rule = PolicyRule {
        name: "check".into(),
        pointer: "/count".into(),
        op: RuleOperator::Lt,
        value: None,
        values: None,
        negate: false,
        level: RuleLevel::Error,
        message: None,
    };
    let policy = PolicyConfig {
        rules: vec![rule],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
}

#[test]
fn contains_without_value_fails() {
    let receipt = json!({"text": "hello world"});
    let rule = PolicyRule {
        name: "check".into(),
        pointer: "/text".into(),
        op: RuleOperator::Contains,
        value: None,
        values: None,
        negate: false,
        level: RuleLevel::Error,
        message: None,
    };
    let policy = PolicyConfig {
        rules: vec![rule],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
}
