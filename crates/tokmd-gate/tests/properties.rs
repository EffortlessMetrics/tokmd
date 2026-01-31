//! Property-based tests for tokmd-gate.
//!
//! These tests verify the correctness of policy evaluation, JSON pointer
//! resolution, and serialization round-trips.

use proptest::prelude::*;
use serde_json::{Value, json};
use tokmd_gate::{
    GateResult, PolicyConfig, PolicyRule, RuleLevel, RuleOperator, evaluate_policy, resolve_pointer,
};

// ============================================================================
// Strategies
// ============================================================================

/// Strategy for generating valid JSON pointer tokens.
fn arb_pointer_token() -> impl Strategy<Value = String> {
    "[a-zA-Z_][a-zA-Z0-9_]{0,10}".prop_map(|s| s)
}

/// Strategy for generating arbitrary JSON values (limited depth).
fn arb_json_value() -> impl Strategy<Value = Value> {
    prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(|b| json!(b)),
        (-1000i64..1000i64).prop_map(|n| json!(n)),
        "[a-zA-Z0-9_ ]{0,20}".prop_map(|s| json!(s)),
    ]
}

/// Strategy for all rule operators.
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

/// Strategy for rule levels.
fn arb_level() -> impl Strategy<Value = RuleLevel> {
    prop_oneof![Just(RuleLevel::Error), Just(RuleLevel::Warn),]
}

// ============================================================================
// resolve_pointer tests
// ============================================================================

proptest! {
    /// Empty pointer returns the whole document.
    #[test]
    fn pointer_empty_returns_root(value in arb_json_value()) {
        let result = resolve_pointer(&value, "");
        prop_assert_eq!(result, Some(&value));
    }

    /// Pointer to non-existent key returns None.
    #[test]
    fn pointer_missing_key_returns_none(key in arb_pointer_token()) {
        let doc = json!({"other": 1});
        let pointer = format!("/{}", key);
        prop_assume!(key != "other");

        let result = resolve_pointer(&doc, &pointer);
        prop_assert!(result.is_none());
    }

    /// Pointer without leading slash returns None.
    #[test]
    fn pointer_without_slash_returns_none(key in arb_pointer_token()) {
        let doc = json!({&key: 1});
        let result = resolve_pointer(&doc, &key);
        prop_assert!(result.is_none(), "Pointer without / should fail");
    }

    /// Pointer resolves nested objects correctly.
    #[test]
    fn pointer_resolves_nested_objects(
        key1 in arb_pointer_token(),
        key2 in arb_pointer_token(),
        value in arb_json_value()
    ) {
        prop_assume!(key1 != key2); // Avoid self-referential structures

        let doc = json!({
            &key1: {
                &key2: value.clone()
            }
        });
        let pointer = format!("/{}/{}", key1, key2);

        let result = resolve_pointer(&doc, &pointer);
        prop_assert_eq!(result, Some(&value));
    }

    /// Pointer resolves array indices correctly.
    #[test]
    fn pointer_resolves_array_indices(idx in 0usize..10) {
        let arr: Vec<Value> = (0..10).map(|i| json!(i)).collect();
        let doc = json!({"items": arr});
        let pointer = format!("/items/{}", idx);

        let result = resolve_pointer(&doc, &pointer);
        prop_assert_eq!(result, Some(&json!(idx as i64)));
    }

    /// Out-of-bounds array index returns None.
    #[test]
    fn pointer_oob_array_returns_none(idx in 100usize..200) {
        let doc = json!({"items": [1, 2, 3]});
        let pointer = format!("/items/{}", idx);

        let result = resolve_pointer(&doc, &pointer);
        prop_assert!(result.is_none());
    }

    /// Escape sequences are handled correctly.
    #[test]
    fn pointer_handles_escapes(_dummy in 0..1u8) {
        // Test ~0 -> ~ and ~1 -> /
        let doc = json!({
            "a/b": {
                "c~d": 42
            }
        });

        let result = resolve_pointer(&doc, "/a~1b/c~0d");
        prop_assert_eq!(result, Some(&json!(42)));
    }

    /// Pointer through non-container returns None.
    #[test]
    fn pointer_through_scalar_returns_none(key in arb_pointer_token()) {
        let doc = json!({"scalar": 42});
        let pointer = format!("/scalar/{}", key);

        let result = resolve_pointer(&doc, &pointer);
        prop_assert!(result.is_none());
    }
}

// ============================================================================
// RuleOperator serialization tests
// ============================================================================

proptest! {
    /// All operators round-trip through JSON correctly.
    #[test]
    fn operator_roundtrip(op in arb_operator()) {
        let json = serde_json::to_string(&op).expect("serialize");
        let parsed: RuleOperator = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(op, parsed);
    }

    /// Operators serialize to snake_case.
    #[test]
    fn operator_is_snake_case(op in arb_operator()) {
        let json = serde_json::to_string(&op).expect("serialize");
        let s = json.trim_matches('"');

        prop_assert!(
            !s.chars().any(|c| c.is_uppercase()),
            "Operator should be lowercase: {}",
            s
        );
    }

    /// RuleLevel round-trips correctly.
    #[test]
    fn level_roundtrip(level in arb_level()) {
        let json = serde_json::to_string(&level).expect("serialize");
        let parsed: RuleLevel = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(level, parsed);
    }
}

// ============================================================================
// Numeric comparison tests
// ============================================================================

proptest! {
    /// Greater-than is strict (a > a is false).
    #[test]
    fn gt_is_strict(n in -1000i64..1000) {
        let receipt = json!({"n": n});
        let rule = make_rule("test", "/n", RuleOperator::Gt, json!(n));

        let result = evaluate_single_rule(&receipt, &rule);
        prop_assert!(!result.passed, "{} > {} should be false", n, n);
    }

    /// Less-than is strict (a < a is false).
    #[test]
    fn lt_is_strict(n in -1000i64..1000) {
        let receipt = json!({"n": n});
        let rule = make_rule("test", "/n", RuleOperator::Lt, json!(n));

        let result = evaluate_single_rule(&receipt, &rule);
        prop_assert!(!result.passed, "{} < {} should be false", n, n);
    }

    /// Greater-than-or-equal includes equality.
    #[test]
    fn gte_includes_equal(n in -1000i64..1000) {
        let receipt = json!({"n": n});
        let rule = make_rule("test", "/n", RuleOperator::Gte, json!(n));

        let result = evaluate_single_rule(&receipt, &rule);
        prop_assert!(result.passed, "{} >= {} should be true", n, n);
    }

    /// Less-than-or-equal includes equality.
    #[test]
    fn lte_includes_equal(n in -1000i64..1000) {
        let receipt = json!({"n": n});
        let rule = make_rule("test", "/n", RuleOperator::Lte, json!(n));

        let result = evaluate_single_rule(&receipt, &rule);
        prop_assert!(result.passed, "{} <= {} should be true", n, n);
    }

    /// Greater-than respects ordering.
    #[test]
    fn gt_respects_order(a in -500i64..500, delta in 1i64..500) {
        let b = a + delta;
        let receipt = json!({"n": b});
        let rule = make_rule("test", "/n", RuleOperator::Gt, json!(a));

        let result = evaluate_single_rule(&receipt, &rule);
        prop_assert!(result.passed, "{} > {} should be true", b, a);
    }

    /// Less-than respects ordering.
    #[test]
    fn lt_respects_order(a in -500i64..500, delta in 1i64..500) {
        let b = a - delta;
        let receipt = json!({"n": b});
        let rule = make_rule("test", "/n", RuleOperator::Lt, json!(a));

        let result = evaluate_single_rule(&receipt, &rule);
        prop_assert!(result.passed, "{} < {} should be true", b, a);
    }

    /// Floating point comparisons work correctly.
    #[test]
    fn float_comparisons(a in -100.0f64..100.0, b in -100.0f64..100.0) {
        let receipt = json!({"n": a});

        let gt_rule = make_rule("test", "/n", RuleOperator::Gt, json!(b));
        let gt_result = evaluate_single_rule(&receipt, &gt_rule);
        prop_assert_eq!(gt_result.passed, a > b);

        let lt_rule = make_rule("test", "/n", RuleOperator::Lt, json!(b));
        let lt_result = evaluate_single_rule(&receipt, &lt_rule);
        prop_assert_eq!(lt_result.passed, a < b);
    }
}

// ============================================================================
// Equality tests
// ============================================================================

proptest! {
    /// Equal values pass equality check.
    #[test]
    fn eq_same_values(n in -1000i64..1000) {
        let receipt = json!({"n": n});
        let rule = make_rule("test", "/n", RuleOperator::Eq, json!(n));

        let result = evaluate_single_rule(&receipt, &rule);
        prop_assert!(result.passed);
    }

    /// Different values fail equality check.
    #[test]
    fn eq_different_values(a in -500i64..500, delta in 1i64..500) {
        let b = a + delta;
        let receipt = json!({"n": a});
        let rule = make_rule("test", "/n", RuleOperator::Eq, json!(b));

        let result = evaluate_single_rule(&receipt, &rule);
        prop_assert!(!result.passed);
    }

    /// Not-equal is the negation of equal.
    #[test]
    fn ne_is_negation_of_eq(a in -1000i64..1000, b in -1000i64..1000) {
        let receipt = json!({"n": a});

        let eq_rule = make_rule("test", "/n", RuleOperator::Eq, json!(b));
        let eq_result = evaluate_single_rule(&receipt, &eq_rule);

        let ne_rule = make_rule("test", "/n", RuleOperator::Ne, json!(b));
        let ne_result = evaluate_single_rule(&receipt, &ne_rule);

        prop_assert_ne!(eq_result.passed, ne_result.passed);
    }

    /// String equality is case-sensitive.
    #[test]
    fn string_eq_case_sensitive(s in "[a-zA-Z]{3,10}") {
        let receipt = json!({"s": &s});

        // Same case should match
        let same_rule = make_rule("test", "/s", RuleOperator::Eq, json!(&s));
        prop_assert!(evaluate_single_rule(&receipt, &same_rule).passed);

        // Swapped case should not match (unless palindromic case)
        let swapped: String = s
            .chars()
            .map(|c| {
                if c.is_uppercase() {
                    c.to_lowercase().next().unwrap()
                } else {
                    c.to_uppercase().next().unwrap()
                }
            })
            .collect();

        if swapped != s {
            let diff_rule = make_rule("test", "/s", RuleOperator::Eq, json!(&swapped));
            prop_assert!(!evaluate_single_rule(&receipt, &diff_rule).passed);
        }
    }
}

// ============================================================================
// Negate tests
// ============================================================================

proptest! {
    /// Negate inverts the result.
    #[test]
    fn negate_inverts_result(n in -1000i64..1000, threshold in -1000i64..1000) {
        let receipt = json!({"n": n});

        // Without negate
        let rule = make_rule("test", "/n", RuleOperator::Gt, json!(threshold));
        let result = evaluate_single_rule(&receipt, &rule);

        // With negate
        let negated_rule = PolicyRule {
            negate: true,
            ..make_rule("test", "/n", RuleOperator::Gt, json!(threshold))
        };
        let negated_result = evaluate_single_rule(&receipt, &negated_rule);

        prop_assert_ne!(
            result.passed,
            negated_result.passed,
            "Negate should invert: {} vs {}",
            result.passed,
            negated_result.passed
        );
    }

    /// Double negate is identity (negate on Eq + Ne).
    #[test]
    fn negate_eq_equals_ne(a in -1000i64..1000, b in -1000i64..1000) {
        let receipt = json!({"n": a});

        // Eq with negate
        let eq_negated = PolicyRule {
            negate: true,
            ..make_rule("test", "/n", RuleOperator::Eq, json!(b))
        };
        let eq_neg_result = evaluate_single_rule(&receipt, &eq_negated);

        // Ne without negate
        let ne_rule = make_rule("test", "/n", RuleOperator::Ne, json!(b));
        let ne_result = evaluate_single_rule(&receipt, &ne_rule);

        prop_assert_eq!(eq_neg_result.passed, ne_result.passed);
    }
}

// ============================================================================
// Exists operator tests
// ============================================================================

proptest! {
    /// Exists returns true when pointer resolves.
    #[test]
    fn exists_true_when_present(key in arb_pointer_token(), value in arb_json_value()) {
        let receipt = json!({&key: value});
        let rule = PolicyRule {
            name: "test".into(),
            pointer: format!("/{}", key),
            op: RuleOperator::Exists,
            value: None,
            values: None,
            negate: false,
            level: RuleLevel::Error,
            message: None,
        };

        let result = evaluate_single_rule(&receipt, &rule);
        prop_assert!(result.passed);
    }

    /// Exists returns false when pointer doesn't resolve.
    #[test]
    fn exists_false_when_missing(key in arb_pointer_token()) {
        let receipt = json!({"other": 1});
        prop_assume!(key != "other");

        let rule = PolicyRule {
            name: "test".into(),
            pointer: format!("/{}", key),
            op: RuleOperator::Exists,
            value: None,
            values: None,
            negate: false,
            level: RuleLevel::Error,
            message: None,
        };

        let result = evaluate_single_rule(&receipt, &rule);
        prop_assert!(!result.passed);
    }

    /// Negated exists for absent key passes.
    #[test]
    fn negated_exists_passes_when_missing(key in arb_pointer_token()) {
        let receipt = json!({"other": 1});
        prop_assume!(key != "other");

        let rule = PolicyRule {
            name: "test".into(),
            pointer: format!("/{}", key),
            op: RuleOperator::Exists,
            value: None,
            values: None,
            negate: true,
            level: RuleLevel::Error,
            message: None,
        };

        let result = evaluate_single_rule(&receipt, &rule);
        prop_assert!(result.passed);
    }
}

// ============================================================================
// In operator tests
// ============================================================================

proptest! {
    /// Value in list passes.
    #[test]
    fn in_passes_when_member(needle in "[a-z]{3,8}", others in prop::collection::vec("[a-z]{3,8}", 1..=3)) {
        prop_assume!(!others.contains(&needle));

        let mut list: Vec<Value> = others.iter().map(|s| json!(s)).collect();
        list.push(json!(&needle));

        let receipt = json!({"val": &needle});
        let rule = PolicyRule {
            name: "test".into(),
            pointer: "/val".into(),
            op: RuleOperator::In,
            value: None,
            values: Some(list),
            negate: false,
            level: RuleLevel::Error,
            message: None,
        };

        let result = evaluate_single_rule(&receipt, &rule);
        prop_assert!(result.passed);
    }

    /// Value not in list fails.
    #[test]
    fn in_fails_when_not_member(needle in "[a-z]{3,8}", list in prop::collection::vec("[A-Z]{3,8}", 1..=4)) {
        let values: Vec<Value> = list.iter().map(|s| json!(s)).collect();

        let receipt = json!({"val": &needle});
        let rule = PolicyRule {
            name: "test".into(),
            pointer: "/val".into(),
            op: RuleOperator::In,
            value: None,
            values: Some(values),
            negate: false,
            level: RuleLevel::Error,
            message: None,
        };

        let result = evaluate_single_rule(&receipt, &rule);
        prop_assert!(!result.passed);
    }
}

// ============================================================================
// Contains operator tests
// ============================================================================

proptest! {
    /// String contains substring passes.
    #[test]
    fn contains_string_passes(prefix in "[a-z]{2,5}", needle in "[a-z]{2,5}", suffix in "[a-z]{2,5}") {
        let haystack = format!("{}{}{}", prefix, needle, suffix);
        let receipt = json!({"text": haystack});
        let rule = make_rule("test", "/text", RuleOperator::Contains, json!(&needle));

        let result = evaluate_single_rule(&receipt, &rule);
        prop_assert!(result.passed);
    }

    /// String doesn't contain substring fails.
    #[test]
    fn contains_string_fails(haystack in "[a-z]{5,15}", needle in "[A-Z]{3,5}") {
        // haystack is lowercase, needle is uppercase - won't contain
        let receipt = json!({"text": &haystack});
        let rule = make_rule("test", "/text", RuleOperator::Contains, json!(&needle));

        let result = evaluate_single_rule(&receipt, &rule);
        prop_assert!(!result.passed);
    }

    /// Array contains element passes.
    #[test]
    fn contains_array_passes(needle in 0i64..100, others in prop::collection::vec(100i64..200, 1..=5)) {
        let mut arr: Vec<Value> = others.iter().map(|n| json!(n)).collect();
        arr.push(json!(needle));

        let receipt = json!({"arr": arr});
        let rule = make_rule("test", "/arr", RuleOperator::Contains, json!(needle));

        let result = evaluate_single_rule(&receipt, &rule);
        prop_assert!(result.passed);
    }
}

// ============================================================================
// GateResult tests
// ============================================================================

proptest! {
    /// GateResult counts errors correctly.
    #[test]
    fn gate_result_counts_errors(
        n_pass in 0usize..5,
        n_fail_error in 0usize..5,
        n_fail_warn in 0usize..5
    ) {
        let mut results = Vec::new();

        for i in 0..n_pass {
            results.push(tokmd_gate::RuleResult {
                name: format!("pass_{}", i),
                passed: true,
                level: RuleLevel::Error,
                actual: None,
                expected: "x".into(),
                message: None,
            });
        }

        for i in 0..n_fail_error {
            results.push(tokmd_gate::RuleResult {
                name: format!("fail_error_{}", i),
                passed: false,
                level: RuleLevel::Error,
                actual: None,
                expected: "x".into(),
                message: Some("error".into()),
            });
        }

        for i in 0..n_fail_warn {
            results.push(tokmd_gate::RuleResult {
                name: format!("fail_warn_{}", i),
                passed: false,
                level: RuleLevel::Warn,
                actual: None,
                expected: "x".into(),
                message: Some("warn".into()),
            });
        }

        let gate = GateResult::from_results(results);

        prop_assert_eq!(gate.errors, n_fail_error);
        prop_assert_eq!(gate.warnings, n_fail_warn);
        prop_assert_eq!(gate.passed, n_fail_error == 0);
    }
}

// ============================================================================
// PolicyConfig parsing tests
// ============================================================================

proptest! {
    /// Empty policy parses successfully.
    #[test]
    fn empty_policy_parses(_dummy in 0..1u8) {
        let toml = "";
        let result = PolicyConfig::from_toml(toml);
        prop_assert!(result.is_ok());

        let config = result.unwrap();
        prop_assert!(config.rules.is_empty());
        prop_assert!(!config.fail_fast);
        prop_assert!(!config.allow_missing);
    }

    /// Policy with all flags parses correctly.
    #[test]
    fn policy_flags_parse(fail_fast in any::<bool>(), allow_missing in any::<bool>()) {
        let toml = format!(
            "fail_fast = {}\nallow_missing = {}\n",
            fail_fast, allow_missing
        );

        let result = PolicyConfig::from_toml(&toml);
        prop_assert!(result.is_ok());

        let config = result.unwrap();
        prop_assert_eq!(config.fail_fast, fail_fast);
        prop_assert_eq!(config.allow_missing, allow_missing);
    }
}

// ============================================================================
// Helper functions
// ============================================================================

fn make_rule(name: &str, pointer: &str, op: RuleOperator, value: Value) -> PolicyRule {
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

fn evaluate_single_rule(receipt: &Value, rule: &PolicyRule) -> tokmd_gate::RuleResult {
    let policy = PolicyConfig {
        rules: vec![rule.clone()],
        fail_fast: false,
        allow_missing: false,
    };
    let result = evaluate_policy(receipt, &policy);
    result.rule_results.into_iter().next().unwrap()
}
