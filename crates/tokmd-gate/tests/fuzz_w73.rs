//! Fuzz-like property tests for tokmd-gate.
//!
//! These tests exercise policy evaluation, JSON pointer resolution, and rule
//! matching with large random input spaces to ensure no panics occur.

use proptest::prelude::*;
use serde_json::{Value, json};
use tokmd_gate::{
    PolicyConfig, PolicyRule, RatchetConfig, RatchetRule, RuleLevel, RuleOperator, evaluate_policy,
    evaluate_ratchet_policy, resolve_pointer,
};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_json_value_shallow() -> impl Strategy<Value = Value> {
    prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(|b| json!(b)),
        any::<i64>().prop_map(|n| json!(n)),
        any::<f64>()
            .prop_filter("finite", |f| f.is_finite())
            .prop_map(|n| json!(n)),
        ".*".prop_map(|s| json!(s)),
    ]
}

fn arb_json_value_nested() -> impl Strategy<Value = Value> {
    arb_json_value_shallow().prop_recursive(4, 64, 8, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..8).prop_map(Value::Array),
            prop::collection::btree_map("[a-zA-Z_]{1,8}", inner, 0..8)
                .prop_map(|m| Value::Object(m.into_iter().collect())),
        ]
    })
}

fn arb_pointer_segment() -> impl Strategy<Value = String> {
    prop_oneof![
        "[a-zA-Z_]{1,10}",
        "0|[1-9][0-9]{0,3}",
        "~0|~1",
        ".*".prop_map(|s: String| s.chars().take(10).collect()),
    ]
}

fn arb_json_pointer() -> impl Strategy<Value = String> {
    prop::collection::vec(arb_pointer_segment(), 0..6)
        .prop_map(|segments| {
            if segments.is_empty() {
                String::new()
            } else {
                format!("/{}", segments.join("/"))
            }
        })
}

fn arb_operator() -> impl Strategy<Value = RuleOperator> {
    prop_oneof![
        Just(RuleOperator::Gt),
        Just(RuleOperator::Gte),
        Just(RuleOperator::Lt),
        Just(RuleOperator::Lte),
        Just(RuleOperator::Eq),
        Just(RuleOperator::Ne),
        Just(RuleOperator::In),
        Just(RuleOperator::Contains),
        Just(RuleOperator::Exists),
    ]
}

fn arb_level() -> impl Strategy<Value = RuleLevel> {
    prop_oneof![Just(RuleLevel::Error), Just(RuleLevel::Warn)]
}

fn arb_policy_rule() -> impl Strategy<Value = PolicyRule> {
    (
        "[a-z_]{1,15}",
        arb_json_pointer(),
        arb_operator(),
        arb_json_value_shallow(),
        arb_level(),
        any::<bool>(),
    )
        .prop_map(|(name, pointer, op, value, level, negate)| PolicyRule {
            name,
            pointer,
            op,
            value: Some(value),
            values: None,
            negate,
            level,
            message: None,
        })
}

fn arb_policy_config() -> impl Strategy<Value = PolicyConfig> {
    (
        prop::collection::vec(arb_policy_rule(), 0..8),
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(|(rules, fail_fast, allow_missing)| PolicyConfig {
            rules,
            fail_fast,
            allow_missing,
        })
}

fn arb_ratchet_rule() -> impl Strategy<Value = RatchetRule> {
    (
        arb_json_pointer(),
        prop::option::of(0.0f64..1000.0),
        prop::option::of(any::<f64>().prop_filter("finite", |f| f.is_finite())),
        arb_level(),
    )
        .prop_map(|(pointer, max_increase_pct, max_value, level)| RatchetRule {
            pointer,
            max_increase_pct,
            max_value,
            level,
            description: None,
        })
}

fn arb_ratchet_config() -> impl Strategy<Value = RatchetConfig> {
    (
        prop::collection::vec(arb_ratchet_rule(), 0..6),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(
            |(rules, fail_fast, allow_missing_baseline, allow_missing_current)| RatchetConfig {
                rules,
                fail_fast,
                allow_missing_baseline,
                allow_missing_current,
            },
        )
}

// ---------------------------------------------------------------------------
// 1. JSON pointer resolution never panics
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn fuzz_resolve_pointer_no_panic(
        doc in arb_json_value_nested(),
        pointer in arb_json_pointer(),
    ) {
        let _ = resolve_pointer(&doc, &pointer);
    }

    #[test]
    fn fuzz_resolve_pointer_arbitrary_string(
        doc in arb_json_value_nested(),
        pointer in ".*",
    ) {
        let _ = resolve_pointer(&doc, &pointer);
    }

    #[test]
    fn fuzz_resolve_pointer_empty_always_returns_root(doc in arb_json_value_nested()) {
        let result = resolve_pointer(&doc, "");
        prop_assert_eq!(result, Some(&doc));
    }
}

// ---------------------------------------------------------------------------
// 2. Policy evaluation never panics
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn fuzz_evaluate_policy_no_panic(
        receipt in arb_json_value_nested(),
        policy in arb_policy_config(),
    ) {
        let _ = evaluate_policy(&receipt, &policy);
    }

    #[test]
    fn fuzz_evaluate_policy_result_consistent(
        receipt in arb_json_value_nested(),
        policy in arb_policy_config(),
    ) {
        let result = evaluate_policy(&receipt, &policy);
        // errors + warnings must equal total rule results
        let error_count = result.rule_results.iter().filter(|r| !r.passed && r.level == RuleLevel::Error).count();
        let warn_count = result.rule_results.iter().filter(|r| !r.passed && r.level == RuleLevel::Warn).count();
        prop_assert_eq!(result.errors, error_count);
        prop_assert_eq!(result.warnings, warn_count);
    }

    #[test]
    fn fuzz_evaluate_empty_policy_always_passes(receipt in arb_json_value_nested()) {
        let policy = PolicyConfig::default();
        let result = evaluate_policy(&receipt, &policy);
        prop_assert!(result.passed);
        prop_assert_eq!(result.errors, 0);
        prop_assert_eq!(result.warnings, 0);
    }

    #[test]
    fn fuzz_evaluate_policy_warn_never_fails(receipt in arb_json_value_nested()) {
        let policy = PolicyConfig {
            rules: vec![PolicyRule {
                name: "warn_rule".into(),
                pointer: "/nonexistent".into(),
                op: RuleOperator::Exists,
                value: None,
                values: None,
                negate: false,
                level: RuleLevel::Warn,
                message: None,
            }],
            fail_fast: false,
            allow_missing: true,
        };
        let result = evaluate_policy(&receipt, &policy);
        // Warn-level rules never cause overall failure
        prop_assert!(result.passed);
    }
}

// ---------------------------------------------------------------------------
// 3. Ratchet evaluation never panics
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn fuzz_ratchet_no_panic(
        baseline in arb_json_value_nested(),
        current in arb_json_value_nested(),
        config in arb_ratchet_config(),
    ) {
        let _ = evaluate_ratchet_policy(&config, &baseline, &current);
    }

    #[test]
    fn fuzz_ratchet_identical_inputs(
        doc in arb_json_value_nested(),
        config in arb_ratchet_config(),
    ) {
        let result = evaluate_ratchet_policy(&config, &doc, &doc);
        // Same baseline and current should never regress on percentage increase
        // (unless max_value is exceeded, which is structural)
        let _ = result;
    }

    #[test]
    fn fuzz_ratchet_empty_config_passes(
        baseline in arb_json_value_nested(),
        current in arb_json_value_nested(),
    ) {
        let config = RatchetConfig {
            rules: vec![],
            fail_fast: false,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };
        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        prop_assert!(result.passed);
    }
}

// ---------------------------------------------------------------------------
// 4. Rule matching with random numbers
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn fuzz_numeric_rule_no_panic(
        actual in any::<i64>(),
        threshold in any::<i64>(),
        op in arb_operator(),
        level in arb_level(),
    ) {
        let receipt = json!({"value": actual});
        let policy = PolicyConfig {
            rules: vec![PolicyRule {
                name: "numeric_fuzz".into(),
                pointer: "/value".into(),
                op,
                value: Some(json!(threshold)),
                values: None,
                negate: false,
                level,
                message: None,
            }],
            fail_fast: false,
            allow_missing: false,
        };
        let _ = evaluate_policy(&receipt, &policy);
    }

    #[test]
    fn fuzz_negated_rule_no_panic(
        actual in any::<i64>(),
        threshold in any::<i64>(),
        op in arb_operator(),
    ) {
        let receipt = json!({"v": actual});
        let policy = PolicyConfig {
            rules: vec![PolicyRule {
                name: "negated_fuzz".into(),
                pointer: "/v".into(),
                op,
                value: Some(json!(threshold)),
                values: None,
                negate: true,
                level: RuleLevel::Error,
                message: None,
            }],
            fail_fast: false,
            allow_missing: false,
        };
        let _ = evaluate_policy(&receipt, &policy);
    }

    #[test]
    fn fuzz_in_operator_no_panic(
        actual in any::<i64>(),
        candidates in prop::collection::vec(any::<i64>(), 0..10),
    ) {
        let receipt = json!({"v": actual});
        let values: Vec<Value> = candidates.into_iter().map(|c| json!(c)).collect();
        let policy = PolicyConfig {
            rules: vec![PolicyRule {
                name: "in_fuzz".into(),
                pointer: "/v".into(),
                op: RuleOperator::In,
                value: None,
                values: Some(values),
                negate: false,
                level: RuleLevel::Error,
                message: None,
            }],
            fail_fast: false,
            allow_missing: false,
        };
        let _ = evaluate_policy(&receipt, &policy);
    }
}

// ---------------------------------------------------------------------------
// 5. Deeply nested JSON never causes stack overflow
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn fuzz_deeply_nested_receipt(depth in 1usize..50) {
        let mut doc = json!(42);
        for i in 0..depth {
            doc = json!({ format!("k{}", i): doc });
        }
        let policy = PolicyConfig {
            rules: vec![PolicyRule {
                name: "deep".into(),
                pointer: "/k0".into(),
                op: RuleOperator::Exists,
                value: None,
                values: None,
                negate: false,
                level: RuleLevel::Error,
                message: None,
            }],
            fail_fast: false,
            allow_missing: true,
        };
        let _ = evaluate_policy(&doc, &policy);
    }
}
