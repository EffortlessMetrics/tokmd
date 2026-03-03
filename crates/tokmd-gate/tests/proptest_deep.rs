//! Deep property-based tests for tokmd-gate.
//!
//! Covers evaluation determinism, invalid pointer safety, comparison
//! transitivity/complementarity, GateResult invariants, and ratchet determinism.

use proptest::prelude::*;
use serde_json::{Value, json};
use tokmd_gate::{
    GateResult, PolicyConfig, PolicyRule, RuleLevel, RuleOperator, evaluate_policy, resolve_pointer,
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

fn make_rule(name: &str, pointer: &str, op: RuleOperator, val: Value) -> PolicyRule {
    PolicyRule {
        name: name.into(),
        pointer: pointer.into(),
        op,
        value: Some(val),
        values: None,
        negate: false,
        level: RuleLevel::Error,
        message: None,
    }
}

fn make_config(rules: Vec<PolicyRule>) -> PolicyConfig {
    PolicyConfig {
        rules,
        fail_fast: false,
        allow_missing: false,
    }
}

fn evaluate_single_rule(receipt: &Value, rule: &PolicyRule) -> GateResult {
    let config = make_config(vec![rule.clone()]);
    evaluate_policy(receipt, &config)
}

// =========================================================================
// Evaluation determinism: same input always produces same result
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn evaluation_deterministic_across_all_operators(
        n in -1000i64..1000,
        threshold in -1000i64..1000,
        op in arb_operator(),
        level in arb_level(),
    ) {
        let receipt = json!({"val": n});
        let rule = PolicyRule {
            name: "test".into(),
            pointer: "/val".into(),
            op,
            value: Some(json!(threshold)),
            values: None,
            negate: false,
            level,
            message: None,
        };
        let r1 = evaluate_single_rule(&receipt, &rule);
        let r2 = evaluate_single_rule(&receipt, &rule);
        prop_assert_eq!(r1.passed, r2.passed, "Evaluation should be deterministic");
        prop_assert_eq!(r1.errors, r2.errors);
        prop_assert_eq!(r1.warnings, r2.warnings);
    }
}

// =========================================================================
// Invalid pointer: never panics
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn invalid_pointer_no_panic(
        pointer in "[a-zA-Z0-9/_]{0,30}",
        n in -100i64..100,
    ) {
        let receipt = json!({"a": {"b": n}});
        let _ = resolve_pointer(&receipt, &pointer);
    }

    #[test]
    fn deeply_nested_pointer_safety(
        depth in 1usize..10,
        n in 0i64..100,
    ) {
        let mut doc = json!(n);
        for i in (0..depth).rev() {
            let key = format!("k{}", i);
            doc = json!({key: doc});
        }
        let pointer: String = (0..depth).map(|i| format!("/k{}", i)).collect();
        let result = resolve_pointer(&doc, &pointer);
        prop_assert_eq!(result, Some(&json!(n)));
    }

    #[test]
    fn allow_missing_behavior(
        key in "[a-z]{1,5}",
        n in 0i64..100,
    ) {
        let receipt = json!({"present": n});
        let rule = make_rule("test", &format!("/{}", key), RuleOperator::Gt, json!(0));
        let config = PolicyConfig {
            rules: vec![rule],
            fail_fast: false,
            allow_missing: true,
        };
        let result = evaluate_policy(&receipt, &config);
        if key != "present" {
            prop_assert!(result.passed, "allow_missing=true should pass for missing keys");
        }
    }
}

// =========================================================================
// Gt/Lt complementary: for a != b, exactly one of (a > b) or (a < b) is true
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn gt_lt_complementary(a in -500i64..500, b in -500i64..500) {
        prop_assume!(a != b);
        let receipt = json!({"val": a});

        let gt_rule = make_rule("gt", "/val", RuleOperator::Gt, json!(b));
        let lt_rule = make_rule("lt", "/val", RuleOperator::Lt, json!(b));

        let gt_result = evaluate_single_rule(&receipt, &gt_rule);
        let lt_result = evaluate_single_rule(&receipt, &lt_rule);

        prop_assert_ne!(
            gt_result.passed, lt_result.passed,
            "For a != b, exactly one of a > b or a < b should hold"
        );
    }

    #[test]
    fn gte_lte_cover_all_cases(a in -500i64..500, b in -500i64..500) {
        let receipt = json!({"val": a});

        let gte_rule = make_rule("gte", "/val", RuleOperator::Gte, json!(b));
        let lte_rule = make_rule("lte", "/val", RuleOperator::Lte, json!(b));

        let gte_result = evaluate_single_rule(&receipt, &gte_rule);
        let lte_result = evaluate_single_rule(&receipt, &lte_rule);

        prop_assert!(
            gte_result.passed || lte_result.passed,
            "At least one of a >= b or a <= b must hold"
        );
    }

    #[test]
    fn comparison_transitivity(
        a in -300i64..300,
        b in -300i64..300,
        c in -300i64..300,
    ) {
        prop_assume!(a > b && b > c);
        let receipt = json!({"val": a});
        let rule = make_rule("gt", "/val", RuleOperator::Gt, json!(c));
        let result = evaluate_single_rule(&receipt, &rule);
        prop_assert!(
            result.passed,
            "Transitivity: {} > {} > {} implies {} > {}",
            a, b, c, a, c
        );
    }
}

// =========================================================================
// GateResult invariants
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn gate_result_passed_iff_zero_errors(
        vals in prop::collection::vec(-100i64..100, 1..=5),
    ) {
        let receipt = json!({"val": vals[0]});
        let rules: Vec<PolicyRule> = vals.iter().map(|&v| PolicyRule {
            name: format!("rule_{}", v),
            pointer: "/val".into(),
            op: RuleOperator::Gte,
            value: Some(json!(v)),
            values: None,
            negate: false,
            level: RuleLevel::Error,
            message: None,
        }).collect();
        let config = make_config(rules);
        let result = evaluate_policy(&receipt, &config);

        if result.errors == 0 {
            prop_assert!(result.passed, "Zero errors should mean passed");
        } else {
            prop_assert!(!result.passed, "Non-zero errors should mean not passed");
        }
    }

    #[test]
    fn gate_result_total_counts(
        n in -50i64..50,
        thresholds in prop::collection::vec(-50i64..50, 1..=8),
    ) {
        let receipt = json!({"val": n});
        let rules: Vec<PolicyRule> = thresholds.iter().enumerate().map(|(i, &t)| PolicyRule {
            name: format!("rule_{}", i),
            pointer: "/val".into(),
            op: RuleOperator::Gte,
            value: Some(json!(t)),
            values: None,
            negate: false,
            level: if i % 2 == 0 { RuleLevel::Error } else { RuleLevel::Warn },
            message: None,
        }).collect();
        let total_rules = rules.len();
        let config = make_config(rules);
        let result = evaluate_policy(&receipt, &config);

        prop_assert!(
            result.errors + result.warnings <= total_rules,
            "errors ({}) + warnings ({}) should not exceed total rules ({})",
            result.errors, result.warnings, total_rules
        );
    }
}

// =========================================================================
// Ratchet determinism: re-evaluating same policy gives same results
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn ratchet_determinism(
        n in -100i64..100,
        threshold in -100i64..100,
        op in arb_operator(),
    ) {
        let receipt = json!({"metric": n});
        let rule = PolicyRule {
            name: "ratchet_test".into(),
            pointer: "/metric".into(),
            op,
            value: Some(json!(threshold)),
            values: None,
            negate: false,
            level: RuleLevel::Error,
            message: None,
        };
        let config = make_config(vec![rule]);

        let r1 = evaluate_policy(&receipt, &config);
        let r2 = evaluate_policy(&receipt, &config);
        let r3 = evaluate_policy(&receipt, &config);

        prop_assert_eq!(r1.passed, r2.passed);
        prop_assert_eq!(r2.passed, r3.passed);
        prop_assert_eq!(r1.errors, r3.errors);
        prop_assert_eq!(r1.warnings, r3.warnings);
    }

    #[test]
    fn ratchet_no_change_passes(n in 0i64..1000) {
        let receipt = json!({"metric": n});
        let rule = make_rule("ratchet", "/metric", RuleOperator::Gte, json!(n));
        let result = evaluate_single_rule(&receipt, &rule);
        prop_assert!(result.passed, "Value equal to threshold should pass gte");
    }
}