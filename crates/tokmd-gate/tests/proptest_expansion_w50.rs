//! Property-based tests for tokmd-gate (W50 expansion).
//!
//! Verifies policy evaluation, JSON pointer resolution, gate status
//! computation, and ratchet evaluation with arbitrary inputs.

use proptest::prelude::*;
use serde_json::{Value, json};
use tokmd_gate::{
    GateResult, PolicyConfig, PolicyRule, RatchetConfig, RatchetGateResult, RatchetRule, RuleLevel,
    RuleOperator, RuleResult, evaluate_policy, evaluate_ratchet_policy, resolve_pointer,
};

// ── Strategies ───────────────────────────────────────────────────────────────

fn arb_rule_operator() -> impl Strategy<Value = RuleOperator> {
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

fn arb_rule_level() -> impl Strategy<Value = RuleLevel> {
    prop_oneof![Just(RuleLevel::Warn), Just(RuleLevel::Error)]
}

fn arb_json_number() -> impl Strategy<Value = Value> {
    (-1_000_000i64..1_000_000).prop_map(|n| json!(n))
}

fn arb_numeric_rule() -> impl Strategy<Value = PolicyRule> {
    (
        "[a-z_]{1,20}",
        "/[a-z]{1,10}",
        prop_oneof![
            Just(RuleOperator::Gt),
            Just(RuleOperator::Gte),
            Just(RuleOperator::Lt),
            Just(RuleOperator::Lte),
            Just(RuleOperator::Eq),
            Just(RuleOperator::Ne),
        ],
        arb_json_number(),
        any::<bool>(),
        arb_rule_level(),
    )
        .prop_map(|(name, pointer, op, value, negate, level)| PolicyRule {
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

fn arb_exists_rule() -> impl Strategy<Value = PolicyRule> {
    (
        "[a-z_]{1,20}",
        "/[a-z]{1,10}",
        any::<bool>(),
        arb_rule_level(),
    )
        .prop_map(|(name, pointer, negate, level)| PolicyRule {
            name,
            pointer,
            op: RuleOperator::Exists,
            value: None,
            values: None,
            negate,
            level,
            message: None,
        })
}

fn arb_policy_rule() -> impl Strategy<Value = PolicyRule> {
    prop_oneof![arb_numeric_rule(), arb_exists_rule()]
}

fn arb_policy_config() -> impl Strategy<Value = PolicyConfig> {
    (
        prop::collection::vec(arb_policy_rule(), 0..10),
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(|(rules, fail_fast, allow_missing)| PolicyConfig {
            rules,
            fail_fast,
            allow_missing,
        })
}

fn arb_simple_json_doc() -> impl Strategy<Value = Value> {
    (
        -1_000_000i64..1_000_000,
        -1_000_000i64..1_000_000,
        "[a-z]{1,10}",
    )
        .prop_map(|(a, b, s)| json!({"a": a, "b": b, "c": s}))
}

fn arb_ratchet_rule() -> impl Strategy<Value = RatchetRule> {
    (
        "/[a-z]{1,10}",
        prop::option::of(0.0f64..100.0),
        prop::option::of(0.0f64..1_000_000.0),
        arb_rule_level(),
    )
        .prop_map(
            |(pointer, max_increase_pct, max_value, level)| RatchetRule {
                pointer,
                max_increase_pct,
                max_value,
                level,
                description: None,
            },
        )
}

// ── JSON Pointer tests ───────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn resolve_pointer_never_panics(
        doc in arb_simple_json_doc(),
        pointer in prop::string::string_regex("/[a-z0-9/]{0,30}").unwrap()
    ) {
        // Should never panic, even on arbitrary pointers
        let _ = resolve_pointer(&doc, &pointer);
    }

    #[test]
    fn empty_pointer_returns_whole_doc(doc in arb_simple_json_doc()) {
        let result = resolve_pointer(&doc, "");
        prop_assert_eq!(result, Some(&doc));
    }

    #[test]
    fn missing_leading_slash_returns_none(
        doc in arb_simple_json_doc(),
        key in "[a-z]{1,10}"
    ) {
        // Without leading slash, should return None
        let result = resolve_pointer(&doc, &key);
        prop_assert!(result.is_none());
    }

    #[test]
    fn known_key_resolves(value in -1_000_000i64..1_000_000) {
        let doc = json!({"metric": value});
        let result = resolve_pointer(&doc, "/metric");
        prop_assert_eq!(result, Some(&json!(value)));
    }

    #[test]
    fn nested_key_resolves(value in -1_000_000i64..1_000_000) {
        let doc = json!({"outer": {"inner": value}});
        let result = resolve_pointer(&doc, "/outer/inner");
        prop_assert_eq!(result, Some(&json!(value)));
    }

    #[test]
    fn array_index_resolves(values in prop::collection::vec(-100i64..100, 1..10)) {
        let doc = json!({"items": values});
        for (i, v) in values.iter().enumerate() {
            let pointer = format!("/items/{}", i);
            prop_assert_eq!(resolve_pointer(&doc, &pointer), Some(&json!(*v)));
        }
    }
}

// ── Policy evaluation tests ──────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn evaluate_policy_never_panics(
        policy in arb_policy_config(),
        doc in arb_simple_json_doc()
    ) {
        let _ = evaluate_policy(&doc, &policy);
    }

    #[test]
    fn empty_policy_always_passes(doc in arb_simple_json_doc()) {
        let policy = PolicyConfig::default();
        let result = evaluate_policy(&doc, &policy);
        prop_assert!(result.passed);
        prop_assert_eq!(result.errors, 0);
        prop_assert_eq!(result.warnings, 0);
    }

    #[test]
    fn gate_result_error_count_matches(results in prop::collection::vec(
        (any::<bool>(), arb_rule_level()),
        0..20
    )) {
        let rule_results: Vec<RuleResult> = results.iter().map(|(passed, level)| {
            RuleResult {
                name: "test".into(),
                passed: *passed,
                level: *level,
                actual: None,
                expected: "x".into(),
                message: None,
            }
        }).collect();

        let gate = GateResult::from_results(rule_results.clone());

        let expected_errors = results.iter()
            .filter(|(passed, level)| !passed && *level == RuleLevel::Error)
            .count();
        let expected_warnings = results.iter()
            .filter(|(passed, level)| !passed && *level == RuleLevel::Warn)
            .count();

        prop_assert_eq!(gate.errors, expected_errors);
        prop_assert_eq!(gate.warnings, expected_warnings);
        prop_assert_eq!(gate.passed, expected_errors == 0);
    }

    #[test]
    fn warn_only_rules_never_fail_gate(
        doc in arb_simple_json_doc(),
        rules in prop::collection::vec(arb_exists_rule(), 1..5)
    ) {
        let warn_rules: Vec<PolicyRule> = rules.into_iter().map(|mut r| {
            r.level = RuleLevel::Warn;
            r
        }).collect();
        let policy = PolicyConfig {
            rules: warn_rules,
            fail_fast: false,
            allow_missing: false,
        };
        let result = evaluate_policy(&doc, &policy);
        prop_assert!(result.passed, "Warn-only rules should never fail the gate");
    }

    #[test]
    fn negate_flips_exists_result(key in "[a-z]{1,10}") {
        let doc = json!({&*key: 42});
        let pointer = format!("/{}", key);

        // exists without negate => pass
        let rule_pass = PolicyRule {
            name: "test".into(),
            pointer: pointer.clone(),
            op: RuleOperator::Exists,
            value: None,
            values: None,
            negate: false,
            level: RuleLevel::Error,
            message: None,
        };
        let policy = PolicyConfig { rules: vec![rule_pass], fail_fast: false, allow_missing: false };
        let r1 = evaluate_policy(&doc, &policy);
        prop_assert!(r1.passed);

        // exists with negate => fail
        let rule_fail = PolicyRule {
            name: "test".into(),
            pointer,
            op: RuleOperator::Exists,
            value: None,
            values: None,
            negate: true,
            level: RuleLevel::Error,
            message: None,
        };
        let policy = PolicyConfig { rules: vec![rule_fail], fail_fast: false, allow_missing: false };
        let r2 = evaluate_policy(&doc, &policy);
        prop_assert!(!r2.passed);
    }
}

// ── Ratchet evaluation tests ─────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn ratchet_empty_rules_passes(
        baseline in arb_simple_json_doc(),
        current in arb_simple_json_doc()
    ) {
        let config = RatchetConfig::default();
        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        prop_assert!(result.passed);
        prop_assert_eq!(result.errors, 0);
    }

    #[test]
    fn ratchet_result_counts_correct(results_data in prop::collection::vec(
        (any::<bool>(), arb_rule_level()),
        0..15
    )) {
        let ratchet_results: Vec<tokmd_gate::RatchetResult> = results_data.iter().map(|(passed, level)| {
            tokmd_gate::RatchetResult {
                rule: RatchetRule {
                    pointer: "/x".into(),
                    max_increase_pct: None,
                    max_value: None,
                    level: *level,
                    description: None,
                },
                passed: *passed,
                baseline_value: Some(1.0),
                current_value: 2.0,
                change_pct: Some(100.0),
                message: "test".into(),
            }
        }).collect();

        let gate = RatchetGateResult::from_results(ratchet_results);
        let expected_errors = results_data.iter()
            .filter(|(p, l)| !p && *l == RuleLevel::Error)
            .count();
        prop_assert_eq!(gate.errors, expected_errors);
        prop_assert_eq!(gate.passed, expected_errors == 0);
    }

    #[test]
    fn ratchet_same_values_pass(value in 0.0f64..1_000.0) {
        let baseline = json!({"metric": value});
        let current = json!({"metric": value});
        let config = RatchetConfig {
            rules: vec![RatchetRule {
                pointer: "/metric".into(),
                max_increase_pct: Some(0.0),
                max_value: None,
                level: RuleLevel::Error,
                description: None,
            }],
            fail_fast: false,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };
        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        prop_assert!(result.passed, "Same values should always pass ratchet");
    }
}

// ── TOML roundtrip tests ─────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn policy_config_toml_roundtrip(
        fail_fast in any::<bool>(),
        allow_missing in any::<bool>()
    ) {
        let toml_str = format!(
            "fail_fast = {}\nallow_missing = {}\n",
            fail_fast, allow_missing
        );
        let config = PolicyConfig::from_toml(&toml_str).unwrap();
        prop_assert_eq!(config.fail_fast, fail_fast);
        prop_assert_eq!(config.allow_missing, allow_missing);
    }

    #[test]
    fn ratchet_config_toml_roundtrip(
        fail_fast in any::<bool>(),
        allow_missing_baseline in any::<bool>()
    ) {
        let toml_str = format!(
            "fail_fast = {}\nallow_missing_baseline = {}\n",
            fail_fast, allow_missing_baseline
        );
        let config = RatchetConfig::from_toml(&toml_str).unwrap();
        prop_assert_eq!(config.fail_fast, fail_fast);
        prop_assert_eq!(config.allow_missing_baseline, allow_missing_baseline);
    }

    #[test]
    fn rule_operator_display_roundtrip(op in arb_rule_operator()) {
        let display = op.to_string();
        prop_assert!(!display.is_empty());
    }
}
