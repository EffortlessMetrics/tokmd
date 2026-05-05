//! Deterministic property tests extracted from `fuzz_run_json`.
//!
//! Validates invariants for:
//! - Run json FFI entrypoint never panics

use proptest::prelude::*;
use serde_json::Value;
use tokmd_core::ffi::run_json;

proptest! {
    #[test]
    fn run_json_invariants(
        mode in "\\PC*",
        args_json in "\\PC*"
    ) {
        let result = run_json(&mode, &args_json);

        // Invariant: result is always valid JSON
        let envelope: Value =
            serde_json::from_str(&result).expect("run_json must always return valid JSON");

        // Invariant: envelope always contains an "ok" boolean field
        prop_assert!(
            envelope.get("ok").and_then(Value::as_bool).is_some(),
            "envelope must have boolean 'ok' field, got: {}",
            result
        );
    }
}
