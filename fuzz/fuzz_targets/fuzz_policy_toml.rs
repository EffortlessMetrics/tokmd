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
            // Exercise the next layer: evaluate against a minimal receipt
            if let Ok(receipt) = serde_json::from_str::<Value>(MINIMAL_RECEIPT) {
                let _ = evaluate_policy(&receipt, &policy);
            }
        }
    }
});
