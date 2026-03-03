//! Edge-case tests for tokmd-gate.
//!
//! Focuses on areas not covered by existing test files:
//! - Operators with missing or invalid value fields
//! - Contains on non-container types
//! - GateError variant Display strings
//! - RatchetConfig from_file error paths
//! - RuleOperator and RuleLevel defaults
//! - Combined policy + ratchet evaluation scenarios
//! - PolicyConfig and RatchetConfig serialization roundtrips

use serde_json::json;
use tokmd_gate::{
    GateResult, PolicyConfig, PolicyRule, RatchetConfig, RatchetGateResult, RatchetRule, RuleLevel,
    RuleOperator, RuleResult, evaluate_policy, evaluate_ratchet_policy, resolve_pointer,
};

// =========================================================================
// Scenario: Operators with missing or invalid value fields
// =========================================================================

mod operator_edge_cases {
    use super::*;

    #[test]
    fn comparison_on_non_numeric_actual_fails_gracefully() {
        // Given a string value at the pointer
        let receipt = json!({"name": "hello"});
        let rule = make_rule("test", "/name", RuleOperator::Gt, json!(10));

        // When evaluated
        let result = evaluate_single(&receipt, &rule);

        // Then it fails (can't compare string > number)
        assert!(!result.passed);
    }

    #[test]
    fn eq_on_null_vs_null() {
        let receipt = json!({"val": null});
        let rule = make_rule("test", "/val", RuleOperator::Eq, json!(null));

        let result = evaluate_single(&receipt, &rule);
        assert!(result.passed, "null == null should pass");
    }

    #[test]
    fn eq_on_boolean_values() {
        let receipt = json!({"active": true});

        let rule_true = make_rule("test", "/active", RuleOperator::Eq, json!(true));
        assert!(evaluate_single(&receipt, &rule_true).passed);

        let rule_false = make_rule("test", "/active", RuleOperator::Eq, json!(false));
        assert!(!evaluate_single(&receipt, &rule_false).passed);
    }

    #[test]
    fn contains_on_numeric_value_fails_gracefully() {
        // Contains should fail on non-string, non-array
        let receipt = json!({"count": 42});
        let rule = make_rule("test", "/count", RuleOperator::Contains, json!(4));

        let result = evaluate_single(&receipt, &rule);
        assert!(!result.passed);
    }

    #[test]
    fn contains_on_null_fails_gracefully() {
        let receipt = json!({"val": null});
        let rule = make_rule("test", "/val", RuleOperator::Contains, json!("x"));

        let result = evaluate_single(&receipt, &rule);
        assert!(!result.passed);
    }

    #[test]
    fn contains_on_object_fails_gracefully() {
        let receipt = json!({"obj": {"key": "value"}});
        let rule = make_rule("test", "/obj", RuleOperator::Contains, json!("key"));

        let result = evaluate_single(&receipt, &rule);
        assert!(!result.passed);
    }

    #[test]
    fn contains_empty_string_in_any_string() {
        // Empty string is contained in any string
        let receipt = json!({"text": "hello"});
        let rule = make_rule("test", "/text", RuleOperator::Contains, json!(""));

        let result = evaluate_single(&receipt, &rule);
        assert!(result.passed, "empty string is always contained");
    }

    #[test]
    fn contains_in_empty_array() {
        let receipt = json!({"arr": []});
        let rule = make_rule("test", "/arr", RuleOperator::Contains, json!(1));

        let result = evaluate_single(&receipt, &rule);
        assert!(!result.passed, "empty array contains nothing");
    }

    #[test]
    fn in_operator_empty_values_list() {
        let receipt = json!({"val": "x"});
        let rule = PolicyRule {
            name: "test".into(),
            pointer: "/val".into(),
            op: RuleOperator::In,
            value: None,
            values: Some(vec![]),
            negate: false,
            level: RuleLevel::Error,
            message: None,
        };

        let result = evaluate_single(&receipt, &rule);
        assert!(!result.passed, "no value is in an empty list");
    }

    #[test]
    fn in_operator_numeric_values() {
        let receipt = json!({"count": 42});
        let rule = PolicyRule {
            name: "test".into(),
            pointer: "/count".into(),
            op: RuleOperator::In,
            value: None,
            values: Some(vec![json!(10), json!(42), json!(100)]),
            negate: false,
            level: RuleLevel::Error,
            message: None,
        };

        let result = evaluate_single(&receipt, &rule);
        assert!(result.passed, "42 should be in [10, 42, 100]");
    }

    #[test]
    fn numeric_comparison_with_string_coercion() {
        // String "42.5" should be coerced to f64 for comparison
        let receipt = json!({"val": "42.5"});
        let rule = make_rule("test", "/val", RuleOperator::Gt, json!(40));

        let result = evaluate_single(&receipt, &rule);
        assert!(result.passed, "string '42.5' > 40 via coercion");
    }
}

// =========================================================================
// Scenario: RuleOperator and RuleLevel defaults
// =========================================================================

mod type_defaults {
    use super::*;

    #[test]
    fn rule_operator_default_is_eq() {
        assert_eq!(RuleOperator::default(), RuleOperator::Eq);
    }

    #[test]
    fn rule_level_default_is_error() {
        assert_eq!(RuleLevel::default(), RuleLevel::Error);
    }

    #[test]
    fn policy_config_default_has_empty_rules() {
        let config = PolicyConfig::default();
        assert!(config.rules.is_empty());
        assert!(!config.fail_fast);
        assert!(!config.allow_missing);
    }

    #[test]
    fn ratchet_config_default_has_empty_rules() {
        let config = RatchetConfig::default();
        assert!(config.rules.is_empty());
        assert!(!config.fail_fast);
        assert!(!config.allow_missing_baseline);
        assert!(!config.allow_missing_current);
    }
}

// =========================================================================
// Scenario: GateError Display formatting
// =========================================================================

mod gate_error_display {
    use tokmd_gate::GateError;

    #[test]
    fn io_error_display() {
        let err = GateError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        let msg = err.to_string();
        assert!(msg.contains("read policy file"), "got: {}", msg);
        assert!(msg.contains("file not found"), "got: {}", msg);
    }

    #[test]
    fn invalid_pointer_display() {
        let err = GateError::InvalidPointer("bad/pointer".to_string());
        let msg = err.to_string();
        assert!(msg.contains("bad/pointer"), "got: {}", msg);
    }

    #[test]
    fn type_mismatch_display() {
        let err = GateError::TypeMismatch {
            expected: "number".to_string(),
            actual: "string".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("number"), "got: {}", msg);
        assert!(msg.contains("string"), "got: {}", msg);
    }

    #[test]
    fn invalid_operator_display() {
        let err = GateError::InvalidOperator {
            op: "gt".to_string(),
            value_type: "string".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("gt"), "got: {}", msg);
        assert!(msg.contains("string"), "got: {}", msg);
    }

    #[test]
    fn missing_field_display() {
        let err = GateError::MissingField {
            name: "rule1".to_string(),
            field: "pointer".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("rule1"), "got: {}", msg);
        assert!(msg.contains("pointer"), "got: {}", msg);
    }
}

// =========================================================================
// Scenario: Config from_file error paths
// =========================================================================

mod config_file_errors {
    use super::*;
    use std::path::Path;

    #[test]
    fn policy_from_nonexistent_file_returns_error() {
        let result = PolicyConfig::from_file(Path::new("/no/such/policy.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn ratchet_from_nonexistent_file_returns_error() {
        let result = RatchetConfig::from_file(Path::new("/no/such/ratchet.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn policy_from_invalid_toml_returns_error() {
        let result = PolicyConfig::from_toml("[broken\nfoo = bar");
        assert!(result.is_err());
    }

    #[test]
    fn ratchet_from_invalid_toml_returns_error() {
        let result = RatchetConfig::from_toml("[broken\nfoo = bar");
        assert!(result.is_err());
    }
}

// =========================================================================
// Scenario: PolicyConfig and result serde roundtrips
// =========================================================================

mod serde_roundtrips {
    use super::*;

    #[test]
    fn policy_config_roundtrip_through_json() {
        let config = PolicyConfig {
            rules: vec![PolicyRule {
                name: "max_code".into(),
                pointer: "/summary/total_code".into(),
                op: RuleOperator::Lte,
                value: Some(json!(100000)),
                values: None,
                negate: false,
                level: RuleLevel::Error,
                message: Some("Code exceeds limit".into()),
            }],
            fail_fast: true,
            allow_missing: false,
        };

        let json = serde_json::to_string(&config).unwrap();
        let back: PolicyConfig = serde_json::from_str(&json).unwrap();
        assert!(back.fail_fast);
        assert!(!back.allow_missing);
        assert_eq!(back.rules.len(), 1);
        assert_eq!(back.rules[0].name, "max_code");
        assert_eq!(back.rules[0].op, RuleOperator::Lte);
    }

    #[test]
    fn gate_result_roundtrip_through_json() {
        let gate = GateResult::from_results(vec![
            RuleResult {
                name: "r1".into(),
                passed: true,
                level: RuleLevel::Error,
                actual: Some(json!(42)),
                expected: "/count > 10".into(),
                message: None,
            },
            RuleResult {
                name: "r2".into(),
                passed: false,
                level: RuleLevel::Warn,
                actual: Some(json!("MIT")),
                expected: "/license in [GPL]".into(),
                message: Some("License mismatch".into()),
            },
        ]);

        let json = serde_json::to_string(&gate).unwrap();
        let back: GateResult = serde_json::from_str(&json).unwrap();
        assert!(back.passed); // Only warns
        assert_eq!(back.errors, 0);
        assert_eq!(back.warnings, 1);
        assert_eq!(back.rule_results.len(), 2);
    }

    #[test]
    fn ratchet_config_roundtrip_through_json() {
        let config = RatchetConfig {
            rules: vec![RatchetRule {
                pointer: "/complexity/avg".into(),
                max_increase_pct: Some(10.0),
                max_value: Some(50.0),
                level: RuleLevel::Error,
                description: Some("Keep complexity low".into()),
            }],
            fail_fast: true,
            allow_missing_baseline: true,
            allow_missing_current: false,
        };

        let json = serde_json::to_string(&config).unwrap();
        let back: RatchetConfig = serde_json::from_str(&json).unwrap();
        assert!(back.fail_fast);
        assert!(back.allow_missing_baseline);
        assert!(!back.allow_missing_current);
        assert_eq!(back.rules.len(), 1);
        assert_eq!(back.rules[0].max_increase_pct, Some(10.0));
        assert_eq!(back.rules[0].max_value, Some(50.0));
    }

    #[test]
    fn ratchet_gate_result_roundtrip() {
        let result = RatchetGateResult::from_results(vec![]);
        let json = serde_json::to_string(&result).unwrap();
        let back: RatchetGateResult = serde_json::from_str(&json).unwrap();
        assert!(back.passed);
        assert_eq!(back.errors, 0);
        assert_eq!(back.warnings, 0);
    }
}

// =========================================================================
// Scenario: Combined policy + ratchet evaluation
// =========================================================================

mod combined_evaluation {
    use super::*;

    #[test]
    fn policy_and_ratchet_both_pass() {
        let receipt = json!({
            "summary": {"total_code": 5000},
            "complexity": {"avg": 8.0}
        });
        let baseline = json!({
            "complexity": {"avg": 7.5}
        });

        // Policy rules
        let policy = PolicyConfig {
            rules: vec![make_rule(
                "max_code",
                "/summary/total_code",
                RuleOperator::Lte,
                json!(100000),
            )],
            fail_fast: false,
            allow_missing: false,
        };

        let policy_result = evaluate_policy(&receipt, &policy);
        assert!(policy_result.passed);

        // Ratchet rules
        let ratchet = RatchetConfig {
            rules: vec![RatchetRule {
                pointer: "/complexity/avg".into(),
                max_increase_pct: Some(20.0),
                max_value: None,
                level: RuleLevel::Error,
                description: None,
            }],
            fail_fast: false,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };

        let ratchet_result = evaluate_ratchet_policy(&ratchet, &baseline, &receipt);
        assert!(ratchet_result.passed);

        // Combined verdict: both pass
        assert!(policy_result.passed && ratchet_result.passed);
    }

    #[test]
    fn policy_passes_but_ratchet_fails() {
        let receipt = json!({
            "summary": {"total_code": 5000},
            "complexity": {"avg": 15.0}
        });
        let baseline = json!({
            "complexity": {"avg": 7.0}
        });

        let policy = PolicyConfig {
            rules: vec![make_rule(
                "max_code",
                "/summary/total_code",
                RuleOperator::Lte,
                json!(100000),
            )],
            fail_fast: false,
            allow_missing: false,
        };

        let ratchet = RatchetConfig {
            rules: vec![RatchetRule {
                pointer: "/complexity/avg".into(),
                max_increase_pct: Some(10.0),
                max_value: None,
                level: RuleLevel::Error,
                description: None,
            }],
            fail_fast: false,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };

        let policy_result = evaluate_policy(&receipt, &policy);
        let ratchet_result = evaluate_ratchet_policy(&ratchet, &baseline, &receipt);

        assert!(policy_result.passed);
        assert!(!ratchet_result.passed, "complexity increased >10%");
        assert!(!(policy_result.passed && ratchet_result.passed)); // Combined fails
    }

    #[test]
    fn ratchet_with_both_constraints_failing() {
        let baseline = json!({"value": 100.0});
        let current = json!({"value": 200.0}); // 100% increase AND over max_value

        let config = RatchetConfig {
            rules: vec![RatchetRule {
                pointer: "/value".into(),
                max_increase_pct: Some(10.0),
                max_value: Some(150.0),
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
        // max_value is checked first, so message should be about exceeding max
        assert!(
            result.ratchet_results[0]
                .message
                .contains("exceeds maximum")
        );
    }
}

// =========================================================================
// Scenario: Pointer edge cases not covered by existing tests
// =========================================================================

mod pointer_edge_cases {
    use super::*;

    #[test]
    fn pointer_to_empty_string_key() {
        // RFC 6901: "/" refers to key ""
        let doc = json!({"": "empty key"});
        assert_eq!(resolve_pointer(&doc, "/"), Some(&json!("empty key")));
    }

    #[test]
    fn pointer_to_deeply_nested_value() {
        let doc = json!({"a": {"b": {"c": {"d": {"e": 99}}}}});
        assert_eq!(resolve_pointer(&doc, "/a/b/c/d/e"), Some(&json!(99)));
    }

    #[test]
    fn pointer_through_mixed_objects_and_arrays() {
        let doc = json!({
            "data": [
                {"name": "first", "scores": [10, 20]},
                {"name": "second", "scores": [30, 40]}
            ]
        });
        assert_eq!(resolve_pointer(&doc, "/data/1/scores/0"), Some(&json!(30)));
        assert_eq!(resolve_pointer(&doc, "/data/0/name"), Some(&json!("first")));
    }

    #[test]
    fn pointer_with_numeric_key_on_object() {
        // Numeric token on an object should look up the string key "0"
        let doc = json!({"0": "zero", "1": "one"});
        assert_eq!(resolve_pointer(&doc, "/0"), Some(&json!("zero")));
        assert_eq!(resolve_pointer(&doc, "/1"), Some(&json!("one")));
    }
}

// =========================================================================
// Helpers
// =========================================================================

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

fn evaluate_single(receipt: &serde_json::Value, rule: &PolicyRule) -> RuleResult {
    let policy = PolicyConfig {
        rules: vec![rule.clone()],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(receipt, &policy);
    result.rule_results.into_iter().next().unwrap()
}
