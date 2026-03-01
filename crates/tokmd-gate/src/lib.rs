//! # tokmd-gate
//!
//! **Tier 3 (Policy Evaluation)**
//!
//! Policy evaluation engine for CI gating based on analysis receipts.
//!
//! ## What belongs here
//! * Policy rule types and parsing
//! * JSON Pointer resolution
//! * Rule evaluation logic
//! * Ratchet evaluation for trend tracking
//!
//! ## Example
//! ```ignore
//! use tokmd_gate::{PolicyConfig, evaluate_policy};
//!
//! let receipt = serde_json::from_str(json)?;
//! let policy = PolicyConfig::from_file("policy.toml")?;
//! let result = evaluate_policy(&receipt, &policy)?;
//! ```

mod evaluate;
mod pointer;
mod ratchet;
mod types;

pub use evaluate::evaluate_policy;
pub use pointer::resolve_pointer;
pub use ratchet::evaluate_ratchet_policy;
pub use types::{
    GateError, GateResult, PolicyConfig, PolicyRule, RatchetConfig, RatchetGateResult,
    RatchetResult, RatchetRule, RuleLevel, RuleOperator, RuleResult,
};

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── resolve_pointer (public API) ──────────────────────────────────
    #[test]
    fn resolve_pointer_simple_path() {
        let doc = json!({"a": {"b": 42}});
        assert_eq!(resolve_pointer(&doc, "/a/b"), Some(&json!(42)));
    }

    #[test]
    fn resolve_pointer_missing_path() {
        let doc = json!({"a": 1});
        assert_eq!(resolve_pointer(&doc, "/b"), None);
    }

    #[test]
    fn resolve_pointer_empty_is_whole_doc() {
        let doc = json!({"x": 1});
        assert_eq!(resolve_pointer(&doc, ""), Some(&doc));
    }

    // ── evaluate_policy (public API) ──────────────────────────────────
    #[test]
    fn evaluate_policy_all_pass() {
        let receipt = json!({"tokens": 100, "files": 5});
        let policy = PolicyConfig {
            rules: vec![PolicyRule {
                name: "max_tokens".into(),
                pointer: "/tokens".into(),
                op: RuleOperator::Lte,
                value: Some(json!(1000)),
                values: None,
                negate: false,
                level: RuleLevel::Error,
                message: None,
            }],
            fail_fast: false,
            allow_missing: false,
        };

        let result = evaluate_policy(&receipt, &policy);
        assert!(result.passed);
        assert_eq!(result.errors, 0);
        assert_eq!(result.warnings, 0);
    }

    #[test]
    fn evaluate_policy_with_failure() {
        let receipt = json!({"tokens": 2000});
        let policy = PolicyConfig {
            rules: vec![PolicyRule {
                name: "max_tokens".into(),
                pointer: "/tokens".into(),
                op: RuleOperator::Lte,
                value: Some(json!(1000)),
                values: None,
                negate: false,
                level: RuleLevel::Error,
                message: Some("Too many tokens".into()),
            }],
            fail_fast: false,
            allow_missing: false,
        };

        let result = evaluate_policy(&receipt, &policy);
        assert!(!result.passed);
        assert_eq!(result.errors, 1);
    }

    #[test]
    fn evaluate_policy_warn_does_not_fail() {
        let receipt = json!({"tokens": 2000});
        let policy = PolicyConfig {
            rules: vec![PolicyRule {
                name: "token_warning".into(),
                pointer: "/tokens".into(),
                op: RuleOperator::Lte,
                value: Some(json!(1000)),
                values: None,
                negate: false,
                level: RuleLevel::Warn,
                message: None,
            }],
            fail_fast: false,
            allow_missing: false,
        };

        let result = evaluate_policy(&receipt, &policy);
        assert!(result.passed); // Warnings don't fail
        assert_eq!(result.warnings, 1);
    }

    // ── evaluate_ratchet_policy (public API) ──────────────────────────
    #[test]
    fn ratchet_policy_pass() {
        let baseline = json!({"complexity": 10.0});
        let current = json!({"complexity": 10.5}); // 5% increase
        let config = RatchetConfig {
            rules: vec![RatchetRule {
                pointer: "/complexity".into(),
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
        assert!(result.passed);
        assert_eq!(result.errors, 0);
    }

    #[test]
    fn ratchet_policy_fail_regression() {
        let baseline = json!({"complexity": 10.0});
        let current = json!({"complexity": 15.0}); // 50% increase
        let config = RatchetConfig {
            rules: vec![RatchetRule {
                pointer: "/complexity".into(),
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
        assert!(!result.passed);
        assert_eq!(result.errors, 1);
    }

    // ── PolicyConfig parsing ──────────────────────────────────────────
    #[test]
    fn policy_config_from_toml() {
        let toml = r#"
fail_fast = false
allow_missing = true

[[rules]]
name = "check_tokens"
pointer = "/tokens"
op = "lte"
value = 500000
"#;
        let policy = PolicyConfig::from_toml(toml).unwrap();
        assert!(!policy.fail_fast);
        assert!(policy.allow_missing);
        assert_eq!(policy.rules.len(), 1);
        assert_eq!(policy.rules[0].name, "check_tokens");
    }

    #[test]
    fn policy_config_default_is_empty() {
        let policy = PolicyConfig::default();
        assert!(policy.rules.is_empty());
        assert!(!policy.fail_fast);
        assert!(!policy.allow_missing);
    }

    #[test]
    fn ratchet_config_from_toml() {
        let toml = r#"
fail_fast = true
allow_missing_baseline = true

[[rules]]
pointer = "/complexity/avg"
max_increase_pct = 5.0
level = "error"
"#;
        let config = RatchetConfig::from_toml(toml).unwrap();
        assert!(config.fail_fast);
        assert!(config.allow_missing_baseline);
        assert_eq!(config.rules.len(), 1);
    }

    // ── GateResult construction ───────────────────────────────────────
    #[test]
    fn gate_result_from_empty_results() {
        let result = GateResult::from_results(vec![]);
        assert!(result.passed);
        assert_eq!(result.errors, 0);
        assert_eq!(result.warnings, 0);
    }

    #[test]
    fn ratchet_gate_result_from_empty_results() {
        let result = RatchetGateResult::from_results(vec![]);
        assert!(result.passed);
        assert_eq!(result.errors, 0);
        assert_eq!(result.warnings, 0);
    }

    // ── RuleOperator Display ──────────────────────────────────────────
    #[test]
    fn rule_operator_display() {
        assert_eq!(RuleOperator::Gt.to_string(), ">");
        assert_eq!(RuleOperator::Lte.to_string(), "<=");
        assert_eq!(RuleOperator::Eq.to_string(), "==");
        assert_eq!(RuleOperator::In.to_string(), "in");
        assert_eq!(RuleOperator::Exists.to_string(), "exists");
    }

    // ── RuleOperator/RuleLevel defaults ───────────────────────────────
    #[test]
    fn rule_operator_default_is_eq() {
        assert_eq!(RuleOperator::default(), RuleOperator::Eq);
    }

    #[test]
    fn rule_level_default_is_error() {
        assert_eq!(RuleLevel::default(), RuleLevel::Error);
    }
}
