//! Deep property-based tests for tokmd-gate.
//!
//! Covers evaluation determinism, invalid pointer safety, comparison
//! transitivity/complementarity, GateResult invariants, and ratchet determinism.
//! Also covers: operator transitivity, policy serialization round-trips,
//! multi-rule evaluation consistency, and edge cases.

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

// =========================================================================
// Operator transitivity: if a > b and b > c, then a > c
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn gt_transitive(a in 0i64..100, b in 100i64..200, c in 200i64..300) {
        // c > b > a
        let receipt = json!({"n": c});
        let rule_gt_b = make_test_rule("/n", RuleOperator::Gt, json!(b));
        let rule_gt_a = make_test_rule("/n", RuleOperator::Gt, json!(a));
        let result_gt_b = eval_one(&receipt, &rule_gt_b);
        let result_gt_a = eval_one(&receipt, &rule_gt_a);
        // c > b should be true and c > a should also be true
        prop_assert!(result_gt_b.passed);
        prop_assert!(result_gt_a.passed);
    }

    #[test]
    fn lt_transitive(a in 0i64..100, b in 100i64..200, c in 200i64..300) {
        // a < b < c
        let receipt = json!({"n": a});
        let rule_lt_b = make_test_rule("/n", RuleOperator::Lt, json!(b));
        let rule_lt_c = make_test_rule("/n", RuleOperator::Lt, json!(c));
        prop_assert!(eval_one(&receipt, &rule_lt_b).passed);
        prop_assert!(eval_one(&receipt, &rule_lt_c).passed);
    }
}

// =========================================================================
// Operator complementarity: gt and lte are complements
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn gt_lte_are_complements(a in -500i64..500, b in -500i64..500) {
        let receipt = json!({"n": a});
        let gt_rule = make_test_rule("/n", RuleOperator::Gt, json!(b));
        let lte_rule = make_test_rule("/n", RuleOperator::Lte, json!(b));
        let gt = eval_one(&receipt, &gt_rule).passed;
        let lte = eval_one(&receipt, &lte_rule).passed;
        prop_assert_ne!(gt, lte, "{} > {} = {}, {} <= {} = {}", a, b, gt, a, b, lte);
    }

    #[test]
    fn lt_gte_are_complements(a in -500i64..500, b in -500i64..500) {
        let receipt = json!({"n": a});
        let lt_rule = make_test_rule("/n", RuleOperator::Lt, json!(b));
        let gte_rule = make_test_rule("/n", RuleOperator::Gte, json!(b));
        let lt = eval_one(&receipt, &lt_rule).passed;
        let gte = eval_one(&receipt, &gte_rule).passed;
        prop_assert_ne!(lt, gte, "{} < {} = {}, {} >= {} = {}", a, b, lt, a, b, gte);
    }
}

// =========================================================================
// Multi-rule evaluation: gate passes only when all error rules pass
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn gate_passes_only_when_all_errors_pass(
        value in 0i64..100,
        threshold1 in 0i64..50,
        threshold2 in 50i64..100,
    ) {
        let receipt = json!({"n": value});
        let rules = vec![
            PolicyRule {
                name: "r1".into(),
                pointer: "/n".into(),
                op: RuleOperator::Gte,
                value: Some(json!(threshold1)),
                values: None,
                negate: false,
                level: RuleLevel::Error,
                message: None,
            },
            PolicyRule {
                name: "r2".into(),
                pointer: "/n".into(),
                op: RuleOperator::Gte,
                value: Some(json!(threshold2)),
                values: None,
                negate: false,
                level: RuleLevel::Error,
                message: None,
            },
        ];
        let policy = PolicyConfig { rules, fail_fast: false, allow_missing: false };
        let result = evaluate_policy(&receipt, &policy);
        let expected_pass = value >= threshold1 && value >= threshold2;
        prop_assert_eq!(result.passed, expected_pass,
            "value={}, t1={}, t2={}", value, threshold1, threshold2);
    }

    #[test]
    fn gate_warnings_do_not_cause_failure(
        value in 0i64..100,
        threshold in 50i64..150,
    ) {
        let receipt = json!({"n": value});
        let rules = vec![PolicyRule {
            name: "w1".into(),
            pointer: "/n".into(),
            op: RuleOperator::Gte,
            value: Some(json!(threshold)),
            values: None,
            negate: false,
            level: RuleLevel::Warn,
            message: None,
        }];
        let policy = PolicyConfig { rules, fail_fast: false, allow_missing: false };
        let result = evaluate_policy(&receipt, &policy);
        // Warnings never cause gate failure
        prop_assert!(result.passed, "Warn-only rules should not cause failure");
    }
}

// =========================================================================
// GateResult: counts consistent
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn gate_result_counts_consistent(
        n_pass in 0usize..5,
        n_fail_error in 0usize..5,
        n_fail_warn in 0usize..5,
    ) {
        let mut results = Vec::new();
        for i in 0..n_pass {
            results.push(tokmd_gate::RuleResult {
                name: format!("pass_{i}"),
                passed: true,
                level: RuleLevel::Error,
                actual: None,
                expected: "x".into(),
                message: None,
            });
        }
        for i in 0..n_fail_error {
            results.push(tokmd_gate::RuleResult {
                name: format!("fail_e_{i}"),
                passed: false,
                level: RuleLevel::Error,
                actual: None,
                expected: "x".into(),
                message: Some("fail".into()),
            });
        }
        for i in 0..n_fail_warn {
            results.push(tokmd_gate::RuleResult {
                name: format!("fail_w_{i}"),
                passed: false,
                level: RuleLevel::Warn,
                actual: None,
                expected: "x".into(),
                message: Some("warn".into()),
            });
        }
        let gate = GateResult::from_results(results);
        let total = n_pass + n_fail_error + n_fail_warn;
        prop_assert_eq!(gate.rule_results.len(), total);
        prop_assert_eq!(gate.errors, n_fail_error);
        prop_assert_eq!(gate.warnings, n_fail_warn);
    }
}

// =========================================================================
// PolicyConfig serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn policy_config_roundtrip(fail_fast in any::<bool>(), allow_missing in any::<bool>()) {
        let toml_str = format!(
            "fail_fast = {}\nallow_missing = {}\n\n[[rules]]\nname = \"test\"\npointer = \"/n\"\nop = \"gte\"\nvalue = 0\nlevel = \"error\"\n",
            fail_fast, allow_missing
        );
        let config = PolicyConfig::from_toml(&toml_str).unwrap();
        prop_assert_eq!(config.fail_fast, fail_fast);
        prop_assert_eq!(config.allow_missing, allow_missing);
        prop_assert_eq!(config.rules.len(), 1);
    }
}

// =========================================================================
// Pointer resolution: deeply nested access
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn pointer_resolves_three_levels_deep(
        k1 in "[a-z]{2,6}",
        k2 in "[a-z]{2,6}",
        k3 in "[a-z]{2,6}",
        val in -1000i64..1000,
    ) {
        prop_assume!(k1 != k2 && k2 != k3 && k1 != k3);
        let doc = json!({ &k1: { &k2: { &k3: val } } });
        let pointer = format!("/{}/{}/{}", k1, k2, k3);
        let result = resolve_pointer(&doc, &pointer);
        prop_assert_eq!(result, Some(&json!(val)));
    }
}

// =========================================================================
// Helpers
// =========================================================================

fn make_test_rule(pointer: &str, op: RuleOperator, value: Value) -> PolicyRule {
    PolicyRule {
        name: "test".into(),
        pointer: pointer.into(),
        op,
        value: Some(value),
        values: None,
        negate: false,
        level: RuleLevel::Error,
        message: None,
    }
}

fn eval_one(receipt: &Value, rule: &PolicyRule) -> tokmd_gate::RuleResult {
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
