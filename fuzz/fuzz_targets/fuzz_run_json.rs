//! Fuzz target for the FFI `run_json` entrypoint.
//!
//! Feeds arbitrary mode strings and JSON argument payloads into the
//! single-entrypoint FFI function to verify it never panics and always
//! returns a well-formed JSON envelope.

#![no_main]
use libfuzzer_sys::fuzz_target;
use serde_json::Value;
use tokmd_core::ffi::run_json;

/// Cap input size to keep iterations fast.
const MAX_INPUT_SIZE: usize = 64 * 1024; // 64 KB

/// High-value mode names to exercise successful dispatch and feature-gated paths.
const KNOWN_MODES: &[&str] = &[
    "lang", "module", "export", "analyze", "cockpit", "diff", "version",
];

fn assert_envelope_shape(mode: &str, args_json: &str) {
    let result = run_json(mode, args_json);

    let envelope: Value =
        serde_json::from_str(&result).expect("run_json must always return valid JSON");

    assert!(
        envelope.get("ok").and_then(Value::as_bool).is_some(),
        "envelope must have boolean 'ok' field, got: {}",
        result
    );
}

fuzz_target!(|data: &[u8]| {
    if data.is_empty() || data.len() > MAX_INPUT_SIZE {
        return;
    }
    let Ok(input) = std::str::from_utf8(data) else {
        return;
    };

    // Split on first newline: mode\nargs_json
    let (mode, args_json) = match input.find('\n') {
        Some(pos) => (&input[..pos], &input[pos + 1..]),
        None => (input, "{}"),
    };

    // Explore arbitrary user inputs.
    assert_envelope_shape(mode, args_json);

    // Replay the same argument payload through canonical modes to improve
    // branch coverage of mode dispatch and per-mode argument decoding.
    for known_mode in KNOWN_MODES {
        assert_envelope_shape(known_mode, args_json);
    }

    // Exercise unknown-mode behavior with a stable sentinel.
    assert_envelope_shape("__unknown_mode__", args_json);
});
