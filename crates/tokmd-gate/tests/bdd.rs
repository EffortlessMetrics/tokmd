//! BDD-style scenario tests for tokmd-gate.
//!
//! Each test reads as a Given/When/Then scenario covering policy rules,
//! JSON pointer matching, threshold evaluation, and ratchet rules.

use serde_json::{json, Value};
use tokmd_gate::{
    evaluate_policy, resolve_pointer, PolicyConfig, PolicyRule, RuleLevel, RuleOperator,
};

// ============================================================================
// Helpers
// ============================================================================

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

fn eval(receipt: &Value, rules: Vec<PolicyRule>) -> tokmd_gate::GateResult {
    evaluate_policy(receipt, &policy(rules))
}

// ============================================================================
// Scenario: JSON Pointer resolution
// ============================================================================

#[test]
fn given_deeply_nested_object_when_pointer_resolves_then_returns_leaf() {
    // Given a receipt with 4-level nesting
    let receipt = json!({
        "a": { "b": { "c": { "d": 99 } } }
    });
    // When resolving a deep pointer
    let result = resolve_pointer(&receipt, "/a/b/c/d");
    // Then the leaf value is returned
    assert_eq!(result, Some(&json!(99)));
}

#[test]
fn given_null_value_when_pointer_resolves_then_returns_null() {
    let receipt = json!({"key": null});
    let result = resolve_pointer(&receipt, "/key");
    assert_eq!(result, Some(&Value::Null));
}

#[test]
fn given_empty_string_key_when_pointer_uses_slash_then_resolves() {
    // RFC 6901: "/" points to the empty-string key
    let receipt = json!({"": "empty_key_value"});
    let result = resolve_pointer(&receipt, "/");
    assert_eq!(result, Some(&json!("empty_key_value")));
}

#[test]
fn given_mixed_array_and_object_when_pointer_navigates_then_resolves() {
    let receipt = json!({
        "languages": [
            {"name": "Rust", "loc": 5000},
            {"name": "Python", "loc": 2000}
        ]
    });
    // Navigate into array, then into object
    assert_eq!(
        resolve_pointer(&receipt, "/languages/0/name"),
        Some(&json!("Rust"))
    );
    assert_eq!(
        resolve_pointer(&receipt, "/languages/1/loc"),
        Some(&json!(2000))
    );
}

#[test]
fn given_boolean_value_when_pointer_resolves_then_returns_bool() {
    let receipt = json!({"flags": {"ci": true, "draft": false}});
    assert_eq!(
        resolve_pointer(&receipt, "/flags/ci"),
        Some(&json!(true))
    );
    assert_eq!(
        resolve_pointer(&receipt, "/flags/draft"),
        Some(&json!(false))
    );
}

#[test]
fn given_non_numeric_index_for_array_when_pointer_resolves_then_returns_none() {
    let receipt = json!({"items": [1, 2, 3]});
    assert_eq!(resolve_pointer(&receipt, "/items/abc"), None);
}

// ============================================================================
// Scenario: Threshold evaluation — numeric operators
// ============================================================================

#[test]
fn given_token_count_below_max_when_lte_rule_evaluated_then_passes() {
    let receipt = json!({"derived": {"totals": {"tokens": 250_000}}});
    let result = eval(
        &receipt,
        vec![rule(
            "max_tokens",
            "/derived/totals/tokens",
            RuleOperator::Lte,
            json!(500_000),
        )],
    );
    assert!(result.passed);
    assert_eq!(result.errors, 0);
}

#[test]
fn given_token_count_above_max_when_lte_rule_evaluated_then_fails() {
    let receipt = json!({"derived": {"totals": {"tokens": 750_000}}});
    let result = eval(
        &receipt,
        vec![rule(
            "max_tokens",
            "/derived/totals/tokens",
            RuleOperator::Lte,
            json!(500_000),
        )],
    );
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
}

#[test]
fn given_code_lines_at_minimum_when_gte_rule_evaluated_then_passes() {
    let receipt = json!({"code": 100});
    let result = eval(
        &receipt,
        vec![rule("min_code", "/code", RuleOperator::Gte, json!(100))],
    );
    assert!(result.passed);
}

#[test]
fn given_code_lines_below_minimum_when_gte_rule_evaluated_then_fails() {
    let receipt = json!({"code": 50});
    let result = eval(
        &receipt,
        vec![rule("min_code", "/code", RuleOperator::Gte, json!(100))],
    );
    assert!(!result.passed);
}

#[test]
fn given_float_metric_when_gt_boundary_tested_then_strict_comparison() {
    let receipt = json!({"density": 0.75});
    // Exact boundary: 0.75 > 0.75 should fail
    let result = eval(
        &receipt,
        vec![rule(
            "density_gt",
            "/density",
            RuleOperator::Gt,
            json!(0.75),
        )],
    );
    assert!(!result.passed);

    // Just above threshold
    let receipt2 = json!({"density": 0.76});
    let result2 = eval(
        &receipt2,
        vec![rule(
            "density_gt",
            "/density",
            RuleOperator::Gt,
            json!(0.75),
        )],
    );
    assert!(result2.passed);
}

#[test]
fn given_negative_values_when_compared_then_ordering_is_correct() {
    let receipt = json!({"delta": -5});
    assert!(eval(
        &receipt,
        vec![rule("lt_zero", "/delta", RuleOperator::Lt, json!(0))]
    )
    .passed);
    assert!(!eval(
        &receipt,
        vec![rule("gt_zero", "/delta", RuleOperator::Gt, json!(0))]
    )
    .passed);
}

// ============================================================================
// Scenario: String and equality operators
// ============================================================================

#[test]
fn given_string_value_when_eq_matches_then_passes() {
    let receipt = json!({"license": "MIT"});
    let result = eval(
        &receipt,
        vec![rule(
            "license_eq",
            "/license",
            RuleOperator::Eq,
            json!("MIT"),
        )],
    );
    assert!(result.passed);
}

#[test]
fn given_string_value_when_ne_matches_different_then_passes() {
    let receipt = json!({"license": "MIT"});
    let result = eval(
        &receipt,
        vec![rule(
            "not_gpl",
            "/license",
            RuleOperator::Ne,
            json!("GPL-3.0"),
        )],
    );
    assert!(result.passed);
}

#[test]
fn given_string_value_when_ne_matches_same_then_fails() {
    let receipt = json!({"license": "MIT"});
    let result = eval(
        &receipt,
        vec![rule(
            "not_mit",
            "/license",
            RuleOperator::Ne,
            json!("MIT"),
        )],
    );
    assert!(!result.passed);
}

#[test]
fn given_boolean_value_when_eq_checks_then_works() {
    let receipt = json!({"published": true});
    let result = eval(
        &receipt,
        vec![rule(
            "is_published",
            "/published",
            RuleOperator::Eq,
            json!(true),
        )],
    );
    assert!(result.passed);

    let result2 = eval(
        &receipt,
        vec![rule(
            "not_published",
            "/published",
            RuleOperator::Eq,
            json!(false),
        )],
    );
    assert!(!result2.passed);
}

// ============================================================================
// Scenario: "in" operator
// ============================================================================

#[test]
fn given_license_in_approved_list_when_in_rule_evaluated_then_passes() {
    let receipt = json!({"license": "Apache-2.0"});
    let r = PolicyRule {
        name: "approved_license".into(),
        pointer: "/license".into(),
        op: RuleOperator::In,
        value: None,
        values: Some(vec![json!("MIT"), json!("Apache-2.0"), json!("BSD-3-Clause")]),
        negate: false,
        level: RuleLevel::Error,
        message: Some("License not in approved list".into()),
    };
    let result = eval(&receipt, vec![r]);
    assert!(result.passed);
}

#[test]
fn given_license_not_in_approved_list_when_in_rule_evaluated_then_fails_with_message() {
    let receipt = json!({"license": "AGPL-3.0"});
    let r = PolicyRule {
        name: "approved_license".into(),
        pointer: "/license".into(),
        op: RuleOperator::In,
        value: None,
        values: Some(vec![json!("MIT"), json!("Apache-2.0")]),
        negate: false,
        level: RuleLevel::Error,
        message: Some("License not in approved list".into()),
    };
    let result = eval(&receipt, vec![r]);
    assert!(!result.passed);
    let rule_result = &result.rule_results[0];
    assert_eq!(
        rule_result.message.as_deref(),
        Some("License not in approved list")
    );
}

#[test]
fn given_numeric_value_when_in_list_of_numbers_then_matches() {
    let receipt = json!({"tier": 2});
    let r = PolicyRule {
        name: "valid_tier".into(),
        pointer: "/tier".into(),
        op: RuleOperator::In,
        value: None,
        values: Some(vec![json!(1), json!(2), json!(3)]),
        negate: false,
        level: RuleLevel::Error,
        message: None,
    };
    let result = eval(&receipt, vec![r]);
    assert!(result.passed);
}

// ============================================================================
// Scenario: "contains" operator
// ============================================================================

#[test]
fn given_string_field_when_contains_substring_then_passes() {
    let receipt = json!({"description": "A high-performance CLI tool"});
    let result = eval(
        &receipt,
        vec![rule(
            "has_cli",
            "/description",
            RuleOperator::Contains,
            json!("CLI"),
        )],
    );
    assert!(result.passed);
}

#[test]
fn given_array_field_when_contains_element_then_passes() {
    let receipt = json!({"tags": ["rust", "cli", "analysis"]});
    let result = eval(
        &receipt,
        vec![rule(
            "has_rust_tag",
            "/tags",
            RuleOperator::Contains,
            json!("rust"),
        )],
    );
    assert!(result.passed);
}

#[test]
fn given_array_field_when_not_contains_element_then_fails() {
    let receipt = json!({"tags": ["rust", "cli"]});
    let result = eval(
        &receipt,
        vec![rule(
            "has_python",
            "/tags",
            RuleOperator::Contains,
            json!("python"),
        )],
    );
    assert!(!result.passed);
}

#[test]
fn given_numeric_field_when_contains_checked_then_fails() {
    // Contains on a non-string/non-array should fail
    let receipt = json!({"count": 42});
    let result = eval(
        &receipt,
        vec![rule(
            "contains_num",
            "/count",
            RuleOperator::Contains,
            json!(4),
        )],
    );
    assert!(!result.passed);
}

// ============================================================================
// Scenario: "exists" operator
// ============================================================================

#[test]
fn given_present_field_when_exists_rule_then_passes() {
    let receipt = json!({"metadata": {"version": "1.0"}});
    let r = PolicyRule {
        name: "has_version".into(),
        pointer: "/metadata/version".into(),
        op: RuleOperator::Exists,
        value: None,
        values: None,
        negate: false,
        level: RuleLevel::Error,
        message: None,
    };
    let result = eval(&receipt, vec![r]);
    assert!(result.passed);
}

#[test]
fn given_absent_field_when_negated_exists_rule_then_passes() {
    let receipt = json!({"metadata": {}});
    let r = PolicyRule {
        name: "no_secrets".into(),
        pointer: "/metadata/secrets".into(),
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
fn given_present_field_when_negated_exists_rule_then_fails() {
    let receipt = json!({"secrets": "hunter2"});
    let r = PolicyRule {
        name: "no_secrets".into(),
        pointer: "/secrets".into(),
        op: RuleOperator::Exists,
        value: None,
        values: None,
        negate: true,
        level: RuleLevel::Error,
        message: Some("Secrets should not be present".into()),
    };
    let result = eval(&receipt, vec![r]);
    assert!(!result.passed);
    assert_eq!(
        result.rule_results[0].message.as_deref(),
        Some("Secrets should not be present")
    );
}

// ============================================================================
// Scenario: Negate modifier
// ============================================================================

#[test]
fn given_negate_with_lte_when_value_is_within_then_fails() {
    // negate + lte: "assert NOT (tokens <= 500k)" i.e. tokens must exceed 500k
    let receipt = json!({"tokens": 100_000});
    let r = PolicyRule {
        negate: true,
        ..rule("must_exceed", "/tokens", RuleOperator::Lte, json!(500_000))
    };
    let result = eval(&receipt, vec![r]);
    assert!(!result.passed); // 100k <= 500k is true, negated = false
}

#[test]
fn given_negate_with_contains_when_element_absent_then_passes() {
    let receipt = json!({"tags": ["rust", "cli"]});
    let r = PolicyRule {
        negate: true,
        ..rule("no_python", "/tags", RuleOperator::Contains, json!("python"))
    };
    let result = eval(&receipt, vec![r]);
    assert!(result.passed); // "python" not in tags → false, negated = true
}

// ============================================================================
// Scenario: Warn vs Error levels
// ============================================================================

#[test]
fn given_failing_warn_rule_when_evaluated_then_gate_still_passes() {
    let receipt = json!({"complexity": 15.0});
    let r = PolicyRule {
        level: RuleLevel::Warn,
        message: Some("Complexity is high".into()),
        ..rule("warn_complexity", "/complexity", RuleOperator::Lte, json!(10.0))
    };
    let result = eval(&receipt, vec![r]);
    assert!(result.passed); // Warnings don't fail the gate
    assert_eq!(result.warnings, 1);
    assert_eq!(result.errors, 0);
}

#[test]
fn given_failing_error_rule_when_evaluated_then_gate_fails() {
    let receipt = json!({"complexity": 15.0});
    let r = PolicyRule {
        level: RuleLevel::Error,
        ..rule("err_complexity", "/complexity", RuleOperator::Lte, json!(10.0))
    };
    let result = eval(&receipt, vec![r]);
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
}

#[test]
fn given_mixed_warn_and_error_when_only_warn_fails_then_gate_passes() {
    let receipt = json!({"tokens": 400_000, "complexity": 15.0});
    let rules = vec![
        rule("max_tokens", "/tokens", RuleOperator::Lte, json!(500_000)), // passes
        PolicyRule {
            level: RuleLevel::Warn,
            ..rule("warn_complexity", "/complexity", RuleOperator::Lte, json!(10.0))
        }, // fails as warning
    ];
    let result = eval(&receipt, rules);
    assert!(result.passed);
    assert_eq!(result.warnings, 1);
    assert_eq!(result.errors, 0);
}

// ============================================================================
// Scenario: Missing values and allow_missing
// ============================================================================

#[test]
fn given_missing_pointer_when_allow_missing_false_then_fails() {
    let receipt = json!({"foo": 1});
    let result = evaluate_policy(
        &receipt,
        &PolicyConfig {
            rules: vec![rule("check", "/missing", RuleOperator::Eq, json!(1))],
            fail_fast: false,
            allow_missing: false,
        },
    );
    assert!(!result.passed);
    assert!(result.rule_results[0]
        .message
        .as_ref()
        .unwrap()
        .contains("not found"));
}

#[test]
fn given_missing_pointer_when_allow_missing_true_then_passes() {
    let receipt = json!({"foo": 1});
    let result = evaluate_policy(
        &receipt,
        &PolicyConfig {
            rules: vec![rule("check", "/missing", RuleOperator::Eq, json!(1))],
            fail_fast: false,
            allow_missing: true,
        },
    );
    assert!(result.passed);
}

// ============================================================================
// Scenario: fail_fast behavior
// ============================================================================

#[test]
fn given_fail_fast_when_first_error_rule_fails_then_stops_early() {
    let receipt = json!({"a": 100, "b": 200, "c": 300});
    let result = evaluate_policy(
        &receipt,
        &PolicyConfig {
            rules: vec![
                rule("a_check", "/a", RuleOperator::Lt, json!(50)),  // fails
                rule("b_check", "/b", RuleOperator::Lt, json!(50)),  // would fail
                rule("c_check", "/c", RuleOperator::Lt, json!(1000)), // would pass
            ],
            fail_fast: true,
            allow_missing: false,
        },
    );
    assert!(!result.passed);
    assert_eq!(result.rule_results.len(), 1);
}

#[test]
fn given_fail_fast_when_warn_rule_fails_then_does_not_stop() {
    let receipt = json!({"a": 100, "b": 200});
    let result = evaluate_policy(
        &receipt,
        &PolicyConfig {
            rules: vec![
                PolicyRule {
                    level: RuleLevel::Warn,
                    ..rule("warn_a", "/a", RuleOperator::Lt, json!(50))
                }, // fails but warn
                rule("b_check", "/b", RuleOperator::Lt, json!(50)), // fails error
            ],
            fail_fast: true,
            allow_missing: false,
        },
    );
    // Should have evaluated both rules: warn doesn't trigger fail_fast
    assert_eq!(result.rule_results.len(), 2);
    assert_eq!(result.warnings, 1);
    assert_eq!(result.errors, 1);
}

// ============================================================================
// Scenario: TOML policy parsing
// ============================================================================

#[test]
fn given_toml_with_all_operators_when_parsed_then_all_rules_load() {
    let toml = r#"
[[rules]]
name = "gt_check"
pointer = "/a"
op = "gt"
value = 10

[[rules]]
name = "gte_check"
pointer = "/b"
op = "gte"
value = 20

[[rules]]
name = "lt_check"
pointer = "/c"
op = "lt"
value = 30

[[rules]]
name = "lte_check"
pointer = "/d"
op = "lte"
value = 40

[[rules]]
name = "eq_check"
pointer = "/e"
op = "eq"
value = "hello"

[[rules]]
name = "ne_check"
pointer = "/f"
op = "ne"
value = "world"

[[rules]]
name = "in_check"
pointer = "/g"
op = "in"
values = ["a", "b", "c"]

[[rules]]
name = "contains_check"
pointer = "/h"
op = "contains"
value = "needle"

[[rules]]
name = "exists_check"
pointer = "/i"
op = "exists"
"#;
    let config = PolicyConfig::from_toml(toml).unwrap();
    assert_eq!(config.rules.len(), 9);
    assert_eq!(config.rules[0].op, RuleOperator::Gt);
    assert_eq!(config.rules[6].op, RuleOperator::In);
    assert!(config.rules[6].values.is_some());
    assert_eq!(config.rules[8].op, RuleOperator::Exists);
}

#[test]
fn given_toml_with_negate_and_level_when_parsed_then_fields_set() {
    let toml = r#"
[[rules]]
name = "negated_warn"
pointer = "/x"
op = "eq"
value = 42
negate = true
level = "warn"
message = "Custom message"
"#;
    let config = PolicyConfig::from_toml(toml).unwrap();
    let r = &config.rules[0];
    assert!(r.negate);
    assert_eq!(r.level, RuleLevel::Warn);
    assert_eq!(r.message.as_deref(), Some("Custom message"));
}

#[test]
fn given_invalid_toml_when_parsed_then_returns_error() {
    let bad_toml = "this is not valid { toml ]";
    assert!(PolicyConfig::from_toml(bad_toml).is_err());
}

// ============================================================================
// Scenario: Numeric string coercion
// ============================================================================

#[test]
fn given_string_number_when_compared_numerically_then_coerces() {
    let receipt = json!({"count": "42"});
    let result = eval(
        &receipt,
        vec![rule("str_gt", "/count", RuleOperator::Gt, json!(40))],
    );
    assert!(result.passed);
}

#[test]
fn given_string_number_vs_string_threshold_when_eq_then_compares_as_string() {
    // Two strings that look like numbers: should compare as strings
    let receipt = json!({"version": "2.0"});
    let result = eval(
        &receipt,
        vec![rule("ver_eq", "/version", RuleOperator::Eq, json!("2.0"))],
    );
    assert!(result.passed);
}

// ============================================================================
// Scenario: RuleResult actual/expected fields
// ============================================================================

#[test]
fn given_passing_rule_when_evaluated_then_actual_is_set_and_message_is_none() {
    let receipt = json!({"tokens": 100});
    let result = eval(
        &receipt,
        vec![rule("check", "/tokens", RuleOperator::Lte, json!(500))],
    );
    let rr = &result.rule_results[0];
    assert!(rr.passed);
    assert_eq!(rr.actual, Some(json!(100)));
    assert!(rr.message.is_none());
}

#[test]
fn given_failing_rule_with_custom_message_when_evaluated_then_message_is_set() {
    let receipt = json!({"tokens": 1000});
    let r = PolicyRule {
        message: Some("Token budget exceeded!".into()),
        ..rule("check", "/tokens", RuleOperator::Lte, json!(500))
    };
    let result = eval(&receipt, vec![r]);
    let rr = &result.rule_results[0];
    assert!(!rr.passed);
    assert_eq!(rr.message.as_deref(), Some("Token budget exceeded!"));
    assert_eq!(rr.actual, Some(json!(1000)));
}

// ============================================================================
// Scenario: Ratchet rules via TOML
// ============================================================================

#[test]
fn given_ratchet_toml_when_parsed_then_config_is_correct() {
    let toml = r#"
fail_fast = false
allow_missing_baseline = true
allow_missing_current = false

[[rules]]
pointer = "/complexity/avg"
max_increase_pct = 5.0
level = "error"
description = "Cyclomatic complexity"

[[rules]]
pointer = "/tokens"
max_value = 1000000.0
level = "warn"
"#;
    let config = tokmd_gate::RatchetConfig::from_toml(toml).unwrap();
    assert!(!config.fail_fast);
    assert!(config.allow_missing_baseline);
    assert!(!config.allow_missing_current);
    assert_eq!(config.rules.len(), 2);
    assert_eq!(config.rules[0].max_increase_pct, Some(5.0));
    assert_eq!(config.rules[1].max_value, Some(1_000_000.0));
    assert_eq!(config.rules[1].level, RuleLevel::Warn);
}

// ============================================================================
// Scenario: Ratchet evaluation
// ============================================================================

#[test]
fn given_metric_decreased_when_ratchet_evaluated_then_passes() {
    let baseline = json!({"complexity": 10.0});
    let current = json!({"complexity": 8.0});
    let config = tokmd_gate::RatchetConfig {
        rules: vec![tokmd_gate::RatchetRule {
            pointer: "/complexity".into(),
            max_increase_pct: Some(5.0),
            max_value: None,
            level: RuleLevel::Error,
            description: Some("Complexity must not increase".into()),
        }],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = tokmd_gate::evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(result.passed);
    assert_eq!(result.errors, 0);
}

#[test]
fn given_metric_exceeds_max_value_when_ratchet_evaluated_then_fails() {
    let baseline = json!({"tokens": 500});
    let current = json!({"tokens": 2000});
    let config = tokmd_gate::RatchetConfig {
        rules: vec![tokmd_gate::RatchetRule {
            pointer: "/tokens".into(),
            max_increase_pct: None,
            max_value: Some(1500.0),
            level: RuleLevel::Error,
            description: None,
        }],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = tokmd_gate::evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(!result.passed);
    assert!(result.ratchet_results[0]
        .message
        .contains("exceeds maximum"));
}

#[test]
fn given_metric_increase_over_pct_when_ratchet_evaluated_then_fails() {
    let baseline = json!({"loc": 1000});
    let current = json!({"loc": 1200}); // 20% increase
    let config = tokmd_gate::RatchetConfig {
        rules: vec![tokmd_gate::RatchetRule {
            pointer: "/loc".into(),
            max_increase_pct: Some(10.0),
            max_value: None,
            level: RuleLevel::Error,
            description: None,
        }],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = tokmd_gate::evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(!result.passed);
    assert!(result.ratchet_results[0].message.contains("20.00%"));
}

#[test]
fn given_ratchet_warn_level_when_fails_then_gate_still_passes() {
    let baseline = json!({"complexity": 10.0});
    let current = json!({"complexity": 20.0}); // 100% increase
    let config = tokmd_gate::RatchetConfig {
        rules: vec![tokmd_gate::RatchetRule {
            pointer: "/complexity".into(),
            max_increase_pct: Some(5.0),
            max_value: None,
            level: RuleLevel::Warn,
            description: None,
        }],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };
    let result = tokmd_gate::evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(result.passed); // Warn doesn't fail gate
    assert_eq!(result.warnings, 1);
}
