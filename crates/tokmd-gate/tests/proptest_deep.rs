//! Deep property-based tests for tokmd-gate.
//!
//! Covers deterministic policy evaluation, invalid pointer safety,
//! operator serde round-trips, and GateResult invariants.

use proptest::prelude::*;
use serde_json::json;
use tokmd_gate::{
    GateResult, PolicyConfig, PolicyRule, RuleLevel, RuleOperator, RuleResult, evaluate_policy,
    resolve_pointer,
};

// =========================================================================
// Strategies
// =========================================================================

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

// =========================================================================
// Gate evaluation is deterministic: same inputs → same result
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn evaluate_policy_is_deterministic(
        val in -500i64..500,
        threshold in -500i64..500,
        op in arb_operator(),
        level in arb_level(),
    ) {
        let receipt = json!({"metric": val});
        let policy = PolicyConfig {
            rules: vec![PolicyRule {
                name: "det_test".into(),
                pointer: "/metric".into(),
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

        let r1 = evaluate_policy(&receipt, &policy);
        let r2 = evaluate_policy(&receipt, &policy);

        prop_assert_eq!(r1.passed, r2.passed, "Determinism: passed must be same");
        prop_assert_eq!(r1.errors, r2.errors, "Determinism: errors must be same");
        prop_assert_eq!(r1.warnings, r2.warnings, "Determinism: warnings must be same");
        prop_assert_eq!(
            r1.rule_results.len(),
            r2.rule_results.len(),
            "Determinism: rule_results count must be same"
        );
    }
}

// =========================================================================
// Invalid JSON pointers never panic
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn resolve_pointer_never_panics(
        pointer in "\\PC{0,50}",
    ) {
        let doc = json!({"a": {"b": [1, 2, 3]}, "c": "hello"});
        // Must not panic regardless of input
        let _ = resolve_pointer(&doc, &pointer);
    }

    #[test]
    fn resolve_pointer_no_leading_slash_is_none(
        key in "[a-zA-Z_][a-zA-Z0-9_]{0,15}",
    ) {
        let doc = json!({&key: 42});
        let result = resolve_pointer(&doc, &key);
        prop_assert!(result.is_none(), "Pointer without / must return None");
    }

    #[test]
    fn resolve_pointer_empty_is_root(
        val in -1000i64..1000,
    ) {
        let doc = json!({"x": val});
        let result = resolve_pointer(&doc, "");
        prop_assert_eq!(result, Some(&doc));
    }
}

// =========================================================================
// evaluate_policy never panics on arbitrary JSON
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn evaluate_policy_never_panics_missing_pointer(
        pointer in "/[a-zA-Z_]{1,10}(/[a-zA-Z_]{1,10}){0,3}",
        op in arb_operator(),
        level in arb_level(),
        allow_missing in any::<bool>(),
    ) {
        // Receipt that likely doesn't match the generated pointer
        let receipt = json!({"unrelated": 42});
        let policy = PolicyConfig {
            rules: vec![PolicyRule {
                name: "fuzz_rule".into(),
                pointer,
                op,
                value: Some(json!(100)),
                values: None,
                negate: false,
                level,
                message: None,
            }],
            fail_fast: false,
            allow_missing,
        };
        // Must not panic
        let _ = evaluate_policy(&receipt, &policy);
    }
}

// =========================================================================
// GateResult invariants: errors == 0 ↔ passed
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn gate_result_passed_iff_no_errors(
        n_pass in 0usize..5,
        n_warn in 0usize..5,
        n_error in 0usize..5,
    ) {
        let mut results = Vec::new();
        for i in 0..n_pass {
            results.push(RuleResult {
                name: format!("pass_{}", i),
                passed: true,
                level: RuleLevel::Error,
                actual: None,
                expected: "x".into(),
                message: None,
            });
        }
        for i in 0..n_warn {
            results.push(RuleResult {
                name: format!("warn_{}", i),
                passed: false,
                level: RuleLevel::Warn,
                actual: None,
                expected: "x".into(),
                message: Some("warning".into()),
            });
        }
        for i in 0..n_error {
            results.push(RuleResult {
                name: format!("error_{}", i),
                passed: false,
                level: RuleLevel::Error,
                actual: None,
                expected: "x".into(),
                message: Some("error".into()),
            });
        }

        let gate = GateResult::from_results(results);

        prop_assert_eq!(gate.errors, n_error);
        prop_assert_eq!(gate.warnings, n_warn);
        if n_error == 0 {
            prop_assert!(gate.passed, "Should pass when no errors");
        } else {
            prop_assert!(!gate.passed, "Should fail when errors > 0");
        }
    }
}

// =========================================================================
// RuleOperator serde round-trip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn rule_operator_serde_roundtrip(op in arb_operator()) {
        let json = serde_json::to_string(&op).unwrap();
        let parsed: RuleOperator = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(op, parsed);
    }

    #[test]
    fn rule_level_serde_roundtrip(level in arb_level()) {
        let json = serde_json::to_string(&level).unwrap();
        let parsed: RuleLevel = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(level, parsed);
    }
}

// =========================================================================
// Empty policy always passes
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn empty_policy_always_passes(val in -10000i64..10000) {
        let receipt = json!({"anything": val});
        let policy = PolicyConfig::default();
        let result = evaluate_policy(&receipt, &policy);
        prop_assert!(result.passed, "Empty policy must always pass");
        prop_assert_eq!(result.errors, 0);
        prop_assert_eq!(result.warnings, 0);
        prop_assert!(result.rule_results.is_empty());
    }
}
