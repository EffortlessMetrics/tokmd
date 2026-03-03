//! Deep property-based tests for tokmd-gate.
//!
//! Covers: operator transitivity, policy serialization round-trips,
//! multi-rule evaluation consistency, and edge cases.

use proptest::prelude::*;
use serde_json::{Value, json};
use tokmd_gate::{
    GateResult, PolicyConfig, PolicyRule, RuleLevel, RuleOperator, evaluate_policy, resolve_pointer,
};

// =========================================================================
// Operator transitivity: if a > b and b > c, then a > c
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn gt_transitive(a in 0i64..100, b in 100i64..200, c in 200i64..300) {
        // c > b > a
        let receipt = json!({"n": c});
        let rule_gt_b = make_rule("/n", RuleOperator::Gt, json!(b));
        let rule_gt_a = make_rule("/n", RuleOperator::Gt, json!(a));
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
        let rule_lt_b = make_rule("/n", RuleOperator::Lt, json!(b));
        let rule_lt_c = make_rule("/n", RuleOperator::Lt, json!(c));
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
        let gt_rule = make_rule("/n", RuleOperator::Gt, json!(b));
        let lte_rule = make_rule("/n", RuleOperator::Lte, json!(b));
        let gt = eval_one(&receipt, &gt_rule).passed;
        let lte = eval_one(&receipt, &lte_rule).passed;
        prop_assert_ne!(gt, lte, "{} > {} = {}, {} <= {} = {}", a, b, gt, a, b, lte);
    }

    #[test]
    fn lt_gte_are_complements(a in -500i64..500, b in -500i64..500) {
        let receipt = json!({"n": a});
        let lt_rule = make_rule("/n", RuleOperator::Lt, json!(b));
        let gte_rule = make_rule("/n", RuleOperator::Gte, json!(b));
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
// GateResult: total rules = passed + errors + warnings
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

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

fn make_rule(pointer: &str, op: RuleOperator, value: Value) -> PolicyRule {
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
