//! Ratchet rule evaluation logic.

mod change;
mod evaluate;
mod policy;

pub use policy::evaluate_ratchet_policy;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{RatchetConfig, RatchetRule, RuleLevel};
    use evaluate::{evaluate_ratchet, evaluate_ratchet_with_options};
    use serde_json::json;

    fn make_rule(
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

    #[test]
    fn test_ratchet_no_regression() {
        let baseline = json!({"complexity": {"avg": 10.0}});
        let current = json!({"complexity": {"avg": 9.0}});
        let rule = make_rule("/complexity/avg", Some(10.0), None);

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(result.passed);
        assert_eq!(result.baseline_value, Some(10.0));
        assert_eq!(result.current_value, 9.0);
        assert!(result.change_pct.unwrap() < 0.0); // Decreased
    }

    #[test]
    fn test_ratchet_acceptable_increase() {
        let baseline = json!({"complexity": {"avg": 10.0}});
        let current = json!({"complexity": {"avg": 10.5}}); // 5% increase
        let rule = make_rule("/complexity/avg", Some(10.0), None);

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(result.passed);
        let pct = result.change_pct.unwrap();
        assert!((pct - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_ratchet_excessive_increase() {
        let baseline = json!({"complexity": {"avg": 10.0}});
        let current = json!({"complexity": {"avg": 12.0}}); // 20% increase
        let rule = make_rule("/complexity/avg", Some(10.0), None);

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(!result.passed);
        assert!(result.message.contains("20.00%"));
        assert!(result.message.contains("exceeds"));
    }

    #[test]
    fn test_ratchet_max_value_pass() {
        let baseline = json!({"tokens": 1000});
        let current = json!({"tokens": 900});
        let rule = make_rule("/tokens", None, Some(1500.0));

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(result.passed);
    }

    #[test]
    fn test_ratchet_max_value_fail() {
        let baseline = json!({"tokens": 1000});
        let current = json!({"tokens": 2000});
        let rule = make_rule("/tokens", None, Some(1500.0));

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(!result.passed);
        assert!(result.message.contains("exceeds maximum"));
    }

    #[test]
    fn test_ratchet_both_constraints() {
        // Both max_value and max_increase_pct
        let baseline = json!({"loc": 1000});
        let current = json!({"loc": 1050}); // 5% increase, under max
        let rule = make_rule("/loc", Some(10.0), Some(2000.0));

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(result.passed);

        // Now test max_value exceeded
        let current_over = json!({"loc": 2500}); // Under % but over max
        let result_over = evaluate_ratchet(&rule, &baseline, &current_over);
        assert!(!result_over.passed);
    }

    #[test]
    fn test_ratchet_missing_baseline() {
        let baseline = json!({"other": 100});
        let current = json!({"complexity": {"avg": 10.0}});
        let rule = make_rule("/complexity/avg", Some(10.0), None);

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(!result.passed);
        assert!(result.message.contains("Baseline value not found"));
    }

    #[test]
    fn test_ratchet_missing_baseline_allowed() {
        let baseline = json!({"other": 100});
        let current = json!({"complexity": {"avg": 10.0}});
        let rule = make_rule("/complexity/avg", Some(10.0), None);

        let result = evaluate_ratchet_with_options(&rule, &baseline, &current, true, false);
        assert!(result.passed);
    }

    #[test]
    fn test_ratchet_missing_current() {
        let baseline = json!({"complexity": {"avg": 10.0}});
        let current = json!({"other": 100});
        let rule = make_rule("/complexity/avg", Some(10.0), None);

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(!result.passed);
        assert!(result.message.contains("Current value not found"));
    }

    #[test]
    fn test_ratchet_missing_current_allowed() {
        let baseline = json!({"complexity": {"avg": 10.0}});
        let current = json!({"other": 100});
        let rule = make_rule("/complexity/avg", Some(10.0), None);

        let result = evaluate_ratchet_with_options(&rule, &baseline, &current, false, true);
        assert!(result.passed);
    }

    #[test]
    fn test_ratchet_zero_baseline() {
        let baseline = json!({"count": 0});
        let current = json!({"count": 0});
        let rule = make_rule("/count", Some(10.0), None);

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(result.passed);
        assert_eq!(result.change_pct, Some(0.0));
    }

    #[test]
    fn test_ratchet_zero_baseline_increase() {
        let baseline = json!({"count": 0});
        let current = json!({"count": 5});
        let rule = make_rule("/count", Some(10.0), None);

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(!result.passed);
        assert!(result.change_pct.unwrap().is_infinite());
    }

    #[test]
    fn test_ratchet_string_numeric_coercion() {
        let baseline = json!({"count": "100"});
        let current = json!({"count": "105"});
        let rule = make_rule("/count", Some(10.0), None);

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(result.passed);
        assert_eq!(result.baseline_value, Some(100.0));
        assert_eq!(result.current_value, 105.0);
    }

    #[test]
    fn test_ratchet_only_max_value() {
        // Rule with only max_value (no baseline comparison)
        let baseline = json!({});
        let current = json!({"tokens": 1000});
        let rule = make_rule("/tokens", None, Some(2000.0));

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(result.passed);
    }

    #[test]
    fn test_evaluate_ratchet_policy() {
        let baseline = json!({
            "complexity": {"avg": 10.0},
            "tokens": 1000
        });
        let current = json!({
            "complexity": {"avg": 10.5}, // 5% increase
            "tokens": 900  // decrease
        });

        let config = RatchetConfig {
            rules: vec![
                make_rule("/complexity/avg", Some(10.0), None),
                make_rule("/tokens", None, Some(2000.0)),
            ],
            fail_fast: false,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };

        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(result.passed);
        assert_eq!(result.ratchet_results.len(), 2);
        assert_eq!(result.errors, 0);
    }

    #[test]
    fn test_evaluate_ratchet_policy_fail() {
        let baseline = json!({"complexity": {"avg": 10.0}});
        let current = json!({"complexity": {"avg": 15.0}}); // 50% increase

        let config = RatchetConfig {
            rules: vec![make_rule("/complexity/avg", Some(10.0), None)],
            fail_fast: false,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };

        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(!result.passed);
        assert_eq!(result.errors, 1);
    }

    #[test]
    fn test_ratchet_policy_fail_fast() {
        let baseline = json!({
            "a": 10.0,
            "b": 10.0
        });
        let current = json!({
            "a": 20.0,  // 100% increase - fails
            "b": 20.0   // 100% increase - also fails
        });

        let config = RatchetConfig {
            rules: vec![
                make_rule("/a", Some(10.0), None),
                make_rule("/b", Some(10.0), None),
            ],
            fail_fast: true,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };

        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(!result.passed);
        assert_eq!(result.ratchet_results.len(), 1); // Stopped after first
    }

    #[test]
    fn test_ratchet_policy_warn_level() {
        let baseline = json!({"complexity": {"avg": 10.0}});
        let current = json!({"complexity": {"avg": 15.0}}); // 50% increase

        let rule = RatchetRule {
            pointer: "/complexity/avg".to_string(),
            max_increase_pct: Some(10.0),
            max_value: None,
            level: RuleLevel::Warn,
            description: None,
        };

        let config = RatchetConfig {
            rules: vec![rule],
            fail_fast: false,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };

        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(result.passed); // Warnings don't fail
        assert_eq!(result.warnings, 1);
        assert_eq!(result.errors, 0);
    }

    #[test]
    fn test_ratchet_with_description() {
        let baseline = json!({"complexity": 10.0});
        let current = json!({"complexity": 9.0});
        let rule = RatchetRule {
            pointer: "/complexity".to_string(),
            max_increase_pct: Some(10.0),
            max_value: None,
            level: RuleLevel::Error,
            description: Some("Cyclomatic complexity check".to_string()),
        };

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(result.passed);
        assert!(result.message.contains("Cyclomatic complexity check"));
    }

    #[test]
    fn test_ratchet_boundary_exact_max_increase() {
        let baseline = json!({"value": 100.0});
        let current = json!({"value": 110.0}); // Exactly 10% increase
        let rule = make_rule("/value", Some(10.0), None);

        // At exactly the boundary, should pass (not strictly greater)
        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(result.passed);
    }

    #[test]
    fn test_ratchet_boundary_just_over_max_increase() {
        let baseline = json!({"value": 100.0});
        let current = json!({"value": 110.01}); // Just over 10% increase
        let rule = make_rule("/value", Some(10.0), None);

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(!result.passed);
    }

    #[test]
    fn test_ratchet_boundary_exact_max_value() {
        let baseline = json!({"value": 50.0});
        let current = json!({"value": 100.0}); // Exactly at max
        let rule = make_rule("/value", None, Some(100.0));

        // At exactly max_value, should pass (not strictly greater)
        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(result.passed);
    }

    #[test]
    fn test_ratchet_boundary_just_over_max_value() {
        let baseline = json!({"value": 50.0});
        let current = json!({"value": 100.01}); // Just over max
        let rule = make_rule("/value", None, Some(100.0));

        let result = evaluate_ratchet(&rule, &baseline, &current);
        assert!(!result.passed);
    }

    #[test]
    fn test_ratchet_config_from_toml() {
        let toml = r#"
fail_fast = true
allow_missing_baseline = true

[[rules]]
pointer = "/complexity/avg"
max_increase_pct = 10.0
description = "Complexity limit"
level = "error"

[[rules]]
pointer = "/tokens"
max_value = 500000
level = "warn"
"#;

        let config = RatchetConfig::from_toml(toml).unwrap();
        assert!(config.fail_fast);
        assert!(config.allow_missing_baseline);
        assert_eq!(config.rules.len(), 2);
        assert_eq!(config.rules[0].pointer, "/complexity/avg");
        assert_eq!(config.rules[0].max_increase_pct, Some(10.0));
        assert_eq!(config.rules[1].max_value, Some(500000.0));
    }

    #[test]
    fn test_fail_fast_does_not_stop_on_pass() {
        let baseline = json!({
            "a": 10.0,
            "b": 10.0
        });
        let current = json!({
            "a": 10.0,  // No change - passes
            "b": 20.0   // 100% increase - fails
        });

        let config = RatchetConfig {
            rules: vec![
                make_rule("/a", Some(10.0), None),
                make_rule("/b", Some(10.0), None),
            ],
            fail_fast: true,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };

        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(!result.passed);
        // Should have evaluated both rules (fail_fast shouldn't stop on pass)
        assert_eq!(result.ratchet_results.len(), 2);
        assert_eq!(result.errors, 1);
    }

    #[test]
    fn test_no_fail_fast_evaluates_all_rules() {
        let baseline = json!({
            "a": 10.0,
            "b": 10.0
        });
        let current = json!({
            "a": 20.0,  // 100% increase - fails
            "b": 10.5   // 5% increase - passes
        });

        let config = RatchetConfig {
            rules: vec![
                make_rule("/a", Some(10.0), None),
                make_rule("/b", Some(10.0), None),
            ],
            fail_fast: false,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };

        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(!result.passed);
        assert_eq!(result.ratchet_results.len(), 2);
        assert_eq!(result.errors, 1);
    }

    #[test]
    fn test_gate_result_counts_only_failed_rules() {
        let baseline = json!({"a": 10.0, "b": 10.0});
        let current = json!({"a": 10.5, "b": 15.0}); // a passes, b fails

        let rule_a = RatchetRule {
            pointer: "/a".to_string(),
            max_increase_pct: Some(10.0),
            max_value: None,
            level: RuleLevel::Warn,
            description: None,
        };
        let rule_b = RatchetRule {
            pointer: "/b".to_string(),
            max_increase_pct: Some(10.0),
            max_value: None,
            level: RuleLevel::Warn,
            description: None,
        };

        let config = RatchetConfig {
            rules: vec![rule_a, rule_b],
            fail_fast: false,
            allow_missing_baseline: false,
            allow_missing_current: false,
        };

        let result = evaluate_ratchet_policy(&config, &baseline, &current);
        assert!(result.passed); // Only warnings
        assert_eq!(result.warnings, 1); // Only b failed
        assert_eq!(result.errors, 0);
    }
}
