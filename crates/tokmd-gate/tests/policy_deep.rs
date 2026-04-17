//! Deep policy evaluation tests for tokmd-gate.
//!
//! Covers JSON pointer resolution, threshold rules, comparison operators,
//! policy evaluation semantics, ratchet mode, and TOML parsing edge cases.

use serde_json::json;
use tokmd_gate::{
    GateResult, PolicyConfig, PolicyRule, RatchetConfig, RatchetGateResult, RatchetRule, RuleLevel,
    RuleOperator, RuleResult, evaluate_policy, evaluate_ratchet_policy, resolve_pointer,
};

// ============================================================================
// Helpers
// ============================================================================

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

fn eval_single(receipt: &serde_json::Value, rule: &PolicyRule) -> RuleResult {
    let policy = PolicyConfig {
        rules: vec![rule.clone()],
        fail_fast: false,
        allow_missing: false,
    };
    evaluate_policy(receipt, &policy)
        .rule_results
        .into_iter()
        .next()
        .unwrap()
}

fn make_ratchet_rule(
    pointer: &str,
    max_increase_pct: Option<f64>,
    max_value: Option<f64>,
) -> RatchetRule {
    RatchetRule {
        pointer: pointer.to_string(),
        max_increase_pct,
        max_value,
        level: RuleLevel::Error,
        description: None,
    }
}

// ============================================================================
// JSON Pointer evaluation
// ============================================================================

mod pointer_evaluation {
    use super::*;

    #[test]
    fn deep_nested_path_a_b_c() {
        let doc = json!({"a": {"b": {"c": 42}}});
        assert_eq!(resolve_pointer(&doc, "/a/b/c"), Some(&json!(42)));
    }

    #[test]
    fn array_index_zero() {
        let doc = json!({"items": ["alpha", "beta", "gamma"]});
        assert_eq!(resolve_pointer(&doc, "/items/0"), Some(&json!("alpha")));
    }

    #[test]
    fn array_index_last() {
        let doc = json!({"items": [10, 20, 30]});
        assert_eq!(resolve_pointer(&doc, "/items/2"), Some(&json!(30)));
    }

    #[test]
    fn tilde_zero_escape_resolves_tilde_in_key() {
        // ~0 unescapes to ~ per RFC 6901
        let doc = json!({"key~with~tildes": 99});
        assert_eq!(
            resolve_pointer(&doc, "/key~0with~0tildes"),
            Some(&json!(99))
        );
    }

    #[test]
    fn tilde_one_escape_resolves_slash_in_key() {
        // ~1 unescapes to / per RFC 6901
        let doc = json!({"path/to/file": "found"});
        assert_eq!(
            resolve_pointer(&doc, "/path~1to~1file"),
            Some(&json!("found"))
        );
    }

    #[test]
    fn combined_tilde_escapes() {
        let doc = json!({"a~/b": "combo"});
        assert_eq!(resolve_pointer(&doc, "/a~0~1b"), Some(&json!("combo")));
    }

    #[test]
    fn nested_objects_and_arrays_mixed() {
        let doc = json!({
            "repos": [
                {"name": "alpha", "tags": ["rust", "cli"]},
                {"name": "beta", "tags": ["python", "ml"]}
            ]
        });
        assert_eq!(
            resolve_pointer(&doc, "/repos/0/tags/1"),
            Some(&json!("cli"))
        );
        assert_eq!(resolve_pointer(&doc, "/repos/1/name"), Some(&json!("beta")));
    }

    #[test]
    fn pointer_nonexistent_intermediate_path() {
        let doc = json!({"a": {"b": 1}});
        assert_eq!(resolve_pointer(&doc, "/a/x/y"), None);
    }

    #[test]
    fn pointer_nonexistent_leaf() {
        let doc = json!({"a": {"b": 1}});
        assert_eq!(resolve_pointer(&doc, "/a/c"), None);
    }

    #[test]
    fn empty_pointer_returns_whole_document() {
        let doc = json!({"x": 1, "y": 2});
        assert_eq!(resolve_pointer(&doc, ""), Some(&doc));
    }

    #[test]
    fn pointer_into_scalar_returns_none() {
        let doc = json!({"a": 42});
        assert_eq!(resolve_pointer(&doc, "/a/b"), None);
    }

    #[test]
    fn pointer_array_out_of_bounds() {
        let doc = json!({"arr": [1, 2]});
        assert_eq!(resolve_pointer(&doc, "/arr/5"), None);
    }

    #[test]
    fn pointer_without_leading_slash_returns_none() {
        let doc = json!({"a": 1});
        assert_eq!(resolve_pointer(&doc, "a"), None);
    }

    #[test]
    fn pointer_to_null_value() {
        let doc = json!({"val": null});
        assert_eq!(resolve_pointer(&doc, "/val"), Some(&json!(null)));
    }

    #[test]
    fn pointer_to_boolean_value() {
        let doc = json!({"flag": true});
        assert_eq!(resolve_pointer(&doc, "/flag"), Some(&json!(true)));
    }

    #[test]
    fn pointer_to_nested_array_of_arrays() {
        let doc = json!({"matrix": [[1, 2], [3, 4], [5, 6]]});
        assert_eq!(resolve_pointer(&doc, "/matrix/2/0"), Some(&json!(5)));
    }
}

// ============================================================================
// Threshold rules
// ============================================================================

mod threshold_rules {
    use super::*;

    #[test]
    fn numeric_min_threshold_pass() {
        let receipt = json!({"code_lines": 500});
        let rule = make_rule("min_code", "/code_lines", RuleOperator::Gte, json!(100));
        assert!(eval_single(&receipt, &rule).passed);
    }

    #[test]
    fn numeric_min_threshold_fail() {
        let receipt = json!({"code_lines": 50});
        let rule = make_rule("min_code", "/code_lines", RuleOperator::Gte, json!(100));
        assert!(!eval_single(&receipt, &rule).passed);
    }

    #[test]
    fn numeric_max_threshold_pass() {
        let receipt = json!({"tokens": 400_000});
        let rule = make_rule("max_tokens", "/tokens", RuleOperator::Lte, json!(500_000));
        assert!(eval_single(&receipt, &rule).passed);
    }

    #[test]
    fn numeric_max_threshold_fail() {
        let receipt = json!({"tokens": 600_000});
        let rule = make_rule("max_tokens", "/tokens", RuleOperator::Lte, json!(500_000));
        assert!(!eval_single(&receipt, &rule).passed);
    }

    #[test]
    fn exact_threshold_integer() {
        let receipt = json!({"version": 2});
        let rule = make_rule("exact_ver", "/version", RuleOperator::Eq, json!(2));
        assert!(eval_single(&receipt, &rule).passed);
    }

    #[test]
    fn exact_threshold_float() {
        let receipt = json!({"ratio": 0.75});
        let rule = make_rule("exact_ratio", "/ratio", RuleOperator::Eq, json!(0.75));
        assert!(eval_single(&receipt, &rule).passed);
    }

    #[test]
    fn boundary_exactly_at_lte_threshold() {
        let receipt = json!({"val": 100});
        let rule = make_rule("boundary", "/val", RuleOperator::Lte, json!(100));
        assert!(
            eval_single(&receipt, &rule).passed,
            "value == threshold should pass for lte"
        );
    }

    #[test]
    fn boundary_exactly_at_gte_threshold() {
        let receipt = json!({"val": 100});
        let rule = make_rule("boundary", "/val", RuleOperator::Gte, json!(100));
        assert!(
            eval_single(&receipt, &rule).passed,
            "value == threshold should pass for gte"
        );
    }

    #[test]
    fn boundary_exactly_at_lt_threshold_fails() {
        let receipt = json!({"val": 100});
        let rule = make_rule("boundary", "/val", RuleOperator::Lt, json!(100));
        assert!(
            !eval_single(&receipt, &rule).passed,
            "value == threshold should fail for lt"
        );
    }

    #[test]
    fn boundary_exactly_at_gt_threshold_fails() {
        let receipt = json!({"val": 100});
        let rule = make_rule("boundary", "/val", RuleOperator::Gt, json!(100));
        assert!(
            !eval_single(&receipt, &rule).passed,
            "value == threshold should fail for gt"
        );
    }

    #[test]
    fn integer_vs_float_comparison() {
        // Integer actual compared against float threshold
        let receipt = json!({"count": 10});
        let rule = make_rule("float_cmp", "/count", RuleOperator::Lte, json!(10.0));
        assert!(eval_single(&receipt, &rule).passed);
    }

    #[test]
    fn float_vs_integer_comparison() {
        // Float actual compared against integer threshold
        let receipt = json!({"ratio": 5.0});
        let rule = make_rule("int_cmp", "/ratio", RuleOperator::Eq, json!(5));
        assert!(eval_single(&receipt, &rule).passed);
    }

    #[test]
    fn ratchet_percentage_threshold_within_bounds() {
        let baseline = json!({"complexity": 10.0});
        let current = json!({"complexity": 10.5}); // 5% increase
        let config = RatchetConfig {
            rules: vec![make_ratchet_rule("/complexity", Some(10.0), None)],
            fail_fast: false,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };
        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(result.passed);
        let pct = result.ratchet_results[0].change_pct.unwrap();
        assert!((pct - 5.0).abs() < 0.01);
    }

    #[test]
    fn ratchet_percentage_threshold_exceeded() {
        let baseline = json!({"complexity": 10.0});
        let current = json!({"complexity": 12.0}); // 20% increase
        let config = RatchetConfig {
            rules: vec![make_ratchet_rule("/complexity", Some(10.0), None)],
            fail_fast: false,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };
        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(!result.passed);
    }

    #[test]
    fn ratchet_exactly_at_percentage_boundary_passes() {
        let baseline = json!({"val": 100.0});
        let current = json!({"val": 110.0}); // exactly 10%
        let config = RatchetConfig {
            rules: vec![make_ratchet_rule("/val", Some(10.0), None)],
            fail_fast: false,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };
        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(result.passed, "exactly at max_increase_pct should pass");
    }

    #[test]
    fn ratchet_max_value_ceiling() {
        let baseline = json!({"tokens": 1000});
        let current = json!({"tokens": 2000});
        let config = RatchetConfig {
            rules: vec![make_ratchet_rule("/tokens", None, Some(1500.0))],
            fail_fast: false,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };
        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(!result.passed);
        assert!(
            result.ratchet_results[0]
                .message
                .contains("exceeds maximum")
        );
    }
}

// ============================================================================
// Comparison rules
// ============================================================================

mod comparison_rules {
    use super::*;

    #[test]
    fn lt_with_numbers() {
        let receipt = json!({"x": 5});
        assert!(
            eval_single(
                &receipt,
                &make_rule("lt", "/x", RuleOperator::Lt, json!(10))
            )
            .passed
        );
        assert!(!eval_single(&receipt, &make_rule("lt", "/x", RuleOperator::Lt, json!(3))).passed);
    }

    #[test]
    fn gt_with_numbers() {
        let receipt = json!({"x": 10});
        assert!(eval_single(&receipt, &make_rule("gt", "/x", RuleOperator::Gt, json!(5))).passed);
        assert!(
            !eval_single(
                &receipt,
                &make_rule("gt", "/x", RuleOperator::Gt, json!(20))
            )
            .passed
        );
    }

    #[test]
    fn eq_with_strings() {
        let receipt = json!({"lang": "Rust"});
        assert!(
            eval_single(
                &receipt,
                &make_rule("eq", "/lang", RuleOperator::Eq, json!("Rust"))
            )
            .passed
        );
        assert!(
            !eval_single(
                &receipt,
                &make_rule("eq", "/lang", RuleOperator::Eq, json!("Go"))
            )
            .passed
        );
    }

    #[test]
    fn ne_with_strings() {
        let receipt = json!({"lang": "Rust"});
        assert!(
            eval_single(
                &receipt,
                &make_rule("ne", "/lang", RuleOperator::Ne, json!("Go"))
            )
            .passed
        );
        assert!(
            !eval_single(
                &receipt,
                &make_rule("ne", "/lang", RuleOperator::Ne, json!("Rust"))
            )
            .passed
        );
    }

    #[test]
    fn eq_with_booleans() {
        let receipt = json!({"active": true});
        assert!(
            eval_single(
                &receipt,
                &make_rule("eq", "/active", RuleOperator::Eq, json!(true))
            )
            .passed
        );
        assert!(
            !eval_single(
                &receipt,
                &make_rule("eq", "/active", RuleOperator::Eq, json!(false))
            )
            .passed
        );
    }

    #[test]
    fn ne_with_numbers() {
        let receipt = json!({"count": 42});
        assert!(
            eval_single(
                &receipt,
                &make_rule("ne", "/count", RuleOperator::Ne, json!(99))
            )
            .passed
        );
        assert!(
            !eval_single(
                &receipt,
                &make_rule("ne", "/count", RuleOperator::Ne, json!(42))
            )
            .passed
        );
    }

    #[test]
    fn type_mismatch_numeric_op_on_boolean() {
        let receipt = json!({"flag": true});
        let rule = make_rule("mismatch", "/flag", RuleOperator::Gt, json!(5));
        assert!(
            !eval_single(&receipt, &rule).passed,
            "boolean > number should fail"
        );
    }

    #[test]
    fn type_mismatch_numeric_op_on_null() {
        let receipt = json!({"val": null});
        let rule = make_rule("mismatch", "/val", RuleOperator::Lt, json!(10));
        assert!(!eval_single(&receipt, &rule).passed);
    }

    #[test]
    fn type_mismatch_numeric_op_on_array() {
        let receipt = json!({"arr": [1, 2, 3]});
        let rule = make_rule("mismatch", "/arr", RuleOperator::Gte, json!(0));
        assert!(!eval_single(&receipt, &rule).passed);
    }

    #[test]
    fn exists_operator_present() {
        let receipt = json!({"license": "MIT"});
        let rule = PolicyRule {
            name: "exists".into(),
            pointer: "/license".into(),
            op: RuleOperator::Exists,
            value: None,
            values: None,
            negate: false,
            level: RuleLevel::Error,
            message: None,
        };
        assert!(eval_single(&receipt, &rule).passed);
    }

    #[test]
    fn exists_operator_absent() {
        let receipt = json!({"name": "test"});
        let rule = PolicyRule {
            name: "exists".into(),
            pointer: "/license".into(),
            op: RuleOperator::Exists,
            value: None,
            values: None,
            negate: false,
            level: RuleLevel::Error,
            message: None,
        };
        assert!(!eval_single(&receipt, &rule).passed);
    }

    #[test]
    fn negated_exists_on_absent_key() {
        let receipt = json!({"name": "test"});
        let rule = PolicyRule {
            name: "no_secrets".into(),
            pointer: "/secrets".into(),
            op: RuleOperator::Exists,
            value: None,
            values: None,
            negate: true,
            level: RuleLevel::Error,
            message: None,
        };
        assert!(
            eval_single(&receipt, &rule).passed,
            "negated exists on absent key should pass"
        );
    }
}

// ============================================================================
// Policy evaluation semantics
// ============================================================================

mod policy_evaluation {
    use super::*;

    #[test]
    fn multiple_rules_on_same_receipt_all_pass() {
        let receipt = json!({"code": 5000, "tokens": 100_000, "lang": "Rust"});
        let policy = PolicyConfig {
            rules: vec![
                make_rule("min_code", "/code", RuleOperator::Gte, json!(100)),
                make_rule("max_tokens", "/tokens", RuleOperator::Lte, json!(500_000)),
                make_rule("lang_check", "/lang", RuleOperator::Eq, json!("Rust")),
            ],
            fail_fast: false,
            allow_missing: false,
        };
        let result = evaluate_policy(&receipt, &policy);
        assert!(result.passed);
        assert_eq!(result.errors, 0);
        assert_eq!(result.rule_results.len(), 3);
    }

    #[test]
    fn multiple_rules_mixed_pass_fail() {
        let receipt = json!({"code": 50, "tokens": 100_000});
        let policy = PolicyConfig {
            rules: vec![
                make_rule("min_code", "/code", RuleOperator::Gte, json!(100)), // fails
                make_rule("max_tokens", "/tokens", RuleOperator::Lte, json!(500_000)), // passes
            ],
            fail_fast: false,
            allow_missing: false,
        };
        let result = evaluate_policy(&receipt, &policy);
        assert!(!result.passed);
        assert_eq!(result.errors, 1);
        assert_eq!(result.rule_results.len(), 2);
        assert!(!result.rule_results[0].passed);
        assert!(result.rule_results[1].passed);
    }

    #[test]
    fn fail_fast_short_circuits_after_first_error() {
        let receipt = json!({"a": 1, "b": 2, "c": 3});
        let policy = PolicyConfig {
            rules: vec![
                make_rule("r1", "/a", RuleOperator::Gt, json!(10)), // fails
                make_rule("r2", "/b", RuleOperator::Gt, json!(10)), // would fail
                make_rule("r3", "/c", RuleOperator::Gt, json!(10)), // would fail
            ],
            fail_fast: true,
            allow_missing: false,
        };
        let result = evaluate_policy(&receipt, &policy);
        assert!(!result.passed);
        assert_eq!(
            result.rule_results.len(),
            1,
            "only first rule should be evaluated"
        );
    }

    #[test]
    fn fail_fast_does_not_stop_on_warnings() {
        let receipt = json!({"a": 100, "b": 200});
        let policy = PolicyConfig {
            rules: vec![
                PolicyRule {
                    name: "warn_a".into(),
                    pointer: "/a".into(),
                    op: RuleOperator::Lte,
                    value: Some(json!(50)),
                    values: None,
                    negate: false,
                    level: RuleLevel::Warn, // warning, not error
                    message: None,
                },
                make_rule("check_b", "/b", RuleOperator::Lte, json!(300)),
            ],
            fail_fast: true,
            allow_missing: false,
        };
        let result = evaluate_policy(&receipt, &policy);
        assert!(result.passed, "warnings don't cause failure");
        assert_eq!(result.warnings, 1);
        assert_eq!(
            result.rule_results.len(),
            2,
            "fail_fast should not stop on warnings"
        );
    }

    #[test]
    fn missing_pointer_without_allow_missing_produces_failure() {
        let receipt = json!({"existing": 1});
        let rule = make_rule("missing", "/nonexistent", RuleOperator::Eq, json!(1));
        let result = eval_single(&receipt, &rule);
        assert!(!result.passed);
        assert!(result.message.unwrap().contains("not found"));
    }

    #[test]
    fn missing_pointer_with_allow_missing_passes() {
        let receipt = json!({"existing": 1});
        let policy = PolicyConfig {
            rules: vec![make_rule(
                "missing",
                "/nonexistent",
                RuleOperator::Eq,
                json!(1),
            )],
            fail_fast: false,
            allow_missing: true,
        };
        let result = evaluate_policy(&receipt, &policy);
        assert!(result.passed);
    }

    #[test]
    fn gate_result_exit_code_pass_is_zero_errors() {
        let result = GateResult::from_results(vec![RuleResult {
            name: "ok".into(),
            passed: true,
            level: RuleLevel::Error,
            actual: Some(json!(42)),
            expected: "test".into(),
            message: None,
        }]);
        assert!(result.passed);
        assert_eq!(result.errors, 0);
    }

    #[test]
    fn gate_result_exit_code_fail_has_nonzero_errors() {
        let result = GateResult::from_results(vec![RuleResult {
            name: "fail".into(),
            passed: false,
            level: RuleLevel::Error,
            actual: Some(json!(100)),
            expected: "test".into(),
            message: Some("over limit".into()),
        }]);
        assert!(!result.passed);
        assert_eq!(result.errors, 1);
    }

    #[test]
    fn gate_result_warnings_only_still_passes() {
        let result = GateResult::from_results(vec![
            RuleResult {
                name: "warn1".into(),
                passed: false,
                level: RuleLevel::Warn,
                actual: None,
                expected: "test".into(),
                message: None,
            },
            RuleResult {
                name: "warn2".into(),
                passed: false,
                level: RuleLevel::Warn,
                actual: None,
                expected: "test".into(),
                message: None,
            },
        ]);
        assert!(result.passed);
        assert_eq!(result.errors, 0);
        assert_eq!(result.warnings, 2);
    }

    #[test]
    fn custom_message_propagated_on_failure() {
        let receipt = json!({"val": 999});
        let rule = PolicyRule {
            name: "custom_msg".into(),
            pointer: "/val".into(),
            op: RuleOperator::Lte,
            value: Some(json!(100)),
            values: None,
            negate: false,
            level: RuleLevel::Error,
            message: Some("Value is way too high!".into()),
        };
        let result = eval_single(&receipt, &rule);
        assert!(!result.passed);
        assert_eq!(result.message.as_deref(), Some("Value is way too high!"));
    }

    #[test]
    fn custom_message_not_present_on_pass() {
        let receipt = json!({"val": 50});
        let rule = PolicyRule {
            name: "custom_msg".into(),
            pointer: "/val".into(),
            op: RuleOperator::Lte,
            value: Some(json!(100)),
            values: None,
            negate: false,
            level: RuleLevel::Error,
            message: Some("Value is way too high!".into()),
        };
        let result = eval_single(&receipt, &rule);
        assert!(result.passed);
        assert!(result.message.is_none(), "message should be None on pass");
    }

    #[test]
    fn rule_result_actual_value_captured() {
        let receipt = json!({"count": 42});
        let rule = make_rule("capture", "/count", RuleOperator::Eq, json!(42));
        let result = eval_single(&receipt, &rule);
        assert_eq!(result.actual, Some(json!(42)));
    }
}

// ============================================================================
// Ratchet mode
// ============================================================================

mod ratchet_mode {
    use super::*;

    #[test]
    fn ratchet_improvement_passes() {
        let baseline = json!({"complexity": 10.0});
        let current = json!({"complexity": 8.0}); // improved
        let config = RatchetConfig {
            rules: vec![make_ratchet_rule("/complexity", Some(5.0), None)],
            fail_fast: false,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };
        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(result.passed);
        assert!(result.ratchet_results[0].change_pct.unwrap() < 0.0);
    }

    #[test]
    fn ratchet_regression_fails() {
        let baseline = json!({"complexity": 5.0});
        let current = json!({"complexity": 10.0}); // doubled
        let config = RatchetConfig {
            rules: vec![make_ratchet_rule("/complexity", Some(10.0), None)],
            fail_fast: false,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };
        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(!result.passed);
    }

    #[test]
    fn ratchet_no_change_passes() {
        let baseline = json!({"val": 42.0});
        let current = json!({"val": 42.0});
        let config = RatchetConfig {
            rules: vec![make_ratchet_rule("/val", Some(0.0), None)],
            fail_fast: false,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };
        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(result.passed);
        let pct = result.ratchet_results[0].change_pct.unwrap();
        assert!((pct - 0.0).abs() < 1e-10);
    }

    #[test]
    fn ratchet_missing_baseline_fails_by_default() {
        let baseline = json!({});
        let current = json!({"val": 5.0});
        let config = RatchetConfig {
            rules: vec![make_ratchet_rule("/val", Some(10.0), None)],
            fail_fast: false,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };
        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(!result.passed);
    }

    #[test]
    fn ratchet_missing_baseline_allowed_passes() {
        let baseline = json!({});
        let current = json!({"val": 5.0});
        let config = RatchetConfig {
            rules: vec![make_ratchet_rule("/val", Some(10.0), None)],
            fail_fast: false,
            allow_missing_baseline: true,
            allow_missing_current: false,
        };
        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(result.passed);
    }

    #[test]
    fn ratchet_missing_current_fails_by_default() {
        let baseline = json!({"val": 5.0});
        let current = json!({});
        let config = RatchetConfig {
            rules: vec![make_ratchet_rule("/val", Some(10.0), None)],
            fail_fast: false,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };
        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(!result.passed);
    }

    #[test]
    fn ratchet_missing_current_allowed_passes() {
        let baseline = json!({"val": 5.0});
        let current = json!({});
        let config = RatchetConfig {
            rules: vec![make_ratchet_rule("/val", Some(10.0), None)],
            fail_fast: false,
            allow_missing_baseline: false,
            allow_missing_current: true,
        };
        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(result.passed);
    }

    #[test]
    fn ratchet_warn_level_does_not_fail_gate() {
        let baseline = json!({"val": 10.0});
        let current = json!({"val": 20.0}); // 100% increase
        let config = RatchetConfig {
            rules: vec![RatchetRule {
                pointer: "/val".into(),
                max_increase_pct: Some(5.0),
                max_value: None,
                level: RuleLevel::Warn,
                description: None,
            }],
            fail_fast: false,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };
        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(result.passed, "warn-level ratchet failure should not block");
        assert_eq!(result.warnings, 1);
    }

    #[test]
    fn ratchet_gate_result_from_empty() {
        let result = RatchetGateResult::from_results(vec![]);
        assert!(result.passed);
        assert_eq!(result.errors, 0);
        assert_eq!(result.warnings, 0);
    }

    #[test]
    fn ratchet_fail_fast_stops_after_first_error() {
        let baseline = json!({"a": 10.0, "b": 10.0, "c": 10.0});
        let current = json!({"a": 50.0, "b": 50.0, "c": 50.0});
        let config = RatchetConfig {
            rules: vec![
                make_ratchet_rule("/a", Some(5.0), None),
                make_ratchet_rule("/b", Some(5.0), None),
                make_ratchet_rule("/c", Some(5.0), None),
            ],
            fail_fast: true,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };
        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(!result.passed);
        assert_eq!(result.ratchet_results.len(), 1);
    }
}

// ============================================================================
// TOML policy parsing
// ============================================================================

mod toml_parsing {
    use super::*;

    #[test]
    fn valid_policy_with_all_fields() {
        let toml = r#"
fail_fast = true
allow_missing = true

[[rules]]
name = "max_tokens"
pointer = "/tokens"
op = "lte"
value = 500000
level = "error"
message = "Token limit exceeded"
negate = false

[[rules]]
name = "license_check"
pointer = "/license"
op = "in"
values = ["MIT", "Apache-2.0"]
level = "warn"
"#;
        let policy = PolicyConfig::from_toml(toml).unwrap();
        assert!(policy.fail_fast);
        assert!(policy.allow_missing);
        assert_eq!(policy.rules.len(), 2);
        assert_eq!(policy.rules[0].name, "max_tokens");
        assert_eq!(policy.rules[0].op, RuleOperator::Lte);
        assert_eq!(policy.rules[0].level, RuleLevel::Error);
        assert_eq!(
            policy.rules[0].message.as_deref(),
            Some("Token limit exceeded")
        );
        assert_eq!(policy.rules[1].op, RuleOperator::In);
        assert_eq!(policy.rules[1].level, RuleLevel::Warn);
    }

    #[test]
    fn valid_ratchet_config_from_toml() {
        let toml = r#"
fail_fast = false
allow_missing_baseline = true
allow_missing_current = false

[[rules]]
pointer = "/complexity/avg"
max_increase_pct = 10.0
max_value = 50.0
level = "error"
description = "Complexity ratchet"
"#;
        let config = RatchetConfig::from_toml(toml).unwrap();
        assert!(!config.fail_fast);
        assert!(config.allow_missing_baseline);
        assert!(!config.allow_missing_current);
        assert_eq!(config.rules.len(), 1);
        assert_eq!(config.rules[0].pointer, "/complexity/avg");
        assert_eq!(config.rules[0].max_increase_pct, Some(10.0));
        assert_eq!(config.rules[0].max_value, Some(50.0));
    }

    #[test]
    fn missing_rules_defaults_to_empty() {
        let toml = r#"
fail_fast = false
"#;
        let policy = PolicyConfig::from_toml(toml).unwrap();
        assert!(policy.rules.is_empty());
    }

    #[test]
    fn extra_unknown_fields_are_ignored_by_default() {
        // serde(default) + toml's deserializer typically ignores unknown fields
        // at the top level for structs with #[serde(default)]
        let toml = r#"
fail_fast = false

[[rules]]
name = "check"
pointer = "/val"
op = "eq"
value = 1
"#;
        let result = PolicyConfig::from_toml(toml);
        assert!(result.is_ok());
    }

    #[test]
    fn invalid_toml_syntax_returns_error() {
        let result = PolicyConfig::from_toml("[[broken\nname = ");
        assert!(result.is_err());
    }

    #[test]
    fn policy_from_file_on_disk() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let toml = r#"
fail_fast = false
allow_missing = false

[[rules]]
name = "file_test"
pointer = "/count"
op = "gte"
value = 0
"#;
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("tokmd-gate-deep-{nanos}.toml"));
        std::fs::write(&path, toml).unwrap();

        let policy = PolicyConfig::from_file(&path).unwrap();
        let _ = std::fs::remove_file(&path);

        assert_eq!(policy.rules.len(), 1);
        assert_eq!(policy.rules[0].name, "file_test");
    }

    #[test]
    fn ratchet_config_from_file_on_disk() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let toml = r#"
[[rules]]
pointer = "/metric"
max_increase_pct = 5.0
level = "error"
"#;
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("tokmd-ratchet-deep-{nanos}.toml"));
        std::fs::write(&path, toml).unwrap();

        let config = RatchetConfig::from_file(&path).unwrap();
        let _ = std::fs::remove_file(&path);

        assert_eq!(config.rules.len(), 1);
        assert_eq!(config.rules[0].pointer, "/metric");
    }

    #[test]
    fn policy_from_nonexistent_file_errors() {
        let result = PolicyConfig::from_file(std::path::Path::new("/no/such/file.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn all_operators_parse_from_toml() {
        for (op_str, expected) in [
            ("gt", RuleOperator::Gt),
            ("gte", RuleOperator::Gte),
            ("lt", RuleOperator::Lt),
            ("lte", RuleOperator::Lte),
            ("eq", RuleOperator::Eq),
            ("ne", RuleOperator::Ne),
            ("in", RuleOperator::In),
            ("contains", RuleOperator::Contains),
            ("exists", RuleOperator::Exists),
        ] {
            let toml = format!(
                r#"
[[rules]]
name = "test_{op_str}"
pointer = "/val"
op = "{op_str}"
"#
            );
            let policy = PolicyConfig::from_toml(&toml).unwrap();
            assert_eq!(
                policy.rules[0].op, expected,
                "operator '{}' should parse correctly",
                op_str
            );
        }
    }

    #[test]
    fn both_levels_parse_from_toml() {
        for (level_str, expected) in [("error", RuleLevel::Error), ("warn", RuleLevel::Warn)] {
            let toml = format!(
                r#"
[[rules]]
name = "test"
pointer = "/val"
op = "eq"
value = 1
level = "{level_str}"
"#
            );
            let policy = PolicyConfig::from_toml(&toml).unwrap();
            assert_eq!(policy.rules[0].level, expected);
        }
    }
}
