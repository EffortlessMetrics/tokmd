//! Fuzz target for TOML configuration parsing.
//!
//! Tests `TomlConfig::parse()` with arbitrary TOML input to find
//! panics, hangs, or excessive memory usage in the TOML deserializer.

#![no_main]
use libfuzzer_sys::fuzz_target;
use tokmd_config::TomlConfig;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Try parsing as TOML config - we only care about panics/hangs,
        // not whether it successfully parses
        let _ = TomlConfig::parse(s);
    }
});
