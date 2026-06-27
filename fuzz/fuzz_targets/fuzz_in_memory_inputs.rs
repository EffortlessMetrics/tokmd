#![no_main]

use libfuzzer_sys::fuzz_target;
use serde_json::Value;

// We need to access `parse_in_memory_inputs` but it's private in `ffi::inputs`.
// We can just construct JSON objects and pass them to `run_json` with "lang" mode,
// and that achieves exactly what we want without exposing private functions.
use tokmd_core::ffi::run_json;

const MAX_INPUT_SIZE: usize = 64 * 1024;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() || data.len() > MAX_INPUT_SIZE {
        return;
    }

    // Attempt to parse bytes as arbitrary string
    if let Ok(input_str) = std::str::from_utf8(data) {
        // Construct a JSON with in-memory inputs
        let args = serde_json::json!({
            "inputs": [
                {
                    "path": input_str,
                    "text": "fn main() {}"
                }
            ]
        });

        let args_str = args.to_string();

        // This will route through parse_in_memory_inputs and validate_in_memory_input_path
        let result = run_json("lang", &args_str);

        // Ensure we always get a valid JSON envelope back
        let parsed: Result<Value, _> = serde_json::from_str(&result);
        assert!(
            parsed.is_ok(),
            "run_json must return valid JSON, got: {}",
            result
        );
    }
});
