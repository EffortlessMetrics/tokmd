//! Fuzz target for policy TOML parsing.
//!
//! Tests `PolicyConfig::from_toml()` with arbitrary TOML input to find
//! panics, hangs, or memory issues in policy rule deserialization.

#![no_main]
use libfuzzer_sys::fuzz_target;
use serde_json::Value;
use tokmd_gate::{PolicyConfig, evaluate_policy};

/// Max input size to prevent pathological parse times
const MAX_INPUT_SIZE: usize = 64 * 1024; // 64KB

/// Minimal receipt for exercising policy evaluation after successful parse
const MINIMAL_RECEIPT: &str =
    r#"{"derived":{"totals":{"code":100,"comments":10,"blanks":5,"tokens":500}}}"#;

fuzz_target!(|data: &[u8]| {
    if data.len() > MAX_INPUT_SIZE {
        return;
    }
    if let Ok(s) = std::str::from_utf8(data) {
        // Try parsing as policy TOML
        if let Ok(policy) = PolicyConfig::from_toml(s) {
            // Verify parsed policy has expected structure
            let rule_count = policy.rules.len();

            // Verify all rules have expected structure
            for (i, rule) in policy.rules.iter().enumerate() {
                // Rule name must be a valid string (can be accessed without panic)
                let _ = rule.name.len();

                // Verify pointer follows JSON Pointer RFC 6901:
                // - Empty string "" is the root pointer (valid)
                // - Non-empty pointers must start with "/"
                if !rule.pointer.is_empty() {
                    assert!(
                        rule.pointer.starts_with('/'),
                        "Policy rule[{i}] '{}' non-empty pointer must start with '/': got {:?}",
                        rule.name,
                        rule.pointer
                    );
                }

                // Verify operator can be displayed (exercises Display impl)
                let _ = rule.op.to_string();
            }

            // Exercise the next layer: evaluate against a minimal receipt
            if let Ok(receipt) = serde_json::from_str::<Value>(MINIMAL_RECEIPT) {
                let result = evaluate_policy(&receipt, &policy);

                // Verify result structure
                // The number of rule results should match the number of rules
                assert_eq!(
                    result.rule_results.len(),
                    rule_count,
                    "evaluate_policy should produce one result per rule"
                );

                // Verify errors + warnings consistency
                let counted_errors = result
                    .rule_results
                    .iter()
                    .filter(|r| !r.passed && r.level == tokmd_gate::RuleLevel::Error)
                    .count();
                let counted_warnings = result
                    .rule_results
                    .iter()
                    .filter(|r| !r.passed && r.level == tokmd_gate::RuleLevel::Warn)
                    .count();

                assert_eq!(
                    result.errors, counted_errors,
                    "GateResult.errors should match counted errors"
                );
                assert_eq!(
                    result.warnings, counted_warnings,
                    "GateResult.warnings should match counted warnings"
                );

                // passed should be true iff errors == 0
                assert_eq!(
                    result.passed,
                    result.errors == 0,
                    "GateResult.passed should be true iff errors == 0"
                );
            }
        }
    }
});
