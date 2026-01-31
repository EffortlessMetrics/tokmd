//! Fuzz target for policy evaluation logic.
//!
//! Tests `evaluate_policy()` with arbitrary JSON receipts and policy rules
//! to find panics or unexpected behavior in rule evaluation.
//!
//! Corpus format: `receipt_json\npolicy_toml`
//! The input is split on the first newline to separate receipt from policy.

#![no_main]
use libfuzzer_sys::fuzz_target;
use serde_json::Value;
use tokmd_gate::{PolicyConfig, evaluate_policy};

/// Max input sizes to prevent pathological parse times
const MAX_RECEIPT_SIZE: usize = 64 * 1024; // 64KB for JSON receipt
const MAX_POLICY_SIZE: usize = 16 * 1024; // 16KB for TOML policy

fuzz_target!(|data: &[u8]| {
    // Split on newline: receipt_json\npolicy_toml
    let Some(pos) = data.iter().position(|&b| b == b'\n') else {
        return;
    };
    let (receipt_bytes, policy_bytes) = data.split_at(pos);
    let policy_bytes = &policy_bytes[1..]; // skip the newline

    if receipt_bytes.len() > MAX_RECEIPT_SIZE || policy_bytes.len() > MAX_POLICY_SIZE {
        return;
    }

    let Ok(receipt_str) = std::str::from_utf8(receipt_bytes) else {
        return;
    };
    let Ok(policy_str) = std::str::from_utf8(policy_bytes) else {
        return;
    };
    let Ok(receipt) = serde_json::from_str::<Value>(receipt_str) else {
        return;
    };
    let Ok(policy) = PolicyConfig::from_toml(policy_str) else {
        return;
    };

    // Evaluate - should never panic
    let result = evaluate_policy(&receipt, &policy);

    // Validate result invariants
    let rule_count = policy.rules.len();
    assert_eq!(result.rule_results.len(), rule_count);
    assert!(result.errors + result.warnings <= rule_count);
});
