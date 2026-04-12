Option A (recommended)
Fix FFI JSON bounds to prevent test failures on scalar non-object payloads.
By modifying `tokmd_core::ffi::run_json_inner` to error if the parsed JSON is not a JSON Object.
This resolves property test panics on unexpected types during the transition from string inputs to value evaluation and maintains full feature parity across environments for binding contracts.

Option B
Relax the property tests in `crates/tokmd-python/tests/property_tests.rs`.
However, the JSON bounds error also happens in production. We should fail gracefully on unexpected JSON.

Decision
Proceed with Option A.
