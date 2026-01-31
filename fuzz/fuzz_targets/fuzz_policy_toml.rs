//! Fuzz target for policy TOML parsing.
//!
//! Tests `PolicyConfig::from_toml()` with arbitrary TOML input to find
//! panics, hangs, or memory issues in policy rule deserialization.

#![no_main]
use libfuzzer_sys::fuzz_target;
use tokmd_gate::PolicyConfig;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Try parsing as policy TOML - we only care about panics/hangs
        let _ = PolicyConfig::from_toml(s);
    }
});
