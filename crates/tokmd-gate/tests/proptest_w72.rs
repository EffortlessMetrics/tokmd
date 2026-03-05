//! Wave 72 property-based invariant tests for tokmd-gate.
//!
//! Covers: reflexive operator invariants (eq, neq, gt, gte, lt, lte),
//! evaluation determinism, negate semantics, ratchet identity, and
//! gate result consistency.

use proptest::prelude::*;
use serde_json::{Value, json};
use tokmd_gate::{
    GateResult, PolicyConfig, PolicyRule, RatchetConfig, RatchetGateResult, RatchetRule, RuleLevel,
    RuleOperator, RuleResult, evaluate_policy, evaluate_ratchet_policy, resolve_pointer,
};

// ── Strategies ───────────────────────────────────────────────────────────────

fn arb_json_number() -> impl Strategy<Value = Value> {
    (-1_000_000i64..1_000_000).prop_map(|n| json!(n))
}

fn arb_positive_f64() -> impl Strategy<Value = f64> {
    1.0f64..1_000_000.0
}

fn make_numeric_rule(name: &str, pointer: &str, op: RuleOperator, value: Value) -> PolicyRule {
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

fn make_policy(rules: Vec<PolicyRule>) -> PolicyConfig {
    PolicyConfig {
        rules,
        fail_fast: false,
        allow_missing: false,
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    // ========================================================================
    // 1. eq(x, x) always passes
    // ========================================================================

    #[test]
    fn eq_reflexive(x in arb_json_number()) {
        let receipt = json!({"v": x});
        let rule = make_numeric_rule("eq_self", "/v", RuleOperator::Eq, x);
        let result = evaluate_policy(&receipt, &make_policy(vec![rule]));
        prop_assert!(result.passed, "eq(x, x) should always pass");
    }

    // ========================================================================
    // 2. neq(x, x) always fails
    // ========================================================================

    #[test]
    fn ne_irreflexive(x in arb_json_number()) {
        let receipt = json!({"v": x});
        let rule = make_numeric_rule("ne_self", "/v", RuleOperator::Ne, x);
        let result = evaluate_policy(&receipt, &make_policy(vec![rule]));
        prop_assert!(!result.passed, "ne(x, x) should always fail");
    }

    // ========================================================================
    // 3. gt(x, x) always fails
    // ========================================================================

    #[test]
    fn gt_irreflexive(x in arb_json_number()) {
        let receipt = json!({"v": x});
        let rule = make_numeric_rule("gt_self", "/v", RuleOperator::Gt, x);
        let result = evaluate_policy(&receipt, &make_policy(vec![rule]));
        prop_assert!(!result.passed, "gt(x, x) should always fail");
    }

    // ========================================================================
    // 4. gte(x, x) always passes
    // ========================================================================

    #[test]
    fn gte_reflexive(x in arb_json_number()) {
        let receipt = json!({"v": x});
        let rule = make_numeric_rule("gte_self", "/v", RuleOperator::Gte, x);
        let result = evaluate_policy(&receipt, &make_policy(vec![rule]));
        prop_assert!(result.passed, "gte(x, x) should always pass");
    }

    // ========================================================================
    // 5. lt(x, x) always fails
    // ========================================================================

    #[test]
    fn lt_irreflexive(x in arb_json_number()) {
        let receipt = json!({"v": x});
        let rule = make_numeric_rule("lt_self", "/v", RuleOperator::Lt, x);
        let result = evaluate_policy(&receipt, &make_policy(vec![rule]));
        prop_assert!(!result.passed, "lt(x, x) should always fail");
    }

    // ========================================================================
    // 6. lte(x, x) always passes
    // ========================================================================

    #[test]
    fn lte_reflexive(x in arb_json_number()) {
        let receipt = json!({"v": x});
        let rule = make_numeric_rule("lte_self", "/v", RuleOperator::Lte, x);
        let result = evaluate_policy(&receipt, &make_policy(vec![rule]));
        prop_assert!(result.passed, "lte(x, x) should always pass");
    }

    // ========================================================================
    // 7. Gate evaluation is deterministic
    // ========================================================================

    #[test]
    fn evaluation_is_deterministic(x in -1_000_000i64..1_000_000, threshold in -1_000_000i64..1_000_000) {
        let receipt = json!({"v": x});
        let rule = make_numeric_rule("det", "/v", RuleOperator::Lte, json!(threshold));
        let policy = make_policy(vec![rule]);
        let r1 = evaluate_policy(&receipt, &policy);
        let r2 = evaluate_policy(&receipt, &policy);
        prop_assert_eq!(r1.passed, r2.passed);
        prop_assert_eq!(r1.errors, r2.errors);
        prop_assert_eq!(r1.warnings, r2.warnings);
    }

    // ========================================================================
    // 8. Negate inverts the result
    // ========================================================================

    #[test]
    fn negate_inverts_pass(x in arb_json_number()) {
        let receipt = json!({"v": x.clone()});
        let normal = make_numeric_rule("n", "/v", RuleOperator::Eq, x.clone());
        let negated = PolicyRule { negate: true, ..normal.clone() };

        let r_normal = evaluate_policy(&receipt, &make_policy(vec![normal]));
        let r_negated = evaluate_policy(&receipt, &make_policy(vec![negated]));
        prop_assert_ne!(r_normal.passed, r_negated.passed,
            "Negate should invert: normal={}, negated={}", r_normal.passed, r_negated.passed);
    }

    // ========================================================================
    // 9. Exists on present key always passes
    // ========================================================================

    #[test]
    fn exists_on_present_key(x in arb_json_number()) {
        let receipt = json!({"v": x});
        let rule = PolicyRule {
            name: "exists_check".into(),
            pointer: "/v".into(),
            op: RuleOperator::Exists,
            value: None,
            values: None,
            negate: false,
            level: RuleLevel::Error,
            message: None,
        };
        let result = evaluate_policy(&receipt, &make_policy(vec![rule]));
        prop_assert!(result.passed, "Exists on present key should pass");
    }

    // ========================================================================
    // 10. Exists on absent key always fails
    // ========================================================================

    #[test]
    fn exists_on_absent_key(x in arb_json_number()) {
        let receipt = json!({"other": x});
        let rule = PolicyRule {
            name: "exists_missing".into(),
            pointer: "/missing".into(),
            op: RuleOperator::Exists,
            value: None,
            values: None,
            negate: false,
            level: RuleLevel::Error,
            message: None,
        };
        let result = evaluate_policy(&receipt, &make_policy(vec![rule]));
        prop_assert!(!result.passed, "Exists on absent key should fail");
    }

    // ========================================================================
    // 11. resolve_pointer on root is always Some
    // ========================================================================

    #[test]
    fn resolve_pointer_root(x in arb_json_number()) {
        let doc = json!({"v": x});
        let resolved = resolve_pointer(&doc, "");
        prop_assert!(resolved.is_some(), "Empty pointer should resolve to root");
    }

    // ========================================================================
    // 12. GateResult: errors == 0 implies passed
    // ========================================================================

    #[test]
    fn gate_result_errors_zero_implies_passed(
        warn_count in 0usize..5,
        pass_count in 0usize..5,
    ) {
        let mut results = Vec::new();
        for i in 0..warn_count {
            results.push(RuleResult {
                name: format!("w{i}"),
                passed: false,
                level: RuleLevel::Warn,
                actual: None,
                expected: "x".into(),
                message: None,
            });
        }
        for i in 0..pass_count {
            results.push(RuleResult {
                name: format!("p{i}"),
                passed: true,
                level: RuleLevel::Error,
                actual: None,
                expected: "x".into(),
                message: None,
            });
        }
        let gate = GateResult::from_results(results);
        prop_assert!(gate.passed, "No error failures → gate should pass");
        prop_assert_eq!(gate.errors, 0);
        prop_assert_eq!(gate.warnings, warn_count);
    }

    // ========================================================================
    // 13. Ratchet: identical baseline and current always passes
    // ========================================================================

    #[test]
    fn ratchet_identity_passes(val in arb_positive_f64()) {
        let baseline = json!({"m": val});
        let current = json!({"m": val});
        let config = RatchetConfig {
            rules: vec![RatchetRule {
                pointer: "/m".into(),
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
        prop_assert!(result.passed, "Same baseline/current should pass ratchet");
    }

    // ========================================================================
    // 14. Ratchet: large increase always fails
    // ========================================================================

    #[test]
    fn ratchet_large_increase_fails(base in 1.0f64..1000.0) {
        let current_val = base * 3.0; // 200% increase
        let baseline = json!({"m": base});
        let current = json!({"m": current_val});
        let config = RatchetConfig {
            rules: vec![RatchetRule {
                pointer: "/m".into(),
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
        prop_assert!(!result.passed, "200% increase should fail 10% ratchet");
    }

    // ========================================================================
    // 15. RatchetGateResult: empty rules always pass
    // ========================================================================

    #[test]
    fn ratchet_empty_rules_pass(_seed in 0u32..100) {
        let result = RatchetGateResult::from_results(vec![]);
        prop_assert!(result.passed);
        prop_assert_eq!(result.errors, 0);
        prop_assert_eq!(result.warnings, 0);
    }
}
