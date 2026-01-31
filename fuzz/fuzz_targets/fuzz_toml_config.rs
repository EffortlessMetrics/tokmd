//! Fuzz target for TOML configuration parsing.
//!
//! Tests `TomlConfig::parse()` with arbitrary TOML input to find
//! panics, hangs, or excessive memory usage in the TOML deserializer.
//! After successful parse, exercises serialization round-trip and field access.

#![no_main]
use libfuzzer_sys::fuzz_target;
use tokmd_config::TomlConfig;

/// Max input size to prevent pathological parse times
const MAX_INPUT_SIZE: usize = 64 * 1024; // 64KB

fuzz_target!(|data: &[u8]| {
    if data.len() > MAX_INPUT_SIZE {
        return;
    }
    if let Ok(s) = std::str::from_utf8(data) {
        // Try parsing as TOML config
        if let Ok(config) = TomlConfig::parse(s) {
            // Exercise the next layer: serialization round-trip
            if let Ok(json) = serde_json::to_string(&config) {
                // Round-trip through JSON
                let _ = serde_json::from_str::<TomlConfig>(&json);
            }

            // Access nested fields to exercise structure traversal
            let _ = config.scan.paths.as_ref().map(|p| p.len());
            let _ = config.module.roots.as_ref().map(|r| r.len());
            let _ = config.view.len();
        }
    }
});
