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
// Round-trip: PolicyRule serializes and deserializes to the same value
// ============================================================================

/// Strategy for generating a complete PolicyRule.
fn arb_policy_rule() -> impl Strategy<Value = PolicyRule> {
    (
        "[a-z_]{1,10}",         // name
        arb_pointer_token(),    // pointer key
        arb_numeric_operator(), // op (numeric only for simpler value generation)
        -1000i64..1000i64,      // value
        any::<bool>(),          // negate
        arb_level(),            // level
    )
        .prop_map(|(name, key, op, val, negate, level)| PolicyRule {
            name,
            pointer: format!("/{}", key),
            op,
            value: Some(json!(val)),
            values: None,
            negate,
            level,
            message: None,
        })
}

/// Strategy for numeric operators only (excludes In, Contains, Exists).
fn arb_numeric_operator() -> impl Strategy<Value = RuleOperator> {
    prop_oneof![
        Just(RuleOperator::Gt),
        Just(RuleOperator::Gte),
        Just(RuleOperator::Lt),
        Just(RuleOperator::Lte),
        Just(RuleOperator::Eq),
        Just(RuleOperator::Ne),
    ]
}

proptest! {
    /// Any gate rule round-trips through JSON serialization.
    #[test]
    fn policy_rule_json_roundtrip(rule in arb_policy_rule()) {
        let json_str = serde_json::to_string(&rule).expect("serialize rule");
        let parsed: PolicyRule = serde_json::from_str(&json_str).expect("deserialize rule");

        prop_assert_eq!(&rule.name, &parsed.name);
        prop_assert_eq!(&rule.pointer, &parsed.pointer);
        prop_assert_eq!(rule.op, parsed.op);
        prop_assert_eq!(rule.value, parsed.value);
        prop_assert_eq!(rule.negate, parsed.negate);
        prop_assert_eq!(rule.level, parsed.level);
    }

    /// A full PolicyConfig round-trips through JSON.
    #[test]
    fn policy_config_json_roundtrip(
        fail_fast in any::<bool>(),
        allow_missing in any::<bool>(),
        rules in prop::collection::vec(arb_policy_rule(), 0..5)
    ) {
        let config = PolicyConfig {
            rules,
            fail_fast,
            allow_missing,
        };

        let json_str = serde_json::to_string(&config).expect("serialize config");
        let parsed: PolicyConfig = serde_json::from_str(&json_str).expect("deserialize config");

        prop_assert_eq!(config.fail_fast, parsed.fail_fast);
        prop_assert_eq!(config.allow_missing, parsed.allow_missing);
        prop_assert_eq!(config.rules.len(), parsed.rules.len());

        for (orig, rt) in config.rules.iter().zip(parsed.rules.iter()) {
            prop_assert_eq!(&orig.name, &rt.name);
            prop_assert_eq!(&orig.pointer, &rt.pointer);
            prop_assert_eq!(orig.op, rt.op);
            prop_assert_eq!(&orig.value, &rt.value);
            prop_assert_eq!(orig.negate, rt.negate);
            prop_assert_eq!(orig.level, rt.level);
        }
    }
}

// ============================================================================
// Evaluation determinism: same receipt + same rules = same result
// ============================================================================

proptest! {
    /// Evaluating the same receipt with the same policy twice yields identical results.
    #[test]
    fn evaluation_is_deterministic(
        n in -1000i64..1000,
        threshold in -1000i64..1000,
        op in arb_numeric_operator()
    ) {
        let receipt = json!({"val": n});
        let rule = make_rule("det", "/val", op, json!(threshold));
        let policy = PolicyConfig {
            rules: vec![rule],
            fail_fast: false,
            allow_missing: false,
        };

        let r1 = evaluate_policy(&receipt, &policy);
        let r2 = evaluate_policy(&receipt, &policy);

        prop_assert_eq!(r1.passed, r2.passed);
        prop_assert_eq!(r1.errors, r2.errors);
        prop_assert_eq!(r1.warnings, r2.warnings);
        prop_assert_eq!(r1.rule_results.len(), r2.rule_results.len());
        for (a, b) in r1.rule_results.iter().zip(r2.rule_results.iter()) {
            prop_assert_eq!(a.passed, b.passed);
            prop_assert_eq!(&a.name, &b.name);
            prop_assert_eq!(&a.actual, &b.actual);
        }
    }
}

// ============================================================================
// Empty rules: empty rule set always passes
// ============================================================================

proptest! {
    /// An empty policy always passes regardless of receipt content.
    #[test]
    fn empty_rules_always_pass(value in arb_json_value()) {
        let receipt = json!({"data": value});
        let policy = PolicyConfig {
            rules: vec![],
            fail_fast: false,
            allow_missing: false,
        };

        let result = evaluate_policy(&receipt, &policy);
        prop_assert!(result.passed, "empty rule set must always pass");
        prop_assert_eq!(result.errors, 0);
        prop_assert_eq!(result.warnings, 0);
        prop_assert!(result.rule_results.is_empty());
    }

    /// An empty policy with any combination of flags still passes.
    #[test]
    fn empty_rules_pass_with_any_flags(
        fail_fast in any::<bool>(),
        allow_missing in any::<bool>()
    ) {
        let receipt = json!({"x": 42});
        let policy = PolicyConfig {
            rules: vec![],
            fail_fast,
            allow_missing,
        };

        let result = evaluate_policy(&receipt, &policy);
        prop_assert!(result.passed);
        prop_assert!(result.rule_results.is_empty());
    }
}

// ============================================================================
// Single rule evaluation: known paths produce predictable pass/fail
// ============================================================================

proptest! {
    /// A Lte rule passes when actual <= threshold and fails otherwise.
    #[test]
    fn lte_rule_predictable(actual in -500i64..500, threshold in -500i64..500) {
        let receipt = json!({"n": actual});
        let rule = make_rule("lte_check", "/n", RuleOperator::Lte, json!(threshold));

        let result = evaluate_single_rule(&receipt, &rule);
        prop_assert_eq!(result.passed, actual <= threshold,
            "Lte: {} <= {} should be {}", actual, threshold, actual <= threshold);
    }

    /// A Gte rule passes when actual >= threshold and fails otherwise.
    #[test]
    fn gte_rule_predictable(actual in -500i64..500, threshold in -500i64..500) {
        let receipt = json!({"n": actual});
        let rule = make_rule("gte_check", "/n", RuleOperator::Gte, json!(threshold));

        let result = evaluate_single_rule(&receipt, &rule);
        prop_assert_eq!(result.passed, actual >= threshold,
            "Gte: {} >= {} should be {}", actual, threshold, actual >= threshold);
    }

    /// A Gt rule passes when actual > threshold and fails otherwise.
    #[test]
    fn gt_rule_predictable(actual in -500i64..500, threshold in -500i64..500) {
        let receipt = json!({"n": actual});
        let rule = make_rule("gt_check", "/n", RuleOperator::Gt, json!(threshold));

        let result = evaluate_single_rule(&receipt, &rule);
        prop_assert_eq!(result.passed, actual > threshold,
            "Gt: {} > {} should be {}", actual, threshold, actual > threshold);
    }

    /// A Lt rule passes when actual < threshold and fails otherwise.
    #[test]
    fn lt_rule_predictable(actual in -500i64..500, threshold in -500i64..500) {
        let receipt = json!({"n": actual});
        let rule = make_rule("lt_check", "/n", RuleOperator::Lt, json!(threshold));

        let result = evaluate_single_rule(&receipt, &rule);
        prop_assert_eq!(result.passed, actual < threshold,
            "Lt: {} < {} should be {}", actual, threshold, actual < threshold);
    }
}

// ============================================================================
// Rule composition: all must pass for gate to pass (AND semantics)
// ============================================================================

proptest! {
    /// Gate passes iff every error-level rule passes (AND semantics).
    #[test]
    fn and_semantics_all_pass(values in prop::collection::vec(-500i64..500, 1..=5)) {
        let mut obj = serde_json::Map::new();
        let mut rules = Vec::new();

        for (i, &v) in values.iter().enumerate() {
            let key = format!("k{}", i);
            obj.insert(key.clone(), json!(v));
            // Each rule: value <= 999 — always true for our range
            rules.push(make_rule(
                &format!("rule_{}", i),
                &format!("/{}", key),
                RuleOperator::Lte,
                json!(999),
            ));
        }

        let receipt = Value::Object(obj);
        let policy = PolicyConfig { rules, fail_fast: false, allow_missing: false };
        let result = evaluate_policy(&receipt, &policy);

        prop_assert!(result.passed, "all rules pass → gate must pass");
        prop_assert_eq!(result.errors, 0);
    }

    /// Gate fails if any single error-level rule fails (AND semantics).
    #[test]
    fn and_semantics_one_fails(
        passing_count in 0usize..4,
        fail_val in 500i64..999
    ) {
        let mut obj = serde_json::Map::new();
        let mut rules = Vec::new();

        // Add passing rules
        for i in 0..passing_count {
            let key = format!("p{}", i);
            obj.insert(key.clone(), json!(10));
            rules.push(make_rule(
                &format!("pass_{}", i),
                &format!("/{}", key),
                RuleOperator::Lte,
                json!(999),
            ));
        }

        // Add one failing rule: value > 100 where threshold is 100
        obj.insert("fail_key".to_string(), json!(fail_val));
        rules.push(make_rule("failing", "/fail_key", RuleOperator::Lte, json!(100)));

        let receipt = Value::Object(obj);
        let policy = PolicyConfig { rules, fail_fast: false, allow_missing: false };
        let result = evaluate_policy(&receipt, &policy);

        prop_assert!(!result.passed, "one failing error rule → gate must fail");
        prop_assert_eq!(result.errors, 1);
    }

    /// Warn-only failures don't block the gate (only errors block).
    #[test]
    fn warn_failures_dont_block(
        n_warn_fail in 1usize..5,
        n_pass in 0usize..3
    ) {
        let mut obj = serde_json::Map::new();
        let mut rules = Vec::new();

        for i in 0..n_pass {
            let key = format!("pass_{}", i);
            obj.insert(key.clone(), json!(5));
            rules.push(make_rule(
                &format!("pass_{}", i),
                &format!("/{}", key),
                RuleOperator::Lte,
                json!(999),
            ));
        }

        for i in 0..n_warn_fail {
            let key = format!("warn_{}", i);
            obj.insert(key.clone(), json!(500));
            rules.push(PolicyRule {
                level: RuleLevel::Warn,
                ..make_rule(
                    &format!("warn_{}", i),
                    &format!("/{}", key),
                    RuleOperator::Lte,
                    json!(10),
                )
            });
        }

        let receipt = Value::Object(obj);
        let policy = PolicyConfig { rules, fail_fast: false, allow_missing: false };
        let result = evaluate_policy(&receipt, &policy);

        prop_assert!(result.passed, "warn-only failures must not fail the gate");
        prop_assert_eq!(result.errors, 0);
        prop_assert_eq!(result.warnings, n_warn_fail);
    }
}

// ============================================================================
// Edge cases: invalid JSON pointers fail gracefully
// ============================================================================

proptest! {
    /// A pointer without leading slash causes the rule to fail (not panic).
    #[test]
    fn invalid_pointer_no_leading_slash(key in "[a-z]{2,8}") {
        let receipt = json!({&key: 42});
        // No leading slash — invalid RFC 6901 pointer
        let rule = make_rule("bad_ptr", &key, RuleOperator::Eq, json!(42));

        let result = evaluate_single_rule(&receipt, &rule);
        prop_assert!(!result.passed, "invalid pointer must fail, not panic");
    }

    /// A pointer targeting a non-existent deep path fails gracefully.
    #[test]
    fn missing_deep_path_fails(
        depth in 1usize..5,
        key in "[a-z]{2,6}"
    ) {
        let receipt = json!({"root": 1});
        let pointer = format!("/{}", (0..depth).map(|_| key.as_str()).collect::<Vec<_>>().join("/"));
        let rule = make_rule("deep_miss", &pointer, RuleOperator::Eq, json!(1));

        let result = evaluate_single_rule(&receipt, &rule);
        prop_assert!(!result.passed, "deep missing path must fail");
    }

    /// A pointer targeting a non-existent path with allow_missing passes.
    #[test]
    fn missing_path_allow_missing_passes(key in "[a-z]{2,8}") {
        let receipt = json!({"other": 1});
        prop_assume!(key != "other");

        let rule = make_rule("missing", &format!("/{}", key), RuleOperator::Eq, json!(1));
        let policy = PolicyConfig {
            rules: vec![rule],
            fail_fast: false,
            allow_missing: true,
        };

        let result = evaluate_policy(&receipt, &policy);
        prop_assert!(result.passed, "missing path with allow_missing must pass");
    }

    /// Numeric comparison on non-numeric value fails gracefully.
    #[test]
    fn numeric_op_on_string_non_numeric(s in "[a-z]{3,10}") {
        // Filter out strings that happen to parse as f64 (e.g. "inf", "nan").
        prop_assume!(s.parse::<f64>().is_err());

        let receipt = json!({"val": &s});
        let rule = make_rule("num_on_str", "/val", RuleOperator::Gt, json!(0));

        let result = evaluate_single_rule(&receipt, &rule);
        // Non-numeric string → compare_numeric returns Err → passed = false
        prop_assert!(!result.passed, "numeric op on non-numeric string must fail");
    }

    /// Contains on a non-container type (number) fails gracefully.
    #[test]
    fn contains_on_number_fails(n in -1000i64..1000) {
        let receipt = json!({"val": n});
        let rule = make_rule("contains_num", "/val", RuleOperator::Contains, json!("x"));

        let result = evaluate_single_rule(&receipt, &rule);
        prop_assert!(!result.passed, "contains on a number must fail");
    }
}

// ============================================================================
// Ordering: gate results deterministic regardless of rule ordering
// ============================================================================

proptest! {
    /// Shuffling the rule order does not change the overall pass/fail or counts.
    #[test]
    fn rule_order_does_not_affect_outcome(seed in any::<u64>()) {
        // Build a receipt with known values and several rules (mix of pass/fail)
        let receipt = json!({
            "a": 10,
            "b": 200,
            "c": 50,
            "d": 5
        });

        let rules_original = vec![
            make_rule("r_a", "/a", RuleOperator::Lte, json!(100)),  // pass
            make_rule("r_b", "/b", RuleOperator::Lte, json!(100)),  // fail
            make_rule("r_c", "/c", RuleOperator::Gte, json!(25)),   // pass
            make_rule("r_d", "/d", RuleOperator::Gt, json!(10)),    // fail
        ];

        // Deterministic shuffle based on seed
        let mut rules_shuffled = rules_original.clone();
        let n = rules_shuffled.len();
        let mut s = seed;
        for i in (1..n).rev() {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let j = (s as usize) % (i + 1);
            rules_shuffled.swap(i, j);
        }

        let policy_orig = PolicyConfig {
            rules: rules_original,
            fail_fast: false,
            allow_missing: false,
        };
        let policy_shuf = PolicyConfig {
            rules: rules_shuffled,
            fail_fast: false,
            allow_missing: false,
        };

        let r1 = evaluate_policy(&receipt, &policy_orig);
        let r2 = evaluate_policy(&receipt, &policy_shuf);

        prop_assert_eq!(r1.passed, r2.passed, "pass/fail must be order-independent");
        prop_assert_eq!(r1.errors, r2.errors, "error count must be order-independent");
        prop_assert_eq!(r1.warnings, r2.warnings, "warning count must be order-independent");

        // Individual rule results should match when collected by name
        let mut map1: std::collections::BTreeMap<String, bool> = std::collections::BTreeMap::new();
        let mut map2: std::collections::BTreeMap<String, bool> = std::collections::BTreeMap::new();
        for r in &r1.rule_results { map1.insert(r.name.clone(), r.passed); }
        for r in &r2.rule_results { map2.insert(r.name.clone(), r.passed); }
        prop_assert_eq!(map1, map2, "per-rule pass/fail must be order-independent");
    }

    /// Reversing rule order yields same outcome.
    #[test]
    fn reversed_rules_same_outcome(
        vals in prop::collection::vec(-500i64..500, 2..=6)
    ) {
        let mut obj = serde_json::Map::new();
        let mut rules = Vec::new();

        for (i, &v) in vals.iter().enumerate() {
            let key = format!("v{}", i);
            obj.insert(key.clone(), json!(v));
            rules.push(make_rule(
                &format!("r{}", i),
                &format!("/{}", key),
                RuleOperator::Gte,
                json!(0),
            ));
        }

        let receipt = Value::Object(obj);

        let policy_fwd = PolicyConfig {
            rules: rules.clone(),
            fail_fast: false,
            allow_missing: false,
        };

        let mut rules_rev = rules;
        rules_rev.reverse();
        let policy_rev = PolicyConfig {
            rules: rules_rev,
            fail_fast: false,
            allow_missing: false,
        };

        let r_fwd = evaluate_policy(&receipt, &policy_fwd);
        let r_rev = evaluate_policy(&receipt, &policy_rev);

        prop_assert_eq!(r_fwd.passed, r_rev.passed);
        prop_assert_eq!(r_fwd.errors, r_rev.errors);
        prop_assert_eq!(r_fwd.warnings, r_rev.warnings);
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
