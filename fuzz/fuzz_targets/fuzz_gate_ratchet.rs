//! Fuzz target for ratchet policy evaluation.
//!
//! Tests `evaluate_ratchet_policy()` with arbitrary TOML configs and
//! JSON baseline/current documents to find panics or invariant violations.
//!
//! Corpus format: `baseline_json\ncurrent_json\nratchet_toml`
//! The input is split on the first two newlines.

#![no_main]
use libfuzzer_sys::fuzz_target;
use serde_json::Value;
use tokmd_gate::{RatchetConfig, evaluate_ratchet_policy};

/// Max input sizes to prevent pathological parse times
const MAX_JSON_SIZE: usize = 64 * 1024; // 64 KB per JSON doc
const MAX_TOML_SIZE: usize = 16 * 1024; // 16 KB for TOML config

fuzz_target!(|data: &[u8]| {
    // Split into three sections on newlines: baseline\ncurrent\nconfig
    let Some(first_nl) = data.iter().position(|&b| b == b'\n') else {
        return;
    };
    let (baseline_bytes, rest) = data.split_at(first_nl);
    let rest = &rest[1..]; // skip newline

    let Some(second_nl) = rest.iter().position(|&b| b == b'\n') else {
        return;
    };
    let (current_bytes, config_bytes) = rest.split_at(second_nl);
    let config_bytes = &config_bytes[1..]; // skip newline

    if baseline_bytes.len() > MAX_JSON_SIZE
        || current_bytes.len() > MAX_JSON_SIZE
        || config_bytes.len() > MAX_TOML_SIZE
    {
        return;
    }

    let Ok(baseline_str) = std::str::from_utf8(baseline_bytes) else {
        return;
    };
    let Ok(current_str) = std::str::from_utf8(current_bytes) else {
        return;
    };
    let Ok(config_str) = std::str::from_utf8(config_bytes) else {
        return;
    };

    let Ok(baseline) = serde_json::from_str::<Value>(baseline_str) else {
        return;
    };
    let Ok(current) = serde_json::from_str::<Value>(current_str) else {
        return;
    };
    let Ok(config) = RatchetConfig::from_toml(config_str) else {
        return;
    };

    // Evaluate — must never panic
    let result = evaluate_ratchet_policy(&config, &baseline, &current);

    // Invariant: number of results matches number of rules
    assert_eq!(
        result.ratchet_results.len(),
        config.rules.len(),
        "result count must equal rule count"
    );
});
